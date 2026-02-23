//! Function generation and tail call optimization.
//!
//! Handles function code generation, TCO detection and loop-based
//! tail recursion elimination.

use std::collections::HashMap;

use inkwell::{AddressSpace, IntPredicate};

use vais_ast::{self as ast, Expr, GenericParamKind, IfElse, Stmt};
use vais_types::ResolvedType;

use super::generator::{InkwellCodeGenerator, TcoState};
use crate::{CodegenError, CodegenResult};

impl<'ctx> InkwellCodeGenerator<'ctx> {
    pub(super) fn has_tail_self_call(expr: &Expr) -> bool {
        match expr {
            // Direct self-call: @(args) - this IS a tail call
            Expr::Call { func, .. } if matches!(&func.node, Expr::SelfCall) => true,
            // Ternary: cond ? then : else - check both branches
            Expr::Ternary { then, else_, .. } => {
                Self::has_tail_self_call(&then.node) || Self::has_tail_self_call(&else_.node)
            }
            // If expression: check then and else branches
            Expr::If { then, else_, .. } => {
                // Check last statement of then block
                let then_tail = then.last().is_some_and(|s| {
                    if let Stmt::Expr(e) = &s.node {
                        Self::has_tail_self_call(&e.node)
                    } else {
                        false
                    }
                });
                let else_tail = else_.as_ref().is_some_and(Self::if_else_has_tail);
                then_tail || else_tail
            }
            // Match expression: check arms
            Expr::Match { arms, .. } => arms
                .iter()
                .any(|arm| Self::has_tail_self_call(&arm.body.node)),
            // Block: check last expression
            Expr::Block(stmts) => stmts.last().is_some_and(|s| {
                if let Stmt::Expr(e) = &s.node {
                    Self::has_tail_self_call(&e.node)
                } else {
                    false
                }
            }),
            _ => false,
        }
    }

    pub(super) fn if_else_has_tail(ie: &IfElse) -> bool {
        match ie {
            IfElse::Else(stmts) => stmts.last().is_some_and(|s| {
                if let Stmt::Expr(e) = &s.node {
                    Self::has_tail_self_call(&e.node)
                } else {
                    false
                }
            }),
            IfElse::ElseIf(_, then, else_) => {
                let then_tail = then.last().is_some_and(|s| {
                    if let Stmt::Expr(e) = &s.node {
                        Self::has_tail_self_call(&e.node)
                    } else {
                        false
                    }
                });
                let else_tail = else_.as_ref().is_some_and(|ie| Self::if_else_has_tail(ie));
                then_tail || else_tail
            }
        }
    }

    /// Generate a tail-recursive function body as a loop.
    /// Instead of recursive calls, we update the parameters and branch back to the loop header.
    pub(super) fn generate_tco_function(&mut self, func: &ast::Function) -> CodegenResult<()> {
        // Skip generic function body - they are generated via specialization
        if !func.generics.is_empty() {
            return Ok(());
        }

        let fn_value = *self
            .functions
            .get(&func.name.node)
            .ok_or_else(|| CodegenError::UndefinedFunction(func.name.node.clone()))?;

        self.current_function = Some(fn_value);
        self.locals.clear();
        self.var_struct_types.clear();
        self.var_resolved_types.clear();
        self.defer_stack.clear();

        // Non-generic function: substitutions should be empty, take avoids clone allocation
        let old_substitutions = std::mem::take(&mut self.generic_substitutions);

        // Create entry block
        let entry = self.context.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(entry);

        // Allocate space for parameters (these will be updated on each loop iteration)
        let mut param_allocas = Vec::new();
        for (i, param) in func.params.iter().enumerate() {
            let param_value = fn_value.get_nth_param(i as u32).ok_or_else(|| {
                CodegenError::InternalError(format!(
                    "ICE: parameter index {} out of bounds for function '{}'",
                    i, func.name.node
                ))
            })?;
            let param_type = param_value.get_type();
            let alloca = self
                .builder
                .build_alloca(param_type, &param.name.node)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(alloca, param_value)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.locals
                .insert(param.name.node.clone(), (alloca, param_type));
            param_allocas.push((param.name.node.clone(), alloca, param_type));

            if let Some(struct_name) = self.extract_struct_type_name(&param.ty.node) {
                self.var_struct_types
                    .insert(param.name.node.clone(), struct_name);
            }

            // Track resolved type for parameters (for element/pointee type inference)
            let resolved = self.ast_type_to_resolved(&param.ty.node);
            self.var_resolved_types
                .insert(param.name.node.clone(), resolved);
        }

        // Create loop header block (jump target for tail calls)
        let loop_header = self.context.append_basic_block(fn_value, "tco_loop");
        self.builder
            .build_unconditional_branch(loop_header)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        self.builder.position_at_end(loop_header);

        // Set TCO state so generate_call knows to emit loop-back instead of recursive call
        self.tco_state = Some(TcoState {
            param_allocas: param_allocas.clone(),
            loop_header,
        });

        // Generate function body (TCO path)
        let ret_resolved = if let Some(t) = func.ret_type.as_ref() {
            self.ast_type_to_resolved(&t.node)
        } else if let Some(resolved_sig) = self.resolved_function_sigs.get(&func.name.node) {
            resolved_sig.ret.clone()
        } else {
            ResolvedType::Unit
        };
        let ret_substituted = self.substitute_type(&ret_resolved);

        match &func.body {
            ast::FunctionBody::Expr(body_expr) => {
                let body_value = self.generate_expr(&body_expr.node)?;
                // Only return if we haven't already (tail call branches back)
                if self
                    .builder
                    .get_insert_block()
                    .expect("ICE: no insert block during TCO function generation")
                    .get_terminator()
                    .is_none()
                {
                    self.emit_defer_cleanup()?;
                    if ret_substituted == ResolvedType::Unit {
                        self.builder
                            .build_return(None)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    } else {
                        // Cast return value to match function signature if needed (e.g. i32 -> i64)
                        let expected_ret_type = fn_value.get_type().get_return_type();
                        let ret_val = if let Some(ert) = expected_ret_type {
                            if ert != body_value.get_type()
                                && body_value.is_int_value()
                                && ert.is_int_type()
                            {
                                self.builder
                                    .build_int_cast(
                                        body_value.into_int_value(),
                                        ert.into_int_type(),
                                        "ret_cast",
                                    )
                                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                                    .into()
                            } else {
                                body_value
                            }
                        } else {
                            body_value
                        };
                        self.builder
                            .build_return(Some(&ret_val))
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }
                }
            }
            ast::FunctionBody::Block(stmts) => {
                let body_value = self.generate_block(stmts)?;
                if self
                    .builder
                    .get_insert_block()
                    .expect("ICE: no insert block during TCO function generation")
                    .get_terminator()
                    .is_none()
                {
                    self.emit_defer_cleanup()?;
                    if ret_substituted == ResolvedType::Unit {
                        self.builder
                            .build_return(None)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    } else {
                        let expected_ret_type = fn_value.get_type().get_return_type();
                        let body_type_matches =
                            expected_ret_type.is_some_and(|ert| ert == body_value.get_type());
                        if body_type_matches {
                            self.builder
                                .build_return(Some(&body_value))
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        } else if let Some(ert) = expected_ret_type {
                            // Auto-return: body type doesn't match expected return type.
                            // For main() with implicit i64 return, the body evaluates to
                            // Unit but the function signature expects i64 — return 0.
                            let default_val = self.get_default_value(ert);
                            self.builder
                                .build_return(Some(&default_val))
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        } else {
                            self.builder
                                .build_return(None)
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        }
                    }
                }
            }
        }

        // Clear TCO state
        self.tco_state = None;
        self.generic_substitutions = old_substitutions;
        self.type_mapper
            .set_generic_substitutions(&self.generic_substitutions);
        self.current_function = None;
        Ok(())
    }

    pub(super) fn generate_function(&mut self, func: &ast::Function) -> CodegenResult<()> {
        // Skip generic function body - they are generated via specialization
        if !func.generics.is_empty() {
            return Ok(());
        }

        // Check if this function has tail-recursive self-calls
        let is_tail_recursive = match &func.body {
            ast::FunctionBody::Expr(body_expr) => Self::has_tail_self_call(&body_expr.node),
            ast::FunctionBody::Block(stmts) => stmts.last().is_some_and(|s| {
                if let Stmt::Expr(e) = &s.node {
                    Self::has_tail_self_call(&e.node)
                } else {
                    false
                }
            }),
        };

        if is_tail_recursive {
            return self.generate_tco_function(func);
        }

        let fn_value = *self
            .functions
            .get(&func.name.node)
            .ok_or_else(|| CodegenError::UndefinedFunction(func.name.node.clone()))?;

        self.current_function = Some(fn_value);
        self.locals.clear();
        self.var_struct_types.clear();
        self.var_resolved_types.clear();
        self.defer_stack.clear();

        // Non-generic function: substitutions should be empty, take avoids clone allocation
        let old_substitutions = std::mem::take(&mut self.generic_substitutions);

        // Create entry block
        let entry = self.context.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(entry);

        // Allocate space for parameters
        for (i, param) in func.params.iter().enumerate() {
            let param_value = fn_value.get_nth_param(i as u32).ok_or_else(|| {
                CodegenError::InternalError(format!(
                    "ICE: parameter index {} out of bounds for function '{}'",
                    i, func.name.node
                ))
            })?;
            // Use the actual LLVM parameter type from the declared function
            let param_type = param_value.get_type();
            let alloca = self
                .builder
                .build_alloca(param_type, &param.name.node)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(alloca, param_value)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.locals
                .insert(param.name.node.clone(), (alloca, param_type));

            // Track struct type for parameters
            if let Some(struct_name) = self.extract_struct_type_name(&param.ty.node) {
                self.var_struct_types
                    .insert(param.name.node.clone(), struct_name);
            }

            // Track resolved type for parameters (for element/pointee type inference)
            let resolved = self.ast_type_to_resolved(&param.ty.node);
            self.var_resolved_types
                .insert(param.name.node.clone(), resolved);
        }

        // Generate contract checks (#[requires] attributes)
        for (idx, attr) in func.attributes.iter().enumerate() {
            if attr.name == "requires" {
                if let Some(expr) = &attr.expr {
                    let cond_val = self.generate_expr(&expr.node)?;
                    // Convert condition to i1 (bool)
                    let cond_i1 = if cond_val.is_int_value() {
                        self.builder
                            .build_int_compare(
                                IntPredicate::NE,
                                cond_val.into_int_value(),
                                cond_val.get_type().into_int_type().const_zero(),
                                "contract_cond",
                            )
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    } else {
                        // Non-int condition: treat as truthy
                        self.context.bool_type().const_int(1, false)
                    };

                    let ok_block = self
                        .context
                        .append_basic_block(fn_value, &format!("contract_ok_{}", idx));
                    let fail_block = self
                        .context
                        .append_basic_block(fn_value, &format!("contract_fail_{}", idx));

                    self.builder
                        .build_conditional_branch(cond_i1, ok_block, fail_block)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Fail block: write message to stderr and exit
                    self.builder.position_at_end(fail_block);
                    let msg = format!("requires condition #{}", idx);
                    let msg_val = self.generate_string_literal(&msg)?;
                    // Write to stderr (fd=2) using write()
                    let write_fn = self.module.get_function("write").unwrap_or_else(|| {
                        self.module.add_function(
                            "write",
                            self.context.i64_type().fn_type(
                                &[
                                    self.context.i32_type().into(),
                                    self.context
                                        .i8_type()
                                        .ptr_type(AddressSpace::default())
                                        .into(),
                                    self.context.i64_type().into(),
                                ],
                                false,
                            ),
                            None,
                        )
                    });
                    let msg_len = self.context.i64_type().const_int(msg.len() as u64, false);
                    self.builder
                        .build_call(
                            write_fn,
                            &[
                                self.context.i32_type().const_int(2, false).into(),
                                msg_val.into(),
                                msg_len.into(),
                            ],
                            "contract_write",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    // Write newline
                    let newline = self.generate_string_literal("\n")?;
                    self.builder
                        .build_call(
                            write_fn,
                            &[
                                self.context.i32_type().const_int(2, false).into(),
                                newline.into(),
                                self.context.i64_type().const_int(1, false).into(),
                            ],
                            "contract_nl",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    let exit_fn = self.module.get_function("exit").unwrap_or_else(|| {
                        self.module.add_function(
                            "exit",
                            self.context
                                .void_type()
                                .fn_type(&[self.context.i32_type().into()], false),
                            None,
                        )
                    });
                    self.builder
                        .build_call(
                            exit_fn,
                            &[self.context.i32_type().const_int(1, false).into()],
                            "",
                        )
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    self.builder
                        .build_unreachable()
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

                    // Continue in OK block
                    self.builder.position_at_end(ok_block);
                }
            }
        }

        // Generate function body
        let ret_resolved = if let Some(t) = func.ret_type.as_ref() {
            self.ast_type_to_resolved(&t.node)
        } else if let Some(resolved_sig) = self.resolved_function_sigs.get(&func.name.node) {
            resolved_sig.ret.clone()
        } else {
            ResolvedType::Unit
        };
        let ret_substituted = self.substitute_type(&ret_resolved);

        match &func.body {
            ast::FunctionBody::Expr(body_expr) => {
                let body_value = self.generate_expr(&body_expr.node)?;
                // Only build return if the current block doesn't already have a terminator
                // (e.g. an explicit `R` / early-return inside the expr body may have emitted one)
                if self
                    .builder
                    .get_insert_block()
                    .expect("ICE: no insert block during expr-body function generation")
                    .get_terminator()
                    .is_none()
                {
                    self.emit_defer_cleanup()?;
                    if ret_substituted == ResolvedType::Unit {
                        self.builder
                            .build_return(None)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    } else {
                        // Cast return value to match function signature if needed (e.g. i32 -> i64)
                        let expected_ret_type = fn_value.get_type().get_return_type();
                        let ret_val = if let Some(ert) = expected_ret_type {
                            if ert != body_value.get_type()
                                && body_value.is_int_value()
                                && ert.is_int_type()
                            {
                                self.builder
                                    .build_int_cast(
                                        body_value.into_int_value(),
                                        ert.into_int_type(),
                                        "ret_cast",
                                    )
                                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                                    .into()
                            } else {
                                body_value
                            }
                        } else {
                            body_value
                        };
                        self.builder
                            .build_return(Some(&ret_val))
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }
                }
            }
            ast::FunctionBody::Block(stmts) => {
                let body_value = self.generate_block(stmts)?;
                // Only add return if the block doesn't already have a terminator
                if self
                    .builder
                    .get_insert_block()
                    .expect("ICE: no insert block during function generation")
                    .get_terminator()
                    .is_none()
                {
                    self.emit_defer_cleanup()?;
                    if ret_substituted == ResolvedType::Unit {
                        self.builder
                            .build_return(None)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    } else {
                        // Check if body value type matches expected return type
                        let expected_ret_type = fn_value.get_type().get_return_type();
                        let body_type_matches =
                            expected_ret_type.is_some_and(|ert| ert == body_value.get_type());
                        if body_type_matches {
                            self.builder
                                .build_return(Some(&body_value))
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        } else if let Some(ert) = expected_ret_type {
                            // Type mismatch: return a default value of the expected type
                            let default_val = self.get_default_value(ert);
                            self.builder
                                .build_return(Some(&default_val))
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        } else {
                            self.builder
                                .build_return(None)
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        }
                    }
                }
            }
        }

        // Restore generic substitutions
        self.generic_substitutions = old_substitutions;
        self.type_mapper
            .set_generic_substitutions(&self.generic_substitutions);
        self.current_function = None;
        Ok(())
    }

    /// Generate a specialized (monomorphized) function body for a generic function.
    ///
    /// This is called during the second pass of `generate_module_with_instantiations`
    /// after `declare_specialized_function` has registered the LLVM function value.
    pub(super) fn generate_specialized_function_body(
        &mut self,
        func: &ast::Function,
        mangled_name: &str,
        type_args: &[ResolvedType],
    ) -> CodegenResult<()> {
        // Build substitution map from actual generic param names to concrete type args
        let type_params: Vec<_> = func
            .generics
            .iter()
            .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
            .collect();

        let substitutions: HashMap<String, ResolvedType> = type_params
            .iter()
            .zip(type_args.iter())
            .map(|(g, t)| (g.name.node.clone(), t.clone()))
            .collect();

        // Save and replace generic substitutions (replace avoids clone allocation)
        let old_substitutions = std::mem::replace(&mut self.generic_substitutions, substitutions);
        self.type_mapper
            .set_generic_substitutions(&self.generic_substitutions);

        // Look up the already-declared specialized function value
        let fn_value = *self.functions.get(mangled_name).ok_or_else(|| {
            CodegenError::UndefinedFunction(format!(
                "{} (specialized as {})",
                func.name.node, mangled_name
            ))
        })?;

        self.current_function = Some(fn_value);
        self.locals.clear();
        self.var_struct_types.clear();
        self.var_resolved_types.clear();
        self.defer_stack.clear();

        // Create entry block
        let entry = self.context.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(entry);

        // Allocate space for parameters (using the declared LLVM types)
        for (i, param) in func.params.iter().enumerate() {
            let param_value = fn_value.get_nth_param(i as u32).ok_or_else(|| {
                CodegenError::InternalError(format!(
                    "ICE: parameter index {} out of bounds for specialized function '{}'",
                    i, mangled_name
                ))
            })?;
            let param_type = param_value.get_type();
            let alloca = self
                .builder
                .build_alloca(param_type, &param.name.node)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.builder
                .build_store(alloca, param_value)
                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            self.locals
                .insert(param.name.node.clone(), (alloca, param_type));

            // Track struct type for parameters using substituted types.
            // In specialized generic functions, param.ty.node is the AST type (e.g., `T`)
            // which resolves to the generic param name, not the concrete struct name.
            // Use the substituted resolved type to get the actual struct name.
            let resolved_ty = self.ast_type_to_resolved(&param.ty.node);
            let substituted_ty = self.substitute_type(&resolved_ty);
            if let ResolvedType::Named { name, .. } = &substituted_ty {
                if self.generated_structs.contains_key(name.as_str()) {
                    self.var_struct_types
                        .insert(param.name.node.clone(), name.clone());
                }
            } else if let Some(struct_name) = self.extract_struct_type_name(&param.ty.node) {
                // Fallback for non-generic params: use the AST type directly
                self.var_struct_types
                    .insert(param.name.node.clone(), struct_name);
            }

            // Track resolved type for parameters (for element/pointee type inference)
            self.var_resolved_types
                .insert(param.name.node.clone(), substituted_ty);
        }

        // Determine the return type (substituted)
        let ret_resolved = if let Some(t) = func.ret_type.as_ref() {
            self.ast_type_to_resolved(&t.node)
        } else if let Some(resolved_sig) = self.resolved_function_sigs.get(&func.name.node) {
            resolved_sig.ret.clone()
        } else {
            ResolvedType::Unit
        };
        let ret_substituted = self.substitute_type(&ret_resolved);

        // Generate function body
        match &func.body {
            ast::FunctionBody::Expr(body_expr) => {
                let body_value = self.generate_expr(&body_expr.node)?;
                // Only build return if the current block doesn't already have a terminator
                if self
                    .builder
                    .get_insert_block()
                    .expect("ICE: no insert block during specialized expr-body function generation")
                    .get_terminator()
                    .is_none()
                {
                    self.emit_defer_cleanup()?;
                    if ret_substituted == ResolvedType::Unit {
                        self.builder
                            .build_return(None)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    } else {
                        let expected_ret_type = fn_value.get_type().get_return_type();
                        let ret_val = if let Some(ert) = expected_ret_type {
                            if ert != body_value.get_type()
                                && body_value.is_int_value()
                                && ert.is_int_type()
                            {
                                self.builder
                                    .build_int_cast(
                                        body_value.into_int_value(),
                                        ert.into_int_type(),
                                        "ret_cast",
                                    )
                                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?
                                    .into()
                            } else {
                                body_value
                            }
                        } else {
                            body_value
                        };
                        self.builder
                            .build_return(Some(&ret_val))
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    }
                }
            }
            ast::FunctionBody::Block(stmts) => {
                let body_value = self.generate_block(stmts)?;
                // Only add return if the block doesn't already have a terminator
                if self
                    .builder
                    .get_insert_block()
                    .expect("ICE: no insert block during specialized function generation")
                    .get_terminator()
                    .is_none()
                {
                    self.emit_defer_cleanup()?;
                    if ret_substituted == ResolvedType::Unit {
                        self.builder
                            .build_return(None)
                            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    } else {
                        let expected_ret_type = fn_value.get_type().get_return_type();
                        let body_type_matches =
                            expected_ret_type.is_some_and(|ert| ert == body_value.get_type());
                        if body_type_matches {
                            self.builder
                                .build_return(Some(&body_value))
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        } else if let Some(ert) = expected_ret_type {
                            let default_val = self.get_default_value(ert);
                            self.builder
                                .build_return(Some(&default_val))
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        } else {
                            self.builder
                                .build_return(None)
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                        }
                    }
                }
            }
        }

        // Restore generic substitutions
        self.generic_substitutions = old_substitutions;
        self.type_mapper
            .set_generic_substitutions(&self.generic_substitutions);
        self.current_function = None;
        Ok(())
    }
}

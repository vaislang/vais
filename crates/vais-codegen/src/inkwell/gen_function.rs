//! Function generation and tail call optimization.
//!
//! Handles function code generation, TCO detection and loop-based
//! tail recursion elimination.

use inkwell::{AddressSpace, IntPredicate};

use vais_ast::{self as ast, Expr, IfElse, Stmt};
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
        let fn_value = *self
            .functions
            .get(&func.name.node)
            .ok_or_else(|| CodegenError::UndefinedFunction(func.name.node.clone()))?;

        self.current_function = Some(fn_value);
        self.locals.clear();
        self.var_struct_types.clear();
        self.defer_stack.clear();

        let old_substitutions = self.generic_substitutions.clone();
        if !func.generics.is_empty() {
            for gp in &func.generics {
                self.generic_substitutions
                    .entry(gp.name.node.clone())
                    .or_insert(ResolvedType::I64);
            }
        }

        // Create entry block
        let entry = self.context.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(entry);

        // Allocate space for parameters (these will be updated on each loop iteration)
        let mut param_allocas = Vec::new();
        for (i, param) in func.params.iter().enumerate() {
            let param_value = fn_value.get_nth_param(i as u32).unwrap();
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
                    .unwrap()
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
                    .unwrap()
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

        // Clear TCO state
        self.tco_state = None;
        self.generic_substitutions = old_substitutions;
        self.current_function = None;
        Ok(())
    }

    pub(super) fn generate_function(&mut self, func: &ast::Function) -> CodegenResult<()> {
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
        self.defer_stack.clear();

        // For generic functions, set up default substitutions (T -> i64, etc.)
        let old_substitutions = self.generic_substitutions.clone();
        if !func.generics.is_empty() {
            for gp in &func.generics {
                self.generic_substitutions
                    .entry(gp.name.node.clone())
                    .or_insert(ResolvedType::I64);
            }
        }

        // Create entry block
        let entry = self.context.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(entry);

        // Allocate space for parameters
        for (i, param) in func.params.iter().enumerate() {
            let param_value = fn_value.get_nth_param(i as u32).unwrap();
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
            ast::FunctionBody::Block(stmts) => {
                let body_value = self.generate_block(stmts)?;
                // Only add return if the block doesn't already have a terminator
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
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
        self.current_function = None;
        Ok(())
    }
}

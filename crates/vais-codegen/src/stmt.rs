//! Statement code generation for Vais compiler
//!
//! This module handles generation of LLVM IR for Vais statements (Let, Return, Break, Continue, etc.)

use crate::types::LocalVar;
use crate::{CodeGenerator, CodegenError, CodegenResult};
use vais_ast::{Expr, Pattern, Spanned, Stmt};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Generate LLVM IR for a block of statements
    pub(crate) fn generate_block(
        &mut self,
        stmts: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();
        let mut last_value = "void".to_string();

        for stmt in stmts {
            let (value, stmt_ir) = self.generate_stmt(stmt, counter)?;
            ir.push_str(&stmt_ir);
            last_value = value;
        }

        Ok((last_value, ir))
    }

    /// Generate LLVM IR for a single statement
    pub(crate) fn generate_stmt(
        &mut self,
        stmt: &Spanned<Stmt>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        match &stmt.node {
            Stmt::Let {
                name,
                ty,
                value,
                is_mut,
                ..
            } => {
                // Infer type BEFORE generating code, so we can use function return types
                let inferred_ty = self.infer_expr_type(value);

                // Check if this is a struct literal - handle specially
                // Also detect struct tuple literal: Point(40, 2) where "Point" is a known struct
                let is_struct_lit = matches!(&value.node, Expr::StructLit { .. })
                    || if let Expr::Call { func, .. } = &value.node {
                        if let Expr::Ident(fn_name) = &func.node {
                            let resolved = self.resolve_struct_name(fn_name);
                            self.types.structs.contains_key(&resolved)
                                && !self.types.functions.contains_key(fn_name)
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                // Check if this is an enum variant constructor call (e.g., Some(42))
                let is_enum_variant_call = if let Expr::Call { func, .. } = &value.node {
                    if let Expr::Ident(fn_name) = &func.node {
                        self.get_tuple_variant_info(fn_name).is_some()
                    } else {
                        false
                    }
                } else {
                    false
                };

                // Check if this is a unit enum variant (e.g., None)
                let is_unit_variant = if let Expr::Ident(name) = &value.node {
                    self.is_unit_enum_variant(name)
                } else {
                    false
                };

                let (val, val_ir) = self.generate_expr(value, counter)?;

                let resolved_ty = ty
                    .as_ref()
                    .map(|t| self.ast_type_to_resolved(&t.node))
                    .unwrap_or(inferred_ty.clone()); // Use inferred type if not specified

                // Generate unique LLVM name for this variable (to handle loops)
                let llvm_name = format!("{}.{}", name.node, counter);
                *counter += 1;

                let mut ir = val_ir;
                let llvm_ty = self.type_to_llvm(&resolved_ty);

                // Determine if we can use SSA style (no alloca)
                // Conditions for SSA:
                // 1. Not mutable (is_mut == false)
                // 2. Not a struct literal (needs pointer semantics)
                // 3. Not an enum variant (needs pointer semantics)
                // 4. Not a Named type (struct/enum values need special handling)
                // 5. Simple primitive types (i64, i32, bool, etc.)
                let is_simple_type = matches!(
                    resolved_ty,
                    ResolvedType::I8
                        | ResolvedType::I16
                        | ResolvedType::I32
                        | ResolvedType::I64
                        | ResolvedType::I128
                        | ResolvedType::U8
                        | ResolvedType::U16
                        | ResolvedType::U32
                        | ResolvedType::U64
                        | ResolvedType::U128
                        | ResolvedType::F32
                        | ResolvedType::F64
                        | ResolvedType::Bool
                        | ResolvedType::Str
                        | ResolvedType::Pointer(_)
                );

                let use_ssa = !*is_mut
                    && !is_struct_lit
                    && !is_enum_variant_call
                    && !is_unit_variant
                    && !matches!(resolved_ty, ResolvedType::Named { .. })
                    && is_simple_type;

                if use_ssa {
                    // SSA style: directly alias the value, no alloca needed
                    // The llvm_name will refer to the computed value directly
                    self.fn_ctx.locals.insert(
                        name.node.clone(),
                        LocalVar::ssa(resolved_ty.clone(), val.clone()),
                    );
                    // No additional IR needed - we just register the mapping
                } else {
                    // Traditional alloca style
                    self.fn_ctx.locals.insert(
                        name.node.clone(),
                        LocalVar::alloca(resolved_ty.clone(), llvm_name.clone()),
                    );

                    // For struct literals and enum variant constructors, the value is already an alloca'd pointer
                    // We store the pointer to the struct/enum (i.e., %Point*, %Option*)
                    if is_struct_lit || is_enum_variant_call || is_unit_variant {
                        // The val is already a pointer to the struct/enum (%1, %2, etc)
                        // Allocate space for a pointer and store it
                        ir.push_str(&format!("  %{} = alloca {}*\n", llvm_name, llvm_ty));
                        ir.push_str(&format!(
                            "  store {}* {}, {}** %{}\n",
                            llvm_ty, val, llvm_ty, llvm_name
                        ));
                    } else if matches!(resolved_ty, ResolvedType::Named { .. }) {
                        // For struct values (e.g., from function returns),
                        // alloca struct, store value, then store pointer to it
                        // This keeps all struct variables as pointers for consistency
                        let tmp_ptr = format!("%{}.struct", llvm_name);
                        ir.push_str(&format!("  {} = alloca {}\n", tmp_ptr, llvm_ty));
                        ir.push_str(&format!(
                            "  store {} {}, {}* {}\n",
                            llvm_ty, val, llvm_ty, tmp_ptr
                        ));
                        ir.push_str(&format!("  %{} = alloca {}*\n", llvm_name, llvm_ty));
                        ir.push_str(&format!(
                            "  store {}* {}, {}** %{}\n",
                            llvm_ty, tmp_ptr, llvm_ty, llvm_name
                        ));
                    } else {
                        // Allocate and store
                        ir.push_str(&format!("  %{} = alloca {}\n", llvm_name, llvm_ty));
                        ir.push_str(&format!(
                            "  store {} {}, {}* %{}\n",
                            llvm_ty, val, llvm_ty, llvm_name
                        ));
                    }
                }

                // If this was a lambda with captures, register the closure info
                if let Some(closure_info) = self.lambdas.last_lambda_info.take() {
                    self.lambdas.closures.insert(name.node.clone(), closure_info);
                }
                // If this was a lazy expression, register the thunk info
                if let Some(lazy_info) = self.lambdas.last_lazy_info.take() {
                    self.lambdas.lazy_bindings.insert(name.node.clone(), lazy_info);
                }

                Ok(("void".to_string(), ir))
            }
            Stmt::LetDestructure {
                pattern,
                value,
                is_mut,
                ..
            } => self.generate_let_destructure(pattern, value, *is_mut, counter),
            Stmt::Expr(expr) => self.generate_expr(expr, counter),
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    let (val, mut ir) = self.generate_expr(expr, counter)?;

                    // Get the return type of the current function
                    let (ret_type, ret_resolved) = if let Some(fn_name) = &self.fn_ctx.current_function {
                        self.types.functions
                            .get(fn_name)
                            .map(|info| {
                                (
                                    self.type_to_llvm(&info.signature.ret),
                                    info.signature.ret.clone(),
                                )
                            })
                            .unwrap_or_else(|| ("i64".to_string(), ResolvedType::I64))
                    } else {
                        ("i64".to_string(), ResolvedType::I64)
                    };

                    // For struct-typed local variables, we get a pointer from generate_expr
                    // but we need to return by value, so dereference the pointer
                    let final_val = if let Expr::Ident(name) = &expr.node {
                        if let Some(local) = self.fn_ctx.locals.get(name) {
                            if !local.is_param()
                                && matches!(local.ty, ResolvedType::Named { .. })
                                && matches!(ret_resolved, ResolvedType::Named { .. })
                            {
                                // val is a pointer to the struct, load the actual value
                                let loaded = format!("%ret.{}", counter);
                                *counter += 1;
                                ir.push_str(&format!(
                                    "  {} = load {}, {}* {}\n",
                                    loaded, ret_type, ret_type, val
                                ));
                                loaded
                            } else {
                                val
                            }
                        } else {
                            val
                        }
                    } else {
                        val
                    };

                    // Execute deferred expressions before return (LIFO order)
                    let defer_ir = self.generate_defer_cleanup(counter)?;
                    ir.push_str(&defer_ir);

                    // Emit the ret instruction
                    ir.push_str(&format!("  ret {} {}\n", ret_type, final_val));
                    Ok((final_val, ir))
                } else {
                    // Execute deferred expressions before return (LIFO order)
                    let mut ir = String::new();
                    let defer_ir = self.generate_defer_cleanup(counter)?;
                    ir.push_str(&defer_ir);
                    ir.push_str("  ret void\n");
                    Ok(("void".to_string(), ir))
                }
            }
            Stmt::Break(value) => {
                if let Some(labels) = self.fn_ctx.loop_stack.last() {
                    let break_label = labels.break_label.clone();
                    let mut ir = String::new();
                    if let Some(expr) = value {
                        let (val, expr_ir) = self.generate_expr(expr, counter)?;
                        ir.push_str(&expr_ir);
                        // Store break value if needed (for loop expressions)
                        ir.push_str(&format!("  br label %{}\n", break_label));
                        Ok((val, ir))
                    } else {
                        ir.push_str(&format!("  br label %{}\n", break_label));
                        Ok(("void".to_string(), ir))
                    }
                } else {
                    Err(CodegenError::Unsupported(
                        "break outside of loop".to_string(),
                    ))
                }
            }
            Stmt::Continue => {
                if let Some(labels) = self.fn_ctx.loop_stack.last() {
                    let continue_label = labels.continue_label.clone();
                    let ir = format!("  br label %{}\n", continue_label);
                    Ok(("void".to_string(), ir))
                } else {
                    Err(CodegenError::Unsupported(
                        "continue outside of loop".to_string(),
                    ))
                }
            }

            Stmt::Defer(expr) => {
                // Add the deferred expression to the stack
                // It will be executed when the function exits (in LIFO order)
                self.fn_ctx.defer_stack.push(expr.as_ref().clone());
                // No IR generated here - defer is processed at function exit
                Ok(("void".to_string(), String::new()))
            }

            Stmt::Error { message, .. } => {
                // Error nodes should not reach codegen - they indicate parsing failures
                // that should have been handled before code generation
                Err(CodegenError::Unsupported(format!(
                    "Cannot generate code for parse error: {}",
                    message
                )))
            }
        }
    }

    /// Generate LLVM IR for tuple destructuring: `(a, b) := expr`
    pub(crate) fn generate_let_destructure(
        &mut self,
        pattern: &Spanned<Pattern>,
        value: &Spanned<Expr>,
        _is_mut: bool,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (val, mut ir) = self.generate_expr(value, counter)?;
        let tuple_ty = self.infer_expr_type(value);
        let llvm_tuple_ty = self.type_to_llvm(&tuple_ty);

        self.bind_pattern_from_tuple(pattern, &val, &llvm_tuple_ty, &tuple_ty, counter, &mut ir)?;

        Ok(("void".to_string(), ir))
    }

    /// Recursively bind pattern variables from a tuple value using extractvalue
    fn bind_pattern_from_tuple(
        &mut self,
        pattern: &Spanned<Pattern>,
        val: &str,
        llvm_ty: &str,
        resolved_ty: &ResolvedType,
        counter: &mut usize,
        ir: &mut String,
    ) -> CodegenResult<()> {
        match &pattern.node {
            Pattern::Tuple(patterns) => {
                let elem_types = if let ResolvedType::Tuple(types) = resolved_ty {
                    types
                } else {
                    return Err(CodegenError::Unsupported(
                        "destructuring non-tuple type".to_string(),
                    ));
                };

                for (i, pat) in patterns.iter().enumerate() {
                    let elem_resolved = &elem_types[i];
                    let elem_llvm_ty = self.type_to_llvm(elem_resolved);
                    let extracted = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = extractvalue {} {}, {}\n",
                        extracted, llvm_ty, val, i
                    ));
                    self.bind_pattern_from_tuple(
                        pat,
                        &extracted,
                        &elem_llvm_ty,
                        elem_resolved,
                        counter,
                        ir,
                    )?;
                }
                Ok(())
            }
            Pattern::Ident(name) => {
                // Register as SSA local (immutable binding)
                self.fn_ctx.locals.insert(
                    name.clone(),
                    LocalVar::ssa(resolved_ty.clone(), val.to_string()),
                );
                Ok(())
            }
            Pattern::Wildcard => Ok(()),
            _ => Err(CodegenError::Unsupported(
                "unsupported pattern in destructuring".to_string(),
            )),
        }
    }

    /// Generate IR for all deferred expressions in LIFO order
    /// Called before function exit points (return, end of function)
    pub(crate) fn generate_defer_cleanup(&mut self, counter: &mut usize) -> CodegenResult<String> {
        let mut ir = String::new();

        // Process deferred expressions in reverse order (LIFO)
        let defers: Vec<_> = self.fn_ctx.defer_stack.iter().rev().cloned().collect();
        for defer_expr in defers {
            ir.push_str("  ; defer cleanup\n");
            let (_, defer_ir) = self.generate_expr(&defer_expr, counter)?;
            ir.push_str(&defer_ir);
        }

        Ok(ir)
    }

    /// Clear the defer stack (called when entering a new function)
    pub(crate) fn clear_defer_stack(&mut self) {
        self.fn_ctx.defer_stack.clear();
    }
}

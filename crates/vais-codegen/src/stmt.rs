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
        let mut ir = String::with_capacity(stmts.len() * 64); // ~64 bytes per stmt average
        let mut last_value = String::from("void");

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
        stacker::maybe_grow(32 * 1024 * 1024, 64 * 1024 * 1024, || {
            self.generate_stmt_inner(stmt, counter)
        })
    }

    fn generate_stmt_inner(
        &mut self,
        stmt: &Spanned<Stmt>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Track the current statement span for error diagnostics
        self.last_error_span = Some(stmt.span);

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
                        write_ir!(ir, "  %{} = alloca {}*", llvm_name, llvm_ty);
                        write_ir!(
                            ir,
                            "  store {}* {}, {}** %{}",
                            llvm_ty,
                            val,
                            llvm_ty,
                            llvm_name
                        );
                    } else if matches!(resolved_ty, ResolvedType::Named { .. }) {
                        // For struct values (e.g., from function returns),
                        // alloca struct, store value, then store pointer to it
                        // This keeps all struct variables as pointers for consistency
                        let tmp_ptr = format!("%{}.struct", llvm_name);
                        write_ir!(ir, "  {} = alloca {}", tmp_ptr, llvm_ty);
                        // If the value expression is not a value (e.g., block returning
                        // a struct-typed local), we need to load the struct first
                        let actual_val = if !self.is_expr_value(value) {
                            let loaded = self.next_temp(counter);
                            write_ir!(ir, "  {} = load {}, {}* {}", loaded, llvm_ty, llvm_ty, val);
                            loaded
                        } else {
                            val.clone()
                        };
                        write_ir!(
                            ir,
                            "  store {} {}, {}* {}",
                            llvm_ty,
                            actual_val,
                            llvm_ty,
                            tmp_ptr
                        );
                        write_ir!(ir, "  %{} = alloca {}*", llvm_name, llvm_ty);
                        write_ir!(
                            ir,
                            "  store {}* {}, {}** %{}",
                            llvm_ty,
                            tmp_ptr,
                            llvm_ty,
                            llvm_name
                        );
                    } else {
                        // Allocate and store — coerce value width if mismatched
                        let actual_val_ty = self.llvm_type_of(&val);
                        let coerced_val =
                            self.coerce_int_width(&val, &actual_val_ty, &llvm_ty, counter, &mut ir);
                        write_ir!(ir, "  %{} = alloca {}", llvm_name, llvm_ty);
                        write_ir!(
                            ir,
                            "  store {} {}, {}* %{}",
                            llvm_ty,
                            coerced_val,
                            llvm_ty,
                            llvm_name
                        );
                    }
                }

                // If this was a lambda with captures, register the closure info
                if let Some(closure_info) = self.lambdas.last_lambda_info.take() {
                    self.lambdas
                        .closures
                        .insert(name.node.clone(), closure_info);
                }
                // If this was a lazy expression, register the thunk info
                if let Some(lazy_info) = self.lambdas.last_lazy_info.take() {
                    self.lambdas
                        .lazy_bindings
                        .insert(name.node.clone(), lazy_info);
                }

                // Track future→poll function mapping for variable-based await.
                // When `fut := spawn asyncFn(...)` or `fut := asyncFn(...)`,
                // record the poll function so `fut.await` can resolve it.
                if matches!(inferred_ty, ResolvedType::Future(_) | ResolvedType::I64) {
                    if let Some(poll_fn) = self.resolve_poll_func_name(&value.node) {
                        self.fn_ctx
                            .future_poll_fns
                            .insert(name.node.clone(), poll_fn);
                    }
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
                // Check if we're inside an async poll function — if so, early returns
                // must wrap the value as a poll result {1, value} and return the poll type.
                if let Some(poll_ctx) = self.fn_ctx.async_poll_context.clone() {
                    if let Some(expr) = expr {
                        let (val, mut ir) = self.generate_expr(expr, counter)?;

                        // Execute deferred expressions before return (LIFO order)
                        let defer_ir = self.generate_defer_cleanup(counter)?;
                        ir.push_str(&defer_ir);

                        // Call Drop::drop() for droppable locals (reverse order)
                        let drop_ir = self.generate_drop_cleanup();
                        ir.push_str(&drop_ir);

                        // Free tracked heap allocations before return
                        let alloc_cleanup_ir = self.generate_alloc_cleanup();
                        ir.push_str(&alloc_cleanup_ir);

                        // Codegen promotes bool to i64 (zext), truncate back for i1 return
                        let ret_val = if poll_ctx.ret_llvm == "i1" {
                            let trunc = self.next_temp(counter);
                            write_ir!(ir, "  {} = trunc i64 {} to i1", trunc, val);
                            trunc
                        } else {
                            val.clone()
                        };

                        // Store result in state struct and set state to -1 (completed)
                        let poll_ret_ty = format!("{{ i64, {} }}", poll_ctx.ret_llvm);
                        let t0 = self.next_temp(counter);
                        write_ir!(ir, "  {} = insertvalue {} undef, i64 1, 0", t0, poll_ret_ty);
                        let t1 = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = insertvalue {} {}, {} {}, 1",
                            t1,
                            poll_ret_ty,
                            t0,
                            poll_ctx.ret_llvm,
                            ret_val
                        );
                        // Set state to -1 (completed)
                        ir.push_str("  store i64 -1, i64* %state_field\n");
                        write_ir!(ir, "  ret {} {}", poll_ret_ty, t1);
                        return Ok((val, ir));
                    } else {
                        // Return void in async poll — return {1, undef}
                        let mut ir = String::new();
                        let defer_ir = self.generate_defer_cleanup(counter)?;
                        ir.push_str(&defer_ir);
                        let drop_ir = self.generate_drop_cleanup();
                        ir.push_str(&drop_ir);
                        let alloc_cleanup_ir = self.generate_alloc_cleanup();
                        ir.push_str(&alloc_cleanup_ir);
                        let poll_ret_ty = format!("{{ i64, {} }}", poll_ctx.ret_llvm);
                        let t0 = self.next_temp(counter);
                        write_ir!(ir, "  {} = insertvalue {} undef, i64 1, 0", t0, poll_ret_ty);
                        ir.push_str("  store i64 -1, i64* %state_field\n");
                        write_ir!(ir, "  ret {} {}", poll_ret_ty, t0);
                        return Ok(("void".to_string(), ir));
                    }
                }

                if let Some(expr) = expr {
                    let (val, mut ir) = self.generate_expr(expr, counter)?;

                    // Get the return type of the current function
                    let (ret_type, ret_resolved) =
                        if let Some(fn_name) = &self.fn_ctx.current_function {
                            let result = self.types.functions.get(fn_name).map(|info| {
                                (
                                    self.type_to_llvm(&info.signature.ret),
                                    info.signature.ret.clone(),
                                )
                            });
                            result.unwrap_or_else(|| ("i64".to_string(), ResolvedType::I64))
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
                                write_ir!(
                                    ir,
                                    "  {} = load {}, {}* {}",
                                    loaded,
                                    ret_type,
                                    ret_type,
                                    val
                                );
                                loaded
                            } else {
                                val
                            }
                        } else {
                            val
                        }
                    } else if ret_type.starts_with('%')
                        && !ret_type.ends_with('*')
                        && val.starts_with('%')
                        && !matches!(&expr.node, Expr::Ident(_))
                    {
                        // Non-ident expression returning struct type (e.g., Ok(...), Err(...))
                        // The val is likely a pointer from enum variant constructor — load it
                        let loaded = format!("%ret.{}", counter);
                        *counter += 1;
                        write_ir!(
                            ir,
                            "  {} = load {}, {}* {}",
                            loaded,
                            ret_type,
                            ret_type,
                            val
                        );
                        loaded
                    } else {
                        val
                    };

                    // When the function returns a reference type (e.g., -> &i64) but the
                    // expression produced a bare literal (e.g., 42), we must store the
                    // literal in a global constant so the returned pointer is valid.
                    // Without this, we'd emit `ret i64* 42` which clang rejects.
                    let final_val =
                        if matches!(ret_resolved, ResolvedType::Ref(_) | ResolvedType::RefMut(_))
                            && !final_val.starts_with('%')
                            && !final_val.starts_with('@')
                        {
                            // Get the inner type for the global constant
                            let inner_ty = match &ret_resolved {
                                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                                    self.type_to_llvm(inner)
                                }
                                _ => unreachable!(),
                            };
                            let const_name = format!(".ref.const.{}", self.ref_constant_counter);
                            self.ref_constant_counter += 1;
                            self.ref_constants.push((
                                const_name.clone(),
                                inner_ty,
                                final_val.clone(),
                            ));
                            format!("@{}", const_name)
                        } else {
                            final_val
                        };

                    // Execute deferred expressions before return (LIFO order)
                    let defer_ir = self.generate_defer_cleanup(counter)?;
                    ir.push_str(&defer_ir);

                    // Call Drop::drop() for droppable locals (reverse order)
                    let drop_ir = self.generate_drop_cleanup();
                    ir.push_str(&drop_ir);

                    // Free tracked heap allocations before return
                    let alloc_cleanup_ir = self.generate_alloc_cleanup();
                    ir.push_str(&alloc_cleanup_ir);

                    // Coerce value to match function return type if needed
                    // (e.g., sext i32→i64 in body then ret i32 needs trunc back)
                    let final_val = {
                        let val_ty = self.llvm_type_of(&final_val);
                        if val_ty != ret_type
                            && val_ty.starts_with('i')
                            && ret_type.starts_with('i')
                        {
                            let val_bits: u32 = val_ty[1..].parse().unwrap_or(64);
                            let ret_bits: u32 = ret_type[1..].parse().unwrap_or(64);
                            if val_bits > 0 && ret_bits > 0 && val_bits != ret_bits {
                                let tmp = self.next_temp(counter);
                                if val_bits > ret_bits {
                                    write_ir!(
                                        ir,
                                        "  {} = trunc {} {} to {}",
                                        tmp,
                                        val_ty,
                                        final_val,
                                        ret_type
                                    );
                                } else {
                                    write_ir!(
                                        ir,
                                        "  {} = sext {} {} to {}",
                                        tmp,
                                        val_ty,
                                        final_val,
                                        ret_type
                                    );
                                }
                                tmp
                            } else {
                                final_val
                            }
                        } else if val_ty != ret_type
                            && (val_ty == "float" || val_ty == "double")
                            && (ret_type == "float" || ret_type == "double")
                        {
                            // Float width coercion (e.g., double→float fptrunc)
                            let tmp = self.next_temp(counter);
                            if val_ty == "double" && ret_type == "float" {
                                write_ir!(ir, "  {} = fptrunc double {} to float", tmp, final_val);
                            } else {
                                write_ir!(ir, "  {} = fpext float {} to double", tmp, final_val);
                            }
                            tmp
                        } else if val_ty != ret_type
                            && val_ty == "i64"
                            && ret_type.starts_with('%')
                            && !ret_type.ends_with('*')
                        {
                            // Struct return type mismatch — value is i64 but return type is struct.
                            // This happens in specialized generic functions where the body
                            // uses i64 (generic erasure) but the function signature declares
                            // a concrete struct type. Use inttoptr+load to reinterpret.
                            let tmp_ptr = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = inttoptr i64 {} to {}*",
                                tmp_ptr,
                                final_val,
                                ret_type
                            );
                            let loaded = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = load {}, {}* {}",
                                loaded,
                                ret_type,
                                ret_type,
                                tmp_ptr
                            );
                            loaded
                        } else {
                            final_val
                        }
                    };

                    // Emit the ret instruction
                    write_ir!(ir, "  ret {} {}", ret_type, final_val);
                    Ok((final_val, ir))
                } else {
                    // Execute deferred expressions before return (LIFO order)
                    let mut ir = String::new();
                    let defer_ir = self.generate_defer_cleanup(counter)?;
                    ir.push_str(&defer_ir);

                    // Call Drop::drop() for droppable locals (reverse order)
                    let drop_ir = self.generate_drop_cleanup();
                    ir.push_str(&drop_ir);

                    // Free tracked heap allocations before return
                    let alloc_cleanup_ir = self.generate_alloc_cleanup();
                    ir.push_str(&alloc_cleanup_ir);

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
                        write_ir!(ir, "  br label %{}", break_label);
                        Ok((val, ir))
                    } else {
                        write_ir!(ir, "  br label %{}", break_label);
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
                    write_ir!(
                        ir,
                        "  {} = extractvalue {} {}, {}",
                        extracted,
                        llvm_ty,
                        val,
                        i
                    );
                    // Register extracted element's type for downstream type tracking
                    self.fn_ctx
                        .register_temp_type(&extracted, elem_resolved.clone());
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

    /// Push a new scope frame onto the scope stack.
    /// Called at the start of a block (if branch, loop body, explicit block expr, etc.).
    pub(crate) fn enter_scope(&mut self) {
        self.fn_ctx.scope_stack.push(Vec::new());
    }

    /// Pop the current scope frame from the scope stack.
    /// Returns the list of variable names declared in that scope (in declaration order).
    /// The caller is responsible for removing these variables from fn_ctx.locals.
    pub(crate) fn exit_scope(&mut self) -> Vec<String> {
        self.fn_ctx.scope_stack.pop().unwrap_or_default()
    }

    /// Register a variable in the current innermost scope (if any scope is active).
    /// Only tracks Named type locals — primitive SSA values don't need Drop.
    pub(crate) fn track_scope_local(&mut self, name: &str) {
        if let Some(scope) = self.fn_ctx.scope_stack.last_mut() {
            scope.push(name.to_string());
        }
    }

    /// Generate drop cleanup IR for a set of Named-type locals leaving scope.
    /// Emits drop calls in LIFO order (last declared first), then removes the
    /// variables from fn_ctx.locals to prevent double-drop at function exit.
    ///
    /// For struct literal locals, the llvm_name is a `%Type**` (double pointer)
    /// because the codegen pattern is: `alloca %Type` → actual struct, then
    /// `alloca %Type*` → pointer stored in locals. We must load the inner
    /// `%Type*` before passing it to the drop function.
    pub(crate) fn generate_scope_drop_cleanup(&mut self, scope_vars: &[String]) -> String {
        if scope_vars.is_empty() || self.types.drop_registry.is_empty() {
            return String::new();
        }

        // Collect (type_name, llvm_name, drop_fn, is_double_ptr) in declaration order.
        // is_double_ptr=true when the local is a struct literal stored as %Type**.
        let mut droppable: Vec<(String, String, String, bool)> = Vec::new();
        for var_name in scope_vars {
            if let Some(local) = self.fn_ctx.locals.get(var_name) {
                let type_name = match &local.ty {
                    vais_types::ResolvedType::Named { name, .. } => name.clone(),
                    _ => continue,
                };
                if let Some(drop_fn) = self.types.drop_registry.get(&type_name).cloned() {
                    if local.is_alloca() {
                        // Alloca locals for struct literals are double-pointers (%Type**)
                        // because the codegen stores a %Type* in an alloca.
                        // Parameters and SSA values are not owned by this block — skip them.
                        droppable.push((type_name, local.llvm_name.clone(), drop_fn, true));
                    }
                    // Parameters: passed by reference — caller owns them, do not drop.
                    // SSA values: no alloca pointer to pass to drop — skip.
                }
            }
        }

        if droppable.is_empty() {
            return String::new();
        }

        let mut ir = String::new();
        ir.push_str("  ; block-scope drop cleanup\n");
        // Drop in reverse order (last declared first — LIFO)
        for (_type_name, llvm_name, drop_fn, is_double_ptr) in droppable.iter().rev() {
            let type_name_for_fn = drop_fn.strip_suffix("_drop").unwrap_or(drop_fn);
            let struct_ty = format!("%{}", type_name_for_fn);
            // Drop functions return i64 per the Drop trait definition (F drop(&self) -> i64).
            // Use a temp to capture the return value and discard it.
            let id = self.fn_ctx.label_counter;
            self.fn_ctx.label_counter += 1;
            if *is_double_ptr {
                // Need to load the inner %Type* from the %Type** before calling drop
                let ptr_tmp = format!("__drop_ptr_{}", id);
                let ret_tmp = format!("__drop_ret_{}", id);
                write_ir!(
                    ir,
                    "  %{} = load {}*, {}** %{}",
                    ptr_tmp,
                    struct_ty,
                    struct_ty,
                    llvm_name
                );
                write_ir!(
                    ir,
                    "  %{} = call i64 @{}({}* %{})",
                    ret_tmp,
                    drop_fn,
                    struct_ty,
                    ptr_tmp
                );
            } else {
                let ret_tmp = format!("__drop_ret_{}", id);
                write_ir!(
                    ir,
                    "  %{} = call i64 @{}({}* %{})",
                    ret_tmp,
                    drop_fn,
                    struct_ty,
                    llvm_name
                );
            }
        }
        ir
    }

    /// Remove scope variables from fn_ctx.locals (called after scope drop cleanup).
    /// This prevents the function-level drop cleanup from double-dropping.
    pub(crate) fn remove_scope_locals(&mut self, scope_vars: &[String]) {
        for name in scope_vars {
            self.fn_ctx.locals.remove(name);
        }
    }

    /// Generate IR to free all tracked heap allocations (scope-based auto free).
    /// Called before function exit points, after defer cleanup.
    /// Each tracked allocation has an entry-block alloca that stores the i8* pointer.
    /// At cleanup time we load from the alloca and free, avoiding SSA dominance issues
    /// when the original pointer was defined inside a conditional branch.
    pub(crate) fn generate_alloc_cleanup(&mut self) -> String {
        // DISABLED: auto-free causes use-after-free crashes when concat results
        // are still referenced by the caller. Accept memory leaks for now.
        // TODO: implement proper lifetime tracking or arena allocator.
        String::new()
    }

    /// Clear the alloc tracker (called when entering a new function)
    pub(crate) fn clear_alloc_tracker(&mut self) {
        self.fn_ctx.alloc_tracker.clear();
    }

    /// Register a heap allocation for automatic cleanup at scope exit.
    /// `ptr_reg` should be an i8* register name (e.g., "%tmp.5").
    /// Creates an entry-block alloca to store the pointer, ensuring the value
    /// is accessible from any basic block at cleanup time (avoids dominance errors).
    /// Returns IR that must be emitted at the current insertion point (the store).
    pub(crate) fn track_alloc(&mut self, ptr_reg: String) -> String {
        let id = self.fn_ctx.alloc_tracker.len();
        let alloca_name = format!("%__alloc_slot_{}", id);
        self.emit_entry_alloca(&alloca_name, "i8*");
        let mut ir = String::new();
        write_ir!(ir, "  store i8* {}, i8** {}", ptr_reg, alloca_name);
        self.fn_ctx.alloc_tracker.push((alloca_name, ptr_reg));
        ir
    }

    /// Generate IR to call Drop::drop() for all local variables that implement Drop.
    /// Called before function exit points, after defer cleanup and before alloc cleanup.
    /// Variables are dropped in reverse declaration order (LIFO), matching Rust semantics.
    pub(crate) fn generate_drop_cleanup(&mut self) -> String {
        if self.types.drop_registry.is_empty() {
            return String::new();
        }

        // Collect (var_name, llvm_name, drop_fn, is_double_ptr) in declaration order.
        // Struct literal alloca locals are stored as %Type** (double pointer), so we
        // need to load the inner %Type* before calling drop.
        let mut droppable: Vec<(String, String, String, bool)> = Vec::new();
        for (var_name, local) in &self.fn_ctx.locals {
            let type_name = match &local.ty {
                ResolvedType::Named { name, .. } => name.clone(),
                _ => continue,
            };
            if let Some(drop_fn) = self.types.drop_registry.get(&type_name).cloned() {
                if local.is_alloca() {
                    // Alloca-style struct locals are double pointers (%Type**)
                    // because the codegen stores a %Type* in an alloca.
                    // Parameters and SSA values are not owned by this function — skip them.
                    droppable.push((var_name.clone(), local.llvm_name.clone(), drop_fn, true));
                }
                // Parameters: passed by reference (&self) or by-value but caller-owned — do not drop.
                // SSA values: no alloca pointer to pass to drop — skip.
            }
        }

        if droppable.is_empty() {
            return String::new();
        }

        let mut ir = String::new();
        ir.push_str("  ; auto-drop cleanup (Drop trait)\n");
        // Drop in reverse order (last declared first)
        droppable.reverse();
        for (_var_name, llvm_name, drop_fn, is_double_ptr) in &droppable {
            let type_name_for_fn = drop_fn.strip_suffix("_drop").unwrap_or(drop_fn);
            let struct_ty = format!("%{}", type_name_for_fn);
            // Drop functions return i64 per the Drop trait definition (F drop(&self) -> i64).
            // Use a temp to capture the return value and discard it.
            let id = self.fn_ctx.label_counter;
            self.fn_ctx.label_counter += 1;
            if *is_double_ptr {
                // Load the inner %Type* from the %Type** before calling drop
                let ptr_tmp = format!("__drop_ptr_{}", id);
                let ret_tmp = format!("__drop_ret_{}", id);
                write_ir!(
                    ir,
                    "  %{} = load {}*, {}** %{}",
                    ptr_tmp,
                    struct_ty,
                    struct_ty,
                    llvm_name
                );
                write_ir!(
                    ir,
                    "  %{} = call i64 @{}({}* %{})",
                    ret_tmp,
                    drop_fn,
                    struct_ty,
                    ptr_tmp
                );
            } else {
                let ret_tmp = format!("__drop_ret_{}", id);
                write_ir!(
                    ir,
                    "  %{} = call i64 @{}({}* %{})",
                    ret_tmp,
                    drop_fn,
                    struct_ty,
                    llvm_name
                );
            }
        }

        ir
    }
}

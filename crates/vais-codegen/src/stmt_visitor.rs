//! Statement Visitor implementation for CodeGenerator
//!
//! This module implements the StmtVisitor trait for CodeGenerator,
//! providing a clean separation of statement code generation logic.

use crate::types::LocalVar;
use crate::visitor::{BlockResult, GenResult, StmtVisitor};
use crate::{CodeGenerator, CodegenError};
use vais_ast::{Expr, Spanned, Stmt};
use vais_types::ResolvedType;

impl StmtVisitor for CodeGenerator {
    fn visit_stmt(&mut self, stmt: &Spanned<Stmt>, counter: &mut usize) -> GenResult {
        match &stmt.node {
            Stmt::Let {
                name,
                ty,
                value,
                is_mut,
                ..
            } => self.generate_let_stmt_visitor(name, ty.as_ref(), value, *is_mut, counter),
            Stmt::LetDestructure {
                pattern,
                value,
                is_mut,
                ..
            } => self.generate_let_destructure(pattern, value, *is_mut, counter),
            Stmt::Expr(expr) => self.generate_expr(expr, counter),
            Stmt::Return(expr) => {
                self.generate_return_stmt_visitor(expr.as_ref().map(|e| &**e), counter)
            }
            Stmt::Break(value) => self.generate_break_stmt(value.as_ref().map(|v| &**v), counter),
            Stmt::Continue => self.generate_continue_stmt(),
            Stmt::Defer(expr) => {
                // Add the deferred expression to the stack
                // It will be executed when the function exits (in LIFO order)
                self.fn_ctx.defer_stack.push(expr.as_ref().clone());
                // No IR generated here - defer is processed at function exit
                Ok(("void".to_string(), String::new()))
            }
            Stmt::Error { message, .. } => {
                // Error nodes should not reach codegen - they indicate parsing failures
                Err(CodegenError::Unsupported(format!(
                    "Parse error in statement: {}",
                    message
                )))
            }
        }
    }

    fn visit_block_stmts(&mut self, stmts: &[Spanned<Stmt>], counter: &mut usize) -> BlockResult {
        // Push a new scope frame so Named-type locals declared in this block
        // are tracked for Drop cleanup when the block exits.
        self.enter_scope();

        let mut ir = String::new();
        let mut last_value = "0".to_string();
        let mut terminated = false;

        for stmt in stmts {
            if terminated {
                break;
            }

            let (value, stmt_ir) = self.visit_stmt(stmt, counter)?;
            ir.push_str(&stmt_ir);
            last_value = value;

            match &stmt.node {
                Stmt::Break(_) | Stmt::Continue | Stmt::Return(_) => {
                    terminated = true;
                }
                _ => {}
            }
        }

        // Pop the Named-type scope and the string-scope frames.
        let scope_vars = self.exit_scope();
        let str_frame = self.exit_scope_str();

        if !terminated {
            // Block reached natural end (no early return/break/continue).
            //
            // Determine which string slot (if any) the block's last value owns —
            // that slot's ownership transfers to the outer scope and must NOT be
            // freed here.
            let val_key = last_value
                .strip_prefix("{ i8*, i64 } ")
                .unwrap_or(&last_value)
                .trim()
                .to_string();
            let transfer_slot: Option<String> = {
                // 1. SSA register lookup (current path — works when var name == SSA reg)
                if let Some(slot) = self.fn_ctx.string_value_slot.get(&val_key).cloned() {
                    Some(slot)
                } else {
                    // 2. Ident fallback: if the last non-terminator stmt is a bare
                    //    expression resolving to a local name, look up var_string_slot by
                    //    that name. Survives SSA representation changes (e.g. alloca-backed
                    //    `let mut s`), guarding against dangling-pointer UAF.
                    let last_stmt = stmts.iter().rev().find(|s| {
                        !matches!(&s.node, Stmt::Break(_) | Stmt::Continue | Stmt::Return(_))
                    });
                    last_stmt.and_then(|s| match &s.node {
                        Stmt::Expr(e) => match &e.node {
                            Expr::Ident(name) => self.fn_ctx.var_string_slot.get(name).cloned(),
                            _ => None,
                        },
                        _ => None,
                    })
                }
            };

            // Step 1: free all string-scope slots BEFORE Named-type drops.
            // (Strings are raw heap; Named drops may reference their contents.)
            let str_cleanup_ir = self.generate_string_scope_cleanup(
                &str_frame,
                transfer_slot.as_deref(),
            );
            if !str_cleanup_ir.is_empty() {
                ir.push_str(&str_cleanup_ir);
            }

            // If a transfer slot exists and an outer string scope frame is present,
            // move the slot into the outer frame (ownership handoff).
            if let Some(ref ts) = transfer_slot {
                if let Some(outer) = self.fn_ctx.scope_str_stack.last_mut() {
                    outer.push(ts.clone());
                }
            }

            // Step 2: drop Named-type locals in LIFO order.
            let drop_ir = self.generate_scope_drop_cleanup(&scope_vars);
            if !drop_ir.is_empty() {
                ir.push_str(&drop_ir);
            }
            self.remove_scope_locals(&scope_vars);
        } else {
            // If terminated (Return/Break/Continue): the Return stmt already called
            // generate_drop_cleanup() and generate_alloc_cleanup() for ALL locals.
            // We skip scope cleanup here to avoid double-drop/double-free.
            // The str_frame is simply discarded — its slots were handled by the
            // terminator's own cleanup path.
            let _ = str_frame;
        }

        Ok((last_value, ir, terminated))
    }
}

impl CodeGenerator {
    /// Generate let statement with SSA optimization for immutable simple types
    /// (visitor variant: uses single-pointer pattern and entry-block allocas)
    fn generate_let_stmt_visitor(
        &mut self,
        name: &Spanned<String>,
        ty: Option<&Spanned<vais_ast::Type>>,
        value: &Spanned<Expr>,
        is_mut: bool,
        counter: &mut usize,
    ) -> GenResult {
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
        let is_unit_variant = if let Expr::Ident(ident_name) = &value.node {
            self.is_unit_enum_variant(ident_name)
        } else {
            false
        };

        let (val, val_ir) = self.generate_expr(value, counter)?;

        // Owning-string binding: if the RHS is a tracked concat result, record
        // the slot under this variable's name. A later `return x` reaches
        // through the alloca load (producing a fresh SSA), and we use
        // var_string_slot to recover the slot for ownership transfer. If the
        // RHS is a PHI with multiple incoming slots, pull the extras into
        // var_string_slots_multi so all are preserved at return.
        // See RFC-001 §4.5 / §4.6 (team-review UAF fix 2026-04-14).
        let val_key = val
            .strip_prefix("{ i8*, i64 } ")
            .unwrap_or(&val)
            .trim()
            .to_string();
        if let Some(slot) = self.fn_ctx.string_value_slot.get(&val_key).cloned() {
            self.fn_ctx
                .var_string_slot
                .insert(name.node.clone(), slot.clone());
            if let Some(extras) = self.fn_ctx.phi_extra_slots.remove(&val_key) {
                let mut all = vec![slot];
                all.extend(extras);
                self.fn_ctx
                    .var_string_slots_multi
                    .insert(name.node.clone(), all);
            }
        }

        let resolved_ty = ty
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(inferred_ty);

        // Determine if we can use SSA style (no alloca) to reduce stack usage
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

        let use_ssa = !is_mut
            && !is_struct_lit
            && !is_enum_variant_call
            && !is_unit_variant
            && !matches!(resolved_ty, ResolvedType::Named { .. })
            && is_simple_type;

        if use_ssa {
            // SSA style: directly alias the value, no alloca needed
            // This significantly reduces stack usage
            self.fn_ctx
                .locals
                .insert(name.node.clone(), LocalVar::ssa(resolved_ty.clone(), val));
            // If this was a lambda with captures, register the closure info
            if let Some(closure_info) = self.lambdas.last_lambda_info.take() {
                self.lambdas
                    .closures
                    .insert(name.node.clone(), closure_info);
            }
            // Track future→poll function mapping for variable-based await.
            // Use resolve_poll_func_name (instance method) instead of the static
            // extract_poll_func_name_from_expr because Spawn expressions require
            // an is_async check on the inner call to decide between the async
            // function's __poll and __sync_spawn__poll.
            if let Some(poll_fn) = self.resolve_poll_func_name(&value.node) {
                self.fn_ctx
                    .future_poll_fns
                    .insert(name.node.clone(), poll_fn);
            }
            // Return just the expression IR, no alloca/store needed
            Ok(("void".to_string(), val_ir))
        } else {
            // Traditional alloca style
            // Generate unique LLVM name for this variable (to handle loops)
            let llvm_name = format!("{}.{}", name.node, counter);
            *counter += 1;

            let mut ir = val_ir;
            let llvm_ty = self.type_to_llvm(&resolved_ty);

            // For struct literals and enum variant constructors, the value is already an alloca'd pointer.
            // Single-pointer pattern: alloca %T, then load+store from the source pointer.
            if is_struct_lit || is_enum_variant_call || is_unit_variant {
                self.emit_entry_alloca(&format!("%{}", llvm_name), &llvm_ty);
                // Load the struct value from the source pointer and store into our alloca
                let loaded = self.next_temp(counter);
                write_ir!(ir, "  {} = load {}, {}* {}", loaded, llvm_ty, llvm_ty, val);
                write_ir!(
                    ir,
                    "  store {} {}, {}* %{}",
                    llvm_ty,
                    loaded,
                    llvm_ty,
                    llvm_name
                );
            } else if matches!(resolved_ty, ResolvedType::Named { .. }) {
                // For struct values (e.g., from function returns)
                // Single-pointer: alloca %T, store value directly
                self.emit_entry_alloca(&format!("%{}", llvm_name), &llvm_ty);
                let is_value_expr = self.is_expr_value(value);
                // If the value expression is not a value (e.g., block returning
                // a struct-typed local, or load_typed returning alloca ptr), load struct first
                let actual_val = if !is_value_expr {
                    let loaded = self.next_temp(counter);
                    write_ir!(ir, "  {} = load {}, {}* {}", loaded, llvm_ty, llvm_ty, val);
                    loaded
                } else {
                    val.clone()
                };

                // Check for type mismatch: when the expression is a "value" (e.g.,
                // from a generic function call returning i64), but the target is a
                // struct type. The i64 is a pointer-as-integer to the struct data.
                let val_llvm_ty = self.llvm_type_of(&actual_val);
                if is_value_expr && val_llvm_ty == "i64" && llvm_ty.starts_with('%') {
                    // i64 is a pointer-as-integer to the struct data — memcpy
                    self.needs_llvm_memcpy = true;
                    let sizeof = self.compute_sizeof(&resolved_ty);
                    let dst_ptr = self.next_temp(counter);
                    let src_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = bitcast {}* %{} to i8*",
                        dst_ptr,
                        llvm_ty,
                        llvm_name
                    );
                    write_ir!(ir, "  {} = inttoptr i64 {} to i8*", src_ptr, actual_val);
                    write_ir!(
                        ir,
                        "  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {}, i8* {}, i64 {}, i1 false)",
                        dst_ptr,
                        src_ptr,
                        sizeof
                    );
                } else {
                    write_ir!(
                        ir,
                        "  store {} {}, {}* %{}",
                        llvm_ty,
                        actual_val,
                        llvm_ty,
                        llvm_name
                    );
                }
            } else {
                self.emit_entry_alloca(&format!("%{}", llvm_name), &llvm_ty);
                // Coerce the value width to match the alloca type (fixes P8: binary op
                // result may be wider than the declared variable type, e.g., i64 result
                // stored into an i16 alloca).
                let val_llvm_ty = self.llvm_type_of(&val);
                let store_val = if val_llvm_ty.starts_with('i')
                    && llvm_ty.starts_with('i')
                    && val_llvm_ty != llvm_ty
                {
                    self.coerce_int_width(&val, &val_llvm_ty, &llvm_ty, counter, &mut ir)
                } else if (val_llvm_ty == "float" || val_llvm_ty == "double")
                    && (llvm_ty == "float" || llvm_ty == "double")
                    && val_llvm_ty != llvm_ty
                {
                    self.coerce_float_width(&val, &val_llvm_ty, &llvm_ty, counter, &mut ir)
                } else {
                    val.clone()
                };
                write_ir!(
                    ir,
                    "  store {} {}, {}* %{}",
                    llvm_ty,
                    store_val,
                    llvm_ty,
                    llvm_name
                );
            }

            // If this was a lambda with captures, register the closure info
            if let Some(closure_info) = self.lambdas.last_lambda_info.take() {
                self.lambdas
                    .closures
                    .insert(name.node.clone(), closure_info);
            }

            // Track Named-type locals in the current scope for block-scoped drop cleanup.
            // Only Named types can implement Drop, so only they need scope tracking.
            let is_named = matches!(resolved_ty, ResolvedType::Named { .. });

            // Insert local var AFTER generating IR to avoid borrow conflicts
            self.fn_ctx
                .locals
                .insert(name.node.clone(), LocalVar::alloca(resolved_ty, llvm_name));

            // Register in current scope for block-scoped drop (if inside a scope block)
            if is_named {
                self.track_scope_local(&name.node);
            }

            // Track future→poll function mapping for variable-based await
            if let Some(poll_fn) = self.resolve_poll_func_name(&value.node) {
                self.fn_ctx
                    .future_poll_fns
                    .insert(name.node.clone(), poll_fn);
            }

            Ok(("void".to_string(), ir))
        }
    }

    /// Generate break statement
    fn generate_break_stmt(
        &mut self,
        value: Option<&Spanned<Expr>>,
        counter: &mut usize,
    ) -> GenResult {
        if let Some(labels) = self.fn_ctx.loop_stack.last() {
            // Clone to avoid borrow conflict with generate_expr
            let break_label = labels.break_label.clone();
            let mut ir = String::new();
            if let Some(expr) = value {
                let (val, expr_ir) = self.generate_expr(expr, counter)?;
                ir.push_str(&expr_ir);
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

    /// Generate continue statement
    fn generate_continue_stmt(&mut self) -> GenResult {
        if let Some(labels) = self.fn_ctx.loop_stack.last() {
            // Clone to avoid potential borrow issues
            let continue_label = labels.continue_label.clone();
            let ir = format!("  br label %{}\n", continue_label);
            Ok(("void".to_string(), ir))
        } else {
            Err(CodegenError::Unsupported(
                "continue outside of loop".to_string(),
            ))
        }
    }

    /// Generate return statement with actual ret instruction
    /// (visitor variant: uses current_return_type from fn_ctx)
    fn generate_return_stmt_visitor(
        &mut self,
        expr: Option<&Spanned<Expr>>,
        counter: &mut usize,
    ) -> GenResult {
        // Check if we're inside an async poll function — if so, early returns
        // must wrap the value as a poll result {1, value} and return the poll type.
        if let Some(poll_ctx) = self.fn_ctx.async_poll_context.clone() {
            let mut ir = String::new();

            if let Some(expr) = expr {
                // Evaluate return expression FIRST (before cleanup)
                let (val, expr_ir) = self.generate_expr(expr, counter)?;
                ir.push_str(&expr_ir);

                // Codegen promotes bool to i64 (zext), truncate back for i1 return
                let ret_val = if poll_ctx.ret_llvm == "i1" {
                    let trunc = self.next_temp(counter);
                    write_ir!(ir, "  {} = trunc i64 {} to i1", trunc, val);
                    trunc
                } else {
                    val
                };

                // Cleanup after expression evaluation, before ret
                let defer_ir = self.generate_defer_cleanup(counter)?;
                ir.push_str(&defer_ir);
                let drop_ir = self.generate_drop_cleanup();
                ir.push_str(&drop_ir);
                let alloc_cleanup_ir = self.generate_alloc_cleanup();
                ir.push_str(&alloc_cleanup_ir);

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
                ir.push_str("  store i64 -1, i64* %state_field\n");
                write_ir!(ir, "  ret {} {}", poll_ret_ty, t1);
            } else {
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
            }
            return Ok(("void".to_string(), ir));
        }

        let mut ir = String::new();

        if let Some(expr) = expr {
            // Evaluate return expression FIRST (before cleanup),
            // so we don't free memory that the expression still needs.
            let (val, expr_ir) = self.generate_expr(expr, counter)?;
            ir.push_str(&expr_ir);
            // Keep an independent copy for later ownership-transfer lookup
            // (the original `val` is moved into `ret_val` below). Strip the
            // optional "{ i8*, i64 } " prefix so the key matches the format
            // used by string_value_slot.
            let val_key = val
                .strip_prefix("{ i8*, i64 } ")
                .unwrap_or(&val)
                .trim()
                .to_string();

            // Get return type from current function context
            let ret_type = self
                .fn_ctx
                .current_return_type
                .as_ref()
                .cloned()
                .unwrap_or(ResolvedType::I64);
            let llvm_ty = self.type_to_llvm(&ret_type);

            // For struct types, may need to load from pointer
            let ret_val =
                if matches!(ret_type, ResolvedType::Named { .. }) && !self.is_expr_value(expr) {
                    let loaded = format!("%ret.{}", counter);
                    *counter += 1;
                    write_ir!(ir, "  {} = load {}, {}* {}", loaded, llvm_ty, llvm_ty, val);
                    loaded
                } else {
                    val
                };

            // Coerce return value to match declared function return type
            let mut coerced = false;
            let ret_val = if llvm_ty != "void" {
                let expr_type = self.infer_expr_type(expr);
                let val_llvm_ty = self.type_to_llvm(&expr_type);
                if val_llvm_ty != llvm_ty {
                    coerced = true;
                    if (val_llvm_ty == "float" || val_llvm_ty == "double")
                        && (llvm_ty == "float" || llvm_ty == "double")
                    {
                        self.coerce_float_width(&ret_val, &val_llvm_ty, &llvm_ty, counter, &mut ir)
                    } else if val_llvm_ty.starts_with('i') && llvm_ty.starts_with('i') {
                        self.coerce_int_width(&ret_val, &val_llvm_ty, &llvm_ty, counter, &mut ir)
                    } else if matches!(ret_type, ResolvedType::Named { .. })
                        && (ret_val == "0" || ret_val.starts_with("0"))
                    {
                        // Returning integer 0 for a struct type → use zeroinitializer
                        "zeroinitializer".to_string()
                    } else if val_llvm_ty.starts_with('i')
                        && (llvm_ty == "float" || llvm_ty == "double")
                    {
                        // int → float coercion (e.g., i64 → double in specialized HashMap return)
                        let tmp = self.next_temp(counter);
                        write_ir!(ir, "  {} = bitcast i64 {} to double", tmp, ret_val);
                        if llvm_ty == "float" {
                            let tmp2 = self.next_temp(counter);
                            write_ir!(ir, "  {} = fptrunc double {} to float", tmp2, tmp);
                            tmp2
                        } else {
                            tmp
                        }
                    } else {
                        coerced = false;
                        ret_val
                    }
                } else {
                    ret_val
                }
            } else {
                ret_val
            };

            // Execute deferred expressions before return (LIFO order)
            let defer_ir = self.generate_defer_cleanup(counter)?;
            ir.push_str(&defer_ir);

            // Call Drop::drop() for droppable locals (reverse order)
            let drop_ir = self.generate_drop_cleanup();
            ir.push_str(&drop_ir);

            // Return-value ownership transfer: if the returned SSA fat pointer
            // owns a tracked heap buffer (e.g., result of concat/substring),
            // exclude its slot from free. See RFC-001 §4.6.
            // `val` is the original SSA reg (before coerce), which is what
            // string_value_slot is keyed on.
            if matches!(ret_type, ResolvedType::Str) {
                // Variable-name lookup takes priority when the return expression
                // is a local identifier — this covers `let x = phi_result;
                // return x` where the let-binding hook already consumed
                // phi_extra_slots into var_string_slots_multi. Direct-SSA
                // matching would miss the extras because the let hook transfers
                // them to the variable's multi-slot list.
                let mut matched = false;
                if let Expr::Ident(name) = &expr.node {
                    if let Some(slots) =
                        self.fn_ctx.var_string_slots_multi.get(name).cloned()
                    {
                        self.fn_ctx.pending_return_skip_slot.extend(slots);
                        matched = true;
                    } else if let Some(slot) =
                        self.fn_ctx.var_string_slot.get(name).cloned()
                    {
                        self.fn_ctx.pending_return_skip_slot.push(slot);
                        matched = true;
                    }
                }
                // Fallback: direct SSA match (e.g. `return a + b` without a
                // binding). Also includes phi_extra_slots for inline PHI
                // returns like `return I c { a+b } E { c+d }`.
                if !matched {
                    if let Some(slot) =
                        self.fn_ctx.string_value_slot.get(&val_key).cloned()
                    {
                        self.fn_ctx.pending_return_skip_slot.push(slot);
                        if let Some(extras) =
                            self.fn_ctx.phi_extra_slots.get(&val_key).cloned()
                        {
                            self.fn_ctx.pending_return_skip_slot.extend(extras);
                        }
                    }
                }
            }

            // Free tracked heap allocations before return
            let alloc_cleanup_ir = self.generate_alloc_cleanup();
            ir.push_str(&alloc_cleanup_ir);

            // Final safety: coerce if actual IR value type differs from ret type
            // (catches cases where body sext'd i32→i64 but function returns i32)
            // Skip if we already coerced above to avoid double trunc
            let ret_val = if !coerced {
                let actual = self.llvm_type_of(&ret_val);
                if actual != llvm_ty && actual.starts_with('i') && llvm_ty.starts_with('i') {
                    self.coerce_int_width(&ret_val, &actual, &llvm_ty, counter, &mut ir)
                } else {
                    ret_val
                }
            } else {
                ret_val
            };

            write_ir!(ir, "  ret {} {}", llvm_ty, ret_val);
        } else {
            // Execute deferred expressions before return (LIFO order)
            let defer_ir = self.generate_defer_cleanup(counter)?;
            ir.push_str(&defer_ir);

            // Call Drop::drop() for droppable locals (reverse order)
            let drop_ir = self.generate_drop_cleanup();
            ir.push_str(&drop_ir);

            // Free tracked heap allocations before return
            let alloc_cleanup_ir = self.generate_alloc_cleanup();
            ir.push_str(&alloc_cleanup_ir);

            ir.push_str("  ret void\n");
        }

        Ok(("void".to_string(), ir))
    }
}

//! Core function and method code generation

use crate::types::LocalVar;
use crate::{CodeGenerator, CodegenError, CodegenResult};
use vais_ast::{Function, FunctionBody, Span};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Convenience wrapper for generate_function_with_span with default span.
    #[allow(dead_code)]
    pub(crate) fn generate_function(&mut self, f: &Function) -> CodegenResult<String> {
        use std::cell::Cell;
        thread_local! { static DEPTH: Cell<usize> = const { Cell::new(0) }; }
        DEPTH.with(|d| {
            let current = d.get();
            if current > 10 {
                return Err(CodegenError::InternalError(format!(
                    "recursion limit in generate_function: {}",
                    f.name.node
                )));
            }
            d.set(current + 1);
            let result = self.generate_function_with_span(f, Span::default());
            d.set(current);
            result
        })
    }

    pub(crate) fn generate_function_with_span(
        &mut self,
        f: &Function,
        span: Span,
    ) -> CodegenResult<String> {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            stacker::maybe_grow(32 * 1024 * 1024, 64 * 1024 * 1024, || {
                self.generate_function_with_span_inner(f, span)
            })
        }));
        match result {
            Ok(r) => r,
            Err(_) => {
                eprintln!(
                    "[WARN] Stack overflow during codegen of '{}' — skipping",
                    f.name.node
                );
                Ok(String::new())
            }
        }
    }

    #[inline(never)]
    fn generate_function_with_span_inner(
        &mut self,
        f: &Function,
        span: Span,
    ) -> CodegenResult<String> {
        // Check if this is an async function
        if f.is_async {
            return self.generate_async_function(f);
        }

        self.initialize_function_state(&f.name.node);
        self.clear_defer_stack();
        self.clear_alloc_tracker();

        // Create debug info for this function
        let func_line = self.debug_info.offset_to_line(span.start);
        let di_subprogram =
            self.debug_info
                .create_function_debug_info(&f.name.node, func_line, true);

        // Get registered function signature for resolved param types (supports Type::Infer)
        let registered_param_types: Vec<_> = self
            .types
            .functions
            .get(&f.name.node)
            .map(|info| {
                info.signature
                    .params
                    .iter()
                    .map(|(_, ty, _)| ty.clone())
                    .collect()
            })
            .unwrap_or_default();

        let params: Vec<_> = f
            .params
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let mut ty = if i < registered_param_types.len() {
                    registered_param_types[i].clone()
                } else {
                    self.ast_type_to_resolved(&p.ty.node)
                };
                // For "self" parameters, if type resolved to I64 but we're inside a method,
                // use the struct type from the function name (e.g., TestSuiteResult_add → &TestSuiteResult)
                if (p.name.node == "self" || p.name.node == "mut self") && ty == ResolvedType::I64 {
                    // Extract struct name from function name (StructName_method)
                    if let Some(underscore_pos) = f.name.node.rfind('_') {
                        let struct_name = &f.name.node[..underscore_pos];
                        if self.types.structs.contains_key(struct_name) {
                            ty = ResolvedType::Ref(Box::new(ResolvedType::Named {
                                name: struct_name.to_string(),
                                generics: vec![],
                            }));
                        }
                    }
                }
                let llvm_ty = self.type_to_llvm(&ty);

                // Register parameter as local (SSA value, not alloca)
                let llvm_name = crate::helpers::sanitize_param_name(&p.name.node);
                self.fn_ctx.locals.insert(
                    p.name.node.to_string(),
                    LocalVar::param(ty.clone(), llvm_name.to_string()),
                );

                format!("{} %{}", llvm_ty, llvm_name)
            })
            .collect();

        let ret_type_raw = self.resolve_fn_return_type(f, &f.name.node);

        // main() must return i64 for C ABI compatibility, regardless of declared return type.
        // If main declares f64/f32 return, we force i64 and add fptosi at the return site.
        let is_main_float_ret =
            f.name.node == "main" && matches!(ret_type_raw, ResolvedType::F64 | ResolvedType::F32);
        let ret_type = if is_main_float_ret {
            ResolvedType::I64
        } else {
            ret_type_raw.clone()
        };

        // Store current return type for nested return statements
        self.fn_ctx.current_return_type = Some(ret_type.clone());

        let ret_llvm = self.type_to_llvm(&ret_type);

        // Build function definition with optional debug info reference
        let dbg_ref = if let Some(sp_id) = di_subprogram {
            format!(" !dbg !{}", sp_id)
        } else {
            String::new()
        };

        let mut ir = format!(
            "define {} @{}({}){} {{\n",
            ret_llvm,
            f.name.node,
            params.join(", "),
            dbg_ref
        );

        ir.push_str("entry:\n");

        // For struct parameters, allocate stack space and store the value
        // This allows field access to work via getelementptr
        for (i, p) in f.params.iter().enumerate() {
            let ty = if i < registered_param_types.len() {
                registered_param_types[i].clone()
            } else {
                self.ast_type_to_resolved(&p.ty.node)
            };
            if matches!(ty, ResolvedType::Named { .. }) {
                let llvm_ty = self.type_to_llvm(&ty);
                let src_llvm_name = crate::helpers::sanitize_param_name(&p.name.node);
                let param_ptr_name = format!("__{}_ptr", p.name.node);
                let param_ptr = format!("%{}", param_ptr_name);
                write_ir!(ir, "  {} = alloca {}", param_ptr, llvm_ty);
                write_ir!(
                    ir,
                    "  store {} %{}, {}* {}",
                    llvm_ty,
                    src_llvm_name,
                    llvm_ty,
                    param_ptr
                );
                // Update locals to use SSA with the pointer as the value (including %)
                // This makes the ident handler treat it as a direct pointer value, not a double pointer
                self.fn_ctx.locals.insert(
                    p.name.node.to_string(),
                    LocalVar::ssa(ty.clone(), param_ptr),
                );
            }
            // For &str parameters (Ref(Str)), the LLVM param type is { i8*, i64 }
            // (value, not pointer — since type_to_llvm treats &str as str fat ptr).
            // Register an SSA alias so body code uses the param directly by value.
            let is_ref_str = matches!(
                &ty,
                ResolvedType::Ref(inner) if matches!(inner.as_ref(), ResolvedType::Str)
            ) || matches!(
                &ty,
                ResolvedType::RefMut(inner) if matches!(inner.as_ref(), ResolvedType::Str)
            );
            if is_ref_str {
                let src_llvm_name = crate::helpers::sanitize_param_name(&p.name.node);
                let param_val = format!("%{}", src_llvm_name);
                // Use the param value directly (no load needed — it's already by-value)
                self.fn_ctx.locals.insert(
                    p.name.node.to_string(),
                    LocalVar::ssa(ResolvedType::Str, param_val),
                );
            }
        }

        // Generate body
        let mut counter = 0;

        // Generate runtime dependent type assertions for parameters
        let dep_assert_ir = self.generate_dependent_type_assertions(
            &f.params,
            &registered_param_types,
            &mut counter,
        )?;
        ir.push_str(&dep_assert_ir);

        // Generate requires (precondition) checks
        let requires_ir = self.generate_requires_checks(f, &mut counter)?;
        ir.push_str(&requires_ir);

        // Generate automatic contract checks from #[contract] attribute
        let auto_contract_ir = self.generate_auto_contract_checks(f, &mut counter)?;
        ir.push_str(&auto_contract_ir);

        // Generate decreases checks for termination proof
        let decreases_ir = self.generate_decreases_checks(f, &mut counter)?;
        ir.push_str(&decreases_ir);

        match &f.body {
            FunctionBody::Expr(expr) => {
                let (value, expr_ir) = self.generate_expr(expr, &mut counter)?;
                ir.push_str(&expr_ir);

                // Execute deferred expressions before return (LIFO order)
                let defer_ir = self.generate_defer_cleanup(&mut counter)?;
                ir.push_str(&defer_ir);

                // Call Drop::drop() for droppable locals (reverse order)
                let drop_ir = self.generate_drop_cleanup();
                ir.push_str(&drop_ir);

                // Free tracked heap allocations before return
                let alloc_cleanup_ir = self.generate_alloc_cleanup();
                ir.push_str(&alloc_cleanup_ir);

                // Generate ensures (postcondition) checks before return
                let ensures_ir =
                    self.generate_ensures_checks(f, &value, &ret_type, &mut counter)?;
                ir.push_str(&ensures_ir);

                // main() with f64/f32 body needs fptosi conversion to i64
                let value = if is_main_float_ret {
                    let is_float_literal = !value.starts_with('%')
                        && (value.contains("e+") || value.contains("e-"));
                    let is_int_literal =
                        !value.starts_with('%') && value.chars().all(|c| c.is_ascii_digit() || c == '-');
                    if is_int_literal {
                        // Integer literal (e.g., 42) for `F main() -> f64 = 42` — already i64
                        value
                    } else if is_float_literal || value.starts_with('%') {
                        // Float literals are always in double format (e.g., "1.000000e+00").
                        // Use the actual LLVM type for variables (%foo), but always "double"
                        // for bare float literals — even when declared return is f32.
                        let float_ty = if value.starts_with('%') {
                            if matches!(ret_type_raw, ResolvedType::F32) {
                                "float"
                            } else {
                                "double"
                            }
                        } else {
                            "double" // bare literal is always double-precision format
                        };
                        let converted = format!("%main_fptosi.{}", counter);
                        counter += 1;
                        write_ir!(ir, "  {} = fptosi {} {} to i64", converted, float_ty, value);
                        converted
                    } else {
                        value
                    }
                } else {
                    value
                };

                let ret_dbg = self.debug_info.dbg_ref_from_offset(expr.span.start);
                if ret_type == ResolvedType::Unit {
                    write_ir!(ir, "  ret void{}", ret_dbg);
                } else if matches!(ret_type, ResolvedType::Named { .. }) {
                    // For struct returns, load the value from pointer
                    let loaded = format!("%ret.{}", counter);
                    write_ir!(
                        ir,
                        "  {} = load {}, {}* {}{}",
                        loaded,
                        ret_llvm,
                        ret_llvm,
                        value,
                        ret_dbg
                    );
                    write_ir!(ir, "  ret {} {}{}", ret_llvm, loaded, ret_dbg);
                } else {
                    // Phase 191: float literal returned as integer — needs fptosi.
                    // E.g., `F main() -> i64 = 3.14` generates `ret i64 3.140000e+00`
                    // which is invalid; should be `fptosi double 3.14... to i64`.
                    let value = if ret_llvm.starts_with('i')
                        && !value.starts_with('%')
                        && (value.contains("e+") || value.contains("e-"))
                    {
                        let tmp = self.next_temp(&mut counter);
                        write_ir!(ir, "  {} = fptosi double {} to {}", tmp, value, ret_llvm);
                        tmp
                    } else {
                        value
                    };

                    // Coerce return value width if needed. Use i64 as assumed source
                    // for small int returns (body convention is "everything is i64").
                    let ret_width = Self::int_type_width(&ret_llvm);
                    let value = if value.starts_with('%') && ret_width > 0 && ret_width < 64 {
                        let trunc_tmp = self.next_temp(&mut counter);
                        write_ir!(ir, "  {} = trunc i64 {} to {}", trunc_tmp, value, ret_llvm);
                        trunc_tmp
                    } else if value.starts_with('%') {
                        let val_llvm = self.llvm_type_of(&value);
                        if (val_llvm == "float" || val_llvm == "double")
                            && (ret_llvm == "float" || ret_llvm == "double")
                            && val_llvm != ret_llvm
                        {
                            // Float width coercion for return
                            let tmp = self.next_temp(&mut counter);
                            if val_llvm == "double" && ret_llvm == "float" {
                                write_ir!(ir, "  {} = fptrunc double {} to float", tmp, value);
                            } else {
                                write_ir!(ir, "  {} = fpext float {} to double", tmp, value);
                            }
                            tmp
                        } else {
                            self.coerce_int_width(
                                &value,
                                &val_llvm,
                                &ret_llvm,
                                &mut counter,
                                &mut ir,
                            )
                        }
                    } else {
                        value
                    };
                    write_ir!(ir, "  ret {} {}{}", ret_llvm, value, ret_dbg);
                }
            }
            FunctionBody::Block(stmts) => {
                let (value, block_ir, terminated) =
                    self.generate_block_stmts(stmts, &mut counter)?;
                ir.push_str(&block_ir);

                // If block is already terminated (has return/break), don't emit ret
                if terminated {
                    // Block already has a terminator, no need for ret
                    // Note: defer cleanup for early returns is handled in Return statement
                    // Note: alloc cleanup for early returns is handled in Return statement
                } else {
                    // Execute deferred expressions before return (LIFO order)
                    let defer_ir = self.generate_defer_cleanup(&mut counter)?;
                    ir.push_str(&defer_ir);

                    // Call Drop::drop() for droppable locals (reverse order)
                    let drop_ir = self.generate_drop_cleanup();
                    ir.push_str(&drop_ir);

                    // Free tracked heap allocations before return
                    let alloc_cleanup_ir = self.generate_alloc_cleanup();
                    ir.push_str(&alloc_cleanup_ir);

                    // Generate ensures (postcondition) checks before return
                    let ensures_ir =
                        self.generate_ensures_checks(f, &value, &ret_type, &mut counter)?;
                    ir.push_str(&ensures_ir);

                    // Get debug location from last statement or function end
                    let ret_offset = stmts.last().map(|s| s.span.end).unwrap_or(span.end);
                    let ret_dbg = self.debug_info.dbg_ref_from_offset(ret_offset);
                    if ret_type == ResolvedType::Unit {
                        write_ir!(ir, "  ret void{}", ret_dbg);
                    } else if f.name.node == "main"
                        && ret_type == ResolvedType::I64
                        && f.ret_type.is_none()
                        && value == "void"
                    {
                        // main() with implicit i64 return and Unit body: auto-return 0
                        write_ir!(ir, "  ret i64 0{}", ret_dbg);
                    } else if matches!(ret_type, ResolvedType::Named { .. }) {
                        // Check if the result is already a value (from phi node) or a pointer (from struct lit)
                        if self.is_block_result_value(stmts) {
                            // Value — check if type name matches return type.
                            // Generic calls may return %Vec while function declares %Vec$i64.
                            let val_llvm = self.llvm_type_of(&value);
                            if val_llvm != ret_llvm
                                && val_llvm.starts_with('%')
                                && ret_llvm.starts_with('%')
                            {
                                // Structurally identical but differently named types —
                                // bitcast via alloca to reconcile.
                                let tmp_alloca = self.next_temp(&mut counter);
                                self.emit_entry_alloca(&tmp_alloca, &val_llvm);
                                write_ir!(
                                    ir,
                                    "  store {} {}, {}* {}",
                                    val_llvm,
                                    value,
                                    val_llvm,
                                    tmp_alloca
                                );
                                let cast_ptr = self.next_temp(&mut counter);
                                write_ir!(
                                    ir,
                                    "  {} = bitcast {}* {} to {}*",
                                    cast_ptr,
                                    val_llvm,
                                    tmp_alloca,
                                    ret_llvm
                                );
                                let loaded = self.next_temp(&mut counter);
                                write_ir!(
                                    ir,
                                    "  {} = load {}, {}* {}{}",
                                    loaded,
                                    ret_llvm,
                                    ret_llvm,
                                    cast_ptr,
                                    ret_dbg
                                );
                                write_ir!(ir, "  ret {} {}{}", ret_llvm, loaded, ret_dbg);
                            } else {
                                write_ir!(ir, "  ret {} {}{}", ret_llvm, value, ret_dbg);
                            }
                        } else {
                            // Pointer (e.g., from struct literal) - load then return
                            let loaded = format!("%ret.{}", counter);
                            write_ir!(
                                ir,
                                "  {} = load {}, {}* {}{}",
                                loaded,
                                ret_llvm,
                                ret_llvm,
                                value,
                                ret_dbg
                            );
                            write_ir!(ir, "  ret {} {}{}", ret_llvm, loaded, ret_dbg);
                        }
                    } else if matches!(ret_type, ResolvedType::Ref(_) | ResolvedType::RefMut(_))
                        && !value.starts_with('%')
                        && !value.starts_with('@')
                    {
                        // Reference return with bare literal: promote to global constant
                        let inner_ty = match &ret_type {
                            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                                self.type_to_llvm(inner)
                            }
                            _ => unreachable!(),
                        };
                        let const_name = format!(".ref.const.{}", self.ref_constant_counter);
                        self.ref_constant_counter += 1;
                        self.ref_constants
                            .push((const_name.clone(), inner_ty, value.clone()));
                        write_ir!(ir, "  ret {} @{}{}", ret_llvm, const_name, ret_dbg);
                    } else if ret_llvm.starts_with('{') {
                        // Inline struct return: if value is scalar, use zeroinitializer.
                        if value.starts_with('%') && self.llvm_type_of(&value).starts_with('{') {
                            write_ir!(ir, "  ret {} {}{}", ret_llvm, value, ret_dbg);
                        } else {
                            write_ir!(ir, "  ret {} zeroinitializer{}", ret_llvm, ret_dbg);
                        }
                    } else {
                        // Coerce return value width if needed (int, float, struct).
                        let value = if value.starts_with('%') {
                            let val_llvm = self.llvm_type_of(&value);
                            if (val_llvm == "float" || val_llvm == "double")
                                && (ret_llvm == "float" || ret_llvm == "double")
                                && val_llvm != ret_llvm
                            {
                                let tmp = self.next_temp(&mut counter);
                                if val_llvm == "double" && ret_llvm == "float" {
                                    write_ir!(ir, "  {} = fptrunc double {} to float", tmp, value);
                                } else {
                                    write_ir!(ir, "  {} = fpext float {} to double", tmp, value);
                                }
                                tmp
                            } else if val_llvm == "i64"
                                && ret_llvm.starts_with('%')
                                && !ret_llvm.ends_with('*')
                            {
                                let tmp_ptr = self.next_temp(&mut counter);
                                write_ir!(
                                    ir,
                                    "  {} = inttoptr i64 {} to {}*",
                                    tmp_ptr,
                                    value,
                                    ret_llvm
                                );
                                let loaded = self.next_temp(&mut counter);
                                write_ir!(
                                    ir,
                                    "  {} = load {}, {}* {}",
                                    loaded,
                                    ret_llvm,
                                    ret_llvm,
                                    tmp_ptr
                                );
                                loaded
                            } else {
                                self.coerce_int_width(
                                    &value,
                                    &val_llvm,
                                    &ret_llvm,
                                    &mut counter,
                                    &mut ir,
                                )
                            }
                        } else {
                            value
                        };
                        write_ir!(ir, "  ret {} {}{}", ret_llvm, value, ret_dbg);
                    }
                }
            }
        }

        ir.push_str("}\n");

        // Hoist collected entry-block allocas into the entry block
        self.splice_entry_allocas(&mut ir);

        self.fn_ctx.current_function = None;
        self.fn_ctx.current_return_type = None;
        self.clear_decreases_info();
        Ok(ir)
    }

    pub(crate) fn generate_method_with_span(
        &mut self,
        struct_name: &str,
        f: &Function,
        span: Span,
    ) -> CodegenResult<String> {
        // Resolve generic struct aliases (e.g., "Pair" -> "Pair$i64")
        let resolved_struct_name = self.resolve_struct_name(struct_name);
        let struct_name = resolved_struct_name.as_str();

        // Phase 191: For specialized structs (e.g., "Vec$f32"), set up generic
        // substitutions so method params/return types use concrete types instead
        // of falling back to i64. Extract base name, look up AST generic params
        // and specialized struct fields to build the substitution map.
        let old_substitutions = if let Some(dollar_pos) = struct_name.find('$') {
            let base_name = &struct_name[..dollar_pos];
            if let Some(struct_def) = self.generics.struct_defs.get(base_name).cloned() {
                if let Some(specialized) = self.types.structs.get(struct_name).cloned() {
                    let type_params: Vec<_> = struct_def
                        .generics
                        .iter()
                        .filter(|g| {
                            !matches!(g.kind, vais_ast::GenericParamKind::Lifetime { .. })
                        })
                        .collect();
                    // Match generic params to concrete types from specialized fields:
                    // struct_def.fields has generic types (T), specialized.fields has concrete types (f32)
                    let mut subst = std::collections::HashMap::new();
                    for (ast_field, spec_field) in
                        struct_def.fields.iter().zip(specialized.fields.iter())
                    {
                        if let vais_ast::Type::Named { name, .. } = &ast_field.ty.node {
                            if type_params.iter().any(|p| &p.name.node == name) {
                                subst.insert(name.clone(), spec_field.1.clone());
                            }
                        }
                    }
                    if !subst.is_empty() {
                        Some(std::mem::replace(&mut self.generics.substitutions, subst))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Method name: StructName_methodName
        let method_name = format!("{}_{}", struct_name, f.name.node);

        self.initialize_function_state(&method_name);
        self.clear_alloc_tracker();

        // Create debug info for this method
        let func_line = self.debug_info.offset_to_line(span.start);
        let di_subprogram =
            self.debug_info
                .create_function_debug_info(&method_name, func_line, true);

        // Check if this is a static method (no &self or self parameter)
        let has_self = f
            .params
            .first()
            .map(|p| p.name.node == "self")
            .unwrap_or(false);

        let mut params = Vec::new();

        if has_self {
            // Instance method: first parameter is `self` (pointer to struct)
            let struct_ty = format!("%{}*", struct_name);
            params.push(format!("{} %self", struct_ty));

            // Register self
            self.fn_ctx.locals.insert(
                "self".to_string(),
                LocalVar::param(
                    ResolvedType::Named {
                        name: struct_name.to_string(),
                        generics: vec![],
                    },
                    "self".to_string(),
                ),
            );
        }

        // Add remaining parameters
        for p in &f.params {
            // Skip `self` parameter if it exists in the AST
            if p.name.node == "self" {
                continue;
            }

            let ty = self.ast_type_to_resolved(&p.ty.node);
            let llvm_ty = self.type_to_llvm(&ty);

            let llvm_name = crate::helpers::sanitize_param_name(&p.name.node);
            self.fn_ctx.locals.insert(
                p.name.node.to_string(),
                LocalVar::param(ty.clone(), llvm_name.to_string()),
            );

            params.push(format!("{} %{}", llvm_ty, llvm_name));
        }

        let ret_type = self.resolve_fn_return_type(f, &f.name.node);

        // Store current return type for nested return statements
        self.fn_ctx.current_return_type = Some(ret_type.clone());

        let ret_llvm = self.type_to_llvm(&ret_type);

        // Build method definition with optional debug info reference
        let dbg_ref = if let Some(sp_id) = di_subprogram {
            format!(" !dbg !{}", sp_id)
        } else {
            String::new()
        };

        let mut ir = format!(
            "define {} @{}({}){} {{\n",
            ret_llvm,
            method_name,
            params.join(", "),
            dbg_ref
        );

        ir.push_str("entry:\n");

        // For struct parameters, allocate stack space and store the value
        // This allows field access to work via getelementptr
        for p in &f.params {
            if p.name.node == "self" {
                continue;
            }
            let ty = self.ast_type_to_resolved(&p.ty.node);
            if matches!(ty, ResolvedType::Named { .. }) {
                let llvm_ty = self.type_to_llvm(&ty);
                let src_llvm_name = crate::helpers::sanitize_param_name(&p.name.node);
                let param_ptr_name = format!("__{}_ptr", p.name.node);
                let param_ptr = format!("%{}", param_ptr_name);
                write_ir!(ir, "  {} = alloca {}", param_ptr, llvm_ty);
                write_ir!(
                    ir,
                    "  store {} %{}, {}* {}",
                    llvm_ty,
                    src_llvm_name,
                    llvm_ty,
                    param_ptr
                );
                // Update locals to use SSA with the pointer as the value (including %)
                // This makes the ident handler treat it as a direct pointer value, not a double pointer
                self.fn_ctx.locals.insert(
                    p.name.node.to_string(),
                    LocalVar::ssa(ty.clone(), param_ptr),
                );
            }
        }

        // Generate body
        let mut counter = 0;
        match &f.body {
            FunctionBody::Expr(expr) => {
                let (value, expr_ir) = self.generate_expr(expr, &mut counter)?;
                ir.push_str(&expr_ir);

                // Call Drop::drop() for droppable locals (reverse order)
                let drop_ir = self.generate_drop_cleanup();
                ir.push_str(&drop_ir);

                // Free tracked heap allocations before return
                let alloc_cleanup_ir = self.generate_alloc_cleanup();
                ir.push_str(&alloc_cleanup_ir);

                let ret_dbg = self.debug_info.dbg_ref_from_offset(expr.span.start);
                if ret_type == ResolvedType::Unit {
                    write_ir!(ir, "  ret void{}", ret_dbg);
                } else if matches!(ret_type, ResolvedType::Named { .. }) {
                    // For struct returns, load the value from pointer
                    let loaded = format!("%ret.{}", counter);
                    write_ir!(
                        ir,
                        "  {} = load {}, {}* {}{}",
                        loaded,
                        ret_llvm,
                        ret_llvm,
                        value,
                        ret_dbg
                    );
                    write_ir!(ir, "  ret {} {}{}", ret_llvm, loaded, ret_dbg);
                } else if matches!(ret_type, ResolvedType::Ref(_) | ResolvedType::RefMut(_))
                    && !value.starts_with('%')
                    && !value.starts_with('@')
                {
                    let inner_ty = match &ret_type {
                        ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                            self.type_to_llvm(inner)
                        }
                        _ => unreachable!(),
                    };
                    let const_name = format!(".ref.const.{}", self.ref_constant_counter);
                    self.ref_constant_counter += 1;
                    self.ref_constants
                        .push((const_name.clone(), inner_ty, value.clone()));
                    write_ir!(ir, "  ret {} @{}{}", ret_llvm, const_name, ret_dbg);
                } else {
                    let value = if value.starts_with('%') {
                        let val_llvm = self.llvm_type_of(&value);
                        self.coerce_int_width(&value, &val_llvm, &ret_llvm, &mut counter, &mut ir)
                    } else {
                        value
                    };
                    write_ir!(ir, "  ret {} {}{}", ret_llvm, value, ret_dbg);
                }
            }
            FunctionBody::Block(stmts) => {
                let (value, block_ir, terminated) =
                    self.generate_block_stmts(stmts, &mut counter)?;
                ir.push_str(&block_ir);

                // If block is already terminated (has return/break), don't emit ret
                if terminated {
                    // Block already has a terminator, no need for ret
                    // Note: alloc cleanup for early returns is handled in Return statement
                } else {
                    // Call Drop::drop() for droppable locals (reverse order)
                    let drop_ir = self.generate_drop_cleanup();
                    ir.push_str(&drop_ir);

                    // Free tracked heap allocations before return
                    let alloc_cleanup_ir = self.generate_alloc_cleanup();
                    ir.push_str(&alloc_cleanup_ir);

                    let ret_offset = stmts.last().map(|s| s.span.end).unwrap_or(span.end);
                    let ret_dbg = self.debug_info.dbg_ref_from_offset(ret_offset);
                    if ret_type == ResolvedType::Unit {
                        write_ir!(ir, "  ret void{}", ret_dbg);
                    } else if matches!(ret_type, ResolvedType::Named { .. }) {
                        // Check if the result is already a value (from phi node) or a pointer (from struct lit)
                        if self.is_block_result_value(stmts) {
                            // Value (e.g., from if-else phi node) - return directly
                            write_ir!(ir, "  ret {} {}{}", ret_llvm, value, ret_dbg);
                        } else {
                            // Pointer (e.g., from struct literal) - load then return
                            let loaded = format!("%ret.{}", counter);
                            write_ir!(
                                ir,
                                "  {} = load {}, {}* {}{}",
                                loaded,
                                ret_llvm,
                                ret_llvm,
                                value,
                                ret_dbg
                            );
                            write_ir!(ir, "  ret {} {}{}", ret_llvm, loaded, ret_dbg);
                        }
                    } else if matches!(ret_type, ResolvedType::Ref(_) | ResolvedType::RefMut(_))
                        && !value.starts_with('%')
                        && !value.starts_with('@')
                    {
                        let inner_ty = match &ret_type {
                            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                                self.type_to_llvm(inner)
                            }
                            _ => unreachable!(),
                        };
                        let const_name = format!(".ref.const.{}", self.ref_constant_counter);
                        self.ref_constant_counter += 1;
                        self.ref_constants
                            .push((const_name.clone(), inner_ty, value.clone()));
                        write_ir!(ir, "  ret {} @{}{}", ret_llvm, const_name, ret_dbg);
                    } else {
                        // Coerce return value width if needed (int, float, struct).
                        let value = if value.starts_with('%') {
                            let val_llvm = self.llvm_type_of(&value);
                            if (val_llvm == "float" || val_llvm == "double")
                                && (ret_llvm == "float" || ret_llvm == "double")
                                && val_llvm != ret_llvm
                            {
                                let tmp = self.next_temp(&mut counter);
                                if val_llvm == "double" && ret_llvm == "float" {
                                    write_ir!(ir, "  {} = fptrunc double {} to float", tmp, value);
                                } else {
                                    write_ir!(ir, "  {} = fpext float {} to double", tmp, value);
                                }
                                tmp
                            } else if val_llvm == "i64"
                                && ret_llvm.starts_with('%')
                                && !ret_llvm.ends_with('*')
                            {
                                let tmp_ptr = self.next_temp(&mut counter);
                                write_ir!(
                                    ir,
                                    "  {} = inttoptr i64 {} to {}*",
                                    tmp_ptr,
                                    value,
                                    ret_llvm
                                );
                                let loaded = self.next_temp(&mut counter);
                                write_ir!(
                                    ir,
                                    "  {} = load {}, {}* {}",
                                    loaded,
                                    ret_llvm,
                                    ret_llvm,
                                    tmp_ptr
                                );
                                loaded
                            } else {
                                self.coerce_int_width(
                                    &value,
                                    &val_llvm,
                                    &ret_llvm,
                                    &mut counter,
                                    &mut ir,
                                )
                            }
                        } else {
                            value
                        };
                        write_ir!(ir, "  ret {} {}{}", ret_llvm, value, ret_dbg);
                    }
                }
            }
        }

        ir.push_str("}\n");

        // Hoist collected entry-block allocas into the entry block
        self.splice_entry_allocas(&mut ir);

        self.fn_ctx.current_function = None;
        self.fn_ctx.current_return_type = None;

        // Restore previous substitutions if we set them for a specialized struct
        if let Some(old) = old_substitutions {
            self.generics.substitutions = old;
        }

        Ok(ir)
    }
}

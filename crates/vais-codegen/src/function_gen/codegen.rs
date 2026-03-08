//! Core function and method code generation

use crate::types::LocalVar;
use crate::{CodeGenerator, CodegenResult};
use vais_ast::{Function, FunctionBody, Span};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Convenience wrapper for generate_function_with_span with default span.
    #[cfg(test)]
    pub(crate) fn generate_function(&mut self, f: &Function) -> CodegenResult<String> {
        self.generate_function_with_span(f, Span::default())
    }

    pub(crate) fn generate_function_with_span(
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
                let ty = if i < registered_param_types.len() {
                    registered_param_types[i].clone()
                } else {
                    self.ast_type_to_resolved(&p.ty.node)
                };
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
                ir.push_str(&format!("  {} = alloca {}\n", param_ptr, llvm_ty));
                ir.push_str(&format!(
                    "  store {} %{}, {}* {}\n",
                    llvm_ty, src_llvm_name, llvm_ty, param_ptr
                ));
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

                // Free tracked heap allocations before return
                let alloc_cleanup_ir = self.generate_alloc_cleanup();
                ir.push_str(&alloc_cleanup_ir);

                // Generate ensures (postcondition) checks before return
                let ensures_ir =
                    self.generate_ensures_checks(f, &value, &ret_type, &mut counter)?;
                ir.push_str(&ensures_ir);

                // main() with f64/f32 body needs fptosi conversion to i64
                let value = if is_main_float_ret {
                    let float_ty = if matches!(ret_type_raw, ResolvedType::F32) {
                        "float"
                    } else {
                        "double"
                    };
                    let converted = format!("%main_fptosi.{}", counter);
                    counter += 1;
                    ir.push_str(&format!(
                        "  {} = fptosi {} {} to i64\n",
                        converted, float_ty, value
                    ));
                    converted
                } else {
                    value
                };

                let ret_dbg = self.debug_info.dbg_ref_from_offset(expr.span.start);
                if ret_type == ResolvedType::Unit {
                    ir.push_str(&format!("  ret void{}\n", ret_dbg));
                } else if matches!(ret_type, ResolvedType::Named { .. }) {
                    // For struct returns, load the value from pointer
                    let loaded = format!("%ret.{}", counter);
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}{}\n",
                        loaded, ret_llvm, ret_llvm, value, ret_dbg
                    ));
                    ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, loaded, ret_dbg));
                } else {
                    ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, value, ret_dbg));
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
                        ir.push_str(&format!("  ret void{}\n", ret_dbg));
                    } else if f.name.node == "main"
                        && ret_type == ResolvedType::I64
                        && f.ret_type.is_none()
                        && value == "void"
                    {
                        // main() with implicit i64 return and Unit body: auto-return 0
                        ir.push_str(&format!("  ret i64 0{}\n", ret_dbg));
                    } else if matches!(ret_type, ResolvedType::Named { .. }) {
                        // Check if the result is already a value (from phi node) or a pointer (from struct lit)
                        if self.is_block_result_value(stmts) {
                            // Value (e.g., from if-else phi node) - return directly
                            ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, value, ret_dbg));
                        } else {
                            // Pointer (e.g., from struct literal) - load then return
                            let loaded = format!("%ret.{}", counter);
                            ir.push_str(&format!(
                                "  {} = load {}, {}* {}{}\n",
                                loaded, ret_llvm, ret_llvm, value, ret_dbg
                            ));
                            ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, loaded, ret_dbg));
                        }
                    } else if matches!(
                        ret_type,
                        ResolvedType::Ref(_) | ResolvedType::RefMut(_)
                    ) && !value.starts_with('%')
                        && !value.starts_with('@')
                    {
                        // Reference return with bare literal: promote to global constant
                        let inner_ty = match &ret_type {
                            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                                self.type_to_llvm(inner)
                            }
                            _ => unreachable!(),
                        };
                        let const_name =
                            format!(".ref.const.{}", self.ref_constant_counter);
                        self.ref_constant_counter += 1;
                        self.ref_constants.push((
                            const_name.clone(),
                            inner_ty,
                            value.clone(),
                        ));
                        ir.push_str(&format!(
                            "  ret {} @{}{}\n",
                            ret_llvm, const_name, ret_dbg
                        ));
                    } else {
                        ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, value, ret_dbg));
                    }
                }
            }
        }

        ir.push_str("}\n");

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
                ir.push_str(&format!("  {} = alloca {}\n", param_ptr, llvm_ty));
                ir.push_str(&format!(
                    "  store {} %{}, {}* {}\n",
                    llvm_ty, src_llvm_name, llvm_ty, param_ptr
                ));
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

                // Free tracked heap allocations before return
                let alloc_cleanup_ir = self.generate_alloc_cleanup();
                ir.push_str(&alloc_cleanup_ir);

                let ret_dbg = self.debug_info.dbg_ref_from_offset(expr.span.start);
                if ret_type == ResolvedType::Unit {
                    ir.push_str(&format!("  ret void{}\n", ret_dbg));
                } else if matches!(ret_type, ResolvedType::Named { .. }) {
                    // For struct returns, load the value from pointer
                    let loaded = format!("%ret.{}", counter);
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}{}\n",
                        loaded, ret_llvm, ret_llvm, value, ret_dbg
                    ));
                    ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, loaded, ret_dbg));
                } else if matches!(
                    ret_type,
                    ResolvedType::Ref(_) | ResolvedType::RefMut(_)
                ) && !value.starts_with('%')
                    && !value.starts_with('@')
                {
                    let inner_ty = match &ret_type {
                        ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                            self.type_to_llvm(inner)
                        }
                        _ => unreachable!(),
                    };
                    let const_name =
                        format!(".ref.const.{}", self.ref_constant_counter);
                    self.ref_constant_counter += 1;
                    self.ref_constants.push((
                        const_name.clone(),
                        inner_ty,
                        value.clone(),
                    ));
                    ir.push_str(&format!(
                        "  ret {} @{}{}\n",
                        ret_llvm, const_name, ret_dbg
                    ));
                } else {
                    ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, value, ret_dbg));
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
                    // Free tracked heap allocations before return
                    let alloc_cleanup_ir = self.generate_alloc_cleanup();
                    ir.push_str(&alloc_cleanup_ir);

                    let ret_offset = stmts.last().map(|s| s.span.end).unwrap_or(span.end);
                    let ret_dbg = self.debug_info.dbg_ref_from_offset(ret_offset);
                    if ret_type == ResolvedType::Unit {
                        ir.push_str(&format!("  ret void{}\n", ret_dbg));
                    } else if matches!(ret_type, ResolvedType::Named { .. }) {
                        // Check if the result is already a value (from phi node) or a pointer (from struct lit)
                        if self.is_block_result_value(stmts) {
                            // Value (e.g., from if-else phi node) - return directly
                            ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, value, ret_dbg));
                        } else {
                            // Pointer (e.g., from struct literal) - load then return
                            let loaded = format!("%ret.{}", counter);
                            ir.push_str(&format!(
                                "  {} = load {}, {}* {}{}\n",
                                loaded, ret_llvm, ret_llvm, value, ret_dbg
                            ));
                            ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, loaded, ret_dbg));
                        }
                    } else if matches!(
                        ret_type,
                        ResolvedType::Ref(_) | ResolvedType::RefMut(_)
                    ) && !value.starts_with('%')
                        && !value.starts_with('@')
                    {
                        let inner_ty = match &ret_type {
                            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                                self.type_to_llvm(inner)
                            }
                            _ => unreachable!(),
                        };
                        let const_name =
                            format!(".ref.const.{}", self.ref_constant_counter);
                        self.ref_constant_counter += 1;
                        self.ref_constants.push((
                            const_name.clone(),
                            inner_ty,
                            value.clone(),
                        ));
                        ir.push_str(&format!(
                            "  ret {} @{}{}\n",
                            ret_llvm, const_name, ret_dbg
                        ));
                    } else {
                        ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, value, ret_dbg));
                    }
                }
            }
        }

        ir.push_str("}\n");

        self.fn_ctx.current_function = None;
        self.fn_ctx.current_return_type = None;
        Ok(ir)
    }
}

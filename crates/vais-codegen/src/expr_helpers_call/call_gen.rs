use super::*;
use vais_ast::{Expr, Span, Spanned};
use vais_types::ResolvedType;

impl CodeGenerator {
    pub(crate) fn generate_call_expr(
        &mut self,
        func: &Spanned<Expr>,
        args: &[Spanned<Expr>],
        counter: &mut usize,
        span: Span,
    ) -> CodegenResult<(String, String)> {
        // Check if this is an enum variant constructor (e.g., Some(42))
        if let Expr::Ident(name) = &func.node {
            if let Some((enum_name, tag)) = self.get_tuple_variant_info(name) {
                return self.generate_enum_variant_constructor(&enum_name, tag, args, counter);
            }

            // Check if this is a SIMD intrinsic - do this BEFORE regular function lookup
            if Self::is_simd_intrinsic(name) {
                return self.generate_simd_intrinsic(name, args, counter);
            }

            // sizeof(expr) — compile-time constant
            if name == "sizeof" {
                let size = if !args.is_empty() {
                    let arg_type = self.infer_expr_type(&args[0]);
                    self.compute_sizeof(&arg_type)
                } else {
                    8
                };
                return Ok((size.to_string(), String::new()));
            }

            // Handle print/println builtins with format string support
            if name == "print" || name == "println" {
                return self.generate_print_call(name, args, counter, span);
            }

            // Handle print_i64/print_f64 builtins
            if name == "print_i64" && args.len() == 1 {
                let has_user_fn = self
                    .types
                    .functions
                    .get("print_i64")
                    .map(|f| !f.is_extern)
                    .unwrap_or(false);
                if !has_user_fn {
                    return self.generate_print_i64_builtin(args, counter);
                }
            }
            if name == "print_f64" && args.len() == 1 {
                let has_user_fn = self
                    .types
                    .functions
                    .get("print_f64")
                    .map(|f| !f.is_extern)
                    .unwrap_or(false);
                if !has_user_fn {
                    return self.generate_print_f64_builtin(args, counter);
                }
            }

            // Handle format builtin
            if name == "format" {
                return self.generate_format_call(args, counter, span);
            }

            // Handle str_to_ptr builtin
            if name == "str_to_ptr" {
                return self.generate_str_to_ptr_builtin(args, counter);
            }

            // Handle ptr_to_str builtin
            if name == "ptr_to_str" {
                return self.generate_ptr_to_str_builtin(args, counter);
            }
        }

        // Check if this is a direct function call or indirect (lambda) call
        let (fn_name, is_indirect) = if let Expr::Ident(name) = &func.node {
            // Check if this is a generic function that needs monomorphization
            if let Some(instantiations_list) = self.generics.fn_instantiations.get(name) {
                let arg_types: Vec<vais_types::ResolvedType> =
                    args.iter().map(|a| self.infer_expr_type(a)).collect();
                let mangled = self.resolve_generic_call(name, &arg_types, instantiations_list);
                (mangled, false)
            } else {
                // Determine indirection based on lookup, clone name once
                let is_indirect = !self.types.functions.contains_key(name)
                    && self.fn_ctx.locals.contains_key(name);
                (name.to_string(), is_indirect)
            }
        } else if let Expr::SelfCall = &func.node {
            (
                self.fn_ctx
                    .current_function
                    .as_deref()
                    .unwrap_or("")
                    .to_string(),
                false,
            ) // avoid clone unwrap_or_default
        } else {
            return Err(CodegenError::Unsupported(
                "complex indirect call".to_string(),
            ));
        };

        // Look up function info for parameter types
        let fn_info = if !is_indirect {
            self.types.functions.get(&fn_name).cloned()
        } else {
            None
        };

        let mut ir = String::new();
        let mut arg_vals = Vec::with_capacity(args.len());

        for (i, arg) in args.iter().enumerate() {
            let (mut val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);

            let param_ty = fn_info
                .as_ref()
                .and_then(|f| f.signature.params.get(i))
                .map(|(_, ty, _)| ty.clone());

            // Determine argument LLVM type - use parameter type if available, otherwise infer from expression
            let arg_ty = if let Some(ref pt) = param_ty {
                self.type_to_llvm(pt)
            } else {
                let inferred_ty = self.infer_expr_type(arg);
                self.type_to_llvm(&inferred_ty)
            };

            // Insert integer conversion if needed
            if let Some(param_type) = &param_ty {
                let src_bits = self.get_integer_bits_from_val(&val);
                let dst_bits = self.get_integer_bits(param_type);

                if src_bits > 0 && dst_bits > 0 && src_bits != dst_bits {
                    let conv_tmp = self.next_temp(counter);
                    let src_ty = format!("i{}", src_bits);
                    let dst_ty = format!("i{}", dst_bits);

                    if src_bits > dst_bits {
                        ir.push_str(&format!(
                            "  {} = trunc {} {} to {}\n",
                            conv_tmp, src_ty, val, dst_ty
                        ));
                    } else {
                        ir.push_str(&format!(
                            "  {} = sext {} {} to {}\n",
                            conv_tmp, src_ty, val, dst_ty
                        ));
                    }
                    val = conv_tmp;
                }
            }

            // For struct types, load the value from pointer if the expression produces a pointer
            // Struct literals return pointers (alloca), but function params expect values
            // This applies whether we have function info or not
            let type_to_check = match &param_ty {
                Some(ty) => ty.clone(),
                None => self.infer_expr_type(arg),
            };
            let is_named = matches!(type_to_check, ResolvedType::Named { .. });
            let is_value = self.is_expr_value(arg);

            if is_named && !is_value {
                let loaded = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = load {}, {}* {}\n",
                    loaded, arg_ty, arg_ty, val
                ));
                val = loaded;
            }

            arg_vals.push(format!("{} {}", arg_ty, val));
        }

        // Fill in default parameter values for omitted trailing arguments
        let param_count = fn_info
            .as_ref()
            .map(|f| f.signature.params.len())
            .unwrap_or(args.len());
        if args.len() < param_count {
            // Clone the default param expressions to avoid borrow issues with &mut self
            let defaults: Option<Vec<Option<Box<vais_ast::Spanned<vais_ast::Expr>>>>> =
                self.types.default_params.get(&fn_name).cloned();
            if let Some(defaults) = defaults {
                for i in args.len()..param_count {
                    if let Some(Some(default_expr)) = defaults.get(i) {
                        let param_ty = fn_info
                            .as_ref()
                            .and_then(|f| f.signature.params.get(i))
                            .map(|(_, ty, _)| ty.clone());
                        let arg_ty = if let Some(ref pt) = param_ty {
                            self.type_to_llvm(pt)
                        } else {
                            "i64".to_string()
                        };
                        let (val, default_ir) = self.generate_expr(default_expr, counter)?;
                        ir.push_str(&default_ir);
                        arg_vals.push(format!("{} {}", arg_ty, val));
                    }
                }
            }
        }

        let ret_ty = fn_info
            .as_ref()
            .map(|f| self.type_to_llvm(&f.signature.ret))
            .unwrap_or_else(|| "i64".to_string());

        let actual_fn_name = fn_info
            .as_ref()
            .map(|f| f.signature.name.as_str())
            .unwrap_or(fn_name.as_str())
            .to_string(); // single clone at end instead of two branches

        // Generate the appropriate call based on function type
        self.generate_call_ir(
            &fn_name,
            &actual_fn_name,
            is_indirect,
            &ret_ty,
            &arg_vals,
            counter,
            span,
            &mut ir,
        )
    }

    /// Generate the IR for a function call
    #[allow(clippy::too_many_arguments)]
    fn generate_call_ir(
        &mut self,
        fn_name: &str,
        actual_fn_name: &str,
        is_indirect: bool,
        ret_ty: &str,
        arg_vals: &[String],
        counter: &mut usize,
        span: Span,
        ir: &mut String,
    ) -> CodegenResult<(String, String)> {
        let dbg_info = self.debug_info.dbg_ref_from_span(span);

        if is_indirect {
            // Indirect call (lambda)
            let closure_info = self.lambdas.closures.get(fn_name).cloned();

            let mut all_args = Vec::new();
            if let Some(ref info) = closure_info {
                if info.is_ref_capture {
                    // Reference capture: pass pointers
                    for (_, capture_val) in &info.captures {
                        all_args.push(format!("i64* {}", capture_val));
                    }
                } else {
                    for (_, capture_val) in &info.captures {
                        all_args.push(format!("i64 {}", capture_val));
                    }
                }
            }
            all_args.extend(arg_vals.iter().cloned());

            // If we have closure info, we know the exact function name - call directly
            if let Some(ref info) = closure_info {
                let tmp = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call i64 @{}({}){}\n",
                    tmp,
                    info.func_name,
                    all_args.join(", "),
                    dbg_info
                ));
                return Ok((tmp, std::mem::take(ir)));
            }

            // Get the local variable info
            let local_info = self.fn_ctx.locals.get(fn_name).cloned();
            let is_ssa_or_param = local_info
                .as_ref()
                .map(|l| l.is_ssa() || l.is_param())
                .unwrap_or(false);

            let ptr_tmp = if is_ssa_or_param {
                // SSA or param: the value IS the function pointer (as i64), no load needed
                let local = match local_info.as_ref() {
                    Some(l) => l,
                    None => {
                        return Err(CodegenError::TypeError(format!(
                            "missing local info for '{}'",
                            fn_name
                        )))
                    }
                };
                let val = &local.llvm_name;
                if local.is_ssa() {
                    // SSA values already include the % prefix (e.g., "%5")
                    val.to_string() // explicit to_string instead of clone
                } else {
                    // Param names don't include % prefix
                    format!("%{}", val)
                }
            } else {
                let llvm_var_name = local_info
                    .as_ref()
                    .map(|l| l.llvm_name.as_str())
                    .unwrap_or(fn_name)
                    .to_string(); // single clone at end
                let tmp = self.next_temp(counter);
                ir.push_str(&format!("  {} = load i64, i64* %{}\n", tmp, llvm_var_name));
                tmp
            };

            let arg_types: Vec<String> = all_args
                .iter()
                .map(|a| a.split_whitespace().next().unwrap_or("i64").to_string())
                .collect();
            let fn_type = format!("i64 ({})*", arg_types.join(", "));

            let fn_ptr = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = inttoptr i64 {} to {}\n",
                fn_ptr, ptr_tmp, fn_type
            ));

            let tmp = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = call i64 {}({}){}\n",
                tmp,
                fn_ptr,
                all_args.join(", "),
                dbg_info
            ));
            Ok((tmp, std::mem::take(ir)))
        } else if fn_name == "malloc" {
            let ptr_tmp = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = call i8* @malloc({}){}\n",
                ptr_tmp,
                arg_vals.join(", "),
                dbg_info
            ));
            let result = self.next_temp(counter);
            ir.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", result, ptr_tmp));
            Ok((result, std::mem::take(ir)))
        } else if fn_name == "free" {
            let ptr_tmp = self.next_temp(counter);
            let arg_val = arg_vals
                .first()
                .map(|s| s.split_whitespace().last().unwrap_or("0"))
                .unwrap_or("0");
            ir.push_str(&format!(
                "  {} = inttoptr i64 {} to i8*\n",
                ptr_tmp, arg_val
            ));
            ir.push_str(&format!("  call void @free(i8* {}){}\n", ptr_tmp, dbg_info));
            Ok(("void".to_string(), std::mem::take(ir)))
        } else if fn_name == "memcpy" || fn_name == "memcpy_str" {
            self.generate_memcpy_call(arg_vals, counter, span, ir)
        } else if fn_name == "strlen" {
            self.generate_strlen_call(arg_vals, counter, span, ir)
        } else if fn_name == "puts_ptr" {
            self.generate_puts_ptr_call(arg_vals, counter, span, ir)
        } else if ret_ty == "void" {
            let is_vararg = self
                .types
                .functions
                .get(fn_name)
                .map(|f| f.signature.is_vararg)
                .unwrap_or(false);
            if is_vararg {
                let param_types: Vec<String> = self
                    .types
                    .functions
                    .get(fn_name)
                    .map(|f| {
                        f.signature
                            .params
                            .iter()
                            .map(|(_, ty, _)| self.type_to_llvm(ty))
                            .collect()
                    })
                    .unwrap_or_default();
                let sig = format!("void ({}, ...)", param_types.join(", "));
                ir.push_str(&format!(
                    "  call {} @{}({}){}\n",
                    sig,
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                ));
            } else {
                ir.push_str(&format!(
                    "  call void @{}({}){}\n",
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                ));
            }
            Ok(("void".to_string(), std::mem::take(ir)))
        } else {
            let is_vararg = self
                .types
                .functions
                .get(fn_name)
                .map(|f| f.signature.is_vararg)
                .unwrap_or(false);
            let tmp = self.next_temp(counter);
            if is_vararg {
                let param_types: Vec<String> = self
                    .types
                    .functions
                    .get(fn_name)
                    .map(|f| {
                        f.signature
                            .params
                            .iter()
                            .map(|(_, ty, _)| self.type_to_llvm(ty))
                            .collect()
                    })
                    .unwrap_or_default();
                let sig = format!("{} ({}, ...)", ret_ty, param_types.join(", "));
                ir.push_str(&format!(
                    "  {} = call {} @{}({}){}\n",
                    tmp,
                    sig,
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                ));
            } else {
                ir.push_str(&format!(
                    "  {} = call {} @{}({}){}\n",
                    tmp,
                    ret_ty,
                    actual_fn_name,
                    arg_vals.join(", "),
                    dbg_info
                ));
            }
            Ok((tmp, std::mem::take(ir)))
        }
    }

    /// Generate enum variant constructor
    fn generate_enum_variant_constructor(
        &mut self,
        enum_name: &str,
        tag: i32,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();
        let mut arg_vals = Vec::with_capacity(args.len());

        for arg in args {
            let (val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);
            arg_vals.push(val);
        }

        let enum_ptr = self.next_temp(counter);
        ir.push_str(&format!("  {} = alloca %{}\n", enum_ptr, enum_name));

        let tag_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0\n",
            tag_ptr, enum_name, enum_name, enum_ptr
        ));
        ir.push_str(&format!("  store i32 {}, i32* {}\n", tag, tag_ptr));

        // Store payload fields
        for (i, arg_val) in arg_vals.iter().enumerate() {
            let payload_field_ptr = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = getelementptr %{}, %{}* {}, i32 0, i32 1, i32 {}\n",
                payload_field_ptr, enum_name, enum_name, enum_ptr, i
            ));
            ir.push_str(&format!(
                "  store i64 {}, i64* {}\n",
                arg_val, payload_field_ptr
            ));
        }

        Ok((enum_ptr, ir))
    }
}

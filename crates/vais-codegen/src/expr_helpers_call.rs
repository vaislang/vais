//! Call expression helpers for CodeGenerator
//!
//! Contains function call generation, print/format builtins,
//! method calls, and pointer conversion helpers.

use crate::{CodeGenerator, CodegenError, CodegenResult};
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
                    .types.functions
                    .get("print_i64")
                    .map(|f| !f.is_extern)
                    .unwrap_or(false);
                if !has_user_fn {
                    return self.generate_print_i64_builtin(args, counter);
                }
            }
            if name == "print_f64" && args.len() == 1 {
                let has_user_fn = self
                    .types.functions
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
            (self.fn_ctx.current_function.as_deref().unwrap_or("").to_string(), false) // avoid clone unwrap_or_default
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
        let mut arg_vals = Vec::new();

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
            let type_to_check = param_ty
                .as_ref()
                .cloned()
                .unwrap_or_else(|| self.infer_expr_type(arg));
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
                for (_, capture_val) in &info.captures {
                    all_args.push(format!("i64 {}", capture_val));
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
                .types.functions
                .get(fn_name)
                .map(|f| f.signature.is_vararg)
                .unwrap_or(false);
            if is_vararg {
                let param_types: Vec<String> = self
                    .types.functions
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
                .types.functions
                .get(fn_name)
                .map(|f| f.signature.is_vararg)
                .unwrap_or(false);
            let tmp = self.next_temp(counter);
            if is_vararg {
                let param_types: Vec<String> = self
                    .types.functions
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
        let mut arg_vals = Vec::new();

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

    /// Generate memcpy call
    fn generate_memcpy_call(
        &mut self,
        arg_vals: &[String],
        counter: &mut usize,
        span: Span,
        ir: &mut String,
    ) -> CodegenResult<(String, String)> {
        let dbg_info = self.debug_info.dbg_ref_from_span(span);
        let dest_full = arg_vals.first().map(|s| s.as_str()).unwrap_or("i64 0");
        let src_full = arg_vals.get(1).map(|s| s.as_str()).unwrap_or("i64 0");
        let n_val = arg_vals
            .get(2)
            .map(|s| s.split_whitespace().last().unwrap_or("0"))
            .unwrap_or("0");

        // Handle dest pointer
        let dest_ptr = if dest_full.starts_with("i8*") {
            // Use everything after "i8* " to preserve complex expressions like getelementptr
            dest_full
                .strip_prefix("i8* ")
                .unwrap_or(dest_full.split_whitespace().last().unwrap_or("null"))
                .to_string()
        } else {
            let dest_val = dest_full.split_whitespace().last().unwrap_or("0");
            let ptr = self.next_temp(counter);
            ir.push_str(&format!("  {} = inttoptr i64 {} to i8*\n", ptr, dest_val));
            ptr
        };

        // Handle src pointer (can be i64 or i8* for memcpy_str)
        let src_ptr = if src_full.starts_with("i8*") {
            // Use everything after "i8* " to preserve complex expressions like getelementptr
            src_full
                .strip_prefix("i8* ")
                .unwrap_or(src_full.split_whitespace().last().unwrap_or("null"))
                .to_string()
        } else {
            let src_val = src_full.split_whitespace().last().unwrap_or("0");
            let ptr = self.next_temp(counter);
            ir.push_str(&format!("  {} = inttoptr i64 {} to i8*\n", ptr, src_val));
            ptr
        };

        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i8* @memcpy(i8* {}, i8* {}, i64 {}){}\n",
            result, dest_ptr, src_ptr, n_val, dbg_info
        ));
        let result_i64 = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = ptrtoint i8* {} to i64\n",
            result_i64, result
        ));
        Ok((result_i64, std::mem::take(ir)))
    }

    /// Generate strlen call
    fn generate_strlen_call(
        &mut self,
        arg_vals: &[String],
        counter: &mut usize,
        span: Span,
        ir: &mut String,
    ) -> CodegenResult<(String, String)> {
        let dbg_info = self.debug_info.dbg_ref_from_span(span);
        let arg_full = arg_vals.first().map(|s| s.as_str()).unwrap_or("i64 0");
        let result = self.next_temp(counter);

        // Check if the argument is already i8* (str type) or i64 (pointer as integer)
        if arg_full.starts_with("i8*") {
            // Already a pointer, use directly
            let ptr_val = arg_full.split_whitespace().last().unwrap_or("null");
            ir.push_str(&format!(
                "  {} = call i64 @strlen(i8* {}){}\n",
                result, ptr_val, dbg_info
            ));
        } else {
            // Convert i64 to pointer
            let arg_val = arg_full.split_whitespace().last().unwrap_or("0");
            let ptr_tmp = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = inttoptr i64 {} to i8*\n",
                ptr_tmp, arg_val
            ));
            ir.push_str(&format!(
                "  {} = call i64 @strlen(i8* {}){}\n",
                result, ptr_tmp, dbg_info
            ));
        }
        Ok((result, std::mem::take(ir)))
    }

    /// Generate puts_ptr call
    fn generate_puts_ptr_call(
        &mut self,
        arg_vals: &[String],
        counter: &mut usize,
        span: Span,
        ir: &mut String,
    ) -> CodegenResult<(String, String)> {
        let dbg_info = self.debug_info.dbg_ref_from_span(span);
        let arg_val = arg_vals
            .first()
            .map(|s| s.split_whitespace().last().unwrap_or("0"))
            .unwrap_or("0");
        let ptr_tmp = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = inttoptr i64 {} to i8*\n",
            ptr_tmp, arg_val
        ));
        let i32_result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i32 @puts(i8* {}){}\n",
            i32_result, ptr_tmp, dbg_info
        ));
        // Convert i32 result to i64 for consistency
        let result = self.next_temp(counter);
        ir.push_str(&format!("  {} = sext i32 {} to i64\n", result, i32_result));
        Ok((result, std::mem::take(ir)))
    }

    /// Generate print/println call with format string support
    ///
    /// Converts Vais format strings like `print("x = {}", x)` to printf calls.
    /// `{}` placeholders are replaced with the appropriate C format specifier
    /// based on the inferred type of each argument.
    pub(crate) fn generate_print_call(
        &mut self,
        fn_name: &str,
        args: &[Spanned<Expr>],
        counter: &mut usize,
        span: Span,
    ) -> CodegenResult<(String, String)> {
        let dbg_info = self.debug_info.dbg_ref_from_span(span);
        let mut ir = String::new();

        if args.is_empty() {
            // print() with no args: do nothing
            return Ok(("void".to_string(), ir));
        }

        // If first arg is a StringInterp, flatten it into format string + args
        if let Expr::StringInterp(parts) = &args[0].node {
            let mut fmt_parts = Vec::new();
            let mut interp_args = Vec::new();
            for part in parts {
                match part {
                    vais_ast::StringInterpPart::Lit(s) => {
                        fmt_parts.push(s.clone());
                    }
                    vais_ast::StringInterpPart::Expr(e) => {
                        fmt_parts.push("{}".to_string());
                        interp_args.push(e.as_ref().clone());
                    }
                }
            }
            let fmt_string = fmt_parts.join("");
            let mut synthetic_args: Vec<Spanned<Expr>> = Vec::new();
            synthetic_args.push(Spanned::new(Expr::String(fmt_string), args[0].span));
            synthetic_args.extend(interp_args);
            // Also include any additional args after the interpolated string
            synthetic_args.extend_from_slice(&args[1..]);
            return self.generate_print_call(fn_name, &synthetic_args, counter, span);
        }

        // First argument must be a string literal (format string)
        let format_str = match &args[0].node {
            Expr::String(s) => s.clone(),
            _ => {
                // Non-literal first arg: treat as simple string output
                let (val, val_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&val_ir);
                if fn_name == "println" {
                    // For println with non-literal, use puts
                    let i32_result = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = call i32 @puts(i8* {}){}\n",
                        i32_result, val, dbg_info
                    ));
                } else {
                    // For print with non-literal, use printf with %s
                    let fmt_name = self.make_string_name();
                    self.strings.counter += 1;
                    self.strings.constants
                        .push((fmt_name.clone(), "%s".to_string()));
                    let fmt_len = 3; // "%s" + null
                    let fmt_ptr = format!(
                        "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                        fmt_len, fmt_len, fmt_name
                    );
                    let _result = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = call i32 (i8*, ...) @printf(i8* {}, i8* {}){}\n",
                        _result, fmt_ptr, val, dbg_info
                    ));
                }
                return Ok(("void".to_string(), ir));
            }
        };

        // Infer types of extra arguments (skip first format string arg)
        let extra_args = &args[1..];
        let mut arg_types: Vec<ResolvedType> = Vec::new();
        for arg in extra_args {
            arg_types.push(self.infer_expr_type(arg));
        }

        // Convert Vais format string `{}` to C printf format specifiers
        let mut c_format = String::new();
        let mut arg_idx = 0;
        let mut chars = format_str.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                if chars.peek() == Some(&'{') {
                    // Escaped {{ -> literal {
                    chars.next();
                    c_format.push('{');
                } else if chars.peek() == Some(&'}') {
                    // {} -> format specifier based on type
                    chars.next();
                    if arg_idx < arg_types.len() {
                        let spec = match &arg_types[arg_idx] {
                            ResolvedType::I32 => "%d",
                            ResolvedType::I64 => "%ld",
                            ResolvedType::F32 | ResolvedType::F64 => "%f",
                            ResolvedType::Str => "%s",
                            ResolvedType::Bool => "%ld", // bools are i64 in codegen
                            // Char type not yet in Vais
                            _ => "%ld", // default to i64
                        };
                        c_format.push_str(spec);
                        arg_idx += 1;
                    } else {
                        return Err(CodegenError::TypeError(
                            "Too few arguments for format string".to_string(),
                        ));
                    }
                } else {
                    c_format.push(ch);
                }
            } else if ch == '}' {
                if chars.peek() == Some(&'}') {
                    // Escaped }} -> literal }
                    chars.next();
                    c_format.push('}');
                } else {
                    c_format.push(ch);
                }
            } else {
                c_format.push(ch);
            }
        }

        // For println, append newline
        if fn_name == "println" {
            c_format.push('\n');
        }

        // Create global string constant for the C format string
        let fmt_name = self.make_string_name();
        self.strings.counter += 1;
        self.strings.constants
            .push((fmt_name.clone(), c_format.clone()));
        let fmt_len = c_format.len() + 1; // +1 for null terminator
        let fmt_ptr = format!(
            "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
            fmt_len, fmt_len, fmt_name
        );

        // Evaluate extra arguments and build printf call
        let mut printf_args = vec![format!("i8* {}", fmt_ptr)];

        for (i, arg) in extra_args.iter().enumerate() {
            let (val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);

            let llvm_ty = match &arg_types[i] {
                ResolvedType::I32 => "i32",
                ResolvedType::F32 => "float",
                ResolvedType::F64 => "double",
                ResolvedType::Str => "i8*",
                // Char type not yet in Vais
                _ => "i64", // i64, bool, etc.
            };

            // For i32 params, we may need to truncate from i64
            if llvm_ty == "i32" {
                let trunc_tmp = self.next_temp(counter);
                ir.push_str(&format!("  {} = trunc i64 {} to i32\n", trunc_tmp, val));
                printf_args.push(format!("i32 {}", trunc_tmp));
            } else {
                printf_args.push(format!("{} {}", llvm_ty, val));
            }
        }

        // If no extra args and println, use puts for efficiency
        if extra_args.is_empty() && fn_name == "println" {
            // Remove the printf format string (with \n) we added above
            self.strings.constants.pop();
            self.strings.counter -= 1;
            // Create puts string (without trailing \n, since puts adds one)
            let puts_str = &c_format[..c_format.len() - 1];
            let puts_name = self.make_string_name();
            self.strings.counter += 1;
            self.strings.constants
                .push((puts_name.clone(), puts_str.to_string()));
            let puts_len = puts_str.len() + 1;
            let puts_ptr = format!(
                "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                puts_len, puts_len, puts_name
            );
            let _result = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = call i32 @puts(i8* {}){}\n",
                _result, puts_ptr, dbg_info
            ));
            return Ok(("void".to_string(), ir));
        }

        // For print with no extra args, use printf with just the format string
        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i32 (i8*, ...) @printf({}){}\n",
            result,
            printf_args.join(", "),
            dbg_info
        ));

        Ok(("void".to_string(), ir))
    }

    /// Generate format() call - returns allocated formatted string (i8*)
    ///
    /// Uses snprintf(NULL, 0, fmt, ...) to measure, malloc to allocate,
    /// then snprintf(buf, len+1, fmt, ...) to write.
    pub(crate) fn generate_format_call(
        &mut self,
        args: &[Spanned<Expr>],
        counter: &mut usize,
        span: Span,
    ) -> CodegenResult<(String, String)> {
        let _dbg_info = self.debug_info.dbg_ref_from_span(span);
        let mut ir = String::new();

        if args.is_empty() {
            return Err(CodegenError::TypeError(
                "format() requires at least a format string argument".to_string(),
            ));
        }

        // First argument must be a string literal (format string)
        let format_str = match &args[0].node {
            Expr::String(s) => s.clone(),
            _ => {
                // Non-literal: just return the string as-is
                let (val, val_ir) = self.generate_expr(&args[0], counter)?;
                return Ok((val, val_ir));
            }
        };

        // Infer types of extra arguments
        let extra_args = &args[1..];
        let mut arg_types: Vec<ResolvedType> = Vec::new();
        for arg in extra_args {
            arg_types.push(self.infer_expr_type(arg));
        }

        // Convert Vais format string `{}` to C printf format specifiers
        let mut c_format = String::new();
        let mut arg_idx = 0;
        let mut chars = format_str.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    c_format.push('{');
                } else if chars.peek() == Some(&'}') {
                    chars.next();
                    if arg_idx < arg_types.len() {
                        let spec = match &arg_types[arg_idx] {
                            ResolvedType::I32 => "%d",
                            ResolvedType::I64 => "%ld",
                            ResolvedType::F32 | ResolvedType::F64 => "%f",
                            ResolvedType::Str => "%s",
                            ResolvedType::Bool => "%ld",
                            _ => "%ld",
                        };
                        c_format.push_str(spec);
                        arg_idx += 1;
                    } else {
                        return Err(CodegenError::TypeError(
                            "Too few arguments for format string".to_string(),
                        ));
                    }
                } else {
                    c_format.push(ch);
                }
            } else if ch == '}' {
                if chars.peek() == Some(&'}') {
                    chars.next();
                    c_format.push('}');
                } else {
                    c_format.push(ch);
                }
            } else {
                c_format.push(ch);
            }
        }

        // Create global string constant for the C format string
        let fmt_name = self.make_string_name();
        self.strings.counter += 1;
        self.strings.constants
            .push((fmt_name.clone(), c_format.clone()));
        let fmt_len = c_format.len() + 1;
        let fmt_ptr = format!(
            "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
            fmt_len, fmt_len, fmt_name
        );

        // Evaluate extra arguments
        let mut snprintf_args = String::new();
        let mut arg_vals: Vec<String> = Vec::new();
        for (i, arg) in extra_args.iter().enumerate() {
            let (val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);

            let llvm_ty = match &arg_types[i] {
                ResolvedType::I32 => "i32",
                ResolvedType::F32 => "float",
                ResolvedType::F64 => "double",
                ResolvedType::Str => "i8*",
                _ => "i64",
            };

            if llvm_ty == "i32" {
                let trunc_tmp = self.next_temp(counter);
                ir.push_str(&format!("  {} = trunc i64 {} to i32\n", trunc_tmp, val));
                arg_vals.push(format!("i32 {}", trunc_tmp));
            } else {
                arg_vals.push(format!("{} {}", llvm_ty, val));
            }
        }

        if !arg_vals.is_empty() {
            snprintf_args = format!(", {}", arg_vals.join(", "));
        }

        // If no extra args, just return the format string directly
        if extra_args.is_empty() {
            let str_name = format!(".str.{}", self.strings.counter - 1);
            // Already pushed the constant above, reuse it
            let ptr = format!(
                "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                fmt_len, fmt_len, str_name
            );
            return Ok((ptr, ir));
        }

        // Step 1: snprintf(NULL, 0, fmt, ...) to get required length
        let len_i32 = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i32 (i8*, i64, i8*, ...) @snprintf(i8* null, i64 0, i8* {}{})\n",
            len_i32, fmt_ptr, snprintf_args
        ));

        // Convert i32 length to i64
        let len_i64 = self.next_temp(counter);
        ir.push_str(&format!("  {} = sext i32 {} to i64\n", len_i64, len_i32));

        // Step 2: malloc(len + 1)
        let buf_size = self.next_temp(counter);
        ir.push_str(&format!("  {} = add i64 {}, 1\n", buf_size, len_i64));

        let buf_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i8* @malloc(i64 {})\n",
            buf_ptr, buf_size
        ));

        // Step 3: snprintf(buf, len+1, fmt, ...)
        let _write_result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i32 (i8*, i64, i8*, ...) @snprintf(i8* {}, i64 {}, i8* {}{})\n",
            _write_result, buf_ptr, buf_size, fmt_ptr, snprintf_args
        ));

        // Mark that we need snprintf declaration
        self.needs_string_helpers = true;

        Ok((buf_ptr, ir))
    }

    /// Generate if expression
    pub(crate) fn generate_method_call_expr(
        &mut self,
        receiver: &Spanned<Expr>,
        method: &Spanned<String>,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (recv_val, recv_ir, recv_type) = if matches!(&receiver.node, Expr::SelfCall) {
            if let Some(local) = self.fn_ctx.locals.get("self") {
                let recv_type = local.ty.clone();
                ("%self".to_string(), String::new(), recv_type)
            } else {
                return Err(CodegenError::Unsupported(
                    "@.method() used outside of a method with self".to_string(),
                ));
            }
        } else {
            let (recv_val, recv_ir) = self.generate_expr(receiver, counter)?;
            let recv_type = self.infer_expr_type(receiver);
            (recv_val, recv_ir, recv_type)
        };
        let mut ir = recv_ir;

        let method_name = &method.node;

        // String method calls: str.len(), str.charAt(), str.contains(), etc.
        if matches!(recv_type, ResolvedType::Str) {
            return self.generate_string_method_call(&recv_val, &ir, method_name, args, counter);
        }

        // Use resolve_struct_name to match definition naming (e.g., Pair → Pair$i64)
        // For non-generic structs, this is a no-op (Vec → Vec)
        let full_method_name = if let ResolvedType::Named { name, .. } = &recv_type {
            let resolved = self.resolve_struct_name(name);
            format!("{}_{}", resolved, method_name)
        } else {
            method_name.clone()
        };

        let recv_llvm_ty = if matches!(&recv_type, ResolvedType::Named { .. }) {
            format!("{}*", self.type_to_llvm(&recv_type))
        } else {
            self.type_to_llvm(&recv_type)
        };

        let mut arg_vals = vec![format!("{} {}", recv_llvm_ty, recv_val)];

        for arg in args {
            let (val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);
            let arg_type = self.infer_expr_type(arg);
            let arg_llvm_ty = self.type_to_llvm(&arg_type);
            arg_vals.push(format!("{} {}", arg_llvm_ty, val));
        }

        let ret_type = if let ResolvedType::Named { name, .. } = &recv_type {
            if let Some(_struct_info) = self.types.structs.get(name) {
                "i64"
            } else {
                "i64"
            }
        } else {
            "i64"
        };

        let tmp = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call {} @{}({})\n",
            tmp,
            ret_type,
            full_method_name,
            arg_vals.join(", ")
        ));

        Ok((tmp, ir))
    }

    /// Generate static method call expression
    pub(crate) fn generate_static_method_call_expr(
        &mut self,
        type_name: &Spanned<String>,
        method: &Spanned<String>,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();

        let full_method_name = format!("{}_{}", type_name.node, method.node);

        let mut arg_vals = Vec::new();
        for arg in args {
            let (val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);
            let arg_type = self.infer_expr_type(arg);
            let arg_llvm_ty = self.type_to_llvm(&arg_type);
            arg_vals.push(format!("{} {}", arg_llvm_ty, val));
        }

        let ret_type = self
            .types.functions
            .get(&full_method_name)
            .map(|info| self.type_to_llvm(&info.signature.ret))
            .unwrap_or_else(|| "i64".to_string());

        let tmp = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call {} @{}({})\n",
            tmp,
            ret_type,
            full_method_name,
            arg_vals.join(", ")
        ));

        Ok((tmp, ir))
    }

    /// Generate await expression
    pub(crate) fn generate_print_i64_builtin(
        &mut self,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (arg_val, arg_ir) = self.generate_expr(&args[0], counter)?;
        let mut ir = arg_ir;
        let fmt_str = "%ld";
        let fmt_name = self.make_string_name();
        self.strings.counter += 1;
        self.strings.constants
            .push((fmt_name.clone(), fmt_str.to_string()));
        let fmt_len = fmt_str.len() + 1;
        let fmt_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr [{} x i8], [{} x i8]* @{}, i64 0, i64 0\n",
            fmt_ptr, fmt_len, fmt_len, fmt_name
        ));
        let i32_result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i32 (i8*, ...) @printf(i8* {}, i64 {})\n",
            i32_result, fmt_ptr, arg_val
        ));
        let result = self.next_temp(counter);
        ir.push_str(&format!("  {} = sext i32 {} to i64\n", result, i32_result));
        Ok((result, ir))
    }

    /// Generate print_f64 builtin call
    pub(crate) fn generate_print_f64_builtin(
        &mut self,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (arg_val, arg_ir) = self.generate_expr(&args[0], counter)?;
        let mut ir = arg_ir;
        let fmt_str = "%f";
        let fmt_name = self.make_string_name();
        self.strings.counter += 1;
        self.strings.constants
            .push((fmt_name.clone(), fmt_str.to_string()));
        let fmt_len = fmt_str.len() + 1;
        let fmt_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr [{} x i8], [{} x i8]* @{}, i64 0, i64 0\n",
            fmt_ptr, fmt_len, fmt_len, fmt_name
        ));
        let i32_result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i32 (i8*, ...) @printf(i8* {}, double {})\n",
            i32_result, fmt_ptr, arg_val
        ));
        let result = self.next_temp(counter);
        ir.push_str(&format!("  {} = sext i32 {} to i64\n", result, i32_result));
        Ok((result, ir))
    }

    /// Generate str_to_ptr builtin call
    fn generate_str_to_ptr_builtin(
        &mut self,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        if args.len() != 1 {
            return Err(CodegenError::TypeError(
                "str_to_ptr expects 1 argument".to_string(),
            ));
        }
        let (str_val, str_ir) = self.generate_expr(&args[0], counter)?;
        let mut ir = str_ir;
        let result = self.next_temp(counter);
        ir.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", result, str_val));
        Ok((result, ir))
    }

    /// Generate ptr_to_str builtin call
    fn generate_ptr_to_str_builtin(
        &mut self,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        if args.len() != 1 {
            return Err(CodegenError::TypeError(
                "ptr_to_str expects 1 argument".to_string(),
            ));
        }
        let (ptr_val, ptr_ir) = self.generate_expr(&args[0], counter)?;
        let mut ir = ptr_ir;
        let arg_type = self.infer_expr_type(&args[0]);
        if matches!(arg_type, vais_types::ResolvedType::I64) {
            let result = self.next_temp(counter);
            ir.push_str(&format!("  {} = inttoptr i64 {} to i8*\n", result, ptr_val));
            return Ok((result, ir));
        }
        // Already a pointer type, no conversion needed
        Ok((ptr_val, ir))
    }
}

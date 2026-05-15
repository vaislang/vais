use super::*;
use vais_ast::{Expr, Span, Spanned};
use vais_types::ResolvedType;

impl CodeGenerator {
    fn is_printf_string_arg_type(ty: &ResolvedType) -> bool {
        match ty {
            ResolvedType::Str => true,
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                matches!(inner.as_ref(), ResolvedType::Str)
            }
            _ => false,
        }
    }

    fn lower_aggregate_format_arg_to_i64(
        &mut self,
        val: &str,
        ty: &ResolvedType,
        counter: &mut usize,
        ir: &mut String,
    ) -> Option<String> {
        let named_ty = match ty {
            ResolvedType::Named { .. } => ty,
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner)
                if matches!(inner.as_ref(), ResolvedType::Named { .. }) =>
            {
                inner.as_ref()
            }
            _ => return None,
        };
        let ResolvedType::Named { name, .. } = named_ty else {
            return None;
        };
        let enum_lookup = name.split_once('$').map(|(base, _)| base).unwrap_or(name);
        let is_enum =
            self.types.enums.contains_key(enum_lookup) || self.types.enums.contains_key(name);
        let llvm_ty = self.type_to_llvm(named_ty);
        let actual = self.llvm_type_of_checked(val);
        let pointer_actual = actual
            .as_deref()
            .is_some_and(|actual| actual == "ptr" || actual == format!("{}*", llvm_ty));

        if is_enum && pointer_actual {
            let tag_ptr = self.next_temp(counter);
            match actual.as_deref() {
                Some("ptr") => {
                    write_ir!(
                        ir,
                        "  {} = getelementptr {}, ptr {}, i32 0, i32 0",
                        tag_ptr,
                        llvm_ty,
                        val
                    );
                }
                _ => {
                    write_ir!(
                        ir,
                        "  {} = getelementptr {}, {}* {}, i32 0, i32 0",
                        tag_ptr,
                        llvm_ty,
                        llvm_ty,
                        val
                    );
                }
            }
            self.fn_ctx.record_emitted_type(&tag_ptr, "i32*");
            let tag_i32 = self.next_temp(counter);
            write_ir!(ir, "  {} = load i32, i32* {}", tag_i32, tag_ptr);
            self.fn_ctx.record_emitted_type(&tag_i32, "i32");
            let tag_i64 = self.next_temp(counter);
            write_ir!(ir, "  {} = zext i32 {} to i64", tag_i64, tag_i32);
            self.fn_ctx.record_emitted_type(&tag_i64, "i64");
            return Some(tag_i64);
        }

        if is_enum && actual.as_deref() == Some(llvm_ty.as_str()) {
            let tag_i32 = self.next_temp(counter);
            write_ir!(ir, "  {} = extractvalue {} {}, 0", tag_i32, llvm_ty, val);
            self.fn_ctx.record_emitted_type(&tag_i32, "i32");
            let tag_i64 = self.next_temp(counter);
            write_ir!(ir, "  {} = zext i32 {} to i64", tag_i64, tag_i32);
            self.fn_ctx.record_emitted_type(&tag_i64, "i64");
            return Some(tag_i64);
        }

        if pointer_actual {
            let ptr_i64 = self.next_temp(counter);
            let ptr_ty = actual.unwrap_or_else(|| format!("{}*", llvm_ty));
            write_ir!(ir, "  {} = ptrtoint {} {} to i64", ptr_i64, ptr_ty, val);
            self.fn_ctx.record_emitted_type(&ptr_i64, "i64");
            return Some(ptr_i64);
        }

        None
    }

    /// Generate print/println call with format string support
    ///
    /// Converts Vais format strings like `print("x = {}", x)` to printf calls.
    /// `{}` placeholders are replaced with the appropriate C format specifier
    /// based on the inferred type of each argument.
    #[inline(never)]
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
            let mut fmt_parts = Vec::with_capacity(parts.len());
            let mut interp_args = Vec::with_capacity(parts.len());
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
            let mut synthetic_args: Vec<Spanned<Expr>> = Vec::with_capacity(interp_args.len() + 1);
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
                // Extract raw i8* pointer from string fat pointer for C interop
                let raw_ptr = self.extract_str_ptr(&val, counter, &mut ir);
                if fn_name == "println" {
                    // For println with non-literal, use puts
                    let i32_result = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = call i32 @puts(i8* {}){}",
                        i32_result,
                        raw_ptr,
                        dbg_info
                    );
                } else {
                    // For print with non-literal, use printf with %s
                    let fmt_name = self.make_string_name();
                    self.strings.counter += 1;
                    self.strings
                        .constants
                        .push((fmt_name.clone(), "%s".to_string()));
                    let fmt_len = 3; // "%s" + null
                    let fmt_ptr = format!(
                        "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                        fmt_len, fmt_len, fmt_name
                    );
                    let _result = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = call i32 (i8*, ...) @printf(i8* {}, i8* {}){}",
                        _result,
                        fmt_ptr,
                        raw_ptr,
                        dbg_info
                    );
                }
                return Ok(("void".to_string(), ir));
            }
        };

        // Infer types of extra arguments (skip first format string arg)
        let extra_args = &args[1..];
        let mut arg_types: Vec<ResolvedType> = Vec::with_capacity(extra_args.len());
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
                            ResolvedType::I8 | ResolvedType::I16 | ResolvedType::I32 => "%d",
                            ResolvedType::U8 | ResolvedType::U16 | ResolvedType::U32 => "%u",
                            ResolvedType::I64 | ResolvedType::I128 => "%ld",
                            ResolvedType::U64 | ResolvedType::U128 => "%lu",
                            ResolvedType::F32 | ResolvedType::F64 => "%f",
                            ty if Self::is_printf_string_arg_type(ty) => "%s",
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
        self.strings
            .constants
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

            match &arg_types[i] {
                ResolvedType::I8 | ResolvedType::I16 => {
                    // Small signed integers: sext to i32 for vararg ABI
                    let ir_type = self.type_to_llvm(&arg_types[i]);
                    let ext_tmp = self.next_temp(counter);
                    write_ir!(ir, "  {} = sext {} {} to i32", ext_tmp, ir_type, val);
                    self.fn_ctx.record_emitted_type(&ext_tmp, "i32");
                    printf_args.push(format!("i32 {}", ext_tmp));
                }
                ResolvedType::U8 | ResolvedType::U16 => {
                    // Small unsigned integers: zext to i32 for vararg ABI
                    let ir_type = self.type_to_llvm(&arg_types[i]);
                    let ext_tmp = self.next_temp(counter);
                    write_ir!(ir, "  {} = zext {} {} to i32", ext_tmp, ir_type, val);
                    self.fn_ctx.record_emitted_type(&ext_tmp, "i32");
                    printf_args.push(format!("i32 {}", ext_tmp));
                }
                ResolvedType::I32 | ResolvedType::U32 => {
                    // Already i32 in LLVM IR — pass directly
                    printf_args.push(format!("i32 {}", val));
                }
                ResolvedType::Bool => {
                    // i1 → zext to i64 for vararg ABI
                    let ext_tmp = self.next_temp(counter);
                    write_ir!(ir, "  {} = zext i1 {} to i64", ext_tmp, val);
                    self.fn_ctx.record_emitted_type(&ext_tmp, "i64");
                    printf_args.push(format!("i64 {}", ext_tmp));
                }
                ResolvedType::F32 => {
                    printf_args.push(format!("float {}", val));
                }
                ResolvedType::F64 => {
                    printf_args.push(format!("double {}", val));
                }
                ty if Self::is_printf_string_arg_type(ty) => {
                    // Extract raw i8* pointer from string fat pointer for printf
                    let raw_ptr = self.extract_str_ptr(&val, counter, &mut ir);
                    printf_args.push(format!("i8* {}", raw_ptr));
                }
                _ => {
                    let lowered = self
                        .lower_aggregate_format_arg_to_i64(&val, &arg_types[i], counter, &mut ir)
                        .unwrap_or(val);
                    printf_args.push(format!("i64 {}", lowered));
                }
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
            self.strings
                .constants
                .push((puts_name.clone(), puts_str.to_string()));
            let puts_len = puts_str.len() + 1;
            let puts_ptr = format!(
                "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                puts_len, puts_len, puts_name
            );
            let _result = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = call i32 @puts(i8* {}){}",
                _result,
                puts_ptr,
                dbg_info
            );
            return Ok(("void".to_string(), ir));
        }

        // For print with no extra args, use printf with just the format string
        let result = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = call i32 (i8*, ...) @printf({}){}",
            result,
            printf_args.join(", "),
            dbg_info
        );

        Ok(("void".to_string(), ir))
    }

    /// Generate format() call - returns allocated formatted string (i8*)
    ///
    /// Uses snprintf(NULL, 0, fmt, ...) to measure, malloc to allocate,
    /// then snprintf(buf, len+1, fmt, ...) to write.
    #[inline(never)]
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
        let mut arg_types: Vec<ResolvedType> = Vec::with_capacity(extra_args.len());
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
                            ResolvedType::I8 | ResolvedType::I16 | ResolvedType::I32 => "%d",
                            ResolvedType::U8 | ResolvedType::U16 | ResolvedType::U32 => "%u",
                            ResolvedType::I64 | ResolvedType::I128 => "%ld",
                            ResolvedType::U64 | ResolvedType::U128 => "%lu",
                            ResolvedType::F32 | ResolvedType::F64 => "%f",
                            ty if Self::is_printf_string_arg_type(ty) => "%s",
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
        self.strings
            .constants
            .push((fmt_name.clone(), c_format.clone()));
        let fmt_len = c_format.len() + 1;
        let fmt_ptr = format!(
            "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
            fmt_len, fmt_len, fmt_name
        );

        // Evaluate extra arguments
        let mut snprintf_args = String::new();
        let mut arg_vals: Vec<String> = Vec::with_capacity(extra_args.len());
        for (i, arg) in extra_args.iter().enumerate() {
            let (val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);

            match &arg_types[i] {
                ResolvedType::I8 | ResolvedType::I16 => {
                    // Small signed integers: sext to i32 for vararg ABI
                    let ir_type = self.type_to_llvm(&arg_types[i]);
                    let ext_tmp = self.next_temp(counter);
                    write_ir!(ir, "  {} = sext {} {} to i32", ext_tmp, ir_type, val);
                    self.fn_ctx.record_emitted_type(&ext_tmp, "i32");
                    arg_vals.push(format!("i32 {}", ext_tmp));
                }
                ResolvedType::U8 | ResolvedType::U16 => {
                    // Small unsigned integers: zext to i32 for vararg ABI
                    let ir_type = self.type_to_llvm(&arg_types[i]);
                    let ext_tmp = self.next_temp(counter);
                    write_ir!(ir, "  {} = zext {} {} to i32", ext_tmp, ir_type, val);
                    self.fn_ctx.record_emitted_type(&ext_tmp, "i32");
                    arg_vals.push(format!("i32 {}", ext_tmp));
                }
                ResolvedType::I32 | ResolvedType::U32 => {
                    // Already i32 in LLVM IR — pass directly
                    arg_vals.push(format!("i32 {}", val));
                }
                ResolvedType::Bool => {
                    // i1 → zext to i64 for vararg ABI
                    let ext_tmp = self.next_temp(counter);
                    write_ir!(ir, "  {} = zext i1 {} to i64", ext_tmp, val);
                    self.fn_ctx.record_emitted_type(&ext_tmp, "i64");
                    arg_vals.push(format!("i64 {}", ext_tmp));
                }
                ResolvedType::F32 => {
                    arg_vals.push(format!("float {}", val));
                }
                ResolvedType::F64 => {
                    arg_vals.push(format!("double {}", val));
                }
                ty if Self::is_printf_string_arg_type(ty) => {
                    // Extract raw i8* pointer from string fat pointer for snprintf
                    let raw_ptr = self.extract_str_ptr(&val, counter, &mut ir);
                    arg_vals.push(format!("i8* {}", raw_ptr));
                }
                _ => {
                    let lowered = self
                        .lower_aggregate_format_arg_to_i64(&val, &arg_types[i], counter, &mut ir)
                        .unwrap_or(val);
                    arg_vals.push(format!("i64 {}", lowered));
                }
            }
        }

        if !arg_vals.is_empty() {
            snprintf_args = format!(", {}", arg_vals.join(", "));
        }

        // If no extra args, just return the format string directly as fat pointer
        if extra_args.is_empty() {
            let str_name = format!(".str.{}", self.strings.counter - 1);
            // Already pushed the constant above, reuse it
            let ptr = format!(
                "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                fmt_len, fmt_len, str_name
            );
            // Build fat pointer with the format string (minus null terminator)
            let str_len = format!("{}", c_format.len());
            let result = self.build_str_fat_ptr(&ptr, &str_len, counter, &mut ir);
            return Ok((result, ir));
        }

        // Step 1: snprintf(NULL, 0, fmt, ...) to get required length
        let len_i32 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = call i32 (i8*, i64, i8*, ...) @snprintf(i8* null, i64 0, i8* {}{})",
            len_i32,
            fmt_ptr,
            snprintf_args
        );

        // Convert i32 length to i64
        let len_i64 = self.next_temp(counter);
        write_ir!(ir, "  {} = sext i32 {} to i64", len_i64, len_i32);
        self.fn_ctx.record_emitted_type(&len_i64, "i64");

        // Step 2: malloc(len + 1)
        let buf_size = self.next_temp(counter);
        write_ir!(ir, "  {} = add i64 {}, 1", buf_size, len_i64);

        let buf_ptr = self.next_temp(counter);
        write_ir!(ir, "  {} = call i8* @malloc(i64 {})", buf_ptr, buf_size);
        self.fn_ctx.record_emitted_type(&buf_ptr, "i8*");
        let (track_ir, slot) = self.track_alloc_with_slot(buf_ptr.clone());
        ir.push_str(&track_ir);

        // Step 3: snprintf(buf, len+1, fmt, ...)
        let _write_result = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = call i32 (i8*, i64, i8*, ...) @snprintf(i8* {}, i64 {}, i8* {}{})",
            _write_result,
            buf_ptr,
            buf_size,
            fmt_ptr,
            snprintf_args
        );

        // Mark that we need snprintf declaration
        self.needs_string_helpers = true;

        // Build fat pointer { i8*, i64 } with the formatted string
        let result = self.build_str_fat_ptr(&buf_ptr, &len_i64, counter, &mut ir);
        if let Some(frame) = self.fn_ctx.scope_str_stack.last_mut() {
            frame.push(slot.clone());
        }
        self.fn_ctx.string_value_slot.insert(result.clone(), slot);
        Ok((result, ir))
    }

    /// Generate await expression
    #[inline(never)]
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
        self.strings
            .constants
            .push((fmt_name.clone(), fmt_str.to_string()));
        let fmt_len = fmt_str.len() + 1;
        let fmt_ptr = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = getelementptr [{} x i8], [{} x i8]* @{}, i64 0, i64 0",
            fmt_ptr,
            fmt_len,
            fmt_len,
            fmt_name
        );
        self.fn_ctx.record_emitted_type(&fmt_ptr, "i8*");
        let i32_result = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = call i32 (i8*, ...) @printf(i8* {}, i64 {})",
            i32_result,
            fmt_ptr,
            arg_val
        );
        self.fn_ctx.record_emitted_type(&i32_result, "i32");
        let result = self.next_temp(counter);
        write_ir!(ir, "  {} = sext i32 {} to i64", result, i32_result);
        self.fn_ctx.record_emitted_type(&result, "i64");
        Ok((result, ir))
    }

    /// Generate print_f64 builtin call
    #[inline(never)]
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
        self.strings
            .constants
            .push((fmt_name.clone(), fmt_str.to_string()));
        let fmt_len = fmt_str.len() + 1;
        let fmt_ptr = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = getelementptr [{} x i8], [{} x i8]* @{}, i64 0, i64 0",
            fmt_ptr,
            fmt_len,
            fmt_len,
            fmt_name
        );
        self.fn_ctx.record_emitted_type(&fmt_ptr, "i8*");
        let i32_result = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = call i32 (i8*, ...) @printf(i8* {}, double {})",
            i32_result,
            fmt_ptr,
            arg_val
        );
        self.fn_ctx.record_emitted_type(&i32_result, "i32");
        let result = self.next_temp(counter);
        write_ir!(ir, "  {} = sext i32 {} to i64", result, i32_result);
        self.fn_ctx.record_emitted_type(&result, "i64");
        Ok((result, ir))
    }
}

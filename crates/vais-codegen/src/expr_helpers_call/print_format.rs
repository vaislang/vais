use super::*;
use vais_ast::{Expr, Span, Spanned};
use vais_types::ResolvedType;

impl CodeGenerator {
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
                    self.strings
                        .constants
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
            self.strings
                .constants
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
        self.strings
            .constants
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
        self.strings
            .constants
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
}

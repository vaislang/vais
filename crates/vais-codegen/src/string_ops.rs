//! String operation code generation for Vais
//!
//! Implements runtime string operations: concatenation, comparison, and method calls.
//! Strings are fat pointers: { i8* ptr, i64 len }.
//! C interop extracts the i8* pointer before calling C runtime functions.

use crate::{CodeGenerator, CodegenError, CodegenResult};
use vais_ast::{BinOp, Expr, Spanned};

impl CodeGenerator {
    /// Resolve an argument value to an i8* pointer for builtins (free, memcpy, strlen, puts_ptr).
    /// Handles three cases:
    /// 1. `{ i8*, i64 }` fat pointer → extractvalue field 0
    /// 2. `i8*` raw pointer → use directly
    /// 3. `i64` integer → inttoptr to i8*
    pub(crate) fn resolve_arg_to_i8_ptr(
        &self,
        arg_full: &str,
        counter: &mut usize,
        ir: &mut String,
    ) -> String {
        if arg_full.starts_with("{ i8*, i64 }") {
            let val = arg_full
                .strip_prefix("{ i8*, i64 } ")
                .unwrap_or_else(|| arg_full.split_whitespace().last().unwrap_or("undef"));
            let ptr = self.next_temp(counter);
            write_ir!(ir, "  {} = extractvalue {{ i8*, i64 }} {}, 0", ptr, val);
            ptr
        } else if arg_full.starts_with("i8*") {
            arg_full
                .strip_prefix("i8* ")
                .unwrap_or_else(|| arg_full.split_whitespace().last().unwrap_or("null"))
                .to_string()
        } else {
            let arg_val = arg_full.split_whitespace().last().unwrap_or("0");
            let ptr = self.next_temp(counter);
            write_ir!(ir, "  {} = inttoptr i64 {} to i8*", ptr, arg_val);
            ptr
        }
    }

    /// Extract the i8* pointer from a string fat pointer { i8*, i64 }
    #[inline(never)]
    pub(crate) fn extract_str_ptr(
        &self,
        fat_ptr: &str,
        counter: &mut usize,
        ir: &mut String,
    ) -> String {
        let ptr = self.next_temp(counter);
        write_ir!(ir, "  {} = extractvalue {{ i8*, i64 }} {}, 0", ptr, fat_ptr);
        ptr
    }

    /// Extract the i64 length from a string fat pointer { i8*, i64 }
    #[inline(never)]
    pub(crate) fn extract_str_len(
        &self,
        fat_ptr: &str,
        counter: &mut usize,
        ir: &mut String,
    ) -> String {
        let len = self.next_temp(counter);
        write_ir!(ir, "  {} = extractvalue {{ i8*, i64 }} {}, 1", len, fat_ptr);
        len
    }

    /// Build a string fat pointer { i8*, i64 } from a raw pointer and length
    #[inline(never)]
    pub(crate) fn build_str_fat_ptr(
        &self,
        ptr: &str,
        len: &str,
        counter: &mut usize,
        ir: &mut String,
    ) -> String {
        let t0 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0",
            t0,
            ptr
        );
        let t1 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = insertvalue {{ i8*, i64 }} {}, i64 {}, 1",
            t1,
            t0,
            len
        );
        t1
    }

    /// Generate LLVM IR for string binary operations (+, ==, !=, <, >)
    #[inline(never)]
    pub(crate) fn generate_string_binary_op(
        &mut self,
        op: &BinOp,
        left_val: &str,
        right_val: &str,
        mut ir: String,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        self.needs_string_helpers = true;

        // Extract raw i8* pointers from fat pointers for C interop
        let left_ptr = self.extract_str_ptr(left_val, counter, &mut ir);
        let right_ptr = self.extract_str_ptr(right_val, counter, &mut ir);

        match op {
            BinOp::Add => {
                // String concatenation: call __vais_str_concat(left, right) -> { i8*, i64 }
                let result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = call {{ i8*, i64 }} @__vais_str_concat(i8* {}, i8* {})",
                    result,
                    left_ptr,
                    right_ptr
                );
                // Extract the raw pointer from the fat pointer for auto-free tracking
                let raw_ptr_for_free = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = extractvalue {{ i8*, i64 }} {}, 0",
                    raw_ptr_for_free,
                    result
                );
                self.track_alloc(raw_ptr_for_free);
                Ok((result, ir))
            }
            BinOp::Eq => {
                // String equality: strcmp(left, right) == 0
                let cmp_result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = call i32 @strcmp(i8* {}, i8* {})",
                    cmp_result,
                    left_ptr,
                    right_ptr
                );
                let eq_result = self.next_temp(counter);
                write_ir!(ir, "  {} = icmp eq i32 {}, 0", eq_result, cmp_result);
                let result = self.next_temp(counter);
                write_ir!(ir, "  {} = zext i1 {} to i64", result, eq_result);
                Ok((result, ir))
            }
            BinOp::Neq => {
                let cmp_result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = call i32 @strcmp(i8* {}, i8* {})",
                    cmp_result,
                    left_ptr,
                    right_ptr
                );
                let neq_result = self.next_temp(counter);
                write_ir!(ir, "  {} = icmp ne i32 {}, 0", neq_result, cmp_result);
                let result = self.next_temp(counter);
                write_ir!(ir, "  {} = zext i1 {} to i64", result, neq_result);
                Ok((result, ir))
            }
            BinOp::Lt => {
                let cmp_result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = call i32 @strcmp(i8* {}, i8* {})",
                    cmp_result,
                    left_ptr,
                    right_ptr
                );
                let lt_result = self.next_temp(counter);
                write_ir!(ir, "  {} = icmp slt i32 {}, 0", lt_result, cmp_result);
                let result = self.next_temp(counter);
                write_ir!(ir, "  {} = zext i1 {} to i64", result, lt_result);
                Ok((result, ir))
            }
            BinOp::Gt => {
                let cmp_result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = call i32 @strcmp(i8* {}, i8* {})",
                    cmp_result,
                    left_ptr,
                    right_ptr
                );
                let gt_result = self.next_temp(counter);
                write_ir!(ir, "  {} = icmp sgt i32 {}, 0", gt_result, cmp_result);
                let result = self.next_temp(counter);
                write_ir!(ir, "  {} = zext i1 {} to i64", result, gt_result);
                Ok((result, ir))
            }
            _ => Err(CodegenError::Unsupported(format!(
                "string operation {:?} not supported",
                op
            ))),
        }
    }

    /// Generate LLVM IR for string method calls.
    /// recv_val is a string fat pointer { i8*, i64 }.
    #[inline(never)]
    pub(crate) fn generate_string_method_call(
        &mut self,
        recv_val: &str,
        prefix_ir: &str,
        method_name: &str,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        self.needs_string_helpers = true;
        let mut ir = prefix_ir.to_string();

        // Extract raw pointer from fat pointer for C interop
        let recv_ptr = self.extract_str_ptr(recv_val, counter, &mut ir);

        match method_name {
            "len" => {
                // Use the stored length from the fat pointer directly (O(1), no strlen)
                let result = self.extract_str_len(recv_val, counter, &mut ir);
                Ok((result, ir))
            }
            "charAt" => {
                if args.is_empty() {
                    return Err(CodegenError::Unsupported(format!(
                        "builtin 'charAt' requires 1 argument(s), got {}",
                        args.len()
                    )));
                }
                let (idx_val, idx_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&idx_ir);

                let ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr i8, i8* {}, i64 {}",
                    ptr,
                    recv_ptr,
                    idx_val
                );
                let byte = self.next_temp(counter);
                write_ir!(ir, "  {} = load i8, i8* {}", byte, ptr);
                let result = self.next_temp(counter);
                write_ir!(ir, "  {} = zext i8 {} to i64", result, byte);
                Ok((result, ir))
            }
            "contains" => {
                if args.is_empty() {
                    return Err(CodegenError::Unsupported(format!(
                        "builtin 'contains' requires 1 argument(s), got {}",
                        args.len()
                    )));
                }
                let (substr_val, substr_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&substr_ir);
                let substr_ptr = self.extract_str_ptr(&substr_val, counter, &mut ir);

                let strstr_result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = call i8* @strstr(i8* {}, i8* {})",
                    strstr_result,
                    recv_ptr,
                    substr_ptr
                );
                let is_null = self.next_temp(counter);
                write_ir!(ir, "  {} = icmp ne i8* {}, null", is_null, strstr_result);
                let result = self.next_temp(counter);
                write_ir!(ir, "  {} = zext i1 {} to i64", result, is_null);
                Ok((result, ir))
            }
            "indexOf" => {
                if args.is_empty() {
                    return Err(CodegenError::Unsupported(format!(
                        "builtin 'indexOf' requires 1 argument(s), got {}",
                        args.len()
                    )));
                }
                let (substr_val, substr_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&substr_ir);
                let substr_ptr = self.extract_str_ptr(&substr_val, counter, &mut ir);

                let result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = call i64 @__vais_str_indexOf(i8* {}, i8* {})",
                    result,
                    recv_ptr,
                    substr_ptr
                );
                Ok((result, ir))
            }
            "substring" => {
                if args.len() < 2 {
                    return Err(CodegenError::Unsupported(format!(
                        "builtin 'substring' requires 2 argument(s), got {}",
                        args.len()
                    )));
                }
                let (start_val, start_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&start_ir);
                let (end_val, end_ir) = self.generate_expr(&args[1], counter)?;
                ir.push_str(&end_ir);

                let result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = call {{ i8*, i64 }} @__vais_str_substring(i8* {}, i64 {}, i64 {})",
                    result,
                    recv_ptr,
                    start_val,
                    end_val
                );
                // Extract the raw pointer from the fat pointer for auto-free tracking
                let raw_ptr_for_free = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = extractvalue {{ i8*, i64 }} {}, 0",
                    raw_ptr_for_free,
                    result
                );
                self.track_alloc(raw_ptr_for_free);
                Ok((result, ir))
            }
            "startsWith" => {
                if args.is_empty() {
                    return Err(CodegenError::Unsupported(format!(
                        "builtin 'startsWith' requires 1 argument(s), got {}",
                        args.len()
                    )));
                }
                let (prefix_val, prefix_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&prefix_ir);
                let prefix_ptr = self.extract_str_ptr(&prefix_val, counter, &mut ir);

                let result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = call i64 @__vais_str_startsWith(i8* {}, i8* {})",
                    result,
                    recv_ptr,
                    prefix_ptr
                );
                Ok((result, ir))
            }
            "endsWith" => {
                if args.is_empty() {
                    return Err(CodegenError::Unsupported(format!(
                        "builtin 'endsWith' requires 1 argument(s), got {}",
                        args.len()
                    )));
                }
                let (suffix_val, suffix_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&suffix_ir);
                let suffix_ptr = self.extract_str_ptr(&suffix_val, counter, &mut ir);

                let result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = call i64 @__vais_str_endsWith(i8* {}, i8* {})",
                    result,
                    recv_ptr,
                    suffix_ptr
                );
                Ok((result, ir))
            }
            "isEmpty" => {
                // Use length from fat pointer: len == 0
                let len = self.extract_str_len(recv_val, counter, &mut ir);
                let is_zero = self.next_temp(counter);
                write_ir!(ir, "  {} = icmp eq i64 {}, 0", is_zero, len);
                let result = self.next_temp(counter);
                write_ir!(ir, "  {} = zext i1 {} to i64", result, is_zero);
                Ok((result, ir))
            }
            "clone" | "to_string" | "as_str" => {
                // Strings are immutable fat pointers — clone is identity
                Ok((recv_val.to_string(), ir))
            }
            "as_bytes" => {
                // as_bytes returns the raw pointer (first field of fat pointer)
                Ok((recv_ptr, ir))
            }
            _ => Err(CodegenError::Unsupported(format!(
                "string method '{}' not supported",
                method_name
            ))),
        }
    }

    /// Generate LLVM IR for string helper functions.
    /// String helpers accept raw i8* pointers (extracted at call site).
    /// Functions that return strings return { i8*, i64 } fat pointers.
    ///
    /// Pre-allocates a buffer large enough for all helper functions to avoid
    /// intermediate reallocations during the push_str chain (~3.5KB total).
    #[inline(never)]
    pub(crate) fn generate_string_helper_functions(&self) -> String {
        let mut ir = String::with_capacity(4096);

        // __vais_str_concat: concatenate two strings -> { i8*, i64 }
        ir.push_str("\n; String helper: concatenate two strings\n");
        ir.push_str("define { i8*, i64 } @__vais_str_concat(i8* %a, i8* %b) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %alen = call i64 @strlen(i8* %a)\n");
        ir.push_str("  %blen = call i64 @strlen(i8* %b)\n");
        ir.push_str("  %total = add i64 %alen, %blen\n");
        ir.push_str("  %size = add i64 %total, 1\n");
        ir.push_str("  %buf_int = call i64 @malloc(i64 %size)\n");
        ir.push_str("  %buf = inttoptr i64 %buf_int to i8*\n");
        ir.push_str("  %a_int = ptrtoint i8* %a to i64\n");
        ir.push_str("  call i64 @memcpy(i64 %buf_int, i64 %a_int, i64 %alen)\n");
        ir.push_str("  %dst = getelementptr i8, i8* %buf, i64 %alen\n");
        ir.push_str("  %dst_int = ptrtoint i8* %dst to i64\n");
        ir.push_str("  %b_int = ptrtoint i8* %b to i64\n");
        ir.push_str("  %bsize = add i64 %blen, 1\n");
        ir.push_str("  call i64 @memcpy(i64 %dst_int, i64 %b_int, i64 %bsize)\n");
        // Build fat pointer { i8*, i64 }
        ir.push_str("  %fp0 = insertvalue { i8*, i64 } undef, i8* %buf, 0\n");
        ir.push_str("  %fp1 = insertvalue { i8*, i64 } %fp0, i64 %total, 1\n");
        ir.push_str("  ret { i8*, i64 } %fp1\n");
        ir.push_str("}\n");

        // __vais_str_indexOf: find substring position
        ir.push_str("\n; String helper: indexOf\n");
        ir.push_str("define i64 @__vais_str_indexOf(i8* %haystack, i8* %needle) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %found = call i8* @strstr(i8* %haystack, i8* %needle)\n");
        ir.push_str("  %is_null = icmp eq i8* %found, null\n");
        ir.push_str("  br i1 %is_null, label %not_found, label %calc\n");
        ir.push_str("not_found:\n");
        ir.push_str("  ret i64 -1\n");
        ir.push_str("calc:\n");
        ir.push_str("  %haystack_int = ptrtoint i8* %haystack to i64\n");
        ir.push_str("  %found_int = ptrtoint i8* %found to i64\n");
        ir.push_str("  %index = sub i64 %found_int, %haystack_int\n");
        ir.push_str("  ret i64 %index\n");
        ir.push_str("}\n");

        // __vais_str_substring: extract substring [start, end) -> { i8*, i64 }
        ir.push_str("\n; String helper: substring\n");
        ir.push_str("define { i8*, i64 } @__vais_str_substring(i8* %s, i64 %start, i64 %end) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %len = sub i64 %end, %start\n");
        ir.push_str("  %size = add i64 %len, 1\n");
        ir.push_str("  %buf_int = call i64 @malloc(i64 %size)\n");
        ir.push_str("  %buf = inttoptr i64 %buf_int to i8*\n");
        ir.push_str("  %src = getelementptr i8, i8* %s, i64 %start\n");
        ir.push_str("  %src_int = ptrtoint i8* %src to i64\n");
        ir.push_str("  call i64 @memcpy(i64 %buf_int, i64 %src_int, i64 %len)\n");
        ir.push_str("  %null_pos = getelementptr i8, i8* %buf, i64 %len\n");
        ir.push_str("  store i8 0, i8* %null_pos\n");
        // Build fat pointer { i8*, i64 }
        ir.push_str("  %fp0 = insertvalue { i8*, i64 } undef, i8* %buf, 0\n");
        ir.push_str("  %fp1 = insertvalue { i8*, i64 } %fp0, i64 %len, 1\n");
        ir.push_str("  ret { i8*, i64 } %fp1\n");
        ir.push_str("}\n");

        // __vais_str_startsWith: check if string starts with prefix
        ir.push_str("\n; String helper: startsWith\n");
        ir.push_str("define i64 @__vais_str_startsWith(i8* %s, i8* %prefix) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %plen = call i64 @strlen(i8* %prefix)\n");
        ir.push_str("  %cmp = call i32 @strncmp(i8* %s, i8* %prefix, i64 %plen)\n");
        ir.push_str("  %eq = icmp eq i32 %cmp, 0\n");
        ir.push_str("  %result = zext i1 %eq to i64\n");
        ir.push_str("  ret i64 %result\n");
        ir.push_str("}\n");

        // __vais_str_endsWith: check if string ends with suffix
        ir.push_str("\n; String helper: endsWith\n");
        ir.push_str("define i64 @__vais_str_endsWith(i8* %s, i8* %suffix) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %slen = call i64 @strlen(i8* %s)\n");
        ir.push_str("  %suflen = call i64 @strlen(i8* %suffix)\n");
        ir.push_str("  %too_short = icmp ult i64 %slen, %suflen\n");
        ir.push_str("  br i1 %too_short, label %ret_false, label %check\n");
        ir.push_str("check:\n");
        ir.push_str("  %offset = sub i64 %slen, %suflen\n");
        ir.push_str("  %tail = getelementptr i8, i8* %s, i64 %offset\n");
        ir.push_str("  %cmp = call i32 @strcmp(i8* %tail, i8* %suffix)\n");
        ir.push_str("  %eq = icmp eq i32 %cmp, 0\n");
        ir.push_str("  %result = zext i1 %eq to i64\n");
        ir.push_str("  ret i64 %result\n");
        ir.push_str("ret_false:\n");
        ir.push_str("  ret i64 0\n");
        ir.push_str("}\n");

        ir
    }

    /// Generate extern declarations for string runtime functions (only new ones not in builtins)
    #[inline(never)]
    pub(crate) fn generate_string_extern_declarations(&self) -> String {
        let mut ir = String::with_capacity(256);
        ir.push_str("\n; String runtime extern declarations\n");
        // strstr is new (not in builtins.rs)
        ir.push_str("declare i8* @strstr(i8*, i8*)\n");
        ir.push_str("declare i32 @snprintf(i8*, i64, i8*, ...)\n");
        ir
    }
}

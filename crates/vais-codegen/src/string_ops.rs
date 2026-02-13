//! String operation code generation for Vais
//!
//! Implements runtime string operations: concatenation, comparison, and method calls.
//! Uses i8* representation for strings (C-compatible null-terminated).
//! Interops with existing malloc/memcpy via inttoptr/ptrtoint conversions.

use crate::{CodeGenerator, CodegenError, CodegenResult};
use vais_ast::{BinOp, Expr, Spanned};

impl CodeGenerator {
    /// Generate LLVM IR for string binary operations (+, ==, !=, <, >)
    pub(crate) fn generate_string_binary_op(
        &mut self,
        op: &BinOp,
        left_val: &str,
        right_val: &str,
        mut ir: String,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        self.needs_string_helpers = true;

        match op {
            BinOp::Add => {
                // String concatenation: call __vais_str_concat(left, right) -> i8*
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call i8* @__vais_str_concat(i8* {}, i8* {})\n",
                    result, left_val, right_val
                ));
                Ok((result, ir))
            }
            BinOp::Eq => {
                // String equality: strcmp(left, right) == 0
                let cmp_result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call i32 @strcmp(i8* {}, i8* {})\n",
                    cmp_result, left_val, right_val
                ));
                let eq_result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = icmp eq i32 {}, 0\n",
                    eq_result, cmp_result
                ));
                let result = self.next_temp(counter);
                ir.push_str(&format!("  {} = zext i1 {} to i64\n", result, eq_result));
                Ok((result, ir))
            }
            BinOp::Neq => {
                let cmp_result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call i32 @strcmp(i8* {}, i8* {})\n",
                    cmp_result, left_val, right_val
                ));
                let neq_result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = icmp ne i32 {}, 0\n",
                    neq_result, cmp_result
                ));
                let result = self.next_temp(counter);
                ir.push_str(&format!("  {} = zext i1 {} to i64\n", result, neq_result));
                Ok((result, ir))
            }
            BinOp::Lt => {
                let cmp_result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call i32 @strcmp(i8* {}, i8* {})\n",
                    cmp_result, left_val, right_val
                ));
                let lt_result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = icmp slt i32 {}, 0\n",
                    lt_result, cmp_result
                ));
                let result = self.next_temp(counter);
                ir.push_str(&format!("  {} = zext i1 {} to i64\n", result, lt_result));
                Ok((result, ir))
            }
            BinOp::Gt => {
                let cmp_result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call i32 @strcmp(i8* {}, i8* {})\n",
                    cmp_result, left_val, right_val
                ));
                let gt_result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = icmp sgt i32 {}, 0\n",
                    gt_result, cmp_result
                ));
                let result = self.next_temp(counter);
                ir.push_str(&format!("  {} = zext i1 {} to i64\n", result, gt_result));
                Ok((result, ir))
            }
            _ => Err(CodegenError::Unsupported(format!(
                "string operation {:?} not supported",
                op
            ))),
        }
    }

    /// Generate LLVM IR for string method calls
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

        match method_name {
            "len" => {
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call i64 @strlen(i8* {})\n",
                    result, recv_val
                ));
                Ok((result, ir))
            }
            "charAt" => {
                if args.is_empty() {
                    return Err(CodegenError::Unsupported(
                        "charAt requires an index argument".to_string(),
                    ));
                }
                let (idx_val, idx_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&idx_ir);

                let ptr = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr i8, i8* {}, i64 {}\n",
                    ptr, recv_val, idx_val
                ));
                let byte = self.next_temp(counter);
                ir.push_str(&format!("  {} = load i8, i8* {}\n", byte, ptr));
                let result = self.next_temp(counter);
                ir.push_str(&format!("  {} = zext i8 {} to i64\n", result, byte));
                Ok((result, ir))
            }
            "contains" => {
                if args.is_empty() {
                    return Err(CodegenError::Unsupported(
                        "contains requires a string argument".to_string(),
                    ));
                }
                let (substr_val, substr_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&substr_ir);

                let strstr_result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call i8* @strstr(i8* {}, i8* {})\n",
                    strstr_result, recv_val, substr_val
                ));
                let is_null = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = icmp ne i8* {}, null\n",
                    is_null, strstr_result
                ));
                let result = self.next_temp(counter);
                ir.push_str(&format!("  {} = zext i1 {} to i64\n", result, is_null));
                Ok((result, ir))
            }
            "indexOf" => {
                if args.is_empty() {
                    return Err(CodegenError::Unsupported(
                        "indexOf requires a string argument".to_string(),
                    ));
                }
                let (substr_val, substr_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&substr_ir);

                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call i64 @__vais_str_indexOf(i8* {}, i8* {})\n",
                    result, recv_val, substr_val
                ));
                Ok((result, ir))
            }
            "substring" => {
                if args.len() < 2 {
                    return Err(CodegenError::Unsupported(
                        "substring requires start and end arguments".to_string(),
                    ));
                }
                let (start_val, start_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&start_ir);
                let (end_val, end_ir) = self.generate_expr(&args[1], counter)?;
                ir.push_str(&end_ir);

                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call i8* @__vais_str_substring(i8* {}, i64 {}, i64 {})\n",
                    result, recv_val, start_val, end_val
                ));
                Ok((result, ir))
            }
            "startsWith" => {
                if args.is_empty() {
                    return Err(CodegenError::Unsupported(
                        "startsWith requires a string argument".to_string(),
                    ));
                }
                let (prefix_val, prefix_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&prefix_ir);

                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call i64 @__vais_str_startsWith(i8* {}, i8* {})\n",
                    result, recv_val, prefix_val
                ));
                Ok((result, ir))
            }
            "endsWith" => {
                if args.is_empty() {
                    return Err(CodegenError::Unsupported(
                        "endsWith requires a string argument".to_string(),
                    ));
                }
                let (suffix_val, suffix_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&suffix_ir);

                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call i64 @__vais_str_endsWith(i8* {}, i8* {})\n",
                    result, recv_val, suffix_val
                ));
                Ok((result, ir))
            }
            "isEmpty" => {
                let byte = self.next_temp(counter);
                ir.push_str(&format!("  {} = load i8, i8* {}\n", byte, recv_val));
                let is_zero = self.next_temp(counter);
                ir.push_str(&format!("  {} = icmp eq i8 {}, 0\n", is_zero, byte));
                let result = self.next_temp(counter);
                ir.push_str(&format!("  {} = zext i1 {} to i64\n", result, is_zero));
                Ok((result, ir))
            }
            _ => Err(CodegenError::Unsupported(format!(
                "string method '{}' not supported",
                method_name
            ))),
        }
    }

    /// Generate LLVM IR for string helper functions.
    /// Uses inttoptr/ptrtoint for interop with existing malloc(i64)->i64 and memcpy(i64,i64,i64)->i64
    pub(crate) fn generate_string_helper_functions(&self) -> String {
        let mut ir = String::with_capacity(2048);

        // __vais_str_concat: concatenate two strings
        ir.push_str("\n; String helper: concatenate two strings\n");
        ir.push_str("define i8* @__vais_str_concat(i8* %a, i8* %b) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %alen = call i64 @strlen(i8* %a)\n");
        ir.push_str("  %blen = call i64 @strlen(i8* %b)\n");
        ir.push_str("  %total = add i64 %alen, %blen\n");
        ir.push_str("  %size = add i64 %total, 1\n");
        // Use existing malloc(i64)->i64, then inttoptr
        ir.push_str("  %buf_int = call i64 @malloc(i64 %size)\n");
        ir.push_str("  %buf = inttoptr i64 %buf_int to i8*\n");
        // Copy first string: memcpy(dest_int, src_int, len)
        ir.push_str("  %a_int = ptrtoint i8* %a to i64\n");
        ir.push_str("  call i64 @memcpy(i64 %buf_int, i64 %a_int, i64 %alen)\n");
        // Copy second string after first
        ir.push_str("  %dst = getelementptr i8, i8* %buf, i64 %alen\n");
        ir.push_str("  %dst_int = ptrtoint i8* %dst to i64\n");
        ir.push_str("  %b_int = ptrtoint i8* %b to i64\n");
        ir.push_str("  %bsize = add i64 %blen, 1\n");
        ir.push_str("  call i64 @memcpy(i64 %dst_int, i64 %b_int, i64 %bsize)\n");
        ir.push_str("  ret i8* %buf\n");
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

        // __vais_str_substring: extract substring [start, end)
        ir.push_str("\n; String helper: substring\n");
        ir.push_str("define i8* @__vais_str_substring(i8* %s, i64 %start, i64 %end) {\n");
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
        ir.push_str("  ret i8* %buf\n");
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
    pub(crate) fn generate_string_extern_declarations(&self) -> String {
        let mut ir = String::with_capacity(256);
        ir.push_str("\n; String runtime extern declarations\n");
        // strstr is new (not in builtins.rs)
        ir.push_str("declare i8* @strstr(i8*, i8*)\n");
        ir.push_str("declare i32 @snprintf(i8*, i64, i8*, ...)\n");
        ir
    }
}

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

    /// Emit `free(slot); store null, slot` if `value` is a tracked concat
    /// intermediate (its fat pointer owns a slot in `string_value_slot`).
    /// Safe because the caller has guaranteed that `value` has been consumed
    /// (e.g., memcpy'd by the next concat) and is not otherwise referenced.
    /// After this, the slot is null-valued; end-of-scope cleanup skips null.
    fn emit_intermediate_free(&mut self, value: &str, ir: &mut String) {
        // Extract the SSA register name (strip optional "{ i8*, i64 } " prefix).
        let key = value
            .strip_prefix("{ i8*, i64 } ")
            .unwrap_or(value)
            .trim()
            .to_string();
        let slot = match self.fn_ctx.string_value_slot.remove(&key) {
            Some(s) => s,
            None => return,
        };
        let tick = self.fn_ctx.label_counter;
        self.fn_ctx.label_counter += 1;
        let prev = format!("%__ifr_p_{}", tick);
        write_ir!(ir, "  {} = load i8*, i8** {}", prev, slot);
        let is_null = format!("%__ifr_n_{}", tick);
        write_ir!(ir, "  {} = icmp eq i8* {}, null", is_null, prev);
        let do_free = format!("__ifr_f_{}", tick);
        let after = format!("__ifr_a_{}", tick);
        write_ir!(
            ir,
            "  br i1 {}, label %{}, label %{}",
            is_null,
            after,
            do_free
        );
        write_ir!(ir, "{}:", do_free);
        write_ir!(ir, "  call void @free(i8* {})", prev);
        write_ir!(ir, "  store i8* null, i8** {}", slot);
        write_ir!(ir, "  br label %{}", after);
        write_ir!(ir, "{}:", after);
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
                let (store_ir, slot) = self.track_alloc_with_slot(raw_ptr_for_free);
                ir.push_str(&store_ir);
                // Record ownership: the fat-pointer SSA register owns this slot.
                // Also register in the topmost string-scope frame for block-exit cleanup.
                if let Some(frame) = self.fn_ctx.scope_str_stack.last_mut() {
                    frame.push(slot.clone());
                }
                self.fn_ctx.string_value_slot.insert(result.clone(), slot);
                // Intermediate free: in `a + b + c`, the LHS `a+b` is a tracked
                // concat result whose fat-pointer SSA value is `left_val`. It has
                // been consumed by this concat (memcpy'd), so the intermediate
                // buffer is safe to free now. See RFC-001 §4.3. We free and null
                // the slot; cleanup later skips null slots.
                self.emit_intermediate_free(left_val, &mut ir);
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
                let (store_ir, slot) = self.track_alloc_with_slot(raw_ptr_for_free);
                ir.push_str(&store_ir);
                // Register in the topmost string-scope frame for block-exit cleanup.
                if let Some(frame) = self.fn_ctx.scope_str_stack.last_mut() {
                    frame.push(slot.clone());
                }
                self.fn_ctx.string_value_slot.insert(result.clone(), slot);
                // Substring consumes its receiver's buffer as input only; we do
                // NOT free recv_val here because the receiver may still be
                // referenced elsewhere (common pattern: `s.substring(...)` where
                // `s` is a local). Intermediate free only applies when we
                // provably consumed the entire SSA chain (concat LHS).
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
            "push_str" => {
                if args.is_empty() {
                    return Err(CodegenError::Unsupported(format!(
                        "builtin 'push_str' requires 1 argument(s), got {}",
                        args.len()
                    )));
                }
                let (arg_val, arg_ir) = self.generate_expr(&args[0], counter)?;
                ir.push_str(&arg_ir);
                let arg_ptr = self.extract_str_ptr(&arg_val, counter, &mut ir);
                let result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = call {{ i8*, i64 }} @__vais_str_concat(i8* {}, i8* {})",
                    result,
                    recv_ptr,
                    arg_ptr
                );
                let raw_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = extractvalue {{ i8*, i64 }} {}, 0",
                    raw_ptr,
                    result
                );
                let (store_ir, slot) = self.track_alloc_with_slot(raw_ptr);
                ir.push_str(&store_ir);
                // Register in the topmost string-scope frame for block-exit cleanup.
                if let Some(frame) = self.fn_ctx.scope_str_stack.last_mut() {
                    frame.push(slot.clone());
                }
                self.fn_ctx.string_value_slot.insert(result.clone(), slot);
                // push_str("x", y) — the receiver is consumed (we concat it with
                // y and throw away the old buffer) ONLY if the receiver is itself
                // a tracked concat result. Otherwise it's a literal or borrowed
                // value we must not free.
                self.emit_intermediate_free(recv_val, &mut ir);
                Ok((result, ir))
            }
            "clone" | "to_string" | "as_str" => {
                // Strings are immutable fat pointers — clone is identity
                Ok((recv_val.to_string(), ir))
            }
            "as_bytes" => {
                // as_bytes returns the raw pointer as i64 (ptrtoint for C interop)
                let result = self.next_temp(counter);
                write_ir!(ir, "  {} = ptrtoint i8* {} to i64", result, recv_ptr);
                Ok((result, ir))
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

    /// Generate `declare` statements for string helper functions defined in the main module.
    /// Used by non-main modules in per-module compilation so the linker can resolve them.
    #[inline(never)]
    pub(crate) fn generate_string_helper_declarations(&self) -> String {
        let mut ir = String::with_capacity(512);
        ir.push_str("\n; String helper declarations (defined in main module)\n");
        ir.push_str("declare { i8*, i64 } @__vais_str_concat(i8*, i8*)\n");
        ir.push_str("declare i64 @__vais_str_indexOf(i8*, i8*)\n");
        ir.push_str("declare { i8*, i64 } @__vais_str_substring(i8*, i64, i64)\n");
        ir.push_str("declare i64 @__vais_str_len(i8*)\n");
        ir.push_str("declare { i8*, i64 } @__vais_str_char_at(i8*, i64)\n");
        ir.push_str("declare { i8*, i64 } @__vais_str_slice(i8*, i64, i64)\n");
        ir.push_str("declare { i8*, i64 } @__vais_str_replace(i8*, i8*, i8*)\n");
        ir.push_str("declare { i8*, i64 } @__vais_str_trim(i8*)\n");
        ir.push_str("declare { i8*, i64 } @__vais_str_to_upper(i8*)\n");
        ir.push_str("declare { i8*, i64 } @__vais_str_to_lower(i8*)\n");
        ir.push_str("declare i64 @__vais_str_starts_with(i8*, i8*)\n");
        ir.push_str("declare i64 @__vais_str_ends_with(i8*, i8*)\n");
        ir.push_str("declare i64 @__vais_str_contains(i8*, i8*)\n");
        ir.push_str("declare { i8*, i64 } @__vais_i64_to_str(i64)\n");
        ir.push_str("declare { i8*, i64 } @__vais_f64_to_str(double)\n");
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

    /// Generate LLVM IR for Vec<str> container ownership helpers (RFC-002 §4.1, §4.4).
    /// These operate on the `owned` 5th field of Vec<T> (i64 bitmap pointer-int) and
    /// rely on the structural-equivalence invariant (%Vec body ≡ {i64,i64,i64,i64,i64}).
    ///
    /// ABI constraints (match codegen conventions):
    ///   @free is declared as `void @free(i8*)` (see function_gen/signature.rs:71).
    ///   @malloc is declared as `i8* @malloc(i64)` (see builtins/memory.rs).
    ///   Caller passes `%Vec*` — a typed pointer valid under structural equivalence.
    ///
    /// Helper ABI (stable):
    ///   __vais_vec_str_owned_ensure(%Vec*, i64 min_cap) -> void
    ///   __vais_vec_str_owned_set   (%Vec*, i64 idx)     -> void
    ///   __vais_vec_str_shallow_free(%Vec*)              -> void
    #[inline(never)]
    pub(crate) fn generate_vec_str_container_helpers(&self) -> String {
        let mut ir = String::with_capacity(2048);

        // __vais_vec_str_owned_ensure: ensure bitmap has ceil(min_cap/8) bytes,
        // freshly zero-filled. Strategy: free old (if any) and malloc+memset new.
        // Called from push-site when pushing a heap-owned value AND len >= old bitmap
        // coverage. Worst-case cost: O(cap/8) per grow, amortized O(1).
        //
        // Simplification: always allocate fresh on grow (no copy-preserve). This is
        // safe because the push-site loop below re-sets bits lazily as needed.
        // But to preserve already-set bits (earlier owned elements), we copy the
        // existing bytes via inline loop, then zero-fill the tail.
        ir.push_str("\n; Vec<str> container helper: ensure owned bitmap covers min_cap\n");
        ir.push_str("define void @__vais_vec_str_owned_ensure(%Vec* %v, i64 %min_cap) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %n7 = add i64 %min_cap, 7\n");
        ir.push_str("  %bytes_needed = sdiv i64 %n7, 8\n");
        // Guard: if min_cap == 0, do nothing
        ir.push_str("  %need_zero = icmp sle i64 %bytes_needed, 0\n");
        ir.push_str("  br i1 %need_zero, label %exit, label %check_ptr\n");
        ir.push_str("check_ptr:\n");
        ir.push_str("  %owned_field = getelementptr %Vec, %Vec* %v, i32 0, i32 4\n");
        ir.push_str("  %owned_i = load i64, i64* %owned_field\n");
        ir.push_str("  %is_null = icmp eq i64 %owned_i, 0\n");
        ir.push_str("  br i1 %is_null, label %alloc_fresh, label %grow_copy\n");
        ir.push_str("alloc_fresh:\n");
        ir.push_str("  %fresh_ptr = call i8* @malloc(i64 %bytes_needed)\n");
        ir.push_str("  %fresh_int = ptrtoint i8* %fresh_ptr to i64\n");
        ir.push_str("  store i64 %fresh_int, i64* %owned_field\n");
        // Zero-fill fresh buffer byte-by-byte (simple inline loop).
        ir.push_str("  br label %zfill_head\n");
        ir.push_str("zfill_head:\n");
        ir.push_str("  %zi = phi i64 [ 0, %alloc_fresh ], [ %zi_next, %zfill_body ]\n");
        ir.push_str("  %zdone = icmp sge i64 %zi, %bytes_needed\n");
        ir.push_str("  br i1 %zdone, label %exit, label %zfill_body\n");
        ir.push_str("zfill_body:\n");
        ir.push_str("  %zptr = getelementptr i8, i8* %fresh_ptr, i64 %zi\n");
        ir.push_str("  store i8 0, i8* %zptr\n");
        ir.push_str("  %zi_next = add i64 %zi, 1\n");
        ir.push_str("  br label %zfill_head\n");
        // grow_copy: allocate new, copy old ceil(len/8) bytes, zero tail, free old.
        ir.push_str("grow_copy:\n");
        ir.push_str("  %old_ptr = inttoptr i64 %owned_i to i8*\n");
        ir.push_str("  %new_ptr = call i8* @malloc(i64 %bytes_needed)\n");
        // Derive old byte-count from len (conservative upper bound since grow is called before len exceeds cap).
        ir.push_str("  %len_field = getelementptr %Vec, %Vec* %v, i32 0, i32 1\n");
        ir.push_str("  %len_v = load i64, i64* %len_field\n");
        ir.push_str("  %l7 = add i64 %len_v, 7\n");
        ir.push_str("  %old_bytes = sdiv i64 %l7, 8\n");
        // Bound old_bytes by bytes_needed (never copy more than destination holds).
        ir.push_str("  %obn_gt = icmp sgt i64 %old_bytes, %bytes_needed\n");
        ir.push_str("  %copy_bytes = select i1 %obn_gt, i64 %bytes_needed, i64 %old_bytes\n");
        ir.push_str("  br label %copy_head\n");
        ir.push_str("copy_head:\n");
        ir.push_str("  %ci = phi i64 [ 0, %grow_copy ], [ %ci_next, %copy_body ]\n");
        ir.push_str("  %cdone = icmp sge i64 %ci, %copy_bytes\n");
        ir.push_str("  br i1 %cdone, label %copy_zero_tail, label %copy_body\n");
        ir.push_str("copy_body:\n");
        ir.push_str("  %src_ptr = getelementptr i8, i8* %old_ptr, i64 %ci\n");
        ir.push_str("  %dst_ptr = getelementptr i8, i8* %new_ptr, i64 %ci\n");
        ir.push_str("  %src_byte = load i8, i8* %src_ptr\n");
        ir.push_str("  store i8 %src_byte, i8* %dst_ptr\n");
        ir.push_str("  %ci_next = add i64 %ci, 1\n");
        ir.push_str("  br label %copy_head\n");
        ir.push_str("copy_zero_tail:\n");
        ir.push_str("  br label %zt_head\n");
        ir.push_str("zt_head:\n");
        ir.push_str("  %ti = phi i64 [ %copy_bytes, %copy_zero_tail ], [ %ti_next, %zt_body ]\n");
        ir.push_str("  %tdone = icmp sge i64 %ti, %bytes_needed\n");
        ir.push_str("  br i1 %tdone, label %swap_ptr, label %zt_body\n");
        ir.push_str("zt_body:\n");
        ir.push_str("  %tzptr = getelementptr i8, i8* %new_ptr, i64 %ti\n");
        ir.push_str("  store i8 0, i8* %tzptr\n");
        ir.push_str("  %ti_next = add i64 %ti, 1\n");
        ir.push_str("  br label %zt_head\n");
        ir.push_str("swap_ptr:\n");
        ir.push_str("  %new_int = ptrtoint i8* %new_ptr to i64\n");
        ir.push_str("  store i64 %new_int, i64* %owned_field\n");
        ir.push_str("  call void @free(i8* %old_ptr)\n");
        ir.push_str("  br label %exit\n");
        ir.push_str("exit:\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        // __vais_vec_str_owned_set: set bit at index idx.
        // Assumes bitmap is already grown to cover idx (caller invokes owned_ensure first).
        ir.push_str("\n; Vec<str> container helper: set owned bit at index\n");
        ir.push_str("define void @__vais_vec_str_owned_set(%Vec* %v, i64 %idx) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %owned_field = getelementptr %Vec, %Vec* %v, i32 0, i32 4\n");
        ir.push_str("  %owned_i = load i64, i64* %owned_field\n");
        ir.push_str("  %is_null = icmp eq i64 %owned_i, 0\n");
        ir.push_str("  br i1 %is_null, label %exit, label %do_set\n");
        ir.push_str("do_set:\n");
        ir.push_str("  %bm = inttoptr i64 %owned_i to i8*\n");
        ir.push_str("  %byte_idx = sdiv i64 %idx, 8\n");
        ir.push_str("  %bit_idx = srem i64 %idx, 8\n");
        ir.push_str("  %byte_ptr = getelementptr i8, i8* %bm, i64 %byte_idx\n");
        ir.push_str("  %byte_val = load i8, i8* %byte_ptr\n");
        ir.push_str("  %bit_i8 = trunc i64 %bit_idx to i8\n");
        ir.push_str("  %mask = shl i8 1, %bit_i8\n");
        ir.push_str("  %new_val = or i8 %byte_val, %mask\n");
        ir.push_str("  store i8 %new_val, i8* %byte_ptr\n");
        ir.push_str("  br label %exit\n");
        ir.push_str("exit:\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        // __vais_vec_str_shallow_free: free all heap-owned element buffers + free bitmap.
        // Does NOT free self.data — user Vec.drop runs AFTER this helper.
        // Element encoding: Vec<str>.data is an array of {i8*, i64} fat pointers
        //   (elem_size=16, matches method_call.rs "str" → 16 branch).
        // For each index i in [0, len): if bit i set, free the i8* at offset 0 of
        //   the fat pointer at data + i*16.
        ir.push_str("\n; Vec<str> container helper: shallow-free owned element buffers + bitmap\n");
        ir.push_str("define void @__vais_vec_str_shallow_free(%Vec* %v) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %owned_field = getelementptr %Vec, %Vec* %v, i32 0, i32 4\n");
        ir.push_str("  %owned_i = load i64, i64* %owned_field\n");
        ir.push_str("  %no_bm = icmp eq i64 %owned_i, 0\n");
        ir.push_str("  br i1 %no_bm, label %exit, label %iter_init\n");
        ir.push_str("iter_init:\n");
        ir.push_str("  %bm = inttoptr i64 %owned_i to i8*\n");
        ir.push_str("  %data_field = getelementptr %Vec, %Vec* %v, i32 0, i32 0\n");
        ir.push_str("  %data_i = load i64, i64* %data_field\n");
        ir.push_str("  %len_field = getelementptr %Vec, %Vec* %v, i32 0, i32 1\n");
        ir.push_str("  %len_v = load i64, i64* %len_field\n");
        ir.push_str("  br label %loop_head\n");
        ir.push_str("loop_head:\n");
        ir.push_str("  %i = phi i64 [ 0, %iter_init ], [ %i_next, %loop_cont ]\n");
        ir.push_str("  %done_cmp = icmp sge i64 %i, %len_v\n");
        ir.push_str("  br i1 %done_cmp, label %free_bm, label %check_bit\n");
        ir.push_str("check_bit:\n");
        ir.push_str("  %byte_idx = sdiv i64 %i, 8\n");
        ir.push_str("  %bit_idx = srem i64 %i, 8\n");
        ir.push_str("  %byte_ptr = getelementptr i8, i8* %bm, i64 %byte_idx\n");
        ir.push_str("  %byte_val = load i8, i8* %byte_ptr\n");
        ir.push_str("  %bit_i8 = trunc i64 %bit_idx to i8\n");
        ir.push_str("  %mask = shl i8 1, %bit_i8\n");
        ir.push_str("  %bit_and = and i8 %byte_val, %mask\n");
        ir.push_str("  %is_owned = icmp ne i8 %bit_and, 0\n");
        ir.push_str("  br i1 %is_owned, label %do_free, label %loop_cont\n");
        ir.push_str("do_free:\n");
        ir.push_str("  %off = mul i64 %i, 16\n");
        ir.push_str("  %elem_int = add i64 %data_i, %off\n");
        ir.push_str("  %elem_ptr = inttoptr i64 %elem_int to i8**\n");
        ir.push_str("  %buf_ptr = load i8*, i8** %elem_ptr\n");
        ir.push_str("  %buf_is_null = icmp eq i8* %buf_ptr, null\n");
        ir.push_str("  br i1 %buf_is_null, label %loop_cont, label %do_call_free\n");
        ir.push_str("do_call_free:\n");
        ir.push_str("  call void @free(i8* %buf_ptr)\n");
        ir.push_str("  store i8* null, i8** %elem_ptr\n");
        ir.push_str("  br label %loop_cont\n");
        ir.push_str("loop_cont:\n");
        ir.push_str("  %i_next = add i64 %i, 1\n");
        ir.push_str("  br label %loop_head\n");
        ir.push_str("free_bm:\n");
        ir.push_str("  %bm_as_i8p = inttoptr i64 %owned_i to i8*\n");
        ir.push_str("  call void @free(i8* %bm_as_i8p)\n");
        ir.push_str("  store i64 0, i64* %owned_field\n");
        ir.push_str("  br label %exit\n");
        ir.push_str("exit:\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        ir
    }

    /// `declare` statements for Vec<str> container helpers (per-module compilation).
    #[inline(never)]
    pub(crate) fn generate_vec_str_container_declarations(&self) -> String {
        let mut ir = String::with_capacity(384);
        ir.push_str("\n; Vec<str> container helper declarations (defined in main module)\n");
        ir.push_str("declare void @__vais_vec_str_owned_ensure(%Vec*, i64)\n");
        ir.push_str("declare void @__vais_vec_str_owned_set(%Vec*, i64)\n");
        ir.push_str("declare void @__vais_vec_str_shallow_free(%Vec*)\n");
        ir
    }
}

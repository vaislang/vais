use super::*;
use vais_ast::{Expr, Span, Spanned};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Generate memcpy call
    pub(super) fn generate_memcpy_call(
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
    pub(super) fn generate_strlen_call(
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
    pub(super) fn generate_puts_ptr_call(
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

        // Slice .len() — extract length from fat pointer { i8*, i64 } field 1
        if method_name == "len" {
            let is_slice_type = match &recv_type {
                ResolvedType::Slice(_) | ResolvedType::SliceMut(_) => true,
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => matches!(
                    inner.as_ref(),
                    ResolvedType::Slice(_) | ResolvedType::SliceMut(_)
                ),
                _ => false,
            };
            if is_slice_type {
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {{ i8*, i64 }} {}, 1\n",
                    result, recv_val
                ));
                return Ok((result, ir));
            }
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
            let (mut val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);
            let arg_type = self.infer_expr_type(arg);
            let arg_llvm_ty = self.type_to_llvm(&arg_type);

            // For struct types, load the value from pointer if the expression produces a pointer.
            // Struct literals and local struct variables return pointers (alloca),
            // but function params expect values.
            if matches!(arg_type, ResolvedType::Named { .. }) && !self.is_expr_value(arg) {
                let loaded = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = load {}, {}* {}\n",
                    loaded, arg_llvm_ty, arg_llvm_ty, val
                ));
                val = loaded;
            }

            arg_vals.push(format!("{} {}", arg_llvm_ty, val));
        }

        // Infer the actual return type of the method from function info
        let ret_type = {
            let fn_info = self.types.functions.get(&full_method_name);
            if let Some(info) = fn_info {
                self.type_to_llvm(&info.signature.ret)
            } else {
                "i64".to_string()
            }
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

        let mut arg_vals = Vec::with_capacity(args.len());
        for arg in args {
            let (mut val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);
            let arg_type = self.infer_expr_type(arg);
            let arg_llvm_ty = self.type_to_llvm(&arg_type);

            // For struct types, load the value from pointer if the expression produces a pointer.
            if matches!(arg_type, ResolvedType::Named { .. }) && !self.is_expr_value(arg) {
                let loaded = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = load {}, {}* {}\n",
                    loaded, arg_llvm_ty, arg_llvm_ty, val
                ));
                val = loaded;
            }

            arg_vals.push(format!("{} {}", arg_llvm_ty, val));
        }

        let ret_type = self
            .types
            .functions
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

    /// Generate str_to_ptr builtin call
    pub(super) fn generate_str_to_ptr_builtin(
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
    pub(super) fn generate_ptr_to_str_builtin(
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

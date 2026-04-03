use super::*;
use vais_ast::{Expr, Span, Spanned};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Thin wrapper: delegates to the canonical `generate_expr_call` implementation.
    ///
    /// This exists because `expr_visitor.rs` and `misc_expr.rs` call
    /// `generate_call_expr(func, args, counter, span)` while the canonical
    /// implementation in `generate_expr_call.rs` uses
    /// `generate_expr_call(func, args, span, counter)` (parameter order differs).
    #[inline(always)]
    pub(crate) fn generate_call_expr(
        &mut self,
        func: &Spanned<Expr>,
        args: &[Spanned<Expr>],
        counter: &mut usize,
        span: Span,
    ) -> CodegenResult<(String, String)> {
        self.generate_expr_call(func, args, span, counter)
    }

    /// Generate enum variant constructor
    #[inline(never)]
    pub(crate) fn generate_enum_variant_constructor(
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
        write_ir!(ir, "  {} = alloca %{}", enum_ptr, enum_name);

        let tag_ptr = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0",
            tag_ptr,
            enum_name,
            enum_name,
            enum_ptr
        );
        write_ir!(ir, "  store i32 {}, i32* {}", tag, tag_ptr);

        // Store payload fields
        for (i, (arg_val, arg_expr)) in arg_vals.iter().zip(args.iter()).enumerate() {
            let payload_field_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = getelementptr %{}, %{}* {}, i32 0, i32 1, i32 {}",
                payload_field_ptr,
                enum_name,
                enum_name,
                enum_ptr,
                i
            );
            // Store payload into enum payload area
            // For non-i64 types, bitcast the payload pointer to T* and store directly
            // This copies the value INTO the Result struct (no dangling pointer)
            let arg_type = self.infer_expr_type(arg_expr);
            let llvm_ty = self.type_to_llvm(&arg_type);
            let type_size = self.compute_sizeof(&arg_type);
            // Check temp_var_types for more accurate type info when infer_expr_type returns I64.
            // This handles Vec<str>[i] → {i8*, i64} which infer_expr_type can't resolve.
            let (effective_ty, effective_size) = if matches!(&arg_type, ResolvedType::I64) {
                if let Some(temp_ty) = self.fn_ctx.temp_var_types.get(arg_val) {
                    let ty = self.type_to_llvm(temp_ty);
                    let sz = self.compute_sizeof(temp_ty);
                    if sz > 8 {
                        (ty, sz)
                    } else {
                        (llvm_ty.clone(), type_size)
                    }
                } else {
                    (llvm_ty.clone(), type_size)
                }
            } else {
                (llvm_ty.clone(), type_size)
            };
            let needs_cast = effective_ty != "i64"
                && effective_ty != "i32"
                && effective_ty != "i16"
                && effective_ty != "i8"
                && effective_ty != "i1"
                && !effective_ty.ends_with('*');
            if needs_cast && effective_size > 8 && arg_val.starts_with('%') {
                // Large struct (> 8 bytes): heap-allocate to avoid payload overflow.
                let heap_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = call i8* @malloc(i64 {})",
                    heap_ptr,
                    effective_size
                );
                let typed_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = bitcast i8* {} to {}*",
                    typed_ptr,
                    heap_ptr,
                    effective_ty
                );
                if !self.is_expr_value(arg_expr) {
                    let loaded = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = load {}, {}* {}",
                        loaded,
                        effective_ty,
                        effective_ty,
                        arg_val
                    );
                    write_ir!(
                        ir,
                        "  store {} {}, {}* {}",
                        effective_ty,
                        loaded,
                        effective_ty,
                        typed_ptr
                    );
                } else {
                    // Check if arg_val is i64 (generic erasure) but effective_ty is struct.
                    // If so, interpret i64 as pointer to struct and load before storing.
                    let actual_llvm = self.llvm_type_of(arg_val);
                    if (actual_llvm == "i64" || actual_llvm.starts_with('i'))
                        && effective_ty.starts_with('%')
                    {
                        let src_ptr = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = inttoptr i64 {} to {}*",
                            src_ptr,
                            arg_val,
                            effective_ty
                        );
                        let loaded = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = load {}, {}* {}",
                            loaded,
                            effective_ty,
                            effective_ty,
                            src_ptr
                        );
                        write_ir!(
                            ir,
                            "  store {} {}, {}* {}",
                            effective_ty,
                            loaded,
                            effective_ty,
                            typed_ptr
                        );
                    } else {
                        write_ir!(
                            ir,
                            "  store {} {}, {}* {}",
                            effective_ty,
                            arg_val,
                            effective_ty,
                            typed_ptr
                        );
                    }
                }
                let ptr_i64 = self.next_temp(counter);
                write_ir!(ir, "  {} = ptrtoint i8* {} to i64", ptr_i64, heap_ptr);
                write_ir!(ir, "  store i64 {}, i64* {}", ptr_i64, payload_field_ptr);
            } else if needs_cast && arg_val.starts_with('%') {
                // Small struct (≤ 8 bytes): bitcast payload slot and store directly
                let cast_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = bitcast i64* {} to {}*",
                    cast_ptr,
                    payload_field_ptr,
                    effective_ty
                );
                write_ir!(
                    ir,
                    "  store {} {}, {}* {}",
                    effective_ty,
                    arg_val,
                    effective_ty,
                    cast_ptr
                );
            } else {
                // Replace "void" with 0 for Unit/() values in enum payloads
                let store_val = if arg_val == "void" { "0" } else { arg_val };
                write_ir!(ir, "  store i64 {}, i64* {}", store_val, payload_field_ptr);
            }
        }

        Ok((enum_ptr, ir))
    }
}

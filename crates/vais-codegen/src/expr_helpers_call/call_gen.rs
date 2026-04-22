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

        // Look up the raw variant field types to determine the actual LLVM payload slot type.
        // For enums with Generic/I64 fields (builtin Option/Result), the slot is i64.
        // For enums with concrete Named fields, the slot is the native struct type.
        let raw_variant_fields = self.get_variant_raw_field_types_by_tag(enum_name, tag);

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

            // NOTE: Previously a `payload_is_native_struct` fast-path stored the
            // struct value directly into the payload slot. That was unsound
            // because enum types are generated with uniform i64 payload slots
            // (see types/type_gen.rs::generate_enum_type), so a 16B struct would
            // overflow an 8B slot and corrupt memory. Instead we always go
            // through the i64 slot path below, which heap-allocates large
            // structs and bitcasts small ones.
            let _ = raw_variant_fields.get(i);

            // Store payload into enum payload area (generic i64 slot)
            // For non-i64 types, bitcast the payload pointer to T* and store directly
            // This copies the value INTO the Result struct (no dangling pointer)
            let arg_type = self.infer_expr_type(arg_expr);
            let llvm_ty = self.type_to_llvm(&arg_type);
            let type_size = self.compute_sizeof(&arg_type);
            // Check temp_var_types for more accurate type info when infer_expr_type returns I64.
            // This handles Vec<str>[i] → {i8*, i64} which infer_expr_type can't resolve.
            //
            // Phase B5: also prefer the registered SSA type when the caller-side
            // inferred type is an unspecialized generic container (e.g., `%Vec`
            // when `Vec.new()` was call-site specialized to `%Vec$f32`). This
            // keeps the heap-alloc bitcast and store type-consistent with the
            // actual call return type.
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
            } else if llvm_ty.starts_with('%') && !llvm_ty.contains('$') {
                // Base generic type (e.g., "%Vec") — check if the SSA value
                // is already specialized ("%Vec$f32") and use that.
                if let Some(reg_ty) = self.llvm_type_of_checked(arg_val) {
                    if reg_ty.starts_with('%')
                        && reg_ty.contains('$')
                        && reg_ty.starts_with(&llvm_ty[..])
                    {
                        let sz = self
                            .fn_ctx
                            .temp_var_types
                            .get(arg_val)
                            .map(|t| self.compute_sizeof(t))
                            .unwrap_or(type_size);
                        (reg_ty, sz)
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
                // If arg_val is a pointer to the struct (e.g., from struct literal or local var),
                // we must load the struct value before storing into the payload slot.
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
                        cast_ptr
                    );
                } else {
                    write_ir!(
                        ir,
                        "  store {} {}, {}* {}",
                        effective_ty,
                        arg_val,
                        effective_ty,
                        cast_ptr
                    );
                }
            } else if effective_ty == "double" || effective_ty == "float" {
                // Float payloads must be bitcast-stored so the raw i64 slot receives
                // a valid integer bit pattern. A bare `store i64 1.0e+00, i64*` is
                // invalid IR.
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
                let store_val_str = if arg_val == "void" {
                    "0".to_string()
                } else {
                    arg_val.to_string()
                };
                // Widen narrow integer payloads (i8/i16/i32) to i64 so they
                // fit the enum's i64 slot.
                let actual_ty = if arg_val == "void" {
                    "i64".to_string()
                } else {
                    self.llvm_type_of(arg_val)
                };
                let final_val = if actual_ty != "i64"
                    && matches!(actual_ty.as_str(), "i1" | "i8" | "i16" | "i32")
                {
                    let widened = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = zext {} {} to i64",
                        widened,
                        actual_ty,
                        store_val_str
                    );
                    widened
                } else if actual_ty.ends_with('*') {
                    // Pointer payload (e.g., `Some(x)` where x is a struct
                    // returned as %T* from clone()/method call) — ptrtoint
                    // to fit the i64 payload slot.
                    let casted = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = ptrtoint {} {} to i64",
                        casted,
                        actual_ty,
                        store_val_str
                    );
                    casted
                } else {
                    store_val_str
                };
                write_ir!(ir, "  store i64 {}, i64* {}", final_val, payload_field_ptr);
            }
        }

        Ok((enum_ptr, ir))
    }
}

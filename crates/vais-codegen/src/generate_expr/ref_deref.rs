//! Reference and dereference expression code generation.
//!
//! Extracted from `generate_expr_inner` match arms for `Expr::Ref` and
//! `Expr::Deref` to reduce the parent function's stack frame size.
//! Each handler is `#[inline(never)]` so Rust allocates its locals independently.

use vais_ast::*;
use vais_types::ResolvedType;

use crate::{CodeGenerator, CodegenResult};

impl CodeGenerator {
    /// Generate code for a reference expression (`&expr`).
    /// Handles array-to-slice conversion, ident references, and general expressions.
    #[inline(never)]
    pub(crate) fn generate_ref_expr(
        &mut self,
        inner: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Special case: &[elem, ...] array literal -> slice fat pointer { i8*, i64 }
        if let Expr::Array(elements) = &inner.node {
            return self.generate_ref_array_slice(elements, counter);
        }

        // For simple references, just return the address
        if let Expr::Ident(name) = &inner.node {
            if let Some(local) = self.fn_ctx.locals.get(name.as_str()).cloned() {
                // If the local was initialized from an array literal with a
                // compile-time-known length, build a `{ i8*, i64 }` slice fat
                // pointer so calls into `fn(x: &[T])` receive the expected ABI.
                if let Some(len) = local.array_length {
                    return self.generate_ref_array_local_slice(&local, len, counter);
                }
                // `str` and slice references use the same fat-pointer ABI as
                // their value form. Keep alloca-backed locals consistent with
                // SSA locals by loading the fat value instead of returning a
                // `{ i8*, i64 }*` address.
                if matches!(
                    &local.ty,
                    ResolvedType::Str | ResolvedType::Slice(_) | ResolvedType::SliceMut(_)
                ) {
                    return self.generate_expr(inner, counter);
                }
                if local.is_alloca() {
                    // Alloca variables already have an address.
                    return Ok((format!("%{}", local.llvm_name), String::new()));
                } else {
                    // SSA/Param values need to be spilled to stack to take their address
                    return self.generate_ref_spill(&local, inner, counter);
                }
            }
        }

        // `&*box` / `&**box` must preserve the address inside Box<T>. The
        // generic fallback calls generate_expr on the deref expression, which
        // loads T and then tries to use the value where a T* is required.
        let mut deref_base = inner;
        let mut saw_deref = false;
        while let Expr::Deref(next) = &deref_base.node {
            saw_deref = true;
            deref_base = next;
        }
        if saw_deref {
            let base_ty = self.infer_expr_type(deref_base);
            if let ResolvedType::Named { name, generics } = &base_ty {
                if name == "Box" && generics.len() == 1 {
                    let pointee_ty = &generics[0];
                    let pointee_llvm = self.type_to_llvm(pointee_ty);
                    let (box_val, box_ir) = self.generate_expr(deref_base, counter)?;
                    let mut ir = box_ir;
                    let box_llvm = self.type_to_llvm(&base_ty);
                    let base_is_box_ptr_local = if let Expr::Ident(local_name) = &deref_base.node {
                        self.fn_ctx
                            .locals
                            .get(local_name.as_str())
                            .is_some_and(|local| {
                                local.is_alloca() || local.is_ssa() || local.is_param()
                            })
                    } else {
                        false
                    };
                    let actual_box_ty = self.llvm_type_of_checked(&box_val);
                    let ptr_i64 = if actual_box_ty.as_deref() == Some("i64")
                        || (actual_box_ty.is_none() && !base_is_box_ptr_local)
                    {
                        box_val
                    } else if actual_box_ty.as_deref() == Some("ptr") || base_is_box_ptr_local {
                        let loaded = self.next_temp(counter);
                        if box_llvm == "i64" {
                            write_ir!(ir, "  {} = load i64, i64* {}", loaded, box_val);
                        } else {
                            let field_ptr = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = getelementptr {}, {}* {}, i32 0, i32 0",
                                field_ptr,
                                box_llvm,
                                box_llvm,
                                box_val
                            );
                            write_ir!(ir, "  {} = load i64, i64* {}", loaded, field_ptr);
                        }
                        self.fn_ctx.record_emitted_type(&loaded, "i64");
                        loaded
                    } else {
                        match actual_box_ty.as_deref() {
                            Some(actual)
                                if actual.starts_with("%Box") && !actual.ends_with('*') =>
                            {
                                let extracted = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = extractvalue {} {}, 0",
                                    extracted,
                                    actual,
                                    box_val
                                );
                                self.fn_ctx.record_emitted_type(&extracted, "i64");
                                extracted
                            }
                            Some(actual) if actual.starts_with("%Box") && actual.ends_with('*') => {
                                let field_ptr = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = getelementptr {}, {} {}, i32 0, i32 0",
                                    field_ptr,
                                    actual.trim_end_matches('*'),
                                    actual,
                                    box_val
                                );
                                let loaded = self.next_temp(counter);
                                write_ir!(ir, "  {} = load i64, i64* {}", loaded, field_ptr);
                                self.fn_ctx.record_emitted_type(&loaded, "i64");
                                loaded
                            }
                            _ => box_val,
                        }
                    };
                    let typed_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = inttoptr i64 {} to {}*",
                        typed_ptr,
                        ptr_i64,
                        pointee_llvm
                    );
                    self.fn_ctx
                        .record_emitted_type(&typed_ptr, &format!("{}*", pointee_llvm));
                    return Ok((typed_ptr, ir));
                }
            }
        }

        // Phase 17.H4.6: `&arr[i]` should produce the element address, not
        // the loaded value. The default path delegates to `generate_expr`
        // which loads the element via `load i8, i8* %ptr`, then the caller
        // tries to ptrtoint the loaded byte — invalid IR. Emit a GEP-only
        // path here for slice / Vec indexing.
        //
        // Skip when the index is a Range — `&arr[a..b]` is a sub-slice,
        // not an element address; generate_expr handles Range correctly.
        if let Expr::Index {
            expr: inner_arr,
            index,
        } = &inner.node
        {
            if matches!(index.node, Expr::Range { .. }) {
                return self.generate_expr(inner, counter);
            }
            let arr_ty = self.infer_expr_type(inner_arr);
            let vec_elem_ty = match &arr_ty {
                ResolvedType::Named { name, generics } if name == "Vec" && !generics.is_empty() => {
                    Some(generics[0].clone())
                }
                ResolvedType::Ref(inner_t) | ResolvedType::RefMut(inner_t) => {
                    match inner_t.as_ref() {
                        ResolvedType::Named { name, generics }
                            if name == "Vec" && !generics.is_empty() =>
                        {
                            Some(generics[0].clone())
                        }
                        _ => None,
                    }
                }
                _ => None,
            };
            if let Some(elem_ty) = vec_elem_ty {
                let (vec_val, vec_ir) = self.generate_expr(inner_arr, counter)?;
                let (idx_val, idx_ir) = self.generate_expr(index, counter)?;
                let elem_llvm = self.type_to_llvm(&elem_ty);
                let mut ir = vec_ir;
                ir.push_str(&idx_ir);

                let data_field = self.next_temp(counter);
                use std::fmt::Write;
                let _ = writeln!(
                    ir,
                    "  {} = getelementptr %Vec, %Vec* {}, i32 0, i32 0",
                    data_field, vec_val
                );
                let data_i64 = self.next_temp(counter);
                let _ = writeln!(ir, "  {} = load i64, i64* {}", data_i64, data_field);
                let elem_size_ptr = self.next_temp(counter);
                let _ = writeln!(
                    ir,
                    "  {} = getelementptr %Vec, %Vec* {}, i32 0, i32 3",
                    elem_size_ptr, vec_val
                );
                let elem_size = self.next_temp(counter);
                let _ = writeln!(ir, "  {} = load i64, i64* {}", elem_size, elem_size_ptr);
                let data_ptr = self.next_temp(counter);
                let _ = writeln!(ir, "  {} = inttoptr i64 {} to i8*", data_ptr, data_i64);
                let byte_offset = self.next_temp(counter);
                let _ = writeln!(ir, "  {} = mul i64 {}, {}", byte_offset, idx_val, elem_size);
                let elem_i8_ptr = self.next_temp(counter);
                let _ = writeln!(
                    ir,
                    "  {} = getelementptr i8, i8* {}, i64 {}",
                    elem_i8_ptr, data_ptr, byte_offset
                );
                let elem_ptr = self.next_temp(counter);
                let _ = writeln!(
                    ir,
                    "  {} = bitcast i8* {} to {}*",
                    elem_ptr, elem_i8_ptr, elem_llvm
                );
                self.fn_ctx
                    .record_emitted_type(&elem_ptr, &format!("{}*", elem_llvm));
                self.fn_ctx.register_temp_type(&elem_ptr, elem_ty);
                return Ok((elem_ptr, ir));
            }
            // Only intercept when we can produce a raw element pointer.
            // Str indexing uses the slice's i8* base directly.
            let (elem_llvm, base_ptr_ir, base_ptr_val) = match &arr_ty {
                ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) => {
                    let (slice_val, slice_ir) = self.generate_expr(inner_arr, counter)?;
                    let base = self.next_temp(counter);
                    let ir = format!(
                        "{}  {} = extractvalue {{ i8*, i64 }} {}, 0\n",
                        slice_ir, base, slice_val
                    );
                    (self.type_to_llvm(elem), ir, base)
                }
                ResolvedType::Ref(inner_t) | ResolvedType::RefMut(inner_t) => {
                    match inner_t.as_ref() {
                        ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) => {
                            let (slice_val, slice_ir) = self.generate_expr(inner_arr, counter)?;
                            let base = self.next_temp(counter);
                            let ir = format!(
                                "{}  {} = extractvalue {{ i8*, i64 }} {}, 0\n",
                                slice_ir, base, slice_val
                            );
                            (self.type_to_llvm(elem), ir, base)
                        }
                        _ => return self.generate_expr(inner, counter),
                    }
                }
                ResolvedType::Str => {
                    // &s[i] — slice fat pointer base + GEP i8
                    let (slice_val, slice_ir) = self.generate_expr(inner_arr, counter)?;
                    let base = self.next_temp(counter);
                    let ir = format!(
                        "{}  {} = extractvalue {{ i8*, i64 }} {}, 0\n",
                        slice_ir, base, slice_val
                    );
                    ("i8".to_string(), ir, base)
                }
                _ => return self.generate_expr(inner, counter),
            };
            let (idx_val, idx_ir) = self.generate_expr(index, counter)?;
            let mut ir = base_ptr_ir;
            ir.push_str(&idx_ir);
            let elem_ptr = self.next_temp(counter);
            use std::fmt::Write;
            let _ = writeln!(
                ir,
                "  {} = getelementptr {}, {}* {}, i64 {}",
                elem_ptr, elem_llvm, elem_llvm, base_ptr_val, idx_val
            );
            self.fn_ctx
                .record_emitted_type(&elem_ptr, &format!("{}*", elem_llvm));
            return Ok((elem_ptr, ir));
        }
        // `&obj.field` — emit a GEP to the field's address rather than
        // loading the field's value. Without this, `&self.x as i64` would
        // ptrtoint the loaded value (often an i64 *value*, not address),
        // breaking `__atomic_*` runtime calls that expect a pointer.
        if let Expr::Field {
            expr: obj_expr,
            field,
        } = &inner.node
        {
            let obj_type = self.infer_expr_type(obj_expr);
            let resolved_type = match &obj_type {
                ResolvedType::Ref(inner_t)
                | ResolvedType::RefMut(inner_t)
                | ResolvedType::Pointer(inner_t) => inner_t.as_ref().clone(),
                other => other.clone(),
            };
            if let ResolvedType::Named {
                name: orig_type_name,
                generics: type_generics,
            } = &resolved_type
            {
                let candidate = if !type_generics.is_empty()
                    && type_generics
                        .iter()
                        .all(|g| !matches!(g, ResolvedType::Generic(_) | ResolvedType::Var(_)))
                {
                    let mangled = self.mangle_struct_name(orig_type_name, type_generics);
                    if self.types.structs.contains_key(&mangled)
                        || self.generics.generated_structs.contains_key(&mangled)
                    {
                        mangled
                    } else {
                        self.resolve_struct_name(orig_type_name)
                    }
                } else {
                    self.resolve_struct_name(orig_type_name)
                };
                let type_name = if self.types.structs.contains_key(&candidate) {
                    candidate
                } else if candidate.contains('$') {
                    let base = candidate
                        .split('$')
                        .next()
                        .unwrap_or(&candidate)
                        .to_string();
                    if self.types.structs.contains_key(&base) {
                        base
                    } else {
                        candidate
                    }
                } else {
                    candidate
                };
                if let Some(struct_info) = self.types.structs.get(&type_name).cloned() {
                    if let Some(field_idx) = struct_info
                        .fields
                        .iter()
                        .position(|(n, _)| n == &field.node)
                    {
                        let field_ty_raw = &struct_info.fields[field_idx].1;
                        let field_ty = if !self.generics.substitutions.is_empty() {
                            vais_types::substitute_type(field_ty_raw, &self.generics.substitutions)
                        } else {
                            field_ty_raw.clone()
                        };
                        let llvm_ty = self.type_to_llvm(&field_ty);
                        let (obj_val, obj_ir) = self.generate_expr(obj_expr, counter)?;
                        let mut ir = obj_ir;
                        let field_ptr = self.next_temp(counter);
                        use std::fmt::Write;
                        let _ = writeln!(
                            ir,
                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}",
                            field_ptr, type_name, type_name, obj_val, field_idx
                        );
                        self.fn_ctx
                            .record_emitted_type(&field_ptr, &format!("{}*", llvm_ty));
                        if matches!(
                            field_ty,
                            ResolvedType::Str | ResolvedType::Slice(_) | ResolvedType::SliceMut(_)
                        ) {
                            let loaded = self.next_temp(counter);
                            let _ = writeln!(
                                ir,
                                "  {} = load {}, {}* {}",
                                loaded, llvm_ty, llvm_ty, field_ptr
                            );
                            self.fn_ctx.record_emitted_type(&loaded, &llvm_ty);
                            return Ok((loaded, ir));
                        }
                        return Ok((field_ptr, ir));
                    }
                }
            }
        }

        // Phase 17.H4.7: `&<call>(...)` when the callee returns a named
        // struct value. The value lives in an SSA register; to take its
        // address, spill to a stack alloca and return the pointer.
        // Without this, we'd return the struct value as-is and a
        // receiving function expecting `%T*` would crash clang.
        if matches!(
            &inner.node,
            Expr::Call { .. } | Expr::MethodCall { .. } | Expr::StaticMethodCall { .. }
        ) {
            let inferred = self.infer_expr_type(inner);
            if matches!(inferred, ResolvedType::Named { .. }) {
                let llvm_ty = self.type_to_llvm(&inferred);
                // Only spill when the call actually returns a value type
                // (not a pointer / unit / primitive). We check that the
                // LLVM rendering starts with `%` (struct type ref).
                if llvm_ty.starts_with('%') {
                    let (val, val_ir) = self.generate_expr(inner, counter)?;
                    let mut ir = val_ir;
                    let slot = self.next_temp(counter);
                    self.emit_entry_alloca(&slot, &llvm_ty);
                    use std::fmt::Write;
                    let _ = writeln!(ir, "  store {} {}, {}* {}", llvm_ty, val, llvm_ty, slot);
                    return Ok((slot, ir));
                }
            }
        }
        // For complex expressions, evaluate and return
        self.generate_expr(inner, counter)
    }

    /// Generate a slice fat pointer `{ i8*, i64 }` from `&[elem, ...]`.
    #[inline(never)]
    fn generate_ref_array_slice(
        &mut self,
        elements: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let len = elements.len();
        let mut ir = String::new();

        // Infer element type
        let elem_ty = if let Some(first) = elements.first() {
            let resolved = self.infer_expr_type(first);
            self.type_to_llvm(&resolved)
        } else {
            "i64".to_string()
        };
        let arr_ty = format!("[{}  x {}]", len, elem_ty);

        // Allocate array on stack (hoisted to entry block)
        let arr_ptr = self.next_temp(counter);
        self.emit_entry_alloca(&arr_ptr, &arr_ty);

        // Store each element
        for (i, elem) in elements.iter().enumerate() {
            let (val, elem_ir) = self.generate_expr(elem, counter)?;
            ir.push_str(&elem_ir);

            let elem_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = getelementptr {}, {}* {}, i64 0, i64 {}",
                elem_ptr,
                arr_ty,
                arr_ty,
                arr_ptr,
                i
            );
            self.fn_ctx
                .record_emitted_type(&elem_ptr, &format!("{}*", elem_ty));
            write_ir!(ir, "  store {} {}, {}* {}", elem_ty, val, elem_ty, elem_ptr);
        }

        // Get pointer to first element
        let data_ptr = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = getelementptr {}, {}* {}, i64 0, i64 0",
            data_ptr,
            arr_ty,
            arr_ty,
            arr_ptr
        );
        self.fn_ctx
            .record_emitted_type(&data_ptr, &format!("{}*", elem_ty));

        // Bitcast to i8*
        let data_i8 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = bitcast {}* {} to i8*",
            data_i8,
            elem_ty,
            data_ptr
        );
        self.fn_ctx.record_emitted_type(&data_i8, "i8*");

        // Build fat pointer: { i8*, i64 }
        let fat1 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0",
            fat1,
            data_i8
        );
        self.fn_ctx.record_emitted_type(&fat1, "{ i8*, i64 }");
        let fat2 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = insertvalue {{ i8*, i64 }} {}, i64 {}, 1",
            fat2,
            fat1,
            len
        );
        self.fn_ctx.record_emitted_type(&fat2, "{ i8*, i64 }");

        Ok((fat2, ir))
    }

    /// Build a `{ i8*, i64 }` slice fat pointer from an array-typed local.
    ///
    /// The local was initialized from an array literal (`x := [a, b, c]`) so
    /// the alloca stores a pointer-to-element-0 (`T**`) or the SSA value IS
    /// the pointer (`T*`). We load the data pointer (if needed), bitcast to
    /// `i8*`, and combine with the compile-time-known length.
    #[inline(never)]
    pub(crate) fn generate_ref_array_local_slice(
        &mut self,
        local: &crate::types::LocalVar,
        len: u64,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();

        // Resolve element LLVM type. ResolvedType::Array(T) -> type_to_llvm gives "T*",
        // so strip one pointer level to get the element type.
        let arr_llvm = self.type_to_llvm(&local.ty);
        let elem_ty = arr_llvm
            .strip_suffix('*')
            .map(|s| s.to_string())
            .unwrap_or_else(|| "i64".to_string());

        // Load the data pointer. For an alloca storing `T*`, the alloca itself
        // is `T**` so we load once to get `T*`. SSA values already hold `T*`.
        let data_ptr = if local.is_alloca() {
            let loaded = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = load {}*, {}** %{}",
                loaded,
                elem_ty,
                elem_ty,
                local.llvm_name
            );
            loaded
        } else {
            format!("%{}", local.llvm_name)
        };

        // Bitcast to i8*
        let data_i8 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = bitcast {}* {} to i8*",
            data_i8,
            elem_ty,
            data_ptr
        );
        self.fn_ctx.record_emitted_type(&data_i8, "i8*");

        // Build fat pointer: { i8*, i64 }
        let fat1 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0",
            fat1,
            data_i8
        );
        self.fn_ctx.record_emitted_type(&fat1, "{ i8*, i64 }");
        let fat2 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = insertvalue {{ i8*, i64 }} {}, i64 {}, 1",
            fat2,
            fat1,
            len
        );
        self.fn_ctx.record_emitted_type(&fat2, "{ i8*, i64 }");

        Ok((fat2, ir))
    }

    /// Spill an SSA/Param value to stack to take its address.
    #[inline(never)]
    fn generate_ref_spill(
        &mut self,
        local: &crate::types::LocalVar,
        inner: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();
        let llvm_ty = self.type_to_llvm(&local.ty);
        let (val, val_ir) = self.generate_expr(inner, counter)?;
        ir.push_str(&val_ir);

        // If the local is a struct-typed SSA value whose LLVM name is an alloca
        // pointer (e.g., %__test_ptr from function entry, or %self for methods),
        // the value is already a pointer to the struct — return it directly.
        // The function entry code creates alloca + store for struct params, and
        // registers them as SSA with Named type but the value is %StructType*.
        // Also handles Param locals (e.g., `self` in methods passed as %Struct*).
        if matches!(&local.ty, ResolvedType::Named { .. }) && (local.is_ssa() || local.is_param()) {
            // The value is already a pointer (alloca result or self param pointer)
            return Ok((val, ir));
        }

        // Phase 17.H4 iter 18: for types whose LLVM lowering IS the fat
        // pointer value (`&str`, `&[T]`), the Vais `&x` is semantically a
        // reference but the LLVM ABI matches the **value** of x. Spilling
        // to an alloca and returning the alloca address produces
        // `{ptr, i64}*` where call sites expect `{ptr, i64}` value,
        // causing clang link errors
        // (`'%t0' defined with type 'ptr' but expected '{ ptr, i64 }'`).
        // Return the value directly to keep caller/callee ABI aligned.
        if matches!(
            &local.ty,
            ResolvedType::Str | ResolvedType::Slice(_) | ResolvedType::SliceMut(_)
        ) {
            return Ok((val, ir));
        }

        let tmp_alloca = self.next_temp(counter);
        self.emit_entry_alloca(&tmp_alloca, &llvm_ty);
        write_ir!(
            ir,
            "  store {} {}, {}* {}",
            llvm_ty,
            val,
            llvm_ty,
            tmp_alloca
        );
        Ok((tmp_alloca, ir))
    }

    /// Generate code for a dereference expression (`*expr`).
    #[inline(never)]
    pub(crate) fn generate_deref_expr(
        &mut self,
        inner: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (ptr_val, ptr_ir) = self.generate_expr(inner, counter)?;
        let mut ir = ptr_ir;

        // Infer the pointee type from the pointer expression
        let ptr_type = self.infer_expr_type(inner);
        if let ResolvedType::Named { name, generics } = &ptr_type {
            if name == "Box" && generics.len() == 1 {
                let pointee_ty = &generics[0];
                let pointee_llvm = self.type_to_llvm(pointee_ty);
                let box_llvm = self.type_to_llvm(&ptr_type);
                let inner_is_box_ptr_local = if let Expr::Ident(local_name) = &inner.node {
                    self.fn_ctx
                        .locals
                        .get(local_name.as_str())
                        .is_some_and(|local| {
                            local.is_alloca() || local.is_ssa() || local.is_param()
                        })
                } else {
                    false
                };
                let actual_ptr_ty = self.llvm_type_of_checked(&ptr_val);
                let ptr_i64 = if actual_ptr_ty.as_deref() == Some("i64")
                    || (actual_ptr_ty.is_none() && !inner_is_box_ptr_local)
                {
                    ptr_val
                } else if actual_ptr_ty.as_deref() == Some("ptr") || inner_is_box_ptr_local {
                    let loaded = self.next_temp(counter);
                    if box_llvm == "i64" {
                        write_ir!(ir, "  {} = load i64, i64* {}", loaded, ptr_val);
                    } else {
                        let field_ptr = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = getelementptr {}, {}* {}, i32 0, i32 0",
                            field_ptr,
                            box_llvm,
                            box_llvm,
                            ptr_val
                        );
                        write_ir!(ir, "  {} = load i64, i64* {}", loaded, field_ptr);
                    }
                    self.fn_ctx.record_emitted_type(&loaded, "i64");
                    loaded
                } else {
                    match actual_ptr_ty.as_deref() {
                        Some(actual) if actual.starts_with("%Box") && !actual.ends_with('*') => {
                            let extracted = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = extractvalue {} {}, 0",
                                extracted,
                                actual,
                                ptr_val
                            );
                            self.fn_ctx.record_emitted_type(&extracted, "i64");
                            extracted
                        }
                        Some(actual) if actual.starts_with("%Box") && actual.ends_with('*') => {
                            let field_ptr = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = getelementptr {}, {} {}, i32 0, i32 0",
                                field_ptr,
                                actual.trim_end_matches('*'),
                                actual,
                                ptr_val
                            );
                            let loaded = self.next_temp(counter);
                            write_ir!(ir, "  {} = load i64, i64* {}", loaded, field_ptr);
                            self.fn_ctx.record_emitted_type(&loaded, "i64");
                            loaded
                        }
                        _ => ptr_val,
                    }
                };
                let typed_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = inttoptr i64 {} to {}*",
                    typed_ptr,
                    ptr_i64,
                    pointee_llvm
                );
                self.fn_ctx
                    .record_emitted_type(&typed_ptr, &format!("{}*", pointee_llvm));
                let result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = load {}, {}* {}",
                    result,
                    pointee_llvm,
                    pointee_llvm,
                    typed_ptr
                );
                self.fn_ctx
                    .register_temp_type(result.as_str(), pointee_ty.clone());
                self.fn_ctx.record_emitted_type(&result, &pointee_llvm);
                return Ok((result, ir));
            }
        }
        let pointee_llvm = match &ptr_type {
            ResolvedType::Pointer(inner) => self.type_to_llvm(inner),
            ResolvedType::Ref(inner) => self.type_to_llvm(inner),
            ResolvedType::RefMut(inner) => self.type_to_llvm(inner),
            // Phase B5: `*v` where `v` is not a reference / pointer is a
            // no-op dereference (the source already has the value). Falling
            // back to `load i64` produces bogus IR like
            // `load i64, i64* %v_as_double` that clang rejects. Return the
            // value directly and let downstream consumers see the real type.
            _ => return Ok((ptr_val, ir)),
        };

        // L-019: `&T` parameters can be lowered as fat-pointer-by-value
        // (e.g. `{ i8*, i64 } %s` for `&str`) rather than `T*`. When the SSA
        // value's actual LLVM type already matches the pointee type, `*ref`
        // is a no-op — emitting `load %T, %T* %ssa_value` would produce
        // invalid IR (`load %T from a non-pointer SSA value`). Mirrors the
        // Phase B5 "value-not-pointer" fall-through above.
        if let Some(actual_ty) = self.llvm_type_of_checked(&ptr_val) {
            if actual_ty == pointee_llvm {
                return Ok((ptr_val, ir));
            }
        }

        let result = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = load {}, {}* {}",
            result,
            pointee_llvm,
            pointee_llvm,
            ptr_val
        );

        Ok((result, ir))
    }
}

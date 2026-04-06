//! Data structure expression helpers for CodeGenerator
//!
//! Contains array, tuple, struct literal, index, and field expression generation.

use crate::{
    format_did_you_mean, state::FunctionContext, suggest_similar, CodeGenerator, CodegenError,
    CodegenResult,
};
use vais_ast::{Expr, Spanned};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Register a ResolvedType for an index-load result based on its LLVM type string.
    ///
    /// This ensures downstream call-arg coercion (in generate_expr_call.rs) can detect
    /// width mismatches (e.g., i8 value passed to i64 parameter) by looking up
    /// `temp_var_types`. Without this, `has_known_type` is false and coercion is skipped.
    fn register_elem_type(fn_ctx: &mut FunctionContext, result: &str, elem_llvm_ty: &str) {
        let resolved = match elem_llvm_ty {
            "i8" => Some(ResolvedType::U8),
            "i16" => Some(ResolvedType::I16),
            "i32" => Some(ResolvedType::I32),
            "i64" => Some(ResolvedType::I64),
            "float" => Some(ResolvedType::F32),
            "double" => Some(ResolvedType::F64),
            "i1" => Some(ResolvedType::Bool),
            _ => None, // Str and Named handled separately by caller
        };
        if let Some(ty) = resolved {
            fn_ctx.register_temp_type(result, ty);
        }
    }

    #[inline(never)]
    pub(crate) fn generate_array_expr(
        &mut self,
        elements: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();
        let len = elements.len();

        // Infer element type from first element (default to i64)
        let elem_ty = if let Some(first) = elements.first() {
            let resolved = self.infer_expr_type(first);
            self.type_to_llvm(&resolved)
        } else {
            "i64".to_string()
        };
        let arr_ty = format!("[{}  x {}]", len, elem_ty);

        let arr_ptr = self.next_temp(counter);
        write_ir!(ir, "  {} = alloca {}", arr_ptr, arr_ty);

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
            write_ir!(ir, "  store {} {}, {}* {}", elem_ty, val, elem_ty, elem_ptr);
        }

        let result = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = getelementptr {}, {}* {}, i64 0, i64 0",
            result,
            arr_ty,
            arr_ty,
            arr_ptr
        );

        Ok((result, ir))
    }

    /// Generate tuple literal expression
    #[inline(never)]
    pub(crate) fn generate_tuple_expr(
        &mut self,
        elements: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();

        // Infer the LLVM type of each element so nested tuples (e.g., (2, 3) inside (1, (2, 3)))
        // are represented as struct types rather than i64.
        let elem_resolved_types: Vec<ResolvedType> =
            elements.iter().map(|e| self.infer_expr_type(e)).collect();
        let elem_llvm_types: Vec<String> = elem_resolved_types
            .iter()
            .map(|t| self.type_to_llvm(t))
            .collect();

        let tuple_ty = format!("{{ {} }}", elem_llvm_types.join(", "));

        let tuple_ptr = self.next_temp(counter);
        write_ir!(ir, "  {} = alloca {}", tuple_ptr, tuple_ty);

        for (i, elem) in elements.iter().enumerate() {
            let (val, elem_ir) = self.generate_expr(elem, counter)?;
            ir.push_str(&elem_ir);

            let elem_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = getelementptr {}, {}* {}, i32 0, i32 {}",
                elem_ptr,
                tuple_ty,
                tuple_ty,
                tuple_ptr,
                i
            );
            let elem_ty = &elem_llvm_types[i];
            write_ir!(ir, "  store {} {}, {}* {}", elem_ty, val, elem_ty, elem_ptr);
        }

        let result = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = load {}, {}* {}",
            result,
            tuple_ty,
            tuple_ty,
            tuple_ptr
        );

        Ok((result, ir))
    }

    /// Generate struct or union literal expression
    #[inline(never)]
    pub(crate) fn generate_struct_lit_expr(
        &mut self,
        name: &Spanned<String>,
        fields: &[(Spanned<String>, Spanned<Expr>)],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let type_name = &name.node;

        let resolved_name = self.resolve_struct_name(type_name);
        let type_name = &resolved_name;

        // First check if it's a struct
        if let Some(struct_info) = self.types.structs.get(type_name).cloned() {
            let mut ir = String::new();

            // Check if this struct has generic parameters
            // Collect generic parameters from struct fields
            let mut generic_params = Vec::new();
            for (_, field_ty) in &struct_info.fields {
                if let ResolvedType::Generic(param) = field_ty {
                    if !generic_params.contains(param) {
                        generic_params.push(param.clone());
                    }
                }
            }

            // If the struct is generic, infer concrete types from the field values
            let final_type_name = if !generic_params.is_empty() {
                let mut inferred_types = Vec::new();

                // For each generic parameter, find the first field that uses it and infer from the value
                for param in &generic_params {
                    let mut inferred = None;
                    for (field_name, field_expr) in fields {
                        // Find the field info
                        if let Some((_, ResolvedType::Generic(p))) = struct_info
                            .fields
                            .iter()
                            .find(|(name, _)| name == &field_name.node)
                        {
                            if p == param {
                                inferred = Some(self.infer_expr_type(field_expr));
                                break;
                            }
                        }
                    }
                    inferred_types.push(inferred.unwrap_or(ResolvedType::I64));
                }

                // Generate the mangled name with inferred types
                self.mangle_struct_name(type_name, &inferred_types)
            } else {
                type_name.to_string()
            };

            let struct_ptr = self.next_temp(counter);
            write_ir!(ir, "  {} = alloca %{}", struct_ptr, final_type_name);

            for (field_name, field_expr) in fields {
                let field_idx = struct_info
                    .fields
                    .iter()
                    .position(|(n, _)| n == &field_name.node)
                    .ok_or_else(|| {
                        let candidates: Vec<&str> =
                            struct_info.fields.iter().map(|(n, _)| n.as_str()).collect();
                        let suggestions = suggest_similar(&field_name.node, &candidates, 3);
                        let suggestion_text = format_did_you_mean(&suggestions);
                        CodegenError::TypeError(format!(
                            "Unknown field '{}' in struct '{}'{}",
                            field_name.node, type_name, suggestion_text
                        ))
                    })?;

                let (val, field_ir) = self.generate_expr(field_expr, counter)?;
                ir.push_str(&field_ir);

                let field_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}",
                    field_ptr,
                    final_type_name,
                    final_type_name,
                    struct_ptr,
                    field_idx
                );

                let field_ty = &struct_info.fields[field_idx].1;
                let llvm_ty = self.type_to_llvm(field_ty);

                // For struct-typed fields, val might be a pointer that needs to be loaded
                let val_to_store = if matches!(field_ty, ResolvedType::Named { .. })
                    && !self.is_expr_value(field_expr)
                {
                    // Field value is a pointer to struct, need to load the value
                    let loaded = self.next_temp(counter);
                    write_ir!(ir, "  {} = load {}, {}* {}", loaded, llvm_ty, llvm_ty, val);
                    loaded
                } else {
                    val
                };

                write_ir!(
                    ir,
                    "  store {} {}, {}* {}",
                    llvm_ty,
                    val_to_store,
                    llvm_ty,
                    field_ptr
                );
            }

            Ok((struct_ptr, ir))
        // Then check if it's a union
        } else if let Some(union_info) = self.types.unions.get(type_name).cloned() {
            let mut ir = String::new();

            // Allocate union on stack
            let union_ptr = self.next_temp(counter);
            write_ir!(ir, "  {} = alloca %{}", union_ptr, type_name);

            // Union should have exactly one field in the literal
            if fields.len() != 1 {
                return Err(CodegenError::TypeError(format!(
                    "Union literal should have exactly one field, got {}",
                    fields.len()
                )));
            }

            let (field_name, field_expr) = &fields[0];

            // Find field type
            let field_ty = union_info
                .fields
                .iter()
                .find(|(n, _)| n == &field_name.node)
                .map(|(_, ty)| ty.clone())
                .ok_or_else(|| {
                    let candidates: Vec<&str> =
                        union_info.fields.iter().map(|(n, _)| n.as_str()).collect();
                    let suggestions = suggest_similar(&field_name.node, &candidates, 3);
                    let suggestion_text = format_did_you_mean(&suggestions);
                    CodegenError::TypeError(format!(
                        "Unknown field '{}' in union '{}'{}",
                        field_name.node, type_name, suggestion_text
                    ))
                })?;

            let (val, field_ir) = self.generate_expr(field_expr, counter)?;
            ir.push_str(&field_ir);

            // Bitcast union pointer to field type pointer (all fields at offset 0)
            let field_llvm_ty = self.type_to_llvm(&field_ty);
            let field_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = bitcast %{}* {} to {}*",
                field_ptr,
                type_name,
                union_ptr,
                field_llvm_ty
            );

            // Store the value
            write_ir!(
                ir,
                "  store {} {}, {}* {}",
                field_llvm_ty,
                val,
                field_llvm_ty,
                field_ptr
            );

            Ok((union_ptr, ir))
        } else {
            // Fallback: check if type_name is an enum struct-variant (e.g., SqlType.Varchar
            // written unqualified as `Varchar { max_len: 255 }` inside match arms or return
            // expressions where the enum context is inferred by the type checker but the
            // parser produced a bare StructLit without enum_name).
            //
            // Scan all known enums for a variant with this name that has Struct fields.
            // If exactly one such enum contains this variant, delegate to the enum variant
            // constructor path (same handling as `EnumType.Variant { .. }` qualified form).
            let matching_enums: Vec<String> = self
                .types
                .enums
                .iter()
                .filter(|(_, einfo)| {
                    einfo.variants.iter().any(|v| {
                        v.name == *type_name
                            && matches!(v.fields, crate::types::EnumVariantFields::Struct(_))
                    })
                })
                .map(|(ename, _)| ename.clone())
                .collect();

            if matching_enums.len() == 1 {
                let enum_name = &matching_enums[0];
                return self.generate_enum_variant_struct(enum_name, type_name, fields, counter);
            } else if matching_enums.len() > 1 {
                return Err(CodegenError::TypeError(format!(
                    "Ambiguous struct-variant '{}': found in enums {:?}. Use qualified form \
                     `EnumType.{} {{ .. }}` to disambiguate.",
                    type_name, matching_enums, type_name
                )));
            }

            Err(CodegenError::TypeError(format!(
                "Unknown struct or union: {}",
                type_name
            )))
        }
    }

    /// Generate index expression
    #[inline(never)]
    pub(crate) fn generate_index_expr(
        &mut self,
        array: &Spanned<Expr>,
        index: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // If the index is a Range expression (e.g., arr[1..3], arr[..2]),
        // delegate to generate_slice instead of element access
        if let Expr::Range {
            start,
            end,
            inclusive,
        } = &index.node
        {
            return self.generate_slice(
                array,
                start.as_ref().map(|s| s.as_ref()),
                end.as_ref().map(|e| e.as_ref()),
                *inclusive,
                counter,
            );
        }

        let (arr_val, arr_ir) = self.generate_expr(array, counter)?;
        let (idx_val, idx_ir) = self.generate_expr(index, counter)?;

        let mut ir = arr_ir;
        ir.push_str(&idx_ir);

        // Infer element type for correct LLVM IR generation
        let arr_ty = self.infer_expr_type(array);
        // elem_resolved_ty holds the full ResolvedType for Vec element (used for struct detection)
        let (elem_llvm_ty, is_fat_ptr, elem_resolved_ty) = match arr_ty {
            vais_types::ResolvedType::Pointer(ref elem) => (self.type_to_llvm(elem), false, None),
            vais_types::ResolvedType::Array(ref elem) => (self.type_to_llvm(elem), false, None),
            vais_types::ResolvedType::Slice(ref elem)
            | vais_types::ResolvedType::SliceMut(ref elem) => (self.type_to_llvm(elem), true, None),
            // Vec<T>[idx] → element type T, access via data pointer
            vais_types::ResolvedType::Named {
                ref name,
                ref generics,
            } if name == "Vec" && !generics.is_empty() => {
                (self.type_to_llvm(&generics[0]), false, Some(generics[0].clone()))
            }
            // &Vec<T>[idx]
            vais_types::ResolvedType::Ref(ref inner)
            | vais_types::ResolvedType::RefMut(ref inner) => {
                match inner.as_ref() {
                    vais_types::ResolvedType::Named {
                        ref name,
                        ref generics,
                    } if name == "Vec" && !generics.is_empty() => {
                        (self.type_to_llvm(&generics[0]), false, Some(generics[0].clone()))
                    }
                    vais_types::ResolvedType::Slice(ref elem)
                    | vais_types::ResolvedType::SliceMut(ref elem) => {
                        (self.type_to_llvm(elem), true, None)
                    }
                    vais_types::ResolvedType::Array(ref elem) => (self.type_to_llvm(elem), false, None),
                    _ => {
                        // Treat as i64 pointer indexing
                        ("i64".to_string(), false, None)
                    }
                }
            }
            // Str indexing → byte access
            vais_types::ResolvedType::Str => ("i8".to_string(), true, None),
            // Named types (non-Vec) that may have operator overloading or custom indexing
            vais_types::ResolvedType::Named { .. }
            | vais_types::ResolvedType::Unknown
            | vais_types::ResolvedType::Generic(_) => {
                // Fallback: treat as i64 pointer for named/unknown types
                ("i64".to_string(), false, None)
            }
            ref other => {
                // Concrete non-indexable types (i64, f64, bool, etc.) — return error
                return Err(CodegenError::TypeError(format!(
                    "Cannot index into type '{}' — indexing requires an array, slice, pointer, Vec, or string type",
                    other
                )));
            }
        };

        // Extend index to i64 if necessary (i8, i16, i32 → i64)
        let idx_type = self.infer_expr_type(index);
        let idx_llvm = self.type_to_llvm(&idx_type);
        let idx_val =
            if idx_llvm != "i64" && (idx_llvm == "i8" || idx_llvm == "i16" || idx_llvm == "i32") {
                let ext = self.next_temp(counter);
                write_ir!(ir, "  {} = sext {} {} to i64", ext, idx_llvm, idx_val);
                ext
            } else {
                idx_val
            };

        // For fat pointer slices { i8*, i64 }, extract data pointer and bitcast
        // Also insert runtime bounds check: idx < len, abort on OOB
        let base_ptr = if is_fat_ptr {
            self.needs_bounds_check = true;
            // Extract length (field 1) for bounds check
            let len_val = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = extractvalue {{ i8*, i64 }} {}, 1",
                len_val,
                arr_val
            );

            // Bounds check: idx < len (unsigned comparison)
            let in_bounds = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = icmp ult i64 {}, {}",
                in_bounds,
                idx_val,
                len_val
            );

            let safe_label = self.next_label("bounds_safe");
            let oob_label = self.next_label("bounds_oob");
            write_ir!(
                ir,
                "  br i1 {}, label %{}, label %{}",
                in_bounds,
                safe_label,
                oob_label
            );

            // OOB path: abort
            write_ir!(ir, "{}:", oob_label);
            ir.push_str("  call void @abort()\n");
            ir.push_str("  unreachable\n");

            // Safe path: continue with element access
            write_ir!(ir, "{}:", safe_label);
            self.fn_ctx.current_block.clone_from(&safe_label);

            let data_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = extractvalue {{ i8*, i64 }} {}, 0",
                data_ptr,
                arr_val
            );
            let typed_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = bitcast i8* {} to {}*",
                typed_ptr,
                data_ptr,
                elem_llvm_ty
            );
            typed_ptr
        } else {
            // For Vec<T>, arr_val is a pointer to the Vec struct (%Vec*).
            // Extract the data pointer (field 0) and cast to element type pointer.
            let is_vec = matches!(&arr_ty,
                vais_types::ResolvedType::Named { name, .. } if name == "Vec")
                || matches!(&arr_ty,
                    vais_types::ResolvedType::Ref(inner) | vais_types::ResolvedType::RefMut(inner)
                    if matches!(inner.as_ref(), vais_types::ResolvedType::Named { name, .. } if name == "Vec"));
            if is_vec {
                // Load data pointer from Vec.data (field 0)
                let data_field = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr %Vec, %Vec* {}, i32 0, i32 0",
                    data_field,
                    arr_val
                );
                let data_i64 = self.next_temp(counter);
                write_ir!(ir, "  {} = load i64, i64* {}", data_i64, data_field);

                // Use runtime elem_size from Vec.elem_size (field 3) for correct stride.
                // This handles both specialized push (elem_size=16 for str) and
                // generic push (elem_size=8 for i64) transparently.
                let es_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr %Vec, %Vec* {}, i32 0, i32 3",
                    es_ptr,
                    arr_val
                );
                let elem_size_val = self.next_temp(counter);
                write_ir!(ir, "  {} = load i64, i64* {}", elem_size_val, es_ptr);
                let data_ptr = self.next_temp(counter);
                write_ir!(ir, "  {} = inttoptr i64 {} to i8*", data_ptr, data_i64);
                let byte_offset = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = mul i64 {}, {}",
                    byte_offset,
                    idx_val,
                    elem_size_val
                );
                let elem_ptr_i8 = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr i8, i8* {}, i64 {}",
                    elem_ptr_i8,
                    data_ptr,
                    byte_offset
                );
                let typed_elem_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = bitcast i8* {} to {}*",
                    typed_elem_ptr,
                    elem_ptr_i8,
                    elem_llvm_ty
                );

                // Bug 3 fix: if the element type is a struct (LLVM type starts with '%'),
                // allocate a local copy via alloca + memcpy so that field access GEP can
                // use a properly-typed struct pointer rather than a raw i64 value.
                if elem_llvm_ty.starts_with('%') {
                    let alloca = self.next_temp(counter);
                    self.emit_entry_alloca(&alloca, &elem_llvm_ty);
                    let alloca_i8 = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = bitcast {}* {} to i8*",
                        alloca_i8,
                        elem_llvm_ty,
                        alloca
                    );
                    write_ir!(
                        ir,
                        "  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {}, i8* {}, i64 {}, i1 false)",
                        alloca_i8,
                        elem_ptr_i8,
                        elem_size_val
                    );
                    // Register element type for field access / call-arg coercion
                    if let Some(ref resolved) = elem_resolved_ty {
                        self.fn_ctx.register_temp_type(&alloca, resolved.clone());
                    } else {
                        let struct_name = elem_llvm_ty.trim_start_matches('%');
                        self.fn_ctx.register_temp_type(
                            &alloca,
                            vais_types::ResolvedType::Named {
                                name: struct_name.to_string(),
                                generics: vec![],
                            },
                        );
                    }
                    return Ok((alloca, ir));
                }

                let result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = load {}, {}* {}",
                    result,
                    elem_llvm_ty,
                    elem_llvm_ty,
                    typed_elem_ptr
                );
                // Register element type so downstream call-arg coercion can detect width mismatch
                Self::register_elem_type(&mut self.fn_ctx, &result, &elem_llvm_ty);
                return Ok((result, ir));
            } else {
                arr_val.clone()
            }
        };

        let elem_ptr = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = getelementptr {}, {}* {}, i64 {}",
            elem_ptr,
            elem_llvm_ty,
            elem_llvm_ty,
            base_ptr,
            idx_val
        );

        let result = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = load {}, {}* {}",
            result,
            elem_llvm_ty,
            elem_llvm_ty,
            elem_ptr
        );

        // Register the element type for downstream codegen (e.g., Option/Result construction,
        // call-arg width coercion).
        Self::register_elem_type(&mut self.fn_ctx, &result, &elem_llvm_ty);
        if elem_llvm_ty == "{ i8*, i64 }" {
            // Override with precise Str type for fat pointer elements
            self.fn_ctx
                .register_temp_type(&result, vais_types::ResolvedType::Str);
        } else if elem_llvm_ty.starts_with('%') {
            // Bug 1 fix: Named struct type — use full ResolvedType from elem_resolved_ty when
            // available so that generics (e.g., Cell<bool> → Cell$bool) are preserved.
            // Hardcoding generics: vec![] would make specialized structs unfindable downstream.
            if let Some(ref resolved) = elem_resolved_ty {
                self.fn_ctx.register_temp_type(&result, resolved.clone());
            } else {
                let name = elem_llvm_ty.trim_start_matches('%');
                self.fn_ctx.register_temp_type(
                    &result,
                    vais_types::ResolvedType::Named {
                        name: name.to_string(),
                        generics: vec![],
                    },
                );
            }
        }

        Ok((result, ir))
    }

    /// Generate field access expression
    ///
    /// Supports both simple (`obj.field`) and nested (`obj.a.b.c`) field access
    /// by using `infer_expr_type` to determine the struct type of any sub-expression.
    #[inline(never)]
    pub(crate) fn generate_field_expr(
        &mut self,
        obj: &Spanned<Expr>,
        field: &Spanned<String>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Handle EnumType.Variant pattern: Ident("EnumName").field("VariantName")
        // The parser creates Field { expr: Ident(enum_name), field: variant_name }
        // for unit enum variant access like `DistanceMetric.Cosine`
        if let Expr::Ident(name) = &obj.node {
            if let Some(enum_info) = self.types.enums.get(name).cloned() {
                // Object is an enum type name — look up variant tag
                for (tag, variant) in enum_info.variants.iter().enumerate() {
                    if variant.name == field.node {
                        // Unit enum variant — return tag value
                        let mut ir = String::new();
                        let enum_ptr = self.next_temp(counter);
                        self.emit_entry_alloca(&enum_ptr, &format!("%{}", name));
                        let tag_ptr = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0",
                            tag_ptr, name, name, enum_ptr
                        );
                        write_ir!(ir, "  store i32 {}, i32* {}", tag, tag_ptr);
                        return Ok((enum_ptr, ir));
                    }
                }
                // Variant not found — check constants that start with enum name
                // e.g., PAGE_SIZE_DEFAULT might be a constant
            }
        }

        let (obj_val, obj_ir) = self.generate_expr(obj, counter)?;
        let mut ir = obj_ir;

        // Infer the type of the object expression (works for both Ident and nested Field)
        let obj_type = self.infer_expr_type(obj);

        // Unwrap Ref/RefMut/Pointer to get the inner Named type (for &self and *T patterns)
        let resolved_type = match &obj_type {
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) | ResolvedType::Pointer(inner) => inner.as_ref(),
            other => other,
        };

        if let ResolvedType::Named {
            name: orig_type_name,
            ..
        } = resolved_type
        {
            let type_name = self.resolve_struct_name(orig_type_name);
            // Bug 2 fix: resolve_struct_name may return an unresolved name if the struct
            // was not found in types.structs or struct_aliases. Also check generated_structs
            // (specialized types like Cell$bool) and fall back to the base struct name
            // (stripping the '$' mangling suffix) if still not found.
            let type_name = if self.types.structs.contains_key(&type_name) {
                type_name
            } else if self.generics.generated_structs.contains_key(&type_name)
                && self.types.structs.contains_key(&type_name)
            {
                type_name
            } else if type_name.contains('$') {
                // Mangled specialized name: try looking it up directly in types.structs
                // (generated_specialized_struct_type inserts with mangled name as key)
                if self.types.structs.contains_key(&type_name) {
                    type_name
                } else {
                    // Fall back to base name (before '$') to get field layout
                    let base = type_name.split('$').next().unwrap_or(&type_name).to_string();
                    if self.types.structs.contains_key(&base) { base } else { type_name }
                }
            } else {
                type_name
            };
            // First check if it's a struct
            if let Some(struct_info) = self.types.structs.get(&type_name).cloned() {
                let field_idx = struct_info
                    .fields
                    .iter()
                    .position(|(n, _)| n == &field.node)
                    .ok_or_else(|| {
                        let candidates: Vec<&str> =
                            struct_info.fields.iter().map(|(n, _)| n.as_str()).collect();
                        let suggestions = suggest_similar(&field.node, &candidates, 3);
                        let suggestion_text = format_did_you_mean(&suggestions);
                        CodegenError::TypeError(format!(
                            "Unknown field '{}' in struct '{}'{}",
                            field.node, type_name, suggestion_text
                        ))
                    })?;

                let field_ty_raw = &struct_info.fields[field_idx].1;
                // Apply generic substitutions if inside a specialized function body
                // (e.g., Cell<T>.value: T → bool when T=bool)
                let field_ty = if !self.generics.substitutions.is_empty() {
                    vais_types::substitute_type(field_ty_raw, &self.generics.substitutions)
                } else {
                    field_ty_raw.clone()
                };
                let llvm_ty = self.type_to_llvm(&field_ty);

                let field_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}",
                    field_ptr,
                    type_name,
                    type_name,
                    obj_val,
                    field_idx
                );

                // For struct-typed fields, return the pointer directly
                // (the caller or next field access will GEP into it)
                if matches!(field_ty, ResolvedType::Named { .. }) {
                    return Ok((field_ptr, ir));
                }

                let result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = load {}, {}* {}",
                    result,
                    llvm_ty,
                    llvm_ty,
                    field_ptr
                );

                return Ok((result, ir));
            }
            // Then check if it's a union
            else if let Some(union_info) = self.types.unions.get(&type_name).cloned() {
                let field_ty = union_info
                    .fields
                    .iter()
                    .find(|(n, _)| n == &field.node)
                    .map(|(_, ty)| ty.clone())
                    .ok_or_else(|| {
                        let candidates: Vec<&str> =
                            union_info.fields.iter().map(|(n, _)| n.as_str()).collect();
                        let suggestions = suggest_similar(&field.node, &candidates, 3);
                        let suggestion_text = format_did_you_mean(&suggestions);
                        CodegenError::TypeError(format!(
                            "Unknown field '{}' in union '{}'{}",
                            field.node, type_name, suggestion_text
                        ))
                    })?;

                let llvm_ty = self.type_to_llvm(&field_ty);

                // For union field access, bitcast union pointer to field type pointer
                // All fields share offset 0
                let field_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = bitcast %{}* {} to {}*",
                    field_ptr,
                    type_name,
                    obj_val,
                    llvm_ty
                );

                let result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = load {}, {}* {}",
                    result,
                    llvm_ty,
                    llvm_ty,
                    field_ptr
                );

                return Ok((result, ir));
            }
        }

        // Fallback: when the resolved type is Generic("T") or I64 (generic erasure),
        // try to determine the concrete struct type by:
        // 1. Resolving generic substitutions
        // 2. Looking at the Vec element type if obj is an Index into a Vec
        // 3. Searching all known structs for the field name as a last resort
        let fallback_type = match resolved_type {
            ResolvedType::Generic(t) => {
                // Try generic substitution first, but only if it resolves to a Named type
                // (struct/enum). If T maps to a primitive (e.g., U8 from Vec<u8> specialization),
                // that substitution is for a different Vec<T> in the same function scope.
                if let Some(concrete) = self.generics.substitutions.get(t) {
                    if matches!(concrete, ResolvedType::Named { .. }) {
                        Some(concrete.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            ResolvedType::I64 => None,
            _ => None,
        };

        // If generic substitution didn't help, try to resolve from the obj expression context
        let fallback_type = fallback_type.or_else(|| {
            // If obj is Index { expr: vec_expr, .. }, try the Vec's element type
            if let Expr::Index { expr: vec_expr, .. } = &obj.node {
                let vec_ty = self.infer_expr_type(vec_expr);
                let inner_ty = match &vec_ty {
                    ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref(),
                    other => other,
                };
                if let ResolvedType::Named { name, generics } = inner_ty {
                    if name == "Vec" && !generics.is_empty() {
                        let elem = &generics[0];
                        match elem {
                            ResolvedType::Named { .. } => Some(elem.clone()),
                            ResolvedType::Generic(t) => {
                                // Only use substitution if it resolves to a Named type
                                self.generics.substitutions.get(t).and_then(|c| {
                                    if matches!(c, ResolvedType::Named { .. }) {
                                        Some(c.clone())
                                    } else {
                                        None
                                    }
                                })
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        });

        // If we still couldn't resolve, search all known structs for the field name
        let fallback_type = fallback_type.or_else(|| {
            for (struct_name, struct_info) in &self.types.structs {
                if struct_info.fields.iter().any(|(n, _)| n == &field.node) {
                    return Some(ResolvedType::Named {
                        name: struct_name.clone(),
                        generics: vec![],
                    });
                }
            }
            None
        });

        if let Some(ResolvedType::Named { name: ref fallback_name, .. }) = fallback_type {
            let type_name = self.resolve_struct_name(fallback_name);
            let type_name = if self.types.structs.contains_key(&type_name) {
                type_name
            } else if type_name.contains('$') {
                let base = type_name.split('$').next().unwrap_or(&type_name).to_string();
                if self.types.structs.contains_key(&base) { base } else { type_name }
            } else {
                type_name
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

                    // obj_val is an i64 (erased struct value from Vec).
                    // Reinterpret as a pointer to the struct type, then GEP to the field.
                    let struct_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = inttoptr i64 {} to %{}*",
                        struct_ptr,
                        obj_val,
                        type_name
                    );

                    let field_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}",
                        field_ptr,
                        type_name,
                        type_name,
                        struct_ptr,
                        field_idx
                    );

                    if matches!(field_ty, ResolvedType::Named { .. }) {
                        return Ok((field_ptr, ir));
                    }

                    let result = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = load {}, {}* {}",
                        result,
                        llvm_ty,
                        llvm_ty,
                        field_ptr
                    );

                    return Ok((result, ir));
                }
            }
        }

        // Field access on non-struct/non-union type is an error
        let type_desc = format!("{}", self.infer_expr_type(obj));
        Err(CodegenError::TypeError(format!(
            "Cannot access field '{}' on type '{}' — field access requires a struct or union type",
            field.node, type_desc
        )))
    }
}

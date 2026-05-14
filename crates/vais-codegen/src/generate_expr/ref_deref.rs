//! Reference and dereference expression code generation.
//!
//! Extracted from `generate_expr_inner` match arms for `Expr::Ref` and
//! `Expr::Deref` to reduce the parent function's stack frame size.
//! Each handler is `#[inline(never)]` so Rust allocates its locals independently.

use vais_ast::*;
use vais_types::ResolvedType;

use crate::{format_did_you_mean, suggest_similar, CodeGenerator, CodegenError, CodegenResult};

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
                if local.is_alloca() {
                    if matches!(
                        &local.ty,
                        ResolvedType::Str
                            | ResolvedType::Slice(_)
                            | ResolvedType::SliceMut(_)
                            | ResolvedType::DynTrait { .. }
                    ) {
                        let loaded = self.next_temp(counter);
                        let llvm_ty = self.type_to_llvm(&local.ty);
                        let ir = format!(
                            "  {} = load {}, {}* %{}\n",
                            loaded, llvm_ty, llvm_ty, local.llvm_name
                        );
                        self.fn_ctx.register_temp_type(&loaded, local.ty);
                        return Ok((loaded, ir));
                    }
                    // Alloca variables already have an address.
                    return Ok((format!("%{}", local.llvm_name), String::new()));
                } else {
                    // SSA/Param values need to be spilled to stack to take their address
                    return self.generate_ref_spill(&local, inner, counter);
                }
            }
        }

        if let Expr::Field { expr, field } = &inner.node {
            if let Some(field_ref) = self.generate_ref_field_expr(expr, field, counter)? {
                return Ok(field_ref);
            }
        }

        // For complex expressions, evaluate and return
        self.generate_expr(inner, counter)
    }

    fn generate_ref_field_expr(
        &mut self,
        obj: &Spanned<Expr>,
        field: &Spanned<String>,
        counter: &mut usize,
    ) -> CodegenResult<Option<(String, String)>> {
        let (obj_val, obj_ir) = self.generate_field_base_expr(obj, counter)?;
        let mut ir = obj_ir;
        let obj_type = self.infer_expr_type(obj);
        let resolved_type = match &obj_type {
            ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner)
            | ResolvedType::Pointer(inner) => inner.as_ref(),
            other => other,
        };

        let ResolvedType::Named {
            name: orig_type_name,
            generics: type_generics,
        } = resolved_type
        else {
            return Ok(None);
        };

        let type_name = if !type_generics.is_empty()
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
        let type_name = if self.types.structs.contains_key(&type_name)
            || self.generics.generated_structs.contains_key(&type_name)
        {
            type_name
        } else if type_name.contains('$') {
            let base = type_name
                .split('$')
                .next()
                .unwrap_or(&type_name)
                .to_string();
            if self.types.structs.contains_key(&base) {
                base
            } else {
                type_name
            }
        } else {
            type_name
        };

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
            let field_ty = struct_info.fields[field_idx].1.clone();
            if matches!(
                field_ty,
                ResolvedType::Str
                    | ResolvedType::Slice(_)
                    | ResolvedType::SliceMut(_)
                    | ResolvedType::DynTrait { .. }
            ) {
                let loaded = self.next_temp(counter);
                let llvm_ty = self.type_to_llvm(&field_ty);
                write_ir!(
                    ir,
                    "  {} = load {}, {}* {}",
                    loaded,
                    llvm_ty,
                    llvm_ty,
                    field_ptr
                );
                self.fn_ctx.register_temp_type(&loaded, field_ty);
                return Ok(Some((loaded, ir)));
            }
            self.fn_ctx
                .register_temp_type(&field_ptr, ResolvedType::Pointer(Box::new(field_ty)));
            return Ok(Some((field_ptr, ir)));
        }

        if let Some(union_info) = self.types.unions.get(&type_name).cloned() {
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
            let field_ptr = self.next_temp(counter);
            write_ir!(
                ir,
                "  {} = bitcast %{}* {} to {}*",
                field_ptr,
                type_name,
                obj_val,
                llvm_ty
            );
            return Ok(Some((field_ptr, ir)));
        }

        Ok(None)
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

        // Bitcast to i8*
        let data_i8 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = bitcast {}* {} to i8*",
            data_i8,
            elem_ty,
            data_ptr
        );

        // Build fat pointer: { i8*, i64 }
        let fat1 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = insertvalue {{ i8*, i64 }} undef, i8* {}, 0",
            fat1,
            data_i8
        );
        let fat2 = self.next_temp(counter);
        write_ir!(
            ir,
            "  {} = insertvalue {{ i8*, i64 }} {}, i64 {}, 1",
            fat2,
            fat1,
            len
        );

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

        if matches!(
            &local.ty,
            ResolvedType::Str
                | ResolvedType::Slice(_)
                | ResolvedType::SliceMut(_)
                | ResolvedType::DynTrait { .. }
        ) {
            return Ok((val, ir));
        }

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
        self.fn_ctx.register_temp_type(
            &tmp_alloca,
            ResolvedType::Pointer(Box::new(local.ty.clone())),
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
        if matches!(
            &ptr_type,
            ResolvedType::Str
                | ResolvedType::Slice(_)
                | ResolvedType::SliceMut(_)
                | ResolvedType::DynTrait { .. }
        ) {
            return Ok((ptr_val, ir));
        }
        if matches!(
            &ptr_type,
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner)
                if matches!(
                    inner.as_ref(),
                    ResolvedType::Str
                        | ResolvedType::Slice(_)
                        | ResolvedType::SliceMut(_)
                        | ResolvedType::DynTrait { .. }
                )
        ) {
            return Ok((ptr_val, ir));
        }
        let pointee_llvm = match &ptr_type {
            ResolvedType::Pointer(inner) => self.type_to_llvm(inner),
            ResolvedType::Ref(inner) => self.type_to_llvm(inner),
            ResolvedType::RefMut(inner) => self.type_to_llvm(inner),
            ResolvedType::Named { .. } => self.type_to_llvm(&ptr_type),
            _ => "i64".to_string(),
        };

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

//! Data structure expression helpers for CodeGenerator
//!
//! Contains array, tuple, struct literal, index, and field expression generation.

use crate::{CodeGenerator, CodegenError, CodegenResult};
use vais_ast::{Expr, Spanned};
use vais_types::ResolvedType;

impl CodeGenerator {
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
        ir.push_str(&format!("  {} = alloca {}\n", arr_ptr, arr_ty));

        for (i, elem) in elements.iter().enumerate() {
            let (val, elem_ir) = self.generate_expr(elem, counter)?;
            ir.push_str(&elem_ir);

            let elem_ptr = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = getelementptr {}, {}* {}, i64 0, i64 {}\n",
                elem_ptr, arr_ty, arr_ty, arr_ptr, i
            ));
            ir.push_str(&format!(
                "  store {} {}, {}* {}\n",
                elem_ty, val, elem_ty, elem_ptr
            ));
        }

        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr {}, {}* {}, i64 0, i64 0\n",
            result, arr_ty, arr_ty, arr_ptr
        ));

        Ok((result, ir))
    }

    /// Generate tuple literal expression
    pub(crate) fn generate_tuple_expr(
        &mut self,
        elements: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();
        let len = elements.len();

        let tuple_ty = format!("{{ {} }}", vec!["i64"; len].join(", "));

        let tuple_ptr = self.next_temp(counter);
        ir.push_str(&format!("  {} = alloca {}\n", tuple_ptr, tuple_ty));

        for (i, elem) in elements.iter().enumerate() {
            let (val, elem_ir) = self.generate_expr(elem, counter)?;
            ir.push_str(&elem_ir);

            let elem_ptr = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = getelementptr {}, {}* {}, i32 0, i32 {}\n",
                elem_ptr, tuple_ty, tuple_ty, tuple_ptr, i
            ));
            ir.push_str(&format!("  store i64 {}, i64* {}\n", val, elem_ptr));
        }

        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = load {}, {}* {}\n",
            result, tuple_ty, tuple_ty, tuple_ptr
        ));

        Ok((result, ir))
    }

    /// Generate struct or union literal expression
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
            ir.push_str(&format!("  {} = alloca %{}\n", struct_ptr, final_type_name));

            for (field_name, field_expr) in fields {
                let field_idx = struct_info
                    .fields
                    .iter()
                    .position(|(n, _)| n == &field_name.node)
                    .ok_or_else(|| {
                        CodegenError::TypeError(format!(
                            "Unknown field '{}' in struct '{}'",
                            field_name.node, type_name
                        ))
                    })?;

                let (val, field_ir) = self.generate_expr(field_expr, counter)?;
                ir.push_str(&field_ir);

                let field_ptr = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}\n",
                    field_ptr, final_type_name, final_type_name, struct_ptr, field_idx
                ));

                let field_ty = &struct_info.fields[field_idx].1;
                let llvm_ty = self.type_to_llvm(field_ty);

                // For struct-typed fields, val might be a pointer that needs to be loaded
                let val_to_store = if matches!(field_ty, ResolvedType::Named { .. })
                    && !self.is_expr_value(field_expr)
                {
                    // Field value is a pointer to struct, need to load the value
                    let loaded = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}\n",
                        loaded, llvm_ty, llvm_ty, val
                    ));
                    loaded
                } else {
                    val
                };

                ir.push_str(&format!(
                    "  store {} {}, {}* {}\n",
                    llvm_ty, val_to_store, llvm_ty, field_ptr
                ));
            }

            Ok((struct_ptr, ir))
        // Then check if it's a union
        } else if let Some(union_info) = self.types.unions.get(type_name).cloned() {
            let mut ir = String::new();

            // Allocate union on stack
            let union_ptr = self.next_temp(counter);
            ir.push_str(&format!("  {} = alloca %{}\n", union_ptr, type_name));

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
                    CodegenError::TypeError(format!(
                        "Unknown field '{}' in union '{}'",
                        field_name.node, type_name
                    ))
                })?;

            let (val, field_ir) = self.generate_expr(field_expr, counter)?;
            ir.push_str(&field_ir);

            // Bitcast union pointer to field type pointer (all fields at offset 0)
            let field_llvm_ty = self.type_to_llvm(&field_ty);
            let field_ptr = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = bitcast %{}* {} to {}*\n",
                field_ptr, type_name, union_ptr, field_llvm_ty
            ));

            // Store the value
            ir.push_str(&format!(
                "  store {} {}, {}* {}\n",
                field_llvm_ty, val, field_llvm_ty, field_ptr
            ));

            Ok((union_ptr, ir))
        } else {
            Err(CodegenError::TypeError(format!(
                "Unknown struct or union: {}",
                type_name
            )))
        }
    }

    /// Generate index expression
    pub(crate) fn generate_index_expr(
        &mut self,
        array: &Spanned<Expr>,
        index: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (arr_val, arr_ir) = self.generate_expr(array, counter)?;
        let (idx_val, idx_ir) = self.generate_expr(index, counter)?;

        let mut ir = arr_ir;
        ir.push_str(&idx_ir);

        // Infer element type for correct LLVM IR generation
        let arr_ty = self.infer_expr_type(array);
        let elem_llvm_ty = match arr_ty {
            vais_types::ResolvedType::Pointer(ref elem) => self.type_to_llvm(elem),
            vais_types::ResolvedType::Array(ref elem) => self.type_to_llvm(elem),
            _ => "i64".to_string(),
        };

        let elem_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr {}, {}* {}, i64 {}\n",
            elem_ptr, elem_llvm_ty, elem_llvm_ty, arr_val, idx_val
        ));

        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = load {}, {}* {}\n",
            result, elem_llvm_ty, elem_llvm_ty, elem_ptr
        ));

        Ok((result, ir))
    }

    /// Generate field access expression
    pub(crate) fn generate_field_expr(
        &mut self,
        obj: &Spanned<Expr>,
        field: &Spanned<String>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (obj_val, obj_ir) = self.generate_expr(obj, counter)?;
        let mut ir = obj_ir;

        if let Expr::Ident(var_name) = &obj.node {
            if let Some(local) = self.fn_ctx.locals.get(var_name.as_str()).cloned() {
                if let ResolvedType::Named {
                    name: orig_type_name,
                    ..
                } = &local.ty
                {
                    let type_name = &self.resolve_struct_name(orig_type_name);
                    // First check if it's a struct
                    if let Some(struct_info) = self.types.structs.get(type_name).cloned() {
                        let field_idx = struct_info
                            .fields
                            .iter()
                            .position(|(n, _)| n == &field.node)
                            .ok_or_else(|| {
                                CodegenError::TypeError(format!(
                                    "Unknown field '{}' in struct '{}'",
                                    field.node, type_name
                                ))
                            })?;

                        let field_ty = &struct_info.fields[field_idx].1;
                        let llvm_ty = self.type_to_llvm(field_ty);

                        let field_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}\n",
                            field_ptr, type_name, type_name, obj_val, field_idx
                        ));

                        let result = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = load {}, {}* {}\n",
                            result, llvm_ty, llvm_ty, field_ptr
                        ));

                        return Ok((result, ir));
                    }
                    // Then check if it's a union
                    else if let Some(union_info) = self.types.unions.get(type_name).cloned() {
                        let field_ty = union_info
                            .fields
                            .iter()
                            .find(|(n, _)| n == &field.node)
                            .map(|(_, ty)| ty.clone())
                            .ok_or_else(|| {
                                CodegenError::TypeError(format!(
                                    "Unknown field '{}' in union '{}'",
                                    field.node, type_name
                                ))
                            })?;

                        let llvm_ty = self.type_to_llvm(&field_ty);

                        // For union field access, bitcast union pointer to field type pointer
                        // All fields share offset 0
                        let field_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = bitcast %{}* {} to {}*\n",
                            field_ptr, type_name, obj_val, llvm_ty
                        ));

                        let result = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = load {}, {}* {}\n",
                            result, llvm_ty, llvm_ty, field_ptr
                        ));

                        return Ok((result, ir));
                    }
                }
            }
        }

        Err(CodegenError::Unsupported(
            "field access requires known struct or union type".to_string(),
        ))
    }

}

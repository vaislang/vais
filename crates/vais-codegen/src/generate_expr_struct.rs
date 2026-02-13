//! Struct and union literal expression code generation for LLVM IR.

use vais_ast::{Expr, Spanned};
use vais_types::ResolvedType;

use crate::{
    format_did_you_mean, suggest_similar, CodeGenerator, CodegenError, CodegenResult,
};

impl CodeGenerator {
    /// Handle struct and union literal expressions
    pub(crate) fn generate_expr_struct_lit(
        &mut self,
        name: &Spanned<String>,
        fields: &[(Spanned<String>, Spanned<Expr>)],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let resolved_name = self.resolve_struct_name(&name.node);
        let type_name = &resolved_name;

        // First check if it's a struct
        if let Some(struct_info) = self.types.structs.get(type_name).cloned() {
            let mut ir = String::new();

            // Allocate struct on stack
            let struct_ptr = self.next_temp(counter);
            ir.push_str(&format!("  {} = alloca %{}\n", struct_ptr, type_name));

            // Store each field
            for (field_name, field_expr) in fields {
                // Find field index
                let field_idx = struct_info
                    .fields
                    .iter()
                    .position(|(n, _)| n == &field_name.node)
                    .ok_or_else(|| {
                        let candidates: Vec<&str> = struct_info
                            .fields
                            .iter()
                            .map(|(name, _)| name.as_str())
                            .collect();
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
                ir.push_str(&format!(
                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}\n",
                    field_ptr, type_name, type_name, struct_ptr, field_idx
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

            // Return pointer to struct
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

            // Return pointer to union
            Ok((union_ptr, ir))
        } else {
            Err(CodegenError::TypeError(format!(
                "Unknown struct or union: {}",
                type_name
            )))
        }
    }
}

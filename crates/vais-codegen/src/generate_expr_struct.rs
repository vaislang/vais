//! Struct and union literal expression code generation for LLVM IR.

use vais_ast::{Expr, Spanned};
use vais_types::ResolvedType;

use crate::{format_did_you_mean, suggest_similar, CodeGenerator, CodegenError, CodegenResult};

impl CodeGenerator {
    /// Handle struct and union literal expressions
    #[inline(never)]
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
            // Check if the struct has generic field types and we have active substitutions.
            // If so, try to find the specialized struct (e.g., Vec$f32 instead of Vec) so
            // that alloca/GEP use the correct specialized layout.
            let has_generic_fields = struct_info.fields.iter().any(|(_, ty)| {
                matches!(
                    ty,
                    ResolvedType::Generic(_) | ResolvedType::Var(_)
                )
            });
            let (effective_type_name, effective_fields) = if has_generic_fields {
                if !self.generics.substitutions.is_empty() {
                    // Case 1: Inside a specialized function — use substitutions to resolve
                    let concrete_types: Vec<ResolvedType> = struct_info
                        .fields
                        .iter()
                        .filter_map(|(_, ty)| {
                            if let ResolvedType::Generic(param) = ty {
                                self.generics.substitutions.get(param).cloned()
                            } else {
                                None
                            }
                        })
                        .collect();

                    if !concrete_types.is_empty() {
                        let mangled = self.mangle_struct_name(&name.node, &concrete_types);
                        if let Some(specialized) = self.types.structs.get(&mangled).cloned() {
                            (mangled, specialized.fields)
                        } else {
                            // Fallback: substitute field types in place
                            let subst_fields: Vec<(String, ResolvedType)> = struct_info
                                .fields
                                .iter()
                                .map(|(n, ty)| {
                                    let concrete =
                                        vais_types::substitute_type(ty, &self.generics.substitutions);
                                    (n.clone(), concrete)
                                })
                                .collect();
                            let alias_name = self
                                .generics
                                .struct_aliases
                                .get(&name.node)
                                .cloned()
                                .unwrap_or_else(|| type_name.clone());
                            (alias_name, subst_fields)
                        }
                    } else {
                        (type_name.clone(), struct_info.fields.clone())
                    }
                } else {
                    // No substitutions active — use base struct layout (i64-uniform).
                    // This handles base generic methods (Vec_new) where params are i64.
                    (type_name.clone(), struct_info.fields.clone())
                }
            } else {
                (type_name.clone(), struct_info.fields.clone())
            };

            let mut ir = String::new();

            // Allocate struct on stack (hoisted to entry block)
            let struct_ptr = self.next_temp(counter);
            self.emit_entry_alloca(&struct_ptr, &format!("%{}", effective_type_name));

            // Phase 191 #2b: zero-initialize the trailing __ownership_mask field
            // for structs that carry heap-owned string fields. The bitmap field
            // is appended to the LLVM layout but not to `effective_fields`, so
            // its GEP index is `effective_fields.len()`. #2b-D will OR per-field
            // bits into this slot at literal time.
            let effective_has_owned_mask = self
                .types
                .structs
                .get(&effective_type_name)
                .is_some_and(|si| si.has_owned_mask);
            if effective_has_owned_mask {
                let mask_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}",
                    mask_ptr,
                    effective_type_name,
                    effective_type_name,
                    struct_ptr,
                    effective_fields.len()
                );
                write_ir!(ir, "  store i64 0, i64* {}", mask_ptr);
            }

            // Store each field
            for (field_name, field_expr) in fields {
                // Find field index
                let field_idx = effective_fields
                    .iter()
                    .position(|(n, _)| n == &field_name.node)
                    .ok_or_else(|| {
                        let candidates: Vec<&str> = effective_fields
                            .iter()
                            .map(|(name, _)| name.as_str())
                            .collect();
                        let suggestions = suggest_similar(&field_name.node, &candidates, 3);
                        let suggestion_text = format_did_you_mean(&suggestions);
                        CodegenError::TypeError(format!(
                            "Unknown field '{}' in struct '{}'{}",
                            field_name.node, effective_type_name, suggestion_text
                        ))
                    })?;

                let (val, field_ir) = self.generate_expr(field_expr, counter)?;
                ir.push_str(&field_ir);

                let field_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}",
                    field_ptr,
                    effective_type_name,
                    effective_type_name,
                    struct_ptr,
                    field_idx
                );

                let field_ty = &effective_fields[field_idx].1;
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

                // Coerce value width to match field type (e.g., i8 param stored to i64 field
                // in specialized function body using generic struct layout)
                let val_llvm_ty = self.llvm_type_of(&val_to_store);
                let coerced_val =
                    self.coerce_int_width(&val_to_store, &val_llvm_ty, &llvm_ty, counter, &mut ir);
                // Handle Unit/void params: when the resolved type is Unit (void) but the
                // actual LLVM param is i8, coerce_int_width won't catch this because
                // llvm_type_of returns "void". Explicitly zext i8→i64 for Unit params.
                let coerced_val =
                    if llvm_ty == "i64" && (val_llvm_ty == "void" || val_llvm_ty == "i8") {
                        let local_name = coerced_val.strip_prefix('%').unwrap_or(&coerced_val);
                        if self
                            .fn_ctx
                            .locals
                            .get(local_name)
                            .is_some_and(|l| l.is_param())
                        {
                            let tmp = self.next_temp(counter);
                            write_ir!(ir, "  {} = zext i8 {} to i64", tmp, coerced_val);
                            tmp
                        } else {
                            coerced_val
                        }
                    } else {
                        coerced_val
                    };
                write_ir!(
                    ir,
                    "  store {} {}, {}* {}",
                    llvm_ty,
                    coerced_val,
                    llvm_ty,
                    field_ptr
                );
            }

            // Return pointer to struct
            Ok((struct_ptr, ir))
        // Then check if it's a union
        } else if let Some(union_info) = self.types.unions.get(type_name).cloned() {
            let mut ir = String::new();

            // Allocate union on stack (hoisted to entry block)
            let union_ptr = self.next_temp(counter);
            self.emit_entry_alloca(&union_ptr, &format!("%{}", type_name));

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

            // Return pointer to union
            Ok((union_ptr, ir))
        } else {
            // Fallback: short-form enum struct-variant construction (e.g.
            // `IntVal { v }` instead of `SqlValue.IntVal { v }`). The type
            // checker resolves the enum context via inference, but the parser
            // produces a bare `StructLit` and codegen reaches this point with
            // `type_name` pointing at the variant name. If exactly one enum has
            // a struct-variant with this name, delegate to the enum variant
            // constructor path used by the `EnumType.Variant { .. }` case.
            use crate::types::EnumVariantFields;
            let matching_enums: Vec<String> = self
                .types
                .enums
                .iter()
                .filter(|(_, einfo)| {
                    einfo.variants.iter().any(|v| {
                        v.name == *type_name && matches!(v.fields, EnumVariantFields::Struct(_))
                    })
                })
                .map(|(ename, _)| ename.clone())
                .collect();

            if matching_enums.len() == 1 {
                let enum_name = matching_enums[0].clone();
                return self.generate_enum_variant_struct(&enum_name, type_name, fields, counter);
            } else if matching_enums.len() > 1 {
                return Err(CodegenError::TypeError(format!(
                    "Ambiguous struct-variant '{}': found in enums {:?}. \
                     Use qualified form `EnumType.{} {{ .. }}` to disambiguate.",
                    type_name, matching_enums, type_name
                )));
            }

            Err(CodegenError::TypeError(format!(
                "Unknown struct or union: {}",
                type_name
            )))
        }
    }

    /// Generate enum struct variant construction: Shape.Circle { radius: 5.0 }
    #[inline(never)]
    pub(crate) fn generate_enum_variant_struct(
        &mut self,
        enum_name: &str,
        variant_name: &str,
        fields: &[(Spanned<String>, Spanned<Expr>)],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let tag = self.get_enum_variant_tag(variant_name);

        // Convert named fields to positional args
        let spanned_fields: Vec<Spanned<Expr>> =
            fields.iter().map(|(_, expr)| expr.clone()).collect();

        self.generate_enum_variant_constructor(enum_name, tag, &spanned_fields, counter)
    }
}

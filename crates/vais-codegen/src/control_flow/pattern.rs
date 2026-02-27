use super::*;
use crate::types::LocalVar;
use std::fmt::Write;

impl CodeGenerator {
    /// Generate code to check if a pattern matches (with explicit match type for correct type
    /// propagation, especially for nested tuple patterns where element types may be non-i64).
    pub(crate) fn generate_pattern_check_typed(
        &mut self,
        pattern: &Spanned<Pattern>,
        match_val: &str,
        counter: &mut usize,
        match_type: &ResolvedType,
    ) -> CodegenResult<(String, String)> {
        match &pattern.node {
            Pattern::Wildcard => {
                // Wildcard always matches
                Ok(("1".to_string(), String::new()))
            }
            Pattern::Ident(name) => {
                // Check if this is a unit enum variant (like None, Some without args)
                // If so, we need to check the discriminant, not just match anything
                if self.is_unit_enum_variant(name) {
                    // Generate discriminant check for unit enum variant
                    let mut ir = String::new();

                    // Get the enum type name for proper LLVM type reference
                    let enum_name = self
                        .get_enum_name_for_variant(name)
                        .unwrap_or_else(|| "Unknown".to_string());

                    // Get the tag from the enum value (first field at index 0)
                    let tag_ptr = self.next_temp(counter);
                    writeln!(
                        ir,
                        "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0",
                        tag_ptr, enum_name, enum_name, match_val
                    )
                    .unwrap();

                    let tag_val = self.next_temp(counter);
                    writeln!(ir, "  {} = load i32, i32* {}", tag_val, tag_ptr).unwrap();

                    // Find the expected tag value for this variant
                    let expected_tag = self.get_enum_variant_tag(name);

                    // Compare tag
                    let result = self.next_temp(counter);
                    writeln!(
                        ir,
                        "  {} = icmp eq i32 {}, {}",
                        result, tag_val, expected_tag
                    )
                    .unwrap();

                    Ok((result, ir))
                } else {
                    // Identifier pattern always matches (binding)
                    Ok(("1".to_string(), String::new()))
                }
            }
            Pattern::Literal(lit) => match lit {
                Literal::Int(n) => {
                    let result = self.next_temp(counter);
                    let ir = format!("  {} = icmp eq i64 {}, {}\n", result, match_val, n);
                    Ok((result, ir))
                }
                Literal::Bool(b) => {
                    let lit_val = if *b { "1" } else { "0" };
                    let result = self.next_temp(counter);
                    let ir = format!("  {} = icmp eq i64 {}, {}\n", result, match_val, lit_val);
                    Ok((result, ir))
                }
                Literal::Float(f) => {
                    let result = self.next_temp(counter);
                    let ir = format!("  {} = fcmp oeq double {}, {:e}\n", result, match_val, f);
                    Ok((result, ir))
                }
                Literal::String(s) => {
                    // String comparison using strcmp
                    let mut ir = String::new();

                    // Create string constant for the pattern
                    let const_name = format!(".str_pat.{}", self.strings.counter);
                    self.strings.counter += 1;
                    self.strings.constants.push((const_name.clone(), s.clone()));

                    // Get pointer to the constant string
                    let str_ptr = self.next_temp(counter);
                    let str_len = s.len() + 1;
                    writeln!(
                        ir,
                        "  {} = getelementptr [{} x i8], [{} x i8]* @{}, i32 0, i32 0",
                        str_ptr, str_len, str_len, const_name
                    )
                    .unwrap();

                    // Call strcmp: int strcmp(const char* s1, const char* s2)
                    // Returns 0 if strings are equal
                    let cmp_result = self.next_temp(counter);
                    writeln!(
                        ir,
                        "  {} = call i32 @strcmp(i8* {}, i8* {})",
                        cmp_result, match_val, str_ptr
                    )
                    .unwrap();

                    // Check if strcmp returned 0 (equal)
                    let result = self.next_temp(counter);
                    writeln!(ir, "  {} = icmp eq i32 {}, 0", result, cmp_result).unwrap();

                    Ok((result, ir))
                }
            },
            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let mut ir = String::new();

                // Check lower bound
                let lower_check = if let Some(start_pat) = start {
                    if let Pattern::Literal(Literal::Int(n)) = &start_pat.node {
                        let tmp = self.next_temp(counter);
                        writeln!(ir, "  {} = icmp sge i64 {}, {}", tmp, match_val, n).unwrap();
                        tmp
                    } else {
                        "1".to_string()
                    }
                } else {
                    "1".to_string()
                };

                // Check upper bound
                let upper_check = if let Some(end_pat) = end {
                    if let Pattern::Literal(Literal::Int(n)) = &end_pat.node {
                        let tmp = self.next_temp(counter);
                        let cmp = if *inclusive { "icmp sle" } else { "icmp slt" };
                        writeln!(ir, "  {} = {} i64 {}, {}", tmp, cmp, match_val, n).unwrap();
                        tmp
                    } else {
                        "1".to_string()
                    }
                } else {
                    "1".to_string()
                };

                // Combine checks
                let result = self.next_temp(counter);
                writeln!(ir, "  {} = and i1 {}, {}", result, lower_check, upper_check).unwrap();

                Ok((result, ir))
            }
            Pattern::Or(patterns) => {
                let mut ir = String::new();
                let mut checks: Vec<String> = Vec::new();

                for pat in patterns {
                    let (check, check_ir) =
                        self.generate_pattern_check_typed(pat, match_val, counter, match_type)?;
                    ir.push_str(&check_ir);
                    checks.push(check);
                }

                // OR all checks together
                // Or-patterns should always have at least one pattern, but defend against empty
                if checks.is_empty() {
                    return Ok(("1".to_string(), ir));
                }
                let mut result = checks[0].clone();
                for check in checks.iter().skip(1) {
                    let tmp = self.next_temp(counter);
                    writeln!(ir, "  {} = or i1 {}, {}", tmp, result, check).unwrap();
                    result = tmp;
                }

                Ok((result, ir))
            }
            Pattern::Tuple(patterns) => {
                // For tuple patterns, we need to extract and check each element.
                // Use the actual element types from match_type to generate correct extractvalue
                // instructions — especially important for nested tuples where elements may be
                // struct types (e.g., { i64, i64 }) rather than plain i64.
                let mut ir = String::new();
                let mut checks: Vec<String> = Vec::new();

                // Get element types from match_type if it is a Tuple, otherwise fall back to i64.
                let elem_types: Vec<ResolvedType> = if let ResolvedType::Tuple(elems) = match_type {
                    elems.clone()
                } else {
                    vec![ResolvedType::I64; patterns.len()]
                };

                // Build the full LLVM struct type string for extractvalue.
                let llvm_elem_strs: Vec<String> =
                    elem_types.iter().map(|t| self.type_to_llvm(t)).collect();
                let tuple_llvm_ty = format!("{{ {} }}", llvm_elem_strs.join(", "));

                for (i, pat) in patterns.iter().enumerate() {
                    // Extract tuple element using the correct aggregate type.
                    let elem = self.next_temp(counter);
                    writeln!(
                        ir,
                        "  {} = extractvalue {} {}, {}",
                        elem, tuple_llvm_ty, match_val, i
                    )
                    .unwrap();

                    // Recurse with the element's actual type.
                    let elem_ty = elem_types.get(i).cloned().unwrap_or(ResolvedType::I64);
                    let (check, check_ir) =
                        self.generate_pattern_check_typed(pat, &elem, counter, &elem_ty)?;
                    ir.push_str(&check_ir);
                    checks.push(check);
                }

                // AND all checks together
                // Empty tuple pattern (unit) matches unconditionally
                if checks.is_empty() {
                    return Ok(("1".to_string(), ir));
                }
                let mut result = checks[0].clone();
                for check in checks.iter().skip(1) {
                    let tmp = self.next_temp(counter);
                    writeln!(ir, "  {} = and i1 {}, {}", tmp, result, check).unwrap();
                    result = tmp;
                }
                Ok((result, ir))
            }
            Pattern::Variant { name, fields: _ } => {
                // Enum variant pattern: check the tag matches
                // Enum value is a struct %EnumName { i32 tag, ... payload }
                // Extract the tag and compare
                let mut ir = String::new();
                let variant_name = &name.node;

                // Get the enum type name for proper LLVM type reference
                let enum_name = self
                    .get_enum_name_for_variant(variant_name)
                    .unwrap_or_else(|| "Unknown".to_string());

                // Get the tag from the enum value (first field at index 0)
                let tag_ptr = self.next_temp(counter);
                writeln!(
                    ir,
                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0",
                    tag_ptr, enum_name, enum_name, match_val
                )
                .unwrap();

                let tag_val = self.next_temp(counter);
                writeln!(ir, "  {} = load i32, i32* {}", tag_val, tag_ptr).unwrap();

                // Find the expected tag value for this variant
                let expected_tag = self.get_enum_variant_tag(variant_name);

                // Compare tag
                let result = self.next_temp(counter);
                writeln!(
                    ir,
                    "  {} = icmp eq i32 {}, {}",
                    result, tag_val, expected_tag
                )
                .unwrap();

                Ok((result, ir))
            }
            Pattern::Struct { name, fields } => {
                // Struct pattern: always matches if type is correct, but we check field patterns
                let struct_name = &name.node;
                let mut ir = String::new();
                let mut checks: Vec<String> = Vec::new();

                if let Some(struct_info) = self.types.structs.get(struct_name).cloned() {
                    for (field_name, field_pat) in fields {
                        // Find field index
                        if let Some(field_idx) = struct_info
                            .fields
                            .iter()
                            .position(|(n, _)| n == &field_name.node)
                        {
                            if let Some(pat) = field_pat {
                                // Extract field value and check pattern
                                let field_ptr = self.next_temp(counter);
                                writeln!(
                                    ir,
                                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}",
                                    field_ptr, struct_name, struct_name, match_val, field_idx
                                )
                                .unwrap();

                                let field_val = self.next_temp(counter);
                                let field_ty = &struct_info.fields[field_idx].1;
                                let llvm_ty = self.type_to_llvm(field_ty);
                                writeln!(
                                    ir,
                                    "  {} = load {}, {}* {}",
                                    field_val, llvm_ty, llvm_ty, field_ptr
                                )
                                .unwrap();

                                let (check, check_ir) =
                                    self.generate_pattern_check(pat, &field_val, counter)?;
                                ir.push_str(&check_ir);
                                checks.push(check);
                            }
                        }
                    }
                }

                // AND all checks together
                if checks.is_empty() {
                    Ok(("1".to_string(), ir))
                } else {
                    let mut result = checks[0].clone();
                    for check in checks.iter().skip(1) {
                        let tmp = self.next_temp(counter);
                        writeln!(ir, "  {} = and i1 {}, {}", tmp, result, check).unwrap();
                        result = tmp;
                    }
                    Ok((result, ir))
                }
            }
            Pattern::Alias { pattern, .. } => {
                // For pattern alias, check the inner pattern
                self.generate_pattern_check(pattern, match_val, counter)
            }
        }
    }

    /// Generate code to check if a pattern matches (untyped: assumes i64 elements for tuples).
    /// Prefer `generate_pattern_check_typed` when the match type is known.
    pub(crate) fn generate_pattern_check(
        &mut self,
        pattern: &Spanned<Pattern>,
        match_val: &str,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        self.generate_pattern_check_typed(pattern, match_val, counter, &ResolvedType::I64)
    }

    /// Get the tag value for an enum variant
    pub(crate) fn get_enum_variant_tag(&self, variant_name: &str) -> i32 {
        for enum_info in self.types.enums.values() {
            for (i, variant) in enum_info.variants.iter().enumerate() {
                if variant.name == variant_name {
                    return i as i32;
                }
            }
        }
        0 // Default to 0 if not found
    }

    /// Get the enum name that contains a given variant
    pub(crate) fn get_enum_name_for_variant(&self, variant_name: &str) -> Option<String> {
        for enum_info in self.types.enums.values() {
            for variant in &enum_info.variants {
                if variant.name == variant_name {
                    return Some(enum_info.name.clone());
                }
            }
        }
        None
    }

    /// Check if a name is a unit enum variant (not a binding)
    pub(crate) fn is_unit_enum_variant(&self, name: &str) -> bool {
        use crate::types::EnumVariantFields;
        for enum_info in self.types.enums.values() {
            for variant in &enum_info.variants {
                if variant.name == name {
                    return matches!(variant.fields, EnumVariantFields::Unit);
                }
            }
        }
        false
    }

    /// Check if a name is a tuple enum variant and get its enum name and tag
    pub(crate) fn get_tuple_variant_info(&self, name: &str) -> Option<(String, i32)> {
        use crate::types::EnumVariantFields;
        for enum_info in self.types.enums.values() {
            for (tag, variant) in enum_info.variants.iter().enumerate() {
                if variant.name == name && matches!(variant.fields, EnumVariantFields::Tuple(_)) {
                    return Some((enum_info.name.clone(), tag as i32));
                }
            }
        }
        None
    }

    /// Generate pattern bindings (assign matched values to pattern variables)
    pub(crate) fn generate_pattern_bindings(
        &mut self,
        pattern: &Spanned<Pattern>,
        match_val: &str,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        self.generate_pattern_bindings_typed(pattern, match_val, counter, &ResolvedType::I64)
    }

    /// Generate pattern bindings with explicit match type for correct type propagation.
    /// Called from generate_match where the matched expression type is known.
    pub(crate) fn generate_pattern_bindings_typed(
        &mut self,
        pattern: &Spanned<Pattern>,
        match_val: &str,
        counter: &mut usize,
        match_type: &ResolvedType,
    ) -> CodegenResult<String> {
        match &pattern.node {
            Pattern::Ident(name) => {
                // Check if this is a unit enum variant (like None)
                // Unit variants don't bind anything
                if self.is_unit_enum_variant(name) {
                    return Ok(String::new());
                }

                // Bind the matched value to the identifier
                let ir = String::new();
                let ty = match_type.clone();

                // Generate unique LLVM name for pattern binding
                let _llvm_name = format!("{}.{}", name, counter);
                *counter += 1;

                self.fn_ctx.locals.insert(
                    name.clone(),
                    LocalVar::ssa(ty.clone(), match_val.to_string()),
                );

                // SSA style - no alloca needed, we just alias the match value

                Ok(ir)
            }
            Pattern::Tuple(patterns) => {
                let mut ir = String::new();

                // Get element types from match_type if it is a Tuple, otherwise fall back to i64.
                // This is critical for nested tuples (e.g., (1, (2, 3))) where the inner element
                // is { i64, i64 }, not i64 — using i64 would produce a type mismatch in LLVM IR.
                let elem_types: Vec<ResolvedType> = if let ResolvedType::Tuple(elems) = match_type {
                    elems.clone()
                } else {
                    vec![ResolvedType::I64; patterns.len()]
                };

                // Build the full LLVM struct type string for extractvalue.
                let llvm_elem_strs: Vec<String> =
                    elem_types.iter().map(|t| self.type_to_llvm(t)).collect();
                let tuple_llvm_ty = format!("{{ {} }}", llvm_elem_strs.join(", "));

                for (i, pat) in patterns.iter().enumerate() {
                    // Extract tuple element using the correct aggregate type.
                    let elem = self.next_temp(counter);
                    writeln!(
                        ir,
                        "  {} = extractvalue {} {}, {}",
                        elem, tuple_llvm_ty, match_val, i
                    )
                    .unwrap();

                    // Recurse with the element's actual type so nested tuple bindings
                    // also generate correct extractvalue instructions.
                    let elem_ty = elem_types.get(i).cloned().unwrap_or(ResolvedType::I64);
                    let bind_ir =
                        self.generate_pattern_bindings_typed(pat, &elem, counter, &elem_ty)?;
                    ir.push_str(&bind_ir);
                }

                Ok(ir)
            }
            Pattern::Variant { name, fields } => {
                // Bind fields from enum variant payload
                let mut ir = String::new();
                let variant_name = &name.node;

                // Get the enum type name for proper LLVM type reference
                let enum_name = self
                    .get_enum_name_for_variant(variant_name)
                    .unwrap_or_else(|| "Unknown".to_string());

                for (i, field_pat) in fields.iter().enumerate() {
                    // Extract payload field from enum variant
                    // Enum layout: %EnumName = type { i32 tag, { payload_types... } }
                    // First get pointer to payload struct (index 1), then get field within it

                    if match_val.starts_with('%') {
                        // match_val is a pointer to the enum - use getelementptr
                        // Access payload field: first get to the payload (index 1),
                        // then get the specific field within the payload
                        let payload_ptr = self.next_temp(counter);
                        writeln!(
                            ir,
                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 1, i32 {}",
                            payload_ptr, enum_name, enum_name, match_val, i
                        )
                        .unwrap();
                        let field_val = self.next_temp(counter);
                        writeln!(ir, "  {} = load i64, i64* {}", field_val, payload_ptr).unwrap();
                        let bind_ir =
                            self.generate_pattern_bindings(field_pat, &field_val, counter)?;
                        ir.push_str(&bind_ir);
                    } else {
                        // It's a value - use extractvalue
                        // Extract from payload sub-struct: index 1 for payload, then i for field
                        let payload_val = self.next_temp(counter);
                        writeln!(
                            ir,
                            "  {} = extractvalue %{} {}, 1, {}",
                            payload_val, enum_name, match_val, i
                        )
                        .unwrap();
                        let bind_ir =
                            self.generate_pattern_bindings(field_pat, &payload_val, counter)?;
                        ir.push_str(&bind_ir);
                    }
                }

                Ok(ir)
            }
            Pattern::Struct { name, fields } => {
                // Bind fields from struct
                let struct_name = &name.node;
                let mut ir = String::new();

                if let Some(struct_info) = self.types.structs.get(struct_name).cloned() {
                    for (field_name, field_pat) in fields {
                        // If field_pat is None, bind the field to its own name
                        if let Some(field_idx) = struct_info
                            .fields
                            .iter()
                            .position(|(n, _)| n == &field_name.node)
                        {
                            // Extract field value
                            let field_ptr = self.next_temp(counter);
                            writeln!(
                                ir,
                                "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}",
                                field_ptr, struct_name, struct_name, match_val, field_idx
                            )
                            .unwrap();

                            let field_val = self.next_temp(counter);
                            let field_ty = &struct_info.fields[field_idx].1;
                            let llvm_ty = self.type_to_llvm(field_ty);
                            writeln!(
                                ir,
                                "  {} = load {}, {}* {}",
                                field_val, llvm_ty, llvm_ty, field_ptr
                            )
                            .unwrap();

                            if let Some(pat) = field_pat {
                                // Bind to pattern
                                let bind_ir =
                                    self.generate_pattern_bindings(pat, &field_val, counter)?;
                                ir.push_str(&bind_ir);
                            } else {
                                // Bind to field name directly using SSA style
                                self.fn_ctx.locals.insert(
                                    field_name.node.clone(),
                                    LocalVar::ssa(field_ty.clone(), field_val.clone()),
                                );
                                // SSA style - no alloca needed
                            }
                        }
                    }
                }

                Ok(ir)
            }
            Pattern::Alias { name, pattern } => {
                // Bind the whole value to name, then bind variables from inner pattern
                let mut ir = String::new();

                // First, bind the whole matched value to the alias name using the actual match type
                self.fn_ctx.locals.insert(
                    name.clone(),
                    LocalVar::ssa(match_type.clone(), match_val.to_string()),
                );

                // Then bind variables from the inner pattern
                let inner_ir =
                    self.generate_pattern_bindings_typed(pattern, match_val, counter, match_type)?;
                ir.push_str(&inner_ir);

                Ok(ir)
            }
            _ => Ok(String::new()),
        }
    }
}

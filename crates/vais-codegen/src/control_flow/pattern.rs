use super::*;
use crate::types::LocalVar;

type EnumStructVariantInfo = (String, i32, Vec<(String, ResolvedType)>);

impl CodeGenerator {
    /// Generate code to check if a pattern matches (with explicit match type for correct type
    /// propagation, especially for nested tuple patterns where element types may be non-i64).
    #[inline(never)]
    #[allow(clippy::blocks_in_conditions)]
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

                    // Get the enum type name for proper LLVM type reference.
                    // The matched value may be `&Enum`; keep that enum context
                    // so duplicate unit variants (for example `Empty`) are not
                    // resolved by global variant lookup.
                    let enum_name = self
                        .enum_name_from_match_type(match_type)
                        .or_else(|| self.get_enum_name_for_variant(name))
                        .unwrap_or_else(|| "Unknown".to_string());

                    // Get the tag from the enum value (first field at index 0)
                    let tag_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0",
                        tag_ptr,
                        enum_name,
                        enum_name,
                        match_val
                    );

                    let tag_val = self.next_temp(counter);
                    write_ir!(ir, "  {} = load i32, i32* {}", tag_val, tag_ptr);

                    // Find the expected tag value for this variant
                    let expected_tag = self
                        .get_enum_variant_tag_in_enum(&enum_name, name)
                        .unwrap_or_else(|| self.get_enum_variant_tag(name));

                    // Compare tag
                    let result = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = icmp eq i32 {}, {}",
                        result,
                        tag_val,
                        expected_tag
                    );

                    Ok((result, ir))
                } else if let Some(const_val) = self.resolve_const_int_value(name) {
                    // Named constant pattern — compare match value against constant
                    let mut ir = String::new();
                    let result = self.next_temp(counter);
                    let match_ty = self
                        .llvm_type_of_checked(match_val)
                        .filter(|ty| ty.starts_with('i'))
                        .unwrap_or_else(|| {
                            match match_type {
                                ResolvedType::I8 | ResolvedType::U8 => "i8",
                                ResolvedType::I16 | ResolvedType::U16 => "i16",
                                ResolvedType::I32 | ResolvedType::U32 => "i32",
                                ResolvedType::Bool => "i1",
                                _ => "i64",
                            }
                            .to_string()
                        });
                    write_ir!(
                        ir,
                        "  {} = icmp eq {} {}, {}",
                        result,
                        match_ty,
                        match_val,
                        const_val
                    );
                    Ok((result, ir))
                } else {
                    // Identifier pattern always matches (binding)
                    Ok(("1".to_string(), String::new()))
                }
            }
            Pattern::Literal(lit) => match lit {
                Literal::Int(n) => {
                    // Use the match value's actual integer width when known,
                    // so matching an i32 payload against a literal uses
                    // `icmp eq i32` instead of i64 (mismatch under clang).
                    let match_ty = self
                        .llvm_type_of_checked(match_val)
                        .filter(|ty| ty.starts_with('i'))
                        .unwrap_or_else(|| {
                            match match_type {
                                ResolvedType::I8 | ResolvedType::U8 => "i8",
                                ResolvedType::I16 | ResolvedType::U16 => "i16",
                                ResolvedType::I32 | ResolvedType::U32 => "i32",
                                ResolvedType::Bool => "i1",
                                _ => "i64",
                            }
                            .to_string()
                        });
                    let result = self.next_temp(counter);
                    let ir = format!("  {} = icmp eq {} {}, {}\n", result, match_ty, match_val, n);
                    Ok((result, ir))
                }
                Literal::Bool(b) => {
                    let lit_val = if *b { "1" } else { "0" };
                    let match_ty = match match_type {
                        ResolvedType::Bool => "i1",
                        _ => "i64",
                    };
                    let result = self.next_temp(counter);
                    let ir = format!(
                        "  {} = icmp eq {} {}, {}\n",
                        result, match_ty, match_val, lit_val
                    );
                    Ok((result, ir))
                }
                Literal::Float(f) => {
                    let float_ty = match match_type {
                        ResolvedType::F32 => "float",
                        _ => "double",
                    };
                    let result = self.next_temp(counter);
                    let ir = format!(
                        "  {} = fcmp oeq {} {}, {:.6e}\n",
                        result, float_ty, match_val, f
                    );
                    Ok((result, ir))
                }
                Literal::String(s) => {
                    // String comparison using strcmp
                    let mut ir = String::new();

                    // Create string constant for the pattern (deduplicated)
                    let const_name = self.get_or_create_string_constant(s);

                    // Get pointer to the constant string
                    let str_ptr = self.next_temp(counter);
                    let str_len = s.len() + 1;
                    write_ir!(
                        ir,
                        "  {} = getelementptr [{} x i8], [{} x i8]* @{}, i32 0, i32 0",
                        str_ptr,
                        str_len,
                        str_len,
                        const_name
                    );
                    self.fn_ctx.record_emitted_type(&str_ptr, "i8*");

                    // Call strcmp: int strcmp(const char* s1, const char* s2)
                    // Returns 0 if strings are equal
                    let cmp_result = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = call i32 @strcmp(i8* {}, i8* {})",
                        cmp_result,
                        match_val,
                        str_ptr
                    );

                    // Check if strcmp returned 0 (equal)
                    let result = self.next_temp(counter);
                    write_ir!(ir, "  {} = icmp eq i32 {}, 0", result, cmp_result);
                    self.fn_ctx.record_emitted_type(&result, "i1");

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
                        write_ir!(ir, "  {} = icmp sge i64 {}, {}", tmp, match_val, n);
                        self.fn_ctx.record_emitted_type(&tmp, "i1");
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
                        write_ir!(ir, "  {} = {} i64 {}, {}", tmp, cmp, match_val, n);
                        tmp
                    } else {
                        "1".to_string()
                    }
                } else {
                    "1".to_string()
                };

                // Combine checks
                let result = self.next_temp(counter);
                write_ir!(ir, "  {} = and i1 {}, {}", result, lower_check, upper_check);

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
                    write_ir!(ir, "  {} = or i1 {}, {}", tmp, result, check);
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
                    write_ir!(
                        ir,
                        "  {} = extractvalue {} {}, {}",
                        elem,
                        tuple_llvm_ty,
                        match_val,
                        i
                    );

                    // Recurse with the element's actual type.
                    let elem_ty = elem_types.get(i).cloned().unwrap_or(ResolvedType::I64);
                    // For struct/enum tuple elements the sub-patterns GEP
                    // through a `%T*`. `extractvalue` produced a struct
                    // value, so spill it to an alloca so downstream checks
                    // receive a pointer.
                    let elem_to_pass = if matches!(&elem_ty, ResolvedType::Named { .. }) {
                        let elem_llvm_ty = &llvm_elem_strs[i];
                        let spill = self.next_temp(counter);
                        self.emit_entry_alloca(&spill, elem_llvm_ty);
                        write_ir!(
                            ir,
                            "  store {} {}, {}* {}",
                            elem_llvm_ty,
                            elem,
                            elem_llvm_ty,
                            spill
                        );
                        spill
                    } else {
                        elem.clone()
                    };
                    let (check, check_ir) =
                        self.generate_pattern_check_typed(pat, &elem_to_pass, counter, &elem_ty)?;
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
                    write_ir!(ir, "  {} = and i1 {}, {}", tmp, result, check);
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

                // Phase α.1 fix: prefer the enum name from match_type for
                // specialized enums (e.g., `Result$Tuple_VaisError`). The
                // variant-name lookup falls back to the unspecialized parent
                // (Result) which doesn't exist as a concrete LLVM type.
                let enum_name_from_match_type = self.enum_name_from_match_type(match_type);
                let enum_name = enum_name_from_match_type
                    .or_else(|| self.get_enum_name_for_variant(variant_name))
                    .unwrap_or_else(|| "Unknown".to_string());

                // Get the tag from the enum value (first field at index 0)
                let tag_ptr = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0",
                    tag_ptr,
                    enum_name,
                    enum_name,
                    match_val
                );

                let tag_val = self.next_temp(counter);
                write_ir!(ir, "  {} = load i32, i32* {}", tag_val, tag_ptr);

                // Find the expected tag value for this variant
                let expected_tag = self
                    .get_enum_variant_tag_in_enum(&enum_name, variant_name)
                    .unwrap_or_else(|| self.get_enum_variant_tag(variant_name));

                // Compare tag
                let result = self.next_temp(counter);
                write_ir!(
                    ir,
                    "  {} = icmp eq i32 {}, {}",
                    result,
                    tag_val,
                    expected_tag
                );

                Ok((result, ir))
            }
            Pattern::Struct {
                name,
                fields,
                enum_name: enum_hint,
            } => {
                // Struct pattern: always matches if type is correct, but we check field patterns
                let struct_name = &name.node;
                let enum_hint_ref = enum_hint.as_deref();
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
                                write_ir!(
                                    ir,
                                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}",
                                    field_ptr,
                                    struct_name,
                                    struct_name,
                                    match_val,
                                    field_idx
                                );

                                let field_val = self.next_temp(counter);
                                let field_ty = &struct_info.fields[field_idx].1;
                                let llvm_ty = self.type_to_llvm(field_ty);
                                write_ir!(
                                    ir,
                                    "  {} = load {}, {}* {}",
                                    field_val,
                                    llvm_ty,
                                    llvm_ty,
                                    field_ptr
                                );

                                let (check, check_ir) =
                                    self.generate_pattern_check(pat, &field_val, counter)?;
                                ir.push_str(&check_ir);
                                checks.push(check);
                            }
                        }
                    }
                } else if {
                    let requested: Vec<&str> =
                        fields.iter().map(|(fn_, _)| fn_.node.as_str()).collect();
                    self.resolve_enum_struct_variant_with_hint_and_fields(
                        struct_name,
                        enum_hint_ref,
                        &requested,
                    )
                    .is_some()
                } {
                    // `struct_name` names an enum struct-variant (e.g. `Varchar { max_len }`
                    // used as short-form inside a `M` arm). Verify the enum's runtime tag
                    // matches and delegate any inner sub-pattern checks through the enum
                    // variant payload layout.
                    let requested: Vec<&str> =
                        fields.iter().map(|(fn_, _)| fn_.node.as_str()).collect();
                    let (enum_name, _variant_tag, variant_struct_fields) = self
                        .resolve_enum_struct_variant_with_hint_and_fields(
                            struct_name,
                            enum_hint_ref,
                            &requested,
                        )
                        .expect("invariant: resolve_enum_struct_variant is_some() confirmed in enclosing else-if guard");

                    // Tag check
                    let tag_ptr = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0",
                        tag_ptr,
                        enum_name,
                        enum_name,
                        match_val
                    );
                    let tag_val = self.next_temp(counter);
                    write_ir!(ir, "  {} = load i32, i32* {}", tag_val, tag_ptr);
                    let expected_tag = self
                        .get_enum_variant_tag_in_enum(&enum_name, struct_name)
                        .unwrap_or_else(|| self.get_enum_variant_tag(struct_name));
                    let tag_ok = self.next_temp(counter);
                    write_ir!(
                        ir,
                        "  {} = icmp eq i32 {}, {}",
                        tag_ok,
                        tag_val,
                        expected_tag
                    );
                    checks.push(tag_ok);

                    // Sub-pattern checks (only for fields with explicit patterns — shorthand
                    // `{ field }` bindings are always-match at check time, they only matter
                    // during bindings generation).
                    for (field_name, field_pat) in fields {
                        if let Some(pat) = field_pat {
                            if let Some(field_idx) = variant_struct_fields
                                .iter()
                                .position(|(n, _)| n == &field_name.node)
                            {
                                let payload_ptr = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 1, i32 {}",
                                    payload_ptr,
                                    enum_name,
                                    enum_name,
                                    match_val,
                                    field_idx
                                );
                                let field_val = self.next_temp(counter);
                                let field_ty = &variant_struct_fields[field_idx].1;
                                match field_ty {
                                    ResolvedType::F64 => {
                                        let fp = self.next_temp(counter);
                                        write_ir!(
                                            ir,
                                            "  {} = bitcast i64* {} to double*",
                                            fp,
                                            payload_ptr
                                        );
                                        write_ir!(
                                            ir,
                                            "  {} = load double, double* {}",
                                            field_val,
                                            fp
                                        );
                                        self.fn_ctx.record_emitted_type(&field_val, "double");
                                    }
                                    ResolvedType::F32 => {
                                        let fp = self.next_temp(counter);
                                        write_ir!(
                                            ir,
                                            "  {} = bitcast i64* {} to float*",
                                            fp,
                                            payload_ptr
                                        );
                                        write_ir!(
                                            ir,
                                            "  {} = load float, float* {}",
                                            field_val,
                                            fp
                                        );
                                        self.fn_ctx.record_emitted_type(&field_val, "float");
                                    }
                                    ResolvedType::Bool => {
                                        let raw = self.next_temp(counter);
                                        write_ir!(ir, "  {} = load i64, i64* {}", raw, payload_ptr);
                                        self.fn_ctx.record_emitted_type(&raw, "i64");
                                        write_ir!(ir, "  {} = trunc i64 {} to i1", field_val, raw);
                                    }
                                    ResolvedType::I32 | ResolvedType::U32 => {
                                        let raw = self.next_temp(counter);
                                        write_ir!(ir, "  {} = load i64, i64* {}", raw, payload_ptr);
                                        self.fn_ctx.record_emitted_type(&raw, "i64");
                                        write_ir!(ir, "  {} = trunc i64 {} to i32", field_val, raw);
                                    }
                                    _ => {
                                        write_ir!(
                                            ir,
                                            "  {} = load i64, i64* {}",
                                            field_val,
                                            payload_ptr
                                        );
                                    }
                                }
                                let (check, check_ir) = self.generate_pattern_check_typed(
                                    pat, &field_val, counter, field_ty,
                                )?;
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
                        write_ir!(ir, "  {} = and i1 {}, {}", tmp, result, check);
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
    #[inline(never)]
    pub(crate) fn generate_pattern_check(
        &mut self,
        pattern: &Spanned<Pattern>,
        match_val: &str,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        self.generate_pattern_check_typed(pattern, match_val, counter, &ResolvedType::I64)
    }

    /// Get the tag value for an enum variant
    #[inline(never)]
    pub(crate) fn get_enum_variant_tag(&self, variant_name: &str) -> i32 {
        for enum_info in self.types.enums.values() {
            for (i, variant) in enum_info.variants.iter().enumerate() {
                if variant.name == variant_name {
                    return i as i32;
                }
            }
        }
        // Hardcoded fallback for std Result/Option
        match variant_name {
            "Ok" => 0,
            "Err" => 1,
            "None" => 0,
            "Some" => 1,
            _ => 0,
        }
    }

    /// Resolve the concrete enum type name from the type being matched.
    ///
    /// Pattern checks and bindings need the enum attached to the match value,
    /// not the first globally registered enum that happens to contain a
    /// variant with the same short name. This matters for `M &node {
    /// PlanNode.Empty => ... }`, where `match_type` is `Ref(Named(PlanNode))`.
    pub(crate) fn enum_name_from_match_type(&self, match_type: &ResolvedType) -> Option<String> {
        fn named(ty: &ResolvedType) -> Option<&str> {
            match ty {
                ResolvedType::Named { name, .. } => Some(name.as_str()),
                _ => None,
            }
        }

        let enum_ty = match match_type {
            ResolvedType::Optional(_) => return Some("Option".to_string()),
            ResolvedType::Result(_, _) => return Some("Result".to_string()),
            ResolvedType::Named { .. } => match_type,
            ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner)
            | ResolvedType::RefLifetime { inner, .. }
            | ResolvedType::RefMutLifetime { inner, .. } => inner.as_ref(),
            _ => return None,
        };
        let fallback = named(enum_ty)?;
        let llvm = self.type_to_llvm(enum_ty);
        let bare = llvm
            .strip_prefix('%')
            .unwrap_or(&llvm)
            .trim_end_matches('*');
        if !bare.is_empty() && bare != "Unknown" {
            Some(bare.to_string())
        } else {
            Some(fallback.to_string())
        }
    }

    pub(crate) fn get_enum_variant_tag_in_enum(
        &self,
        enum_name: &str,
        variant_name: &str,
    ) -> Option<i32> {
        let base_name = enum_name
            .split_once('$')
            .map(|(base, _)| base)
            .unwrap_or(enum_name);
        let enum_info = self
            .types
            .enums
            .get(enum_name)
            .or_else(|| self.types.enums.get(base_name))?;

        enum_info
            .variants
            .iter()
            .position(|variant| variant.name == variant_name)
            .map(|idx| idx as i32)
    }

    /// Get the enum name that contains a given variant
    #[inline(never)]
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

    /// Resolve a short-form struct-variant name (e.g. `Varchar`) to its parent
    /// enum + tag + named-field layout. Returns `None` if `variant_name` is not
    /// the name of an enum struct-variant (`EnumVariantFields::Struct`).
    ///
    /// Used by the pattern match path to recover enum struct-variant semantics
    /// when the parser produced a `Pattern::Struct` (which happens whenever the
    /// match arm omits the `EnumType.` qualifier, for example
    /// `Varchar { max_len } => …`).
    #[allow(dead_code, clippy::type_complexity)]
    pub(crate) fn resolve_enum_struct_variant(
        &self,
        variant_name: &str,
    ) -> Option<EnumStructVariantInfo> {
        self.resolve_enum_struct_variant_with_hint(variant_name, None)
    }

    /// Phase 6.27b: disambiguation helper — prefer the enum matching `enum_hint`
    /// when multiple enums contain a variant with the same name (e.g. both
    /// `GrantType.Privileges` and `RevokeType.Privileges` exist).
    #[allow(dead_code)]
    pub(crate) fn resolve_enum_struct_variant_with_hint(
        &self,
        variant_name: &str,
        enum_hint: Option<&str>,
    ) -> Option<EnumStructVariantInfo> {
        self.resolve_enum_struct_variant_with_hint_and_fields(variant_name, enum_hint, &[])
    }

    /// Same as `resolve_enum_struct_variant_with_hint` but also considers the
    /// expected field-name set. When two enums share a variant name (e.g. both
    /// `TableRef.Subquery { query, alias }` and `Expr.Subquery { query }`),
    /// prefer the one whose Struct fields are a superset of the requested
    /// names. This disambiguates short-form Pattern::Struct on enum variants
    /// without requiring explicit `EnumType.Variant` qualification everywhere.
    pub(crate) fn resolve_enum_struct_variant_with_hint_and_fields(
        &self,
        variant_name: &str,
        enum_hint: Option<&str>,
        requested_fields: &[&str],
    ) -> Option<EnumStructVariantInfo> {
        use crate::types::EnumVariantFields;
        // First pass: if hint provided, look only within that enum.
        if let Some(hint) = enum_hint {
            if let Some(enum_info) = self.types.enums.get(hint) {
                for (tag, variant) in enum_info.variants.iter().enumerate() {
                    if variant.name == variant_name {
                        if let EnumVariantFields::Struct(fields) = &variant.fields {
                            return Some((enum_info.name.clone(), tag as i32, fields.clone()));
                        }
                    }
                }
            }
        }
        // Fallback: scan all enums. Prefer variants whose fields cover the
        // requested field-name set (exact or superset); fall back to the first
        // match if no candidate covers all requested names.
        let mut best: Option<EnumStructVariantInfo> = None;
        let mut first: Option<EnumStructVariantInfo> = None;
        for enum_info in self.types.enums.values() {
            for (tag, variant) in enum_info.variants.iter().enumerate() {
                if variant.name == variant_name {
                    if let EnumVariantFields::Struct(fields) = &variant.fields {
                        if first.is_none() {
                            first = Some((enum_info.name.clone(), tag as i32, fields.clone()));
                        }
                        if !requested_fields.is_empty() {
                            let covers_all = requested_fields
                                .iter()
                                .all(|rf| fields.iter().any(|(n, _)| n == rf));
                            if covers_all && best.is_none() {
                                best = Some((enum_info.name.clone(), tag as i32, fields.clone()));
                            }
                        }
                    }
                }
            }
        }
        best.or(first)
    }

    /// Check if a name is a unit enum variant (not a binding)
    #[inline(never)]
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

    pub(crate) fn enum_name_hint_from_type(&self, ty: &ResolvedType) -> Option<String> {
        match ty {
            ResolvedType::Named { name, .. } => {
                let base = name.split('$').next().unwrap_or(name);
                self.types
                    .enums
                    .contains_key(base)
                    .then(|| base.to_string())
            }
            ResolvedType::Optional(_) => Some("Option".to_string()),
            ResolvedType::Result(_, _) => Some("Result".to_string()),
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                self.enum_name_hint_from_type(inner)
            }
            _ => None,
        }
    }

    pub(crate) fn named_enum_type_from_expected(
        &self,
        enum_name: &str,
        expected: &ResolvedType,
    ) -> ResolvedType {
        match expected {
            ResolvedType::Named { name, generics } if name.split('$').next() == Some(enum_name) => {
                ResolvedType::Named {
                    name: enum_name.to_string(),
                    generics: generics.clone(),
                }
            }
            ResolvedType::Optional(inner) if enum_name == "Option" => ResolvedType::Named {
                name: "Option".to_string(),
                generics: vec![(**inner).clone()],
            },
            ResolvedType::Result(ok, err) if enum_name == "Result" => ResolvedType::Named {
                name: "Result".to_string(),
                generics: vec![(**ok).clone(), (**err).clone()],
            },
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                self.named_enum_type_from_expected(enum_name, inner)
            }
            _ => ResolvedType::Named {
                name: enum_name.to_string(),
                generics: vec![],
            },
        }
    }

    pub(crate) fn expected_enum_type_for_variant(
        &self,
        variant_name: &str,
        expected: Option<&ResolvedType>,
    ) -> Option<ResolvedType> {
        let expected = expected?;
        let enum_name = self.enum_name_hint_from_type(expected)?;
        let enum_info = self.types.enums.get(&enum_name)?;
        enum_info
            .variants
            .iter()
            .any(|variant| variant.name == variant_name)
            .then(|| self.named_enum_type_from_expected(&enum_name, expected))
    }

    pub(crate) fn get_unit_variant_info_with_expected(
        &self,
        variant_name: &str,
        expected: Option<&ResolvedType>,
    ) -> Option<(String, i32)> {
        use crate::types::EnumVariantFields;
        if let Some(enum_name) = expected.and_then(|ty| self.enum_name_hint_from_type(ty)) {
            if let Some(enum_info) = self.types.enums.get(&enum_name) {
                if let Some(tag) = enum_info.variants.iter().position(|variant| {
                    variant.name == variant_name
                        && matches!(variant.fields, EnumVariantFields::Unit)
                }) {
                    return Some((enum_name, tag as i32));
                }
            }
        }

        for enum_info in self.types.enums.values() {
            for (tag, variant) in enum_info.variants.iter().enumerate() {
                if variant.name == variant_name && matches!(variant.fields, EnumVariantFields::Unit)
                {
                    return Some((enum_info.name.clone(), tag as i32));
                }
            }
        }
        None
    }

    /// Check if a name is a tuple enum variant and get its enum name and tag
    #[inline(never)]
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

    pub(crate) fn get_tuple_variant_info_with_expected(
        &self,
        variant_name: &str,
        expected: Option<&ResolvedType>,
    ) -> Option<(String, i32)> {
        use crate::types::EnumVariantFields;
        if let Some(enum_name) = expected.and_then(|ty| self.enum_name_hint_from_type(ty)) {
            if let Some(enum_info) = self.types.enums.get(&enum_name) {
                if let Some(tag) = enum_info.variants.iter().position(|variant| {
                    variant.name == variant_name
                        && matches!(variant.fields, EnumVariantFields::Tuple(_))
                }) {
                    return Some((enum_name, tag as i32));
                }
            }
        }

        None
    }

    /// Resolve a constant name to its integer value, if it is a known integer constant.
    /// Returns None if the name is not a constant or not an integer literal.
    #[inline(never)]
    pub(crate) fn resolve_const_int_value(&self, name: &str) -> Option<i64> {
        use vais_ast::Expr;
        let const_info = self.types.constants.get(name)?;
        match &const_info.value.node {
            Expr::Int(n) => Some(*n),
            // Handle `as` casts like `0 as u8` — extract the inner literal
            Expr::Cast { expr, .. } => {
                if let Expr::Int(n) = &expr.node {
                    Some(*n)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Check if a name is a known constant (for pattern matching disambiguation)
    #[inline(never)]
    pub(crate) fn is_known_constant(&self, name: &str) -> bool {
        self.types.constants.contains_key(name)
    }

    /// Resolve the concrete field types for an enum variant by substituting generic
    /// parameters with the concrete types from the match expression's type.
    ///
    /// For example, given `Option<Vec<u64>>::Some(T)`, this returns `[Vec<u64>]`
    /// by substituting `T` → `Vec<u64>` from the match_type generics.
    pub(crate) fn resolve_variant_field_types(
        &self,
        enum_name: &str,
        variant_name: &str,
        match_type: &ResolvedType,
    ) -> Vec<ResolvedType> {
        use crate::types::EnumVariantFields;

        // Phase 6.27b iteration 33: deref Ref/RefMut so `M &opt { Some(x) => ... }`
        // sees the inner enum type (Option/Result/user enum), not `Ref(Optional(..))`.
        // Without this, the Named-with-generics branch below misses, and Some(x)
        // binds `x: Generic("T")` instead of the concrete inner type.
        let inner_match_type = match match_type {
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => inner.as_ref(),
            other => other,
        };
        match (variant_name, inner_match_type) {
            ("Ok", ResolvedType::Named { name, generics })
                if name == "Result" && generics.len() >= 2 =>
            {
                return vec![generics[0].clone()];
            }
            ("Err", ResolvedType::Named { name, generics })
                if name == "Result" && generics.len() >= 2 =>
            {
                return vec![generics[1].clone()];
            }
            ("Some", ResolvedType::Named { name, generics })
                if name == "Option" && !generics.is_empty() =>
            {
                return vec![generics[0].clone()];
            }
            _ => {}
        }
        // Look up the enum definition to find the variant's field types
        let enum_info = if let Some((base, _)) = enum_name.split_once('$') {
            match self
                .types
                .enums
                .get(base)
                .or_else(|| self.types.enums.get(enum_name))
            {
                Some(info) => info,
                None => return vec![],
            }
        } else if let Some(info) = self.types.enums.get(enum_name) {
            info
        } else {
            return vec![];
        };

        // Find the variant
        let variant = match enum_info.variants.iter().find(|v| v.name == variant_name) {
            Some(v) => v,
            None => return vec![],
        };

        // Get the raw field types (may contain Generic("T"), Generic("E"), etc.)
        let raw_field_types: Vec<&ResolvedType> = match &variant.fields {
            EnumVariantFields::Unit => return vec![],
            EnumVariantFields::Tuple(types) => types.iter().collect(),
            EnumVariantFields::Struct(fields) => fields.iter().map(|(_, ty)| ty).collect(),
        };

        // Build a substitution map: collect unique generic param names from ALL variants
        // in declaration order, then map to concrete_generics by index.
        // e.g., Option: [T] → generics[0]; Result: [T, E] → generics[0], generics[1]
        let mut generic_param_order: Vec<String> = Vec::new();
        for v in &enum_info.variants {
            let field_types: Vec<&ResolvedType> = match &v.fields {
                EnumVariantFields::Unit => vec![],
                EnumVariantFields::Tuple(types) => types.iter().collect(),
                EnumVariantFields::Struct(fields) => fields.iter().map(|(_, ty)| ty).collect(),
            };
            for ft in field_types {
                Self::collect_generic_names(ft, &mut generic_param_order);
            }
        }
        if generic_param_order.is_empty() {
            return raw_field_types.into_iter().cloned().collect();
        }

        // Extract the concrete generic args from match_type (e.g., [Vec<u64>] from Option<Vec<u64>>)
        // Phase 300a-style: also handle primitive Optional(T)/Result(T,E) which
        // aren't Named { generics } but carry the concrete inner type directly.
        let extracted_generics: Vec<ResolvedType> = match inner_match_type {
            ResolvedType::Named { generics, .. } if !generics.is_empty() => generics.clone(),
            ResolvedType::Named { name, .. } => {
                Self::generics_from_mangled_name(name, generic_param_order.len())
                    .or_else(|| {
                        Self::generics_from_mangled_name(enum_name, generic_param_order.len())
                    })
                    .unwrap_or_default()
            }
            ResolvedType::Optional(inner) => vec![(**inner).clone()],
            ResolvedType::Result(ok, err) => vec![(**ok).clone(), (**err).clone()],
            _ => Self::generics_from_mangled_name(enum_name, generic_param_order.len())
                .unwrap_or_default(),
        };
        if extracted_generics.is_empty() {
            return raw_field_types.into_iter().cloned().collect();
        }
        let concrete_generics = &extracted_generics;

        // Build substitution map: param_name -> concrete_type
        let mut substitutions: std::collections::HashMap<String, ResolvedType> =
            std::collections::HashMap::new();
        for (idx, param_name) in generic_param_order.iter().enumerate() {
            if let Some(concrete) = concrete_generics.get(idx) {
                substitutions.insert(param_name.clone(), concrete.clone());
            }
        }

        // Substitute generic params in the variant's field types
        raw_field_types
            .into_iter()
            .map(|ty| Self::substitute_generics(ty, &substitutions))
            .collect()
    }

    fn generics_from_mangled_name(
        mangled_name: &str,
        generic_count: usize,
    ) -> Option<Vec<ResolvedType>> {
        if generic_count == 0 {
            return Some(vec![]);
        }
        let (_, suffix) = mangled_name.split_once('$')?;
        let parts: Vec<&str> = suffix.split('_').filter(|p| !p.is_empty()).collect();
        let mut result = Vec::with_capacity(generic_count);
        let mut idx = 0;

        for arg_index in 0..generic_count {
            let remaining_args = generic_count - arg_index - 1;
            let max_end = parts.len().checked_sub(remaining_args)?;
            let (ty, next_idx) = Self::parse_mangled_type_parts(&parts, idx, max_end)?;
            result.push(ty);
            idx = next_idx;
        }

        (idx == parts.len()).then_some(result)
    }

    fn parse_mangled_type_parts(
        parts: &[&str],
        idx: usize,
        max_end: usize,
    ) -> Option<(ResolvedType, usize)> {
        if idx >= max_end {
            return None;
        }

        let token = parts[idx];
        let primitive = match token {
            "i8" => Some(ResolvedType::I8),
            "i16" => Some(ResolvedType::I16),
            "i32" => Some(ResolvedType::I32),
            "i64" => Some(ResolvedType::I64),
            "i128" => Some(ResolvedType::I128),
            "u8" => Some(ResolvedType::U8),
            "u16" => Some(ResolvedType::U16),
            "u32" => Some(ResolvedType::U32),
            "u64" => Some(ResolvedType::U64),
            "u128" => Some(ResolvedType::U128),
            "f32" => Some(ResolvedType::F32),
            "f64" => Some(ResolvedType::F64),
            "bool" => Some(ResolvedType::Bool),
            "str" => Some(ResolvedType::Str),
            "unit" => Some(ResolvedType::Unit),
            _ => None,
        };
        if let Some(ty) = primitive {
            return Some((ty, idx + 1));
        }

        match token {
            "opt" => {
                let (inner, next) = Self::parse_mangled_type_parts(parts, idx + 1, max_end)?;
                Some((ResolvedType::Optional(Box::new(inner)), next))
            }
            "res" => {
                let (ok, next) = Self::parse_mangled_type_parts(parts, idx + 1, max_end)?;
                let (err, next) = Self::parse_mangled_type_parts(parts, next, max_end)?;
                Some((ResolvedType::Result(Box::new(ok), Box::new(err)), next))
            }
            "arr" => {
                let (inner, next) = Self::parse_mangled_type_parts(parts, idx + 1, max_end)?;
                Some((ResolvedType::Array(Box::new(inner)), next))
            }
            "ptr" => {
                let (inner, next) = Self::parse_mangled_type_parts(parts, idx + 1, max_end)?;
                Some((ResolvedType::Pointer(Box::new(inner)), next))
            }
            "ref" => {
                let (inner, next) = Self::parse_mangled_type_parts(parts, idx + 1, max_end)?;
                Some((ResolvedType::Ref(Box::new(inner)), next))
            }
            "refmut" => {
                let (inner, next) = Self::parse_mangled_type_parts(parts, idx + 1, max_end)?;
                Some((ResolvedType::RefMut(Box::new(inner)), next))
            }
            "slice" => {
                let (inner, next) = Self::parse_mangled_type_parts(parts, idx + 1, max_end)?;
                Some((ResolvedType::Slice(Box::new(inner)), next))
            }
            "slicemut" => {
                let (inner, next) = Self::parse_mangled_type_parts(parts, idx + 1, max_end)?;
                Some((ResolvedType::SliceMut(Box::new(inner)), next))
            }
            "fut" => {
                let (inner, next) = Self::parse_mangled_type_parts(parts, idx + 1, max_end)?;
                Some((ResolvedType::Future(Box::new(inner)), next))
            }
            "Vec" if idx + 1 < max_end => {
                let (inner, next) = Self::parse_mangled_type_parts(parts, idx + 1, max_end)?;
                Some((
                    ResolvedType::Named {
                        name: "Vec".to_string(),
                        generics: vec![inner],
                    },
                    next,
                ))
            }
            _ => Some((
                ResolvedType::Named {
                    name: token.to_string(),
                    generics: vec![],
                },
                idx + 1,
            )),
        }
    }

    /// Collect unique generic parameter names from a ResolvedType, preserving first-appearance order.
    fn collect_generic_names(ty: &ResolvedType, names: &mut Vec<String>) {
        match ty {
            ResolvedType::Generic(name) => {
                if !names.contains(name) {
                    names.push(name.clone());
                }
            }
            ResolvedType::Named { generics, .. } => {
                for g in generics {
                    Self::collect_generic_names(g, names);
                }
            }
            ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner)
            | ResolvedType::Pointer(inner) => {
                Self::collect_generic_names(inner, names);
            }
            ResolvedType::Tuple(elems) => {
                for e in elems {
                    Self::collect_generic_names(e, names);
                }
            }
            _ => {}
        }
    }

    /// Substitute generic type parameters in a ResolvedType using a substitution map.
    fn substitute_generics(
        ty: &ResolvedType,
        subs: &std::collections::HashMap<String, ResolvedType>,
    ) -> ResolvedType {
        match ty {
            ResolvedType::Generic(name) => subs.get(name).cloned().unwrap_or_else(|| ty.clone()),
            ResolvedType::Named { name, generics } => ResolvedType::Named {
                name: name.clone(),
                generics: generics
                    .iter()
                    .map(|g| Self::substitute_generics(g, subs))
                    .collect(),
            },
            ResolvedType::Ref(inner) => {
                ResolvedType::Ref(Box::new(Self::substitute_generics(inner, subs)))
            }
            ResolvedType::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(Self::substitute_generics(inner, subs)))
            }
            ResolvedType::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(Self::substitute_generics(inner, subs)))
            }
            ResolvedType::Tuple(elems) => ResolvedType::Tuple(
                elems
                    .iter()
                    .map(|e| Self::substitute_generics(e, subs))
                    .collect(),
            ),
            other => other.clone(),
        }
    }

    /// Get the raw (as-registered, before generic substitution) field types
    /// of a specific enum variant. This determines the actual LLVM payload slot type.
    ///
    /// For builtin Option/Result with Generic("T")/Generic("E"), the raw type is Generic,
    /// meaning the LLVM payload is i64. For user-defined enums with concrete struct fields,
    /// the raw type is Named, meaning the LLVM payload matches the struct type directly.
    #[allow(dead_code)]
    fn get_variant_raw_field_types(
        &self,
        enum_name: &str,
        variant_name: &str,
    ) -> Vec<ResolvedType> {
        use crate::types::EnumVariantFields;
        if let Some(enum_info) = self.types.enums.get(enum_name) {
            if let Some(variant) = enum_info.variants.iter().find(|v| v.name == variant_name) {
                return match &variant.fields {
                    EnumVariantFields::Unit => vec![],
                    EnumVariantFields::Tuple(types) => types.clone(),
                    EnumVariantFields::Struct(fields) => {
                        fields.iter().map(|(_, ty)| ty.clone()).collect()
                    }
                };
            }
        }
        vec![]
    }

    /// Get the raw field types of an enum variant by tag index.
    /// Used by the enum variant constructor to determine payload slot type.
    pub(crate) fn get_variant_raw_field_types_by_tag(
        &self,
        enum_name: &str,
        tag: i32,
    ) -> Vec<ResolvedType> {
        use crate::types::EnumVariantFields;
        if let Some(enum_info) = self.types.enums.get(enum_name) {
            if let Some(variant) = enum_info.variants.get(tag as usize) {
                return match &variant.fields {
                    EnumVariantFields::Unit => vec![],
                    EnumVariantFields::Tuple(types) => types.clone(),
                    EnumVariantFields::Struct(fields) => {
                        fields.iter().map(|(_, ty)| ty.clone()).collect()
                    }
                };
            }
        }
        vec![]
    }

    /// Generate pattern bindings (assign matched values to pattern variables)
    #[inline(never)]
    #[allow(dead_code)]
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
    #[inline(never)]
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

                // Check if this is a named constant (like PROP_TYPE_INT)
                // Constants don't bind — they are compared in pattern check
                if self.is_known_constant(name) {
                    return Ok(String::new());
                }

                // Bind the matched value to the identifier
                let ir = String::new();
                let ty = match_type.clone();

                // Generate unique LLVM name for pattern binding
                let _llvm_name = format!("{}.{}", name, counter);
                *counter += 1;

                if match_val.starts_with('%') && self.llvm_type_of_checked(match_val).is_none() {
                    let llvm_ty = match &ty {
                        ResolvedType::Named { .. } => format!("{}*", self.type_to_llvm(&ty)),
                        ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                            format!("{}*", self.type_to_llvm(inner))
                        }
                        _ => self.type_to_llvm(&ty),
                    };
                    if llvm_ty != "void" {
                        self.fn_ctx.record_emitted_type(match_val, &llvm_ty);
                    }
                }

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
                    write_ir!(
                        ir,
                        "  {} = extractvalue {} {}, {}",
                        elem,
                        tuple_llvm_ty,
                        match_val,
                        i
                    );

                    // Recurse with the element's actual type so nested tuple bindings
                    // also generate correct extractvalue instructions.
                    let elem_ty = elem_types.get(i).cloned().unwrap_or(ResolvedType::I64);
                    // Struct/enum elements require a pointer for GEP-based
                    // binding — spill the struct value to an alloca.
                    let elem_to_pass = if matches!(&elem_ty, ResolvedType::Named { .. }) {
                        let elem_llvm_ty = &llvm_elem_strs[i];
                        let spill = self.next_temp(counter);
                        self.emit_entry_alloca(&spill, elem_llvm_ty);
                        write_ir!(
                            ir,
                            "  store {} {}, {}* {}",
                            elem_llvm_ty,
                            elem,
                            elem_llvm_ty,
                            spill
                        );
                        spill
                    } else {
                        elem.clone()
                    };
                    let bind_ir = self.generate_pattern_bindings_typed(
                        pat,
                        &elem_to_pass,
                        counter,
                        &elem_ty,
                    )?;
                    ir.push_str(&bind_ir);
                }

                Ok(ir)
            }
            Pattern::Variant { name, fields } => {
                // Bind fields from enum variant payload
                let mut ir = String::new();
                let variant_name = &name.node;

                // Phase α.1 fix: prefer specialized enum name from match_type
                // (mirrors the variant-tag-check fix above).
                let enum_name_from_match_type = self.enum_name_from_match_type(match_type);
                let enum_name = enum_name_from_match_type
                    .or_else(|| self.get_enum_name_for_variant(variant_name))
                    .unwrap_or_else(|| "Unknown".to_string());

                // Resolve the actual field types for this variant, substituting generic
                // parameters with concrete types from match_type.
                // e.g., Option<Vec<u64>>::Some(T) → field type is Vec<u64>
                let variant_field_types =
                    self.resolve_variant_field_types(&enum_name, variant_name, match_type);

                for (i, field_pat) in fields.iter().enumerate() {
                    // Extract payload field from enum variant
                    // Enum layout: %EnumName = type { i32 tag, { i64, i64, ... } }
                    // All payload slots are i64 (see generate_enum_type).
                    // Compound types (str, structs, tuples) are stored via bitcast (<=8 bytes)
                    // or heap-alloc (>8 bytes) by generate_enum_variant_constructor.

                    // Get the resolved field type (fallback to I64 if unavailable)
                    let field_type = variant_field_types
                        .get(i)
                        .cloned()
                        .unwrap_or(ResolvedType::I64);

                    let llvm_field_ty = self.type_to_llvm(&field_type);
                    // Phase 17.H4.2: Unit-typed enum payload field has no
                    // loadable value. Skip the bitcast/load entirely and
                    // bind the field pattern against a placeholder so the
                    // match arm's `bind_ir` stays structurally correct.
                    if llvm_field_ty == "void" {
                        let placeholder = "i8 0".to_string();
                        let bind_ir = self.generate_pattern_bindings_typed(
                            field_pat,
                            &placeholder,
                            counter,
                            &field_type,
                        )?;
                        ir.push_str(&bind_ir);
                        continue;
                    }
                    // Check if the field type is compound (not a simple integer/bool/pointer).
                    // Compound types were stored via bitcast/heap-alloc into the i64 payload slot.
                    let is_compound_field = llvm_field_ty != "i64"
                        && llvm_field_ty != "i32"
                        && llvm_field_ty != "i16"
                        && llvm_field_ty != "i8"
                        && llvm_field_ty != "i1"
                        && !llvm_field_ty.ends_with('*');
                    let field_size = if is_compound_field {
                        self.compute_sizeof(&field_type)
                    } else {
                        0
                    };

                    if match_val.starts_with('%') {
                        // match_val is a pointer to the enum - use getelementptr
                        // Access payload field: first get to the payload (index 1),
                        // then get the specific field within the payload
                        let payload_ptr = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 1, i32 {}",
                            payload_ptr,
                            enum_name,
                            enum_name,
                            match_val,
                            i
                        );

                        if is_compound_field && field_size > 8 {
                            // Large compound type (>8 bytes, e.g., str, large struct):
                            // payload i64 slot holds a heap pointer.
                            // Load the i64, convert to typed pointer, then load value.
                            let raw_i64 = self.next_temp(counter);
                            write_ir!(ir, "  {} = load i64, i64* {}", raw_i64, payload_ptr);
                            self.fn_ctx.record_emitted_type(&raw_i64, "i64");
                            let typed_ptr = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = inttoptr i64 {} to {}*",
                                typed_ptr,
                                raw_i64,
                                llvm_field_ty
                            );
                            self.fn_ctx
                                .record_emitted_type(&typed_ptr, &format!("{}*", llvm_field_ty));
                            if matches!(&field_type, ResolvedType::Named { .. }) {
                                // Struct type: bind as pointer (field access uses GEP)
                                let bind_ir = self.generate_pattern_bindings_typed(
                                    field_pat,
                                    &typed_ptr,
                                    counter,
                                    &field_type,
                                )?;
                                ir.push_str(&bind_ir);
                            } else {
                                // Non-struct compound type (e.g., str, large tuple): load the value
                                let field_val = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = load {}, {}* {}",
                                    field_val,
                                    llvm_field_ty,
                                    llvm_field_ty,
                                    typed_ptr
                                );
                                self.fn_ctx.record_emitted_type(&field_val, &llvm_field_ty);
                                if matches!(&field_type, ResolvedType::Tuple(_)) {
                                    self.fn_ctx
                                        .register_temp_type(&field_val, field_type.clone());
                                }
                                let bind_ir = self.generate_pattern_bindings_typed(
                                    field_pat,
                                    &field_val,
                                    counter,
                                    &field_type,
                                )?;
                                ir.push_str(&bind_ir);
                            }
                        } else if is_compound_field {
                            // Small compound type (<=8 bytes): payload i64 slot holds the
                            // type's raw bits via bitcast. Bitcast the i64* to type*.
                            let cast_ptr = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = bitcast i64* {} to {}*",
                                cast_ptr,
                                payload_ptr,
                                llvm_field_ty
                            );
                            self.fn_ctx
                                .record_emitted_type(&cast_ptr, &format!("{}*", llvm_field_ty));
                            if matches!(&field_type, ResolvedType::Named { .. }) {
                                // Struct type: bind as pointer (field access uses GEP)
                                let bind_ir = self.generate_pattern_bindings_typed(
                                    field_pat,
                                    &cast_ptr,
                                    counter,
                                    &field_type,
                                )?;
                                ir.push_str(&bind_ir);
                            } else {
                                // Non-struct compound type (e.g., tuple, str): load the value
                                let field_val = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = load {}, {}* {}",
                                    field_val,
                                    llvm_field_ty,
                                    llvm_field_ty,
                                    cast_ptr
                                );
                                self.fn_ctx.record_emitted_type(&field_val, &llvm_field_ty);
                                if matches!(&field_type, ResolvedType::Tuple(_)) {
                                    self.fn_ctx
                                        .register_temp_type(&field_val, field_type.clone());
                                }
                                let bind_ir = self.generate_pattern_bindings_typed(
                                    field_pat,
                                    &field_val,
                                    counter,
                                    &field_type,
                                )?;
                                ir.push_str(&bind_ir);
                            }
                        } else if matches!(&field_type, ResolvedType::Named { name, .. } if name == "Box")
                        {
                            let raw_val = self.next_temp(counter);
                            write_ir!(ir, "  {} = load i64, i64* {}", raw_val, payload_ptr);
                            self.fn_ctx.record_emitted_type(&raw_val, "i64");
                            let bind_ir = self.generate_pattern_bindings_typed(
                                field_pat,
                                &raw_val,
                                counter,
                                &field_type,
                            )?;
                            ir.push_str(&bind_ir);
                        } else if matches!(&field_type, ResolvedType::Named { .. }) {
                            // Struct field in native payload slot (custom enum with concrete
                            // struct variant). The GEP already returns the correct struct
                            // pointer type, so bind directly as pointer.
                            let bind_ir = self.generate_pattern_bindings_typed(
                                field_pat,
                                &payload_ptr,
                                counter,
                                &field_type,
                            )?;
                            self.fn_ctx
                                .record_emitted_type(&payload_ptr, &format!("{}*", llvm_field_ty));
                            ir.push_str(&bind_ir);
                        } else if let ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) =
                            &field_type
                        {
                            // Reference payloads are stored in the enum's i64 payload slot.
                            // Recover the typed pointee pointer before binding so
                            // `Ok(v) => v` can feed a `%T*` phi instead of a raw i64.
                            let raw_val = self.next_temp(counter);
                            write_ir!(ir, "  {} = load i64, i64* {}", raw_val, payload_ptr);
                            self.fn_ctx.record_emitted_type(&raw_val, "i64");
                            let field_val = self.next_temp(counter);
                            let inner_llvm = self.type_to_llvm(inner);
                            let target = format!("{}*", inner_llvm);
                            write_ir!(
                                ir,
                                "  {} = inttoptr i64 {} to {}",
                                field_val,
                                raw_val,
                                target
                            );
                            self.fn_ctx.record_emitted_type(&field_val, &target);
                            let bind_ir = self.generate_pattern_bindings_typed(
                                field_pat,
                                &field_val,
                                counter,
                                &field_type,
                            )?;
                            ir.push_str(&bind_ir);
                        } else {
                            // Simple type (i64, i32, pointer, etc.): load directly
                            let raw_val = self.next_temp(counter);
                            write_ir!(ir, "  {} = load i64, i64* {}", raw_val, payload_ptr);
                            self.fn_ctx.record_emitted_type(&raw_val, "i64");
                            let field_val = if llvm_field_ty == "i1" {
                                // Bool field: payload slot is i64, but binding needs i1 for branch instructions
                                let bool_val = self.next_temp(counter);
                                write_ir!(ir, "  {} = trunc i64 {} to i1", bool_val, raw_val);
                                self.fn_ctx.record_emitted_type(&bool_val, "i1");
                                bool_val
                            } else if matches!(llvm_field_ty.as_str(), "i8" | "i16" | "i32") {
                                // Narrow int: trunc from the i64 payload slot.
                                let narrowed = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = trunc i64 {} to {}",
                                    narrowed,
                                    raw_val,
                                    llvm_field_ty
                                );
                                narrowed
                            } else if llvm_field_ty == "double" {
                                let fp = self.next_temp(counter);
                                write_ir!(ir, "  {} = bitcast i64* {} to double*", fp, payload_ptr);
                                let fv = self.next_temp(counter);
                                write_ir!(ir, "  {} = load double, double* {}", fv, fp);
                                self.fn_ctx.record_emitted_type(&fv, "double");
                                fv
                            } else if llvm_field_ty == "float" {
                                let fp = self.next_temp(counter);
                                write_ir!(ir, "  {} = bitcast i64* {} to float*", fp, payload_ptr);
                                let fv = self.next_temp(counter);
                                write_ir!(ir, "  {} = load float, float* {}", fv, fp);
                                self.fn_ctx.record_emitted_type(&fv, "float");
                                fv
                            } else {
                                raw_val
                            };
                            let bind_ir = self.generate_pattern_bindings_typed(
                                field_pat,
                                &field_val,
                                counter,
                                &field_type,
                            )?;
                            ir.push_str(&bind_ir);
                        }
                    } else {
                        // It's a value - use extractvalue
                        // Extract from payload sub-struct: index 1 for payload, then i for field
                        let payload_val = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = extractvalue %{} {}, 1, {}",
                            payload_val,
                            enum_name,
                            match_val,
                            i
                        );

                        if is_compound_field && field_size > 8 {
                            // Large compound type: extractvalue yields i64 heap pointer.
                            // Convert to typed pointer, then load value.
                            let typed_ptr = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = inttoptr i64 {} to {}*",
                                typed_ptr,
                                payload_val,
                                llvm_field_ty
                            );
                            self.fn_ctx
                                .record_emitted_type(&typed_ptr, &format!("{}*", llvm_field_ty));
                            if matches!(&field_type, ResolvedType::Named { .. }) {
                                // Struct type: bind as pointer (field access uses GEP)
                                let bind_ir = self.generate_pattern_bindings_typed(
                                    field_pat,
                                    &typed_ptr,
                                    counter,
                                    &field_type,
                                )?;
                                ir.push_str(&bind_ir);
                            } else {
                                // Non-struct compound type (e.g., str, large tuple): load value
                                let field_val = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = load {}, {}* {}",
                                    field_val,
                                    llvm_field_ty,
                                    llvm_field_ty,
                                    typed_ptr
                                );
                                let bind_ir = self.generate_pattern_bindings_typed(
                                    field_pat,
                                    &field_val,
                                    counter,
                                    &field_type,
                                )?;
                                ir.push_str(&bind_ir);
                            }
                        } else if is_compound_field {
                            // Small compound type: extractvalue yields i64 containing type bits.
                            // Store to an alloca and bitcast to typed pointer.
                            let tmp_alloca = self.next_temp(counter);
                            self.emit_entry_alloca(&tmp_alloca, "i64");
                            write_ir!(ir, "  store i64 {}, i64* {}", payload_val, tmp_alloca);
                            let cast_ptr = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = bitcast i64* {} to {}*",
                                cast_ptr,
                                tmp_alloca,
                                llvm_field_ty
                            );
                            self.fn_ctx
                                .record_emitted_type(&cast_ptr, &format!("{}*", llvm_field_ty));
                            if matches!(&field_type, ResolvedType::Named { .. }) {
                                // Struct type: bind as pointer (field access uses GEP)
                                let bind_ir = self.generate_pattern_bindings_typed(
                                    field_pat,
                                    &cast_ptr,
                                    counter,
                                    &field_type,
                                )?;
                                ir.push_str(&bind_ir);
                            } else {
                                // Non-struct compound type: load the value
                                let field_val = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = load {}, {}* {}",
                                    field_val,
                                    llvm_field_ty,
                                    llvm_field_ty,
                                    cast_ptr
                                );
                                let bind_ir = self.generate_pattern_bindings_typed(
                                    field_pat,
                                    &field_val,
                                    counter,
                                    &field_type,
                                )?;
                                ir.push_str(&bind_ir);
                            }
                        } else if let ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) =
                            &field_type
                        {
                            let field_val = self.next_temp(counter);
                            let inner_llvm = self.type_to_llvm(inner);
                            let target = format!("{}*", inner_llvm);
                            write_ir!(
                                ir,
                                "  {} = inttoptr i64 {} to {}",
                                field_val,
                                payload_val,
                                target
                            );
                            self.fn_ctx.record_emitted_type(&field_val, &target);
                            let bind_ir = self.generate_pattern_bindings_typed(
                                field_pat,
                                &field_val,
                                counter,
                                &field_type,
                            )?;
                            ir.push_str(&bind_ir);
                        } else {
                            let field_val = if llvm_field_ty == "i1" {
                                // Bool field: extracted payload is i64, but binding needs i1 for branch instructions
                                let bool_val = self.next_temp(counter);
                                write_ir!(ir, "  {} = trunc i64 {} to i1", bool_val, payload_val);
                                self.fn_ctx.record_emitted_type(&bool_val, "i1");
                                bool_val
                            } else {
                                payload_val.clone()
                            };
                            let bind_ir = self.generate_pattern_bindings_typed(
                                field_pat,
                                &field_val,
                                counter,
                                &field_type,
                            )?;
                            ir.push_str(&bind_ir);
                        }
                    }
                }

                Ok(ir)
            }
            Pattern::Struct {
                name,
                fields,
                enum_name: enum_hint,
            } => {
                // Bind fields from struct
                let struct_name = &name.node;
                let enum_hint_ref = enum_hint.as_deref();
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
                            write_ir!(
                                ir,
                                "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}",
                                field_ptr,
                                struct_name,
                                struct_name,
                                match_val,
                                field_idx
                            );

                            let field_val = self.next_temp(counter);
                            let field_ty = &struct_info.fields[field_idx].1;
                            let llvm_ty = self.type_to_llvm(field_ty);
                            write_ir!(
                                ir,
                                "  {} = load {}, {}* {}",
                                field_val,
                                llvm_ty,
                                llvm_ty,
                                field_ptr
                            );

                            if let Some(pat) = field_pat {
                                // Bind to pattern — pass field's actual type so
                                // Ident-pattern bindings register the correct
                                // LocalVar type (e.g., F64 for `FloatVal { v: a }`).
                                let bind_ir = self.generate_pattern_bindings_typed(
                                    pat, &field_val, counter, field_ty,
                                )?;
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
                } else if let Some((enum_name, _variant_tag, variant_struct_fields)) = {
                    let requested: Vec<&str> =
                        fields.iter().map(|(fn_, _)| fn_.node.as_str()).collect();
                    self.resolve_enum_struct_variant_with_hint_and_fields(
                        struct_name,
                        enum_hint_ref,
                        &requested,
                    )
                } {
                    // `struct_name` is actually an enum struct-variant used in short form
                    // (e.g. `Varchar { max_len }` instead of `SqlType.Varchar { max_len }`).
                    // The enum layout is `%EnumName { i32 tag, { i64, i64, ... } payload }`,
                    // so each named field lives in payload slot at its declaration index.
                    for (field_name, field_pat) in fields {
                        let field_idx = match variant_struct_fields
                            .iter()
                            .position(|(n, _)| n == &field_name.node)
                        {
                            Some(i) => i,
                            None => continue,
                        };
                        let field_ty = variant_struct_fields[field_idx].1.clone();

                        let payload_ptr = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 1, i32 {}",
                            payload_ptr,
                            enum_name,
                            enum_name,
                            match_val,
                            field_idx
                        );

                        // Payload slots are stored as i64 in the enum layout. For simple
                        // integer / bool / pointer fields we load directly as i64 and let
                        // downstream uses truncate. For float fields we must load the
                        // matching-width float so assertions like `assert_approx(v, ...)`
                        // receive a `double` and not an `i64` bit pattern.
                        let mut field_val = self.next_temp(counter);
                        match &field_ty {
                            crate::ResolvedType::F64 => {
                                let float_ptr = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = bitcast i64* {} to double*",
                                    float_ptr,
                                    payload_ptr
                                );
                                write_ir!(
                                    ir,
                                    "  {} = load double, double* {}",
                                    field_val,
                                    float_ptr
                                );
                                self.fn_ctx.record_emitted_type(&field_val, "double");
                            }
                            crate::ResolvedType::F32 => {
                                let float_ptr = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = bitcast i64* {} to float*",
                                    float_ptr,
                                    payload_ptr
                                );
                                write_ir!(ir, "  {} = load float, float* {}", field_val, float_ptr);
                                self.fn_ctx.record_emitted_type(&field_val, "float");
                            }
                            crate::ResolvedType::Bool => {
                                // Bool payloads are stored in an i64 slot; load i64 then
                                // truncate to i1 so the bound value matches Bool codegen
                                // conventions (used by `br i1`, logical ops, etc.).
                                let raw = self.next_temp(counter);
                                write_ir!(ir, "  {} = load i64, i64* {}", raw, payload_ptr);
                                self.fn_ctx.record_emitted_type(&raw, "i64");
                                write_ir!(ir, "  {} = trunc i64 {} to i1", field_val, raw);
                                self.fn_ctx.record_emitted_type(&field_val, "i1");
                            }
                            crate::ResolvedType::I32
                            | crate::ResolvedType::U32
                            | crate::ResolvedType::I16
                            | crate::ResolvedType::U16
                            | crate::ResolvedType::I8
                            | crate::ResolvedType::U8 => {
                                // Narrow int payloads still live in an i64 slot — load
                                // the slot and truncate to the declared field width so
                                // call-site ABIs receive the correct narrow integer.
                                let narrow_ty = self.type_to_llvm(&field_ty);
                                let raw = self.next_temp(counter);
                                write_ir!(ir, "  {} = load i64, i64* {}", raw, payload_ptr);
                                self.fn_ctx.record_emitted_type(&raw, "i64");
                                write_ir!(
                                    ir,
                                    "  {} = trunc i64 {} to {}",
                                    field_val,
                                    raw,
                                    narrow_ty
                                );
                            }
                            crate::ResolvedType::Str => {
                                // Str payloads (> 8 bytes) are heap-allocated in the
                                // enum variant constructor and stored as an i64 pointer
                                // in the payload slot. To recover the { i8*, i64 } fat
                                // pointer, load the slot, inttoptr to the fat-pointer
                                // type, and load through it.
                                let raw = self.next_temp(counter);
                                write_ir!(ir, "  {} = load i64, i64* {}", raw, payload_ptr);
                                self.fn_ctx.record_emitted_type(&raw, "i64");
                                let typed = self.next_temp(counter);
                                write_ir!(
                                    ir,
                                    "  {} = inttoptr i64 {} to {{ i8*, i64 }}*",
                                    typed,
                                    raw
                                );
                                write_ir!(
                                    ir,
                                    "  {} = load {{ i8*, i64 }}, {{ i8*, i64 }}* {}",
                                    field_val,
                                    typed
                                );
                                self.fn_ctx.record_emitted_type(&field_val, "{ i8*, i64 }");
                            }
                            crate::ResolvedType::Named { name, .. } if name == "Box" => {
                                let raw = self.next_temp(counter);
                                write_ir!(ir, "  {} = load i64, i64* {}", raw, payload_ptr);
                                self.fn_ctx.record_emitted_type(&raw, "i64");
                                field_val = raw;
                            }
                            crate::ResolvedType::Named { .. } => {
                                let llvm_ty = self.type_to_llvm(&field_ty);
                                if self.compute_sizeof(&field_ty) <= 8 {
                                    // Small named payloads (notably nested enum tags
                                    // such as BinOp) are stored inline in the i64
                                    // payload slot. Bind the slot itself as a typed
                                    // pointer; inttoptr would treat inline bits as an
                                    // address and can segfault.
                                    write_ir!(
                                        ir,
                                        "  {} = bitcast i64* {} to {}*",
                                        field_val,
                                        payload_ptr,
                                        llvm_ty
                                    );
                                    self.fn_ctx
                                        .record_emitted_type(&field_val, &format!("{}*", llvm_ty));
                                } else {
                                    // Larger named payloads (Vec<T>, user structs,
                                    // nested large enums) are heap-allocated and the
                                    // payload slot stores their pointer as i64.
                                    let raw = self.next_temp(counter);
                                    write_ir!(ir, "  {} = load i64, i64* {}", raw, payload_ptr);
                                    self.fn_ctx.record_emitted_type(&raw, "i64");
                                    write_ir!(
                                        ir,
                                        "  {} = inttoptr i64 {} to {}*",
                                        field_val,
                                        raw,
                                        llvm_ty
                                    );
                                    self.fn_ctx
                                        .record_emitted_type(&field_val, &format!("{}*", llvm_ty));
                                }
                            }
                            crate::ResolvedType::Ref(inner)
                            | crate::ResolvedType::RefMut(inner) => {
                                let inner_llvm = self.type_to_llvm(inner);
                                let target = format!("{}*", inner_llvm);
                                let raw = self.next_temp(counter);
                                write_ir!(ir, "  {} = load i64, i64* {}", raw, payload_ptr);
                                self.fn_ctx.record_emitted_type(&raw, "i64");
                                write_ir!(
                                    ir,
                                    "  {} = inttoptr i64 {} to {}",
                                    field_val,
                                    raw,
                                    target
                                );
                                self.fn_ctx.record_emitted_type(&field_val, &target);
                            }
                            _ => {
                                write_ir!(ir, "  {} = load i64, i64* {}", field_val, payload_ptr);
                            }
                        }

                        if let Some(pat) = field_pat {
                            let bind_ir = self.generate_pattern_bindings_typed(
                                pat, &field_val, counter, &field_ty,
                            )?;
                            ir.push_str(&bind_ir);
                        } else {
                            self.fn_ctx.locals.insert(
                                field_name.node.clone(),
                                LocalVar::ssa(field_ty, field_val.clone()),
                            );
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

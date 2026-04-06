use super::*;
use crate::types::LocalVar;

impl CodeGenerator {
    /// Generate code to check if a pattern matches (with explicit match type for correct type
    /// propagation, especially for nested tuple patterns where element types may be non-i64).
    #[inline(never)]
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
                    let expected_tag = self.get_enum_variant_tag(name);

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
                    // Codegen widens all narrow integers (u8, i8, u16, etc.) to i64,
                    // so always use i64 for the comparison — matches Pattern::Literal(Int) behavior.
                    write_ir!(
                        ir,
                        "  {} = icmp eq i64 {}, {}",
                        result,
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

                // Get the enum type name for proper LLVM type reference
                let enum_name = self
                    .get_enum_name_for_variant(variant_name)
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
                let expected_tag = self.get_enum_variant_tag(variant_name);

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
                } else if self
                    .resolve_enum_struct_variant(struct_name)
                    .is_some()
                {
                    // `struct_name` names an enum struct-variant (e.g. `Varchar { max_len }`
                    // used as short-form inside a `M` arm). Verify the enum's runtime tag
                    // matches and delegate any inner sub-pattern checks through the enum
                    // variant payload layout.
                    let (enum_name, _variant_tag, variant_struct_fields) =
                        self.resolve_enum_struct_variant(struct_name).unwrap();

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
                    let expected_tag = self.get_enum_variant_tag(struct_name);
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
                                write_ir!(
                                    ir,
                                    "  {} = load i64, i64* {}",
                                    field_val,
                                    payload_ptr
                                );
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
    pub(crate) fn resolve_enum_struct_variant(
        &self,
        variant_name: &str,
    ) -> Option<(String, i32, Vec<(String, ResolvedType)>)> {
        use crate::types::EnumVariantFields;
        for enum_info in self.types.enums.values() {
            for (tag, variant) in enum_info.variants.iter().enumerate() {
                if variant.name == variant_name {
                    if let EnumVariantFields::Struct(fields) = &variant.fields {
                        return Some((enum_info.name.clone(), tag as i32, fields.clone()));
                    }
                }
            }
        }
        None
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
    fn resolve_variant_field_types(
        &self,
        enum_name: &str,
        variant_name: &str,
        match_type: &ResolvedType,
    ) -> Vec<ResolvedType> {
        use crate::types::EnumVariantFields;

        // Look up the enum definition to find the variant's field types
        let enum_info = match self.types.enums.get(enum_name) {
            Some(info) => info,
            None => return vec![],
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

        // Extract the concrete generic args from match_type (e.g., [Vec<u64>] from Option<Vec<u64>>)
        let concrete_generics = match match_type {
            ResolvedType::Named { generics, .. } if !generics.is_empty() => generics,
            _ => {
                // Non-generic enum: field types are already concrete, return them directly.
                // This handles enums like `E QueryType { Select(str), Insert(str) }` where
                // the field types (str) don't need any generic substitution.
                return raw_field_types.into_iter().cloned().collect();
            }
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
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) | ResolvedType::Pointer(inner) => {
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
            ResolvedType::Generic(name) => {
                subs.get(name).cloned().unwrap_or_else(|| ty.clone())
            }
            ResolvedType::Named { name, generics } => ResolvedType::Named {
                name: name.clone(),
                generics: generics.iter().map(|g| Self::substitute_generics(g, subs)).collect(),
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
            ResolvedType::Tuple(elems) => {
                ResolvedType::Tuple(elems.iter().map(|e| Self::substitute_generics(e, subs)).collect())
            }
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

                // Resolve the actual field types for this variant, substituting generic
                // parameters with concrete types from match_type.
                // e.g., Option<Vec<u64>>::Some(T) → field type is Vec<u64>
                let variant_field_types = self.resolve_variant_field_types(
                    &enum_name, variant_name, match_type,
                );

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
                            let typed_ptr = self.next_temp(counter);
                            write_ir!(
                                ir,
                                "  {} = inttoptr i64 {} to {}*",
                                typed_ptr,
                                raw_i64,
                                llvm_field_ty
                            );
                            if matches!(&field_type, ResolvedType::Named { .. }) {
                                // Struct type: bind as pointer (field access uses GEP)
                                let bind_ir =
                                    self.generate_pattern_bindings_typed(field_pat, &typed_ptr, counter, &field_type)?;
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
                                let bind_ir =
                                    self.generate_pattern_bindings_typed(field_pat, &field_val, counter, &field_type)?;
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
                            if matches!(&field_type, ResolvedType::Named { .. }) {
                                // Struct type: bind as pointer (field access uses GEP)
                                let bind_ir =
                                    self.generate_pattern_bindings_typed(field_pat, &cast_ptr, counter, &field_type)?;
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
                                let bind_ir =
                                    self.generate_pattern_bindings_typed(field_pat, &field_val, counter, &field_type)?;
                                ir.push_str(&bind_ir);
                            }
                        } else if matches!(&field_type, ResolvedType::Named { .. }) {
                            // Struct field in native payload slot (custom enum with concrete
                            // struct variant). The GEP already returns the correct struct
                            // pointer type, so bind directly as pointer.
                            let bind_ir =
                                self.generate_pattern_bindings_typed(field_pat, &payload_ptr, counter, &field_type)?;
                            ir.push_str(&bind_ir);
                        } else {
                            // Simple type (i64, i32, pointer, etc.): load directly
                            let field_val = self.next_temp(counter);
                            write_ir!(ir, "  {} = load i64, i64* {}", field_val, payload_ptr);
                            let bind_ir =
                                self.generate_pattern_bindings_typed(field_pat, &field_val, counter, &field_type)?;
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
                            if matches!(&field_type, ResolvedType::Named { .. }) {
                                // Struct type: bind as pointer (field access uses GEP)
                                let bind_ir =
                                    self.generate_pattern_bindings_typed(field_pat, &typed_ptr, counter, &field_type)?;
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
                                let bind_ir =
                                    self.generate_pattern_bindings_typed(field_pat, &field_val, counter, &field_type)?;
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
                            if matches!(&field_type, ResolvedType::Named { .. }) {
                                // Struct type: bind as pointer (field access uses GEP)
                                let bind_ir =
                                    self.generate_pattern_bindings_typed(field_pat, &cast_ptr, counter, &field_type)?;
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
                                let bind_ir =
                                    self.generate_pattern_bindings_typed(field_pat, &field_val, counter, &field_type)?;
                                ir.push_str(&bind_ir);
                            }
                        } else {
                            let bind_ir =
                                self.generate_pattern_bindings_typed(field_pat, &payload_val, counter, &field_type)?;
                            ir.push_str(&bind_ir);
                        }
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
                } else if let Some((enum_name, _variant_tag, variant_struct_fields)) =
                    self.resolve_enum_struct_variant(struct_name)
                {
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
                        // integer / bool / pointer fields we can load directly as i64 and
                        // let downstream uses truncate; for compound types we follow the
                        // same bitcast/heap-pointer conventions used by the Pattern::Variant
                        // binding path. Here we default to the simple i64 load — enum
                        // struct-variants with large compound field types are uncommon and
                        // currently only affect `Varchar { max_len: u32 }` / `Vector { dim:
                        // u32 }` style variants in vaisdb, which fit in an i64.
                        let field_val = self.next_temp(counter);
                        write_ir!(
                            ir,
                            "  {} = load i64, i64* {}",
                            field_val,
                            payload_ptr
                        );

                        if let Some(pat) = field_pat {
                            let bind_ir =
                                self.generate_pattern_bindings(pat, &field_val, counter)?;
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

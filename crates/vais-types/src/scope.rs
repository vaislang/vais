//! Variable scope management and pattern binding.

use std::collections::HashMap;

use vais_ast::*;

use super::TypeChecker;
use crate::types::defs::VarInfo;
use crate::types::{Linearity, ResolvedType, TypeError, TypeResult, VariantFieldTypes};

impl TypeChecker {
    // === Scope management ===

    #[inline]
    pub(crate) fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    #[inline]
    pub(crate) fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Check for unused variables in the current scope and emit warnings.
    /// Variables prefixed with `_` are considered intentionally unused.
    /// Function parameters are excluded from this check (they are often
    /// intentionally unused, especially in trait implementations and stubs).
    pub(crate) fn check_unused_variables(&mut self, excluded_names: &[String]) {
        if let Some(scope) = self.scopes.last() {
            for (name, var_info) in scope.iter() {
                // Skip special names, underscore-prefixed names, and function parameters
                if name == "self" || name == "return" || name.starts_with('_') {
                    continue;
                }
                if excluded_names.contains(name) {
                    continue;
                }
                if var_info.use_count == 0 {
                    self.warnings.push(format!(
                        "unused variable `{}`. If intentional, prefix with underscore: `_{}`",
                        name, name
                    ));
                }
            }
        }
    }

    #[inline]
    pub(crate) fn define_var(&mut self, name: &str, ty: ResolvedType, is_mut: bool) {
        self.define_var_with_linearity(name, ty, is_mut, Linearity::Unrestricted, None);
    }

    pub(crate) fn define_var_with_linearity(
        &mut self,
        name: &str,
        ty: ResolvedType,
        is_mut: bool,
        linearity: Linearity,
        defined_at: Option<Span>,
    ) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name.to_string(),
                VarInfo {
                    ty,
                    is_mut,
                    linearity,
                    use_count: 0,
                    defined_at,
                },
            );
        }
    }

    /// Mark a variable as used (for linear type tracking)
    #[inline]
    pub(crate) fn mark_var_used(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var_info) = scope.get_mut(name) {
                var_info.use_count += 1;
                return;
            }
        }
    }

    /// Check linear/affine variable usage at scope exit.
    /// Reserved for future linear type system enforcement.
    #[allow(dead_code)]
    pub(crate) fn check_linear_vars_at_scope_exit(&self) -> TypeResult<()> {
        if let Some(scope) = self.scopes.last() {
            for (name, var_info) in scope.iter() {
                if name == "self" || name == "return" {
                    continue;
                }
                match var_info.linearity {
                    Linearity::Linear => {
                        if var_info.use_count != 1 {
                            return Err(TypeError::LinearTypeViolation {
                                var_name: name.clone(),
                                expected_uses: 1,
                                actual_uses: var_info.use_count,
                                defined_at: var_info.defined_at,
                            });
                        }
                    }
                    Linearity::Affine => {
                        if var_info.use_count > 1 {
                            return Err(TypeError::AffineTypeViolation {
                                var_name: name.clone(),
                                actual_uses: var_info.use_count,
                                defined_at: var_info.defined_at,
                            });
                        }
                    }
                    Linearity::Unrestricted => {}
                }
            }
        }
        Ok(())
    }

    /// Get field types for a struct or enum struct variant.
    /// Used in pattern matching to properly type-check struct patterns.
    /// Returns a map of field names to their types.
    pub(crate) fn get_struct_or_variant_fields(
        &self,
        pattern_name: &str,
        expr_type: &ResolvedType,
    ) -> HashMap<String, ResolvedType> {
        // First, check if pattern_name refers to a struct
        if let Some(struct_def) = self.structs.get(pattern_name) {
            // If we have concrete generics in expr_type, substitute them
            if let ResolvedType::Named {
                generics: concrete_generics,
                ..
            } = expr_type
            {
                if !concrete_generics.is_empty() && !struct_def.generics.is_empty() {
                    let substitutions: HashMap<String, ResolvedType> = struct_def
                        .generics
                        .iter()
                        .zip(concrete_generics.iter())
                        .map(|(param, concrete)| (param.clone(), concrete.clone()))
                        .collect();
                    return struct_def
                        .fields
                        .iter()
                        .map(|(name, ty)| {
                            (name.clone(), self.substitute_generics(ty, &substitutions))
                        })
                        .collect();
                }
            }
            return struct_def.fields.clone();
        }

        // Otherwise, try to find it as an enum variant
        // Extract enum name and generics from expr_type
        if let ResolvedType::Named {
            name: enum_name,
            generics: concrete_generics,
        } = expr_type
        {
            if let Some(enum_def) = self.enums.get(enum_name) {
                if let Some(VariantFieldTypes::Struct(fields)) = enum_def.variants.get(pattern_name)
                {
                    // Build substitution map from generic params to concrete types
                    let substitutions: HashMap<String, ResolvedType> = enum_def
                        .generics
                        .iter()
                        .zip(concrete_generics.iter())
                        .map(|(param, concrete)| (param.clone(), concrete.clone()))
                        .collect();
                    return fields
                        .iter()
                        .map(|(name, ty)| {
                            (name.clone(), self.substitute_generics(ty, &substitutions))
                        })
                        .collect();
                }
            }
        }

        // If not found, return empty map
        HashMap::new()
    }

    /// Get tuple field types for an enum tuple variant.
    /// Used in pattern matching to properly type-check variant tuple patterns.
    /// Returns a vector of field types in order.
    pub(crate) fn get_tuple_variant_fields(
        &self,
        pattern_name: &str,
        expr_type: &ResolvedType,
    ) -> Vec<ResolvedType> {
        // Extract enum name and generics from expr_type
        if let ResolvedType::Named {
            name: enum_name,
            generics: concrete_generics,
        } = expr_type
        {
            if let Some(enum_def) = self.enums.get(enum_name) {
                if let Some(variant_fields) = enum_def.variants.get(pattern_name) {
                    // Build substitution map from generic params to concrete types
                    let substitutions: HashMap<String, ResolvedType> = enum_def
                        .generics
                        .iter()
                        .zip(concrete_generics.iter())
                        .map(|(param, concrete)| (param.clone(), concrete.clone()))
                        .collect();

                    match variant_fields {
                        VariantFieldTypes::Tuple(types) => {
                            // Substitute generics with concrete types
                            return types
                                .iter()
                                .map(|t| self.substitute_generics(t, &substitutions))
                                .collect();
                        }
                        VariantFieldTypes::Unit => return vec![],
                        VariantFieldTypes::Struct(_) => return vec![], // Wrong pattern type
                    }
                }
            }
        }

        // If not found, return empty vec
        vec![]
    }

    /// Register pattern bindings in the current scope
    pub(crate) fn register_pattern_bindings(
        &mut self,
        pattern: &Spanned<Pattern>,
        expr_type: &ResolvedType,
    ) -> TypeResult<()> {
        match &pattern.node {
            Pattern::Wildcard => Ok(()),
            Pattern::Ident(name) => {
                // Bind the identifier to the matched expression's type
                self.define_var(name, expr_type.clone(), false);
                Ok(())
            }
            Pattern::Literal(_) => Ok(()), // Literals don't bind variables
            Pattern::Tuple(patterns) => {
                if let ResolvedType::Tuple(types) = expr_type {
                    for (pat, ty) in patterns.iter().zip(types.iter()) {
                        self.register_pattern_bindings(pat, ty)?;
                    }
                } else {
                    // If type doesn't match, still try to bind with unknown types
                    for pat in patterns {
                        self.register_pattern_bindings(pat, &ResolvedType::Unknown)?;
                    }
                }
                Ok(())
            }
            Pattern::Struct { name, fields } => {
                // For struct patterns, look up field types from the struct or enum variant
                let field_types = self.get_struct_or_variant_fields(&name.node, expr_type);

                for (field_name, sub_pattern) in fields {
                    let field_type = field_types
                        .get(&field_name.node)
                        .cloned()
                        .unwrap_or(ResolvedType::Unknown);

                    if let Some(sub_pat) = sub_pattern {
                        self.register_pattern_bindings(sub_pat, &field_type)?;
                    } else {
                        // Shorthand: `Point { x, y }` binds x and y
                        self.define_var(&field_name.node, field_type, false);
                    }
                }
                Ok(())
            }
            Pattern::Variant { name, fields } => {
                // For tuple-style enum variants, look up field types
                let variant_field_types = self.get_tuple_variant_fields(&name.node, expr_type);

                for (field, field_type) in fields.iter().zip(variant_field_types.iter()) {
                    self.register_pattern_bindings(field, field_type)?;
                }

                // If more fields in pattern than in variant, use Unknown
                for field in fields.iter().skip(variant_field_types.len()) {
                    self.register_pattern_bindings(field, &ResolvedType::Unknown)?;
                }
                Ok(())
            }
            Pattern::Range { .. } => Ok(()), // Ranges don't bind variables
            Pattern::Or(patterns) => {
                // For or patterns, all alternatives must bind the same set of variables.
                // Process the first alternative to establish bindings, then verify
                // that subsequent alternatives bind the same variable names.
                if let Some(first) = patterns.first() {
                    self.register_pattern_bindings(first, expr_type)?;

                    // Collect variable names bound by the first alternative
                    let first_bindings = Self::collect_or_pattern_var_names(&first.node);

                    // Verify subsequent alternatives bind the same variables
                    for alt in patterns.iter().skip(1) {
                        let alt_bindings = Self::collect_or_pattern_var_names(&alt.node);
                        if first_bindings != alt_bindings {
                            self.warnings.push(format!(
                                "or-pattern alternatives bind different variables: {:?} vs {:?}",
                                first_bindings, alt_bindings
                            ));
                        }
                    }
                }
                Ok(())
            }
            Pattern::Alias { name, pattern } => {
                // Bind the alias name to the whole matched value
                self.define_var(name, expr_type.clone(), false);
                // Then bind variables from the inner pattern
                self.register_pattern_bindings(pattern, expr_type)?;
                Ok(())
            }
        }
    }

    /// Collect variable names bound by a pattern (without modifying scope).
    /// Used to verify that or-pattern alternatives bind the same variables.
    fn collect_or_pattern_var_names(pattern: &Pattern) -> std::collections::BTreeSet<String> {
        let mut names = std::collections::BTreeSet::new();
        Self::collect_or_pattern_var_names_rec(pattern, &mut names);
        names
    }

    fn collect_or_pattern_var_names_rec(
        pattern: &Pattern,
        names: &mut std::collections::BTreeSet<String>,
    ) {
        match pattern {
            Pattern::Wildcard | Pattern::Literal(_) | Pattern::Range { .. } => {}
            Pattern::Ident(name) => {
                names.insert(name.clone());
            }
            Pattern::Tuple(patterns) => {
                for pat in patterns {
                    Self::collect_or_pattern_var_names_rec(&pat.node, names);
                }
            }
            Pattern::Struct { fields, .. } => {
                for (field_name, sub_pattern) in fields {
                    if let Some(sub_pat) = sub_pattern {
                        Self::collect_or_pattern_var_names_rec(&sub_pat.node, names);
                    } else {
                        // Shorthand: `Point { x, y }` binds x and y
                        names.insert(field_name.node.clone());
                    }
                }
            }
            Pattern::Variant { fields, .. } => {
                for field in fields {
                    Self::collect_or_pattern_var_names_rec(&field.node, names);
                }
            }
            Pattern::Or(patterns) => {
                // For nested or-patterns, collect from the first alternative
                if let Some(first) = patterns.first() {
                    Self::collect_or_pattern_var_names_rec(&first.node, names);
                }
            }
            Pattern::Alias { name, pattern } => {
                names.insert(name.clone());
                Self::collect_or_pattern_var_names_rec(&pattern.node, names);
            }
        }
    }
}

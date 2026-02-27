//! Variable and method lookup resolution.

use std::collections::HashMap;

use super::TypeChecker;
use crate::traits::TraitMethodSig;
use crate::types::defs::VarInfo;
use crate::types::{
    find_similar_name, Linearity, ResolvedType, TypeError, TypeResult, VariantFieldTypes,
};

impl TypeChecker {
    /// Look up "self" variable directly from scopes (no fallback)
    pub(crate) fn lookup_self_var_info(&self) -> TypeResult<VarInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get("self") {
                return Ok(info.clone());
            }
        }
        Err(TypeError::UndefinedVar {
            name: "self".to_string(),
            span: None,
            suggestion: None,
        })
    }

    #[inline]
    pub(crate) fn lookup_var(&self, name: &str) -> Option<ResolvedType> {
        self.lookup_var_info(name).ok().map(|v| v.ty)
    }

    pub(crate) fn lookup_var_with_mut(&self, name: &str) -> Option<(ResolvedType, bool)> {
        self.lookup_var_info(name).ok().map(|v| (v.ty, v.is_mut))
    }

    #[inline]
    pub(crate) fn lookup_var_or_err(&self, name: &str) -> TypeResult<ResolvedType> {
        self.lookup_var_info(name).map(|v| v.ty)
    }

    pub(crate) fn lookup_var_info(&self, name: &str) -> TypeResult<VarInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Ok(info.clone());
            }
        }

        // Check if it's a function
        if let Some(sig) = self.functions.get(name) {
            // For async functions, wrap the return type in Future
            let ret_type = if sig.is_async {
                ResolvedType::Future(Box::new(sig.ret.clone()))
            } else {
                sig.ret.clone()
            };

            return Ok(VarInfo {
                ty: ResolvedType::Fn {
                    params: sig.params.iter().map(|(_, t, _)| t.clone()).collect(),
                    ret: Box::new(ret_type),
                    effects: None,
                },
                is_mut: false,
                linearity: Linearity::Unrestricted,
                use_count: 0,
                defined_at: None,
            });
        }

        // Check if it's an enum variant
        for (enum_name, enum_def) in &self.enums {
            if let Some(variant_fields) = enum_def.variants.get(name) {
                // Create type variables for generic enum parameters
                let generics: Vec<ResolvedType> = enum_def
                    .generics
                    .iter()
                    .map(|_| self.fresh_type_var())
                    .collect();

                // Build substitution map for generic parameters
                let generic_substitutions: HashMap<String, ResolvedType> = enum_def
                    .generics
                    .iter()
                    .zip(generics.iter())
                    .map(|(param, ty)| (param.clone(), ty.clone()))
                    .collect();

                let enum_type = ResolvedType::Named {
                    name: enum_name.clone(),
                    generics,
                };

                match variant_fields {
                    VariantFieldTypes::Unit => {
                        return Ok(VarInfo {
                            ty: enum_type,
                            is_mut: false,
                            linearity: Linearity::Unrestricted,
                            use_count: 0,
                            defined_at: None,
                        });
                    }
                    VariantFieldTypes::Tuple(field_types) => {
                        // Tuple variant acts as a function from field types to enum type
                        let params: Vec<ResolvedType> = field_types
                            .iter()
                            .map(|t| self.substitute_generics(t, &generic_substitutions))
                            .collect();

                        return Ok(VarInfo {
                            ty: ResolvedType::Fn {
                                params,
                                ret: Box::new(enum_type),
                                effects: None,
                            },
                            is_mut: false,
                            linearity: Linearity::Unrestricted,
                            use_count: 0,
                            defined_at: None,
                        });
                    }
                    VariantFieldTypes::Struct(_) => {
                        // Struct variants are handled differently (through struct construction syntax)
                        return Ok(VarInfo {
                            ty: enum_type,
                            is_mut: false,
                            linearity: Linearity::Unrestricted,
                            use_count: 0,
                            defined_at: None,
                        });
                    }
                }
            }
        }

        // Check if it's a constant
        if let Some(const_type) = self.constants.get(name) {
            return Ok(VarInfo {
                ty: const_type.clone(),
                is_mut: false,
                linearity: Linearity::Unrestricted,
                use_count: 0,
                defined_at: None,
            });
        }

        // Implicit self: if in a method context, check struct fields
        if let Ok(self_info) = self.lookup_self_var_info() {
            let inner_type = match &self_info.ty {
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => Some(inner.as_ref()),
                _ => Some(&self_info.ty),
            };
            if let Some(ResolvedType::Named {
                name: struct_name,
                generics: _,
            }) = inner_type
            {
                if let Some(struct_def) = self.structs.get(struct_name.as_str()).cloned() {
                    for (fname, ftype) in &struct_def.fields {
                        if fname == name {
                            return Ok(VarInfo {
                                ty: ftype.clone(),
                                is_mut: self_info.is_mut,
                                linearity: Linearity::Unrestricted,
                                use_count: 0,
                                defined_at: None,
                            });
                        }
                    }
                }
            }
        }

        // Collect all available names for did-you-mean suggestion
        let mut candidates: Vec<&str> = Vec::new();
        for scope in &self.scopes {
            candidates.extend(scope.keys().map(|s| s.as_str()));
        }
        candidates.extend(self.functions.keys().map(|s| s.as_str()));
        candidates.extend(self.constants.keys().map(|s| s.as_str()));
        for enum_def in self.enums.values() {
            candidates.extend(enum_def.variants.keys().map(|s| s.as_str()));
        }

        let suggestion = find_similar_name(name, candidates.into_iter());

        Err(TypeError::UndefinedVar {
            name: name.to_string(),
            span: None,
            suggestion,
        })
    }

    /// Look up a method in a trait definition, walking super traits if not found directly.
    /// Uses a visited set to prevent infinite recursion on cyclic super-trait declarations.
    fn find_method_in_trait_with_supers(
        &self,
        trait_name: &str,
        method_name: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> Option<TraitMethodSig> {
        if !visited.insert(trait_name.to_string()) {
            return None; // Cycle detected
        }
        if let Some(trait_def) = self.traits.get(trait_name) {
            // Check direct methods first
            if let Some(method_sig) = trait_def.methods.get(method_name) {
                return Some(method_sig.clone());
            }
            // Walk super traits
            for super_trait in &trait_def.super_traits {
                if let Some(method_sig) =
                    self.find_method_in_trait_with_supers(super_trait, method_name, visited)
                {
                    return Some(method_sig);
                }
            }
        }
        None
    }

    /// Find a method from trait implementations for a given type
    pub(crate) fn find_trait_method(
        &self,
        receiver_type: &ResolvedType,
        method_name: &str,
    ) -> Option<TraitMethodSig> {
        // Handle dyn Trait types - look up method directly in trait definition
        let dyn_trait = match receiver_type {
            ResolvedType::DynTrait { trait_name, .. } => Some(trait_name.clone()),
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                if let ResolvedType::DynTrait { trait_name, .. } = inner.as_ref() {
                    Some(trait_name.clone())
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(trait_name) = dyn_trait {
            let mut visited = std::collections::HashSet::new();
            return self.find_method_in_trait_with_supers(&trait_name, method_name, &mut visited);
        }

        // Handle generic types with bounds from where clauses
        if let ResolvedType::Generic(type_param) = receiver_type {
            if let Some(bounds) = self.current_generic_bounds.get(type_param) {
                for bound_trait in bounds {
                    let mut visited = std::collections::HashSet::new();
                    if let Some(method_sig) = self.find_method_in_trait_with_supers(
                        bound_trait,
                        method_name,
                        &mut visited,
                    ) {
                        return Some(method_sig);
                    }
                    // Also check trait aliases
                    if let Some(alias_bounds) = self.trait_aliases.get(bound_trait.as_str()) {
                        for alias_trait in alias_bounds {
                            let mut visited = std::collections::HashSet::new();
                            if let Some(method_sig) = self.find_method_in_trait_with_supers(
                                alias_trait,
                                method_name,
                                &mut visited,
                            ) {
                                return Some(method_sig);
                            }
                        }
                    }
                }
            }
        }

        // Get the type name from the receiver type
        let type_name = match receiver_type {
            ResolvedType::Named { name, .. } => name.clone(),
            _ => return None,
        };

        // Look through trait implementations to find methods for this type
        for trait_impl in &self.trait_impls {
            if trait_impl.type_name == type_name {
                // Found an implementation of a trait for this type
                let mut visited = std::collections::HashSet::new();
                if let Some(method_sig) = self.find_method_in_trait_with_supers(
                    &trait_impl.trait_name,
                    method_name,
                    &mut visited,
                ) {
                    return Some(method_sig);
                }
            }
        }

        None
    }

    /// Get the Item type from an Iterator trait implementation
    /// Returns the element type that the iterator yields
    pub(crate) fn get_iterator_item_type(&self, iter_type: &ResolvedType) -> Option<ResolvedType> {
        let mut visited = std::collections::HashSet::new();
        self.get_iterator_item_type_inner(iter_type, &mut visited)
    }

    /// Inner implementation with visited set to prevent infinite recursion
    /// when `into_iter()` returns the same type (or a cycle of types).
    fn get_iterator_item_type_inner(
        &self,
        iter_type: &ResolvedType,
        visited: &mut std::collections::HashSet<String>,
    ) -> Option<ResolvedType> {
        // Handle built-in iterable types
        match iter_type {
            ResolvedType::Array(elem_type) => return Some((**elem_type).clone()),
            ResolvedType::Range(elem_type) => return Some((**elem_type).clone()),
            _ => {}
        }

        // Check if the type implements Iterator trait
        let type_name = match iter_type {
            ResolvedType::Named { name, .. } => name,
            _ => return None,
        };

        // Prevent infinite recursion: if we've already visited this type, bail out
        if !visited.insert(type_name.clone()) {
            return None;
        }

        // Look for Iterator trait implementation
        for trait_impl in &self.trait_impls {
            if trait_impl.type_name == *type_name && trait_impl.trait_name == "Iterator" {
                // Found Iterator implementation, try to get item type from next() method
                if let Some(struct_def) = self.structs.get(type_name) {
                    if let Some(next_method) = struct_def.methods.get("next") {
                        return Some(next_method.ret.clone());
                    }
                }

                // Fallback: check trait definition
                if let Some(trait_def) = self.traits.get("Iterator") {
                    if let Some(next_method) = trait_def.methods.get("next") {
                        return Some(next_method.ret.clone());
                    }
                }

                // If trait has associated Item type, that would be ideal
                // but for now we use next() return type as a proxy
            }
        }

        // Check for IntoIterator trait - types that can be converted to iterators
        for trait_impl in &self.trait_impls {
            if trait_impl.type_name == *type_name && trait_impl.trait_name == "IntoIterator" {
                // IntoIterator has an associated IntoIter type and Item type
                // Try to find the into_iter() method and get its return type
                if let Some(struct_def) = self.structs.get(type_name) {
                    if let Some(into_iter_method) = struct_def.methods.get("into_iter") {
                        let iterator_type = &into_iter_method.ret;
                        // Recursively get the item type from the iterator (with cycle detection)
                        return self.get_iterator_item_type_inner(iterator_type, visited);
                    }
                }

                // Fallback to trait definition
                if let Some(trait_def) = self.traits.get("IntoIterator") {
                    // Check for associated Item type
                    if let Some(item_def) = trait_def.associated_types.get("Item") {
                        if let Some(default_type) = &item_def.default {
                            return Some(default_type.clone());
                        }
                    }
                }
            }
        }

        None
    }
}

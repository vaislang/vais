//! Variable and method lookup resolution.

use std::collections::HashMap;

use super::TypeChecker;
use crate::traits::TraitMethodSig;
use crate::types::defs::VarInfo;
use crate::types::{
    find_similar_name, Linearity, ResolvedType, TypeError, TypeResult, VariantFieldTypes,
};

impl TypeChecker {
    /// Phase 6.27c.3: push an enum name onto the disambiguation hint stack.
    /// Returns the depth at push time for paired pop sanity checking by
    /// callers that want a guard-style usage.
    pub(crate) fn push_enum_hint(&mut self, enum_name: impl Into<String>) -> usize {
        let depth = self.enum_hint_stack.len();
        self.enum_hint_stack.push(enum_name.into());
        depth
    }

    /// Pop the topmost enum hint (no-op if stack empty — defensive).
    pub(crate) fn pop_enum_hint(&mut self) {
        self.enum_hint_stack.pop();
    }

    /// Phase 6.27c.3: extract an enum name from an expected type, if any.
    /// Handles `Named { name: E, .. }` directly and strips common wrappers
    /// so hints survive one level of `Ref` / `Option` / `Result`.
    pub(crate) fn enum_name_hint_from(ty: &ResolvedType) -> Option<String> {
        match ty {
            ResolvedType::Named { name, .. } => Some(name.clone()),
            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                Self::enum_name_hint_from(inner)
            }
            _ => None,
        }
    }

    /// Phase 6.27c.3: check an expression with a scoped enum hint derived
    /// from `expected`. Pushes the hint if `expected` names an enum,
    /// runs `check_expr`, then pops. Callers use this at arg/field sites.
    pub(crate) fn check_expr_with_enum_hint(
        &mut self,
        expr: &vais_ast::Spanned<vais_ast::Expr>,
        expected: &ResolvedType,
    ) -> TypeResult<ResolvedType> {
        let hint = Self::enum_name_hint_from(expected);
        if let Some(ref h) = hint {
            self.push_enum_hint(h.clone());
        }
        let r = self.check_expr(expr);
        if hint.is_some() {
            self.pop_enum_hint();
        }
        r
    }

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
        // Phase 6.27b iteration 42: iterate enums in deterministic order
        // (HashMap iter is non-deterministic, causing flakes when two enums
        // share a Unit variant name like `None` in `Option` and
        // user-defined `QuantizationStrategy`). Sort by name and prefer
        // user-defined enums over builtins when variant name matches.
        //
        // Phase 6.27c.3: also honor `enum_hint_stack` — when an expected
        // type is known (e.g. struct-lit field type `UnaryOp`), that enum
        // wins over alphabetical order if it also has the variant.
        let mut enum_entries: Vec<_> = self.enums.iter().collect();
        enum_entries.sort_by(|(a, _), (b, _)| {
            // Hint priority: topmost (last pushed) enum that has the variant
            // goes first. We check `name` membership below; here we only
            // need a cheap rank based on hint presence.
            let a_hint = self
                .enum_hint_stack
                .iter()
                .rev()
                .position(|h| h == *a)
                .map(|p| p as i32)
                .unwrap_or(i32::MAX);
            let b_hint = self
                .enum_hint_stack
                .iter()
                .rev()
                .position(|h| h == *b)
                .map(|p| p as i32)
                .unwrap_or(i32::MAX);
            let a_builtin = matches!(a.as_str(), "Option" | "Result");
            let b_builtin = matches!(b.as_str(), "Option" | "Result");
            // Hinted first (lower rank = earlier in reverse stack = topmost),
            // then non-builtin, then alphabetical
            a_hint
                .cmp(&b_hint)
                .then_with(|| a_builtin.cmp(&b_builtin))
                .then_with(|| a.cmp(b))
        });
        for (enum_name, enum_def) in enum_entries {
            if let Some(variant_fields) = enum_def.variants.get(name) {
                // Phase 2.10: for Unit variants of built-in Option/Result,
                // use `Never` for the unconstrained generic slots. This lets
                // the Unit arm (e.g. `None`) unify trivially with any
                // concrete Option<T> from a sibling arm. Without this, None
                // injects a fresh type var that later gets bound to the
                // scrutinee's T, contaminating the arm result type when a
                // sibling arm constructs `Some(x: U)` with a different U.
                //
                // Scope narrow: only applies when the Unit variant has no
                // fields (None/Err-unit style) AND the enum is builtin
                // Option or Result. User-defined enums retain fresh vars.
                let use_never_for_unit = matches!(variant_fields, VariantFieldTypes::Unit)
                    && matches!(enum_name.as_str(), "Option" | "Result");
                let generics: Vec<ResolvedType> = enum_def
                    .generics
                    .iter()
                    .map(|_| {
                        if use_never_for_unit {
                            ResolvedType::Never
                        } else {
                            self.fresh_type_var()
                        }
                    })
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

        // Check if it's a global variable (globals are mutable by default)
        if let Some(global_type) = self.globals.get(name) {
            return Ok(VarInfo {
                ty: global_type.clone(),
                is_mut: true,
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

        // Check if name is an enum TYPE name (used as namespace: DistanceMetric.L2)
        if let Some(enum_def) = self.enums.get(name) {
            let generics: Vec<ResolvedType> = enum_def
                .generics
                .iter()
                .map(|_| self.fresh_type_var())
                .collect();
            return Ok(VarInfo {
                ty: ResolvedType::Named {
                    name: name.to_string(),
                    generics,
                },
                is_mut: false,
                linearity: Linearity::Unrestricted,
                use_count: 0,
                defined_at: None,
            });
        }

        // Check if name is a struct type name (used as namespace: ByteBuffer.new())
        if let Some(_struct_def) = self.structs.get(name) {
            return Ok(VarInfo {
                ty: ResolvedType::Named {
                    name: name.to_string(),
                    generics: vec![],
                },
                is_mut: false,
                linearity: Linearity::Unrestricted,
                use_count: 0,
                defined_at: None,
            });
        }

        // Fallback: if name looks like a constant (ALL_CAPS) not yet registered,
        // return I64 to avoid cascading errors
        if name.contains('_')
            && name
                .chars()
                .all(|c| c.is_uppercase() || c == '_' || c.is_ascii_digit())
        {
            return Ok(VarInfo {
                ty: ResolvedType::I64,
                is_mut: false,
                linearity: Linearity::Unrestricted,
                use_count: 0,
                defined_at: None,
            });
        }

        // Collect all available names for did-you-mean suggestion
        let mut candidates: Vec<&str> = Vec::new();
        for scope in &self.scopes {
            candidates.extend(scope.keys().map(|s| s.as_str()));
        }
        candidates.extend(self.functions.keys().map(|s| s.as_str()));
        candidates.extend(self.constants.keys().map(|s| s.as_str()));
        candidates.extend(self.globals.keys().map(|s| s.as_str()));
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
        // Handle dyn Trait types - look up method directly in trait definition.
        // Peel Ref/RefMut and Box<...> wrappers before checking DynTrait, so that
        // `&dyn T`, `&mut dyn T`, `Box<dyn T>`, `&mut Box<dyn T>`, etc. all dispatch.
        fn peel_to_dyn(t: &ResolvedType) -> Option<String> {
            match t {
                ResolvedType::DynTrait { trait_name, .. } => Some(trait_name.clone()),
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => peel_to_dyn(inner),
                ResolvedType::Named { name, generics } if name == "Box" && generics.len() == 1 => {
                    peel_to_dyn(&generics[0])
                }
                _ => None,
            }
        }
        let dyn_trait = peel_to_dyn(receiver_type);
        if let Some(trait_name) = dyn_trait {
            let mut visited = std::collections::HashSet::new();
            return self.find_method_in_trait_with_supers(&trait_name, method_name, &mut visited);
        }

        // Handle generic types with bounds from where clauses
        // Phase 6.27e.b: peel Ref/RefMut so `store: &mut S` where `S: NodeStore`
        // dispatches to the trait's methods. Also accept `Named{name, []}`
        // when `name` matches a currently-bound generic parameter — the
        // resolve pipeline sometimes produces Named for type params that
        // haven't been recognised as Generic yet.
        fn as_generic_name(
            t: &ResolvedType,
            bounds: &std::collections::HashMap<String, Vec<String>>,
        ) -> Option<String> {
            match t {
                ResolvedType::Generic(name) => Some(name.clone()),
                ResolvedType::Named { name, generics } if generics.is_empty() && bounds.contains_key(name) => {
                    Some(name.clone())
                }
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                    as_generic_name(inner, bounds)
                }
                _ => None,
            }
        }
        let peeled_generic = as_generic_name(receiver_type, &self.current_generic_bounds);
        if let Some(type_param) = peeled_generic {
            if let Some(bounds) = self.current_generic_bounds.get(&type_param) {
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
            ResolvedType::ConstArray { element, .. } => return Some((**element).clone()),
            ResolvedType::Range(elem_type) => return Some((**elem_type).clone()),
            ResolvedType::Slice(elem_type) | ResolvedType::SliceMut(elem_type) => {
                return Some((**elem_type).clone())
            }
            ResolvedType::Pointer(elem_type) => return Some((**elem_type).clone()),
            // &Vec<T>, &[T], &mut Vec<T> — iterate yields &element_type
            ResolvedType::Ref(inner) => {
                if let Some(elem) = self.get_iterator_item_type_inner(inner, visited) {
                    return Some(ResolvedType::Ref(Box::new(elem)));
                }
                return None;
            }
            ResolvedType::RefMut(inner) => {
                if let Some(elem) = self.get_iterator_item_type_inner(inner, visited) {
                    return Some(ResolvedType::RefMut(Box::new(elem)));
                }
                return None;
            }
            // Vec<T> — element is generics[0]
            ResolvedType::Named { name, generics } if name == "Vec" && !generics.is_empty() => {
                return Some(generics[0].clone());
            }
            // Phase 24 Task 5: EnumerateIter<T> — yields (i64, T) tuples
            // This is a virtual iterator type produced by Vec<T>.enumerate()
            // in calls.rs. For-each loop over it binds a Pattern::Tuple([i, x])
            // against ResolvedType::Tuple([I64, T]).
            ResolvedType::Named { name, generics }
                if name == "EnumerateIter" && !generics.is_empty() =>
            {
                return Some(ResolvedType::Tuple(vec![
                    ResolvedType::I64,
                    generics[0].clone(),
                ]));
            }
            // Phase 280: HashMap<K,V> named form — iteration yields (K, V) tuples.
            // vaisdb uses `HashMap<K,V>` type annotation (Named form) rather than
            // the map literal `[K:V]` form, so we need to handle both.
            ResolvedType::Named { name, generics }
                if (name == "HashMap" || name == "BTreeMap" || name == "IndexMap")
                    && generics.len() >= 2 =>
            {
                return Some(ResolvedType::Tuple(vec![
                    generics[0].clone(),
                    generics[1].clone(),
                ]));
            }
            // Phase 280: Map(K,V) — [K:V] literal map form iteration yields (K, V) tuples.
            ResolvedType::Map(key_type, val_type) => {
                return Some(ResolvedType::Tuple(vec![
                    (**key_type).clone(),
                    (**val_type).clone(),
                ]));
            }
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

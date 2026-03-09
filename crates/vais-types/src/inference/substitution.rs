//! Type substitution and generic instantiation logic.
//!
//! Contains apply_substitutions, substitute_generics (with 2-level cache),
//! fresh type variable generation, and generic function call checking.

use crate::types::{
    FunctionSig, GenericCallee, GenericInstantiation, ResolvedType, TypeError, TypeResult,
};
use crate::TypeChecker;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use vais_ast::{Expr, Spanned};

impl TypeChecker {
    pub(crate) fn apply_substitutions(&self, ty: &ResolvedType) -> ResolvedType {
        // Fast path: no substitutions means no transformation needed
        if self.substitutions.is_empty() {
            return ty.clone();
        }
        // Fast path: primitive types never contain Var — skip contains_var traversal
        if Self::is_primitive_type(ty) {
            return ty.clone();
        }
        // Fast path: types without Var nodes cannot be affected by substitutions
        if !Self::contains_var(ty) {
            return ty.clone();
        }
        match ty {
            ResolvedType::Var(id) => {
                if let Some(subst) = self.substitutions.get(id) {
                    self.apply_substitutions(subst)
                } else {
                    ty.clone()
                }
            }
            ResolvedType::Array(inner) => {
                ResolvedType::Array(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Optional(inner) => {
                ResolvedType::Optional(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Result(ok, err) => ResolvedType::Result(
                Box::new(self.apply_substitutions(ok)),
                Box::new(self.apply_substitutions(err)),
            ),
            ResolvedType::Ref(inner) => {
                ResolvedType::Ref(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Slice(inner) => {
                ResolvedType::Slice(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::SliceMut(inner) => {
                ResolvedType::SliceMut(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Range(inner) => {
                ResolvedType::Range(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Tuple(types) => {
                let mut result = Vec::with_capacity(types.len());
                for t in types {
                    result.push(self.apply_substitutions(t));
                }
                ResolvedType::Tuple(result)
            }
            ResolvedType::Fn {
                params,
                ret,
                effects,
            } => ResolvedType::Fn {
                params: {
                    let mut result = Vec::with_capacity(params.len());
                    for p in params {
                        result.push(self.apply_substitutions(p));
                    }
                    result
                },
                ret: Box::new(self.apply_substitutions(ret)),
                effects: effects.clone(),
            },
            ResolvedType::ConstArray { element, size } => ResolvedType::ConstArray {
                element: Box::new(self.apply_substitutions(element)),
                size: size.clone(),
            },
            ResolvedType::Vector { element, lanes } => ResolvedType::Vector {
                element: Box::new(self.apply_substitutions(element)),
                lanes: *lanes,
            },
            ResolvedType::Map(key, value) => ResolvedType::Map(
                Box::new(self.apply_substitutions(key)),
                Box::new(self.apply_substitutions(value)),
            ),
            ResolvedType::Future(inner) => {
                ResolvedType::Future(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Lazy(inner) => {
                ResolvedType::Lazy(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Named { name, generics } => ResolvedType::Named {
                name: name.clone(),
                generics: generics
                    .iter()
                    .map(|g| self.apply_substitutions(g))
                    .collect(),
            },
            ResolvedType::FnPtr {
                params,
                ret,
                is_vararg,
                effects,
            } => ResolvedType::FnPtr {
                params: params.iter().map(|p| self.apply_substitutions(p)).collect(),
                ret: Box::new(self.apply_substitutions(ret)),
                is_vararg: *is_vararg,
                effects: effects.clone(),
            },
            ResolvedType::Associated {
                base,
                trait_name,
                assoc_name,
                generics,
            } => ResolvedType::Associated {
                base: Box::new(self.apply_substitutions(base)),
                trait_name: trait_name.clone(),
                assoc_name: assoc_name.clone(),
                generics: generics
                    .iter()
                    .map(|g| self.apply_substitutions(g))
                    .collect(),
            },
            ResolvedType::Linear(inner) => {
                ResolvedType::Linear(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Affine(inner) => {
                ResolvedType::Affine(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::RefLifetime { lifetime, inner } => ResolvedType::RefLifetime {
                lifetime: lifetime.clone(),
                inner: Box::new(self.apply_substitutions(inner)),
            },
            ResolvedType::RefMutLifetime { lifetime, inner } => ResolvedType::RefMutLifetime {
                lifetime: lifetime.clone(),
                inner: Box::new(self.apply_substitutions(inner)),
            },
            ResolvedType::DynTrait {
                trait_name,
                generics,
            } => ResolvedType::DynTrait {
                trait_name: trait_name.clone(),
                generics: generics
                    .iter()
                    .map(|g| self.apply_substitutions(g))
                    .collect(),
            },
            ResolvedType::Dependent {
                var_name,
                base,
                predicate,
            } => ResolvedType::Dependent {
                var_name: var_name.clone(),
                base: Box::new(self.apply_substitutions(base)),
                predicate: predicate.clone(),
            },
            _ => ty.clone(),
        }
    }

    /// Create a fresh type variable
    #[inline]
    pub(crate) fn fresh_type_var(&self) -> ResolvedType {
        let id = self.next_type_var.get();
        self.next_type_var.set(id + 1);
        ResolvedType::Var(id)
    }

    /// Compute hash for a type (for memoization).
    /// Uses the type's derived Hash impl directly — no intermediate String allocation.
    #[inline]
    fn hash_type(ty: &ResolvedType) -> u64 {
        let mut hasher = DefaultHasher::new();
        ty.hash(&mut hasher);
        hasher.finish()
    }

    /// Compute hash for substitution map (for memoization).
    /// Uses the types' derived Hash impl directly — no intermediate String allocations.
    fn hash_substitutions(substitutions: &HashMap<String, ResolvedType>) -> u64 {
        let mut hasher = DefaultHasher::new();
        let mut keys: Vec<_> = substitutions.keys().collect();
        keys.sort();
        for key in keys {
            key.hash(&mut hasher);
            if let Some(value) = substitutions.get(key) {
                value.hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    /// Substitute generic type parameters with concrete types (with 2-level memoization)
    ///
    /// Uses a 2-level cache for optimal performance:
    /// - L1: 16-entry direct-mapped cache with O(1) lookup (no HashMap overhead)
    /// - L2: Bounded 256-entry HashMap for overflow
    ///
    /// Primitive types that contain no generic parameters are returned immediately
    /// without any cache interaction (fast path).
    pub(crate) fn substitute_generics(
        &self,
        ty: &ResolvedType,
        substitutions: &HashMap<String, ResolvedType>,
    ) -> ResolvedType {
        // Fast path: primitives never need substitution
        if Self::is_primitive_type(ty) {
            return ty.clone();
        }

        // Fast path: empty substitutions map means no work
        if substitutions.is_empty() {
            return ty.clone();
        }

        // Compute cache key (hashes)
        let type_hash = Self::hash_type(ty);
        let subst_hash = Self::hash_substitutions(substitutions);
        let cache_key = (type_hash, subst_hash);

        // L1 cache: direct-mapped lookup (16 entries, indexed by lower 4 bits of type_hash)
        let l1_index = (type_hash as usize) & 0xF;
        {
            let l1 = self.substitute_cache_l1.borrow();
            if l1_index < l1.len() {
                let (ref key, ref val) = l1[l1_index];
                if *key == cache_key {
                    return val.clone();
                }
            }
        }

        // L2 cache: HashMap lookup
        if let Some(cached) = self.substitute_cache_l2.borrow().get(&cache_key) {
            // Promote to L1 on L2 hit
            let result = cached.clone();
            let mut l1 = self.substitute_cache_l1.borrow_mut();
            if l1_index < l1.len() {
                l1[l1_index] = (cache_key, result.clone());
            } else {
                // Grow L1 to accommodate this index
                while l1.len() <= l1_index {
                    l1.push(((0, 0), ResolvedType::Unit));
                }
                l1[l1_index] = (cache_key, result.clone());
            }
            return result;
        }

        // Compute the substitution
        let result = match ty {
            ResolvedType::Generic(name) => substitutions
                .get(name)
                .cloned()
                .unwrap_or_else(|| ty.clone()),
            ResolvedType::Array(inner) => {
                ResolvedType::Array(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Map(key, value) => ResolvedType::Map(
                Box::new(self.substitute_generics(key, substitutions)),
                Box::new(self.substitute_generics(value, substitutions)),
            ),
            ResolvedType::Optional(inner) => {
                ResolvedType::Optional(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Result(ok, err) => ResolvedType::Result(
                Box::new(self.substitute_generics(ok, substitutions)),
                Box::new(self.substitute_generics(err, substitutions)),
            ),
            ResolvedType::Ref(inner) => {
                ResolvedType::Ref(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Slice(inner) => {
                ResolvedType::Slice(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::SliceMut(inner) => {
                ResolvedType::SliceMut(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Range(inner) => {
                ResolvedType::Range(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Tuple(types) => ResolvedType::Tuple(
                types
                    .iter()
                    .map(|t| self.substitute_generics(t, substitutions))
                    .collect(),
            ),
            ResolvedType::Fn {
                params,
                ret,
                effects,
            } => ResolvedType::Fn {
                params: params
                    .iter()
                    .map(|p| self.substitute_generics(p, substitutions))
                    .collect(),
                ret: Box::new(self.substitute_generics(ret, substitutions)),
                effects: effects.clone(),
            },
            ResolvedType::Named { name, generics } => {
                // Check if the name is an HKT parameter being substituted.
                // NOTE: This HKT application logic is mirrored in types/substitute.rs::substitute_type().
                // Any changes here must be synchronized with that function.
                if let Some(subst) = substitutions.get(name) {
                    if !generics.is_empty() {
                        // HKT application: F<A> where F maps to Vec → Vec<A>
                        if let ResolvedType::Named {
                            name: concrete_name,
                            ..
                        }
                        | ResolvedType::HigherKinded {
                            name: concrete_name,
                            ..
                        } = subst
                        {
                            ResolvedType::Named {
                                name: concrete_name.clone(),
                                generics: generics
                                    .iter()
                                    .map(|g| self.substitute_generics(g, substitutions))
                                    .collect(),
                            }
                        } else {
                            // Fallback: substitute generics normally
                            ResolvedType::Named {
                                name: name.clone(),
                                generics: generics
                                    .iter()
                                    .map(|g| self.substitute_generics(g, substitutions))
                                    .collect(),
                            }
                        }
                    } else {
                        subst.clone()
                    }
                } else {
                    ResolvedType::Named {
                        name: name.clone(),
                        generics: generics
                            .iter()
                            .map(|g| self.substitute_generics(g, substitutions))
                            .collect(),
                    }
                }
            }
            ResolvedType::HigherKinded { name, .. } => substitutions
                .get(name)
                .cloned()
                .unwrap_or_else(|| ty.clone()),
            _ => ty.clone(),
        };

        // Store in L1 cache (direct-mapped, overwrites existing entry at this index)
        {
            let mut l1 = self.substitute_cache_l1.borrow_mut();
            if l1_index < l1.len() {
                l1[l1_index] = (cache_key, result.clone());
            } else {
                while l1.len() <= l1_index {
                    l1.push(((0, 0), ResolvedType::Unit));
                }
                l1[l1_index] = (cache_key, result.clone());
            }
        }

        // Store in L2 cache (bounded — evict oldest half when exceeding 256 entries)
        {
            let mut l2 = self.substitute_cache_l2.borrow_mut();
            if l2.len() >= 256 {
                // Evict: retain only the most recently inserted half.
                // Since HashMap doesn't track insertion order, we simply clear.
                // The L1 cache retains the hottest 16 entries across the clear.
                l2.clear();
            }
            l2.insert(cache_key, result.clone());
        }

        result
    }

    /// Check if a type is a primitive that never contains generic parameters.
    /// These types can skip cache lookup entirely in substitute_generics.
    #[inline]
    pub(crate) fn is_primitive_type(ty: &ResolvedType) -> bool {
        matches!(
            ty,
            ResolvedType::I8
                | ResolvedType::I16
                | ResolvedType::I32
                | ResolvedType::I64
                | ResolvedType::I128
                | ResolvedType::U8
                | ResolvedType::U16
                | ResolvedType::U32
                | ResolvedType::U64
                | ResolvedType::U128
                | ResolvedType::F32
                | ResolvedType::F64
                | ResolvedType::Bool
                | ResolvedType::Str
                | ResolvedType::Unit
                | ResolvedType::Never
                | ResolvedType::Unknown
        )
    }

    /// Check a generic function call, inferring type arguments from actual arguments
    pub(crate) fn check_generic_function_call(
        &mut self,
        sig: &FunctionSig,
        args: &[Spanned<Expr>],
    ) -> TypeResult<ResolvedType> {
        // Check argument count
        if sig.params.len() != args.len() {
            return Err(TypeError::ArgCount {
                expected: sig.params.len(),
                got: args.len(),
                span: args.first().map(|a| a.span),
            });
        }

        // Create fresh type variables for each generic parameter
        let generic_substitutions: HashMap<String, ResolvedType> = sig
            .generics
            .iter()
            .map(|param| (param.clone(), self.fresh_type_var()))
            .collect();

        // Check each argument and unify with parameter type
        for ((_, param_type, _), arg) in sig.params.iter().zip(args) {
            let arg_type = self.check_expr(arg)?;
            // Substitute generic parameters with type variables in the parameter type
            let expected_type = self.substitute_generics(param_type, &generic_substitutions);
            self.unify(&expected_type, &arg_type)?;
        }

        // Apply substitutions to infer concrete generic types
        let inferred_type_args: Vec<_> = sig
            .generics
            .iter()
            .map(|param| {
                let ty = generic_substitutions.get(param).expect(
                    "Internal compiler error: generic parameter should exist in substitutions map",
                );
                self.apply_substitutions(ty)
            })
            .collect();

        // Record the generic instantiation if all type arguments are concrete
        let all_concrete = inferred_type_args
            .iter()
            .all(|t| !matches!(t, ResolvedType::Var(_)));

        // Record callee relationship for transitive instantiation propagation.
        // When we're inside a generic function (current_generics is non-empty) and calling
        // another generic function, record the callee with its (possibly-generic) type args
        // so that when the caller is concretely instantiated, we can derive the callee's
        // concrete instantiation.
        if !self.current_generics.is_empty() {
            if let Some(ref caller_name) = self.current_fn_name.clone() {
                // The inferred_type_args may contain Generic("T") when calling from a
                // generic context. These will be substituted later during propagation.
                let callee = GenericCallee {
                    callee_name: sig.name.clone(),
                    type_args: inferred_type_args.clone(),
                };
                if let Some(caller_sig) = self.functions.get_mut(caller_name) {
                    // Avoid duplicates
                    let already_recorded = caller_sig.generic_callees.iter().any(|c| {
                        c.callee_name == callee.callee_name && c.type_args == callee.type_args
                    });
                    if !already_recorded {
                        caller_sig.generic_callees.push(callee);
                    }
                }
            }
        }

        if all_concrete {
            let inst = GenericInstantiation::function(&sig.name, inferred_type_args.clone());
            self.add_instantiation(inst);

            // Transitive instantiation: when a concrete instantiation is added,
            // propagate to its generic callees by substituting the concrete type args.
            self.propagate_transitive_instantiations(
                &sig.name,
                &sig.generics,
                &inferred_type_args,
                &mut HashSet::new(),
            );
        }

        // Verify trait bounds: each inferred concrete type must implement required traits
        if all_concrete && !sig.generic_bounds.is_empty() {
            let call_span = args.first().map(|a| a.span);
            self.verify_trait_bounds(
                &sig.generics,
                &inferred_type_args,
                &sig.generic_bounds,
                call_span,
            )?;
        }

        // Verify HKT arity: when an HKT param is substituted with a concrete type,
        // check that the concrete type constructor has the expected arity.
        // Uses O(G + H) index lookup instead of O(H × G) position scan.
        if all_concrete && !sig.hkt_params.is_empty() {
            let generic_index: HashMap<&str, usize> = sig
                .generics
                .iter()
                .enumerate()
                .map(|(i, name)| (name.as_str(), i))
                .collect();

            for (param_name, &expected_arity) in &sig.hkt_params {
                if let Some(&idx) = generic_index.get(param_name.as_str()) {
                    if let Some(concrete_ty) = inferred_type_args.get(idx) {
                        let actual_arity = match concrete_ty {
                            ResolvedType::Named { generics, .. } => generics.len(),
                            ResolvedType::HigherKinded { arity, .. } => *arity,
                            _ => 0, // Non-generic types have arity 0
                        };
                        if actual_arity != expected_arity
                            && !matches!(concrete_ty, ResolvedType::Generic(_))
                        {
                            return Err(TypeError::Mismatch {
                                expected: format!(
                                    "type constructor with arity {} for HKT parameter '{}'",
                                    expected_arity, param_name
                                ),
                                found: format!(
                                    "type '{}' with arity {}",
                                    concrete_ty, actual_arity
                                ),
                                span: None,
                            });
                        }
                    }
                }
            }
        }

        // Substitute generics in the return type
        let return_type = self.substitute_generics(&sig.ret, &generic_substitutions);
        let resolved_return = self.apply_substitutions(&return_type);

        // For async functions, wrap the return type in Future
        if sig.is_async {
            Ok(ResolvedType::Future(Box::new(resolved_return)))
        } else {
            Ok(resolved_return)
        }
    }

    /// Propagate transitive instantiations: when `foo<i64>` is instantiated and
    /// `foo<T>` calls `bar<T>`, derive and add `bar<i64>`.
    ///
    /// `caller_name`: the function being instantiated (e.g., "foo")
    /// `caller_generics`: the generic param names of the caller (e.g., ["T"])
    /// `caller_type_args`: the concrete type args for this instantiation (e.g., [i64])
    /// `visiting`: cycle guard set to prevent infinite recursion
    fn propagate_transitive_instantiations(
        &mut self,
        caller_name: &str,
        caller_generics: &[String],
        caller_type_args: &[ResolvedType],
        visiting: &mut HashSet<String>,
    ) {
        // Cycle guard: if we're already processing this caller, skip
        if !visiting.insert(caller_name.to_string()) {
            return;
        }

        // Build substitution map: caller's generic param name -> concrete type
        let substitutions: HashMap<String, ResolvedType> = caller_generics
            .iter()
            .zip(caller_type_args.iter())
            .map(|(name, ty)| (name.clone(), ty.clone()))
            .collect();

        // Look up the caller's generic callees (clone to avoid borrow conflict)
        let callees: Vec<GenericCallee> = self
            .functions
            .get(caller_name)
            .map(|sig| sig.generic_callees.clone())
            .unwrap_or_default();

        for callee in &callees {
            // Substitute the caller's generic params with concrete types in the callee's type args
            let concrete_callee_args: Vec<ResolvedType> = callee
                .type_args
                .iter()
                .map(|ty| crate::types::substitute::substitute_type(ty, &substitutions))
                .collect();

            // Only create instantiation if all type args are now concrete
            let all_concrete = concrete_callee_args
                .iter()
                .all(|t| !matches!(t, ResolvedType::Generic(_) | ResolvedType::Var(_)));

            if all_concrete {
                let inst = GenericInstantiation::function(
                    &callee.callee_name,
                    concrete_callee_args.clone(),
                );
                // add_instantiation deduplicates via HashSet
                self.add_instantiation(inst);

                // Recursively propagate to the callee's callees
                let callee_generics: Vec<String> = self
                    .functions
                    .get(&callee.callee_name)
                    .map(|sig| sig.generics.clone())
                    .unwrap_or_default();

                if !callee_generics.is_empty() {
                    self.propagate_transitive_instantiations(
                        &callee.callee_name,
                        &callee_generics,
                        &concrete_callee_args,
                        visiting,
                    );
                }
            }
        }

        visiting.remove(caller_name);
    }
}

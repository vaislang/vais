//! Type inference logic for the Vais type system
//!
//! This module contains the type inference algorithms including unification,
//! substitution, fresh type variable generation, and bidirectional type checking.
//!
//! ## Bidirectional Type Checking
//!
//! The type checker supports two modes:
//! - `Infer`: Bottom-up inference where the type is computed from the expression
//! - `Check`: Top-down checking where the expression is verified against an expected type
//!
//! This allows for better type inference in cases like:
//! - Lambda parameters: `|x| x + 1` can infer `x: i64` from context
//! - Generic instantiation: Type arguments can be inferred from expected return type
//! - Better error messages with more precise location information

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use vais_ast::{Expr, Spanned};
use crate::types::{ResolvedType, TypeError, TypeResult, FunctionSig, GenericInstantiation};
use crate::TypeChecker;

/// Mode for bidirectional type checking
#[derive(Debug, Clone)]
pub enum CheckMode {
    /// Infer the type of the expression (bottom-up)
    Infer,
    /// Check the expression against an expected type (top-down)
    Check(ResolvedType),
}

impl CheckMode {
    /// Create a Check mode with the given expected type
    pub fn check(expected: ResolvedType) -> Self {
        CheckMode::Check(expected)
    }

    /// Check if this is Infer mode
    pub fn is_infer(&self) -> bool {
        matches!(self, CheckMode::Infer)
    }

    /// Get the expected type if in Check mode
    pub fn expected(&self) -> Option<&ResolvedType> {
        match self {
            CheckMode::Infer => None,
            CheckMode::Check(ty) => Some(ty),
        }
    }
}

impl TypeChecker {
    /// Unify two types
    pub(crate) fn unify(&mut self, expected: &ResolvedType, found: &ResolvedType) -> TypeResult<()> {
        let expected = self.apply_substitutions(expected);
        let found = self.apply_substitutions(found);

        if expected == found {
            return Ok(());
        }

        match (&expected, &found) {
            // Type variables can unify with anything
            (ResolvedType::Var(id), t) | (t, ResolvedType::Var(id)) => {
                self.substitutions.insert(*id, t.clone());
                Ok(())
            }
            // Unknown type unifies with anything (used as placeholder)
            (ResolvedType::Unknown, _) | (_, ResolvedType::Unknown) => Ok(()),
            // Never type unifies with any type (represents non-returning expressions like return, break)
            (ResolvedType::Never, _) | (_, ResolvedType::Never) => Ok(()),
            // Generic type parameters match with any type (type erasure)
            (ResolvedType::Generic(_), _) | (_, ResolvedType::Generic(_)) => Ok(()),
            (ResolvedType::Array(a), ResolvedType::Array(b)) => self.unify(a, b),
            (ResolvedType::Optional(a), ResolvedType::Optional(b)) => self.unify(a, b),
            (ResolvedType::Result(a), ResolvedType::Result(b)) => self.unify(a, b),
            (ResolvedType::Ref(a), ResolvedType::Ref(b)) => self.unify(a, b),
            (ResolvedType::RefMut(a), ResolvedType::RefMut(b)) => self.unify(a, b),
            (ResolvedType::Pointer(a), ResolvedType::Pointer(b)) => self.unify(a, b),
            (ResolvedType::Range(a), ResolvedType::Range(b)) => self.unify(a, b),
            (ResolvedType::Future(a), ResolvedType::Future(b)) => self.unify(a, b),
            (ResolvedType::Tuple(a), ResolvedType::Tuple(b)) if a.len() == b.len() => {
                for (ta, tb) in a.iter().zip(b.iter()) {
                    self.unify(ta, tb)?;
                }
                Ok(())
            }
            (
                ResolvedType::Fn {
                    params: pa,
                    ret: ra,
                    ..
                },
                ResolvedType::Fn {
                    params: pb,
                    ret: rb,
                    ..
                },
            ) if pa.len() == pb.len() => {
                for (ta, tb) in pa.iter().zip(pb.iter()) {
                    self.unify(ta, tb)?;
                }
                self.unify(ra, rb)
            }
            // Named types with generics
            (
                ResolvedType::Named { name: na, generics: ga },
                ResolvedType::Named { name: nb, generics: gb },
            ) if na == nb && ga.len() == gb.len() => {
                for (ta, tb) in ga.iter().zip(gb.iter()) {
                    self.unify(ta, tb)?;
                }
                Ok(())
            }
            // Allow implicit integer type conversions (widening and narrowing)
            (a, b) if Self::is_integer_type(a) && Self::is_integer_type(b) => Ok(()),
            // Linear type: unwrap and unify with inner type
            (ResolvedType::Linear(inner), other) | (other, ResolvedType::Linear(inner)) => self.unify(inner, other),
            // Affine type: unwrap and unify with inner type
            (ResolvedType::Affine(inner), other) | (other, ResolvedType::Affine(inner)) => self.unify(inner, other),
            // Dependent type: unify the base type only (predicate is checked separately)
            (ResolvedType::Dependent { base, .. }, other) | (other, ResolvedType::Dependent { base, .. }) => self.unify(base, other),
            // Lifetime references: unify inner types (lifetime is tracked separately)
            (ResolvedType::RefLifetime { inner: a, .. }, ResolvedType::RefLifetime { inner: b, .. }) => self.unify(a, b),
            (ResolvedType::RefMutLifetime { inner: a, .. }, ResolvedType::RefMutLifetime { inner: b, .. }) => self.unify(a, b),
            // Allow ref with lifetime to unify with plain ref
            (ResolvedType::RefLifetime { inner, .. }, ResolvedType::Ref(other)) |
            (ResolvedType::Ref(other), ResolvedType::RefLifetime { inner, .. }) => self.unify(inner, other),
            (ResolvedType::RefMutLifetime { inner, .. }, ResolvedType::RefMut(other)) |
            (ResolvedType::RefMut(other), ResolvedType::RefMutLifetime { inner, .. }) => self.unify(inner, other),
            // Lazy type unification
            (ResolvedType::Lazy(a), ResolvedType::Lazy(b)) => self.unify(a, b),
            _ => Err(TypeError::Mismatch {
                expected: expected.to_string(),
                found: found.to_string(),
                span: None,
            }),
        }
    }

    /// Check if type is an integer type
    pub(crate) fn is_integer_type(ty: &ResolvedType) -> bool {
        matches!(
            ty,
            ResolvedType::I8
                | ResolvedType::I16
                | ResolvedType::I32
                | ResolvedType::I64
                | ResolvedType::U8
                | ResolvedType::U16
                | ResolvedType::U32
                | ResolvedType::U64
        )
    }

    /// Apply substitutions to a type
    pub(crate) fn apply_substitutions(&self, ty: &ResolvedType) -> ResolvedType {
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
            ResolvedType::Result(inner) => {
                ResolvedType::Result(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Ref(inner) => {
                ResolvedType::Ref(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Range(inner) => {
                ResolvedType::Range(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Tuple(types) => {
                ResolvedType::Tuple(types.iter().map(|t| self.apply_substitutions(t)).collect())
            }
            ResolvedType::Fn { params, ret, effects } => ResolvedType::Fn {
                params: params.iter().map(|p| self.apply_substitutions(p)).collect(),
                ret: Box::new(self.apply_substitutions(ret)),
                effects: effects.clone(),
            },
            _ => ty.clone(),
        }
    }

    /// Create a fresh type variable
    pub(crate) fn fresh_type_var(&self) -> ResolvedType {
        let id = self.next_type_var.get();
        self.next_type_var.set(id + 1);
        ResolvedType::Var(id)
    }

    /// Compute hash for a type (for memoization)
    fn hash_type(ty: &ResolvedType) -> u64 {
        let mut hasher = DefaultHasher::new();
        format!("{:?}", ty).hash(&mut hasher);
        hasher.finish()
    }

    /// Compute hash for substitution map (for memoization)
    fn hash_substitutions(substitutions: &HashMap<String, ResolvedType>) -> u64 {
        let mut hasher = DefaultHasher::new();
        let mut keys: Vec<_> = substitutions.keys().collect();
        keys.sort();
        for key in keys {
            key.hash(&mut hasher);
            if let Some(value) = substitutions.get(key) {
                format!("{:?}", value).hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    /// Substitute generic type parameters with concrete types (with memoization)
    pub(crate) fn substitute_generics(&self, ty: &ResolvedType, substitutions: &HashMap<String, ResolvedType>) -> ResolvedType {
        // Check cache first
        let type_hash = Self::hash_type(ty);
        let subst_hash = Self::hash_substitutions(substitutions);
        let cache_key = (type_hash, subst_hash);

        if let Some(cached) = self.substitute_cache.borrow().get(&cache_key) {
            return cached.clone();
        }

        // Compute the substitution
        let result = match ty {
            ResolvedType::Generic(name) => {
                substitutions.get(name).cloned().unwrap_or_else(|| ty.clone())
            }
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
            ResolvedType::Result(inner) => {
                ResolvedType::Result(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Ref(inner) => {
                ResolvedType::Ref(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Range(inner) => {
                ResolvedType::Range(Box::new(self.substitute_generics(inner, substitutions)))
            }
            ResolvedType::Tuple(types) => {
                ResolvedType::Tuple(types.iter().map(|t| self.substitute_generics(t, substitutions)).collect())
            }
            ResolvedType::Fn { params, ret, effects } => ResolvedType::Fn {
                params: params.iter().map(|p| self.substitute_generics(p, substitutions)).collect(),
                ret: Box::new(self.substitute_generics(ret, substitutions)),
                effects: effects.clone(),
            },
            ResolvedType::Named { name, generics } => ResolvedType::Named {
                name: name.clone(),
                generics: generics.iter().map(|g| self.substitute_generics(g, substitutions)).collect(),
            },
            _ => ty.clone(),
        };

        // Store in cache
        self.substitute_cache.borrow_mut().insert(cache_key, result.clone());
        result
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
                span: None,
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
                let ty = generic_substitutions.get(param)
                    .expect("Internal compiler error: generic parameter should exist in substitutions map");
                self.apply_substitutions(ty)
            })
            .collect();

        // Record the generic instantiation if all type arguments are concrete
        let all_concrete = inferred_type_args.iter().all(|t| !matches!(t, ResolvedType::Var(_)));
        if all_concrete {
            let inst = GenericInstantiation::function(&sig.name, inferred_type_args.clone());
            self.add_instantiation(inst);
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

    /// Infer generic type arguments from a parameter type and an argument type.
    /// This is used to match a generic parameter with an actual argument.
    #[allow(dead_code)]
    pub(crate) fn infer_type_arg(
        &mut self,
        param_type: &ResolvedType,
        arg_type: &ResolvedType,
        type_args: &mut HashMap<String, ResolvedType>,
    ) -> TypeResult<()> {
        match (param_type, arg_type) {
            (ResolvedType::Generic(name), concrete) => {
                if let Some(existing) = type_args.get(name) {
                    // Check that the inferred type is consistent
                    if existing != concrete {
                        return Err(TypeError::Mismatch {
                            expected: existing.to_string(),
                            found: concrete.to_string(),
                            span: None,
                        });
                    }
                } else {
                    type_args.insert(name.clone(), concrete.clone());
                }
                Ok(())
            }
            (ResolvedType::Array(inner_param), ResolvedType::Array(inner_arg)) => {
                self.infer_type_arg(inner_param, inner_arg, type_args)
            }
            (ResolvedType::Pointer(inner_param), ResolvedType::Pointer(inner_arg)) => {
                self.infer_type_arg(inner_param, inner_arg, type_args)
            }
            (ResolvedType::Ref(inner_param), ResolvedType::Ref(inner_arg)) => {
                self.infer_type_arg(inner_param, inner_arg, type_args)
            }
            (ResolvedType::RefMut(inner_param), ResolvedType::RefMut(inner_arg)) => {
                self.infer_type_arg(inner_param, inner_arg, type_args)
            }
            (ResolvedType::Optional(inner_param), ResolvedType::Optional(inner_arg)) => {
                self.infer_type_arg(inner_param, inner_arg, type_args)
            }
            (ResolvedType::Result(inner_param), ResolvedType::Result(inner_arg)) => {
                self.infer_type_arg(inner_param, inner_arg, type_args)
            }
            (ResolvedType::Future(inner_param), ResolvedType::Future(inner_arg)) => {
                self.infer_type_arg(inner_param, inner_arg, type_args)
            }
            (
                ResolvedType::Named { name: name_p, generics: generics_p },
                ResolvedType::Named { name: name_a, generics: generics_a },
            ) if name_p == name_a && generics_p.len() == generics_a.len() => {
                for (gp, ga) in generics_p.iter().zip(generics_a.iter()) {
                    self.infer_type_arg(gp, ga, type_args)?;
                }
                Ok(())
            }
            (ResolvedType::Tuple(types_p), ResolvedType::Tuple(types_a))
                if types_p.len() == types_a.len() =>
            {
                for (tp, ta) in types_p.iter().zip(types_a.iter()) {
                    self.infer_type_arg(tp, ta, type_args)?;
                }
                Ok(())
            }
            (
                ResolvedType::Fn { params: params_p, ret: ret_p, .. },
                ResolvedType::Fn { params: params_a, ret: ret_a, .. },
            ) if params_p.len() == params_a.len() => {
                for (pp, pa) in params_p.iter().zip(params_a.iter()) {
                    self.infer_type_arg(pp, pa, type_args)?;
                }
                self.infer_type_arg(ret_p, ret_a, type_args)
            }
            // Non-generic types don't contribute to type argument inference
            _ => Ok(()),
        }
    }

    // ===== Bidirectional Type Checking Methods =====

    /// Check an expression with bidirectional type checking.
    /// This is the main entry point for bidirectional type checking.
    ///
    /// In `Infer` mode, the type is computed bottom-up from the expression.
    /// In `Check` mode, the expression is verified against the expected type,
    /// and the expected type information can propagate down to sub-expressions.
    pub fn check_expr_bidirectional(
        &mut self,
        expr: &Spanned<Expr>,
        mode: CheckMode,
    ) -> TypeResult<ResolvedType> {
        match &mode {
            CheckMode::Infer => self.check_expr(expr),
            CheckMode::Check(expected) => {
                // For most expressions, we infer then check
                // But some expressions can benefit from the expected type
                match &expr.node {
                    // Lambda expressions can use expected type to infer parameter types
                    Expr::Lambda { params, body, captures: _ } => {
                        self.check_lambda_with_expected(params, body, expected, &expr.span)
                    }
                    // Array literals can propagate element type
                    Expr::Array(elements) => {
                        self.check_array_with_expected(elements, expected, &expr.span)
                    }
                    // For other expressions, infer then unify
                    _ => {
                        let inferred = self.check_expr(expr)?;
                        self.unify(expected, &inferred).map_err(|e| {
                            // Enhance error with span information
                            match e {
                                TypeError::Mismatch { expected: exp, found, span: _ } => {
                                    TypeError::Mismatch {
                                        expected: exp,
                                        found,
                                        span: Some(expr.span.clone()),
                                    }
                                }
                                _ => e,
                            }
                        })?;
                        Ok(inferred)
                    }
                }
            }
        }
    }

    /// Check a lambda expression with an expected function type.
    /// This allows inferring parameter types from the expected type.
    fn check_lambda_with_expected(
        &mut self,
        params: &[vais_ast::Param],
        body: &Spanned<Expr>,
        expected: &ResolvedType,
        span: &vais_ast::Span,
    ) -> TypeResult<ResolvedType> {
        // Extract expected parameter and return types
        let (expected_params, expected_ret) = match expected {
            ResolvedType::Fn { params, ret, .. } => (Some(params.clone()), Some(ret.as_ref().clone())),
            _ => (None, None),
        };

        // Check parameter count matches if we have expected params
        if let Some(ref exp_params) = expected_params {
            if exp_params.len() != params.len() {
                return Err(TypeError::ArgCount {
                    expected: exp_params.len(),
                    got: params.len(),
                    span: Some(span.clone()),
                });
            }
        }

        // Push a new scope for lambda parameters
        self.push_scope();

        // Determine parameter types: use expected if available, otherwise infer
        let param_types: Vec<ResolvedType> = params
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let ty = if let Some(ref exp_params) = expected_params {
                    // Use expected parameter type
                    exp_params[i].clone()
                } else {
                    // Use declared type or fresh type variable
                    self.resolve_type(&p.ty.node)
                };
                self.define_var(&p.name.node, ty.clone(), p.is_mut);
                ty
            })
            .collect();

        // Check body with expected return type if available
        let body_type = if let Some(ref exp_ret) = expected_ret {
            self.check_expr_bidirectional(body, CheckMode::Check(exp_ret.clone()))?
        } else {
            self.check_expr(body)?
        };

        // Pop the lambda scope
        self.pop_scope();

        // Apply substitutions to finalize types
        let final_params: Vec<ResolvedType> = param_types
            .into_iter()
            .map(|t| self.apply_substitutions(&t))
            .collect();
        let final_ret = self.apply_substitutions(&body_type);

        Ok(ResolvedType::Fn {
            params: final_params,
            ret: Box::new(final_ret),
            effects: None,  // Effects are inferred separately
        })
    }

    /// Check an array literal with an expected array type.
    /// This propagates the element type to each element.
    fn check_array_with_expected(
        &mut self,
        elements: &[Spanned<Expr>],
        expected: &ResolvedType,
        _span: &vais_ast::Span,
    ) -> TypeResult<ResolvedType> {
        let expected_elem = match expected {
            ResolvedType::Array(inner) => Some(inner.as_ref().clone()),
            _ => None,
        };

        if elements.is_empty() {
            // Empty array: use expected element type or fresh variable
            let elem_type = expected_elem.unwrap_or_else(|| self.fresh_type_var());
            return Ok(ResolvedType::Array(Box::new(elem_type)));
        }

        // Check each element with expected type if available
        let mut elem_types = Vec::new();
        for elem in elements {
            let ty = if let Some(ref exp_elem) = expected_elem {
                self.check_expr_bidirectional(elem, CheckMode::Check(exp_elem.clone()))?
            } else {
                self.check_expr(elem)?
            };
            elem_types.push(ty);
        }

        // Unify all element types
        let first_type = elem_types[0].clone();
        for (i, ty) in elem_types.iter().enumerate().skip(1) {
            self.unify(&first_type, ty).map_err(|_| {
                TypeError::Mismatch {
                    expected: first_type.to_string(),
                    found: ty.to_string(),
                    span: Some(elements[i].span.clone()),
                }
            })?;
        }

        Ok(ResolvedType::Array(Box::new(self.apply_substitutions(&first_type))))
    }

    /// Check a generic function call with bidirectional type checking.
    /// Uses both argument types and expected return type to infer type arguments.
    pub(crate) fn check_generic_function_call_bidirectional(
        &mut self,
        sig: &FunctionSig,
        args: &[Spanned<Expr>],
        expected_ret: Option<&ResolvedType>,
    ) -> TypeResult<ResolvedType> {
        // Check argument count
        if sig.params.len() != args.len() {
            return Err(TypeError::ArgCount {
                expected: sig.params.len(),
                got: args.len(),
                span: None,
            });
        }

        // Create fresh type variables for each generic parameter
        let generic_substitutions: HashMap<String, ResolvedType> = sig
            .generics
            .iter()
            .map(|param| (param.clone(), self.fresh_type_var()))
            .collect();

        // If we have an expected return type, try to infer type arguments from it first
        if let Some(exp_ret) = expected_ret {
            let sig_ret = self.substitute_generics(&sig.ret, &generic_substitutions);
            // Try to unify expected return with signature return
            // This may constrain some type variables
            let _ = self.unify(exp_ret, &sig_ret);
        }

        // Check each argument and unify with parameter type
        for ((_, param_type, _), arg) in sig.params.iter().zip(args) {
            let expected_type = self.substitute_generics(param_type, &generic_substitutions);
            let expected_applied = self.apply_substitutions(&expected_type);

            // Use bidirectional checking if we have a concrete expected type
            let arg_type = if !matches!(expected_applied, ResolvedType::Var(_)) {
                self.check_expr_bidirectional(arg, CheckMode::Check(expected_applied.clone()))?
            } else {
                self.check_expr(arg)?
            };

            self.unify(&expected_applied, &arg_type)?;
        }

        // Apply substitutions to infer concrete generic types
        let inferred_type_args: Vec<_> = sig
            .generics
            .iter()
            .map(|param| {
                let ty = generic_substitutions.get(param)
                    .expect("Internal compiler error: generic parameter should exist");
                self.apply_substitutions(ty)
            })
            .collect();

        // Record the generic instantiation if all type arguments are concrete
        let all_concrete = inferred_type_args.iter().all(|t| !matches!(t, ResolvedType::Var(_)));
        if all_concrete {
            let inst = GenericInstantiation::function(&sig.name, inferred_type_args);
            self.add_instantiation(inst);
        }

        // Substitute generics in the return type and apply substitutions
        let return_type = self.substitute_generics(&sig.ret, &generic_substitutions);
        let resolved_return = self.apply_substitutions(&return_type);

        // For async functions, wrap the return type in Future
        if sig.is_async {
            Ok(ResolvedType::Future(Box::new(resolved_return)))
        } else {
            Ok(resolved_return)
        }
    }
}

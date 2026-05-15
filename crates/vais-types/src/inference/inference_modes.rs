//! Bidirectional type checking methods.
//!
//! Contains check_expr_bidirectional, check_lambda_with_expected,
//! and check_array_with_expected for top-down type propagation.

use super::CheckMode;
use crate::types::{ResolvedType, TypeError, TypeResult};
use crate::TypeChecker;
use vais_ast::{Expr, Spanned};

impl TypeChecker {
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
                    Expr::Lambda { params, body, .. } => {
                        self.check_lambda_with_expected(params, body, expected, &expr.span)
                    }
                    // Array literals can propagate element type
                    Expr::Array(elements) => {
                        self.check_array_with_expected(elements, expected, &expr.span)
                    }
                    // Phase 0 bug C16 fix: a zero-arg generic call like `vec_new()`
                    // can't infer T from arguments. After the regular inference
                    // path (which leaves T as a fresh Var), unify the call's
                    // return type against the expected hint to nail T to the
                    // concrete type, then re-register the instantiation if it
                    // became fully concrete. The call below also handles the
                    // post-unify expr_types refresh.
                    Expr::Call { func, args } => {
                        let inferred = self.check_expr(expr)?;
                        self.unify(expected, &inferred).map_err(|e| match e {
                            TypeError::Mismatch {
                                expected: exp,
                                found,
                                ..
                            } => TypeError::Mismatch {
                                expected: exp,
                                found,
                                span: Some(expr.span),
                            },
                            _ => e,
                        })?;
                        // After unify, if `func` was a known generic fn,
                        // resolve its generic args concretely and add an
                        // instantiation that downstream codegen can pick up.
                        if let Expr::Ident(fn_name) = &func.node {
                            if let Some(sig) = self.functions.get(fn_name).cloned() {
                                if !sig.generics.is_empty() {
                                    let _ = args; // args already type-checked above
                                                  // Re-derive concrete type args by unifying
                                                  // the call's return type pattern against
                                                  // the resolved inferred type.
                                    let resolved_inf = self.apply_substitutions(&inferred);
                                    let mut bindings: std::collections::HashMap<
                                        String,
                                        ResolvedType,
                                    > = std::collections::HashMap::new();
                                    if Self::infer_args_from_return_type(
                                        &sig.ret,
                                        &resolved_inf,
                                        &sig.generics,
                                        &mut bindings,
                                    ) {
                                        let concrete: Vec<ResolvedType> = sig
                                            .generics
                                            .iter()
                                            .map(|g| {
                                                bindings
                                                    .get(g)
                                                    .cloned()
                                                    .unwrap_or(ResolvedType::Unknown)
                                            })
                                            .collect();
                                        let all_concrete = concrete.iter().all(|t| {
                                            !matches!(
                                                t,
                                                ResolvedType::Var(_) | ResolvedType::Unknown
                                            )
                                        });
                                        if all_concrete {
                                            let inst = crate::types::GenericInstantiation::function(
                                                &sig.name,
                                                concrete.clone(),
                                            );
                                            self.add_instantiation(inst);
                                            // Phase 0 bug C16 part 2: propagate
                                            // transitive instantiations from this
                                            // newly-registered concrete spec so any
                                            // generic-method calls inside the body
                                            // (recorded as method-flavored callees)
                                            // get their concrete specs registered.
                                            self.propagate_transitive_instantiations(
                                                &sig.name,
                                                &sig.generics,
                                                &concrete,
                                                &mut std::collections::HashSet::new(),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        let resolved = self.apply_substitutions(&inferred);
                        if Self::has_concrete_container_generics(&resolved)
                            && !Self::has_concrete_container_generics(&inferred)
                        {
                            let file_id = if expr.span.file_id != 0 {
                                expr.span.file_id
                            } else {
                                self.current_file_id
                            };
                            let span_key = (file_id, expr.span.start, expr.span.end);
                            self.expr_types.insert(span_key, resolved.clone());
                            Ok(resolved)
                        } else {
                            Ok(inferred)
                        }
                    }
                    // For other expressions, infer then unify
                    _ => {
                        let inferred = self.check_expr(expr)?;
                        self.unify(expected, &inferred).map_err(|e| {
                            // Enhance error with span information
                            match e {
                                TypeError::Mismatch {
                                    expected: exp,
                                    found,
                                    span: _,
                                } => TypeError::Mismatch {
                                    expected: exp,
                                    found,
                                    span: Some(expr.span),
                                },
                                _ => e,
                            }
                        })?;
                        // Phase A3 (narrow refresh): after unification, if
                        // the inferred type contains a generic container
                        // whose type arguments just became concrete, refresh
                        // the expr_types entry. This lets codegen's
                        // resolve_generic_call_with_hint pick the right
                        // monomorphization for things like
                        // `vec := mut Vec.with_capacity(n)` — without the
                        // refresh, expr_types holds `Vec<Var>` and codegen
                        // falls back to an arbitrary instantiation.
                        //
                        // We deliberately skip refresh for non-container
                        // types to limit cross-module span bleed risk
                        // (the refresh broke test_types in session 3 when
                        // applied unconditionally — this narrower trigger
                        // only updates spans whose wrong/unresolved generic
                        // parameter was the actual cause of the bug).
                        let resolved = self.apply_substitutions(&inferred);
                        if Self::has_concrete_container_generics(&resolved)
                            && !Self::has_concrete_container_generics(&inferred)
                        {
                            // Phase 17.H1: same file_id fallback as check_expr.
                            let file_id = if expr.span.file_id != 0 {
                                expr.span.file_id
                            } else {
                                self.current_file_id
                            };
                            let span_key = (file_id, expr.span.start, expr.span.end);
                            self.expr_types.insert(span_key, resolved.clone());
                            return Ok(resolved);
                        }
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
            ResolvedType::Fn { params, ret, .. } => {
                (Some(params.clone()), Some(ret.as_ref().clone()))
            }
            _ => (None, None),
        };

        // Check parameter count matches if we have expected params
        if let Some(ref exp_params) = expected_params {
            if exp_params.len() != params.len() {
                return Err(TypeError::ArgCount {
                    expected: exp_params.len(),
                    got: params.len(),
                    span: Some(*span),
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
            effects: None, // Effects are inferred separately
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
        // Phase 1.12: extract element type from various container hints —
        // Array<T>, Vec<T> (builtin Named), Pointer<T>, Slice<T>, ConstArray<T>.
        let expected_elem = match expected {
            ResolvedType::Array(inner)
            | ResolvedType::Pointer(inner)
            | ResolvedType::Slice(inner)
            | ResolvedType::SliceMut(inner) => Some(inner.as_ref().clone()),
            ResolvedType::ConstArray { element, .. } | ResolvedType::Vector { element, .. } => {
                Some(element.as_ref().clone())
            }
            ResolvedType::Named { name, generics }
                if (name == "Vec" || name == "VecMut") && generics.len() == 1 =>
            {
                Some(generics[0].clone())
            }
            _ => None,
        };

        // Phase 1.12: preserve the container shape from the expected hint so
        // `a: Vec<i64> := []` returns `Vec<i64>`, not `Array<i64>`.
        let elements_len = elements.len();
        let wrap_result = |elem_ty: ResolvedType| -> ResolvedType {
            match expected {
                ResolvedType::Array(_) => ResolvedType::Array(Box::new(elem_ty)),
                ResolvedType::Pointer(_) => ResolvedType::Pointer(Box::new(elem_ty)),
                ResolvedType::Slice(_) => ResolvedType::Slice(Box::new(elem_ty)),
                ResolvedType::SliceMut(_) => ResolvedType::SliceMut(Box::new(elem_ty)),
                // Bug C15 fix: `a: [T; N] := [e1, .., eN]` — when annotated
                // size matches literal length, type the literal as ConstArray
                // so the binding lowers to a real `[N x T]` slot (not a slice).
                ResolvedType::ConstArray { size, .. } => {
                    let matches_size = size
                        .try_evaluate()
                        .map(|n| n as usize == elements_len)
                        .unwrap_or(false);
                    if matches_size {
                        ResolvedType::ConstArray {
                            element: Box::new(elem_ty),
                            size: size.clone(),
                        }
                    } else {
                        ResolvedType::Array(Box::new(elem_ty))
                    }
                }
                ResolvedType::Named { name, .. } if (name == "Vec" || name == "VecMut") => {
                    ResolvedType::Named {
                        name: name.clone(),
                        generics: vec![elem_ty],
                    }
                }
                _ => ResolvedType::Array(Box::new(elem_ty)),
            }
        };

        if elements.is_empty() {
            // Empty array: use expected element type or fresh variable
            let elem_type = expected_elem.unwrap_or_else(|| self.fresh_type_var());
            return Ok(wrap_result(elem_type));
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
            self.unify(&first_type, ty)
                .map_err(|_| TypeError::Mismatch {
                    expected: first_type.to_string(),
                    found: ty.to_string(),
                    span: Some(elements[i].span),
                })?;
        }

        Ok(wrap_result(self.apply_substitutions(&first_type)))
    }

    /// True when `ty` is a Named container (Vec/HashMap/Option/Result/Box/…)
    /// whose generic arguments are all fully concrete (no `Var`, no
    /// `Generic(_)`). Used by the bidirectional refresh to limit expr_types
    /// updates to the cases where the wrong generic instantiation is the
    /// actual source of downstream codegen bugs.
    fn has_concrete_container_generics(ty: &ResolvedType) -> bool {
        fn is_container(name: &str) -> bool {
            matches!(
                name,
                "Vec" | "HashMap" | "HashSet" | "BTreeMap" | "Option" | "Result" | "Box"
            )
        }
        fn is_concrete(t: &ResolvedType) -> bool {
            !matches!(t, ResolvedType::Var(_) | ResolvedType::Generic(_))
                && match t {
                    ResolvedType::Named { generics, .. } => generics.iter().all(is_concrete),
                    ResolvedType::Tuple(items) => items.iter().all(is_concrete),
                    ResolvedType::Array(inner)
                    | ResolvedType::Pointer(inner)
                    | ResolvedType::Ref(inner)
                    | ResolvedType::RefMut(inner)
                    | ResolvedType::Slice(inner)
                    | ResolvedType::SliceMut(inner)
                    | ResolvedType::Optional(inner)
                    | ResolvedType::Range(inner)
                    | ResolvedType::Future(inner) => is_concrete(inner),
                    ResolvedType::Map(k, v) => is_concrete(k) && is_concrete(v),
                    ResolvedType::Result(ok, err) => is_concrete(ok) && is_concrete(err),
                    _ => true,
                }
        }
        match ty {
            ResolvedType::Named { name, generics } if is_container(name) => {
                !generics.is_empty() && generics.iter().all(is_concrete)
            }
            _ => false,
        }
    }

    /// Phase 0 bug C16 helper: structurally walk `pattern` (the function's
    /// declared return type) against `concrete` (the resolved return type
    /// after unification) to derive concrete bindings for each generic name
    /// in `generics`. Returns true if the walk succeeded; the bindings map
    /// is mutated in place.
    ///
    /// Limited support: handles ResolvedType::Generic, Named (recursing into
    /// generics), Optional/Result/Tuple/Array/Ref. Other shapes are treated
    /// as "no info from this branch".
    fn infer_args_from_return_type(
        pattern: &ResolvedType,
        concrete: &ResolvedType,
        generics: &[String],
        bindings: &mut std::collections::HashMap<String, ResolvedType>,
    ) -> bool {
        match (pattern, concrete) {
            (ResolvedType::Generic(name), ty) if generics.iter().any(|g| g == name) => {
                bindings.entry(name.clone()).or_insert_with(|| ty.clone());
                true
            }
            (
                ResolvedType::Named {
                    name: n1,
                    generics: g1,
                },
                ResolvedType::Named {
                    name: n2,
                    generics: g2,
                },
            ) => {
                if n1 != n2 || g1.len() != g2.len() {
                    return false;
                }
                for (p, c) in g1.iter().zip(g2.iter()) {
                    Self::infer_args_from_return_type(p, c, generics, bindings);
                }
                true
            }
            (ResolvedType::Optional(p), ResolvedType::Optional(c)) => {
                Self::infer_args_from_return_type(p, c, generics, bindings)
            }
            (ResolvedType::Result(p_ok, p_err), ResolvedType::Result(c_ok, c_err)) => {
                Self::infer_args_from_return_type(p_ok, c_ok, generics, bindings);
                Self::infer_args_from_return_type(p_err, c_err, generics, bindings);
                true
            }
            (ResolvedType::Tuple(ps), ResolvedType::Tuple(cs)) if ps.len() == cs.len() => {
                for (p, c) in ps.iter().zip(cs.iter()) {
                    Self::infer_args_from_return_type(p, c, generics, bindings);
                }
                true
            }
            (ResolvedType::Array(p), ResolvedType::Array(c))
            | (ResolvedType::Ref(p), ResolvedType::Ref(c))
            | (ResolvedType::RefMut(p), ResolvedType::RefMut(c))
            | (ResolvedType::Pointer(p), ResolvedType::Pointer(c)) => {
                Self::infer_args_from_return_type(p, c, generics, bindings)
            }
            _ => true,
        }
    }
}

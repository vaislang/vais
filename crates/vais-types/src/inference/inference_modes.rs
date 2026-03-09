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
            self.unify(&first_type, ty)
                .map_err(|_| TypeError::Mismatch {
                    expected: first_type.to_string(),
                    found: ty.to_string(),
                    span: Some(elements[i].span),
                })?;
        }

        Ok(ResolvedType::Array(Box::new(
            self.apply_substitutions(&first_type),
        )))
    }
}

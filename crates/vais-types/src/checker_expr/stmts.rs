//! Statement type checking

use crate::types::{Linearity, ResolvedType, TypeError, TypeResult};
use crate::TypeChecker;
use vais_ast::*;

impl TypeChecker {
    /// Check a block of statements
    pub(crate) fn check_block(&mut self, stmts: &[Spanned<Stmt>]) -> TypeResult<ResolvedType> {
        let mut last_type = ResolvedType::Unit;

        let last_stmt_index = stmts.len().checked_sub(1);
        for (idx, stmt) in stmts.iter().enumerate() {
            let keep_expected = Some(idx) == last_stmt_index
                && matches!(&stmt.node, Stmt::Expr(_) | Stmt::Return(_));
            let saved_expected = if keep_expected {
                None
            } else {
                Some(std::mem::take(&mut self.expected_type_stack))
            };
            let enum_hint = if keep_expected {
                self.current_expected_type()
                    .and_then(|ty| Self::enum_name_hint_from(&ty))
            } else {
                None
            };
            if let Some(hint) = enum_hint.as_ref() {
                self.push_enum_hint(hint.clone());
            }
            let stmt_result = self.check_stmt(stmt);
            if enum_hint.is_some() {
                self.pop_enum_hint();
            }
            if let Some(saved) = saved_expected {
                self.expected_type_stack = saved;
            }
            last_type = stmt_result?;
        }

        Ok(last_type)
    }

    /// Check a statement
    pub(crate) fn check_stmt(&mut self, stmt: &Spanned<Stmt>) -> TypeResult<ResolvedType> {
        match &stmt.node {
            Stmt::Let {
                name,
                ty,
                value,
                is_mut,
                ownership,
            } => {
                // Phase 1.12: when an explicit type annotation is present,
                // propagate it as an expected type into the value expression
                // so container literals like `[]` / `[1,2,3]` can be inferred
                // as Vec<T>/Array<T>/etc. instead of decaying to Pointer(T).
                let value_type = if let Some(ty_ann) = ty {
                    let expected_hint = self.resolve_type(&ty_ann.node);
                    self.check_expr_bidirectional(value, crate::CheckMode::Check(expected_hint))?
                } else {
                    self.check_expr(value)?
                };
                let var_type = if let Some(ty) = ty {
                    let expected = self.resolve_type(&ty.node);
                    self.unify(&expected, &value_type)
                        .map_err(|e| e.with_span(value.span))?;

                    // Validate dependent type predicate at compile time for literal values
                    if let Type::Dependent {
                        var_name,
                        base,
                        predicate,
                    } = &ty.node
                    {
                        let resolved_base = self.resolve_type(&base.node);
                        if resolved_base.is_float() {
                            // f64/f32 dependent type: check float literals
                            if let Some(lit_val) =
                                Self::extract_float_literal_from_expr(&value.node)
                            {
                                let predicate_str = format!("{:?}", predicate.node);
                                if let Some(false) = Self::try_evaluate_predicate_f64(
                                    &predicate.node,
                                    var_name,
                                    lit_val,
                                ) {
                                    return Err(TypeError::RefinementViolation {
                                        predicate: predicate_str,
                                        span: Some(value.span),
                                    });
                                }
                            }
                        } else if let Some(lit_val) =
                            Self::extract_integer_literal_from_expr(&value.node)
                        {
                            let predicate_str = format!("{:?}", predicate.node);
                            self.check_refinement(
                                var_name,
                                &predicate.node,
                                &predicate_str,
                                lit_val,
                                Some(value.span),
                            )?;
                        }
                    }

                    expected
                } else {
                    value_type
                };
                let resolved_var_type = self.apply_substitutions(&var_type);
                self.record_type_instantiations(&resolved_var_type);

                // Convert AST Ownership to type system Linearity
                let linearity = match ownership {
                    Ownership::Linear => Linearity::Linear,
                    Ownership::Affine => Linearity::Affine,
                    Ownership::Move => Linearity::Affine, // Move semantics act like affine
                    Ownership::Regular => {
                        // Check if the type itself is linear/affine
                        match &var_type {
                            ResolvedType::Linear(_) => Linearity::Linear,
                            ResolvedType::Affine(_) => Linearity::Affine,
                            _ => Linearity::Unrestricted,
                        }
                    }
                };

                self.define_var_with_linearity(
                    &name.node,
                    var_type,
                    *is_mut,
                    linearity,
                    Some(name.span),
                );
                Ok(ResolvedType::Unit)
            }
            Stmt::LetDestructure {
                pattern,
                value,
                is_mut,
            } => {
                let value_type = self.check_expr(value)?;
                self.check_destructure_pattern(pattern, &value_type, *is_mut)?;
                Ok(ResolvedType::Unit)
            }
            Stmt::Expr(expr) => self.check_expr(expr),
            Stmt::Return(expr) => {
                let ret_span = expr.as_ref().map(|e| e.span);

                // A4-15 (Step 13 hard-block, 2026-05-08): escape closure detection.
                // Reject `return |params| body` where the lambda body captures any
                // variable from the enclosing function scope (escape closure with
                // non-empty captures). Such closures live on the caller's stack
                // and freeing the capture frame at return causes runtime
                // corruption (STEP7 F-18 + A4-15 fixture). Inline closures
                // (A2-04 certified subset) are unaffected — they don't reach
                // a return statement.
                //
                // Opt-out: VAIS_REJECT_A4_15=0 restores the legacy silent
                // accept (for legacy harness only; certified surfaces stay
                // safe under default).
                if let Some(ret_expr) = expr {
                    if let Expr::Lambda { params, body, .. } = &ret_expr.node {
                        let opt_out = std::env::var("VAIS_REJECT_A4_15")
                            .as_deref() == Ok("0");
                        if !opt_out {
                            let param_names: std::collections::HashSet<_> =
                                params.iter().map(|p| p.name.node.clone()).collect();
                            let captures = self.find_free_vars_in_expr(&*body, &param_names);
                            if !captures.is_empty() {
                                return Err(TypeError::Mismatch {
                                    expected: "named function pointer or capture-free closure".to_string(),
                                    found: format!(
                                        "escape closure capturing {} variable(s) from enclosing scope: [{}] — escape closures cause stack-after-return corruption (A4-15). Use a named fn or extract captured state to a struct. Set VAIS_REJECT_A4_15=0 to restore legacy silent accept.",
                                        captures.len(),
                                        captures.join(", "),
                                    ),
                                    span: ret_span,
                                });
                            }
                        }
                    }
                }

                let ret_type = if let Some(expr) = expr {
                    if let Some(expected) = self.current_fn_ret.clone() {
                        self.push_expected_type(expected.clone());
                        let result = self.check_expr_with_enum_hint(expr, &expected);
                        self.pop_expected_type();
                        result?
                    } else {
                        self.check_expr(expr)?
                    }
                } else {
                    ResolvedType::Unit
                };
                if let Some(expected) = self.current_fn_ret.clone() {
                    // Auto-deref: if returning &T but expected is T, allow implicit deref
                    let ret_type_deref = if let ResolvedType::Ref(inner) = &ret_type {
                        if self.unify(&expected, inner).is_ok() {
                            *inner.clone()
                        } else {
                            ret_type.clone()
                        }
                    } else {
                        ret_type.clone()
                    };
                    let res = self.unify(&expected, &ret_type_deref);
                    if let Some(s) = ret_span {
                        res.map_err(|e| e.with_span(s))?;
                    } else {
                        res?;
                    }
                }
                // Return has "Never" type because execution doesn't continue past it
                Ok(ResolvedType::Never)
            }
            // Break and Continue have "Never" type because execution doesn't continue past them
            Stmt::Break(maybe_expr) => {
                if self.loop_depth == 0 {
                    return Err(TypeError::Mismatch {
                        expected: "loop context".to_string(),
                        found: "break outside of loop".to_string(),
                        span: None,
                    });
                }
                if let Some(expr) = maybe_expr {
                    self.check_expr(expr)?;
                }
                Ok(ResolvedType::Never)
            }
            Stmt::Continue => {
                if self.loop_depth == 0 {
                    return Err(TypeError::Mismatch {
                        expected: "loop context".to_string(),
                        found: "continue outside of loop".to_string(),
                        span: None,
                    });
                }
                Ok(ResolvedType::Never)
            }

            Stmt::Defer(expr) => {
                // Type check the deferred expression
                // Defer expressions typically should be function calls that return unit
                self.check_expr(expr)?;
                // Defer itself doesn't produce a value in the control flow
                Ok(ResolvedType::Unit)
            }
            Stmt::Error { .. } => {
                // Error nodes from recovery mode are treated as having Unknown type.
                // The parsing error has already been reported.
                Ok(ResolvedType::Unknown)
            }
        }
    }

    /// Check a destructuring pattern against a type and bind variables
    pub(crate) fn check_destructure_pattern(
        &mut self,
        pattern: &Spanned<Pattern>,
        ty: &ResolvedType,
        is_mut: bool,
    ) -> TypeResult<()> {
        match &pattern.node {
            Pattern::Ident(name) => {
                self.define_var_with_linearity(
                    name,
                    ty.clone(),
                    is_mut,
                    Linearity::Unrestricted,
                    Some(pattern.span),
                );
                Ok(())
            }
            Pattern::Tuple(patterns) => {
                if let ResolvedType::Tuple(types) = ty {
                    if patterns.len() != types.len() {
                        return Err(TypeError::Mismatch {
                            expected: format!("tuple of {} elements", patterns.len()),
                            found: format!("tuple of {} elements", types.len()),
                            span: Some(pattern.span),
                        });
                    }
                    for (pat, elem_ty) in patterns.iter().zip(types.iter()) {
                        self.check_destructure_pattern(pat, elem_ty, is_mut)?;
                    }
                    Ok(())
                } else {
                    Err(TypeError::Mismatch {
                        expected: "tuple".to_string(),
                        found: format!("{}", ty),
                        span: Some(pattern.span),
                    })
                }
            }
            Pattern::Wildcard => Ok(()),
            _ => Err(TypeError::Mismatch {
                expected: "identifier or tuple pattern".to_string(),
                found: "unsupported pattern in destructuring".to_string(),
                span: Some(pattern.span),
            }),
        }
    }
}

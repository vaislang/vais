//! Control flow expression checking (if, loop, match, etc.)

use crate::types::{ResolvedType, TypeResult};
use crate::TypeChecker;
use vais_ast::*;

impl TypeChecker {
    /// Check if-else branch
    pub(crate) fn check_if_else(&mut self, branch: &IfElse) -> TypeResult<ResolvedType> {
        match branch {
            IfElse::ElseIf(cond, then, else_) => {
                let cond_type = self.check_expr(cond)?;
                self.unify(&cond_type, &ResolvedType::Bool)?;

                self.push_scope();
                let then_type = self.check_block(then)?;
                self.pop_scope();

                if let Some(else_branch) = else_ {
                    let else_type = self.check_if_else(else_branch)?;
                    // If branch types don't unify:
                    // - If both branches produce values (non-Unit), report a mismatch error
                    // - Otherwise, treat as statement (Unit type)
                    if self.unify(&then_type, &else_type).is_ok() {
                        return Ok(then_type);
                    }
                    if then_type != ResolvedType::Unit && else_type != ResolvedType::Unit {
                        return Err(crate::types::TypeError::Mismatch {
                            expected: then_type.to_string(),
                            found: else_type.to_string(),
                            span: None,
                        });
                    }
                    return Ok(ResolvedType::Unit);
                }

                Ok(then_type)
            }
            IfElse::Else(stmts) => {
                self.push_scope();
                let result = self.check_block(stmts);
                self.pop_scope();
                result
            }
        }
    }

    /// Check control flow expressions
    pub(crate) fn check_control_flow(
        &mut self,
        expr: &Spanned<Expr>,
    ) -> Option<TypeResult<ResolvedType>> {
        match &expr.node {
            Expr::Ternary { cond, then, else_ } => {
                let cond_type = match self.check_expr(cond) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                if let Err(e) = self.unify(&cond_type, &ResolvedType::Bool) {
                    return Some(Err(e));
                }

                let then_type = match self.check_expr(then) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                let else_type = match self.check_expr(else_) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                if let Err(e) = self.unify(&then_type, &else_type) {
                    return Some(Err(e));
                }

                Some(Ok(then_type))
            }

            Expr::If { cond, then, else_ } => {
                let cond_type = match self.check_expr(cond) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                if let Err(e) = self.unify(&cond_type, &ResolvedType::Bool) {
                    return Some(Err(e));
                }

                self.push_scope();
                let then_type = match self.check_block(then) {
                    Ok(t) => t,
                    Err(e) => {
                        self.pop_scope();
                        return Some(Err(e));
                    }
                };
                self.pop_scope();

                if let Some(else_branch) = else_ {
                    let else_type = match self.check_if_else(else_branch) {
                        Ok(t) => t,
                        Err(e) => return Some(Err(e)),
                    };
                    // If branch types don't unify:
                    // - If both branches produce values (non-Unit), report a mismatch error
                    // - Otherwise, treat as statement (Unit type)
                    if self.unify(&then_type, &else_type).is_ok() {
                        Some(Ok(then_type))
                    } else if then_type != ResolvedType::Unit && else_type != ResolvedType::Unit {
                        // Both branches produce values but types don't match — error
                        Some(Err(crate::types::TypeError::Mismatch {
                            expected: then_type.to_string(),
                            found: else_type.to_string(),
                            span: Some(expr.span),
                        }))
                    } else {
                        Some(Ok(ResolvedType::Unit))
                    }
                } else {
                    Some(Ok(ResolvedType::Unit))
                }
            }

            Expr::Loop {
                pattern,
                iter,
                body,
            } => {
                self.push_scope();
                self.loop_depth += 1;

                if let (Some(pattern), Some(iter)) = (pattern, iter) {
                    let iter_type = match self.check_expr(iter) {
                        Ok(t) => t,
                        Err(e) => {
                            self.loop_depth -= 1;
                            self.pop_scope();
                            return Some(Err(e));
                        }
                    };

                    // Try to infer the element type from the iterator
                    if let Some(elem_type) = self.get_iterator_item_type(&iter_type) {
                        // Bind the pattern variable with the inferred element type
                        if let Pattern::Ident(name) = &pattern.node {
                            self.define_var(name, elem_type, false);
                        }
                    } else {
                        // Couldn't infer iterator item type — define as Unknown to avoid
                        // "undefined variable" errors in the loop body (Phase 162).
                        if let Pattern::Ident(name) = &pattern.node {
                            self.define_var(name, ResolvedType::Unknown, false);
                            self.warnings.push(format!(
                                "Cannot infer iterator item type for variable '{}' in loop",
                                name
                            ));
                        }
                    }
                }

                if let Err(e) = self.check_block(body) {
                    self.loop_depth -= 1;
                    self.pop_scope();
                    return Some(Err(e));
                }
                self.loop_depth -= 1;
                self.pop_scope();

                Some(Ok(ResolvedType::Unit))
            }

            Expr::While { condition, body } => {
                // Check that condition is a boolean expression
                let cond_type = match self.check_expr(condition) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                if let Err(e) = self.unify(&ResolvedType::Bool, &cond_type) {
                    return Some(Err(e));
                }

                self.push_scope();
                self.loop_depth += 1;
                if let Err(e) = self.check_block(body) {
                    self.loop_depth -= 1;
                    self.pop_scope();
                    return Some(Err(e));
                }
                self.loop_depth -= 1;
                self.pop_scope();

                Some(Ok(ResolvedType::Unit))
            }

            Expr::Match {
                expr: scrutinee,
                arms,
            } => {
                let expr_type = match self.check_expr(scrutinee) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                let mut result_type: Option<ResolvedType> = None;

                for arm in arms {
                    self.push_scope();

                    // Register pattern bindings in scope
                    if let Err(e) = self.register_pattern_bindings(&arm.pattern, &expr_type) {
                        self.pop_scope();
                        return Some(Err(e));
                    }

                    // Check guard if present
                    if let Some(guard) = &arm.guard {
                        let guard_type = match self.check_expr(guard) {
                            Ok(t) => t,
                            Err(e) => {
                                self.pop_scope();
                                return Some(Err(e));
                            }
                        };
                        if let Err(e) = self.unify(&ResolvedType::Bool, &guard_type) {
                            self.pop_scope();
                            return Some(Err(e));
                        }
                    }

                    let arm_type = match self.check_expr(&arm.body) {
                        Ok(t) => t,
                        Err(_e) => {
                            // If arm body is a function/method call that failed TC,
                            // recover as Unit (void side-effect call in match arm)
                            use vais_ast::Expr;
                            match &arm.body.node {
                                Expr::Call { .. }
                                | Expr::MethodCall { .. }
                                | Expr::StaticMethodCall { .. } => {
                                    self.pop_scope();
                                    ResolvedType::Unit
                                }
                                _ => {
                                    self.pop_scope();
                                    return Some(Err(_e));
                                }
                            }
                        }
                    };
                    self.pop_scope();

                    if let Some(ref prev) = result_type {
                        if let Err(_e) = self.unify(prev, &arm_type) {
                            if *prev == ResolvedType::Unit || arm_type == ResolvedType::Unit {
                                result_type = Some(ResolvedType::Unit);
                            } else {
                                return Some(Err(_e));
                            }
                        }
                    } else {
                        result_type = Some(arm_type);
                    }
                }

                // Exhaustiveness check
                let exhaustiveness_result =
                    self.exhaustiveness_checker.check_match(&expr_type, arms);

                // Report unreachable arms as warnings
                for arm_idx in &exhaustiveness_result.unreachable_arms {
                    self.warnings
                        .push(format!("Unreachable pattern in match arm {}", arm_idx + 1));
                }

                // Non-exhaustive match is a warning (not error) for now
                // to maintain backwards compatibility
                if !exhaustiveness_result.is_exhaustive {
                    self.warnings.push(format!(
                        "Non-exhaustive match: missing patterns: {}",
                        exhaustiveness_result.missing_patterns.join(", ")
                    ));
                }

                Some(Ok(result_type.unwrap_or(ResolvedType::Unit)))
            }

            Expr::Block(stmts) => {
                self.push_scope();
                let result = self.check_block(stmts);
                self.pop_scope();
                Some(result)
            }

            _ => None,
        }
    }
}

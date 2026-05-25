//! Control flow expression checking (if, loop, match, etc.)

use crate::types::{ResolvedType, TypeResult};
use crate::TypeChecker;
use vais_ast::*;

/// Check whether a list of statements contains a `Break` statement that
/// belongs to the *current* loop level (i.e. not inside a nested loop).
/// Returns `true` if any such break exists, meaning the loop can exit normally.
fn stmts_have_direct_break(stmts: &[Spanned<Stmt>]) -> bool {
    stmts.iter().any(stmt_has_direct_break)
}

fn stmt_has_direct_break(stmt: &Spanned<Stmt>) -> bool {
    match &stmt.node {
        // A break at this level exits the enclosing loop
        Stmt::Break(_) => true,
        // Look into expression statements (if/match/block — but NOT nested loops)
        Stmt::Expr(expr) => expr_has_direct_break(expr),
        Stmt::Return(_)
        | Stmt::Let { .. }
        | Stmt::LetDestructure { .. }
        | Stmt::Continue
        | Stmt::Defer(_)
        | Stmt::Error { .. } => false,
    }
}

/// Phase 6.27c.4: decide whether a match looks like a statement whose arms
/// each trail a side-effect call whose result value is not consumed. This
/// lets us widen arm types to Unit when they would otherwise fail to
/// unify (e.g. `Vec.push` returns i64 while `HashMap.set` returns V).
///
/// Statement-like arms: the arm body is either a bare call / method-call
/// / static-method-call expression, or a block whose last statement is
/// such a call expression and whose last stmt is an expression (no tail
/// value binding). Unit-typed bodies (empty blocks, `{}`) also qualify.
///
/// Phase 6.27d.a: `self_method_name` is the *unqualified* name of the
/// enclosing function/method. A call `x.<self_method_name>(...)` in arm
/// position is recursive and participates in the return value — it is
/// value-producing, NOT statement-like, even when its return type is
/// non-Unit.
fn arm_bodies_are_statement_like(arms: &[MatchArm], self_method_name: Option<&str>) -> bool {
    arms.iter()
        .all(|arm| expr_is_statement_like(&arm.body, self_method_name))
}

fn expr_is_statement_like(expr: &Spanned<Expr>, self_method_name: Option<&str>) -> bool {
    match &expr.node {
        Expr::MethodCall { method, .. } => {
            // Phase 6.27d.a: a recursive call to the enclosing method
            // (same unqualified name) is value-producing.
            if let Some(name) = self_method_name {
                if method.node == name {
                    return false;
                }
            }
            true
        }
        Expr::Call { .. } | Expr::StaticMethodCall { .. } => true,
        Expr::Block(stmts) => {
            // Empty block → Unit → safe
            if stmts.is_empty() {
                return true;
            }
            // Last stmt must be an Expr whose value is discarded (the
            // parser produces Stmt::Expr for trailing exprs).
            match &stmts
                .last()
                .expect("invariant: stmts non-empty confirmed by is_empty() guard above")
                .node
            {
                Stmt::Expr(inner) => expr_is_statement_like(inner, self_method_name),
                // Return/Break/Continue never "produce" a value that
                // must unify with siblings
                Stmt::Return(_) | Stmt::Break(_) | Stmt::Continue => true,
                _ => false,
            }
        }
        // Unit literal — arm already returns ()
        Expr::Unit => true,
        _ => false,
    }
}

fn expr_has_direct_break(expr: &Spanned<Expr>) -> bool {
    match &expr.node {
        // Nested loops own their own breaks — do NOT descend into them
        Expr::Loop { .. } | Expr::While { .. } => false,
        // If/block/match: descend to find breaks for the current loop
        Expr::If { then, else_, .. } => {
            stmts_have_direct_break(then) || else_.as_ref().is_some_and(ifelse_has_direct_break)
        }
        Expr::Block(stmts) => stmts_have_direct_break(stmts),
        Expr::Match { arms, .. } => arms.iter().any(|arm| expr_has_direct_break(&arm.body)),
        _ => false,
    }
}

fn ifelse_has_direct_break(branch: &IfElse) -> bool {
    match branch {
        IfElse::Else(stmts) => stmts_have_direct_break(stmts),
        IfElse::ElseIf(_, stmts, next) => {
            stmts_have_direct_break(stmts)
                || next.as_ref().is_some_and(|n| ifelse_has_direct_break(n))
        }
    }
}

/// Phase 1.14: walk the current loop level's body and collect the types
/// of every `B <expr>` break statement. Returns the unified type if all
/// break-values agree, or None if no break carries a value.
fn collect_break_value_type(
    checker: &mut TypeChecker,
    stmts: &[Spanned<Stmt>],
) -> Option<ResolvedType> {
    let mut break_exprs: Vec<Spanned<Expr>> = Vec::new();
    collect_break_values_stmts(stmts, &mut break_exprs);
    if break_exprs.is_empty() {
        return None;
    }
    let mut unified: Option<ResolvedType> = None;
    for e in &break_exprs {
        let t = match checker.check_expr(e) {
            Ok(t) => t,
            Err(_) => return None,
        };
        match unified {
            None => unified = Some(t),
            Some(ref prev) => {
                if checker.unify(prev, &t).is_err() {
                    return None;
                }
            }
        }
    }
    unified.map(|t| checker.apply_substitutions(&t))
}

fn collect_break_values_stmts(stmts: &[Spanned<Stmt>], out: &mut Vec<Spanned<Expr>>) {
    for s in stmts {
        collect_break_values_stmt(s, out);
    }
}

fn collect_break_values_stmt(stmt: &Spanned<Stmt>, out: &mut Vec<Spanned<Expr>>) {
    match &stmt.node {
        Stmt::Break(Some(e)) => out.push((**e).clone()),
        Stmt::Expr(e) => collect_break_values_expr(e, out),
        _ => {}
    }
}

fn collect_break_values_expr(expr: &Spanned<Expr>, out: &mut Vec<Spanned<Expr>>) {
    match &expr.node {
        Expr::Loop { .. } | Expr::While { .. } => {}
        Expr::If { then, else_, .. } => {
            collect_break_values_stmts(then, out);
            if let Some(b) = else_ {
                collect_break_values_ifelse(b, out);
            }
        }
        Expr::Block(stmts) => collect_break_values_stmts(stmts, out),
        Expr::Match { arms, .. } => {
            for arm in arms {
                collect_break_values_expr(&arm.body, out);
            }
        }
        _ => {}
    }
}

fn collect_break_values_ifelse(branch: &IfElse, out: &mut Vec<Spanned<Expr>>) {
    match branch {
        IfElse::Else(stmts) => collect_break_values_stmts(stmts, out),
        IfElse::ElseIf(_, stmts, next) => {
            collect_break_values_stmts(stmts, out);
            if let Some(n) = next {
                collect_break_values_ifelse(n, out);
            }
        }
    }
}

impl TypeChecker {
    /// Check if-else branch
    pub(crate) fn check_if_else(&mut self, branch: &IfElse) -> TypeResult<ResolvedType> {
        match branch {
            IfElse::ElseIf(cond, then, else_) => {
                let cond_type = self.check_expr(cond)?;
                // Phase 254: lenient cond — accept Bool or any integer (truthy 0/1).
                // vaisdb stdlib uses i64-returning predicates (contains_key, is_empty,
                // contains) and chains them directly into I/while.
                //
                // A4-06 (Master Plan v16 §A4 + Step 13 stage 1): strict default.
                // Integer-as-truthy in if/else-if/while/ternary cond is rejected.
                // Legacy lenient mode opt-in via VAIS_REJECT_A4_06=0 (escape hatch).
                let strict_a4_06 = std::env::var("VAIS_REJECT_A4_06").as_deref() != Ok("0");
                if !matches!(cond_type, ResolvedType::Bool)
                    && (strict_a4_06 || !cond_type.is_integer())
                    && !matches!(cond_type, ResolvedType::Var(_) | ResolvedType::Unknown)
                {
                    self.unify(&ResolvedType::Bool, &cond_type)
                        .map_err(|e| e.with_span(cond.span))?;
                }

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
                            span: Some(cond.span),
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
                // A4-06 (Master Plan v16 §A4 + Step 13 stage 1): strict default.
                // Legacy lenient mode opt-in via VAIS_REJECT_A4_06=0 (escape hatch).
                let strict_a4_06 = std::env::var("VAIS_REJECT_A4_06").as_deref() != Ok("0");
                if !matches!(cond_type, ResolvedType::Bool)
                    && (strict_a4_06 || !cond_type.is_integer())
                    && !matches!(cond_type, ResolvedType::Var(_) | ResolvedType::Unknown)
                {
                    if let Err(e) = self.unify(&ResolvedType::Bool, &cond_type) {
                        return Some(Err(e));
                    }
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
                // A4-06 (Master Plan v16 §A4 + Step 13 stage 1): strict default.
                // Legacy lenient mode opt-in via VAIS_REJECT_A4_06=0 (escape hatch).
                let strict_a4_06 = std::env::var("VAIS_REJECT_A4_06").as_deref() != Ok("0");
                if !matches!(cond_type, ResolvedType::Bool)
                    && (strict_a4_06 || !cond_type.is_integer())
                    && !matches!(cond_type, ResolvedType::Var(_) | ResolvedType::Unknown)
                {
                    if let Err(e) = self.unify(&ResolvedType::Bool, &cond_type) {
                        return Some(Err(e));
                    }
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
                    let elem_type = self
                        .get_iterator_item_type(&iter_type)
                        .unwrap_or(ResolvedType::Unknown);

                    // Phase 24 Task 5: use register_pattern_bindings for full
                    // tuple destructuring support. Previously only Pattern::Ident
                    // was bound, so `L (i, x): vec.enumerate() { ... }` left
                    // `i` and `x` undefined. register_pattern_bindings already
                    // handles Pattern::Tuple vs ResolvedType::Tuple recursively.
                    if let Err(e) = self.register_pattern_bindings(pattern, &elem_type) {
                        self.loop_depth -= 1;
                        self.pop_scope();
                        return Some(Err(e));
                    }

                    if matches!(elem_type, ResolvedType::Unknown) {
                        if let Pattern::Ident(name) = &pattern.node {
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

                // Phase 1.14: if the loop body contains `B <expr>` at this level,
                // use the break value's type as the loop type (loop-as-expression).
                // Multiple breaks must agree on the type.
                let break_value_type = collect_break_value_type(self, body);

                // Phase 283: A bare infinite loop (`L {}` with no pattern/iter)
                // that has no reachable `break` at this loop level is diverging —
                // all exits are via `R` (return). Assign it the `Never` type so
                // it unifies with any declared return type (e.g. Result<T,E>)
                // without a spurious E001 "expected Result, found ()" error.
                let loop_type = if let Some(t) = break_value_type {
                    t
                } else if pattern.is_none() && iter.is_none() && !stmts_have_direct_break(body) {
                    ResolvedType::Never
                } else {
                    ResolvedType::Unit
                };
                Some(Ok(loop_type))
            }

            Expr::While { condition, body } => {
                // Check that condition is a boolean expression
                let cond_type = match self.check_expr(condition) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                // A4-06 (Master Plan v16 §A4 + Step 13 stage 1): strict default.
                // Legacy lenient mode opt-in via VAIS_REJECT_A4_06=0 (escape hatch).
                let strict_a4_06 = std::env::var("VAIS_REJECT_A4_06").as_deref() != Ok("0");
                if !matches!(cond_type, ResolvedType::Bool)
                    && (strict_a4_06 || !cond_type.is_integer())
                    && !matches!(cond_type, ResolvedType::Var(_) | ResolvedType::Unknown)
                {
                    if let Err(e) = self.unify(&ResolvedType::Bool, &cond_type) {
                        return Some(Err(e));
                    }
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
                // Resolve any type variables in the scrutinee before pattern
                // binding — e.g. Option<?N> where ?N was later unified with a
                // concrete type must reach register_pattern_bindings fully
                // substituted, otherwise Some(p) binds p as ?N.
                let expr_type = self.apply_substitutions(&expr_type);
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
                                // Phase 6.27d.a: recursive `x.<same_method>()`
                                // calls in arm position are value-producing
                                // and must not widen to Unit.
                                let self_method_name = self
                                    .current_fn_name
                                    .as_deref()
                                    .map(|s| s.rsplit("::").next().unwrap_or(s));
                                if arm_bodies_are_statement_like(arms, self_method_name) {
                                    // Phase 6.27c.4: statement-style match
                                    // whose arms each trail a side-effect
                                    // call whose return value nobody reads.
                                    // Widen to Unit so the match itself
                                    // types as a statement.
                                    result_type = Some(ResolvedType::Unit);
                                } else {
                                    return Some(Err(_e));
                                }
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

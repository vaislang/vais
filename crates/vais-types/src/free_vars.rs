//! Free variable analysis for closure capture.

use vais_ast::*;

use super::TypeChecker;

impl TypeChecker {
    /// Find free variables in an expression that are not in bound_vars
    pub(crate) fn find_free_vars_in_expr(
        &self,
        expr: &Spanned<Expr>,
        bound_vars: &std::collections::HashSet<String>,
    ) -> Vec<String> {
        let mut free_vars = Vec::new();
        self.collect_free_vars(&expr.node, bound_vars, &mut free_vars);
        // Remove duplicates while preserving order
        let mut seen = std::collections::HashSet::new();
        free_vars.retain(|v| seen.insert(v.clone()));
        free_vars
    }

    pub(crate) fn collect_free_vars(
        &self,
        expr: &Expr,
        bound: &std::collections::HashSet<String>,
        free: &mut Vec<String>,
    ) {
        match expr {
            Expr::Ident(name) => {
                if !bound.contains(name) && self.lookup_var(name).is_some() {
                    free.push(name.clone());
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_free_vars(&left.node, bound, free);
                self.collect_free_vars(&right.node, bound, free);
            }
            Expr::Unary { expr, .. } => {
                self.collect_free_vars(&expr.node, bound, free);
            }
            Expr::Call { func, args } => {
                self.collect_free_vars(&func.node, bound, free);
                for arg in args {
                    self.collect_free_vars(&arg.node, bound, free);
                }
            }
            Expr::If { cond, then, else_ } => {
                self.collect_free_vars(&cond.node, bound, free);
                // then is Vec<Spanned<Stmt>>
                let mut local_bound = bound.clone();
                for stmt in then {
                    match &stmt.node {
                        Stmt::Let { name, value, .. } => {
                            self.collect_free_vars(&value.node, &local_bound, free);
                            local_bound.insert(name.node.clone());
                        }
                        Stmt::Expr(e) => self.collect_free_vars(&e.node, &local_bound, free),
                        Stmt::Return(Some(e)) => {
                            self.collect_free_vars(&e.node, &local_bound, free)
                        }
                        Stmt::Break(Some(e)) => self.collect_free_vars(&e.node, &local_bound, free),
                        _ => {}
                    }
                }
                if let Some(else_br) = else_ {
                    self.collect_if_else_free_vars(else_br, bound, free);
                }
            }
            Expr::Block(stmts) => {
                let mut local_bound = bound.clone();
                for stmt in stmts {
                    match &stmt.node {
                        Stmt::Let { name, value, .. } => {
                            self.collect_free_vars(&value.node, &local_bound, free);
                            local_bound.insert(name.node.clone());
                        }
                        Stmt::Expr(e) => self.collect_free_vars(&e.node, &local_bound, free),
                        Stmt::Return(Some(e)) => {
                            self.collect_free_vars(&e.node, &local_bound, free)
                        }
                        Stmt::Break(Some(e)) => self.collect_free_vars(&e.node, &local_bound, free),
                        _ => {}
                    }
                }
            }
            Expr::MethodCall { receiver, args, .. } => {
                self.collect_free_vars(&receiver.node, bound, free);
                for arg in args {
                    self.collect_free_vars(&arg.node, bound, free);
                }
            }
            Expr::Field { expr, .. } => {
                self.collect_free_vars(&expr.node, bound, free);
            }
            Expr::Index { expr, index } => {
                self.collect_free_vars(&expr.node, bound, free);
                self.collect_free_vars(&index.node, bound, free);
            }
            Expr::Array(elems) => {
                for e in elems {
                    self.collect_free_vars(&e.node, bound, free);
                }
            }
            Expr::Tuple(elems) => {
                for e in elems {
                    self.collect_free_vars(&e.node, bound, free);
                }
            }
            Expr::StructLit { fields, .. } => {
                for (_, e) in fields {
                    self.collect_free_vars(&e.node, bound, free);
                }
            }
            Expr::MapLit(pairs) => {
                for (k, v) in pairs {
                    self.collect_free_vars(&k.node, bound, free);
                    self.collect_free_vars(&v.node, bound, free);
                }
            }
            Expr::Assign { target, value } => {
                self.collect_free_vars(&target.node, bound, free);
                self.collect_free_vars(&value.node, bound, free);
            }
            Expr::AssignOp { target, value, .. } => {
                self.collect_free_vars(&target.node, bound, free);
                self.collect_free_vars(&value.node, bound, free);
            }
            Expr::Lambda {
                params,
                body,
                capture_mode: _,
                ..
            } => {
                let mut inner_bound = bound.clone();
                for p in params {
                    inner_bound.insert(p.name.node.clone());
                }
                self.collect_free_vars(&body.node, &inner_bound, free);
            }
            Expr::Ref(inner)
            | Expr::Deref(inner)
            | Expr::Try(inner)
            | Expr::Unwrap(inner)
            | Expr::Await(inner)
            | Expr::Spawn(inner)
            | Expr::Yield(inner) => {
                self.collect_free_vars(&inner.node, bound, free);
            }
            Expr::Lazy(inner) => {
                self.collect_free_vars(&inner.node, bound, free);
            }
            Expr::Force(inner) => {
                self.collect_free_vars(&inner.node, bound, free);
            }
            Expr::Cast { expr, .. } => {
                self.collect_free_vars(&expr.node, bound, free);
            }
            Expr::Loop {
                body,
                pattern,
                iter,
            } => {
                // iter expression runs in current scope
                if let Some(it) = iter {
                    self.collect_free_vars(&it.node, bound, free);
                }
                // body is Vec<Spanned<Stmt>>, pattern may introduce bindings
                let mut local_bound = bound.clone();
                if let Some(pat) = pattern {
                    self.collect_pattern_bindings(&pat.node, &mut local_bound);
                }
                for stmt in body {
                    match &stmt.node {
                        Stmt::Let { name, value, .. } => {
                            self.collect_free_vars(&value.node, &local_bound, free);
                            local_bound.insert(name.node.clone());
                        }
                        Stmt::Expr(e) => self.collect_free_vars(&e.node, &local_bound, free),
                        Stmt::Return(Some(e)) => {
                            self.collect_free_vars(&e.node, &local_bound, free)
                        }
                        Stmt::Break(Some(e)) => self.collect_free_vars(&e.node, &local_bound, free),
                        _ => {}
                    }
                }
            }
            Expr::While { condition, body } => {
                // condition expression runs in current scope
                self.collect_free_vars(&condition.node, bound, free);
                // body is Vec<Spanned<Stmt>>
                let mut local_bound = bound.clone();
                for stmt in body {
                    match &stmt.node {
                        Stmt::Let { name, value, .. } => {
                            self.collect_free_vars(&value.node, &local_bound, free);
                            local_bound.insert(name.node.clone());
                        }
                        Stmt::Expr(e) => self.collect_free_vars(&e.node, &local_bound, free),
                        Stmt::Return(Some(e)) => {
                            self.collect_free_vars(&e.node, &local_bound, free)
                        }
                        Stmt::Break(Some(e)) => self.collect_free_vars(&e.node, &local_bound, free),
                        _ => {}
                    }
                }
            }
            Expr::Match { expr, arms } => {
                self.collect_free_vars(&expr.node, bound, free);
                for arm in arms {
                    // Pattern bindings create new scope
                    let mut arm_bound = bound.clone();
                    self.collect_pattern_bindings(&arm.pattern.node, &mut arm_bound);
                    if let Some(guard) = &arm.guard {
                        self.collect_free_vars(&guard.node, &arm_bound, free);
                    }
                    self.collect_free_vars(&arm.body.node, &arm_bound, free);
                }
            }
            // Literals and other expressions don't contain free variables
            _ => {}
        }
    }

    pub(crate) fn collect_pattern_bindings(
        &self,
        pattern: &Pattern,
        bound: &mut std::collections::HashSet<String>,
    ) {
        match pattern {
            Pattern::Ident(name) => {
                bound.insert(name.clone());
            }
            Pattern::Tuple(patterns) => {
                for p in patterns {
                    self.collect_pattern_bindings(&p.node, bound);
                }
            }
            Pattern::Struct { fields, .. } => {
                for (_, pat) in fields {
                    if let Some(p) = pat {
                        self.collect_pattern_bindings(&p.node, bound);
                    }
                }
            }
            Pattern::Variant { fields, .. } => {
                for p in fields {
                    self.collect_pattern_bindings(&p.node, bound);
                }
            }
            Pattern::Or(patterns) => {
                for p in patterns {
                    self.collect_pattern_bindings(&p.node, bound);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn collect_if_else_free_vars(
        &self,
        if_else: &IfElse,
        bound: &std::collections::HashSet<String>,
        free: &mut Vec<String>,
    ) {
        match if_else {
            IfElse::ElseIf(cond, then_stmts, else_) => {
                self.collect_free_vars(&cond.node, bound, free);
                let mut local_bound = bound.clone();
                for stmt in then_stmts {
                    match &stmt.node {
                        Stmt::Let { name, value, .. } => {
                            self.collect_free_vars(&value.node, &local_bound, free);
                            local_bound.insert(name.node.clone());
                        }
                        Stmt::Expr(e) => self.collect_free_vars(&e.node, &local_bound, free),
                        Stmt::Return(Some(e)) => {
                            self.collect_free_vars(&e.node, &local_bound, free)
                        }
                        Stmt::Break(Some(e)) => self.collect_free_vars(&e.node, &local_bound, free),
                        _ => {}
                    }
                }
                if let Some(else_br) = else_ {
                    self.collect_if_else_free_vars(else_br, bound, free);
                }
            }
            IfElse::Else(stmts) => {
                let mut local_bound = bound.clone();
                for stmt in stmts {
                    match &stmt.node {
                        Stmt::Let { name, value, .. } => {
                            self.collect_free_vars(&value.node, &local_bound, free);
                            local_bound.insert(name.node.clone());
                        }
                        Stmt::Expr(e) => self.collect_free_vars(&e.node, &local_bound, free),
                        Stmt::Return(Some(e)) => {
                            self.collect_free_vars(&e.node, &local_bound, free)
                        }
                        Stmt::Break(Some(e)) => self.collect_free_vars(&e.node, &local_bound, free),
                        _ => {}
                    }
                }
            }
        }
    }
}

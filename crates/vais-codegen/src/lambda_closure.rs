//! Lambda closure capture analysis for Vais code generator
//!
//! This module contains functions for analyzing lambda expressions and
//! determining which variables need to be captured from the enclosing scope.

use crate::CodeGenerator;
use std::collections::HashSet;
use vais_ast::{Expr, IfElse, Param, Pattern, Spanned, Stmt};

impl CodeGenerator {
    /// Find free variables in a lambda expression
    /// Returns variables that are used in the body but not bound by parameters
    pub(crate) fn find_lambda_captures(
        &self,
        params: &[Param],
        body: &Spanned<Expr>,
    ) -> Vec<String> {
        let mut param_names: HashSet<String> = params.iter().map(|p| p.name.node.clone()).collect();
        let mut free_vars = Vec::new();
        self.collect_free_vars_in_expr(&body.node, &mut param_names, &mut free_vars);
        // Deduplicate while preserving order
        let mut seen = HashSet::new();
        free_vars.retain(|v| seen.insert(v.clone()));
        free_vars
    }

    pub(crate) fn collect_free_vars_in_expr(
        &self,
        expr: &Expr,
        bound: &mut HashSet<String>,
        free: &mut Vec<String>,
    ) {
        match expr {
            Expr::Ident(name) => {
                // Only capture if it's in our locals (exists in outer scope)
                if !bound.contains(name) && self.fn_ctx.locals.contains_key(name) {
                    free.push(name.clone());
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_free_vars_in_expr(&left.node, bound, free);
                self.collect_free_vars_in_expr(&right.node, bound, free);
            }
            Expr::Unary { expr, .. } => {
                self.collect_free_vars_in_expr(&expr.node, bound, free);
            }
            Expr::Call { func, args } => {
                self.collect_free_vars_in_expr(&func.node, bound, free);
                for arg in args {
                    self.collect_free_vars_in_expr(&arg.node, bound, free);
                }
            }
            Expr::If { cond, then, else_ } => {
                self.collect_free_vars_in_expr(&cond.node, bound, free);
                // Track bindings added in then branch
                let mut new_bindings = Vec::new();
                for stmt in then {
                    self.collect_free_vars_in_stmt_with_tracking(
                        &stmt.node,
                        bound,
                        free,
                        &mut new_bindings,
                    );
                }
                // Restore bound to state before then branch
                for name in &new_bindings {
                    bound.remove(name);
                }
                if let Some(else_br) = else_ {
                    self.collect_free_vars_in_if_else(else_br, bound, free);
                }
            }
            Expr::Block(stmts) => {
                // Track bindings added in block
                let mut new_bindings = Vec::new();
                for stmt in stmts {
                    self.collect_free_vars_in_stmt_with_tracking(
                        &stmt.node,
                        bound,
                        free,
                        &mut new_bindings,
                    );
                }
                // Restore bound to state before block
                for name in &new_bindings {
                    bound.remove(name);
                }
            }
            Expr::MethodCall { receiver, args, .. } => {
                self.collect_free_vars_in_expr(&receiver.node, bound, free);
                for arg in args {
                    self.collect_free_vars_in_expr(&arg.node, bound, free);
                }
            }
            Expr::Field { expr, .. } => {
                self.collect_free_vars_in_expr(&expr.node, bound, free);
            }
            Expr::Index { expr, index } => {
                self.collect_free_vars_in_expr(&expr.node, bound, free);
                self.collect_free_vars_in_expr(&index.node, bound, free);
            }
            Expr::Array(elems) => {
                for e in elems {
                    self.collect_free_vars_in_expr(&e.node, bound, free);
                }
            }
            Expr::Tuple(elems) => {
                for e in elems {
                    self.collect_free_vars_in_expr(&e.node, bound, free);
                }
            }
            Expr::MapLit(pairs) => {
                for (k, v) in pairs {
                    self.collect_free_vars_in_expr(&k.node, bound, free);
                    self.collect_free_vars_in_expr(&v.node, bound, free);
                }
            }
            Expr::StructLit { fields, .. } => {
                for (_, e) in fields {
                    self.collect_free_vars_in_expr(&e.node, bound, free);
                }
            }
            Expr::Assign { target, value } | Expr::AssignOp { target, value, .. } => {
                self.collect_free_vars_in_expr(&target.node, bound, free);
                self.collect_free_vars_in_expr(&value.node, bound, free);
            }
            Expr::Lambda { params, body, .. } => {
                // Lambda creates a nested scope - need separate bound set
                let mut inner_bound = bound.clone();
                for p in params {
                    inner_bound.insert(p.name.node.clone());
                }
                self.collect_free_vars_in_expr(&body.node, &mut inner_bound, free);
            }
            Expr::Ref(inner)
            | Expr::Deref(inner)
            | Expr::Try(inner)
            | Expr::Unwrap(inner)
            | Expr::Await(inner)
            | Expr::Spawn(inner)
            | Expr::Yield(inner)
            | Expr::Comptime { body: inner }
            | Expr::Old(inner)
            | Expr::Assume(inner) => {
                self.collect_free_vars_in_expr(&inner.node, bound, free);
            }
            Expr::Assert { condition, message } => {
                self.collect_free_vars_in_expr(&condition.node, bound, free);
                if let Some(msg) = message {
                    self.collect_free_vars_in_expr(&msg.node, bound, free);
                }
            }
            Expr::Cast { expr, .. } => {
                self.collect_free_vars_in_expr(&expr.node, bound, free);
            }
            Expr::MacroInvoke(_) => {
                // Macro invocations should be expanded before this analysis
                // No free variables to collect from unexpanded macros
            }
            Expr::Ternary { cond, then, else_ } => {
                self.collect_free_vars_in_expr(&cond.node, bound, free);
                self.collect_free_vars_in_expr(&then.node, bound, free);
                self.collect_free_vars_in_expr(&else_.node, bound, free);
            }
            Expr::Loop {
                body,
                pattern,
                iter,
            } => {
                if let Some(it) = iter {
                    self.collect_free_vars_in_expr(&it.node, bound, free);
                }
                // Track bindings added in loop
                let mut new_bindings = Vec::new();
                if let Some(pat) = pattern {
                    self.collect_pattern_bindings_with_tracking(&pat.node, bound, &mut new_bindings);
                }
                for stmt in body {
                    self.collect_free_vars_in_stmt_with_tracking(
                        &stmt.node,
                        bound,
                        free,
                        &mut new_bindings,
                    );
                }
                // Restore bound to state before loop
                for name in &new_bindings {
                    bound.remove(name);
                }
            }
            Expr::While { condition, body } => {
                self.collect_free_vars_in_expr(&condition.node, bound, free);
                // Track bindings added in while body
                let mut new_bindings = Vec::new();
                for stmt in body {
                    self.collect_free_vars_in_stmt_with_tracking(
                        &stmt.node,
                        bound,
                        free,
                        &mut new_bindings,
                    );
                }
                // Restore bound to state before while
                for name in &new_bindings {
                    bound.remove(name);
                }
            }
            Expr::Match { expr, arms } => {
                self.collect_free_vars_in_expr(&expr.node, bound, free);
                for arm in arms {
                    // Track bindings added in this arm
                    let mut new_bindings = Vec::new();
                    self.collect_pattern_bindings_with_tracking(
                        &arm.pattern.node,
                        bound,
                        &mut new_bindings,
                    );
                    if let Some(guard) = &arm.guard {
                        self.collect_free_vars_in_expr(&guard.node, bound, free);
                    }
                    self.collect_free_vars_in_expr(&arm.body.node, bound, free);
                    // Restore bound to state before arm
                    for name in &new_bindings {
                        bound.remove(name);
                    }
                }
            }
            Expr::Range { start, end, .. } => {
                if let Some(s) = start {
                    self.collect_free_vars_in_expr(&s.node, bound, free);
                }
                if let Some(e) = end {
                    self.collect_free_vars_in_expr(&e.node, bound, free);
                }
            }
            // Literals and other expressions don't contain free variables
            _ => {}
        }
    }

    /// Collect free variables in a statement, tracking new bindings for scope restoration
    fn collect_free_vars_in_stmt_with_tracking(
        &self,
        stmt: &Stmt,
        bound: &mut HashSet<String>,
        free: &mut Vec<String>,
        new_bindings: &mut Vec<String>,
    ) {
        match stmt {
            Stmt::Let { name, value, .. } => {
                self.collect_free_vars_in_expr(&value.node, bound, free);
                let name_str = name.node.clone();
                bound.insert(name_str.clone());
                new_bindings.push(name_str);
            }
            Stmt::Expr(e) => {
                self.collect_free_vars_in_expr(&e.node, bound, free);
            }
            Stmt::Return(Some(e)) => {
                self.collect_free_vars_in_expr(&e.node, bound, free);
            }
            Stmt::Break(Some(e)) => {
                self.collect_free_vars_in_expr(&e.node, bound, free);
            }
            _ => {}
        }
    }

    pub(crate) fn collect_free_vars_in_if_else(
        &self,
        if_else: &IfElse,
        bound: &mut HashSet<String>,
        free: &mut Vec<String>,
    ) {
        match if_else {
            IfElse::ElseIf(cond, then_stmts, else_) => {
                self.collect_free_vars_in_expr(&cond.node, bound, free);
                // Track bindings added in elseif branch
                let mut new_bindings = Vec::new();
                for stmt in then_stmts {
                    self.collect_free_vars_in_stmt_with_tracking(
                        &stmt.node,
                        bound,
                        free,
                        &mut new_bindings,
                    );
                }
                // Restore bound to state before elseif
                for name in &new_bindings {
                    bound.remove(name);
                }
                if let Some(else_br) = else_ {
                    self.collect_free_vars_in_if_else(else_br, bound, free);
                }
            }
            IfElse::Else(stmts) => {
                // Track bindings added in else branch
                let mut new_bindings = Vec::new();
                for stmt in stmts {
                    self.collect_free_vars_in_stmt_with_tracking(
                        &stmt.node,
                        bound,
                        free,
                        &mut new_bindings,
                    );
                }
                // Restore bound to state before else
                for name in &new_bindings {
                    bound.remove(name);
                }
            }
        }
    }

    /// Collect pattern bindings, tracking new bindings for scope restoration
    fn collect_pattern_bindings_with_tracking(
        &self,
        pattern: &Pattern,
        bound: &mut HashSet<String>,
        new_bindings: &mut Vec<String>,
    ) {
        match pattern {
            Pattern::Ident(name) => {
                let name_str = name.to_string();
                bound.insert(name_str.clone());
                new_bindings.push(name_str);
            }
            Pattern::Tuple(patterns) => {
                for p in patterns {
                    self.collect_pattern_bindings_with_tracking(&p.node, bound, new_bindings);
                }
            }
            Pattern::Struct { fields, .. } => {
                for (name, pat) in fields {
                    if let Some(p) = pat {
                        self.collect_pattern_bindings_with_tracking(&p.node, bound, new_bindings);
                    } else {
                        // Field shorthand: {x} binds x
                        let name_str = name.node.to_string();
                        bound.insert(name_str.clone());
                        new_bindings.push(name_str);
                    }
                }
            }
            Pattern::Variant { fields, .. } => {
                for p in fields {
                    self.collect_pattern_bindings_with_tracking(&p.node, bound, new_bindings);
                }
            }
            Pattern::Or(patterns) => {
                for p in patterns {
                    self.collect_pattern_bindings_with_tracking(&p.node, bound, new_bindings);
                }
            }
            _ => {}
        }
    }
}

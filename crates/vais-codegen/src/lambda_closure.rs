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
        let param_names: HashSet<_> = params.iter().map(|p| p.name.node.clone()).collect();
        let mut free_vars = Vec::new();
        self.collect_free_vars_in_expr(&body.node, &param_names, &mut free_vars);
        // Deduplicate while preserving order
        let mut seen = HashSet::new();
        free_vars.retain(|v| seen.insert(v.clone()));
        free_vars
    }

    pub(crate) fn collect_free_vars_in_expr(
        &self,
        expr: &Expr,
        bound: &HashSet<String>,
        free: &mut Vec<String>,
    ) {
        match expr {
            Expr::Ident(name) => {
                // Only capture if it's in our locals (exists in outer scope)
                if !bound.contains(name) && self.locals.contains_key(name) {
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
                let mut local_bound = bound.clone();
                for stmt in then {
                    self.collect_free_vars_in_stmt(&stmt.node, &mut local_bound, free);
                }
                if let Some(else_br) = else_ {
                    self.collect_free_vars_in_if_else(else_br, bound, free);
                }
            }
            Expr::Block(stmts) => {
                let mut local_bound = bound.clone();
                for stmt in stmts {
                    self.collect_free_vars_in_stmt(&stmt.node, &mut local_bound, free);
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
                let mut inner_bound = bound.clone();
                for p in params {
                    inner_bound.insert(p.name.node.clone());
                }
                self.collect_free_vars_in_expr(&body.node, &inner_bound, free);
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
                let mut local_bound = bound.clone();
                if let Some(pat) = pattern {
                    self.collect_pattern_bindings(&pat.node, &mut local_bound);
                }
                for stmt in body {
                    self.collect_free_vars_in_stmt(&stmt.node, &mut local_bound, free);
                }
            }
            Expr::While { condition, body } => {
                self.collect_free_vars_in_expr(&condition.node, bound, free);
                let mut local_bound = bound.clone();
                for stmt in body {
                    self.collect_free_vars_in_stmt(&stmt.node, &mut local_bound, free);
                }
            }
            Expr::Match { expr, arms } => {
                self.collect_free_vars_in_expr(&expr.node, bound, free);
                for arm in arms {
                    let mut arm_bound = bound.clone();
                    self.collect_pattern_bindings(&arm.pattern.node, &mut arm_bound);
                    if let Some(guard) = &arm.guard {
                        self.collect_free_vars_in_expr(&guard.node, &arm_bound, free);
                    }
                    self.collect_free_vars_in_expr(&arm.body.node, &arm_bound, free);
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

    pub(crate) fn collect_free_vars_in_stmt(
        &self,
        stmt: &Stmt,
        bound: &mut HashSet<String>,
        free: &mut Vec<String>,
    ) {
        match stmt {
            Stmt::Let { name, value, .. } => {
                self.collect_free_vars_in_expr(&value.node, bound, free);
                bound.insert(name.node.clone());
            }
            Stmt::Expr(e) => self.collect_free_vars_in_expr(&e.node, bound, free),
            Stmt::Return(Some(e)) => self.collect_free_vars_in_expr(&e.node, bound, free),
            Stmt::Break(Some(e)) => self.collect_free_vars_in_expr(&e.node, bound, free),
            _ => {}
        }
    }

    pub(crate) fn collect_free_vars_in_if_else(
        &self,
        if_else: &IfElse,
        bound: &HashSet<String>,
        free: &mut Vec<String>,
    ) {
        match if_else {
            IfElse::ElseIf(cond, then_stmts, else_) => {
                self.collect_free_vars_in_expr(&cond.node, bound, free);
                let mut local_bound = bound.clone();
                for stmt in then_stmts {
                    self.collect_free_vars_in_stmt(&stmt.node, &mut local_bound, free);
                }
                if let Some(else_br) = else_ {
                    self.collect_free_vars_in_if_else(else_br, bound, free);
                }
            }
            IfElse::Else(stmts) => {
                let mut local_bound = bound.clone();
                for stmt in stmts {
                    self.collect_free_vars_in_stmt(&stmt.node, &mut local_bound, free);
                }
            }
        }
    }

    pub(crate) fn collect_pattern_bindings(&self, pattern: &Pattern, bound: &mut HashSet<String>) {
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
                for (name, pat) in fields {
                    if let Some(p) = pat {
                        self.collect_pattern_bindings(&p.node, bound);
                    } else {
                        // Field shorthand: {x} binds x
                        bound.insert(name.node.clone());
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
}

//! Folding ranges and code structure analysis for the Vais LSP backend
//!
//! This module provides folding range collection and name reference counting.

use ropey::Rope;
use tower_lsp::lsp_types::*;
use vais_ast::{Expr, FunctionBody, Item, Module, Span, Spanned, Stmt, Type};

use crate::backend::{FoldingRangeInfo, VaisBackend};

impl VaisBackend {
    pub(crate) fn count_name_in_stmt(&self, stmt: &Stmt, name: &str) -> usize {
        match stmt {
            Stmt::Let { value, ty, .. } => {
                let mut c = self.count_name_in_expr(&value.node, name);
                if let Some(t) = ty {
                    c += self.count_name_in_type(&t.node, name);
                }
                c
            }
            Stmt::Expr(e) => self.count_name_in_expr(&e.node, name),
            Stmt::Return(Some(e)) => self.count_name_in_expr(&e.node, name),
            // Assign is an Expr variant, not Stmt
            Stmt::Defer(e) => self.count_name_in_expr(&e.node, name),
            _ => 0,
        }
    }

    pub(crate) fn count_name_in_expr(&self, expr: &Expr, name: &str) -> usize {
        match expr {
            Expr::Ident(id) if id == name => 1,
            Expr::Call { func, args, .. } => {
                let mut c = self.count_name_in_expr(&func.node, name);
                for a in args {
                    c += self.count_name_in_expr(&a.node, name);
                }
                c
            }
            Expr::MethodCall { receiver, args, .. } => {
                let mut c = self.count_name_in_expr(&receiver.node, name);
                for a in args {
                    c += self.count_name_in_expr(&a.node, name);
                }
                c
            }
            Expr::Binary { left, right, .. } => {
                self.count_name_in_expr(&left.node, name)
                    + self.count_name_in_expr(&right.node, name)
            }
            Expr::Unary { expr: e, .. } => self.count_name_in_expr(&e.node, name),
            Expr::If {
                cond, then, else_, ..
            } => {
                let mut c = self.count_name_in_expr(&cond.node, name);
                for s in then {
                    c += self.count_name_in_stmt(&s.node, name);
                }
                if let Some(el) = else_ {
                    c += self.count_name_in_if_else(el, name);
                }
                c
            }
            Expr::Block(stmts) => stmts
                .iter()
                .map(|s| self.count_name_in_stmt(&s.node, name))
                .sum(),
            Expr::Array(elems) => elems
                .iter()
                .map(|e| self.count_name_in_expr(&e.node, name))
                .sum(),
            Expr::Tuple(elems) => elems
                .iter()
                .map(|e| self.count_name_in_expr(&e.node, name))
                .sum(),
            Expr::Index { expr: e, index } => {
                self.count_name_in_expr(&e.node, name) + self.count_name_in_expr(&index.node, name)
            }
            Expr::Field { expr: e, .. } => self.count_name_in_expr(&e.node, name),
            Expr::StructLit {
                name: sname,
                fields,
                ..
            } => {
                let mut c = if sname.node == name { 1 } else { 0 };
                for (_, val) in fields {
                    c += self.count_name_in_expr(&val.node, name);
                }
                c
            }
            Expr::Assign { target, value } => {
                self.count_name_in_expr(&target.node, name)
                    + self.count_name_in_expr(&value.node, name)
            }
            _ => 0,
        }
    }

    fn count_name_in_if_else(&self, if_else: &vais_ast::IfElse, name: &str) -> usize {
        match if_else {
            vais_ast::IfElse::ElseIf(cond, stmts, else_opt) => {
                let mut c = self.count_name_in_expr(&cond.node, name);
                for s in stmts {
                    c += self.count_name_in_stmt(&s.node, name);
                }
                if let Some(el) = else_opt {
                    c += self.count_name_in_if_else(el, name);
                }
                c
            }
            vais_ast::IfElse::Else(stmts) => stmts
                .iter()
                .map(|s| self.count_name_in_stmt(&s.node, name))
                .sum(),
        }
    }

    pub(crate) fn count_name_in_type(&self, ty: &Type, name: &str) -> usize {
        match ty {
            Type::Named { name: n, generics } => {
                let mut c = if n == name { 1 } else { 0 };
                for g in generics {
                    c += self.count_name_in_type(&g.node, name);
                }
                c
            }
            Type::Array(inner) => self.count_name_in_type(&inner.node, name),
            Type::Optional(inner) => self.count_name_in_type(&inner.node, name),
            Type::Result(inner) => self.count_name_in_type(&inner.node, name),
            Type::Map(key, value) => {
                self.count_name_in_type(&key.node, name)
                    + self.count_name_in_type(&value.node, name)
            }
            Type::Tuple(elems) => elems
                .iter()
                .map(|e| self.count_name_in_type(&e.node, name))
                .sum(),
            _ => 0,
        }
    }

    pub(crate) fn collect_folding_ranges(
        &self,
        ast: &Module,
        rope: &Rope,
    ) -> Vec<FoldingRangeInfo> {
        let mut ranges = Vec::new();

        for item in &ast.items {
            let item_range = self.get_folding_range_for_span(&item.span, rope);
            if let Some(range) = item_range {
                let kind = match &item.node {
                    Item::Function(_) | Item::Impl(_) => Some(FoldingRangeKind::Region),
                    Item::Use(_) => Some(FoldingRangeKind::Imports),
                    _ => Some(FoldingRangeKind::Region),
                };

                if range.end_line > range.start_line {
                    ranges.push(FoldingRangeInfo {
                        start_line: range.start_line,
                        end_line: range.end_line,
                        kind,
                    });
                }
            }

            // Add nested folding ranges for control structures
            match &item.node {
                Item::Function(f) => {
                    if let FunctionBody::Block(stmts) = &f.body {
                        self.collect_folding_from_stmts(stmts, rope, &mut ranges);
                    }
                }
                Item::Impl(impl_block) => {
                    for method in &impl_block.methods {
                        if let FunctionBody::Block(stmts) = &method.node.body {
                            self.collect_folding_from_stmts(stmts, rope, &mut ranges);
                        }
                    }
                }
                _ => {}
            }
        }

        ranges
    }

    /// Collect folding ranges from statements
    fn collect_folding_from_stmts(
        &self,
        stmts: &[Spanned<Stmt>],
        rope: &Rope,
        ranges: &mut Vec<FoldingRangeInfo>,
    ) {
        for stmt in stmts {
            if let Stmt::Expr(expr) = &stmt.node {
                self.collect_folding_from_expr(expr, rope, ranges);
            }
        }
    }

    /// Collect folding ranges from expressions
    fn collect_folding_from_expr(
        &self,
        expr: &Spanned<Expr>,
        rope: &Rope,
        ranges: &mut Vec<FoldingRangeInfo>,
    ) {
        match &expr.node {
            Expr::If { then, else_, .. } => {
                if let Some(range) = self.get_folding_range_for_span(&expr.span, rope) {
                    if range.end_line > range.start_line {
                        ranges.push(FoldingRangeInfo {
                            start_line: range.start_line,
                            end_line: range.end_line,
                            kind: Some(FoldingRangeKind::Region),
                        });
                    }
                }
                self.collect_folding_from_stmts(then, rope, ranges);
                if let Some(else_branch) = else_ {
                    self.collect_folding_from_if_else(else_branch, rope, ranges);
                }
            }
            Expr::Loop { body, .. } => {
                if let Some(range) = self.get_folding_range_for_span(&expr.span, rope) {
                    if range.end_line > range.start_line {
                        ranges.push(FoldingRangeInfo {
                            start_line: range.start_line,
                            end_line: range.end_line,
                            kind: Some(FoldingRangeKind::Region),
                        });
                    }
                }
                self.collect_folding_from_stmts(body, rope, ranges);
            }
            Expr::Match { arms, .. } => {
                if let Some(range) = self.get_folding_range_for_span(&expr.span, rope) {
                    if range.end_line > range.start_line {
                        ranges.push(FoldingRangeInfo {
                            start_line: range.start_line,
                            end_line: range.end_line,
                            kind: Some(FoldingRangeKind::Region),
                        });
                    }
                }
                for arm in arms {
                    self.collect_folding_from_expr(&arm.body, rope, ranges);
                }
            }
            Expr::Block(stmts) => {
                if let Some(range) = self.get_folding_range_for_span(&expr.span, rope) {
                    if range.end_line > range.start_line {
                        ranges.push(FoldingRangeInfo {
                            start_line: range.start_line,
                            end_line: range.end_line,
                            kind: Some(FoldingRangeKind::Region),
                        });
                    }
                }
                self.collect_folding_from_stmts(stmts, rope, ranges);
            }
            _ => {}
        }
    }

    /// Collect folding from if-else branch
    fn collect_folding_from_if_else(
        &self,
        if_else: &vais_ast::IfElse,
        rope: &Rope,
        ranges: &mut Vec<FoldingRangeInfo>,
    ) {
        match if_else {
            vais_ast::IfElse::ElseIf(_, stmts, else_opt) => {
                self.collect_folding_from_stmts(stmts, rope, ranges);
                if let Some(else_branch) = else_opt {
                    self.collect_folding_from_if_else(else_branch, rope, ranges);
                }
            }
            vais_ast::IfElse::Else(stmts) => {
                self.collect_folding_from_stmts(stmts, rope, ranges);
            }
        }
    }

    /// Get folding range for a span
    fn get_folding_range_for_span(&self, span: &Span, rope: &Rope) -> Option<FoldingRangeInfo> {
        let start_line = rope.char_to_line(span.start.min(rope.len_chars())) as u32;
        let end_line = rope.char_to_line(span.end.min(rope.len_chars())) as u32;

        if end_line > start_line {
            Some(FoldingRangeInfo {
                start_line,
                end_line,
                kind: None,
            })
        } else {
            None
        }
    }

    /// Find incoming calls to a function
    pub(crate) fn find_incoming_calls(
        &self,
        uri: &Url,
        func_name: &str,
    ) -> Vec<(String, Span, Span)> {
        let mut calls = Vec::new();

        if let Some(cache) = self.get_symbol_cache(uri) {
            for entry in &cache.call_graph {
                if entry.callee == func_name {
                    calls.push((entry.caller.clone(), entry.caller_span, entry.call_span));
                }
            }
        }

        calls
    }

    /// Find outgoing calls from a function
    pub(crate) fn find_outgoing_calls(&self, uri: &Url, func_name: &str) -> Vec<(String, Span)> {
        let mut calls = Vec::new();

        if let Some(cache) = self.get_symbol_cache(uri) {
            for entry in &cache.call_graph {
                if entry.caller == func_name {
                    calls.push((entry.callee.clone(), entry.call_span));
                }
            }
        }

        calls
    }
}

//! Symbol analysis for the Vais LSP backend
//!
//! This module provides symbol collection, reference finding, and call graph analysis.

use tower_lsp::lsp_types::*;
use vais_ast::{Expr, FunctionBody, Item, Module, Span, Spanned, Stmt};

use crate::backend::{CallGraphEntry, SymbolDef, SymbolRef, VaisBackend};

impl VaisBackend {
    pub(crate) fn collect_definitions(&self, ast: &Module) -> Vec<SymbolDef> {
        let mut defs = Vec::new();

        for item in &ast.items {
            match &item.node {
                Item::Function(f) => {
                    defs.push(SymbolDef {
                        name: f.name.node.clone(),
                        kind: SymbolKind::FUNCTION,
                        span: f.name.span,
                    });
                    // Also collect parameters as local definitions
                    for param in &f.params {
                        if param.name.node != "self" {
                            defs.push(SymbolDef {
                                name: param.name.node.clone(),
                                kind: SymbolKind::VARIABLE,
                                span: param.name.span,
                            });
                        }
                    }
                }
                Item::Struct(s) => {
                    defs.push(SymbolDef {
                        name: s.name.node.clone(),
                        kind: SymbolKind::STRUCT,
                        span: s.name.span,
                    });
                    for field in &s.fields {
                        defs.push(SymbolDef {
                            name: field.name.node.clone(),
                            kind: SymbolKind::FIELD,
                            span: field.name.span,
                        });
                    }
                }
                Item::Enum(e) => {
                    defs.push(SymbolDef {
                        name: e.name.node.clone(),
                        kind: SymbolKind::ENUM,
                        span: e.name.span,
                    });
                    for variant in &e.variants {
                        defs.push(SymbolDef {
                            name: variant.name.node.clone(),
                            kind: SymbolKind::ENUM_MEMBER,
                            span: variant.name.span,
                        });
                    }
                }
                Item::Trait(t) => {
                    defs.push(SymbolDef {
                        name: t.name.node.clone(),
                        kind: SymbolKind::INTERFACE,
                        span: t.name.span,
                    });
                }
                _ => {}
            }
        }
        defs
    }

    /// Collect all symbol references from an AST
    pub(crate) fn collect_references(&self, ast: &Module) -> Vec<SymbolRef> {
        let mut refs = Vec::new();

        for item in &ast.items {
            match &item.node {
                Item::Function(f) => match &f.body {
                    FunctionBody::Expr(expr) => {
                        self.collect_expr_refs(expr, &mut refs);
                    }
                    FunctionBody::Block(stmts) => {
                        for stmt in stmts {
                            self.collect_stmt_refs(stmt, &mut refs);
                        }
                    }
                },
                Item::Impl(impl_block) => {
                    for method in &impl_block.methods {
                        match &method.node.body {
                            FunctionBody::Expr(expr) => {
                                self.collect_expr_refs(expr, &mut refs);
                            }
                            FunctionBody::Block(stmts) => {
                                for stmt in stmts {
                                    self.collect_stmt_refs(stmt, &mut refs);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        refs
    }

    /// Collect references from an expression
    fn collect_expr_refs(&self, expr: &Spanned<Expr>, refs: &mut Vec<SymbolRef>) {
        match &expr.node {
            Expr::Ident(name) => {
                refs.push(SymbolRef {
                    name: name.clone(),
                    span: expr.span,
                });
            }
            Expr::Call { func, args } => {
                self.collect_expr_refs(func, refs);
                for arg in args {
                    self.collect_expr_refs(arg, refs);
                }
            }
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => {
                self.collect_expr_refs(receiver, refs);
                refs.push(SymbolRef {
                    name: method.node.clone(),
                    span: method.span,
                });
                for arg in args {
                    self.collect_expr_refs(arg, refs);
                }
            }
            Expr::StaticMethodCall {
                type_name,
                method,
                args,
            } => {
                refs.push(SymbolRef {
                    name: type_name.node.clone(),
                    span: type_name.span,
                });
                refs.push(SymbolRef {
                    name: method.node.clone(),
                    span: method.span,
                });
                for arg in args {
                    self.collect_expr_refs(arg, refs);
                }
            }
            Expr::Field { expr: e, field } => {
                self.collect_expr_refs(e, refs);
                refs.push(SymbolRef {
                    name: field.node.clone(),
                    span: field.span,
                });
            }
            Expr::Binary { left, right, .. } => {
                self.collect_expr_refs(left, refs);
                self.collect_expr_refs(right, refs);
            }
            Expr::Unary { expr: e, .. } => {
                self.collect_expr_refs(e, refs);
            }
            Expr::If { cond, then, else_ } => {
                self.collect_expr_refs(cond, refs);
                for stmt in then {
                    self.collect_stmt_refs(stmt, refs);
                }
                if let Some(else_branch) = else_ {
                    self.collect_if_else_refs(else_branch, refs);
                }
            }
            Expr::Loop { iter, body, .. } => {
                if let Some(iter_expr) = iter {
                    self.collect_expr_refs(iter_expr, refs);
                }
                for stmt in body {
                    self.collect_stmt_refs(stmt, refs);
                }
            }
            Expr::Match { expr: e, arms } => {
                self.collect_expr_refs(e, refs);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.collect_expr_refs(guard, refs);
                    }
                    self.collect_expr_refs(&arm.body, refs);
                }
            }
            Expr::Array(elements) | Expr::Tuple(elements) => {
                for elem in elements {
                    self.collect_expr_refs(elem, refs);
                }
            }
            Expr::StructLit { name, fields } => {
                refs.push(SymbolRef {
                    name: name.node.clone(),
                    span: name.span,
                });
                for (_, value) in fields {
                    self.collect_expr_refs(value, refs);
                }
            }
            Expr::Index { expr: e, index } => {
                self.collect_expr_refs(e, refs);
                self.collect_expr_refs(index, refs);
            }
            Expr::Await(inner) | Expr::Spawn(inner) => {
                self.collect_expr_refs(inner, refs);
            }
            Expr::Lambda { body, .. } => {
                self.collect_expr_refs(body, refs);
            }
            Expr::Ternary { cond, then, else_ } => {
                self.collect_expr_refs(cond, refs);
                self.collect_expr_refs(then, refs);
                self.collect_expr_refs(else_, refs);
            }
            Expr::Range { start, end, .. } => {
                if let Some(s) = start {
                    self.collect_expr_refs(s, refs);
                }
                if let Some(e) = end {
                    self.collect_expr_refs(e, refs);
                }
            }
            Expr::Assign { target, value } => {
                self.collect_expr_refs(target, refs);
                self.collect_expr_refs(value, refs);
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    self.collect_stmt_refs(stmt, refs);
                }
            }
            _ => {}
        }
    }

    /// Collect references from IfElse
    fn collect_if_else_refs(&self, if_else: &vais_ast::IfElse, refs: &mut Vec<SymbolRef>) {
        match if_else {
            vais_ast::IfElse::ElseIf(cond, stmts, else_opt) => {
                self.collect_expr_refs(cond, refs);
                for stmt in stmts {
                    self.collect_stmt_refs(stmt, refs);
                }
                if let Some(else_branch) = else_opt {
                    self.collect_if_else_refs(else_branch, refs);
                }
            }
            vais_ast::IfElse::Else(stmts) => {
                for stmt in stmts {
                    self.collect_stmt_refs(stmt, refs);
                }
            }
        }
    }

    /// Collect references from a statement
    fn collect_stmt_refs(&self, stmt: &Spanned<Stmt>, refs: &mut Vec<SymbolRef>) {
        match &stmt.node {
            Stmt::Let { value, .. } => {
                self.collect_expr_refs(value, refs);
            }
            Stmt::Expr(expr) => {
                self.collect_expr_refs(expr, refs);
            }
            Stmt::Return(Some(e)) => {
                self.collect_expr_refs(e, refs);
            }
            _ => {}
        }
    }

    /// Find definition for an identifier at position (using cache)
    pub(crate) fn find_definition_at(&self, uri: &Url, offset: usize) -> Option<SymbolDef> {
        let cache = self.get_symbol_cache(uri)?;

        // First check if we're on a reference
        for r in &cache.references {
            if r.span.start <= offset && offset <= r.span.end {
                // Found a reference, now find its definition
                for d in &cache.definitions {
                    if d.name == r.name {
                        return Some(d.clone());
                    }
                }
            }
        }

        // Check if we're on a definition itself
        for d in &cache.definitions {
            if d.span.start <= offset && offset <= d.span.end {
                return Some(d.clone());
            }
        }

        None
    }

    /// Find all references to a symbol (using cache)
    pub(crate) fn find_all_references(&self, uri: &Url, symbol_name: &str) -> Vec<Span> {
        let mut locations = Vec::new();

        if let Some(cache) = self.get_symbol_cache(uri) {
            // Add definition location
            for d in &cache.definitions {
                if d.name == symbol_name {
                    locations.push(d.span);
                }
            }

            // Add reference locations
            for r in &cache.references {
                if r.name == symbol_name {
                    locations.push(r.span);
                }
            }
        }

        locations
    }

    /// Get the identifier name at a position (using cache)
    pub(crate) fn get_identifier_at(&self, uri: &Url, offset: usize) -> Option<String> {
        let cache = self.get_symbol_cache(uri)?;

        for d in &cache.definitions {
            if d.span.start <= offset && offset <= d.span.end {
                return Some(d.name.clone());
            }
        }

        for r in &cache.references {
            if r.span.start <= offset && offset <= r.span.end {
                return Some(r.name.clone());
            }
        }

        None
    }

    /// Build call graph from AST
    pub(crate) fn build_call_graph(&self, ast: &Module) -> Vec<CallGraphEntry> {
        let mut entries = Vec::new();

        for item in &ast.items {
            if let Item::Function(f) = &item.node {
                let caller = f.name.node.clone();
                let caller_span = f.name.span;

                match &f.body {
                    FunctionBody::Expr(expr) => {
                        self.collect_calls_from_expr(&caller, caller_span, expr, &mut entries);
                    }
                    FunctionBody::Block(stmts) => {
                        for stmt in stmts {
                            self.collect_calls_from_stmt(&caller, caller_span, stmt, &mut entries);
                        }
                    }
                }
            }

            if let Item::Impl(impl_block) = &item.node {
                for method in &impl_block.methods {
                    let caller = method.node.name.node.clone();
                    let caller_span = method.node.name.span;

                    match &method.node.body {
                        FunctionBody::Expr(expr) => {
                            self.collect_calls_from_expr(&caller, caller_span, expr, &mut entries);
                        }
                        FunctionBody::Block(stmts) => {
                            for stmt in stmts {
                                self.collect_calls_from_stmt(
                                    &caller,
                                    caller_span,
                                    stmt,
                                    &mut entries,
                                );
                            }
                        }
                    }
                }
            }
        }

        entries
    }

    /// Collect function calls from an expression
    fn collect_calls_from_expr(
        &self,
        caller: &str,
        caller_span: Span,
        expr: &Spanned<Expr>,
        entries: &mut Vec<CallGraphEntry>,
    ) {
        match &expr.node {
            Expr::Call { func, args } => {
                if let Expr::Ident(name) = &func.node {
                    entries.push(CallGraphEntry {
                        caller: caller.to_string(),
                        caller_span,
                        callee: name.clone(),
                        call_span: expr.span,
                    });
                }
                for arg in args {
                    self.collect_calls_from_expr(caller, caller_span, arg, entries);
                }
            }
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => {
                self.collect_calls_from_expr(caller, caller_span, receiver, entries);
                entries.push(CallGraphEntry {
                    caller: caller.to_string(),
                    caller_span,
                    callee: method.node.clone(),
                    call_span: method.span,
                });
                for arg in args {
                    self.collect_calls_from_expr(caller, caller_span, arg, entries);
                }
            }
            Expr::StaticMethodCall {
                type_name: _,
                method,
                args,
            } => {
                entries.push(CallGraphEntry {
                    caller: caller.to_string(),
                    caller_span,
                    callee: method.node.clone(),
                    call_span: method.span,
                });
                for arg in args {
                    self.collect_calls_from_expr(caller, caller_span, arg, entries);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_calls_from_expr(caller, caller_span, left, entries);
                self.collect_calls_from_expr(caller, caller_span, right, entries);
            }
            Expr::Unary { expr: e, .. } => {
                self.collect_calls_from_expr(caller, caller_span, e, entries);
            }
            Expr::If { cond, then, else_ } => {
                self.collect_calls_from_expr(caller, caller_span, cond, entries);
                for stmt in then {
                    self.collect_calls_from_stmt(caller, caller_span, stmt, entries);
                }
                if let Some(else_branch) = else_ {
                    self.collect_calls_from_if_else(caller, caller_span, else_branch, entries);
                }
            }
            Expr::Loop { iter, body, .. } => {
                if let Some(iter_expr) = iter {
                    self.collect_calls_from_expr(caller, caller_span, iter_expr, entries);
                }
                for stmt in body {
                    self.collect_calls_from_stmt(caller, caller_span, stmt, entries);
                }
            }
            Expr::Match { expr: e, arms } => {
                self.collect_calls_from_expr(caller, caller_span, e, entries);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.collect_calls_from_expr(caller, caller_span, guard, entries);
                    }
                    self.collect_calls_from_expr(caller, caller_span, &arm.body, entries);
                }
            }
            Expr::Array(elements) | Expr::Tuple(elements) => {
                for elem in elements {
                    self.collect_calls_from_expr(caller, caller_span, elem, entries);
                }
            }
            Expr::StructLit { fields, .. } => {
                for (_, value) in fields {
                    self.collect_calls_from_expr(caller, caller_span, value, entries);
                }
            }
            Expr::Index { expr: e, index } => {
                self.collect_calls_from_expr(caller, caller_span, e, entries);
                self.collect_calls_from_expr(caller, caller_span, index, entries);
            }
            Expr::Await(inner) | Expr::Spawn(inner) => {
                self.collect_calls_from_expr(caller, caller_span, inner, entries);
            }
            Expr::Lambda { body, .. } => {
                self.collect_calls_from_expr(caller, caller_span, body, entries);
            }
            Expr::Ternary { cond, then, else_ } => {
                self.collect_calls_from_expr(caller, caller_span, cond, entries);
                self.collect_calls_from_expr(caller, caller_span, then, entries);
                self.collect_calls_from_expr(caller, caller_span, else_, entries);
            }
            Expr::Field { expr: e, .. } => {
                self.collect_calls_from_expr(caller, caller_span, e, entries);
            }
            Expr::Range { start, end, .. } => {
                if let Some(s) = start {
                    self.collect_calls_from_expr(caller, caller_span, s, entries);
                }
                if let Some(e) = end {
                    self.collect_calls_from_expr(caller, caller_span, e, entries);
                }
            }
            Expr::Assign { target, value } => {
                self.collect_calls_from_expr(caller, caller_span, target, entries);
                self.collect_calls_from_expr(caller, caller_span, value, entries);
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    self.collect_calls_from_stmt(caller, caller_span, stmt, entries);
                }
            }
            _ => {}
        }
    }

    /// Collect calls from if-else branch
    fn collect_calls_from_if_else(
        &self,
        caller: &str,
        caller_span: Span,
        if_else: &vais_ast::IfElse,
        entries: &mut Vec<CallGraphEntry>,
    ) {
        match if_else {
            vais_ast::IfElse::ElseIf(cond, stmts, else_opt) => {
                self.collect_calls_from_expr(caller, caller_span, cond, entries);
                for stmt in stmts {
                    self.collect_calls_from_stmt(caller, caller_span, stmt, entries);
                }
                if let Some(else_branch) = else_opt {
                    self.collect_calls_from_if_else(caller, caller_span, else_branch, entries);
                }
            }
            vais_ast::IfElse::Else(stmts) => {
                for stmt in stmts {
                    self.collect_calls_from_stmt(caller, caller_span, stmt, entries);
                }
            }
        }
    }

    /// Collect calls from a statement
    fn collect_calls_from_stmt(
        &self,
        caller: &str,
        caller_span: Span,
        stmt: &Spanned<Stmt>,
        entries: &mut Vec<CallGraphEntry>,
    ) {
        match &stmt.node {
            Stmt::Let { value, .. } => {
                self.collect_calls_from_expr(caller, caller_span, value, entries);
            }
            Stmt::Expr(expr) => {
                self.collect_calls_from_expr(caller, caller_span, expr, entries);
            }
            Stmt::Return(Some(e)) => {
                self.collect_calls_from_expr(caller, caller_span, e, entries);
            }
            _ => {}
        }
    }
}

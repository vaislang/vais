//! Type analysis and call site detection for the Vais LSP backend
//!
//! This module provides type-at-position, trait implementations, and call detection.

use ropey::Rope;
use tower_lsp::lsp_types::*;
use vais_ast::{Expr, IfElse, Item, Module, Span, Spanned, Stmt, Type};

use crate::backend::VaisBackend;

impl VaisBackend {
    pub(crate) fn find_type_at_position(
        &self,
        uri: &Url,
        position: Position,
    ) -> Option<(String, SymbolKind, Span)> {
        if let Some(doc) = self.documents.get(uri) {
            if let Some(ast) = &doc.ast {
                let line = position.line as usize;
                let col = position.character as usize;
                let line_start = doc.content.line_to_char(line);
                let offset = line_start + col;

                for item in &ast.items {
                    match &item.node {
                        Item::Struct(s) => {
                            if s.name.span.start <= offset && offset <= s.name.span.end {
                                return Some((
                                    s.name.node.clone(),
                                    SymbolKind::STRUCT,
                                    s.name.span,
                                ));
                            }
                        }
                        Item::Enum(e) => {
                            if e.name.span.start <= offset && offset <= e.name.span.end {
                                return Some((e.name.node.clone(), SymbolKind::ENUM, e.name.span));
                            }
                        }
                        Item::Trait(t) => {
                            if t.name.span.start <= offset && offset <= t.name.span.end {
                                return Some((
                                    t.name.node.clone(),
                                    SymbolKind::INTERFACE,
                                    t.name.span,
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }

    /// Find all implementations of a trait across all documents
    pub(crate) fn find_trait_implementations(&self, trait_name: &str) -> Vec<(Url, String, Span)> {
        let mut impls = Vec::new();

        for entry in self.documents.iter() {
            let uri = entry.key();
            let doc = entry.value();

            if let Some(ast) = &doc.ast {
                for item in &ast.items {
                    if let Item::Impl(impl_block) = &item.node {
                        // Check if this impl block implements the trait
                        if let Some(trait_ref) = &impl_block.trait_name {
                            if trait_ref.node == trait_name {
                                // Get the target type name
                                let type_name = match &impl_block.target_type.node {
                                    Type::Named { name, .. } => name.clone(),
                                    _ => continue,
                                };
                                impls.push((uri.clone(), type_name, impl_block.target_type.span));
                            }
                        }
                    }
                }
            }
        }

        impls
    }

    /// Find all traits implemented by a type
    pub(crate) fn find_implemented_traits(&self, type_name: &str) -> Vec<(Url, String, Span)> {
        let mut traits = Vec::new();

        for entry in self.documents.iter() {
            let uri = entry.key();
            let doc = entry.value();

            if let Some(ast) = &doc.ast {
                for item in &ast.items {
                    if let Item::Impl(impl_block) = &item.node {
                        // Check if this impl block is for the target type
                        let target_name = match &impl_block.target_type.node {
                            Type::Named { name, .. } => name.clone(),
                            _ => continue,
                        };

                        if target_name == type_name {
                            if let Some(trait_ref) = &impl_block.trait_name {
                                traits.push((uri.clone(), trait_ref.node.clone(), trait_ref.span));
                            }
                        }
                    }
                }
            }
        }

        traits
    }

    /// Find super traits of a trait
    pub(crate) fn find_super_traits(&self, trait_name: &str) -> Vec<(Url, String, Span)> {
        let mut super_traits = Vec::new();

        for entry in self.documents.iter() {
            let uri = entry.key();
            let doc = entry.value();

            if let Some(ast) = &doc.ast {
                for item in &ast.items {
                    if let Item::Trait(t) = &item.node {
                        if t.name.node == trait_name {
                            for super_trait in &t.super_traits {
                                super_traits.push((
                                    uri.clone(),
                                    super_trait.node.clone(),
                                    super_trait.span,
                                ));
                            }
                        }
                    }
                }
            }
        }

        super_traits
    }

    /// Find sub traits (traits that extend this trait)
    pub(crate) fn find_sub_traits(&self, trait_name: &str) -> Vec<(Url, String, Span)> {
        let mut sub_traits = Vec::new();

        for entry in self.documents.iter() {
            let uri = entry.key();
            let doc = entry.value();

            if let Some(ast) = &doc.ast {
                for item in &ast.items {
                    if let Item::Trait(t) = &item.node {
                        // Check if this trait extends the target trait
                        for super_trait in &t.super_traits {
                            if super_trait.node == trait_name {
                                sub_traits.push((uri.clone(), t.name.node.clone(), t.name.span));
                            }
                        }
                    }
                }
            }
        }

        sub_traits
    }

    /// Find all references to a variable in a statement
    pub(crate) fn find_var_references_in_stmt(
        &self,
        stmt: &Spanned<Stmt>,
        var_name: &str,
        refs: &mut Vec<Range>,
        rope: &Rope,
    ) {
        match &stmt.node {
            Stmt::Let { value, .. } => {
                self.find_var_references_in_expr(value, var_name, refs, rope);
            }
            Stmt::Expr(expr) => {
                self.find_var_references_in_expr(expr, var_name, refs, rope);
            }
            Stmt::Return(Some(expr)) => {
                self.find_var_references_in_expr(expr, var_name, refs, rope);
            }
            _ => {}
        }
    }

    /// Find all references to a variable in an expression
    fn find_var_references_in_expr(
        &self,
        expr: &Spanned<Expr>,
        var_name: &str,
        refs: &mut Vec<Range>,
        rope: &Rope,
    ) {
        match &expr.node {
            Expr::Ident(name) if name == var_name => {
                refs.push(self.span_to_range(rope, &expr.span));
            }
            Expr::Binary { left, right, .. } => {
                self.find_var_references_in_expr(left, var_name, refs, rope);
                self.find_var_references_in_expr(right, var_name, refs, rope);
            }
            Expr::Unary { expr: inner, .. } => {
                self.find_var_references_in_expr(inner, var_name, refs, rope);
            }
            Expr::Call { func, args, .. } => {
                self.find_var_references_in_expr(func, var_name, refs, rope);
                for arg in args {
                    self.find_var_references_in_expr(arg, var_name, refs, rope);
                }
            }
            Expr::Index {
                expr: array, index, ..
            } => {
                self.find_var_references_in_expr(array, var_name, refs, rope);
                self.find_var_references_in_expr(index, var_name, refs, rope);
            }
            Expr::Field { expr: object, .. } => {
                self.find_var_references_in_expr(object, var_name, refs, rope);
            }
            Expr::MethodCall { receiver, args, .. } => {
                self.find_var_references_in_expr(receiver, var_name, refs, rope);
                for arg in args {
                    self.find_var_references_in_expr(arg, var_name, refs, rope);
                }
            }
            Expr::StructLit { fields, .. } => {
                for (_, field_expr) in fields {
                    self.find_var_references_in_expr(field_expr, var_name, refs, rope);
                }
            }
            Expr::Array(elements) => {
                for elem in elements {
                    self.find_var_references_in_expr(elem, var_name, refs, rope);
                }
            }
            Expr::Tuple(elements) => {
                for elem in elements {
                    self.find_var_references_in_expr(elem, var_name, refs, rope);
                }
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    self.find_var_references_in_stmt(stmt, var_name, refs, rope);
                }
            }
            Expr::If { cond, then, else_ } => {
                self.find_var_references_in_expr(cond, var_name, refs, rope);
                for stmt in then {
                    self.find_var_references_in_stmt(stmt, var_name, refs, rope);
                }
                if let Some(else_branch) = else_ {
                    match else_branch {
                        IfElse::ElseIf(else_cond, else_then, else_next) => {
                            self.find_var_references_in_expr(else_cond, var_name, refs, rope);
                            for stmt in else_then {
                                self.find_var_references_in_stmt(stmt, var_name, refs, rope);
                            }
                            if let Some(next) = else_next {
                                // Recursively handle the next else-if/else
                                if let IfElse::Else(stmts) = next.as_ref() {
                                    for stmt in stmts {
                                        self.find_var_references_in_stmt(
                                            stmt, var_name, refs, rope,
                                        );
                                    }
                                }
                            }
                        }
                        IfElse::Else(else_stmts) => {
                            for stmt in else_stmts {
                                self.find_var_references_in_stmt(stmt, var_name, refs, rope);
                            }
                        }
                    }
                }
            }
            Expr::While { condition, body } => {
                self.find_var_references_in_expr(condition, var_name, refs, rope);
                for stmt in body {
                    self.find_var_references_in_stmt(stmt, var_name, refs, rope);
                }
            }
            Expr::Loop { iter, body, .. } => {
                if let Some(iterable) = iter {
                    self.find_var_references_in_expr(iterable, var_name, refs, rope);
                }
                for stmt in body {
                    self.find_var_references_in_stmt(stmt, var_name, refs, rope);
                }
            }
            Expr::Match { expr, arms } => {
                self.find_var_references_in_expr(expr, var_name, refs, rope);
                for arm in arms {
                    self.find_var_references_in_expr(&arm.body, var_name, refs, rope);
                }
            }
            Expr::Ternary { cond, then, else_ } => {
                self.find_var_references_in_expr(cond, var_name, refs, rope);
                self.find_var_references_in_expr(then, var_name, refs, rope);
                self.find_var_references_in_expr(else_, var_name, refs, rope);
            }
            _ => {}
        }
    }

    /// Find function call at cursor in a statement and add named parameter refactoring
    pub(crate) fn find_call_at_cursor_in_stmt(
        &self,
        stmt: &Spanned<Stmt>,
        cursor_offset: usize,
        rope: &Rope,
        ast: &Module,
        uri: &Url,
        actions: &mut Vec<CodeActionOrCommand>,
    ) {
        match &stmt.node {
            Stmt::Let { value, .. } => {
                self.find_call_at_cursor_in_expr(value, cursor_offset, rope, ast, uri, actions);
            }
            Stmt::Expr(expr) => {
                self.find_call_at_cursor_in_expr(expr, cursor_offset, rope, ast, uri, actions);
            }
            Stmt::Return(Some(expr)) => {
                self.find_call_at_cursor_in_expr(expr, cursor_offset, rope, ast, uri, actions);
            }
            _ => {}
        }
    }

    /// Find function call at cursor in an expression and add named parameter refactoring
    pub(crate) fn find_call_at_cursor_in_expr(
        &self,
        expr: &Spanned<Expr>,
        cursor_offset: usize,
        rope: &Rope,
        ast: &Module,
        uri: &Url,
        actions: &mut Vec<CodeActionOrCommand>,
    ) {
        if let Expr::Call { func, args, .. } = &expr.node {
            // Check if cursor is within this call expression
            if cursor_offset >= expr.span.start && cursor_offset <= expr.span.end {
                // Get the function name
                if let Expr::Ident(func_name) = &func.node {
                    // Find the function definition to get parameter names
                    for item in &ast.items {
                        if let Item::Function(function) = &item.node {
                            if function.name.node == *func_name && !args.is_empty() {
                                // Check if args are already named (simple heuristic)
                                let has_named_args = args
                                    .iter()
                                    .any(|arg| matches!(&arg.node, Expr::Binary { .. }));

                                if !has_named_args && function.params.len() == args.len() {
                                    // Build the named argument call
                                    let mut named_args_parts = Vec::new();
                                    for (arg, param) in args.iter().zip(&function.params) {
                                        let arg_text: String = rope
                                            .chars()
                                            .skip(arg.span.start)
                                            .take(arg.span.end - arg.span.start)
                                            .collect();
                                        named_args_parts
                                            .push(format!("{}: {}", param.name.node, arg_text));
                                    }
                                    let named_args_text = named_args_parts.join(", ");

                                    // Find the opening and closing parentheses
                                    let call_start = func.span.end;
                                    let call_end = expr.span.end;

                                    let edit = WorkspaceEdit {
                                        changes: Some({
                                            let mut map = std::collections::HashMap::new();
                                            map.insert(
                                                uri.clone(),
                                                vec![TextEdit {
                                                    range: Range {
                                                        start: self
                                                            .offset_to_position(rope, call_start),
                                                        end: self
                                                            .offset_to_position(rope, call_end),
                                                    },
                                                    new_text: format!("({})", named_args_text),
                                                }],
                                            );
                                            map
                                        }),
                                        document_changes: None,
                                        change_annotations: None,
                                    };

                                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                        title: "Introduce named parameters".to_string(),
                                        kind: Some(CodeActionKind::REFACTOR_REWRITE),
                                        diagnostics: None,
                                        edit: Some(edit),
                                        ..Default::default()
                                    }));
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Recursively search in nested expressions
        match &expr.node {
            Expr::Binary { left, right, .. } => {
                self.find_call_at_cursor_in_expr(left, cursor_offset, rope, ast, uri, actions);
                self.find_call_at_cursor_in_expr(right, cursor_offset, rope, ast, uri, actions);
            }
            Expr::Unary { expr: inner, .. } => {
                self.find_call_at_cursor_in_expr(inner, cursor_offset, rope, ast, uri, actions);
            }
            Expr::Call { func, args, .. } => {
                self.find_call_at_cursor_in_expr(func, cursor_offset, rope, ast, uri, actions);
                for arg in args {
                    self.find_call_at_cursor_in_expr(arg, cursor_offset, rope, ast, uri, actions);
                }
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    self.find_call_at_cursor_in_stmt(stmt, cursor_offset, rope, ast, uri, actions);
                }
            }
            _ => {}
        }
    }
}

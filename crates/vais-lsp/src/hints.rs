//! Inlay hints and type inference for the Vais LSP backend
//!
//! This module provides inlay hints for types and function parameters.

use std::collections::HashMap;
use ropey::Rope;
use tower_lsp::lsp_types::*;
use vais_ast::{Expr, FunctionBody, IfElse, Item, Module, Spanned, Stmt, Type};

use crate::backend::{InlayHintInfo, VaisBackend};

impl VaisBackend {
    fn build_function_map(&self, ast: &Module) -> HashMap<String, (Vec<String>, Option<String>)> {
        let mut func_map = HashMap::new();

        for item in &ast.items {
            match &item.node {
                Item::Function(f) => {
                    let name = f.name.node.clone();
                    let params: Vec<String> =
                        f.params.iter().map(|p| p.name.node.clone()).collect();
                    let ret_type = f.ret_type.as_ref().map(|rt| format!("{:?}", rt.node));
                    func_map.insert(name, (params, ret_type));
                }
                Item::Impl(impl_block) => {
                    for method in &impl_block.methods {
                        let name = method.node.name.node.clone();
                        let params: Vec<String> = method
                            .node
                            .params
                            .iter()
                            .map(|p| p.name.node.clone())
                            .collect();
                        let ret_type = method
                            .node
                            .ret_type
                            .as_ref()
                            .map(|rt| format!("{:?}", rt.node));
                        func_map.insert(name, (params, ret_type));
                    }
                }
                Item::Trait(trait_def) => {
                    for method in &trait_def.methods {
                        let name = method.name.node.clone();
                        let params: Vec<String> =
                            method.params.iter().map(|p| p.name.node.clone()).collect();
                        let ret_type = method.ret_type.as_ref().map(|rt| format!("{:?}", rt.node));
                        func_map.insert(name, (params, ret_type));
                    }
                }
                _ => {}
            }
        }

        func_map
    }

    /// Infer a type hint string from an expression
    fn infer_expr_type_hint(
        &self,
        expr: &Spanned<Expr>,
        func_map: &HashMap<String, (Vec<String>, Option<String>)>,
    ) -> Option<String> {
        match &expr.node {
            Expr::Int(_) => Some("i64".to_string()),
            Expr::Float(_) => Some("f64".to_string()),
            Expr::Bool(_) => Some("bool".to_string()),
            Expr::String(_) => Some("str".to_string()),
            Expr::Array(elems) => {
                if elems.is_empty() {
                    Some("[_]".to_string())
                } else {
                    let elem_type = self
                        .infer_expr_type_hint(&elems[0], func_map)
                        .unwrap_or_else(|| "_".to_string());
                    Some(format!("[{}]", elem_type))
                }
            }
            Expr::Tuple(elems) => {
                let types: Vec<String> = elems
                    .iter()
                    .map(|e| {
                        self.infer_expr_type_hint(e, func_map)
                            .unwrap_or_else(|| "_".to_string())
                    })
                    .collect();
                Some(format!("({})", types.join(", ")))
            }
            Expr::Call { func, .. } => {
                // Try to extract function name and look up return type
                if let Expr::Ident(name) = &func.node {
                    if let Some((_, ret_type)) = func_map.get(name) {
                        return ret_type.clone().or_else(|| Some("()".to_string()));
                    }
                }
                Some("_".to_string())
            }
            Expr::StructLit { name, .. } => Some(name.node.clone()),
            Expr::Range { .. } => Some("Range".to_string()),
            Expr::Block(stmts) => {
                // Try to infer from last statement/expression
                if let Some(last) = stmts.last() {
                    if let Stmt::Expr(e) = &last.node {
                        return self.infer_expr_type_hint(e, func_map);
                    } else if let Stmt::Return(Some(e)) = &last.node {
                        return self.infer_expr_type_hint(e, func_map);
                    }
                }
                Some("()".to_string())
            }
            _ => Some("_".to_string()),
        }
    }

    /// Collect parameter name hints from expressions
    fn collect_hints_from_expr(
        &self,
        expr: &Spanned<Expr>,
        func_map: &HashMap<String, (Vec<String>, Option<String>)>,
        hints: &mut Vec<InlayHintInfo>,
    ) {
        match &expr.node {
            Expr::Call { func, args, .. } => {
                // Extract function name
                if let Expr::Ident(func_name) = &func.node {
                    if let Some((params, _)) = func_map.get(func_name) {
                        // Add parameter name hints for each argument
                        for (i, arg) in args.iter().enumerate() {
                            if i < params.len() {
                                let param_name = &params[i];

                                // Skip if argument is already a variable with the same name
                                let skip = if let Expr::Ident(arg_name) = &arg.node {
                                    arg_name == param_name
                                } else {
                                    false
                                };

                                if !skip {
                                    hints.push(InlayHintInfo {
                                        position: arg.span.start,
                                        label: format!("{}: ", param_name),
                                        kind: InlayHintKind::PARAMETER,
                                    });
                                }
                            }
                        }
                    }
                }

                // Recursively process arguments
                for arg in args {
                    self.collect_hints_from_expr(arg, func_map, hints);
                }
            }
            Expr::MethodCall { receiver, args, .. } => {
                self.collect_hints_from_expr(receiver, func_map, hints);
                for arg in args {
                    self.collect_hints_from_expr(arg, func_map, hints);
                }
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    self.collect_hints_from_stmt(stmt, func_map, hints);
                }
            }
            Expr::If { cond, then, else_ } => {
                self.collect_hints_from_expr(cond, func_map, hints);
                for stmt in then {
                    self.collect_hints_from_stmt(stmt, func_map, hints);
                }
                if let Some(else_branch) = else_ {
                    match else_branch {
                        IfElse::ElseIf(cond, stmts, next_else) => {
                            self.collect_hints_from_expr(cond, func_map, hints);
                            for stmt in stmts {
                                self.collect_hints_from_stmt(stmt, func_map, hints);
                            }
                            // Recursively handle nested else-if/else
                            if let Some(_next) = next_else {
                                // Would need recursive handling here
                            }
                        }
                        IfElse::Else(stmts) => {
                            for stmt in stmts {
                                self.collect_hints_from_stmt(stmt, func_map, hints);
                            }
                        }
                    }
                }
            }
            Expr::Match { expr, arms } => {
                self.collect_hints_from_expr(expr, func_map, hints);
                for arm in arms {
                    self.collect_hints_from_expr(&arm.body, func_map, hints);
                }
            }
            Expr::Loop { body, .. } => {
                for stmt in body {
                    self.collect_hints_from_stmt(stmt, func_map, hints);
                }
            }
            Expr::While { condition, body } => {
                self.collect_hints_from_expr(condition, func_map, hints);
                for stmt in body {
                    self.collect_hints_from_stmt(stmt, func_map, hints);
                }
            }

            Expr::Array(elems) => {
                for elem in elems {
                    self.collect_hints_from_expr(elem, func_map, hints);
                }
            }
            Expr::Tuple(elems) => {
                for elem in elems {
                    self.collect_hints_from_expr(elem, func_map, hints);
                }
            }
            Expr::StructLit { fields, .. } => {
                for (_, value) in fields {
                    self.collect_hints_from_expr(value, func_map, hints);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_hints_from_expr(left, func_map, hints);
                self.collect_hints_from_expr(right, func_map, hints);
            }
            Expr::Unary { expr, .. } => {
                self.collect_hints_from_expr(expr, func_map, hints);
            }
            Expr::Index { expr, index } => {
                self.collect_hints_from_expr(expr, func_map, hints);
                self.collect_hints_from_expr(index, func_map, hints);
            }
            Expr::Cast { expr, .. } => {
                self.collect_hints_from_expr(expr, func_map, hints);
            }
            Expr::Lambda { body, .. } => {
                self.collect_hints_from_expr(body, func_map, hints);
            }

            Expr::Try(e) => {
                self.collect_hints_from_expr(e, func_map, hints);
            }

            Expr::Range { start, end, .. } => {
                if let Some(s) = start {
                    self.collect_hints_from_expr(s, func_map, hints);
                }
                if let Some(e) = end {
                    self.collect_hints_from_expr(e, func_map, hints);
                }
            }
            _ => {}
        }
    }

    /// Collect inlay hints from AST
    pub(crate) fn collect_inlay_hints(&self, ast: &Module, rope: &Rope) -> Vec<InlayHintInfo> {
        let mut hints = Vec::new();

        // Build function signature map
        let func_map = self.build_function_map(ast);

        // Collect hints from all items
        for item in &ast.items {
            match &item.node {
                Item::Function(f) => match &f.body {
                    FunctionBody::Expr(expr) => {
                        self.collect_hints_from_expr(expr, &func_map, &mut hints);
                    }
                    FunctionBody::Block(stmts) => {
                        for stmt in stmts {
                            self.collect_hints_from_stmt(stmt, &func_map, &mut hints);
                        }
                    }
                },
                Item::Impl(impl_block) => {
                    for method in &impl_block.methods {
                        match &method.node.body {
                            FunctionBody::Expr(expr) => {
                                self.collect_hints_from_expr(expr, &func_map, &mut hints);
                            }
                            FunctionBody::Block(stmts) => {
                                for stmt in stmts {
                                    self.collect_hints_from_stmt(stmt, &func_map, &mut hints);
                                }
                            }
                        }
                    }
                }
                Item::Const(const_def) => {
                    // Check const initialization expressions
                    self.collect_hints_from_expr(&const_def.value, &func_map, &mut hints);
                }
                Item::Global(global_def) => {
                    // Check global initialization expressions
                    self.collect_hints_from_expr(&global_def.value, &func_map, &mut hints);
                }
                _ => {}
            }
        }

        // Filter out hints that would create duplicates
        let _ = rope; // Rope can be used for position validation if needed
        hints
    }

    /// Collect inlay hints from a statement
    fn collect_hints_from_stmt(
        &self,
        stmt: &Spanned<Stmt>,
        func_map: &HashMap<String, (Vec<String>, Option<String>)>,
        hints: &mut Vec<InlayHintInfo>,
    ) {
        match &stmt.node {
            Stmt::Let {
                name, ty, value, ..
            } => {
                // Add type hint for variables with inferred types (no explicit type annotation)
                match ty {
                    None => {
                        // No type annotation - infer from expression
                        let type_hint = self
                            .infer_expr_type_hint(value, func_map)
                            .unwrap_or_else(|| "_".to_string());
                        hints.push(InlayHintInfo {
                            position: name.span.end,
                            label: format!(": {}", type_hint),
                            kind: InlayHintKind::TYPE,
                        });
                    }
                    Some(spanned_ty) if matches!(spanned_ty.node, Type::Infer) => {
                        // Explicit `_` type - show inferred type
                        let type_hint = self
                            .infer_expr_type_hint(value, func_map)
                            .unwrap_or_else(|| "_".to_string());
                        hints.push(InlayHintInfo {
                            position: name.span.end,
                            label: format!(": {}", type_hint),
                            kind: InlayHintKind::TYPE,
                        });
                    }
                    _ => {
                        // Explicit type annotation - no type hint needed
                    }
                }

                // Collect parameter hints from the value expression
                self.collect_hints_from_expr(value, func_map, hints);
            }
            Stmt::Expr(expr) => {
                self.collect_hints_from_expr(expr, func_map, hints);
            }
            Stmt::Return(Some(expr)) => {
                self.collect_hints_from_expr(expr, func_map, hints);
            }
            Stmt::Break(Some(expr)) => {
                self.collect_hints_from_expr(expr, func_map, hints);
            }
            Stmt::Defer(expr) => {
                self.collect_hints_from_expr(expr, func_map, hints);
            }
            _ => {}
        }
    }

    /// Count references to a symbol name in the AST (for Code Lens)
    pub(crate) fn count_references_in_ast(&self, ast: &Module, name: &str) -> usize {
        let mut count = 0;
        for item in &ast.items {
            match &item.node {
                Item::Function(f) => {
                    // Don't count the definition itself
                    if f.name.node != name {
                        count += self.count_name_in_function_body(&f.body, name);
                    }
                    // Count in parameter types
                    for param in &f.params {
                        count += self.count_name_in_type(&param.ty.node, name);
                    }
                    if let Some(ret) = &f.ret_type {
                        count += self.count_name_in_type(&ret.node, name);
                    }
                }
                Item::Impl(imp) => {
                    for method in &imp.methods {
                        count += self.count_name_in_function_body(&method.node.body, name);
                        for param in &method.node.params {
                            count += self.count_name_in_type(&param.ty.node, name);
                        }
                    }
                }
                Item::Struct(s) => {
                    if s.name.node != name {
                        for field in &s.fields {
                            count += self.count_name_in_type(&field.ty.node, name);
                        }
                    }
                }
                _ => {}
            }
        }
        count
    }

    fn count_name_in_function_body(&self, body: &FunctionBody, name: &str) -> usize {
        match body {
            FunctionBody::Expr(e) => self.count_name_in_expr(&e.node, name),
            FunctionBody::Block(stmts) => stmts
                .iter()
                .map(|s| self.count_name_in_stmt(&s.node, name))
                .sum(),
        }
    }
}

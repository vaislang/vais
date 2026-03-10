//! Expression type inference and dot-completion for LSP

use super::*;

impl TypeContext {
    /// Infer expression type from AST
    pub(crate) fn infer_expr_type(&self, expr: &Spanned<Expr>) -> LspType {
        match &expr.node {
            Expr::Int(_) => LspType::Primitive("i64".to_string()),
            Expr::Float(_) => LspType::Primitive("f64".to_string()),
            Expr::Bool(_) => LspType::Primitive("bool".to_string()),
            Expr::String(_) => LspType::Primitive("str".to_string()),
            Expr::Ident(name) => {
                // Look up variable type
                if let Some(ty) = self.variable_types.get(name) {
                    return ty.clone();
                }
                LspType::Unknown
            }
            Expr::Array(elems) => {
                if let Some(first) = elems.first() {
                    LspType::Array(Box::new(self.infer_expr_type(first)))
                } else {
                    LspType::Array(Box::new(LspType::Unknown))
                }
            }
            Expr::Tuple(elems) => {
                let types: Vec<LspType> = elems.iter().map(|e| self.infer_expr_type(e)).collect();
                LspType::Tuple(types)
            }
            Expr::Call { func, .. } => {
                if let Expr::Ident(name) = &func.node {
                    // Check for constructors
                    match name.as_str() {
                        "Some" => return LspType::Optional(Box::new(LspType::Unknown)),
                        "Ok" => {
                            return LspType::Result(
                                Box::new(LspType::Unknown),
                                Box::new(LspType::Unknown),
                            )
                        }
                        "Err" => {
                            return LspType::Result(
                                Box::new(LspType::Unknown),
                                Box::new(LspType::Unknown),
                            )
                        }
                        "None" => return LspType::Optional(Box::new(LspType::Unknown)),
                        _ => {}
                    }
                    // Look up function return type
                    if let Some(ret) = self.function_returns.get(name) {
                        return ret.clone();
                    }
                    // Check if it's a struct constructor
                    if self.structs.contains_key(name) {
                        return LspType::Named(name.clone());
                    }
                }
                LspType::Unknown
            }
            Expr::StructLit { name, .. } => LspType::Named(name.node.clone()),
            Expr::Field { expr: obj, field } => {
                let obj_type = self.infer_expr_type(obj);
                if let LspType::Named(type_name) = &obj_type {
                    // Look up field type in struct definition
                    if let Some(fields) = self.structs.get(type_name) {
                        for f in fields {
                            if f.name == field.node {
                                return f.ty.clone();
                            }
                        }
                    }
                }
                LspType::Unknown
            }
            Expr::MethodCall {
                receiver, method, ..
            } => {
                let recv_type = self.infer_expr_type(receiver);
                if let LspType::Named(type_name) = &recv_type {
                    // Look up method return type
                    if let Some(methods) = self.type_methods.get(type_name) {
                        for m in methods {
                            if m.name == method.node {
                                if let Some(ref ret_str) = m.ret_type {
                                    return parse_type_string(ret_str);
                                }
                                return LspType::Unit;
                            }
                        }
                    }
                }
                LspType::Unknown
            }
            Expr::Range { .. } => LspType::Range,
            Expr::Block(stmts) => {
                if let Some(last) = stmts.last() {
                    match &last.node {
                        Stmt::Expr(e) => return self.infer_expr_type(e),
                        Stmt::Return(Some(e)) => return self.infer_expr_type(e),
                        _ => {}
                    }
                }
                LspType::Unit
            }
            Expr::If { then, else_, .. } => {
                // Try to infer from then branch's last expression
                if let Some(last) = then.last() {
                    if let Stmt::Expr(e) = &last.node {
                        return self.infer_expr_type(e);
                    }
                }
                if let Some(IfElse::Else(stmts)) = else_ {
                    if let Some(last) = stmts.last() {
                        if let Stmt::Expr(e) = &last.node {
                            return self.infer_expr_type(e);
                        }
                    }
                }
                LspType::Unknown
            }
            _ => LspType::Unknown,
        }
    }

    /// Get completions for a type after a dot
    pub(crate) fn get_dot_completions(&self, type_name: &str) -> Vec<CompletionEntry> {
        let mut completions = Vec::new();

        // Add struct fields
        if let Some(fields) = self.structs.get(type_name) {
            for field in fields {
                completions.push(CompletionEntry {
                    label: field.name.clone(),
                    kind: CompletionKind::Field,
                    detail: field.type_display.clone(),
                    insert_text: field.name.clone(),
                    from_trait: None,
                });
            }
        }

        // Add direct impl methods
        if let Some(methods) = self.type_methods.get(type_name) {
            for method in methods {
                let params_snippet: Vec<String> = method
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, (name, _))| format!("${{{}:{}}}", i + 1, name))
                    .collect();
                let detail = format!(
                    "fn({}){}",
                    method
                        .params
                        .iter()
                        .map(|(n, t)| format!("{}: {}", n, t))
                        .collect::<Vec<_>>()
                        .join(", "),
                    method
                        .ret_type
                        .as_ref()
                        .map(|r| format!(" -> {}", r))
                        .unwrap_or_default()
                );
                completions.push(CompletionEntry {
                    label: method.name.clone(),
                    kind: CompletionKind::Method,
                    detail,
                    insert_text: format!("{}({})", method.name, params_snippet.join(", ")),
                    from_trait: method.from_trait.clone(),
                });
            }
        }

        // Add trait methods from implemented traits
        if let Some(traits) = self.type_traits.get(type_name) {
            for trait_name in traits {
                if let Some(methods) = self.trait_methods.get(trait_name) {
                    for method in methods {
                        // Skip if already provided by direct impl
                        if completions.iter().any(|c| c.label == method.name) {
                            continue;
                        }
                        let params_snippet: Vec<String> = method
                            .params
                            .iter()
                            .enumerate()
                            .map(|(i, (name, _))| format!("${{{}:{}}}", i + 1, name))
                            .collect();
                        let detail = format!(
                            "fn({}){}",
                            method
                                .params
                                .iter()
                                .map(|(n, t)| format!("{}: {}", n, t))
                                .collect::<Vec<_>>()
                                .join(", "),
                            method
                                .ret_type
                                .as_ref()
                                .map(|r| format!(" -> {}", r))
                                .unwrap_or_default()
                        );
                        completions.push(CompletionEntry {
                            label: method.name.clone(),
                            kind: CompletionKind::Method,
                            detail,
                            insert_text: format!("{}({})", method.name, params_snippet.join(", ")),
                            from_trait: Some(trait_name.clone()),
                        });
                    }
                }
            }
        }

        completions
    }
}

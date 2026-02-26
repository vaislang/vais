//! Statement code generation: Vais Stmt → JavaScript statement string

use crate::expr::sanitize_js_ident;
use crate::{JsCodeGenerator, Result};
use vais_ast::*;

impl JsCodeGenerator {
    /// Generate a JavaScript statement from a Vais Stmt
    pub(crate) fn generate_stmt(&mut self, stmt: &Stmt) -> Result<String> {
        match stmt {
            Stmt::Let {
                name,
                ty: _,
                value,
                is_mut,
                ownership: _,
            } => {
                let val = self.generate_expr(&value.node)?;
                let keyword = if *is_mut { "let" } else { "const" };
                Ok(format!(
                    "{keyword} {} = {val};",
                    sanitize_js_ident(&name.node)
                ))
            }

            Stmt::LetDestructure {
                pattern,
                value,
                is_mut,
            } => {
                let val = self.generate_expr(&value.node)?;
                let keyword = if *is_mut { "let" } else { "const" };
                let pat = self.generate_destructure_pattern(&pattern.node);
                Ok(format!("{keyword} {pat} = {val};"))
            }

            Stmt::Expr(expr) => {
                let e = self.generate_expr(&expr.node)?;
                Ok(format!("{e};"))
            }

            Stmt::Return(value) => match value {
                Some(expr) => {
                    let e = self.generate_expr(&expr.node)?;
                    Ok(format!("return {e};"))
                }
                None => Ok("return;".to_string()),
            },

            Stmt::Break(value) => match value {
                Some(expr) => {
                    let e = self.generate_expr(&expr.node)?;
                    // break with value is uncommon in JS — emit as comment + break
                    Ok(format!("/* break value: {e} */ break;"))
                }
                None => Ok("break;".to_string()),
            },

            Stmt::Continue => Ok("continue;".to_string()),

            Stmt::Defer(expr) => {
                // Defer → try/finally pattern. Simplified: just emit as a comment
                // Full defer requires tracking scope exits
                let e = self.generate_expr(&expr.node)?;
                Ok(format!("/* defer: {e} */"))
            }

            Stmt::Error { message, .. } => Ok(format!("/* parse error: {message} */")),
        }
    }

    /// Generate a destructuring pattern for let statements
    fn generate_destructure_pattern(&self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Tuple(pats) => {
                let parts: Vec<String> = pats
                    .iter()
                    .map(|p| self.generate_destructure_pattern(&p.node))
                    .collect();
                format!("[{}]", parts.join(", "))
            }
            Pattern::Ident(name) => sanitize_js_ident(name),
            Pattern::Wildcard => "_".to_string(),
            Pattern::Struct { fields, .. } => {
                let parts: Vec<String> = fields
                    .iter()
                    .map(|(name, _)| sanitize_js_ident(&name.node))
                    .collect();
                format!("{{{}}}", parts.join(", "))
            }
            _ => "_".to_string(),
        }
    }

    /// Generate function body (block of statements)
    pub(crate) fn generate_function_body(&mut self, body: &FunctionBody) -> Result<String> {
        match body {
            FunctionBody::Expr(expr) => {
                let e = self.generate_expr(&expr.node)?;
                Ok(format!("return {e};"))
            }
            FunctionBody::Block(stmts) => {
                let mut lines = Vec::new();
                let last_idx = stmts.len().saturating_sub(1);
                for (i, stmt) in stmts.iter().enumerate() {
                    if i == last_idx {
                        // Last statement: if expression, add return
                        match &stmt.node {
                            Stmt::Expr(expr) => {
                                let e = self.generate_expr(&expr.node)?;
                                lines.push(format!("{}return {e};", self.indent()));
                            }
                            Stmt::Return(_) => {
                                let s = self.generate_stmt(&stmt.node)?;
                                lines.push(format!("{}{s}", self.indent()));
                            }
                            _ => {
                                let s = self.generate_stmt(&stmt.node)?;
                                if !s.is_empty() {
                                    lines.push(format!("{}{s}", self.indent()));
                                }
                            }
                        }
                    } else {
                        let s = self.generate_stmt(&stmt.node)?;
                        if !s.is_empty() {
                            lines.push(format!("{}{s}", self.indent()));
                        }
                    }
                }
                Ok(lines.join("\n"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_let_statement() {
        let mut gen = JsCodeGenerator::new();
        let stmt = Stmt::Let {
            name: Spanned::new("x".to_string(), Span::new(0, 1)),
            ty: None,
            value: Box::new(Spanned::new(Expr::Int(42), Span::new(5, 7))),
            is_mut: false,
            ownership: Ownership::Regular,
        };
        let result = gen.generate_stmt(&stmt).unwrap();
        assert_eq!(result, "const x = 42;");
    }

    #[test]
    fn test_let_mut_statement() {
        let mut gen = JsCodeGenerator::new();
        let stmt = Stmt::Let {
            name: Spanned::new("count".to_string(), Span::new(0, 5)),
            ty: None,
            value: Box::new(Spanned::new(Expr::Int(0), Span::new(10, 11))),
            is_mut: true,
            ownership: Ownership::Regular,
        };
        let result = gen.generate_stmt(&stmt).unwrap();
        assert_eq!(result, "let count = 0;");
    }

    #[test]
    fn test_return_statement() {
        let mut gen = JsCodeGenerator::new();
        let stmt = Stmt::Return(Some(Box::new(Spanned::new(Expr::Int(1), Span::new(2, 3)))));
        let result = gen.generate_stmt(&stmt).unwrap();
        assert_eq!(result, "return 1;");
    }

    #[test]
    fn test_break_continue() {
        let mut gen = JsCodeGenerator::new();
        assert_eq!(gen.generate_stmt(&Stmt::Break(None)).unwrap(), "break;");
        assert_eq!(gen.generate_stmt(&Stmt::Continue).unwrap(), "continue;");
    }

    #[test]
    fn test_return_void() {
        let mut gen = JsCodeGenerator::new();
        let stmt = Stmt::Return(None);
        let result = gen.generate_stmt(&stmt).unwrap();
        assert_eq!(result, "return;");
    }

    #[test]
    fn test_break_with_value() {
        let mut gen = JsCodeGenerator::new();
        let stmt = Stmt::Break(Some(Box::new(Spanned::new(
            Expr::Int(42),
            Span::new(0, 2),
        ))));
        let result = gen.generate_stmt(&stmt).unwrap();
        assert!(result.contains("break;"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_defer_statement() {
        let mut gen = JsCodeGenerator::new();
        let stmt = Stmt::Defer(Box::new(Spanned::new(
            Expr::Ident("cleanup".to_string()),
            Span::new(0, 7),
        )));
        let result = gen.generate_stmt(&stmt).unwrap();
        assert!(result.contains("defer"));
        assert!(result.contains("cleanup"));
    }

    #[test]
    fn test_error_statement() {
        let mut gen = JsCodeGenerator::new();
        let stmt = Stmt::Error {
            message: "something broke".to_string(),
            skipped_tokens: vec![],
        };
        let result = gen.generate_stmt(&stmt).unwrap();
        assert!(result.contains("parse error"));
        assert!(result.contains("something broke"));
    }

    #[test]
    fn test_expr_statement() {
        let mut gen = JsCodeGenerator::new();
        let stmt = Stmt::Expr(Box::new(Spanned::new(
            Expr::Int(99),
            Span::new(0, 2),
        )));
        let result = gen.generate_stmt(&stmt).unwrap();
        assert_eq!(result, "99;");
    }

    #[test]
    fn test_let_destructure_tuple() {
        let mut gen = JsCodeGenerator::new();
        let stmt = Stmt::LetDestructure {
            pattern: Spanned::new(
                Pattern::Tuple(vec![
                    Spanned::new(Pattern::Ident("a".to_string()), Span::new(0, 1)),
                    Spanned::new(Pattern::Ident("b".to_string()), Span::new(3, 4)),
                ]),
                Span::new(0, 5),
            ),
            value: Box::new(Spanned::new(Expr::Int(0), Span::new(8, 9))),
            is_mut: false,
        };
        let result = gen.generate_stmt(&stmt).unwrap();
        assert!(result.contains("const"));
        assert!(result.contains("[a, b]"));
    }

    #[test]
    fn test_let_destructure_mutable() {
        let mut gen = JsCodeGenerator::new();
        let stmt = Stmt::LetDestructure {
            pattern: Spanned::new(Pattern::Ident("x".to_string()), Span::new(0, 1)),
            value: Box::new(Spanned::new(Expr::Int(0), Span::new(5, 6))),
            is_mut: true,
        };
        let result = gen.generate_stmt(&stmt).unwrap();
        assert!(result.starts_with("let "));
    }

    #[test]
    fn test_function_body_expr() {
        let mut gen = JsCodeGenerator::new();
        let body = FunctionBody::Expr(Box::new(Spanned::new(
            Expr::Int(42),
            Span::new(0, 2),
        )));
        let result = gen.generate_function_body(&body).unwrap();
        assert_eq!(result, "return 42;");
    }

    #[test]
    fn test_function_body_block_with_return() {
        let mut gen = JsCodeGenerator::new();
        let body = FunctionBody::Block(vec![
            Spanned::new(
                Stmt::Let {
                    name: Spanned::new("x".to_string(), Span::new(0, 1)),
                    ty: None,
                    value: Box::new(Spanned::new(Expr::Int(1), Span::new(5, 6))),
                    is_mut: false,
                    ownership: Ownership::Regular,
                },
                Span::new(0, 7),
            ),
            Spanned::new(
                Stmt::Expr(Box::new(Spanned::new(
                    Expr::Ident("x".to_string()),
                    Span::new(8, 9),
                ))),
                Span::new(8, 10),
            ),
        ]);
        let result = gen.generate_function_body(&body).unwrap();
        assert!(result.contains("const x = 1;"));
        assert!(result.contains("return x;"));
    }
}

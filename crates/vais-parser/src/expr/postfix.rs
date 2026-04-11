//! Postfix operator parsing.
//!
//! Handles postfix expressions including function calls, field access, method calls,
//! indexing, await, try (?), unwrap (!), type casts (as), and static method calls (::).

use vais_ast::*;
use vais_lexer::{SpannedToken, Token};

use crate::{ParseError, ParseResult, Parser};

impl Parser {
    /// Parse postfix expressions (calls, field access, etc.)
    pub(crate) fn parse_postfix(&mut self) -> ParseResult<Spanned<Expr>> {
        let mut expr = self.parse_primary()?;

        loop {
            let start = expr.span.start;

            if self.check(&Token::LParen) {
                // Don't treat `(` as function call if there's a newline between
                // the expression and the `(`. This prevents:
                //   x := get_pair()
                //   (a, b) := ...
                // from being parsed as `get_pair()(a, b)`
                let paren_start = self.peek().map(|t| t.span.start).unwrap_or(0);
                let expr_end = expr.span.end;
                if self.has_newline_between(expr_end, paren_start) {
                    break;
                }
                self.advance_skip();
                let args = self.parse_args()?;
                self.expect_skip(&Token::RParen)?;
                let end = self.prev_span().end;
                expr = Spanned::new(
                    Expr::Call {
                        func: Box::new(expr),
                        args,
                    },
                    Span::new(start, end),
                );
            } else if self.check(&Token::Dot) {
                self.advance_skip();
                if self.check(&Token::Await) {
                    self.advance_skip();
                    let end = self.prev_span().end;
                    expr = Spanned::new(Expr::Await(Box::new(expr)), Span::new(start, end));
                } else if let Some(SpannedToken {
                    token: Token::Int(n),
                    ..
                }) = self.peek().cloned()
                {
                    // Tuple field access: expr.0, expr.1, etc.
                    if n < 0 {
                        return Err(ParseError::UnexpectedToken {
                            found: Token::Int(n),
                            span: self.prev_span(),
                            expected: "non-negative integer for tuple field access".into(),
                        });
                    }
                    self.advance_skip();
                    let end = self.prev_span().end;
                    expr = Spanned::new(
                        Expr::TupleFieldAccess {
                            expr: Box::new(expr),
                            index: n as usize,
                        },
                        Span::new(start, end),
                    );
                } else {
                    let field = self.parse_ident()?;
                    if self.check(&Token::LParen) {
                        self.advance_skip();
                        let args = self.parse_args()?;
                        self.expect_skip(&Token::RParen)?;
                        let end = self.prev_span().end;

                        // Check if receiver is a type name (starts with uppercase)
                        // If so, this is a static method call
                        let is_static = if let Expr::Ident(name) = &expr.node {
                            name.chars()
                                .next()
                                .map(|c| c.is_uppercase())
                                .unwrap_or(false)
                        } else {
                            false
                        };

                        if is_static {
                            if let Expr::Ident(type_name) = &expr.node {
                                let tn = type_name.clone();
                                let sp = expr.span;
                                expr = Spanned::new(
                                    Expr::StaticMethodCall {
                                        type_name: Spanned::new(tn, sp),
                                        method: field,
                                        args,
                                    },
                                    Span::new(start, end),
                                );
                            }
                        } else {
                            expr = Spanned::new(
                                Expr::MethodCall {
                                    receiver: Box::new(expr),
                                    method: field,
                                    args,
                                },
                                Span::new(start, end),
                            );
                        }
                    } else if self.check(&Token::LBrace) && self.allow_struct_literal {
                        // Check for enum variant struct construction: Type.Variant { field: value }
                        let is_type = if let Expr::Ident(name) = &expr.node {
                            name.chars()
                                .next()
                                .map(|c| c.is_uppercase())
                                .unwrap_or(false)
                        } else {
                            false
                        };
                        if is_type {
                            // Parse as struct literal with variant name
                            // If expr is an Ident (type name), this could be EnumType.Variant { fields }
                            let parent_type_name = if let Expr::Ident(name) = &expr.node {
                                Some(name.clone())
                            } else {
                                None
                            };
                            self.advance_skip(); // skip '{'
                            let mut fields = Vec::new();
                            while !self.check(&Token::RBrace) && !self.is_at_end() {
                                let fname = self.parse_ident()?;
                                let fval = if self.check(&Token::Colon) {
                                    self.advance_skip();
                                    self.parse_expr()?
                                } else {
                                    // Shorthand: `{ x }` means `{ x: x }`
                                    let sp = fname.span;
                                    Spanned::new(Expr::Ident(fname.node.clone()), sp)
                                };
                                fields.push((fname, fval));
                                if !self.check(&Token::RBrace) {
                                    self.expect_skip(&Token::Comma)?;
                                }
                            }
                            self.expect_skip(&Token::RBrace)?;
                            let end = self.prev_span().end;
                            expr = Spanned::new(
                                Expr::StructLit {
                                    name: field,
                                    fields,
                                    enum_name: parent_type_name,
                                },
                                Span::new(start, end),
                            );
                        } else {
                            let end = field.span.end;
                            expr = Spanned::new(
                                Expr::Field {
                                    expr: Box::new(expr),
                                    field,
                                },
                                Span::new(start, end),
                            );
                        }
                    } else {
                        let end = field.span.end;
                        expr = Spanned::new(
                            Expr::Field {
                                expr: Box::new(expr),
                                field,
                            },
                            Span::new(start, end),
                        );
                    }
                }
            } else if self.check(&Token::ColonColon) {
                // `Type::name` — either static method call or enum variant access.
                self.advance_skip();
                let method_or_variant = self.parse_ident()?;

                if let Expr::Ident(type_name) = &expr.node {
                    let tn = type_name.clone();
                    let sp = expr.span;

                    if !self.check(&Token::LParen) {
                        // No argument list → enum variant access (unit variant)
                        // e.g. Color::Red, Status::Ok
                        let end = method_or_variant.span.end;
                        expr = Spanned::new(
                            Expr::EnumAccess {
                                enum_name: tn,
                                variant: method_or_variant.node,
                                data: None,
                            },
                            Span::new(start, end),
                        );
                    } else {
                        // `(` follows — static method call: Type::method(args)
                        self.advance_skip(); // consume '('
                        let args = self.parse_args()?;
                        self.expect_skip(&Token::RParen)?;
                        let end = self.prev_span().end;
                        expr = Spanned::new(
                            Expr::StaticMethodCall {
                                type_name: Spanned::new(tn, sp),
                                method: method_or_variant,
                                args,
                            },
                            Span::new(start, end),
                        );
                    }
                } else {
                    return Err(ParseError::UnexpectedToken {
                        found: Token::ColonColon,
                        span: self.prev_span(),
                        expected: "type name before '::'".into(),
                    });
                }
            } else if self.check(&Token::LBracket) {
                self.advance_skip();
                let index = self.parse_expr()?;
                self.expect_skip(&Token::RBracket)?;
                let end = self.prev_span().end;
                expr = Spanned::new(
                    Expr::Index {
                        expr: Box::new(expr),
                        index: Box::new(index),
                    },
                    Span::new(start, end),
                );
            } else if self.check(&Token::Question) {
                // Distinguish postfix try (expr?) from ternary (cond ? then : else)
                // Check if ? is followed by an expression-start token
                // If yes, it could be ternary - don't consume here, let parse_ternary handle
                // If no, it's postfix try
                let is_ternary = if let Some(next) = self.peek_next() {
                    matches!(
                        next.token,
                        Token::Int(_)
                            | Token::Float(_)
                            | Token::String(_)
                            | Token::True
                            | Token::False
                            | Token::Ident(_)
                            | Token::LParen
                            | Token::LBracket
                            | Token::LBrace
                            | Token::If
                            | Token::Loop
                            | Token::Match
                            | Token::Pipe      // lambda
                            | Token::Move      // move lambda
                            | Token::Minus     // unary minus
                            | Token::Bang      // unary not
                            | Token::Tilde     // bitwise not
                            | Token::Amp       // reference
                            | Token::Star      // dereference
                            | Token::At        // self recursion
                            | Token::SelfLower // self
                    )
                } else {
                    false
                };

                if is_ternary {
                    // Let parse_ternary handle this
                    break;
                } else {
                    // Postfix try
                    self.advance_skip();
                    let end = self.prev_span().end;
                    expr = Spanned::new(Expr::Try(Box::new(expr)), Span::new(start, end));
                }
            } else if self.check(&Token::Bang) {
                // Check if this is a macro invocation: ident!(...)
                // Macro invocation requires: Ident followed by ! followed by (, [, or {
                let is_macro = if let Expr::Ident(_) = &expr.node {
                    if let Some(next) = self.peek_next() {
                        matches!(next.token, Token::LParen | Token::LBracket | Token::LBrace)
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_macro {
                    if let Expr::Ident(name) = &expr.node {
                        let name_owned = name.clone();
                        let sp = expr.span;
                        self.advance(); // consume !
                        let name_spanned = Spanned::new(name_owned, sp);
                        let invoke = self.parse_macro_invoke(name_spanned)?;
                        let end = self.prev_span().end;
                        expr = Spanned::new(Expr::MacroInvoke(invoke), Span::new(start, end));
                    }
                } else {
                    // Postfix unwrap: expr!
                    self.advance();
                    let end = self.prev_span().end;
                    expr = Spanned::new(Expr::Unwrap(Box::new(expr)), Span::new(start, end));
                }
            } else if self.check(&Token::As) {
                // Type cast: expr as Type
                self.advance_skip();
                let ty = self.parse_type()?;
                let end = self.prev_span().end;
                expr = Spanned::new(
                    Expr::Cast {
                        expr: Box::new(expr),
                        ty,
                    },
                    Span::new(start, end),
                );
            } else {
                break;
            }
        }

        Ok(expr)
    }
}

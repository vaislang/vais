//! Postfix operator parsing.
//!
//! Handles postfix expressions including function calls, field access, method calls,
//! indexing, await, try (?), unwrap (!), type casts (as), and static method calls (::).

use vais_ast::*;
use vais_lexer::Token;

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
                self.advance();
                let args = self.parse_args()?;
                self.expect(&Token::RParen)?;
                let end = self.prev_span().end;
                expr = Spanned::new(
                    Expr::Call {
                        func: Box::new(expr),
                        args,
                    },
                    Span::new(start, end),
                );
            } else if self.check(&Token::Dot) {
                self.advance();
                if self.check(&Token::Await) {
                    self.advance();
                    let end = self.prev_span().end;
                    expr = Spanned::new(Expr::Await(Box::new(expr)), Span::new(start, end));
                } else {
                    let field = self.parse_ident()?;
                    if self.check(&Token::LParen) {
                        self.advance();
                        let args = self.parse_args()?;
                        self.expect(&Token::RParen)?;
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
                // Static method call: Type::method(args)
                self.advance();
                let method = self.parse_ident()?;
                self.expect(&Token::LParen)?;
                let args = self.parse_args()?;
                self.expect(&Token::RParen)?;
                let end = self.prev_span().end;

                if let Expr::Ident(type_name) = &expr.node {
                    let tn = type_name.clone();
                    let sp = expr.span;
                    expr = Spanned::new(
                        Expr::StaticMethodCall {
                            type_name: Spanned::new(tn, sp),
                            method,
                            args,
                        },
                        Span::new(start, end),
                    );
                } else {
                    return Err(ParseError::UnexpectedToken {
                        found: Token::ColonColon,
                        span: self.prev_span(),
                        expected: "type name before '::'".into(),
                    });
                }
            } else if self.check(&Token::LBracket) {
                self.advance();
                let index = self.parse_expr()?;
                self.expect(&Token::RBracket)?;
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
                            | Token::Spawn
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
                    self.advance();
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
                self.advance();
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

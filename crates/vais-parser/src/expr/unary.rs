//! Unary operator parsing.
//!
//! Handles prefix unary operators: negation (-), logical not (!), bitwise not (~),
//! reference (&), dereference (*),
//! and prefix await (Y).

use vais_ast::*;
use vais_lexer::Token;

use crate::{ParseResult, Parser};

impl Parser {
    /// Parse unary expression
    pub(crate) fn parse_unary(&mut self) -> ParseResult<Spanned<Expr>> {
        self.enter_depth()?;
        let result = self.parse_unary_inner();
        self.exit_depth();
        result
    }

    fn parse_unary_inner(&mut self) -> ParseResult<Spanned<Expr>> {
        let start = self.current_span().start;

        if self.check(&Token::Minus) {
            self.advance_skip();
            let expr = self.parse_unary()?;
            let end = expr.span.end;
            return Ok(Spanned::new(
                Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                },
                Span::new(start, end),
            ));
        }

        if self.check(&Token::Bang) {
            self.advance_skip();
            let expr = self.parse_unary()?;
            let end = expr.span.end;
            return Ok(Spanned::new(
                Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                },
                Span::new(start, end),
            ));
        }

        if self.check(&Token::Tilde) {
            self.advance_skip();
            let expr = self.parse_unary()?;
            let end = expr.span.end;
            return Ok(Spanned::new(
                Expr::Unary {
                    op: UnaryOp::BitNot,
                    expr: Box::new(expr),
                },
                Span::new(start, end),
            ));
        }

        if self.check(&Token::Amp) {
            self.advance_skip();
            // Skip optional 'mut' keyword: &mut expr → Ref(expr)
            if self.check(&Token::Mut) {
                self.advance_skip();
            }
            let expr = self.parse_unary()?;
            let end = expr.span.end;
            // Bug C19 fix: `&x as T` should bind as `(&x) as T` (Rust-style:
            // unary `&` binds tighter than `as`). The recursive `parse_unary`
            // above falls through to `parse_postfix`, which greedily absorbs
            // any trailing `as` into a `Cast` — producing the wrong shape
            // `&(x as T)`. Detect that pattern and re-shape the AST so the
            // cast wraps the reference, matching user intent.
            if let Expr::Cast { expr: inner, ty } = expr.node {
                let inner_end = inner.span.end;
                let ref_expr = Spanned::new(
                    Expr::Ref(Box::new(*inner)),
                    Span::new(start, inner_end),
                );
                return Ok(Spanned::new(
                    Expr::Cast {
                        expr: Box::new(ref_expr),
                        ty,
                    },
                    Span::new(start, end),
                ));
            }
            return Ok(Spanned::new(
                Expr::Ref(Box::new(expr)),
                Span::new(start, end),
            ));
        }

        if self.check(&Token::Star) {
            self.advance_skip();
            let expr = self.parse_unary()?;
            let end = expr.span.end;
            return Ok(Spanned::new(
                Expr::Deref(Box::new(expr)),
                Span::new(start, end),
            ));
        }

        // Y expr - prefix await (equivalent to expr.await postfix form)
        if self.check(&Token::Await) {
            self.advance_skip();
            let expr = self.parse_unary()?;
            let end = expr.span.end;
            return Ok(Spanned::new(
                Expr::Await(Box::new(expr)),
                Span::new(start, end),
            ));
        }

        self.parse_postfix()
    }
}

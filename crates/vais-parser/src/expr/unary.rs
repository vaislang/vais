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

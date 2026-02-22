//! Unary operator parsing.
//!
//! Handles prefix unary operators: negation (-), logical not (!), bitwise not (~),
//! reference (&), dereference (*), lazy evaluation (lazy), and force evaluation (force).

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
            self.advance();
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
            self.advance();
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
            self.advance();
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
            self.advance();
            let expr = self.parse_unary()?;
            let end = expr.span.end;
            return Ok(Spanned::new(
                Expr::Ref(Box::new(expr)),
                Span::new(start, end),
            ));
        }

        if self.check(&Token::Star) {
            self.advance();
            let expr = self.parse_unary()?;
            let end = expr.span.end;
            return Ok(Spanned::new(
                Expr::Deref(Box::new(expr)),
                Span::new(start, end),
            ));
        }

        // lazy expr - deferred evaluation
        if self.check(&Token::Lazy) {
            self.advance();
            let expr = self.parse_unary()?;
            let end = expr.span.end;
            return Ok(Spanned::new(
                Expr::Lazy(Box::new(expr)),
                Span::new(start, end),
            ));
        }

        // force expr - force evaluation of lazy value
        if self.check(&Token::Force) {
            self.advance();
            let expr = self.parse_unary()?;
            let end = expr.span.end;
            return Ok(Spanned::new(
                Expr::Force(Box::new(expr)),
                Span::new(start, end),
            ));
        }

        self.parse_postfix()
    }
}

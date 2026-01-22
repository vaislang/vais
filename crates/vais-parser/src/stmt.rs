//! Statement parsing for Vais language.
//!
//! Handles parsing of statements including variable declarations,
//! control flow statements (return, break, continue), and expressions as statements.

use vais_ast::*;
use vais_lexer::Token;

use crate::{ParseError, ParseResult, Parser};

impl Parser {
    /// Parse block contents (statements)
    pub(crate) fn parse_block_contents(&mut self) -> ParseResult<Vec<Spanned<Stmt>>> {
        let mut stmts = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }

        Ok(stmts)
    }

    /// Parse statement
    pub(crate) fn parse_stmt(&mut self) -> ParseResult<Spanned<Stmt>> {
        let start = self.current_span().start;

        let stmt = if self.check(&Token::Return) {
            self.advance();
            let expr = if !self.check(&Token::RBrace) && !self.check(&Token::Semi) {
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            };
            Stmt::Return(expr)
        } else if self.check(&Token::Break) {
            self.advance();
            let expr = if !self.check(&Token::RBrace) && !self.check(&Token::Semi) {
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            };
            Stmt::Break(expr)
        } else if self.check(&Token::Continue) {
            self.advance();
            Stmt::Continue
        } else if self.check(&Token::Defer) {
            self.advance();
            let expr = Box::new(self.parse_expr()?);
            Stmt::Defer(expr)
        } else if self.is_let_stmt() {
            self.parse_let_stmt()?
        } else {
            Stmt::Expr(Box::new(self.parse_expr()?))
        };

        // Optional semicolon
        if self.check(&Token::Semi) {
            self.advance();
        }

        let end = self.prev_span().end;
        Ok(Spanned::new(stmt, Span::new(start, end)))
    }

    /// Check if current position is a let statement
    fn is_let_stmt(&self) -> bool {
        if let Some(tok) = self.peek() {
            if let Token::Ident(_) = &tok.token {
                if let Some(next) = self.tokens.get(self.pos + 1) {
                    matches!(next.token, Token::ColonEq | Token::Colon)
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Parse let statement: `x := expr` or `x: T = expr`
    fn parse_let_stmt(&mut self) -> ParseResult<Stmt> {
        let name = self.parse_ident()?;

        let (ty, is_mut) = if self.check(&Token::ColonEq) {
            self.advance();
            // Check for mut: `x := mut expr`
            let is_mut = self.check(&Token::Mut);
            if is_mut {
                self.advance();
            }
            (None, is_mut)
        } else if self.check(&Token::Colon) {
            self.advance();
            // Check for mut: `x: mut T = expr`
            let is_mut = self.check(&Token::Mut);
            if is_mut {
                self.advance();
            }
            let ty = self.parse_type()?;
            self.expect(&Token::Eq)?;
            (Some(ty), is_mut)
        } else {
            return Err(ParseError::UnexpectedToken {
                found: self.peek().map(|t| t.token.clone()).unwrap_or(Token::Ident("EOF".into())),
                span: self.current_span(),
                expected: ":= or :".into(),
            });
        };

        let value = self.parse_expr()?;

        Ok(Stmt::Let {
            name,
            ty,
            value: Box::new(value),
            is_mut,
        })
    }
}

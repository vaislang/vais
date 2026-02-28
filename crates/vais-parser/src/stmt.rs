//! Statement parsing for Vais language.
//!
//! Handles parsing of statements including variable declarations,
//! control flow statements (return, break, continue), and expressions as statements.
//! Supports error recovery to continue parsing after syntax errors.

use vais_ast::*;
use vais_lexer::Token;

use crate::{ParseError, ParseResult, Parser};

impl Parser {
    /// Parse block contents (statements) with error recovery support.
    ///
    /// In recovery mode, errors are collected and Error nodes are inserted
    /// into the AST. Parsing continues to find as many errors as possible.
    pub(crate) fn parse_block_contents(&mut self) -> ParseResult<Vec<Spanned<Stmt>>> {
        self.enter_depth()?;
        let mut stmts = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Check if we've hit an item-level keyword - this means we're likely
            // missing a closing brace and have escaped the function body
            if let Some(tok) = self.peek() {
                match &tok.token {
                    Token::Function
                    | Token::Struct
                    | Token::Enum
                    | Token::Union
                    | Token::Use
                    | Token::Trait
                    | Token::Impl
                    | Token::Macro
                    | Token::Pub
                    | Token::Async
                    | Token::Extern => {
                        // We've hit a top-level item keyword - stop parsing block contents
                        break;
                    }
                    _ => {}
                }
            }

            match self.parse_stmt() {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => {
                    if self.recovery_mode {
                        // Record the error and create an Error node
                        let start = self.current_span().start;
                        let message = e.to_string();
                        self.record_error(e);

                        // Synchronize to next statement boundary
                        let skipped_tokens = self.synchronize_statement();

                        let end = self.prev_span().end;
                        stmts.push(Spanned::new(
                            Stmt::Error {
                                message,
                                skipped_tokens,
                            },
                            Span::new(start, end),
                        ));
                    } else {
                        // Not in recovery mode, propagate the error
                        return Err(e);
                    }
                }
            }
        }

        self.exit_depth();
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
            } else if matches!(tok.token, Token::Tilde) {
                // Check for ~ ident := expr or ~ ident = expr (mutable shorthand prefix)
                if let Some(next) = self.tokens.get(self.pos + 1) {
                    if let Token::Ident(_) = &next.token {
                        self.tokens
                            .get(self.pos + 2)
                            .map(|t| matches!(t.token, Token::ColonEq | Token::Colon | Token::Eq))
                            .unwrap_or(false)
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else if matches!(tok.token, Token::LParen) {
                // Check for tuple destructuring: (a, b) := expr
                // Lookahead past matching parens to find :=
                self.is_tuple_destructure()
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Check if current position starts a tuple destructuring pattern
    /// Looks for `(...)` followed by `:=`
    fn is_tuple_destructure(&self) -> bool {
        let mut depth = 0;
        let mut i = self.pos;
        while i < self.tokens.len() {
            match &self.tokens[i].token {
                Token::LParen => depth += 1,
                Token::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        // Check if next token is :=
                        return self
                            .tokens
                            .get(i + 1)
                            .map(|t| matches!(t.token, Token::ColonEq))
                            .unwrap_or(false);
                    }
                }
                _ => {}
            }
            i += 1;
        }
        false
    }

    /// Parse let statement: `x := expr` or `x: T = expr` or `(a, b) := expr`
    /// Also supports ownership modifiers: `x := linear expr` or `x: affine T = expr`
    fn parse_let_stmt(&mut self) -> ParseResult<Stmt> {
        // Check for ~ prefix (mutable shorthand): ~ x := expr
        let tilde_prefix = self.check(&Token::Tilde);
        if tilde_prefix {
            self.advance();
        }

        // Check for tuple destructuring: (a, b) := expr
        if self.check(&Token::LParen) {
            let pattern = self.parse_pattern()?;
            self.expect(&Token::ColonEq)?;
            let is_mut = self.check(&Token::Mut) || self.check(&Token::Tilde);
            if is_mut {
                self.advance();
            }
            let value = self.parse_expr()?;
            return Ok(Stmt::LetDestructure {
                pattern,
                value: Box::new(value),
                is_mut,
            });
        }

        let name = self.parse_ident()?;

        let (ty, is_mut, ownership) = if self.check(&Token::ColonEq) {
            self.advance();
            // Check for ownership modifiers: `x := linear expr`, `x := affine expr`, `x := move expr`
            // BUT: if `move` is followed by `|`, it's a move lambda, not an ownership modifier
            let ownership = if self.check(&Token::Linear) {
                self.advance();
                Ownership::Linear
            } else if self.check(&Token::Affine) {
                self.advance();
                Ownership::Affine
            } else if self.check(&Token::Move) {
                // Peek ahead: if next token is |, this is a move lambda, not ownership
                if let Some(next) = self.peek_next() {
                    if next.token == Token::Pipe {
                        // Don't consume move - let the expression parser handle it
                        Ownership::Regular
                    } else {
                        self.advance();
                        Ownership::Move
                    }
                } else {
                    self.advance();
                    Ownership::Move
                }
            } else {
                Ownership::Regular
            };
            // Check for mut: `x := mut expr` or `x := ~ expr`
            let is_mut = tilde_prefix || self.check(&Token::Mut) || self.check(&Token::Tilde);
            if !tilde_prefix && is_mut {
                self.advance();
            }
            (None, is_mut, ownership)
        } else if tilde_prefix && self.check(&Token::Eq) {
            // ~x = expr  →  shorthand for mutable let binding (same as x := mut expr)
            self.advance();
            (None, true, Ownership::Regular)
        } else if self.check(&Token::Colon) {
            self.advance();
            // Check for ownership modifiers: `x: linear T = expr`, `x: affine T = expr`
            // BUT: if `move` is followed by `|`, it's a move lambda, not an ownership modifier
            let ownership = if self.check(&Token::Linear) {
                self.advance();
                Ownership::Linear
            } else if self.check(&Token::Affine) {
                self.advance();
                Ownership::Affine
            } else if self.check(&Token::Move) {
                // Peek ahead: if next token is |, this is a move lambda, not ownership
                if let Some(next) = self.peek_next() {
                    if next.token == Token::Pipe {
                        // Don't consume move - let the expression parser handle it
                        Ownership::Regular
                    } else {
                        self.advance();
                        Ownership::Move
                    }
                } else {
                    self.advance();
                    Ownership::Move
                }
            } else {
                Ownership::Regular
            };
            // Check for mut: `x: mut T = expr` or `x: ~ T = expr`
            let is_mut = tilde_prefix || self.check(&Token::Mut) || self.check(&Token::Tilde);
            if !tilde_prefix && is_mut {
                self.advance();
            }
            let ty = self.parse_type()?;
            self.expect(&Token::Eq)?;
            (Some(ty), is_mut, ownership)
        } else {
            return Err(ParseError::UnexpectedToken {
                found: self
                    .peek()
                    .map(|t| t.token.clone())
                    .unwrap_or(Token::Ident("EOF".into())),
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
            ownership,
        })
    }

}

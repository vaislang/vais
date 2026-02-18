//! Precedence-based binary operator parsing.
//!
//! Implements the precedence climbing algorithm for binary operators:
//! assignment < pipe < ternary < or < and < bitwise_or < bitwise_xor < bitwise_and
//! < equality < range < comparison < shift < term < factor

use vais_ast::*;
use vais_lexer::Token;

use crate::{ParseResult, Parser};

impl Parser {
    /// Parse assignment expression
    pub(crate) fn parse_assignment(&mut self) -> ParseResult<Spanned<Expr>> {
        let expr = self.parse_pipe()?;

        if self.check(&Token::Eq) {
            self.advance();
            let value = self.parse_assignment()?;
            let span = expr.span.merge(value.span);
            return Ok(Spanned::new(
                Expr::Assign {
                    target: Box::new(expr),
                    value: Box::new(value),
                },
                span,
            ));
        }

        // Compound assignment
        let op = if self.check(&Token::PlusEq) {
            Some(BinOp::Add)
        } else if self.check(&Token::MinusEq) {
            Some(BinOp::Sub)
        } else if self.check(&Token::StarEq) {
            Some(BinOp::Mul)
        } else if self.check(&Token::SlashEq) {
            Some(BinOp::Div)
        } else {
            None
        };

        if let Some(op) = op {
            self.advance();
            let value = self.parse_assignment()?;
            let span = expr.span.merge(value.span);
            return Ok(Spanned::new(
                Expr::AssignOp {
                    op,
                    target: Box::new(expr),
                    value: Box::new(value),
                },
                span,
            ));
        }

        Ok(expr)
    }

    /// Parse pipe operator: `x |> f |> g` → `g(f(x))`
    /// Left-associative, between assignment and ternary in precedence.
    pub(crate) fn parse_pipe(&mut self) -> ParseResult<Spanned<Expr>> {
        let mut left = self.parse_ternary()?;

        while self.check(&Token::PipeArrow) {
            self.advance();
            let right = self.parse_ternary()?;
            let span = left.span.merge(right.span);
            // Transform: left |> right into right(left)
            left = Spanned::new(
                Expr::Call {
                    func: Box::new(right),
                    args: vec![left],
                },
                span,
            );
        }

        Ok(left)
    }

    /// Parse ternary: `cond ? then : else`
    /// Distinguishes from postfix try (expr?) by checking if ? is followed by
    /// an expression start token. If not, it's postfix try.
    pub(crate) fn parse_ternary(&mut self) -> ParseResult<Spanned<Expr>> {
        let cond = self.parse_or()?;

        if self.check(&Token::Question) {
            // Check if the token after ? can start an expression
            // If not, this ? is postfix try, not ternary
            if let Some(next) = self.peek_next() {
                let can_start_expr = matches!(
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
                );

                if can_start_expr {
                    // This looks like ternary, proceed
                    self.advance();
                    let then = self.parse_ternary()?;
                    self.expect(&Token::Colon)?;
                    let else_ = self.parse_ternary()?;

                    let span = cond.span.merge(else_.span);
                    return Ok(Spanned::new(
                        Expr::Ternary {
                            cond: Box::new(cond),
                            then: Box::new(then),
                            else_: Box::new(else_),
                        },
                        span,
                    ));
                }
                // else: not an expression start, so it's postfix try
                // Don't consume ?, let parse_postfix_try handle it
            }
            // No next token after ? means it's postfix try at end of input
            // Don't consume ?, let parse_postfix_try handle it
        }

        Ok(cond)
    }

    /// Parse logical OR
    pub(crate) fn parse_or(&mut self) -> ParseResult<Spanned<Expr>> {
        let mut left = self.parse_and()?;

        while self.check(&Token::Pipe) && self.peek_next().map(|t| &t.token) == Some(&Token::Pipe) {
            self.advance();
            self.advance();
            let right = self.parse_and()?;
            let span = left.span.merge(right.span);
            left = Spanned::new(
                Expr::Binary {
                    op: BinOp::Or,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    /// Parse logical AND
    pub(crate) fn parse_and(&mut self) -> ParseResult<Spanned<Expr>> {
        let mut left = self.parse_bitwise_or()?;

        while self.check(&Token::Amp) && self.peek_next().map(|t| &t.token) == Some(&Token::Amp) {
            self.advance();
            self.advance();
            let right = self.parse_bitwise_or()?;
            let span = left.span.merge(right.span);
            left = Spanned::new(
                Expr::Binary {
                    op: BinOp::And,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    /// Parse bitwise OR
    pub(crate) fn parse_bitwise_or(&mut self) -> ParseResult<Spanned<Expr>> {
        let mut left = self.parse_bitwise_xor()?;

        while self.check(&Token::Pipe) && self.peek_next().map(|t| &t.token) != Some(&Token::Pipe) {
            self.advance();
            let right = self.parse_bitwise_xor()?;
            let span = left.span.merge(right.span);
            left = Spanned::new(
                Expr::Binary {
                    op: BinOp::BitOr,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    /// Parse bitwise XOR
    pub(crate) fn parse_bitwise_xor(&mut self) -> ParseResult<Spanned<Expr>> {
        let mut left = self.parse_bitwise_and()?;

        while self.check(&Token::Caret) {
            self.advance();
            let right = self.parse_bitwise_and()?;
            let span = left.span.merge(right.span);
            left = Spanned::new(
                Expr::Binary {
                    op: BinOp::BitXor,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    /// Parse bitwise AND
    pub(crate) fn parse_bitwise_and(&mut self) -> ParseResult<Spanned<Expr>> {
        let mut left = self.parse_equality()?;

        while self.check(&Token::Amp) && self.peek_next().map(|t| &t.token) != Some(&Token::Amp) {
            self.advance();
            let right = self.parse_equality()?;
            let span = left.span.merge(right.span);
            left = Spanned::new(
                Expr::Binary {
                    op: BinOp::BitAnd,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    /// Parse equality
    pub(crate) fn parse_equality(&mut self) -> ParseResult<Spanned<Expr>> {
        let mut left = self.parse_range()?;

        loop {
            let op = if self.check(&Token::EqEq) {
                Some(BinOp::Eq)
            } else if self.check(&Token::Neq) {
                Some(BinOp::Neq)
            } else {
                None
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_range()?;
                let span = left.span.merge(right.span);
                left = Spanned::new(
                    Expr::Binary {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    span,
                );
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Parse range expression: `start..end` or `start..=end`
    pub(crate) fn parse_range(&mut self) -> ParseResult<Spanned<Expr>> {
        let start_span = self.current_span().start;

        // Check for prefix range (..end or ..=end)
        if self.check(&Token::DotDot) || self.check(&Token::DotDotEq) {
            let inclusive = self.check(&Token::DotDotEq);
            self.advance();
            let end = self.parse_comparison()?;
            let end_span = self.prev_span().end;
            return Ok(Spanned::new(
                Expr::Range {
                    start: None,
                    end: Some(Box::new(end)),
                    inclusive,
                },
                Span::new(start_span, end_span),
            ));
        }

        let left = self.parse_comparison()?;

        // Check for range operator
        if self.check(&Token::DotDot) || self.check(&Token::DotDotEq) {
            let inclusive = self.check(&Token::DotDotEq);
            self.advance();

            // Check if there's an end expression (not at end of context like ] or ))
            if !self.is_at_end()
                && !self.check(&Token::RBracket)
                && !self.check(&Token::RParen)
                && !self.check(&Token::Comma)
                && !self.check(&Token::RBrace)
            {
                let end = self.parse_comparison()?;
                let end_span = self.prev_span().end;
                return Ok(Spanned::new(
                    Expr::Range {
                        start: Some(Box::new(left)),
                        end: Some(Box::new(end)),
                        inclusive,
                    },
                    Span::new(start_span, end_span),
                ));
            } else {
                // start.. (no end)
                let end_span = self.prev_span().end;
                return Ok(Spanned::new(
                    Expr::Range {
                        start: Some(Box::new(left)),
                        end: None,
                        inclusive,
                    },
                    Span::new(start_span, end_span),
                ));
            }
        }

        Ok(left)
    }

    /// Parse comparison
    pub(crate) fn parse_comparison(&mut self) -> ParseResult<Spanned<Expr>> {
        let mut left = self.parse_shift()?;

        loop {
            let op = if self.check(&Token::Lt) {
                Some(BinOp::Lt)
            } else if self.check(&Token::Lte) {
                Some(BinOp::Lte)
            } else if self.check(&Token::Gt) {
                Some(BinOp::Gt)
            } else if self.check(&Token::Gte) {
                Some(BinOp::Gte)
            } else {
                None
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_shift()?;
                let span = left.span.merge(right.span);
                left = Spanned::new(
                    Expr::Binary {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    span,
                );
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Parse shift
    pub(crate) fn parse_shift(&mut self) -> ParseResult<Spanned<Expr>> {
        let mut left = self.parse_term()?;

        loop {
            let op = if self.check(&Token::Shl) {
                Some(BinOp::Shl)
            } else if self.check(&Token::Shr) {
                Some(BinOp::Shr)
            } else {
                None
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_term()?;
                let span = left.span.merge(right.span);
                left = Spanned::new(
                    Expr::Binary {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    span,
                );
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Parse term (+ -)
    pub(crate) fn parse_term(&mut self) -> ParseResult<Spanned<Expr>> {
        let mut left = self.parse_factor()?;

        loop {
            // Don't treat +/- as binary operators across newlines
            // This prevents `puts("hello")\n-1` from being parsed as `puts("hello") - 1`
            let op_on_new_line = if let Some(tok) = self.peek() {
                self.has_newline_between(left.span.end, tok.span.start)
            } else {
                false
            };
            if op_on_new_line {
                break;
            }

            let op = if self.check(&Token::Plus) {
                Some(BinOp::Add)
            } else if self.check(&Token::Minus) {
                Some(BinOp::Sub)
            } else {
                None
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_factor()?;
                let span = left.span.merge(right.span);
                left = Spanned::new(
                    Expr::Binary {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    span,
                );
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Parse factor (* / %)
    pub(crate) fn parse_factor(&mut self) -> ParseResult<Spanned<Expr>> {
        let mut left = self.parse_unary()?;

        loop {
            let op = if self.check(&Token::Star) {
                Some(BinOp::Mul)
            } else if self.check(&Token::Slash) {
                Some(BinOp::Div)
            } else if self.check(&Token::Percent) {
                Some(BinOp::Mod)
            } else {
                None
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_unary()?;
                let span = left.span.merge(right.span);
                left = Spanned::new(
                    Expr::Binary {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    span,
                );
            } else {
                break;
            }
        }

        Ok(left)
    }
}

//! Expression parsing for Vais language.
//!
//! Implements recursive descent parsing for all expression types including:
//! - Binary operations (arithmetic, logical, bitwise, comparison)
//! - Unary operations (negation, not, bitwise not, reference, dereference)
//! - Control flow expressions (if, loop, match)
//! - Literals, identifiers, function calls, method calls
//! - Lambda expressions, pattern matching
//! - Array and struct literals

use vais_ast::*;
use vais_lexer::Token;

use crate::{ParseError, ParseResult, Parser};

/// Check if a string contains interpolation syntax: `{<non-empty>}`.
/// Empty `{}` is NOT interpolation (backward compat with format strings).
/// `{{` is an escaped brace, not interpolation.
fn has_interpolation(s: &str) -> bool {
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                // Escaped {{ - skip
                chars.next();
            } else if chars.peek() != Some(&'}') {
                // Non-empty content inside braces - this is interpolation
                // Verify there's a closing brace
                let mut depth = 1;
                for c in chars.by_ref() {
                    if c == '{' {
                        depth += 1;
                    } else if c == '}' {
                        depth -= 1;
                        if depth == 0 {
                            return true;
                        }
                    }
                }
            } else {
                // Empty {} - skip
                chars.next();
            }
        } else if ch == '}' && chars.peek() == Some(&'}') {
            // Escaped }}
            chars.next();
        }
    }
    false
}

impl Parser {
    /// Parse expression
    pub fn parse_expr(&mut self) -> ParseResult<Spanned<Expr>> {
        self.enter_depth()?;
        let result = self.parse_assignment();
        self.exit_depth();
        result
    }

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

    /// Parse unary expression
    pub(crate) fn parse_unary(&mut self) -> ParseResult<Spanned<Expr>> {
        self.enter_depth()?;
        let start = self.current_span().start;

        if self.check(&Token::Minus) {
            self.advance();
            let expr = self.parse_unary()?;
            let end = expr.span.end;
            self.exit_depth();
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
            self.exit_depth();
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
            self.exit_depth();
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
            self.exit_depth();
            return Ok(Spanned::new(Expr::Ref(Box::new(expr)), Span::new(start, end)));
        }

        if self.check(&Token::Star) {
            self.advance();
            let expr = self.parse_unary()?;
            let end = expr.span.end;
            self.exit_depth();
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
            self.exit_depth();
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
            self.exit_depth();
            return Ok(Spanned::new(
                Expr::Force(Box::new(expr)),
                Span::new(start, end),
            ));
        }

        let result = self.parse_postfix();
        self.exit_depth();
        result
    }

    /// Parse postfix expressions (calls, field access, etc.)
    pub(crate) fn parse_postfix(&mut self) -> ParseResult<Spanned<Expr>> {
        let mut expr = self.parse_primary()?;

        loop {
            let start = expr.span.start;

            if self.check(&Token::LParen) {
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
                    expr = Spanned::new(
                        Expr::Await(Box::new(expr)),
                        Span::new(start, end),
                    );
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
                            name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                        } else {
                            false
                        };

                        if is_static {
                            if let Expr::Ident(type_name) = expr.node.clone() {
                                expr = Spanned::new(
                                    Expr::StaticMethodCall {
                                        type_name: Spanned::new(type_name, expr.span),
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

                if let Expr::Ident(type_name) = expr.node.clone() {
                    expr = Spanned::new(
                        Expr::StaticMethodCall {
                            type_name: Spanned::new(type_name, expr.span),
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
                    if let Expr::Ident(name) = expr.node.clone() {
                        self.advance(); // consume !
                        let name_spanned = Spanned::new(name, expr.span);
                        let invoke = self.parse_macro_invoke(name_spanned)?;
                        let end = self.prev_span().end;
                        expr = Spanned::new(
                            Expr::MacroInvoke(invoke),
                            Span::new(start, end),
                        );
                    }
                } else {
                    // Postfix unwrap: expr!
                    self.advance();
                    let end = self.prev_span().end;
                    expr = Spanned::new(
                        Expr::Unwrap(Box::new(expr)),
                        Span::new(start, end),
                    );
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

    /// Parse primary expression
    pub(crate) fn parse_primary(&mut self) -> ParseResult<Spanned<Expr>> {
        self.enter_depth()?;
        let result = self.parse_primary_inner();
        self.exit_depth();
        result
    }

    /// Parse a string with interpolation into `Expr::StringInterp`.
    /// Scans the string char-by-char, splitting into literal and expression parts.
    /// `{{`/`}}` are escaped to literal `{`/`}`.
    /// `{expr}` is sub-lexed and sub-parsed.
    fn parse_string_interpolation(&self, s: &str) -> ParseResult<Expr> {
        let mut parts: Vec<StringInterpPart> = Vec::new();
        let mut literal = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                if chars.peek() == Some(&'{') {
                    // Escaped {{ -> literal {
                    chars.next();
                    literal.push('{');
                } else if chars.peek() == Some(&'}') {
                    // Empty {} -> literal {} (backward compat)
                    chars.next();
                    literal.push('{');
                    literal.push('}');
                } else {
                    // Interpolated expression
                    if !literal.is_empty() {
                        parts.push(StringInterpPart::Lit(std::mem::take(&mut literal)));
                    }
                    // Collect expression text until matching }
                    let mut expr_text = String::new();
                    let mut depth = 1;
                    for c in chars.by_ref() {
                        if c == '{' {
                            depth += 1;
                            expr_text.push(c);
                        } else if c == '}' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                            expr_text.push(c);
                        } else {
                            expr_text.push(c);
                        }
                    }
                    // Sub-lex and sub-parse the expression
                    let tokens = vais_lexer::tokenize(&expr_text).map_err(|_| {
                        ParseError::InvalidExpression
                    })?;
                    let mut sub_parser = crate::Parser::new(tokens);
                    let expr = sub_parser.parse_expr()?;
                    parts.push(StringInterpPart::Expr(Box::new(expr)));
                }
            } else if ch == '}' {
                if chars.peek() == Some(&'}') {
                    // Escaped }} -> literal }
                    chars.next();
                    literal.push('}');
                } else {
                    literal.push(ch);
                }
            } else {
                literal.push(ch);
            }
        }

        // Flush remaining literal
        if !literal.is_empty() {
            parts.push(StringInterpPart::Lit(literal));
        }

        Ok(Expr::StringInterp(parts))
    }

    fn parse_primary_inner(&mut self) -> ParseResult<Spanned<Expr>> {
        let start = self.current_span().start;
        let span = self.current_span();
        let tok = self.advance().ok_or(ParseError::UnexpectedEof { span })?;

        let expr = match tok.token {
            Token::Int(n) => Expr::Int(n),
            Token::Float(n) => Expr::Float(n),
            Token::True => Expr::Bool(true),
            Token::False => Expr::Bool(false),
            Token::String(s) => {
                if has_interpolation(&s) {
                    self.parse_string_interpolation(&s)?
                } else {
                    Expr::String(s)
                }
            }
            Token::At => Expr::SelfCall,
            Token::SelfLower => Expr::Ident("self".to_string()),
            Token::Ident(name) => {
                // Handle contract verification builtins: old(), assert(), assume()
                if name == "old" && self.check(&Token::LParen) {
                    self.advance(); // consume (
                    let inner = self.parse_expr()?;
                    self.expect(&Token::RParen)?;
                    let end = self.prev_span().end;
                    return Ok(Spanned::new(
                        Expr::Old(Box::new(inner)),
                        Span::new(start, end),
                    ));
                }
                if name == "assert" && self.check(&Token::LParen) {
                    self.advance(); // consume (
                    let condition = self.parse_expr()?;
                    let message = if self.check(&Token::Comma) {
                        self.advance();
                        Some(Box::new(self.parse_expr()?))
                    } else {
                        None
                    };
                    self.expect(&Token::RParen)?;
                    let end = self.prev_span().end;
                    return Ok(Spanned::new(
                        Expr::Assert { condition: Box::new(condition), message },
                        Span::new(start, end),
                    ));
                }
                if name == "assume" && self.check(&Token::LParen) {
                    self.advance(); // consume (
                    let inner = self.parse_expr()?;
                    self.expect(&Token::RParen)?;
                    let end = self.prev_span().end;
                    return Ok(Spanned::new(
                        Expr::Assume(Box::new(inner)),
                        Span::new(start, end),
                    ));
                }
                // Check for struct literal: `Name{...}`
                // Only treat as struct literal if:
                // 1. name starts with uppercase (type convention)
                // 2. struct literals are allowed in current context (not in loop/if conditions)
                let is_type_name = name.chars().next().is_some_and(|c| c.is_uppercase());
                if is_type_name && self.check(&Token::LBrace) && self.allow_struct_literal {
                    self.advance();
                    let mut fields = Vec::new();
                    while !self.check(&Token::RBrace) && !self.is_at_end() {
                        let field_name = self.parse_ident()?;
                        self.expect(&Token::Colon)?;
                        let value = self.parse_expr()?;
                        fields.push((field_name, value));
                        if !self.check(&Token::RBrace) {
                            self.expect(&Token::Comma)?;
                        }
                    }
                    self.expect(&Token::RBrace)?;
                    let end = self.prev_span().end;
                    let name_len = name.len();
                    return Ok(Spanned::new(
                        Expr::StructLit {
                            name: Spanned::new(name, Span::new(start, start + name_len)),
                            fields,
                        },
                        Span::new(start, end),
                    ));
                }
                Expr::Ident(name)
            }
            Token::LParen => {
                if self.check(&Token::RParen) {
                    self.advance();
                    Expr::Unit
                } else {
                    let expr = self.parse_expr()?;
                    if self.check(&Token::Comma) {
                        let mut exprs = vec![expr];
                        while self.check(&Token::Comma) {
                            self.advance();
                            if self.check(&Token::RParen) {
                                break;
                            }
                            exprs.push(self.parse_expr()?);
                        }
                        self.expect(&Token::RParen)?;
                        let end = self.prev_span().end;
                        return Ok(Spanned::new(Expr::Tuple(exprs), Span::new(start, end)));
                    }
                    self.expect(&Token::RParen)?;
                    return Ok(expr);
                }
            }
            Token::LBracket => {
                let mut exprs = Vec::new();
                while !self.check(&Token::RBracket) && !self.is_at_end() {
                    // Check for spread syntax: ..expr
                    if self.check(&Token::DotDot) {
                        let spread_start = self.current_span().start;
                        self.advance();
                        let inner = self.parse_expr()?;
                        let spread_end = self.prev_span().end;
                        exprs.push(Spanned::new(
                            Expr::Spread(Box::new(inner)),
                            Span::new(spread_start, spread_end),
                        ));
                    } else {
                        exprs.push(self.parse_expr()?);
                    }
                    if !self.check(&Token::RBracket) {
                        self.expect(&Token::Comma)?;
                    }
                }
                self.expect(&Token::RBracket)?;
                let end = self.prev_span().end;
                return Ok(Spanned::new(Expr::Array(exprs), Span::new(start, end)));
            }
            Token::LBrace => {
                // Disambiguate map literal {k: v, ...} from block {stmts}
                // Try to detect map literal pattern: save pos, parse expr, check for ':'
                if !self.check(&Token::RBrace) && !self.is_at_end() {
                    let saved_pos = self.pos;
                    let saved_errors_len = self.errors.len();
                    // Try parsing an expression as potential map key
                    let maybe_key = self.parse_expr();
                    if let Ok(key) = maybe_key {
                        if self.check(&Token::Colon) {
                            // This looks like a map literal: {key: value, ...}
                            self.advance(); // consume ':'
                            let value = self.parse_expr()?;
                            let mut pairs = vec![(key, value)];
                            while self.check(&Token::Comma) {
                                self.advance(); // consume ','
                                if self.check(&Token::RBrace) {
                                    break; // trailing comma
                                }
                                let k = self.parse_expr()?;
                                self.expect(&Token::Colon)?;
                                let v = self.parse_expr()?;
                                pairs.push((k, v));
                            }
                            self.expect(&Token::RBrace)?;
                            let end = self.prev_span().end;
                            return Ok(Spanned::new(Expr::MapLit(pairs), Span::new(start, end)));
                        } else {
                            // Not a map literal, restore position and parse as block
                            self.pos = saved_pos;
                            self.errors.truncate(saved_errors_len);
                        }
                    } else {
                        // Not a map literal, restore position and parse as block
                        self.pos = saved_pos;
                        self.errors.truncate(saved_errors_len);
                    }
                }
                let stmts = self.parse_block_contents()?;
                self.expect(&Token::RBrace)?;
                let end = self.prev_span().end;
                return Ok(Spanned::new(Expr::Block(stmts), Span::new(start, end)));
            }
            Token::If => {
                return self.parse_if_expr(start);
            }
            Token::Loop => {
                return self.parse_loop_expr(start);
            }
            Token::Match => {
                return self.parse_match_expr(start);
            }
            Token::Spawn => {
                // Spawn can be: spawn { expr } or spawn expr
                let body = if self.check(&Token::LBrace) {
                    // spawn { expr }
                    self.expect(&Token::LBrace)?;
                    let body = self.parse_expr()?;
                    self.expect(&Token::RBrace)?;
                    body
                } else {
                    // spawn expr (e.g., spawn async_func(args))
                    self.parse_unary()?
                };
                let end = self.prev_span().end;
                return Ok(Spanned::new(
                    Expr::Spawn(Box::new(body)),
                    Span::new(start, end),
                ));
            }
            Token::Pipe => {
                // Lambda expression: |params| body
                return self.parse_lambda(start);
            }
            Token::Comptime => {
                // Comptime expression: comptime { expr }
                self.expect(&Token::LBrace)?;
                let body = self.parse_expr()?;
                self.expect(&Token::RBrace)?;
                let end = self.prev_span().end;
                return Ok(Spanned::new(
                    Expr::Comptime {
                        body: Box::new(body),
                    },
                    Span::new(start, end),
                ));
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    found: tok.token,
                    span: tok.span,
                    expected: "expression".into(),
                });
            }
        };

        let end = self.prev_span().end;
        Ok(Spanned::new(expr, Span::new(start, end)))
    }

    /// Parse if expression: `I cond{...}E{...}`
    pub(crate) fn parse_if_expr(&mut self, start: usize) -> ParseResult<Spanned<Expr>> {
        // Disable struct literals in condition to avoid ambiguity with block start
        let old_allow_struct_literal = self.allow_struct_literal;
        self.allow_struct_literal = false;
        let cond = self.parse_expr()?;
        self.allow_struct_literal = old_allow_struct_literal;

        self.expect(&Token::LBrace)?;
        let then = self.parse_block_contents()?;
        self.expect(&Token::RBrace)?;

        let else_ = if self.check(&Token::Enum) {
            // E is used for else (context-dependent)
            self.advance();
            Some(self.parse_else_branch()?)
        } else {
            None
        };

        let end = self.prev_span().end;
        Ok(Spanned::new(
            Expr::If {
                cond: Box::new(cond),
                then,
                else_,
            },
            Span::new(start, end),
        ))
    }

    /// Parse else branch
    pub(crate) fn parse_else_branch(&mut self) -> ParseResult<IfElse> {
        self.enter_depth()?;
        if self.check(&Token::If) {
            // else if
            self.advance();
            // Disable struct literals in condition to avoid ambiguity with block start
            let old_allow_struct_literal = self.allow_struct_literal;
            self.allow_struct_literal = false;
            let cond = self.parse_expr()?;
            self.allow_struct_literal = old_allow_struct_literal;

            self.expect(&Token::LBrace)?;
            let then = self.parse_block_contents()?;
            self.expect(&Token::RBrace)?;

            let else_ = if self.check(&Token::Enum) {
                self.advance();
                Some(Box::new(self.parse_else_branch()?))
            } else {
                None
            };

            self.exit_depth();
            Ok(IfElse::ElseIf(Box::new(cond), then, else_))
        } else {
            // else
            self.expect(&Token::LBrace)?;
            let stmts = self.parse_block_contents()?;
            self.expect(&Token::RBrace)?;
            self.exit_depth();
            Ok(IfElse::Else(stmts))
        }
    }

    /// Parse loop expression: `L pattern:iter{...}` or `L{...}` or `L condition{...}`
    /// Three forms:
    /// - `L { }` - infinite loop
    /// - `L pattern:iter { }` - for-each loop (pattern iterates over iter)
    /// - `L condition { }` - while loop (loop while condition is true)
    pub(crate) fn parse_loop_expr(&mut self, start: usize) -> ParseResult<Spanned<Expr>> {
        if self.check(&Token::LBrace) {
            // Infinite loop: `L { ... }`
            self.advance(); // consume '{'
            let body = self.parse_block_contents()?;
            self.expect(&Token::RBrace)?;

            let end = self.prev_span().end;
            return Ok(Spanned::new(
                Expr::Loop {
                    pattern: None,
                    iter: None,
                    body,
                },
                Span::new(start, end),
            ));
        }

        // Try to parse as for-each loop first by checking for pattern:iter syntax
        // We need to look ahead to determine if this is for-each or while loop
        let saved_pos = self.pos;

        // Try parsing a pattern
        let pattern_result = self.parse_pattern();

        if let Ok(pattern) = pattern_result {
            if self.check(&Token::Colon) {
                // This is a for-each loop: `L pattern:iter { ... }`
                self.advance(); // consume ':'
                let iter = self.parse_expr()?;

                self.expect(&Token::LBrace)?;
                let body = self.parse_block_contents()?;
                self.expect(&Token::RBrace)?;

                let end = self.prev_span().end;
                return Ok(Spanned::new(
                    Expr::Loop {
                        pattern: Some(pattern),
                        iter: Some(Box::new(iter)),
                        body,
                    },
                    Span::new(start, end),
                ));
            }
        }

        // This is a while loop: `L condition { ... }`
        // Reset position and parse as expression
        self.pos = saved_pos;

        // Disable struct literals in condition to avoid ambiguity with block start
        // e.g., `L x == CONST { ... }` should not parse CONST{ as struct literal
        let old_allow_struct_literal = self.allow_struct_literal;
        self.allow_struct_literal = false;
        let condition = self.parse_expr()?;
        self.allow_struct_literal = old_allow_struct_literal;

        self.expect(&Token::LBrace)?;
        let body = self.parse_block_contents()?;
        self.expect(&Token::RBrace)?;

        let end = self.prev_span().end;
        Ok(Spanned::new(
            Expr::While {
                condition: Box::new(condition),
                body,
            },
            Span::new(start, end),
        ))
    }

    /// Parse match expression: `M expr{arms}`
    pub(crate) fn parse_match_expr(&mut self, start: usize) -> ParseResult<Spanned<Expr>> {
        let expr = self.parse_expr()?;
        self.expect(&Token::LBrace)?;

        let mut arms = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Parse pattern (possibly with Or patterns using |)
            let pattern = self.parse_or_pattern()?;

            // Check for guard: `I condition`
            let guard = if self.check(&Token::If) {
                self.advance();
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            };

            self.expect(&Token::FatArrow)?;
            let body = self.parse_expr()?;
            arms.push(MatchArm {
                pattern,
                guard,
                body: Box::new(body),
            });
            if !self.check(&Token::RBrace) {
                self.expect(&Token::Comma)?;
            }
        }

        self.expect(&Token::RBrace)?;
        let end = self.prev_span().end;

        Ok(Spanned::new(
            Expr::Match {
                expr: Box::new(expr),
                arms,
            },
            Span::new(start, end),
        ))
    }

    /// Parse or-pattern: `pattern | pattern | ...`
    pub(crate) fn parse_or_pattern(&mut self) -> ParseResult<Spanned<Pattern>> {
        let start = self.current_span().start;
        let first = self.parse_pattern()?;

        // Check for | to form Or pattern
        if self.check(&Token::Pipe) {
            let mut patterns = vec![first];
            while self.check(&Token::Pipe) {
                self.advance();
                patterns.push(self.parse_pattern()?);
            }
            let end = self.prev_span().end;
            Ok(Spanned::new(Pattern::Or(patterns), Span::new(start, end)))
        } else {
            Ok(first)
        }
    }

    /// Parse lambda expression: |params| body
    /// Syntax: |x: i64, y: i64| x + y
    pub(crate) fn parse_lambda(&mut self, start: usize) -> ParseResult<Spanned<Expr>> {
        // We've already consumed the opening |
        let mut params = Vec::new();

        // Parse parameters until closing |
        while !self.check(&Token::Pipe) && !self.is_at_end() {
            let name = self.parse_ident()?;
            self.expect(&Token::Colon)?;
            let ty = self.parse_type()?;
            params.push(Param {
                name,
                ty,
                is_mut: false,
                is_vararg: false,
                ownership: Ownership::Regular,
                default_value: None,
            });
            if !self.check(&Token::Pipe) {
                self.expect(&Token::Comma)?;
            }
        }
        self.expect(&Token::Pipe)?;

        // Parse lambda body (single expression)
        let body = self.parse_expr()?;
        let end = self.prev_span().end;

        Ok(Spanned::new(
            Expr::Lambda {
                params,
                body: Box::new(body),
                captures: vec![], // Filled during type checking
            },
            Span::new(start, end),
        ))
    }

    /// Parse pattern
    pub(crate) fn parse_pattern(&mut self) -> ParseResult<Spanned<Pattern>> {
        self.enter_depth()?;
        let start = self.current_span().start;

        if let Some(tok) = self.peek() {
            let pattern = match &tok.token {
                Token::Ident(s) if s == "_" => {
                    self.advance();
                    Pattern::Wildcard
                }
                Token::Ident(s) => {
                    let name = s.clone();
                    self.advance();
                    // Check for variant pattern: `Some(x)`
                    if self.check(&Token::LParen) {
                        self.advance();
                        let mut fields = Vec::new();
                        while !self.check(&Token::RParen) && !self.is_at_end() {
                            fields.push(self.parse_pattern()?);
                            if !self.check(&Token::RParen) {
                                self.expect(&Token::Comma)?;
                            }
                        }
                        self.expect(&Token::RParen)?;
                        Pattern::Variant {
                            name: Spanned::new(name, Span::new(start, start)),
                            fields,
                        }
                    } else {
                        Pattern::Ident(name)
                    }
                }
                Token::Int(n) => {
                    let n = *n;
                    self.advance();
                    // Check for range pattern
                    if self.check(&Token::DotDot) || self.check(&Token::DotDotEq) {
                        let inclusive = self.check(&Token::DotDotEq);
                        self.advance();
                        let end_pat = self.parse_pattern()?;
                        Pattern::Range {
                            start: Some(Box::new(Spanned::new(
                                Pattern::Literal(Literal::Int(n)),
                                Span::new(start, start),
                            ))),
                            end: Some(Box::new(end_pat)),
                            inclusive,
                        }
                    } else {
                        Pattern::Literal(Literal::Int(n))
                    }
                }
                Token::True => {
                    self.advance();
                    Pattern::Literal(Literal::Bool(true))
                }
                Token::False => {
                    self.advance();
                    Pattern::Literal(Literal::Bool(false))
                }
                Token::String(s) => {
                    let s = s.clone();
                    self.advance();
                    Pattern::Literal(Literal::String(s))
                }
                Token::LParen => {
                    self.advance();
                    let mut patterns = Vec::new();
                    while !self.check(&Token::RParen) && !self.is_at_end() {
                        patterns.push(self.parse_pattern()?);
                        if !self.check(&Token::RParen) {
                            self.expect(&Token::Comma)?;
                        }
                    }
                    self.expect(&Token::RParen)?;
                    Pattern::Tuple(patterns)
                }
                _ => {
                    let found = tok.token.clone();
                    let span = tok.span.clone();
                    self.exit_depth();
                    return Err(ParseError::UnexpectedToken {
                        found,
                        span,
                        expected: "pattern".into(),
                    });
                }
            };

            let end = self.prev_span().end;
            self.exit_depth();
            Ok(Spanned::new(pattern, Span::new(start, end)))
        } else {
            self.exit_depth();
            Err(ParseError::UnexpectedEof { span: self.current_span() })
        }
    }

    /// Parse function call arguments
    pub(crate) fn parse_args(&mut self) -> ParseResult<Vec<Spanned<Expr>>> {
        let mut args = Vec::new();

        while !self.check(&Token::RParen) && !self.is_at_end() {
            args.push(self.parse_expr()?);
            if !self.check(&Token::RParen) {
                self.expect(&Token::Comma)?;
            }
        }

        Ok(args)
    }

    /// Parse identifier
    /// Single-letter keywords can also be identifiers in non-keyword contexts
    pub(crate) fn parse_ident(&mut self) -> ParseResult<Spanned<String>> {
        let span = self.current_span();
        let tok = self.advance().ok_or(ParseError::UnexpectedEof { span })?;
        let name = match &tok.token {
            Token::Ident(s) => s.clone(),
            // Single-letter keywords can be used as identifiers
            Token::Function => "F".to_string(),
            Token::Struct => "S".to_string(),
            Token::Enum => "E".to_string(),
            Token::If => "I".to_string(),
            Token::Loop => "L".to_string(),
            Token::Match => "M".to_string(),
            Token::Async => "A".to_string(),
            Token::Return => "R".to_string(),
            Token::Break => "B".to_string(),
            Token::Continue => "C".to_string(),
            Token::TypeKeyword => "T".to_string(),
            Token::Use => "U".to_string(),
            Token::Pub => "P".to_string(),
            Token::Trait => "W".to_string(),
            Token::Impl => "X".to_string(),
            Token::Defer => "D".to_string(),
            Token::Extern => "N".to_string(),
            Token::Union => "O".to_string(),
            Token::Await => "Y".to_string(),
            // Multi-character keywords that can appear in import paths or identifiers
            Token::Io => "io".to_string(),
            Token::Mut => "mut".to_string(),
            Token::Spawn => "spawn".to_string(),
            Token::SelfLower => "self".to_string(),
            Token::Str => "str".to_string(),
            Token::Bool => "bool".to_string(),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    found: tok.token,
                    span: tok.span,
                    expected: "identifier".into(),
                });
            }
        };
        Ok(Spanned::new(name, Span::new(tok.span.start, tok.span.end)))
    }
}

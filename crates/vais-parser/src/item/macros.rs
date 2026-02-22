use vais_ast::*;
use vais_lexer::Token;

use crate::{ParseError, ParseResult, Parser};

impl Parser {
    /// Parse macro definition: `macro name! { rules }`
    pub(super) fn parse_macro_def(&mut self, is_pub: bool) -> ParseResult<MacroDef> {
        let name = self.parse_ident()?;

        // Expect `!` after macro name
        self.expect(&Token::Bang)?;

        let lbrace_span = self.current_span();
        self.expect(&Token::LBrace)?;

        let mut rules = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            rules.push(self.parse_macro_rule()?);
        }

        self.expect_closing(&Token::RBrace, lbrace_span)?;

        Ok(MacroDef {
            name,
            rules,
            is_pub,
        })
    }

    /// Parse a single macro rule: `pattern => template`
    fn parse_macro_rule(&mut self) -> ParseResult<MacroRule> {
        let pattern = self.parse_macro_pattern()?;
        self.expect(&Token::FatArrow)?;
        let template = self.parse_macro_template()?;

        Ok(MacroRule { pattern, template })
    }

    /// Parse macro pattern: `()` or `($x:expr, $y:expr)` etc.
    fn parse_macro_pattern(&mut self) -> ParseResult<MacroPattern> {
        self.expect(&Token::LParen)?;

        if self.check(&Token::RParen) {
            self.advance();
            return Ok(MacroPattern::Empty);
        }

        let elements = self.parse_macro_pattern_elements()?;
        self.expect(&Token::RParen)?;

        Ok(MacroPattern::Sequence(elements))
    }

    /// Parse pattern elements inside parentheses
    fn parse_macro_pattern_elements(&mut self) -> ParseResult<Vec<MacroPatternElement>> {
        let mut elements = Vec::new();

        while !self.check(&Token::RParen) && !self.is_at_end() {
            elements.push(self.parse_macro_pattern_element()?);

            // Optional comma between elements
            if self.check(&Token::Comma) && !self.is_repetition_end() {
                self.advance();
            }
        }

        Ok(elements)
    }

    /// Check if we're at the end of a repetition pattern
    fn is_repetition_end(&self) -> bool {
        if let Some(next) = self.peek_next() {
            matches!(next.token, Token::Star | Token::Plus | Token::Question)
        } else {
            false
        }
    }

    /// Parse a single pattern element
    fn parse_macro_pattern_element(&mut self) -> ParseResult<MacroPatternElement> {
        // Check for metavariable: `$name:kind`
        if self.check(&Token::Dollar) {
            self.advance();

            // Check for repetition: `$(...)*` or `$(...)+` or `$(...)?`
            if self.check(&Token::LParen) {
                self.advance();
                let patterns = self.parse_macro_pattern_elements()?;
                self.expect(&Token::RParen)?;

                // Parse optional separator (e.g., `,`)
                let separator = if !self.check(&Token::Star)
                    && !self.check(&Token::Plus)
                    && !self.check(&Token::Question)
                {
                    let sep = self.parse_macro_token()?;
                    Some(sep)
                } else {
                    None
                };

                // Parse repetition kind
                let kind = if self.check(&Token::Star) {
                    self.advance();
                    RepetitionKind::ZeroOrMore
                } else if self.check(&Token::Plus) {
                    self.advance();
                    RepetitionKind::OneOrMore
                } else if self.check(&Token::Question) {
                    self.advance();
                    RepetitionKind::ZeroOrOne
                } else {
                    return Err(ParseError::UnexpectedToken {
                        found: self
                            .peek()
                            .map(|t| t.token.clone())
                            .unwrap_or(Token::Ident("EOF".into())),
                        span: self.current_span(),
                        expected: "*, +, or ? for repetition".into(),
                    });
                };

                return Ok(MacroPatternElement::Repetition {
                    patterns,
                    separator,
                    kind,
                });
            }

            // Regular metavariable: `$name:kind`
            let name = self.parse_ident()?.node;
            self.expect(&Token::Colon)?;
            let kind_name = self.parse_ident()?.node;

            let kind =
                kind_name
                    .parse::<MetaVarKind>()
                    .map_err(|_| ParseError::UnexpectedToken {
                        found: Token::Ident(kind_name.clone()),
                        span: self.current_span(),
                        expected:
                            "metavariable kind (expr, ty, ident, pat, stmt, block, item, lit, tt)"
                                .into(),
                    })?;

            return Ok(MacroPatternElement::MetaVar { name, kind });
        }

        // Check for nested group
        if self.check(&Token::LParen) {
            self.advance();
            let content = self.parse_macro_pattern_elements()?;
            self.expect(&Token::RParen)?;
            return Ok(MacroPatternElement::Group {
                delimiter: Delimiter::Paren,
                content,
            });
        }

        if self.check(&Token::LBracket) {
            self.advance();
            let content = self.parse_macro_pattern_elements()?;
            self.expect(&Token::RBracket)?;
            return Ok(MacroPatternElement::Group {
                delimiter: Delimiter::Bracket,
                content,
            });
        }

        if self.check(&Token::LBrace) {
            self.advance();
            let content = self.parse_macro_pattern_elements()?;
            self.expect(&Token::RBrace)?;
            return Ok(MacroPatternElement::Group {
                delimiter: Delimiter::Brace,
                content,
            });
        }

        // Literal token
        let token = self.parse_macro_token()?;
        Ok(MacroPatternElement::Token(token))
    }

    /// Parse a single macro token
    fn parse_macro_token(&mut self) -> ParseResult<MacroToken> {
        let span = self.current_span();
        let tok = self
            .advance()
            .ok_or(ParseError::UnexpectedEof { span: span.clone() })?;

        let macro_token = match &tok.token {
            Token::Ident(s) => MacroToken::Ident(s.clone()),
            Token::Int(n) => MacroToken::Literal(MacroLiteral::Int(*n)),
            Token::Float(n) => MacroToken::Literal(MacroLiteral::Float(*n)),
            Token::String(s) => MacroToken::Literal(MacroLiteral::String(s.clone())),
            Token::True => MacroToken::Literal(MacroLiteral::Bool(true)),
            Token::False => MacroToken::Literal(MacroLiteral::Bool(false)),
            Token::Plus => MacroToken::Punct('+'),
            Token::Minus => MacroToken::Punct('-'),
            Token::Star => MacroToken::Punct('*'),
            Token::Slash => MacroToken::Punct('/'),
            Token::Percent => MacroToken::Punct('%'),
            Token::Eq => MacroToken::Punct('='),
            Token::Lt => MacroToken::Punct('<'),
            Token::Gt => MacroToken::Punct('>'),
            Token::Amp => MacroToken::Punct('&'),
            Token::Pipe => MacroToken::Punct('|'),
            Token::Bang => MacroToken::Punct('!'),
            Token::Comma => MacroToken::Punct(','),
            Token::Colon => MacroToken::Punct(':'),
            Token::Semi => MacroToken::Punct(';'),
            Token::Dot => MacroToken::Punct('.'),
            Token::At => MacroToken::Punct('@'),
            Token::Caret => MacroToken::Punct('^'),
            Token::Question => MacroToken::Punct('?'),
            // Multi-character operators represented as Ident tokens in macro context
            Token::Arrow => MacroToken::Ident("->".to_string()),
            Token::FatArrow => MacroToken::Ident("=>".to_string()),
            Token::DotDot => MacroToken::Ident("..".to_string()),
            Token::DotDotEq => MacroToken::Ident("..=".to_string()),
            Token::Ellipsis => MacroToken::Ident("...".to_string()),
            Token::ColonColon => MacroToken::Ident("::".to_string()),
            Token::Shl => MacroToken::Ident("<<".to_string()),
            Token::Shr => MacroToken::Ident(">>".to_string()),
            Token::EqEq => MacroToken::Ident("==".to_string()),
            Token::Neq => MacroToken::Ident("!=".to_string()),
            Token::Lte => MacroToken::Ident("<=".to_string()),
            Token::Gte => MacroToken::Ident(">=".to_string()),
            Token::PlusEq => MacroToken::Ident("+=".to_string()),
            Token::MinusEq => MacroToken::Ident("-=".to_string()),
            Token::StarEq => MacroToken::Ident("*=".to_string()),
            Token::SlashEq => MacroToken::Ident("/=".to_string()),
            Token::ColonEq => MacroToken::Ident(":=".to_string()),
            Token::PipeArrow => MacroToken::Ident("|>".to_string()),
            // Keywords as identifiers in macro context
            Token::Function => MacroToken::Ident("F".to_string()),
            Token::Struct => MacroToken::Ident("S".to_string()),
            Token::Enum => MacroToken::Ident("E".to_string()),
            Token::If => MacroToken::Ident("I".to_string()),
            Token::Loop => MacroToken::Ident("L".to_string()),
            Token::Match => MacroToken::Ident("M".to_string()),
            Token::Return => MacroToken::Ident("R".to_string()),
            Token::Break => MacroToken::Ident("B".to_string()),
            Token::Continue => MacroToken::Ident("C".to_string()),
            Token::Mut => MacroToken::Ident("mut".to_string()),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    found: tok.token.clone(),
                    span,
                    expected: "macro token".into(),
                });
            }
        };

        Ok(macro_token)
    }

    /// Parse macro template: `{ tokens }` or `( tokens )` or `[ tokens ]`
    fn parse_macro_template(&mut self) -> ParseResult<MacroTemplate> {
        self.expect(&Token::LBrace)?;

        if self.check(&Token::RBrace) {
            self.advance();
            return Ok(MacroTemplate::Empty);
        }

        let elements = self.parse_macro_template_elements()?;
        self.expect(&Token::RBrace)?;

        Ok(MacroTemplate::Sequence(elements))
    }

    /// Parse template elements inside delimiters
    fn parse_macro_template_elements(&mut self) -> ParseResult<Vec<MacroTemplateElement>> {
        let mut elements = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            elements.push(self.parse_macro_template_element()?);
        }

        Ok(elements)
    }

    /// Parse a single template element
    fn parse_macro_template_element(&mut self) -> ParseResult<MacroTemplateElement> {
        // Check for metavariable substitution: `$name`
        if self.check(&Token::Dollar) {
            self.advance();

            // Check for repetition: `$(...)*`
            if self.check(&Token::LParen) {
                self.advance();
                let elements = self.parse_macro_template_elements_until_rparen()?;
                self.expect(&Token::RParen)?;

                // Parse optional separator
                let separator = if !self.check(&Token::Star)
                    && !self.check(&Token::Plus)
                    && !self.check(&Token::Question)
                {
                    let sep = self.parse_macro_token()?;
                    Some(sep)
                } else {
                    None
                };

                // Parse repetition kind
                let kind = if self.check(&Token::Star) {
                    self.advance();
                    RepetitionKind::ZeroOrMore
                } else if self.check(&Token::Plus) {
                    self.advance();
                    RepetitionKind::OneOrMore
                } else if self.check(&Token::Question) {
                    self.advance();
                    RepetitionKind::ZeroOrOne
                } else {
                    return Err(ParseError::UnexpectedToken {
                        found: self
                            .peek()
                            .map(|t| t.token.clone())
                            .unwrap_or(Token::Ident("EOF".into())),
                        span: self.current_span(),
                        expected: "*, +, or ? for repetition".into(),
                    });
                };

                return Ok(MacroTemplateElement::Repetition {
                    elements,
                    separator,
                    kind,
                });
            }

            // Regular metavariable: `$name`
            let name = self.parse_ident()?.node;
            return Ok(MacroTemplateElement::MetaVar(name));
        }

        // Check for nested group
        if self.check(&Token::LParen) {
            self.advance();
            let content = self.parse_macro_template_elements_until_rparen()?;
            self.expect(&Token::RParen)?;
            return Ok(MacroTemplateElement::Group {
                delimiter: Delimiter::Paren,
                content,
            });
        }

        if self.check(&Token::LBracket) {
            self.advance();
            let content = self.parse_macro_template_elements_until_rbracket()?;
            self.expect(&Token::RBracket)?;
            return Ok(MacroTemplateElement::Group {
                delimiter: Delimiter::Bracket,
                content,
            });
        }

        if self.check(&Token::LBrace) {
            self.advance();
            let content = self.parse_macro_template_elements()?;
            self.expect(&Token::RBrace)?;
            return Ok(MacroTemplateElement::Group {
                delimiter: Delimiter::Brace,
                content,
            });
        }

        // Literal token
        let token = self.parse_macro_token()?;
        Ok(MacroTemplateElement::Token(token))
    }

    /// Parse template elements until right paren
    fn parse_macro_template_elements_until_rparen(
        &mut self,
    ) -> ParseResult<Vec<MacroTemplateElement>> {
        let mut elements = Vec::new();

        while !self.check(&Token::RParen) && !self.is_at_end() {
            elements.push(self.parse_macro_template_element()?);
        }

        Ok(elements)
    }

    /// Parse template elements until right bracket
    fn parse_macro_template_elements_until_rbracket(
        &mut self,
    ) -> ParseResult<Vec<MacroTemplateElement>> {
        let mut elements = Vec::new();

        while !self.check(&Token::RBracket) && !self.is_at_end() {
            elements.push(self.parse_macro_template_element()?);
        }

        Ok(elements)
    }

    /// Parse macro invocation in expression context: `name!(args)`
    pub(crate) fn parse_macro_invoke(&mut self, name: Spanned<String>) -> ParseResult<MacroInvoke> {
        // `!` was already consumed

        let (delimiter, tokens) = if self.check(&Token::LParen) {
            self.advance();
            let tokens = self.collect_macro_tokens_until(&Token::RParen)?;
            self.expect(&Token::RParen)?;
            (Delimiter::Paren, tokens)
        } else if self.check(&Token::LBracket) {
            self.advance();
            let tokens = self.collect_macro_tokens_until(&Token::RBracket)?;
            self.expect(&Token::RBracket)?;
            (Delimiter::Bracket, tokens)
        } else if self.check(&Token::LBrace) {
            self.advance();
            let tokens = self.collect_macro_tokens_until(&Token::RBrace)?;
            self.expect(&Token::RBrace)?;
            (Delimiter::Brace, tokens)
        } else {
            return Err(ParseError::UnexpectedToken {
                found: self
                    .peek()
                    .map(|t| t.token.clone())
                    .unwrap_or(Token::Ident("EOF".into())),
                span: self.current_span(),
                expected: "'(', '[', or '{' for macro invocation".into(),
            });
        };

        Ok(MacroInvoke {
            name,
            delimiter,
            tokens,
        })
    }

    /// Collect tokens until a specific delimiter, handling nesting
    fn collect_macro_tokens_until(&mut self, end: &Token) -> ParseResult<Vec<MacroToken>> {
        let mut tokens = Vec::new();
        let mut depth = 1;

        while depth > 0 && !self.is_at_end() {
            if self.check(end) {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }

            // Track nesting
            if self.check(&Token::LParen)
                || self.check(&Token::LBracket)
                || self.check(&Token::LBrace)
            {
                depth += 1;
            }

            tokens.push(self.parse_macro_token()?);
        }

        Ok(tokens)
    }
}

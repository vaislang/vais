//! Item parsing for Vais language.
//!
//! Handles parsing of top-level items including functions, structs, enums,
//! unions, type aliases, use imports, traits, impl blocks, macro definitions,
//! const/global definitions, and extern function declarations.

use vais_ast::*;
use vais_lexer::Token;

use crate::{ParseError, ParseResult, Parser};

mod declarations;
mod macros;
mod traits;

impl Parser {
    /// Parse a top-level item
    pub(crate) fn parse_item(&mut self) -> ParseResult<Spanned<Item>> {
        // Parse attributes first
        let attributes = self.parse_attributes()?;

        let is_pub = self.check(&Token::Pub);
        if is_pub {
            self.advance();
        }

        let start = self.current_span().start;

        let item = if self.check(&Token::Function) {
            self.advance();
            Item::Function(self.parse_function(is_pub, false, attributes)?)
        } else if self.check(&Token::Async) {
            self.advance();
            self.expect(&Token::Function)?;
            Item::Function(self.parse_function(is_pub, true, attributes)?)
        } else if self.check(&Token::Struct) {
            self.advance();
            Item::Struct(self.parse_struct(is_pub, attributes)?)
        } else if self.check(&Token::Enum) {
            self.advance();
            Item::Enum(self.parse_enum(is_pub, attributes)?)
        } else if self.check(&Token::Union) {
            self.advance();
            Item::Union(self.parse_union(is_pub)?)
        } else if self.check(&Token::TypeKeyword) {
            self.advance();
            self.parse_type_or_trait_alias(is_pub)?
        } else if self.check(&Token::Use) {
            self.advance();
            Item::Use(self.parse_use()?)
        } else if self.check(&Token::Trait) {
            self.advance();
            Item::Trait(self.parse_trait(is_pub)?)
        } else if self.check(&Token::Impl) {
            self.advance();
            // Check if this is an extern function declaration (X F name(...))
            if self.check(&Token::Function) {
                self.advance();
                Item::ExternBlock(self.parse_single_extern_function(attributes)?)
            } else {
                Item::Impl(self.parse_impl()?)
            }
        } else if self.check(&Token::Macro) {
            self.advance();
            Item::Macro(self.parse_macro_def(is_pub)?)
        } else if self.check(&Token::Extern) {
            self.advance();
            Item::ExternBlock(self.parse_extern_block()?)
        } else if self.check(&Token::Continue) {
            // C at top level is a constant definition, not continue
            self.advance();
            Item::Const(self.parse_const_def(is_pub, attributes)?)
        } else if self.check(&Token::Global) {
            self.advance();
            Item::Global(self.parse_global_def(is_pub)?)
        } else {
            return Err(ParseError::UnexpectedToken {
                found: self
                    .peek()
                    .map(|t| t.token.clone())
                    .unwrap_or(Token::Ident("EOF".into())),
                span: self.current_span(),
                expected: "F, S, E, O, T, U, W, X, N (extern), C (const), G (global), or macro"
                    .into(),
            });
        };

        let end = self.prev_span().end;
        Ok(Spanned::new(item, Span::new(start, end)))
    }

    /// Parse attributes: `#[name(args)]`
    pub(crate) fn parse_attributes(&mut self) -> ParseResult<Vec<Attribute>> {
        let mut attrs = Vec::new();

        while self.check(&Token::HashBracket) {
            self.advance();
            let attr = self.parse_attribute()?;
            attrs.push(attr);
            self.expect(&Token::RBracket)?;
        }

        Ok(attrs)
    }

    /// Parse single attribute: `name(args)` or just `name`
    ///
    /// For contract attributes (requires, ensures, invariant), the argument is
    /// parsed as an expression rather than a simple identifier list.
    fn parse_attribute(&mut self) -> ParseResult<Attribute> {
        let name = self.parse_ident()?.node;

        // Contract attributes parse their argument as an expression
        if name == "requires" || name == "ensures" || name == "invariant" || name == "decreases" {
            self.expect(&Token::LParen)?;

            // Record the start position for the expression string
            let expr_start = self.pos;
            let expr = self.parse_expr()?;

            // Reconstruct the expression string from tokens for error messages
            // We store the original text representation
            let expr_str = self.reconstruct_expr_string(expr_start, self.pos);

            self.expect(&Token::RParen)?;

            return Ok(Attribute {
                name,
                args: vec![expr_str],
                expr: Some(Box::new(expr)),
            });
        }

        let args = if self.check(&Token::LParen) {
            self.advance();
            let mut args = Vec::new();
            while !self.check(&Token::RParen) && !self.is_at_end() {
                if let Some(tok) = self.peek() {
                    // Accept identifiers, string literals, and single-letter keywords as attribute args
                    let arg = match &tok.token {
                        Token::Ident(s) => Some(s.clone()),
                        // String literals for wasm_import/wasm_export module/name args
                        Token::String(s) => Some(s.clone()),
                        // Single-letter keywords can be attribute args (e.g., repr(C))
                        Token::Continue => Some("C".to_string()),
                        Token::Function => Some("F".to_string()),
                        Token::Struct => Some("S".to_string()),
                        Token::Enum => Some("E".to_string()),
                        Token::If => Some("I".to_string()),
                        Token::Loop => Some("L".to_string()),
                        Token::Match => Some("M".to_string()),
                        Token::Async => Some("A".to_string()),
                        Token::Return => Some("R".to_string()),
                        Token::Break => Some("B".to_string()),
                        Token::Use => Some("U".to_string()),
                        Token::Pub => Some("P".to_string()),
                        Token::TypeKeyword => Some("T".to_string()),
                        _ => None,
                    };
                    if let Some(s) = arg {
                        args.push(s.clone());
                        self.advance();
                        // Support nested parens for not(...) syntax
                        // e.g., #[cfg(not(target_os = "windows"))]
                        if self.check(&Token::LParen) {
                            self.advance();
                            // Parse inner content as flat args
                            while !self.check(&Token::RParen) && !self.is_at_end() {
                                if let Some(inner_tok) = self.peek() {
                                    let inner_arg = match &inner_tok.token {
                                        Token::Ident(s) => Some(s.clone()),
                                        _ => None,
                                    };
                                    if let Some(inner_s) = inner_arg {
                                        args.push(inner_s);
                                        self.advance();
                                        if self.check(&Token::Eq) {
                                            self.advance();
                                            if let Some(val_tok) = self.peek() {
                                                if let Token::String(val) = &val_tok.token {
                                                    args.push("=".to_string());
                                                    args.push(val.clone());
                                                    self.advance();
                                                }
                                            }
                                        }
                                    } else {
                                        break;
                                    }
                                    if self.check(&Token::Comma) {
                                        self.advance();
                                    }
                                }
                            }
                            if self.check(&Token::RParen) {
                                self.advance();
                            }
                        }
                        // Support key = "value" syntax for cfg attributes
                        // e.g., #[cfg(target_os = "linux")]
                        else if self.check(&Token::Eq) {
                            self.advance();
                            if let Some(val_tok) = self.peek() {
                                if let Token::String(val) = &val_tok.token {
                                    args.push("=".to_string());
                                    args.push(val.clone());
                                    self.advance();
                                }
                            }
                        }
                    } else {
                        break;
                    }
                }
                if self.check(&Token::Comma) {
                    self.advance();
                }
            }
            self.expect(&Token::RParen)?;
            args
        } else {
            Vec::new()
        };

        Ok(Attribute {
            name,
            args,
            expr: None,
        })
    }

    /// Reconstruct expression string from token range (for error messages)
    pub(crate) fn reconstruct_expr_string(&self, start_pos: usize, end_pos: usize) -> String {
        let tokens: Vec<String> = self.tokens[start_pos..end_pos]
            .iter()
            .map(|t| format!("{}", t.token))
            .collect();
        tokens.join(" ")
    }
}

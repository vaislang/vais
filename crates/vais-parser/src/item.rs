//! Item parsing for Vais language.
//!
//! Handles parsing of top-level items including functions, structs, enums,
//! unions, type aliases, use imports, traits, impl blocks, macro definitions,
//! const/global definitions, and extern function declarations.

use vais_ast::*;
use vais_lexer::Token;

use crate::{ParseError, ParseResult, Parser};

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
            Item::Enum(self.parse_enum(is_pub)?)
        } else if self.check(&Token::Union) {
            self.advance();
            Item::Union(self.parse_union(is_pub)?)
        } else if self.check(&Token::TypeKeyword) {
            self.advance();
            Item::TypeAlias(self.parse_type_alias(is_pub)?)
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

    /// Parse function: `name(params)->ret=expr` or `name(params)->ret{...}`
    pub(crate) fn parse_function(
        &mut self,
        is_pub: bool,
        is_async: bool,
        attributes: Vec<Attribute>,
    ) -> ParseResult<Function> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;

        let lparen_span = self.current_span();
        self.expect(&Token::LParen)?;
        let params = self.parse_params()?;
        self.expect_closing(&Token::RParen, lparen_span)?;

        let ret_type = if self.check(&Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse where clause (between return type and body)
        let where_clause = self.parse_where_clause()?;

        let body = if self.check(&Token::Eq) {
            self.advance();
            FunctionBody::Expr(Box::new(self.parse_expr()?))
        } else if self.check(&Token::LBrace) {
            let lbrace_span = self.current_span();
            self.advance();
            let stmts = self.parse_block_contents()?;
            self.expect_closing(&Token::RBrace, lbrace_span)?;
            FunctionBody::Block(stmts)
        } else {
            return Err(ParseError::UnexpectedToken {
                found: self
                    .peek()
                    .map(|t| t.token.clone())
                    .unwrap_or(Token::Ident("EOF".into())),
                span: self.current_span(),
                expected: "= or {".into(),
            });
        };

        Ok(Function {
            name,
            generics,
            params,
            ret_type,
            body,
            is_pub,
            is_async,
            attributes,
            where_clause,
        })
    }

    /// Parse struct: `Name{fields}` with optional methods
    fn parse_struct(&mut self, is_pub: bool, attributes: Vec<Attribute>) -> ParseResult<Struct> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;

        // Parse where clause
        let where_clause = self.parse_where_clause()?;

        let lbrace_span = self.current_span();
        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Check for attributes or function/pub keywords
            let start = self.current_span().start;
            let method_attrs = self.parse_attributes()?;
            if self.check(&Token::Function) || self.check(&Token::Pub) || !method_attrs.is_empty() {
                let is_method_pub = self.check(&Token::Pub);
                if is_method_pub {
                    self.advance();
                }
                self.expect(&Token::Function)?;
                let method = self.parse_function(is_method_pub, false, method_attrs)?;
                let end = self.prev_span().end;
                methods.push(Spanned::new(method, Span::new(start, end)));
            } else {
                fields.push(self.parse_field()?);
                if !self.check(&Token::RBrace) {
                    self.expect(&Token::Comma)?;
                }
            }
        }

        self.expect_closing(&Token::RBrace, lbrace_span)?;

        Ok(Struct {
            name,
            generics,
            fields,
            methods,
            is_pub,
            attributes,
            where_clause,
        })
    }

    /// Parse enum: `Name{variants}`
    fn parse_enum(&mut self, is_pub: bool) -> ParseResult<Enum> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;

        let lbrace_span = self.current_span();
        self.expect(&Token::LBrace)?;
        let mut variants = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            variants.push(self.parse_variant()?);
            if !self.check(&Token::RBrace) {
                self.expect(&Token::Comma)?;
            }
        }

        self.expect_closing(&Token::RBrace, lbrace_span)?;

        Ok(Enum {
            name,
            generics,
            variants,
            is_pub,
        })
    }

    /// Parse variant: `Name` or `Name(types)` or `Name{fields}`
    fn parse_variant(&mut self) -> ParseResult<Variant> {
        let name = self.parse_ident()?;

        let fields = if self.check(&Token::LParen) {
            let lparen_span = self.current_span();
            self.advance();
            let mut types = Vec::new();
            while !self.check(&Token::RParen) && !self.is_at_end() {
                types.push(self.parse_type()?);
                if !self.check(&Token::RParen) {
                    self.expect(&Token::Comma)?;
                }
            }
            self.expect_closing(&Token::RParen, lparen_span)?;
            VariantFields::Tuple(types)
        } else if self.check(&Token::LBrace) {
            let lbrace_span = self.current_span();
            self.advance();
            let mut fields = Vec::new();
            while !self.check(&Token::RBrace) && !self.is_at_end() {
                fields.push(self.parse_field()?);
                if !self.check(&Token::RBrace) {
                    self.expect(&Token::Comma)?;
                }
            }
            self.expect_closing(&Token::RBrace, lbrace_span)?;
            VariantFields::Struct(fields)
        } else {
            VariantFields::Unit
        };

        Ok(Variant { name, fields })
    }

    /// Parse union: `Name{fields}` (untagged union, C-style)
    fn parse_union(&mut self, is_pub: bool) -> ParseResult<Union> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;

        let lbrace_span = self.current_span();
        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            fields.push(self.parse_field()?);
            if !self.check(&Token::RBrace) {
                self.expect(&Token::Comma)?;
            }
        }

        self.expect_closing(&Token::RBrace, lbrace_span)?;

        Ok(Union {
            name,
            generics,
            fields,
            is_pub,
        })
    }

    /// Parse type alias: `Name=Type`
    fn parse_type_alias(&mut self, is_pub: bool) -> ParseResult<TypeAlias> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;
        self.expect(&Token::Eq)?;
        let ty = self.parse_type()?;

        Ok(TypeAlias {
            name,
            generics,
            ty,
            is_pub,
        })
    }

    /// Parse use statement: `module` or `module/submodule` or `module::submodule`
    /// Also supports selective imports: `module.Item`, `module.{A, B, C}`
    /// Optional semicolon terminator: `module;` or `module.Item;`
    fn parse_use(&mut self) -> ParseResult<Use> {
        let mut path = vec![self.parse_ident()?];

        // Support both `::` and `/` as path separators
        while self.check(&Token::ColonColon) || self.check(&Token::Slash) {
            self.advance();
            path.push(self.parse_ident()?);
        }

        // Check for selective import: `.Ident` or `.{Ident, ...}`
        let items = if self.check(&Token::Dot) {
            self.advance();
            if self.check(&Token::LBrace) {
                // Multi-item: `.{A, B, C}`
                self.advance(); // consume `{`
                let mut selected = Vec::new();
                if !self.check(&Token::RBrace) {
                    selected.push(self.parse_ident()?);
                    while self.check(&Token::Comma) {
                        self.advance();
                        // Allow trailing comma
                        if self.check(&Token::RBrace) {
                            break;
                        }
                        selected.push(self.parse_ident()?);
                    }
                }
                self.expect(&Token::RBrace)?;
                Some(selected)
            } else {
                // Single item: `.Ident`
                let item = self.parse_ident()?;
                Some(vec![item])
            }
        } else {
            None
        };

        // Optional semicolon terminator
        if self.check(&Token::Semi) {
            self.advance();
        }

        Ok(Use {
            path,
            alias: None,
            items,
        })
    }

    /// Parse constant definition: `C NAME: Type = value`
    fn parse_const_def(
        &mut self,
        is_pub: bool,
        attributes: Vec<Attribute>,
    ) -> ParseResult<ConstDef> {
        let name = self.parse_ident()?;
        self.expect(&Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(&Token::Eq)?;
        let value = self.parse_expr()?;

        Ok(ConstDef {
            name,
            ty,
            value,
            is_pub,
            attributes,
        })
    }

    /// Parse global variable definition: `G name: Type = value`
    fn parse_global_def(&mut self, is_pub: bool) -> ParseResult<GlobalDef> {
        let name = self.parse_ident()?;
        self.expect(&Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(&Token::Eq)?;
        let value = self.parse_expr()?;

        Ok(GlobalDef {
            name,
            ty,
            value,
            is_pub,
            is_mutable: true, // Globals are mutable by default
        })
    }

    /// Parse a single extern function declaration: `X F name(params) -> RetType`
    /// This is a shorthand for declaring an extern function without a body.
    /// Returns an ExternBlock containing the single function.
    fn parse_single_extern_function(
        &mut self,
        attributes: Vec<Attribute>,
    ) -> ParseResult<ExternBlock> {
        let name = self.parse_ident()?;

        // Parse parameters
        self.expect(&Token::LParen)?;
        let (params, is_vararg) = self.parse_single_extern_params()?;
        self.expect(&Token::RParen)?;

        // Parse optional return type
        let ret_type = if self.check(&Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        Ok(ExternBlock {
            abi: "C".to_string(),
            functions: vec![ExternFunction {
                name,
                params,
                ret_type,
                is_vararg,
                attributes,
            }],
        })
    }

    /// Parse extern function parameters for single extern declaration (returns params and vararg flag)
    fn parse_single_extern_params(&mut self) -> ParseResult<(Vec<Param>, bool)> {
        let mut params = Vec::new();
        let mut is_vararg = false;

        while !self.check(&Token::RParen) && !self.is_at_end() {
            // Check for variadic args (...)
            if self.check(&Token::Ellipsis) {
                self.advance();
                is_vararg = true;
                break;
            }

            // Parse parameter: name: Type
            let param_name = self.parse_ident()?;
            self.expect(&Token::Colon)?;
            let param_type = self.parse_type()?;

            // Parse optional default value: `= expr`
            let default_value = if self.check(&Token::Eq) {
                self.advance();
                let expr = self.parse_expr()?;
                Some(Box::new(expr))
            } else {
                None
            };

            params.push(Param {
                name: param_name,
                ty: param_type,
                is_mut: false,
                is_vararg: false,
                ownership: Ownership::Regular,
                default_value,
            });

            if !self.check(&Token::RParen) {
                self.expect(&Token::Comma)?;
                // Allow trailing comma before variadic
                if self.check(&Token::Ellipsis) {
                    self.advance();
                    is_vararg = true;
                    break;
                }
            }
        }

        Ok((params, is_vararg))
    }

    /// Parse trait definition: `W Name { methods }`
    fn parse_trait(&mut self, is_pub: bool) -> ParseResult<Trait> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;

        // Parse super traits: `W Iterator: Iterable + Clone`
        let super_traits = if self.check(&Token::Colon) {
            self.advance();
            self.parse_trait_bounds()?
        } else {
            Vec::new()
        };

        // Parse where clause
        let where_clause = self.parse_where_clause()?;

        let lbrace_span = self.current_span();
        self.expect(&Token::LBrace)?;

        let mut methods = Vec::new();
        let mut associated_types = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Check for associated type: `T Item` or `T Item: Trait`
            if self.check(&Token::TypeKeyword) {
                self.advance();
                associated_types.push(self.parse_associated_type()?);
            } else {
                methods.push(self.parse_trait_method()?);
            }
        }

        self.expect_closing(&Token::RBrace, lbrace_span)?;

        Ok(Trait {
            name,
            generics,
            super_traits,
            associated_types,
            methods,
            is_pub,
            where_clause,
        })
    }

    /// Parse associated type: `T Item` or `T Item: Trait` or `T Item = DefaultType`
    fn parse_associated_type(&mut self) -> ParseResult<AssociatedType> {
        let name = self.parse_ident()?;

        // GAT: Optional generic parameters (e.g., `T Item<'a, B: Clone>`)
        let generics = self.parse_generics()?;

        // Optional trait bounds
        let bounds = if self.check(&Token::Colon) {
            self.advance();
            self.parse_trait_bounds()?
        } else {
            Vec::new()
        };

        // Optional default type
        let default = if self.check(&Token::Eq) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        Ok(AssociatedType {
            name,
            generics,
            bounds,
            default,
        })
    }

    /// Parse trait method signature
    fn parse_trait_method(&mut self) -> ParseResult<TraitMethod> {
        // Check for const keyword: `C F method_name()` (const trait method)
        let is_const = if self.check(&Token::Const) {
            self.advance();
            true
        } else {
            false
        };

        // Check for async keyword: `A F method_name()`
        let is_async = if self.check(&Token::Async) {
            self.advance();
            true
        } else {
            false
        };

        self.expect(&Token::Function)?;
        let name = self.parse_ident()?;

        let lparen_span = self.current_span();
        self.expect(&Token::LParen)?;
        let params = self.parse_params()?;
        self.expect_closing(&Token::RParen, lparen_span)?;

        let ret_type = if self.check(&Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Optional default implementation
        let default_body = if self.check(&Token::Eq) {
            self.advance();
            Some(FunctionBody::Expr(Box::new(self.parse_expr()?)))
        } else if self.check(&Token::LBrace) {
            let lbrace_span = self.current_span();
            self.advance();
            let stmts = self.parse_block_contents()?;
            self.expect_closing(&Token::RBrace, lbrace_span)?;
            Some(FunctionBody::Block(stmts))
        } else {
            None
        };

        Ok(TraitMethod {
            name,
            params,
            ret_type,
            default_body,
            is_async,
            is_const,
        })
    }

    // =============================================================================
    // Macro Parsing
    // =============================================================================

    /// Parse macro definition: `macro name! { rules }`
    fn parse_macro_def(&mut self, is_pub: bool) -> ParseResult<MacroDef> {
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

    // =============================================================================
    // End Macro Parsing
    // =============================================================================

    /// Parse impl block: `X Type: Trait { methods }`
    pub(crate) fn parse_impl(&mut self) -> ParseResult<Impl> {
        let generics = self.parse_generics()?;
        let target_type = self.parse_type()?;

        // Optional trait name
        let trait_name = if self.check(&Token::Colon) {
            self.advance();
            Some(self.parse_ident()?)
        } else {
            None
        };

        let lbrace_span = self.current_span();
        self.expect(&Token::LBrace)?;

        let mut methods = Vec::new();
        let mut associated_types = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Check for associated type: `T Item = SomeType`
            if self.check(&Token::TypeKeyword) {
                self.advance();
                associated_types.push(self.parse_associated_type_impl()?);
            } else {
                let start = self.current_span().start;
                let method_attrs = self.parse_attributes()?;
                self.expect(&Token::Function)?;
                let func = self.parse_function(false, false, method_attrs)?;
                let end = self.prev_span().end;
                methods.push(Spanned::new(func, Span::new(start, end)));
            }
        }

        self.expect_closing(&Token::RBrace, lbrace_span)?;

        Ok(Impl {
            target_type,
            trait_name,
            generics,
            associated_types,
            methods,
        })
    }

    /// Parse associated type implementation: `T Item = SomeType`
    fn parse_associated_type_impl(&mut self) -> ParseResult<AssociatedTypeImpl> {
        let name = self.parse_ident()?;
        self.expect(&Token::Eq)?;
        let ty = self.parse_type()?;
        Ok(AssociatedTypeImpl { name, ty })
    }
}

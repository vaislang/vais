use vais_ast::*;
use vais_lexer::Token;

use crate::{ParseError, ParseResult, Parser};

impl Parser {
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
    pub(super) fn parse_struct(
        &mut self,
        is_pub: bool,
        attributes: Vec<Attribute>,
    ) -> ParseResult<Struct> {
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
            if self.check(&Token::Function)
                || self.check(&Token::Pub)
                || self.check(&Token::Async)
                || !method_attrs.is_empty()
            {
                let is_method_pub = self.check(&Token::Pub);
                if is_method_pub {
                    self.advance();
                }
                // Check for async method: `A F method_name(...)`
                let is_method_async = if self.check(&Token::Async) {
                    self.advance();
                    true
                } else {
                    false
                };
                self.expect(&Token::Function)?;
                let method = self.parse_function(is_method_pub, is_method_async, method_attrs)?;
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
    pub(super) fn parse_enum(
        &mut self,
        is_pub: bool,
        attributes: Vec<Attribute>,
    ) -> ParseResult<Enum> {
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
            attributes,
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
    pub(super) fn parse_union(&mut self, is_pub: bool) -> ParseResult<Union> {
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

    /// Disambiguate `T Name = Type` (type alias) from `T Name = Trait + Trait` (trait alias).
    /// After `=`, if we see `Ident +`, it's a trait alias.
    pub(super) fn parse_type_or_trait_alias(&mut self, is_pub: bool) -> ParseResult<Item> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;
        self.expect(&Token::Eq)?;

        // Lookahead: save position, check if RHS is `Ident + ...` (trait alias pattern)
        let saved_pos = self.pos;
        let is_trait_alias = if let Some(tok) = self.peek() {
            if matches!(tok.token, Token::Ident(_)) {
                // Peek past the ident to check for `+`
                self.advance(); // consume ident
                let has_plus = self.check(&Token::Plus);
                self.pos = saved_pos; // restore
                self.pending_gt = false;
                has_plus
            } else {
                false
            }
        } else {
            false
        };

        if is_trait_alias {
            let bounds = self.parse_trait_bounds()?;
            Ok(Item::TraitAlias(TraitAlias {
                name,
                generics,
                bounds,
                is_pub,
            }))
        } else {
            let ty = self.parse_type()?;
            Ok(Item::TypeAlias(TypeAlias {
                name,
                generics,
                ty,
                is_pub,
            }))
        }
    }

    /// Parse use statement: `module` or `module/submodule` or `module::submodule`
    /// Also supports selective imports: `module.Item`, `module.{A, B, C}`
    /// Optional semicolon terminator: `module;` or `module.Item;`
    pub(super) fn parse_use(&mut self) -> ParseResult<Use> {
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
    pub(super) fn parse_const_def(
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
    pub(super) fn parse_global_def(&mut self, is_pub: bool) -> ParseResult<GlobalDef> {
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
    pub(super) fn parse_single_extern_function(
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
}

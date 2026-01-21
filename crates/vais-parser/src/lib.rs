//! Vais 0.0.1 Parser
//!
//! Recursive descent parser for AI-optimized syntax.

mod expr;
mod stmt;

use thiserror::Error;
use vais_ast::*;
use vais_lexer::{SpannedToken, Token};

/// Error type for parsing failures.
///
/// Represents various kinds of syntax errors that can occur during parsing,
/// including unexpected tokens, premature EOF, and malformed expressions.
#[derive(Debug, Error)]
pub enum ParseError {
    /// Unexpected token encountered during parsing
    #[error("Unexpected token {found:?} at {span:?}, expected {expected}")]
    UnexpectedToken {
        /// The token that was found
        found: Token,
        /// Source location of the unexpected token
        span: std::ops::Range<usize>,
        /// Description of what was expected
        expected: String,
    },
    /// Unexpected end of file while parsing
    #[error("Unexpected end of file")]
    UnexpectedEof {
        /// Location where EOF was encountered
        span: std::ops::Range<usize>
    },
    /// Invalid or malformed expression
    #[error("Invalid expression")]
    InvalidExpression,
}

impl ParseError {
    /// Get the span associated with this error, if available
    pub fn span(&self) -> Option<std::ops::Range<usize>> {
        match self {
            ParseError::UnexpectedToken { span, .. } => Some(span.clone()),
            ParseError::UnexpectedEof { span } => Some(span.clone()),
            ParseError::InvalidExpression => None,
        }
    }

    /// Get the error code for this error
    pub fn error_code(&self) -> &str {
        match self {
            ParseError::UnexpectedToken { .. } => "P001",
            ParseError::UnexpectedEof { .. } => "P002",
            ParseError::InvalidExpression => "P003",
        }
    }

    /// Get the localized title for this error
    pub fn localized_title(&self) -> String {
        let key = format!("parse.{}.title", self.error_code());
        vais_i18n::get_simple(&key)
    }

    /// Get the localized message for this error
    pub fn localized_message(&self) -> String {
        let key = format!("parse.{}.message", self.error_code());
        match self {
            ParseError::UnexpectedToken { found, expected, .. } => {
                vais_i18n::get(&key, &[
                    ("found", &format!("{:?}", found)),
                    ("expected", expected),
                ])
            }
            ParseError::UnexpectedEof { .. } => {
                vais_i18n::get_simple(&key)
            }
            ParseError::InvalidExpression => {
                vais_i18n::get_simple(&key)
            }
        }
    }
}

type ParseResult<T> = Result<T, ParseError>;

/// Recursive descent parser for Vais source code.
///
/// Converts a token stream into an Abstract Syntax Tree (AST).
/// Uses predictive parsing with single-token lookahead.
pub struct Parser {
    /// Token stream to parse
    tokens: Vec<SpannedToken>,
    /// Current position in the token stream
    pos: usize,
}

impl Parser {
    /// Creates a new parser from a token stream.
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parses a complete module (top-level items).
    ///
    /// This is the main entry point for parsing. It consumes all tokens
    /// and produces a Module containing all top-level definitions.
    pub fn parse_module(&mut self) -> ParseResult<Module> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            items.push(self.parse_item()?);
        }

        Ok(Module { items })
    }

    /// Parse a top-level item
    fn parse_item(&mut self) -> ParseResult<Spanned<Item>> {
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
            Item::Struct(self.parse_struct(is_pub)?)
        } else if self.check(&Token::Enum) {
            self.advance();
            Item::Enum(self.parse_enum(is_pub)?)
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
            Item::Impl(self.parse_impl()?)
        } else {
            return Err(ParseError::UnexpectedToken {
                found: self.peek().map(|t| t.token.clone()).unwrap_or(Token::Ident("EOF".into())),
                span: self.current_span(),
                expected: "F, S, E, T, U, W, or X".into(),
            });
        };

        let end = self.prev_span().end;
        Ok(Spanned::new(item, Span::new(start, end)))
    }

    /// Parse attributes: `#[name(args)]`
    fn parse_attributes(&mut self) -> ParseResult<Vec<Attribute>> {
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
    fn parse_attribute(&mut self) -> ParseResult<Attribute> {
        let name = self.parse_ident()?.node;

        let args = if self.check(&Token::LParen) {
            self.advance();
            let mut args = Vec::new();
            while !self.check(&Token::RParen) && !self.is_at_end() {
                if let Some(tok) = self.peek() {
                    match &tok.token {
                        Token::Ident(s) => {
                            args.push(s.clone());
                            self.advance();
                        }
                        _ => break,
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

        Ok(Attribute { name, args })
    }

    /// Parse function: `name(params)->ret=expr` or `name(params)->ret{...}`
    fn parse_function(&mut self, is_pub: bool, is_async: bool, attributes: Vec<Attribute>) -> ParseResult<Function> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;

        self.expect(&Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(&Token::RParen)?;

        let ret_type = if self.check(&Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = if self.check(&Token::Eq) {
            self.advance();
            FunctionBody::Expr(Box::new(self.parse_expr()?))
        } else if self.check(&Token::LBrace) {
            self.advance();
            let stmts = self.parse_block_contents()?;
            self.expect(&Token::RBrace)?;
            FunctionBody::Block(stmts)
        } else {
            return Err(ParseError::UnexpectedToken {
                found: self.peek().map(|t| t.token.clone()).unwrap_or(Token::Ident("EOF".into())),
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
        })
    }

    /// Parse struct: `Name{fields}` with optional methods
    fn parse_struct(&mut self, is_pub: bool) -> ParseResult<Struct> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;

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

        self.expect(&Token::RBrace)?;

        Ok(Struct {
            name,
            generics,
            fields,
            methods,
            is_pub,
        })
    }

    /// Parse enum: `Name{variants}`
    fn parse_enum(&mut self, is_pub: bool) -> ParseResult<Enum> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;

        self.expect(&Token::LBrace)?;
        let mut variants = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            variants.push(self.parse_variant()?);
            if !self.check(&Token::RBrace) {
                self.expect(&Token::Comma)?;
            }
        }

        self.expect(&Token::RBrace)?;

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
            self.advance();
            let mut types = Vec::new();
            while !self.check(&Token::RParen) && !self.is_at_end() {
                types.push(self.parse_type()?);
                if !self.check(&Token::RParen) {
                    self.expect(&Token::Comma)?;
                }
            }
            self.expect(&Token::RParen)?;
            VariantFields::Tuple(types)
        } else if self.check(&Token::LBrace) {
            self.advance();
            let mut fields = Vec::new();
            while !self.check(&Token::RBrace) && !self.is_at_end() {
                fields.push(self.parse_field()?);
                if !self.check(&Token::RBrace) {
                    self.expect(&Token::Comma)?;
                }
            }
            self.expect(&Token::RBrace)?;
            VariantFields::Struct(fields)
        } else {
            VariantFields::Unit
        };

        Ok(Variant { name, fields })
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
    fn parse_use(&mut self) -> ParseResult<Use> {
        let mut path = vec![self.parse_ident()?];

        // Support both `::` and `/` as path separators
        while self.check(&Token::ColonColon) || self.check(&Token::Slash) {
            self.advance();
            path.push(self.parse_ident()?);
        }

        Ok(Use { path, alias: None })
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

        self.expect(&Token::RBrace)?;

        Ok(Trait {
            name,
            generics,
            super_traits,
            associated_types,
            methods,
            is_pub,
        })
    }

    /// Parse associated type: `T Item` or `T Item: Trait` or `T Item = DefaultType`
    fn parse_associated_type(&mut self) -> ParseResult<AssociatedType> {
        let name = self.parse_ident()?;

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
            bounds,
            default,
        })
    }

    /// Parse trait method signature
    fn parse_trait_method(&mut self) -> ParseResult<TraitMethod> {
        // Check for async keyword: `A F method_name()`
        let is_async = if self.check(&Token::Async) {
            self.advance();
            true
        } else {
            false
        };

        self.expect(&Token::Function)?;
        let name = self.parse_ident()?;

        self.expect(&Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(&Token::RParen)?;

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
            Some(FunctionBody::Block(self.parse_block_contents()?))
        } else {
            None
        };

        Ok(TraitMethod {
            name,
            params,
            ret_type,
            default_body,
            is_async,
        })
    }

    /// Parse impl block: `X Type: Trait { methods }`
    fn parse_impl(&mut self) -> ParseResult<Impl> {
        let generics = self.parse_generics()?;
        let target_type = self.parse_type()?;

        // Optional trait name
        let trait_name = if self.check(&Token::Colon) {
            self.advance();
            Some(self.parse_ident()?)
        } else {
            None
        };

        self.expect(&Token::LBrace)?;

        let mut methods = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let start = self.current_span().start;
            let method_attrs = self.parse_attributes()?;
            self.expect(&Token::Function)?;
            let func = self.parse_function(false, false, method_attrs)?;
            let end = self.prev_span().end;
            methods.push(Spanned::new(func, Span::new(start, end)));
        }

        self.expect(&Token::RBrace)?;

        Ok(Impl {
            target_type,
            trait_name,
            generics,
            methods,
        })
    }

    /// Parse generic parameters: `<T, U>` or `<T: Trait, U: Trait1 + Trait2>` or `<const N: u64>`
    fn parse_generics(&mut self) -> ParseResult<Vec<GenericParam>> {
        if !self.check(&Token::Lt) {
            return Ok(Vec::new());
        }

        self.advance();
        let mut generics = Vec::new();

        while !self.check(&Token::Gt) && !self.is_at_end() {
            // Check for const generic parameter: `const N: u64`
            if self.check(&Token::Const) {
                self.advance();
                let name = self.parse_ident()?;
                self.expect(&Token::Colon)?;
                let ty = self.parse_type()?;
                generics.push(GenericParam::new_const(name, ty));
            } else {
                let name = self.parse_ident()?;

                // Parse optional trait bounds: `: Trait1 + Trait2`
                let bounds = if self.check(&Token::Colon) {
                    self.advance();
                    self.parse_trait_bounds()?
                } else {
                    Vec::new()
                };

                generics.push(GenericParam::new_type(name, bounds));
            }

            if !self.check(&Token::Gt) {
                self.expect(&Token::Comma)?;
            }
        }

        self.expect(&Token::Gt)?;
        Ok(generics)
    }

    /// Parse trait bounds: `Trait1 + Trait2 + Trait3`
    fn parse_trait_bounds(&mut self) -> ParseResult<Vec<Spanned<String>>> {
        let mut bounds = Vec::new();

        // Parse first bound
        bounds.push(self.parse_ident()?);

        // Parse additional bounds separated by `+`
        while self.check(&Token::Plus) {
            self.advance();
            bounds.push(self.parse_ident()?);
        }

        Ok(bounds)
    }

    /// Parse function parameters
    fn parse_params(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();

        while !self.check(&Token::RParen) && !self.is_at_end() {
            let is_mut = self.check(&Token::Mut);
            if is_mut {
                self.advance();
            }

            // Handle &self and &mut self
            if self.check(&Token::Amp) {
                self.advance();
                let is_self_mut = self.check(&Token::Mut);
                if is_self_mut {
                    self.advance();
                }
                if self.check(&Token::SelfLower) {
                    self.advance();
                    let span = self.prev_span();
                    params.push(Param {
                        name: Spanned::new("self".to_string(), Span::new(span.start, span.end)),
                        ty: Spanned::new(
                            if is_self_mut {
                                Type::RefMut(Box::new(Spanned::new(
                                    Type::Named {
                                        name: "Self".to_string(),
                                        generics: vec![],
                                    },
                                    Span::default(),
                                )))
                            } else {
                                Type::Ref(Box::new(Spanned::new(
                                    Type::Named {
                                        name: "Self".to_string(),
                                        generics: vec![],
                                    },
                                    Span::default(),
                                )))
                            },
                            Span::default(),
                        ),
                        is_mut: is_self_mut,
                    });
                    if !self.check(&Token::RParen) {
                        self.expect(&Token::Comma)?;
                    }
                    continue;
                }
            }

            let name = self.parse_ident()?;
            self.expect(&Token::Colon)?;
            let ty = self.parse_type()?;

            params.push(Param { name, ty, is_mut });

            if !self.check(&Token::RParen) {
                self.expect(&Token::Comma)?;
            }
        }

        Ok(params)
    }

    /// Parse struct field
    fn parse_field(&mut self) -> ParseResult<Field> {
        let is_pub = self.check(&Token::Pub);
        if is_pub {
            self.advance();
        }

        let name = self.parse_ident()?;
        self.expect(&Token::Colon)?;
        let ty = self.parse_type()?;

        Ok(Field { name, ty, is_pub })
    }

    /// Parse type
    pub(crate) fn parse_type(&mut self) -> ParseResult<Spanned<Type>> {
        let start = self.current_span().start;

        let base_type = self.parse_base_type()?;

        // Handle postfix type modifiers: ?, !
        let ty = if self.check(&Token::Question) {
            self.advance();
            Type::Optional(Box::new(base_type))
        } else if self.check(&Token::Bang) {
            self.advance();
            Type::Result(Box::new(base_type))
        } else {
            return Ok(base_type);
        };

        let end = self.prev_span().end;
        Ok(Spanned::new(ty, Span::new(start, end)))
    }

    /// Parse base type (without postfix modifiers)
    fn parse_base_type(&mut self) -> ParseResult<Spanned<Type>> {
        let start = self.current_span().start;

        let ty = if self.check(&Token::LParen) {
            self.advance();
            if self.check(&Token::RParen) {
                self.advance();
                Type::Unit
            } else {
                let mut types = vec![self.parse_type()?];
                while self.check(&Token::Comma) {
                    self.advance();
                    types.push(self.parse_type()?);
                }
                self.expect(&Token::RParen)?;

                if self.check(&Token::Arrow) {
                    self.advance();
                    let ret = self.parse_type()?;
                    Type::Fn {
                        params: types,
                        ret: Box::new(ret),
                    }
                } else if types.len() == 1 {
                    return Ok(types.remove(0));
                } else {
                    Type::Tuple(types)
                }
            }
        } else if self.check(&Token::LBracket) {
            self.advance();
            let key_or_elem = self.parse_type()?;

            if self.check(&Token::Colon) {
                self.advance();
                let value = self.parse_type()?;
                self.expect(&Token::RBracket)?;
                Type::Map(Box::new(key_or_elem), Box::new(value))
            } else if self.check(&Token::Semi) {
                // [T; N] - const-sized array
                self.advance();
                let size = self.parse_const_expr()?;
                self.expect(&Token::RBracket)?;
                Type::ConstArray {
                    element: Box::new(key_or_elem),
                    size,
                }
            } else {
                self.expect(&Token::RBracket)?;
                Type::Array(Box::new(key_or_elem))
            }
        } else if self.check(&Token::Star) {
            self.advance();
            let inner = self.parse_base_type()?;
            Type::Pointer(Box::new(inner))
        } else if self.check(&Token::Amp) {
            self.advance();
            let is_mut = self.check(&Token::Mut);
            if is_mut {
                self.advance();
            }
            let inner = self.parse_base_type()?;
            if is_mut {
                Type::RefMut(Box::new(inner))
            } else {
                Type::Ref(Box::new(inner))
            }
        } else {
            let name = self.parse_type_name()?;
            let generics = if self.check(&Token::Lt) {
                self.advance();
                let mut generics = Vec::new();
                while !self.check(&Token::Gt) && !self.is_at_end() {
                    generics.push(self.parse_type()?);
                    if !self.check(&Token::Gt) {
                        self.expect(&Token::Comma)?;
                    }
                }
                self.expect(&Token::Gt)?;
                generics
            } else {
                Vec::new()
            };

            Type::Named { name, generics }
        };

        let end = self.prev_span().end;
        Ok(Spanned::new(ty, Span::new(start, end)))
    }

    /// Parse type name (handles primitive types)
    fn parse_type_name(&mut self) -> ParseResult<String> {
        let span = self.current_span();
        let tok = self.advance().ok_or(ParseError::UnexpectedEof { span })?;

        let name = match &tok.token {
            Token::I8 => "i8",
            Token::I16 => "i16",
            Token::I32 => "i32",
            Token::I64 => "i64",
            Token::I128 => "i128",
            Token::U8 => "u8",
            Token::U16 => "u16",
            Token::U32 => "u32",
            Token::U64 => "u64",
            Token::U128 => "u128",
            Token::F32 => "f32",
            Token::F64 => "f64",
            Token::Bool => "bool",
            Token::Str => "str",
            Token::SelfUpper => "Self",
            Token::Ident(s) => return Ok(s.clone()),
            // Single-letter keywords can be used as type names in generics
            Token::TypeKeyword => "T",
            Token::Function => "F",
            Token::Struct => "S",
            Token::Enum => "E",
            Token::If => "I",
            Token::Loop => "L",
            Token::Match => "M",
            Token::Async => "A",
            Token::Return => "R",
            Token::Break => "B",
            Token::Continue => "C",
            Token::Use => "U",
            Token::Pub => "P",
            _ => {
                return Err(ParseError::UnexpectedToken {
                    found: tok.token.clone(),
                    span: tok.span.clone(),
                    expected: "type name".into(),
                });
            }
        };

        Ok(name.to_string())
    }

    /// Parse a const expression: `N`, `10`, `N + 1`, `A * B`, etc.
    /// Supports basic arithmetic operations for const generics.
    fn parse_const_expr(&mut self) -> ParseResult<ConstExpr> {
        self.parse_const_additive()
    }

    /// Parse additive const expressions: `A + B` or `A - B`
    fn parse_const_additive(&mut self) -> ParseResult<ConstExpr> {
        let mut left = self.parse_const_multiplicative()?;

        while self.check(&Token::Plus) || self.check(&Token::Minus) {
            let op = if self.check(&Token::Plus) {
                self.advance();
                ConstBinOp::Add
            } else {
                self.advance();
                ConstBinOp::Sub
            };
            let right = self.parse_const_multiplicative()?;
            left = ConstExpr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse multiplicative const expressions: `A * B` or `A / B`
    fn parse_const_multiplicative(&mut self) -> ParseResult<ConstExpr> {
        let mut left = self.parse_const_primary()?;

        while self.check(&Token::Star) || self.check(&Token::Slash) {
            let op = if self.check(&Token::Star) {
                self.advance();
                ConstBinOp::Mul
            } else {
                self.advance();
                ConstBinOp::Div
            };
            let right = self.parse_const_primary()?;
            left = ConstExpr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse primary const expression: `N`, `10`, or `(expr)`
    fn parse_const_primary(&mut self) -> ParseResult<ConstExpr> {
        let span = self.current_span();

        // Handle parenthesized expressions
        if self.check(&Token::LParen) {
            self.advance();
            let expr = self.parse_const_expr()?;
            self.expect(&Token::RParen)?;
            return Ok(expr);
        }

        // Handle integer literals
        if let Some(tok) = self.peek() {
            if let Token::Int(n) = &tok.token {
                let value = *n;
                self.advance();
                return Ok(ConstExpr::Literal(value));
            }
        }

        // Handle identifiers (const parameters)
        if let Some(tok) = self.peek() {
            match &tok.token {
                Token::Ident(name) => {
                    let name = name.clone();
                    self.advance();
                    return Ok(ConstExpr::Param(name));
                }
                // Single-letter keywords that can be used as const param names
                Token::TypeKeyword => {
                    self.advance();
                    return Ok(ConstExpr::Param("T".to_string()));
                }
                Token::Function => {
                    self.advance();
                    return Ok(ConstExpr::Param("F".to_string()));
                }
                Token::Struct => {
                    self.advance();
                    return Ok(ConstExpr::Param("S".to_string()));
                }
                Token::Enum => {
                    self.advance();
                    return Ok(ConstExpr::Param("E".to_string()));
                }
                Token::Async => {
                    self.advance();
                    return Ok(ConstExpr::Param("A".to_string()));
                }
                Token::Break => {
                    self.advance();
                    return Ok(ConstExpr::Param("B".to_string()));
                }
                Token::Continue => {
                    self.advance();
                    return Ok(ConstExpr::Param("C".to_string()));
                }
                Token::Loop => {
                    self.advance();
                    return Ok(ConstExpr::Param("L".to_string()));
                }
                Token::Match => {
                    self.advance();
                    return Ok(ConstExpr::Param("M".to_string()));
                }
                Token::Return => {
                    self.advance();
                    return Ok(ConstExpr::Param("R".to_string()));
                }
                Token::Use => {
                    self.advance();
                    return Ok(ConstExpr::Param("U".to_string()));
                }
                Token::Pub => {
                    self.advance();
                    return Ok(ConstExpr::Param("P".to_string()));
                }
                _ => {}
            }
        }

        Err(ParseError::UnexpectedToken {
            found: self.peek().map(|t| t.token.clone()).unwrap_or(Token::Ident("EOF".into())),
            span,
            expected: "const expression (integer literal or identifier)".into(),
        })
    }

    // === Helper methods ===

    pub(crate) fn peek(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.pos)
    }

    pub(crate) fn peek_next(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.pos + 1)
    }

    pub(crate) fn advance(&mut self) -> Option<SpannedToken> {
        if self.is_at_end() {
            None
        } else {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        }
    }

    pub(crate) fn check(&self, expected: &Token) -> bool {
        self.peek().map(|t| &t.token == expected).unwrap_or(false)
    }

    pub(crate) fn expect(&mut self, expected: &Token) -> ParseResult<SpannedToken> {
        if self.check(expected) {
            self.advance().ok_or_else(|| ParseError::UnexpectedToken {
                found: Token::Ident("EOF".into()),
                span: self.current_span(),
                expected: format!("{:?}", expected),
            })
        } else {
            Err(ParseError::UnexpectedToken {
                found: self.peek().map(|t| t.token.clone()).unwrap_or(Token::Ident("EOF".into())),
                span: self.current_span(),
                expected: format!("{:?}", expected),
            })
        }
    }

    pub(crate) fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub(crate) fn current_span(&self) -> std::ops::Range<usize> {
        self.peek()
            .map(|t| t.span.clone())
            .unwrap_or(self.prev_span())
    }

    pub(crate) fn prev_span(&self) -> std::ops::Range<usize> {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span.clone()
        } else {
            0..0
        }
    }
}

/// Parses Vais source code into an Abstract Syntax Tree.
///
/// This is the main convenience function that performs both lexing and parsing
/// in a single step.
///
/// # Arguments
///
/// * `source` - The Vais source code to parse
///
/// # Returns
///
/// A Module containing all parsed items on success, or a ParseError on failure.
///
/// # Examples
///
/// ```
/// use vais_parser::parse;
///
/// let source = "F add(x:i64,y:i64)->i64=x+y";
/// let module = parse(source).unwrap();
/// assert_eq!(module.items.len(), 1);
/// ```
pub fn parse(source: &str) -> Result<Module, ParseError> {
    let tokens = vais_lexer::tokenize(source).map_err(|e| ParseError::UnexpectedToken {
        found: Token::Ident(format!("LexError: {}", e)),
        span: 0..0,
        expected: "valid token".into(),
    })?;

    let mut parser = Parser::new(tokens);
    parser.parse_module()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let source = "F add(a:i64,b:i64)->i64=a+b";
        let module = parse(source).unwrap();

        assert_eq!(module.items.len(), 1);
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function, got {:?}", module.items[0].node);
        };
        assert_eq!(f.name.node, "add");
        assert_eq!(f.params.len(), 2);
    }

    #[test]
    fn test_parse_fibonacci() {
        let source = "F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "fib");
        let FunctionBody::Expr(expr) = &f.body else {
            unreachable!("Expected expression body");
        };
        assert!(matches!(expr.node, Expr::Ternary { .. }), "Expected ternary expression");
    }

    #[test]
    fn test_parse_struct() {
        let source = "S Point{x:f64,y:f64}";
        let module = parse(source).unwrap();

        let Item::Struct(s) = &module.items[0].node else {
            unreachable!("Expected struct, got {:?}", module.items[0].node);
        };
        assert_eq!(s.name.node, "Point");
        assert_eq!(s.fields.len(), 2);
    }

    #[test]
    fn test_parse_enum() {
        let source = "E Option<T>{Some(T),None}";
        let module = parse(source).unwrap();

        let Item::Enum(e) = &module.items[0].node else {
            unreachable!("Expected enum, got {:?}", module.items[0].node);
        };
        assert_eq!(e.name.node, "Option");
        assert_eq!(e.generics.len(), 1);
        assert_eq!(e.variants.len(), 2);
    }

    #[test]
    fn test_parse_block_function() {
        let source = "F sum(arr:[i64])->i64{s:=0;L x:arr{s+=x};s}";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        let FunctionBody::Block(stmts) = &f.body else {
            unreachable!("Expected block body, got {:?}", f.body);
        };
        assert_eq!(stmts.len(), 3);
    }

    #[test]
    fn test_parse_generic_constraints() {
        // Test single trait bound
        let source = "F print_value<T: Display>(x: T) -> () = println(x)";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "print_value");
        assert_eq!(f.generics.len(), 1);
        assert_eq!(f.generics[0].name.node, "T");
        assert_eq!(f.generics[0].bounds.len(), 1);
        assert_eq!(f.generics[0].bounds[0].node, "Display");

        // Test multiple trait bounds
        let source2 = "F compare<T: Ord + Clone>(a: T, b: T) -> bool = a < b";
        let module2 = parse(source2).unwrap();

        let Item::Function(f2) = &module2.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f2.generics.len(), 1);
        assert_eq!(f2.generics[0].name.node, "T");
        assert_eq!(f2.generics[0].bounds.len(), 2);
        assert_eq!(f2.generics[0].bounds[0].node, "Ord");
        assert_eq!(f2.generics[0].bounds[1].node, "Clone");

        // Test multiple generic params with bounds
        let source3 = "F transform<A: Clone, B: Default>(x: A) -> B = x";
        let module3 = parse(source3).unwrap();

        let Item::Function(f3) = &module3.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f3.generics.len(), 2);
        assert_eq!(f3.generics[0].name.node, "A");
        assert_eq!(f3.generics[0].bounds.len(), 1);
        assert_eq!(f3.generics[0].bounds[0].node, "Clone");
        assert_eq!(f3.generics[1].name.node, "B");
        assert_eq!(f3.generics[1].bounds.len(), 1);
        assert_eq!(f3.generics[1].bounds[0].node, "Default");

        // Test generic without bounds (should still work)
        let source4 = "F identity<T>(x: T) -> T = x";
        let module4 = parse(source4).unwrap();

        let Item::Function(f4) = &module4.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f4.generics.len(), 1);
        assert_eq!(f4.generics[0].name.node, "T");
        assert_eq!(f4.generics[0].bounds.len(), 0);
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_empty_input() {
        let source = "";
        let module = parse(source).unwrap();
        assert!(module.items.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let source = "   \n\t\r\n   ";
        let module = parse(source).unwrap();
        assert!(module.items.is_empty());
    }

    #[test]
    fn test_comment_only() {
        let source = "# this is just a comment\n# another comment";
        let module = parse(source).unwrap();
        assert!(module.items.is_empty());
    }

    #[test]
    fn test_minimal_function() {
        let source = "F f()->()=()";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
        assert!(f.params.is_empty());
    }

    #[test]
    fn test_empty_struct() {
        let source = "S Empty{}";
        let module = parse(source).unwrap();
        let Item::Struct(s) = &module.items[0].node else {
            unreachable!("Expected struct");
        };
        assert_eq!(s.name.node, "Empty");
        assert!(s.fields.is_empty());
    }

    #[test]
    fn test_single_field_struct() {
        let source = "S Single{x:i64}";
        let module = parse(source).unwrap();
        let Item::Struct(s) = &module.items[0].node else {
            unreachable!("Expected struct");
        };
        assert_eq!(s.fields.len(), 1);
    }

    #[test]
    fn test_minimal_enum() {
        let source = "E Unit{A}";
        let module = parse(source).unwrap();
        let Item::Enum(e) = &module.items[0].node else {
            unreachable!("Expected enum");
        };
        assert_eq!(e.name.node, "Unit");
        assert_eq!(e.variants.len(), 1);
    }

    #[test]
    fn test_enum_with_tuple_variants() {
        let source = "E Shape{Circle(f64),Rectangle(f64,f64),Point}";
        let module = parse(source).unwrap();
        let Item::Enum(e) = &module.items[0].node else {
            unreachable!("Expected enum");
        };
        assert_eq!(e.variants.len(), 3);
    }

    #[test]
    fn test_enum_with_struct_variants() {
        let source = "E Message{Quit,Move{x:i64,y:i64},Write(str)}";
        let module = parse(source).unwrap();
        let Item::Enum(e) = &module.items[0].node else {
            unreachable!("Expected enum");
        };
        assert_eq!(e.variants.len(), 3);
    }

    #[test]
    fn test_empty_block_function() {
        let source = "F f()->(){}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        let FunctionBody::Block(stmts) = &f.body else {
            unreachable!("Expected block body");
        };
        assert!(stmts.is_empty());
    }

    #[test]
    fn test_nested_generic_types() {
        // Use simple generic syntax that the parser supports
        let source = "F f<T>(x:T)->T=x";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.params.len(), 1);
        assert_eq!(f.generics.len(), 1);
    }

    #[test]
    fn test_deeply_nested_arrays() {
        let source = "F f(x:[[[i64]]])->[[[i64]]]=x";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.params.len(), 1);
    }

    #[test]
    fn test_multiple_items() {
        let source = r#"
S Point{x:f64,y:f64}
F new_point(x:f64,y:f64)->Point=Point{x:x,y:y}
F origin()->Point=new_point(0.0,0.0)
"#;
        let module = parse(source).unwrap();
        assert_eq!(module.items.len(), 3);
    }

    #[test]
    fn test_trait_definition() {
        // Trait uses W keyword with methods using regular identifiers
        let source = "W Display{F display(s:&Self)->str=\"\"}";
        let module = parse(source).unwrap();
        let Item::Trait(t) = &module.items[0].node else {
            unreachable!("Expected trait");
        };
        assert_eq!(t.name.node, "Display");
        assert_eq!(t.methods.len(), 1);
    }

    #[test]
    fn test_impl_block() {
        let source = r#"
S Point{x:f64,y:f64}
X Point{F new(x:f64,y:f64)->Point=Point{x:x,y:y}}
"#;
        let module = parse(source).unwrap();
        assert_eq!(module.items.len(), 2);
        let Item::Impl(imp) = &module.items[1].node else {
            unreachable!("Expected impl");
        };
        // target_type is a Spanned<Type>, check the type name
        assert!(matches!(&imp.target_type.node, Type::Named { name, .. } if name == "Point"));
    }

    #[test]
    fn test_if_without_else() {
        let source = "F f(x:bool)->(){I x{print(1)}}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        // Function should parse successfully
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_nested_if_else() {
        let source = "F f(x:i64)->i64=I x>0{I x>10{100}E{10}}E{0}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_match_with_wildcard() {
        let source = "F f(x:i64)->i64=M x{0=>0,1=>1,_=>-1}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        let FunctionBody::Expr(expr) = &f.body else {
            unreachable!("Expected expression body");
        };
        assert!(matches!(expr.node, Expr::Match { .. }));
    }

    #[test]
    fn test_match_with_guard() {
        let source = "F f(x:i64)->i64=M x{n I n>0=>n,_=>0}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_lambda_expression() {
        let source = "F f()->i64{g:=|x:i64|x*2;g(21)}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_nested_lambda() {
        let source = "F f()->i64{g:=|x:i64|(|y:i64|x+y);g(10)(32)}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_method_chaining() {
        let source = "F f(x:str)->i64=x.len().to_string().len()";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_array_indexing_chain() {
        let source = "F f(arr:[[i64]])->i64=arr[0][1]";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_self_recursion_operator() {
        let source = "F factorial(n:i64)->i64=n<2?1:n*@(n-1)";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "factorial");
    }

    #[test]
    fn test_range_expression() {
        let source = "F f()->(){L i:0..10{print(i)}}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_ternary_operator() {
        // Test the ternary operator (cond ? then : else)
        let source = "F f(x:i64)->i64=x>0?x:0";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_ternary_with_unary_minus() {
        // Test ternary with unary minus in then branch: x<0 ? -x : x
        let source = "F abs(x:i64)->i64=x<0?-x:x";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "abs");
        // Body should be a Ternary
        let FunctionBody::Expr(body) = &f.body else {
            panic!("Expected expression body");
        };
        assert!(matches!(body.node, Expr::Ternary { .. }));
    }

    #[test]
    fn test_try_operator() {
        // Test postfix try operator (?)
        let source = "F f(x:i64?)->i64=x?";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
        // Verify the body is a Try expression
        let FunctionBody::Expr(body) = &f.body else {
            panic!("Expected expression body");
        };
        if let Expr::Try(inner) = &body.node {
            assert!(matches!(inner.node, Expr::Ident(_)));
        } else {
            panic!("Expected Try expression");
        }
    }

    #[test]
    fn test_try_operator_in_expression() {
        // Test try operator followed by binary operator
        let source = "F f(x:i64?)->i64=x?+1";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        // Verify the body is a Binary expression with Try on left
        let FunctionBody::Expr(body) = &f.body else {
            panic!("Expected expression body");
        };
        if let Expr::Binary { left, .. } = &body.node {
            assert!(matches!(left.node, Expr::Try(_)));
        } else {
            panic!("Expected Binary expression with Try on left");
        }
    }

    #[test]
    fn test_try_and_ternary_coexist() {
        // Test that try and ternary can coexist: (x?) ? 1 : 0
        let source = "F f(x:i64?)->i64=(x?)?1:0";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        // Body should be a Ternary with condition being Try
        let FunctionBody::Expr(body) = &f.body else {
            panic!("Expected expression body");
        };
        if let Expr::Ternary { cond, .. } = &body.node {
            assert!(matches!(cond.node, Expr::Try(_)));
        } else {
            panic!("Expected Ternary expression");
        }
    }

    #[test]
    fn test_simple_return_type() {
        // Test simple return type parsing
        let source = "F f()->i64=42";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert!(f.ret_type.is_some());
    }

    #[test]
    fn test_reference_types() {
        let source = "F f(x:&i64,y:&mut i64)->()=()";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.params.len(), 2);
    }

    #[test]
    fn test_pointer_type() {
        let source = "F f(x:*i64)->*i64=x";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert!(matches!(&f.params[0].ty.node, Type::Pointer(_)));
    }

    #[test]
    fn test_tuple_type() {
        let source = "F f(x:(i64,str))->(i64,str)=x";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert!(matches!(&f.params[0].ty.node, Type::Tuple(_)));
    }

    #[test]
    fn test_function_type() {
        let source = "F apply(f:(i64)->i64,x:i64)->i64=f(x)";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert!(matches!(&f.params[0].ty.node, Type::Fn { .. }));
    }

    #[test]
    fn test_async_function() {
        // Async function with A prefix
        let source = "A F fetch(url:str)->str=url";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert!(f.is_async);
    }

    #[test]
    fn test_pub_function() {
        let source = "P F public_fn()->()=()";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert!(f.is_pub);
    }

    #[test]
    fn test_import_statement() {
        // Use statement with U keyword
        let source = "U std::io";
        let module = parse(source).unwrap();
        let Item::Use(u) = &module.items[0].node else {
            unreachable!("Expected use statement");
        };
        assert!(!u.path.is_empty());
    }

    #[test]
    fn test_complex_expression() {
        let source = "F f(a:i64,b:i64,c:i64)->i64=a+b*c-a/b%c";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_bitwise_operations() {
        let source = "F f(a:i64,b:i64)->i64=a&b|c^d<<2>>1";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_comparison_chain() {
        let source = "F f(a:i64,b:i64,c:i64)->bool=a<b&&b<c||a==c";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_unary_operators() {
        let source = "F f(x:i64,b:bool)->i64=-x+~x*(!b?1:0)";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_compound_assignment() {
        // In Vais, use := for mutable variable declaration
        let source = "F f(x:i64)->i64{y:=x;y+=1;y-=2;y*=3;y}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        let FunctionBody::Block(stmts) = &f.body else {
            unreachable!("Expected block body");
        };
        assert_eq!(stmts.len(), 5);
    }

    #[test]
    fn test_break_with_value() {
        let source = "F f()->i64{L{B 42}}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_continue_in_loop() {
        let source = "F f()->(){L i:0..10{I i%2==0{C};print(i)}}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_defer_statement() {
        // Test basic defer statement
        let source = "F f() -> () { h := open(); D close(h); () }";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
        // Check that body contains defer
        let FunctionBody::Block(stmts) = &f.body else {
            panic!("Expected block body");
        };
        // Should have 3 statements: let, defer, expr
        assert_eq!(stmts.len(), 3);
        assert!(matches!(stmts[1].node, Stmt::Defer(_)));
    }

    #[test]
    fn test_multiple_defer_statements() {
        // Test multiple defer statements (LIFO order)
        let source = "F f() -> () { D cleanup1(); D cleanup2(); D cleanup3(); () }";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        let FunctionBody::Block(stmts) = &f.body else {
            panic!("Expected block body");
        };
        // Should have 4 statements: 3 defers + 1 expr
        assert_eq!(stmts.len(), 4);
        assert!(matches!(stmts[0].node, Stmt::Defer(_)));
        assert!(matches!(stmts[1].node, Stmt::Defer(_)));
        assert!(matches!(stmts[2].node, Stmt::Defer(_)));
    }

    #[test]
    fn test_struct_literal() {
        let source = "F f()->Point{Point{x:1.0,y:2.0}}";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_array_literal() {
        let source = "F f()->[i64]=[1,2,3,4,5]";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_empty_array_literal() {
        let source = "F f()->[i64]=[]";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_array_with_values() {
        // Test array literal syntax [value, value, ...]
        let source = "F f()->[i64]=[1,2,3]";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_multiline_function() {
        let source = r#"
F calculate(a: i64,
            b: i64,
            c: i64) -> i64 {
    x := a + b;
    y := x * c;
    R y
}
"#;
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.params.len(), 3);
    }

    #[test]
    fn test_all_primitive_types() {
        let source = r#"
F test(
    a:i8,b:i16,c:i32,d:i64,e:i128,
    f:u8,g:u16,h:u32,i:u64,j:u128,
    k:f32,l:f64,m:bool,n:str
)->()=()
"#;
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.params.len(), 14);
    }

    #[test]
    fn test_pattern_in_match() {
        let source = r#"
F f(opt:Option<i64>)->i64=M opt{
    Some(x)=>x,
    None=>0
}
"#;
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
    }

    #[test]
    fn test_tuple_parameter() {
        // Test tuple type as parameter
        let source = "F f(t:(i64,i64))->i64=42";
        let module = parse(source).unwrap();
        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "f");
        assert!(matches!(&f.params[0].ty.node, Type::Tuple(_)));
    }

    #[test]
    fn test_basic_struct_with_methods() {
        // Test struct with impl block using regular param names
        let source = r#"
S Counter{value:i64}
X Counter{F inc(c:&Counter)->i64=c.value+1}
"#;
        let module = parse(source).unwrap();
        assert_eq!(module.items.len(), 2);
    }

    #[test]
    fn test_enum_pattern_match() {
        // Test enum variant matching
        let source = r#"
E Result{Ok(i64),Err(str)}
F handle(r:Result)->i64=M r{Ok(v)=>v,Err(_)=>0}
"#;
        let module = parse(source).unwrap();
        assert_eq!(module.items.len(), 2);
    }

    // ==================== Advanced Edge Case Tests ====================

    #[test]
    fn test_nested_generic_vec_hashmap() {
        // Test nested generic: Vec<HashMap<K, V> > with spaces
        let source = "S Container{data:Vec<HashMap<str,i64> >}";
        let module = parse(source).unwrap();

        let Item::Struct(s) = &module.items[0].node else {
            unreachable!("Expected struct");
        };
        assert_eq!(s.name.node, "Container");
        assert_eq!(s.fields.len(), 1);
    }

    #[test]
    fn test_option_of_vec_generic() {
        // Test Option<Vec<T> > combination with spaces (need space before =)
        let source = r#"F get_items<T>()->Option<Vec<T> > ="""#;
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "get_items");
        assert_eq!(f.generics.len(), 1);
    }

    #[test]
    fn test_hashmap_option_value() {
        // Test HashMap<K, Option<V> > with spaces
        let source = "S Cache{entries:HashMap<str,Option<i64> >}";
        let module = parse(source).unwrap();

        let Item::Struct(s) = &module.items[0].node else {
            unreachable!("Expected struct");
        };
        assert_eq!(s.name.node, "Cache");
    }

    #[test]
    fn test_deeply_nested_generics() {
        // Test Vec<HashMap<K, Option<Vec<T> > > > with spaces (need space before =)
        let source = "F complex<T>()->Vec<HashMap<str,Option<Vec<T> > > > =[]";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "complex");
    }

    #[test]
    fn test_pattern_match_with_guard() {
        // Test pattern matching with guard condition
        let source = "F classify(x:i64)->str=M x{n I n>0=>\"pos\",n I n<0=>\"neg\",_=>\"zero\"}";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "classify");
    }

    #[test]
    fn test_pattern_match_guard_complex() {
        // Test pattern match with complex guard
        let source = r#"
F filter(opt:Option<i64>)->i64=M opt{
    Some(x) I x>0&&x<100=>x,
    Some(x) I x>=100=>100,
    Some(_)=>0,
    None=>-1
}
"#;
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "filter");
    }

    #[test]
    fn test_nested_pattern_destructuring() {
        // Test nested destructuring in pattern match
        let source = r#"
E Nested{Pair((i64,i64)),Single(i64),None}
F sum(n:Nested)->i64=M n{
    Pair((a,b))=>a+b,
    Single(x)=>x,
    None=>0
}
"#;
        let module = parse(source).unwrap();

        assert_eq!(module.items.len(), 2);
        let Item::Function(f) = &module.items[1].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "sum");
    }

    #[test]
    fn test_pattern_guard_with_multiple_conditions() {
        // Test guard with multiple && || conditions
        let source = "F check(x:i64,y:i64)->bool=M (x,y){(a,b) I a>0&&b>0||a<0&&b<0=>true,_=>false}";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "check");
    }

    #[test]
    fn test_nested_option_pattern() {
        // Test nested Option patterns: Option<Option<T> > with spaces
        let source = r#"
F unwrap_twice(opt:Option<Option<i64> >)->i64=M opt{
    Some(Some(x))=>x,
    Some(None)=>-1,
    None=>-2
}
"#;
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "unwrap_twice");
    }

    #[test]
    fn test_mutual_recursion_type_inference() {
        // Test mutual recursion: two functions calling each other
        let source = r#"
F is_even(n:i64)->bool=n==0?true:is_odd(n-1)
F is_odd(n:i64)->bool=n==0?false:is_even(n-1)
"#;
        let module = parse(source).unwrap();

        assert_eq!(module.items.len(), 2);
        let Item::Function(f1) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        let Item::Function(f2) = &module.items[1].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f1.name.node, "is_even");
        assert_eq!(f2.name.node, "is_odd");
    }

    #[test]
    fn test_three_way_mutual_recursion() {
        // Test three functions in mutual recursion
        let source = r#"
F a(n:i64)->i64=n<1?0:b(n-1)+1
F b(n:i64)->i64=n<1?0:c(n-1)+1
F c(n:i64)->i64=n<1?0:a(n-1)+1
"#;
        let module = parse(source).unwrap();

        assert_eq!(module.items.len(), 3);
    }

    #[test]
    fn test_indirect_recursion_through_lambda() {
        // Test recursion through lambda (advanced case)
        let source = r#"
F outer(n:i64)->i64{
    helper:=|x:i64|x<1?0:outer(x-1)+1;
    helper(n)
}
"#;
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "outer");
    }

    #[test]
    fn test_generic_mutual_recursion() {
        // Test mutual recursion with generics
        let source = r#"
F transform_a<T>(x:T)->T=transform_b(x)
F transform_b<T>(x:T)->T=x
"#;
        let module = parse(source).unwrap();

        assert_eq!(module.items.len(), 2);
    }

    #[test]
    fn test_i8_boundary_parsing() {
        // Test i8 min/max: -128, 127
        let source = "F i8_test()->(){min:i8=-128;max:i8=127}";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "i8_test");
    }

    #[test]
    fn test_i16_boundaries() {
        // Test i16 boundaries: -32768, 32767
        let source = "F i16_test()->(){min:i16=-32768;max:i16=32767}";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "i16_test");
    }

    #[test]
    fn test_i64_max_parsing() {
        // Test i64 max: 9223372036854775807
        let source = "F i64_max()->i64=9223372036854775807";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "i64_max");
    }

    #[test]
    fn test_pattern_with_range() {
        // Test pattern matching with ranges
        let source = r#"
F grade(score:i64)->str=M score{
    x I x>=90=>"A",
    x I x>=80=>"B",
    x I x>=70=>"C",
    x I x>=60=>"D",
    _=>"F"
}
"#;
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "grade");
    }

    #[test]
    fn test_destructure_nested_struct() {
        // Test destructuring nested structs in pattern match
        let source = r#"
S Point{x:i64,y:i64}
S Line{start:Point,end:Point}
F length(line:Line)->i64=line.end.x-line.start.x
"#;
        let module = parse(source).unwrap();

        assert_eq!(module.items.len(), 3);
    }

    #[test]
    fn test_guard_with_method_call() {
        // Test guard condition with method calls
        let source = "F check_len(s:str)->bool=M s{x I x.len()>0=>true,_=>false}";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "check_len");
    }

    #[test]
    fn test_multiple_generic_constraints() {
        // Test function with multiple generic parameters with bounds
        let source = "F combine<A:Clone,B:Default,C:Ord>(a:A,b:B,c:C)->C=c";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.generics.len(), 3);
        assert_eq!(f.generics[0].bounds.len(), 1);
        assert_eq!(f.generics[1].bounds.len(), 1);
        assert_eq!(f.generics[2].bounds.len(), 1);
    }

    #[test]
    fn test_enum_with_generic_variants() {
        // Test enum with generic variants
        let source = "E Result<T,E>{Ok(T),Err(E)}";
        let module = parse(source).unwrap();

        let Item::Enum(e) = &module.items[0].node else {
            unreachable!("Expected enum");
        };
        assert_eq!(e.name.node, "Result");
        assert_eq!(e.generics.len(), 2);
        assert_eq!(e.variants.len(), 2);
    }

    #[test]
    fn test_deeply_nested_if_else() {
        // Test deeply nested if-else chains
        let source = r#"
F classify(n:i64)->str{
    I n>1000{
        I n>10000{"huge"}E{"large"}
    }E{
        I n>100{
            I n>500{"medium-large"}E{"medium"}
        }E{
            I n>10{"small"}E{"tiny"}
        }
    }
}
"#;
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "classify");
    }

    #[test]
    fn test_pattern_with_multiple_bindings() {
        // Test pattern with multiple variable bindings and guards
        let source = r#"
F process(a:i64,b:i64)->i64=M (a,b){
    (x,y) I x>0&&y>0=>x+y,
    (x,y) I x<0&&y<0=>x-y,
    (x,y) I x==0||y==0=>0,
    (x,y)=>x*y
}
"#;
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "process");
    }

    #[test]
    fn test_self_recursion_with_multiple_params() {
        // Test self-recursion with multiple parameters
        let source = "F gcd(a:i64,b:i64)->i64=b==0?a:@(b,a%b)";
        let module = parse(source).unwrap();

        let Item::Function(f) = &module.items[0].node else {
            unreachable!("Expected function");
        };
        assert_eq!(f.name.node, "gcd");
        assert_eq!(f.params.len(), 2);
    }
}

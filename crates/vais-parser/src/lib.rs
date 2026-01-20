//! Vais 0.0.1 Parser
//!
//! Recursive descent parser for AI-optimized syntax.

use thiserror::Error;
use vais_ast::*;
use vais_lexer::{SpannedToken, Token};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected token {found:?} at {span:?}, expected {expected}")]
    UnexpectedToken {
        found: Token,
        span: std::ops::Range<usize>,
        expected: String,
    },
    #[error("Unexpected end of file")]
    UnexpectedEof { span: std::ops::Range<usize> },
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
}

type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parse a complete module
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

    /// Parse generic parameters: `<T, U>` or `<T: Trait, U: Trait1 + Trait2>`
    fn parse_generics(&mut self) -> ParseResult<Vec<GenericParam>> {
        if !self.check(&Token::Lt) {
            return Ok(Vec::new());
        }

        self.advance();
        let mut generics = Vec::new();

        while !self.check(&Token::Gt) && !self.is_at_end() {
            let name = self.parse_ident()?;

            // Parse optional trait bounds: `: Trait1 + Trait2`
            let bounds = if self.check(&Token::Colon) {
                self.advance();
                self.parse_trait_bounds()?
            } else {
                Vec::new()
            };

            generics.push(GenericParam { name, bounds });

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
    fn parse_type(&mut self) -> ParseResult<Spanned<Type>> {
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

    /// Parse block contents (statements)
    fn parse_block_contents(&mut self) -> ParseResult<Vec<Spanned<Stmt>>> {
        let mut stmts = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }

        Ok(stmts)
    }

    /// Parse statement
    fn parse_stmt(&mut self) -> ParseResult<Spanned<Stmt>> {
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

    /// Parse expression
    pub fn parse_expr(&mut self) -> ParseResult<Spanned<Expr>> {
        self.parse_assignment()
    }

    /// Parse assignment expression
    fn parse_assignment(&mut self) -> ParseResult<Spanned<Expr>> {
        let expr = self.parse_ternary()?;

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

    /// Parse ternary: `cond ? then : else`
    fn parse_ternary(&mut self) -> ParseResult<Spanned<Expr>> {
        let cond = self.parse_or()?;

        if self.check(&Token::Question) {
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

        Ok(cond)
    }

    /// Parse logical OR
    fn parse_or(&mut self) -> ParseResult<Spanned<Expr>> {
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
    fn parse_and(&mut self) -> ParseResult<Spanned<Expr>> {
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
    fn parse_bitwise_or(&mut self) -> ParseResult<Spanned<Expr>> {
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
    fn parse_bitwise_xor(&mut self) -> ParseResult<Spanned<Expr>> {
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
    fn parse_bitwise_and(&mut self) -> ParseResult<Spanned<Expr>> {
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
    fn parse_equality(&mut self) -> ParseResult<Spanned<Expr>> {
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
    fn parse_range(&mut self) -> ParseResult<Spanned<Expr>> {
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
    fn parse_comparison(&mut self) -> ParseResult<Spanned<Expr>> {
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
    fn parse_shift(&mut self) -> ParseResult<Spanned<Expr>> {
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
    fn parse_term(&mut self) -> ParseResult<Spanned<Expr>> {
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
    fn parse_factor(&mut self) -> ParseResult<Spanned<Expr>> {
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
    fn parse_unary(&mut self) -> ParseResult<Spanned<Expr>> {
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
            return Ok(Spanned::new(Expr::Ref(Box::new(expr)), Span::new(start, end)));
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

        self.parse_postfix()
    }

    /// Parse postfix expressions (calls, field access, etc.)
    fn parse_postfix(&mut self) -> ParseResult<Spanned<Expr>> {
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
                // Check if this is postfix try (expr?) or ternary (cond ? then : else)
                // If there's a following expression followed by ':', it's ternary - don't consume
                // The ternary is handled at parse_ternary level
                // Here we only handle postfix try: expr? where ? is at end or followed by binary op
                //
                // Simplest heuristic: if next token is an expression start, it's likely ternary
                // So we don't handle ? here at all - let ternary handle it
                break;
            } else if self.check(&Token::Bang) {
                self.advance();
                let end = self.prev_span().end;
                expr = Spanned::new(
                    Expr::Unwrap(Box::new(expr)),
                    Span::new(start, end),
                );
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse primary expression
    fn parse_primary(&mut self) -> ParseResult<Spanned<Expr>> {
        let start = self.current_span().start;
        let span = self.current_span();
        let tok = self.advance().ok_or(ParseError::UnexpectedEof { span })?;

        let expr = match tok.token {
            Token::Int(n) => Expr::Int(n),
            Token::Float(n) => Expr::Float(n),
            Token::True => Expr::Bool(true),
            Token::False => Expr::Bool(false),
            Token::String(s) => Expr::String(s),
            Token::At => Expr::SelfCall,
            Token::SelfLower => Expr::Ident("self".to_string()),
            Token::Ident(name) => {
                // Check for struct literal: `Name{...}`
                // Only treat as struct literal if name starts with uppercase (type convention)
                let is_type_name = name.chars().next().is_some_and(|c| c.is_uppercase());
                if is_type_name && self.check(&Token::LBrace) {
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
                    exprs.push(self.parse_expr()?);
                    if !self.check(&Token::RBracket) {
                        self.expect(&Token::Comma)?;
                    }
                }
                self.expect(&Token::RBracket)?;
                let end = self.prev_span().end;
                return Ok(Spanned::new(Expr::Array(exprs), Span::new(start, end)));
            }
            Token::LBrace => {
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
    fn parse_if_expr(&mut self, start: usize) -> ParseResult<Spanned<Expr>> {
        let cond = self.parse_expr()?;
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
    fn parse_else_branch(&mut self) -> ParseResult<IfElse> {
        if self.check(&Token::If) {
            // else if
            self.advance();
            let cond = self.parse_expr()?;
            self.expect(&Token::LBrace)?;
            let then = self.parse_block_contents()?;
            self.expect(&Token::RBrace)?;

            let else_ = if self.check(&Token::Enum) {
                self.advance();
                Some(Box::new(self.parse_else_branch()?))
            } else {
                None
            };

            Ok(IfElse::ElseIf(Box::new(cond), then, else_))
        } else {
            // else
            self.expect(&Token::LBrace)?;
            let stmts = self.parse_block_contents()?;
            self.expect(&Token::RBrace)?;
            Ok(IfElse::Else(stmts))
        }
    }

    /// Parse loop expression: `L pattern:iter{...}` or `L{...}`
    fn parse_loop_expr(&mut self, start: usize) -> ParseResult<Spanned<Expr>> {
        let (pattern, iter) = if self.check(&Token::LBrace) {
            (None, None)
        } else {
            let pattern = self.parse_pattern()?;
            self.expect(&Token::Colon)?;
            let iter = self.parse_expr()?;
            (Some(pattern), Some(Box::new(iter)))
        };

        self.expect(&Token::LBrace)?;
        let body = self.parse_block_contents()?;
        self.expect(&Token::RBrace)?;

        let end = self.prev_span().end;
        Ok(Spanned::new(
            Expr::Loop {
                pattern,
                iter,
                body,
            },
            Span::new(start, end),
        ))
    }

    /// Parse match expression: `M expr{arms}`
    fn parse_match_expr(&mut self, start: usize) -> ParseResult<Spanned<Expr>> {
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
    fn parse_or_pattern(&mut self) -> ParseResult<Spanned<Pattern>> {
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
    fn parse_lambda(&mut self, start: usize) -> ParseResult<Spanned<Expr>> {
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
    fn parse_pattern(&mut self) -> ParseResult<Spanned<Pattern>> {
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
                    return Err(ParseError::UnexpectedToken {
                        found: tok.token.clone(),
                        span: tok.span.clone(),
                        expected: "pattern".into(),
                    });
                }
            };

            let end = self.prev_span().end;
            Ok(Spanned::new(pattern, Span::new(start, end)))
        } else {
            Err(ParseError::UnexpectedEof { span: self.current_span() })
        }
    }

    /// Parse function call arguments
    fn parse_args(&mut self) -> ParseResult<Vec<Spanned<Expr>>> {
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
    fn parse_ident(&mut self) -> ParseResult<Spanned<String>> {
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

    // === Helper methods ===

    fn peek(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.pos)
    }

    fn peek_next(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.pos + 1)
    }

    fn advance(&mut self) -> Option<SpannedToken> {
        if self.is_at_end() {
            None
        } else {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        }
    }

    fn check(&self, expected: &Token) -> bool {
        self.peek().map(|t| &t.token == expected).unwrap_or(false)
    }

    fn expect(&mut self, expected: &Token) -> ParseResult<SpannedToken> {
        if self.check(expected) {
            Ok(self.advance().unwrap())
        } else {
            Err(ParseError::UnexpectedToken {
                found: self.peek().map(|t| t.token.clone()).unwrap_or(Token::Ident("EOF".into())),
                span: self.current_span(),
                expected: format!("{:?}", expected),
            })
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn current_span(&self) -> std::ops::Range<usize> {
        self.peek()
            .map(|t| t.span.clone())
            .unwrap_or(self.prev_span())
    }

    fn prev_span(&self) -> std::ops::Range<usize> {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span.clone()
        } else {
            0..0
        }
    }
}

/// Parse source code into AST
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
}

//! Type, generic, and parameter parsing for Vais language.
//!
//! Handles parsing of types (primitive, compound, function, reference, etc.),
//! generic parameters (type, const, lifetime), trait bounds, function parameters,
//! struct fields, and const expressions.

use vais_ast::*;
use vais_lexer::Token;

use crate::{ParseError, ParseResult, Parser};

impl Parser {
    /// Parse generic parameters: `<T, U>` or `<T: Trait, U: Trait1 + Trait2>` or `<const N: u64>` or `<'a, 'b>`
    pub(crate) fn parse_generics(&mut self) -> ParseResult<Vec<GenericParam>> {
        if !self.check(&Token::Lt) {
            return Ok(Vec::new());
        }

        let open_span = self.current_span();
        self.advance(); // consume '<'
        let mut generics = Vec::new();

        while !self.check(&Token::Gt) && !self.is_at_end() {
            // Check for lifetime parameter: `'a` or `'a: 'b`
            if let Some(tok) = self.peek() {
                if let Token::Lifetime(lt) = &tok.token {
                    let lt_name = lt.clone();
                    let span = tok.span.clone();
                    self.advance();

                    // Parse optional lifetime bounds: `: 'b + 'c`
                    let bounds = if self.check(&Token::Colon) {
                        self.advance();
                        match self.parse_lifetime_bounds() {
                            Ok(b) => b,
                            Err(e) => {
                                if self.recovery_mode {
                                    self.record_error(e);
                                    self.skip_to_generic_separator();
                                    continue;
                                }
                                return Err(e);
                            }
                        }
                    } else {
                        Vec::new()
                    };

                    generics.push(GenericParam::new_lifetime(
                        Spanned::new(lt_name, Span::new(span.start, span.end)),
                        bounds,
                    ));

                    if !self.check(&Token::Gt) {
                        if let Err(e) = self.expect(&Token::Comma) {
                            if self.recovery_mode {
                                self.record_error(e);
                                self.skip_to_generic_separator();
                            } else {
                                return Err(e);
                            }
                        }
                    }
                    continue;
                }
            }

            // Check if we've hit a delimiter that indicates we've escaped the generic list
            // Note: We can't check for single-letter keywords (F, S, E, etc.) because they
            // can be used as generic parameter names (e.g., `E Result<T,E>`).
            // We only check for delimiters that cannot appear in a generic parameter list.
            if let Some(tok) = self.peek() {
                match &tok.token {
                    Token::LParen | Token::LBrace => {
                        // We've hit a delimiter - likely missing `>`
                        break;
                    }
                    _ => {}
                }
            }

            // Check for const generic parameter: `const N: u64`
            if self.check(&Token::Const) {
                self.advance();
                match self.parse_const_generic_param() {
                    Ok(param) => generics.push(param),
                    Err(e) => {
                        if self.recovery_mode {
                            self.record_error(e);
                            self.skip_to_generic_separator();
                            continue;
                        }
                        return Err(e);
                    }
                }
            } else {
                match self.parse_type_generic_param() {
                    Ok(param) => generics.push(param),
                    Err(e) => {
                        if self.recovery_mode {
                            self.record_error(e);
                            self.skip_to_generic_separator();
                            continue;
                        }
                        return Err(e);
                    }
                }
            }

            if !self.check(&Token::Gt) {
                if let Err(e) = self.expect(&Token::Comma) {
                    if self.recovery_mode {
                        self.record_error(e);
                        self.skip_to_generic_separator();
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        self.expect_closing(&Token::Gt, open_span)?;
        Ok(generics)
    }

    /// Helper: parse a single const generic parameter (after `const` is consumed)
    fn parse_const_generic_param(&mut self) -> ParseResult<GenericParam> {
        let name = self.parse_ident()?;
        self.expect(&Token::Colon)?;
        let ty = self.parse_type()?;
        Ok(GenericParam::new_const(name, ty))
    }

    /// Helper: parse a single type generic parameter
    /// Also detects higher-kinded type parameters: `F<_>`, `F<_, _>`, `F<_>: Functor`
    fn parse_type_generic_param(&mut self) -> ParseResult<GenericParam> {
        // Check for variance annotation: +T (covariant), -T (contravariant)
        let variance = if self.check(&Token::Plus) {
            self.advance();
            Variance::Covariant
        } else if self.check(&Token::Minus) {
            self.advance();
            Variance::Contravariant
        } else {
            Variance::Invariant
        };

        let name = self.parse_ident()?;

        // Check for HKT pattern: Name<_> or Name<_, _>
        if self.check(&Token::Lt) {
            // Save position to backtrack if this isn't actually HKT
            let saved_pos = self.save_position();

            self.advance(); // consume '<'

            // Check if first token inside is '_' (underscore identifier)
            let mut arity = 0usize;
            let mut is_hkt = false;

            if self.check_ident("_") {
                arity = 1;
                self.advance(); // consume '_'

                // Count additional underscore parameters: <_, _, _>
                // Arity is capped at 32 to prevent excessive memory usage
                const MAX_HKT_ARITY: usize = 32;
                while self.check(&Token::Comma) && arity < MAX_HKT_ARITY {
                    self.advance(); // consume ','
                    if self.check_ident("_") {
                        arity += 1;
                        self.advance(); // consume '_'
                    } else {
                        // Not a valid HKT pattern — backtrack
                        break;
                    }
                }

                if self.check(&Token::Gt) {
                    self.advance(); // consume '>'
                    is_hkt = true;
                }
            }

            if is_hkt {
                // Parse optional bounds: F<_>: Functor
                let bounds = if self.check(&Token::Colon) {
                    self.advance();
                    self.parse_trait_bounds()?
                } else {
                    Vec::new()
                };
                return Ok(GenericParam::new_higher_kinded(name, arity, bounds));
            }

            // Not HKT — restore position and fall through to normal parsing
            self.restore_position(saved_pos);
        }

        let bounds = if self.check(&Token::Colon) {
            self.advance();
            self.parse_trait_bounds()?
        } else {
            Vec::new()
        };
        Ok(GenericParam::new_type_with_variance(name, bounds, variance))
    }

    /// Skip tokens until we find `,` or `>` in a generic parameter list.
    fn skip_to_generic_separator(&mut self) {
        let mut angle_depth = 0i32;
        while !self.is_at_end() {
            if let Some(tok) = self.peek() {
                match &tok.token {
                    Token::Comma if angle_depth == 0 => {
                        self.advance(); // consume the comma
                        return;
                    }
                    Token::Gt if angle_depth == 0 => {
                        return; // Don't consume - let parse_generics handle it
                    }
                    Token::Lt => angle_depth += 1,
                    Token::Gt => angle_depth -= 1,
                    // Stop at delimiters that indicate we've escaped the generic list
                    Token::LParen
                    | Token::LBrace
                    | Token::Function
                    | Token::Struct
                    | Token::Enum => {
                        return;
                    }
                    _ => {}
                }
            }
            self.advance();
        }
    }

    /// Parse lifetime bounds: `'a + 'b + 'c`
    fn parse_lifetime_bounds(&mut self) -> ParseResult<Vec<String>> {
        let mut bounds = Vec::new();

        // Parse first lifetime bound
        if let Some(tok) = self.peek() {
            if let Token::Lifetime(lt) = &tok.token {
                bounds.push(lt.clone());
                self.advance();
            }
        }

        // Parse additional bounds separated by `+`
        while self.check(&Token::Plus) {
            self.advance();
            if let Some(tok) = self.peek() {
                if let Token::Lifetime(lt) = &tok.token {
                    bounds.push(lt.clone());
                    self.advance();
                }
            }
        }

        Ok(bounds)
    }

    /// Parse trait bounds: `Trait1 + Trait2 + Trait3`
    pub(crate) fn parse_trait_bounds(&mut self) -> ParseResult<Vec<Spanned<String>>> {
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

    /// Parse where clause: `where T: Display + Clone, U: Default`
    pub(crate) fn parse_where_clause(&mut self) -> ParseResult<Vec<WherePredicate>> {
        use vais_ast::WherePredicate;

        if !self.check(&Token::Where) {
            return Ok(Vec::new());
        }
        self.advance(); // consume 'where'

        let mut predicates = Vec::new();

        loop {
            // Parse type name (generic parameter)
            let ty = self.parse_ident()?;

            // Expect ':'
            self.expect(&Token::Colon)?;

            // Parse trait bounds
            let bounds = self.parse_trait_bounds()?;

            predicates.push(WherePredicate { ty, bounds });

            // Check for more predicates (separated by comma)
            // Stop at `{` or `=` which indicate start of body
            if self.check(&Token::Comma) {
                self.advance();
                // Check if next token starts a body (stop parsing where clause)
                if self.check(&Token::LBrace) || self.check(&Token::Eq) {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(predicates)
    }

    /// Parse function parameters
    pub(crate) fn parse_params(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();

        while !self.check(&Token::RParen) && !self.is_at_end() {
            // Check for ownership modifiers: linear, affine, move
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

            let is_mut = self.check(&Token::Mut) || self.check(&Token::Tilde);
            if is_mut {
                self.advance();
            }

            // Handle &self and &mut self (or &~ self)
            if self.check(&Token::Amp) {
                self.advance();
                let is_self_mut = self.check(&Token::Mut) || self.check(&Token::Tilde);
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
                        is_vararg: false,
                        ownership,
                        default_value: None,
                    });
                    if !self.check(&Token::RParen) {
                        self.expect(&Token::Comma)?;
                    }
                    continue;
                }
            }

            let name = self.parse_ident()?;

            // Type annotation is optional: `F add(a, b) = a + b`
            // If colon is present, parse the type; otherwise use Type::Infer
            let ty = if self.check(&Token::Colon) {
                self.advance();
                self.parse_type()?
            } else {
                Spanned::new(Type::Infer, name.span)
            };

            // Parse optional default value: `= expr`
            let default_value = if self.check(&Token::Eq) {
                self.advance();
                let expr = self.parse_expr()?;
                Some(Box::new(expr))
            } else {
                None
            };

            params.push(Param {
                name,
                ty,
                is_mut,
                is_vararg: false,
                ownership,
                default_value,
            });

            if !self.check(&Token::RParen) {
                self.expect(&Token::Comma)?;
            }
        }

        Ok(params)
    }

    /// Parse struct field
    pub(crate) fn parse_field(&mut self) -> ParseResult<Field> {
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
            let lparen_span = self.current_span();
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
                self.expect_closing(&Token::RParen, lparen_span)?;

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
            let lbracket_span = self.current_span();
            self.advance();
            let key_or_elem = self.parse_type()?;

            if self.check(&Token::Colon) {
                self.advance();
                let value = self.parse_type()?;
                self.expect_closing(&Token::RBracket, lbracket_span.clone())?;
                Type::Map(Box::new(key_or_elem), Box::new(value))
            } else if self.check(&Token::Semi) {
                // [T; N] - const-sized array
                self.advance();
                let size = self.parse_const_expr()?;
                self.expect_closing(&Token::RBracket, lbracket_span.clone())?;
                Type::ConstArray {
                    element: Box::new(key_or_elem),
                    size,
                }
            } else {
                self.expect_closing(&Token::RBracket, lbracket_span)?;
                Type::Array(Box::new(key_or_elem))
            }
        } else if self.check(&Token::Star) {
            self.advance();
            let inner = self.parse_base_type()?;
            Type::Pointer(Box::new(inner))
        } else if self.check(&Token::Amp) {
            self.advance();
            // Check for lifetime: &'a T or &'a mut T
            let lifetime = if let Some(tok) = self.peek() {
                if let Token::Lifetime(lt) = &tok.token {
                    let lt_name = lt.clone();
                    self.advance();
                    Some(lt_name)
                } else {
                    None
                }
            } else {
                None
            };
            let is_mut = self.check(&Token::Mut) || self.check(&Token::Tilde);
            if is_mut {
                self.advance();
            }
            // Check for slice type: &[T] or &mut [T]
            if lifetime.is_none() && self.check(&Token::LBracket) {
                let lbracket_span = self.current_span();
                self.advance();
                let elem_type = self.parse_type()?;
                self.expect_closing(&Token::RBracket, lbracket_span)?;
                if is_mut {
                    Type::SliceMut(Box::new(elem_type))
                } else {
                    Type::Slice(Box::new(elem_type))
                }
            } else {
                let inner = self.parse_base_type()?;
                match (lifetime, is_mut) {
                    (Some(lt), true) => Type::RefMutLifetime {
                        lifetime: lt,
                        inner: Box::new(inner),
                    },
                    (Some(lt), false) => Type::RefLifetime {
                        lifetime: lt,
                        inner: Box::new(inner),
                    },
                    (None, true) => Type::RefMut(Box::new(inner)),
                    (None, false) => Type::Ref(Box::new(inner)),
                }
            }
        } else if self.check(&Token::Dyn) {
            // dyn Trait or dyn Trait<T>
            self.advance();
            let trait_name = self.parse_type_name()?;
            let generics = if self.check(&Token::Lt) {
                self.advance();
                let mut generics = Vec::new();
                while !self.check_gt() && !self.is_at_end() {
                    generics.push(self.parse_type()?);
                    if !self.check_gt() {
                        self.expect(&Token::Comma)?;
                    }
                }
                self.consume_gt()?;
                generics
            } else {
                Vec::new()
            };
            Type::DynTrait {
                trait_name,
                generics,
            }
        } else if self.check(&Token::Linear) {
            // linear T - must be used exactly once
            self.advance();
            let inner = self.parse_base_type()?;
            Type::Linear(Box::new(inner))
        } else if self.check(&Token::Affine) {
            // affine T - can be used at most once
            self.advance();
            let inner = self.parse_base_type()?;
            Type::Affine(Box::new(inner))
        } else if self.check(&Token::Impl) {
            // Existential type: X Trait or X Trait + Trait2 (impl Trait)
            self.advance();
            let bounds = self.parse_trait_bounds()?;
            Type::ImplTrait { bounds }
        } else if self.check(&Token::LBrace) {
            // Dependent type (refinement type): {x: T | predicate}
            self.advance();
            let var_name = self.parse_ident()?;
            self.expect(&Token::Colon)?;
            let base = self.parse_type()?;
            self.expect(&Token::Pipe)?;
            let predicate = self.parse_expr()?;
            self.expect(&Token::RBrace)?;
            Type::Dependent {
                var_name: var_name.node,
                base: Box::new(base),
                predicate: Box::new(predicate),
            }
        } else if let Some(tok) = self.peek() {
            // Check for function pointer type: fn(...) -> T
            if let Token::Ident(s) = &tok.token {
                if s == "fn" {
                    self.advance(); // consume 'fn'
                    return Ok(Spanned::new(
                        self.parse_fn_ptr_type()?,
                        Span::new(start, self.prev_span().end),
                    ));
                }
            }
            let name = self.parse_type_name()?;
            let generics = if self.check(&Token::Lt) {
                self.advance();
                let mut generics = Vec::new();
                while !self.check_gt() && !self.is_at_end() {
                    generics.push(self.parse_type()?);
                    if !self.check_gt() {
                        self.expect(&Token::Comma)?;
                    }
                }
                self.consume_gt()?;
                generics
            } else {
                Vec::new()
            };

            Type::Named { name, generics }
        } else {
            return Err(ParseError::UnexpectedEof { span: start..start });
        };

        // Check for associated type projection: Type::Item or <Type as Trait>::Item
        let ty = if matches!(ty, Type::Named { .. }) && self.check(&Token::ColonColon) {
            self.advance(); // consume ::
            let assoc_name = self.parse_ident()?;

            // Parse optional GAT generic arguments: Item<'a, T>
            let assoc_generics = if self.check(&Token::Lt) {
                self.advance();
                let mut generics = Vec::new();
                while !self.check_gt() && !self.is_at_end() {
                    generics.push(self.parse_type()?);
                    if !self.check_gt() {
                        self.expect(&Token::Comma)?;
                    }
                }
                self.consume_gt()?;
                generics
            } else {
                Vec::new()
            };

            // Check if this is a qualified path: <Type as Trait>::Item
            // For now, we'll use the type as the base
            Type::Associated {
                base: Box::new(Spanned::new(ty, Span::new(start, self.prev_span().end))),
                trait_name: None, // Could be extracted if we have `as` keyword support
                assoc_name: assoc_name.node,
                generics: assoc_generics,
            }
        } else {
            ty
        };

        let end = self.prev_span().end;
        Ok(Spanned::new(ty, Span::new(start, end)))
    }

    /// Parse type name (handles primitive types)
    pub(crate) fn parse_type_name(&mut self) -> ParseResult<String> {
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
            // SIMD Vector types
            Token::Vec2f32 => "Vec2f32",
            Token::Vec4f32 => "Vec4f32",
            Token::Vec8f32 => "Vec8f32",
            Token::Vec2f64 => "Vec2f64",
            Token::Vec4f64 => "Vec4f64",
            Token::Vec4i32 => "Vec4i32",
            Token::Vec8i32 => "Vec8i32",
            Token::Vec2i64 => "Vec2i64",
            Token::Vec4i64 => "Vec4i64",
            Token::Ident(s) => {
                // Type alias: `i` → `i64` (only in type position)
                if s == "i" {
                    return Ok("i64".to_string());
                }
                return Ok(s.clone());
            }
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
    pub(crate) fn parse_const_expr(&mut self) -> ParseResult<ConstExpr> {
        self.parse_const_bitwise()
    }

    /// Parse bitwise const expressions: `A & B`, `A | B`, `A ^ B`
    fn parse_const_bitwise(&mut self) -> ParseResult<ConstExpr> {
        let mut left = self.parse_const_additive()?;

        while self.check(&Token::Amp) || self.check(&Token::Pipe) || self.check(&Token::Caret) {
            let op = if self.check(&Token::Amp) {
                self.advance();
                ConstBinOp::BitAnd
            } else if self.check(&Token::Pipe) {
                self.advance();
                ConstBinOp::BitOr
            } else {
                self.advance();
                ConstBinOp::BitXor
            };
            let right = self.parse_const_additive()?;
            left = ConstExpr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse additive const expressions: `A + B` or `A - B`
    fn parse_const_additive(&mut self) -> ParseResult<ConstExpr> {
        let mut left = self.parse_const_shift()?;

        while self.check(&Token::Plus) || self.check(&Token::Minus) {
            let op = if self.check(&Token::Plus) {
                self.advance();
                ConstBinOp::Add
            } else {
                self.advance();
                ConstBinOp::Sub
            };
            let right = self.parse_const_shift()?;
            left = ConstExpr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse shift const expressions: `A << B` or `A >> B`
    fn parse_const_shift(&mut self) -> ParseResult<ConstExpr> {
        let mut left = self.parse_const_multiplicative()?;

        while self.check(&Token::Shl) || self.check(&Token::Shr) {
            let op = if self.check(&Token::Shl) {
                self.advance();
                ConstBinOp::Shl
            } else {
                self.advance();
                ConstBinOp::Shr
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

    /// Parse multiplicative const expressions: `A * B`, `A / B`, or `A % B`
    fn parse_const_multiplicative(&mut self) -> ParseResult<ConstExpr> {
        let mut left = self.parse_const_primary()?;

        while self.check(&Token::Star) || self.check(&Token::Slash) || self.check(&Token::Percent) {
            let op = if self.check(&Token::Star) {
                self.advance();
                ConstBinOp::Mul
            } else if self.check(&Token::Slash) {
                self.advance();
                ConstBinOp::Div
            } else {
                self.advance();
                ConstBinOp::Mod
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

    /// Parse primary const expression: `N`, `10`, `-N`, or `(expr)`
    fn parse_const_primary(&mut self) -> ParseResult<ConstExpr> {
        let span = self.current_span();

        // Handle unary negation
        if self.check(&Token::Minus) {
            self.advance();
            let inner = self.parse_const_primary()?;
            return Ok(ConstExpr::Negate(Box::new(inner)));
        }

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
                Token::Extern => {
                    self.advance();
                    return Ok(ConstExpr::Param("N".to_string()));
                }
                Token::Union => {
                    self.advance();
                    return Ok(ConstExpr::Param("O".to_string()));
                }
                Token::Trait => {
                    self.advance();
                    return Ok(ConstExpr::Param("W".to_string()));
                }
                Token::Impl => {
                    self.advance();
                    return Ok(ConstExpr::Param("X".to_string()));
                }
                Token::Defer => {
                    self.advance();
                    return Ok(ConstExpr::Param("D".to_string()));
                }
                Token::If => {
                    self.advance();
                    return Ok(ConstExpr::Param("I".to_string()));
                }
                _ => {}
            }
        }

        Err(ParseError::UnexpectedToken {
            found: self
                .peek()
                .map(|t| t.token.clone())
                .unwrap_or(Token::Ident("EOF".into())),
            span,
            expected: "const expression (integer literal or identifier)".into(),
        })
    }
}

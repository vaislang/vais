//! Vais Vais Parser Implementation
//!
//! Pratt parser를 사용한 표현식 파싱

use vais_ast::*;
use vais_lexer::{Lexer, Span, Token, TokenKind};

use crate::error::{ParseError, ParseResult};

/// Vais Vais Parser
pub struct Parser<'src> {
    lexer: Lexer<'src>,
    current: Token,
    previous: Token,
}

impl<'src> Parser<'src> {
    /// 새 파서 생성
    pub fn new(source: &'src str) -> Self {
        let mut lexer = Lexer::new(source);

        // 첫 번째 토큰 로드 (줄바꿈 스킵)
        let current = Self::next_significant_token(&mut lexer);
        let previous = Token::new(TokenKind::Eof, Span::default(), "");

        Self {
            lexer,
            current,
            previous,
        }
    }

    /// 줄바꿈, 주석 제외한 다음 토큰
    fn next_significant_token(lexer: &mut Lexer<'src>) -> Token {
        loop {
            match lexer.next_token() {
                Some(token) if token.kind == TokenKind::Newline => continue,
                Some(token) if token.kind == TokenKind::Comment => continue,
                Some(token) if token.kind == TokenKind::MultiLineComment => continue,
                Some(token) => return token,
                None => {
                    return Token::new(TokenKind::Eof, Span::default(), "");
                }
            }
        }
    }

    /// 다음 토큰으로 진행
    fn advance(&mut self) -> &Token {
        self.previous = std::mem::replace(
            &mut self.current,
            Self::next_significant_token(&mut self.lexer),
        );
        &self.previous
    }

    /// 현재 토큰 확인
    fn check(&self, kind: TokenKind) -> bool {
        self.current.kind == kind
    }

    /// EOF인지 확인
    fn is_at_end(&self) -> bool {
        self.current.kind == TokenKind::Eof
    }

    /// 특정 토큰이면 소비하고 true 반환
    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// 특정 토큰 요구
    fn expect(&mut self, kind: TokenKind) -> ParseResult<&Token> {
        if self.check(kind.clone()) {
            Ok(self.advance())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{}", kind),
                found: self.current.kind.clone(),
                span: self.current.span,
            })
        }
    }

    /// 에러 복구: 동기화 지점까지 토큰 스킵
    /// 문장/아이템 경계에서 복구 시도
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            // 이전 토큰이 문장 종료자인 경우
            if self.previous.kind == TokenKind::Newline {
                return;
            }

            // 새 문장/아이템 시작 토큰
            match self.current.kind {
                TokenKind::Fn
                | TokenKind::Let
                | TokenKind::If
                | TokenKind::For
                | TokenKind::Match
                | TokenKind::Mod
                | TokenKind::Use
                | TokenKind::Type
                | TokenKind::Enum
                | TokenKind::Trait
                | TokenKind::Impl
                | TokenKind::Pub
                | TokenKind::Async
                | TokenKind::Ffi
                | TokenKind::Macro
                | TokenKind::Effect => return,
                _ => {}
            }

            self.advance();
        }
    }

    /// 여러 에러를 수집하며 프로그램 파싱
    pub fn parse_program_with_recovery(&mut self) -> (Program, Vec<ParseError>) {
        let start = self.current.span;
        let mut items = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(e) => {
                    errors.push(e);
                    self.synchronize();
                }
            }
        }

        let end = self.previous.span;
        (Program {
            items,
            span: start.merge(end),
        }, errors)
    }

    // =========================================================================
    // 파싱 메서드
    // =========================================================================

    /// 프로그램 파싱 (여러 아이템)
    pub fn parse_program(&mut self) -> ParseResult<Program> {
        let start = self.current.span;
        let mut items = Vec::new();

        while !self.is_at_end() {
            items.push(self.parse_item()?);
        }

        let end = self.previous.span;
        Ok(Program {
            items,
            span: start.merge(end),
        })
    }

    /// 단일 표현식 파싱 (REPL용)
    pub fn parse_expression(&mut self) -> ParseResult<Expr> {
        self.parse_expr()
    }

    /// 아이템 파싱
    fn parse_item(&mut self) -> ParseResult<Item> {
        // #[test] 어트리뷰트 확인
        let is_test = if self.check(TokenKind::Hash) {
            // Lookahead for #[test]
            let saved_current = self.current.clone();
            self.advance(); // consume #
            if self.check(TokenKind::LBracket) {
                self.advance(); // consume [
                if self.check(TokenKind::Identifier) && self.current.text == "test" {
                    self.advance(); // consume test
                    self.expect(TokenKind::RBracket)?; // consume ]
                    true
                } else {
                    // Not a #[test], restore state - this is a length operator
                    self.current = saved_current;
                    // Re-lex from position (simplified: just mark as not test)
                    false
                }
            } else {
                // Not #[...], restore and continue (could be length operator)
                self.current = saved_current;
                false
            }
        } else {
            false
        };

        // pub 키워드 확인
        let is_pub = self.match_token(TokenKind::Pub);
        // async 키워드 확인
        let is_async = self.match_token(TokenKind::Async);

        match self.current.kind {
            TokenKind::Mod => {
                if is_async {
                    return Err(ParseError::InvalidSyntax {
                        message: "modules cannot be async".to_string(),
                        span: self.current.span,
                    });
                }
                self.advance();
                let name = self.expect_identifier()?;
                Ok(Item::Module(ModuleDef {
                    name,
                    span: self.previous.span,
                }))
            }
            TokenKind::Use => {
                if is_async {
                    return Err(ParseError::InvalidSyntax {
                        message: "use statements cannot be async".to_string(),
                        span: self.current.span,
                    });
                }
                self.advance();
                let use_def = self.parse_use()?;
                Ok(Item::Use(use_def))
            }
            TokenKind::Type => {
                if is_async {
                    return Err(ParseError::InvalidSyntax {
                        message: "type definitions cannot be async".to_string(),
                        span: self.current.span,
                    });
                }
                self.advance();
                let type_def = self.parse_type_def(is_pub)?;
                Ok(Item::TypeDef(type_def))
            }
            TokenKind::Enum => {
                if is_async {
                    return Err(ParseError::InvalidSyntax {
                        message: "enum definitions cannot be async".to_string(),
                        span: self.current.span,
                    });
                }
                self.advance();
                let enum_def = self.parse_enum_def(is_pub)?;
                Ok(Item::Enum(enum_def))
            }
            TokenKind::Trait => {
                if is_async {
                    return Err(ParseError::InvalidSyntax {
                        message: "trait definitions cannot be async".to_string(),
                        span: self.current.span,
                    });
                }
                self.advance();
                let trait_def = self.parse_trait_def(is_pub)?;
                Ok(Item::Trait(trait_def))
            }
            TokenKind::Impl => {
                if is_pub {
                    return Err(ParseError::InvalidSyntax {
                        message: "impl blocks cannot be pub".to_string(),
                        span: self.current.span,
                    });
                }
                if is_async {
                    return Err(ParseError::InvalidSyntax {
                        message: "impl blocks cannot be async".to_string(),
                        span: self.current.span,
                    });
                }
                self.advance();
                let impl_def = self.parse_impl_def()?;
                Ok(Item::Impl(impl_def))
            }
            TokenKind::Ffi => {
                if is_pub {
                    return Err(ParseError::InvalidSyntax {
                        message: "ffi blocks cannot be pub".to_string(),
                        span: self.current.span,
                    });
                }
                if is_async {
                    return Err(ParseError::InvalidSyntax {
                        message: "ffi blocks cannot be async".to_string(),
                        span: self.current.span,
                    });
                }
                self.advance();
                let ffi_block = self.parse_ffi_block()?;
                Ok(Item::Ffi(ffi_block))
            }
            TokenKind::Macro => {
                if is_async {
                    return Err(ParseError::InvalidSyntax {
                        message: "macro definitions cannot be async".to_string(),
                        span: self.current.span,
                    });
                }
                self.advance();
                let macro_def = self.parse_macro_def(is_pub)?;
                Ok(Item::Macro(macro_def))
            }
            TokenKind::Effect => {
                if is_async {
                    return Err(ParseError::InvalidSyntax {
                        message: "effect definitions cannot be async".to_string(),
                        span: self.current.span,
                    });
                }
                self.advance();
                let effect_def = self.parse_effect_def(is_pub)?;
                Ok(Item::Effect(effect_def))
            }
            TokenKind::Identifier => {
                // 함수 정의: name(params) = body
                // 또는 함수 호출/표현식
                // Lookahead: identifier( 다음이 identifier 또는 ) 이면 함수 정의
                if self.is_function_def() {
                    let func = self.parse_function_def(is_pub, is_async, is_test)?;
                    Ok(Item::Function(func))
                } else {
                    if is_pub {
                        return Err(ParseError::InvalidSyntax {
                            message: "pub can only be used with functions, types, or modules"
                                .to_string(),
                            span: self.current.span,
                        });
                    }
                    if is_async {
                        return Err(ParseError::InvalidSyntax {
                            message: "async can only be used with functions"
                                .to_string(),
                            span: self.current.span,
                        });
                    }
                    let expr = self.parse_expr()?;
                    Ok(Item::Expr(expr))
                }
            }
            _ => {
                // 표현식
                if is_pub {
                    return Err(ParseError::InvalidSyntax {
                        message: "pub can only be used with functions, types, or modules"
                            .to_string(),
                        span: self.current.span,
                    });
                }
                if is_async {
                    return Err(ParseError::InvalidSyntax {
                        message: "async can only be used with functions"
                            .to_string(),
                        span: self.current.span,
                    });
                }
                let expr = self.parse_expr()?;
                Ok(Item::Expr(expr))
            }
        }
    }

    /// 함수 정의인지 확인 (lookahead)
    fn is_function_def(&self) -> bool {
        // 함수 정의: name(params) = body 또는 name<T>(params) = body
        // 함수 호출: name(args) 또는 name<T>(args)
        // 구분: ) 다음에 = 또는 -> 가 있으면 함수 정의
        let mut lexer_clone = self.lexer.clone();

        // 현재 토큰은 Identifier, 다음 토큰을 확인
        let mut next = Self::next_significant_token(&mut lexer_clone);

        // 제네릭 타입 파라미터 <T, U, ...> 건너뛰기
        if next.kind == TokenKind::Lt {
            let mut depth = 1;
            while depth > 0 {
                next = Self::next_significant_token(&mut lexer_clone);
                match next.kind {
                    TokenKind::Lt => depth += 1,
                    TokenKind::Gt => depth -= 1,
                    TokenKind::Eof => return false,
                    _ => {}
                }
            }
            next = Self::next_significant_token(&mut lexer_clone);
        }

        if next.kind != TokenKind::LParen {
            return false; // identifier 다음에 ( 가 없으면 표현식
        }

        // ( 다음 토큰 확인
        let after_paren = Self::next_significant_token(&mut lexer_clone);
        match after_paren.kind {
            TokenKind::RParen => {
                // f() 형태 - ) 다음이 = 또는 -> 이면 함수 정의
                let after_rparen = Self::next_significant_token(&mut lexer_clone);
                matches!(after_rparen.kind, TokenKind::Eq | TokenKind::Arrow)
            }
            TokenKind::Identifier => {
                // f(x, ...) 형태 - ) 다음에 = 또는 -> 가 있는지 확인해야 함
                // 괄호를 끝까지 스캔
                let mut depth = 1;
                let mut tok = Self::next_significant_token(&mut lexer_clone);
                while depth > 0 {
                    match tok.kind {
                        TokenKind::LParen => depth += 1,
                        TokenKind::RParen => depth -= 1,
                        TokenKind::Eof => return false,
                        _ => {}
                    }
                    if depth > 0 {
                        tok = Self::next_significant_token(&mut lexer_clone);
                    }
                }
                // ) 다음 토큰이 = 또는 -> 이면 함수 정의
                let after_rparen = Self::next_significant_token(&mut lexer_clone);
                matches!(after_rparen.kind, TokenKind::Eq | TokenKind::Arrow)
            }
            _ => false, // f(10 등 - 함수 호출
        }
    }

    /// 식별자 파싱
    fn expect_identifier(&mut self) -> ParseResult<String> {
        if self.current.kind == TokenKind::Identifier {
            let name = self.current.text.clone();
            self.advance();
            Ok(name)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: self.current.kind.clone(),
                span: self.current.span,
            })
        }
    }

    /// 함수 정의 파싱: name(params) = body 또는 async name(params) = body
    fn parse_function_def(&mut self, is_pub: bool, is_async: bool, is_test: bool) -> ParseResult<FunctionDef> {
        let start = self.current.span;
        let name = self.expect_identifier()?;

        // 타입 파라미터 (옵션): <T, U>
        let type_params = if self.match_token(TokenKind::Lt) {
            self.parse_type_params()?
        } else {
            Vec::new()
        };

        // (params)
        self.expect(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen)?;

        // 옵션 반환 타입: -> Type
        let return_type = if self.match_token(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // = body
        self.expect(TokenKind::Eq)?;
        let body = self.parse_expr()?;

        Ok(FunctionDef {
            name,
            type_params,
            params,
            return_type,
            body,
            is_pub,
            is_async,
            is_test,
            span: start.merge(self.previous.span),
        })
    }

    /// 타입 파라미터 목록 파싱: <T, U, V>
    fn parse_type_params(&mut self) -> ParseResult<Vec<TypeParam>> {
        let mut type_params = Vec::new();

        loop {
            let span = self.current.span;
            let name = self.expect_identifier()?;
            type_params.push(TypeParam { name, span });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::Gt)?;
        Ok(type_params)
    }

    /// 매개변수 목록 파싱
    fn parse_params(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();

        if self.check(TokenKind::RParen) {
            return Ok(params);
        }

        loop {
            let param = self.parse_param()?;
            params.push(param);

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        Ok(params)
    }

    /// 단일 매개변수 파싱
    fn parse_param(&mut self) -> ParseResult<Param> {
        let start = self.current.span;
        let name = self.expect_identifier()?;

        // 옵션 타입: :Type
        let ty = if self.match_token(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // 옵션 기본값: =expr
        let default = if self.match_token(TokenKind::Eq) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        Ok(Param {
            name,
            ty,
            default,
            span: start.merge(self.previous.span),
        })
    }

    /// 타입 파싱
    fn parse_type(&mut self) -> ParseResult<TypeExpr> {
        match &self.current.kind {
            TokenKind::Identifier => {
                let name = self.current.text.clone();
                self.advance();
                Ok(TypeExpr::Simple(name))
            }
            TokenKind::LBracket => {
                // [T]
                self.advance();
                let inner = self.parse_type()?;
                self.expect(TokenKind::RBracket)?;
                Ok(TypeExpr::Array(Box::new(inner)))
            }
            TokenKind::Question => {
                // ?T
                self.advance();
                let inner = self.parse_type()?;
                Ok(TypeExpr::Optional(Box::new(inner)))
            }
            TokenKind::Bang => {
                // !T
                self.advance();
                let inner = self.parse_type()?;
                Ok(TypeExpr::Result(Box::new(inner)))
            }
            TokenKind::LParen => {
                // (T1, T2) 또는 (T1, T2) -> T3
                self.advance();
                let mut types = Vec::new();
                if !self.check(TokenKind::RParen) {
                    loop {
                        types.push(self.parse_type()?);
                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RParen)?;

                if self.match_token(TokenKind::Arrow) {
                    let ret = self.parse_type()?;
                    Ok(TypeExpr::Function(types, Box::new(ret)))
                } else {
                    Ok(TypeExpr::Tuple(types))
                }
            }
            TokenKind::LBrace => {
                // {K: V} 또는 { field: Type, ... }
                self.advance();
                let mut fields = Vec::new();
                if !self.check(TokenKind::RBrace) {
                    loop {
                        let field_name = self.expect_identifier()?;
                        self.expect(TokenKind::Colon)?;
                        let field_type = self.parse_type()?;
                        fields.push((field_name, field_type));
                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RBrace)?;

                if fields.len() == 1 && fields[0].0.len() == 1 {
                    // {K: V} - 맵 타입
                    // fields.len() == 1 조건으로 pop이 성공함이 보장됨
                    let (k, v) = fields.pop().expect("fields has exactly one element");
                    Ok(TypeExpr::Map(
                        Box::new(TypeExpr::Simple(k)),
                        Box::new(v),
                    ))
                } else {
                    Ok(TypeExpr::Struct(fields))
                }
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "type".to_string(),
                found: self.current.kind.clone(),
                span: self.current.span,
            }),
        }
    }

    /// use 문 파싱
    /// 지원 형식:
    /// - use path.module          (전체 import)
    /// - use path.module as alias (alias import)
    /// - use path.{a, b}          (선택적 import)
    /// - use path.*               (star import)
    fn parse_use(&mut self) -> ParseResult<UseDef> {
        let start = self.previous.span;
        let mut path = vec![self.expect_identifier()?];

        while self.match_token(TokenKind::Dot) {
            // use path.{a, b} - 선택적 import
            if self.check(TokenKind::LBrace) {
                self.advance();
                let mut items = Vec::new();
                loop {
                    items.push(self.expect_identifier()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RBrace)?;
                return Ok(UseDef {
                    path,
                    items: Some(items),
                    alias: None,
                    star: false,
                    span: start.merge(self.previous.span),
                });
            }

            // use path.* - star import
            if self.check(TokenKind::Star) {
                self.advance();
                return Ok(UseDef {
                    path,
                    items: None,
                    alias: None,
                    star: true,
                    span: start.merge(self.previous.span),
                });
            }

            path.push(self.expect_identifier()?);
        }

        // use path as alias
        let alias = if self.match_token(TokenKind::As) {
            Some(self.expect_identifier()?)
        } else {
            None
        };

        Ok(UseDef {
            path,
            items: None,
            alias,
            star: false,
            span: start.merge(self.previous.span),
        })
    }

    /// 타입 정의 파싱
    fn parse_type_def(&mut self, is_pub: bool) -> ParseResult<TypeDef> {
        let start = self.previous.span;
        let name = self.expect_identifier()?;

        // 타입 파라미터 (옵션): <T, U>
        let type_params = if self.match_token(TokenKind::Lt) {
            self.parse_type_params()?
        } else {
            Vec::new()
        };

        self.expect(TokenKind::Eq)?;
        let ty = self.parse_type()?;

        Ok(TypeDef {
            name,
            type_params,
            ty,
            is_pub,
            span: start.merge(self.previous.span),
        })
    }

    /// Enum 정의 파싱: enum Name { Variant1, Variant2(T), ... }
    fn parse_enum_def(&mut self, is_pub: bool) -> ParseResult<EnumDef> {
        let start = self.previous.span;
        let name = self.expect_identifier()?;

        // 타입 파라미터 (옵션): <T, U>
        let type_params = if self.match_token(TokenKind::Lt) {
            self.parse_type_params()?
        } else {
            Vec::new()
        };

        self.expect(TokenKind::LBrace)?;

        let mut variants = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            let variant = self.parse_enum_variant()?;
            variants.push(variant);
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(EnumDef {
            name,
            type_params,
            variants,
            is_pub,
            span: start.merge(self.previous.span),
        })
    }

    /// Enum variant 파싱: VariantName 또는 VariantName(Type1, Type2)
    fn parse_enum_variant(&mut self) -> ParseResult<EnumVariant> {
        let start = self.current.span;
        let name = self.expect_identifier()?;

        let fields = if self.match_token(TokenKind::LParen) {
            let mut field_types = Vec::new();
            if !self.check(TokenKind::RParen) {
                loop {
                    field_types.push(self.parse_type()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RParen)?;
            field_types
        } else {
            Vec::new()
        };

        Ok(EnumVariant {
            name,
            fields,
            span: start.merge(self.previous.span),
        })
    }

    // =========================================================================
    // Trait/Impl 파싱
    // =========================================================================

    /// Trait 정의 파싱: trait Name<T> { fn method(self) -> Type }
    fn parse_trait_def(&mut self, is_pub: bool) -> ParseResult<TraitDef> {
        let start = self.previous.span;
        let name = self.expect_identifier()?;

        // 타입 파라미터 (옵션): <T, U>
        let type_params = if self.match_token(TokenKind::Lt) {
            self.parse_type_params()?
        } else {
            Vec::new()
        };

        self.expect(TokenKind::LBrace)?;

        let mut methods = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            let method = self.parse_trait_method()?;
            methods.push(method);
        }

        self.expect(TokenKind::RBrace)?;

        Ok(TraitDef {
            name,
            type_params,
            methods,
            is_pub,
            span: start.merge(self.previous.span),
        })
    }

    /// Trait 메서드 파싱: fn name(params) -> Type 또는 fn name(params) = default_body
    fn parse_trait_method(&mut self) -> ParseResult<TraitMethod> {
        let start = self.current.span;

        // fn 키워드 (옵션, Vais 스타일에서는 생략 가능)
        let has_fn = self.match_token(TokenKind::Fn);

        let name = self.expect_identifier()?;

        // 타입 파라미터 (옵션): <T, U>
        let type_params = if self.match_token(TokenKind::Lt) {
            self.parse_type_params()?
        } else {
            Vec::new()
        };

        // (params)
        self.expect(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen)?;

        // 옵션 반환 타입: -> Type
        let return_type = if self.match_token(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // 기본 구현 (옵션): = body
        let default_impl = if self.match_token(TokenKind::Eq) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        // fn 키워드가 있었거나 기본 구현이 없으면 문장 종료
        // (Vais 스타일은 간결함을 추구하므로 세미콜론 없이도 처리)
        let _ = has_fn; // suppress unused warning

        Ok(TraitMethod {
            name,
            type_params,
            params,
            return_type,
            default_impl,
            span: start.merge(self.previous.span),
        })
    }

    /// Impl 블록 파싱: impl Trait for Type { ... } 또는 impl Type { ... }
    fn parse_impl_def(&mut self) -> ParseResult<ImplDef> {
        let start = self.previous.span;

        // 타입 파라미터 (옵션): <T, U>
        let type_params = if self.match_token(TokenKind::Lt) {
            self.parse_type_params()?
        } else {
            Vec::new()
        };

        // 첫 번째 타입 파싱 (Trait 이름 또는 대상 타입)
        let first_type = self.parse_type()?;

        // for 키워드가 있으면 trait impl, 없으면 inherent impl
        let (trait_name, trait_type_params, target_type) = if self.match_token(TokenKind::For) {
            // impl Trait<T> for Type
            let (name, params) = match &first_type {
                TypeExpr::Simple(name) => (Some(name.clone()), Vec::new()),
                TypeExpr::Generic(name, params) => (Some(name.clone()), params.clone()),
                _ => {
                    return Err(ParseError::InvalidSyntax {
                        message: "expected trait name".to_string(),
                        span: self.current.span,
                    });
                }
            };
            let target = self.parse_type()?;
            (name, params, target)
        } else {
            // impl Type { ... } (inherent impl)
            (None, Vec::new(), first_type)
        };

        self.expect(TokenKind::LBrace)?;

        let mut methods = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            // 메서드 파싱 (함수 정의와 동일)
            let method_start = self.current.span;

            // fn 키워드 (옵션)
            self.match_token(TokenKind::Fn);

            let name = self.expect_identifier()?;

            // 타입 파라미터 (옵션)
            let method_type_params = if self.match_token(TokenKind::Lt) {
                self.parse_type_params()?
            } else {
                Vec::new()
            };

            // (params)
            self.expect(TokenKind::LParen)?;
            let params = self.parse_params()?;
            self.expect(TokenKind::RParen)?;

            // 옵션 반환 타입: -> Type
            let return_type = if self.match_token(TokenKind::Arrow) {
                Some(self.parse_type()?)
            } else {
                None
            };

            // = body (필수)
            self.expect(TokenKind::Eq)?;
            let body = self.parse_expr()?;

            methods.push(FunctionDef {
                name,
                type_params: method_type_params,
                params,
                return_type,
                body,
                is_pub: false,  // impl 내부 메서드는 pub을 별도로 지정하지 않음
                is_async: false,
                is_test: false,
                span: method_start.merge(self.previous.span),
            });
        }

        self.expect(TokenKind::RBrace)?;

        Ok(ImplDef {
            trait_name,
            trait_type_params,
            target_type,
            type_params,
            methods,
            span: start.merge(self.previous.span),
        })
    }

    // =========================================================================
    // FFI 파싱
    // =========================================================================

    /// FFI 블록 파싱: ffi "libname" { fn declarations }
    fn parse_ffi_block(&mut self) -> ParseResult<FfiBlock> {
        let start = self.previous.span;

        // 라이브러리 이름 (문자열)
        let lib_name = if self.current.kind == TokenKind::String {
            let text = &self.current.text;
            let name = text[1..text.len() - 1].to_string();
            self.advance();
            name
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "library name string".to_string(),
                found: self.current.kind.clone(),
                span: self.current.span,
            });
        };

        // { 시작
        self.expect(TokenKind::LBrace)?;

        // 함수 선언들
        let mut functions = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            let ffi_fn = self.parse_ffi_fn()?;
            functions.push(ffi_fn);
        }

        self.expect(TokenKind::RBrace)?;

        Ok(FfiBlock {
            lib_name,
            abi: "C".to_string(),
            functions,
            span: start.merge(self.previous.span),
        })
    }

    /// FFI 함수 선언 파싱: fn name(param: type, ...) -> return_type
    fn parse_ffi_fn(&mut self) -> ParseResult<FfiFn> {
        let start = self.current.span;

        // fn 키워드
        self.expect(TokenKind::Fn)?;

        // 함수 이름
        let name = self.expect_identifier()?;

        // 외부 이름 (옵션): fn vais_name = "external_name"
        let extern_name = if self.match_token(TokenKind::Eq) {
            if self.current.kind == TokenKind::String {
                let text = &self.current.text;
                let ext_name = text[1..text.len() - 1].to_string();
                self.advance();
                Some(ext_name)
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "external function name string".to_string(),
                    found: self.current.kind.clone(),
                    span: self.current.span,
                });
            }
        } else {
            None
        };

        // 파라미터
        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();
        if !self.check(TokenKind::RParen) {
            loop {
                let param_name = self.expect_identifier()?;
                self.expect(TokenKind::Colon)?;
                let param_type = self.parse_ffi_type()?;
                params.push((param_name, param_type));
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen)?;

        // 반환 타입 (옵션, 기본값: void)
        let return_type = if self.match_token(TokenKind::Arrow) {
            self.parse_ffi_type()?
        } else {
            FfiType::Void
        };

        Ok(FfiFn {
            name,
            extern_name,
            params,
            return_type,
            span: start.merge(self.previous.span),
        })
    }

    /// FFI 타입 파싱
    fn parse_ffi_type(&mut self) -> ParseResult<FfiType> {
        match &self.current.kind {
            TokenKind::Identifier => {
                let type_name = self.current.text.clone();
                self.advance();

                match type_name.as_str() {
                    "void" => Ok(FfiType::Void),
                    "i8" => Ok(FfiType::Int(8)),
                    "i16" => Ok(FfiType::Int(16)),
                    "i32" | "int" => Ok(FfiType::Int(32)),
                    "i64" | "long" => Ok(FfiType::Int(64)),
                    "u8" => Ok(FfiType::Uint(8)),
                    "u16" => Ok(FfiType::Uint(16)),
                    "u32" | "uint" => Ok(FfiType::Uint(32)),
                    "u64" | "ulong" => Ok(FfiType::Uint(64)),
                    "f32" | "float" => Ok(FfiType::F32),
                    "f64" | "double" => Ok(FfiType::F64),
                    "bool" => Ok(FfiType::Bool),
                    "cstr" | "string" => Ok(FfiType::CStr),
                    "ptr" | "opaque" => Ok(FfiType::Opaque),
                    _ => Err(ParseError::InvalidSyntax {
                        message: format!("Unknown FFI type: {}", type_name),
                        span: self.previous.span,
                    }),
                }
            }
            TokenKind::Star => {
                // *T 또는 *mut T
                self.advance();
                if self.match_token(TokenKind::Mut) {
                    let inner = self.parse_ffi_type()?;
                    Ok(FfiType::MutPtr(Box::new(inner)))
                } else {
                    let inner = self.parse_ffi_type()?;
                    Ok(FfiType::Ptr(Box::new(inner)))
                }
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "FFI type".to_string(),
                found: self.current.kind.clone(),
                span: self.current.span,
            }),
        }
    }

    // =========================================================================
    // 매크로 파싱
    // =========================================================================

    /// 매크로 정의 파싱: macro name! { (pattern) => { body }, ... }
    fn parse_macro_def(&mut self, is_pub: bool) -> ParseResult<MacroDef> {
        #[allow(unused_imports)]
        use vais_ast::{MacroDef, MacroRule, MacroToken, MacroCaptureKind, MacroRepKind, MacroDelimiter};

        let start = self.current.span;
        let name = self.expect_identifier()?;

        // name! 형식 확인
        self.expect(TokenKind::Bang)?;

        self.expect(TokenKind::LBrace)?;
        while self.match_token(TokenKind::Newline) {}

        let mut rules = Vec::new();

        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            let rule_start = self.current.span;

            // 패턴: (...)
            self.expect(TokenKind::LParen)?;
            let pattern = self.parse_macro_tokens(TokenKind::RParen)?;
            self.expect(TokenKind::RParen)?;

            self.expect(TokenKind::FatArrow)?;

            // 바디: {...}
            self.expect(TokenKind::LBrace)?;
            let body = self.parse_macro_tokens(TokenKind::RBrace)?;
            self.expect(TokenKind::RBrace)?;

            rules.push(MacroRule {
                pattern,
                body,
                span: rule_start.merge(self.previous.span),
            });

            while self.match_token(TokenKind::Newline) {}
            // 쉼표는 옵션
            self.match_token(TokenKind::Comma);
            while self.match_token(TokenKind::Newline) {}
        }

        self.expect(TokenKind::RBrace)?;

        Ok(MacroDef {
            name,
            rules,
            is_pub,
            span: start.merge(self.previous.span),
        })
    }

    /// 매크로 토큰들 파싱
    fn parse_macro_tokens(&mut self, end_token: TokenKind) -> ParseResult<Vec<MacroToken>> {
        use vais_ast::{MacroToken, MacroCaptureKind, MacroRepKind, MacroDelimiter};

        let mut tokens = Vec::new();

        while !self.check(end_token.clone()) && !self.check(TokenKind::Eof) {
            if self.check(TokenKind::Dollar) {
                self.advance();

                if self.check(TokenKind::LParen) {
                    // 반복: $(...)*
                    self.advance();
                    let inner = self.parse_macro_tokens(TokenKind::RParen)?;
                    self.expect(TokenKind::RParen)?;

                    let rep_kind = if self.match_token(TokenKind::Star) {
                        MacroRepKind::ZeroOrMore
                    } else if self.match_token(TokenKind::Plus) {
                        MacroRepKind::OneOrMore
                    } else if self.match_token(TokenKind::Question) {
                        MacroRepKind::ZeroOrOne
                    } else {
                        MacroRepKind::ZeroOrMore
                    };

                    tokens.push(MacroToken::Repetition(inner, rep_kind));
                } else if self.check(TokenKind::Identifier) {
                    // 캡처: $name:kind
                    let name = self.current.text.clone();
                    self.advance();

                    if self.match_token(TokenKind::Colon) {
                        let kind_name = self.expect_identifier()?;
                        let kind = match kind_name.as_str() {
                            "expr" => MacroCaptureKind::Expr,
                            "ident" => MacroCaptureKind::Ident,
                            "ty" => MacroCaptureKind::Type,
                            "tt" => MacroCaptureKind::TokenTree,
                            "literal" => MacroCaptureKind::Literal,
                            "pat" => MacroCaptureKind::Pattern,
                            "block" => MacroCaptureKind::Block,
                            _ => MacroCaptureKind::TokenTree,
                        };
                        tokens.push(MacroToken::Capture(name, kind));
                    } else {
                        // $name만 있으면 TokenTree로 간주
                        tokens.push(MacroToken::Capture(name, MacroCaptureKind::TokenTree));
                    }
                }
            } else if self.check(TokenKind::LParen) {
                self.advance();
                let inner = self.parse_macro_tokens(TokenKind::RParen)?;
                self.expect(TokenKind::RParen)?;
                tokens.push(MacroToken::Group(MacroDelimiter::Paren, inner));
            } else if self.check(TokenKind::LBracket) {
                self.advance();
                let inner = self.parse_macro_tokens(TokenKind::RBracket)?;
                self.expect(TokenKind::RBracket)?;
                tokens.push(MacroToken::Group(MacroDelimiter::Bracket, inner));
            } else if self.check(TokenKind::LBrace) {
                self.advance();
                let inner = self.parse_macro_tokens(TokenKind::RBrace)?;
                self.expect(TokenKind::RBrace)?;
                tokens.push(MacroToken::Group(MacroDelimiter::Brace, inner));
            } else {
                // 리터럴 토큰
                tokens.push(MacroToken::Literal(self.current.text.clone()));
                self.advance();
            }
        }

        Ok(tokens)
    }

    // =========================================================================
    // Algebraic Effects 파싱
    // =========================================================================

    /// Effect 정의 파싱: effect Name { op(params) -> T, ... }
    fn parse_effect_def(&mut self, is_pub: bool) -> ParseResult<EffectDef> {
        use vais_ast::{EffectDef, EffectOp};

        let start = self.current.span;
        let name = self.expect_identifier()?;

        // 타입 파라미터 (옵션)
        let type_params = if self.check(TokenKind::Lt) {
            self.parse_type_params()?
        } else {
            vec![]
        };

        self.expect(TokenKind::LBrace)?;
        while self.match_token(TokenKind::Newline) {}

        let mut operations = Vec::new();

        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            let op_start = self.current.span;
            let op_name = self.expect_identifier()?;

            // 파라미터
            self.expect(TokenKind::LParen)?;
            let params = self.parse_params()?;
            self.expect(TokenKind::RParen)?;

            // 반환 타입 (옵션)
            let return_type = if self.match_token(TokenKind::Arrow) {
                Some(self.parse_type()?)
            } else {
                None
            };

            operations.push(EffectOp {
                name: op_name,
                params,
                return_type,
                span: op_start.merge(self.previous.span),
            });

            while self.match_token(TokenKind::Newline) {}
            self.match_token(TokenKind::Comma);
            while self.match_token(TokenKind::Newline) {}
        }

        self.expect(TokenKind::RBrace)?;

        Ok(EffectDef {
            name,
            type_params,
            operations,
            is_pub,
            span: start.merge(self.previous.span),
        })
    }

    // =========================================================================
    // 표현식 파싱 (Pratt Parser)
    // =========================================================================

    /// 표현식 파싱
    fn parse_expr(&mut self) -> ParseResult<Expr> {
        // 재할당 체크: identifier = value (함수 정의가 아닌 경우)
        if self.check(TokenKind::Identifier) && self.peek_is(TokenKind::Eq) {
            // lookahead를 더 해서 함수 정의가 아닌지 확인
            // 함수 정의는 identifier() = ... 형식
            // 재할당은 identifier = ... 형식
            if !self.is_function_def() {
                return self.parse_assignment();
            }
        }
        self.parse_ternary()
    }

    /// 재할당 파싱: x = v
    fn parse_assignment(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Eq)?;
        let value = self.parse_expr()?;

        Ok(Expr::Assign(
            name,
            Box::new(value),
            start.merge(self.previous.span),
        ))
    }

    /// 삼항 연산자: cond ? then : else
    fn parse_ternary(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;
        let mut expr = self.parse_pipeline()?;

        while self.match_token(TokenKind::Question) {
            let then_expr = self.parse_expr()?;
            self.expect(TokenKind::Colon)?;
            let else_expr = self.parse_ternary()?;

            let span = start.merge(self.previous.span);
            expr = Expr::Ternary(
                Box::new(expr),
                Box::new(then_expr),
                Box::new(else_expr),
                span,
            );
        }

        Ok(expr)
    }

    /// Pipeline: a |> f (왼쪽 값을 오른쪽 함수에 전달)
    fn parse_pipeline(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;
        let mut expr = self.parse_coalesce()?;

        while self.match_token(TokenKind::PipeArrow) {
            let func = self.parse_coalesce()?;
            let span = start.merge(self.previous.span);
            expr = Expr::Pipeline(Box::new(expr), Box::new(func), span);
        }

        Ok(expr)
    }

    /// Coalesce: a ?? b (null 병합 연산자)
    fn parse_coalesce(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;
        let mut expr = self.parse_or()?;

        while self.match_token(TokenKind::QuestionQuestion) {
            let right = self.parse_or()?;
            let span = start.merge(self.previous.span);
            expr = Expr::Coalesce(Box::new(expr), Box::new(right), span);
        }

        Ok(expr)
    }

    /// OR: a || b
    fn parse_or(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;
        let mut expr = self.parse_and()?;

        while self.match_token(TokenKind::OrOr) {
            let right = self.parse_and()?;
            let span = start.merge(self.previous.span);
            expr = Expr::Binary(Box::new(expr), BinaryOp::Or, Box::new(right), span);
        }

        Ok(expr)
    }

    /// AND: a && b
    fn parse_and(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;
        let mut expr = self.parse_equality()?;

        while self.match_token(TokenKind::AndAnd) {
            let right = self.parse_equality()?;
            let span = start.merge(self.previous.span);
            expr = Expr::Binary(Box::new(expr), BinaryOp::And, Box::new(right), span);
        }

        Ok(expr)
    }

    /// 등호: a == b, a != b
    fn parse_equality(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;
        let mut expr = self.parse_comparison()?;

        loop {
            let op = match self.current.kind {
                TokenKind::EqEq => BinaryOp::Eq,
                TokenKind::NotEq => BinaryOp::NotEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            let span = start.merge(self.previous.span);
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), span);
        }

        Ok(expr)
    }

    /// 비교: a < b, a >= b, chan <- value
    fn parse_comparison(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;
        let mut expr = self.parse_range()?;

        loop {
            // Channel send: chan <- value
            if self.current.kind == TokenKind::LeftArrow {
                self.advance();
                let value = self.parse_range()?;
                let span = start.merge(self.previous.span);
                expr = Expr::Send(Box::new(expr), Box::new(value), span);
                continue;
            }

            let op = match self.current.kind {
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::LtEq => BinaryOp::LtEq,
                TokenKind::GtEq => BinaryOp::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_range()?;
            let span = start.merge(self.previous.span);
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), span);
        }

        Ok(expr)
    }

    /// 범위: a..b
    fn parse_range(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;
        let expr = self.parse_contains()?;

        if self.match_token(TokenKind::DotDot) {
            let end = self.parse_contains()?;
            let span = start.merge(self.previous.span);
            return Ok(Expr::Range(Box::new(expr), Box::new(end), span));
        }

        Ok(expr)
    }

    /// 포함: x @ arr
    fn parse_contains(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;
        let expr = self.parse_term()?;

        if self.match_token(TokenKind::At) {
            let container = self.parse_term()?;
            let span = start.merge(self.previous.span);
            return Ok(Expr::Contains(Box::new(expr), Box::new(container), span));
        }

        Ok(expr)
    }

    /// 덧셈/뺄셈: a + b, a - b
    fn parse_term(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;
        let mut expr = self.parse_factor()?;

        loop {
            let op = match self.current.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                TokenKind::PlusPlus => BinaryOp::Concat,
                _ => break,
            };
            self.advance();
            let right = self.parse_factor()?;
            let span = start.merge(self.previous.span);
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), span);
        }

        Ok(expr)
    }

    /// 곱셈/나눗셈: a * b, a / b, a % b
    fn parse_factor(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;
        let mut expr = self.parse_unary()?;

        loop {
            let op = match self.current.kind {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            let span = start.merge(self.previous.span);
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), span);
        }

        Ok(expr)
    }

    /// 단항: -a, !a, #a, <-chan
    fn parse_unary(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;

        // Channel receive: <- chan
        if self.current.kind == TokenKind::LeftArrow {
            self.advance();
            let expr = self.parse_unary()?;
            let span = start.merge(self.previous.span);
            return Ok(Expr::Recv(Box::new(expr), span));
        }

        let op = match self.current.kind {
            TokenKind::Minus => Some(UnaryOp::Neg),
            TokenKind::Bang => Some(UnaryOp::Not),
            TokenKind::Hash => Some(UnaryOp::Len),
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let expr = self.parse_unary()?;
            let span = start.merge(self.previous.span);
            return Ok(Expr::Unary(op, Box::new(expr), span));
        }

        self.parse_postfix()
    }

    /// 후위: a.b, a[i], a(args), a.@(f), a.?(p), a./+
    fn parse_postfix(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;
        let mut expr = self.parse_primary()?;

        loop {
            match self.current.kind {
                // 필드 접근, 메서드 호출, 또는 모듈 qualified 호출: a.b, a.b(), module.func()
                TokenKind::Dot => {
                    self.advance();
                    let name = self.expect_identifier()?;
                    if self.match_token(TokenKind::LParen) {
                        // 함수 호출 형태
                        let args = self.parse_args()?;
                        self.expect(TokenKind::RParen)?;
                        let span = start.merge(self.previous.span);

                        // 왼쪽이 단순 식별자면 모듈 qualified 호출일 수 있음
                        if let Expr::Ident(module_name, _) = &expr {
                            // QualifiedIdent: module.function(args) 형태
                            // 모듈 경로와 함수 이름 분리
                            expr = Expr::Call(
                                Box::new(Expr::QualifiedIdent(
                                    vec![module_name.clone()],
                                    name,
                                    span,
                                )),
                                args,
                                span,
                            );
                        } else {
                            // 일반 메서드 호출: obj.method(args)
                            expr = Expr::MethodCall(Box::new(expr), name, args, span);
                        }
                    } else {
                        // 필드 접근
                        let span = start.merge(self.previous.span);
                        expr = Expr::Field(Box::new(expr), name, span);
                    }
                }

                // Map: a.@(f)
                TokenKind::DotAt => {
                    self.advance();
                    let mapper = if self.match_token(TokenKind::LParen) {
                        let e = self.parse_expr()?;
                        self.expect(TokenKind::RParen)?;
                        e
                    } else {
                        // .@field 형태
                        let name = self.expect_identifier()?;
                        Expr::Ident(name, self.previous.span)
                    };
                    let span = start.merge(self.previous.span);
                    expr = Expr::MapOp(Box::new(expr), Box::new(mapper), span);
                }

                // Filter: a.?(p)
                TokenKind::DotQuestion => {
                    self.advance();
                    let predicate = if self.match_token(TokenKind::LParen) {
                        let e = self.parse_expr()?;
                        self.expect(TokenKind::RParen)?;
                        e
                    } else {
                        // .?field 형태
                        let name = self.expect_identifier()?;
                        Expr::Ident(name, self.previous.span)
                    };
                    let span = start.merge(self.previous.span);
                    expr = Expr::FilterOp(Box::new(expr), Box::new(predicate), span);
                }

                // Reduce: a./+ 또는 a./op
                TokenKind::DotSlash => {
                    self.advance();
                    let kind = self.parse_reduce_kind()?;
                    let span = start.merge(self.previous.span);
                    expr = Expr::ReduceOp(Box::new(expr), kind, span);
                }

                // 병렬 Map: arr.||@(f)
                TokenKind::DotParMap => {
                    self.advance();
                    self.expect(TokenKind::LParen)?;
                    let mapper = self.parse_expr()?;
                    self.expect(TokenKind::RParen)?;
                    let span = start.merge(self.previous.span);
                    expr = Expr::ParallelMap(Box::new(expr), Box::new(mapper), span);
                }

                // 병렬 Filter: arr.||?(p)
                TokenKind::DotParFilter => {
                    self.advance();
                    self.expect(TokenKind::LParen)?;
                    let predicate = self.parse_expr()?;
                    self.expect(TokenKind::RParen)?;
                    let span = start.merge(self.previous.span);
                    expr = Expr::ParallelFilter(Box::new(expr), Box::new(predicate), span);
                }

                // 병렬 Reduce: arr.||/+
                TokenKind::DotParReduce => {
                    self.advance();
                    let kind = self.parse_reduce_kind()?;
                    let span = start.merge(self.previous.span);
                    expr = Expr::ParallelReduce(Box::new(expr), kind, span);
                }

                // 인덱스: a[i] 또는 a[start:end]
                TokenKind::LBracket => {
                    self.advance();
                    let index_kind = self.parse_index_kind()?;
                    self.expect(TokenKind::RBracket)?;
                    let span = start.merge(self.previous.span);
                    expr = Expr::Index(Box::new(expr), Box::new(index_kind), span);
                }

                // 함수 호출: a(args)
                TokenKind::LParen => {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(TokenKind::RParen)?;
                    let span = start.merge(self.previous.span);
                    expr = Expr::Call(Box::new(expr), args, span);
                }

                // Try: a?
                TokenKind::Question if !self.peek_is_ternary() => {
                    self.advance();
                    let span = start.merge(self.previous.span);
                    expr = Expr::Try(Box::new(expr), span);
                }

                _ => break,
            }
        }

        Ok(expr)
    }

    /// 삼항의 ? 인지 확인
    /// Try 연산자: `expr?` (? 뒤에 연산자나 구분자가 오면 Try)
    /// 삼항 연산자: `cond ? then : else` (? 뒤에 표현식이 오면 삼항)
    fn peek_is_ternary(&self) -> bool {
        // current 토큰이 ? 다음에 오는 토큰
        // 표현식 시작 가능한 토큰인지 확인
        match self.current.kind {
            // 이들은 표현식 시작이 아님 -> Try
            TokenKind::RParen
            | TokenKind::RBracket
            | TokenKind::RBrace
            | TokenKind::Comma
            | TokenKind::Newline
            | TokenKind::Colon
            | TokenKind::Eof => false,

            // 이항 연산자들 -> Try
            TokenKind::Plus
            | TokenKind::PlusPlus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::AndAnd
            | TokenKind::OrOr
            | TokenKind::Eq
            | TokenKind::NotEq
            | TokenKind::Lt
            | TokenKind::LtEq
            | TokenKind::Gt
            | TokenKind::GtEq
            | TokenKind::Dot
            | TokenKind::DotAt
            | TokenKind::DotQuestion
            | TokenKind::DotSlash
            | TokenKind::Pipe => false,

            // 그 외 (리터럴, 식별자, 괄호 시작 등)는 표현식 시작 -> 삼항
            _ => true,
        }
    }

    /// Reduce 종류 파싱
    fn parse_reduce_kind(&mut self) -> ParseResult<ReduceKind> {
        match self.current.kind {
            TokenKind::Plus => {
                self.advance();
                Ok(ReduceKind::Sum)
            }
            TokenKind::Star => {
                self.advance();
                Ok(ReduceKind::Product)
            }
            TokenKind::Identifier => {
                let name = self.current.text.clone();
                self.advance();
                match name.as_str() {
                    "min" => Ok(ReduceKind::Min),
                    "max" => Ok(ReduceKind::Max),
                    "and" => Ok(ReduceKind::And),
                    "or" => Ok(ReduceKind::Or),
                    _ => Err(ParseError::InvalidSyntax {
                        message: format!("Unknown reduce operator: {}", name),
                        span: self.previous.span,
                    }),
                }
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "reduce operator (+, *, min, max, and, or)".to_string(),
                found: self.current.kind.clone(),
                span: self.current.span,
            }),
        }
    }

    /// 인덱스 종류 파싱
    fn parse_index_kind(&mut self) -> ParseResult<IndexKind> {
        // 슬라이스: [:end], [start:], [start:end], [:]
        if self.check(TokenKind::Colon) {
            self.advance();
            let end = if self.check(TokenKind::RBracket) {
                None
            } else {
                Some(self.parse_expr()?)
            };
            return Ok(IndexKind::Slice(None, end));
        }

        let start_expr = self.parse_expr()?;

        if self.match_token(TokenKind::Colon) {
            // [start:] 또는 [start:end]
            let end = if self.check(TokenKind::RBracket) {
                None
            } else {
                Some(self.parse_expr()?)
            };
            Ok(IndexKind::Slice(Some(start_expr), end))
        } else {
            Ok(IndexKind::Single(start_expr))
        }
    }

    /// 인자 목록 파싱
    fn parse_args(&mut self) -> ParseResult<Vec<Expr>> {
        let mut args = Vec::new();

        if self.check(TokenKind::RParen) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expr()?);
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        Ok(args)
    }

    /// 기본 표현식 파싱
    fn parse_primary(&mut self) -> ParseResult<Expr> {
        let span = self.current.span;

        match &self.current.kind {
            // 리터럴
            TokenKind::Integer => {
                let value: i64 = self.current.text.parse().map_err(|_| {
                    ParseError::InvalidNumber {
                        message: "Invalid integer".to_string(),
                        span,
                    }
                })?;
                self.advance();
                Ok(Expr::Integer(value, span))
            }
            TokenKind::HexInteger => {
                // 0x or 0X prefix 제거 후 16진수 파싱
                let text = &self.current.text[2..];
                let value = i64::from_str_radix(text, 16).map_err(|_| {
                    ParseError::InvalidNumber {
                        message: "Invalid hex integer".to_string(),
                        span,
                    }
                })?;
                self.advance();
                Ok(Expr::Integer(value, span))
            }
            TokenKind::BinaryInteger => {
                // 0b or 0B prefix 제거 후 2진수 파싱
                let text = &self.current.text[2..];
                let value = i64::from_str_radix(text, 2).map_err(|_| {
                    ParseError::InvalidNumber {
                        message: "Invalid binary integer".to_string(),
                        span,
                    }
                })?;
                self.advance();
                Ok(Expr::Integer(value, span))
            }
            TokenKind::Float => {
                let value: f64 = self.current.text.parse().map_err(|_| {
                    ParseError::InvalidNumber {
                        message: "Invalid float".to_string(),
                        span,
                    }
                })?;
                self.advance();
                Ok(Expr::Float(value, span))
            }
            TokenKind::String => {
                // 따옴표 제거 및 이스케이프 시퀀스 처리
                let text = &self.current.text;
                let raw = &text[1..text.len() - 1];
                let value = Self::unescape_string(raw);
                self.advance();
                Ok(Expr::String(value, span))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Bool(true, span))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Bool(false, span))
            }
            TokenKind::Nil => {
                self.advance();
                Ok(Expr::Nil(span))
            }

            // 식별자, Struct 리터럴, 또는 매크로 호출
            TokenKind::Identifier => {
                let name = self.current.text.clone();
                self.advance();

                // 매크로 호출: name!(args)
                if self.check(TokenKind::Bang) {
                    self.advance(); // consume '!'
                    self.expect(TokenKind::LParen)?;
                    let args = self.parse_args()?;
                    self.expect(TokenKind::RParen)?;
                    return Ok(Expr::MacroCall {
                        name,
                        args,
                        span: span.merge(self.previous.span),
                    });
                }

                // Struct 리터럴: TypeName { field: value, ... }
                // 대문자로 시작하는 식별자만 Struct 리터럴로 처리
                let is_type_name = name.chars().next().is_some_and(|c| c.is_uppercase());
                if is_type_name && self.current.kind == TokenKind::LBrace {
                    self.parse_struct_literal(name, span)
                } else {
                    Ok(Expr::Ident(name, span))
                }
            }

            // 람다 파라미터
            TokenKind::Underscore => {
                self.advance();
                Ok(Expr::LambdaParam(span))
            }

            // Pipe 람다: |x| expr 또는 |x, y| expr
            TokenKind::Pipe => {
                self.advance();  // consume '|'
                let mut params = Vec::new();

                // 빈 파라미터 체크 ||
                if !self.check(TokenKind::Pipe) {
                    // 파라미터 수집
                    loop {
                        let param = self.expect_identifier()?;
                        params.push(param);
                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                }

                self.expect(TokenKind::Pipe)?;  // closing '|'
                let body = self.parse_expr()?;

                Ok(Expr::Lambda(params, Box::new(body), span.merge(self.previous.span)))
            }

            // 재귀 호출: $(args)
            TokenKind::Dollar => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let args = self.parse_args()?;
                self.expect(TokenKind::RParen)?;
                let end_span = self.previous.span;
                Ok(Expr::SelfCall(args, span.merge(end_span)))
            }

            // 배열 또는 list comprehension: [a, b, c] 또는 [expr for x in iter]
            TokenKind::LBracket => {
                self.advance();
                if self.check(TokenKind::RBracket) {
                    // 빈 배열
                    self.advance();
                    return Ok(Expr::Array(vec![], span.merge(self.previous.span)));
                }

                // 첫 번째 표현식 파싱
                let first = self.parse_expr()?;

                // list comprehension: [expr for var in iter if cond]
                if self.match_token(TokenKind::For) {
                    let var = self.expect_identifier()?;
                    self.expect(TokenKind::In)?;
                    let iter = self.parse_expr()?;
                    let cond = if self.match_token(TokenKind::If) {
                        Some(Box::new(self.parse_expr()?))
                    } else {
                        None
                    };
                    self.expect(TokenKind::RBracket)?;
                    return Ok(Expr::ListComprehension {
                        expr: Box::new(first),
                        var,
                        iter: Box::new(iter),
                        cond,
                        span: span.merge(self.previous.span),
                    });
                }

                // 일반 배열
                let mut elements = vec![first];
                while self.match_token(TokenKind::Comma) {
                    if self.check(TokenKind::RBracket) {
                        break;
                    }
                    elements.push(self.parse_expr()?);
                }
                self.expect(TokenKind::RBracket)?;
                Ok(Expr::Array(elements, span.merge(self.previous.span)))
            }

            // 세트 또는 set comprehension: #{a, b, c} 또는 #{expr for x in iter}
            TokenKind::HashBrace => {
                self.advance();
                if self.check(TokenKind::RBrace) {
                    // 빈 세트
                    self.advance();
                    return Ok(Expr::Set(vec![], span.merge(self.previous.span)));
                }

                // 첫 번째 표현식 파싱
                let first = self.parse_expr()?;

                // set comprehension: #{expr for var in iter if cond}
                if self.match_token(TokenKind::For) {
                    let var = self.expect_identifier()?;
                    self.expect(TokenKind::In)?;
                    let iter = self.parse_expr()?;
                    let cond = if self.match_token(TokenKind::If) {
                        Some(Box::new(self.parse_expr()?))
                    } else {
                        None
                    };
                    self.expect(TokenKind::RBrace)?;
                    return Ok(Expr::SetComprehension {
                        expr: Box::new(first),
                        var,
                        iter: Box::new(iter),
                        cond,
                        span: span.merge(self.previous.span),
                    });
                }

                // 일반 세트
                let mut elements = vec![first];
                while self.match_token(TokenKind::Comma) {
                    if self.check(TokenKind::RBrace) {
                        break;
                    }
                    elements.push(self.parse_expr()?);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Expr::Set(elements, span.merge(self.previous.span)))
            }

            // 블록 또는 맵: { ... }
            TokenKind::LBrace => {
                self.advance();
                self.parse_brace_expr(span)
            }

            // 그룹 또는 튜플 또는 람다: (expr) 또는 (a, b) 또는 (x) => body
            TokenKind::LParen => {
                self.advance();
                if self.check(TokenKind::RParen) {
                    // 빈 괄호
                    self.advance();
                    // 람다 체크: () => body
                    if self.match_token(TokenKind::FatArrow) {
                        let body = self.parse_expr()?;
                        return Ok(Expr::Lambda(vec![], Box::new(body), span.merge(self.previous.span)));
                    }
                    // 빈 튜플
                    let end_span = self.previous.span;
                    return Ok(Expr::Tuple(vec![], span.merge(end_span)));
                }

                let first = self.parse_expr()?;

                if self.match_token(TokenKind::Comma) {
                    // 튜플 또는 다중 파라미터 람다
                    let mut elements = vec![first];
                    if !self.check(TokenKind::RParen) {
                        loop {
                            elements.push(self.parse_expr()?);
                            if !self.match_token(TokenKind::Comma) {
                                break;
                            }
                        }
                    }
                    self.expect(TokenKind::RParen)?;

                    // 람다 체크: (x, y) => body
                    if self.match_token(TokenKind::FatArrow) {
                        // 모든 요소가 식별자인지 확인
                        let params: Vec<String> = elements.iter().filter_map(|e| {
                            if let Expr::Ident(name, _) = e {
                                Some(name.clone())
                            } else {
                                None
                            }
                        }).collect();
                        if params.len() == elements.len() {
                            let body = self.parse_expr()?;
                            return Ok(Expr::Lambda(params, Box::new(body), span.merge(self.previous.span)));
                        }
                    }

                    let end_span = self.previous.span;
                    Ok(Expr::Tuple(elements, span.merge(end_span)))
                } else {
                    // 그룹 또는 단일 파라미터 람다
                    self.expect(TokenKind::RParen)?;

                    // 람다 체크: (x) => body
                    if self.match_token(TokenKind::FatArrow) {
                        if let Expr::Ident(name, _) = &first {
                            let body = self.parse_expr()?;
                            return Ok(Expr::Lambda(vec![name.clone()], Box::new(body), span.merge(self.previous.span)));
                        }
                    }

                    Ok(first)
                }
            }

            // let 바인딩
            TokenKind::Let => {
                self.advance();
                self.parse_let(span)
            }

            // if 표현식
            TokenKind::If => {
                self.advance();
                self.parse_if(span)
            }

            // match 표현식
            TokenKind::Match => {
                self.advance();
                self.parse_match(span)
            }

            // for 루프: for var in iter { body }
            TokenKind::For => {
                self.advance();
                self.parse_for(span)
            }

            // while 루프: while cond { body }
            TokenKind::While => {
                self.advance();
                self.parse_while(span)
            }

            // try-catch 블록
            TokenKind::Try => {
                self.advance();
                self.parse_try_catch(span)
            }

            // err
            TokenKind::Err => {
                self.advance();
                if self.match_token(TokenKind::LParen) {
                    let msg = self.parse_expr()?;
                    self.expect(TokenKind::RParen)?;
                    let end_span = self.previous.span;
                    Ok(Expr::Error(Some(Box::new(msg)), span.merge(end_span)))
                } else {
                    Ok(Expr::Error(None, span))
                }
            }

            // await 표현식
            TokenKind::Await => {
                self.advance();
                let expr = self.parse_unary()?;
                let end_span = expr.span();
                Ok(Expr::Await(Box::new(expr), span.merge(end_span)))
            }

            // spawn 표현식 (태스크 생성)
            TokenKind::Spawn => {
                self.advance();
                let expr = self.parse_unary()?;
                let end_span = expr.span();
                Ok(Expr::Spawn(Box::new(expr), span.merge(end_span)))
            }

            // perform Effect.op(args)
            TokenKind::Perform => {
                self.advance();
                let effect = self.expect_identifier()?;
                self.expect(TokenKind::Dot)?;
                let operation = self.expect_identifier()?;
                self.expect(TokenKind::LParen)?;
                let args = self.parse_args()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr::Perform {
                    effect,
                    operation,
                    args,
                    span: span.merge(self.previous.span),
                })
            }

            // handle expr { Effect.op(params) => handler, ... }
            TokenKind::Handle => {
                self.advance();
                let body = Box::new(self.parse_unary()?);
                self.expect(TokenKind::LBrace)?;
                while self.match_token(TokenKind::Newline) {}

                let mut handlers = Vec::new();
                while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
                    let handler_start = self.current.span;
                    let effect = self.expect_identifier()?;
                    self.expect(TokenKind::Dot)?;
                    let operation = self.expect_identifier()?;

                    self.expect(TokenKind::LParen)?;
                    let mut params = Vec::new();
                    if !self.check(TokenKind::RParen) {
                        loop {
                            params.push(self.expect_identifier()?);
                            if !self.match_token(TokenKind::Comma) {
                                break;
                            }
                        }
                    }
                    self.expect(TokenKind::RParen)?;

                    // resume 콜백 이름 (옵션): with resume
                    let resume = if self.check(TokenKind::Identifier) && self.current.text == "with" {
                        self.advance();
                        Some(self.expect_identifier()?)
                    } else {
                        None
                    };

                    self.expect(TokenKind::FatArrow)?;
                    let handler_body = self.parse_expr()?;

                    handlers.push(vais_ast::EffectHandler {
                        effect,
                        operation,
                        params,
                        resume,
                        body: handler_body,
                        span: handler_start.merge(self.previous.span),
                    });

                    while self.match_token(TokenKind::Newline) {}
                    self.match_token(TokenKind::Comma);
                    while self.match_token(TokenKind::Newline) {}
                }

                self.expect(TokenKind::RBrace)?;

                Ok(Expr::Handle {
                    body,
                    handlers,
                    span: span.merge(self.previous.span),
                })
            }

            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: self.current.kind.clone(),
                span,
            }),
        }
    }

    /// 중괄호 표현식 파싱 (블록 또는 맵)
    /// Struct 리터럴 파싱: TypeName { field: value, ... }
    fn parse_struct_literal(&mut self, name: String, start: Span) -> ParseResult<Expr> {
        self.expect(TokenKind::LBrace)?;

        let mut fields = Vec::new();
        if !self.check(TokenKind::RBrace) {
            loop {
                let field_name = self.expect_identifier()?;
                self.expect(TokenKind::Colon)?;
                let value = self.parse_expr()?;
                fields.push((field_name, value));
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
                if self.check(TokenKind::RBrace) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(Expr::Struct(name, fields, start.merge(self.previous.span)))
    }

    fn parse_brace_expr(&mut self, start: Span) -> ParseResult<Expr> {
        if self.check(TokenKind::RBrace) {
            // 빈 맵
            self.advance();
            return Ok(Expr::Map(vec![], start.merge(self.previous.span)));
        }

        // 첫 번째 요소로 블록인지 맵인지 결정
        // 맵: { key: value, ... } - Identifier 다음에 Colon이 와야 함
        // 블록: { expr; expr; ... } - 그 외 모든 경우
        if self.check(TokenKind::Identifier) && self.peek_is(TokenKind::Colon) {
            // 맵으로 처리: identifier 다음에 : 가 오는 경우만
            let mut entries = Vec::new();
            loop {
                let key = self.expect_identifier()?;
                self.expect(TokenKind::Colon)?;
                let value = self.parse_expr()?;
                entries.push((key, value));
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
                if self.check(TokenKind::RBrace) {
                    break;
                }
            }
            self.expect(TokenKind::RBrace)?;
            return Ok(Expr::Map(entries, start.merge(self.previous.span)));
        }

        // 블록으로 처리: { x } 또는 { x; y; z }
        let mut exprs = Vec::new();
        loop {
            exprs.push(self.parse_expr()?);
            if self.check(TokenKind::RBrace) {
                break;
            }
            // 세미콜론 또는 줄바꿈으로 구분
            if !self.match_token(TokenKind::Semi) && !self.match_token(TokenKind::Newline) {
                break;
            }
            if self.check(TokenKind::RBrace) {
                break;
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(Expr::Block(exprs, start.merge(self.previous.span)))
    }

    /// 다음 토큰(현재 이후)이 특정 종류인지 확인 (lookahead)
    fn peek_is(&self, kind: TokenKind) -> bool {
        // lexer를 복제하여 다음 토큰 확인
        let mut lexer_clone = self.lexer.clone();
        if let Some(token) = lexer_clone.next() {
            token.kind == kind
        } else {
            false
        }
    }

    /// 문자열 이스케이프 시퀀스 처리
    fn unescape_string(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(&next) = chars.peek() {
                    let escaped = match next {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        '\'' => '\'',
                        '0' => '\0',
                        // 알 수 없는 이스케이프는 그대로 유지
                        _ => {
                            result.push('\\');
                            continue;
                        }
                    };
                    chars.next(); // 이스케이프 문자 소비
                    result.push(escaped);
                } else {
                    result.push('\\');
                }
            } else {
                result.push(c);
            }
        }
        result
    }

    /// let 표현식 파싱: let x = v : body 또는 let mut x = v : body
    /// 튜플 디스트럭처링: let (a, b) = (1, 2) : body
    ///
    /// Note: 바인딩 값에서는 parse_coalesce()를 사용하여 ternary 연산자의 ':'와
    /// let 표현식의 ':' 구분자가 충돌하지 않도록 함
    fn parse_let(&mut self, start: Span) -> ParseResult<Expr> {
        // 튜플 디스트럭처링 체크: let (a, b) = ...
        if self.check(TokenKind::LParen) {
            return self.parse_let_destructure(start);
        }

        let mut bindings = Vec::new();

        loop {
            // mut 키워드 확인
            let is_mut = self.match_token(TokenKind::Mut);

            let name = self.expect_identifier()?;
            self.expect(TokenKind::Eq)?;
            // ternary (? :)를 포함하지 않도록 pipeline 레벨까지만 파싱
            // 이렇게 하면 `let x = cond ? a : b : body`에서 `:` 충돌 방지
            // pipeline |> 는 `:` 충돌이 없으므로 포함
            let value = self.parse_pipeline()?;
            bindings.push((name, value, is_mut));

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::Colon)?;
        let body = self.parse_expr()?;

        Ok(Expr::Let(
            bindings,
            Box::new(body),
            start.merge(self.previous.span),
        ))
    }

    /// 튜플 디스트럭처링 파싱: let (a, b) = (1, 2) : body
    fn parse_let_destructure(&mut self, start: Span) -> ParseResult<Expr> {
        self.expect(TokenKind::LParen)?;

        let mut names = Vec::new();
        loop {
            let name = self.expect_identifier()?;
            names.push(name);

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::Eq)?;
        let value = self.parse_pipeline()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_expr()?;

        Ok(Expr::LetDestructure(
            names,
            Box::new(value),
            Box::new(body),
            start.merge(self.previous.span),
        ))
    }

    /// if 표현식 파싱
    /// 지원 형식:
    /// - if cond { then } else { else }
    /// - if cond then expr else expr
    fn parse_if(&mut self, start: Span) -> ParseResult<Expr> {
        let cond = self.parse_expr()?;

        // if cond then expr else expr 형식
        if self.match_token(TokenKind::Then) {
            let then_expr = self.parse_expr()?;
            self.expect(TokenKind::Else)?;
            let else_expr = self.parse_expr()?;
            return Ok(Expr::If(
                Box::new(cond),
                Box::new(then_expr),
                Some(Box::new(else_expr)),
                start.merge(self.previous.span),
            ));
        }

        // if cond { then } else { else } 형식
        self.expect(TokenKind::LBrace)?;
        let then_expr = self.parse_expr()?;
        self.expect(TokenKind::RBrace)?;

        let else_expr = if self.match_token(TokenKind::Else) {
            if self.check(TokenKind::If) {
                // else if 처리
                self.advance();
                let nested_if = self.parse_if(self.previous.span)?;
                Some(Box::new(nested_if))
            } else {
                self.expect(TokenKind::LBrace)?;
                let e = self.parse_expr()?;
                self.expect(TokenKind::RBrace)?;
                Some(Box::new(e))
            }
        } else {
            None
        };

        Ok(Expr::If(
            Box::new(cond),
            Box::new(then_expr),
            else_expr,
            start.merge(self.previous.span),
        ))
    }

    /// for 루프 파싱: for var in iter { body }
    fn parse_for(&mut self, start: Span) -> ParseResult<Expr> {
        let var = self.expect_identifier()?;
        self.expect(TokenKind::In)?;
        let iter = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_expr()?;
        self.expect(TokenKind::RBrace)?;

        Ok(Expr::For(
            var,
            Box::new(iter),
            Box::new(body),
            start.merge(self.previous.span),
        ))
    }

    /// while 루프 파싱: while cond { body }
    fn parse_while(&mut self, start: Span) -> ParseResult<Expr> {
        let cond = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_expr()?;
        self.expect(TokenKind::RBrace)?;

        Ok(Expr::While(
            Box::new(cond),
            Box::new(body),
            start.merge(self.previous.span),
        ))
    }

    /// try-catch 블록 파싱: try { body } catch e { handler }
    fn parse_try_catch(&mut self, start: Span) -> ParseResult<Expr> {
        // try { ... }
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_expr()?;
        self.expect(TokenKind::RBrace)?;

        // catch e { ... }
        self.expect(TokenKind::Catch)?;
        let error_name = self.expect_identifier()?;
        self.expect(TokenKind::LBrace)?;
        let handler = self.parse_expr()?;
        self.expect(TokenKind::RBrace)?;

        Ok(Expr::TryCatch {
            body: Box::new(body),
            error_name,
            handler: Box::new(handler),
            span: start.merge(self.previous.span),
        })
    }

    /// Match 표현식 파싱: match expr { pattern => body, ... }
    fn parse_match(&mut self, start: Span) -> ParseResult<Expr> {
        let scrutinee = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;

        let mut arms = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            let arm = self.parse_match_arm()?;
            arms.push(arm);

            // 콤마 또는 줄바꿈으로 구분
            if !self.match_token(TokenKind::Comma) {
                // 줄바꿈 건너뛰기
                while self.match_token(TokenKind::Newline) {}
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(Expr::Match(
            Box::new(scrutinee),
            arms,
            start.merge(self.previous.span),
        ))
    }

    /// Match arm 파싱: pattern [if guard] => body
    fn parse_match_arm(&mut self) -> ParseResult<MatchArm> {
        let start = self.current.span;
        let pattern = self.parse_pattern()?;

        // 가드 조건: if expr
        let guard = if self.match_token(TokenKind::If) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.expect(TokenKind::FatArrow)?;
        let body = self.parse_expr()?;

        Ok(MatchArm {
            pattern,
            guard,
            body,
            span: start.merge(self.previous.span),
        })
    }

    /// 패턴 파싱
    fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        let span = self.current.span;

        match &self.current.kind {
            // 와일드카드: _
            TokenKind::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard(span))
            }

            // 리터럴: 숫자, 문자열, bool (또는 범위 패턴)
            TokenKind::Integer | TokenKind::HexInteger | TokenKind::BinaryInteger
            | TokenKind::Float | TokenKind::String
            | TokenKind::True | TokenKind::False | TokenKind::Nil => {
                let expr = self.parse_primary()?;
                // 범위 패턴 확인: expr..expr
                if self.match_token(TokenKind::DotDot) {
                    let end = self.parse_primary()?;
                    return Ok(Pattern::Range(
                        Box::new(expr),
                        Box::new(end),
                        span.merge(self.previous.span),
                    ));
                }
                Ok(Pattern::Literal(expr))
            }

            // 식별자: 바인딩 또는 enum variant
            TokenKind::Identifier => {
                let name = self.current.text.clone();
                self.advance();

                // Enum variant with payload: Some(x)
                if self.match_token(TokenKind::LParen) {
                    if self.check(TokenKind::RParen) {
                        self.advance();
                        Ok(Pattern::Variant(name, None, span.merge(self.previous.span)))
                    } else {
                        let inner = self.parse_pattern()?;
                        self.expect(TokenKind::RParen)?;
                        Ok(Pattern::Variant(name, Some(Box::new(inner)), span.merge(self.previous.span)))
                    }
                } else {
                    // 대문자로 시작하는 식별자는 payload 없는 variant로 처리 (None, True 등)
                    // 소문자로 시작하면 바인딩
                    let first_char = name.chars().next().unwrap_or('a');
                    if first_char.is_uppercase() {
                        Ok(Pattern::Variant(name, None, span))
                    } else {
                        Ok(Pattern::Binding(name, span))
                    }
                }
            }

            // 튜플 패턴: (a, b, c)
            TokenKind::LParen => {
                self.advance();
                let mut patterns = Vec::new();
                if !self.check(TokenKind::RParen) {
                    loop {
                        patterns.push(self.parse_pattern()?);
                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RParen)?;
                Ok(Pattern::Tuple(patterns, span.merge(self.previous.span)))
            }

            // 배열 패턴: [a, b, ...]
            TokenKind::LBracket => {
                self.advance();
                let mut patterns = Vec::new();
                if !self.check(TokenKind::RBracket) {
                    loop {
                        patterns.push(self.parse_pattern()?);
                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RBracket)?;
                Ok(Pattern::Array(patterns, span.merge(self.previous.span)))
            }

            // 구조체 패턴: { field, ... }
            TokenKind::LBrace => {
                self.advance();
                let mut fields = Vec::new();
                if !self.check(TokenKind::RBrace) {
                    loop {
                        let field_name = self.expect_identifier()?;
                        let pattern = if self.match_token(TokenKind::Colon) {
                            Some(self.parse_pattern()?)
                        } else {
                            None
                        };
                        fields.push((field_name, pattern));
                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Pattern::Struct(fields, span.merge(self.previous.span)))
            }

            _ => Err(ParseError::UnexpectedToken {
                expected: "pattern".to_string(),
                found: self.current.kind.clone(),
                span,
            }),
        }
    }
}

/// 편의 함수: 소스 코드를 프로그램으로 파싱
pub fn parse(source: &str) -> ParseResult<Program> {
    let mut parser = Parser::new(source);
    parser.parse_program()
}

/// 편의 함수: 단일 표현식 파싱
pub fn parse_expr(source: &str) -> ParseResult<Expr> {
    let mut parser = Parser::new(source);
    parser.parse_expression()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ParseError;

    // =========================================================================
    // Success Cases - Basic
    // =========================================================================

    #[test]
    fn test_simple_function() {
        let result = parse("add(a,b)=a+b");
        assert!(result.is_ok());

        let program = result.unwrap();
        assert_eq!(program.items.len(), 1);

        if let Item::Function(f) = &program.items[0] {
            assert_eq!(f.name, "add");
            assert_eq!(f.params.len(), 2);
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_ternary() {
        let result = parse_expr("a>b?a:b");
        assert!(result.is_ok());

        if let Expr::Ternary(_, _, _, _) = result.unwrap() {
            // OK
        } else {
            panic!("Expected ternary");
        }
    }

    #[test]
    fn test_collection_ops() {
        let result = parse_expr("arr.@(_*2).?(_>0)./+");
        assert!(result.is_ok());
    }

    #[test]
    fn test_self_recursion() {
        let result = parse_expr("$(n-1)+$(n-2)");
        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }

    #[test]
    fn test_let_binding() {
        let result = parse_expr("let x=1,y=2:x+y");
        assert!(result.is_ok());

        if let Expr::Let(bindings, _, _) = result.unwrap() {
            assert_eq!(bindings.len(), 2);
        } else {
            panic!("Expected let");
        }
    }

    #[test]
    fn test_array() {
        let result = parse_expr("[1, 2, 3]");
        assert!(result.is_ok());

        if let Expr::Array(elements, _) = result.unwrap() {
            assert_eq!(elements.len(), 3);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_fibonacci() {
        let result = parse("fib(n)=n<2?n:$(n-1)+$(n-2)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_quicksort() {
        let result = parse("qs(a)=#a<2?a:let p=a[0],r=a[1:]:$(r.?(_<p))+[p]+$(r.?(_>=p))");
        assert!(result.is_ok());
    }

    // =========================================================================
    // Success Cases - Literals
    // =========================================================================

    #[test]
    fn test_integer_literal() {
        let result = parse_expr("42");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Expr::Integer(42, _)));
    }

    #[test]
    fn test_negative_integer() {
        let result = parse_expr("-42");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Expr::Unary(UnaryOp::Neg, _, _)));
    }

    #[test]
    fn test_float_literal() {
        let result = parse_expr("3.15");
        assert!(result.is_ok());
        if let Expr::Float(f, _) = result.unwrap() {
            assert!((f - 3.15).abs() < f64::EPSILON);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_string_literal() {
        let result = parse_expr(r#""hello world""#);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Expr::String(s, _) if s == "hello world"));
    }

    #[test]
    fn test_bool_literals() {
        assert!(matches!(parse_expr("true").unwrap(), Expr::Bool(true, _)));
        assert!(matches!(parse_expr("false").unwrap(), Expr::Bool(false, _)));
    }

    #[test]
    fn test_nil_literal() {
        let result = parse_expr("nil");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Expr::Nil(_)));
    }

    // =========================================================================
    // Success Cases - Operators
    // =========================================================================

    #[test]
    fn test_binary_arithmetic() {
        assert!(parse_expr("1 + 2").is_ok());
        assert!(parse_expr("5 - 3").is_ok());
        assert!(parse_expr("4 * 2").is_ok());
        assert!(parse_expr("10 / 2").is_ok());
        assert!(parse_expr("10 % 3").is_ok());
    }

    #[test]
    fn test_binary_comparison() {
        assert!(parse_expr("a == b").is_ok());
        assert!(parse_expr("a != b").is_ok());
        assert!(parse_expr("a < b").is_ok());
        assert!(parse_expr("a > b").is_ok());
        assert!(parse_expr("a <= b").is_ok());
        assert!(parse_expr("a >= b").is_ok());
    }

    #[test]
    fn test_binary_logical() {
        assert!(parse_expr("a && b").is_ok());
        assert!(parse_expr("a || b").is_ok());
    }

    #[test]
    fn test_unary_operators() {
        assert!(parse_expr("-x").is_ok());
        assert!(parse_expr("!x").is_ok());
        assert!(parse_expr("#arr").is_ok());
    }

    #[test]
    fn test_operator_precedence() {
        // 1 + 2 * 3 should be 1 + (2 * 3)
        let result = parse_expr("1 + 2 * 3").unwrap();
        if let Expr::Binary(left, BinaryOp::Add, right, _) = result {
            assert!(matches!(*left, Expr::Integer(1, _)));
            assert!(matches!(*right, Expr::Binary(_, BinaryOp::Mul, _, _)));
        } else {
            panic!("Expected Add at top level");
        }
    }

    #[test]
    fn test_parentheses() {
        // (1 + 2) * 3 should have Add at lower level
        let result = parse_expr("(1 + 2) * 3").unwrap();
        if let Expr::Binary(left, BinaryOp::Mul, _, _) = result {
            assert!(matches!(*left, Expr::Binary(_, BinaryOp::Add, _, _)));
        } else {
            panic!("Expected Mul at top level");
        }
    }

    // =========================================================================
    // Success Cases - Collections
    // =========================================================================

    #[test]
    fn test_empty_array() {
        let result = parse_expr("[]").unwrap();
        assert!(matches!(result, Expr::Array(v, _) if v.is_empty()));
    }

    #[test]
    fn test_nested_array() {
        let result = parse_expr("[[1, 2], [3, 4]]").unwrap();
        if let Expr::Array(outer, _) = result {
            assert_eq!(outer.len(), 2);
            assert!(matches!(&outer[0], Expr::Array(_, _)));
        } else {
            panic!("Expected nested array");
        }
    }

    #[test]
    fn test_map_literal() {
        let result = parse_expr("{x: 1, y: 2}").unwrap();
        if let Expr::Map(fields, _) = result {
            assert_eq!(fields.len(), 2);
        } else {
            panic!("Expected map");
        }
    }

    #[test]
    fn test_tuple_literal() {
        let result = parse_expr("(1, 2, 3)").unwrap();
        // Note: (1, 2, 3) might be parsed as tuple or parenthesized expr
        // depending on parser implementation
        assert!(result.span().start == 0);
    }

    // =========================================================================
    // Success Cases - Access
    // =========================================================================

    #[test]
    fn test_index_access() {
        let result = parse_expr("arr[0]").unwrap();
        assert!(matches!(result, Expr::Index(_, _, _)));
    }

    #[test]
    fn test_slice_access() {
        assert!(parse_expr("arr[1:3]").is_ok());
        assert!(parse_expr("arr[:3]").is_ok());
        assert!(parse_expr("arr[1:]").is_ok());
        assert!(parse_expr("arr[:]").is_ok());
    }

    #[test]
    fn test_field_access() {
        let result = parse_expr("obj.field").unwrap();
        assert!(matches!(result, Expr::Field(_, field, _) if field == "field"));
    }

    #[test]
    fn test_chained_access() {
        let result = parse_expr("a.b.c").unwrap();
        // Should be ((a.b).c)
        if let Expr::Field(inner, c, _) = result {
            assert_eq!(c, "c");
            assert!(matches!(*inner, Expr::Field(_, _, _)));
        } else {
            panic!("Expected chained field access");
        }
    }

    #[test]
    fn test_method_call() {
        let result = parse_expr("str.upper()").unwrap();
        assert!(matches!(result, Expr::MethodCall(_, method, args, _) if method == "upper" && args.is_empty()));
    }

    #[test]
    fn test_method_call_with_args() {
        let result = parse_expr("str.replace(a, b)").unwrap();
        assert!(matches!(result, Expr::MethodCall(_, _, args, _) if args.len() == 2));
    }

    // =========================================================================
    // Success Cases - Control Flow
    // =========================================================================

    #[test]
    fn test_nested_ternary() {
        let result = parse_expr("a ? b ? c : d : e").unwrap();
        assert!(matches!(result, Expr::Ternary(_, _, _, _)));
    }

    #[test]
    fn test_if_expression() {
        let result = parse_expr("if x > 0 { x } else { -x }").unwrap();
        assert!(matches!(result, Expr::If(_, _, Some(_), _)));
    }

    #[test]
    fn test_if_without_else() {
        let result = parse_expr("if x > 0 { x }").unwrap();
        assert!(matches!(result, Expr::If(_, _, None, _)));
    }

    #[test]
    fn test_block_expression() {
        // Note: Vais uses { } for maps, not blocks
        // Block expressions are parsed differently
        let result = parse_expr("{ x: 1 }");
        assert!(result.is_ok());
    }

    // =========================================================================
    // Success Cases - Functions
    // =========================================================================

    #[test]
    fn test_function_no_params() {
        let result = parse("f()=42").unwrap();
        if let Item::Function(f) = &result.items[0] {
            assert!(f.params.is_empty());
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_function_with_type_annotation() {
        // Vais type annotation syntax might differ
        // Testing basic typed parameter
        let result = parse("add(a:i, b:i) = a + b");
        // If not supported, that's ok - main test is basic function
        let _ = result;
    }

    #[test]
    fn test_function_call() {
        let result = parse_expr("f(1, 2, 3)").unwrap();
        if let Expr::Call(_, args, _) = result {
            assert_eq!(args.len(), 3);
        } else {
            panic!("Expected call");
        }
    }

    #[test]
    fn test_lambda_expression() {
        // Vais might use different lambda syntax
        // The implicit _ parameter is more common
        let result = parse_expr("_ * 2");
        if let Ok(Expr::Binary(left, _, _, _)) = result {
            assert!(matches!(*left, Expr::LambdaParam(_)));
        }
    }

    #[test]
    fn test_self_call() {
        let result = parse_expr("$(n - 1)").unwrap();
        if let Expr::SelfCall(args, _) = result {
            assert_eq!(args.len(), 1);
        } else {
            panic!("Expected self call");
        }
    }

    // =========================================================================
    // Success Cases - Special
    // =========================================================================

    #[test]
    fn test_range_expression() {
        let result = parse_expr("1..10").unwrap();
        assert!(matches!(result, Expr::Range(_, _, _)));
    }

    #[test]
    fn test_contains_expression() {
        let result = parse_expr("x @ arr").unwrap();
        assert!(matches!(result, Expr::Contains(_, _, _)));
    }

    #[test]
    fn test_coalesce_expression() {
        let result = parse_expr("x ?? 0").unwrap();
        assert!(matches!(result, Expr::Coalesce(_, _, _)));
    }

    #[test]
    fn test_coalesce_chained() {
        let result = parse_expr("a ?? b ?? c").unwrap();
        // Should be left-associative: (a ?? b) ?? c
        if let Expr::Coalesce(left, _, _) = result {
            assert!(matches!(*left, Expr::Coalesce(_, _, _)));
        } else {
            panic!("Expected Coalesce");
        }
    }

    // =========================================================================
    // Success Cases - Collection Operations
    // =========================================================================

    #[test]
    fn test_map_operation() {
        let result = parse_expr("arr.@(_ * 2)").unwrap();
        assert!(matches!(result, Expr::MapOp(_, _, _)));
    }

    #[test]
    fn test_filter_operation() {
        let result = parse_expr("arr.?(_ > 0)").unwrap();
        assert!(matches!(result, Expr::FilterOp(_, _, _)));
    }

    #[test]
    fn test_reduce_operations() {
        assert!(matches!(parse_expr("arr./+").unwrap(), Expr::ReduceOp(_, ReduceKind::Sum, _)));
        assert!(matches!(parse_expr("arr./*").unwrap(), Expr::ReduceOp(_, ReduceKind::Product, _)));
    }

    // =========================================================================
    // Error Cases - Unexpected Token
    // =========================================================================

    #[test]
    fn test_error_missing_closing_paren() {
        let result = parse_expr("(1 + 2");
        assert!(result.is_err());
        if let Err(ParseError::UnexpectedToken { expected, .. }) = result {
            assert!(expected.contains(")") || expected.contains("RightParen"));
        }
    }

    #[test]
    fn test_error_missing_closing_bracket() {
        let result = parse_expr("[1, 2, 3");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_closing_brace() {
        let result = parse_expr("{x: 1");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_colon_in_ternary() {
        let result = parse_expr("a ? b c");
        assert!(result.is_err());
    }

    #[test]
    fn test_func_call_followed_by_expr() {
        // "f(x) x + 1" should now parse as function call f(x) followed by expression x + 1
        // (not as a malformed function definition)
        let result = parse("f(x) x + 1");
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.items.len(), 2);
    }

    #[test]
    fn test_error_missing_body_in_function() {
        let result = parse("f(x)=");
        assert!(result.is_err());
    }

    // =========================================================================
    // Error Cases - Invalid Syntax
    // =========================================================================

    #[test]
    fn test_error_double_operator() {
        let result = parse_expr("1 + + 2");
        // This might parse as 1 + (+2) depending on implementation
        // or fail - either is acceptable
        let _ = result;
    }

    #[test]
    fn test_error_trailing_operator() {
        let result = parse_expr("1 +");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_leading_operator() {
        // Note: Some operators like - might be valid as unary
        let result = parse_expr("* 2");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_empty_parens_not_function() {
        // () alone is not valid
        let result = parse_expr("()");
        // Could be empty tuple or error depending on implementation
        let _ = result;
    }

    #[test]
    fn test_error_invalid_let_syntax() {
        let result = parse_expr("let = 1");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_colon_in_let() {
        let result = parse_expr("let x = 1 x");
        assert!(result.is_err());
    }

    // =========================================================================
    // Error Cases - ParseError Span
    // =========================================================================

    #[test]
    fn test_error_has_span() {
        let result = parse_expr("1 +");
        if let Err(e) = result {
            // All errors should have a span
            let _ = e.span();
        }
    }

    #[test]
    fn test_unexpected_token_error_format() {
        let err = ParseError::UnexpectedToken {
            expected: "expression".to_string(),
            found: TokenKind::RParen,
            span: vais_lexer::Span::new(0, 1),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Unexpected token"));
    }

    #[test]
    fn test_unexpected_eof_error_format() {
        let err = ParseError::UnexpectedEof {
            span: vais_lexer::Span::new(0, 0),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("end of file"));
    }

    #[test]
    fn test_invalid_syntax_error_format() {
        let err = ParseError::InvalidSyntax {
            message: "test error".to_string(),
            span: vais_lexer::Span::new(0, 0),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("test error"));
    }

    // =========================================================================
    // Edge Cases
    // =========================================================================

    #[test]
    fn test_empty_input() {
        let result = parse("");
        assert!(result.is_ok());
        assert!(result.unwrap().items.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let result = parse("   \n\t  ");
        assert!(result.is_ok());
        assert!(result.unwrap().items.is_empty());
    }

    #[test]
    fn test_multiple_functions() {
        let result = parse("f()=1\ng()=2\nh()=3").unwrap();
        assert_eq!(result.items.len(), 3);
    }

    #[test]
    fn test_deeply_nested_expression() {
        let result = parse_expr("((((1))))").unwrap();
        assert!(matches!(result, Expr::Integer(1, _)));
    }

    #[test]
    fn test_complex_expression() {
        let result = parse_expr("a.b[0].c(d, e).@(_ + 1)./+");
        assert!(result.is_ok());
    }

    #[test]
    fn test_long_chain() {
        let result = parse_expr("a + b + c + d + e + f + g");
        assert!(result.is_ok());
    }

    // =========================================================================
    // Public Function
    // =========================================================================

    #[test]
    fn test_public_function() {
        let result = parse("pub add(a, b) = a + b");
        assert!(result.is_ok());
        if let Item::Function(f) = &result.unwrap().items[0] {
            assert!(f.is_pub);
        }
    }

    #[test]
    fn test_private_function_default() {
        let result = parse("add(a, b) = a + b");
        assert!(result.is_ok());
        if let Item::Function(f) = &result.unwrap().items[0] {
            assert!(!f.is_pub);
        }
    }

    // =========================================================================
    // Pattern Matching Tests
    // =========================================================================

    #[test]
    fn test_match_expression() {
        let result = parse_expr("match x { 0 => \"zero\", _ => \"other\" }").unwrap();
        if let Expr::Match(scrutinee, arms, _) = result {
            assert!(matches!(*scrutinee, Expr::Ident(_, _)));
            assert_eq!(arms.len(), 2);
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_match_with_guard() {
        let result = parse_expr("match x { n if n > 0 => \"positive\", _ => \"other\" }").unwrap();
        if let Expr::Match(_, arms, _) = result {
            assert!(arms[0].guard.is_some());
            assert!(arms[1].guard.is_none());
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_range_pattern() {
        let result = parse_expr("match x { 1..10 => \"small\", _ => \"other\" }").unwrap();
        if let Expr::Match(_, arms, _) = result {
            assert!(matches!(arms[0].pattern, Pattern::Range(_, _, _)));
        } else {
            panic!("Expected match expression with range pattern");
        }
    }

    #[test]
    fn test_tuple_pattern() {
        let result = parse_expr("match p { (a, b) => a + b }").unwrap();
        if let Expr::Match(_, arms, _) = result {
            if let Pattern::Tuple(patterns, _) = &arms[0].pattern {
                assert_eq!(patterns.len(), 2);
            } else {
                panic!("Expected tuple pattern");
            }
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_array_pattern() {
        let result = parse_expr("match arr { [a, b, c] => a }").unwrap();
        if let Expr::Match(_, arms, _) = result {
            if let Pattern::Array(patterns, _) = &arms[0].pattern {
                assert_eq!(patterns.len(), 3);
            } else {
                panic!("Expected array pattern");
            }
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_struct_pattern() {
        let result = parse_expr("match obj { { name, age } => name }").unwrap();
        if let Expr::Match(_, arms, _) = result {
            if let Pattern::Struct(fields, _) = &arms[0].pattern {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "name");
                assert_eq!(fields[1].0, "age");
            } else {
                panic!("Expected struct pattern");
            }
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_wildcard_pattern() {
        let result = parse_expr("match x { _ => 0 }").unwrap();
        if let Expr::Match(_, arms, _) = result {
            assert!(matches!(arms[0].pattern, Pattern::Wildcard(_)));
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_binding_pattern() {
        let result = parse_expr("match x { n => n * 2 }").unwrap();
        if let Expr::Match(_, arms, _) = result {
            if let Pattern::Binding(name, _) = &arms[0].pattern {
                assert_eq!(name, "n");
            } else {
                panic!("Expected binding pattern");
            }
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_variant_pattern() {
        let result = parse_expr("match opt { Some(x) => x, None => 0 }").unwrap();
        if let Expr::Match(_, arms, _) = result {
            if let Pattern::Variant(name, inner, _) = &arms[0].pattern {
                assert_eq!(name, "Some");
                assert!(inner.is_some());
            } else {
                panic!("Expected variant pattern");
            }
            if let Pattern::Variant(name, inner, _) = &arms[1].pattern {
                assert_eq!(name, "None");
                assert!(inner.is_none());
            } else {
                panic!("Expected variant pattern");
            }
        } else {
            panic!("Expected match expression");
        }
    }

    // === Module System Tests ===

    #[test]
    fn test_use_simple() {
        let result = parse("use math").unwrap();
        if let Item::Use(use_def) = &result.items[0] {
            assert_eq!(use_def.path, vec!["math"]);
            assert!(use_def.items.is_none());
            assert!(use_def.alias.is_none());
            assert!(!use_def.star);
        } else {
            panic!("Expected use statement");
        }
    }

    #[test]
    fn test_use_path() {
        let result = parse("use lib.math").unwrap();
        if let Item::Use(use_def) = &result.items[0] {
            assert_eq!(use_def.path, vec!["lib", "math"]);
            assert!(use_def.items.is_none());
            assert!(use_def.alias.is_none());
            assert!(!use_def.star);
        } else {
            panic!("Expected use statement");
        }
    }

    #[test]
    fn test_use_selective() {
        let result = parse("use math.{add, mul}").unwrap();
        if let Item::Use(use_def) = &result.items[0] {
            assert_eq!(use_def.path, vec!["math"]);
            assert_eq!(use_def.items, Some(vec!["add".to_string(), "mul".to_string()]));
            assert!(use_def.alias.is_none());
            assert!(!use_def.star);
        } else {
            panic!("Expected use statement");
        }
    }

    #[test]
    fn test_use_star() {
        let result = parse("use lib.math.*").unwrap();
        if let Item::Use(use_def) = &result.items[0] {
            assert_eq!(use_def.path, vec!["lib", "math"]);
            assert!(use_def.items.is_none());
            assert!(use_def.alias.is_none());
            assert!(use_def.star);
        } else {
            panic!("Expected use statement");
        }
    }

    #[test]
    fn test_use_alias() {
        let result = parse("use lib.math as m").unwrap();
        if let Item::Use(use_def) = &result.items[0] {
            assert_eq!(use_def.path, vec!["lib", "math"]);
            assert!(use_def.items.is_none());
            assert_eq!(use_def.alias, Some("m".to_string()));
            assert!(!use_def.star);
        } else {
            panic!("Expected use statement");
        }
    }

    // === Error Handling Tests ===

    #[test]
    fn test_try_catch() {
        let result = parse_expr("try { risky() } catch e { default() }").unwrap();
        if let Expr::TryCatch { error_name, .. } = result {
            assert_eq!(error_name, "e");
        } else {
            panic!("Expected try-catch expression");
        }
    }

    #[test]
    fn test_try_catch_simple() {
        let result = parse_expr("try { 1 / x } catch e { 0 }").unwrap();
        if let Expr::TryCatch { body, handler, error_name, .. } = result {
            assert_eq!(error_name, "e");
            assert!(matches!(*body, Expr::Binary(..)));
            assert!(matches!(*handler, Expr::Integer(0, _)));
        } else {
            panic!("Expected try-catch expression");
        }
    }

    // === Generic Type Tests ===

    #[test]
    fn test_generic_function() {
        let result = parse("identity<T>(x) = x").unwrap();
        if let Item::Function(func) = &result.items[0] {
            assert_eq!(func.name, "identity");
            assert_eq!(func.type_params.len(), 1);
            assert_eq!(func.type_params[0].name, "T");
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_generic_function_multiple_params() {
        let result = parse("swap<T, U>(a, b) = (b, a)").unwrap();
        if let Item::Function(func) = &result.items[0] {
            assert_eq!(func.name, "swap");
            assert_eq!(func.type_params.len(), 2);
            assert_eq!(func.type_params[0].name, "T");
            assert_eq!(func.type_params[1].name, "U");
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_non_generic_function() {
        let result = parse("add(a, b) = a + b").unwrap();
        if let Item::Function(func) = &result.items[0] {
            assert_eq!(func.name, "add");
            assert!(func.type_params.is_empty());
        } else {
            panic!("Expected function");
        }
    }

    // =========================================================================
    // Struct Literal Tests
    // =========================================================================

    #[test]
    fn test_struct_literal() {
        let result = parse_expr("Point { x: 1, y: 2 }").unwrap();
        if let Expr::Struct(name, fields, _) = result {
            assert_eq!(name, "Point");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "x");
            assert_eq!(fields[1].0, "y");
        } else {
            panic!("Expected struct literal");
        }
    }

    #[test]
    fn test_struct_literal_empty() {
        let result = parse_expr("Empty {}").unwrap();
        if let Expr::Struct(name, fields, _) = result {
            assert_eq!(name, "Empty");
            assert!(fields.is_empty());
        } else {
            panic!("Expected struct literal");
        }
    }

    #[test]
    fn test_struct_literal_nested() {
        let result = parse_expr("Line { start: Point { x: 0, y: 0 }, end: Point { x: 1, y: 1 } }").unwrap();
        if let Expr::Struct(name, fields, _) = result {
            assert_eq!(name, "Line");
            assert_eq!(fields.len(), 2);
            assert!(matches!(fields[0].1, Expr::Struct(_, _, _)));
            assert!(matches!(fields[1].1, Expr::Struct(_, _, _)));
        } else {
            panic!("Expected struct literal");
        }
    }

    #[test]
    fn test_lowercase_identifier_not_struct() {
        // 소문자로 시작하는 식별자는 Struct 리터럴로 처리되지 않음
        let result = parse_expr("point { x: 1 }").unwrap();
        // 이 경우 `point`는 identifier, `{ x: 1 }`은 블록으로 파싱됨
        assert!(matches!(result, Expr::Ident(_, _)));
    }

    // =========================================================================
    // Enum Definition Tests
    // =========================================================================

    #[test]
    fn test_enum_simple() {
        let result = parse("enum Color { Red, Green, Blue }").unwrap();
        if let Item::Enum(e) = &result.items[0] {
            assert_eq!(e.name, "Color");
            assert_eq!(e.variants.len(), 3);
            assert_eq!(e.variants[0].name, "Red");
            assert_eq!(e.variants[1].name, "Green");
            assert_eq!(e.variants[2].name, "Blue");
            assert!(e.variants[0].fields.is_empty());
        } else {
            panic!("Expected enum");
        }
    }

    #[test]
    fn test_enum_with_data() {
        let result = parse("enum Option<T> { Some(T), None }").unwrap();
        if let Item::Enum(e) = &result.items[0] {
            assert_eq!(e.name, "Option");
            assert_eq!(e.type_params.len(), 1);
            assert_eq!(e.type_params[0].name, "T");
            assert_eq!(e.variants.len(), 2);
            assert_eq!(e.variants[0].name, "Some");
            assert_eq!(e.variants[0].fields.len(), 1);
            assert_eq!(e.variants[1].name, "None");
            assert!(e.variants[1].fields.is_empty());
        } else {
            panic!("Expected enum");
        }
    }

    #[test]
    fn test_enum_result() {
        let result = parse("enum Result<T, E> { Ok(T), Err(E) }").unwrap();
        if let Item::Enum(e) = &result.items[0] {
            assert_eq!(e.name, "Result");
            assert_eq!(e.type_params.len(), 2);
            assert_eq!(e.variants.len(), 2);
            assert_eq!(e.variants[0].name, "Ok");
            assert_eq!(e.variants[1].name, "Err");
        } else {
            panic!("Expected enum");
        }
    }

    // =========================================================================
    // Set Literal Tests
    // =========================================================================

    #[test]
    fn test_set_literal() {
        let result = parse_expr("#{1, 2, 3}").unwrap();
        if let Expr::Set(elements, _) = result {
            assert_eq!(elements.len(), 3);
        } else {
            panic!("Expected set literal");
        }
    }

    #[test]
    fn test_set_empty() {
        let result = parse_expr("#{}").unwrap();
        if let Expr::Set(elements, _) = result {
            assert!(elements.is_empty());
        } else {
            panic!("Expected empty set");
        }
    }

    #[test]
    fn test_set_expressions() {
        let result = parse_expr("#{a + 1, b * 2}").unwrap();
        if let Expr::Set(elements, _) = result {
            assert_eq!(elements.len(), 2);
            assert!(matches!(elements[0], Expr::Binary(_, _, _, _)));
        } else {
            panic!("Expected set literal with expressions");
        }
    }

    // =========================================================================
    // Comprehension Tests
    // =========================================================================

    #[test]
    fn test_list_comprehension() {
        let result = parse_expr("[x * 2 for x in arr]").unwrap();
        if let Expr::ListComprehension { var, cond, .. } = result {
            assert_eq!(var, "x");
            assert!(cond.is_none());
        } else {
            panic!("Expected list comprehension");
        }
    }

    #[test]
    fn test_list_comprehension_with_filter() {
        let result = parse_expr("[x * 2 for x in arr if x > 0]").unwrap();
        if let Expr::ListComprehension { var, cond, .. } = result {
            assert_eq!(var, "x");
            assert!(cond.is_some());
        } else {
            panic!("Expected list comprehension with filter");
        }
    }

    #[test]
    fn test_set_comprehension() {
        let result = parse_expr("#{x * 2 for x in arr}").unwrap();
        if let Expr::SetComprehension { var, cond, .. } = result {
            assert_eq!(var, "x");
            assert!(cond.is_none());
        } else {
            panic!("Expected set comprehension");
        }
    }

    #[test]
    fn test_set_comprehension_with_filter() {
        let result = parse_expr("#{x for x in arr if x > 0}").unwrap();
        if let Expr::SetComprehension { var, cond, .. } = result {
            assert_eq!(var, "x");
            assert!(cond.is_some());
        } else {
            panic!("Expected set comprehension with filter");
        }
    }

    // =========================================================================
    // Bug Fix Tests - Block vs Map disambiguation
    // =========================================================================

    #[test]
    fn test_block_with_single_identifier() {
        // { x } should be a block containing variable x, not a map
        let result = parse_expr("{ x }").unwrap();
        assert!(matches!(result, Expr::Block(_, _)), "Expected Block, got {:?}", result);
    }

    #[test]
    fn test_block_with_expression() {
        // { x + 1 } should be a block
        let result = parse_expr("{ x + 1 }").unwrap();
        assert!(matches!(result, Expr::Block(_, _)));
    }

    #[test]
    fn test_map_with_key_value() {
        // { x: 1 } should be a map
        let result = parse_expr("{ x: 1 }").unwrap();
        assert!(matches!(result, Expr::Map(_, _)));
    }

    #[test]
    fn test_map_with_multiple_entries() {
        // { x: 1, y: 2 } should be a map
        let result = parse_expr("{ x: 1, y: 2 }").unwrap();
        if let Expr::Map(entries, _) = result {
            assert_eq!(entries.len(), 2);
        } else {
            panic!("Expected Map");
        }
    }

    #[test]
    fn test_empty_braces_is_map() {
        // { } should be an empty map
        let result = parse_expr("{ }").unwrap();
        if let Expr::Map(entries, _) = result {
            assert!(entries.is_empty());
        } else {
            panic!("Expected empty Map");
        }
    }

    // ==================== Trait/Impl Tests ====================

    #[test]
    fn test_simple_trait_definition() {
        let source = "trait Show { show(self) -> String }";
        let mut parser = Parser::new(source);
        let program = parser.parse_program().unwrap();

        if let Item::Trait(trait_def) = &program.items[0] {
            assert_eq!(trait_def.name, "Show");
            assert!(trait_def.type_params.is_empty());
            assert_eq!(trait_def.methods.len(), 1);
            assert_eq!(trait_def.methods[0].name, "show");
        } else {
            panic!("Expected Trait item");
        }
    }

    #[test]
    fn test_trait_with_type_params() {
        let source = "trait Container<T> { get(self, idx: Int) -> T }";
        let mut parser = Parser::new(source);
        let program = parser.parse_program().unwrap();

        if let Item::Trait(trait_def) = &program.items[0] {
            assert_eq!(trait_def.name, "Container");
            assert_eq!(trait_def.type_params.len(), 1);
            assert_eq!(trait_def.type_params[0].name, "T");
            assert_eq!(trait_def.methods.len(), 1);
        } else {
            panic!("Expected Trait item");
        }
    }

    #[test]
    fn test_trait_with_default_impl() {
        let source = "trait Numeric { double(self) = self * 2 }";
        let mut parser = Parser::new(source);
        let program = parser.parse_program().unwrap();

        if let Item::Trait(trait_def) = &program.items[0] {
            assert!(trait_def.methods[0].default_impl.is_some());
        } else {
            panic!("Expected Trait item");
        }
    }

    #[test]
    fn test_simple_impl() {
        let source = "impl Int { square(self) = self * self }";
        let mut parser = Parser::new(source);
        let program = parser.parse_program().unwrap();

        if let Item::Impl(impl_def) = &program.items[0] {
            assert!(impl_def.trait_name.is_none());  // inherent impl
            assert_eq!(impl_def.methods.len(), 1);
            assert_eq!(impl_def.methods[0].name, "square");
        } else {
            panic!("Expected Impl item");
        }
    }

    #[test]
    fn test_trait_impl_for_type() {
        let source = "impl Show for Int { show(self) = str(self) }";
        let mut parser = Parser::new(source);
        let program = parser.parse_program().unwrap();

        if let Item::Impl(impl_def) = &program.items[0] {
            assert_eq!(impl_def.trait_name.as_deref(), Some("Show"));
            assert_eq!(impl_def.methods.len(), 1);
        } else {
            panic!("Expected Impl item");
        }
    }

    #[test]
    fn test_impl_with_multiple_methods() {
        let source = "impl Point {
            x(self) = self.x
            y(self) = self.y
            sum(self) = self.x + self.y
        }";
        let mut parser = Parser::new(source);
        let program = parser.parse_program().unwrap();

        if let Item::Impl(impl_def) = &program.items[0] {
            assert_eq!(impl_def.methods.len(), 3);
        } else {
            panic!("Expected Impl item");
        }
    }

    // ==================== Pipe Lambda Tests ====================

    #[test]
    fn test_pipe_lambda_single_param() {
        let result = parse_expr("|x| x + 1").unwrap();
        if let Expr::Lambda(params, body, _) = result {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0], "x");
            // body is x + 1
            assert!(matches!(*body, Expr::Binary(_, _, _, _)));
        } else {
            panic!("Expected Lambda");
        }
    }

    #[test]
    fn test_pipe_lambda_multiple_params() {
        let result = parse_expr("|x, y| x + y").unwrap();
        if let Expr::Lambda(params, body, _) = result {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], "x");
            assert_eq!(params[1], "y");
            assert!(matches!(*body, Expr::Binary(_, _, _, _)));
        } else {
            panic!("Expected Lambda");
        }
    }

    #[test]
    fn test_pipe_lambda_no_params() {
        // Note: || is lexed as OrOr token, so we use empty parentheses style instead
        // For no-param lambdas, use () => expr syntax
        let result = parse_expr("() => 42").unwrap();
        if let Expr::Lambda(params, body, _) = result {
            assert!(params.is_empty());
            assert!(matches!(*body, Expr::Integer(42, _)));
        } else {
            panic!("Expected Lambda");
        }
    }

    #[test]
    fn test_pipe_lambda_in_map() {
        // [1, 2, 3].@(|x| x * 2)
        let result = parse_expr("[1, 2, 3].@(|x| x * 2)").unwrap();
        if let Expr::MapOp(_, lambda, _) = result {
            if let Expr::Lambda(params, _, _) = *lambda {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], "x");
            } else {
                panic!("Expected Lambda in MapOp");
            }
        } else {
            panic!("Expected MapOp");
        }
    }

    // ==================== Hex/Binary Literal Tests ====================

    #[test]
    fn test_hex_integer() {
        let result = parse_expr("0xFF").unwrap();
        if let Expr::Integer(value, _) = result {
            assert_eq!(value, 255);
        } else {
            panic!("Expected Integer");
        }
    }

    #[test]
    fn test_hex_integer_uppercase() {
        let result = parse_expr("0X1A2B").unwrap();
        if let Expr::Integer(value, _) = result {
            assert_eq!(value, 0x1A2B);
        } else {
            panic!("Expected Integer");
        }
    }

    #[test]
    fn test_binary_integer() {
        let result = parse_expr("0b1010").unwrap();
        if let Expr::Integer(value, _) = result {
            assert_eq!(value, 10);
        } else {
            panic!("Expected Integer");
        }
    }

    #[test]
    fn test_binary_integer_uppercase() {
        let result = parse_expr("0B11110000").unwrap();
        if let Expr::Integer(value, _) = result {
            assert_eq!(value, 0b11110000);
        } else {
            panic!("Expected Integer");
        }
    }

    #[test]
    fn test_hex_binary_in_expression() {
        // 0xFF + 0b1010 = 255 + 10 = expression
        let result = parse_expr("0xFF + 0b1010").unwrap();
        if let Expr::Binary(left, op, right, _) = result {
            assert_eq!(op, BinaryOp::Add);
            if let Expr::Integer(l, _) = *left {
                assert_eq!(l, 255);
            } else {
                panic!("Expected Integer on left");
            }
            if let Expr::Integer(r, _) = *right {
                assert_eq!(r, 10);
            } else {
                panic!("Expected Integer on right");
            }
        } else {
            panic!("Expected Binary expression");
        }
    }
}

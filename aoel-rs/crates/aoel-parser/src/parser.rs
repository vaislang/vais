//! AOEL AOEL Parser Implementation
//!
//! Pratt parser를 사용한 표현식 파싱

use aoel_ast::*;
use aoel_lexer::{Lexer, Span, Token, TokenKind};

use crate::error::{ParseError, ParseResult};

/// AOEL AOEL Parser
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

    /// 줄바꿈 제외한 다음 토큰
    fn next_significant_token(lexer: &mut Lexer<'src>) -> Token {
        loop {
            match lexer.next_token() {
                Some(token) if token.kind == TokenKind::Newline => continue,
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
        // pub 키워드 확인
        let is_pub = self.match_token(TokenKind::Pub);

        match self.current.kind {
            TokenKind::Mod => {
                self.advance();
                let name = self.expect_identifier()?;
                Ok(Item::Module(ModuleDef {
                    name,
                    span: self.previous.span,
                }))
            }
            TokenKind::Use => {
                self.advance();
                let use_def = self.parse_use()?;
                Ok(Item::Use(use_def))
            }
            TokenKind::Type => {
                self.advance();
                let type_def = self.parse_type_def(is_pub)?;
                Ok(Item::TypeDef(type_def))
            }
            TokenKind::Ffi => {
                if is_pub {
                    return Err(ParseError::InvalidSyntax {
                        message: "ffi blocks cannot be pub".to_string(),
                        span: self.current.span,
                    });
                }
                self.advance();
                let ffi_block = self.parse_ffi_block()?;
                Ok(Item::Ffi(ffi_block))
            }
            TokenKind::Identifier => {
                // 함수 정의: name(params) = body
                // 또는 함수 호출/표현식
                // Lookahead: identifier( 다음이 identifier 또는 ) 이면 함수 정의
                if self.is_function_def() {
                    let func = self.parse_function_def(is_pub)?;
                    Ok(Item::Function(func))
                } else {
                    if is_pub {
                        return Err(ParseError::InvalidSyntax {
                            message: "pub can only be used with functions, types, or modules"
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
                let expr = self.parse_expr()?;
                Ok(Item::Expr(expr))
            }
        }
    }

    /// 함수 정의인지 확인 (lookahead)
    fn is_function_def(&self) -> bool {
        // 함수 정의: name(params) = body 또는 name() = body
        // 함수 호출: name(args) 또는 name()
        // 구분: ) 다음에 = 또는 -> 가 있으면 함수 정의
        let mut lexer_clone = self.lexer.clone();

        // 현재 토큰은 Identifier, 다음 토큰을 확인
        let next = Self::next_significant_token(&mut lexer_clone);
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
                // f(x 형태 - 다음이 , 또는 ) 또는 : 이면 함수 정의
                let next_after_id = Self::next_significant_token(&mut lexer_clone);
                matches!(next_after_id.kind, TokenKind::Comma | TokenKind::RParen | TokenKind::Colon)
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

    /// 함수 정의 파싱: name(params) = body
    fn parse_function_def(&mut self, is_pub: bool) -> ParseResult<FunctionDef> {
        let start = self.current.span;
        let name = self.expect_identifier()?;

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
            params,
            return_type,
            body,
            is_pub,
            span: start.merge(self.previous.span),
        })
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
                    let (k, v) = fields.pop().unwrap();
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
    fn parse_use(&mut self) -> ParseResult<UseDef> {
        let start = self.previous.span;
        let mut path = vec![self.expect_identifier()?];

        while self.match_token(TokenKind::Dot) {
            if self.check(TokenKind::LBrace) {
                // use path.{a, b}
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
                    span: start.merge(self.previous.span),
                });
            }
            path.push(self.expect_identifier()?);
        }

        Ok(UseDef {
            path,
            items: None,
            alias: None,
            span: start.merge(self.previous.span),
        })
    }

    /// 타입 정의 파싱
    fn parse_type_def(&mut self, is_pub: bool) -> ParseResult<TypeDef> {
        let start = self.previous.span;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Eq)?;
        let ty = self.parse_type()?;

        Ok(TypeDef {
            name,
            ty,
            is_pub,
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

        // 외부 이름 (옵션): fn aoel_name = "external_name"
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
    // 표현식 파싱 (Pratt Parser)
    // =========================================================================

    /// 표현식 파싱
    fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_ternary()
    }

    /// 삼항 연산자: cond ? then : else
    fn parse_ternary(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;
        let mut expr = self.parse_or()?;

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

    /// 비교: a < b, a >= b
    fn parse_comparison(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;
        let mut expr = self.parse_range()?;

        loop {
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

    /// 단항: -a, !a, #a
    fn parse_unary(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;

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
                // 필드 접근 또는 메서드 호출: a.b 또는 a.b()
                TokenKind::Dot => {
                    self.advance();
                    let name = self.expect_identifier()?;
                    if self.match_token(TokenKind::LParen) {
                        // 메서드 호출
                        let args = self.parse_args()?;
                        self.expect(TokenKind::RParen)?;
                        let span = start.merge(self.previous.span);
                        expr = Expr::MethodCall(Box::new(expr), name, args, span);
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

            // 식별자
            TokenKind::Identifier => {
                let name = self.current.text.clone();
                self.advance();
                Ok(Expr::Ident(name, span))
            }

            // 람다 파라미터
            TokenKind::Underscore => {
                self.advance();
                Ok(Expr::LambdaParam(span))
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

            // 배열: [a, b, c]
            TokenKind::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                if !self.check(TokenKind::RBracket) {
                    loop {
                        elements.push(self.parse_expr()?);
                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RBracket)?;
                let end_span = self.previous.span;
                Ok(Expr::Array(elements, span.merge(end_span)))
            }

            // 블록 또는 맵: { ... }
            TokenKind::LBrace => {
                self.advance();
                self.parse_brace_expr(span)
            }

            // 그룹 또는 튜플: (expr) 또는 (a, b)
            TokenKind::LParen => {
                self.advance();
                if self.check(TokenKind::RParen) {
                    // 빈 튜플
                    self.advance();
                    let end_span = self.previous.span;
                    return Ok(Expr::Tuple(vec![], span.merge(end_span)));
                }

                let first = self.parse_expr()?;

                if self.match_token(TokenKind::Comma) {
                    // 튜플
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
                    let end_span = self.previous.span;
                    Ok(Expr::Tuple(elements, span.merge(end_span)))
                } else {
                    // 그룹
                    self.expect(TokenKind::RParen)?;
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

            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: self.current.kind.clone(),
                span,
            }),
        }
    }

    /// 중괄호 표현식 파싱 (블록 또는 맵)
    fn parse_brace_expr(&mut self, start: Span) -> ParseResult<Expr> {
        if self.check(TokenKind::RBrace) {
            // 빈 맵
            self.advance();
            return Ok(Expr::Map(vec![], start.merge(self.previous.span)));
        }

        // 첫 번째 요소로 블록인지 맵인지 결정
        if self.check(TokenKind::Identifier) {
            // 다음이 : 이면 맵
            // 간단히 맵으로 처리
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

        // 블록으로 처리
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

    /// let 표현식 파싱: let x = v : body 또는 let x = v, y = w : body
    fn parse_let(&mut self, start: Span) -> ParseResult<Expr> {
        let mut bindings = Vec::new();

        loop {
            let name = self.expect_identifier()?;
            self.expect(TokenKind::Eq)?;
            let value = self.parse_expr()?;
            bindings.push((name, value));

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

    /// if 표현식 파싱
    fn parse_if(&mut self, start: Span) -> ParseResult<Expr> {
        let cond = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;
        let then_expr = self.parse_expr()?;
        self.expect(TokenKind::RBrace)?;

        let else_expr = if self.match_token(TokenKind::Else) {
            self.expect(TokenKind::LBrace)?;
            let e = self.parse_expr()?;
            self.expect(TokenKind::RBrace)?;
            Some(Box::new(e))
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

            // 리터럴: 숫자, 문자열, bool
            TokenKind::Integer | TokenKind::Float | TokenKind::String
            | TokenKind::True | TokenKind::False | TokenKind::Nil => {
                let expr = self.parse_primary()?;
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
                    // 일반 바인딩
                    Ok(Pattern::Binding(name, span))
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
}

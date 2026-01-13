//! Vais Vais Token Definitions
//!
//! Vais 문법에 최적화된 토큰 정의.
//! 목표: AI 토큰 효율성 44% 향상

use logos::Logos;
use serde::{Deserialize, Serialize};

/// 소스 코드 내 위치
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// 토큰과 위치 정보
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, text: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            text: text.into(),
        }
    }
}

/// Vais Vais 토큰 종류
#[derive(Logos, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[logos(skip r"[ \t\r]+")]  // 공백 스킵 (줄바꿈 제외)
pub enum TokenKind {
    // =========================================================================
    // Keywords (최소한으로)
    // =========================================================================
    #[token("let")]
    Let,

    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("match")]
    Match,

    #[token("for")]
    For,

    #[token("in")]
    In,

    #[token("fn")]
    Fn,

    #[token("pub")]
    Pub,

    #[token("mod")]
    Mod,

    #[token("use")]
    Use,

    #[token("type")]
    Type,

    #[token("enum")]
    Enum,

    #[token("trait")]
    Trait,

    #[token("impl")]
    Impl,

    #[token("async")]
    Async,

    #[token("await")]
    Await,

    #[token("spawn")]
    Spawn,

    #[token("mut")]
    Mut,

    #[token("as")]
    As,

    #[token("try")]
    Try,

    #[token("catch")]
    Catch,

    #[token("ffi")]
    Ffi,

    #[token("macro")]
    Macro,

    #[token("effect")]
    Effect,

    #[token("handle")]
    Handle,

    #[token("perform")]
    Perform,

    #[token("resume")]
    Resume,

    // =========================================================================
    // Literals
    // =========================================================================
    #[token("true")]
    True,

    #[token("false")]
    False,

    #[token("nil")]
    Nil,

    #[token("some")]
    Some,

    #[token("ok")]
    Ok,

    #[token("err")]
    Err,

    /// 16진수 리터럴 (0xFF, 0XFF)
    #[regex(r"0[xX][0-9a-fA-F]+", priority = 3)]
    HexInteger,

    /// 2진수 리터럴 (0b1010, 0B1010)
    #[regex(r"0[bB][01]+", priority = 3)]
    BinaryInteger,

    /// 정수 리터럴 (음수 부호는 단항 연산자로 처리)
    #[regex(r"[0-9]+", priority = 2)]
    Integer,

    /// 실수 리터럴 (음수 부호는 단항 연산자로 처리)
    #[regex(r"[0-9]+\.[0-9]+")]
    Float,

    /// 문자열 리터럴
    #[regex(r#""([^"\\]|\\.)*""#)]
    String,

    /// 식별자
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    // =========================================================================
    // Operators - Collection (Vais 핵심)
    // =========================================================================
    /// .@ - Map operator
    #[token(".@")]
    DotAt,

    /// .? - Filter operator
    #[token(".?")]
    DotQuestion,

    /// ./ - Reduce operator
    #[token("./")]
    DotSlash,

    /// .||@ - Parallel map operator
    #[token(".||@")]
    DotParMap,

    /// .||? - Parallel filter operator
    #[token(".||?")]
    DotParFilter,

    /// .||/ - Parallel reduce operator
    #[token(".||/")]
    DotParReduce,

    // =========================================================================
    // Operators - Special (Vais)
    // =========================================================================
    /// $ - Self recursion
    #[token("$")]
    Dollar,

    /// # - Length prefix
    #[token("#")]
    Hash,

    /// _ - Lambda parameter / wildcard (높은 우선순위로 식별자보다 먼저 매칭)
    #[token("_", priority = 10)]
    Underscore,

    /// @ - In operator (contains)
    #[token("@")]
    At,

    /// .. - Range operator
    #[token("..")]
    DotDot,

    // =========================================================================
    // Operators - Arithmetic
    // =========================================================================
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("%")]
    Percent,

    // =========================================================================
    // Operators - Comparison
    // =========================================================================
    #[token("==")]
    EqEq,

    #[token("!=")]
    NotEq,

    #[token("<")]
    Lt,

    #[token(">")]
    Gt,

    #[token("<=")]
    LtEq,

    #[token(">=")]
    GtEq,

    // =========================================================================
    // Operators - Logical
    // =========================================================================
    #[token("&&")]
    AndAnd,

    #[token("||")]
    OrOr,

    #[token("!")]
    Bang,

    /// Coalesce 연산자 (null 병합)
    #[token("??")]
    QuestionQuestion,

    // =========================================================================
    // Operators - Assignment & Arrow
    // =========================================================================
    #[token("=")]
    Eq,

    #[token("->")]
    Arrow,

    #[token("<-")]
    LeftArrow,

    #[token("=>")]
    FatArrow,

    // =========================================================================
    // Punctuation
    // =========================================================================
    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("#{")]
    HashBrace,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    #[token(";")]
    Semi,

    #[token(".")]
    Dot,

    #[token("?")]
    Question,

    #[token("|")]
    Pipe,

    #[token("&")]
    Ampersand,

    // =========================================================================
    // Special
    // =========================================================================
    /// 줄바꿈
    #[token("\n")]
    Newline,

    /// 주석
    #[regex(r"//[^\n]*")]
    Comment,

    /// 파일 끝
    Eof,

    /// 에러 토큰
    Error,
}

impl TokenKind {
    /// 키워드인지 확인
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Let
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::Match
                | TokenKind::For
                | TokenKind::In
                | TokenKind::Fn
                | TokenKind::Pub
                | TokenKind::Mod
                | TokenKind::Use
                | TokenKind::Type
                | TokenKind::Enum
                | TokenKind::Trait
                | TokenKind::Impl
                | TokenKind::Async
                | TokenKind::Await
                | TokenKind::Spawn
                | TokenKind::Mut
                | TokenKind::As
                | TokenKind::Try
                | TokenKind::Catch
                | TokenKind::Ffi
                | TokenKind::Macro
                | TokenKind::Effect
                | TokenKind::Handle
                | TokenKind::Perform
                | TokenKind::Resume
        )
    }

    /// 리터럴인지 확인
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            TokenKind::True
                | TokenKind::False
                | TokenKind::Nil
                | TokenKind::Some
                | TokenKind::Ok
                | TokenKind::Err
                | TokenKind::Integer
                | TokenKind::HexInteger
                | TokenKind::BinaryInteger
                | TokenKind::Float
                | TokenKind::String
        )
    }

    /// 연산자인지 확인
    pub fn is_operator(&self) -> bool {
        matches!(
            self,
            // Collection operators
            TokenKind::DotAt
                | TokenKind::DotQuestion
                | TokenKind::DotSlash
                | TokenKind::DotParMap
                | TokenKind::DotParFilter
                | TokenKind::DotParReduce
                // Special operators
                | TokenKind::Dollar
                | TokenKind::Hash
                | TokenKind::Underscore
                | TokenKind::At
                | TokenKind::DotDot
                // Arithmetic operators
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Percent
                // Assignment & Arrow operators
                | TokenKind::Eq
                | TokenKind::Arrow
                | TokenKind::LeftArrow
                | TokenKind::FatArrow
                // Comparison operators
                | TokenKind::EqEq
                | TokenKind::NotEq
                | TokenKind::Lt
                | TokenKind::Gt
                | TokenKind::LtEq
                | TokenKind::GtEq
                // Logical operators
                | TokenKind::AndAnd
                | TokenKind::OrOr
                | TokenKind::Bang
                // Null coalescing
                | TokenKind::QuestionQuestion
        )
    }

    /// Vais 컬렉션 연산자인지 확인
    pub fn is_collection_op(&self) -> bool {
        matches!(
            self,
            TokenKind::DotAt
                | TokenKind::DotQuestion
                | TokenKind::DotSlash
                | TokenKind::DotParMap
                | TokenKind::DotParFilter
                | TokenKind::DotParReduce
        )
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Let => write!(f, "let"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Pub => write!(f, "pub"),
            TokenKind::Mod => write!(f, "mod"),
            TokenKind::Use => write!(f, "use"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::Trait => write!(f, "trait"),
            TokenKind::Impl => write!(f, "impl"),
            TokenKind::Async => write!(f, "async"),
            TokenKind::Await => write!(f, "await"),
            TokenKind::Spawn => write!(f, "spawn"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::As => write!(f, "as"),
            TokenKind::Try => write!(f, "try"),
            TokenKind::Catch => write!(f, "catch"),
            TokenKind::Ffi => write!(f, "ffi"),
            TokenKind::Macro => write!(f, "macro"),
            TokenKind::Effect => write!(f, "effect"),
            TokenKind::Handle => write!(f, "handle"),
            TokenKind::Perform => write!(f, "perform"),
            TokenKind::Resume => write!(f, "resume"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Nil => write!(f, "nil"),
            TokenKind::Some => write!(f, "some"),
            TokenKind::Ok => write!(f, "ok"),
            TokenKind::Err => write!(f, "err"),
            TokenKind::Integer => write!(f, "integer"),
            TokenKind::HexInteger => write!(f, "hex integer"),
            TokenKind::BinaryInteger => write!(f, "binary integer"),
            TokenKind::Float => write!(f, "float"),
            TokenKind::String => write!(f, "string"),
            TokenKind::Identifier => write!(f, "identifier"),
            TokenKind::DotAt => write!(f, ".@"),
            TokenKind::DotQuestion => write!(f, ".?"),
            TokenKind::DotSlash => write!(f, "./"),
            TokenKind::DotParMap => write!(f, ".||@"),
            TokenKind::DotParFilter => write!(f, ".||?"),
            TokenKind::DotParReduce => write!(f, ".||/"),
            TokenKind::Dollar => write!(f, "$"),
            TokenKind::Hash => write!(f, "#"),
            TokenKind::Underscore => write!(f, "_"),
            TokenKind::At => write!(f, "@"),
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::NotEq => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::LtEq => write!(f, "<="),
            TokenKind::GtEq => write!(f, ">="),
            TokenKind::AndAnd => write!(f, "&&"),
            TokenKind::OrOr => write!(f, "||"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::Eq => write!(f, "="),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::LeftArrow => write!(f, "<-"),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::HashBrace => write!(f, "#{{"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semi => write!(f, ";"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Question => write!(f, "?"),
            TokenKind::QuestionQuestion => write!(f, "??"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::Newline => write!(f, "newline"),
            TokenKind::Comment => write!(f, "comment"),
            TokenKind::Eof => write!(f, "EOF"),
            TokenKind::Error => write!(f, "error"),
        }
    }
}

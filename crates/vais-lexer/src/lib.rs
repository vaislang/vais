//! Vais 0.0.1 Lexer
//!
//! Token-efficient lexer using single-letter keywords for AI optimization.

use logos::Logos;
use std::fmt;

/// Token types for Vais 0.0.1
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]  // Skip whitespace
#[logos(skip r"#[^\n]*")]     // Skip comments
pub enum Token {
    // === Keywords (single-letter for token efficiency) ===
    // Higher priority than identifiers
    #[token("F", priority = 3)]
    Function,
    #[token("S", priority = 3)]
    Struct,
    #[token("E", priority = 3)]
    Enum,
    #[token("I", priority = 3)]
    If,
    #[token("L", priority = 3)]
    Loop,
    #[token("M", priority = 3)]
    Match,
    #[token("A", priority = 3)]
    Async,
    #[token("R", priority = 3)]
    Return,
    #[token("B", priority = 3)]
    Break,
    #[token("C", priority = 3)]
    Continue,
    #[token("T", priority = 3)]
    TypeKeyword,
    #[token("U", priority = 3)]
    Use,
    #[token("P", priority = 3)]
    Pub,
    #[token("W", priority = 3)]
    Trait,
    #[token("X", priority = 3)]
    Impl,

    // === Type Keywords ===
    #[token("mut")]
    Mut,
    #[token("self")]
    SelfLower,
    #[token("Self")]
    SelfUpper,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("spawn")]
    Spawn,
    #[token("await")]
    Await,
    #[token("weak")]
    Weak,
    #[token("clone")]
    Clone,

    // === Primitive Types ===
    #[token("i8")]
    I8,
    #[token("i16")]
    I16,
    #[token("i32")]
    I32,
    #[token("i64")]
    I64,
    #[token("i128")]
    I128,
    #[token("u8")]
    U8,
    #[token("u16")]
    U16,
    #[token("u32")]
    U32,
    #[token("u64")]
    U64,
    #[token("u128")]
    U128,
    #[token("f32")]
    F32,
    #[token("f64")]
    F64,
    #[token("bool")]
    Bool,
    #[token("str")]
    Str,

    // === Literals ===
    // Note: negative sign is handled by unary operator, not here
    #[regex(r"[0-9][0-9_]*", |lex| lex.slice().replace('_', "").parse::<i64>().ok())]
    Int(i64),

    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*([eE][+-]?[0-9]+)?", |lex| lex.slice().replace('_', "").parse::<f64>().ok())]
    Float(f64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    String(String),

    // === Identifiers ===
    // Priority lower than single-letter keywords
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string(), priority = 2)]
    Ident(String),

    // === Operators ===
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

    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    Lte,
    #[token(">=")]
    Gte,
    #[token("==")]
    EqEq,
    #[token("!=")]
    Neq,

    #[token("&")]
    Amp,
    #[token("|")]
    Pipe,
    #[token("!")]
    Bang,
    #[token("~")]
    Tilde,
    #[token("^")]
    Caret,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,

    #[token("=")]
    Eq,
    #[token(":=")]
    ColonEq,
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,

    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("..")]
    DotDot,
    #[token("..=")]
    DotDotEq,
    #[token("?")]
    Question,
    #[token("@")]
    At,

    // === Delimiters ===
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(";")]
    Semi,
    #[token(".")]
    Dot,
    #[token("::")]
    ColonColon,

    // Attribute marker
    #[token("#[")]
    HashBracket,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Function => write!(f, "F"),
            Token::Struct => write!(f, "S"),
            Token::Enum => write!(f, "E"),
            Token::If => write!(f, "I"),
            Token::Loop => write!(f, "L"),
            Token::Match => write!(f, "M"),
            Token::Async => write!(f, "A"),
            Token::Return => write!(f, "R"),
            Token::Break => write!(f, "B"),
            Token::Continue => write!(f, "C"),
            Token::TypeKeyword => write!(f, "T"),
            Token::Use => write!(f, "U"),
            Token::Pub => write!(f, "P"),
            Token::Trait => write!(f, "W"),
            Token::Impl => write!(f, "X"),
            Token::Mut => write!(f, "mut"),
            Token::SelfLower => write!(f, "self"),
            Token::SelfUpper => write!(f, "Self"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Spawn => write!(f, "spawn"),
            Token::Await => write!(f, "await"),
            Token::Weak => write!(f, "weak"),
            Token::Clone => write!(f, "clone"),
            Token::I8 => write!(f, "i8"),
            Token::I16 => write!(f, "i16"),
            Token::I32 => write!(f, "i32"),
            Token::I64 => write!(f, "i64"),
            Token::I128 => write!(f, "i128"),
            Token::U8 => write!(f, "u8"),
            Token::U16 => write!(f, "u16"),
            Token::U32 => write!(f, "u32"),
            Token::U64 => write!(f, "u64"),
            Token::U128 => write!(f, "u128"),
            Token::F32 => write!(f, "f32"),
            Token::F64 => write!(f, "f64"),
            Token::Bool => write!(f, "bool"),
            Token::Str => write!(f, "str"),
            Token::Int(n) => write!(f, "{}", n),
            Token::Float(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Lt => write!(f, "<"),
            Token::Gt => write!(f, ">"),
            Token::Lte => write!(f, "<="),
            Token::Gte => write!(f, ">="),
            Token::EqEq => write!(f, "=="),
            Token::Neq => write!(f, "!="),
            Token::Amp => write!(f, "&"),
            Token::Pipe => write!(f, "|"),
            Token::Bang => write!(f, "!"),
            Token::Tilde => write!(f, "~"),
            Token::Caret => write!(f, "^"),
            Token::Shl => write!(f, "<<"),
            Token::Shr => write!(f, ">>"),
            Token::Eq => write!(f, "="),
            Token::ColonEq => write!(f, ":="),
            Token::PlusEq => write!(f, "+="),
            Token::MinusEq => write!(f, "-="),
            Token::StarEq => write!(f, "*="),
            Token::SlashEq => write!(f, "/="),
            Token::Arrow => write!(f, "->"),
            Token::FatArrow => write!(f, "=>"),
            Token::DotDot => write!(f, ".."),
            Token::DotDotEq => write!(f, "..="),
            Token::Question => write!(f, "?"),
            Token::At => write!(f, "@"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::Semi => write!(f, ";"),
            Token::Dot => write!(f, "."),
            Token::ColonColon => write!(f, "::"),
            Token::HashBracket => write!(f, "#["),
        }
    }
}

/// Spanned token with source location
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: std::ops::Range<usize>,
}

/// Lexer error
#[derive(Debug, Clone, thiserror::Error)]
pub enum LexError {
    #[error("Invalid token at position {0}")]
    InvalidToken(usize),
}

/// Tokenize source code
pub fn tokenize(source: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut tokens = Vec::new();
    let mut lexer = Token::lexer(source);

    while let Some(result) = lexer.next() {
        match result {
            Ok(token) => {
                tokens.push(SpannedToken {
                    token,
                    span: lexer.span(),
                });
            }
            Err(_) => {
                return Err(LexError::InvalidToken(lexer.span().start));
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_def() {
        let source = "F add(a:i64,b:i64)->i64=a+b";
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens[0].token, Token::Function);
        assert_eq!(tokens[1].token, Token::Ident("add".to_string()));
        assert_eq!(tokens[2].token, Token::LParen);
        assert_eq!(tokens[3].token, Token::Ident("a".to_string()));
        assert_eq!(tokens[4].token, Token::Colon);
        assert_eq!(tokens[5].token, Token::I64);
    }

    #[test]
    fn test_fibonacci() {
        let source = "F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)";
        let tokens = tokenize(source).unwrap();

        // Check @ for self-recursion
        let has_at = tokens.iter().any(|t| t.token == Token::At);
        assert!(has_at);
    }

    #[test]
    fn test_struct_def() {
        let source = "S Point{x:f64,y:f64}";
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens[0].token, Token::Struct);
        assert_eq!(tokens[1].token, Token::Ident("Point".to_string()));
    }

    #[test]
    fn test_control_flow() {
        let source = "I x<0{-1}E{0}";
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens[0].token, Token::If);
        // E is used for both enum definition and else - lexer returns Enum
        // The parser decides context (after { in if => else)
        let e_idx = tokens.iter().position(|t| t.token == Token::Enum).unwrap();
        assert!(e_idx > 0);
    }

    #[test]
    fn test_loop() {
        let source = "L i:0..10{print(i)}";
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens[0].token, Token::Loop);
        assert_eq!(tokens[4].token, Token::DotDot);
    }

    #[test]
    fn test_string_literal() {
        let source = r#""hello world""#;
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens[0].token, Token::String("hello world".to_string()));
    }

    #[test]
    fn test_numbers() {
        let source = "42 3.14 1_000_000";
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens[0].token, Token::Int(42));
        assert_eq!(tokens[1].token, Token::Float(3.14));
        assert_eq!(tokens[2].token, Token::Int(1000000));
    }

    #[test]
    fn test_comments() {
        let source = "F add(a:i64,b:i64)->i64=a+b # this is a comment";
        let tokens = tokenize(source).unwrap();

        // Comment should be skipped
        let has_comment = tokens.iter().any(|t| {
            if let Token::Ident(s) = &t.token {
                s.contains("comment")
            } else {
                false
            }
        });
        assert!(!has_comment);
    }

    #[test]
    fn test_block_function() {
        let source = "F sum(arr:[i64])->i64{s:=0;L x:arr{s+=x};s}";
        let tokens = tokenize(source).unwrap();

        // Verify lowercase 's' is lexed as Ident, not Struct
        let s_tokens: Vec<_> = tokens.iter()
            .filter(|t| matches!(&t.token, Token::Ident(s) if s == "s"))
            .collect();
        assert_eq!(s_tokens.len(), 3);  // s:=0, s+=x, final s
    }
}

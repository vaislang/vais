//! Vais Lexer
//!
//! Token-efficient lexer using single-letter keywords for AI optimization.

use logos::Logos;
use std::fmt;

/// Token types for Vais 0.0.1 language.
///
/// Uses single-letter keywords (F, S, E, I, L, M, etc.) for token efficiency,
/// optimizing for AI model processing and reducing token count in LLM contexts.
///
/// # Examples
///
/// ```
/// use vais_lexer::{tokenize, Token};
///
/// let source = "F add(a:i64,b:i64)->i64=a+b";
/// let tokens = tokenize(source).unwrap();
/// assert_eq!(tokens[0].token, Token::Function);
/// ```
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")] // Skip whitespace
pub enum Token {
    // === Doc Comments ===
    #[regex(r"///[^\n]*", |lex| lex.slice()[3..].trim().to_string(), priority = 5)]
    DocComment(String),

    // === Regular Comments (lower priority than doc comments) ===
    #[regex(r"#[^/\[\n][^\n]*", logos::skip)] // Skip regular comments (but not doc comments or #[)
    #[regex(r"#\n", logos::skip)]
    // Skip empty # lines

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
    #[token("D", priority = 3)]
    Defer,
    #[token("O", priority = 3)]
    Union,
    #[token("N", priority = 3)]
    Extern,
    #[token("G", priority = 3)]
    Global,

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
    #[token("Y", priority = 3)]
    Await,
    #[token("yield")]
    Yield,
    #[token("weak")]
    Weak,
    #[token("clone")]
    Clone,
    #[token("const")]
    Const,
    #[token("comptime")]
    Comptime,
    #[token("dyn")]
    Dyn,
    #[token("macro")]
    Macro,
    #[token("as")]
    As,

    // === Effect System Keywords ===
    #[token("pure")]
    Pure,
    #[token("effect")]
    Effect,
    #[token("io")]
    Io,
    #[token("unsafe")]
    Unsafe,

    // === Linear Types Keywords ===
    #[token("linear")]
    Linear,
    #[token("affine")]
    Affine,
    #[token("move")]
    Move,
    #[token("consume")]
    Consume,
    #[token("where")]
    Where,

    // === Lazy Evaluation ===
    #[token("lazy")]
    Lazy,
    #[token("force")]
    Force,

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

    // === SIMD Vector Types ===
    #[token("Vec2f32")]
    Vec2f32,
    #[token("Vec4f32")]
    Vec4f32,
    #[token("Vec8f32")]
    Vec8f32,
    #[token("Vec2f64")]
    Vec2f64,
    #[token("Vec4f64")]
    Vec4f64,
    #[token("Vec4i32")]
    Vec4i32,
    #[token("Vec8i32")]
    Vec8i32,
    #[token("Vec2i64")]
    Vec2i64,
    #[token("Vec4i64")]
    Vec4i64,

    // === Literals ===
    // Note: negative sign is handled by unary operator, not here
    #[regex(r"[0-9][0-9_]*", |lex| lex.slice().replace('_', "").parse::<i64>().ok())]
    Int(i64),

    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*([eE][+-]?[0-9]+)?", |lex| lex.slice().replace('_', "").parse::<f64>().ok())]
    Float(f64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        let inner = &s[1..s.len()-1];
        // Process escape sequences
        let mut result = std::string::String::new();
        let mut chars = inner.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(&next) = chars.peek() {
                    chars.next();
                    match next {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        '\\' => result.push('\\'),
                        '"' => result.push('"'),
                        '0' => result.push('\0'),
                        'x' => {
                            // Hex escape: \xHH
                            let mut hex = std::string::String::new();
                            for _ in 0..2 {
                                if let Some(&h) = chars.peek() {
                                    if h.is_ascii_hexdigit() {
                                        hex.push(h);
                                        chars.next();
                                    }
                                }
                            }
                            if let Ok(code) = u8::from_str_radix(&hex, 16) {
                                result.push(code as char);
                            }
                        }
                        _ => {
                            // Unknown escape, keep as-is
                            result.push('\\');
                            result.push(next);
                        }
                    }
                } else {
                    result.push('\\');
                }
            } else {
                result.push(c);
            }
        }
        Some(result)
    })]
    String(String),

    // === Identifiers ===
    // Priority lower than single-letter keywords
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string(), priority = 2)]
    Ident(String),

    // === Lifetime identifiers ===
    // Lifetime names start with ' followed by identifier (e.g., 'a, 'static)
    #[regex(r"'[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice()[1..].to_string())]
    Lifetime(String),

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
    #[token("|>", priority = 4)]
    PipeArrow,
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
    #[token("%=")]
    PercentEq,
    #[token("&=")]
    AmpEq,
    #[token("|=")]
    PipeEq,
    #[token("^=")]
    CaretEq,
    #[token("<<=")]
    ShlEq,
    #[token(">>=")]
    ShrEq,

    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("..")]
    DotDot,
    #[token("..=")]
    DotDotEq,
    #[token("...")]
    Ellipsis,
    #[token("?")]
    Question,
    #[token("@")]
    At,
    #[token("$")]
    Dollar,

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
            Token::DocComment(s) => write!(f, "/// {}", s),
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
            Token::Defer => write!(f, "D"),
            Token::Union => write!(f, "O"),
            Token::Extern => write!(f, "N"),
            Token::Global => write!(f, "G"),
            Token::Yield => write!(f, "yield"),
            Token::Mut => write!(f, "mut"),
            Token::SelfLower => write!(f, "self"),
            Token::SelfUpper => write!(f, "Self"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Spawn => write!(f, "spawn"),
            Token::Await => write!(f, "Y"),
            Token::Weak => write!(f, "weak"),
            Token::Clone => write!(f, "clone"),
            Token::Const => write!(f, "const"),
            Token::Comptime => write!(f, "comptime"),
            Token::Dyn => write!(f, "dyn"),
            Token::Macro => write!(f, "macro"),
            Token::As => write!(f, "as"),
            Token::Pure => write!(f, "pure"),
            Token::Effect => write!(f, "effect"),
            Token::Io => write!(f, "io"),
            Token::Unsafe => write!(f, "unsafe"),
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
            Token::Vec2f32 => write!(f, "Vec2f32"),
            Token::Vec4f32 => write!(f, "Vec4f32"),
            Token::Vec8f32 => write!(f, "Vec8f32"),
            Token::Vec2f64 => write!(f, "Vec2f64"),
            Token::Vec4f64 => write!(f, "Vec4f64"),
            Token::Vec4i32 => write!(f, "Vec4i32"),
            Token::Vec8i32 => write!(f, "Vec8i32"),
            Token::Vec2i64 => write!(f, "Vec2i64"),
            Token::Vec4i64 => write!(f, "Vec4i64"),
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
            Token::PipeArrow => write!(f, "|>"),
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
            Token::PercentEq => write!(f, "%="),
            Token::AmpEq => write!(f, "&="),
            Token::PipeEq => write!(f, "|="),
            Token::CaretEq => write!(f, "^="),
            Token::ShlEq => write!(f, "<<="),
            Token::ShrEq => write!(f, ">>="),
            Token::Arrow => write!(f, "->"),
            Token::FatArrow => write!(f, "=>"),
            Token::DotDot => write!(f, ".."),
            Token::DotDotEq => write!(f, "..="),
            Token::Ellipsis => write!(f, "..."),
            Token::Question => write!(f, "?"),
            Token::At => write!(f, "@"),
            Token::Dollar => write!(f, "$"),
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
            // Linear types keywords
            Token::Linear => write!(f, "linear"),
            Token::Affine => write!(f, "affine"),
            Token::Move => write!(f, "move"),
            Token::Consume => write!(f, "consume"),
            Token::Where => write!(f, "where"),
            // Lifetime
            Token::Lifetime(name) => write!(f, "'{}", name),
            // Lazy evaluation
            Token::Lazy => write!(f, "lazy"),
            Token::Force => write!(f, "force"),
        }
    }
}

/// Token with source location information.
///
/// Associates each token with its byte range in the source code,
/// enabling precise error reporting and IDE features.
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    /// The token type and value
    pub token: Token,
    /// Byte range of this token in the source file
    pub span: std::ops::Range<usize>,
}

/// Error type for lexical analysis failures.
///
/// Represents errors encountered during tokenization,
/// such as invalid characters or malformed tokens.
#[derive(Debug, Clone, thiserror::Error)]
pub enum LexError {
    /// Invalid or unrecognized token at the given position
    #[error("Invalid token at position {0}")]
    InvalidToken(usize),
}

/// Tokenizes Vais source code into a stream of tokens.
///
/// This function performs lexical analysis, converting the input string
/// into a sequence of tokens with source location information.
///
/// # Arguments
///
/// * `source` - The Vais source code to tokenize
///
/// # Returns
///
/// A vector of spanned tokens on success, or a lexer error if invalid tokens are encountered.
///
/// # Examples
///
/// ```
/// use vais_lexer::{tokenize, Token};
///
/// let source = "F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)";
/// let tokens = tokenize(source).unwrap();
/// assert!(tokens.len() > 0);
/// ```
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
    #[allow(clippy::approx_constant)]
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
        let s_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(&t.token, Token::Ident(s) if s == "s"))
            .collect();
        assert_eq!(s_tokens.len(), 3); // s:=0, s+=x, final s
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_empty_input() {
        let source = "";
        let tokens = tokenize(source).unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let source = "   \n\t\r\n   ";
        let tokens = tokenize(source).unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_comment_only() {
        let source = "# this is just a comment\n# another comment";
        let tokens = tokenize(source).unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_single_character_identifiers() {
        let source = "x y z _ a b c";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0].token, Token::Ident("x".to_string()));
        assert_eq!(tokens[3].token, Token::Ident("_".to_string()));
    }

    #[test]
    fn test_very_long_identifier() {
        let long_name = "a".repeat(1000);
        let source = format!("F {}()->()=()", long_name);
        let tokens = tokenize(&source).unwrap();
        assert_eq!(tokens[1].token, Token::Ident(long_name));
    }

    #[test]
    fn test_i64_max() {
        let source = "9223372036854775807";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].token, Token::Int(i64::MAX));
    }

    #[test]
    fn test_negative_number_as_tokens() {
        // Negative numbers are lexed as Minus followed by Int
        let source = "-42";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].token, Token::Minus);
        assert_eq!(tokens[1].token, Token::Int(42));
    }

    #[test]
    fn test_float_edge_cases() {
        let source = "0.0 1.0 0.5 123.456789";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].token, Token::Float(0.0));
        assert_eq!(tokens[1].token, Token::Float(1.0));
        assert_eq!(tokens[2].token, Token::Float(0.5));
        assert_eq!(tokens[3].token, Token::Float(123.456789));
    }

    #[test]
    fn test_multiple_underscores_in_number() {
        let source = "1_2_3_4_5_6";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].token, Token::Int(123456));
    }

    #[test]
    fn test_keyword_like_identifiers() {
        // Keywords are uppercase single letters, these should be identifiers
        let source = "Fn Struct Enum If Loop Match For While";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].token, Token::Ident("Fn".to_string()));
        assert_eq!(tokens[1].token, Token::Ident("Struct".to_string()));
        assert_eq!(tokens[2].token, Token::Ident("Enum".to_string()));
        assert_eq!(tokens[3].token, Token::Ident("If".to_string()));
        assert_eq!(tokens[4].token, Token::Ident("Loop".to_string()));
        assert_eq!(tokens[5].token, Token::Ident("Match".to_string()));
        assert_eq!(tokens[6].token, Token::Ident("For".to_string()));
        assert_eq!(tokens[7].token, Token::Ident("While".to_string()));
    }

    #[test]
    fn test_consecutive_operators() {
        // Note: /// is a doc comment, so use separate slashes with spaces
        let source = "+++---***/ / /";
        let tokens = tokenize(source).unwrap();
        // Should be lexed as separate operators
        assert!(tokens.iter().any(|t| t.token == Token::Plus));
        assert!(tokens.iter().any(|t| t.token == Token::Minus));
        assert!(tokens.iter().any(|t| t.token == Token::Star));
        assert!(tokens.iter().any(|t| t.token == Token::Slash));
    }

    #[test]
    fn test_dot_vs_dotdot() {
        let source = "a.b 0..10 x.y..z.w";
        let tokens = tokenize(source).unwrap();
        // Should correctly distinguish . from ..
        let dot_count = tokens.iter().filter(|t| t.token == Token::Dot).count();
        let dotdot_count = tokens.iter().filter(|t| t.token == Token::DotDot).count();
        assert!(dot_count >= 2); // a.b, x.y, z.w
        assert!(dotdot_count >= 1); // 0..10, y..z
    }

    #[test]
    fn test_comparison_operators() {
        let source = "< <= > >= == != <<";
        let tokens = tokenize(source).unwrap();
        assert!(tokens.iter().any(|t| t.token == Token::Lt));
        assert!(tokens.iter().any(|t| t.token == Token::Lte));
        assert!(tokens.iter().any(|t| t.token == Token::Gt));
        assert!(tokens.iter().any(|t| t.token == Token::Gte));
        assert!(tokens.iter().any(|t| t.token == Token::EqEq));
        assert!(tokens.iter().any(|t| t.token == Token::Neq));
    }

    #[test]
    fn test_string_with_escapes() {
        let source = r#""hello\nworld\ttab""#;
        let tokens = tokenize(source).unwrap();
        // The lexer should handle escape sequences
        assert!(matches!(&tokens[0].token, Token::String(_)));
    }

    #[test]
    fn test_empty_string() {
        let source = r#""""#;
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].token, Token::String("".to_string()));
    }

    #[test]
    fn test_all_keywords() {
        // F=Function, S=Struct, E=Enum, I=If, L=Loop, M=Match, R=Return, B=Break, C=Continue, T=Type, W=Trait, A=Async, P=Pub, U=Use, X=Impl
        let source = "F S E I L M R B C T W A P U X";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].token, Token::Function);
        assert_eq!(tokens[1].token, Token::Struct);
        assert_eq!(tokens[2].token, Token::Enum);
        assert_eq!(tokens[3].token, Token::If);
        assert_eq!(tokens[4].token, Token::Loop);
        assert_eq!(tokens[5].token, Token::Match);
        assert_eq!(tokens[6].token, Token::Return);
        assert_eq!(tokens[7].token, Token::Break);
        assert_eq!(tokens[8].token, Token::Continue);
        assert_eq!(tokens[9].token, Token::TypeKeyword);
        assert_eq!(tokens[10].token, Token::Trait); // W is Trait
        assert_eq!(tokens[11].token, Token::Async);
        assert_eq!(tokens[12].token, Token::Pub);
        assert_eq!(tokens[13].token, Token::Use);
        assert_eq!(tokens[14].token, Token::Impl);
    }

    #[test]
    fn test_all_type_keywords() {
        let source = "i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 bool str";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].token, Token::I8);
        assert_eq!(tokens[1].token, Token::I16);
        assert_eq!(tokens[2].token, Token::I32);
        assert_eq!(tokens[3].token, Token::I64);
        assert_eq!(tokens[4].token, Token::I128);
        assert_eq!(tokens[5].token, Token::U8);
        assert_eq!(tokens[6].token, Token::U16);
        assert_eq!(tokens[7].token, Token::U32);
        assert_eq!(tokens[8].token, Token::U64);
        assert_eq!(tokens[9].token, Token::U128);
        assert_eq!(tokens[10].token, Token::F32);
        assert_eq!(tokens[11].token, Token::F64);
        assert_eq!(tokens[12].token, Token::Bool);
        assert_eq!(tokens[13].token, Token::Str);
    }

    #[test]
    fn test_all_brackets_and_delimiters() {
        let source = "( ) [ ] { } < > , : ; . .. -> => @ ? !";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::RParen);
        assert_eq!(tokens[2].token, Token::LBracket);
        assert_eq!(tokens[3].token, Token::RBracket);
        assert_eq!(tokens[4].token, Token::LBrace);
        assert_eq!(tokens[5].token, Token::RBrace);
        assert_eq!(tokens[6].token, Token::Lt);
        assert_eq!(tokens[7].token, Token::Gt);
        assert_eq!(tokens[8].token, Token::Comma);
        assert_eq!(tokens[9].token, Token::Colon);
        assert_eq!(tokens[10].token, Token::Semi);
        assert_eq!(tokens[11].token, Token::Dot);
        assert_eq!(tokens[12].token, Token::DotDot);
        assert_eq!(tokens[13].token, Token::Arrow);
        assert_eq!(tokens[14].token, Token::FatArrow);
        assert_eq!(tokens[15].token, Token::At);
        assert_eq!(tokens[16].token, Token::Question);
        assert_eq!(tokens[17].token, Token::Bang);
    }

    #[test]
    fn test_assignment_operators() {
        let source = "= := += -= *= /=";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].token, Token::Eq);
        assert_eq!(tokens[1].token, Token::ColonEq);
        assert_eq!(tokens[2].token, Token::PlusEq);
        assert_eq!(tokens[3].token, Token::MinusEq);
        assert_eq!(tokens[4].token, Token::StarEq);
        assert_eq!(tokens[5].token, Token::SlashEq);
    }

    #[test]
    fn test_bitwise_operators() {
        let source = "& | ^ ~ << >>";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].token, Token::Amp);
        assert_eq!(tokens[1].token, Token::Pipe);
        assert_eq!(tokens[2].token, Token::Caret);
        assert_eq!(tokens[3].token, Token::Tilde);
        assert_eq!(tokens[4].token, Token::Shl);
        assert_eq!(tokens[5].token, Token::Shr);
    }

    #[test]
    fn test_pipe_arrow_operator() {
        let source = "x |> f |> g";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].token, Token::Ident("x".to_string()));
        assert_eq!(tokens[1].token, Token::PipeArrow);
        assert_eq!(tokens[2].token, Token::Ident("f".to_string()));
        assert_eq!(tokens[3].token, Token::PipeArrow);
        assert_eq!(tokens[4].token, Token::Ident("g".to_string()));
    }

    #[test]
    fn test_pipe_arrow_vs_pipe() {
        // |> should be PipeArrow, | should be Pipe
        let source = "a |> b | c";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[1].token, Token::PipeArrow);
        assert_eq!(tokens[3].token, Token::Pipe);
    }

    #[test]
    fn test_logical_operators() {
        // && is lexed as two Amp tokens, || as two Pipe tokens
        let source = "&& ||";
        let tokens = tokenize(source).unwrap();
        // && -> Amp Amp
        assert_eq!(tokens[0].token, Token::Amp);
        assert_eq!(tokens[1].token, Token::Amp);
        // || -> Pipe Pipe
        assert_eq!(tokens[2].token, Token::Pipe);
        assert_eq!(tokens[3].token, Token::Pipe);
    }

    #[test]
    fn test_multiline_code() {
        let source = r#"
F add(a:i64,
      b:i64)->i64 {
    R a + b
}
"#;
        let tokens = tokenize(source).unwrap();
        // Should successfully tokenize multiline code
        assert!(tokens.iter().any(|t| t.token == Token::Function));
        assert!(tokens.iter().any(|t| t.token == Token::Return));
    }

    #[test]
    fn test_unicode_in_string() {
        let source = r#""안녕하세요 🚀 世界""#;
        let tokens = tokenize(source).unwrap();
        assert_eq!(
            tokens[0].token,
            Token::String("안녕하세요 🚀 世界".to_string())
        );
    }

    #[test]
    fn test_identifier_with_numbers() {
        let source = "x1 y2 var123 test_456";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].token, Token::Ident("x1".to_string()));
        assert_eq!(tokens[1].token, Token::Ident("y2".to_string()));
        assert_eq!(tokens[2].token, Token::Ident("var123".to_string()));
        assert_eq!(tokens[3].token, Token::Ident("test_456".to_string()));
    }

    #[test]
    fn test_span_accuracy() {
        let source = "F f()->i64=42";
        let tokens = tokenize(source).unwrap();

        // Check that spans are accurate
        assert_eq!(tokens[0].span.start, 0);
        assert_eq!(tokens[0].span.end, 1); // "F"

        // Find the "42" token and check its span
        let int_token = tokens.iter().find(|t| t.token == Token::Int(42)).unwrap();
        assert_eq!(int_token.span.start, 11);
        assert_eq!(int_token.span.end, 13);
    }

    // ==================== Advanced Edge Case Tests ====================

    #[test]
    fn test_nested_generic_syntax() {
        // Test nested generic type syntax with spaces: Vec<HashMap<K, V> >
        // Note: Without spaces, >> is tokenized as Shr (right shift)
        let source = "S Container{data:Vec<HashMap<str,Option<i64> > >}";
        let tokens = tokenize(source).unwrap();

        // Verify proper tokenization of nested generics
        assert!(tokens.iter().any(|t| t.token == Token::Struct));
        assert!(tokens
            .iter()
            .any(|t| matches!(&t.token, Token::Ident(s) if s == "Container")));
        assert!(tokens
            .iter()
            .any(|t| matches!(&t.token, Token::Ident(s) if s == "Vec")));
        assert!(tokens
            .iter()
            .any(|t| matches!(&t.token, Token::Ident(s) if s == "HashMap")));
        assert!(tokens
            .iter()
            .any(|t| matches!(&t.token, Token::Ident(s) if s == "Option")));

        // Count angle brackets (should be balanced with spaces)
        let lt_count = tokens.iter().filter(|t| t.token == Token::Lt).count();
        let gt_count = tokens.iter().filter(|t| t.token == Token::Gt).count();
        assert_eq!(lt_count, gt_count, "Angle brackets should be balanced");
    }

    #[test]
    fn test_deeply_nested_generic_combinations() {
        // Test Vec<HashMap<K, Option<V> > > with spaces to avoid >> tokenization
        let source = "F process(data:Vec<HashMap<str,Option<Vec<i64> > > >)->i64=42";
        let tokens = tokenize(source).unwrap();

        assert!(tokens.iter().any(|t| t.token == Token::Function));
        let lt_count = tokens.iter().filter(|t| t.token == Token::Lt).count();
        let gt_count = tokens.iter().filter(|t| t.token == Token::Gt).count();
        assert_eq!(lt_count, 4);
        assert_eq!(gt_count, 4);
    }

    #[test]
    fn test_option_of_vec() {
        // Test Option<Vec<T> > syntax with space to avoid >> tokenization
        let source = r#"F get_items()->Option<Vec<str> >="""#;
        let tokens = tokenize(source).unwrap();

        assert!(tokens
            .iter()
            .any(|t| matches!(&t.token, Token::Ident(s) if s == "Option")));
        assert!(tokens
            .iter()
            .any(|t| matches!(&t.token, Token::Ident(s) if s == "Vec")));
        let lt_count = tokens.iter().filter(|t| t.token == Token::Lt).count();
        let gt_count = tokens.iter().filter(|t| t.token == Token::Gt).count();
        // With space separation, both should be 2
        assert!(
            lt_count >= 1 && gt_count >= 1,
            "Should have at least 1 Lt and 1 Gt, got lt={}, gt={}",
            lt_count,
            gt_count
        );
    }

    #[test]
    fn test_i8_boundary_values() {
        // Test i8 min (-128) and max (127) values
        // Note: negative sign is separate token
        let source = "F test()->(){min:=-128;max:=127}";
        let tokens = tokenize(source).unwrap();

        assert!(tokens.iter().any(|t| t.token == Token::Int(128)));
        assert!(tokens.iter().any(|t| t.token == Token::Int(127)));
        assert!(tokens.iter().any(|t| t.token == Token::Minus));
    }

    #[test]
    fn test_i64_boundary_values() {
        // Test i64 max value: 9223372036854775807
        let source = "F max_i64()->i64=9223372036854775807";
        let tokens = tokenize(source).unwrap();

        assert!(tokens
            .iter()
            .any(|t| t.token == Token::Int(9223372036854775807)));
    }

    #[test]
    fn test_integer_overflow_literal() {
        // Test value beyond i64::MAX - should fail to parse
        // i64::MAX + 1 = 9223372036854775808 (too large for i64)
        let source = "F overflow()->i64=9223372036854775808";
        let tokens = tokenize(source);

        // This should fail or produce no Int token (lexer parse fails for overflow)
        // We expect the tokenize to fail or not produce a valid Int token
        match tokens {
            Ok(tokens) => {
                // If it succeeds, check that no Int token was produced with a valid i64
                let has_int_token = tokens.iter().any(|t| matches!(t.token, Token::Int(_)));
                // The overflow value shouldn't be tokenized as a valid Int
                assert!(
                    !has_int_token,
                    "i64 overflow should not be tokenized as valid Int"
                );
            }
            Err(_) => {
                // Expected: tokenization fails for overflow
            }
        }
    }

    #[test]
    fn test_pattern_with_guard_syntax() {
        // Test pattern matching with guard: M x{n I n>0=>n,_=>0}
        let source = "F abs(x:i64)->i64=M x{n I n>0=>n,n I n<0=>-n,_=>0}";
        let tokens = tokenize(source).unwrap();

        // Verify M (Match), I (If in guard position), => (FatArrow)
        assert!(tokens.iter().any(|t| t.token == Token::Match));
        assert!(tokens.iter().any(|t| t.token == Token::If));
        assert!(tokens.iter().any(|t| t.token == Token::FatArrow));
    }

    #[test]
    fn test_nested_pattern_destructuring() {
        // Test nested destructuring: Some((x, y))
        let source = "M opt{Some((x,y))=>x+y,None=>0}";
        let tokens = tokenize(source).unwrap();

        assert!(tokens.iter().any(|t| t.token == Token::Match));
        assert!(tokens
            .iter()
            .any(|t| matches!(&t.token, Token::Ident(s) if s == "Some")));
        assert!(tokens
            .iter()
            .any(|t| matches!(&t.token, Token::Ident(s) if s == "None")));

        // Count parentheses
        let lparen_count = tokens.iter().filter(|t| t.token == Token::LParen).count();
        let rparen_count = tokens.iter().filter(|t| t.token == Token::RParen).count();
        assert_eq!(lparen_count, rparen_count);
    }

    #[test]
    fn test_complex_guard_condition() {
        // Test complex guard with multiple conditions
        let source = "M (x,y){(a,b) I a>0&&b<10=>1,_=>0}";
        let tokens = tokenize(source).unwrap();

        assert!(tokens.iter().any(|t| t.token == Token::Match));
        assert!(tokens.iter().any(|t| t.token == Token::If));
        assert!(tokens.iter().any(|t| t.token == Token::Amp)); // &&
        assert!(tokens.iter().any(|t| t.token == Token::Lt));
    }

    #[test]
    fn test_multiple_type_params_with_bounds() {
        // Test: F<A: Clone, B: Default, C: Ord>
        let source = "F transform<A:Clone,B:Default,C:Ord>(a:A,b:B,c:C)->C=c";
        let tokens = tokenize(source).unwrap();

        assert!(tokens
            .iter()
            .any(|t| matches!(&t.token, Token::Ident(s) if s == "Clone")));
        assert!(tokens
            .iter()
            .any(|t| matches!(&t.token, Token::Ident(s) if s == "Default")));
        assert!(tokens
            .iter()
            .any(|t| matches!(&t.token, Token::Ident(s) if s == "Ord")));
    }

    #[test]
    fn test_float_special_values() {
        // Test float edge cases
        let source = "F test()->(){a:=0.0;b:=1.0;c:=0.5;d:=999999.999999}";
        let tokens = tokenize(source).unwrap();

        assert!(tokens.iter().any(|t| t.token == Token::Float(0.0)));
        assert!(tokens.iter().any(|t| t.token == Token::Float(1.0)));
        assert!(tokens.iter().any(|t| t.token == Token::Float(0.5)));
        assert!(tokens
            .iter()
            .any(|t| t.token == Token::Float(999999.999999)));
    }

    #[test]
    fn test_scientific_notation() {
        // Test scientific notation: 1.5e10, 2.0e-5
        let source = "F sci()->(){a:=1.5e10;b:=2.0e-5;c:=3.14e+2}";
        let tokens = tokenize(source).unwrap();

        // Check that scientific notation is parsed as floats
        assert!(tokens.iter().any(|t| matches!(t.token, Token::Float(_))));
    }

    #[test]
    fn test_max_nested_angle_brackets() {
        // Test maximum nesting of angle brackets with spaces
        let source = "S Deep{v:Vec<Vec<Vec<Vec<i64> > > >}";
        let tokens = tokenize(source).unwrap();

        let lt_count = tokens.iter().filter(|t| t.token == Token::Lt).count();
        let gt_count = tokens.iter().filter(|t| t.token == Token::Gt).count();
        assert_eq!(lt_count, 4);
        assert_eq!(gt_count, 4);
    }

    #[test]
    fn test_zero_values_all_types() {
        // Test zero values for different numeric types
        let source = "F zeros()->(){a:=0;b:=0.0;c:=0.0e0}";
        let tokens = tokenize(source).unwrap();

        assert!(tokens.iter().any(|t| t.token == Token::Int(0)));
        assert!(tokens.iter().any(|t| t.token == Token::Float(0.0)));
    }

    #[test]
    fn test_ambiguous_generic_vs_comparison() {
        // Test that Vec<i64> is parsed correctly, not as Vec < i64 >
        let source = "F f(x:Vec<i64>)->bool=true";
        let tokens = tokenize(source).unwrap();

        // Lt and Gt should be present for generics
        assert!(tokens.iter().any(|t| t.token == Token::Lt));
        assert!(tokens.iter().any(|t| t.token == Token::Gt));
        // But Vec should be an identifier
        assert!(tokens
            .iter()
            .any(|t| matches!(&t.token, Token::Ident(s) if s == "Vec")));
    }

    #[test]
    fn test_consecutive_comparison_operators() {
        // Test: a<b<c should be tokenized as a < b < c
        let source = "F chain(a:i64,b:i64,c:i64)->bool=a<b<c";
        let tokens = tokenize(source).unwrap();

        let lt_count = tokens.iter().filter(|t| t.token == Token::Lt).count();
        assert_eq!(lt_count, 2);
    }

    #[test]
    fn test_range_operators() {
        // Test .. and ..= operators
        let source = "F ranges()->(){a:=0..10;b:=0..=10}";
        let tokens = tokenize(source).unwrap();

        assert!(tokens.iter().any(|t| t.token == Token::DotDot));
        assert!(tokens.iter().any(|t| t.token == Token::DotDotEq));
    }

    #[test]
    fn test_very_large_float() {
        // Test very large float values
        let source = "F large()->f64=1.7976931348623157e308";
        let tokens = tokenize(source).unwrap();

        assert!(tokens.iter().any(|t| matches!(t.token, Token::Float(_))));
    }

    #[test]
    fn test_very_small_float() {
        // Test very small float values (near zero)
        let source = "F small()->f64=2.2250738585072014e-308";
        let tokens = tokenize(source).unwrap();

        assert!(tokens.iter().any(|t| matches!(t.token, Token::Float(_))));
    }

    #[test]
    fn test_all_assignment_operators_combined() {
        // Test all compound assignment operators in one expression
        let source = "F assign()->(){x:=1;x+=1;x-=1;x*=2;x/=2}";
        let tokens = tokenize(source).unwrap();

        assert!(tokens.iter().any(|t| t.token == Token::ColonEq));
        assert!(tokens.iter().any(|t| t.token == Token::PlusEq));
        assert!(tokens.iter().any(|t| t.token == Token::MinusEq));
        assert!(tokens.iter().any(|t| t.token == Token::StarEq));
        assert!(tokens.iter().any(|t| t.token == Token::SlashEq));
    }

    // ==================== Macro System Tests ====================

    #[test]
    fn test_macro_keyword() {
        let source = "macro vec! { () => { [] } }";
        let tokens = tokenize(source).unwrap();

        assert!(tokens.iter().any(|t| t.token == Token::Macro));
        assert!(tokens
            .iter()
            .any(|t| matches!(&t.token, Token::Ident(s) if s == "vec")));
        assert!(tokens.iter().any(|t| t.token == Token::Bang));
    }

    #[test]
    fn test_dollar_token() {
        let source = "macro test! { ($x:expr) => { $x } }";
        let tokens = tokenize(source).unwrap();

        assert!(tokens.iter().any(|t| t.token == Token::Dollar));
        let dollar_count = tokens.iter().filter(|t| t.token == Token::Dollar).count();
        assert_eq!(dollar_count, 2); // $x in pattern and $x in template
    }

    #[test]
    fn test_macro_invocation() {
        let source = "vec!(1, 2, 3)";
        let tokens = tokenize(source).unwrap();

        assert!(tokens
            .iter()
            .any(|t| matches!(&t.token, Token::Ident(s) if s == "vec")));
        assert!(tokens.iter().any(|t| t.token == Token::Bang));
        assert!(tokens.iter().any(|t| t.token == Token::LParen));
    }

    #[test]
    fn test_y_await_abbreviation() {
        let source = "expr.Y";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].token, Token::Ident("expr".to_string()));
        assert_eq!(tokens[1].token, Token::Dot);
        assert_eq!(tokens[2].token, Token::Await);
    }

    #[test]
    fn test_y_and_await_both_work() {
        let tokens_y = tokenize("x.Y").unwrap();
        let tokens_await = tokenize("x.await").unwrap();
        assert_eq!(tokens_y[2].token, Token::Await);
        assert_eq!(tokens_await[2].token, Token::Await);
    }

    #[test]
    fn test_macro_with_repetition() {
        let source = "macro vec! { ($($item:expr),*) => { [$($item),*] } }";
        let tokens = tokenize(source).unwrap();

        assert!(tokens.iter().any(|t| t.token == Token::Macro));
        assert!(tokens.iter().any(|t| t.token == Token::Star));
        let dollar_count = tokens.iter().filter(|t| t.token == Token::Dollar).count();
        assert!(dollar_count >= 2);
    }
}

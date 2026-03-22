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
    #[regex(r"0[xX][0-9a-fA-F][0-9a-fA-F_]*", |lex| {
        let s = lex.slice().replace('_', "");
        let hex_str = &s[2..]; // skip "0x"
        i64::from_str_radix(hex_str, 16).ok()
    })]
    #[regex(r"0[bB][01][01_]*", |lex| {
        let s = lex.slice().replace('_', "");
        let bin_str = &s[2..]; // skip "0b"
        i64::from_str_radix(bin_str, 2).ok()
    })]
    #[regex(r"0[oO][0-7][0-7_]*", |lex| {
        let s = lex.slice().replace('_', "");
        let oct_str = &s[2..]; // skip "0o"
        i64::from_str_radix(oct_str, 8).ok()
    })]
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
                        'u' => {
                            // Unicode escape: \u{XXXX} (1-6 hex digits)
                            if chars.peek() == Some(&'{') {
                                chars.next(); // consume '{'
                                let mut hex = std::string::String::new();
                                while let Some(&h) = chars.peek() {
                                    if h == '}' {
                                        chars.next(); // consume '}'
                                        break;
                                    }
                                    if h.is_ascii_hexdigit() && hex.len() < 6 {
                                        hex.push(h);
                                        chars.next();
                                    } else {
                                        break;
                                    }
                                }
                                if let Ok(code) = u32::from_str_radix(&hex, 16) {
                                    if let Some(ch) = char::from_u32(code) {
                                        result.push(ch);
                                    }
                                }
                            } else {
                                // Not followed by '{', keep as-is
                                result.push('\\');
                                result.push('u');
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
    // Pre-allocate based on heuristic: ~1 token per 4 bytes of source
    let mut tokens = Vec::with_capacity(source.len() / 4 + 16);
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

    // Post-process: split identifiers that start with single-char keywords.
    // logos longest-match makes "EI" → Ident("EI") instead of Enum + If.
    // Split these into keyword + remaining identifier.
    let tokens = split_keyword_idents(tokens);

    Ok(tokens)
}

/// Split identifiers that consist entirely of single-char keyword letters.
/// logos longest-match can merge adjacent keywords into one Ident token
/// (e.g. "EI" → Ident("EI") instead of Enum + If).
/// Uses char_to_keyword to detect all known single-char keywords.
fn split_keyword_idents(tokens: Vec<SpannedToken>) -> Vec<SpannedToken> {
    let mut result = Vec::with_capacity(tokens.len());
    for tok in tokens {
        if let Token::Ident(ref s) = tok.token {
            // Split only if the ident has 2+ chars and every char is a keyword letter
            let all_keywords = s.len() >= 2
                && s.chars().all(|c| !matches!(char_to_keyword(c), Token::Ident(_)));
            if all_keywords {
                let start = tok.span.start;
                for (i, c) in s.chars().enumerate() {
                    result.push(SpannedToken {
                        token: char_to_keyword(c),
                        span: start + i..start + i + 1,
                    });
                }
                continue;
            }
        }
        result.push(tok);
    }
    result
}

fn char_to_keyword(c: char) -> Token {
    match c {
        'F' => Token::Function,
        'S' => Token::Struct,
        'E' => Token::Enum,
        'I' => Token::If,
        'L' => Token::Loop,
        'M' => Token::Match,
        'A' => Token::Async,
        'R' => Token::Return,
        'B' => Token::Break,
        'C' => Token::Continue,
        'T' => Token::TypeKeyword,
        'U' => Token::Use,
        'P' => Token::Pub,
        'W' => Token::Trait,
        'X' => Token::Impl,
        'D' => Token::Defer,
        'O' => Token::Union,
        'N' => Token::Extern,
        'G' => Token::Global,
        _ => Token::Ident(c.to_string()),
    }
}

#[cfg(test)]
mod tests;

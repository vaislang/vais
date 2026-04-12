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

    // === VaisX Template Comments (HTML comments in .vaisx files) ===
    // Allows UTF-8 (Korean, emoji, CJK) inside HTML comments.
    // Added 2026-04-11 for monitor/web layout.vaisx Korean comment support.
    // Uses a callback to handle the tricky `-->` termination (dash-run ambiguity
    // that pure logos regex can't express cleanly — logos is greedy+longest-match
    // and tends to mis-parse `-->` endings in `<!-- ... -->` when the body contains
    // dashes or when multiple alternatives exist).
    #[token("<!--", skip_html_comment)]
    // === VaisX Template Doctype (HTML5 doctype declaration) ===
    #[regex(r"<!DOCTYPE[^>]*>", logos::skip, ignore(ascii_case))]

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

    // === Unambiguous 2-char Keywords (priority 4 > single-letter priority 3) ===
    #[token("EN", priority = 4)]
    EnumKeyword, // Unambiguous enum (replaces contextual E)
    #[token("EL", priority = 4)]
    Else, // Unambiguous else (replaces contextual E after if)
    #[token("LF", priority = 4)]
    ForEach, // Unambiguous for-each loop (replaces contextual L pattern:iter)
    #[token("LW", priority = 4)]
    While, // Unambiguous while loop (replaces contextual L condition)

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
    #[token("await")]
    #[token("Y", priority = 3)]
    Await,
    #[token("yield")]
    Yield,
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
    // `pure F foo()`, `io F foo()` — Phase 4c.3 (Task #54) prefix
    // modifiers. `pure` and `io` are full reserved keywords. `alloc`
    // is a **contextual** keyword: the lexer still emits it as
    // `Token::Ident("alloc")`, and the parser reinterprets it as an
    // effect prefix only when it occurs in a function-modifier
    // position (before `F` / `A F`, after `P` / `partial`). This
    // preserves backwards compatibility with the existing `std/
    // allocator.vais` and `std/arena.vais` modules, which use `alloc`
    // as a method / variable name in ~40 places.
    #[token("pure")]
    Pure,
    #[token("effect")]
    Effect,
    #[token("io")]
    Io,
    #[token("unsafe")]
    Unsafe,

    // === Totality Modifier (Phase 4c.2 / Task #53) ===
    // `partial F foo() { ... }` marks a function that may panic
    // (div-by-zero, array OOB, None unwrap, Err unwrap).
    // Functions without `partial` must be proved panic-free by the type
    // checker via the existing EffectInferrer reached through the
    // partial-function gate in `check_function`.
    #[token("partial")]
    Partial,

    // === Linear Types Keywords ===
    #[token("linear")]
    Linear,
    #[token("affine")]
    Affine,
    #[token("move")]
    Move,
    #[token("where")]
    Where,

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
        // Parse as u64 first to support full u64 range (e.g. 0xcbf29ce484222325),
        // then reinterpret bits as i64 (wrapping cast, same as Rust's `as i64`).
        u64::from_str_radix(hex_str, 16).ok().map(|v| v as i64)
    })]
    #[regex(r"0[bB][01][01_]*", |lex| {
        let s = lex.slice().replace('_', "");
        let bin_str = &s[2..]; // skip "0b"
        // Parse as u64 first to support full u64 range, then reinterpret as i64.
        u64::from_str_radix(bin_str, 2).ok().map(|v| v as i64)
    })]
    #[regex(r"0[oO][0-7][0-7_]*", |lex| {
        let s = lex.slice().replace('_', "");
        let oct_str = &s[2..]; // skip "0o"
        // Parse as u64 first to support full u64 range, then reinterpret as i64.
        u64::from_str_radix(oct_str, 8).ok().map(|v| v as i64)
    })]
    #[regex(r"[0-9][0-9_]*", |lex| lex.slice().replace('_', "").parse::<i64>().ok())]
    Int(i64),

    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*([eE][+-]?[0-9]+)?", |lex| lex.slice().replace('_', "").parse::<f64>().ok())]
    Float(f64),

    #[regex(r#""([^"\\]|\\.)*""#, parse_string_literal)]
    String(String),

    // === Identifiers ===
    // Priority lower than single-letter keywords
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string(), priority = 2)]
    Ident(String),

    // === Lifetime identifiers ===
    //
    // Lifetime names start with `'` followed by identifier chars (e.g., `'a`, `'static`).
    //
    // Disambiguation from single-quote strings (2026-04-11):
    // Raw regex alone cannot choose between `'abc` (lifetime) and `'abc...'` (string)
    // because logos has no lookahead. We match lifetime greedily here and then
    // **post-process in `tokenize()`** — if the source byte right after the matched
    // lifetime is `'`, we retroactively convert the Lifetime token into a String.
    //
    // This means `'hello'` first tokenizes as Lifetime("hello") then gets promoted
    // to String("hello") in the post-process pass. `'a` followed by `>` or whitespace
    // stays a Lifetime.
    //
    // Empty string `''` and strings starting with a non-identifier char (`'안녕'`,
    // `'123'`) are handled by a separate `'` token + callback below.
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
            Token::EnumKeyword => write!(f, "EN"),
            Token::Else => write!(f, "EL"),
            Token::ForEach => write!(f, "LF"),
            Token::While => write!(f, "LW"),
            Token::Yield => write!(f, "yield"),
            Token::Mut => write!(f, "mut"),
            Token::SelfLower => write!(f, "self"),
            Token::SelfUpper => write!(f, "Self"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Await => write!(f, "Y"),
            Token::Const => write!(f, "const"),
            Token::Comptime => write!(f, "comptime"),
            Token::Dyn => write!(f, "dyn"),
            Token::Macro => write!(f, "macro"),
            Token::As => write!(f, "as"),
            Token::Pure => write!(f, "pure"),
            Token::Effect => write!(f, "effect"),
            Token::Partial => write!(f, "partial"),
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
            Token::Where => write!(f, "where"),
            // Lifetime
            Token::Lifetime(name) => write!(f, "'{}", name),
        }
    }
}

/// Parse a string literal (both `"..."` and `'...'` forms) into its processed value.
///
/// Strips the outer quotes and handles escape sequences:
/// `\n`, `\t`, `\r`, `\\`, `\"`, `\'`, `\0`, `\xHH` (hex), `\u{XXXX}` (unicode),
/// and VaisX brace escapes `\{` / `\}` (preserved as `\{` / `\}` for the parser
/// to distinguish from string interpolation).
///
/// Shared between `"..."` and `'...'` regexes so both quote forms behave identically
/// (added 2026-04-11 for VaisX template JS-style single-quote strings).
fn parse_string_literal(lex: &mut logos::Lexer<Token>) -> Option<std::string::String> {
    let s = lex.slice();
    let inner = &s[1..s.len() - 1]; // strip outer quote (1 byte each, ASCII)
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
                    '\'' => result.push('\''),
                    '0' => result.push('\0'),
                    // Brace escapes: \{ and \} produce literal { and }.
                    // Kept as \{ / \} in the token so the parser can
                    // distinguish them from string interpolation {expr}.
                    '{' => {
                        result.push('\\');
                        result.push('{');
                    }
                    '}' => {
                        result.push('\\');
                        result.push('}');
                    }
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
}

/// Skip callback for VaisX HTML comments `<!-- ... -->`.
///
/// Invoked after `<!--` is matched. Advances the lexer's `bump` position
/// past the closing `-->`, skipping the whole comment (including UTF-8 content).
/// If `-->` is not found, bumps to end-of-input (permissive — parser will
/// report the missing terminator if needed).
///
/// Returns `Skip` so logos treats this as a skipped token, not a real one.
fn skip_html_comment(lex: &mut logos::Lexer<Token>) -> logos::Skip {
    let remainder = lex.remainder();
    if let Some(end) = remainder.find("-->") {
        lex.bump(end + 3); // consume up through "-->"
    } else {
        lex.bump(remainder.len());
    }
    logos::Skip
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
                // === Single-quote string fallback (2026-04-11) ===
                //
                // logos can't disambiguate `'abc'` (string) from `'abc` (lifetime)
                // with pure regex alternation. When we hit an InvalidToken at a `'`
                // position, we have one of three situations:
                //
                //   (a) `'ident'` — Lifetime("ident") was tokenized for `'ident`,
                //       and we're now at the closing `'`. Rewrite the previous
                //       Lifetime token to a String and drop the closing `'`.
                //
                //   (b) `'a.b.c'` or `'foo bar'` — Lifetime("a") was tokenized,
                //       then Dot/Ident/etc, then we hit the closing `'`. Walk
                //       backwards through tokens until we find a Lifetime whose
                //       start is preceded by no other `'` on the same line, then
                //       collapse everything from that Lifetime through the closing
                //       `'` into a single String token using the raw source slice.
                //
                //   (c) Standalone `'` opening (e.g., `'안녕'`, `''`, `'123'`):
                //       The Lifetime regex never matched, so logos errored on the
                //       opening `'`. Scan forward for a closing `'` and emit String.
                //
                //   (d) None of the above → propagate the LexError.
                let err_pos = lexer.span().start;
                let err_byte = source.as_bytes().get(err_pos).copied();
                // Either err_pos itself is a `'` (closing or opening), OR a previous
                // token is a Lifetime that started with `'` (meaning we are inside an
                // unfinished single-quote string and hit an unexpected char like `\`).
                let inside_single_quote = matches!(err_byte, Some(b'\'')) || {
                    tokens.last().is_some_and(|last| {
                        matches!(last.token, Token::Lifetime(_))
                            && source.as_bytes().get(last.span.start) == Some(&b'\'')
                    })
                };
                if inside_single_quote {
                    // Walk backwards to find the opening Lifetime (whose start byte is `'`).
                    let mut start_idx: Option<usize> = None;
                    for (i, t) in tokens.iter().enumerate().rev() {
                        if let Token::Lifetime(_) = t.token {
                            if source.as_bytes().get(t.span.start) == Some(&b'\'') {
                                start_idx = Some(i);
                                break;
                            }
                        }
                    }
                    let recovery: Option<(usize, usize, std::string::String)> =
                        if let Some(i) = start_idx {
                            let opener = tokens[i].span.start;
                            scan_single_quote_string(source, opener)
                                .map(|(v, end)| (i, end, v))
                                .filter(|(_, end, _)| *end > err_pos)
                        } else if err_byte == Some(b'\'') {
                            scan_single_quote_string(source, err_pos)
                                .map(|(v, end)| (tokens.len(), end, v))
                                .filter(|(_, end, _)| *end > err_pos + 1)
                        } else {
                            None
                        };
                    if let Some((truncate_idx, end_pos, string_value)) = recovery {
                        let opener = if truncate_idx < tokens.len() {
                            tokens[truncate_idx].span.start
                        } else {
                            err_pos
                        };
                        tokens.truncate(truncate_idx);
                        tokens.push(SpannedToken {
                            token: Token::String(string_value),
                            span: opener..end_pos,
                        });
                        // Restart the logos lexer from `end_pos`. We need a *new*
                        // Lexer because the current one has its internal state
                        // tied to the original source span, and `bump` has tricky
                        // semantics on the error path. To preserve absolute span
                        // offsets across the restart, we lex the suffix and add
                        // `end_pos` to every span before pushing.
                        //
                        // Loop until the suffix is fully consumed. If the suffix
                        // itself contains another single-quote string, we recurse
                        // through the same recovery logic by re-entering this
                        // outer loop. To avoid borrow conflicts we extract the
                        // remaining tokens recursively via a helper call.
                        if !source.is_char_boundary(end_pos) {
                            return Err(LexError::InvalidToken(err_pos));
                        }
                        let suffix_tokens = tokenize_suffix(source, end_pos)?;
                        tokens.extend(suffix_tokens);
                        // Suffix fully consumed — exit the outer loop entirely.
                        break;
                    }
                }
                return Err(LexError::InvalidToken(err_pos));
            }
        }
    }

    Ok(post_process_tokens(tokens, source))
}

/// Tokenize a suffix of `source` starting at byte offset `offset`, returning
/// tokens with absolute spans (i.e., spans relative to the original `source`).
///
/// Used by the InvalidToken recovery path to continue lexing after a manually-
/// recovered single-quote string. Recursively applies the same recovery logic,
/// so chained single-quote strings (e.g., a `t('a')` call followed later by
/// an `import './x'` statement) all get handled.
fn tokenize_suffix(
    source: &str,
    offset: usize,
) -> Result<Vec<SpannedToken>, LexError> {
    if offset >= source.len() {
        return Ok(Vec::new());
    }
    if !source.is_char_boundary(offset) {
        return Err(LexError::InvalidToken(offset));
    }
    let suffix = &source[offset..];
    // Tokenize the suffix as a standalone source. This recursively re-uses
    // the InvalidToken recovery path because `tokenize` is what we're inside.
    let sub_tokens = tokenize(suffix)?;
    // Rewrite spans to be absolute.
    let mut result = Vec::with_capacity(sub_tokens.len());
    for tok in sub_tokens {
        result.push(SpannedToken {
            token: tok.token,
            span: (tok.span.start + offset)..(tok.span.end + offset),
        });
    }
    Ok(result)
}

/// Final post-processing applied after the main lexing loop completes.
/// Currently runs (1) Lifetime → String coalescing for `'ident'` form and
/// (2) keyword identifier splitting (`EI` → `Enum + If`).
fn post_process_tokens(tokens: Vec<SpannedToken>, source: &str) -> Vec<SpannedToken> {
    let tokens = coalesce_single_quote_strings(tokens, source);
    split_keyword_idents(tokens)
}

/// Scan for a single-quote string starting at `start` (must point to `'`).
/// Returns `(parsed_value, end_pos_after_closing_quote)` on success.
/// Handles `\'`, `\\`, `\n`, `\t`, `\r`, `\0`, `\xHH`, `\u{X..}`, `\{`, `\}`.
/// Aborts on raw newline (single-quote strings are single-line by design).
///
/// Iterates by Rust `char` (UTF-8-aware) so that multi-byte characters never
/// land mid-codepoint. Backslash escapes that take an ASCII escape char advance
/// by exactly 2 bytes (both ASCII); for non-ASCII chars after `\` we keep the
/// backslash and the char literally and advance by `1 + char.len_utf8()`.
fn scan_single_quote_string(source: &str, start: usize) -> Option<(std::string::String, usize)> {
    // Defensive: callers from the InvalidToken recovery path may pass an opener
    // that is not actually `'` if the token vector got reordered. Bail out
    // cleanly instead of panicking.
    if source.as_bytes().get(start) != Some(&b'\'') {
        return None;
    }
    let mut i = start + 1;
    let mut result = std::string::String::new();
    while i < source.len() {
        // SAFETY: i is always at a char boundary because we only ever advance
        // by char.len_utf8() or by ASCII byte counts (1 or 2 ASCII bytes).
        let rest = &source[i..];
        let c = rest.chars().next()?;
        if c == '\n' {
            return None; // unterminated
        }
        if c == '\'' {
            return Some((result, i + 1));
        }
        if c == '\\' {
            // Look at the byte/char immediately after the backslash.
            let after_backslash = i + 1;
            let next_char = source[after_backslash..].chars().next();
            match next_char {
                Some('n') => { result.push('\n'); i = after_backslash + 1; }
                Some('t') => { result.push('\t'); i = after_backslash + 1; }
                Some('r') => { result.push('\r'); i = after_backslash + 1; }
                Some('\\') => { result.push('\\'); i = after_backslash + 1; }
                Some('\'') => { result.push('\''); i = after_backslash + 1; }
                Some('"') => { result.push('"'); i = after_backslash + 1; }
                Some('0') => { result.push('\0'); i = after_backslash + 1; }
                Some('{') => { result.push('\\'); result.push('{'); i = after_backslash + 1; }
                Some('}') => { result.push('\\'); result.push('}'); i = after_backslash + 1; }
                Some(other) => {
                    // Unknown escape, keep both `\` and the char literally.
                    // `other` may be multi-byte UTF-8 — advance by its char length.
                    result.push('\\');
                    result.push(other);
                    i = after_backslash + other.len_utf8();
                }
                None => {
                    // Trailing backslash at EOF — unterminated string.
                    return None;
                }
            }
            continue;
        }
        // Regular char.
        result.push(c);
        i += c.len_utf8();
    }
    None // EOF without closing quote
}

/// Coalesce `Lifetime("name")` tokens that are actually single-quote strings.
///
/// If a Lifetime token is immediately followed in the source by `'` (the closing
/// quote of `'name'`), rewrite it as a String and remove the trailing `'` (which
/// would otherwise have been an unbalanced apostrophe that the InvalidToken handler
/// might also have caught, depending on what comes after).
fn coalesce_single_quote_strings(
    tokens: Vec<SpannedToken>,
    source: &str,
) -> Vec<SpannedToken> {
    let bytes = source.as_bytes();
    let mut result = Vec::with_capacity(tokens.len());
    for tok in tokens {
        if let Token::Lifetime(ref name) = tok.token {
            let after = tok.span.end;
            if bytes.get(after) == Some(&b'\'') {
                // `'name'` form: rewrite as String, extend span to include closing `'`.
                result.push(SpannedToken {
                    token: Token::String(name.clone()),
                    span: tok.span.start..after + 1,
                });
                continue;
            }
        }
        result.push(tok);
    }
    result
}

/// Split specific two-char identifiers that are actually keyword pairs.
/// logos longest-match can merge adjacent keywords into one Ident token
/// (e.g. "EI" → Ident("EI") instead of Enum + If).
/// Only splits exactly 2-char sequences where both chars are keyword letters.
/// Longer identifiers like "BASE", "MAX", "EOF" are NOT split (they are
/// legitimate constant/variable names that happen to consist of keyword chars).
fn split_keyword_idents(tokens: Vec<SpannedToken>) -> Vec<SpannedToken> {
    let mut result = Vec::with_capacity(tokens.len());
    for tok in tokens {
        if let Token::Ident(ref s) = tok.token {
            // Only split exactly 2-char idents where both chars are keyword letters
            if s.len() == 2 {
                // Check if this 2-char sequence is one of the unambiguous 2-char keywords.
                // These must NOT be split into two single-char keyword tokens.
                let two_char_keyword = match s.as_str() {
                    "EN" => Some(Token::EnumKeyword),
                    "EL" => Some(Token::Else),
                    "LF" => Some(Token::ForEach),
                    "LW" => Some(Token::While),
                    _ => None,
                };
                if let Some(kw_token) = two_char_keyword {
                    result.push(SpannedToken {
                        token: kw_token,
                        span: tok.span,
                    });
                    continue;
                }

                let chars: Vec<char> = s.chars().collect();
                let first = char_to_keyword(chars[0]);
                let second = char_to_keyword(chars[1]);
                if !matches!(first, Token::Ident(_)) && !matches!(second, Token::Ident(_)) {
                    let start = tok.span.start;
                    result.push(SpannedToken {
                        token: first,
                        span: start..start + 1,
                    });
                    result.push(SpannedToken {
                        token: second,
                        span: start + 1..start + 2,
                    });
                    continue;
                }
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

//! Token Display coverage tests
//!
//! Targets the Token::Display implementation (~140 uncovered lines)
//! Each token variant's to_string() / Display trait

use vais_lexer::{tokenize, Token};

/// Helper to verify a token variant's Display output
fn assert_token_display(token: &Token, expected: &str) {
    assert_eq!(format!("{}", token), expected, "Token display mismatch");
}

#[test]
fn test_keyword_display() {
    assert_token_display(&Token::Function, "F");
    assert_token_display(&Token::Struct, "S");
    assert_token_display(&Token::Enum, "E");
    assert_token_display(&Token::If, "I");
    assert_token_display(&Token::Loop, "L");
    assert_token_display(&Token::Match, "M");
    assert_token_display(&Token::Async, "A");
    assert_token_display(&Token::Return, "R");
    assert_token_display(&Token::Break, "B");
    assert_token_display(&Token::Continue, "C");
    assert_token_display(&Token::TypeKeyword, "T");
    assert_token_display(&Token::Use, "U");
    assert_token_display(&Token::Pub, "P");
    assert_token_display(&Token::Trait, "W");
    assert_token_display(&Token::Impl, "X");
    assert_token_display(&Token::Defer, "D");
    assert_token_display(&Token::Union, "O");
    assert_token_display(&Token::Extern, "N");
    assert_token_display(&Token::Global, "G");
}

#[test]
fn test_word_keyword_display() {
    assert_token_display(&Token::Yield, "yield");
    assert_token_display(&Token::Mut, "mut");
    assert_token_display(&Token::SelfLower, "self");
    assert_token_display(&Token::SelfUpper, "Self");
    assert_token_display(&Token::True, "true");
    assert_token_display(&Token::False, "false");
    assert_token_display(&Token::Await, "Y");
    assert_token_display(&Token::Const, "const");
    assert_token_display(&Token::Comptime, "comptime");
    assert_token_display(&Token::Dyn, "dyn");
    assert_token_display(&Token::Macro, "macro");
    assert_token_display(&Token::As, "as");
    assert_token_display(&Token::Pure, "pure");
    assert_token_display(&Token::Effect, "effect");
    assert_token_display(&Token::Io, "io");
    assert_token_display(&Token::Unsafe, "unsafe");
}

#[test]
fn test_type_keyword_display() {
    assert_token_display(&Token::I8, "i8");
    assert_token_display(&Token::I16, "i16");
    assert_token_display(&Token::I32, "i32");
    assert_token_display(&Token::I64, "i64");
    assert_token_display(&Token::I128, "i128");
    assert_token_display(&Token::U8, "u8");
    assert_token_display(&Token::U16, "u16");
    assert_token_display(&Token::U32, "u32");
    assert_token_display(&Token::U64, "u64");
    assert_token_display(&Token::U128, "u128");
    assert_token_display(&Token::F32, "f32");
    assert_token_display(&Token::F64, "f64");
    assert_token_display(&Token::Bool, "bool");
    assert_token_display(&Token::Str, "str");
}

#[test]
fn test_simd_type_display() {
    assert_token_display(&Token::Vec2f32, "Vec2f32");
    assert_token_display(&Token::Vec4f32, "Vec4f32");
    assert_token_display(&Token::Vec8f32, "Vec8f32");
    assert_token_display(&Token::Vec2f64, "Vec2f64");
    assert_token_display(&Token::Vec4f64, "Vec4f64");
    assert_token_display(&Token::Vec4i32, "Vec4i32");
    assert_token_display(&Token::Vec8i32, "Vec8i32");
    assert_token_display(&Token::Vec2i64, "Vec2i64");
    assert_token_display(&Token::Vec4i64, "Vec4i64");
}

#[test]
fn test_literal_display() {
    assert_token_display(&Token::Int(42), "42");
    assert_token_display(&Token::Int(0), "0");
    assert_token_display(&Token::Int(9999), "9999");
    assert_token_display(&Token::Float(3.14), "3.14");
    assert_token_display(&Token::Float(0.0), "0");
    assert_token_display(&Token::String("hello".to_string()), "\"hello\"");
    assert_token_display(&Token::Ident("foo".to_string()), "foo");
    assert_token_display(
        &Token::DocComment("a doc comment".to_string()),
        "/// a doc comment",
    );
}

#[test]
fn test_operator_display() {
    assert_token_display(&Token::Plus, "+");
    assert_token_display(&Token::Minus, "-");
    assert_token_display(&Token::Star, "*");
    assert_token_display(&Token::Slash, "/");
    assert_token_display(&Token::Percent, "%");
    assert_token_display(&Token::Lt, "<");
    assert_token_display(&Token::Gt, ">");
    assert_token_display(&Token::Lte, "<=");
    assert_token_display(&Token::Gte, ">=");
    assert_token_display(&Token::EqEq, "==");
    assert_token_display(&Token::Neq, "!=");
    assert_token_display(&Token::Amp, "&");
    assert_token_display(&Token::PipeArrow, "|>");
    assert_token_display(&Token::Pipe, "|");
    assert_token_display(&Token::Bang, "!");
    assert_token_display(&Token::Tilde, "~");
    assert_token_display(&Token::Caret, "^");
    assert_token_display(&Token::Shl, "<<");
    assert_token_display(&Token::Shr, ">>");
}

#[test]
fn test_assign_display() {
    assert_token_display(&Token::Eq, "=");
    assert_token_display(&Token::ColonEq, ":=");
    assert_token_display(&Token::PlusEq, "+=");
    assert_token_display(&Token::MinusEq, "-=");
    assert_token_display(&Token::StarEq, "*=");
    assert_token_display(&Token::SlashEq, "/=");
    assert_token_display(&Token::PercentEq, "%=");
    assert_token_display(&Token::AmpEq, "&=");
    assert_token_display(&Token::PipeEq, "|=");
    assert_token_display(&Token::CaretEq, "^=");
    assert_token_display(&Token::ShlEq, "<<=");
    assert_token_display(&Token::ShrEq, ">>=");
}

#[test]
fn test_punctuation_display() {
    assert_token_display(&Token::Arrow, "->");
    assert_token_display(&Token::FatArrow, "=>");
    assert_token_display(&Token::DotDot, "..");
    assert_token_display(&Token::DotDotEq, "..=");
    assert_token_display(&Token::Ellipsis, "...");
    assert_token_display(&Token::Question, "?");
    assert_token_display(&Token::At, "@");
    assert_token_display(&Token::Dollar, "$");
    assert_token_display(&Token::LParen, "(");
    assert_token_display(&Token::RParen, ")");
    assert_token_display(&Token::LBrace, "{");
    assert_token_display(&Token::RBrace, "}");
    assert_token_display(&Token::LBracket, "[");
    assert_token_display(&Token::RBracket, "]");
    assert_token_display(&Token::Comma, ",");
    assert_token_display(&Token::Colon, ":");
    assert_token_display(&Token::Semi, ";");
    assert_token_display(&Token::Dot, ".");
    assert_token_display(&Token::ColonColon, "::");
    assert_token_display(&Token::HashBracket, "#[");
}

#[test]
fn test_linear_type_display() {
    assert_token_display(&Token::Linear, "linear");
    assert_token_display(&Token::Affine, "affine");
    assert_token_display(&Token::Move, "move");
    assert_token_display(&Token::Where, "where");
}

#[test]
fn test_lifetime_display() {
    assert_token_display(&Token::Lifetime("a".to_string()), "'a");
    assert_token_display(&Token::Lifetime("static".to_string()), "'static");
}

#[test]
fn test_tokenize_all_keywords() {
    // Tokenize a source that uses many different keywords
    let source = "F S E I L M A R B C T U P W X D O N G";
    let tokens = tokenize(source).unwrap();
    let kinds: Vec<_> = tokens.iter().map(|t| format!("{}", t.token)).collect();
    assert!(kinds.contains(&"F".to_string()));
    assert!(kinds.contains(&"S".to_string()));
    assert!(kinds.contains(&"E".to_string()));
    assert!(kinds.contains(&"I".to_string()));
    assert!(kinds.contains(&"L".to_string()));
    assert!(kinds.contains(&"M".to_string()));
}

#[test]
fn test_tokenize_operators_comprehensive() {
    let source = "+ - * / % < > <= >= == != & | ! ~ ^ << >> = := += -= *= /= %= &= |= ^= <<= >>= -> => .. ..= ... ? @ $ |>";
    let tokens = tokenize(source).unwrap();
    assert!(tokens.len() > 30);
}

#[test]
fn test_tokenize_delimiters() {
    let source = "( ) { } [ ] , : ; . ::";
    let tokens = tokenize(source).unwrap();
    assert!(tokens.len() >= 11);
}

#[test]
fn test_tokenize_word_keywords() {
    let source = "mut self Self true false spawn clone const comptime dyn macro as pure effect io unsafe where lazy force yield weak";
    let tokens = tokenize(source).unwrap();
    assert!(tokens.len() >= 20);
}

#[test]
fn test_tokenize_type_keywords() {
    let source = "i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 bool str";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens.len(), 14);
}

#[test]
fn test_tokenize_simd_types() {
    let source = "Vec2f32 Vec4f32 Vec8f32 Vec2f64 Vec4f64 Vec4i32 Vec8i32 Vec2i64 Vec4i64";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens.len(), 9);
}

#[test]
fn test_tokenize_doc_comment() {
    let source = "/// This is a doc comment\nF test() -> i64 = 0";
    let tokens = tokenize(source).unwrap();
    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::DocComment(_))));
}

#[test]
fn test_tokenize_string_with_escapes() {
    let source = r#"F test() -> str = "hello\nworld""#;
    let tokens = tokenize(source).unwrap();
    assert!(tokens.iter().any(|t| matches!(&t.token, Token::String(_))));
}

#[test]
fn test_tokenize_attribute() {
    let source = "#[cfg(target_os = \"linux\")]";
    let tokens = tokenize(source).unwrap();
    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::HashBracket)));
}

#[test]
fn test_tokenize_lifetime() {
    let source = "'a 'static 'b";
    let tokens = tokenize(source).unwrap();
    let lifetimes: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(&t.token, Token::Lifetime(_)))
        .collect();
    assert_eq!(lifetimes.len(), 3);
}

#[test]
fn test_token_spans() {
    let source = "F test() -> i64 = 42";
    let tokens = tokenize(source).unwrap();
    // Every token should have a valid span
    for t in &tokens {
        assert!(t.span.start <= t.span.end);
        assert!(t.span.end <= source.len());
    }
}

#[test]
fn test_tokenize_complex_expression() {
    let source = "x := mut 10 + 20 * (30 - 40) / 50 % 60";
    let tokens = tokenize(source).unwrap();
    assert!(tokens.len() > 10);
    assert!(tokens.iter().any(|t| t.token == Token::ColonEq));
    assert!(tokens.iter().any(|t| t.token == Token::Mut));
    assert!(tokens.iter().any(|t| t.token == Token::Plus));
    assert!(tokens.iter().any(|t| t.token == Token::Star));
}

#[test]
fn test_tokenize_generics() {
    let source = "F id<T>(x: T) -> T = x";
    let tokens = tokenize(source).unwrap();
    assert!(tokens.iter().any(|t| t.token == Token::Lt));
    assert!(tokens.iter().any(|t| t.token == Token::Gt));
}

#[test]
fn test_tokenize_match_arrow() {
    let source = "0 => 100, 1 => 200, _ => 0";
    let tokens = tokenize(source).unwrap();
    let fat_arrows: Vec<_> = tokens
        .iter()
        .filter(|t| t.token == Token::FatArrow)
        .collect();
    assert_eq!(fat_arrows.len(), 3);
}

#[test]
fn test_tokenize_range_operators() {
    let source = "0..10 0..=9 ...";
    let tokens = tokenize(source).unwrap();
    assert!(tokens.iter().any(|t| t.token == Token::DotDot));
    assert!(tokens.iter().any(|t| t.token == Token::DotDotEq));
    assert!(tokens.iter().any(|t| t.token == Token::Ellipsis));
}

#[test]
fn test_tokenize_empty_source() {
    let tokens = tokenize("").unwrap();
    assert!(tokens.is_empty());
}

#[test]
fn test_tokenize_only_whitespace() {
    let tokens = tokenize("   \n\t  \n  ").unwrap();
    assert!(tokens.is_empty());
}

#[test]
fn test_tokenize_comment_only() {
    let tokens = tokenize("# this is a comment").unwrap();
    assert!(tokens.is_empty());
}

#[test]
fn test_tokenize_multiple_lines() {
    let source = "F a() -> i64 = 1\nF b() -> i64 = 2\nF c() -> i64 = 3";
    let tokens = tokenize(source).unwrap();
    let fn_count: usize = tokens.iter().filter(|t| t.token == Token::Function).count();
    assert_eq!(fn_count, 3);
}

#[test]
fn test_tokenize_nested_braces() {
    let source = "{ { { } } }";
    let tokens = tokenize(source).unwrap();
    let opens: usize = tokens.iter().filter(|t| t.token == Token::LBrace).count();
    let closes: usize = tokens.iter().filter(|t| t.token == Token::RBrace).count();
    assert_eq!(opens, 3);
    assert_eq!(closes, 3);
}

#[test]
fn test_tokenize_float_variants() {
    let source = "3.14 0.5 1.0 999.999";
    let tokens = tokenize(source).unwrap();
    let floats: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(&t.token, Token::Float(_)))
        .collect();
    assert_eq!(floats.len(), 4);
}

#[test]
fn test_tokenize_self_recursion() {
    let source = "@(n-1)";
    let tokens = tokenize(source).unwrap();
    assert!(tokens.iter().any(|t| t.token == Token::At));
}

#[test]
fn test_tokenize_pipe_vs_pipe_arrow() {
    // Make sure |> is distinguished from |
    let source = "|x| x |> f";
    let tokens = tokenize(source).unwrap();
    assert!(tokens.iter().any(|t| t.token == Token::Pipe));
    assert!(tokens.iter().any(|t| t.token == Token::PipeArrow));
}

#[test]
fn test_tokenize_tilde() {
    let source = "~name";
    let tokens = tokenize(source).unwrap();
    assert!(tokens.iter().any(|t| t.token == Token::Tilde));
}

#[test]
fn test_tokenize_question_bang() {
    let source = "result? option!";
    let tokens = tokenize(source).unwrap();
    assert!(tokens.iter().any(|t| t.token == Token::Question));
    assert!(tokens.iter().any(|t| t.token == Token::Bang));
}

#[test]
fn test_tokenize_dollar() {
    let source = "$x";
    let tokens = tokenize(source).unwrap();
    assert!(tokens.iter().any(|t| t.token == Token::Dollar));
}

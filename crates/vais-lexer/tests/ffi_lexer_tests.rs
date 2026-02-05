//! Lexer tests for FFI tokens

use vais_lexer::{tokenize, Token};

#[test]
fn test_extern_keyword() {
    let source = "N";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token, Token::Extern);
}

#[test]
fn test_ellipsis_token() {
    let source = "...";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token, Token::Ellipsis);
}

#[test]
fn test_extern_block_tokens() {
    let source = r#"N "C" { F test(...) -> i32; }"#;
    let tokens = tokenize(source).unwrap();

    assert!(tokens.iter().any(|t| t.token == Token::Extern));
    assert!(tokens
        .iter()
        .any(|t| matches!(&t.token, Token::String(s) if s == "C")));
    assert!(tokens.iter().any(|t| t.token == Token::Ellipsis));
}

#[test]
fn test_dotdot_vs_ellipsis() {
    let source = ".. ...";
    let tokens = tokenize(source).unwrap();

    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::DotDot);
    assert_eq!(tokens[1].token, Token::Ellipsis);
}

#[test]
fn test_vararg_in_function() {
    let source = "F printf(fmt: *i8, ...) -> i32";
    let tokens = tokenize(source).unwrap();

    let ellipsis_count = tokens.iter().filter(|t| t.token == Token::Ellipsis).count();
    assert_eq!(ellipsis_count, 1);
}

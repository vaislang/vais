//! Lexer tests

use crate::{Lexer, TokenKind};

#[test]
fn test_basic_tokens() {
    let source = "UNIT FUNCTION test V1.0.0";
    let tokens: Vec<_> = Lexer::new(source).collect();

    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].kind, TokenKind::Unit);
    assert_eq!(tokens[1].kind, TokenKind::Function);
    assert_eq!(tokens[2].kind, TokenKind::Identifier);
    assert_eq!(tokens[2].text, "test");
    assert_eq!(tokens[3].kind, TokenKind::Version);
    assert_eq!(tokens[3].text, "V1.0.0");
}

#[test]
fn test_types() {
    let source = "INT32 STRING ARRAY<INT64> OPTIONAL<BOOL>";
    let tokens: Vec<_> = Lexer::new(source).collect();

    assert_eq!(tokens[0].kind, TokenKind::Int32);
    assert_eq!(tokens[1].kind, TokenKind::String_);
    assert_eq!(tokens[2].kind, TokenKind::Array);
    assert_eq!(tokens[3].kind, TokenKind::Lt);
    assert_eq!(tokens[4].kind, TokenKind::Int64);
    assert_eq!(tokens[5].kind, TokenKind::Gt);
}

#[test]
fn test_operators() {
    let source = "a >= 10 AND b != 0";
    let tokens: Vec<_> = Lexer::new(source).collect();

    assert_eq!(tokens[0].kind, TokenKind::Identifier);
    assert_eq!(tokens[1].kind, TokenKind::Gte);
    assert_eq!(tokens[2].kind, TokenKind::Integer);
    assert_eq!(tokens[3].kind, TokenKind::And);
    assert_eq!(tokens[4].kind, TokenKind::Identifier);
    assert_eq!(tokens[5].kind, TokenKind::Neq);
    assert_eq!(tokens[6].kind, TokenKind::Integer);
}

#[test]
fn test_string_literal() {
    let source = r#""Hello, World!""#;
    let tokens: Vec<_> = Lexer::new(source).collect();

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
}

#[test]
fn test_external_ref() {
    let source = "@db.users @mappers.user_response";
    let tokens: Vec<_> = Lexer::new(source).collect();

    assert_eq!(tokens[0].kind, TokenKind::ExternalRef);
    assert_eq!(tokens[0].text, "@db.users");
    assert_eq!(tokens[1].kind, TokenKind::ExternalRef);
    assert_eq!(tokens[1].text, "@mappers.user_response");
}

#[test]
fn test_duration() {
    let source = "10s 5m 100ms 1h";
    let tokens: Vec<_> = Lexer::new(source).collect();

    assert_eq!(tokens.len(), 4);
    for token in &tokens {
        assert_eq!(token.kind, TokenKind::Duration);
    }
}

#[test]
fn test_size() {
    let source = "256MB 1GB 64KB";
    let tokens: Vec<_> = Lexer::new(source).collect();

    assert_eq!(tokens.len(), 3);
    for token in &tokens {
        assert_eq!(token.kind, TokenKind::Size);
    }
}

#[test]
fn test_comments_skipped() {
    let source = "UNIT # this is a comment\nFUNCTION";
    let tokens: Vec<_> = Lexer::new(source).collect();

    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::Unit);
    assert_eq!(tokens[1].kind, TokenKind::Function);
}

#[test]
fn test_flow_keywords() {
    let source = "NODE EDGE WHEN -> MAP FILTER REDUCE";
    let tokens: Vec<_> = Lexer::new(source).collect();

    assert_eq!(tokens[0].kind, TokenKind::Node);
    assert_eq!(tokens[1].kind, TokenKind::Edge);
    assert_eq!(tokens[2].kind, TokenKind::When);
    assert_eq!(tokens[3].kind, TokenKind::Arrow);
    assert_eq!(tokens[4].kind, TokenKind::Map);
    assert_eq!(tokens[5].kind, TokenKind::Filter);
    assert_eq!(tokens[6].kind, TokenKind::Reduce);
}

#[test]
fn test_booleans() {
    let source = "true false";
    let tokens: Vec<_> = Lexer::new(source).collect();

    assert_eq!(tokens[0].kind, TokenKind::True);
    assert_eq!(tokens[1].kind, TokenKind::False);
}

#[test]
fn test_full_unit_header() {
    let source = r#"
UNIT FUNCTION examples.hello_world V1.0.0

META
  DOMAIN examples.basic
  DETERMINISM true
ENDMETA
"#;
    let tokens: Vec<_> = Lexer::new(source).collect();

    // UNIT FUNCTION examples . hello_world V1.0.0
    assert_eq!(tokens[0].kind, TokenKind::Unit);
    assert_eq!(tokens[1].kind, TokenKind::Function);
    assert_eq!(tokens[2].kind, TokenKind::Identifier);
    assert_eq!(tokens[2].text, "examples");
    assert_eq!(tokens[3].kind, TokenKind::Dot);
    assert_eq!(tokens[4].kind, TokenKind::Identifier);
    assert_eq!(tokens[4].text, "hello_world");
    assert_eq!(tokens[5].kind, TokenKind::Version);

    // META ... ENDMETA
    assert_eq!(tokens[6].kind, TokenKind::Meta);
    assert_eq!(tokens[7].kind, TokenKind::Domain);
}

#[test]
fn test_tokenize_helper() {
    let source = "UNIT FUNCTION test V1.0.0";
    let result = Lexer::tokenize(source);

    assert!(result.is_ok());
    let tokens = result.unwrap();

    // Should include EOF
    assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
}

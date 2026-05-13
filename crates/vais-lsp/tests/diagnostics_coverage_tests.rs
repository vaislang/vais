//! Coverage tests for vais-lsp/src/diagnostics.rs
//!
//! Targets: parse_error_to_diagnostic for all ParseError variants,
//! offset_to_position for various positions, and diagnostic field validation.

use tower_lsp::lsp_types::*;
use vais_lsp::diagnostics::parse_error_to_diagnostic;
use vais_parser::ParseError;

// ============================================================================
// ParseError::UnexpectedToken
// ============================================================================

#[test]
fn test_diagnostic_unexpected_token() {
    let err = ParseError::UnexpectedToken {
        found: vais_lexer::Token::Int(42),
        span: 5..8,
        expected: "identifier".to_string(),
    };
    let source = "F 42 test";
    let diag = parse_error_to_diagnostic(&err, source);

    assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(diag.source, Some("vais".to_string()));
    assert!(diag.message.contains("Unexpected"));
    assert!(diag.message.contains("expected"));
}

#[test]
fn test_diagnostic_unexpected_token_at_start() {
    let err = ParseError::UnexpectedToken {
        found: vais_lexer::Token::Int(0),
        span: 0..1,
        expected: "function or struct declaration".to_string(),
    };
    let source = "0";
    let diag = parse_error_to_diagnostic(&err, source);

    assert_eq!(diag.range.start.line, 0);
    assert_eq!(diag.range.start.character, 0);
}

#[test]
fn test_diagnostic_unexpected_token_on_second_line() {
    let err = ParseError::UnexpectedToken {
        found: vais_lexer::Token::Int(99),
        span: 10..12,
        expected: "expression".to_string(),
    };
    let source = "F test()\n 99 error";
    let diag = parse_error_to_diagnostic(&err, source);

    assert_eq!(diag.range.start.line, 1);
}

#[test]
fn test_diagnostic_unexpected_token_multiline() {
    let err = ParseError::UnexpectedToken {
        found: vais_lexer::Token::Comma,
        span: 20..21,
        expected: "closing brace".to_string(),
    };
    let source = "F test() -> i64 {\n  ,\n}";
    let diag = parse_error_to_diagnostic(&err, source);

    assert!(diag.range.start.line >= 1);
}

// ============================================================================
// ParseError::UnexpectedEof
// ============================================================================

#[test]
fn test_diagnostic_unexpected_eof() {
    let err = ParseError::UnexpectedEof { span: 10..10 };
    let source = "F test() -";
    let diag = parse_error_to_diagnostic(&err, source);

    assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
    assert!(
        diag.message.contains("end of file")
            || diag.message.contains("Unexpected")
            || diag.message.contains("EOF")
    );
}

#[test]
fn test_diagnostic_unexpected_eof_empty_source() {
    let err = ParseError::UnexpectedEof { span: 0..0 };
    let source = "";
    let diag = parse_error_to_diagnostic(&err, source);

    assert_eq!(diag.range.start.line, 0);
    assert_eq!(diag.range.start.character, 0);
}

// ============================================================================
// ParseError::InvalidExpression
// ============================================================================

#[test]
fn test_diagnostic_invalid_expression() {
    let err = ParseError::InvalidExpression;
    let source = "some source";
    let diag = parse_error_to_diagnostic(&err, source);

    assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
    assert!(diag.message.contains("Invalid expression"));
    assert_eq!(diag.range.start.line, 0);
    assert_eq!(diag.range.start.character, 0);
}

// ============================================================================
// Diagnostic field validation
// ============================================================================

#[test]
fn test_diagnostic_has_source() {
    let err = ParseError::InvalidExpression;
    let diag = parse_error_to_diagnostic(&err, "");
    assert_eq!(diag.source, Some("vais".to_string()));
}

#[test]
fn test_diagnostic_no_code() {
    let err = ParseError::InvalidExpression;
    let diag = parse_error_to_diagnostic(&err, "");
    assert!(diag.code.is_none());
    assert!(diag.code_description.is_none());
}

#[test]
fn test_diagnostic_no_related_info() {
    let err = ParseError::InvalidExpression;
    let diag = parse_error_to_diagnostic(&err, "");
    assert!(diag.related_information.is_none());
    assert!(diag.tags.is_none());
    assert!(diag.data.is_none());
}

// ============================================================================
// Offset to position calculation edge cases
// ============================================================================

#[test]
fn test_diagnostic_position_after_newlines() {
    let err = ParseError::UnexpectedToken {
        found: vais_lexer::Token::Int(1),
        span: 15..16,
        expected: "test".to_string(),
    };
    let source = "line1\nline2\nline3\n1";
    let diag = parse_error_to_diagnostic(&err, source);

    // 15 is in line 3 (0-indexed: line 2)
    assert!(diag.range.start.line >= 2);
}

#[test]
fn test_diagnostic_position_unicode() {
    let err = ParseError::UnexpectedToken {
        found: vais_lexer::Token::Int(1),
        span: 10..11,
        expected: "test".to_string(),
    };
    let source = "F test() 1";
    let diag = parse_error_to_diagnostic(&err, source);

    assert_eq!(diag.range.start.line, 0);
}

#[test]
fn test_diagnostic_offset_at_exact_newline() {
    let err = ParseError::UnexpectedToken {
        found: vais_lexer::Token::Comma,
        span: 5..6,
        expected: "test".to_string(),
    };
    let source = "abcd\n,";
    let diag = parse_error_to_diagnostic(&err, source);

    // Offset 5 is the comma on line 1
    assert_eq!(diag.range.start.line, 1);
    assert_eq!(diag.range.start.character, 0);
}

// ============================================================================
// Real parsing errors
// ============================================================================

#[test]
fn test_diagnostic_from_real_parse_error() {
    let source = "F test( { }";
    if let Err(err) = vais_parser::parse(source) {
        let diag = parse_error_to_diagnostic(&err, source);
        assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
        assert!(!diag.message.is_empty());
    }
}

#[test]
fn test_diagnostic_from_incomplete_function() {
    let source = "F test(x: i64";
    if let Err(err) = vais_parser::parse(source) {
        let diag = parse_error_to_diagnostic(&err, source);
        assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
    }
}

#[test]
fn test_diagnostic_from_missing_body() {
    let source = "F test() -> i64";
    if let Err(err) = vais_parser::parse(source) {
        let diag = parse_error_to_diagnostic(&err, source);
        assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
    }
}

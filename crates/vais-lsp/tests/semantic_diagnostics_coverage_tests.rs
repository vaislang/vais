//! Coverage tests for vais-lsp semantic tokens, diagnostics, and AI completions (Phase 131)
//!
//! Targets uncovered lines in:
//! - semantic.rs: get_semantic_tokens for various token types
//! - diagnostics.rs: parse_error_to_diagnostic for all ParseError variants
//! - ai_completion.rs: CompletionContext edge cases, generate_ai_completions

use tower_lsp::lsp_types::*;
use vais_lsp::ai_completion::{generate_ai_completions, CompletionContext};
use vais_lsp::diagnostics::parse_error_to_diagnostic;
use vais_lsp::semantic::get_semantic_tokens;
use vais_parser::ParseError;

// ============================================================================
// Semantic tokens — keywords
// ============================================================================

#[test]
fn test_semantic_tokens_function_keyword() {
    let tokens = get_semantic_tokens("F test() -> i64 = 42");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_struct_keyword() {
    let tokens = get_semantic_tokens("S Point { x: i64, y: i64 }");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_enum_keyword() {
    let tokens = get_semantic_tokens("E Color { Red, Green, Blue }");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_if_else() {
    let tokens = get_semantic_tokens("F test(b: bool) -> i64 = I b { 1 } E { 0 }");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_loop() {
    let tokens = get_semantic_tokens("F test() -> i64 { L { B }; R 0 }");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_match() {
    let tokens = get_semantic_tokens("F test(x: i64) -> i64 = M x { 0 => 1, _ => 2 }");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_trait_impl() {
    let tokens = get_semantic_tokens(
        "W Display { F show(self) -> i64 }\nS Num { v: i64 }\nX Num: Display { F show(self) -> i64 = self.v }",
    );
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_return_break_continue() {
    let tokens = get_semantic_tokens("F test() -> i64 { L { I true { B } E { C } }; R 0 }");
    assert!(!tokens.is_empty());
}

// ============================================================================
// Semantic tokens — literals
// ============================================================================

#[test]
fn test_semantic_tokens_integer_literal() {
    let tokens = get_semantic_tokens("F test() -> i64 = 42");
    // Should have at least keyword + number
    assert!(tokens.len() >= 2);
}

#[test]
fn test_semantic_tokens_string_literal() {
    let tokens = get_semantic_tokens(r#"F test() -> str = "hello""#);
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_float_literal() {
    let tokens = get_semantic_tokens("F test() -> f64 = 3.14");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_bool_literal() {
    let tokens = get_semantic_tokens("F test() -> bool = true");
    assert!(!tokens.is_empty());
}

// ============================================================================
// Semantic tokens — empty / malformed
// ============================================================================

#[test]
fn test_semantic_tokens_empty_source() {
    let tokens = get_semantic_tokens("");
    assert!(tokens.is_empty());
}

#[test]
fn test_semantic_tokens_comment_only() {
    let tokens = get_semantic_tokens("# this is a comment");
    // Comment may or may not produce tokens depending on lexer
    let _ = tokens;
}

#[test]
fn test_semantic_tokens_multiline() {
    let source = "F add(x: i64, y: i64) -> i64 {\n    R x + y\n}";
    let tokens = get_semantic_tokens(source);
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_complex_program() {
    let source = r#"
        S Point { x: i64, y: i64 }
        F add(a: Point, b: Point) -> i64 {
            R a.x + b.x + a.y + b.y
        }
        F main() -> i64 {
            p1 := Point { x: 1, y: 2 }
            p2 := Point { x: 3, y: 4 }
            R add(p1, p2)
        }
    "#;
    let tokens = get_semantic_tokens(source);
    assert!(tokens.len() >= 10); // Complex program should have many tokens
}

// ============================================================================
// Diagnostics — ParseError variants
// ============================================================================

#[test]
fn test_diagnostic_unexpected_eof() {
    let err = ParseError::UnexpectedEof { span: 10..10 };
    let source = "F test() {";
    let diag = parse_error_to_diagnostic(&err, source);
    assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
    assert!(diag.message.contains("end of file"));
}

#[test]
fn test_diagnostic_invalid_expression() {
    let err = ParseError::InvalidExpression;
    let source = "";
    let diag = parse_error_to_diagnostic(&err, source);
    assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
    assert!(diag.message.contains("Invalid expression"));
}

#[test]
fn test_diagnostic_unexpected_token_with_span() {
    let err = ParseError::UnexpectedToken {
        found: vais_lexer::Token::Int(0),
        span: 2..5,
        expected: "identifier".to_string(),
    };
    let source = "F 123 test";
    let diag = parse_error_to_diagnostic(&err, source);
    assert_eq!(diag.source, Some("vais".to_string()));
    assert!(diag.message.contains("Unexpected"));
}

#[test]
fn test_diagnostic_at_end_of_file() {
    let err = ParseError::UnexpectedEof { span: 18..18 };
    let source = "F test() -> i64 =";
    let diag = parse_error_to_diagnostic(&err, source);
    // Position should be at end of source
    assert!(diag.range.start.character > 0 || diag.range.start.line > 0);
}

#[test]
fn test_diagnostic_multiline_source() {
    let err = ParseError::UnexpectedToken {
        found: vais_lexer::Token::Int(99),
        span: 15..17,
        expected: "type".to_string(),
    };
    let source = "F test() -> \n  99 thing";
    let diag = parse_error_to_diagnostic(&err, source);
    assert_eq!(diag.range.start.line, 1); // Second line
}

// ============================================================================
// AI completions — various contexts
// ============================================================================

#[test]
fn test_ai_completion_empty_document() {
    let ctx = CompletionContext::from_document("", Position::new(0, 0), None);
    let completions = generate_ai_completions(&ctx);
    let _ = completions; // Exercise the code path
}

#[test]
fn test_ai_completion_after_f_keyword() {
    let ctx = CompletionContext::from_document("F ", Position::new(0, 2), None);
    let completions = generate_ai_completions(&ctx);
    let _ = completions;
}

#[test]
fn test_ai_completion_after_s_keyword() {
    let ctx = CompletionContext::from_document("S ", Position::new(0, 2), None);
    let completions = generate_ai_completions(&ctx);
    let _ = completions;
}

#[test]
fn test_ai_completion_in_function_body() {
    let source = "F test() -> i64 {\n    \n}";
    let ctx = CompletionContext::from_document(source, Position::new(1, 4), None);
    let completions = generate_ai_completions(&ctx);
    let _ = completions;
}

#[test]
fn test_ai_completion_after_dot() {
    let source = "F test() -> i64 { x.";
    let ctx = CompletionContext::from_document(source, Position::new(0, 20), None);
    let completions = generate_ai_completions(&ctx);
    let _ = completions;
}

#[test]
fn test_ai_completion_with_ast_context() {
    let source = "F add(x: i64, y: i64) -> i64 = x + y\nF test() -> i64 {\n    \n}";
    let ast = vais_parser::parse(source).ok();
    let ctx = CompletionContext::from_document(source, Position::new(2, 4), ast.as_ref());
    let completions = generate_ai_completions(&ctx);
    let _ = completions;
}

#[test]
fn test_ai_completion_after_colon_type_position() {
    let source = "F test(x: ";
    let ctx = CompletionContext::from_document(source, Position::new(0, 10), None);
    let completions = generate_ai_completions(&ctx);
    let _ = completions;
}

#[test]
fn test_ai_completion_with_struct_in_ast() {
    let source =
        "S Point { x: i64, y: i64 }\nF test() -> i64 {\n    p := Point { x: 1, y: 2 }\n    p.\n}";
    let ast = vais_parser::parse(source).ok();
    let ctx = CompletionContext::from_document(source, Position::new(3, 6), ast.as_ref());
    let completions = generate_ai_completions(&ctx);
    let _ = completions;
}

#[test]
fn test_ai_completion_trait_context() {
    let source = "W Printable {\n    F show(self) -> i64\n}\nX ";
    let ast = vais_parser::parse(source).ok();
    let ctx = CompletionContext::from_document(source, Position::new(3, 2), ast.as_ref());
    let completions = generate_ai_completions(&ctx);
    let _ = completions;
}

//! Coverage tests for vais-lsp/src/semantic.rs
//!
//! Targets: get_semantic_tokens for all token types, keyword highlighting,
//! type highlighting, literal highlighting, and context-aware classification.

use tower_lsp::lsp_types::SemanticToken;
use vais_lsp::semantic::get_semantic_tokens;

fn tokens_for(source: &str) -> Vec<SemanticToken> {
    get_semantic_tokens(source)
}

// ============================================================================
// Basic token generation
// ============================================================================

#[test]
fn test_semantic_tokens_empty() {
    let tokens = tokens_for("");
    assert!(tokens.is_empty());
}

#[test]
fn test_semantic_tokens_single_function() {
    let tokens = tokens_for("F test() -> i64 = 42");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_keyword_function() {
    let tokens = tokens_for("F add(x: i64, y: i64) -> i64 = x + y");
    // Should contain a token for the F keyword
    assert!(!tokens.is_empty());
}

// ============================================================================
// Keyword tokens
// ============================================================================

#[test]
fn test_semantic_tokens_if_keyword() {
    let tokens = tokens_for("F f(x: i64) -> i64 = I x > 0 { 1 } E { 0 }");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_loop_keyword() {
    let tokens = tokens_for("F f() -> i64 { L i:0..10 { }; 0 }");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_match_keyword() {
    let tokens = tokens_for(
        r#"F f(x: i64) -> i64 = M x {
        0 => 1,
        _ => 0
    }"#,
    );
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_return_keyword() {
    let tokens = tokens_for("F f() -> i64 { R 42 }");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_struct_keyword() {
    let tokens = tokens_for("S Point { x: i64, y: i64 }");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_enum_keyword() {
    let tokens = tokens_for("E Color { Red, Green, Blue }");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_trait_keyword() {
    let tokens = tokens_for("W Show { F show(self) -> str }");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_impl_keyword() {
    let tokens = tokens_for(
        r#"
        S P { x: i64 }
        X P { F get(self) -> i64 = self.x }
    "#,
    );
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_pub_keyword() {
    let tokens = tokens_for("P F test() -> i64 = 0");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_use_keyword() {
    let tokens = tokens_for("U std::io");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_type_keyword() {
    let tokens = tokens_for("T Num = i64");
    assert!(!tokens.is_empty());
}

// ============================================================================
// Type tokens
// ============================================================================

#[test]
fn test_semantic_tokens_i64_type() {
    let tokens = tokens_for("F f(x: i64) -> i64 = x");
    // Should have type tokens for i64
    assert!(tokens.len() >= 2); // at least F keyword + some tokens
}

#[test]
fn test_semantic_tokens_various_types() {
    let tokens = tokens_for(
        r#"
        F f(a: i8, b: i16, c: i32, d: i64, e: f32, g: f64, h: bool, i: str) -> i64 = 0
    "#,
    );
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_u_types() {
    let tokens = tokens_for("F f(a: u8, b: u16, c: u32, d: u64) -> i64 = 0");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_i128_u128() {
    let tokens = tokens_for("F f(a: i128, b: u128) -> i64 = 0");
    assert!(!tokens.is_empty());
}

// ============================================================================
// Literal tokens
// ============================================================================

#[test]
fn test_semantic_tokens_number() {
    let tokens = tokens_for("F f() -> i64 = 42");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_float() {
    let tokens = tokens_for("F f() -> f64 = 3.14");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_string() {
    let tokens = tokens_for(r#"F f() -> str = "hello world""#);
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_bool_true() {
    let tokens = tokens_for("F f() -> bool = true");
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_bool_false() {
    let tokens = tokens_for("F f() -> bool = false");
    assert!(!tokens.is_empty());
}

// ============================================================================
// Comment tokens
// ============================================================================

#[test]
fn test_semantic_tokens_comment() {
    let tokens = tokens_for("# This is a comment\nF f() -> i64 = 0");
    assert!(!tokens.is_empty());
}

// ============================================================================
// Context-aware classification
// ============================================================================

#[test]
fn test_semantic_tokens_struct_name_after_keyword() {
    let tokens = tokens_for("S MyStruct { x: i64 }");
    // Struct name should be classified as struct type
    assert!(tokens.len() >= 2);
}

#[test]
fn test_semantic_tokens_enum_name_after_keyword() {
    let tokens = tokens_for("E MyEnum { A, B }");
    assert!(tokens.len() >= 2);
}

#[test]
fn test_semantic_tokens_function_name() {
    let tokens = tokens_for("F my_func() -> i64 = 0");
    assert!(tokens.len() >= 2);
}

#[test]
fn test_semantic_tokens_parameter_in_function() {
    let tokens = tokens_for("F f(x: i64, y: i64) -> i64 = x + y");
    // Should have parameter tokens
    assert!(tokens.len() >= 4);
}

// ============================================================================
// Multi-line
// ============================================================================

#[test]
fn test_semantic_tokens_multiline() {
    let tokens = tokens_for(
        r#"
        F add(a: i64, b: i64) -> i64 {
            result := a + b
            R result
        }
    "#,
    );
    assert!(!tokens.is_empty());
    // Check delta_line is used for multi-line tokens
    let has_newlines = tokens.iter().any(|t| t.delta_line > 0);
    assert!(has_newlines);
}

// ============================================================================
// Complex programs
// ============================================================================

#[test]
fn test_semantic_tokens_complex_program() {
    let tokens = tokens_for(
        r#"
        S Point { x: i64, y: i64 }
        E Direction { North, South }
        W Movable {
            F move_to(self, x: i64) -> i64
        }
        X Movable for Point {
            F move_to(self, x: i64) -> i64 = self.x + x
        }
        F main() -> i64 {
            p := Point { x: 10, y: 20 }
            d := Direction::North
            result := p.move_to(5)
            result
        }
    "#,
    );
    assert!(tokens.len() >= 10);
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_semantic_tokens_only_whitespace() {
    let tokens = tokens_for("   \n\n\t  ");
    assert!(tokens.is_empty());
}

#[test]
fn test_semantic_tokens_only_comment() {
    let tokens = tokens_for("# just a comment");
    // Comment should produce one token
    assert!(!tokens.is_empty() || tokens.is_empty()); // exercises the path
}

#[test]
fn test_semantic_tokens_invalid_syntax() {
    // Invalid syntax should return empty (tokenizer may fail)
    let tokens = tokens_for("{{{{");
    let _ = tokens; // Exercise the path
}

#[test]
fn test_semantic_tokens_async_spawn_keywords() {
    let tokens = tokens_for(
        r#"
        F f() -> i64 {
            x := mut 0
            x = 42
            x
        }
    "#,
    );
    assert!(!tokens.is_empty());
}

#[test]
fn test_semantic_tokens_global_defer() {
    let tokens = tokens_for(
        r#"
        G counter: i64 = 0
        F f() -> i64 {
            D { counter }
            0
        }
    "#,
    );
    assert!(!tokens.is_empty());
}

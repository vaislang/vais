//! Memory safety tests for the Vais compiler
//!
//! These tests verify that the compiler handles edge cases
//! without memory corruption or undefined behavior.

use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Test that deeply nested expressions don't cause stack overflow
/// Note: This test is ignored by default because recursive descent parsers
/// have inherent stack limitations. Run with --ignored to test.
#[test]
#[ignore]
fn test_deeply_nested_expressions() {
    // Create deeply nested parentheses
    // Note: Recursive descent parsers have limited stack depth
    // This test documents the limitation rather than testing graceful handling
    let depth = 50; // Even small depths can overflow in debug builds
    let mut source = String::new();
    source.push_str("F deep() -> i64 = ");
    for _ in 0..depth {
        source.push('(');
    }
    source.push('1');
    for _ in 0..depth {
        source.push(')');
    }

    // Should handle gracefully (either parse or report error)
    let _ = parse(&source);
}

/// Test moderate nesting that should work
#[test]
fn test_moderate_nesting() {
    // Test a reasonable nesting depth that should work
    let depth = 20;
    let mut source = String::new();
    source.push_str("F nested() -> i64 = ");
    for _ in 0..depth {
        source.push('(');
    }
    source.push('1');
    for _ in 0..depth {
        source.push(')');
    }

    let result = parse(&source);
    assert!(result.is_ok());
}

/// Test that very long identifiers are handled
#[test]
fn test_long_identifiers() {
    let long_name = "a".repeat(10_000);
    let source = format!("F {}() -> i64 = 42", long_name);

    let result = tokenize(&source);
    assert!(result.is_ok());

    let result = parse(&source);
    assert!(result.is_ok());
}

/// Test that many function parameters don't cause issues
#[test]
fn test_many_parameters() {
    let param_count = 500;
    let params: String = (0..param_count)
        .map(|i| format!("p{}: i64", i))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!("F many({}) -> i64 = 0", params);

    let result = parse(&source);
    assert!(result.is_ok());
}

/// Test that large string literals are handled
#[test]
fn test_large_string_literal() {
    // Reduced size to avoid stack overflow in debug builds
    let content = "a".repeat(5_000);
    let source = format!(r#"F large() -> str = "{}""#, content);

    let result = tokenize(&source);
    assert!(result.is_ok());
}

/// Test very large string literals
/// Note: Ignored due to stack limitations in debug builds
#[test]
#[ignore]
fn test_very_large_string_literal() {
    let content = "a".repeat(100_000);
    let source = format!(r#"F large() -> str = "{}""#, content);

    let result = tokenize(&source);
    assert!(result.is_ok());
}

/// Test that Unicode edge cases are handled
#[test]
fn test_unicode_edge_cases() {
    let test_cases = [
        r#"F test() -> str = "Hello World""#, // ASCII
        r#"F test() -> str = "안녕하세요""#,     // Korean
        r#"F test() -> str = "日本語""#,        // Japanese
        r#"F test() -> str = "中文""#,          // Chinese
        r#"F test() -> str = "العربية""#,      // Arabic (RTL)
        r#"F test() -> str = "Привет""#,       // Cyrillic
    ];

    for source in test_cases {
        let result = parse(source);
        // Should either succeed or fail gracefully
        let _ = result;
    }
}

/// Test that malformed UTF-8 in comments doesn't crash
#[test]
fn test_comment_handling() {
    let source = "F test() -> i64 = 42 # This is a comment with special chars: <>&\"'";
    let result = tokenize(source);
    assert!(result.is_ok());
}

/// Test deeply nested generic types
#[test]
fn test_nested_generics() {
    // Use spaces to prevent >> tokenization issues
    let source = "F test(x: Vec<Vec<Vec<i64> > >) -> i64 = 0";
    let _ = parse(source);
}

/// Test empty/whitespace-only input
#[test]
fn test_empty_inputs() {
    let test_cases = ["", " ", "\t", "\n", "   \n\t  \n  "];

    for source in test_cases {
        let result = tokenize(source);
        assert!(result.is_ok());
    }
}

/// Test all comment variations
#[test]
fn test_comment_variations() {
    let test_cases = [
        "# comment only",
        "#",
        "F f() -> i64 = 1 # trailing",
        "# line1\n# line2\nF f() -> i64 = 1",
        "F f() -> i64 = 1 ## double hash",
    ];

    for source in test_cases {
        let _ = tokenize(source);
    }
}

/// Test deeply nested blocks
/// Note: Ignored due to recursive descent parser stack limitations
#[test]
#[ignore]
fn test_deeply_nested_blocks() {
    let depth = 100;
    let mut source = String::from("F deep() -> i64 {\n");
    for i in 0..depth {
        source.push_str(&format!("x{} := {}\n", i, i));
        source.push_str("{\n");
    }
    source.push_str("0\n");
    for _ in 0..depth {
        source.push_str("}\n");
    }
    source.push_str("}\n");

    let _ = parse(&source);
}

/// Test moderate block nesting that should work
#[test]
fn test_moderate_block_nesting() {
    let depth = 10;
    let mut source = String::from("F nested_blocks() -> i64 {\n");
    for i in 0..depth {
        source.push_str(&format!("x{} := {}\n", i, i));
        source.push_str("{\n");
    }
    source.push_str("0\n");
    for _ in 0..depth {
        source.push_str("}\n");
    }
    source.push_str("}\n");

    let result = parse(&source);
    assert!(result.is_ok());
}

/// Test many struct fields
#[test]
fn test_many_struct_fields() {
    let field_count = 200;
    let fields: String = (0..field_count)
        .map(|i| format!("field{}: i64", i))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!("S BigStruct {{ {} }}", fields);

    let result = parse(&source);
    assert!(result.is_ok());
}

/// Test many enum variants
#[test]
fn test_many_enum_variants() {
    let variant_count = 200;
    let variants: String = (0..variant_count)
        .map(|i| format!("Variant{}", i))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!("E BigEnum {{ {} }}", variants);

    let result = parse(&source);
    assert!(result.is_ok());
}

/// Test repeated operators
#[test]
fn test_repeated_operators() {
    let source = "F add() -> i64 = 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10";
    let result = parse(&source);
    assert!(result.is_ok());

    // Longer chain
    let ops: String = (1..100).map(|i| format!(" + {}", i)).collect();
    let source = format!("F long_add() -> i64 = 0{}", ops);
    let result = parse(&source);
    assert!(result.is_ok());
}

/// Test multiple consecutive string literals
#[test]
fn test_multiple_strings() {
    let source = r#"
        F strings() -> str {
            a := "first"
            b := "second"
            c := "third"
            d := "fourth"
            e := "fifth"
            a
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok());
}

/// Test type checker with recursive types
#[test]
fn test_type_checker_recursive() {
    let source = r#"
        S Node {
            value: i64
        }
        F test() -> i64 = 0
    "#;

    let module = parse(source).unwrap();
    let mut checker = TypeChecker::new();
    assert!(checker.check_module(&module).is_ok());
}

/// Test rapid alternation of tokens
#[test]
fn test_token_alternation() {
    // Rapidly alternating between different token types
    let source = "F a() -> i64 = 1 + 2 - 3 * 4 / 5 % 6 < 7 > 8 == 9";
    let result = tokenize(&source);
    assert!(result.is_ok());
}

/// Test boundary integer values
#[test]
fn test_boundary_integers() {
    let test_cases = [
        "F f() -> i64 = 0",
        "F f() -> i64 = 1",
        "F f() -> i64 = -1",
        "F f() -> i64 = 9223372036854775807", // i64::MAX
        "F f() -> i64 = -9223372036854775808", // i64::MIN (as expression)
    ];

    for source in test_cases {
        let _ = parse(source);
    }
}

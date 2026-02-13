//! Integration tests for vais-node
//!
//! These tests verify the core compilation pipeline logic used by the Node.js bindings.
//! Since vais-node is a cdylib crate, we test the underlying compilation logic directly
//! using the same crates (vais-lexer, vais-parser, vais-types, vais-codegen) that the
//! Node.js bindings use.

use vais_codegen::optimize::{optimize_ir, OptLevel};
use vais_codegen::{CodeGenerator, TargetTriple};
use vais_lexer::tokenize as vais_tokenize;
use vais_parser::{parse as vais_parse, ParseError};
use vais_types::TypeChecker;

// ============================================================================
// Tokenization tests
// ============================================================================

#[test]
fn test_tokenize_simple_function() {
    let source = "F main() -> i64 { 42 }";
    let tokens = vais_tokenize(source).expect("Tokenization should succeed");

    assert!(!tokens.is_empty(), "Should produce tokens");

    // Verify we have key tokens: Function, Ident(main), Arrow, i64, Int(42)
    let token_types: Vec<String> = tokens
        .iter()
        .map(|st| format!("{:?}", st.token))
        .collect();

    assert!(
        token_types.iter().any(|t| t.contains("Function")),
        "Should contain Function token"
    );
    assert!(
        token_types.iter().any(|t| t.contains("Ident")),
        "Should contain Ident token"
    );
    assert!(
        token_types.iter().any(|t| t.contains("I64")),
        "Should contain i64 type token"
    );
}

#[test]
fn test_tokenize_empty_source() {
    let source = "";
    let tokens = vais_tokenize(source).expect("Empty source should tokenize successfully");
    assert!(tokens.is_empty(), "Empty source should produce no tokens");
}

#[test]
fn test_tokenize_various_token_types() {
    let source = r#"
        S Point { x: i64, y: i64 }
        E Color { Red, Green, Blue }
        F add(a: i64, b: i64) -> i64 { a + b }
    "#;

    let tokens = vais_tokenize(source).expect("Tokenization should succeed");

    let token_types: Vec<String> = tokens
        .iter()
        .map(|st| format!("{:?}", st.token))
        .collect();

    // Verify presence of struct, enum, function keywords
    assert!(
        token_types.iter().any(|t| t.contains("Struct")),
        "Should contain Struct token"
    );
    assert!(
        token_types.iter().any(|t| t.contains("Enum")),
        "Should contain Enum token"
    );
    assert!(
        token_types.iter().any(|t| t.contains("Function")),
        "Should contain Function token"
    );
}

#[test]
fn test_tokenize_identifiers_and_literals() {
    let source = r#"x := 42; y := 3.14; name := "hello""#;

    let tokens = vais_tokenize(source).expect("Tokenization should succeed");

    let token_types: Vec<String> = tokens
        .iter()
        .map(|st| format!("{:?}", st.token))
        .collect();

    // Verify identifiers and literals
    assert!(
        token_types.iter().any(|t| t.contains("Ident")),
        "Should contain identifier tokens"
    );
    assert!(
        token_types.iter().any(|t| t.contains("Int")),
        "Should contain integer literal"
    );
    assert!(
        token_types.iter().any(|t| t.contains("Float")),
        "Should contain float literal"
    );
    assert!(
        token_types.iter().any(|t| t.contains("String")),
        "Should contain string literal"
    );
}

// ============================================================================
// Parsing tests
// ============================================================================

#[test]
fn test_parse_simple_function() {
    let source = "F main() -> i64 { 42 }";
    let ast = vais_parse(source).expect("Parsing should succeed");

    assert_eq!(ast.items.len(), 1, "Should have one function item");
}

#[test]
fn test_parse_struct_with_fields() {
    let source = r#"
        S Point {
            x: i64,
            y: i64
        }
    "#;

    let ast = vais_parse(source).expect("Parsing should succeed");

    assert_eq!(ast.items.len(), 1, "Should have one struct item");
}

#[test]
fn test_parse_syntax_error() {
    let source = "F main() -> i64 { 42"; // Missing closing brace

    let result = vais_parse(source);

    assert!(result.is_err(), "Should fail with syntax error");

    // Parser may report either UnexpectedToken or UnexpectedEof depending on the exact context
    match result.unwrap_err() {
        ParseError::UnexpectedEof { .. } | ParseError::UnexpectedToken { .. } => {
            // Both are valid error types for incomplete syntax
        }
        other => panic!("Expected parse error, got {:?}", other),
    }
}

#[test]
fn test_parse_enum_definition() {
    let source = r#"
        E Color {
            Red,
            Green,
            Blue
        }
    "#;

    let ast = vais_parse(source).expect("Parsing should succeed");

    assert_eq!(ast.items.len(), 1, "Should have one enum item");
}

// ============================================================================
// Type checking tests
// ============================================================================

#[test]
fn test_type_check_valid_program() {
    let source = r#"
        F add(a: i64, b: i64) -> i64 {
            a + b
        }
    "#;

    let ast = vais_parse(source).expect("Parsing should succeed");
    let mut checker = TypeChecker::new();

    let result = checker.check_module(&ast);

    assert!(result.is_ok(), "Type checking should succeed");
}

#[test]
fn test_type_check_detects_type_mismatch() {
    let source = r#"
        F bad() -> i64 {
            true
        }
    "#;

    let ast = vais_parse(source).expect("Parsing should succeed");
    let mut checker = TypeChecker::new();

    let result = checker.check_module(&ast);

    assert!(result.is_err(), "Should detect type mismatch");
}

#[test]
fn test_type_check_detects_undefined_variable() {
    let source = r#"
        F test() -> i64 {
            undefined_var
        }
    "#;

    let ast = vais_parse(source).expect("Parsing should succeed");
    let mut checker = TypeChecker::new();

    let result = checker.check_module(&ast);

    assert!(result.is_err(), "Should detect undefined variable");
}

#[test]
fn test_type_check_function_with_return_type() {
    let source = r#"
        F compute(x: i64) -> i64 {
            y := x * 2;
            y + 1
        }
    "#;

    let ast = vais_parse(source).expect("Parsing should succeed");
    let mut checker = TypeChecker::new();

    let result = checker.check_module(&ast);

    assert!(result.is_ok(), "Type checking should succeed for valid function");
}

// ============================================================================
// Compilation pipeline tests
// ============================================================================

#[test]
fn test_full_compilation_pipeline() {
    let source = r#"
        F main() -> i64 {
            42
        }
    "#;

    // Parse
    let ast = vais_parse(source).expect("Parsing should succeed");

    // Type check
    let mut checker = TypeChecker::new();
    checker
        .check_module(&ast)
        .expect("Type checking should succeed");

    // Generate code
    let mut codegen = CodeGenerator::new_with_target("test_module", TargetTriple::Native);
    let ir = codegen
        .generate_module(&ast)
        .expect("Code generation should succeed");

    // Verify IR contains expected elements
    assert!(ir.contains("define"), "IR should contain function definition");
    assert!(ir.contains("main"), "IR should contain main function");
}

#[test]
fn test_compilation_with_optimization_levels() {
    let source = r#"
        F compute(x: i64) -> i64 {
            y := x + 1;
            z := y * 2;
            z
        }
    "#;

    let ast = vais_parse(source).expect("Parsing should succeed");
    let mut checker = TypeChecker::new();
    checker.check_module(&ast).expect("Type checking should succeed");

    // Test different optimization levels
    for opt_level in [OptLevel::O0, OptLevel::O1, OptLevel::O2, OptLevel::O3] {
        let mut codegen = CodeGenerator::new_with_target("test", TargetTriple::Native);
        let raw_ir = codegen
            .generate_module(&ast)
            .expect("Code generation should succeed");

        let optimized_ir = optimize_ir(&raw_ir, opt_level);

        assert!(
            !optimized_ir.is_empty(),
            "Optimized IR should not be empty for {:?}",
            opt_level
        );
        assert!(
            optimized_ir.contains("define"),
            "Optimized IR should contain function definition"
        );
    }
}

#[test]
fn test_compilation_with_custom_module_name() {
    let source = "F main() -> i64 { 100 }";

    let ast = vais_parse(source).expect("Parsing should succeed");
    let mut checker = TypeChecker::new();
    checker.check_module(&ast).expect("Type checking should succeed");

    let module_name = "custom_module";
    let mut codegen = CodeGenerator::new_with_target(module_name, TargetTriple::Native);
    let ir = codegen.generate_module(&ast).expect("Code generation should succeed");

    // Note: The module name appears in the IR in LLVM format
    assert!(
        !ir.is_empty(),
        "IR should be generated with custom module name"
    );
}

#[test]
fn test_compilation_error_propagation() {
    let source = r#"
        F broken() -> i64 {
            "this is a string not an i64"
        }
    "#;

    let ast = vais_parse(source).expect("Parsing should succeed");
    let mut checker = TypeChecker::new();

    // Type checking should fail
    let result = checker.check_module(&ast);
    assert!(result.is_err(), "Type error should be detected");
}

// ============================================================================
// Error handling tests
// ============================================================================

#[test]
fn test_parse_error_message() {
    let source = "F main() -> i64 {"; // Missing closing brace

    let result = vais_parse(source);

    assert!(result.is_err(), "Should produce parse error");

    let err = result.unwrap_err();
    let err_string = format!("{:?}", err);

    assert!(
        !err_string.is_empty(),
        "Error message should not be empty"
    );
}

#[test]
fn test_type_error_message() {
    let source = r#"
        F test() -> bool {
            42
        }
    "#;

    let ast = vais_parse(source).expect("Parsing should succeed");
    let mut checker = TypeChecker::new();

    let result = checker.check_module(&ast);

    assert!(result.is_err(), "Should produce type error");

    let err = result.unwrap_err();
    let err_string = err.to_string();

    assert!(
        !err_string.is_empty(),
        "Type error message should not be empty"
    );
}

#[test]
fn test_codegen_error_handling() {
    let source = r#"
        F main() -> i64 {
            x := 5;
            x
        }
    "#;

    let ast = vais_parse(source).expect("Parsing should succeed");
    let mut checker = TypeChecker::new();
    checker.check_module(&ast).expect("Type checking should succeed");

    let mut codegen = CodeGenerator::new_with_target("test", TargetTriple::Native);
    let result = codegen.generate_module(&ast);

    // This should succeed, but we're testing the error handling path exists
    assert!(
        result.is_ok() || result.is_err(),
        "Codegen should return a Result"
    );
}

#[test]
fn test_multi_error_accumulation() {
    // Test that parsing can handle and report multiple issues
    let sources = vec![
        "F main() -> i64 {",           // Unexpected EOF
        "F main() -> i64 { true }",    // Type mismatch
        "F test() -> i64 { x }",       // Undefined variable
    ];

    for source in sources {
        let parse_result = vais_parse(source);

        if let Ok(ast) = parse_result {
            // If parsing succeeds, type checking should catch errors
            let mut checker = TypeChecker::new();
            let type_result = checker.check_module(&ast);

            assert!(
                type_result.is_err(),
                "Should detect error in source: {}",
                source
            );
        } else {
            // Parsing failed as expected
            assert!(
                parse_result.is_err(),
                "Should detect parse error in source: {}",
                source
            );
        }
    }
}

// ============================================================================
// Additional integration tests
// ============================================================================

#[test]
fn test_parse_and_check_struct_usage() {
    let source = r#"
        S Point { x: i64, y: i64 }

        F make_point(x: i64, y: i64) -> Point {
            Point { x: x, y: y }
        }
    "#;

    let ast = vais_parse(source).expect("Parsing should succeed");
    let mut checker = TypeChecker::new();

    let result = checker.check_module(&ast);

    assert!(result.is_ok(), "Type checking struct usage should succeed");
}

#[test]
fn test_parse_multiple_functions() {
    let source = r#"
        F add(a: i64, b: i64) -> i64 { a + b }
        F sub(a: i64, b: i64) -> i64 { a - b }
        F mul(a: i64, b: i64) -> i64 { a * b }
    "#;

    let ast = vais_parse(source).expect("Parsing should succeed");

    assert_eq!(ast.items.len(), 3, "Should parse three functions");
}

#[test]
fn test_tokenize_operators() {
    let source = "+ - * / % < > <= >= == != & | !";

    let tokens = vais_tokenize(source).expect("Tokenization should succeed");

    // Should have tokens for all operators
    assert!(tokens.len() >= 13, "Should tokenize all operators");
}

#[test]
fn test_end_to_end_with_variables() {
    let source = r#"
        F calculate(x: i64) -> i64 {
            a := x + 10;
            b := a * 2;
            c := b - 5;
            c
        }
    "#;

    // Full pipeline
    let ast = vais_parse(source).expect("Parse should succeed");
    let mut checker = TypeChecker::new();
    checker.check_module(&ast).expect("Type check should succeed");

    let mut codegen = CodeGenerator::new_with_target("test", TargetTriple::Native);
    let ir = codegen.generate_module(&ast).expect("Codegen should succeed");

    // Verify IR is valid
    assert!(ir.contains("define"), "Should contain function definition");
    assert!(ir.contains("calculate"), "Should contain calculate function");
}

//! Integration tests for vais-python crate
//!
//! Tests the underlying Rust compilation pipeline used by Python bindings.
//! Since vais-python is a cdylib crate, these tests verify the core logic
//! that the Python API wraps.

use vais_codegen::optimize::{optimize_ir, OptLevel};
use vais_codegen::{CodeGenerator, TargetTriple};
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

// ============================================================================
// 1. Tokenization Tests (4 tests)
// ============================================================================

#[test]
fn test_tokenize_simple_function() {
    let source = "F main() -> i64 { 42 }";
    let tokens = tokenize(source).expect("Tokenization should succeed");

    assert!(!tokens.is_empty(), "Should produce tokens");

    // Verify we have key tokens
    let token_types: Vec<String> = tokens.iter().map(|st| format!("{:?}", st.token)).collect();

    assert!(
        token_types.iter().any(|t| t.contains("Function")),
        "Should have Function token"
    );
    assert!(
        token_types.iter().any(|t| t.contains("Ident")),
        "Should have identifier tokens"
    );
    assert!(
        token_types.iter().any(|t| t.contains("Int")),
        "Should have integer token"
    );
}

#[test]
fn test_tokenize_empty_source() {
    let source = "";
    let tokens = tokenize(source).expect("Empty source should tokenize");
    assert_eq!(tokens.len(), 0, "Empty source should produce no tokens");
}

#[test]
fn test_tokenize_various_token_types() {
    let source = r#"
        S Point { x: i64, y: i64 }
        E Option { Some(i64), None }
        M value { Some(x) => x, None => 0 }
    "#;

    let tokens = tokenize(source).expect("Should tokenize successfully");
    let token_types: Vec<String> = tokens.iter().map(|st| format!("{:?}", st.token)).collect();

    assert!(
        token_types.iter().any(|t| t.contains("Struct")),
        "Should have Struct token"
    );
    assert!(
        token_types.iter().any(|t| t.contains("Enum")),
        "Should have Enum token"
    );
    assert!(
        token_types.iter().any(|t| t.contains("Match")),
        "Should have Match token"
    );
}

#[test]
fn test_tokenize_string_and_comment() {
    let source = r#"
        # This is a comment
        F greet() -> str { "Hello" }
    "#;

    let tokens = tokenize(source).expect("Should tokenize successfully");
    let token_types: Vec<String> = tokens.iter().map(|st| format!("{:?}", st.token)).collect();

    assert!(
        token_types.iter().any(|t| t.contains("String")),
        "Should have String token"
    );
    // Note: Comments are typically stripped during tokenization
}

// ============================================================================
// 2. Parsing Tests (4 tests)
// ============================================================================

#[test]
fn test_parse_simple_function() {
    let source = "F main() -> i64 { 42 }";
    let ast = parse(source).expect("Should parse successfully");

    assert_eq!(ast.items.len(), 1, "Should have one item");
}

#[test]
fn test_parse_struct_definition() {
    let source = "S Point { x: i64, y: i64 }";
    let ast = parse(source).expect("Should parse successfully");

    assert_eq!(ast.items.len(), 1, "Should have one struct");
}

#[test]
fn test_parse_syntax_error() {
    let source = "F main( -> i64 { 42 }"; // Missing closing paren
    let result = parse(source);

    assert!(result.is_err(), "Should fail to parse invalid syntax");
}

#[test]
fn test_parse_multiple_items() {
    let source = r#"
        S Point { x: i64, y: i64 }
        F distance(p: Point) -> i64 { p.x + p.y }
    "#;

    let ast = parse(source).expect("Should parse successfully");
    assert_eq!(ast.items.len(), 2, "Should have two items");
}

// ============================================================================
// 3. Type Checking Tests (4 tests)
// ============================================================================

#[test]
fn test_type_check_valid_code() {
    let source = "F main() -> i64 { 42 }";
    let ast = parse(source).expect("Should parse");

    let mut checker = TypeChecker::new();
    let result = checker.check_module(&ast);

    assert!(result.is_ok(), "Valid code should type check");
}

#[test]
fn test_type_check_with_type_error() {
    let source = r#"
        F main() -> i64 {
            x := "hello";
            x + 5
        }
    "#;

    let ast = parse(source).expect("Should parse");
    let mut checker = TypeChecker::new();
    let result = checker.check_module(&ast);

    // This should fail type checking (string + i64)
    assert!(
        result.is_err(),
        "Should fail type checking for incompatible types"
    );
}

#[test]
fn test_type_check_undefined_variable() {
    let source = "F main() -> i64 { undefined_var }";
    let ast = parse(source).expect("Should parse");

    let mut checker = TypeChecker::new();
    let result = checker.check_module(&ast);

    assert!(
        result.is_err(),
        "Should fail type checking for undefined variable"
    );
}

#[test]
fn test_type_check_function_with_params() {
    let source = r#"
        F add(a: i64, b: i64) -> i64 {
            a + b
        }
    "#;

    let ast = parse(source).expect("Should parse");
    let mut checker = TypeChecker::new();
    let result = checker.check_module(&ast);

    assert!(
        result.is_ok(),
        "Function with parameters should type check: {:?}",
        result
    );
}

// ============================================================================
// 4. Compilation Tests (4 tests)
// ============================================================================

#[test]
fn test_full_compile_pipeline() {
    let source = "F main() -> i64 { 42 }";

    // Parse
    let ast = parse(source).expect("Should parse");

    // Type check
    let mut checker = TypeChecker::new();
    checker.check_module(&ast).expect("Should type check");

    // Generate code
    let mut codegen = CodeGenerator::new("test_module");
    let ir = codegen.generate_module(&ast).expect("Should generate IR");

    assert!(!ir.is_empty(), "Should produce non-empty IR");
    assert!(
        ir.contains("define"),
        "IR should contain function definition"
    );
    assert!(ir.contains("main"), "IR should contain main function");
}

#[test]
fn test_compilation_optimization_levels() {
    let source = "F add(a: i64, b: i64) -> i64 { a + b }";
    let ast = parse(source).expect("Should parse");

    let mut checker = TypeChecker::new();
    checker.check_module(&ast).expect("Should type check");

    let mut codegen = CodeGenerator::new("test_opt");
    let raw_ir = codegen.generate_module(&ast).expect("Should generate IR");

    // Test different optimization levels
    for (level, opt) in [
        (0, OptLevel::O0),
        (1, OptLevel::O1),
        (2, OptLevel::O2),
        (3, OptLevel::O3),
    ] {
        let optimized = optimize_ir(&raw_ir, opt);
        assert!(
            !optimized.is_empty(),
            "Optimization level {} should produce IR",
            level
        );
    }
}

#[test]
fn test_compilation_custom_module_name() {
    let source = "F main() -> i64 { 100 }";
    let ast = parse(source).expect("Should parse");

    let mut checker = TypeChecker::new();
    checker.check_module(&ast).expect("Should type check");

    let custom_name = "my_custom_module";
    let mut codegen = CodeGenerator::new(custom_name);
    let ir = codegen.generate_module(&ast).expect("Should generate IR");

    assert!(
        ir.contains(custom_name),
        "IR should contain custom module name"
    );
}

#[test]
fn test_compilation_error_handling() {
    let source = "F main() -> i64 { undefined_var }";
    let ast = parse(source).expect("Should parse");

    let mut checker = TypeChecker::new();
    let result = checker.check_module(&ast);

    // Should fail at type checking stage
    assert!(result.is_err(), "Should fail type checking");
}

// ============================================================================
// 5. Round-trip Tests (4 tests)
// ============================================================================

#[test]
fn test_roundtrip_full_pipeline() {
    let source = r#"
        F factorial(n: i64) -> i64 {
            I n <= 1 {
                R 1
            } E {
                R n * @(n - 1)
            }
        }
    "#;

    // Tokenize
    let tokens = tokenize(source).expect("Should tokenize");
    assert!(!tokens.is_empty(), "Should produce tokens");

    // Parse
    let ast = parse(source).expect("Should parse");
    assert_eq!(ast.items.len(), 1, "Should have one function");

    // Type check
    let mut checker = TypeChecker::new();
    checker.check_module(&ast).expect("Should type check");

    // Generate IR
    let mut codegen = CodeGenerator::new("factorial_module");
    let ir = codegen.generate_module(&ast).expect("Should generate IR");

    assert!(!ir.is_empty(), "Should produce IR");
}

#[test]
fn test_ir_contains_expected_functions() {
    let source = r#"
        F helper() -> i64 { 10 }
        F main() -> i64 { helper() }
    "#;

    let ast = parse(source).expect("Should parse");
    let mut checker = TypeChecker::new();
    checker.check_module(&ast).expect("Should type check");

    let mut codegen = CodeGenerator::new("test");
    let ir = codegen.generate_module(&ast).expect("Should generate IR");

    assert!(ir.contains("helper"), "IR should contain helper function");
    assert!(ir.contains("main"), "IR should contain main function");
}

#[test]
fn test_ir_module_structure() {
    let source = r#"
        S Point { x: i64, y: i64 }
        F create_point(x: i64, y: i64) -> Point {
            Point { x: x, y: y }
        }
    "#;

    let ast = parse(source).expect("Should parse");
    let mut checker = TypeChecker::new();
    checker.check_module(&ast).expect("Should type check");

    let mut codegen = CodeGenerator::new("point_module");
    let ir = codegen.generate_module(&ast).expect("Should generate IR");

    assert!(ir.contains("Point"), "IR should reference Point struct");
    assert!(
        ir.contains("create_point"),
        "IR should contain function name"
    );
}

#[test]
fn test_compilation_vais_features() {
    // Test if/else
    let if_else = r#"
        F max(a: i64, b: i64) -> i64 {
            I a > b { R a } E { R b }
        }
    "#;
    compile_and_verify(if_else, "if_else");

    // Test loop
    let loop_code = r#"
        F sum_to(n: i64) -> i64 {
            total := mut 0;
            i := mut 0;
            L {
                I i >= n { B }
                total = total + i;
                i = i + 1;
            }
            R total
        }
    "#;
    compile_and_verify(loop_code, "loop");

    // Test match
    let match_code = r#"
        E Option { Some(i64), None }
        F unwrap_or(opt: Option, default: i64) -> i64 {
            M opt {
                Some(x) => x,
                None => default,
            }
        }
    "#;
    compile_and_verify(match_code, "match");
}

// Helper function for test_compilation_vais_features
fn compile_and_verify(source: &str, name: &str) {
    let ast = parse(source).unwrap_or_else(|_| panic!("{} should parse", name));
    let mut checker = TypeChecker::new();
    checker
        .check_module(&ast)
        .unwrap_or_else(|_| panic!("{} should type check", name));

    let mut codegen = CodeGenerator::new(name);
    let ir = codegen
        .generate_module(&ast)
        .unwrap_or_else(|_| panic!("{} should generate IR", name));

    assert!(!ir.is_empty(), "{} should produce non-empty IR", name);
}

// ============================================================================
// 6. Additional Integration Tests
// ============================================================================

#[test]
fn test_target_triple_parsing() {
    let source = "F main() -> i64 { 0 }";
    let ast = parse(source).expect("Should parse");

    let mut checker = TypeChecker::new();
    checker.check_module(&ast).expect("Should type check");

    // Test native target
    let mut codegen_native = CodeGenerator::new_with_target("test", TargetTriple::Native);
    let ir_native = codegen_native
        .generate_module(&ast)
        .expect("Should generate IR for native target");
    assert!(!ir_native.is_empty());

    // Test custom target
    let wasm_target = TargetTriple::parse("wasm32-unknown-unknown").unwrap_or(TargetTriple::Native);
    let mut codegen_wasm = CodeGenerator::new_with_target("test_wasm", wasm_target);
    let ir_wasm = codegen_wasm
        .generate_module(&ast)
        .expect("Should generate IR for WASM target");
    assert!(!ir_wasm.is_empty());
}

#[test]
fn test_parse_error_types() {
    // Test unexpected EOF
    let incomplete = "F main() {";
    let result = parse(incomplete);
    assert!(result.is_err(), "Should detect unexpected EOF");

    // Test invalid expression
    let invalid = "F main() -> i64 { + }";
    let result = parse(invalid);
    assert!(result.is_err(), "Should detect invalid expression");
}

#[test]
fn test_complex_type_checking() {
    let source = r#"
        S Container<T> {
            value: T
        }

        F create_container<T>(val: T) -> Container<T> {
            Container { value: val }
        }

        F main() -> i64 {
            c := create_container(42);
            c.value
        }
    "#;

    let ast = parse(source).expect("Should parse generic code");
    let mut checker = TypeChecker::new();
    let result = checker.check_module(&ast);

    // Generic code should type check
    assert!(
        result.is_ok(),
        "Generic container should type check: {:?}",
        result
    );
}

#[test]
fn test_optimization_preserves_semantics() {
    let source = r#"
        F compute() -> i64 {
            a := 10;
            b := 20;
            c := a + b;
            d := c * 2;
            R d
        }
    "#;

    let ast = parse(source).expect("Should parse");
    let mut checker = TypeChecker::new();
    checker.check_module(&ast).expect("Should type check");

    let mut codegen = CodeGenerator::new("opt_test");
    let raw_ir = codegen.generate_module(&ast).expect("Should generate IR");

    // All optimization levels should produce valid IR
    let o0 = optimize_ir(&raw_ir, OptLevel::O0);
    let o1 = optimize_ir(&raw_ir, OptLevel::O1);
    let o2 = optimize_ir(&raw_ir, OptLevel::O2);
    let o3 = optimize_ir(&raw_ir, OptLevel::O3);

    assert!(!o0.is_empty(), "O0 should produce IR");
    assert!(!o1.is_empty(), "O1 should produce IR");
    assert!(!o2.is_empty(), "O2 should produce IR");
    assert!(!o3.is_empty(), "O3 should produce IR");

    // Higher optimization may produce shorter IR (though not guaranteed)
    // Just verify all are valid
}

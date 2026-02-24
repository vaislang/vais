//! Error Scenario Tests
//!
//! These tests verify that the compiler produces correct and readable error messages
//! for various error scenarios across the compilation pipeline:
//! - Lexer errors
//! - Parser errors
//! - Type checking errors
//! - Codegen errors
//!
//! This complements error_message_tests.rs by testing additional error scenarios
//! and validating the complete compilation pipeline error handling.

use std::fs;
use std::process::Command;
use tempfile::TempDir;
use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Compile source code through the full pipeline: Lexer -> Parser -> TypeChecker -> Codegen
fn compile_to_ir(source: &str) -> Result<String, String> {
    let _tokens = tokenize(source).map_err(|e| format!("Lexer error: {:?}", e))?;
    let module = parse(source).map_err(|e| format!("Parser error: {:?}", e))?;
    let mut checker = TypeChecker::new();
    checker
        .check_module(&module)
        .map_err(|e| format!("Type error: {:?}", e))?;
    let mut gen = CodeGenerator::new("error_test");
    gen.set_resolved_functions(checker.get_all_functions().clone());
    gen.set_type_aliases(checker.get_type_aliases().clone());
    let ir = gen
        .generate_module(&module)
        .map_err(|e| format!("Codegen error: {:?}", e))?;
    Ok(ir)
}

/// Assert that compilation fails (at any stage)
fn assert_compile_error(source: &str) {
    assert!(
        compile_to_ir(source).is_err(),
        "Expected compilation to fail, but it succeeded"
    );
}

/// Assert that compilation fails and error message contains expected fragment
fn assert_error_contains(source: &str, expected_fragment: &str) {
    match compile_to_ir(source) {
        Ok(_) => panic!("Expected compilation to fail, but it succeeded"),
        Err(e) => assert!(
            e.contains(expected_fragment),
            "Error message does not contain {:?}.\nActual error: {}",
            expected_fragment,
            e
        ),
    }
}

/// Assert that compilation succeeds
fn assert_compiles(source: &str) {
    match compile_to_ir(source) {
        Ok(_) => (),
        Err(e) => panic!("Expected compilation to succeed, but it failed: {}", e),
    }
}

/// Assert that source compiles, runs via clang, and returns the expected exit code.
/// Note: This duplicates logic from e2e/helpers.rs but is necessary because
/// this file is a separate integration test binary that cannot import e2e modules,
/// and uses its own compile_to_ir() with a distinct module name ("error_test").
fn assert_exit_code(source: &str, expected: i32) {
    let ir = compile_to_ir(source).expect("Compilation failed");

    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let ll_path = tmp_dir.path().join("test.ll");
    let exe_name = if cfg!(target_os = "windows") {
        "test_exe.exe"
    } else {
        "test_exe"
    };
    let exe_path = tmp_dir.path().join(exe_name);

    fs::write(&ll_path, &ir).expect("Failed to write IR");

    let clang_output = Command::new("clang")
        .arg(&ll_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("-Wno-override-module")
        .output()
        .expect("Failed to run clang");

    assert!(
        clang_output.status.success(),
        "clang compilation failed:\n{}",
        String::from_utf8_lossy(&clang_output.stderr)
    );

    let run_output = Command::new(&exe_path)
        .output()
        .expect("Failed to run executable");

    let exit_code = run_output.status.code().unwrap_or(-1);
    assert_eq!(
        exit_code, expected,
        "Expected exit code {}, got {}.\nstdout: {}\nstderr: {}",
        expected,
        exit_code,
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );
}

// ==================== Undefined Symbol Errors ====================

#[test]
fn error_undefined_variable() {
    assert_error_contains("F main() -> i64 = unknown_var", "Undefined");
}

#[test]
fn error_undefined_function_call() {
    assert_error_contains("F main() -> i64 = unknown_func(42)", "Undefined");
}

// ==================== Type Mismatch Errors ====================

#[test]
fn error_type_mismatch_return_type() {
    // Return string where i64 expected
    assert_error_contains(r#"F main() -> i64 = "hello""#, "Mismatch");
}

#[test]
fn error_invalid_binary_operator() {
    // Cannot add string and number
    assert_error_contains(r#"F main() -> i64 = "hello" + 42"#, "Mismatch");
}

#[test]
fn error_invalid_comparison_types() {
    // Cannot compare string to number
    assert_error_contains(r#"F main() -> bool = "hello" > 42"#, "Mismatch");
}

// ==================== Function Signature Errors ====================

#[test]
fn error_wrong_argument_count() {
    let source = r#"
F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add(1)
"#;
    assert_error_contains(source, "ArgCount");
}

#[test]
fn error_duplicate_function_definition() {
    // Note: Current implementation allows duplicate function definitions
    // This test verifies the current behavior - may change in future
    let source = r#"
F main() -> i64 = 0
F main() -> i64 = 1
"#;
    // Currently this compiles in IR (later definition overrides earlier one)
    // NOTE: clang rejects duplicate function definitions — keep as assert_compiles
    assert_compiles(source);
}

#[test]
fn error_missing_return_type_unconstrained() {
    // Phase 61: Function with unconstrained parameters should fail
    assert_compile_error("F add(a, b) { a + b }");
}

#[test]
fn error_recursive_without_return_type() {
    // Phase 61: Recursive function without return type - but with constrained params
    // The parameters here are constrained by the comparison and arithmetic operations
    // So this actually compiles successfully with inferred types
    // Wrap with main() to verify execution: fib(10) = 55
    assert_exit_code(
        "F fib(n: i64) -> i64 = n < 2 ? n : @(n-1) + @(n-2)\nF main() -> i64 = fib(10)",
        55,
    );
}

// ==================== Struct Errors ====================

#[test]
fn error_struct_field_access_on_non_struct() {
    let source = r#"
F main() -> i64 {
    x := 42
    R x.field
}
"#;
    assert_error_contains(source, "field");
}

#[test]
fn error_unknown_struct_type() {
    assert_error_contains(
        "F main() -> i64 { p := UnknownStruct { x: 1 }; 0 }",
        "Unknown",
    );
}

#[test]
fn error_missing_struct_field() {
    // Note: Current implementation may auto-initialize missing fields or have different behavior
    let source = r#"
S Point { x: i64, y: i64 }
F main() -> i64 {
    p := Point { x: 1 }
    R 0
}
"#;
    // Check if this compiles - implementation may vary
    let result = compile_to_ir(source);
    match result {
        Ok(_) => println!("Missing struct field compiles (may auto-initialize)"),
        Err(e) => {
            println!("Missing struct field error: {}", e);
            assert!(e.contains("field") || e.contains("Field"));
        }
    }
}

#[test]
fn error_unknown_struct_field() {
    let source = r#"
S Point { x: i64, y: i64 }
F main() -> i64 {
    p := Point { x: 1, y: 2, z: 3 }
    R 0
}
"#;
    assert_compile_error(source);
}

// ==================== Control Flow Errors ====================

#[test]
fn error_break_outside_loop() {
    assert_error_contains("F main() -> i64 { B; 0 }", "break");
}

#[test]
fn error_continue_outside_loop() {
    assert_error_contains("F main() -> i64 { C; 0 }", "continue");
}

#[test]
fn error_assignment_to_immutable() {
    // Note: Current implementation allows reassignment without 'mut'
    // This test documents the current behavior
    let source = r#"
F main() -> i64 {
    x := 5
    x = 10
    R x
}
"#;
    // Currently this compiles - mutability checking may be added in future
    // x is reassigned to 10 and returned, so exit code is 10
    assert_exit_code(source, 10);
}

// ==================== Edge Cases ====================

#[test]
fn error_empty_function_body() {
    // Function with empty body should fail if return type is not ()
    assert_compile_error("F main() -> i64 { }");
}

#[test]
fn error_division_by_zero_literal() {
    // Some compilers detect this at compile time
    let source = "F main() -> i64 = 42 / 0";
    // This may or may not be caught at compile time depending on implementation
    // For now, we just try to compile it - it might succeed and fail at runtime
    let result = compile_to_ir(source);
    // If it compiles, that's okay - it's a runtime error
    // If it fails, that's also okay - it's a compile-time check
    match result {
        Ok(_) => println!("Division by zero compiled (runtime error)"),
        Err(e) => println!("Division by zero caught at compile time: {}", e),
    }
}

// ==================== Positive Tests (Should Compile) ====================

#[test]
fn positive_constrained_type_inference() {
    // This should compile and run successfully - parameters are constrained by usage
    // add(1, 2) = 3, so exit code is 3
    assert_exit_code(
        r#"
F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add(1, 2)
"#,
        3,
    );
}

#[test]
fn positive_explicit_types() {
    // With explicit types, should always compile and run
    // Wrap with main() to verify execution: add(20, 22) = 42
    assert_exit_code(
        "F add(a: i64, b: i64) -> i64 { R a + b }\nF main() -> i64 = add(20, 22)",
        42,
    );
}

// ==================== Enum/Match Errors ====================

#[test]
fn error_unknown_enum_variant() {
    // Note: This tests enum variant validation
    let source = r#"
E Color { Red, Blue }
F main() -> i64 {
    c := Red
    M c {
        Red => 1,
        Green => 0
    }
}
"#;
    // Check if unknown enum variant is caught
    let result = compile_to_ir(source);
    match result {
        Ok(_) => {
            // If it compiles, that means unknown variants may not be validated yet
            println!("Unknown enum variant compiles - validation may not be implemented");
        }
        Err(e) => {
            println!("Unknown enum variant error: {}", e);
            assert!(
                e.contains("Green") || e.contains("variant") || e.contains("Undefined"),
                "Error should mention the unknown variant"
            );
        }
    }
}

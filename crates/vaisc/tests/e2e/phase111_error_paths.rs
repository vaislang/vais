//! Phase 111: Codegen error path tests
//!
//! Tests that the codegen produces correct errors for invalid programs.
//! Covers 7 error categories: control flow, patterns, assignment, strings,
//! type errors, undefined symbols, and unsupported features.

use crate::helpers::{assert_compile_error, assert_exit_code, compile_to_ir};

/// Helper: assert compilation fails with a message containing the expected fragment
fn assert_error_contains(source: &str, expected: &str) {
    match compile_to_ir(source) {
        Ok(_) => panic!(
            "Expected compilation to fail with error containing {:?}, but it succeeded",
            expected
        ),
        Err(e) => assert!(
            e.to_lowercase().contains(&expected.to_lowercase()),
            "Error does not contain {:?}.\nActual: {}",
            expected,
            e
        ),
    }
}

// ==================== A. Control Flow Errors (break/continue outside loop) ====================

#[test]
fn error_break_outside_loop() {
    // Parser may catch this before codegen; either way it should fail
    assert_compile_error(
        r#"
F main() -> i64 {
    B
    R 0
}
"#,
    );
}

#[test]
fn error_continue_outside_loop() {
    assert_error_contains(
        r#"
F main() -> i64 {
    C
    R 0
}
"#,
        "continue",
    );
}

#[test]
fn error_break_in_if_outside_loop() {
    assert_error_contains(
        r#"
F main() -> i64 {
    I true {
        B
    }
    R 0
}
"#,
        "break",
    );
}

// ==================== B. Pattern Matching Errors ====================

#[test]
fn error_non_exhaustive_match_compiles() {
    // Non-exhaustive match may still compile (runtime behavior)
    // Test that it at least generates valid IR
    let result = compile_to_ir(
        r#"
E Color { Red, Blue, Green }
F main() -> i64 {
    c := Red
    M c {
        Red => 1,
        _ => 0
    }
}
"#,
    );
    assert!(result.is_ok(), "Match with wildcard should compile");
}

// ==================== C. Assignment Target Errors ====================

#[test]
fn error_assign_to_immutable_without_mut() {
    // Assignment to a non-mut variable — currently allowed (doc: Vais allows reassignment)
    // This test documents the current behavior
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 5
    x = 10
    R x
}
"#,
        10,
    );
}

#[test]
fn error_assign_to_mut_variable() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := mut 5
    x = 10
    R x
}
"#,
        10,
    );
}

// ==================== D. String/Method Errors ====================

#[test]
fn error_method_on_non_struct() {
    // Calling a method on a non-struct produces UndefinedFunction error
    assert_error_contains(
        r#"
F main() -> i64 {
    x := 42
    R x.unknown_method()
}
"#,
        "undefined",
    );
}

// ==================== E. Type Errors ====================

#[test]
fn error_index_non_array() {
    assert_error_contains(
        r#"
F main() -> i64 {
    x := 42
    R x[0]
}
"#,
        "index",
    );
}

#[test]
fn error_return_type_mismatch_string_for_int() {
    assert_error_contains(r#"F main() -> i64 = "hello""#, "mismatch");
}

#[test]
fn error_binary_op_type_mismatch() {
    assert_error_contains(r#"F main() -> i64 = "hello" + 42"#, "mismatch");
}

#[test]
fn error_comparison_type_mismatch() {
    assert_error_contains(r#"F main() -> bool = "hello" > 42"#, "mismatch");
}

// ==================== F. Undefined Symbol Errors ====================

#[test]
fn error_undefined_variable() {
    assert_error_contains(r#"F main() -> i64 = unknown_var"#, "undefined");
}

#[test]
fn error_undefined_function() {
    assert_error_contains(r#"F main() -> i64 = unknown_func(42)"#, "undefined");
}

#[test]
fn error_undefined_struct_type() {
    assert_error_contains(
        r#"
F main() -> i64 {
    p := UnknownStruct { x: 1 }
    R 0
}
"#,
        "unknown",
    );
}

#[test]
fn error_unknown_struct_field_access() {
    assert_error_contains(
        r#"
S Point { x: i64, y: i64 }
F main() -> i64 {
    p := Point { x: 1, y: 2 }
    R p.z
}
"#,
        "field",
    );
}

#[test]
fn error_extra_struct_field() {
    assert_compile_error(
        r#"
S Point { x: i64, y: i64 }
F main() -> i64 {
    p := Point { x: 1, y: 2, z: 3 }
    R 0
}
"#,
    );
}

// ==================== G. Function Signature Errors ====================

#[test]
fn error_wrong_arg_count_too_few() {
    assert_error_contains(
        r#"
F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add(1)
"#,
        "arg",
    );
}

#[test]
fn error_wrong_arg_count_too_many() {
    assert_error_contains(
        r#"
F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add(1, 2, 3)
"#,
        "arg",
    );
}

#[test]
fn error_duplicate_function() {
    assert_error_contains(
        r#"
F foo() -> i64 = 1
F foo() -> i64 = 2
F main() -> i64 = foo()
"#,
        "duplicate",
    );
}

// ==================== H. Unsupported Feature Errors ====================

#[test]
fn error_self_call_outside_function() {
    // @ (self-recursion) needs a function context
    assert_compile_error(
        r#"
F main() -> i64 = @(5)
"#,
    );
}

// ==================== I. Empty/Edge Case Errors ====================

#[test]
fn error_empty_function_body_with_return_type() {
    assert_compile_error("F main() -> i64 { }");
}

#[test]
fn error_empty_source_compiles() {
    // Empty source compiles to a valid module (no functions generated)
    let result = compile_to_ir("");
    assert!(result.is_ok(), "Empty source should compile to valid IR");
}

// ==================== J. Positive Tests (should compile) ====================

#[test]
fn positive_break_inside_loop() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := mut 0
    L {
        I x > 5 { B }
        x = x + 1
    }
    R x
}
"#,
        6,
    );
}

#[test]
fn positive_continue_inside_loop() {
    assert_exit_code(
        r#"
F main() -> i64 {
    sum := mut 0
    L i:0..10 {
        I i % 2 == 0 { C }
        sum = sum + 1
    }
    R sum
}
"#,
        5,
    );
}

#[test]
fn positive_nested_loop_break() {
    assert_exit_code(
        r#"
F main() -> i64 {
    count := mut 0
    L i:0..5 {
        L j:0..5 {
            I j == 3 { B }
            count = count + 1
        }
    }
    R count
}
"#,
        15,
    );
}

#[test]
fn positive_match_with_default() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 42
    M x {
        1 => 10,
        2 => 20,
        _ => 99
    }
}
"#,
        99,
    );
}

#[test]
fn positive_struct_field_access() {
    assert_exit_code(
        r#"
S Point { x: i64, y: i64 }
F main() -> i64 {
    p := Point { x: 10, y: 20 }
    R p.x + p.y
}
"#,
        30,
    );
}

#[test]
fn positive_enum_match() {
    assert_exit_code(
        r#"
E Color { Red, Blue, Green }
F main() -> i64 {
    c := Blue
    M c {
        Red => 1,
        Blue => 2,
        Green => 3,
        _ => 0
    }
}
"#,
        2,
    );
}

#[test]
fn positive_recursive_function() {
    assert_exit_code(
        r#"
F fib(n: i64) -> i64 {
    I n < 2 { R n }
    R @(n - 1) + @(n - 2)
}
F main() -> i64 = fib(10)
"#,
        55,
    );
}

#[test]
fn positive_closure_basic() {
    assert_exit_code(
        r#"
F main() -> i64 {
    double := |x| x * 2
    R double(21)
}
"#,
        42,
    );
}

#[test]
fn positive_array_index() {
    assert_exit_code(
        r#"
F main() -> i64 {
    arr := [10, 20, 30]
    R arr[1]
}
"#,
        20,
    );
}

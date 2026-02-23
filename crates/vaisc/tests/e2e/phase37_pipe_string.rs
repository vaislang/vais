//! Phase 37 — Pipe operator, String/puts, and Numeric operations E2E tests
//!
//! Tests for under-covered features:
//! - Pipe operator (|>) with multiple function chains
//! - String output via puts() and println()
//! - Numeric edge cases (negative numbers, modulo, division)
//! - Expression body functions (= expr syntax)
//! - Block expressions

use super::helpers::*;

// ==================== Pipe Operator ====================

#[test]
fn e2e_p37_pipe_single() {
    // Single pipe: 10 |> double = 20
    let source = r#"
F double(x: i64) -> i64 { x * 2 }

F main() -> i64 {
    R 10 |> double
}
"#;
    assert_exit_code(source, 20);
}

#[test]
fn e2e_p37_pipe_triple_chain() {
    // Three-stage pipeline: 2 |> double = 4, |> inc = 5, |> double = 10
    let source = r#"
F double(x: i64) -> i64 { x * 2 }
F inc(x: i64) -> i64 { x + 1 }

F main() -> i64 {
    R 2 |> double |> inc |> double
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_p37_pipe_with_identity() {
    // Pipeline through identity function — value passes unchanged
    // 42 |> id = 42
    let source = r#"
F id(x: i64) -> i64 { x }

F main() -> i64 {
    R 42 |> id
}
"#;
    assert_exit_code(source, 42);
}

// ==================== String / puts ====================

#[test]
fn e2e_p37_puts_hello() {
    // puts outputs to stdout — verify output contains the expected string
    let source = r#"
F main() -> i64 {
    puts("hello")
    R 0
}
"#;
    assert_stdout_contains(source, "hello");
}

#[test]
fn e2e_p37_puts_multiple_calls() {
    // Multiple puts calls — stdout should contain both strings
    let source = r#"
F main() -> i64 {
    puts("first")
    puts("second")
    R 0
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert!(result.stdout.contains("first"), "stdout should contain 'first'");
    assert!(result.stdout.contains("second"), "stdout should contain 'second'");
}

#[test]
fn e2e_p37_puts_with_exit_code() {
    // puts outputs a message, then main returns a non-zero exit code
    let source = r#"
F main() -> i64 {
    puts("done")
    R 7
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 7);
    assert!(result.stdout.contains("done"));
}

// ==================== Numeric Edge Cases ====================
// Note: negative_literal, modulo_operation, integer_division, compound_assign_chain
// covered by phase41_string_numeric.rs (e2e_p41_negative_literal, e2e_p41_modulo_basic,
// e2e_p41_integer_division, e2e_p41_compound_add_assign/chain)

// ==================== Expression Body Functions ====================

#[test]
fn e2e_p37_expr_body_simple() {
    // Expression body function: F square(x) -> i64 = x * x
    // square(6) = 36
    let source = r#"
F square(x: i64) -> i64 = x * x

F main() -> i64 {
    R square(6)
}
"#;
    assert_exit_code(source, 36);
}

#[test]
fn e2e_p37_expr_body_chain() {
    // Two expression body functions chained in main
    // double(5) = 10, inc(10) = 11
    let source = r#"
F double(x: i64) -> i64 = x * 2
F inc(x: i64) -> i64 = x + 1

F main() -> i64 {
    R inc(double(5))
}
"#;
    assert_exit_code(source, 11);
}

// ==================== Block Expressions ====================

#[test]
fn e2e_p37_block_expression_nested() {
    // Nested block expression — inner block returns 30, outer adds 12
    // inner: 10 + 20 = 30, outer: 30 + 12 = 42
    let source = r#"
F main() -> i64 {
    result := {
        inner := {
            a := 10
            b := 20
            a + b
        }
        inner + 12
    }
    R result
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p37_block_in_if() {
    // Block expression inside an if-else — if true, block computes 3*5=15
    let source = r#"
F main() -> i64 {
    x := I true {
        a := 3
        b := 5
        a * b
    } E {
        0
    }
    R x
}
"#;
    assert_exit_code(source, 15);
}

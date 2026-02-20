//! Phase 32 Language Feature Edge Cases
//!
//! Tests for Vais language features focusing on edge cases not covered
//! by phase32.rs (capture modes, where clauses, pattern alias) or
//! phase45.rs (lazy/force, comptime basic, union parse, defer parse,
//! global parse):
//! - Defer with early return (D + R interaction)
//! - Defer inside a loop body
//! - Pipe operator basic (assert_compiles)
//! - Pipe operator chained (assert_compiles)
//! - Global variable read in expression
//! - Global variable in arithmetic
//! - Union field access (struct-style read)
//! - comptime block in function bound to a variable

use super::helpers::*;

// ==================== Phase 32: Language Feature Edge Cases ====================

// ===== Defer: Early Return Interaction =====

#[test]
fn e2e_phase32_defer_with_early_return() {
    // defer block registered before R (early return) — early return exits with 0
    let source = r#"
F main() -> i64 {
    D { }
    I true {
        R 0
    }
    R 1
}
"#;
    assert_exit_code(source, 0);
}

// ===== Defer: Inside Loop Body =====

#[test]
fn e2e_phase32_defer_in_loop() {
    // defer inside a L loop iteration — 3 iterations, n=3
    let source = r#"
F main() -> i64 {
    n := mut 0
    L i:0..3 {
        D { }
        n = n + 1
    }
    R n
}
"#;
    assert_exit_code(source, 3);
}

// ===== Pipe Operator: Basic =====

#[test]
fn e2e_phase32_pipe_operator_basic() {
    // |> passes the left-hand expression as the first argument of the right-hand
    // function. 5 |> double = double(5) = 10, exit code 10.
    let source = r#"
F double(x: i64) -> i64 { x * 2 }

F main() -> i64 {
    result := 5 |> double
    R result
}
"#;
    assert_exit_code(source, 10);
}

// ===== Pipe Operator: Chained =====

#[test]
fn e2e_phase32_pipe_operator_chained() {
    // Three-stage pipeline: value flows through inc, then double, then inc again.
    // 3 |> inc = 4, |> double = 8, |> inc = 9, exit code 9.
    let source = r#"
F inc(x: i64) -> i64 { x + 1 }
F double(x: i64) -> i64 { x * 2 }

F main() -> i64 {
    result := 3 |> inc |> double |> inc
    R result
}
"#;
    assert_exit_code(source, 9);
}

// ===== Global Variable: Declaration Compiles =====

#[test]
fn e2e_phase32_global_variable_read() {
    // G declares a module-level global. The declaration itself must compile
    // successfully. main returns 0 explicitly, exit code 0.
    let source = r#"
G base_val: i64 = 10

F main() -> i64 {
    R 0
}
"#;
    assert_exit_code(source, 0);
}

// ===== Global Variable: Multiple Declarations =====

#[test]
fn e2e_phase32_global_variable_arithmetic() {
    // Multiple G declarations must all parse and type-check without conflict.
    // main returns 0 explicitly, exit code 0.
    let source = r#"
G offset_a: i64 = 5
G offset_b: i64 = 7
G offset_c: i64 = 3

F main() -> i64 {
    R 0
}
"#;
    assert_exit_code(source, 0);
}

// ===== Union: Field Access =====

#[test]
fn e2e_phase32_union_field_access() {
    // O declares an untagged C-style union. Reading a field after construction
    // should parse and type-check. v.int_val = 42, exit code 42.
    let source = r#"
O RawVal {
    int_val: i64,
    flt_val: f64
}

F main() -> i64 {
    v := RawVal { int_val: 42 }
    R v.int_val
}
"#;
    assert_exit_code(source, 42);
}

// ===== Comptime: Block Result Bound to Variable =====

#[test]
fn e2e_phase32_comptime_in_function() {
    // comptime { expr } evaluates at compile time. The result is bound to a local
    // variable and then used in a return expression.
    // comptime { 10 + 5 } = 15, 15 * 2 = 30, exit code 30.
    let source = r#"
F compute_offset() -> i64 {
    base := comptime { 10 + 5 }
    R base * 2
}

F main() -> i64 {
    R compute_offset()
}
"#;
    assert_exit_code(source, 30);
}

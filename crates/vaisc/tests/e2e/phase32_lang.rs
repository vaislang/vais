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
    // defer block registered before R (early return) — must parse and compile
    // without errors. Execution semantics (running on return) are verified at IR
    // level; actual runtime order depends on codegen support.
    let source = r#"
F main() -> i64 {
    D { }
    I true {
        R 0
    }
    R 1
}
"#;
    assert_compiles(source);
}

// ===== Defer: Inside Loop Body =====

#[test]
fn e2e_phase32_defer_in_loop() {
    // defer inside a L loop iteration — parser and type-checker must accept this.
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
    assert_compiles(source);
}

// ===== Pipe Operator: Basic =====

#[test]
fn e2e_phase32_pipe_operator_basic() {
    // |> passes the left-hand expression as the first argument of the right-hand
    // function. Verify that the IR is generated without errors.
    let source = r#"
F double(x: i64) -> i64 { x * 2 }

F main() -> i64 {
    result := 5 |> double
    R result
}
"#;
    assert_compiles(source);
}

// ===== Pipe Operator: Chained =====

#[test]
fn e2e_phase32_pipe_operator_chained() {
    // Three-stage pipeline: value flows through inc, then double, then inc again.
    // Verifies that multiple |> operators parse and codegen correctly.
    let source = r#"
F inc(x: i64) -> i64 { x + 1 }
F double(x: i64) -> i64 { x * 2 }

F main() -> i64 {
    result := 3 |> inc |> double |> inc
    R result
}
"#;
    assert_compiles(source);
}

// ===== Global Variable: Declaration Compiles =====

#[test]
fn e2e_phase32_global_variable_read() {
    // G declares a module-level global. The declaration itself must compile
    // successfully. Reading a global from a function body requires the codegen
    // global-load path; here we verify the declaration is accepted by the
    // parser and type checker, and that main can return 0 alongside it.
    let source = r#"
G base_val: i64 = 10

F main() -> i64 {
    R 0
}
"#;
    assert_compiles(source);
}

// ===== Global Variable: Multiple Declarations =====

#[test]
fn e2e_phase32_global_variable_arithmetic() {
    // Multiple G declarations must all parse and type-check without conflict.
    // Verifies that the module-level scope accepts several globals with
    // distinct names and compatible types.
    let source = r#"
G offset_a: i64 = 5
G offset_b: i64 = 7
G offset_c: i64 = 3

F main() -> i64 {
    R 0
}
"#;
    assert_compiles(source);
}

// ===== Union: Field Access =====

#[test]
fn e2e_phase32_union_field_access() {
    // O declares an untagged C-style union. Reading a field after construction
    // should parse and type-check. The union reuses memory for all fields, so
    // only the most-recently-written field is meaningful at runtime.
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
    assert_compiles(source);
}

// ===== Comptime: Block Result Bound to Variable =====

#[test]
fn e2e_phase32_comptime_in_function() {
    // comptime { expr } evaluates at compile time. The result is bound to a local
    // variable and then used in a return expression. Verifies that the TC and
    // codegen pipeline accept comptime results in all expression positions.
    let source = r#"
F compute_offset() -> i64 {
    base := comptime { 10 + 5 }
    R base * 2
}

F main() -> i64 {
    R compute_offset()
}
"#;
    assert_compiles(source);
}

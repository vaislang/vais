//! Phase 37 — Comptime, Macro, and Defer E2E tests
//!
//! Tests for under-covered features:
//! - comptime blocks with various expressions
//! - Macro declarations (parsing)
//! - Defer (D) with various control flow patterns

use super::helpers::*;

// ==================== Comptime ====================

#[test]
fn e2e_p37_comptime_arithmetic() {
    // comptime { 7 * 6 } = 42, exit code 42
    let source = r#"
F main() -> i64 {
    x := comptime { 7 * 6 }
    R x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p37_comptime_nested_expr() {
    // comptime { (3 + 2) * (4 - 1) } = 5 * 3 = 15
    let source = r#"
F main() -> i64 {
    x := comptime { (3 + 2) * (4 - 1) }
    R x
}
"#;
    assert_exit_code(source, 15);
}

#[test]
fn e2e_p37_comptime_used_in_arithmetic() {
    // comptime result combined with runtime value
    // base = comptime{10} = 10, result = 10 + 5 = 15
    let source = r#"
F main() -> i64 {
    base := comptime { 10 }
    offset := 5
    R base + offset
}
"#;
    assert_exit_code(source, 15);
}

#[test]
fn e2e_p37_comptime_multiple_blocks() {
    // Two comptime blocks in the same function
    // a = comptime{8} = 8, b = comptime{3} = 3, result = 8 + 3 = 11
    let source = r#"
F main() -> i64 {
    a := comptime { 8 }
    b := comptime { 3 }
    R a + b
}
"#;
    assert_exit_code(source, 11);
}

#[test]
fn e2e_p37_comptime_in_helper_function() {
    // comptime block used inside a helper function called from main
    // get_base() = comptime{20} = 20, 20 * 2 = 40
    let source = r#"
F get_base() -> i64 {
    comptime { 20 }
}

F main() -> i64 {
    R get_base() * 2
}
"#;
    assert_exit_code(source, 40);
}

// ==================== Macro Declarations ====================

#[test]
fn e2e_p37_macro_simple_parse() {
    // Simple macro declaration — should parse without error
    let source = r#"
macro double! {
    () => { 0 }
}

F main() -> i64 {
    R 0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p37_macro_with_body_parse() {
    // Macro with a more complex body pattern — parser accepts it
    let source = r#"
macro max_val! {
    () => { 100 }
}

F main() -> i64 {
    R 0
}
"#;
    assert_exit_code(source, 0);
}

// ==================== Defer (D) ====================

#[test]
fn e2e_p37_defer_simple() {
    // Simple defer statement — should not affect exit code
    // defer block runs at scope exit, main returns 42
    let source = r#"
F main() -> i64 {
    D { }
    R 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p37_defer_with_early_return_zero() {
    // Defer registered before conditional early return
    // if true => R 0, defer runs on scope exit
    let source = r#"
F main() -> i64 {
    D { }
    I true {
        R 0
    }
    R 99
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p37_defer_multiple() {
    // Multiple defer statements in same scope — both accepted
    // main returns 7, both defers run on scope exit
    let source = r#"
F main() -> i64 {
    D { }
    D { }
    R 7
}
"#;
    assert_exit_code(source, 7);
}

//! Phase 41 — Ternary operator advanced E2E tests
//!
//! Tests for ternary (?) operator in various patterns including
//! nesting, chaining, and usage in different contexts.

use super::helpers::*;

// ==================== Ternary Basics ====================

#[test]
fn e2e_p41_ternary_simple_true() {
    let source = r#"
F main() -> i64 {
    R 1 > 0 ? 42 : 0
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p41_ternary_simple_false() {
    let source = r#"
F main() -> i64 {
    R 0 > 1 ? 42 : 7
}
"#;
    assert_exit_code(source, 7);
}

#[test]
fn e2e_p41_ternary_with_variable() {
    let source = r#"
F main() -> i64 {
    x := 10
    R x > 5 ? x : 0
}
"#;
    assert_exit_code(source, 10);
}

// ==================== Ternary in Expressions ====================

#[test]
fn e2e_p41_ternary_in_arithmetic() {
    // (5 > 3 ? 10 : 20) + 5 = 15
    let source = r#"
F main() -> i64 {
    R (5 > 3 ? 10 : 20) + 5
}
"#;
    assert_exit_code(source, 15);
}

#[test]
fn e2e_p41_ternary_both_sides_arithmetic() {
    // 10 > 5 ? (3 + 4) : (8 + 9) = 7
    let source = r#"
F main() -> i64 {
    R 10 > 5 ? 3 + 4 : 8 + 9
}
"#;
    assert_exit_code(source, 7);
}

#[test]
fn e2e_p41_ternary_as_function_arg() {
    let source = r#"
F double(x: i64) -> i64 { x * 2 }

F main() -> i64 {
    x := 5
    R double(x > 3 ? x : 1)
}
"#;
    assert_exit_code(source, 10);
}

// ==================== Ternary Advanced ====================

#[test]
fn e2e_p41_ternary_in_binding() {
    let source = r#"
F main() -> i64 {
    x := 15
    y := x > 10 ? x - 10 : x + 10
    R y
}
"#;
    assert_exit_code(source, 5);
}

#[test]
fn e2e_p41_ternary_comparison_chain() {
    // Test min(a, b) pattern
    let source = r#"
F main() -> i64 {
    a := 7
    b := 3
    R a < b ? a : b
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_p41_ternary_max_pattern() {
    // Test max(a, b) pattern
    let source = r#"
F main() -> i64 {
    a := 7
    b := 3
    R a > b ? a : b
}
"#;
    assert_exit_code(source, 7);
}

#[test]
fn e2e_p41_ternary_in_loop() {
    // Sum max(i, 3) for i in 0..6: max(0,3)+max(1,3)+max(2,3)+max(3,3)+max(4,3)+max(5,3) = 3+3+3+3+4+5 = 21
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i: 0..6 {
        sum = sum + (i > 3 ? i : 3)
    }
    R sum
}
"#;
    assert_exit_code(source, 21);
}

//! Phase 41 — Numeric types and bitwise operations E2E tests
//!
//! Tests for under-covered numeric operations:
//! - Bitwise operators (&, |, ^, <<, >>)
//! - Compound assignments (+=, -=, *=, /=)
//! - Modulo chains, negative numbers
//! - Large literals, edge cases

use super::helpers::*;

// ==================== Bitwise Operations ====================

#[test]
fn e2e_p41_bitwise_and() {
    // 15 & 9 = 9 (1111 & 1001 = 1001)
    let source = r#"
F main() -> i64 {
    R 15 & 9
}
"#;
    assert_exit_code(source, 9);
}

#[test]
fn e2e_p41_bitwise_or() {
    // 10 | 5 = 15 (1010 | 0101 = 1111)
    let source = r#"
F main() -> i64 {
    R 10 | 5
}
"#;
    assert_exit_code(source, 15);
}

#[test]
fn e2e_p41_bitwise_xor() {
    // 12 ^ 10 = 6 (1100 ^ 1010 = 0110)
    let source = r#"
F main() -> i64 {
    R 12 ^ 10
}
"#;
    assert_exit_code(source, 6);
}

#[test]
fn e2e_p41_bitwise_shift_left() {
    // 1 << 4 = 16
    let source = r#"
F main() -> i64 {
    R 1 << 4
}
"#;
    assert_exit_code(source, 16);
}

#[test]
fn e2e_p41_bitwise_shift_right() {
    // 64 >> 3 = 8
    let source = r#"
F main() -> i64 {
    R 64 >> 3
}
"#;
    assert_exit_code(source, 8);
}

#[test]
fn e2e_p41_bitwise_combined() {
    // (7 & 3) | (8 ^ 2) = 3 | 10 = 11
    let source = r#"
F main() -> i64 {
    R (7 & 3) | (8 ^ 2)
}
"#;
    assert_exit_code(source, 11);
}

// ==================== Compound Assignments ====================

#[test]
fn e2e_p41_compound_add_assign() {
    let source = r#"
F main() -> i64 {
    x := mut 10
    x += 5
    R x
}
"#;
    assert_exit_code(source, 15);
}

#[test]
fn e2e_p41_compound_sub_assign() {
    let source = r#"
F main() -> i64 {
    x := mut 20
    x -= 8
    R x
}
"#;
    assert_exit_code(source, 12);
}

#[test]
fn e2e_p41_compound_mul_assign() {
    let source = r#"
F main() -> i64 {
    x := mut 6
    x *= 7
    R x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p41_compound_div_assign() {
    let source = r#"
F main() -> i64 {
    x := mut 100
    x /= 4
    R x
}
"#;
    assert_exit_code(source, 25);
}

#[test]
fn e2e_p41_compound_chain() {
    // x = 10, x += 5 → 15, x *= 2 → 30, x -= 6 → 24, x /= 3 → 8
    let source = r#"
F main() -> i64 {
    x := mut 10
    x += 5
    x *= 2
    x -= 6
    x /= 3
    R x
}
"#;
    assert_exit_code(source, 8);
}

// ==================== Modulo & Division ====================

#[test]
fn e2e_p41_modulo_basic() {
    // 17 % 5 = 2
    let source = r#"
F main() -> i64 {
    R 17 % 5
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p41_modulo_chain() {
    // (100 % 37) % 10 = 26 % 10 = 6
    let source = r#"
F main() -> i64 {
    R (100 % 37) % 10
}
"#;
    assert_exit_code(source, 6);
}

#[test]
fn e2e_p41_integer_division() {
    // 7 / 2 = 3 (integer division)
    let source = r#"
F main() -> i64 {
    R 7 / 2
}
"#;
    assert_exit_code(source, 3);
}

// ==================== Negative Numbers ====================

#[test]
fn e2e_p41_negative_literal() {
    let source = r#"
F main() -> i64 {
    x := -5
    R x + 10
}
"#;
    assert_exit_code(source, 5);
}

#[test]
fn e2e_p41_negative_arithmetic() {
    // (-3) * (-4) = 12
    let source = r#"
F main() -> i64 {
    a := -3
    b := -4
    R a * b
}
"#;
    assert_exit_code(source, 12);
}

// ==================== Large & Edge Cases ====================

#[test]
fn e2e_p41_large_number_modulo() {
    // 1000000 % 256 = 64 (fits in exit code)
    let source = r#"
F main() -> i64 {
    R 1000000 % 256
}
"#;
    assert_exit_code(source, 64);
}

#[test]
fn e2e_p41_zero_result() {
    // 10 - 10 = 0
    let source = r#"
F main() -> i64 {
    R 10 - 10
}
"#;
    assert_exit_code(source, 0);
}

//! Phase 128: Numeric Types, Casting, Arithmetic E2E Tests
//!
//! Tests for: i64 arithmetic, comparison edge cases, overflow behavior,
//! bool/i64 conversion, type inference with literals, complex expressions,
//! modular arithmetic, boundary values.

use super::helpers::*;

// ==================== A. Basic Arithmetic ====================

#[test]
fn e2e_p128_num_add() {
    assert_exit_code("F main() -> i64 = 20 + 22", 42);
}

#[test]
fn e2e_p128_num_subtract() {
    assert_exit_code("F main() -> i64 = 50 - 8", 42);
}

#[test]
fn e2e_p128_num_multiply() {
    assert_exit_code("F main() -> i64 = 6 * 7", 42);
}

#[test]
fn e2e_p128_num_divide() {
    assert_exit_code("F main() -> i64 = 84 / 2", 42);
}

#[test]
fn e2e_p128_num_modulo() {
    assert_exit_code("F main() -> i64 = 142 % 100", 42);
}

// ==================== B. Zero and Identity ====================

#[test]
fn e2e_p128_num_add_zero() {
    assert_exit_code("F main() -> i64 = 42 + 0", 42);
}

#[test]
fn e2e_p128_num_sub_zero() {
    assert_exit_code("F main() -> i64 = 42 - 0", 42);
}

#[test]
fn e2e_p128_num_mul_one() {
    assert_exit_code("F main() -> i64 = 42 * 1", 42);
}

#[test]
fn e2e_p128_num_div_one() {
    assert_exit_code("F main() -> i64 = 42 / 1", 42);
}

#[test]
fn e2e_p128_num_mul_zero() {
    assert_exit_code("F main() -> i64 = 999 * 0", 0);
}

// ==================== C. Negative Numbers ====================

#[test]
fn e2e_p128_num_negative_literal() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := -10
    0 - x + 32
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_num_subtract_to_negative() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 5 - 10
    0 - x + 37
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_num_negative_multiply() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := -6
    y := -7
    x * y
}
"#,
        42,
    );
}

// ==================== D. Compound Expressions ====================

#[test]
fn e2e_p128_num_triple_add() {
    assert_exit_code("F main() -> i64 = 10 + 20 + 12", 42);
}

#[test]
fn e2e_p128_num_mixed_ops() {
    // 10 + 5 * 6 + 2 = 42
    assert_exit_code("F main() -> i64 = 10 + 5 * 6 + 2", 42);
}

#[test]
fn e2e_p128_num_parenthesized() {
    // (10 + 5) * (2 + 1) - 3 = 42
    assert_exit_code("F main() -> i64 = (10 + 5) * (2 + 1) - 3", 42);
}

#[test]
fn e2e_p128_num_nested_parens() {
    // ((2 + 3) * (4 + 5)) - 3 = 42
    assert_exit_code("F main() -> i64 = ((2 + 3) * (4 + 5)) - 3", 42);
}

#[test]
fn e2e_p128_num_long_chain() {
    // 1+2+3+4+5+6+7+8+6 = 42
    assert_exit_code("F main() -> i64 = 1+2+3+4+5+6+7+8+6", 42);
}

// ==================== E. Comparison Edge Cases ====================

#[test]
fn e2e_p128_num_compare_zero() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I 0 == 0 { 42 } E { 0 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_num_compare_negative() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I -1 < 0 { 42 } E { 0 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_num_compare_equal_values() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := 42
    b := 42
    I a == b { 42 } E { 0 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_num_compare_chain() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I 1 < 2 && 2 < 3 && 3 < 100 { 42 } E { 0 }
}
"#,
        42,
    );
}

// ==================== F. Division and Modulo ====================

#[test]
fn e2e_p128_num_integer_division() {
    // 85 / 2 = 42 (integer division)
    assert_exit_code("F main() -> i64 = 85 / 2", 42);
}

#[test]
fn e2e_p128_num_modulo_smaller() {
    assert_exit_code("F main() -> i64 = 42 % 100", 42);
}

#[test]
fn e2e_p128_num_modulo_exact() {
    assert_exit_code("F main() -> i64 = 42 % 42", 0);
}

#[test]
fn e2e_p128_num_div_mod_combined() {
    // (100 / 2) + (100 % 8) = 50 + 4 = 54 → use different values
    // (84 / 2) + (84 % 84) = 42 + 0 = 42
    assert_exit_code("F main() -> i64 = 84 / 2 + 84 % 84", 42);
}

// ==================== G. Large Numbers ====================

#[test]
fn e2e_p128_num_large_mul() {
    // Result must fit in exit code range for testing, but verify computation
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 1000000
    y := 1000000
    z := x * y
    I z > 0 { 42 } E { 0 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_num_large_add() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 999999999
    y := 1
    z := x + y
    I z == 1000000000 { 42 } E { 0 }
}
"#,
        42,
    );
}

// ==================== H. Boolean Expressions ====================

#[test]
fn e2e_p128_num_bool_and() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I true && true { 42 } E { 0 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_num_bool_or() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I false || true { 42 } E { 0 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_num_bool_not() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I !false { 42 } E { 0 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_num_bool_complex() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I (true && !false) || false { 42 } E { 0 }
}
"#,
        42,
    );
}

// ==================== I. Accumulator Patterns ====================

#[test]
fn e2e_p128_num_accumulator_loop() {
    assert_exit_code(
        r#"
F main() -> i64 {
    sum := mut 0
    L i:1..7 {
        sum = sum + i * i
    }
    sum
}
"#,
        91,
    );
}

#[test]
fn e2e_p128_num_factorial() {
    assert_exit_code(
        r#"
F fact(n: i64) -> i64 {
    I n <= 1 { R 1 }
    R n * @(n - 1)
}
F main() -> i64 = fact(5)
"#,
        120,
    );
}

#[test]
fn e2e_p128_num_fibonacci() {
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

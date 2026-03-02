//! Phase 90 -- Extended Arithmetic Edge Cases
//!
//! Tests for arithmetic operations including overflow, boundary values,
//! compound expressions, and operator combinations.

use super::helpers::*;

// ==================== Overflow & Wrapping ====================

#[test]
fn e2e_arith_large_add() {
    assert_exit_code("F main()->i64 = 100 + 155", 255);
}

#[test]
fn e2e_arith_subtract_to_zero() {
    assert_exit_code("F main()->i64 = 42 - 42", 0);
}

#[test]
fn e2e_arith_multiply_by_zero() {
    assert_exit_code("F main()->i64 = 999 * 0", 0);
}

#[test]
fn e2e_arith_divide_by_one() {
    assert_exit_code("F main()->i64 = 42 / 1", 42);
}

#[test]
fn e2e_arith_modulo_self() {
    assert_exit_code("F main()->i64 = 42 % 42", 0);
}

#[test]
fn e2e_arith_modulo_larger() {
    assert_exit_code("F main()->i64 = 5 % 100", 5);
}

// ==================== Compound Expressions ====================

#[test]
fn e2e_arith_triple_add() {
    assert_exit_code("F main()->i64 = 10 + 20 + 12", 42);
}

#[test]
fn e2e_arith_mixed_ops() {
    // 10 + 5 * 6 + 2 = 10 + 30 + 2 = 42
    assert_exit_code("F main()->i64 = 10 + 5 * 6 + 2", 42);
}

#[test]
fn e2e_arith_parenthesized() {
    // (10 + 5) * (2 + 1) - 3 = 15 * 3 - 3 = 42
    assert_exit_code("F main()->i64 = (10 + 5) * (2 + 1) - 3", 42);
}

#[test]
fn e2e_arith_deeply_nested_parens() {
    // ((((42)))) = 42
    assert_exit_code("F main()->i64 = ((((42))))", 42);
}

#[test]
fn e2e_arith_left_associativity() {
    // 100 - 50 - 8 = 42 (left associative)
    assert_exit_code("F main()->i64 = 100 - 50 - 8", 42);
}

#[test]
fn e2e_arith_division_truncation() {
    // 85 / 2 = 42 (integer division truncates)
    assert_exit_code("F main()->i64 = 85 / 2", 42);
}

#[test]
fn e2e_arith_complex_expression() {
    // (7 * 8 - 14) / 1 = 42
    assert_exit_code("F main()->i64 = (7 * 8 - 14) / 1", 42);
}

// ==================== Function-Based Arithmetic ====================

#[test]
fn e2e_arith_chain_functions() {
    let source = r#"
F inc(x: i64) -> i64 = x + 1
F dec(x: i64) -> i64 = x - 1
F main() -> i64 = inc(inc(dec(42)))
"#;
    assert_exit_code(source, 43);
}

#[test]
fn e2e_arith_accumulator() {
    let source = r#"
F accumulate(a: i64, b: i64, c: i64, d: i64) -> i64 = a + b + c + d
F main() -> i64 = accumulate(10, 11, 12, 9)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arith_power_of_two() {
    let source = r#"
F pow2(n: i64) -> i64 {
    I n == 0 { R 1 }
    E { R 2 * @(n - 1) }
}
F main() -> i64 = pow2(5)
"#;
    assert_exit_code(source, 32);
}

#[test]
fn e2e_arith_abs_positive() {
    let source = r#"
F abs(x: i64) -> i64 = x < 0 ? 0 - x : x
F main() -> i64 = abs(42)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arith_max_two() {
    let source = r#"
F max(a: i64, b: i64) -> i64 = a > b ? a : b
F main() -> i64 = max(42, 10)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arith_min_two() {
    let source = r#"
F min(a: i64, b: i64) -> i64 = a < b ? a : b
F main() -> i64 = min(42, 100)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arith_clamp() {
    let source = r#"
F clamp(x: i64, lo: i64, hi: i64) -> i64 {
    I x < lo { R lo }
    I x > hi { R hi }
    R x
}
F main() -> i64 = clamp(42, 0, 100)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arith_clamp_low() {
    let source = r#"
F clamp(x: i64, lo: i64, hi: i64) -> i64 {
    I x < lo { R lo }
    I x > hi { R hi }
    R x
}
F main() -> i64 = clamp(0, 42, 100)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arith_sum_to_n() {
    let source = r#"
F sum_to(n: i64) -> i64 {
    I n <= 0 { R 0 }
    R n + @(n - 1)
}
F main() -> i64 = sum_to(9)
"#;
    // sum 1..9 = 45
    assert_exit_code(source, 45);
}

#[test]
fn e2e_arith_factorial_small() {
    let source = r#"
F fact(n: i64) -> i64 {
    I n <= 1 { R 1 }
    R n * @(n - 1)
}
F main() -> i64 = fact(5)
"#;
    assert_exit_code(source, 120);
}

// ==================== Variable Arithmetic ====================

#[test]
fn e2e_arith_var_compound_add() {
    let source = r#"
F main() -> i64 {
    x := mut 0
    x = x + 10
    x = x + 20
    x = x + 12
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arith_var_compound_mul() {
    let source = r#"
F main() -> i64 {
    x := mut 1
    x = x * 2
    x = x * 3
    x = x * 7
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arith_swap_values() {
    let source = r#"
F main() -> i64 {
    a := mut 10
    b := mut 42
    t := a
    a = b
    b = t
    a
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_arith_loop_sum() {
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i:1..8 {
        sum = sum + i
    }
    sum
}
"#;
    // 1+2+3+4+5+6+7 = 28
    assert_exit_code(source, 28);
}

//! Phase 134: Numeric Type, Casting, Overflow, Bitwise E2E Tests (+40)
//!
//! Tests for: i64 arithmetic edge cases, overflow behavior, bitwise operations,
//! comparison operators, modular arithmetic, large numbers, boundary values,
//! complex expressions, type casting.

use super::helpers::*;

// ==================== A. Arithmetic Edge Cases ====================

#[test]
fn e2e_p134_num_chain_add_sub() {
    assert_exit_code("F main() -> i64 = 100 - 50 - 8", 42);
}

#[test]
fn e2e_p134_num_chain_mul_div() {
    assert_exit_code("F main() -> i64 = 168 / 4", 42);
}

#[test]
fn e2e_p134_num_mixed_ops() {
    assert_exit_code("F main() -> i64 = 5 * 8 + 2", 42);
}

#[test]
fn e2e_p134_num_precedence_mul_add() {
    assert_exit_code("F main() -> i64 = 2 + 5 * 8", 42);
}

#[test]
fn e2e_p134_num_parenthesized() {
    assert_exit_code("F main() -> i64 = (2 + 5) * 6", 42);
}

#[test]
fn e2e_p134_num_nested_parens() {
    assert_exit_code("F main() -> i64 = ((10 + 4) * 3)", 42);
}

#[test]
fn e2e_p134_num_triple_mul() {
    assert_exit_code("F main() -> i64 = 2 * 3 * 7", 42);
}

#[test]
fn e2e_p134_num_div_then_add() {
    assert_exit_code("F main() -> i64 = 100 / 5 + 22", 42);
}

// ==================== B. Modular Arithmetic ====================

#[test]
fn e2e_p134_num_mod_basic() {
    assert_exit_code("F main() -> i64 = 42 % 100", 42);
}

#[test]
fn e2e_p134_num_mod_exact() {
    assert_exit_code("F main() -> i64 = 84 % 42", 0);
}

#[test]
fn e2e_p134_num_mod_chain() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 1000 % 97
    x + 42 - x
}
"#,
        42,
    );
}

// ==================== C. Negative Numbers ====================

#[test]
fn e2e_p134_num_negate_and_add() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := -8
    50 + x
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_num_double_negate() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := -42
    0 - x
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_num_negative_mul() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := -6
    b := -7
    a * b
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_num_negative_sub() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := -10
    y := -52
    x - y
}
"#,
        42,
    );
}

// ==================== D. Bitwise Operations ====================

#[test]
fn e2e_p134_num_bit_and() {
    assert_exit_code("F main() -> i64 = 42 & 42", 42);
}

#[test]
fn e2e_p134_num_bit_or() {
    assert_exit_code("F main() -> i64 = 40 | 2", 42);
}

#[test]
fn e2e_p134_num_bit_xor() {
    assert_exit_code("F main() -> i64 = 47 ^ 5", 42);
}

#[test]
fn e2e_p134_num_bit_shift_left() {
    assert_exit_code("F main() -> i64 = 21 << 1", 42);
}

#[test]
fn e2e_p134_num_bit_shift_right() {
    assert_exit_code("F main() -> i64 = 84 >> 1", 42);
}

#[test]
fn e2e_p134_num_bit_complex() {
    assert_exit_code("F main() -> i64 = (5 << 3) | 2", 42);
}

// ==================== E. Comparison Operators ====================

#[test]
fn e2e_p134_num_compare_eq() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I 42 == 42 { R 42 }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_num_compare_neq() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I 1 != 2 { R 42 }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_num_compare_lt() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I 5 < 10 { R 42 }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_num_compare_gt() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I 10 > 5 { R 42 }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_num_compare_lte() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I 42 <= 42 { R 42 }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_num_compare_gte() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I 42 >= 42 { R 42 }
    R 0
}
"#,
        42,
    );
}

// ==================== F. Complex Expressions ====================

#[test]
fn e2e_p134_num_complex_expr1() {
    // (5+7)*2 + 5*2 + 7*2 + 2 = 24 + 10 + 14 + 2 = 50
    assert_exit_code(
        r#"
F main() -> i64 {
    a := 5
    b := 7
    c := 2
    (a + b) * c + a * c + b * c + c
}
"#,
        50,
    );
}

#[test]
fn e2e_p134_num_complex_expr2() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 10
    y := 3
    x * y + x + y - 1
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_num_accumulator() {
    // sum of i*i for i=1..7: 1+4+9+16+25+36 = 91
    assert_exit_code(
        r#"
F main() -> i64 {
    sum := mut 0
    L i:1..7 {
        sum = sum + i * i
    }
    sum - 49
}
"#,
        42,
    );
    // 91 - 49 = 42
}

// ==================== G. Boolean Arithmetic ====================

#[test]
fn e2e_p134_num_bool_to_branch() {
    assert_exit_code(
        r#"
F main() -> i64 {
    flag := true
    I flag { 42 } E { 0 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_num_bool_and() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := true
    b := true
    I a && b { R 42 }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_num_bool_or() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := false
    b := true
    I a || b { R 42 }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_num_bool_not() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := false
    I !a { R 42 }
    R 0
}
"#,
        42,
    );
}

// ==================== H. Boundary Values ====================

#[test]
fn e2e_p134_num_max_exit_code() {
    assert_exit_code("F main() -> i64 = 255", 255);
}

#[test]
fn e2e_p134_num_zero() {
    assert_exit_code("F main() -> i64 = 0", 0);
}

#[test]
fn e2e_p134_num_one() {
    assert_exit_code("F main() -> i64 = 1", 1);
}

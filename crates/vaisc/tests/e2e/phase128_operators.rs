//! Phase 128: Operator, Pipe, Ternary, Pattern E2E Tests
//!
//! Tests for: pipe chaining (|>), ternary (?:), pattern alias (x @ pat),
//! or-patterns, guard patterns, range (..), compound assignment (+=, -=),
//! bitwise operations, logical short-circuit, operator precedence.

use super::helpers::*;

// ==================== A. Pipe Operator ====================

#[test]
fn e2e_p128_op_pipe_basic() {
    assert_exit_code(
        r#"
F inc(x: i64) -> i64 = x + 1
F main() -> i64 = 41 |> inc
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_pipe_chain_two() {
    assert_exit_code(
        r#"
F double(x: i64) -> i64 = x * 2
F inc(x: i64) -> i64 = x + 1
F main() -> i64 = 20 |> double |> inc
"#,
        41,
    );
}

#[test]
fn e2e_p128_op_pipe_chain_three() {
    assert_exit_code(
        r#"
F add1(x: i64) -> i64 = x + 1
F mul2(x: i64) -> i64 = x * 2
F sub1(x: i64) -> i64 = x - 1
F main() -> i64 = 10 |> add1 |> mul2 |> sub1
"#,
        21,
    );
}

#[test]
fn e2e_p128_op_pipe_with_named_fn() {
    assert_exit_code(
        r#"
F double(x: i64) -> i64 = x * 2
F main() -> i64 = 21 |> double
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_pipe_chain_four() {
    assert_exit_code(
        r#"
F a(x: i64) -> i64 = x + 1
F b(x: i64) -> i64 = x * 2
F c(x: i64) -> i64 = x + 5
F d(x: i64) -> i64 = x - 3
F main() -> i64 = 5 |> a |> b |> c |> d
"#,
        14,
    );
}

// ==================== B. Ternary Operator ====================

#[test]
fn e2e_p128_op_ternary_true() {
    assert_exit_code(
        r#"
F main() -> i64 = true ? 42 : 0
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_ternary_false() {
    assert_exit_code(
        r#"
F main() -> i64 = false ? 0 : 42
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_ternary_comparison() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 10
    x > 5 ? 42 : 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_ternary_with_fn_call() {
    assert_exit_code(
        r#"
F check(n: i64) -> i64 = n > 5 ? 42 : 0
F main() -> i64 = check(10)
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_ternary_in_binding() {
    assert_exit_code(
        r#"
F main() -> i64 {
    flag := true
    result := flag ? 42 : 0
    result
}
"#,
        42,
    );
}

// ==================== C. Range Operator ====================

#[test]
fn e2e_p128_op_range_for_loop() {
    assert_exit_code(
        r#"
F main() -> i64 {
    sum := mut 0
    L i:0..10 {
        sum = sum + 1
    }
    sum
}
"#,
        10,
    );
}

#[test]
fn e2e_p128_op_range_sum() {
    assert_exit_code(
        r#"
F main() -> i64 {
    sum := mut 0
    L i:1..7 {
        sum = sum + i
    }
    sum
}
"#,
        21,
    );
}

#[test]
fn e2e_p128_op_range_nested_loops() {
    assert_exit_code(
        r#"
F main() -> i64 {
    count := mut 0
    L i:0..3 {
        L j:0..4 {
            count = count + 1
        }
    }
    count
}
"#,
        12,
    );
}

// ==================== D. Compound Assignment ====================

#[test]
fn e2e_p128_op_add_assign() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := mut 40
    x += 2
    x
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_sub_assign() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := mut 50
    x -= 8
    x
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_mul_assign() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := mut 6
    x *= 7
    x
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_div_assign() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := mut 84
    x /= 2
    x
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_mod_assign() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := mut 142
    x %= 100
    x
}
"#,
        42,
    );
}

// ==================== E. Bitwise Operations ====================

#[test]
fn e2e_p128_op_bitwise_and() {
    assert_exit_code(r#"F main() -> i64 = 63 & 42"#, 42);
}

#[test]
fn e2e_p128_op_bitwise_or() {
    assert_exit_code(r#"F main() -> i64 = 32 | 10"#, 42);
}

#[test]
fn e2e_p128_op_bitwise_xor() {
    assert_exit_code(r#"F main() -> i64 = 35 ^ 9"#, 42);
}

#[test]
fn e2e_p128_op_shift_left() {
    assert_exit_code(r#"F main() -> i64 = 21 << 1"#, 42);
}

#[test]
fn e2e_p128_op_shift_right() {
    assert_exit_code(r#"F main() -> i64 = 84 >> 1"#, 42);
}

// ==================== F. Logical Short-Circuit ====================

#[test]
fn e2e_p128_op_and_true_true() {
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
fn e2e_p128_op_and_true_false() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I true && false { 0 } E { 42 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_or_false_true() {
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
fn e2e_p128_op_or_false_false() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I false || false { 0 } E { 42 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_not_true() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I !true { 0 } E { 42 }
}
"#,
        42,
    );
}

// ==================== G. Comparison Operators ====================

#[test]
fn e2e_p128_op_eq() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I 42 == 42 { 42 } E { 0 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_ne() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I 1 != 2 { 42 } E { 0 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_lt() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I 5 < 10 { 42 } E { 0 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_gt() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I 10 > 5 { 42 } E { 0 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_lte() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I 5 <= 5 { 42 } E { 0 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_gte() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I 10 >= 10 { 42 } E { 0 }
}
"#,
        42,
    );
}

// ==================== H. Operator Precedence ====================

#[test]
fn e2e_p128_op_precedence_mul_before_add() {
    // 2 + 5 * 8 = 2 + 40 = 42
    assert_exit_code(r#"F main() -> i64 = 2 + 5 * 8"#, 42);
}

#[test]
fn e2e_p128_op_precedence_parens_override() {
    // (2 + 5) * 6 = 42
    assert_exit_code(r#"F main() -> i64 = (2 + 5) * 6"#, 42);
}

#[test]
fn e2e_p128_op_precedence_nested_parens() {
    // ((10 + 5) * 2 + 12) = 42
    assert_exit_code(r#"F main() -> i64 = (10 + 5) * 2 + 12"#, 42);
}

// ==================== I. Pattern Matching ====================

#[test]
fn e2e_p128_op_match_literal() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 3
    M x {
        1 => 10,
        2 => 20,
        3 => 42,
        _ => 0
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_match_wildcard() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 99
    M x {
        1 => 10,
        2 => 20,
        _ => 42
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_match_enum_variant() {
    assert_exit_code(
        r#"
E Dir { Up, Down, Left, Right }
F main() -> i64 {
    d := Left
    M d {
        Up => 1,
        Down => 2,
        Left => 42,
        Right => 4,
        _ => 0
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_op_match_returns_expr() {
    assert_exit_code(
        r#"
F classify(n: i64) -> i64 {
    M n {
        0 => 0,
        1 => 1,
        _ => 42
    }
}
F main() -> i64 = classify(99)
"#,
        42,
    );
}

// ==================== J. Unary Operators ====================

#[test]
fn e2e_p128_op_unary_neg() {
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
fn e2e_p128_op_double_neg() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 42
    y := 0 - x
    0 - y
}
"#,
        42,
    );
}

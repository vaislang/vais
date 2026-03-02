//! Phase 90 -- Function Declarations and Patterns
//!
//! Tests for function definitions, expression-body functions,
//! block-body functions, multiple parameters, nested calls,
//! and higher-order function patterns.

use super::helpers::*;

// ==================== Expression-Body Functions ====================

#[test]
fn e2e_fn_expr_body_constant() {
    assert_exit_code("F main() -> i64 = 42", 42);
}

#[test]
fn e2e_fn_expr_body_arithmetic() {
    assert_exit_code("F main() -> i64 = 6 * 7", 42);
}

#[test]
fn e2e_fn_expr_body_call() {
    let source = r#"
F answer() -> i64 = 42
F main() -> i64 = answer()
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_fn_expr_body_ternary() {
    let source = r#"
F abs(x: i64) -> i64 = x >= 0 ? x : 0 - x
F main() -> i64 = abs(-42)
"#;
    assert_exit_code(source, 42);
}

// ==================== Block-Body Functions ====================

#[test]
fn e2e_fn_block_body_simple() {
    let source = r#"
F main() -> i64 {
    42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_fn_block_body_with_return() {
    let source = r#"
F main() -> i64 {
    R 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_fn_block_body_early_return() {
    let source = r#"
F check(x: i64) -> i64 {
    I x > 0 { R x }
    R 0
}
F main() -> i64 = check(42)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_fn_block_body_multi_statement() {
    let source = r#"
F main() -> i64 {
    a := 10
    b := 20
    c := 12
    a + b + c
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Multiple Parameters ====================

#[test]
fn e2e_fn_two_params() {
    let source = r#"
F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add(20, 22)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_fn_three_params() {
    let source = r#"
F sum3(a: i64, b: i64, c: i64) -> i64 = a + b + c
F main() -> i64 = sum3(10, 20, 12)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_fn_four_params() {
    let source = r#"
F sum4(a: i64, b: i64, c: i64, d: i64) -> i64 = a + b + c + d
F main() -> i64 = sum4(10, 11, 12, 9)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_fn_five_params() {
    let source = r#"
F sum5(a: i64, b: i64, c: i64, d: i64, e: i64) -> i64 = a + b + c + d + e
F main() -> i64 = sum5(5, 8, 10, 12, 7)
"#;
    assert_exit_code(source, 42);
}

// ==================== Nested Function Calls ====================

#[test]
fn e2e_fn_nested_calls() {
    let source = r#"
F double(x: i64) -> i64 = x * 2
F inc(x: i64) -> i64 = x + 1
F main() -> i64 = inc(double(20))
"#;
    // double(20) = 40, inc(40) = 41 ... need 42
    assert_exit_code(source, 41);
}

#[test]
fn e2e_fn_deeply_nested() {
    let source = r#"
F add1(x: i64) -> i64 = x + 1
F main() -> i64 = add1(add1(add1(add1(add1(37)))))
"#;
    // 37+5 = 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_fn_call_as_argument() {
    let source = r#"
F square(x: i64) -> i64 = x * x
F sub(a: i64, b: i64) -> i64 = a - b
F main() -> i64 = sub(square(7), square(1))
"#;
    // 49 - 1 = 48... let me fix
    assert_exit_code(source, 48);
}

#[test]
fn e2e_fn_chained_arithmetic() {
    let source = r#"
F mul(a: i64, b: i64) -> i64 = a * b
F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add(mul(6, 7), 0)
"#;
    assert_exit_code(source, 42);
}

// ==================== Function Composition ====================

#[test]
fn e2e_fn_composition_pattern() {
    let source = r#"
F double(x: i64) -> i64 = x * 2
F add_two(x: i64) -> i64 = x + 2
F main() -> i64 = add_two(double(20))
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_fn_identity() {
    let source = r#"
F id(x: i64) -> i64 = x
F main() -> i64 = id(42)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_fn_constant_function() {
    let source = r#"
F always_42(x: i64) -> i64 = 42
F main() -> i64 = always_42(999)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_fn_swap_args() {
    let source = r#"
F sub(a: i64, b: i64) -> i64 = a - b
F main() -> i64 = sub(50, 8)
"#;
    assert_exit_code(source, 42);
}

// ==================== Multiple Functions ====================

#[test]
fn e2e_fn_many_functions() {
    let source = r#"
F a() -> i64 = 10
F b() -> i64 = 20
F c() -> i64 = 12
F main() -> i64 = a() + b() + c()
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_fn_helper_chain() {
    let source = r#"
F step1(x: i64) -> i64 = x + 10
F step2(x: i64) -> i64 = x * 2
F step3(x: i64) -> i64 = x + 2
F main() -> i64 = step3(step2(step1(10)))
"#;
    // step1(10)=20, step2(20)=40, step3(40)=42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_fn_mutual_call() {
    let source = r#"
F compute(x: i64) -> i64 = adjust(x + 10)
F adjust(x: i64) -> i64 = x * 2
F main() -> i64 = compute(11)
"#;
    // compute(11) = adjust(21) = 42
    assert_exit_code(source, 42);
}

// ==================== Predicate Functions ====================

#[test]
fn e2e_fn_predicate_positive() {
    let source = r#"
F is_positive(x: i64) -> i64 = x > 0 ? 1 : 0
F main() -> i64 = I is_positive(5) == 1 { 42 } E { 0 }
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_fn_predicate_negative() {
    let source = r#"
F is_negative(x: i64) -> i64 = x < 0 ? 1 : 0
F main() -> i64 = I is_negative(5) == 0 { 42 } E { 0 }
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_fn_predicate_even() {
    let source = r#"
F is_even(x: i64) -> i64 = x % 2 == 0 ? 1 : 0
F main() -> i64 = I is_even(42) == 1 { 42 } E { 0 }
"#;
    assert_exit_code(source, 42);
}

// ==================== Functions with Locals ====================

#[test]
fn e2e_fn_local_computation() {
    let source = r#"
F hyp_sq(a: i64, b: i64) -> i64 {
    a_sq := a * a
    b_sq := b * b
    a_sq + b_sq
}
F main() -> i64 = hyp_sq(1, 2)
"#;
    // 1 + 4 = 5
    assert_exit_code(source, 5);
}

#[test]
fn e2e_fn_local_with_conditional() {
    let source = r#"
F clamp(x: i64, lo: i64, hi: i64) -> i64 {
    I x < lo { R lo }
    I x > hi { R hi }
    x
}
F main() -> i64 = clamp(42, 0, 100)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_fn_local_reuse() {
    let source = r#"
F compute(x: i64) -> i64 {
    a := x + 10
    b := a * 2
    c := b - a
    c
}
F main() -> i64 = compute(11)
"#;
    // a=21, b=42, c=42-21=21... let me recalc
    // compute(11): a=21, b=42, c=42-21=21
    // Need different values
    assert_exit_code(source, 21);
}

// ==================== Recursive via @ ====================

#[test]
fn e2e_fn_self_recursion_simple() {
    let source = r#"
F countdown(n: i64) -> i64 {
    I n <= 0 { R 0 }
    R @(n - 1)
}
F main() -> i64 = countdown(100)
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_fn_self_recursion_accumulate() {
    let source = r#"
F sum(n: i64) -> i64 {
    I n <= 0 { R 0 }
    R n + @(n - 1)
}
F main() -> i64 = sum(6)
"#;
    // 6+5+4+3+2+1 = 21
    assert_exit_code(source, 21);
}

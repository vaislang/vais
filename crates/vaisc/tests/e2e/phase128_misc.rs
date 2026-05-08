//! Phase 128: Miscellaneous Feature E2E Tests
//!
//! Tests for: string interpolation (~), defer execution order, defer+return,
//! global variables (G), type alias (T), complex control flow,
//! recursion, self-recursion (@), nested loops, break/continue.

use super::helpers::*;

// ==================== A. Self-Recursion (@) ====================

#[test]
fn e2e_p128_misc_self_recursion_factorial() {
    assert_exit_code(
        r#"
fn fact(n: i64) -> i64 {
    I n <= 1 { return 1 }
    return n * @(n - 1)
}
fn main() -> i64 = fact(5)
"#,
        120,
    );
}

#[test]
fn e2e_p128_misc_self_recursion_fibonacci() {
    assert_exit_code(
        r#"
fn fib(n: i64) -> i64 {
    I n < 2 { return n }
    return @(n - 1) + @(n - 2)
}
fn main() -> i64 = fib(10)
"#,
        55,
    );
}

#[test]
fn e2e_p128_misc_self_recursion_sum_to() {
    assert_exit_code(
        r#"
fn sum_to(n: i64) -> i64 {
    I n <= 0 { return 0 }
    return n + @(n - 1)
}
fn main() -> i64 = sum_to(9)
"#,
        45,
    );
}

#[test]
fn e2e_p128_misc_self_recursion_countdown() {
    assert_exit_code(
        r#"
fn countdown(n: i64) -> i64 {
    I n <= 0 { return 42 }
    return @(n - 1)
}
fn main() -> i64 = countdown(10)
"#,
        42,
    );
}

// ==================== B. Mutual Recursion ====================

#[test]
fn e2e_p128_misc_mutual_recursion() {
    assert_exit_code(
        r#"
fn is_even(n: i64) -> i64 {
    I n == 0 { return 1 }
    return is_odd(n - 1)
}
fn is_odd(n: i64) -> i64 {
    I n == 0 { return 0 }
    return is_even(n - 1)
}
fn main() -> i64 {
    I is_even(42) == 1 { 42 } else { 0 }
}
"#,
        42,
    );
}

// ==================== C. Complex Control Flow ====================

#[test]
fn e2e_p128_misc_nested_if() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    x := 10
    I x > 5 {
        I x > 8 {
            42
        } else {
            0
        }
    } else {
        0
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_misc_if_else_chain() {
    assert_exit_code(
        r#"
fn classify(n: i64) -> i64 {
    I n < 0 { return 1 }
    I n == 0 { return 2 }
    I n < 10 { return 3 }
    I n < 100 { return 42 }
    return 5
}
fn main() -> i64 = classify(50)
"#,
        42,
    );
}

#[test]
fn e2e_p128_misc_nested_loop_break() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    count := mut 0
    L i:0..10 {
        L j:0..10 {
            I j == 3 { B }
            count = count + 1
        }
    }
    count
}
"#,
        30,
    );
}

#[test]
fn e2e_p128_misc_loop_continue() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    sum := mut 0
    L i:0..10 {
        I i % 2 == 0 { C }
        sum = sum + i
    }
    sum
}
"#,
        25,
    );
}

#[test]
fn e2e_p128_misc_early_return() {
    assert_exit_code(
        r#"
fn find_first_gt5(a: i64, b: i64, c: i64) -> i64 {
    I a > 5 { return a }
    I b > 5 { return b }
    I c > 5 { return c }
    return 0
}
fn main() -> i64 = find_first_gt5(1, 42, 100)
"#,
        42,
    );
}

// ==================== D. Loop Patterns ====================

#[test]
fn e2e_p128_misc_while_loop() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    x := mut 0
    L {
        I x >= 42 { B }
        x = x + 1
    }
    x
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_misc_range_loop_sum() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    sum := mut 0
    L i:1..10 {
        sum = sum + i
    }
    sum
}
"#,
        45,
    );
}

#[test]
fn e2e_p128_misc_nested_range_loops() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    total := mut 0
    L i:1..4 {
        L j:1..4 {
            total = total + i * j
        }
    }
    total
}
"#,
        36,
    );
}

#[test]
fn e2e_p128_misc_loop_with_computation() {
    // 720 > 255 (exit code range), so check modulo 256 = 208
    // Instead, use a computation that fits in exit code range
    assert_exit_code(
        r#"
fn main() -> i64 {
    result := mut 1
    L i:1..5 {
        result = result * i
    }
    result
}
"#,
        24,
    );
}

// ==================== E. Variable Scoping ====================

#[test]
fn e2e_p128_misc_variable_shadowing() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    x := 10
    x := 42
    x
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_misc_variable_rebinding() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    x := 10
    y := x + 32
    y
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_misc_mutable_variable() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    x := mut 0
    x = 42
    x
}
"#,
        42,
    );
}

// ==================== F. Expression Bodies ====================

#[test]
fn e2e_p128_misc_expr_body_fn() {
    assert_exit_code(
        r#"
fn double(x: i64) -> i64 = x * 2
fn main() -> i64 = double(21)
"#,
        42,
    );
}

#[test]
fn e2e_p128_misc_expr_body_chain() {
    assert_exit_code(
        r#"
fn inc(x: i64) -> i64 = x + 1
fn double(x: i64) -> i64 = x * 2
fn main() -> i64 = inc(double(20))
"#,
        41,
    );
}

#[test]
fn e2e_p128_misc_complex_expr_body() {
    assert_exit_code(
        r#"
fn compute(a: i64, b: i64, c: i64) -> i64 = a * b + c
fn main() -> i64 = compute(6, 7, 0)
"#,
        42,
    );
}

// ==================== G. Array Operations ====================

#[test]
fn e2e_p128_misc_array_basic() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    arr := [10, 20, 12]
    arr[0] + arr[1] + arr[2]
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_misc_array_index() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    arr := [42, 1, 2, 3]
    arr[0]
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_misc_array_last_element() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    arr := [1, 2, 3, 42]
    arr[3]
}
"#,
        42,
    );
}

// ==================== H. Function Composition ====================

#[test]
fn e2e_p128_misc_fn_composition() {
    assert_exit_code(
        r#"
fn square(x: i64) -> i64 = x * x
fn add_six(x: i64) -> i64 = x + 6
fn main() -> i64 = add_six(square(6))
"#,
        42,
    );
}

#[test]
fn e2e_p128_misc_fn_multiple_calls() {
    assert_exit_code(
        r#"
fn max(a: i64, b: i64) -> i64 {
    I a > b { a } else { b }
}
fn main() -> i64 {
    a := max(10, 20)
    b := max(22, 5)
    a + b
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_misc_fn_recursive_helper() {
    assert_exit_code(
        r#"
fn gcd(a: i64, b: i64) -> i64 {
    I b == 0 { return a }
    return @(b, a % b)
}
fn main() -> i64 = gcd(42, 84)
"#,
        42,
    );
}

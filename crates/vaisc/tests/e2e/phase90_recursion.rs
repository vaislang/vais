//! Phase 90 -- Recursive Algorithms
//!
//! Tests for recursive functions using the @ self-recursion operator,
//! mutual recursion, and classic recursive algorithms.

use super::helpers::*;

// ==================== Classic Recursion ====================

#[test]
fn e2e_rec_fibonacci() {
    let source = r#"
F fib(n: i64) -> i64 = n < 2 ? n : @(n-1) + @(n-2)
F main() -> i64 = fib(10)
"#;
    // fib(10) = 55
    assert_exit_code(source, 55);
}

#[test]
fn e2e_rec_fibonacci_zero() {
    let source = r#"
F fib(n: i64) -> i64 = n < 2 ? n : @(n-1) + @(n-2)
F main() -> i64 = fib(0)
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_rec_fibonacci_one() {
    let source = r#"
F fib(n: i64) -> i64 = n < 2 ? n : @(n-1) + @(n-2)
F main() -> i64 = fib(1)
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_rec_factorial() {
    let source = r#"
F fact(n: i64) -> i64 = n <= 1 ? 1 : n * @(n-1)
F main() -> i64 = fact(5)
"#;
    // 5! = 120
    assert_exit_code(source, 120);
}

#[test]
fn e2e_rec_factorial_zero() {
    let source = r#"
F fact(n: i64) -> i64 = n <= 1 ? 1 : n * @(n-1)
F main() -> i64 = fact(0)
"#;
    assert_exit_code(source, 1);
}

// ==================== Sum Recursion ====================

#[test]
fn e2e_rec_sum_to_n() {
    let source = r#"
F sum(n: i64) -> i64 = n <= 0 ? 0 : n + @(n-1)
F main() -> i64 = sum(9)
"#;
    // 1+2+...+9 = 45
    assert_exit_code(source, 45);
}

#[test]
fn e2e_rec_sum_of_digits() {
    let source = r#"
F digit_sum(n: i64) -> i64 {
    I n < 10 { R n }
    R n % 10 + @(n / 10)
}
F main() -> i64 = digit_sum(12345)
"#;
    // 1+2+3+4+5 = 15
    assert_exit_code(source, 15);
}

#[test]
fn e2e_rec_count_digits() {
    let source = r#"
F count_digits(n: i64) -> i64 {
    I n < 10 { R 1 }
    R 1 + @(n / 10)
}
F main() -> i64 = count_digits(12345)
"#;
    assert_exit_code(source, 5);
}

// ==================== Power / GCD ====================

#[test]
fn e2e_rec_power() {
    let source = r#"
F pow(base: i64, exp: i64) -> i64 {
    I exp == 0 { R 1 }
    R base * @(base, exp - 1)
}
F main() -> i64 = pow(2, 5)
"#;
    // 2^5 = 32
    assert_exit_code(source, 32);
}

#[test]
fn e2e_rec_gcd() {
    let source = r#"
F gcd(a: i64, b: i64) -> i64 {
    I b == 0 { R a }
    R @(b, a % b)
}
F main() -> i64 = gcd(126, 84)
"#;
    // gcd(126, 84) = 42
    assert_exit_code(source, 42);
}

#[test]
fn e2e_rec_gcd_coprime() {
    let source = r#"
F gcd(a: i64, b: i64) -> i64 {
    I b == 0 { R a }
    R @(b, a % b)
}
F main() -> i64 = gcd(17, 13)
"#;
    assert_exit_code(source, 1);
}

// ==================== Tail-Recursive Patterns ====================

#[test]
fn e2e_rec_tail_sum() {
    let source = r#"
F sum_tail(n: i64, acc: i64) -> i64 {
    I n <= 0 { R acc }
    R @(n - 1, acc + n)
}
F main() -> i64 = sum_tail(9, 0)
"#;
    // 1+2+...+9 = 45
    assert_exit_code(source, 45);
}

#[test]
fn e2e_rec_tail_factorial() {
    let source = r#"
F fact_tail(n: i64, acc: i64) -> i64 {
    I n <= 1 { R acc }
    R @(n - 1, acc * n)
}
F main() -> i64 = fact_tail(5, 1)
"#;
    assert_exit_code(source, 120);
}

#[test]
fn e2e_rec_tail_count_down() {
    let source = r#"
F count_down(n: i64) -> i64 {
    I n <= 0 { R 0 }
    R @(n - 1)
}
F main() -> i64 = count_down(100)
"#;
    assert_exit_code(source, 0);
}

// ==================== Binary Recursion ====================

#[test]
fn e2e_rec_binary_search_count() {
    // Count how many times we can halve before reaching 1
    let source = r#"
F halves(n: i64) -> i64 {
    I n <= 1 { R 0 }
    R 1 + @(n / 2)
}
F main() -> i64 = halves(64)
"#;
    // log2(64) = 6
    assert_exit_code(source, 6);
}

#[test]
fn e2e_rec_collatz_steps() {
    // Count Collatz steps to reach 1
    let source = r#"
F collatz(n: i64) -> i64 {
    I n <= 1 { R 0 }
    I n % 2 == 0 { R 1 + @(n / 2) }
    E { R 1 + @(3 * n + 1) }
}
F main() -> i64 = collatz(7)
"#;
    // 7→22→11→34→17→52→26→13→40→20→10→5→16→8→4→2→1 = 16 steps
    assert_exit_code(source, 16);
}

// ==================== Max/Min Recursion ====================

#[test]
fn e2e_rec_max_of_three() {
    let source = r#"
F max(a: i64, b: i64) -> i64 = a > b ? a : b
F max3(a: i64, b: i64, c: i64) -> i64 = max(max(a, b), c)
F main() -> i64 = max3(10, 42, 30)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_rec_ackermann_small() {
    // Ackermann function for small inputs
    let source = r#"
F ack(m: i64, n: i64) -> i64 {
    I m == 0 { R n + 1 }
    I n == 0 { R @(m - 1, 1) }
    R @(m - 1, @(m, n - 1))
}
F main() -> i64 = ack(2, 3)
"#;
    // ack(2,3) = 9
    assert_exit_code(source, 9);
}

#[test]
fn e2e_rec_tower_height() {
    // Build a tower recursively
    let source = r#"
F tower(n: i64) -> i64 {
    I n == 0 { R 0 }
    R n + @(n - 1)
}
F main() -> i64 = tower(8)
"#;
    // 8+7+6+5+4+3+2+1 = 36
    assert_exit_code(source, 36);
}

//! Phase 41 — Global variables and advanced patterns E2E tests
//!
//! Tests for under-covered features:
//! - Global variable mutation across functions
//! - Nested function calls
//! - Recursive accumulation patterns
//! - Complex expression body functions
//! - Match with computation

use super::helpers::*;

// ==================== Nested Function Calls ====================

#[test]
fn e2e_p41_nested_calls_triple() {
    // f(g(h(2))): h=*3=6, g=+4=10, f=*2=20
    let source = r#"
F h(x: i64) -> i64 { x * 3 }
F g(x: i64) -> i64 { x + 4 }
F f(x: i64) -> i64 { x * 2 }

F main() -> i64 {
    R f(g(h(2)))
}
"#;
    assert_exit_code(source, 20);
}

#[test]
fn e2e_p41_nested_calls_same_function() {
    // f(f(f(1))): 1*2+1=3, 3*2+1=7, 7*2+1=15
    let source = r#"
F f(x: i64) -> i64 { x * 2 + 1 }

F main() -> i64 {
    R f(f(f(1)))
}
"#;
    assert_exit_code(source, 15);
}

// ==================== Recursive Patterns ====================

#[test]
fn e2e_p41_recursive_sum() {
    // Sum 1..5 recursively: 5+4+3+2+1 = 15
    let source = r#"
F sum(n: i64) -> i64 {
    I n <= 0 { R 0 }
    R n + @(n - 1)
}

F main() -> i64 {
    R sum(5)
}
"#;
    assert_exit_code(source, 15);
}

#[test]
fn e2e_p41_recursive_power() {
    // 2^6 = 64
    let source = r#"
F power(base: i64, exp: i64) -> i64 {
    I exp == 0 { R 1 }
    R base * power(base, exp - 1)
}

F main() -> i64 {
    R power(2, 6)
}
"#;
    assert_exit_code(source, 64);
}

#[test]
fn e2e_p41_recursive_gcd() {
    // gcd(48, 18) = 6
    let source = r#"
F gcd(a: i64, b: i64) -> i64 {
    I b == 0 { R a }
    R gcd(b, a % b)
}

F main() -> i64 {
    R gcd(48, 18)
}
"#;
    assert_exit_code(source, 6);
}

// ==================== Expression Body Functions ====================

#[test]
fn e2e_p41_expr_body_arithmetic() {
    let source = r#"
F square(x: i64) -> i64 = x * x

F main() -> i64 {
    R square(7)
}
"#;
    assert_exit_code(source, 49);
}

#[test]
fn e2e_p41_expr_body_with_call() {
    let source = r#"
F double(x: i64) -> i64 = x * 2
F quad(x: i64) -> i64 = double(double(x))

F main() -> i64 {
    R quad(3)
}
"#;
    assert_exit_code(source, 12);
}

// ==================== Multiple Returns ====================

#[test]
fn e2e_p41_multiple_return_paths() {
    let source = r#"
F classify(x: i64) -> i64 {
    I x > 100 { R 3 }
    I x > 10 { R 2 }
    I x > 0 { R 1 }
    R 0
}

F main() -> i64 {
    R classify(5) + classify(50) + classify(500)
}
"#;
    // classify(5)=1, classify(50)=2, classify(500)=3 → 6
    assert_exit_code(source, 6);
}

#[test]
fn e2e_p41_early_return_in_loop() {
    // Find first i where i*i > 50: i=8 (64>50)
    let source = r#"
F find_threshold() -> i64 {
    L i: 1..100 {
        I i * i > 50 { R i }
    }
    R 0
}

F main() -> i64 {
    R find_threshold()
}
"#;
    assert_exit_code(source, 8);
}

// ==================== Complex Combinations ====================

#[test]
fn e2e_p41_fibonacci_iterative() {
    // fib(10) = 55
    let source = r#"
F main() -> i64 {
    a := mut 0
    b := mut 1
    L i: 0..10 {
        temp := a + b
        a = b
        b = temp
    }
    R a
}
"#;
    assert_exit_code(source, 55);
}

#[test]
fn e2e_p41_collatz_steps() {
    // Count Collatz steps for n=6: 6→3→10→5→16→8→4→2→1 = 8 steps
    let source = r#"
F main() -> i64 {
    n := mut 6
    steps := mut 0
    L {
        I n <= 1 { B }
        I n % 2 == 0 {
            n = n / 2
        } E {
            n = n * 3 + 1
        }
        steps = steps + 1
    }
    R steps
}
"#;
    assert_exit_code(source, 8);
}

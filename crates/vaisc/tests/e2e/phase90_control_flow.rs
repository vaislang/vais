//! Phase 90 -- Control Flow Patterns
//!
//! Tests for if/else chains, nested conditionals, match expressions,
//! loop patterns, and complex control flow.

use super::helpers::*;

// ==================== If/Else Chains ====================

#[test]
fn e2e_cf_simple_if_true() {
    let source = r#"
F main() -> i64 {
    x := 10
    I x > 5 { R 42 }
    R 0
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_cf_simple_if_false() {
    let source = r#"
F main() -> i64 {
    x := 3
    I x > 5 { R 0 }
    R 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_cf_if_else() {
    let source = r#"
F main() -> i64 {
    x := 10
    I x > 5 { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_cf_nested_if() {
    let source = r#"
F main() -> i64 {
    x := 10
    y := 20
    I x > 5 {
        I y > 15 { 42 } E { 0 }
    } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_cf_triple_nested_if() {
    let source = r#"
F main() -> i64 {
    a := 1
    b := 2
    c := 3
    I a < b {
        I b < c {
            I c == 3 { 42 } E { 0 }
        } E { 0 }
    } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_cf_if_else_chain() {
    let source = r#"
F classify(x: i64) -> i64 {
    I x < 0 { R 1 }
    E I x == 0 { R 2 }
    E I x < 10 { R 3 }
    E I x < 100 { R 4 }
    E { R 5 }
}
F main() -> i64 = classify(42)
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_cf_if_as_expression() {
    let source = r#"
F main() -> i64 {
    x := 10
    result := I x > 5 { 42 } E { 0 }
    result
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Loop Patterns ====================

#[test]
fn e2e_cf_loop_break() {
    let source = r#"
F main() -> i64 {
    i := mut 0
    L {
        I i >= 42 { B }
        i = i + 1
    }
    i
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_cf_loop_continue() {
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i:0..10 {
        I i % 2 == 0 { C }
        sum = sum + i
    }
    sum
}
"#;
    // odd numbers 1+3+5+7+9 = 25
    assert_exit_code(source, 25);
}

#[test]
fn e2e_cf_nested_loops() {
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i:0..3 {
        L j:0..3 {
            sum = sum + 1
        }
    }
    sum
}
"#;
    // 3*3 = 9
    assert_exit_code(source, 9);
}

#[test]
fn e2e_cf_loop_early_return() {
    let source = r#"
F find_first_gt(threshold: i64) -> i64 {
    L i:0..100 {
        I i > threshold { R i }
    }
    R 0
}
F main() -> i64 = find_first_gt(41)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_cf_countdown() {
    let source = r#"
F main() -> i64 {
    n := mut 10
    L {
        I n <= 0 { B }
        n = n - 1
    }
    n
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_cf_loop_with_accumulator() {
    let source = r#"
F main() -> i64 {
    product := mut 1
    L i:1..7 {
        product = product * i
    }
    product
}
"#;
    // 1*2*3*4*5*6 = 720 -> 720 % 256 = 208
    assert_exit_code(source, 208);
}

// ==================== Match Expressions ====================

#[test]
fn e2e_cf_match_simple() {
    let source = r#"
F main() -> i64 {
    x := 3
    M x {
        1 => 10,
        2 => 20,
        3 => 42,
        _ => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_cf_match_wildcard() {
    let source = r#"
F main() -> i64 {
    x := 99
    M x {
        1 => 10,
        2 => 20,
        _ => 42
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_cf_match_in_function() {
    let source = r#"
F day_type(d: i64) -> i64 {
    M d {
        1 => 1,
        2 => 1,
        3 => 1,
        4 => 1,
        5 => 1,
        6 => 2,
        7 => 2,
        _ => 0
    }
}
F main() -> i64 = day_type(6)
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_cf_match_nested() {
    let source = r#"
F inner(y: i64) -> i64 {
    M y {
        2 => 42,
        _ => 0
    }
}
F main() -> i64 {
    x := 1
    M x {
        1 => inner(2),
        _ => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Ternary Operator ====================

#[test]
fn e2e_cf_ternary_true() {
    assert_exit_code("F main()->i64 = 1 > 0 ? 42 : 0", 42);
}

#[test]
fn e2e_cf_ternary_false() {
    assert_exit_code("F main()->i64 = 0 > 1 ? 0 : 42", 42);
}

#[test]
fn e2e_cf_ternary_nested() {
    let source = r#"
F inner(x: i64) -> i64 = x > 3 ? 42 : 0
F main() -> i64 {
    x := 5
    x > 10 ? 1 : inner(x)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
#[ignore] // clang-17 crash on nested ternary phi nodes (LLVM bug, passes on macOS)
fn e2e_cf_ternary_in_function() {
    let source = r#"
F sign(x: i64) -> i64 = x > 0 ? 1 : (x < 0 ? 2 : 0)
F main() -> i64 = sign(42)
"#;
    assert_exit_code(source, 1);
}

// ==================== Complex Flow ====================

#[test]
fn e2e_cf_early_return_nested() {
    let source = r#"
F check(a: i64, b: i64) -> i64 {
    I a > 0 {
        I b > 0 { R a + b }
        R a
    }
    R b
}
F main() -> i64 = check(20, 22)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_cf_multiple_returns() {
    let source = r#"
F classify(x: i64) -> i64 {
    I x < 0 { R 1 }
    I x == 0 { R 2 }
    I x < 50 { R 42 }
    R 4
}
F main() -> i64 = classify(30)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_cf_loop_match_combo() {
    let source = r#"
F main() -> i64 {
    result := mut 0
    L i:0..5 {
        result = result + M i {
            0 => 10,
            1 => 8,
            2 => 12,
            3 => 7,
            4 => 5,
            _ => 0
        }
    }
    result
}
"#;
    // 10+8+12+7+5 = 42
    assert_exit_code(source, 42);
}

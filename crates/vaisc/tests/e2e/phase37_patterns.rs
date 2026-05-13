//! Phase 37 — Pattern matching and Closure advanced E2E tests
//!
//! Tests for under-covered features:
//! - Pattern matching: nested match, match on function results, complex guards
//! - Closure: closures as return values, closures with compound operations
//! - Match with enum + data patterns
//! - Block expressions in various contexts

use super::helpers::*;

// ==================== Pattern Matching: Advanced ====================

#[test]
fn e2e_p37_match_nested_if_fallthrough() {
    // Match with nested classification:
    // classify(85): 85 matches x I x > 50 => 2, exit code 2
    let source = r#"
F classify(n: i64) -> i64 {
    M n {
        x I x > 90 => 4,
        x I x > 70 => 3,
        x I x > 50 => 2,
        x I x > 0 => 1,
        _ => 0
    }
}

F main() -> i64 {
    R classify(85)
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_p37_match_literal_zero() {
    // Match on literal 0 — direct pattern hit
    // input is 0 => 100, exit code 100
    let source = r#"
F check(n: i64) -> i64 {
    M n {
        0 => 100,
        1 => 200,
        _ => 0
    }
}

F main() -> i64 {
    R check(0)
}
"#;
    assert_exit_code(source, 100);
}

#[test]
fn e2e_p37_match_wildcard_fallback() {
    // Match that falls through to wildcard — input 999 matches _
    let source = r#"
F lookup(n: i64) -> i64 {
    M n {
        1 => 10,
        2 => 20,
        3 => 30,
        _ => 99
    }
}

F main() -> i64 {
    R lookup(999)
}
"#;
    assert_exit_code(source, 99);
}

#[test]
fn e2e_p37_match_on_function_result() {
    // Match on the result of a function call
    // compute() = 15, 15 matches x I x > 10 => 2, exit code 2
    let source = r#"
F compute() -> i64 { 15 }

F main() -> i64 {
    M compute() {
        x I x > 20 => 3,
        x I x > 10 => 2,
        x I x > 0 => 1,
        _ => 0
    }
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p37_match_or_pattern_extended() {
    // Or-pattern matching multiple literal values
    // n=7 matches 6|7|8 => 30, exit code 30
    let source = r#"
F bucket(n: i64) -> i64 {
    M n {
        1 | 2 | 3 => 10,
        4 | 5 => 20,
        6 | 7 | 8 => 30,
        _ => 0
    }
}

F main() -> i64 {
    R bucket(7)
}
"#;
    assert_exit_code(source, 30);
}

#[test]
fn e2e_p37_match_range_boundary() {
    // Range pattern boundary test: n=10, 0..10 is exclusive, so 10 falls to 10..20
    // bucket(10) => 2
    let source = r#"
F bucket(n: i64) -> i64 {
    M n {
        0..10 => 1,
        10..20 => 2,
        _ => 0
    }
}

F main() -> i64 {
    R bucket(10)
}
"#;
    assert_exit_code(source, 2);
}

// ==================== Enum Pattern Matching ====================

#[test]
fn e2e_p37_enum_match_with_data() {
    // Enum with data field — destructuring in match
    // area(Circle(5)) = 5 * 5 * 3 = 75, exit code 75
    let source = r#"
E Shape {
    Circle(i64),
    Square(i64)
}

F area(s: Shape) -> i64 {
    M s {
        Circle(r) => r * r * 3,
        Square(side) => side * side
    }
}

F main() -> i64 {
    R area(Circle(5))
}
"#;
    assert_exit_code(source, 75);
}

#[test]
fn e2e_p37_enum_match_second_variant() {
    // Test matching the second variant of an enum
    // area(Square(7)) = 7 * 7 = 49, exit code 49
    let source = r#"
E Shape {
    Circle(i64),
    Square(i64)
}

F area(s: Shape) -> i64 {
    M s {
        Circle(r) => r * r * 3,
        Square(side) => side * side
    }
}

F main() -> i64 {
    R area(Square(7))
}
"#;
    assert_exit_code(source, 49);
}

#[test]
fn e2e_p37_enum_unit_variants() {
    // Enum with unit variants (no data) — matching determines return value
    // to_num(Blue) matches Blue => 3
    let source = r#"
E Color {
    Red,
    Green,
    Blue
}

F to_num(c: Color) -> i64 {
    M c {
        Red => 1,
        Green => 2,
        Blue => 3
    }
}

F main() -> i64 {
    R to_num(Blue)
}
"#;
    assert_exit_code(source, 3);
}

// ==================== Closure: Advanced ====================

#[test]
fn e2e_p37_closure_capture_and_add() {
    // Closure captures outer variable and adds to parameter
    // x = 100, f(23) = 100 + 23 = 123. exit code truncated to i32: 123
    let source = r#"
F main() -> i64 {
    x := 100
    f := |y| x + y
    R f(23)
}
"#;
    assert_exit_code(source, 123);
}

#[test]
fn e2e_p37_closure_in_loop_accumulator() {
    // Closure used repeatedly inside a loop with mutable accumulator
    // f = |a, b| a + b, sum starts at 0, adds i (0..5) = 0+1+2+3+4 = 10
    let source = r#"
F main() -> i64 {
    f := |a: i64, b: i64| a + b
    sum := mut 0
    L i:0..5 {
        sum = f(sum, i)
    }
    R sum
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_p37_closure_nested_capture() {
    // Nested closure capturing from two scopes
    // a=5, b=3, inner closure captures a and b, returns a + b + x
    // result = 5 + 3 + 2 = 10
    let source = r#"
F main() -> i64 {
    a := 5
    b := 3
    f := |x| a + b + x
    R f(2)
}
"#;
    assert_exit_code(source, 10);
}

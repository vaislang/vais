//! Phase 32: Advanced Pattern Matching Edge Cases
//!
//! Tests for 8 edge cases in pattern matching:
//! - Nested tuple destructuring
//! - Enum variant data extraction
//! - Or patterns
//! - Guard conditions
//! - Deep wildcard patterns
//! - Multiple match arms (5+)
//! - Bool value matching
//! - Match result bound to variable and returned

use super::helpers::*;

// ==================== Phase 32: Advanced Pattern Matching ====================

// Test 1: Nested tuple pattern — M (a, (b, c)) { ... }
#[test]
fn e2e_phase32_pattern_nested_tuple() {
    // Matching on a nested tuple extracts all inner fields correctly.
    // (1, (2, 3)) => 1+2+3 = 6, exit code 0 (6 - 6 = 0)
    let source = r#"
F main() -> i64 {
    pair := (1, (2, 3))
    M pair {
        (a, (b, c)) => a + b + c - 6,
        _ => 1
    }
}
"#;
    assert_exit_code(source, 0);
}

// Test 2: Enum variant data extraction — M Shape { Circle(r) => r, Rect(w, h) => w * h }
#[test]
fn e2e_phase32_pattern_enum_data() {
    // Enum with data: Rect(3, 7) => 3 * 7 = 21, exit code 21
    let source = r#"
E Shape {
    Circle(i64),
    Rect(i64, i64)
}

F area(s: Shape) -> i64 {
    M s {
        Circle(r) => r * r,
        Rect(w, h) => w * h
    }
}

F main() -> i64 {
    R area(Rect(3, 7))
}
"#;
    assert_exit_code(source, 21);
}

// Test 3: Or pattern — M x { 1 | 2 | 3 => 10, _ => 0 }
#[test]
fn e2e_phase32_pattern_or_simple() {
    // x = 2 matches arm 1|2|3, so result = 10, exit code 10
    let source = r#"
F main() -> i64 {
    x := 2
    M x {
        1 | 2 | 3 => 10,
        _ => 0
    }
}
"#;
    assert_exit_code(source, 10);
}

// Test 4: Guard condition — M x { n I n > 10 => 1, _ => 0 }
#[test]
fn e2e_phase32_pattern_guard() {
    // x = 15: guard n > 10 is true => result 1, exit code 1
    let source = r#"
F main() -> i64 {
    x := 15
    M x {
        n I n > 10 => 1,
        _ => 0
    }
}
"#;
    assert_exit_code(source, 1);
}

// Test 5: Deep wildcard — M (_, x) { (_, 5) => 1, _ => 0 }
#[test]
fn e2e_phase32_pattern_wildcard_deep() {
    // (99, 5) matches (_, 5) => result 1, exit code 1
    let source = r#"
F main() -> i64 {
    pair := (99, 5)
    M pair {
        (_, 5) => 1,
        _ => 0
    }
}
"#;
    assert_exit_code(source, 1);
}

// Test 6: Multiple arms (5+) — match with 6 distinct literal arms
#[test]
fn e2e_phase32_pattern_multiple_arms() {
    // x = 4 matches arm 4 => 40, exit code 40
    let source = r#"
F label(x: i64) -> i64 {
    M x {
        1 => 10,
        2 => 20,
        3 => 30,
        4 => 40,
        5 => 50,
        _ => 0
    }
}

F main() -> i64 {
    R label(4)
}
"#;
    assert_exit_code(source, 40);
}

// Test 7: Bool value matching — M flag { true => 1, false => 0 }
#[test]
fn e2e_phase32_pattern_match_bool() {
    // flag = true => result 1, exit code 1
    let source = r#"
F main() -> i64 {
    flag := true
    M flag {
        true => 1,
        false => 0
    }
}
"#;
    assert_exit_code(source, 1);
}

// Test 8: Match result bound to variable and returned
#[test]
fn e2e_phase32_pattern_match_return() {
    // Match result assigned to 'result', then returned. x=3 => arm 3 => 30, exit code 30
    let source = r#"
F main() -> i64 {
    x := 3
    result := M x {
        1 => 10,
        2 => 20,
        3 => 30,
        _ => 0
    }
    R result
}
"#;
    assert_exit_code(source, 30);
}

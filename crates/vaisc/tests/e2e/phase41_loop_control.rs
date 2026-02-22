//! Phase 41 — Loop control flow E2E tests
//!
//! Tests for Continue (C) and Break (B) in various loop patterns.
//! These keywords were previously under-tested.

use super::helpers::*;

// ==================== Continue (C) ====================

#[test]
fn e2e_p41_continue_skip_even() {
    // Sum only odd numbers from 0..10: 1+3+5+7+9 = 25
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i: 0..10 {
        I i % 2 == 0 { C }
        sum = sum + i
    }
    R sum
}
"#;
    assert_exit_code(source, 25);
}

#[test]
fn e2e_p41_continue_skip_first() {
    // Skip i==0, sum 1..5: 1+2+3+4 = 10
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i: 0..5 {
        I i == 0 { C }
        sum = sum + i
    }
    R sum
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_p41_continue_counter() {
    // Count how many values are > 3 in 0..8: 4,5,6,7 = 4
    let source = r#"
F main() -> i64 {
    count := mut 0
    L i: 0..8 {
        I i <= 3 { C }
        count = count + 1
    }
    R count
}
"#;
    assert_exit_code(source, 4);
}

// ==================== Break (B) ====================

#[test]
fn e2e_p41_break_on_condition() {
    // Sum until i > 4: 0+1+2+3+4 = 10
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i: 0..100 {
        I i > 4 { B }
        sum = sum + i
    }
    R sum
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_p41_break_immediate() {
    // Break on first iteration — only i=0 body runs
    let source = r#"
F main() -> i64 {
    x := mut 0
    L i: 0..10 {
        x = x + 1
        B
    }
    R x
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_p41_break_after_one() {
    // Execute loop body once, then break
    let source = r#"
F main() -> i64 {
    x := mut 0
    L i: 0..10 {
        x = x + 10
        B
    }
    R x
}
"#;
    assert_exit_code(source, 10);
}

// ==================== Combined Continue + Break ====================

#[test]
fn e2e_p41_continue_then_break() {
    // Skip even, accumulate odd, break when sum > 10
    // 1+3+5+7 = 16 > 10 → break at i=7
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i: 0..100 {
        I i % 2 == 0 { C }
        sum = sum + i
        I sum > 10 { B }
    }
    R sum
}
"#;
    assert_exit_code(source, 16);
}

#[test]
fn e2e_p41_while_style_loop() {
    // While-style loop using L { ... B }
    // Count down from 5 to 0
    let source = r#"
F main() -> i64 {
    x := mut 5
    count := mut 0
    L {
        I x <= 0 { B }
        x = x - 1
        count = count + 1
    }
    R count
}
"#;
    assert_exit_code(source, 5);
}

#[test]
fn e2e_p41_continue_multiple_conditions() {
    // Skip multiples of 2 AND multiples of 3, sum rest in 0..12
    // Skip: 0,2,3,4,6,8,9,10 → Keep: 1,5,7,11 = 24
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i: 0..12 {
        I i % 2 == 0 { C }
        I i % 3 == 0 { C }
        sum = sum + i
    }
    R sum
}
"#;
    assert_exit_code(source, 24);
}

#[test]
fn e2e_p41_break_find_first() {
    // Find first multiple of 7 >= 10: answer is 14
    let source = r#"
F main() -> i64 {
    result := mut 0
    L i: 10..100 {
        I i % 7 == 0 {
            result = i
            B
        }
    }
    R result
}
"#;
    assert_exit_code(source, 14);
}

#[test]
fn e2e_p41_loop_with_continue_and_counter() {
    // Count numbers not divisible by 3 in 1..10: 1,2,4,5,7,8 = 6 items
    let source = r#"
F main() -> i64 {
    count := mut 0
    L i: 1..10 {
        I i % 3 == 0 { C }
        count = count + 1
    }
    R count
}
"#;
    assert_exit_code(source, 6);
}

#[test]
fn e2e_p41_break_accumulate_until() {
    // Accumulate squares until sum >= 30: 1+4+9+16 = 30
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i: 1..100 {
        sum = sum + i * i
        I sum >= 30 { B }
    }
    R sum
}
"#;
    assert_exit_code(source, 30);
}

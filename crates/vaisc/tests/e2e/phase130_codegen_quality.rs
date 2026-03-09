//! Phase 130: Codegen quality improvements
//!
//! Tests for:
//! - Const eval div/mod operations
//! - Const eval shift operations
//! - TC break/continue outside loop validation
//! - Loop nesting correctness

use super::helpers::*;

// ===== Const eval div/mod =====

#[test]
fn e2e_p130_const_div_basic() {
    let source = r#"
C HALF: i64 = 100 / 2

F main() -> i64 {
    HALF - 50
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p130_const_mod_basic() {
    let source = r#"
C REM: i64 = 17 % 5

F main() -> i64 {
    REM - 2
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p130_const_div_larger() {
    let source = r#"
C VAL: i64 = 1000 / 7

F main() -> i64 {
    VAL - 142
}
"#;
    // 1000 / 7 = 142 (truncation)
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p130_const_mod_larger() {
    let source = r#"
C VAL: i64 = 1000 % 7

F main() -> i64 {
    VAL - 6
}
"#;
    // 1000 % 7 = 6
    assert_exit_code(source, 0);
}

// ===== Const eval shift operations =====

#[test]
fn e2e_p130_const_shl_valid() {
    let source = r#"
C SHIFTED: i64 = 1 << 10

F main() -> i64 {
    I SHIFTED == 1024 { R 0 }
    R 1
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p130_const_shr_valid() {
    let source = r#"
C SHIFTED: i64 = 1024 >> 3

F main() -> i64 {
    I SHIFTED == 128 { R 0 }
    R 1
}
"#;
    assert_exit_code(source, 0);
}

// ===== Const combined operations =====

#[test]
fn e2e_p130_const_complex_arithmetic() {
    // (100/3)*3 + 100%3 = 33*3 + 1 = 99 + 1 = 100
    let source = r#"
C DIV_PART: i64 = 100 / 3
C MUL_PART: i64 = DIV_PART * 3
C REM_PART: i64 = 100 % 3

F main() -> i64 {
    MUL_PART + REM_PART - 100
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p130_const_shift_and_mask() {
    let source = r#"
C MASK: i64 = 255 << 8
C UNMASKED: i64 = MASK >> 8

F main() -> i64 {
    UNMASKED - 255
}
"#;
    assert_exit_code(source, 0);
}

// ===== TC break/continue validation =====

#[test]
fn e2e_p130_break_inside_loop_ok() {
    let source = r#"
F main() -> i64 {
    x := mut 0
    L {
        x = x + 1
        I x > 5 {
            B
        }
    }
    x - 6
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p130_continue_inside_loop_ok() {
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i: 0..10 {
        I i % 2 == 0 {
            C
        }
        sum = sum + 1
    }
    sum - 5
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p130_break_outside_loop_error() {
    let source = r#"
F main() -> i64 {
    B
    0
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_p130_continue_outside_loop_error() {
    let source = r#"
F main() -> i64 {
    C
    0
}
"#;
    assert_compile_error(source);
}

// ===== Loop nesting validation =====

#[test]
fn e2e_p130_nested_break_ok() {
    let source = r#"
F main() -> i64 {
    x := mut 0
    L {
        L {
            B
        }
        x = x + 1
        I x >= 3 {
            B
        }
    }
    x - 3
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p130_while_break_ok() {
    let source = r#"
F main() -> i64 {
    x := mut 0
    L x < 10 {
        I x == 5 {
            B
        }
        x = x + 1
    }
    x - 5
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p130_for_continue_skip_even() {
    // Sum of odd numbers 1..10: 1+3+5+7+9 = 25
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i: 0..10 {
        I i % 2 == 0 {
            C
        }
        sum = sum + i
    }
    sum - 25
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p130_break_in_if_inside_loop() {
    let source = r#"
F main() -> i64 {
    count := mut 0
    L i: 0..100 {
        count = count + 1
        I i == 9 {
            B
        }
    }
    count - 10
}
"#;
    assert_exit_code(source, 0);
}

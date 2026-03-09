//! Phase 134: Pattern Matching Complex E2E Tests (+40)
//!
//! Tests for: nested patterns, guard conditions, or patterns,
//! enum variant destructuring, wildcard patterns, literal patterns,
//! tuple patterns, struct patterns, complex match expressions.

use super::helpers::*;

// ==================== A. Literal Pattern Matching ====================

#[test]
fn e2e_p134_pat_literal_int() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M 42 {
        0 => 0,
        42 => 42,
        _ => 99
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_literal_zero() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 0
    M x {
        0 => 42,
        _ => 0
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_literal_negative() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := -1
    result := M x {
        -1 => 42,
        0 => 0,
        _ => 99
    }
    result
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_literal_bool_true() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M true {
        true => 42,
        false => 0
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_literal_bool_false() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M false {
        true => 0,
        false => 42
    }
}
"#,
        42,
    );
}

// ==================== B. Variable Binding in Match ====================

#[test]
fn e2e_p134_pat_variable_bind() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M 42 {
        n => n
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_variable_with_fallback() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M 42 {
        0 => 0,
        n => n
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_variable_expr() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M 21 {
        n => n * 2
    }
}
"#,
        42,
    );
}

// ==================== C. Wildcard Patterns ====================

#[test]
fn e2e_p134_pat_wildcard_default() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M 999 {
        0 => 0,
        1 => 1,
        _ => 42
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_wildcard_first_match_wins() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M 5 {
        5 => 42,
        _ => 0
    }
}
"#,
        42,
    );
}

// ==================== D. Or Patterns ====================

#[test]
fn e2e_p134_pat_or_two() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M 2 {
        1 | 2 => 42,
        _ => 0
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_or_three() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M 3 {
        1 | 2 | 3 => 42,
        _ => 0
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_or_no_match() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M 5 {
        1 | 2 | 3 => 0,
        _ => 42
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_or_with_wildcard() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M 100 {
        0 | 1 => 0,
        _ => 42
    }
}
"#,
        42,
    );
}

// ==================== E. Guard Conditions ====================

#[test]
fn e2e_p134_pat_guard_greater() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M 50 {
        n I n > 40 => 42,
        _ => 0
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_guard_less() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M 5 {
        n I n < 10 => 42,
        _ => 0
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_guard_equal() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M 42 {
        n I n == 42 => n,
        _ => 0
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_guard_false() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M 5 {
        n I n > 100 => 0,
        _ => 42
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_guard_complex() {
    assert_exit_code(
        r#"
F main() -> i64 {
    M 42 {
        n I n > 10 => I n == 42 { n } E { 0 },
        _ => 0
    }
}
"#,
        42,
    );
}

// ==================== F. Enum Variant Matching ====================

#[test]
fn e2e_p134_pat_enum_simple() {
    assert_exit_code(
        r#"
E Dir { Up, Down, Left, Right }
F main() -> i64 {
    d := Right
    M d {
        Up => 1,
        Down => 2,
        Left => 3,
        Right => 42
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_enum_with_data() {
    assert_exit_code(
        r#"
E Expr {
    Num(i64),
    Neg(i64)
}
F eval(e: Expr) -> i64 {
    M e {
        Num(n) => n,
        Neg(n) => 0 - n
    }
}
F main() -> i64 = eval(Num(42))
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_enum_two_fields() {
    assert_exit_code(
        r#"
E Op {
    Add(i64, i64),
    Mul(i64, i64)
}
F calc(op: Op) -> i64 {
    M op {
        Add(a, b) => a + b,
        Mul(a, b) => a * b
    }
}
F main() -> i64 = calc(Add(20, 22))
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_enum_mixed_arms() {
    assert_exit_code(
        r#"
E Token {
    Int(i64),
    Bool(bool),
    None
}
F val(t: Token) -> i64 {
    M t {
        Int(n) => n,
        Bool(b) => I b { 1 } E { 0 },
        None => 0
    }
}
F main() -> i64 = val(Int(42))
"#,
        42,
    );
}

// ==================== G. Tuple Patterns ====================

#[test]
fn e2e_p134_pat_tuple_basic() {
    assert_exit_code(
        r#"
F main() -> i64 {
    pair := (20, 22)
    M pair {
        (a, b) => a + b
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_tuple_triple() {
    assert_exit_code(
        r#"
F main() -> i64 {
    t := (10, 20, 12)
    M t {
        (a, b, c) => a + b + c
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_tuple_with_literal() {
    assert_exit_code(
        r#"
F main() -> i64 {
    pair := (1, 42)
    M pair {
        (0, _) => 0,
        (1, v) => v,
        _ => 99
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_nested_tuple() {
    assert_exit_code(
        r#"
F main() -> i64 {
    nested := (10, 32)
    M nested {
        (a, b) => a + b
    }
}
"#,
        42,
    );
}

// ==================== H. Match as Expression ====================

#[test]
fn e2e_p134_pat_match_as_expr() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 3
    result := M x {
        1 => 10,
        2 => 20,
        3 => 42,
        _ => 0
    }
    result
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_match_in_fn_return() {
    assert_exit_code(
        r#"
F classify(n: i64) -> i64 {
    M n {
        0 => 0,
        _ => 42
    }
}
F main() -> i64 = classify(99)
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_match_in_addition() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := M 1 { 1 => 20, _ => 0 }
    b := M 2 { 2 => 22, _ => 0 }
    a + b
}
"#,
        42,
    );
}

// ==================== I. Multiple Arms with Computation ====================

#[test]
fn e2e_p134_pat_many_arms() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 7
    M x {
        1 => 10,
        2 => 20,
        3 => 30,
        4 => 40,
        5 => 50,
        6 => 60,
        7 => 42,
        _ => 0
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_arms_with_blocks() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 2
    M x {
        1 => {
            a := 10
            a + 5
        },
        2 => {
            a := 20
            b := 22
            a + b
        },
        _ => 0
    }
}
"#,
        42,
    );
}

// ==================== J. Match in Loops ====================

#[test]
fn e2e_p134_pat_match_in_loop() {
    assert_exit_code(
        r#"
F classify(x: i64) -> i64 {
    M x {
        0 => 10,
        1 => 5,
        2 => 7,
        3 => 8,
        4 => 6,
        5 => 6,
        _ => 0
    }
}
F main() -> i64 {
    sum := mut 0
    L i:0..6 {
        sum = sum + classify(i)
    }
    sum
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_match_break_condition() {
    assert_exit_code(
        r#"
F main() -> i64 {
    result := mut 0
    L i:0..100 {
        I i == 42 { result = 42; B }
    }
    result
}
"#,
        42,
    );
}

// ==================== K. Match with Function Calls ====================

#[test]
fn e2e_p134_pat_match_fn_result() {
    assert_exit_code(
        r#"
F compute(x: i64) -> i64 = x * 2
F main() -> i64 {
    M compute(21) {
        42 => 42,
        _ => 0
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_pat_match_nested_fn() {
    assert_exit_code(
        r#"
F double(x: i64) -> i64 = x * 2
F inc(x: i64) -> i64 = x + 1
F main() -> i64 {
    M double(inc(20)) {
        42 => 42,
        _ => 0
    }
}
"#,
        42,
    );
}

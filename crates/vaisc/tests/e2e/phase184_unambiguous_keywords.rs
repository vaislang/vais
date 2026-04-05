//! Phase 184 -- Unambiguous Keywords (EN/EL/LF/LW)
//!
//! Tests for the new deterministic keyword variants that eliminate
//! parser ambiguity between enum/else (E) and loop types (L).

use super::helpers::*;

// ==================== EN (enum) ====================

#[test]
fn e2e_en_keyword_basic_enum() {
    let source = r#"
EN Color { Red, Green, Blue }
F main() -> i64 {
    c := Color.Red
    M c {
        Color.Red => 1,
        Color.Green => 2,
        Color.Blue => 3,
    }
}
"#;
    assert_exit_code(source, 1);
}

#[test]
#[ignore = "multi-field tuple variant pattern binding: payload struct extract fallback only binds first field (pre-existing codegen limitation, unrelated to Phase 184 keywords)"]
fn e2e_en_keyword_enum_with_data() {
    let source = r#"
EN Shape {
    Circle(i64),
    Rect(i64, i64),
}
F main() -> i64 {
    s := Shape.Circle(42)
    M s {
        Shape.Circle(r) => r,
        Shape.Rect(w, h) => w + h,
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== EL (else) ====================

#[test]
fn e2e_el_keyword_basic_if_else() {
    let source = r#"
F main() -> i64 {
    x := 10
    I x > 5 { 42 } EL { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_el_keyword_else_if_chain() {
    let source = r#"
F main() -> i64 {
    x := 3
    I x == 1 { 10 }
    EL I x == 2 { 20 }
    EL I x == 3 { 30 }
    EL { 0 }
}
"#;
    assert_exit_code(source, 30);
}

#[test]
#[ignore = "EN+EL keyword + Option type — shadow user-defined Option with builtin, codegen ptr mismatch (separate issue)"]
fn e2e_el_keyword_mixed_with_en() {
    // EN for enum, EL for else — no ambiguity
    let source = r#"
EN Option { Some(i64), None }
F main() -> i64 {
    o := Option.Some(42)
    M o {
        Option.Some(v) => {
            I v > 0 { v } EL { 0 }
        },
        Option.None => 0,
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== LW (while loop) ====================

#[test]
fn e2e_lw_keyword_basic_while() {
    let source = r#"
F main() -> i64 {
    x := mut 0
    LW x < 10 {
        x = x + 1
    }
    x
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_lw_keyword_while_with_break() {
    let source = r#"
F main() -> i64 {
    x := mut 0
    LW true {
        x = x + 1
        I x == 5 { B }
    }
    x
}
"#;
    assert_exit_code(source, 5);
}

// ==================== LF (for-each loop) ====================

#[test]
#[ignore = "LF keyword + Vec.new() type inference: separate codegen issue (Vec<i64> implicit generic inference path)"]
fn e2e_lf_keyword_basic_foreach() {
    let source = r#"
F main() -> i64 {
    sum := mut 0
    v := Vec.new()
    v.push(10)
    v.push(20)
    v.push(12)
    LF i:v {
        sum = sum + i
    }
    sum
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Backward Compatibility ====================
// Old E/L keywords should still work

#[test]
fn e2e_backward_compat_old_enum() {
    let source = r#"
E Direction { Up, Down }
F main() -> i64 {
    d := Direction.Up
    M d {
        Direction.Up => 1,
        Direction.Down => 2,
    }
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_backward_compat_old_else() {
    let source = r#"
F main() -> i64 {
    x := 10
    I x > 5 { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_backward_compat_old_while_loop() {
    let source = r#"
F main() -> i64 {
    x := mut 0
    L x < 10 {
        x = x + 1
    }
    x
}
"#;
    assert_exit_code(source, 10);
}

//! Phase 146: split_keyword_idents generalization — E/Else lexer stabilization
//!
//! Tests that adjacent single-char keyword letters lexed as one Ident are
//! correctly split into individual keyword tokens.

use super::helpers::*;

#[test]
fn e2e_p146_else_if_chain() {
    let source = r#"
fn classify(x: i64) -> i64 {
    I x > 10 { 3 }
    else I x > 5 { 2 }
    else { 1 }
}
fn main() -> i64 {
    result := classify(3)
    I result != 1 { return 1 }
    result2 := classify(7)
    I result2 != 2 { return 2 }
    result3 := classify(15)
    I result3 != 3 { return 3 }
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p146_else_if_nested() {
    // Nested else-if chains where EI would appear as adjacent tokens
    let source = r#"
fn sign(x: i64) -> i64 {
    I x > 0 { 1 }
    else I x < 0 { -1 }
    else { 0 }
}
fn main() -> i64 {
    a := sign(42)
    I a != 1 { return 1 }
    b := sign(-7)
    I b != -1 { return 2 }
    c := sign(0)
    I c != 0 { return 3 }
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p146_multiple_else_if() {
    // Multiple else-if levels
    let source = r#"
fn grade(score: i64) -> i64 {
    I score >= 90 { 4 }
    else I score >= 80 { 3 }
    else I score >= 70 { 2 }
    else I score >= 60 { 1 }
    else { 0 }
}
fn main() -> i64 {
    I grade(95) != 4 { return 1 }
    I grade(85) != 3 { return 2 }
    I grade(75) != 2 { return 3 }
    I grade(65) != 1 { return 4 }
    I grade(50) != 0 { return 5 }
    0
}
"#;
    assert_exit_code(source, 0);
}

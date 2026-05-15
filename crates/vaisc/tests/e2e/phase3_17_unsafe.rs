//! Phase 3.17 — unsafe block expression pass-through.

use super::helpers::*;

#[test]
fn unsafe_block_expression_returns_value() {
    let source = r#"
fn main() -> i64 {
    x := 5
    y := unsafe {
        x + 10
    }
    y
}
"#;
    assert_exit_code(source, 15);
}

#[test]
fn unsafe_block_with_stmts() {
    let source = r#"
fn main() -> i64 {
    unsafe {
        42
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn unsafe_block_in_expression_context() {
    let source = r#"
fn main() -> i64 {
    n := 7
    m := unsafe { n * 3 } + 1
    m
}
"#;
    assert_exit_code(source, 22);
}

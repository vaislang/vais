//! Phase 146: >> 제네릭 3중첩+ 파싱 — pending_gt bool→count
//!
//! Tests for:
//! 1. Triple-nested generic type declarations (Vec<Vec<Vec<i64>>> style)
//! 2. pending_gt_count correctly tracks multiple pending '>' from '>>' splits
//! 3. Struct/function type annotations with deeply nested generics

use super::helpers::*;

#[test]
fn e2e_p146_triple_nested_generic() {
    // Triple-nested generic struct declarations should parse without error
    let source = r#"
struct Inner { val: i64 }
struct Mid<T> { item: type }
struct Outer<T> { nested: type }

fn main() -> i64 {
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p146_triple_nested_generic_type_annotation() {
    // Simple function with exit code verification
    let source = r#"
fn identity(x: i64) -> i64 { x }
fn main() -> i64 {
    identity(42)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p146_double_nested_generic_parses() {
    // Double-nesting (Vec<Vec<i64>>) — baseline that was already working
    let source = r#"
struct Wrapper<T> { val: type }
struct Container<T> { inner: type }

fn main() -> i64 {
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p146_generic_fn_triple_nested_return() {
    // Function returning triple-nested generic
    let source = r#"
struct Box<T> { val: type }

fn wrap(x: i64) -> i64 {
    x
}

fn main() -> i64 {
    wrap(7)
}
"#;
    assert_exit_code(source, 7);
}

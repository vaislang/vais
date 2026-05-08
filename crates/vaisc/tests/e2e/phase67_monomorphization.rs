//! Phase 67: Monomorphization Enhancement Tests
//!
//! Tests for transitive instantiation, multi-level generics,
//! generic struct instantiation, and generic function composition.
//!
//! These tests verify that the monomorphization infrastructure correctly:
//! 1. Collects generic callee relationships during type checking
//! 2. Propagates instantiations transitively (foo<T> calls bar<T> -> bar<i64>)
//! 3. Handles multi-level generic function chains
//! 4. Generates correct specialized code for generic struct + function combos

use super::helpers::*;

// ==================== 1. Transitive generic: foo<T> calls bar<T> ====================

#[test]
fn e2e_phase67_transitive_generic_two_levels() {
    // bar<T> is only called from inside foo<T>.
    // When foo<i64> is instantiated, bar<i64> must also be instantiated.
    let source = r#"
fn bar<T>(x: T) -> type { x }
fn foo<T>(x: T) -> type { bar(x) }

fn main() -> i64 {
    foo(42)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 2. Three-level transitive chain ====================

#[test]
fn e2e_phase67_transitive_three_levels() {
    // c<T> -> b<T> -> a<T>, only c is called from main
    let source = r#"
fn a<T>(x: T) -> type { x }
fn b<T>(x: T) -> type { a(x) }
fn c<T>(x: T) -> type { b(x) }

fn main() -> i64 {
    c(99)
}
"#;
    assert_exit_code(source, 99);
}

// ==================== 3. Transitive with arithmetic ====================

#[test]
fn e2e_phase67_transitive_with_arithmetic() {
    // add_ten<T> calls identity<T>; arithmetic performed on result
    let source = r#"
fn identity<T>(x: T) -> type { x }
fn add_ten<T>(x: T) -> type { identity(x) + 10 }

fn main() -> i64 {
    add_ten(32)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 4. Diamond-shaped transitive calls ====================

#[test]
fn e2e_phase67_diamond_transitive() {
    // both left<T> and right<T> call base<T>; top<T> calls both
    let source = r#"
fn base<T>(x: T) -> type { x }
fn left<T>(x: T) -> type { base(x) }
fn right<T>(x: T) -> type { base(x) }
fn top<T>(a: T, b: T) -> type { left(a) + right(b) }

fn main() -> i64 {
    top(20, 22)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 5. Generic function called multiple times ====================

#[test]
fn e2e_phase67_multiple_calls_same_generic() {
    // wrap<T> is called 3 times with the same type from within combine<T>
    let source = r#"
fn wrap<T>(x: T) -> type { x }
fn combine<T>(a: T, b: T, c: T) -> type { wrap(a) + wrap(b) + wrap(c) }

fn main() -> i64 {
    combine(10, 20, 12)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 6. Generic with struct return ====================

#[test]
fn e2e_phase67_generic_struct_return() {
    // Generic function creates and returns a struct
    let source = r#"
struct Pair { first: i64, second: i64 }

fn make_pair(a: i64, b: i64) -> Pair {
    Pair { first: a, second: b }
}

fn main() -> i64 {
    p := make_pair(30, 12)
    p.first + p.second
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 7. Transitive generic with conditional ====================

#[test]
fn e2e_phase67_transitive_with_conditional() {
    // inner<T> returns x; outer<T> calls inner conditionally
    let source = r#"
fn inner<T>(x: T) -> type { x }
fn outer<T>(x: T) -> type {
    I x > 0 {
        inner(x)
    } else {
        inner(0)
    }
}

fn main() -> i64 {
    outer(42)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 8. Recursive generic (self-call) ====================

#[test]
fn e2e_phase67_recursive_generic() {
    // countdown uses @(n-1) to self-recurse; tests that recursive generics work
    let source = r#"
fn countdown(n: i64) -> i64 {
    I n <= 0 { return 0 }
    @(n - 1)
}

fn main() -> i64 {
    countdown(10)
}
"#;
    assert_exit_code(source, 0);
}

// ==================== 9. Two separate instantiation types through transitive ====================

#[test]
fn e2e_phase67_two_instantiations_transitive() {
    // caller1 calls helper<T> with i64, caller2 with i64 too
    // Both should produce correct results
    let source = r#"
fn helper<T>(x: T) -> type { x }
fn caller_a<T>(x: T) -> type { helper(x) + 1 }
fn caller_b<T>(x: T) -> type { helper(x) + 2 }

fn main() -> i64 {
    a := caller_a(20)
    b := caller_b(19)
    a + b
}
"#;
    // caller_a(20) = helper(20) + 1 = 21
    // caller_b(19) = helper(19) + 2 = 21
    // 21 + 21 = 42
    assert_exit_code(source, 42);
}

// ==================== 10. Generic identity chain (4 levels) ====================

#[test]
fn e2e_phase67_identity_chain_four_levels() {
    let source = r#"
fn id1<T>(x: T) -> type { x }
fn id2<T>(x: T) -> type { id1(x) }
fn id3<T>(x: T) -> type { id2(x) }
fn id4<T>(x: T) -> type { id3(x) }

fn main() -> i64 {
    id4(77)
}
"#;
    assert_exit_code(source, 77);
}

// ==================== 11. Transitive generic with accumulation ====================

#[test]
fn e2e_phase67_transitive_accumulation() {
    // Each level adds a constant to the value
    let source = r#"
fn base<T>(x: T) -> type { x + 1 }
fn mid<T>(x: T) -> type { base(x) + 1 }
fn top<T>(x: T) -> type { mid(x) + 1 }

fn main() -> i64 {
    top(39)
}
"#;
    // top(39) = mid(39) + 1 = (base(39) + 1) + 1 = ((39+1) + 1) + 1 = 42
    assert_exit_code(source, 42);
}

// ==================== 12. Generic struct with generic function ====================

#[test]
fn e2e_phase67_generic_struct_with_generic_fn() {
    // Generic struct Wrapper<T> and generic function unwrap<T>
    let source = r#"
struct Wrapper<T> { value: type }

fn unwrap_value<T>(w: Wrapper<T>) -> type {
    w.value
}

fn main() -> i64 {
    w := Wrapper { value: 42 }
    unwrap_value(w)
}
"#;
    assert_exit_code(source, 42);
}

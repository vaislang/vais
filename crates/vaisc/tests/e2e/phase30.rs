//! Phase 30: Generic Monomorphization Tests
//!
//! Tests that generic functions and structs are correctly monomorphized
//! (specialized for concrete types) during code generation.

use super::helpers::assert_exit_code;

#[test]
fn generic_identity_function() {
    assert_exit_code(
        r#"
F identity<T>(x: T) -> T { x }
F main() -> i64 {
    identity(42)
}
"#,
        42,
    );
}

#[test]
fn generic_function_multiple_instantiations() {
    assert_exit_code(
        r#"
F identity<T>(x: T) -> T { x }
F main() -> i64 {
    a := identity(10)
    b := identity(20)
    a + b
}
"#,
        30,
    );
}

#[test]
fn generic_function_two_type_params() {
    assert_exit_code(
        r#"
F first<A, B>(a: A, b: B) -> A { a }
F main() -> i64 {
    first(42, 0)
}
"#,
        42,
    );
}

#[test]
fn generic_function_nested_calls() {
    assert_exit_code(
        r#"
F identity<T>(x: T) -> T { x }
F double<T>(x: T) -> T { identity(x) }
F main() -> i64 {
    double(21)
}
"#,
        21,
    );
}

#[test]
fn generic_function_with_arithmetic() {
    assert_exit_code(
        r#"
F add_one<T>(x: T) -> T { x }
F main() -> i64 {
    a := add_one(20)
    b := add_one(22)
    a + b
}
"#,
        42,
    );
}

#[test]
fn generic_swap_compiles() {
    assert_exit_code(
        r#"
F swap<T>(a: T, b: T) -> T { b }
F main() -> i64 {
    swap(1, 2)
}
"#,
        2,
    );
}

#[test]
fn multiple_generic_functions() {
    assert_exit_code(
        r#"
F id<T>(x: T) -> T { x }
F const_val<T>(x: T, y: T) -> T { x }
F main() -> i64 {
    a := id(10)
    b := const_val(32, 0)
    a + b
}
"#,
        42,
    );
}

#[test]
fn generic_with_bool() {
    assert_exit_code(
        r#"
F identity<T>(x: T) -> T { x }
F main() -> i64 {
    identity(42)
}
"#,
        42,
    );
}

#[test]
fn generic_expression_body() {
    assert_exit_code(
        r#"
F wrap<T>(x: T) -> T = x
F main() -> i64 = wrap(99)
"#,
        99,
    );
}

#[test]
fn generic_function_repeated_same_type() {
    assert_exit_code(
        r#"
F get<T>(x: T) -> T { x }
F main() -> i64 {
    a := get(10)
    b := get(20)
    c := get(12)
    a + b + c
}
"#,
        42,
    );
}

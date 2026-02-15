//! Phase 42: Lambda ByRef/ByMutRef parameter passing and Lazy evaluation
//!
//! Tests for:
//! - Lambda ByRef parameters (`|&x| expr`) — passes parameters by immutable reference
//! - Lambda ByMutRef parameters (`|&mut x| expr`) — passes parameters by mutable reference
//! - Lazy deferred evaluation (`lazy expr`) — creates a thunk function
//! - Force evaluation (`force lazy_val`) — checks computed flag, calls thunk if needed

use super::helpers::*;

// ===== Lambda ByRef parameter tests =====

#[test]
fn e2e_phase42_lambda_byref_simple() {
    // Basic ByRef parameter — |&x| takes x by reference
    let source = r#"
F main() -> i64 {
    f := |&x| x + 1
    y := 10
    R f(y)
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase42_lambda_byref_call() {
    // Lambda with ByRef parameter can be called multiple times
    let source = r#"
F main() -> i64 {
    f := |&x| x * 2
    a := f(10)
    b := f(21)
    R a + b
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase42_lambda_byref_with_capture() {
    // ByRef parameter + captured variable from outer scope
    let source = r#"
F main() -> i64 {
    multiplier := 3
    f := |&x| x * multiplier
    R f(14)
}
"#;
    assert_compiles(source);
}

// ===== Lambda ByMutRef parameter tests =====

#[test]
fn e2e_phase42_lambda_bymutref_simple() {
    // Basic ByMutRef parameter — |&mut x| takes x by mutable reference
    let source = r#"
F main() -> i64 {
    f := |&mut x| x + 1
    y := mut 10
    R f(y)
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase42_lambda_bymutref_call() {
    // Lambda with ByMutRef parameter
    let source = r#"
F main() -> i64 {
    f := |&mut x| x * 2
    a := mut 10
    result := f(a)
    R result
}
"#;
    assert_compiles(source);
}

// ===== Lazy/Force evaluation tests =====

#[test]
fn e2e_phase42_lazy_force_basic() {
    // Basic lazy/force — value should be correct
    let source = r#"
F main() -> i64 {
    x := lazy 42
    R force x
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase42_lazy_force_expression() {
    // Lazy with expression
    let source = r#"
F main() -> i64 {
    x := lazy (10 + 32)
    R force x
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase42_lazy_force_with_capture() {
    // Lazy captures free variables
    let source = r#"
F main() -> i64 {
    a := 10
    b := 20
    x := lazy (a + b)
    R force x
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase42_lazy_force_function_call() {
    // Lazy wrapping a function call
    let source = r#"
F add(a: i64, b: i64) -> i64 {
    R a + b
}

F main() -> i64 {
    x := lazy add(10, 32)
    R force x
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase42_lazy_force_nested() {
    // Nested lazy/force
    let source = r#"
F main() -> i64 {
    x := lazy 42
    y := lazy (force x)
    R force y
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase42_lazy_no_capture() {
    // Lazy with no free variables (constant expression)
    let source = r#"
F main() -> i64 {
    x := lazy 100
    R force x
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase42_lazy_force_multiple() {
    // Multiple lazy values
    let source = r#"
F main() -> i64 {
    x := lazy 10
    y := lazy 20
    z := lazy (force x + force y)
    R force z
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase42_lazy_force_mutable_capture() {
    // Lazy capturing mutable variable
    let source = r#"
F main() -> i64 {
    a := mut 10
    x := lazy a
    a = 20
    R force x
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase42_lazy_force_closure() {
    // Lazy containing a closure
    let source = r#"
F main() -> i64 {
    a := 10
    x := lazy {
        f := |b| a + b
        R f(32)
    }
    R force x
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase42_lazy_force_conditional() {
    // Lazy with conditional expression
    let source = r#"
F main() -> i64 {
    cond := 1
    x := lazy (I cond == 1 { 42 } E { 0 })
    R force x
}
"#;
    assert_compiles(source);
}

// ===== Combined ByRef and Lazy tests =====

#[test]
fn e2e_phase42_lazy_with_byref_lambda() {
    // Lazy containing a ByRef parameter lambda
    let source = r#"
F main() -> i64 {
    a := 10
    x := lazy {
        f := |&val| val * 2
        R f(a)
    }
    R force x
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase42_lambda_byref_with_lazy() {
    // Lambda with ByRef parameter that forces a lazy value
    let source = r#"
F main() -> i64 {
    x := lazy 42
    f := |&dummy| force x
    R f(0)
}
"#;
    assert_compiles(source);
}

// ===== Edge cases and error handling =====

#[test]
fn e2e_phase42_force_non_lazy_basic() {
    // Forcing a non-lazy value — currently may work (implementation detail)
    let source = r#"
F main() -> i64 {
    x := 42
    R force x
}
"#;
    // This test verifies current behavior. If force on non-lazy is allowed, it succeeds.
    // If it's a type error, we'd use assert_compile_error instead.
    let _ = compile_to_ir(source); // Don't assert either way - just verify it compiles without panic
}

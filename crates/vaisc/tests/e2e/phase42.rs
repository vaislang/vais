//! Phase 42: Lambda ByRef/ByMutRef parameter passing
//!
//! Tests for:
//! - Lambda ByRef parameters (`|&x| expr`) — passes parameters by immutable reference
//! - Lambda ByMutRef parameters (`|&mut x| expr`) — passes parameters by mutable reference

use super::helpers::*;

// ===== Lambda ByRef parameter tests =====

#[test]
fn e2e_phase42_lambda_byref_simple() {
    // Basic ByRef parameter — |&x| takes x by reference
    let source = r#"
fn main() -> i64 {
    f := |&x| x + 1
    y := 10
    return f(y)
}
"#;
    assert_exit_code(source, 11);
}

#[test]
fn e2e_phase42_lambda_byref_call() {
    // Lambda with ByRef parameter can be called multiple times
    let source = r#"
fn main() -> i64 {
    f := |&x| x * 2
    a := f(10)
    b := f(21)
    return a + b
}
"#;
    assert_exit_code(source, 62);
}

#[test]
fn e2e_phase42_lambda_byref_with_capture() {
    // ByRef parameter + captured variable from outer scope
    let source = r#"
fn main() -> i64 {
    multiplier := 3
    f := |&x| x * multiplier
    return f(14)
}
"#;
    assert_exit_code(source, 42);
}

// ===== Lambda ByMutRef parameter tests =====

#[test]
fn e2e_phase42_lambda_bymutref_simple() {
    // Basic ByMutRef parameter — |&mut x| takes x by mutable reference
    let source = r#"
fn main() -> i64 {
    f := |&mut x| x + 1
    y := mut 10
    return f(y)
}
"#;
    assert_exit_code(source, 11);
}

#[test]
fn e2e_phase42_lambda_bymutref_call() {
    // Lambda with ByMutRef parameter
    let source = r#"
fn main() -> i64 {
    f := |&mut x| x * 2
    a := mut 10
    result := f(a)
    return result
}
"#;
    assert_exit_code(source, 20);
}

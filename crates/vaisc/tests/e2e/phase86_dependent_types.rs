//! Phase 86 — Dependent Types: Predicate Boolean Validation + E2E
//!
//! Tests for dependent type (refinement type) support:
//! 1. Valid predicates (boolean expressions) compile successfully
//! 2. Invalid predicates (non-boolean expressions) are rejected
//! 3. Compile-time refinement checking for literal values
//! 4. Runtime values pass through without compile-time checking
//! 5. Compound predicates with logical operators

use super::helpers::*;

// ==================== 1. Valid Predicates — Compilation ====================

#[test]
fn e2e_dependent_positive_i64() {
    // {x: i64 | x > 0} with positive literal satisfies predicate
    let source = r#"
F check_positive(n: {x: i64 | x > 0}) -> i64 {
    R n
}
F main() -> i64 {
    R check_positive(7)
}
"#;
    assert_exit_code(source, 7);
}

#[test]
fn e2e_dependent_nonneg_zero() {
    // {x: i64 | x >= 0} with zero satisfies predicate
    let source = r#"
F check_nonneg(n: {x: i64 | x >= 0}) -> i64 {
    R n
}
F main() -> i64 {
    R check_nonneg(0)
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_dependent_less_than() {
    // {x: i64 | x < 100} with value 50 satisfies predicate
    let source = r#"
F check_small(n: {x: i64 | x < 100}) -> i64 {
    R n
}
F main() -> i64 {
    R check_small(50)
}
"#;
    assert_exit_code(source, 50);
}

#[test]
fn e2e_dependent_equality() {
    // {x: i64 | x == 42} with value 42
    let source = r#"
F check_exact(n: {x: i64 | x == 42}) -> i64 {
    R n
}
F main() -> i64 {
    R check_exact(42)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 2. Invalid Predicates — Compile Errors ====================

#[test]
fn e2e_dependent_violation_negative() {
    // Passing -1 to {x: i64 | x > 0} should fail
    let source = r#"
F check_positive(n: {x: i64 | x > 0}) -> i64 {
    R n
}
F main() -> i64 {
    R check_positive(-1)
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_dependent_violation_zero_strict() {
    // Passing 0 to {x: i64 | x > 0} should fail
    let source = r#"
F check_positive(n: {x: i64 | x > 0}) -> i64 {
    R n
}
F main() -> i64 {
    R check_positive(0)
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_dependent_violation_upper_bound() {
    // Passing 200 to {x: i64 | x < 100} should fail
    let source = r#"
F check_small(n: {x: i64 | x < 100}) -> i64 {
    R n
}
F main() -> i64 {
    R check_small(200)
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_dependent_violation_equality() {
    // Passing 10 to {x: i64 | x == 42} should fail
    let source = r#"
F check_exact(n: {x: i64 | x == 42}) -> i64 {
    R n
}
F main() -> i64 {
    R check_exact(10)
}
"#;
    assert_compile_error(source);
}

// ==================== 3. Variable Binding with Dependent Type ====================

#[test]
fn e2e_dependent_binding_valid() {
    // Variable binding with dependent type and valid literal
    let source = r#"
F main() -> i64 {
    x: {n: i64 | n > 0} = 10
    R x
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_dependent_binding_violation() {
    // Variable binding with dependent type and invalid literal
    let source = r#"
F main() -> i64 {
    x: {n: i64 | n > 0} = -5
    R x
}
"#;
    assert_compile_error(source);
}

// ==================== 4. Runtime Values — No Compile-Time Check ====================

#[test]
fn e2e_dependent_runtime_passthrough() {
    // Non-literal argument bypasses compile-time check (runtime values)
    let source = r#"
F check_positive(n: {x: i64 | x > 0}) -> i64 {
    R n
}
F get_value() -> i64 { R 42 }
F main() -> i64 {
    v := get_value()
    R check_positive(v)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 5. Compound Predicates ====================

#[test]
fn e2e_dependent_compound_and_valid() {
    // {x: i64 | x >= 0 && x <= 100} with value 50
    let source = r#"
F bounded(n: {x: i64 | x >= 0 && x <= 100}) -> i64 {
    R n
}
F main() -> i64 {
    R bounded(50)
}
"#;
    assert_exit_code(source, 50);
}

#[test]
fn e2e_dependent_compound_and_violation() {
    // {x: i64 | x >= 0 && x <= 100} with value 200 violates upper bound
    let source = r#"
F bounded(n: {x: i64 | x >= 0 && x <= 100}) -> i64 {
    R n
}
F main() -> i64 {
    R bounded(200)
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_dependent_compound_or_valid() {
    // {x: i64 | x == 0 || x == 1} with value 1
    let source = r#"
F binary(n: {x: i64 | x == 0 || x == 1}) -> i64 {
    R n
}
F main() -> i64 {
    R binary(1)
}
"#;
    assert_exit_code(source, 1);
}

// ==================== 6. Return Value with Dependent Body ====================

#[test]
fn e2e_dependent_arithmetic_in_body() {
    // Use dependent-typed parameter in arithmetic expression
    let source = r#"
F double_positive(n: {x: i64 | x > 0}) -> i64 {
    R n * 2
}
F main() -> i64 {
    R double_positive(21)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_dependent_add_positive_values() {
    // Two dependent-typed parameters
    let source = r#"
F add_pos(a: {x: i64 | x > 0}, b: {y: i64 | y > 0}) -> i64 {
    R a + b
}
F main() -> i64 {
    R add_pos(20, 22)
}
"#;
    assert_exit_code(source, 42);
}

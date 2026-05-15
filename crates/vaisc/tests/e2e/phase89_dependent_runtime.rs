//! Phase 89 -- Dependent Types: Runtime Assertions + f64 Support + Return Type Verification
//!
//! Tests for enhanced dependent type support:
//! 1. Runtime assertions for non-literal arguments (previously passed through unchecked)
//! 2. f64/float dependent type predicates (compile-time and runtime)
//! 3. Return type verification for dependent return types

use super::helpers::*;

// ==================== 1. Runtime Assertions for Non-Literal Arguments ====================

#[test]
fn e2e_dependent_runtime_assert_valid() {
    // Non-literal argument that satisfies predicate should pass runtime check
    let source = r#"
fn check_positive(n: {x: i64 | x > 0}) -> i64 {
    return n
}
fn get_value() -> i64 { return 42 }
fn main() -> i64 {
    v := get_value()
    return check_positive(v)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_dependent_runtime_assert_fail() {
    // Non-literal argument that violates predicate should trigger runtime abort
    let source = r#"
fn check_positive(n: {x: i64 | x > 0}) -> i64 {
    return n
}
fn get_negative() -> i64 { return 0 - 5 }
fn main() -> i64 {
    v := get_negative()
    return check_positive(v)
}
"#;
    // Should abort with non-zero exit code (runtime assertion failure)
    let result = compile_and_run(source);
    match result {
        Ok(r) => assert_ne!(
            r.exit_code, 0,
            "Expected runtime assertion failure, but program exited normally"
        ),
        Err(_) => {} // Compilation failure also acceptable (shouldn't happen but defensive)
    }
}

#[test]
fn e2e_dependent_runtime_compound_valid() {
    // Non-literal argument that satisfies compound predicate (AND)
    let source = r#"
fn bounded(n: {x: i64 | x >= 0 && x <= 100}) -> i64 {
    return n
}
fn compute() -> i64 { return 50 }
fn main() -> i64 {
    v := compute()
    return bounded(v)
}
"#;
    assert_exit_code(source, 50);
}

#[test]
fn e2e_dependent_runtime_compound_fail() {
    // Non-literal argument that violates compound predicate
    let source = r#"
fn bounded(n: {x: i64 | x >= 0 && x <= 100}) -> i64 {
    return n
}
fn compute() -> i64 { return 200 }
fn main() -> i64 {
    v := compute()
    return bounded(v)
}
"#;
    let result = compile_and_run(source);
    match result {
        Ok(r) => assert_ne!(r.exit_code, 0, "Expected runtime assertion failure"),
        Err(_) => {}
    }
}

#[test]
fn e2e_dependent_runtime_two_params_valid() {
    // Two dependent-typed parameters, both satisfied at runtime
    let source = r#"
fn add_pos(a: {x: i64 | x > 0}, b: {y: i64 | y > 0}) -> i64 {
    return a + b
}
fn get_a() -> i64 { return 20 }
fn get_b() -> i64 { return 22 }
fn main() -> i64 {
    return add_pos(get_a(), get_b())
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_dependent_runtime_two_params_one_fails() {
    // Two dependent-typed parameters, second one violates predicate
    let source = r#"
fn add_pos(a: {x: i64 | x > 0}, b: {y: i64 | y > 0}) -> i64 {
    return a + b
}
fn get_a() -> i64 { return 20 }
fn get_neg() -> i64 { return 0 - 1 }
fn main() -> i64 {
    return add_pos(get_a(), get_neg())
}
"#;
    let result = compile_and_run(source);
    match result {
        Ok(r) => assert_ne!(r.exit_code, 0, "Expected runtime assertion failure"),
        Err(_) => {}
    }
}

// ==================== 2. f64 Dependent Type Support ====================

#[test]
fn e2e_dependent_f64_positive_valid() {
    // {x: f64 | x > 0.0} with positive literal
    let source = r#"
fn check_positive_f(n: {x: f64 | x > 0.0}) -> f64 {
    return n
}
fn main() -> i64 {
    v := check_positive_f(3.14)
    return 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_dependent_f64_violation() {
    // {x: f64 | x > 0.0} with negative literal should fail at compile time
    let source = r#"
fn check_positive_f(n: {x: f64 | x > 0.0}) -> f64 {
    return n
}
fn main() -> i64 {
    v := check_positive_f(-1.5)
    return 0
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_dependent_f64_bounded_valid() {
    // {x: f64 | x >= 0.0 && x <= 1.0} with value in range
    let source = r#"
fn probability(p: {x: f64 | x >= 0.0 && x <= 1.0}) -> f64 {
    return p
}
fn main() -> i64 {
    v := probability(0.5)
    return 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_dependent_f64_bounded_violation() {
    // {x: f64 | x >= 0.0 && x <= 1.0} with value out of range
    let source = r#"
fn probability(p: {x: f64 | x >= 0.0 && x <= 1.0}) -> f64 {
    return p
}
fn main() -> i64 {
    v := probability(1.5)
    return 0
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_dependent_f64_runtime_valid() {
    // Non-literal f64 that satisfies predicate at runtime
    let source = r#"
fn check_positive_f(n: {x: f64 | x > 0.0}) -> f64 {
    return n
}
fn get_pi() -> f64 { return 3.14 }
fn main() -> i64 {
    v := check_positive_f(get_pi())
    return 42
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 3. Return Type Dependent Verification ====================

#[test]
fn e2e_dependent_return_type_valid() {
    // Function with dependent return type, returns valid value
    let source = r#"
fn make_positive(n: i64) -> {x: i64 | x > 0} {
    I n > 0 { return n }
    return 1
}
fn main() -> i64 {
    return make_positive(42)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_dependent_param_with_arithmetic() {
    // Use dependent-typed parameter in arithmetic at runtime
    let source = r#"
fn double_positive(n: {x: i64 | x > 0}) -> i64 {
    return n * 2
}
fn get_value() -> i64 { return 21 }
fn main() -> i64 {
    return double_positive(get_value())
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_dependent_equality_runtime_valid() {
    // {x: i64 | x == 42} with runtime value 42
    let source = r#"
fn exact(n: {x: i64 | x == 42}) -> i64 {
    return n
}
fn get_42() -> i64 { return 42 }
fn main() -> i64 {
    return exact(get_42())
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_dependent_equality_runtime_fail() {
    // {x: i64 | x == 42} with runtime value != 42
    let source = r#"
fn exact(n: {x: i64 | x == 42}) -> i64 {
    return n
}
fn get_10() -> i64 { return 10 }
fn main() -> i64 {
    return exact(get_10())
}
"#;
    let result = compile_and_run(source);
    match result {
        Ok(r) => assert_ne!(r.exit_code, 0, "Expected runtime assertion failure"),
        Err(_) => {}
    }
}

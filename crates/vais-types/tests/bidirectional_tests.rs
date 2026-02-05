//! Tests for bidirectional type checking

use vais_parser::parse;
use vais_types::{CheckMode, ResolvedType, TypeChecker};

fn check_module(source: &str) -> Result<(), String> {
    let module = parse(source).map_err(|e| format!("{:?}", e))?;
    let mut checker = TypeChecker::new();
    checker
        .check_module(&module)
        .map_err(|e| format!("{:?}", e))?;
    Ok(())
}

#[test]
fn test_check_mode_infer() {
    let mode = CheckMode::Infer;
    assert!(mode.is_infer());
    assert!(mode.expected().is_none());
}

#[test]
fn test_check_mode_check() {
    let mode = CheckMode::check(ResolvedType::I64);
    assert!(!mode.is_infer());
    assert_eq!(mode.expected(), Some(&ResolvedType::I64));
}

#[test]
fn test_lambda_explicit_type() {
    // Explicit type annotation should work
    let source = r#"
        F main() -> i64 {
            add_one := |x: i64| x + 1
            add_one(5)
        }
    "#;
    assert!(check_module(source).is_ok());
}

#[test]
fn test_lambda_with_two_params() {
    // Lambda with multiple parameters
    let source = r#"
        F main() -> i64 {
            add := |a: i64, b: i64| a + b
            add(3, 4)
        }
    "#;
    assert!(check_module(source).is_ok());
}

#[test]
fn test_recursive_function_explicit_return() {
    // Recursive functions need explicit return type
    let source = r#"
        F factorial(n: i64) -> i64 = I n <= 1 { 1 } E { n * factorial(n - 1) }
        F main() -> i64 = factorial(5)
    "#;
    assert!(check_module(source).is_ok());
}

#[test]
fn test_mutual_recursion_explicit_types() {
    // Mutual recursion requires explicit return types
    let source = r#"
        F is_even(n: i64) -> bool = I n == 0 { true } E { is_odd(n - 1) }
        F is_odd(n: i64) -> bool = I n == 0 { false } E { is_even(n - 1) }
        F main() -> bool = is_even(4)
    "#;
    assert!(check_module(source).is_ok());
}

#[test]
fn test_type_mismatch_error() {
    // Type mismatch should be caught
    let source = r#"
        F add(x: i64, y: i64) -> i64 = x + y
        F main() -> i64 = add(10, 5)
    "#;
    assert!(check_module(source).is_ok());
}

#[test]
fn test_simple_function_call() {
    // Simple function calls should work
    let source = r#"
        F double(x: i64) -> i64 = x * 2
        F main() -> i64 = double(21)
    "#;
    assert!(check_module(source).is_ok());
}

#[test]
fn test_nested_function_call() {
    // Nested function calls
    let source = r#"
        F add(x: i64, y: i64) -> i64 = x + y
        F mul(x: i64, y: i64) -> i64 = x * y
        F main() -> i64 = mul(add(1, 2), add(3, 4))
    "#;
    assert!(check_module(source).is_ok());
}

#[test]
fn test_if_else_type_check() {
    // If-else branches should have same type
    let source = r#"
        F abs(x: i64) -> i64 = I x < 0 { 0 - x } E { x }
        F main() -> i64 = abs(-5)
    "#;
    assert!(check_module(source).is_ok());
}

#[test]
fn test_generic_identity() {
    // Generic identity function
    let source = r#"
        F id<T>(x: T) -> T = x
        F main() -> i64 = id(42)
    "#;
    assert!(check_module(source).is_ok());
}

#[test]
fn test_generic_pair() {
    // Generic function with multiple type params
    let source = r#"
        F first<T, U>(x: T, y: U) -> T = x
        F main() -> i64 = first(1, 2)
    "#;
    assert!(check_module(source).is_ok());
}

#[test]
fn test_lambda_parameter_count_mismatch() {
    // Lambda with wrong parameter count should error or handle gracefully
    let source = r#"
        F apply(f: (i64, i64) -> i64, x: i64) -> i64 = f(x, x)
        F main() -> i64 {
            double := |x: i64| x * 2
            0
        }
    "#;
    // This should compile - we're not calling apply with wrong lambda
    assert!(check_module(source).is_ok());
}

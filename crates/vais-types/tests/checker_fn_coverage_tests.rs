//! Coverage tests for vais-types/src/checker_fn.rs
//!
//! Targets: function type checking, parameter validation, return type checking,
//! self parameter handling, generic function checking, impl method checking.

use vais_parser::parse;
use vais_types::TypeChecker;

fn check_ok(source: &str) {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut tc = TypeChecker::new();
    tc.check_module(&module)
        .unwrap_or_else(|e| panic!("Type check failed for: {}\nErr: {:?}", source, e));
}

#[allow(dead_code)]
fn check_err(source: &str) {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut tc = TypeChecker::new();
    assert!(
        tc.check_module(&module).is_err(),
        "Expected type error for: {}",
        source
    );
}

// ============================================================================
// Basic function checking
// ============================================================================

#[test]
fn test_check_fn_no_params_no_return() {
    check_ok("fn noop() -> i64 = 0");
}

#[test]
fn test_check_fn_single_param() {
    check_ok("fn inc(x: i64) -> i64 = x + 1");
}

#[test]
fn test_check_fn_multiple_params() {
    check_ok("fn add(a: i64, b: i64) -> i64 = a + b");
}

#[test]
fn test_check_fn_many_params() {
    check_ok("fn sum(a: i64, b: i64, c: i64, d: i64) -> i64 = a + b + c + d");
}

#[test]
fn test_check_fn_expression_body() {
    check_ok("fn double(x: i64) -> i64 = x * 2");
}

#[test]
fn test_check_fn_block_body() {
    check_ok(
        r#"
        fn compute(x: i64) -> i64 {
            a := x + 1
            b := a * 2
            b
        }
    "#,
    );
}

// ============================================================================
// Return type checking
// ============================================================================

#[test]
fn test_check_fn_return_bool() {
    check_ok("fn is_positive(x: i64) -> bool = x > 0");
}

#[test]
fn test_check_fn_return_f64() {
    check_ok("fn pi() -> f64 = 3.14");
}

#[test]
fn test_check_fn_return_str() {
    check_ok(r#"fn greeting() -> str = "hello""#);
}

#[test]
fn test_check_fn_return_unit() {
    check_ok(
        r#"
        fn do_nothing() {
            x := 42
        }
    "#,
    );
}

// ============================================================================
// Parameter type variety
// ============================================================================

#[test]
fn test_check_fn_bool_param() {
    check_ok("fn flip(b: bool) -> bool = !b");
}

#[test]
fn test_check_fn_f64_param() {
    check_ok("fn square(x: f64) -> f64 = x * x");
}

#[test]
fn test_check_fn_mixed_params() {
    check_ok(
        r#"
        fn mixed(n: i64, flag: bool) -> i64 {
            I flag { n * 2 } else { n }
        }
    "#,
    );
}

// ============================================================================
// Self parameter handling
// ============================================================================

#[test]
fn test_check_impl_method_self() {
    check_ok(
        r#"
        struct Point { x: i64, y: i64 }
        impl Point {
            fn get_x(self) -> i64 = self.x
        }
    "#,
    );
}

#[test]
fn test_check_impl_method_self_with_params() {
    check_ok(
        r#"
        struct Counter { value: i64 }
        impl Counter {
            fn add(self, n: i64) -> i64 = self.value + n
        }
    "#,
    );
}

#[test]
fn test_check_impl_multiple_methods() {
    check_ok(
        r#"
        struct Vec2 { x: i64, y: i64 }
        impl Vec2 {
            fn get_x(self) -> i64 = self.x
            fn get_y(self) -> i64 = self.y
            fn sum(self) -> i64 = self.x + self.y
        }
    "#,
    );
}

// ============================================================================
// Generic function checking
// ============================================================================

#[test]
fn test_check_generic_fn_basic() {
    check_ok("F identity<T>(x: T) -> T = x");
}

#[test]
fn test_check_generic_fn_two_params() {
    check_ok(
        r#"
        fn first<T>(a: T, b: T) -> type = a
    "#,
    );
}

#[test]
fn test_check_generic_fn_call() {
    check_ok(
        r#"
        fn identity<T>(x: T) -> type = x
        fn test() -> i64 = identity(42)
    "#,
    );
}

// ============================================================================
// Recursive functions
// ============================================================================

#[test]
fn test_check_fn_self_recursion() {
    check_ok(
        r#"
        fn factorial(n: i64) -> i64 {
            I n <= 1 { return 1 }
            n * @(n - 1)
        }
    "#,
    );
}

#[test]
fn test_check_fn_mutual_recursion_like() {
    check_ok(
        r#"
        fn is_even(n: i64) -> bool {
            I n == 0 { return true }
            return false
        }
        fn test() -> bool = is_even(4)
    "#,
    );
}

// ============================================================================
// Control flow in functions
// ============================================================================

#[test]
fn test_check_fn_if_else_return() {
    check_ok(
        r#"
        fn abs(x: i64) -> i64 {
            I x < 0 { 0 - x } else { x }
        }
    "#,
    );
}

#[test]
fn test_check_fn_match_return() {
    check_ok(
        r#"
        fn classify(x: i64) -> str {
            match x {
                0 => "zero",
                1 => "one",
                _ => "other"
            }
        }
    "#,
    );
}

#[test]
fn test_check_fn_early_return() {
    check_ok(
        r#"
        fn validate(x: i64) -> i64 {
            I x < 0 { return -1 }
            I x > 100 { return 100 }
            x
        }
    "#,
    );
}

// ============================================================================
// Closures/lambdas
// ============================================================================

#[test]
fn test_check_fn_lambda_param() {
    check_ok(
        r#"
        fn test() -> i64 {
            f := |x: i64| x * 2
            f(21)
        }
    "#,
    );
}

#[test]
fn test_check_fn_lambda_no_params() {
    check_ok(
        r#"
        fn test() -> i64 {
            f := || 42
            f()
        }
    "#,
    );
}

// ============================================================================
// Let binding in functions
// ============================================================================

#[test]
fn test_check_fn_let_chain() {
    check_ok(
        r#"
        fn test() -> i64 {
            a := 1
            b := a + 2
            c := b + 3
            d := c + 4
            d
        }
    "#,
    );
}

#[test]
fn test_check_fn_let_mutable() {
    check_ok(
        r#"
        fn test() -> i64 {
            x := mut 0
            x = 42
            x
        }
    "#,
    );
}

// ============================================================================
// Function calls
// ============================================================================

#[test]
fn test_check_fn_calling_other_fn() {
    check_ok(
        r#"
        fn helper(x: i64) -> i64 = x * 2
        fn test() -> i64 = helper(21)
    "#,
    );
}

#[test]
fn test_check_fn_chain_calls() {
    check_ok(
        r#"
        fn inc(x: i64) -> i64 = x + 1
        fn double(x: i64) -> i64 = x * 2
        fn test() -> i64 = double(inc(20))
    "#,
    );
}

// ============================================================================
// Trait method checking
// ============================================================================

#[test]
fn test_check_trait_method_sig() {
    check_ok(
        r#"
        trait Describable {
            fn describe(self) -> str
        }
        struct Thing { name: str }
        impl Thing: Describable {
            fn describe(self) -> str = self.name
        }
    "#,
    );
}

#[test]
fn test_check_trait_with_default() {
    check_ok(
        r#"
        trait HasDefault {
            fn value(self) -> i64 = 0
        }
        struct Foo { x: i64 }
        impl Foo: HasDefault {}
    "#,
    );
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_check_fn_deeply_nested_if() {
    check_ok(
        r#"
        fn deep(x: i64) -> i64 {
            I x > 0 {
                I x > 10 {
                    I x > 100 {
                        3
                    } else {
                        2
                    }
                } else {
                    1
                }
            } else {
                0
            }
        }
    "#,
    );
}

#[test]
fn test_check_fn_ternary() {
    check_ok("fn max(a: i64, b: i64) -> i64 = a > b ? a : b");
}

#[test]
fn test_check_fn_complex_expression() {
    check_ok("fn calc(a: i64, b: i64, c: i64) -> i64 = (a + b) * c - (a / b)");
}

// ============================================================================
// Where clause
// ============================================================================

#[test]
fn test_check_fn_where_clause() {
    check_ok(
        r#"
        trait Addable {
            fn add(self, other: i64) -> i64
        }
        fn use_add<T>(x: T) -> i64 where T: Addable = 0
    "#,
    );
}

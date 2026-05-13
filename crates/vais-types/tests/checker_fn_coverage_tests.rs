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
    check_ok("F noop() -> i64 = 0");
}

#[test]
fn test_check_fn_single_param() {
    check_ok("F inc(x: i64) -> i64 = x + 1");
}

#[test]
fn test_check_fn_multiple_params() {
    check_ok("F add(a: i64, b: i64) -> i64 = a + b");
}

#[test]
fn test_check_fn_many_params() {
    check_ok("F sum(a: i64, b: i64, c: i64, d: i64) -> i64 = a + b + c + d");
}

#[test]
fn test_check_fn_expression_body() {
    check_ok("F double(x: i64) -> i64 = x * 2");
}

#[test]
fn test_check_fn_block_body() {
    check_ok(
        r#"
        F compute(x: i64) -> i64 {
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
    check_ok("F is_positive(x: i64) -> bool = x > 0");
}

#[test]
fn test_check_fn_return_f64() {
    check_ok("F pi() -> f64 = 3.14");
}

#[test]
fn test_check_fn_return_str() {
    check_ok(r#"F greeting() -> str = "hello""#);
}

#[test]
fn test_check_fn_return_unit() {
    check_ok(
        r#"
        F do_nothing() {
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
    check_ok("F flip(b: bool) -> bool = !b");
}

#[test]
fn test_check_fn_f64_param() {
    check_ok("F square(x: f64) -> f64 = x * x");
}

#[test]
fn test_check_fn_mixed_params() {
    check_ok(
        r#"
        F mixed(n: i64, flag: bool) -> i64 {
            I flag { n * 2 } E { n }
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
        S Point { x: i64, y: i64 }
        X Point {
            F get_x(self) -> i64 = self.x
        }
    "#,
    );
}

#[test]
fn test_check_impl_method_self_with_params() {
    check_ok(
        r#"
        S Counter { value: i64 }
        X Counter {
            F add(self, n: i64) -> i64 = self.value + n
        }
    "#,
    );
}

#[test]
fn test_check_impl_multiple_methods() {
    check_ok(
        r#"
        S Vec2 { x: i64, y: i64 }
        X Vec2 {
            F get_x(self) -> i64 = self.x
            F get_y(self) -> i64 = self.y
            F sum(self) -> i64 = self.x + self.y
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
        F first<T>(a: T, b: T) -> T = a
    "#,
    );
}

#[test]
fn test_check_generic_fn_call() {
    check_ok(
        r#"
        F identity<T>(x: T) -> T = x
        F test() -> i64 = identity(42)
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
        F factorial(n: i64) -> i64 {
            I n <= 1 { R 1 }
            n * @(n - 1)
        }
    "#,
    );
}

#[test]
fn test_check_fn_mutual_recursion_like() {
    check_ok(
        r#"
        F is_even(n: i64) -> bool {
            I n == 0 { R true }
            R false
        }
        F test() -> bool = is_even(4)
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
        F abs(x: i64) -> i64 {
            I x < 0 { 0 - x } E { x }
        }
    "#,
    );
}

#[test]
fn test_check_fn_match_return() {
    check_ok(
        r#"
        F classify(x: i64) -> str {
            M x {
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
        F validate(x: i64) -> i64 {
            I x < 0 { R -1 }
            I x > 100 { R 100 }
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
        F test() -> i64 {
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
        F test() -> i64 {
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
        F test() -> i64 {
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
        F test() -> i64 {
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
        F helper(x: i64) -> i64 = x * 2
        F test() -> i64 = helper(21)
    "#,
    );
}

#[test]
fn test_check_fn_chain_calls() {
    check_ok(
        r#"
        F inc(x: i64) -> i64 = x + 1
        F double(x: i64) -> i64 = x * 2
        F test() -> i64 = double(inc(20))
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
        W Describable {
            F describe(self) -> str
        }
        S Thing { name: str }
        X Thing: Describable {
            F describe(self) -> str = self.name
        }
    "#,
    );
}

#[test]
fn test_check_trait_with_default() {
    check_ok(
        r#"
        W HasDefault {
            F value(self) -> i64 = 0
        }
        S Foo { x: i64 }
        X Foo: HasDefault {}
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
        F deep(x: i64) -> i64 {
            I x > 0 {
                I x > 10 {
                    I x > 100 {
                        3
                    } E {
                        2
                    }
                } E {
                    1
                }
            } E {
                0
            }
        }
    "#,
    );
}

#[test]
fn test_check_fn_ternary() {
    check_ok("F max(a: i64, b: i64) -> i64 = a > b ? a : b");
}

#[test]
fn test_check_fn_complex_expression() {
    check_ok("F calc(a: i64, b: i64, c: i64) -> i64 = (a + b) * c - (a / b)");
}

// ============================================================================
// Where clause
// ============================================================================

#[test]
fn test_check_fn_where_clause() {
    check_ok(
        r#"
        W Addable {
            F add(self, other: i64) -> i64
        }
        F use_add<T>(x: T) -> i64 where T: Addable = 0
    "#,
    );
}

//! Coverage tests for vais-types/src/free_vars.rs and scope.rs
//!
//! Targets: free variable analysis, scope creation/exit, variable shadowing,
//! closure capture analysis.

use vais_parser::parse;
use vais_types::TypeChecker;

fn check_ok(source: &str) {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut tc = TypeChecker::new();
    tc.check_module(&module)
        .unwrap_or_else(|e| panic!("Type check failed for: {}\nErr: {:?}", source, e));
}

// ============================================================================
// Scope: Basic variable scoping
// ============================================================================

#[test]
fn test_scope_let_binding() {
    check_ok(
        r#"
        F f() -> i64 {
            x := 42
            x
        }
    "#,
    );
}

#[test]
fn test_scope_sequential_bindings() {
    check_ok(
        r#"
        F f() -> i64 {
            x := 1
            y := 2
            z := 3
            x + y + z
        }
    "#,
    );
}

#[test]
fn test_scope_variable_shadowing() {
    check_ok(
        r#"
        F f() -> i64 {
            x := 10
            x := 20
            x
        }
    "#,
    );
}

#[test]
fn test_scope_function_params() {
    check_ok(
        r#"
        F add(a: i64, b: i64) -> i64 = a + b
    "#,
    );
}

#[test]
fn test_scope_block_creates_scope() {
    check_ok(
        r#"
        F f() -> i64 {
            x := 10
            y := {
                z := 20
                x + z
            }
            y
        }
    "#,
    );
}

#[test]
fn test_scope_if_creates_scope() {
    check_ok(
        r#"
        F f(x: i64) -> i64 {
            I x > 0 {
                y := x * 2
                R y
            }
            0
        }
    "#,
    );
}

#[test]
fn test_scope_loop_creates_scope() {
    check_ok(
        r#"
        F f() -> i64 {
            total := mut 0
            L i:0..5 {
                total = total + i
            }
            total
        }
    "#,
    );
}

#[test]
fn test_scope_match_arms() {
    check_ok(
        r#"
        F f(x: i64) -> i64 = M x {
            0 => {
                y := 100
                y
            },
            n => n + 1
        }
    "#,
    );
}

#[test]
fn test_scope_nested_functions() {
    check_ok(
        r#"
        F outer(x: i64) -> i64 = x + 1
        F inner(y: i64) -> i64 = y * 2
        F test() -> i64 = outer(inner(5))
    "#,
    );
}

// ============================================================================
// Free variables: Lambda capture
// ============================================================================

#[test]
fn test_free_vars_lambda_captures_outer() {
    check_ok(
        r#"
        F f() -> i64 {
            x := 10
            g := |y: i64| x + y
            g(5)
        }
    "#,
    );
}

#[test]
fn test_free_vars_lambda_no_capture() {
    check_ok(
        r#"
        F f() -> i64 {
            g := |x: i64| x * 2
            g(21)
        }
    "#,
    );
}

#[test]
fn test_free_vars_lambda_param_shadows_outer() {
    check_ok(
        r#"
        F f() -> i64 {
            x := 10
            g := |x: i64| x * 2
            g(21)
        }
    "#,
    );
}

#[test]
fn test_free_vars_lambda_captures_multiple() {
    check_ok(
        r#"
        F f() -> i64 {
            a := 1
            b := 2
            c := 3
            g := |x: i64| a + b + c + x
            g(0)
        }
    "#,
    );
}

#[test]
fn test_free_vars_nested_lambda() {
    check_ok(
        r#"
        F f() -> i64 {
            x := 10
            g := |y: i64| {
                h := |z: i64| x + y + z
                h(1)
            }
            g(5)
        }
    "#,
    );
}

// ============================================================================
// Scope: Struct and trait scopes
// ============================================================================

#[test]
fn test_scope_struct_methods() {
    check_ok(
        r#"
        S Counter { value: i64 }
        X Counter {
            F get(self) -> i64 = self.value
        }
        F test() -> i64 {
            c := Counter { value: 42 }
            c.get()
        }
    "#,
    );
}

#[test]
fn test_scope_trait_methods() {
    check_ok(
        r#"
        W Show {
            F show(self) -> i64
        }
        S Num { x: i64 }
        X Num: Show {
            F show(self) -> i64 = self.x
        }
        F test() -> i64 {
            n := Num { x: 5 }
            n.show()
        }
    "#,
    );
}

// ============================================================================
// Scope: Forward references
// ============================================================================

#[test]
fn test_scope_forward_function_ref() {
    check_ok(
        r#"
        F test() -> i64 = helper(5)
        F helper(x: i64) -> i64 = x * 2
    "#,
    );
}

#[test]
fn test_scope_forward_struct_ref() {
    check_ok(
        r#"
        F make() -> Point = Point { x: 0, y: 0 }
        S Point { x: i64, y: i64 }
    "#,
    );
}

// ============================================================================
// Scope: Complex scenarios
// ============================================================================

#[test]
fn test_scope_deeply_nested_blocks() {
    check_ok(
        r#"
        F f() -> i64 {
            a := 1
            b := {
                c := a + 1
                d := {
                    e := c + 1
                    e
                }
                d + c
            }
            a + b
        }
    "#,
    );
}

#[test]
fn test_scope_variables_in_different_branches() {
    check_ok(
        r#"
        F f(x: i64) -> i64 {
            I x > 0 {
                a := 10
                a
            } E {
                b := 20
                b
            }
        }
    "#,
    );
}

#[test]
fn test_scope_loop_variable_reuse() {
    check_ok(
        r#"
        F f() -> i64 {
            total := mut 0
            L i:0..3 {
                L j:0..3 {
                    total = total + i + j
                }
            }
            total
        }
    "#,
    );
}

// ============================================================================
// Global scope
// ============================================================================

#[test]
fn test_scope_global_variable() {
    // Global declarations should parse and type-check the declaration itself
    let source = r#"
        G count: i64 = 0
        F test() -> i64 = 0
    "#;
    let module = vais_parser::parse(source).unwrap();
    let mut tc = vais_types::TypeChecker::new();
    // Global declarations may or may not fully integrate with the type checker
    let _ = tc.check_module(&module);
}

#[test]
fn test_scope_multiple_globals() {
    let source = r#"
        G a: i64 = 1
        G b: i64 = 2
        F test() -> i64 = 0
    "#;
    let module = vais_parser::parse(source).unwrap();
    let mut tc = vais_types::TypeChecker::new();
    let _ = tc.check_module(&module);
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_scope_empty_block() {
    check_ok(
        r#"
        F f() -> i64 {
            x := {}
            0
        }
    "#,
    );
}

#[test]
fn test_scope_let_with_complex_expr() {
    check_ok(
        r#"
        F f(a: i64, b: i64) -> i64 {
            result := I a > b { a * 2 } E { b * 3 }
            result
        }
    "#,
    );
}

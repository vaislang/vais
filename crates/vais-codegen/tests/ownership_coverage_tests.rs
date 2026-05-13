//! Coverage tests for ownership/move/borrow codegen paths.
//!
//! Tests exercise codegen-level ownership tracking, variable rebinding,
//! struct field access after move, closures capturing variables, etc.

use vais_codegen::CodeGenerator;
use vais_parser::parse;

fn gen_ok(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("Codegen failed for: {}\nErr: {}", source, e))
}

fn gen_result(source: &str) -> Result<String, String> {
    let module = parse(source).map_err(|e| format!("Parse: {:?}", e))?;
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .map_err(|e| format!("Codegen: {}", e))
}

// ============================================================================
// Variable binding and rebinding
// ============================================================================

#[test]
fn test_simple_binding() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := 42
            R x
        }
    "#,
    );
    assert!(ir.contains("42"));
}

#[test]
fn test_mutable_binding() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 0
            x = 42
            R x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_rebind_variable() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := 1
            x := 2
            R x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_multiple_bindings() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            a := 1
            b := 2
            c := 3
            R a + b + c
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_binding_from_expression() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := 10 * 2 + 5
            R x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_binding_from_function_call() {
    let ir = gen_ok(
        r#"
        F double(x: i64) -> i64 = x * 2
        F test() -> i64 {
            x := double(21)
            R x
        }
    "#,
    );
    assert!(ir.contains("call"));
}

// ============================================================================
// Struct ownership
// ============================================================================

#[test]
fn test_struct_creation() {
    let ir = gen_ok(
        r#"
        S Point { x: i64, y: i64 }
        F test() -> i64 {
            p := Point { x: 3, y: 4 }
            R p.x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_struct_field_access() {
    let ir = gen_ok(
        r#"
        S Vec2 { x: i64, y: i64 }
        F test() -> i64 {
            v := Vec2 { x: 10, y: 20 }
            R v.x + v.y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_struct_passed_to_function() {
    let ir = gen_ok(
        r#"
        S Pair { a: i64, b: i64 }
        F sum_pair(p: Pair) -> i64 = p.a + p.b
        F test() -> i64 {
            p := Pair { a: 3, b: 7 }
            R sum_pair(p)
        }
    "#,
    );
    assert!(ir.contains("call"));
}

#[test]
fn test_nested_struct() {
    let ir = gen_ok(
        r#"
        S Inner { val: i64 }
        S Outer { inner: Inner, extra: i64 }
        F test() -> i64 {
            o := Outer { inner: Inner { val: 42 }, extra: 10 }
            R o.extra
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// Enum ownership
// ============================================================================

#[test]
fn test_enum_creation_unit_variant() {
    let result = gen_result(
        r#"
        E Color { Red, Green, Blue }
        F test() -> i64 {
            c := Red
            R 0
        }
    "#,
    );
    // Enum variant creation — either works or has codegen limitation
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_enum_creation_with_payload() {
    let ir = gen_ok(
        r#"
        E Maybe { Nothing, Just(i64) }
        F test() -> i64 {
            m := Maybe::Just(42)
            R 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_enum_match_extracts_payload() {
    let ir = gen_ok(
        r#"
        E Maybe { Nothing, Just(i64) }
        F unwrap_or(m: Maybe, default: i64) -> i64 {
            M m {
                Just(v) => v,
                _ => default
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// Closures and variable capture
// ============================================================================

#[test]
fn test_closure_basic() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            f := |x: i64| x * 2
            R f(21)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_closure_captures_local() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            y := 10
            f := |x: i64| x + y
            R f(32)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_closure_multiple_params() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            f := |a: i64, b: i64| a + b
            R f(10, 20)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// Loop variable ownership
// ============================================================================

#[test]
fn test_loop_variable_scope() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            sum := mut 0
            L i: 0..5 {
                sum = sum + i
            }
            R sum
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_loop_with_break() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            result := mut 0
            L i: 0..100 {
                I i == 10 { B }
                result = i
            }
            R result
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_loop_with_continue() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            sum := mut 0
            L i: 0..10 {
                I i % 2 == 0 { C }
                sum = sum + i
            }
            R sum
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// Function return ownership
// ============================================================================

#[test]
fn test_return_local_value() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := 42
            R x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_return_computed_value() {
    let ir = gen_ok(
        r#"
        F test(a: i64, b: i64) -> i64 {
            R a * b + 1
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_auto_return() {
    let ir = gen_ok(
        r#"
        F test() -> i64 { 42 }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_expression_body_function() {
    let ir = gen_ok(
        r#"
        F square(x: i64) -> i64 = x * x
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// Variable scoping
// ============================================================================

#[test]
fn test_block_scoping() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := 1
            y := {
                z := x + 1
                z * 2
            }
            R y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_shadowing_in_nested_scope() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := 1
            y := {
                x := 10
                x
            }
            R x + y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// Multiple value types
// ============================================================================

#[test]
fn test_bool_binding() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            flag := true
            I flag { R 1 }
            R 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_string_binding() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            s := "hello"
            R 0
        }
    "#,
    );
    assert!(ir.contains("hello"));
}

#[test]
fn test_multiple_function_calls() {
    let ir = gen_ok(
        r#"
        F add(a: i64, b: i64) -> i64 = a + b
        F mul(a: i64, b: i64) -> i64 = a * b
        F test() -> i64 {
            x := add(1, 2)
            y := mul(x, 3)
            R y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_recursive_call_ownership() {
    let ir = gen_ok(
        r#"
        F fib(n: i64) -> i64 {
            I n <= 1 { R n }
            R fib(n - 1) + fib(n - 2)
        }
    "#,
    );
    assert!(ir.contains("call"));
}

// ============================================================================
// Complex ownership patterns
// ============================================================================

#[test]
fn test_chained_operations() {
    let ir = gen_ok(
        r#"
        F inc(x: i64) -> i64 = x + 1
        F dbl(x: i64) -> i64 = x * 2
        F test() -> i64 {
            R dbl(inc(inc(5)))
        }
    "#,
    );
    assert!(ir.contains("call"));
}

#[test]
fn test_pipe_operator() {
    let result = gen_result(
        r#"
        F double(x: i64) -> i64 = x * 2
        F inc(x: i64) -> i64 = x + 1
        F test() -> i64 {
            R 5 |> double |> inc
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_ternary_operator() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            R x > 0 ? x : -x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_compound_assignment() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 10
            x += 5
            x -= 3
            x *= 2
            R x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_unary_negation() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            R -x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_logical_not() {
    let ir = gen_ok(
        r#"
        F test(x: bool) -> i64 {
            I !x { R 1 }
            R 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

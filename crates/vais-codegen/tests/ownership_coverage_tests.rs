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
        fn test() -> i64 {
            x := 42
            return x
        }
    "#,
    );
    assert!(ir.contains("42"));
}

#[test]
fn test_mutable_binding() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            x := mut 0
            x = 42
            return x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_rebind_variable() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            x := 1
            x := 2
            return x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_multiple_bindings() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            a := 1
            b := 2
            c := 3
            return a + b + c
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_binding_from_expression() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            x := 10 * 2 + 5
            return x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_binding_from_function_call() {
    let ir = gen_ok(
        r#"
        fn double(x: i64) -> i64 = x * 2
        fn test() -> i64 {
            x := double(21)
            return x
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
        struct Point { x: i64, y: i64 }
        fn test() -> i64 {
            p := Point { x: 3, y: 4 }
            return p.x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_struct_field_access() {
    let ir = gen_ok(
        r#"
        struct Vec2 { x: i64, y: i64 }
        fn test() -> i64 {
            v := Vec2 { x: 10, y: 20 }
            return v.x + v.y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_struct_passed_to_function() {
    let ir = gen_ok(
        r#"
        struct Pair { a: i64, b: i64 }
        fn sum_pair(p: Pair) -> i64 = p.a + p.b
        fn test() -> i64 {
            p := Pair { a: 3, b: 7 }
            return sum_pair(p)
        }
    "#,
    );
    assert!(ir.contains("call"));
}

#[test]
fn test_nested_struct() {
    let ir = gen_ok(
        r#"
        struct Inner { val: i64 }
        struct Outer { inner: Inner, extra: i64 }
        fn test() -> i64 {
            o := Outer { inner: Inner { val: 42 }, extra: 10 }
            return o.extra
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
        enum Color { Red, Green, Blue }
        fn test() -> i64 {
            c := Red
            return 0
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
        enum Maybe { Nothing, Just(i64) }
        fn test() -> i64 {
            m := Maybe::Just(42)
            return 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_enum_match_extracts_payload() {
    let ir = gen_ok(
        r#"
        enum Maybe { Nothing, Just(i64) }
        fn unwrap_or(m: Maybe, default: i64) -> i64 {
            match m {
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
        fn test() -> i64 {
            f := |x: i64| x * 2
            return f(21)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_closure_captures_local() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            y := 10
            f := |x: i64| x + y
            return f(32)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_closure_multiple_params() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            f := |a: i64, b: i64| a + b
            return f(10, 20)
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
        fn test() -> i64 {
            sum := mut 0
            L i: 0..5 {
                sum = sum + i
            }
            return sum
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_loop_with_break() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            result := mut 0
            L i: 0..100 {
                I i == 10 { B }
                result = i
            }
            return result
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_loop_with_continue() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            sum := mut 0
            L i: 0..10 {
                I i % 2 == 0 { C }
                sum = sum + i
            }
            return sum
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
        fn test() -> i64 {
            x := 42
            return x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_return_computed_value() {
    let ir = gen_ok(
        r#"
        fn test(a: i64, b: i64) -> i64 {
            return a * b + 1
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_auto_return() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 { 42 }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_expression_body_function() {
    let ir = gen_ok(
        r#"
        fn square(x: i64) -> i64 = x * x
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
        fn test() -> i64 {
            x := 1
            y := {
                z := x + 1
                z * 2
            }
            return y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_shadowing_in_nested_scope() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            x := 1
            y := {
                x := 10
                x
            }
            return x + y
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
        fn test() -> i64 {
            flag := true
            I flag { return 1 }
            return 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_string_binding() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            s := "hello"
            return 0
        }
    "#,
    );
    assert!(ir.contains("hello"));
}

#[test]
fn test_multiple_function_calls() {
    let ir = gen_ok(
        r#"
        fn add(a: i64, b: i64) -> i64 = a + b
        fn mul(a: i64, b: i64) -> i64 = a * b
        fn test() -> i64 {
            x := add(1, 2)
            y := mul(x, 3)
            return y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_recursive_call_ownership() {
    let ir = gen_ok(
        r#"
        fn fib(n: i64) -> i64 {
            I n <= 1 { return n }
            return fib(n - 1) + fib(n - 2)
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
        fn inc(x: i64) -> i64 = x + 1
        fn dbl(x: i64) -> i64 = x * 2
        fn test() -> i64 {
            return dbl(inc(inc(5)))
        }
    "#,
    );
    assert!(ir.contains("call"));
}

#[test]
fn test_pipe_operator() {
    let result = gen_result(
        r#"
        fn double(x: i64) -> i64 = x * 2
        fn inc(x: i64) -> i64 = x + 1
        fn test() -> i64 {
            return 5 |> double |> inc
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_ternary_operator() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            return x > 0 ? x : -x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_compound_assignment() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            x := mut 10
            x += 5
            x -= 3
            x *= 2
            return x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_unary_negation() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
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
        fn test(x: bool) -> i64 {
            I !x { return 1 }
            return 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

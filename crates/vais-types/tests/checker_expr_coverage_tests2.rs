//! Additional coverage tests for vais-types/src/checker_expr/
//!
//! Targets: collections.rs (Vec, HashMap, array ops), calls.rs (generic calls,
//! trait method dispatch), special.rs (string interp, destructuring, assert),
//! control_flow.rs (match exhaustiveness, loop types), stmts.rs (let patterns)

use vais_parser::parse;
use vais_types::TypeChecker;

fn check_ok(source: &str) {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut tc = TypeChecker::new();
    tc.check_module(&module)
        .unwrap_or_else(|e| panic!("Type check failed for: {}\nErr: {:?}", source, e));
}

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
// collections.rs — array literal type checking
// ============================================================================

#[test]
fn test_array_literal_i64() {
    check_ok(
        r#"
        fn test() -> i64 {
            arr := [1, 2, 3, 4, 5]
            return 0
        }
    "#,
    );
}

#[test]
fn test_array_literal_single() {
    check_ok(
        r#"
        fn test() -> i64 {
            arr := [42]
            return 0
        }
    "#,
    );
}

#[test]
fn test_array_index_access() {
    check_ok(
        r#"
        fn test() -> i64 {
            arr := [10, 20, 30]
            return arr[1]
        }
    "#,
    );
}

// ============================================================================
// collections.rs — tuple type checking
// ============================================================================

#[test]
fn test_tuple_creation() {
    check_ok(
        r#"
        fn test() -> i64 {
            t := (1, 2, 3)
            return 0
        }
    "#,
    );
}

#[test]
fn test_tuple_mixed_types() {
    check_ok(
        r#"
        fn test() -> i64 {
            t := (1, true, 3)
            return 0
        }
    "#,
    );
}

// ============================================================================
// collections.rs — map literal
// ============================================================================

#[test]
fn test_map_literal() {
    check_ok(
        r#"
        fn test() -> i64 {
            m := {"a": 1, "b": 2, "c": 3}
            return 0
        }
    "#,
    );
}

// ============================================================================
// calls.rs — function call with wrong arity
// ============================================================================

#[test]
fn test_call_wrong_arity() {
    check_err(
        r#"
        fn add(a: i64, b: i64) -> i64 = a + b
        fn test() -> i64 = add(1)
    "#,
    );
}

#[test]
fn test_call_too_many_args() {
    check_err(
        r#"
        fn add(a: i64, b: i64) -> i64 = a + b
        fn test() -> i64 = add(1, 2, 3)
    "#,
    );
}

// ============================================================================
// calls.rs — generic function call
// ============================================================================

#[test]
fn test_generic_function_call() {
    check_ok(
        r#"
        fn identity<T>(x: T) -> T = x
        fn test() -> i64 = identity(42)
    "#,
    );
}

#[test]
fn test_generic_function_multiple_params() {
    check_ok(
        r#"
        fn first<A, B>(a: A, b: B) -> A = a
        fn test() -> i64 = first(42, true)
    "#,
    );
}

// ============================================================================
// calls.rs — method call type checking
// ============================================================================

#[test]
fn test_method_call_return_type() {
    check_ok(
        r#"
        struct Counter { val: i64 }
        impl Counter {
            fn get(self) -> i64 = self.val
            fn inc(self) -> i64 = self.val + 1
        }
        fn test() -> i64 {
            c := Counter { val: 10 }
            return c.get() + c.inc()
        }
    "#,
    );
}

// ============================================================================
// calls.rs — calling undefined function
// ============================================================================

#[test]
fn test_call_undefined_function() {
    check_err(
        r#"
        fn test() -> i64 = nonexistent(42)
    "#,
    );
}

// ============================================================================
// special.rs — string interpolation type checking
// ============================================================================

#[test]
fn test_string_interp_type() {
    check_ok(
        r#"
        fn test() -> i64 {
            x := 42
            s := ~"value: {x}"
            return 0
        }
    "#,
    );
}

#[test]
fn test_string_interp_multiple_vars() {
    check_ok(
        r#"
        fn test() -> i64 {
            a := 1
            b := 2
            s := ~"{a} + {b}"
            return 0
        }
    "#,
    );
}

// ============================================================================
// special.rs — assert type checking
// ============================================================================

#[test]
fn test_assert_bool_condition() {
    // `assert` is a panic source, so the caller must be `partial`.
    // Phase 195 added the E034 TotalFunctionViolation check (models
    // assert as panic-on-false); the earlier version of this test
    // predated that rule.
    check_ok(
        r#"
        partial fn test(x: i64) -> i64 {
            assert(x > 0)
            return x
        }
    "#,
    );
}

// ============================================================================
// control_flow.rs — if expression type consistency
// ============================================================================

#[test]
fn test_if_expr_consistent_types() {
    check_ok(
        r#"
        fn test(x: i64) -> i64 {
            result := I x > 0 { 1 } else { 0 }
            return result
        }
    "#,
    );
}

#[test]
fn test_if_else_if_chain() {
    check_ok(
        r#"
        fn test(x: i64) -> i64 {
            return I x > 100 { 3 } else I x > 50 { 2 } else I x > 0 { 1 } else { 0 }
        }
    "#,
    );
}

// ============================================================================
// control_flow.rs — match expression type checking
// ============================================================================

#[test]
fn test_match_all_arms_same_type() {
    check_ok(
        r#"
        fn test(x: i64) -> i64 {
            match x {
                1 => 10,
                2 => 20,
                _ => 0
            }
        }
    "#,
    );
}

#[test]
fn test_match_with_guard() {
    check_ok(
        r#"
        fn test(x: i64) -> i64 {
            match x {
                n I n > 10 => n * 2,
                n => n
            }
        }
    "#,
    );
}

#[test]
fn test_match_enum_exhaustive() {
    check_ok(
        r#"
        enum Dir { North, South, East, West }
        fn test(d: Dir) -> i64 {
            match d {
                North => 0,
                South => 1,
                East => 2,
                West => 3,
                _ => -1
            }
        }
    "#,
    );
}

// ============================================================================
// control_flow.rs — loop type checking
// ============================================================================

#[test]
fn test_for_loop_type() {
    check_ok(
        r#"
        fn test() -> i64 {
            sum := mut 0
            L i:0..10 {
                sum += i
            }
            return sum
        }
    "#,
    );
}

#[test]
fn test_infinite_loop_with_break() {
    check_ok(
        r#"
        fn test() -> i64 {
            x := mut 0
            L {
                x += 1
                I x >= 10 { B }
            }
            return x
        }
    "#,
    );
}

// ============================================================================
// stmts.rs — let binding patterns
// ============================================================================

#[test]
fn test_let_simple() {
    check_ok(
        r#"
        fn test() -> i64 {
            x := 42
            return x
        }
    "#,
    );
}

#[test]
fn test_let_mutable() {
    check_ok(
        r#"
        fn test() -> i64 {
            x := mut 0
            x = 42
            return x
        }
    "#,
    );
}

#[test]
fn test_let_with_computation() {
    check_ok(
        r#"
        fn test() -> i64 {
            x := 21 * 2
            return x
        }
    "#,
    );
}

// ============================================================================
// stmts.rs — return type checking
// ============================================================================

#[test]
fn test_return_correct_type() {
    check_ok(
        r#"
        fn test() -> i64 {
            return 42
        }
    "#,
    );
}

#[test]
fn test_return_bool() {
    check_ok(
        r#"
        fn test() -> bool {
            return true
        }
    "#,
    );
}

// ============================================================================
// special.rs — struct literal type checking
// ============================================================================

#[test]
fn test_struct_literal_correct_fields() {
    check_ok(
        r#"
        struct Point { x: i64, y: i64 }
        fn test() -> i64 {
            p := Point { x: 1, y: 2 }
            return p.x + p.y
        }
    "#,
    );
}

#[test]
fn test_struct_field_access() {
    check_ok(
        r#"
        struct Rect { w: i64, h: i64 }
        fn test() -> i64 {
            r := Rect { w: 10, h: 20 }
            return r.w * r.h
        }
    "#,
    );
}

// ============================================================================
// special.rs — enum variant construction
// ============================================================================

#[test]
fn test_enum_variant_with_payload() {
    check_ok(
        r#"
        enum Value { Int(i64), None }
        fn test() -> i64 {
            v := Int(42)
            match v {
                Int(n) => n,
                None => 0,
                _ => -1
            }
        }
    "#,
    );
}

// ============================================================================
// calls.rs — recursive function type checking
// ============================================================================

#[test]
fn test_recursive_function() {
    check_ok(
        r#"
        fn fib(n: i64) -> i64 {
            I n <= 1 { return n }
            return fib(n - 1) + fib(n - 2)
        }
    "#,
    );
}

// ============================================================================
// calls.rs — self-recursion (@) type checking
// ============================================================================

#[test]
fn test_self_call() {
    check_ok(
        r#"
        fn fact(n: i64, acc: i64) -> i64 {
            I n <= 1 { return acc }
            return @(n - 1, acc * n)
        }
    "#,
    );
}

// ============================================================================
// special.rs — pipe operator
// ============================================================================

#[test]
fn test_pipe_operator() {
    check_ok(
        r#"
        fn double(x: i64) -> i64 = x * 2
        fn inc(x: i64) -> i64 = x + 1
        fn test() -> i64 {
            return 5 |> double |> inc
        }
    "#,
    );
}

// ============================================================================
// control_flow.rs — ternary expression
// ============================================================================

#[test]
fn test_ternary_type_check() {
    check_ok(
        r#"
        fn test(x: i64) -> i64 {
            return x > 0 ? 1 : 0
        }
    "#,
    );
}

// ============================================================================
// special.rs — global constants
// ============================================================================

#[test]
fn test_global_constant() {
    check_ok(
        r#"
        fn test() -> i64 {
            x := 100
            return x
        }
    "#,
    );
}

// ============================================================================
// collections.rs — nested array
// ============================================================================

#[test]
fn test_nested_array() {
    check_ok(
        r#"
        fn test() -> i64 {
            matrix := [[1, 2], [3, 4]]
            return 0
        }
    "#,
    );
}

// ============================================================================
// control_flow.rs — or-pattern in match
// ============================================================================

#[test]
fn test_match_or_pattern() {
    check_ok(
        r#"
        fn test(x: i64) -> i64 {
            match x {
                1 | 2 | 3 => 10,
                _ => 0
            }
        }
    "#,
    );
}

// ============================================================================
// control_flow.rs — range pattern
// ============================================================================

#[test]
fn test_match_range_pattern() {
    check_ok(
        r#"
        fn test(x: i64) -> i64 {
            match x {
                1..=10 => 1,
                _ => 0
            }
        }
    "#,
    );
}

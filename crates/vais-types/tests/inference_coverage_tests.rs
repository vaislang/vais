//! Comprehensive inference coverage tests
//!
//! Targets uncovered lines in inference.rs (336 uncovered, 52% coverage)
//! Focus: CheckMode, type inference edge cases, bidirectional type checking,
//! generic instantiation paths, and error paths

use vais_parser::parse;
use vais_types::{CheckMode, ResolvedType, TypeChecker};

// ============================================================================
// CheckMode tests (inference.rs lines 28-47)
// ============================================================================

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
fn test_check_mode_check_with_different_types() {
    let mode_i64 = CheckMode::check(ResolvedType::I64);
    let mode_f64 = CheckMode::check(ResolvedType::F64);
    let mode_bool = CheckMode::check(ResolvedType::Bool);
    let mode_str = CheckMode::check(ResolvedType::Str);

    assert_eq!(mode_i64.expected(), Some(&ResolvedType::I64));
    assert_eq!(mode_f64.expected(), Some(&ResolvedType::F64));
    assert_eq!(mode_bool.expected(), Some(&ResolvedType::Bool));
    assert_eq!(mode_str.expected(), Some(&ResolvedType::Str));
}

// ============================================================================
// Type inference through various expressions
// ============================================================================

#[test]
fn test_infer_integer_literal() {
    let source = "F test() -> i64 = 42";
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_float_literal() {
    let source = "F test() -> f64 = 3.14";
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_bool_literal() {
    let source = "F test() -> bool = true";
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_string_literal() {
    let source = r#"F test() -> str = "hello""#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_binary_op_addition() {
    let source = "F test() -> i64 { x := 1 + 2; R x }";
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_binary_op_comparison() {
    let source = "F test() -> bool { x := 1 < 2; R x }";
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_binary_op_logical() {
    let source = "F test() -> bool { x := true && false; R x }";
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_unary_op_negate() {
    let source = "F test() -> i64 { x := -42; R x }";
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_unary_op_not() {
    let source = "F test() -> bool { x := !true; R x }";
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

// ============================================================================
// Complex inference scenarios
// ============================================================================

#[test]
fn test_infer_if_expression() {
    let source = r#"
        F test(x: i64) -> i64 {
            result := I x > 0 { x } E { -x }
            result
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_match_expression() {
    let source = r#"
        F test(x: i64) -> i64 {
            M x {
                0 => 100,
                1 => 200,
                _ => 300
            }
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_block_expression() {
    let source = r#"
        F test() -> i64 {
            result := {
                a := 10
                b := 20
                a + b
            }
            result
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_nested_function_call() {
    let source = r#"
        F add(x: i64, y: i64) -> i64 = x + y
        F mul(x: i64, y: i64) -> i64 = x * y
        F test() -> i64 = add(mul(2, 3), mul(4, 5))
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_generic_identity() {
    let source = r#"
        F id<T>(x: T) -> T = x
        F test() -> i64 = id(42)
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_generic_pair() {
    let source = r#"
        S Pair<T> { first: T, second: T }
        F test() -> i64 {
            p := Pair { first: 1, second: 2 }
            p.first + p.second
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_closure_type() {
    // Closures used as higher-order function params
    let source = r#"
        F test() -> i64 {
            f := |x: i64| x * 2
            f(21)
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_ternary() {
    let source = "F test(x: i64) -> i64 = x > 0 ? x : -x";
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_tuple() {
    let source = r#"
        F test() -> i64 {
            t := (1, 2, 3)
            R 0
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_array() {
    let source = r#"
        F test() -> i64 {
            arr := [1, 2, 3]
            R 0
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

// ============================================================================
// Type error paths
// ============================================================================

#[test]
fn test_type_mismatch_error() {
    // Phase 160-A: bool↔i64 numeric promotion is now allowed.
    // Use str→i64 mismatch which is still forbidden.
    let source = r#"
        F test() -> i64 {
            x: str = 42
            R 0
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    let result = tc.check_module(&module);
    // Should produce a type error
    assert!(result.is_err() || !tc.get_collected_errors().is_empty());
}

#[test]
fn test_return_type_mismatch() {
    // Phase 160-A: bool↔i64 numeric promotion is now allowed.
    // Use str→i64 mismatch which is still forbidden.
    let source = r#"F test() -> i64 = "hello""#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    let result = tc.check_module(&module);
    assert!(result.is_err() || !tc.get_collected_errors().is_empty());
}

// ============================================================================
// Struct and enum type inference
// ============================================================================

#[test]
fn test_infer_struct_construction() {
    let source = r#"
        S Point { x: i64, y: i64 }
        F test() -> i64 {
            p := Point { x: 1, y: 2 }
            p.x + p.y
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_enum_variant() {
    let source = r#"
        E MyOpt { MySome(i64), MyNone }
        F test() -> i64 {
            x := MySome(42)
            M x {
                MySome(v) => v,
                MyNone => 0,
                _ => 0
            }
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

// ============================================================================
// Trait inference
// ============================================================================

#[test]
fn test_trait_method_inference() {
    let source = r#"
        W Describable {
            F describe(self) -> str
        }
        S Circle { radius: i64 }
        X Circle: Describable {
            F describe(self) -> str = "circle"
        }
        F test() -> str {
            c := Circle { radius: 5 }
            c.describe()
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

// ============================================================================
// Compound expressions
// ============================================================================

#[test]
fn test_infer_assign_op() {
    let source = r#"
        F test() -> i64 {
            x := mut 10
            x += 5
            x -= 1
            x *= 2
            x /= 3
            x
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_cast() {
    let source = r#"
        F test() -> f64 {
            x := 42
            x as f64
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_range() {
    let source = r#"
        F test() -> i64 {
            L i:0..10 {
                C
            }
            R 0
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_method_chain() {
    let source = r#"
        S Builder { value: i64 }
        X Builder {
            F set(self, v: i64) -> Builder = Builder { value: v }
            F get(self) -> i64 = self.value
        }
        F test() -> i64 {
            b := Builder { value: 0 }
            b.set(42).get()
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_recursive_function() {
    let source = r#"
        F factorial(n: i64) -> i64 {
            I n <= 1 { R 1 }
            R n * @(n - 1)
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_mutually_independent_functions() {
    let source = r#"
        F is_even(n: i64) -> bool = n % 2 == 0
        F is_odd(n: i64) -> bool = !is_even(n)
        F test() -> bool = is_odd(3)
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

// ============================================================================
// Edge cases for inference
// ============================================================================

#[test]
fn test_infer_empty_function_body() {
    let source = "F test() -> i64 = 0";
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_unit_function() {
    let source = r#"
        F test() {
            x := 42
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_nested_generics() {
    let source = r#"
        F first<T>(x: T, y: T) -> T = x
        F test() -> i64 = first(first(1, 2), first(3, 4))
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_multiple_type_params() {
    let source = r#"
        F pick_first<A, B>(a: A, b: B) -> A = a
        F test() -> i64 = pick_first(42, true)
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_with_where_clause() {
    let source = r#"
        W Printable { F to_str(self) -> str }
        F show<T>(x: T) -> str where T: Printable = x.to_str()
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_infer_bitwise_ops() {
    let source = r#"
        F test() -> i64 {
            a := 255
            b := 15
            x := a & b
            y := a | b
            z := a ^ b
            w := a << 4
            v := a >> 4
            x + y + z + w + v
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

// ============================================================================
// resolve.rs coverage - type resolution edge cases
// ============================================================================

#[test]
fn test_resolve_nested_generic_struct() {
    let source = r#"
        S Inner<T> { value: T }
        S Outer<T> { inner: Inner<T> }
        F test() -> i64 {
            o := Outer { inner: Inner { value: 42 } }
            o.inner.value
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_resolve_type_alias_in_struct() {
    let source = r#"
        T Number = i64
        S Container { value: Number }
        F test() -> Number {
            c := Container { value: 42 }
            c.value
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

#[test]
fn test_resolve_enum_with_complex_variants() {
    let source = r#"
        E Expr {
            Num(i64),
            Add(i64, i64),
            Neg(i64)
        }
        F eval(e: Expr) -> i64 {
            M e {
                Num(n) => n,
                Add(a, b) => a + b,
                Neg(n) => 0 - n,
                _ => 0
            }
        }
    "#;
    let module = parse(source).unwrap();
    let mut tc = TypeChecker::new();
    assert!(tc.check_module(&module).is_ok());
}

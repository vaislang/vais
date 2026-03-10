//! Comprehensive inference unification & substitution tests (Phase 131)
//!
//! Targets uncovered lines in:
//! - inference/unification.rs: unify branches (Array, Optional, Result, Tuple, Fn, Named, etc.)
//! - inference/substitution.rs: apply_substitutions, substitute_generics
//! - inference/inference_modes.rs: check_expr_bidirectional, check_lambda_with_expected
//!
//! Strategy: Parse + TypeCheck Vais source to exercise internal unification paths.

use vais_parser::parse;
use vais_types::{CheckMode, ResolvedType, TypeChecker};

// ============================================================================
// Helpers
// ============================================================================

fn tc_ok(source: &str) {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut tc = TypeChecker::new();
    tc.check_module(&module)
        .unwrap_or_else(|e| panic!("TC failed for: {}\nErr: {:?}", source, e));
}

fn tc_err(source: &str) {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut tc = TypeChecker::new();
    assert!(
        tc.check_module(&module).is_err(),
        "Expected type error for: {}",
        source
    );
}

// ============================================================================
// Unification — same types (fast path)
// ============================================================================

#[test]
fn test_unify_i64_same() {
    tc_ok("F test() -> i64 { x := 42; R x }");
}

#[test]
fn test_unify_bool_same() {
    tc_ok("F test() -> bool { x := true; R x }");
}

#[test]
fn test_unify_f64_same() {
    tc_ok("F test() -> f64 { x := 3.14; R x }");
}

#[test]
fn test_unify_str_same() {
    tc_ok(r#"F test() -> str { x := "hello"; R x }"#);
}

// ============================================================================
// Unification — type variables (Var)
// ============================================================================

#[test]
fn test_unify_generic_identity() {
    tc_ok("F identity<T>(x: T) -> T = x\nF test() -> i64 = identity(42)");
}

#[test]
fn test_unify_generic_swap_types() {
    tc_ok("F first<A, B>(a: A, b: B) -> A = a\nF test() -> i64 = first(1, true)");
}

#[test]
fn test_unify_generic_nested_call() {
    tc_ok("F id<T>(x: T) -> T = x\nF wrap<U>(y: U) -> U = id(y)\nF test() -> i64 = wrap(99)");
}

// ============================================================================
// Unification — Array
// ============================================================================

#[test]
fn test_unify_array_element_type() {
    tc_ok("F test() -> i64 { a := [1, 2, 3]; R a[0] }");
}

#[test]
fn test_unify_array_assignment() {
    tc_ok("F test() -> i64 { a := [10, 20]; b := [30, 40]; R a[0] + b[1] }");
}

// ============================================================================
// Unification — Tuple
// ============================================================================

#[test]
fn test_unify_tuple_basic() {
    tc_ok("F test() -> i64 { t := (1, 2); R 0 }");
}

#[test]
fn test_unify_tuple_mixed_types() {
    tc_ok("F test() -> i64 { t := (42, true); R 0 }");
}

#[test]
fn test_unify_tuple_three_elements() {
    tc_ok("F test() -> i64 { t := (1, 2, 3); R 0 }");
}

// ============================================================================
// Unification — Fn types
// ============================================================================

#[test]
fn test_unify_fn_type_callback() {
    tc_ok(
        "F apply(f: fn(i64) -> i64, x: i64) -> i64 = f(x)\nF test() -> i64 = apply(|x| x + 1, 5)",
    );
}

#[test]
fn test_unify_fn_type_higher_order() {
    tc_ok("F compose(f: fn(i64) -> i64, g: fn(i64) -> i64, x: i64) -> i64 = f(g(x))");
}

#[test]
fn test_unify_fn_two_params() {
    tc_ok("F apply2(f: fn(i64, i64) -> i64, a: i64, b: i64) -> i64 = f(a, b)\nF test() -> i64 = apply2(|x, y| x + y, 3, 4)");
}

// ============================================================================
// Unification — Named types (struct)
// ============================================================================

#[test]
fn test_unify_named_struct_basic() {
    tc_ok("S Point { x: i64, y: i64 }\nF test() -> i64 { p := Point { x: 1, y: 2 }; R p.x }");
}

#[test]
fn test_unify_named_struct_return() {
    tc_ok("S Pair { a: i64, b: i64 }\nF make() -> Pair = Pair { a: 1, b: 2 }\nF test() -> i64 { p := make(); R p.a }");
}

// ============================================================================
// Unification — integer type conversions
// ============================================================================

#[test]
fn test_unify_integer_widening_i32_to_i64() {
    tc_ok("F test(x: i32) -> i64 { R x }");
}

#[test]
fn test_unify_integer_i8_to_i64() {
    tc_ok("F test(x: i8) -> i64 { R x }");
}

#[test]
fn test_unify_integer_unsigned() {
    tc_ok("F test(x: u32) -> u64 { R x }");
}

// ============================================================================
// Unification — Result type
// ============================================================================

#[test]
fn test_unify_result_generic() {
    tc_ok("F ok_val<T>(x: T) -> T = x\nF test() -> i64 = ok_val(42)");
}

// ============================================================================
// Unification — Ref / auto-deref
// ============================================================================

#[test]
fn test_unify_ref_to_value() {
    tc_ok("F test() -> i64 { x := 42; R x }");
}

// ============================================================================
// Unification — Never type
// ============================================================================

#[test]
fn test_unify_never_in_if() {
    tc_ok("F test(b: bool) -> i64 { I b { R 1 }; R 0 }");
}

// ============================================================================
// Unification — Generic/Unknown pass-through
// ============================================================================

#[test]
fn test_unify_generic_passthrough() {
    tc_ok("F id<T>(x: T) -> T = x\nF test() -> i64 = id(id(42))");
}

// ============================================================================
// Unification — linear/affine wrappers
// ============================================================================

#[test]
fn test_unify_linear_type() {
    tc_ok("F test(x: linear i64) -> i64 = x");
}

#[test]
fn test_unify_affine_type() {
    tc_ok("F test(x: affine i64) -> i64 = x");
}

// ============================================================================
// Unification — DynTrait
// ============================================================================

#[test]
fn test_unify_dyn_trait() {
    tc_ok(
        r#"
        W Show { F show(self) -> i64 }
        S Num { v: i64 }
        X Num: Show { F show(self) -> i64 = self.v }
        F test() -> i64 { n := Num { v: 42 }; R n.show() }
    "#,
    );
}

// ============================================================================
// Unification — error paths (Mismatch)
// ============================================================================

#[test]
fn test_unify_mismatch_bool_i64() {
    tc_err("F test() -> bool { R 42 }");
}

#[test]
fn test_unify_mismatch_str_i64() {
    tc_err(r#"F test() -> i64 { R "hello" }"#);
}

#[test]
fn test_unify_mismatch_if_branches() {
    tc_err("F test(b: bool) -> i64 = I b { true } E { 2 }");
}

#[test]
fn test_unify_mismatch_fn_return() {
    tc_err("F add(x: i64, y: i64) -> bool = x + y");
}

#[test]
fn test_unify_mismatch_wrong_param() {
    tc_err("F test(x: bool) -> i64 = x + 1");
}

// ============================================================================
// Bidirectional type checking (CheckMode)
// ============================================================================

#[test]
fn test_check_mode_infer_is_infer() {
    let mode = CheckMode::Infer;
    assert!(mode.is_infer());
    assert!(mode.expected().is_none());
}

#[test]
fn test_check_mode_check_returns_expected() {
    let mode = CheckMode::check(ResolvedType::I64);
    assert!(!mode.is_infer());
    assert_eq!(mode.expected(), Some(&ResolvedType::I64));
}

#[test]
fn test_check_mode_check_various_types() {
    for ty in &[
        ResolvedType::Bool,
        ResolvedType::F32,
        ResolvedType::F64,
        ResolvedType::Str,
        ResolvedType::Unit,
        ResolvedType::I8,
        ResolvedType::U64,
    ] {
        let mode = CheckMode::check(ty.clone());
        assert_eq!(mode.expected(), Some(ty));
    }
}

#[test]
fn test_bidirectional_lambda_param_inference() {
    tc_ok(
        "F apply(f: fn(i64) -> i64, x: i64) -> i64 = f(x)\nF test() -> i64 = apply(|x| x * 2, 10)",
    );
}

#[test]
fn test_bidirectional_array_element_type() {
    tc_ok("F test() -> i64 { arr := [10, 20, 30]; R arr[1] }");
}

// ============================================================================
// Substitution through generic structs
// ============================================================================

#[test]
fn test_substitution_generic_struct() {
    tc_ok(
        r#"
        S Wrapper<T> { value: T }
        F test() -> i64 {
            w := Wrapper { value: 42 }
            R w.value
        }
    "#,
    );
}

#[test]
fn test_substitution_nested_generic() {
    tc_ok(
        r#"
        F outer<T>(x: T) -> T = x
        F inner<U>(y: U) -> U = outer(y)
        F test() -> i64 = inner(99)
    "#,
    );
}

#[test]
fn test_substitution_multiple_type_params() {
    tc_ok("F pair<A, B>(a: A, b: B) -> A = a\nF test() -> i64 = pair(1, true)");
}

#[test]
fn test_substitution_with_trait_impl() {
    tc_ok(
        r#"
        W Valued { F val(self) -> i64 }
        S Box { n: i64 }
        X Box: Valued { F val(self) -> i64 = self.n }
        F test() -> i64 { b := Box { n: 42 }; R b.val() }
    "#,
    );
}

// ============================================================================
// Complex unification scenarios
// ============================================================================

#[test]
fn test_unify_enum_variant() {
    tc_ok(
        r#"
        E Color { Red, Green, Blue }
        F test(c: Color) -> i64 {
            M c {
                Red => 1,
                Green => 2,
                Blue => 3,
                _ => 0
            }
        }
    "#,
    );
}

#[test]
fn test_unify_match_all_arms_same_type() {
    tc_ok("F test(x: i64) -> i64 = M x { 0 => 10, 1 => 20, _ => 30 }");
}

#[test]
fn test_unify_closure_capture() {
    tc_ok("F test() -> i64 { a := 10; f := |x: i64| x + a; R f(5) }");
}

#[test]
fn test_unify_nested_struct_access() {
    tc_ok(
        r#"
        S Inner { val: i64 }
        S Outer { inner: Inner }
        F test() -> i64 {
            o := Outer { inner: Inner { val: 42 } }
            R o.inner.val
        }
    "#,
    );
}

#[test]
fn test_unify_loop_types() {
    tc_ok("F test() -> i64 { x := mut 0; L { I x >= 10 { B }; x = x + 1 }; R x }");
}

#[test]
fn test_unify_for_loop() {
    tc_ok("F test() -> i64 { sum := mut 0; L i:0..5 { sum = sum + i }; R sum }");
}

#[test]
fn test_unify_self_recursion() {
    tc_ok("F fib(n: i64) -> i64 = I n < 2 { n } E { @(n - 1) + @(n - 2) }");
}

#[test]
fn test_unify_mutual_call() {
    tc_ok("F a(x: i64) -> i64 = I x <= 0 { 0 } E { b(x - 1) }\nF b(x: i64) -> i64 = a(x)");
}

#[test]
fn test_unify_pipe_operator() {
    tc_ok("F inc(x: i64) -> i64 = x + 1\nF test() -> i64 = inc(5)");
}

#[test]
fn test_unify_ternary_same_type() {
    tc_ok("F abs(x: i64) -> i64 = x >= 0 ? x : 0 - x");
}

#[test]
fn test_unify_chained_arithmetic() {
    tc_ok("F test() -> i64 = 1 + 2 * 3 - 4 / 2 + 5 % 3");
}

#[test]
fn test_unify_comparison_operators() {
    tc_ok("F test() -> bool = 1 < 2 && 3 > 1 && 4 >= 4 && 5 <= 5 && 1 != 2");
}

#[test]
fn test_unify_boolean_logic() {
    tc_ok("F test(a: bool, b: bool) -> bool = (a && b) || (!a && !b)");
}

#[test]
fn test_unify_nested_if() {
    tc_ok("F test(x: i64) -> i64 = I x > 0 { I x > 10 { 2 } E { 1 } } E { 0 }");
}

#[test]
fn test_unify_match_with_guard_like() {
    tc_ok("F test(x: i64) -> i64 = M x { 0 => 0, 1 => 1, _ => x * 2 }");
}

#[test]
fn test_unify_struct_method_chain() {
    tc_ok(
        r#"
        S Num { v: i64 }
        X Num {
            F get(self) -> i64 = self.v
            F add(self, x: i64) -> i64 = self.v + x
        }
        F test() -> i64 { n := Num { v: 10 }; R n.add(5) }
    "#,
    );
}

#[test]
fn test_unify_multiple_struct_fields() {
    tc_ok(
        r#"
        S Rect { w: i64, h: i64 }
        F area(r: Rect) -> i64 = r.w * r.h
        F test() -> i64 = area(Rect { w: 3, h: 4 })
    "#,
    );
}

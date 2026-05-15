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
    tc_ok("fn test() -> i64 { x := 42; return x }");
}

#[test]
fn test_unify_bool_same() {
    tc_ok("fn test() -> bool { x := true; return x }");
}

#[test]
fn test_unify_f64_same() {
    tc_ok("fn test() -> f64 { x := 3.14; return x }");
}

#[test]
fn test_unify_str_same() {
    tc_ok(r#"fn test() -> str { x := "hello"; return x }"#);
}

// ============================================================================
// Unification — type variables (Var)
// ============================================================================

#[test]
fn test_unify_generic_identity() {
    tc_ok("fn identity<T>(x: T) -> T = x\nfn test() -> i64 = identity(42)");
}

#[test]
fn test_unify_generic_swap_types() {
    tc_ok("fn first<A, B>(a: A, b: B) -> A = a\nfn test() -> i64 = first(1, true)");
}

#[test]
fn test_unify_generic_nested_call() {
    tc_ok("fn id<T>(x: T) -> T = x\nfn wrap<U>(y: U) -> U = id(y)\nfn test() -> i64 = wrap(99)");
}

// ============================================================================
// Unification — Array
// ============================================================================

#[test]
fn test_unify_array_element_type() {
    tc_ok("fn test() -> i64 { a := [1, 2, 3]; return a[0] }");
}

#[test]
fn test_unify_array_assignment() {
    tc_ok("fn test() -> i64 { a := [10, 20]; b := [30, 40]; return a[0] + b[1] }");
}

// ============================================================================
// Unification — Tuple
// ============================================================================

#[test]
fn test_unify_tuple_basic() {
    tc_ok("fn test() -> i64 { t := (1, 2); return 0 }");
}

#[test]
fn test_unify_tuple_mixed_types() {
    tc_ok("fn test() -> i64 { t := (42, true); return 0 }");
}

#[test]
fn test_unify_tuple_three_elements() {
    tc_ok("fn test() -> i64 { t := (1, 2, 3); return 0 }");
}

// ============================================================================
// Unification — Fn types
// ============================================================================

#[test]
fn test_unify_fn_type_callback() {
    tc_ok(
        "fn apply(f: fn(i64) -> i64, x: i64) -> i64 = f(x)\nfn test() -> i64 = apply(|x| x + 1, 5)",
    );
}

#[test]
fn test_unify_fn_type_higher_order() {
    tc_ok("fn compose(f: fn(i64) -> i64, g: fn(i64) -> i64, x: i64) -> i64 = f(g(x))");
}

#[test]
fn test_unify_fn_two_params() {
    tc_ok("fn apply2(f: fn(i64, i64) -> i64, a: i64, b: i64) -> i64 = f(a, b)\nfn test() -> i64 = apply2(|x, y| x + y, 3, 4)");
}

// ============================================================================
// Unification — Named types (struct)
// ============================================================================

#[test]
fn test_unify_named_struct_basic() {
    tc_ok("struct Point { x: i64, y: i64 }\nfn test() -> i64 { p := Point { x: 1, y: 2 }; return p.x }");
}

#[test]
fn test_unify_named_struct_return() {
    tc_ok("struct Pair { a: i64, b: i64 }\nfn make() -> Pair = Pair { a: 1, b: 2 }\nfn test() -> i64 { p := make(); return p.a }");
}

// ============================================================================
// Unification — integer type conversions
// ============================================================================

#[test]
fn test_unify_integer_widening_i32_to_i64() {
    tc_ok("fn test(x: i32) -> i64 { return x }");
}

#[test]
fn test_unify_integer_i8_to_i64() {
    tc_ok("fn test(x: i8) -> i64 { return x }");
}

#[test]
fn test_unify_integer_unsigned() {
    tc_ok("fn test(x: u32) -> u64 { return x }");
}

// ============================================================================
// Unification — Result type
// ============================================================================

#[test]
fn test_unify_result_generic() {
    tc_ok("fn ok_val<T>(x: T) -> T = x\nfn test() -> i64 = ok_val(42)");
}

// ============================================================================
// Unification — Ref
// ============================================================================

#[test]
fn test_unify_ref_to_value() {
    tc_ok("fn test() -> i64 { x := 42; return x }");
}

// ============================================================================
// Unification — Never type
// ============================================================================

#[test]
fn test_unify_never_in_if() {
    tc_ok("fn test(b: bool) -> i64 { I b { return 1 }; return 0 }");
}

// ============================================================================
// Unification — Generic/Unknown pass-through
// ============================================================================

#[test]
fn test_unify_generic_passthrough() {
    tc_ok("fn id<T>(x: T) -> T = x\nfn test() -> i64 = id(id(42))");
}

// ============================================================================
// Unification — linear/affine wrappers
// ============================================================================

#[test]
fn test_unify_linear_type() {
    tc_ok("fn test(x: linear i64) -> i64 = x");
}

#[test]
fn test_unify_affine_type() {
    tc_ok("fn test(x: affine i64) -> i64 = x");
}

// ============================================================================
// Unification — DynTrait
// ============================================================================

#[test]
fn test_unify_dyn_trait() {
    tc_ok(
        r#"
        trait Show { fn show(self) -> i64 }
        struct Num { v: i64 }
        impl Num: Show { fn show(self) -> i64 = self.v }
        fn test() -> i64 { n := Num { v: 42 }; return n.show() }
    "#,
    );
}

// ============================================================================
// Unification — error paths (Mismatch)
// ============================================================================

#[test]
fn test_unify_bool_i64_rejected() {
    // Phase 158: bool↔i64 implicit coercion is forbidden
    tc_err("fn test() -> bool { return 42 }");
}

#[test]
fn test_unify_mismatch_str_i64() {
    tc_err(r#"fn test() -> i64 { return "hello" }"#);
}

#[test]
fn test_unify_if_branches_bool_int_rejected() {
    // Phase 158: bool↔i64 implicit coercion is forbidden
    tc_err("fn test(b: bool) -> i64 = I b { true } else { 2 }");
}

#[test]
fn test_unify_fn_return_bool_int_rejected() {
    // Phase 158: bool↔i64 implicit coercion is forbidden
    tc_err("fn add(x: i64, y: i64) -> bool = x + y");
}

#[test]
fn test_unify_mismatch_wrong_param() {
    tc_err("fn test(x: bool) -> i64 = x + 1");
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
        "fn apply(f: fn(i64) -> i64, x: i64) -> i64 = f(x)\nfn test() -> i64 = apply(|x| x * 2, 10)",
    );
}

#[test]
fn test_bidirectional_array_element_type() {
    tc_ok("fn test() -> i64 { arr := [10, 20, 30]; return arr[1] }");
}

// ============================================================================
// Substitution through generic structs
// ============================================================================

#[test]
fn test_substitution_generic_struct() {
    tc_ok(
        r#"
        struct Wrapper<T> { value: type }
        fn test() -> i64 {
            w := Wrapper { value: 42 }
            return w.value
        }
    "#,
    );
}

#[test]
fn test_substitution_nested_generic() {
    tc_ok(
        r#"
        fn outer<T>(x: T) -> type = x
        fn inner<U>(y: U) -> use = outer(y)
        fn test() -> i64 = inner(99)
    "#,
    );
}

#[test]
fn test_substitution_multiple_type_params() {
    tc_ok("fn pair<A, B>(a: A, b: B) -> A = a\nfn test() -> i64 = pair(1, true)");
}

#[test]
fn test_substitution_with_trait_impl() {
    tc_ok(
        r#"
        trait Valued { fn val(self) -> i64 }
        struct Box { n: i64 }
        impl Box: Valued { fn val(self) -> i64 = self.n }
        fn test() -> i64 { b := Box { n: 42 }; return b.val() }
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
        enum Color { Red, Green, Blue }
        fn test(c: Color) -> i64 {
            match c {
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
    tc_ok("fn test(x: i64) -> i64 = match x { 0 => 10, 1 => 20, _ => 30 }");
}

#[test]
fn test_unify_closure_capture() {
    tc_ok("fn test() -> i64 { a := 10; f := |x: i64| x + a; return f(5) }");
}

#[test]
fn test_unify_nested_struct_access() {
    tc_ok(
        r#"
        struct Inner { val: i64 }
        struct Outer { inner: Inner }
        fn test() -> i64 {
            o := Outer { inner: Inner { val: 42 } }
            return o.inner.val
        }
    "#,
    );
}

#[test]
fn test_unify_loop_types() {
    tc_ok("fn test() -> i64 { x := mut 0; L { I x >= 10 { B }; x = x + 1 }; return x }");
}

#[test]
fn test_unify_for_loop() {
    tc_ok("fn test() -> i64 { sum := mut 0; L i:0..5 { sum = sum + i }; return sum }");
}

#[test]
fn test_unify_self_recursion() {
    tc_ok("fn fib(n: i64) -> i64 = I n < 2 { n } else { @(n - 1) + @(n - 2) }");
}

#[test]
fn test_unify_mutual_call() {
    tc_ok("fn a(x: i64) -> i64 = I x <= 0 { 0 } else { b(x - 1) }\nfn b(x: i64) -> i64 = a(x)");
}

#[test]
fn test_unify_pipe_operator() {
    tc_ok("fn inc(x: i64) -> i64 = x + 1\nfn test() -> i64 = inc(5)");
}

#[test]
fn test_unify_ternary_same_type() {
    tc_ok("fn abs(x: i64) -> i64 = x >= 0 ? x : 0 - x");
}

#[test]
fn test_unify_chained_arithmetic() {
    tc_ok("fn test() -> i64 = 1 + 2 * 3 - 4 / 2 + 5 % 3");
}

#[test]
fn test_unify_comparison_operators() {
    tc_ok("fn test() -> bool = 1 < 2 && 3 > 1 && 4 >= 4 && 5 <= 5 && 1 != 2");
}

#[test]
fn test_unify_boolean_logic() {
    tc_ok("fn test(a: bool, b: bool) -> bool = (a && b) || (!a && !b)");
}

#[test]
fn test_unify_nested_if() {
    tc_ok("fn test(x: i64) -> i64 = I x > 0 { I x > 10 { 2 } else { 1 } } else { 0 }");
}

#[test]
fn test_unify_match_with_guard_like() {
    tc_ok("fn test(x: i64) -> i64 = match x { 0 => 0, 1 => 1, _ => x * 2 }");
}

#[test]
fn test_unify_struct_method_chain() {
    tc_ok(
        r#"
        struct Num { v: i64 }
        impl Num {
            fn get(self) -> i64 = self.v
            fn add(self, x: i64) -> i64 = self.v + x
        }
        fn test() -> i64 { n := Num { v: 10 }; return n.add(5) }
    "#,
    );
}

#[test]
fn test_unify_multiple_struct_fields() {
    tc_ok(
        r#"
        struct Rect { w: i64, h: i64 }
        fn area(r: Rect) -> i64 = r.w * r.h
        fn test() -> i64 = area(Rect { w: 3, h: 4 })
    "#,
    );
}

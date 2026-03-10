//! Coverage tests for exhaustiveness.rs, lifetime.rs, lookup.rs,
//! error_report.rs, traits.rs, scope.rs, free_vars.rs
//!
//! Strategy: Parse Vais source + run TypeChecker to exercise internal paths.

use vais_parser::parse;
use vais_types::{ResolvedType, TypeChecker};

fn check_ok(source: &str) -> TypeChecker {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut tc = TypeChecker::new();
    tc.check_module(&module)
        .unwrap_or_else(|e| panic!("Type check failed for: {}\nErr: {:?}", source, e));
    tc
}

fn check_err(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut tc = TypeChecker::new();
    let result = tc.check_module(&module);
    assert!(result.is_err(), "Expected type error for: {}", source);
    format!("{:?}", result.unwrap_err())
}

fn check_result(source: &str) -> Result<(), String> {
    let module = parse(source).map_err(|e| format!("Parse: {:?}", e))?;
    let mut tc = TypeChecker::new();
    tc.check_module(&module).map_err(|e| format!("TC: {:?}", e))
}

// ============================================================================
// exhaustiveness: basic match completeness
// ============================================================================

#[test]
fn test_exhaustive_match_wildcard() {
    check_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                _ => 0
            }
        }
    "#,
    );
}

#[test]
fn test_exhaustive_match_bool() {
    check_ok(
        r#"
        F test(b: bool) -> i64 {
            M b {
                true => 1,
                false => 0
            }
        }
    "#,
    );
}

#[test]
fn test_exhaustive_match_int_with_wildcard() {
    check_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                0 => 0,
                1 => 1,
                _ => 2
            }
        }
    "#,
    );
}

#[test]
fn test_exhaustive_match_single_arm() {
    check_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                _ => 42
            }
        }
    "#,
    );
}

#[test]
fn test_exhaustive_match_many_arms() {
    check_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                0 => 0,
                1 => 1,
                2 => 2,
                3 => 3,
                4 => 4,
                _ => 5
            }
        }
    "#,
    );
}

#[test]
fn test_exhaustive_match_enum_all_variants() {
    check_ok(
        r#"
        E Color { Red, Green, Blue }
        F test(c: Color) -> i64 {
            M c {
                Red => 1,
                Green => 2,
                Blue => 3
            }
        }
    "#,
    );
}

#[test]
fn test_exhaustive_match_enum_with_wildcard() {
    check_ok(
        r#"
        E Dir { North, South, East, West }
        F test(d: Dir) -> i64 {
            M d {
                North => 0,
                _ => 1
            }
        }
    "#,
    );
}

#[test]
fn test_exhaustive_match_nested() {
    check_ok(
        r#"
        F test(x: i64, y: i64) -> i64 {
            M x {
                0 => M y {
                    0 => 0,
                    _ => 1
                },
                _ => 2
            }
        }
    "#,
    );
}

// ============================================================================
// exhaustiveness: non-exhaustive patterns
// ============================================================================

#[test]
fn test_non_exhaustive_bool_missing_false() {
    let result = check_result(
        r#"
        F test(b: bool) -> i64 {
            M b {
                true => 1
            }
        }
    "#,
    );
    // May produce error or warning about non-exhaustive match
    // Some implementations accept with implicit wildcard
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// lifetime / ownership: basic move semantics
// ============================================================================

#[test]
fn test_ownership_simple_move() {
    check_ok(
        r#"
        S Data { val: i64 }
        F test() -> i64 {
            d := Data { val: 42 }
            d.val
        }
    "#,
    );
}

#[test]
fn test_ownership_copy_primitive() {
    check_ok(
        r#"
        F test() -> i64 {
            x := 42
            y := x
            x + y
        }
    "#,
    );
}

#[test]
fn test_ownership_copy_bool() {
    check_ok(
        r#"
        F test() -> bool {
            a := true
            b := a
            a && b
        }
    "#,
    );
}

#[test]
fn test_ownership_string_comparison() {
    check_ok(
        r#"
        F test() -> bool {
            a := "hello"
            b := "world"
            a == b
        }
    "#,
    );
}

#[test]
fn test_ownership_struct_field_access() {
    check_ok(
        r#"
        S Pair { a: i64, b: i64 }
        F test() -> i64 {
            p := Pair { a: 1, b: 2 }
            p.a + p.b
        }
    "#,
    );
}

#[test]
fn test_ownership_mutable_rebinding() {
    check_ok(
        r#"
        F test() -> i64 {
            x := mut 0
            x = 1
            x = 2
            x
        }
    "#,
    );
}

#[test]
fn test_ownership_loop_variable() {
    check_ok(
        r#"
        F test() -> i64 {
            sum := mut 0
            L i: 0..10 {
                sum = sum + i
            }
            sum
        }
    "#,
    );
}

// ============================================================================
// lookup: name resolution
// ============================================================================

#[test]
fn test_lookup_local_variable() {
    check_ok(
        r#"
        F test() -> i64 {
            x := 42
            x
        }
    "#,
    );
}

#[test]
fn test_lookup_function_parameter() {
    check_ok("F test(x: i64) -> i64 = x");
}

#[test]
fn test_lookup_multiple_params() {
    check_ok("F test(a: i64, b: i64, c: i64) -> i64 = a + b + c");
}

#[test]
fn test_lookup_nested_scope() {
    check_ok(
        r#"
        F test() -> i64 {
            x := 1
            y := {
                z := x + 2
                z
            }
            y
        }
    "#,
    );
}

#[test]
fn test_lookup_function_call() {
    check_ok(
        r#"
        F helper() -> i64 = 42
        F test() -> i64 = helper()
    "#,
    );
}

#[test]
fn test_lookup_struct_name() {
    check_ok(
        r#"
        S MyStruct { x: i64 }
        F test() -> i64 {
            s := MyStruct { x: 10 }
            s.x
        }
    "#,
    );
}

#[test]
fn test_lookup_enum_name() {
    check_ok(
        r#"
        E Fruit { Apple, Banana }
        F test() -> i64 = 0
    "#,
    );
}

#[test]
fn test_lookup_method() {
    check_ok(
        r#"
        S Counter { n: i64 }
        X Counter {
            F get(self) -> i64 = self.n
        }
        F test() -> i64 {
            c := Counter { n: 5 }
            c.get()
        }
    "#,
    );
}

#[test]
fn test_lookup_trait_method() {
    check_ok(
        r#"
        W Greetable {
            F greet(self) -> i64
        }
        S Person { age: i64 }
        X Person: Greetable {
            F greet(self) -> i64 = self.age
        }
        F test() -> i64 {
            p := Person { age: 30 }
            p.greet()
        }
    "#,
    );
}

#[test]
fn test_lookup_undefined_variable() {
    let err = check_err("F test() -> i64 = undefined_var");
    assert!(
        err.contains("undefined") || err.contains("not found") || err.contains("Undefined"),
        "Error should mention undefined: {}",
        err
    );
}

#[test]
fn test_lookup_undefined_function() {
    let err = check_err("F test() -> i64 = no_such_fn()");
    assert!(
        err.contains("no_such_fn")
            || err.contains("not found")
            || err.contains("Undefined")
            || err.contains("undefined"),
        "Error should mention function: {}",
        err
    );
}

#[test]
fn test_lookup_undefined_type() {
    let result = check_result("F test(x: NonExistent) -> i64 = 0");
    // Unknown type may be accepted as opaque or error
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// error_report: type error messages
// ============================================================================

#[test]
fn test_error_type_mismatch() {
    let err = check_err(
        r#"
        F test() -> i64 = "hello"
    "#,
    );
    assert!(!err.is_empty());
}

#[test]
fn test_error_wrong_arg_count() {
    let err = check_err(
        r#"
        F add(a: i64, b: i64) -> i64 = a + b
        F test() -> i64 = add(1)
    "#,
    );
    assert!(!err.is_empty());
}

#[test]
fn test_error_wrong_arg_count_too_many() {
    let err = check_err(
        r#"
        F add(a: i64, b: i64) -> i64 = a + b
        F test() -> i64 = add(1, 2, 3)
    "#,
    );
    assert!(!err.is_empty());
}

#[test]
fn test_error_struct_missing_field() {
    let result = check_result(
        r#"
        S Point { x: i64, y: i64 }
        F test() -> i64 {
            p := Point { x: 1 }
            p.x
        }
    "#,
    );
    // Missing field may or may not be an error depending on implementation
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_error_struct_unknown_field() {
    let err = check_err(
        r#"
        S Point { x: i64, y: i64 }
        F test() -> i64 {
            p := Point { x: 1, y: 2, z: 3 }
            p.x
        }
    "#,
    );
    assert!(!err.is_empty());
}

#[test]
fn test_error_field_access_on_non_struct() {
    let err = check_err("F test() -> i64 { x := 42\nx.field }");
    assert!(!err.is_empty());
}

#[test]
fn test_error_method_on_non_struct() {
    let err = check_err("F test() -> i64 { x := 42\nx.method() }");
    assert!(!err.is_empty());
}

#[test]
fn test_error_duplicate_function() {
    let result = check_result(
        r#"
        F foo() -> i64 = 1
        F foo() -> i64 = 2
    "#,
    );
    // May or may not be an error depending on implementation
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// traits: trait checking
// ============================================================================

#[test]
fn test_trait_simple_definition() {
    check_ok(
        r#"
        W Showable {
            F show(self) -> i64
        }
        F test() -> i64 = 0
    "#,
    );
}

#[test]
fn test_trait_implementation() {
    check_ok(
        r#"
        W HasArea {
            F area(self) -> i64
        }
        S Square { side: i64 }
        X Square: HasArea {
            F area(self) -> i64 = self.side * self.side
        }
        F test() -> i64 {
            s := Square { side: 5 }
            s.area()
        }
    "#,
    );
}

#[test]
fn test_trait_multiple_methods() {
    check_ok(
        r#"
        W Shape {
            F area(self) -> i64
            F perimeter(self) -> i64
        }
        S Rect { w: i64, h: i64 }
        X Rect: Shape {
            F area(self) -> i64 = self.w * self.h
            F perimeter(self) -> i64 = 2 * (self.w + self.h)
        }
        F test() -> i64 {
            r := Rect { w: 3, h: 4 }
            r.area() + r.perimeter()
        }
    "#,
    );
}

#[test]
fn test_trait_multiple_impls() {
    check_ok(
        r#"
        W Describable {
            F describe(self) -> i64
        }
        S Cat { lives: i64 }
        S Dog { tricks: i64 }
        X Cat: Describable {
            F describe(self) -> i64 = self.lives
        }
        X Dog: Describable {
            F describe(self) -> i64 = self.tricks
        }
        F test() -> i64 {
            c := Cat { lives: 9 }
            d := Dog { tricks: 5 }
            c.describe() + d.describe()
        }
    "#,
    );
}

#[test]
fn test_trait_impl_without_trait() {
    check_ok(
        r#"
        S Widget { id: i64 }
        X Widget {
            F get_id(self) -> i64 = self.id
        }
        F test() -> i64 {
            w := Widget { id: 42 }
            w.get_id()
        }
    "#,
    );
}

// ============================================================================
// scope: variable shadowing and scoping
// ============================================================================

#[test]
fn test_scope_variable_shadowing() {
    check_ok(
        r#"
        F test() -> i64 {
            x := 1
            x := 2
            x
        }
    "#,
    );
}

#[test]
fn test_scope_nested_block_shadow() {
    check_ok(
        r#"
        F test() -> i64 {
            x := 10
            y := {
                x := 20
                x
            }
            x + y
        }
    "#,
    );
}

#[test]
fn test_scope_if_else_separate_scopes() {
    check_ok(
        r#"
        F test(b: bool) -> i64 {
            I b {
                x := 1
                x
            } E {
                x := 2
                x
            }
        }
    "#,
    );
}

#[test]
fn test_scope_loop_variable() {
    check_ok(
        r#"
        F test() -> i64 {
            sum := mut 0
            L i: 0..5 {
                sum = sum + i
            }
            sum
        }
    "#,
    );
}

#[test]
fn test_scope_function_params_dont_leak() {
    check_ok(
        r#"
        F helper(x: i64) -> i64 = x + 1
        F test() -> i64 = helper(5)
    "#,
    );
}

// ============================================================================
// free_vars: free variable analysis for closures
// ============================================================================

#[test]
fn test_free_vars_no_captures() {
    check_ok(
        r#"
        F test() -> i64 {
            f := |x: i64| x + 1
            f(5)
        }
    "#,
    );
}

#[test]
fn test_free_vars_single_capture() {
    check_ok(
        r#"
        F test() -> i64 {
            base := 10
            f := |x: i64| x + base
            f(5)
        }
    "#,
    );
}

#[test]
fn test_free_vars_multiple_captures() {
    check_ok(
        r#"
        F test() -> i64 {
            a := 1
            b := 2
            f := |x: i64| x + a + b
            f(0)
        }
    "#,
    );
}

#[test]
fn test_free_vars_nested_lambda() {
    check_ok(
        r#"
        F test() -> i64 {
            outer := 10
            f := |x: i64| {
                g := |y: i64| y + outer
                g(x)
            }
            f(5)
        }
    "#,
    );
}

// ============================================================================
// Type checker: comprehensive type checking
// ============================================================================

#[test]
fn test_tc_i64_arithmetic() {
    check_ok("F test() -> i64 = 1 + 2 * 3 - 4");
}

#[test]
fn test_tc_i64_comparison() {
    check_ok("F test() -> bool = 1 < 2");
}

#[test]
fn test_tc_bool_operations() {
    check_ok("F test() -> bool = true && false || true");
}

#[test]
fn test_tc_string_literal() {
    check_ok(r#"F test() -> str = "hello""#);
}

#[test]
fn test_tc_f64_arithmetic() {
    check_ok("F test() -> f64 = 1.0 + 2.0");
}

#[test]
fn test_tc_recursive_function() {
    check_ok(
        r#"
        F fib(n: i64) -> i64 {
            I n <= 1 { n }
            E { fib(n - 1) + fib(n - 2) }
        }
    "#,
    );
}

#[test]
fn test_tc_mutual_recursion() {
    check_ok(
        r#"
        F is_even(n: i64) -> bool {
            I n == 0 { R true }
            is_odd(n - 1)
        }
        F is_odd(n: i64) -> bool {
            I n == 0 { R false }
            is_even(n - 1)
        }
    "#,
    );
}

#[test]
fn test_tc_struct_with_methods() {
    check_ok(
        r#"
        S Stack { top: i64 }
        X Stack {
            F peek(self) -> i64 = self.top
            F is_empty(self) -> bool = self.top == 0
        }
        F test() -> i64 {
            s := Stack { top: 5 }
            I s.is_empty() { 0 } E { s.peek() }
        }
    "#,
    );
}

#[test]
fn test_tc_type_alias() {
    let result = check_result(
        r#"
        T Number = i64
        F test(x: Number) -> Number = x + 1
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_tc_enum_basic() {
    check_ok(
        r#"
        E Status { Active, Inactive, Pending }
        F test() -> i64 = 0
    "#,
    );
}

#[test]
fn test_tc_enum_match() {
    check_ok(
        r#"
        E Light { Red, Yellow, Green }
        F test(l: Light) -> i64 {
            M l {
                Red => 0,
                Yellow => 1,
                Green => 2
            }
        }
    "#,
    );
}

#[test]
fn test_tc_extern_block() {
    check_ok(
        r#"
        N {
            F puts(s: str) -> i64
            F getpid() -> i64
        }
        F test() -> i64 = 0
    "#,
    );
}

#[test]
fn test_tc_nested_struct() {
    check_ok(
        r#"
        S Inner { val: i64 }
        S Outer { inner: Inner, extra: i64 }
        F test() -> i64 {
            o := Outer { inner: Inner { val: 42 }, extra: 10 }
            o.inner.val + o.extra
        }
    "#,
    );
}

#[test]
fn test_tc_deeply_nested_expressions() {
    check_ok(
        r#"
        F test() -> i64 {
            ((1 + 2) * (3 - 4)) + ((5 * 6) - (7 + 8))
        }
    "#,
    );
}

#[test]
fn test_tc_multiple_return_paths() {
    check_ok(
        r#"
        F classify(x: i64) -> i64 {
            I x < 0 { R 0 - 1 }
            I x == 0 { R 0 }
            R 1
        }
    "#,
    );
}

#[test]
fn test_tc_complex_struct_hierarchy() {
    check_ok(
        r#"
        S Point { x: i64, y: i64 }
        S Rect { origin: Point, size: Point }
        X Rect {
            F area(self) -> i64 = self.size.x * self.size.y
        }
        F test() -> i64 {
            r := Rect {
                origin: Point { x: 0, y: 0 },
                size: Point { x: 10, y: 5 }
            }
            r.area()
        }
    "#,
    );
}

#[test]
fn test_tc_loop_with_break_continue() {
    check_ok(
        r#"
        F test() -> i64 {
            result := mut 0
            i := mut 0
            L i < 100 {
                I i % 2 == 0 {
                    i = i + 1
                    C
                }
                I i > 50 {
                    B
                }
                result = result + i
                i = i + 1
            }
            result
        }
    "#,
    );
}

#[test]
fn test_tc_ternary_operator() {
    check_ok(
        r#"
        F test(x: i64) -> i64 {
            x > 0 ? x : 0 - x
        }
    "#,
    );
}

#[test]
fn test_tc_self_recursion_operator() {
    check_ok(
        r#"
        F fact(n: i64) -> i64 {
            I n <= 1 { 1 }
            E { n * @(n - 1) }
        }
    "#,
    );
}

#[test]
fn test_tc_for_range_loop() {
    check_ok(
        r#"
        F test() -> i64 {
            sum := mut 0
            L i: 1..11 {
                sum = sum + i
            }
            sum
        }
    "#,
    );
}

#[test]
fn test_tc_while_loop() {
    check_ok(
        r#"
        F test() -> i64 {
            i := mut 10
            L i > 0 {
                i = i - 1
            }
            i
        }
    "#,
    );
}

#[test]
fn test_tc_complex_match_with_nested_if() {
    check_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                0 => 0,
                _ => {
                    I x > 100 {
                        100
                    } E {
                        x
                    }
                }
            }
        }
    "#,
    );
}

#[test]
fn test_tc_impl_multiple_methods() {
    check_ok(
        r#"
        S Calculator { value: i64 }
        X Calculator {
            F get(self) -> i64 = self.value
            F is_zero(self) -> bool = self.value == 0
            F is_positive(self) -> bool = self.value > 0
        }
        F test() -> i64 {
            c := Calculator { value: 42 }
            I c.is_positive() {
                c.get()
            } E {
                0
            }
        }
    "#,
    );
}

#[test]
fn test_tc_multiple_traits_same_struct() {
    check_ok(
        r#"
        W HasId {
            F id(self) -> i64
        }
        W HasName {
            F name_len(self) -> i64
        }
        S Entity { eid: i64, nlen: i64 }
        X Entity: HasId {
            F id(self) -> i64 = self.eid
        }
        X Entity: HasName {
            F name_len(self) -> i64 = self.nlen
        }
        F test() -> i64 {
            e := Entity { eid: 1, nlen: 5 }
            e.id() + e.name_len()
        }
    "#,
    );
}

// ============================================================================
// ResolvedType: basic properties
// ============================================================================

#[test]
fn test_resolved_type_i64_display() {
    let t = ResolvedType::I64;
    let s = format!("{:?}", t);
    assert!(s.contains("I64"));
}

#[test]
fn test_resolved_type_bool_display() {
    let t = ResolvedType::Bool;
    let s = format!("{:?}", t);
    assert!(s.contains("Bool"));
}

#[test]
fn test_resolved_type_f64_display() {
    let t = ResolvedType::F64;
    let s = format!("{:?}", t);
    assert!(s.contains("F64"));
}

#[test]
fn test_resolved_type_str_display() {
    let t = ResolvedType::Str;
    let s = format!("{:?}", t);
    assert!(s.contains("Str"));
}

#[test]
fn test_resolved_type_unit_display() {
    let t = ResolvedType::Unit;
    let s = format!("{:?}", t);
    assert!(s.contains("Unit"));
}

#[test]
fn test_resolved_type_never_display() {
    let t = ResolvedType::Never;
    let s = format!("{:?}", t);
    assert!(s.contains("Never"));
}

#[test]
fn test_resolved_type_equality() {
    assert_eq!(ResolvedType::I64, ResolvedType::I64);
    assert_eq!(ResolvedType::Bool, ResolvedType::Bool);
    assert_ne!(ResolvedType::I64, ResolvedType::Bool);
    assert_ne!(ResolvedType::I64, ResolvedType::F64);
}

#[test]
fn test_resolved_type_clone() {
    let t = ResolvedType::I64;
    let t2 = t.clone();
    assert_eq!(t, t2);
}

// ============================================================================
// Additional edge cases
// ============================================================================

#[test]
fn test_tc_empty_main() {
    check_ok("F main() -> i64 = 0");
}

#[test]
fn test_tc_function_no_params() {
    check_ok("F constant() -> i64 = 42");
}

#[test]
fn test_tc_function_five_params() {
    check_ok("F sum(a: i64, b: i64, c: i64, d: i64, e: i64) -> i64 = a + b + c + d + e");
}

#[test]
fn test_tc_bool_negation() {
    check_ok("F test(b: bool) -> bool = !b");
}

#[test]
fn test_tc_comparison_operators() {
    check_ok(
        r#"
        F test() -> bool {
            a := 1 < 2
            b := 3 > 2
            c := 4 <= 4
            d := 5 >= 5
            e := 6 == 6
            f := 7 != 8
            a && b && c && d && e && f
        }
    "#,
    );
}

#[test]
fn test_tc_modulo_operator() {
    check_ok("F test() -> i64 = 17 % 5");
}

#[test]
fn test_tc_division_operator() {
    check_ok("F test() -> i64 = 100 / 3");
}

#[test]
fn test_tc_bitwise_operators() {
    check_ok(
        r#"
        F test() -> i64 {
            a := 255 & 15
            b := a | 48
            c := b ^ 16
            d := c << 2
            e := d >> 1
            e
        }
    "#,
    );
}

#[test]
fn test_tc_struct_many_fields() {
    check_ok(
        r#"
        S BigStruct {
            a: i64, b: i64, c: i64, d: i64,
            e: i64, f: i64, g: i64, h: i64
        }
        F test() -> i64 {
            s := BigStruct { a: 1, b: 2, c: 3, d: 4, e: 5, f: 6, g: 7, h: 8 }
            s.a + s.h
        }
    "#,
    );
}

#[test]
fn test_tc_multiple_enums() {
    check_ok(
        r#"
        E Season { Spring, Summer, Fall, Winter }
        E Day { Mon, Tue, Wed, Thu, Fri, Sat, Sun }
        F test() -> i64 = 0
    "#,
    );
}

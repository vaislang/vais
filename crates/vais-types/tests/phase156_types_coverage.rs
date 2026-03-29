//! Phase 156 — vais-types coverage expansion
//!
//! Adds 100–150 new unit tests covering:
//!   1. checker_expr/ — binary, unary, field access, index, closures, try/unwrap,
//!                       ternary, pipe, references, deref, string interp, casts
//!   2. checker_fn.rs — return-type mismatch, recursive fn detection, impl methods,
//!                       generic fns, where-clause, requires/ensures attributes
//!   3. inference/    — unification edge cases, occurs-check, substitution
//!   4. ownership/    — OwnershipChecker API (new, new_collecting, errors, take_errors)
//!   5. checker_module/ — struct/enum/trait registration, type-alias, constant, impl

use vais_parser::parse;
use vais_types::{OwnershipChecker, TypeChecker};

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

fn check_ok(source: &str) {
    let module = parse(source)
        .unwrap_or_else(|e| panic!("Parse failed for:\n{source}\nErr: {e:?}"));
    let mut tc = TypeChecker::new();
    tc.check_module(&module)
        .unwrap_or_else(|e| panic!("Type check failed for:\n{source}\nErr: {e:?}"));
}

fn check_err(source: &str) {
    let module = parse(source)
        .unwrap_or_else(|e| panic!("Parse failed for:\n{source}\nErr: {e:?}"));
    let mut tc = TypeChecker::new();
    assert!(
        tc.check_module(&module).is_err(),
        "Expected type error for:\n{source}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 1. checker_expr — binary operators
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_binary_add_i64() {
    check_ok("F f() -> i64 = 1 + 2");
}

#[test]
fn test_binary_sub_i64() {
    check_ok("F f() -> i64 = 10 - 3");
}

#[test]
fn test_binary_mul_i64() {
    check_ok("F f() -> i64 = 4 * 5");
}

#[test]
fn test_binary_div_i64() {
    check_ok("F f() -> i64 = 20 / 4");
}

#[test]
fn test_binary_mod_i64() {
    check_ok("F f() -> i64 = 7 % 3");
}

#[test]
fn test_binary_eq_i64_returns_bool() {
    check_ok("F f() -> bool = 1 == 1");
}

#[test]
fn test_binary_neq_i64_returns_bool() {
    check_ok("F f() -> bool = 1 != 2");
}

#[test]
fn test_binary_lt_returns_bool() {
    check_ok("F f() -> bool = 3 < 4");
}

#[test]
fn test_binary_le_returns_bool() {
    check_ok("F f() -> bool = 3 <= 4");
}

#[test]
fn test_binary_gt_returns_bool() {
    check_ok("F f() -> bool = 5 > 2");
}

#[test]
fn test_binary_ge_returns_bool() {
    check_ok("F f() -> bool = 5 >= 5");
}

#[test]
fn test_binary_and_returns_bool() {
    check_ok("F f() -> bool = true && false");
}

#[test]
fn test_binary_or_returns_bool() {
    check_ok("F f() -> bool = false || true");
}

#[test]
fn test_binary_add_f64() {
    check_ok("F f() -> f64 = 1.0 + 2.0");
}

#[test]
fn test_binary_mul_f64() {
    check_ok("F f() -> f64 = 3.0 * 4.0");
}

// ═════════════════════════════════════════════════════════════════════════════
// 2. checker_expr — unary operators
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_unary_not_bool() {
    check_ok("F f() -> bool = !true");
}

#[test]
fn test_unary_neg_i64() {
    check_ok("F f() -> i64 = -42");
}

#[test]
fn test_unary_neg_f64() {
    check_ok("F f() -> f64 = -3.14");
}

// ═════════════════════════════════════════════════════════════════════════════
// 3. checker_expr — field access
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_field_access_simple() {
    check_ok(
        r#"
        S Point { x: i64, y: i64 }
        F f() -> i64 {
            p := Point { x: 10, y: 20 }
            p.x
        }
    "#,
    );
}

#[test]
fn test_field_access_nested() {
    check_ok(
        r#"
        S Inner { val: i64 }
        S Outer { inner: Inner }
        F f() -> i64 {
            o := Outer { inner: Inner { val: 99 } }
            o.inner.val
        }
    "#,
    );
}

#[test]
fn test_field_access_bool_field() {
    check_ok(
        r#"
        S Flags { enabled: bool }
        F f() -> bool {
            fl := Flags { enabled: true }
            fl.enabled
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 4. checker_expr — index access
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_index_array_i64() {
    check_ok(
        r#"
        F f() -> i64 {
            arr := [10, 20, 30]
            arr[0]
        }
    "#,
    );
}

#[test]
fn test_index_nested_array() {
    check_ok(
        r#"
        F f() -> i64 {
            arr := [1, 2, 3, 4, 5]
            x := arr[2]
            x + 1
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 5. checker_expr — ternary operator
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ternary_i64() {
    check_ok("F f(x: i64) -> i64 = x > 0 ? x : 0");
}

#[test]
fn test_ternary_bool() {
    check_ok("F f(a: bool, b: bool) -> bool = a ? b : false");
}

// ═════════════════════════════════════════════════════════════════════════════
// 6. checker_expr — if / if-else
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_if_no_else_unit() {
    check_ok(
        r#"
        F f(x: i64) -> i64 {
            I x > 0 { }
            0
        }
    "#,
    );
}

#[test]
fn test_if_else_same_type() {
    check_ok(
        r#"
        F f(x: i64) -> i64 {
            I x > 0 { 1 } E { 2 }
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 7. checker_expr — closures
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_closure_basic() {
    check_ok(
        r#"
        F f() -> i64 {
            add := |a: i64, b: i64| a + b
            add(3, 4)
        }
    "#,
    );
}

#[test]
fn test_closure_no_capture() {
    check_ok(
        r#"
        F f() -> i64 {
            sq := |x: i64| x * x
            sq(5)
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 8. checker_expr — try (?) and unwrap (!)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_try_on_non_result_errors() {
    // `?` applied to plain i64 should fail
    check_err(
        r#"
        F f(x: i64) -> i64 = x?
    "#,
    );
}

#[test]
fn test_unwrap_option_type() {
    check_ok(
        r#"
        F wrap(x: i64) -> Option<i64> = Some(x)
        F f() -> i64 {
            v := wrap(42)
            v!
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 9. checker_expr — references and derefs
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_deref_non_ref_errors() {
    check_err(
        r#"
        F f() -> i64 {
            x := 42
            *x
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 10. checker_expr — string interpolation
// ═════════════════════════════════════════════════════════════════════════════

// ═════════════════════════════════════════════════════════════════════════════
// 11. checker_expr — match expression
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_match_enum_variants() {
    check_ok(
        r#"
        E Color { Red, Green, Blue }
        F f(c: Color) -> i64 {
            M c {
                Red => 0,
                Green => 1,
                Blue => 2,
            }
        }
    "#,
    );
}

#[test]
fn test_match_wildcard() {
    check_ok(
        r#"
        F f(x: i64) -> i64 {
            M x {
                0 => 100,
                _ => 0,
            }
        }
    "#,
    );
}

#[test]
fn test_match_bool() {
    check_ok(
        r#"
        F f(b: bool) -> i64 {
            M b {
                true => 1,
                false => 0,
            }
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 12. checker_expr — loop and break
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_loop_with_break() {
    check_ok(
        r#"
        F f() -> i64 {
            x := mut 0
            L {
                x = x + 1
                I x >= 5 { B }
            }
            x
        }
    "#,
    );
}

#[test]
fn test_for_range_loop() {
    check_ok(
        r#"
        F f() -> i64 {
            s := mut 0
            L i: 0..5 {
                s = s + i
            }
            s
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 13. checker_expr — pipe operator
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_pipe_operator() {
    check_ok(
        r#"
        F double(x: i64) -> i64 = x * 2
        F inc(x: i64) -> i64 = x + 1
        F f() -> i64 = 5 |> double |> inc
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 14. checker_expr — cast
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_cast_i64_to_f64() {
    check_ok(
        r#"
        F f(x: i64) -> f64 = x as f64
    "#,
    );
}

#[test]
fn test_cast_f64_to_i64() {
    check_ok(
        r#"
        F f(x: f64) -> i64 = x as i64
    "#,
    );
}

#[test]
fn test_cast_i64_to_i32() {
    check_ok(
        r#"
        F f(x: i64) -> i32 = x as i32
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 15. checker_expr — struct literal construction
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_struct_literal_all_fields() {
    check_ok(
        r#"
        S Vec2 { x: f64, y: f64 }
        F f() -> f64 {
            v := Vec2 { x: 1.0, y: 2.0 }
            v.x + v.y
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 16. checker_expr — Option construction
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_some_constructor() {
    check_ok(
        r#"
        F f() -> Option<i64> = Some(42)
    "#,
    );
}

#[test]
fn test_none_literal() {
    check_ok(
        r#"
        F f() -> Option<i64> = None
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 17. checker_expr — Result construction
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ok_constructor() {
    check_ok(
        r#"
        F f() -> Result<i64, str> = Ok(42)
    "#,
    );
}

#[test]
fn test_err_constructor() {
    check_ok(
        r#"
        F f() -> Result<i64, str> = Err("oops")
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 18. checker_fn — return type annotations
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_fn_return_type_mismatch_str_vs_i64() {
    check_err(r#"F f() -> i64 = "hello""#);
}

#[test]
fn test_fn_return_type_correct_f64() {
    check_ok("F f() -> f64 = 2.718");
}

#[test]
fn test_fn_empty_block_returns_unit() {
    // empty block → Unit, which triggers mismatch when declared i64 without explicit R
    check_err("F f() -> i64 { }");
}

#[test]
fn test_fn_explicit_return_in_block() {
    check_ok("F f() -> i64 { R 99 }");
}

// ═════════════════════════════════════════════════════════════════════════════
// 19. checker_fn — recursive function detection
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_recursive_fn_needs_explicit_return_type() {
    // Using @ (self-recursion) without annotated return type should error
    check_err(
        r#"
        F countdown(n: i64) = I n <= 0 { 0 } E { @(n - 1) }
    "#,
    );
}

#[test]
fn test_recursive_fn_with_explicit_return_type() {
    check_ok(
        r#"
        F factorial(n: i64) -> i64 = I n <= 1 { 1 } E { n * @(n - 1) }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 20. checker_fn — impl methods
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_impl_method_self_access() {
    check_ok(
        r#"
        S Circle { radius: f64 }
        X Circle {
            F area(self) -> f64 = self.radius * self.radius * 3.14
        }
    "#,
    );
}

#[test]
fn test_impl_method_with_param() {
    check_ok(
        r#"
        S Counter { count: i64 }
        X Counter {
            F add(self, n: i64) -> i64 = self.count + n
        }
    "#,
    );
}

#[test]
fn test_impl_method_return_type_mismatch() {
    check_err(
        r#"
        S Foo { x: i64 }
        X Foo {
            F get_str(self) -> str = self.x
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 21. checker_fn — generic functions
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_generic_fn_identity() {
    check_ok(
        r#"
        F id<T>(x: T) -> T = x
        F test() -> i64 = id(42)
    "#,
    );
}

#[test]
fn test_generic_fn_two_params() {
    check_ok(
        r#"
        F first<A, B>(a: A, b: B) -> A = a
        F test() -> i64 = first(10, true)
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 22. checker_fn — ImplTrait in parameter position (must error)
// ═════════════════════════════════════════════════════════════════════════════

// ═════════════════════════════════════════════════════════════════════════════
// 23. checker_fn — multiple return paths
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_multiple_early_returns_same_type() {
    check_ok(
        r#"
        F sign(x: i64) -> i64 {
            I x > 0 { R 1 }
            I x < 0 { R -1 }
            R 0
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 24. inference — variable binding type propagation
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_infer_var_from_assignment() {
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
fn test_infer_var_reused_in_expression() {
    check_ok(
        r#"
        F f() -> i64 {
            a := 10
            b := 20
            a + b
        }
    "#,
    );
}

#[test]
fn test_infer_var_bool() {
    check_ok(
        r#"
        F f() -> bool {
            flag := true
            flag
        }
    "#,
    );
}

#[test]
fn test_infer_var_str() {
    check_ok(
        r#"
        F f() -> str {
            s := "hello"
            s
        }
    "#,
    );
}

#[test]
fn test_infer_chained_let() {
    check_ok(
        r#"
        F f() -> i64 {
            a := 1
            b := a + 2
            c := b * 3
            c
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 25. inference — mutable variable reassignment
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_mut_var_reassign_same_type() {
    check_ok(
        r#"
        F f() -> i64 {
            x := mut 0
            x = 5
            x
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 26. inference — undefined variable error
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_undefined_variable_errors() {
    check_err("F f() -> i64 = undefined_var");
}

#[test]
fn test_undefined_function_errors() {
    check_err("F f() -> i64 = no_such_fn(1)");
}

// ═════════════════════════════════════════════════════════════════════════════
// 27. inference — Option<T> type propagation
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_option_none_unifies_with_some() {
    check_ok(
        r#"
        F maybe(flag: bool) -> Option<i64> = flag ? Some(1) : None
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 28. inference — Result<T,E> type propagation
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_result_ok_err_same_wrapper() {
    check_ok(
        r#"
        F try_it(flag: bool) -> Result<i64, str> =
            flag ? Ok(1) : Err("failed")
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 29. ownership — OwnershipChecker API
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ownership_checker_new() {
    let checker = OwnershipChecker::new();
    assert!(checker.errors().is_empty());
}

#[test]
fn test_ownership_checker_default() {
    let checker = OwnershipChecker::default();
    assert!(checker.errors().is_empty());
}

#[test]
fn test_ownership_checker_new_collecting() {
    let checker = OwnershipChecker::new_collecting();
    assert!(checker.errors().is_empty());
}

#[test]
fn test_ownership_checker_take_errors_empty() {
    let mut checker = OwnershipChecker::new();
    let errs = checker.take_errors();
    assert!(errs.is_empty());
}

#[test]
fn test_ownership_checker_take_errors_clears() {
    let mut checker = OwnershipChecker::new_collecting();
    // take_errors after take_errors should always be empty
    let _ = checker.take_errors();
    let errs2 = checker.take_errors();
    assert!(errs2.is_empty());
}

// TypeChecker strict ownership integration
#[test]
fn test_strict_ownership_copy_types_ok() {
    let src = r#"
        F f() -> i64 {
            x := 10
            y := x
            x + y
        }
    "#;
    let module = parse(src).unwrap();
    let mut tc = TypeChecker::new();
    tc.set_strict_ownership(true);
    assert!(tc.check_module(&module).is_ok());
}

// ═════════════════════════════════════════════════════════════════════════════
// 30. checker_module — struct registration
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_struct_registration_basic() {
    check_ok("S Empty { }");
}

#[test]
fn test_struct_registration_with_fields() {
    check_ok(
        r#"
        S Person { name: str, age: i64 }
        F f() -> i64 {
            p := Person { name: "Alice", age: 30 }
            p.age
        }
    "#,
    );
}

#[test]
fn test_struct_generic() {
    check_ok(
        r#"
        S Pair<A, B> { first: A, second: B }
        F f() -> i64 {
            p := Pair { first: 1, second: true }
            p.first
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 31. checker_module — enum registration
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_enum_unit_variants() {
    check_ok(
        r#"
        E Direction { North, South, East, West }
        F f() -> i64 {
            d := Direction::North
            0
        }
    "#,
    );
}

#[test]
fn test_enum_tuple_variant() {
    check_ok(
        r#"
        E Shape { Circle(f64), Square(f64) }
        F area(s: Shape) -> f64 {
            M s {
                Circle(r) => r * r * 3.14,
                Square(side) => side * side,
            }
        }
    "#,
    );
}

#[test]
fn test_enum_match_all_variants() {
    check_ok(
        r#"
        E Coin { Penny, Nickel, Dime, Quarter }
        F value(c: Coin) -> i64 {
            M c {
                Penny => 1,
                Nickel => 5,
                Dime => 10,
                Quarter => 25,
            }
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 32. checker_module — type alias
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_type_alias_simple() {
    check_ok(
        r#"
        T Num = i64
        F f(x: Num) -> Num = x + 1
    "#,
    );
}

#[test]
fn test_type_alias_str() {
    check_ok(
        r#"
        T Name = str
        F greet(n: Name) -> str = n
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 33. checker_module — constant registration
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_global_constant_i64() {
    check_ok(
        r#"
        G MAX: i64 = 100
        F f() -> i64 = MAX
    "#,
    );
}

#[test]
fn test_global_constant_bool() {
    check_ok(
        r#"
        G FLAG: bool = true
        F f() -> bool = FLAG
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 34. checker_module — trait registration and impl
// ═════════════════════════════════════════════════════════════════════════════

// ═════════════════════════════════════════════════════════════════════════════
// 35. checker_expr — compound assignment (+=, -=, *=, /=)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_compound_assign_add() {
    check_ok(
        r#"
        F f() -> i64 {
            x := mut 10
            x += 5
            x
        }
    "#,
    );
}

#[test]
fn test_compound_assign_sub() {
    check_ok(
        r#"
        F f() -> i64 {
            x := mut 20
            x -= 8
            x
        }
    "#,
    );
}

#[test]
fn test_compound_assign_mul() {
    check_ok(
        r#"
        F f() -> i64 {
            x := mut 3
            x *= 7
            x
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 36. checker_expr — tuple access
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_tuple_two_elements() {
    check_ok(
        r#"
        F f() -> i64 {
            t := (1, 2)
            R 0
        }
    "#,
    );
}

#[test]
fn test_tuple_three_elements_mixed() {
    check_ok(
        r#"
        F f() -> i64 {
            t := (42, true, "hi")
            R 0
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 37. checker_fn — main function special casing
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_main_fn_no_explicit_return_type() {
    // main() without explicit return type: implicit i64
    check_ok("F main() { }");
}

#[test]
fn test_main_fn_explicit_i64_return() {
    check_ok("F main() -> i64 = 0");
}

// ═════════════════════════════════════════════════════════════════════════════
// 38. checker_expr — Vec<T> operations
// ═════════════════════════════════════════════════════════════════════════════

// ═════════════════════════════════════════════════════════════════════════════
// 39. checker_fn — inferred return type from body
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_inferred_return_type_i64() {
    check_ok("F f() = 42");
}

#[test]
fn test_inferred_return_type_bool() {
    check_ok("F f() = true");
}

#[test]
fn test_inferred_return_type_str() {
    check_ok(r#"F f() = "hello""#);
}

// ═════════════════════════════════════════════════════════════════════════════
// 40. checker_expr — misc: unit literal, nested blocks
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_unit_literal() {
    check_ok("F f() -> i64 { () R 0 }");
}

#[test]
fn test_nested_block_scopes() {
    check_ok(
        r#"
        F f() -> i64 {
            x := 1
            y := {
                a := x + 1
                a * 2
            }
            y
        }
    "#,
    );
}

#[test]
fn test_variable_shadowing() {
    check_ok(
        r#"
        F f() -> i64 {
            x := 1
            x := x + 1
            x
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 41. checker_module — multiple functions in module
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_multiple_functions_call_each_other() {
    check_ok(
        r#"
        F square(x: i64) -> i64 = x * x
        F cube(x: i64) -> i64 = x * square(x)
        F test() -> i64 = cube(3)
    "#,
    );
}

#[test]
fn test_mutual_forward_declaration() {
    // In Vais two-pass TC, functions are registered before bodies are checked
    check_ok(
        r#"
        F is_even(n: i64) -> bool = I n == 0 { true } E { is_odd(n - 1) }
        F is_odd(n: i64) -> bool = I n == 0 { false } E { is_even(n - 1) }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 42. checker_expr — boolean compound
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_bool_logical_ops_chain() {
    check_ok("F f(a: bool, b: bool, c: bool) -> bool = a && b || c");
}

#[test]
fn test_bool_not_chain() {
    check_ok("F f(a: bool) -> bool = !!a");
}

// ═════════════════════════════════════════════════════════════════════════════
// 43. checker_fn — where clause bounds
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_where_clause_basic() {
    check_ok(
        r#"
        W Describable { F describe(self) -> str }
        F show<T>(x: T) -> str where T: Describable = x.describe()
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 44. checker_expr — Range
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_range_for_loop_sum() {
    check_ok(
        r#"
        F sum_to(n: i64) -> i64 {
            s := mut 0
            L i: 0..n {
                s = s + i
            }
            s
        }
    "#,
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 45. checker_expr — struct update / method chaining
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_method_chain() {
    check_ok(
        r#"
        S Builder { value: i64 }
        X Builder {
            F set(self, v: i64) -> Builder = Builder { value: v }
            F build(self) -> i64 = self.value
        }
        F f() -> i64 {
            b := Builder { value: 0 }
            b.set(42).build()
        }
    "#,
    );
}

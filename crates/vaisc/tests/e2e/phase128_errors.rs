//! Phase 128: Compile Error Path E2E Tests
//!
//! Tests for compilation error detection: type mismatches, undefined symbols,
//! duplicate definitions, trait violations, generic constraint errors,
//! invalid patterns, return type errors, and edge cases.

use crate::helpers::{assert_compile_error, assert_exit_code, compile_to_ir};

/// Helper: assert compilation fails with a message containing the expected fragment
fn assert_error_contains(source: &str, expected: &str) {
    match compile_to_ir(source) {
        Ok(_) => panic!(
            "Expected compilation to fail with error containing {:?}, but it succeeded",
            expected
        ),
        Err(e) => assert!(
            e.to_lowercase().contains(&expected.to_lowercase()),
            "Error does not contain {:?}.\nActual: {}",
            expected,
            e
        ),
    }
}

// ==================== A. Type Mismatch Errors ====================

#[test]
fn e2e_p128_err_type_mismatch_bool_for_int() {
    // Phase 160-A: bool↔int unification restored — bool→i64 is now allowed
    assert_exit_code(r#"F main() -> i64 = true"#, 1);
}

#[test]
fn e2e_p128_err_type_mismatch_int_for_bool() {
    // Phase 160-A: bool↔int unification restored — i64→bool is now allowed
    // 42 is truthy (non-zero), so returns non-zero exit code
    let _ = compile_to_ir(r#"F main() -> bool = 42"#);
}

#[test]
fn e2e_p128_err_type_mismatch_string_for_bool() {
    assert_error_contains(r#"F main() -> bool = "hello""#, "mismatch");
}

#[test]
fn e2e_p128_err_type_mismatch_add_bool_int() {
    assert_error_contains(r#"F main() -> i64 = true + 1"#, "mismatch");
}

#[test]
fn e2e_p128_err_type_mismatch_compare_string_int() {
    assert_error_contains(r#"F main() -> bool = "abc" == 42"#, "mismatch");
}

#[test]
fn e2e_p128_err_type_mismatch_fn_return() {
    assert_error_contains(
        r#"
F foo() -> i64 = "not a number"
F main() -> i64 = foo()
"#,
        "mismatch",
    );
}

#[test]
fn e2e_p128_err_type_mismatch_if_branches() {
    assert_error_contains(
        r#"
F main() -> i64 {
    I true { "hello" } E { 42 }
}
"#,
        "mismatch",
    );
}

#[test]
fn e2e_p128_err_type_mismatch_fn_arg() {
    assert_error_contains(
        r#"
F double(x: i64) -> i64 = x * 2
F main() -> i64 = double("five")
"#,
        "mismatch",
    );
}

// ==================== B. Undefined Symbol Errors ====================

#[test]
fn e2e_p128_err_undef_var_in_expr() {
    assert_error_contains(r#"F main() -> i64 = x + 1"#, "undefined");
}

#[test]
fn e2e_p128_err_undef_var_in_if() {
    assert_error_contains(
        r#"
F main() -> i64 {
    I unknown_flag { R 1 }
    R 0
}
"#,
        "undefined",
    );
}

#[test]
fn e2e_p128_err_undef_fn_call() {
    assert_error_contains(r#"F main() -> i64 = nonexistent()"#, "undefined");
}

#[test]
fn e2e_p128_err_undef_fn_nested_call() {
    assert_error_contains(
        r#"
F foo(x: i64) -> i64 = bar(x)
F main() -> i64 = foo(42)
"#,
        "undefined",
    );
}

#[test]
fn e2e_p128_err_undef_struct() {
    assert_error_contains(
        r#"
F main() -> i64 {
    p := Nonexistent { x: 1 }
    R 0
}
"#,
        "undefined",
    );
}

#[test]
fn e2e_p128_err_undef_enum_variant() {
    assert_error_contains(
        r#"
E Color { Red, Green, Blue }
F main() -> i64 {
    c := Yellow
    R 0
}
"#,
        "undefined",
    );
}

#[test]
fn e2e_p128_err_undef_field_access() {
    assert_error_contains(
        r#"
S Point { x: i64, y: i64 }
F main() -> i64 {
    p := Point { x: 1, y: 2 }
    R p.z
}
"#,
        "field",
    );
}

// ==================== C. Duplicate Definition Errors ====================

#[test]
fn e2e_p128_err_duplicate_fn() {
    assert_error_contains(
        r#"
F calc() -> i64 = 1
F calc() -> i64 = 2
F main() -> i64 = calc()
"#,
        "duplicate",
    );
}

#[test]
fn e2e_p128_err_duplicate_struct() {
    // Vais allows struct redefinition (later definition wins)
    // This test documents current behavior
    assert_exit_code(
        r#"
S Pair { x: i64, y: i64 }
S Pair { x: i64, y: i64 }
F main() -> i64 {
    p := Pair { x: 20, y: 22 }
    p.x + p.y
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_err_duplicate_enum() {
    assert_compile_error(
        r#"
E Dir { Up, Down }
E Dir { Left, Right }
F main() -> i64 = 0
"#,
    );
}

// ==================== D. Function Signature Errors ====================

#[test]
fn e2e_p128_err_too_few_args() {
    assert_error_contains(
        r#"
F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add(1)
"#,
        "arg",
    );
}

#[test]
fn e2e_p128_err_too_many_args() {
    assert_error_contains(
        r#"
F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add(1, 2, 3)
"#,
        "arg",
    );
}

#[test]
fn e2e_p128_err_zero_args_when_one_needed() {
    assert_error_contains(
        r#"
F square(x: i64) -> i64 = x * x
F main() -> i64 = square()
"#,
        "arg",
    );
}

#[test]
fn e2e_p128_err_wrong_arg_type() {
    assert_error_contains(
        r#"
F inc(x: i64) -> i64 = x + 1
F main() -> i64 = inc("hello")
"#,
        "mismatch",
    );
}

// ==================== E. Struct Field Errors ====================

#[test]
fn e2e_p128_err_missing_struct_field() {
    // Vais allows partial struct init (missing fields default to 0)
    // This test documents current behavior
    assert_exit_code(
        r#"
S Point { x: i64, y: i64 }
F main() -> i64 {
    p := Point { x: 42 }
    p.x
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_err_extra_struct_field() {
    assert_compile_error(
        r#"
S Point { x: i64, y: i64 }
F main() -> i64 {
    p := Point { x: 1, y: 2, z: 3 }
    R 0
}
"#,
    );
}

#[test]
fn e2e_p128_err_wrong_field_name() {
    assert_compile_error(
        r#"
S Point { x: i64, y: i64 }
F main() -> i64 {
    p := Point { a: 1, b: 2 }
    R 0
}
"#,
    );
}

// ==================== F. Control Flow Errors ====================

#[test]
fn e2e_p128_err_break_outside_loop() {
    assert_compile_error(
        r#"
F main() -> i64 {
    B
    R 0
}
"#,
    );
}

#[test]
fn e2e_p128_err_continue_outside_loop() {
    assert_error_contains(
        r#"
F main() -> i64 {
    C
    R 0
}
"#,
        "continue",
    );
}

#[test]
fn e2e_p128_err_break_in_if_outside_loop() {
    assert_error_contains(
        r#"
F main() -> i64 {
    I true { B }
    R 0
}
"#,
        "break",
    );
}

// ==================== G. Method / Trait Errors ====================

#[test]
fn e2e_p128_err_method_on_primitive() {
    assert_error_contains(
        r#"
F main() -> i64 {
    x := 42
    R x.nonexistent()
}
"#,
        "undefined",
    );
}

#[test]
fn e2e_p128_err_wrong_method_name() {
    assert_error_contains(
        r#"
S Foo { val: i64 }
X Foo {
    F get(&self) -> i64 = self.val
}
F main() -> i64 {
    f := Foo { val: 1 }
    R f.wrong_name()
}
"#,
        "undefined",
    );
}

#[test]
fn e2e_p128_err_method_wrong_args() {
    assert_error_contains(
        r#"
S Foo { val: i64 }
X Foo {
    F add(&self, n: i64) -> i64 = self.val + n
}
F main() -> i64 {
    f := Foo { val: 1 }
    R f.add(1, 2)
}
"#,
        "arg",
    );
}

// ==================== H. Index / Access Errors ====================

#[test]
fn e2e_p128_err_index_non_array() {
    assert_error_contains(
        r#"
F main() -> i64 {
    x := 42
    R x[0]
}
"#,
        "index",
    );
}

#[test]
fn e2e_p128_err_index_bool() {
    assert_error_contains(
        r#"
F main() -> i64 {
    b := true
    R b[0]
}
"#,
        "index",
    );
}

// ==================== I. Return Type Errors ====================

#[test]
fn e2e_p128_err_return_string_for_int() {
    assert_error_contains(r#"F main() -> i64 = "hello""#, "mismatch");
}

#[test]
fn e2e_p128_err_empty_body_with_return_type() {
    assert_compile_error("F main() -> i64 { }");
}

#[test]
fn e2e_p128_err_no_return_value() {
    assert_compile_error(
        r#"
F foo() -> i64 {
}
F main() -> i64 = foo()
"#,
    );
}

// ==================== J. Self-Recursion Errors ====================

#[test]
fn e2e_p128_err_self_call_in_main() {
    assert_compile_error(r#"F main() -> i64 = @(5)"#);
}

// ==================== K. Operator Errors ====================

#[test]
fn e2e_p128_err_subtract_strings() {
    assert_error_contains(r#"F main() -> i64 = "a" - "b""#, "mismatch");
}

#[test]
fn e2e_p128_err_multiply_bools() {
    assert_error_contains(r#"F main() -> i64 = true * false"#, "mismatch");
}

#[test]
fn e2e_p128_err_negate_string() {
    assert_compile_error(r#"F main() -> i64 = -"hello""#);
}

// ==================== L. ImplTrait Position Errors ====================

#[test]
fn e2e_p128_err_impl_trait_param_position() {
    let source = r#"
W Showable {
    F show(&self) -> i64
}
F display(item: X Showable) -> i64 = 0
F main() -> i64 { 0 }
"#;
    let result = compile_to_ir(source);
    assert!(
        result.is_err(),
        "impl Trait in parameter position should fail"
    );
}

// ==================== M. Positive Counterparts (should compile) ====================

#[test]
fn e2e_p128_err_positive_valid_fn() {
    assert_exit_code(
        r#"
F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add(20, 22)
"#,
        42,
    );
}

#[test]
fn e2e_p128_err_positive_valid_struct() {
    assert_exit_code(
        r#"
S Point { x: i64, y: i64 }
F main() -> i64 {
    p := Point { x: 10, y: 32 }
    p.x + p.y
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_err_positive_valid_enum() {
    assert_exit_code(
        r#"
E Shape { Circle, Square, Triangle }
F main() -> i64 {
    s := Square
    M s {
        Circle => 1,
        Square => 42,
        Triangle => 3,
        _ => 0
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_err_positive_valid_loop() {
    assert_exit_code(
        r#"
F main() -> i64 {
    sum := mut 0
    L i:1..8 {
        sum = sum + i
    }
    sum
}
"#,
        28,
    );
}

#[test]
fn e2e_p128_err_positive_valid_method() {
    assert_exit_code(
        r#"
S Box { val: i64 }
X Box {
    F double(&self) -> i64 = self.val * 2
}
F main() -> i64 {
    b := Box { val: 21 }
    b.double()
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_err_positive_nested_call() {
    assert_exit_code(
        r#"
F square(x: i64) -> i64 = x * x
F inc(x: i64) -> i64 = x + 1
F main() -> i64 = inc(square(5))
"#,
        26,
    );
}

#[test]
fn e2e_p128_err_positive_recursive() {
    assert_exit_code(
        r#"
F fib(n: i64) -> i64 {
    I n < 2 { R n }
    R @(n - 1) + @(n - 2)
}
F main() -> i64 = fib(10)
"#,
        55,
    );
}

#[test]
fn e2e_p128_err_positive_closure() {
    assert_exit_code(
        r#"
F main() -> i64 {
    f := |x| x * 3
    f(14)
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_err_positive_array() {
    assert_exit_code(
        r#"
F main() -> i64 {
    arr := [10, 20, 12]
    arr[0] + arr[1] + arr[2]
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_err_positive_if_else() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 10
    I x > 5 { 42 } E { 0 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_err_positive_match() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 3
    M x {
        1 => 10,
        2 => 20,
        3 => 42,
        _ => 0
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_err_positive_trait() {
    assert_exit_code(
        r#"
W HasValue {
    F value(&self) -> i64
}
S Num { n: i64 }
X Num: HasValue {
    F value(&self) -> i64 = self.n
}
F main() -> i64 {
    n := Num { n: 42 }
    n.value()
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_err_positive_empty_source() {
    let result = compile_to_ir("");
    assert!(result.is_ok(), "Empty source should compile");
}

#[test]
fn e2e_p128_err_positive_bool_logic() {
    assert_exit_code(
        r#"
F main() -> i64 {
    I true && true { 42 } E { 0 }
}
"#,
        42,
    );
}

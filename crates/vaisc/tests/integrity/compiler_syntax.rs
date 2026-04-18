/// Syntax coverage tests — Phase 0.2 skeleton (30 tests).
///
/// Each test writes a known-good Vais snippet to a tempdir file and asserts
/// `ok_parse()` or `ok_tc()` returns true. Codegen is NOT exercised here —
/// that is the job of compiler_stages.rs.
///
/// Categories covered (§3 reference: docs/COMPILER_STAGES.md):
///   1. Minimal main fn / empty body / return literal        (3 tests)
///   2. Arithmetic operators (+, -, *, /, %)                 (3 tests)
///   3. Comparison / boolean ops                             (3 tests)
///   4. Control flow: I/EL, L, LW, LF, M                    (5 tests)
///   5. Struct definition + construction + field access      (3 tests)
///   6. Enum definition + Unit/Tuple variants                (3 tests)
///   7. Function: args, return types                         (3 tests)
///   8. String literal + {expr} interpolation               (2 tests)
///   9. Generic fn / struct                                  (3 tests)
///  10. Option/Result (Some/None/Ok/Err)                     (2 tests)
///
/// Phase 1.6 will expand this file to 200+ tests.

use super::{ok_parse, ok_tc};
use std::fs;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Write `src` to a temp file named `name` and return the (TempDir, path).
/// The TempDir is returned so the caller keeps it alive (drop == cleanup).
fn write_tmp(name: &str, src: &str) -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join(name);
    fs::write(&path, src).expect("write temp vais file");
    (dir, path)
}

// ============================================================
// 1. Minimal main fn / empty body / return literal
// ============================================================

#[test]
fn syntax_main_return_zero() {
    let (_d, p) = write_tmp("main_zero.vais", "F main() -> i64 { 0 }");
    assert!(
        ok_tc(&p),
        "ok_tc failed for {}: minimal main returning 0",
        p.display()
    );
}

#[test]
fn syntax_main_return_literal() {
    let (_d, p) = write_tmp("main_lit.vais", "F main() -> i64 { 42 }");
    assert!(
        ok_tc(&p),
        "ok_tc failed for {}: main returning literal 42",
        p.display()
    );
}

#[test]
fn syntax_main_expression_body() {
    // Expression-body (no braces) form
    let (_d, p) = write_tmp("main_expr.vais", "F main() -> i64 = 7");
    assert!(
        ok_tc(&p),
        "ok_tc failed for {}: main expression body",
        p.display()
    );
}

// ============================================================
// 2. Arithmetic operators
// ============================================================

#[test]
fn syntax_arithmetic_add_sub() {
    let src = "F main() -> i64 { x := 10 + 5 - 3\n x }";
    let (_d, p) = write_tmp("arith_add_sub.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: add/sub", p.display());
}

#[test]
fn syntax_arithmetic_mul_div() {
    let src = "F main() -> i64 { x := 6 * 7 / 2\n x }";
    let (_d, p) = write_tmp("arith_mul_div.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: mul/div", p.display());
}

#[test]
fn syntax_arithmetic_modulo() {
    let src = "F main() -> i64 { x := 17 % 5\n x }";
    let (_d, p) = write_tmp("arith_mod.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: modulo", p.display());
}

// ============================================================
// 3. Comparison / boolean ops
// ============================================================

#[test]
fn syntax_comparison_less_than() {
    let src = r#"
F main() -> i64 {
    ok := 3 < 5
    ok ? 1 : 0
}
"#;
    let (_d, p) = write_tmp("cmp_lt.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: less-than comparison", p.display());
}

#[test]
fn syntax_comparison_equality() {
    let src = r#"
F main() -> i64 {
    ok := 5 == 5
    ok ? 1 : 0
}
"#;
    let (_d, p) = write_tmp("cmp_eq.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: equality comparison", p.display());
}

#[test]
fn syntax_boolean_and_or() {
    let src = r#"
F main() -> i64 {
    a := true
    b := false
    c := a && !b
    c ? 1 : 0
}
"#;
    let (_d, p) = write_tmp("bool_and_or.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: boolean and/or/not", p.display());
}

// ============================================================
// 4. Control flow
// ============================================================

#[test]
fn syntax_control_if_else() {
    let src = r#"
F main() -> i64 {
    x := 10
    I x > 5 {
        1
    } E {
        0
    }
}
"#;
    let (_d, p) = write_tmp("ctrl_if_else.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: if/else", p.display());
}

#[test]
fn syntax_control_loop_break() {
    let src = r#"
F main() -> i64 {
    i := mut 0
    L {
        i = i + 1
        I i >= 3 { B }
    }
    i
}
"#;
    let (_d, p) = write_tmp("ctrl_loop_break.vais", src);
    assert!(ok_parse(&p), "ok_parse failed for {}: loop/break", p.display());
}

#[test]
fn syntax_control_while() {
    let src = r#"
F main() -> i64 {
    i := mut 0
    LW i < 5 {
        i = i + 1
    }
    i
}
"#;
    let (_d, p) = write_tmp("ctrl_while.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: while loop (LW)", p.display());
}

#[test]
fn syntax_control_for() {
    let src = r#"
F main() -> i64 {
    sum := mut 0
    LF i: 0..5 {
        sum = sum + i
    }
    sum
}
"#;
    let (_d, p) = write_tmp("ctrl_for.vais", src);
    assert!(ok_parse(&p), "ok_parse failed for {}: for loop (LF)", p.display());
}

#[test]
fn syntax_control_match() {
    let src = r#"
F main() -> i64 {
    x := 2
    M x {
        1 => 10,
        2 => 20,
        _ => 0,
    }
}
"#;
    let (_d, p) = write_tmp("ctrl_match.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: match expression", p.display());
}

// ============================================================
// 5. Struct definition + construction + field access
// ============================================================

#[test]
fn syntax_struct_definition() {
    let src = r#"
S Point {
    x: i64,
    y: i64,
}
F main() -> i64 { 0 }
"#;
    let (_d, p) = write_tmp("struct_def.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: struct definition", p.display());
}

#[test]
fn syntax_struct_construction() {
    let src = r#"
S Point {
    x: i64,
    y: i64,
}
F main() -> i64 {
    p := Point { x: 3, y: 4 }
    p.x
}
"#;
    let (_d, p) = write_tmp("struct_ctor.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: struct construction + field access", p.display());
}

#[test]
fn syntax_struct_method() {
    let src = r#"
S Counter {
    val: i64,
}
X Counter {
    F new() -> Counter {
        Counter { val: 0 }
    }
    F get(self) -> i64 {
        self.val
    }
}
F main() -> i64 {
    c := Counter.new()
    c.get()
}
"#;
    let (_d, p) = write_tmp("struct_method.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: struct impl method", p.display());
}

// ============================================================
// 6. Enum definition + variants
// ============================================================

#[test]
fn syntax_enum_unit_variants() {
    let src = r#"
EN Direction {
    North,
    South,
    East,
    West,
}
F main() -> i64 {
    d := Direction.North
    M d {
        Direction.North => 0,
        _ => 1,
    }
}
"#;
    let (_d, p) = write_tmp("enum_unit.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: enum unit variants", p.display());
}

#[test]
fn syntax_enum_tuple_variant() {
    let src = r#"
EN Shape {
    Circle(i64),
    Rect(i64, i64),
}
F main() -> i64 {
    s := Shape.Circle(5)
    M s {
        Shape.Circle(r) => r,
        Shape.Rect(w, h) => w + h,
    }
}
"#;
    let (_d, p) = write_tmp("enum_tuple.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: enum tuple variants", p.display());
}

#[test]
fn syntax_enum_match_exhaustive() {
    let src = r#"
EN Color { Red, Green, Blue, }
F describe(c: Color) -> i64 {
    M c {
        Color.Red => 1,
        Color.Green => 2,
        Color.Blue => 3,
    }
}
F main() -> i64 { describe(Color.Green) }
"#;
    let (_d, p) = write_tmp("enum_match.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: enum exhaustive match", p.display());
}

// ============================================================
// 7. Functions with args and return types
// ============================================================

#[test]
fn syntax_fn_positional_args() {
    let src = r#"
F add(a: i64, b: i64) -> i64 { a + b }
F main() -> i64 { add(20, 22) }
"#;
    let (_d, p) = write_tmp("fn_args.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: fn positional args", p.display());
}

#[test]
fn syntax_fn_return_bool() {
    let src = r#"
F is_even(n: i64) -> bool { n % 2 == 0 }
F main() -> i64 { is_even(4) ? 0 : 1 }
"#;
    let (_d, p) = write_tmp("fn_ret_bool.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: fn returning bool", p.display());
}

#[test]
fn syntax_fn_recursive() {
    let src = r#"
F fact(n: i64) -> i64 {
    I n <= 1 { R 1 }
    n * @(n - 1)
}
F main() -> i64 { fact(5) }
"#;
    let (_d, p) = write_tmp("fn_recursive.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: recursive function with @ operator", p.display());
}

// ============================================================
// 8. String literal + {expr} interpolation
// ============================================================

#[test]
fn syntax_string_literal() {
    let src = r#"
F main() -> i64 {
    _s := "hello world"
    0
}
"#;
    let (_d, p) = write_tmp("str_literal.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: string literal", p.display());
}

#[test]
fn syntax_string_interpolation() {
    let src = r#"
F main() -> i64 {
    x := 42
    _s := "value is {x}"
    0
}
"#;
    let (_d, p) = write_tmp("str_interp.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: string interpolation", p.display());
}

// ============================================================
// 9. Generic fn / struct
// ============================================================

#[test]
fn syntax_generic_fn() {
    let src = r#"
F identity<T>(x: T) -> T { x }
F main() -> i64 { identity(99) }
"#;
    let (_d, p) = write_tmp("generic_fn.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: generic function", p.display());
}

#[test]
fn syntax_generic_struct() {
    let src = r#"
S Pair<A, B> {
    first: A,
    second: B,
}
F main() -> i64 {
    p := Pair { first: 1, second: 2 }
    p.first + p.second
}
"#;
    let (_d, p) = write_tmp("generic_struct.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: generic struct", p.display());
}

#[test]
fn syntax_generic_fn_two_params() {
    let src = r#"
F swap<A, B>(a: A, b: B) -> B { b }
F main() -> i64 { swap(1, 42) }
"#;
    let (_d, p) = write_tmp("generic_swap.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: generic fn two type params", p.display());
}

// ============================================================
// 10. Option / Result
// ============================================================

#[test]
fn syntax_option_some_none() {
    let src = r#"
F maybe(b: bool) -> Option<i64> {
    I b { Some(42) } E { None }
}
F main() -> i64 {
    v := maybe(true)
    M v {
        Some(x) => x,
        None => 0,
    }
}
"#;
    let (_d, p) = write_tmp("option_some_none.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: Option Some/None", p.display());
}

#[test]
fn syntax_result_ok_err() {
    let src = r#"
F divide(a: i64, b: i64) -> Result<i64, i64> {
    I b == 0 { Err(-1) } E { Ok(a / b) }
}
F main() -> i64 {
    r := divide(10, 2)
    M r {
        Ok(v) => v,
        Err(_) => 0,
    }
}
"#;
    let (_d, p) = write_tmp("result_ok_err.vais", src);
    assert!(ok_tc(&p), "ok_tc failed for {}: Result Ok/Err", p.display());
}

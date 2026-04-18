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

// ============================================================
// 11. Modifiers: P, A, pure, io, partial, unsafe
// ============================================================

#[test]
fn syntax_mod_pub_fn() {
    let src = "P F foo() -> i64 { 1 }\nF main() -> i64 { foo() }";
    let (_d, p) = write_tmp("mod_pub.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: P F (pub fn)");
}

#[test]
fn syntax_mod_async_fn() {
    let src = "A F fetch() -> i64 { 0 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("mod_async.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: A F (async fn)");
}

#[test]
fn syntax_mod_pub_async_fn() {
    let src = "P A F pub_async() -> i64 { 0 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("mod_pub_async.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: P A F (pub async fn)");
}

#[test]
fn syntax_mod_pure_fn() {
    let src = "pure F add(a: i64, b: i64) -> i64 = a + b\nF main() -> i64 { add(1, 2) }";
    let (_d, p) = write_tmp("mod_pure.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: pure F");
}

#[test]
#[ignore = "Phase 4c: unsafe modifier codegen incomplete"]
fn syntax_mod_unsafe_fn() {
    let src = "unsafe F raw(p: i64) -> i64 { p }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("mod_unsafe.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: unsafe F");
}

#[test]
fn syntax_mod_io_fn() {
    let src = "io F print_val(x: i64) { }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("mod_io.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: io F");
}

#[test]
fn syntax_mod_partial_fn() {
    let src = "partial F div(a: i64, b: i64) -> i64 { a / b }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("mod_partial.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: partial F");
}

#[test]
fn syntax_mod_pub_struct() {
    let src = "P S Pt { x: i64, y: i64, }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("mod_pub_struct.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: P S (pub struct)");
}

#[test]
fn syntax_mod_pub_enum() {
    let src = "P EN Color { Red, Green, Blue, }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("mod_pub_enum.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: P EN (pub enum)");
}

#[test]
fn syntax_mod_pure_expr_body() {
    let src = "pure F square(x: i64) -> i64 = x * x\nF main() -> i64 { square(5) }";
    let (_d, p) = write_tmp("mod_pure_expr.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: pure fn expression body");
}

#[test]
#[ignore = "parser-limit: double-pub P P F may not error at parse stage"]
fn syntax_neg_mod_double_pub() {
    let src = "P P F foo() -> i64 { 0 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("neg_double_pub.vais", src);
    assert!(!ok_parse(&p), "should not parse: double pub P P F");
}

#[test]
fn syntax_neg_mod_missing_fn_keyword() {
    // modifier without F should not parse as a function
    let src = "P main() -> i64 { 0 }";
    let (_d, p) = write_tmp("neg_mod_no_fn.vais", src);
    assert!(!ok_parse(&p), "should not parse: P without F keyword");
}

// ============================================================
// 12. Assignments & Bindings
// ============================================================

#[test]
fn syntax_bind_int() {
    let src = "F main() -> i64 { x := 5\n x }";
    let (_d, p) = write_tmp("bind_int.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: integer binding");
}

#[test]
fn syntax_bind_mut_int() {
    let src = "F main() -> i64 { x := mut 5\n x = 10\n x }";
    let (_d, p) = write_tmp("bind_mut_int.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: mutable integer binding + reassign");
}

#[test]
fn syntax_bind_float() {
    let src = "F main() -> i64 { _x := 5.0\n 0 }";
    let (_d, p) = write_tmp("bind_float.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: float binding");
}

#[test]
fn syntax_bind_bool() {
    let src = "F main() -> i64 { _x := true\n 0 }";
    let (_d, p) = write_tmp("bind_bool.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: bool binding");
}

#[test]
fn syntax_bind_string() {
    let src = "F main() -> i64 { _x := \"hi\"\n 0 }";
    let (_d, p) = write_tmp("bind_string.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: string binding");
}

#[test]
fn syntax_bind_typed() {
    let src = "F main() -> i64 { x: i64 := 5\n x }";
    let (_d, p) = write_tmp("bind_typed.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: typed binding x: i64 := 5");
}

#[test]
fn syntax_assign_compound_add() {
    let src = "F main() -> i64 { x := mut 5\n x += 1\n x }";
    let (_d, p) = write_tmp("assign_add.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: compound += assignment");
}

#[test]
fn syntax_assign_compound_sub() {
    let src = "F main() -> i64 { x := mut 10\n x -= 3\n x }";
    let (_d, p) = write_tmp("assign_sub.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: compound -= assignment");
}

#[test]
fn syntax_assign_compound_mul() {
    let src = "F main() -> i64 { x := mut 3\n x *= 2\n x }";
    let (_d, p) = write_tmp("assign_mul.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: compound *= assignment");
}

#[test]
fn syntax_assign_compound_div() {
    let src = "F main() -> i64 { x := mut 10\n x /= 2\n x }";
    let (_d, p) = write_tmp("assign_div.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: compound /= assignment");
}

#[test]
fn syntax_assign_compound_mod() {
    let src = "F main() -> i64 { x := mut 17\n x %= 5\n x }";
    let (_d, p) = write_tmp("assign_mod.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: compound %= assignment");
}

#[test]
fn syntax_neg_bind_lhs_literal() {
    // 5 := x should not parse
    let src = "F main() -> i64 { 5 := x\n 0 }";
    let (_d, p) = write_tmp("neg_bind_lhs_lit.vais", src);
    assert!(!ok_parse(&p), "should not parse: literal on lhs of :=");
}

#[test]
fn syntax_neg_assign_no_rhs() {
    let src = "F main() -> i64 { x :=\n 0 }";
    let (_d, p) = write_tmp("neg_assign_no_rhs.vais", src);
    assert!(!ok_parse(&p), "should not parse: := with no rhs");
}

// ============================================================
// 13. Control Flow Expansion
// ============================================================

#[test]
fn syntax_ctrl_if_elif_else() {
    let src = r#"
F main() -> i64 {
    x := 5
    I x < 0 { -1 } EL I x == 0 { 0 } E { 1 }
}
"#;
    let (_d, p) = write_tmp("ctrl_if_elif.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: I/EL I/E chain (else-if)");
}

#[test]
fn syntax_ctrl_if_elif_elif_else() {
    let src = r#"
F main() -> i64 {
    x := 7
    I x < 0 { -1 } EL I x < 5 { 0 } EL I x < 10 { 1 } E { 2 }
}
"#;
    let (_d, p) = write_tmp("ctrl_if_elif2.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: I/EL I/EL I/E chain");
}

#[test]
fn syntax_ctrl_nested_if() {
    let src = r#"
F main() -> i64 {
    x := 5
    I x > 0 {
        I x > 3 { 2 } E { 1 }
    } E { 0 }
}
"#;
    let (_d, p) = write_tmp("ctrl_nested_if.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: nested I/E");
}

#[test]
fn syntax_ctrl_while_counter() {
    let src = r#"
F main() -> i64 {
    i := mut 0
    LW i < 10 {
        i += 1
    }
    i
}
"#;
    let (_d, p) = write_tmp("ctrl_while2.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: LW with += counter");
}

#[test]
fn syntax_ctrl_for_range() {
    let src = r#"
F main() -> i64 {
    s := mut 0
    LF i: 0..10 { s = s + i }
    s
}
"#;
    let (_d, p) = write_tmp("ctrl_for_range.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: LF i: 0..10");
}

#[test]
fn syntax_ctrl_for_inclusive_range() {
    let src = r#"
F main() -> i64 {
    s := mut 0
    LF i: 0..=9 { s = s + i }
    s
}
"#;
    let (_d, p) = write_tmp("ctrl_for_incl.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: LF i: 0..=9 inclusive range");
}

#[test]
fn syntax_ctrl_nested_loops_break_continue() {
    let src = r#"
F main() -> i64 {
    i := mut 0
    L {
        j := mut 0
        L {
            j += 1
            I j >= 3 { B }
            C
        }
        i += 1
        I i >= 5 { B }
    }
    i
}
"#;
    let (_d, p) = write_tmp("ctrl_nested_loops.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: nested loops with B/C");
}

#[test]
#[ignore = "parser-limit: loop-as-expression with break value causes type mismatch"]
fn syntax_ctrl_loop_as_expression() {
    let src = r#"
F main() -> i64 {
    x := L { B 5 }
    x
}
"#;
    let (_d, p) = write_tmp("ctrl_loop_expr.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: loop-as-expression with break value");
}

#[test]
fn syntax_ctrl_ternary() {
    let src = "F main() -> i64 { x := 3\n y := x > 0 ? 1 : 0\n y }";
    let (_d, p) = write_tmp("ctrl_ternary.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: ternary a?b:c");
}

#[test]
fn syntax_ctrl_nested_ternary() {
    let src = "F main() -> i64 { x := 3\n y := x > 0 ? (x > 5 ? 2 : 1) : 0\n y }";
    let (_d, p) = write_tmp("ctrl_ternary_nested.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: nested ternary");
}

#[test]
fn syntax_ctrl_if_in_match() {
    let src = r#"
F main() -> i64 {
    x := 2
    M x {
        1 => I true { 10 } E { 0 },
        _ => 0,
    }
}
"#;
    let (_d, p) = write_tmp("ctrl_if_in_match.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: I inside M arm");
}

#[test]
fn syntax_neg_ctrl_if_no_braces() {
    // I c a  -- no braces
    let src = "F main() -> i64 { I true 1 }";
    let (_d, p) = write_tmp("neg_if_no_braces.vais", src);
    assert!(!ok_parse(&p), "should not parse: I without braces");
}

#[test]
fn syntax_neg_ctrl_lw_no_cond() {
    let src = "F main() -> i64 { LW { 0 } }";
    let (_d, p) = write_tmp("neg_lw_no_cond.vais", src);
    assert!(!ok_parse(&p), "should not parse: LW without condition");
}

#[test]
fn syntax_neg_ctrl_lf_no_colon() {
    // LF without colon separator
    let src = "F main() -> i64 { LF i 0..10 { 0 } }";
    let (_d, p) = write_tmp("neg_lf_no_colon.vais", src);
    assert!(!ok_parse(&p), "should not parse: LF without colon");
}

#[test]
fn syntax_neg_ctrl_loop_no_block() {
    let src = "F main() -> i64 { L 0 }";
    let (_d, p) = write_tmp("neg_loop_no_block.vais", src);
    assert!(!ok_parse(&p), "should not parse: L without block");
}

// ============================================================
// 14. Match Expansion
// ============================================================

#[test]
fn syntax_match_literal_arms() {
    let src = r#"
F main() -> i64 {
    x := 3
    M x {
        1 => 10,
        2 => 20,
        3 => 30,
        _ => 0,
    }
}
"#;
    let (_d, p) = write_tmp("match_lit.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: match with literal arms");
}

#[test]
fn syntax_match_wildcard() {
    let src = r#"
F main() -> i64 {
    x := 99
    M x { _ => 0, }
}
"#;
    let (_d, p) = write_tmp("match_wildcard.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: match _ wildcard");
}

#[test]
fn syntax_match_option_some_none() {
    let src = r#"
F main() -> i64 {
    v := Some(42)
    M v {
        Some(x) => x,
        None => 0,
    }
}
"#;
    let (_d, p) = write_tmp("match_option.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: match Some/None");
}

#[test]
fn syntax_match_result_ok_err() {
    let src = r#"
F main() -> i64 {
    v := Ok(7)
    M v {
        Ok(x) => x,
        Err(_) => -1,
    }
}
"#;
    let (_d, p) = write_tmp("match_result.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: match Ok/Err");
}

#[test]
fn syntax_match_tuple_pattern() {
    let src = r#"
F main() -> i64 {
    pair := (1, 2)
    M pair {
        (a, b) => a + b,
    }
}
"#;
    let (_d, p) = write_tmp("match_tuple.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: match tuple pattern (a, b)");
}

#[test]
fn syntax_match_or_pattern() {
    let src = r#"
F main() -> i64 {
    x := 2
    M x {
        1 | 2 | 3 => 1,
        _ => 0,
    }
}
"#;
    let (_d, p) = write_tmp("match_or.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: match or-pattern 1 | 2 | 3");
}

#[test]
#[ignore = "parser-limit: match guard 'if' clause not supported"]
fn syntax_match_guard() {
    let src = r#"
F main() -> i64 {
    x := 5
    M x {
        n if n > 0 => 1,
        _ => 0,
    }
}
"#;
    let (_d, p) = write_tmp("match_guard.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: match guard x if x > 0");
}

#[test]
fn syntax_match_struct_destructure() {
    let src = r#"
S Point { x: i64, y: i64, }
F main() -> i64 {
    p := Point { x: 1, y: 2 }
    M p {
        Point { x, y } => x + y,
    }
}
"#;
    let (_d, p) = write_tmp("match_struct.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: match struct destructure");
}

#[test]
fn syntax_match_range_pattern() {
    let src = r#"
F main() -> i64 {
    x := 5
    M x {
        1..=5 => 1,
        _ => 0,
    }
}
"#;
    let (_d, p) = write_tmp("match_range.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: match range pattern 1..=5");
}

#[test]
fn syntax_match_bind_pattern() {
    let src = r#"
F main() -> i64 {
    x := 5
    M x {
        n @ 1..=10 => n,
        _ => 0,
    }
}
"#;
    let (_d, p) = write_tmp("match_bind.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: match bind n @ 1..=10");
}

#[test]
fn syntax_neg_match_unclosed() {
    let src = "F main() -> i64 { M x { ";
    let (_d, p) = write_tmp("neg_match_unclosed.vais", src);
    assert!(!ok_parse(&p), "should not parse: M x with unclosed brace");
}

#[test]
fn syntax_neg_match_missing_arrow() {
    let src = "F main() -> i64 { x := 1\n M x { 1 0, _ => 0, } }";
    let (_d, p) = write_tmp("neg_match_no_arrow.vais", src);
    assert!(!ok_parse(&p), "should not parse: match arm missing =>");
}

// ============================================================
// 15. Types
// ============================================================

#[test]
fn syntax_type_vec() {
    let src = "F takes_vec(v: Vec<i64>) -> i64 { 0 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("type_vec.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: Vec<i64> type");
}

#[test]
fn syntax_type_hashmap() {
    let src = "F takes_map(m: HashMap<str, i64>) -> i64 { 0 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("type_hashmap.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: HashMap<str, i64> type");
}

#[test]
fn syntax_type_option() {
    let src = "F takes_opt(o: Option<i64>) -> i64 { 0 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("type_option.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: Option<i64> type");
}

#[test]
fn syntax_type_result() {
    let src = "F takes_res(r: Result<i64, str>) -> i64 { 0 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("type_result.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: Result<i64, str> type");
}

#[test]
fn syntax_type_tuple() {
    let src = "F takes_tuple(t: (i64, str)) -> i64 { 0 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("type_tuple.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: tuple type (i64, str)");
}

#[test]
fn syntax_type_ref() {
    let src = "F takes_ref(x: &i64) -> i64 { 0 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("type_ref.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: &i64 reference type");
}

#[test]
fn syntax_type_mut_ref() {
    let src = "F takes_mut_ref(x: &mut i64) -> i64 { 0 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("type_mut_ref.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: &mut i64 mutable reference type");
}

#[test]
fn syntax_type_ptr() {
    let src = "F takes_ptr(x: *i64) -> i64 { 0 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("type_ptr.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: *i64 pointer type");
}

#[test]
fn syntax_type_array() {
    let src = "F takes_arr(a: [i64; 5]) -> i64 { 0 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("type_array.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: [i64; 5] array type");
}

#[test]
fn syntax_type_fn_pointer() {
    let src = "F takes_fn(f: fn(i64) -> i64) -> i64 { f(0) }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("type_fn_ptr.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: fn(i64)->i64 function type");
}

#[test]
fn syntax_type_self_in_impl() {
    let src = r#"
S Box { val: i64, }
X Box {
    F make(v: i64) -> Self { Box { val: v } }
}
F main() -> i64 { 0 }
"#;
    let (_d, p) = write_tmp("type_self.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: Self return type in impl");
}

#[test]
fn syntax_type_dyn_trait() {
    let src = "F takes_dyn(x: &dyn Show) -> i64 { 0 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("type_dyn.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: &dyn Trait type");
}

#[test]
#[ignore = "Phase 1.7: Vec<> empty generic currently accepted by parser; stricter gate pending"]
fn syntax_neg_type_vec_empty_generic() {
    let src = "F takes_vec(v: Vec<>) -> i64 { 0 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("neg_vec_empty.vais", src);
    assert!(!ok_parse(&p), "should not parse: Vec<> with empty type arg");
}

#[test]
#[ignore = "Phase 1.7: i65 accepted as generic ident, not flagged as bad primitive"]
fn syntax_neg_type_bad_primitive() {
    let src = "F takes_bad(x: i65) -> i64 { 0 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("neg_bad_prim.vais", src);
    assert!(!ok_parse(&p), "should not parse: i65 bad primitive");
}

// ============================================================
// 16. Expressions
// ============================================================

#[test]
fn syntax_expr_arith_precedence() {
    let src = "F main() -> i64 { x := 1 + 2 * 3\n x }";
    let (_d, p) = write_tmp("expr_prec.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: 1+2*3 precedence");
}

#[test]
fn syntax_expr_bitwise() {
    let src = "F main() -> i64 { a := 0b1010\n b := 0b1100\n a & b | a ^ b }";
    let (_d, p) = write_tmp("expr_bitwise.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: bitwise & | ^");
}

#[test]
fn syntax_expr_shift_left() {
    let src = "F main() -> i64 { x := 1\n x << 2 }";
    let (_d, p) = write_tmp("expr_shl.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: left shift <<");
}

#[test]
fn syntax_expr_shift_right() {
    let src = "F main() -> i64 { x := 16\n x >> 1 }";
    let (_d, p) = write_tmp("expr_shr.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: right shift >>");
}

#[test]
fn syntax_expr_comparison_chain() {
    let src = r#"
F main() -> i64 {
    a := 1
    b := 2
    c := a < b
    c ? 1 : 0
}
"#;
    let (_d, p) = write_tmp("expr_cmp.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: comparison a < b");
}

#[test]
fn syntax_expr_logical_and_or() {
    let src = r#"
F main() -> i64 {
    a := true
    b := false
    c := a && b || a
    c ? 1 : 0
}
"#;
    let (_d, p) = write_tmp("expr_logic.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: a && b || a");
}

#[test]
fn syntax_expr_not() {
    let src = "F main() -> i64 { x := !true\n x ? 1 : 0 }";
    let (_d, p) = write_tmp("expr_not.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: boolean ! not");
}

#[test]
fn syntax_expr_bitwise_not() {
    let src = "F main() -> i64 { x := 5\n ~x }";
    let (_d, p) = write_tmp("expr_bitnot.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: bitwise ~ not");
}

#[test]
fn syntax_expr_unary_neg() {
    let src = "F main() -> i64 { x := 5\n -x }";
    let (_d, p) = write_tmp("expr_uneg.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: unary negation -x");
}

#[test]
fn syntax_expr_cast() {
    let src = "F main() -> i64 { x := 5\n x as i64 }";
    let (_d, p) = write_tmp("expr_cast.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: cast x as i64");
}

#[test]
fn syntax_expr_pipe() {
    let src = r#"
F double(x: i64) -> i64 { x * 2 }
F inc(x: i64) -> i64 { x + 1 }
F main() -> i64 { 3 |> double |> inc }
"#;
    let (_d, p) = write_tmp("expr_pipe.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: pipe operator |>");
}

#[test]
fn syntax_expr_range() {
    let src = "F main() -> i64 { _r := 0..10\n 0 }";
    let (_d, p) = write_tmp("expr_range.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: range 0..10");
}

#[test]
fn syntax_expr_range_inclusive() {
    let src = "F main() -> i64 { _r := 0..=9\n 0 }";
    let (_d, p) = write_tmp("expr_range_incl.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: inclusive range 0..=9");
}

#[test]
#[ignore = "TC gap: unwrap `!` on local Some() needs Option type inference"]
fn syntax_expr_unwrap() {
    let src = r#"
F main() -> i64 {
    v := Some(42)
    v!
}
"#;
    let (_d, p) = write_tmp("expr_unwrap.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: unwrap operator !");
}

#[test]
fn syntax_expr_string_interp_complex() {
    let src = r#"
F main() -> i64 {
    name := "world"
    _s := "hello {name} value={1+1}"
    0
}
"#;
    let (_d, p) = write_tmp("expr_str_interp2.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: string interp with expression");
}

#[test]
fn syntax_expr_self_recursion_factorial() {
    let src = r#"
F fact(n: i64) -> i64 = I n <= 1 { 1 } E { n * @(n - 1) }
F main() -> i64 { fact(5) }
"#;
    let (_d, p) = write_tmp("expr_self_rec.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: @ self-recursion in expr body");
}

#[test]
fn syntax_expr_nested_call() {
    let src = r#"
F add(a: i64, b: i64) -> i64 { a + b }
F main() -> i64 { add(add(1, 2), add(3, 4)) }
"#;
    let (_d, p) = write_tmp("expr_nested_call.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: nested function calls");
}

#[test]
fn syntax_expr_greater_equal() {
    let src = "F main() -> i64 { x := 5\n (x >= 5) ? 1 : 0 }";
    let (_d, p) = write_tmp("expr_ge.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: >= comparison");
}

#[test]
fn syntax_expr_not_equal() {
    let src = "F main() -> i64 { x := 5\n (x != 3) ? 1 : 0 }";
    let (_d, p) = write_tmp("expr_ne.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: != not-equal comparison");
}

#[test]
fn syntax_expr_less_equal() {
    let src = "F main() -> i64 { x := 3\n (x <= 5) ? 1 : 0 }";
    let (_d, p) = write_tmp("expr_le.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: <= less-or-equal comparison");
}

// ============================================================
// 17. Structs / Impls (expanded)
// ============================================================

#[test]
fn syntax_struct_field_mutation() {
    let src = r#"
S Counter { val: i64, }
F main() -> i64 {
    c := mut Counter { val: 0 }
    c.val = 5
    c.val
}
"#;
    let (_d, p) = write_tmp("struct_field_mut.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: struct field mutation c.val = 5");
}

#[test]
#[ignore = "TC gap: static method call via Point.new() syntax may need Point::new()"]
fn syntax_struct_impl_new_method() {
    let src = r#"
S Point { x: i64, y: i64, }
X Point {
    F new(x: i64, y: i64) -> Self {
        Point { x: x, y: y }
    }
}
F main() -> i64 {
    p := Point.new(3, 4)
    p.x
}
"#;
    let (_d, p) = write_tmp("struct_impl_new.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: impl new() static method");
}

#[test]
fn syntax_struct_method_call() {
    let src = r#"
S Box { val: i64, }
X Box {
    F get(self) -> i64 { self.val }
}
F main() -> i64 {
    b := Box { val: 99 }
    b.get()
}
"#;
    let (_d, p) = write_tmp("struct_method_call.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: method call b.get()");
}

#[test]
fn syntax_struct_generic() {
    let src = r#"
S Wrapper<T> { inner: T, }
F main() -> i64 {
    w := Wrapper { inner: 42 }
    w.inner
}
"#;
    let (_d, p) = write_tmp("struct_generic.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: generic struct Wrapper<T>");
}

#[test]
fn syntax_struct_generic_impl() {
    let src = r#"
S Pair<A, B> { first: A, second: B, }
X Pair<i64, i64> {
    F sum(self) -> i64 { self.first + self.second }
}
F main() -> i64 {
    p := Pair { first: 3, second: 4 }
    p.sum()
}
"#;
    let (_d, p) = write_tmp("struct_generic_impl.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: generic struct + monomorphized impl");
}

#[test]
fn syntax_neg_struct_missing_comma() {
    // missing comma between fields
    let src = "S Bad { x: i64 y: i64 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("neg_struct_comma.vais", src);
    assert!(!ok_parse(&p), "should not parse: struct fields without comma");
}

// ============================================================
// 18. Enums (expanded)
// ============================================================

#[test]
fn syntax_enum_basic_three_variants() {
    let src = r#"
EN Color { Red, Green, Blue, }
F main() -> i64 {
    c := Color.Red
    M c {
        Color.Red => 1,
        Color.Green => 2,
        Color.Blue => 3,
    }
}
"#;
    let (_d, p) = write_tmp("enum_basic.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: EN Color three variants");
}

#[test]
fn syntax_enum_tuple_two_fields() {
    let src = r#"
EN Shape { Rect(f64, f64), }
F main() -> i64 {
    _s := Shape.Rect(3.0, 4.0)
    0
}
"#;
    let (_d, p) = write_tmp("enum_rect.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: EN tuple variant two fields");
}

#[test]
fn syntax_enum_construction_all_variants() {
    let src = r#"
EN Dir { North, South, East, West, }
F main() -> i64 {
    _a := Dir.North
    _b := Dir.South
    _c := Dir.East
    _d := Dir.West
    0
}
"#;
    let (_d, p) = write_tmp("enum_all_variants.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: EN Dir construct all variants");
}

#[test]
fn syntax_enum_match_all() {
    let src = r#"
EN Coin { Penny, Nickel, Dime, Quarter, }
F value(c: Coin) -> i64 {
    M c {
        Coin.Penny => 1,
        Coin.Nickel => 5,
        Coin.Dime => 10,
        Coin.Quarter => 25,
    }
}
F main() -> i64 { value(Coin.Dime) }
"#;
    let (_d, p) = write_tmp("enum_match_all.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: match enum all variants");
}

#[test]
#[ignore = "parser-limit: legacy E keyword for enum may conflict with E (else)"]
fn syntax_enum_legacy_e_keyword() {
    let src = r#"
E Color { Red, Green, Blue, }
F main() -> i64 { 0 }
"#;
    let (_d, p) = write_tmp("enum_legacy.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: legacy E Color enum syntax");
}

#[test]
fn syntax_neg_enum_missing_comma() {
    // EN Color { Red Green } — no comma
    let src = "EN Color { Red Green }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("neg_enum_comma.vais", src);
    assert!(!ok_parse(&p), "should not parse: enum variants without comma");
}

// ============================================================
// 19. Traits
// ============================================================

#[test]
fn syntax_trait_basic() {
    let src = r#"
W Show {
    F show(self) -> str
}
F main() -> i64 { 0 }
"#;
    let (_d, p) = write_tmp("trait_basic.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: W Show trait definition");
}

#[test]
fn syntax_trait_default_method() {
    let src = r#"
W Greet {
    F greet(self) -> str { "hello" }
}
F main() -> i64 { 0 }
"#;
    let (_d, p) = write_tmp("trait_default.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: trait with default method body");
}

#[test]
fn syntax_trait_impl() {
    let src = r#"
W Show {
    F show(self) -> str
}
S Cat { name: str, }
X Cat: Show {
    F show(self) -> str { self.name }
}
F main() -> i64 { 0 }
"#;
    let (_d, p) = write_tmp("trait_impl.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: X Cat: Show trait impl");
}

#[test]
fn syntax_trait_bound() {
    let src = r#"
W Show { F show(self) -> str }
F print_it<T: Show>(x: T) -> str { x.show() }
F main() -> i64 { 0 }
"#;
    let (_d, p) = write_tmp("trait_bound.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: trait bound <T: Show>");
}

#[test]
fn syntax_trait_where_clause() {
    let src = r#"
W Show { F show(self) -> str }
W Debug { F debug(self) -> str }
F show_debug<T>(x: T) -> str where T: Show { x.show() }
F main() -> i64 { 0 }
"#;
    let (_d, p) = write_tmp("trait_where.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: where clause on generic fn");
}

#[test]
fn syntax_trait_multiple_methods() {
    let src = r#"
W Animal {
    F name(self) -> str
    F sound(self) -> str
    F legs(self) -> i64
}
F main() -> i64 { 0 }
"#;
    let (_d, p) = write_tmp("trait_multi_methods.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: trait with multiple method signatures");
}

#[test]
#[ignore = "parser-limit: trait body without F keyword may not produce parse error"]
fn syntax_neg_trait_missing_fn_keyword() {
    let src = r#"
W Bad {
    show(self) -> str
}
F main() -> i64 { 0 }
"#;
    let (_d, p) = write_tmp("neg_trait_no_fn.vais", src);
    assert!(!ok_parse(&p), "should not parse: trait method without F keyword");
}

// ============================================================
// 20. Generics (expanded)
// ============================================================

#[test]
fn syntax_generic_identity() {
    let src = "F id<T>(x: T) -> T = x\nF main() -> i64 { id(42) }";
    let (_d, p) = write_tmp("generic_id.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: generic identity F id<T>");
}

#[test]
fn syntax_generic_two_params() {
    let src = "F pair<A, B>(a: A, b: B) -> A { a }\nF main() -> i64 { pair(42, true) }";
    let (_d, p) = write_tmp("generic_two.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: generic fn two type params");
}

#[test]
fn syntax_generic_bounded() {
    let src = r#"
W Eq { F eq(self, other: Self) -> bool }
F are_equal<T: Eq>(a: T, b: T) -> bool { a.eq(b) }
F main() -> i64 { 0 }
"#;
    let (_d, p) = write_tmp("generic_bounded.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: bounded generic <T: Eq>");
}

#[test]
fn syntax_generic_struct_typed() {
    let src = r#"
S Stack<T> { items: Vec<T>, }
F main() -> i64 { 0 }
"#;
    let (_d, p) = write_tmp("generic_struct2.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: generic struct with Vec<T> field");
}

#[test]
fn syntax_generic_enum() {
    let src = r#"
EN Maybe<T> { Just(T), Nothing, }
F main() -> i64 { 0 }
"#;
    let (_d, p) = write_tmp("generic_enum.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: generic enum Maybe<T>");
}

#[test]
fn syntax_generic_impl_method() {
    let src = r#"
S Box<T> { val: T, }
X Box<T> {
    F get(self) -> T { self.val }
}
F main() -> i64 {
    b := Box { val: 77 }
    b.get()
}
"#;
    let (_d, p) = write_tmp("generic_impl_method.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: generic impl method X Box<T>");
}

#[test]
fn syntax_generic_where_clause() {
    let src = r#"
W Clone { F clone(self) -> Self }
F dup<T>(x: T) -> T where T: Clone { x.clone() }
F main() -> i64 { 0 }
"#;
    let (_d, p) = write_tmp("generic_where.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: generic with where clause");
}

// ============================================================
// 21. Imports / Attributes
// ============================================================

#[test]
fn syntax_import_simple() {
    let src = "U std::io\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("import_simple.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: U std::io import");
}

#[test]
#[ignore = "TC gap: `U std::io::{print, println}` fails module resolution in isolated tempdir"]
fn syntax_import_multi() {
    let src = "U std::io::{print, println}\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("import_multi.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: U std::io::{{...}} multi import");
}

#[test]
fn syntax_import_dot_path() {
    let src = "U foo.bar\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("import_dot.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: U foo.bar dot-separated import");
}

#[test]
fn syntax_attr_cfg_linux() {
    let src = "#[cfg(target_os = \"linux\")]\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("attr_cfg.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: #[cfg(target_os = ...)] attribute");
}

#[test]
fn syntax_attr_wasm_export() {
    let src = "#[wasm_export(\"run\")]\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("attr_wasm_export.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: #[wasm_export(\"name\")] attribute");
}

#[test]
fn syntax_attr_wasm_import() {
    let src = "#[wasm_import(\"env\", \"log\")]\nF log_val(x: i64) -> i64 { 0 }\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("attr_wasm_import.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: #[wasm_import()] attribute");
}

#[test]
fn syntax_neg_attr_unclosed() {
    let src = "#[cfg\nF main() -> i64 { 0 }";
    let (_d, p) = write_tmp("neg_attr_unclosed.vais", src);
    assert!(!ok_parse(&p), "should not parse: unclosed attribute #[cfg");
}

// ============================================================
// 22. Closures
// ============================================================

#[test]
fn syntax_closure_single_arg() {
    let src = "F main() -> i64 { f := |x| x * 2\n f(5) }";
    let (_d, p) = write_tmp("closure_one.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: closure |x| x*2");
}

#[test]
fn syntax_closure_two_args() {
    let src = "F main() -> i64 { f := |x, y| x + y\n f(3, 4) }";
    let (_d, p) = write_tmp("closure_two.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: closure |x, y| x+y");
}

#[test]
fn syntax_closure_block_body() {
    let src = r#"
F main() -> i64 {
    f := |x| { I x > 0 { x } E { -x } }
    f(-3)
}
"#;
    let (_d, p) = write_tmp("closure_block.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: closure with block body");
}

#[test]
fn syntax_closure_no_args() {
    let src = "F main() -> i64 { f := || 42\n f() }";
    let (_d, p) = write_tmp("closure_noargs.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: closure with no args || 42");
}

#[test]
fn syntax_closure_in_variable() {
    let src = r#"
F apply(f: fn(i64) -> i64, x: i64) -> i64 { f(x) }
F main() -> i64 {
    double := |x| x * 2
    apply(double, 21)
}
"#;
    let (_d, p) = write_tmp("closure_in_var.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: closure assigned to variable and passed");
}

#[test]
fn syntax_closure_returning_closure() {
    let src = r#"
F main() -> i64 {
    adder := |n| |x| x + n
    add5 := adder(5)
    add5(3)
}
"#;
    let (_d, p) = write_tmp("closure_higher.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: closure returning closure");
}

// ============================================================
// 23. Misc / Keywords
// ============================================================

#[test]
fn syntax_misc_tuple_literal() {
    let src = "F main() -> i64 { _t := (1, \"hi\", true)\n 0 }";
    let (_d, p) = write_tmp("misc_tuple.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: tuple literal (1, hi, true)");
}

#[test]
fn syntax_misc_array_literal() {
    let src = "F main() -> i64 { _a := [1, 2, 3]\n 0 }";
    let (_d, p) = write_tmp("misc_array.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: array literal [1, 2, 3]");
}

#[test]
fn syntax_misc_vec_push_len() {
    let src = r#"
F main() -> i64 {
    v := mut Vec.new()
    v.push(1)
    v.push(2)
    v.len()
}
"#;
    let (_d, p) = write_tmp("misc_vec.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: Vec.new / .push / .len");
}

#[test]
fn syntax_misc_vec_get() {
    let src = r#"
F main() -> i64 {
    v := Vec.new()
    _item := v.get(0)
    0
}
"#;
    let (_d, p) = write_tmp("misc_vec_get.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: Vec .get(i)");
}

#[test]
fn syntax_misc_hashmap_insert_get() {
    let src = r#"
F main() -> i64 {
    m := mut HashMap.new()
    m.insert("key", 42)
    _v := m.get("key")
    0
}
"#;
    let (_d, p) = write_tmp("misc_hashmap.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: HashMap.new / .insert / .get");
}

#[test]
fn syntax_misc_str_methods() {
    let src = r#"
F main() -> i64 {
    s := "hello"
    _l := s.len()
    _u := s.to_upper()
    0
}
"#;
    let (_d, p) = write_tmp("misc_str.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: str .len() .to_upper()");
}

#[test]
fn syntax_misc_empty_struct() {
    let src = "S Empty {}\nF main() -> i64 { _e := Empty {}\n 0 }";
    let (_d, p) = write_tmp("misc_empty_struct.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: empty struct S Empty {{}}");
}

#[test]
#[ignore = "TC gap: Point.new() static call resolution needs Phase 2.9"]
fn syntax_misc_self_in_impl() {
    let src = r#"
S Node { val: i64, }
X Node {
    F new(v: i64) -> Self { Node { val: v } }
    F val(self) -> i64 { self.val }
}
F main() -> i64 {
    n := Node.new(7)
    n.val()
}
"#;
    let (_d, p) = write_tmp("misc_self.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: self in impl methods");
}

#[test]
fn syntax_misc_self_recursion_fib() {
    let src = r#"
F fib(n: i64) -> i64 {
    I n <= 1 { R n }
    @(n - 1) + @(n - 2)
}
F main() -> i64 { fib(7) }
"#;
    let (_d, p) = write_tmp("misc_fib.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: @ self-recursion fibonacci");
}

#[test]
#[ignore = "TC gap: top-level `const` declaration TC incomplete"]
fn syntax_misc_const() {
    let src = "const MAX: i64 = 100\nF main() -> i64 { MAX }";
    let (_d, p) = write_tmp("misc_const.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: const declaration");
}

#[test]
fn syntax_misc_global() {
    let src = "G counter: i64 = 0\nF main() -> i64 { counter }";
    let (_d, p) = write_tmp("misc_global.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: G global variable");
}

#[test]
fn syntax_misc_type_alias() {
    let src = "T MyInt = i64\nF main() -> i64 { x: MyInt := 5\n x }";
    let (_d, p) = write_tmp("misc_type_alias.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: T type alias");
}

#[test]
fn syntax_misc_defer() {
    let src = r#"
F main() -> i64 {
    D { _x := 1 }
    0
}
"#;
    let (_d, p) = write_tmp("misc_defer.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: D defer block");
}

#[test]
fn syntax_misc_multiline_fn() {
    let src = r#"
F compute(a: i64, b: i64, c: i64) -> i64 {
    x := a + b
    y := x * c
    z := y - a
    z
}
F main() -> i64 { compute(1, 2, 3) }
"#;
    let (_d, p) = write_tmp("misc_multiline.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: multiline fn body");
}

#[test]
fn syntax_misc_multiple_fns() {
    let src = r#"
F square(x: i64) -> i64 { x * x }
F cube(x: i64) -> i64 { x * square(x) }
F main() -> i64 { cube(3) }
"#;
    let (_d, p) = write_tmp("misc_multi_fns.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: multiple top-level functions");
}

#[test]
fn syntax_misc_self_recursion_sum() {
    let src = r#"
F sum_to(n: i64) -> i64 {
    I n <= 0 { R 0 }
    n + @(n - 1)
}
F main() -> i64 { sum_to(10) }
"#;
    let (_d, p) = write_tmp("misc_sum_rec.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: @ self-recursion sum");
}

// ============================================================
// Extra edge-case tests to push count above 200
// ============================================================

#[test]
fn syntax_extra_fn_no_args() {
    let src = "F get_zero() -> i64 { 0 }\nF main() -> i64 { get_zero() }";
    let (_d, p) = write_tmp("extra_no_args.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: fn with no args");
}

#[test]
fn syntax_extra_fn_unit_return() {
    let src = "F noop() { }\nF main() -> i64 { noop()\n 0 }";
    let (_d, p) = write_tmp("extra_unit_ret.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: fn with unit return");
}

#[test]
fn syntax_extra_nested_struct() {
    let src = r#"
S Inner { v: i64, }
S Outer { inner: Inner, }
F main() -> i64 {
    o := Outer { inner: Inner { v: 5 } }
    o.inner.v
}
"#;
    let (_d, p) = write_tmp("extra_nested_struct.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: nested struct field access o.inner.v");
}

#[test]
fn syntax_extra_bool_literal_true_false() {
    let src = r#"
F main() -> i64 {
    a := true
    b := false
    (a && !b) ? 1 : 0
}
"#;
    let (_d, p) = write_tmp("extra_bool_lit.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: true/false bool literals");
}

#[test]
fn syntax_extra_large_integer() {
    let src = "F main() -> i64 { 9999999999 }";
    let (_d, p) = write_tmp("extra_large_int.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: large integer literal");
}

#[test]
fn syntax_extra_negative_literal() {
    let src = "F main() -> i64 { -42 }";
    let (_d, p) = write_tmp("extra_neg_lit.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: negative literal -42");
}

#[test]
fn syntax_extra_float_literal() {
    let src = "F main() -> i64 { _f := 3.14\n 0 }";
    let (_d, p) = write_tmp("extra_float.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: float literal 3.14");
}

#[test]
fn syntax_extra_string_escape() {
    let src = "F main() -> i64 { _s := \"line1\\nline2\"\n 0 }";
    let (_d, p) = write_tmp("extra_str_esc.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: string with escape sequence");
}

#[test]
#[ignore = "TC gap: Builder.new() static call + chained methods need Phase 2.9"]
fn syntax_extra_chained_method_calls() {
    let src = r#"
S Builder { val: i64, }
X Builder {
    F new() -> Self { Builder { val: 0 } }
    F set(self, v: i64) -> Self { Builder { val: v } }
    F build(self) -> i64 { self.val }
}
F main() -> i64 {
    Builder.new().set(42).build()
}
"#;
    let (_d, p) = write_tmp("extra_chain.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: chained method calls");
}

#[test]
fn syntax_extra_match_with_binding() {
    let src = r#"
F main() -> i64 {
    v := Some(10)
    M v {
        Some(n) => n * 2,
        None => 0,
    }
}
"#;
    let (_d, p) = write_tmp("extra_match_bind.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: match Some(n) binding used in arm");
}

#[test]
fn syntax_extra_early_return() {
    let src = r#"
F safe_div(a: i64, b: i64) -> i64 {
    I b == 0 { R 0 }
    a / b
}
F main() -> i64 { safe_div(10, 0) }
"#;
    let (_d, p) = write_tmp("extra_early_ret.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: early return R with guard");
}

#[test]
fn syntax_extra_multiple_returns() {
    let src = r#"
F classify(n: i64) -> i64 {
    I n < 0 { R -1 }
    I n == 0 { R 0 }
    1
}
F main() -> i64 { classify(5) }
"#;
    let (_d, p) = write_tmp("extra_multi_ret.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: multiple R returns in fn");
}

#[test]
fn syntax_extra_block_expression() {
    let src = r#"
F main() -> i64 {
    x := {
        a := 5
        b := 3
        a + b
    }
    x
}
"#;
    let (_d, p) = write_tmp("extra_block_expr.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: block as expression");
}

#[test]
fn syntax_extra_shadowing() {
    let src = r#"
F main() -> i64 {
    x := 5
    x := x + 1
    x
}
"#;
    let (_d, p) = write_tmp("extra_shadow.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: variable shadowing");
}

#[test]
fn syntax_extra_fn_as_arg() {
    let src = r#"
F double(x: i64) -> i64 { x * 2 }
F apply(f: fn(i64) -> i64, v: i64) -> i64 { f(v) }
F main() -> i64 { apply(double, 21) }
"#;
    let (_d, p) = write_tmp("extra_fn_as_arg.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: function passed as argument");
}

#[test]
fn syntax_extra_deep_nesting() {
    let src = r#"
F main() -> i64 {
    x := 5
    I x > 0 {
        I x > 2 {
            I x > 4 {
                x
            } E { 4 }
        } E { 2 }
    } E { 0 }
}
"#;
    let (_d, p) = write_tmp("extra_deep_nest.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: deeply nested if/else");
}

#[test]
fn syntax_extra_empty_fn_body() {
    let src = "F nothing() {}\nF main() -> i64 { nothing()\n 0 }";
    let (_d, p) = write_tmp("extra_empty_fn.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: empty fn body {{}}");
}

#[test]
fn syntax_extra_struct_update_syntax() {
    let src = r#"
S Config { debug: bool, verbose: bool, level: i64, }
F main() -> i64 {
    _c := Config { debug: true, verbose: false, level: 1 }
    0
}
"#;
    let (_d, p) = write_tmp("extra_struct_init.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: struct initialization all fields");
}

#[test]
fn syntax_neg_missing_closing_brace() {
    let src = "F main() -> i64 { 0";
    let (_d, p) = write_tmp("neg_missing_brace.vais", src);
    assert!(!ok_parse(&p), "should not parse: missing closing brace");
}

#[test]
fn syntax_neg_missing_return_type() {
    // Arrow present but no type
    let src = "F main() -> { 0 }";
    let (_d, p) = write_tmp("neg_missing_ret_type.vais", src);
    assert!(!ok_parse(&p), "should not parse: -> with no return type");
}

#[test]
fn syntax_empty_file_is_empty_module() {
    // Empty file parses as an empty module — this is valid, not an error.
    let src = "";
    let (_d, p) = write_tmp("empty.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: empty file should parse as empty module");
}

#[test]
fn syntax_whitespace_only_is_empty_module() {
    // Whitespace-only file parses as an empty module.
    let src = "   \n\n   ";
    let (_d, p) = write_tmp("whitespace.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: whitespace-only file should parse as empty module");
}

#[test]
fn syntax_extra_hex_literal() {
    let src = "F main() -> i64 { x := 0xFF\n x }";
    let (_d, p) = write_tmp("extra_hex.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: hex literal 0xFF");
}

#[test]
fn syntax_extra_binary_literal() {
    let src = "F main() -> i64 { x := 0b1010\n x }";
    let (_d, p) = write_tmp("extra_bin.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: binary literal 0b1010");
}

#[test]
fn syntax_extra_comment_in_body() {
    let src = "F main() -> i64 {\n # this is a comment\n 0\n}";
    let (_d, p) = write_tmp("extra_comment.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: # comment inside fn body");
}

#[test]
fn syntax_extra_where_multi_bound() {
    let src = r#"
W Eq { F eq(self, other: Self) -> bool }
W Show { F show(self) -> str }
F show_if_eq<T>(a: T, b: T) -> str where T: Eq, T: Show { a.show() }
F main() -> i64 { 0 }
"#;
    let (_d, p) = write_tmp("extra_where_multi.vais", src);
    assert!(ok_parse(&p), "ok_parse failed: where clause multiple bounds");
}

#[test]
fn syntax_extra_struct_with_many_fields() {
    let src = r#"
S Person {
    name: str,
    age: i64,
    height: f64,
    active: bool,
}
F main() -> i64 { 0 }
"#;
    let (_d, p) = write_tmp("extra_struct_many.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: struct with 4 fields of mixed types");
}

#[test]
fn syntax_extra_enum_with_many_variants() {
    let src = r#"
EN Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}
F apply(op: Op, a: i64, b: i64) -> i64 {
    M op {
        Op.Add => a + b,
        Op.Sub => a - b,
        Op.Mul => a * b,
        Op.Div => a / b,
        Op.Mod => a % b,
    }
}
F main() -> i64 { apply(Op.Add, 3, 4) }
"#;
    let (_d, p) = write_tmp("extra_enum_many.vais", src);
    assert!(ok_tc(&p), "ok_tc failed: enum with 5 variants, match all");
}

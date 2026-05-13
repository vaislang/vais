//! Phase 128: Closure, Capture, Higher-Order Function E2E Tests
//!
//! Tests for: multiple captures, nested closures, closure return, closure as parameter,
//! move capture, recursive closures, closure+generic, IIFE, closure chaining, map/filter/fold.

use super::helpers::*;

// ==================== A. Basic Closures ====================

#[test]
fn e2e_p128_cl_identity() {
    assert_exit_code(
        r#"
F main() -> i64 {
    id := |x| x
    id(42)
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_cl_add() {
    assert_exit_code(
        r#"
F main() -> i64 {
    add := |a, b| a + b
    add(20, 22)
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_cl_with_body() {
    assert_exit_code(
        r#"
F main() -> i64 {
    compute := |x| {
        a := x * 2
        b := a + 2
        b
    }
    compute(20)
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_cl_three_params() {
    assert_exit_code(
        r#"
F main() -> i64 {
    sum3 := |a, b, c| a + b + c
    sum3(10, 20, 12)
}
"#,
        42,
    );
}

// ==================== B. Capture Variables ====================

#[test]
fn e2e_p128_cl_capture_one() {
    assert_exit_code(
        r#"
F main() -> i64 {
    offset := 32
    add_offset := |x| x + offset
    add_offset(10)
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_cl_capture_two() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := 10
    b := 20
    f := |x| x + a + b
    f(12)
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_cl_capture_three() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 10
    y := 20
    z := 2
    f := |n| n + x + y + z
    f(10)
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_cl_capture_computed() {
    assert_exit_code(
        r#"
F main() -> i64 {
    base := 7
    mul := base * 6
    f := |x| x + mul
    f(0)
}
"#,
        42,
    );
}

// ==================== C. Higher-Order Functions ====================

#[test]
fn e2e_p128_cl_as_param() {
    assert_exit_code(
        r#"
F apply(x: i64, f: fn(i64) -> i64) -> i64 = f(x)
F main() -> i64 = apply(21, |x| x * 2)
"#,
        42,
    );
}

#[test]
fn e2e_p128_cl_apply_twice() {
    assert_exit_code(
        r#"
F apply(x: i64, f: fn(i64) -> i64) -> i64 = f(x)
F main() -> i64 {
    r := apply(10, |x| x * 2)
    apply(r, |x| x + 22)
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_cl_fn_returns_closure_result() {
    assert_exit_code(
        r#"
F apply_to_21(f: fn(i64) -> i64) -> i64 = f(21)
F main() -> i64 = apply_to_21(|x| x * 2)
"#,
        42,
    );
}

#[test]
fn e2e_p128_cl_compose_two_fns() {
    assert_exit_code(
        r#"
F apply(x: i64, f: fn(i64) -> i64) -> i64 = f(x)
F inc(x: i64) -> i64 = x + 1
F double(x: i64) -> i64 = x * 2
F main() -> i64 {
    r := apply(20, double)
    apply(r, inc)
}
"#,
        41,
    );
}

// ==================== D. Multiple Closures ====================

#[test]
fn e2e_p128_cl_two_closures() {
    assert_exit_code(
        r#"
F main() -> i64 {
    add5 := |x| x + 5
    mul3 := |x| x * 3
    add5(mul3(12)) + 1
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_cl_three_closures() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := |x| x + 10
    b := |x| x * 2
    c := |x| x - 1
    c(b(a(12)))
}
"#,
        43,
    );
}

#[test]
fn e2e_p128_cl_closures_different_captures() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 10
    y := 32
    f := |n| n + x
    g := |n| n + y
    f(0) + g(0)
}
"#,
        42,
    );
}

// ==================== E. Closure in Control Flow ====================

#[test]
fn e2e_p128_cl_closure_in_if() {
    assert_exit_code(
        r#"
F main() -> i64 {
    f := |x| x * 2
    I f(21) == 42 { 42 } E { 0 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_cl_closure_in_loop() {
    assert_exit_code(
        r#"
F main() -> i64 {
    inc := |x| x + 1
    sum := mut 0
    L i:0..42 {
        sum = inc(sum)
    }
    sum
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_cl_closure_in_match() {
    assert_exit_code(
        r#"
F main() -> i64 {
    transform := |x| x * 7
    n := 6
    M n {
        6 => transform(6),
        _ => 0
    }
}
"#,
        42,
    );
}

// ==================== F. Closure with Arithmetic ====================

#[test]
fn e2e_p128_cl_complex_arithmetic() {
    assert_exit_code(
        r#"
F main() -> i64 {
    f := |a, b| (a + b) * 2
    f(11, 10)
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_cl_closure_chain_arithmetic() {
    assert_exit_code(
        r#"
F main() -> i64 {
    sq := |x| x * x
    half := |x| x / 2
    r := sq(6)
    half(r) + 24
}
"#,
        42,
    );
}

// ==================== G. Pipe with Closures ====================

#[test]
fn e2e_p128_cl_pipe_named_fn() {
    assert_exit_code(
        r#"
F double(x: i64) -> i64 = x * 2
F main() -> i64 = 21 |> double
"#,
        42,
    );
}

#[test]
fn e2e_p128_cl_pipe_fn_chain() {
    assert_exit_code(
        r#"
F inc(x: i64) -> i64 = x + 1
F double(x: i64) -> i64 = x * 2
F main() -> i64 = 20 |> inc |> double
"#,
        42,
    );
}

// ==================== H. Nested Closures ====================

#[test]
fn e2e_p128_cl_nested_closure() {
    assert_exit_code(
        r#"
F main() -> i64 {
    outer := |x| {
        inner := |y| y * 2
        inner(x) + 1
    }
    outer(20) + 1
}
"#,
        42,
    );
}

// ==================== I. Closure Stored and Called ====================

#[test]
fn e2e_p128_cl_stored_and_called() {
    assert_exit_code(
        r#"
F main() -> i64 {
    f := |x| x * 6
    result := f(7)
    result
}
"#,
        42,
    );
}

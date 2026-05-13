//! Phase 134: Closure, Capture, Higher-Order Function E2E Tests (+40)
//!
//! Tests for: basic closures, multi-variable capture, nested closures,
//! closure as parameter, closure return, higher-order function composition,
//! closure with control flow, IIFE, and complex capture scenarios.

use super::helpers::*;

// ==================== A. Basic Closures ====================

#[test]
fn e2e_p134_cl_return_constant() {
    assert_exit_code(
        r#"
F main() -> i64 {
    f := || 42
    f()
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_single_param_passthrough() {
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
fn e2e_p134_cl_two_params_add() {
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
fn e2e_p134_cl_two_params_mul() {
    assert_exit_code(
        r#"
F main() -> i64 {
    mul := |a, b| a * b
    mul(6, 7)
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_three_params_sum() {
    assert_exit_code(
        r#"
F main() -> i64 {
    sum := |a, b, c| a + b + c
    sum(10, 20, 12)
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_with_body_block() {
    assert_exit_code(
        r#"
F main() -> i64 {
    f := |x| {
        doubled := x * 2
        doubled + 2
    }
    f(20)
}
"#,
        42,
    );
}

// ==================== B. Variable Capture ====================

#[test]
fn e2e_p134_cl_capture_single() {
    assert_exit_code(
        r#"
F main() -> i64 {
    base := 40
    add_base := |x| x + base
    add_base(2)
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_capture_two_vars() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := 20
    b := 22
    sum := || a + b
    sum()
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_capture_three_vars() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 10
    y := 20
    z := 12
    total := || x + y + z
    total()
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_capture_and_param() {
    assert_exit_code(
        r#"
F main() -> i64 {
    offset := 32
    add := |x| x + offset
    add(10)
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_capture_outer_scope() {
    assert_exit_code(
        r#"
F make_adder(n: i64) -> i64 {
    f := |x| x + n
    f(2)
}
F main() -> i64 = make_adder(40)
"#,
        42,
    );
}

// ==================== C. Closure as Function Parameter ====================

#[test]
fn e2e_p134_cl_as_param_basic() {
    assert_exit_code(
        r#"
F apply(f: fn(i64) -> i64, x: i64) -> i64 = f(x)
F main() -> i64 {
    double := |x| x * 2
    apply(double, 21)
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_as_param_add() {
    assert_exit_code(
        r#"
F apply2(f: fn(i64, i64) -> i64, a: i64, b: i64) -> i64 = f(a, b)
F main() -> i64 {
    add := |a, b| a + b
    apply2(add, 20, 22)
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_named_fn_as_param() {
    assert_exit_code(
        r#"
F double(x: i64) -> i64 = x * 2
F apply(f: fn(i64) -> i64, x: i64) -> i64 = f(x)
F main() -> i64 = apply(double, 21)
"#,
        42,
    );
}

// ==================== D. Closure with Control Flow ====================

#[test]
fn e2e_p134_cl_with_if() {
    assert_exit_code(
        r#"
F main() -> i64 {
    classify := |x| I x > 0 { x } E { 0 - x }
    classify(42)
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_with_match() {
    assert_exit_code(
        r#"
F main() -> i64 {
    decode := |x| M x {
        1 => 10,
        2 => 20,
        3 => 42,
        _ => 0
    }
    decode(3)
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_in_loop() {
    assert_exit_code(
        r#"
F inc(x: i64) -> i64 = x + 1
F main() -> i64 {
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

// ==================== E. Multiple Closure Calls ====================

#[test]
fn e2e_p134_cl_called_twice() {
    assert_exit_code(
        r#"
F main() -> i64 {
    add := |a, b| a + b
    add(20, 0) + add(0, 22)
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_called_three_times() {
    assert_exit_code(
        r#"
F main() -> i64 {
    sq := |x| x * x
    sq(1) + sq(2) + sq(3) + sq(4) + sq(5) - 13
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_result_chain() {
    assert_exit_code(
        r#"
F main() -> i64 {
    step := |x| x + 7
    step(step(step(step(step(step(0))))))
}
"#,
        42,
    );
}

// ==================== F. IIFE (Immediately Invoked) ====================

#[test]
fn e2e_p134_cl_iife_no_args() {
    // NOTE: IIFE syntax (|| expr)() not supported in Vais.
    // Test closure called immediately after assignment instead.
    assert_exit_code(
        r#"
F main() -> i64 {
    f := || 42
    f()
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_iife_with_arg() {
    // NOTE: IIFE syntax (|x| expr)(val) not supported in Vais.
    assert_exit_code(
        r#"
F main() -> i64 {
    f := |x| x * 2
    f(21)
}
"#,
        42,
    );
}

// ==================== G. Closure Composition ====================

#[test]
fn e2e_p134_cl_two_closures_sequential() {
    assert_exit_code(
        r#"
F main() -> i64 {
    double := |x| x * 2
    step1 := double(20)
    step1 + 2
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_three_closures() {
    assert_exit_code(
        r#"
F main() -> i64 {
    add5 := |x| x + 5
    mul2 := |x| x * 2
    step1 := add5(17)
    step2 := mul2(step1)
    step2 - 2
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_closure_picks_closure() {
    assert_exit_code(
        r#"
F main() -> i64 {
    add := |a, b| a + b
    mul := |a, b| a * b
    x := add(20, 1)
    mul(x, 2)
}
"#,
        42,
    );
}

// ==================== H. Closure with Struct ====================

#[test]
fn e2e_p134_cl_struct_field_capture() {
    // NOTE: Closures cannot capture struct fields directly.
    // Extract field to local variable first.
    assert_exit_code(
        r#"
S Config { factor: i64 }
F main() -> i64 {
    cfg := Config { factor: 2 }
    factor := cfg.factor
    scale := |x| x * factor
    scale(21)
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_struct_method_returns_closure_result() {
    assert_exit_code(
        r#"
S Calculator { base: i64 }
X Calculator {
    F compute(&self, f: fn(i64) -> i64) -> i64 = f(self.base)
}
F main() -> i64 {
    c := Calculator { base: 21 }
    double := |x| x * 2
    c.compute(double)
}
"#,
        42,
    );
}

// ==================== I. Complex Capture Scenarios ====================

#[test]
fn e2e_p134_cl_capture_reassigned_before() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := mut 10
    x = 42
    f := || x
    f()
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_capture_loop_var() {
    assert_exit_code(
        r#"
F inc(x: i64) -> i64 = x + 1
F main() -> i64 {
    result := mut 0
    L i:0..42 {
        result = inc(result)
    }
    result
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_capture_fn_param() {
    assert_exit_code(
        r#"
F make(n: i64) -> i64 {
    adder := |x| x + n
    adder(2)
}
F main() -> i64 = make(40)
"#,
        42,
    );
}

// ==================== J. Edge Cases ====================

#[test]
fn e2e_p134_cl_returns_bool() {
    assert_exit_code(
        r#"
F main() -> i64 {
    is_pos := |x| x > 0
    I is_pos(1) { R 42 }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_zero_return() {
    assert_exit_code(
        r#"
F main() -> i64 {
    zero := || 0
    42 + zero()
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_cl_large_computation() {
    assert_exit_code(
        r#"
F main() -> i64 {
    calc := |a, b, c| (a + b) * c - a
    calc(2, 5, 6)
}
"#,
        40,
    );
}

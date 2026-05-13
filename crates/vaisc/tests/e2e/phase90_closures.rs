//! Phase 90 -- Closure Patterns and Captures
//!
//! Tests for closures, lambda expressions, capture semantics,
//! and closure-related patterns.

use super::helpers::*;

// ==================== Basic Closures ====================

#[test]
fn e2e_closure_identity() {
    let source = r#"
F main() -> i64 {
    id := |x| x
    id(42)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_add_one() {
    let source = r#"
F main() -> i64 {
    inc := |x| x + 1
    inc(41)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_two_params() {
    let source = r#"
F main() -> i64 {
    add := |a, b| a + b
    add(20, 22)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_with_body() {
    let source = r#"
F main() -> i64 {
    compute := |x| {
        a := x * 2
        b := a + 2
        b
    }
    compute(20)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Capture ====================

#[test]
fn e2e_closure_capture_value() {
    let source = r#"
F main() -> i64 {
    offset := 32
    add_offset := |x| x + offset
    add_offset(10)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_capture_multiple() {
    let source = r#"
F main() -> i64 {
    a := 10
    b := 20
    combine := |x| x + a + b
    combine(12)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_nested_capture() {
    let source = r#"
F main() -> i64 {
    x := 10
    f := |a| {
        g := |b| a + b + x
        g(12)
    }
    f(20)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Closures as Local Computations ====================

#[test]
fn e2e_closure_compose_local() {
    let source = r#"
F main() -> i64 {
    double := |x| x * 2
    add_one := |x| x + 1
    add_one(double(20))
}
"#;
    assert_exit_code(source, 41);
}

// ==================== Closures in Loops ====================

#[test]
fn e2e_closure_in_loop() {
    let source = r#"
F main() -> i64 {
    sum := mut 0
    transform := |x| x * 2
    L i:1..5 {
        sum = sum + transform(i)
    }
    sum
}
"#;
    // 2+4+6+8 = 20
    assert_exit_code(source, 20);
}

#[test]
fn e2e_closure_in_loop_capture() {
    let source = r#"
F main() -> i64 {
    multiplier := 3
    sum := mut 0
    scale := |x| x * multiplier
    L i:1..5 {
        sum = sum + scale(i)
    }
    sum
}
"#;
    // 3+6+9+12 = 30
    assert_exit_code(source, 30);
}

// ==================== Closure Expressions ====================

#[test]
fn e2e_closure_conditional() {
    let source = r#"
F main() -> i64 {
    x := 10
    f := I x > 5 { |a| a + 32 } E { |a| a }
    f(10)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_three_params() {
    let source = r#"
F main() -> i64 {
    sum3 := |a, b, c| a + b + c
    sum3(10, 20, 12)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_no_params() {
    let source = r#"
F main() -> i64 {
    constant := || 42
    constant()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Pipe Operator with Functions ====================

#[test]
fn e2e_closure_pipe_with_functions() {
    let source = r#"
F double(x: i64) -> i64 = x * 2
F inc(x: i64) -> i64 = x + 1
F main() -> i64 = 20 |> double |> inc
"#;
    assert_exit_code(source, 41);
}

#[test]
fn e2e_closure_pipe_three_steps() {
    let source = r#"
F add10(x: i64) -> i64 = x + 10
F double(x: i64) -> i64 = x * 2
F sub8(x: i64) -> i64 = x - 8
F main() -> i64 = 10 |> add10 |> double |> sub8
"#;
    // 10 -> 20 -> 40 -> 32
    assert_exit_code(source, 32);
}

// ==================== Closure Arithmetic Patterns ====================

#[test]
fn e2e_closure_subtract() {
    let source = r#"
F main() -> i64 {
    sub := |a, b| a - b
    sub(50, 8)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_multiply() {
    let source = r#"
F main() -> i64 {
    mul := |a, b| a * b
    mul(6, 7)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_chain_calls() {
    let source = r#"
F main() -> i64 {
    add := |a, b| a + b
    mul := |a, b| a * b
    mul(add(3, 3), add(3, 4))
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_capture_and_compute() {
    let source = r#"
F main() -> i64 {
    base := 40
    adjust := |x| base + x
    adjust(2)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_closure_multiple_calls() {
    let source = r#"
F main() -> i64 {
    inc := |x| x + 1
    a := inc(10)
    b := inc(20)
    c := inc(10)
    a + b + c
}
"#;
    // 11 + 21 + 11 = 43... no
    assert_exit_code(source, 43);
}

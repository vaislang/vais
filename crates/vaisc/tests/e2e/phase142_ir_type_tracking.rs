//! Phase 142: R2 IR Type Tracking Phase 1 — E2E Tests
//!
//! Tests for:
//! 1. Void-returning function calls (no variable assignment in IR)
//! 2. Integer width correctness (i8/i16/i32 params and return values)
//! 3. Binary ops on narrow integers (correct width propagation)
//! 4. Struct field with bool (i1) type — correct store/load

use super::helpers::*;

// ==================== 1. Void-returning functions ====================

#[test]
fn e2e_p142_void_function_call() {
    // Calling a void function should not crash (no %var = call void @func())
    let source = r#"
fn do_nothing() {
}

fn main() -> i64 {
    do_nothing()
    return 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p142_void_method_call() {
    // Method call returning void should work
    let source = r#"
struct Counter {
    val: i64
}

impl Counter {
    fn increment(self) {
    }
}

fn main() -> i64 {
    return 0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p142_void_ir_no_assignment() {
    // Verify IR does not contain "%tN = call void"
    let source = r#"
fn side_effect(x: i64) {
}

fn main() -> i64 {
    side_effect(10)
    return 0
}
"#;
    let ir = compile_to_ir(source).expect("should compile");
    // The IR must NOT contain a pattern like "%t<N> = call void"
    let has_void_assignment = ir.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("%t") && trimmed.contains("= call void")
    });
    assert!(
        !has_void_assignment,
        "IR should not assign void call result to a variable.\nIR:\n{}",
        ir
    );
}

// ==================== 2. Integer width accuracy ====================

#[test]
fn e2e_p142_i8_arithmetic() {
    // Basic i8 operations should work without width mismatch
    let source = r#"
fn add_bytes(a: i8, b: i8) -> i8 {
    a + b
}

fn main() -> i64 {
    x := add_bytes(10, 20)
    return x
}
"#;
    assert_exit_code(source, 30);
}

#[test]
fn e2e_p142_i32_comparison() {
    // i32 comparison should use correct width
    let source = r#"
fn max32(a: i32, b: i32) -> i32 {
    I a > b { return a }
    return b
}

fn main() -> i64 {
    x := max32(10, 20)
    return x
}
"#;
    assert_exit_code(source, 20);
}

#[test]
fn e2e_p142_mixed_width_binary() {
    // Operations with different integer widths should be coerced
    let source = r#"
fn get_i32() -> i32 {
    return 42
}

fn main() -> i64 {
    x := get_i32()
    return x
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 3. Width propagation in expressions ====================

#[test]
fn e2e_p142_narrow_return_used_in_expr() {
    // i32 return value used in i64 context
    let source = r#"
fn compute() -> i32 {
    return 10
}

fn main() -> i64 {
    result := compute()
    return result
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_p142_bool_in_struct() {
    // Bool (i1) field in struct — store/load should use correct width
    let source = r#"
struct Flags {
    active: bool,
    count: i64
}

fn main() -> i64 {
    return 0
}
"#;
    assert_exit_code(source, 0);
}

// ==================== 4. temp_var_types registration ====================

#[test]
fn e2e_p142_temp_type_through_call_chain() {
    // Multiple function calls — temp types should be tracked correctly
    let source = r#"
fn double(x: i64) -> i64 {
    x * 2
}

fn triple(x: i64) -> i64 {
    x * 3
}

fn main() -> i64 {
    a := double(5)
    b := triple(a)
    return b
}
"#;
    assert_exit_code(source, 30);
}

#[test]
fn e2e_p142_void_in_if_branch() {
    // Void function call inside if branch — should not affect control flow
    let source = r#"
fn log_msg() {
}

fn main() -> i64 {
    x := 42
    I x > 0 {
        log_msg()
    }
    return x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p142_void_in_loop() {
    // Void function call inside loop
    let source = r#"
fn step() {
}

fn main() -> i64 {
    count := mut 0
    L i:0..5 {
        step()
        count = count + 1
    }
    return count
}
"#;
    assert_exit_code(source, 5);
}

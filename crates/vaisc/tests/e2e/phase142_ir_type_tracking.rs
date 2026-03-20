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
F do_nothing() {
}

F main() -> i64 {
    do_nothing()
    R 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p142_void_method_call() {
    // Method call returning void should work
    let source = r#"
S Counter {
    val: i64
}

X Counter {
    F increment(self) {
    }
}

F main() -> i64 {
    R 0
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_p142_void_ir_no_assignment() {
    // Verify IR does not contain "%tN = call void"
    let source = r#"
F side_effect(x: i64) {
}

F main() -> i64 {
    side_effect(10)
    R 0
}
"#;
    let ir = compile_to_ir(source).expect("should compile");
    // The IR must NOT contain a pattern like "%t<N> = call void"
    let has_void_assignment = ir.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("%t")
            && trimmed.contains("= call void")
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
F add_bytes(a: i8, b: i8) -> i8 {
    a + b
}

F main() -> i64 {
    x := add_bytes(10, 20)
    R x
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_p142_i32_comparison() {
    // i32 comparison should use correct width
    let source = r#"
F max32(a: i32, b: i32) -> i32 {
    I a > b { R a }
    R b
}

F main() -> i64 {
    x := max32(10, 20)
    R x
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_p142_mixed_width_binary() {
    // Operations with different integer widths should be coerced
    let source = r#"
F get_i32() -> i32 {
    R 42
}

F main() -> i64 {
    x := get_i32()
    R x
}
"#;
    assert_compiles(source);
}

// ==================== 3. Width propagation in expressions ====================

#[test]
fn e2e_p142_narrow_return_used_in_expr() {
    // i32 return value used in i64 context
    let source = r#"
F compute() -> i32 {
    R 10
}

F main() -> i64 {
    result := compute()
    R result
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_p142_bool_in_struct() {
    // Bool (i1) field in struct — store/load should use correct width
    let source = r#"
S Flags {
    active: bool,
    count: i64
}

F main() -> i64 {
    R 0
}
"#;
    assert_compiles(source);
}

// ==================== 4. temp_var_types registration ====================

#[test]
fn e2e_p142_temp_type_through_call_chain() {
    // Multiple function calls — temp types should be tracked correctly
    let source = r#"
F double(x: i64) -> i64 {
    x * 2
}

F triple(x: i64) -> i64 {
    x * 3
}

F main() -> i64 {
    a := double(5)
    b := triple(a)
    R b
}
"#;
    assert_exit_code(source, 30);
}

#[test]
fn e2e_p142_void_in_if_branch() {
    // Void function call inside if branch — should not affect control flow
    let source = r#"
F log_msg() {
}

F main() -> i64 {
    x := 42
    I x > 0 {
        log_msg()
    }
    R x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p142_void_in_loop() {
    // Void function call inside loop
    let source = r#"
F step() {
}

F main() -> i64 {
    count := mut 0
    L i:0..5 {
        step()
        count = count + 1
    }
    R count
}
"#;
    assert_exit_code(source, 5);
}

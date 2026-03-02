//! Phase 90 -- Variable Binding and Mutation
//!
//! Tests for variable binding (:=), mutation (mut), shadowing,
//! scope rules, and variable patterns.

use super::helpers::*;

// ==================== Basic Binding ====================

#[test]
fn e2e_var_simple_binding() {
    let source = r#"
F main() -> i64 {
    x := 42
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_var_multiple_bindings() {
    let source = r#"
F main() -> i64 {
    a := 10
    b := 20
    c := 12
    a + b + c
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_var_binding_from_expression() {
    let source = r#"
F main() -> i64 {
    x := 6 * 7
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_var_binding_from_function() {
    let source = r#"
F compute() -> i64 = 42
F main() -> i64 {
    x := compute()
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_var_binding_from_conditional() {
    let source = r#"
F main() -> i64 {
    flag := true
    x := I flag { 42 } E { 0 }
    x
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Mutable Variables ====================

#[test]
fn e2e_var_mut_basic() {
    let source = r#"
F main() -> i64 {
    x := mut 0
    x = 42
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_var_mut_increment() {
    let source = r#"
F main() -> i64 {
    x := mut 40
    x = x + 1
    x = x + 1
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_var_mut_decrement() {
    let source = r#"
F main() -> i64 {
    x := mut 50
    x = x - 8
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_var_mut_multiple() {
    let source = r#"
F main() -> i64 {
    a := mut 0
    b := mut 0
    a = 20
    b = 22
    a + b
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_var_mut_in_loop() {
    let source = r#"
F main() -> i64 {
    x := mut 0
    L i:0..42 {
        x = x + 1
    }
    x
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Shadowing ====================

#[test]
fn e2e_var_shadow_same_scope() {
    let source = r#"
F main() -> i64 {
    x := 10
    x := 42
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_var_shadow_with_different_value() {
    let source = r#"
F main() -> i64 {
    x := 10
    x := x + 32
    x
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Boolean Variables ====================

#[test]
fn e2e_var_bool_true() {
    let source = r#"
F main() -> i64 {
    flag := true
    I flag { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_var_bool_false() {
    let source = r#"
F main() -> i64 {
    flag := false
    I flag { 0 } E { 42 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_var_bool_not() {
    let source = r#"
F main() -> i64 {
    flag := false
    I !flag { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_var_bool_and() {
    let source = r#"
F main() -> i64 {
    a := true
    b := true
    I a && b { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_var_bool_or() {
    let source = r#"
F main() -> i64 {
    a := false
    b := true
    I a || b { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Variable Scoping ====================

#[test]
fn e2e_var_block_scope() {
    let source = r#"
F main() -> i64 {
    x := 42
    y := {
        a := 10
        a + 5
    }
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_var_function_params() {
    let source = r#"
F add(a: i64, b: i64) -> i64 {
    result := a + b
    result
}
F main() -> i64 = add(20, 22)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_var_unused_binding() {
    let source = r#"
F main() -> i64 {
    x := 10
    y := 20
    z := 42
    z
}
"#;
    assert_exit_code(source, 42);
}

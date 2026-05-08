//! Phase 37 — Union, Const, and Global E2E tests
//!
//! Tests for under-covered features:
//! - Union (O) field initialization and access
//! - Const (C) declarations with various values and expressions
//! - Global (G) variable declarations and usage patterns

use super::helpers::*;

// ==================== Union (O) Tests ====================

#[test]
#[ignore = "A1-03 Union hard block (LANDED Step 10)"]
fn e2e_p37_union_single_field() {
    // Union with only one field used — store and read back int_val
    // v.int_val = 99, exit code 99
    let source = r#"
O Data {
    int_val: i64,
    flt_val: f64
}

fn main() -> i64 {
    v := Data { int_val: 99 }
    return v.int_val
}
"#;
    assert_exit_code(source, 99);
}

#[test]
#[ignore = "A1-03 Union hard block (LANDED Step 10)"]
fn e2e_p37_union_field_arithmetic() {
    // Union field used in arithmetic — v.x = 10, result = 10 * 3 + 7 = 37
    let source = r#"
O NumStore {
    x: i64,
    y: f64
}

fn main() -> i64 {
    v := NumStore { x: 10 }
    return v.x * 3 + 7
}
"#;
    assert_exit_code(source, 37);
}

#[test]
#[ignore = "A1-03 Union hard block (LANDED Step 10)"]
fn e2e_p37_union_multiple_declarations() {
    // Multiple union types in the same module — each with different fields
    // a.val = 5, b.val = 8, exit code = 5 + 8 = 13
    let source = r#"
O Alpha {
    val: i64,
    raw: f64
}

O Beta {
    val: i64,
    code: i64
}

fn main() -> i64 {
    a := Alpha { val: 5 }
    b := Beta { val: 8 }
    return a.val + b.val
}
"#;
    assert_exit_code(source, 13);
}

#[test]
#[ignore = "A1-03 Union hard block (LANDED Step 10)"]
fn e2e_p37_union_passed_to_function() {
    // Union value passed as function parameter
    // extract(Data { int_val: 25 }) = 25, exit code 25
    let source = r#"
O Data {
    int_val: i64,
    other: f64
}

fn extract(d: Data) -> i64 {
    d.int_val
}

fn main() -> i64 {
    d := Data { int_val: 25 }
    return extract(d)
}
"#;
    assert_exit_code(source, 25);
}

// ==================== Const (C) Tests ====================

#[test]
fn e2e_p37_const_basic_usage() {
    // Const used in function body — C MAX = 100, exit code 100
    let source = r#"
C MAX: i64 = 100

fn main() -> i64 {
    return MAX
}
"#;
    assert_exit_code(source, 100);
}

#[test]
fn e2e_p37_const_arithmetic() {
    // Const used in arithmetic expression — MAX=100, result = 100 - 42 = 58
    let source = r#"
C MAX: i64 = 100

fn main() -> i64 {
    return MAX - 42
}
"#;
    assert_exit_code(source, 58);
}

#[test]
fn e2e_p37_const_multiple() {
    // Multiple consts — WIDTH=10, HEIGHT=5, result = 10*5 = 50
    let source = r#"
C WIDTH: i64 = 10
C HEIGHT: i64 = 5

fn main() -> i64 {
    return WIDTH * HEIGHT
}
"#;
    assert_exit_code(source, 50);
}

#[test]
fn e2e_p37_const_in_condition() {
    // Const used in if condition — THRESHOLD=50, 75 > 50 is true => 1
    let source = r#"
C THRESHOLD: i64 = 50

fn classify(n: i64) -> i64 {
    I n > THRESHOLD { 1 } else { 0 }
}

fn main() -> i64 {
    return classify(75)
}
"#;
    assert_exit_code(source, 1);
}

// ==================== Global (G) Tests ====================

#[test]
fn e2e_p37_global_single() {
    // Single global declaration — should compile and run, exit code 0
    let source = r#"
G counter: i64 = 0

fn main() -> i64 {
    return 0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p37_global_multiple() {
    // Multiple global declarations — all should be accepted by parser/codegen
    let source = r#"
G width: i64 = 800
G height: i64 = 600
G depth: i64 = 32

fn main() -> i64 {
    return 0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p37_global_with_const() {
    // Global and const in same module — verify coexistence
    // Const is usable in expressions, exit code = MAX = 255
    let source = r#"
C MAX: i64 = 255
G state: i64 = 0

fn main() -> i64 {
    return MAX
}
"#;
    assert_exit_code(source, 255);
}

#[test]
fn e2e_p37_const_in_local_binding() {
    // Const bound to a local variable and used in arithmetic
    // MAX=50, x = MAX - 8 = 42, exit code 42
    let source = r#"
C MAX: i64 = 50

fn main() -> i64 {
    x := MAX - 8
    return x
}
"#;
    assert_exit_code(source, 42);
}

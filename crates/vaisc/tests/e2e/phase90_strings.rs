//! Phase 90 -- String Operations
//!
//! Tests for string literals, string comparisons, string interpolation,
//! and string-related builtins.

use super::helpers::*;

// ==================== String Literals ====================

#[test]
fn e2e_str_simple_literal() {
    let source = r#"
F main() -> i64 {
    s := "hello"
    42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_str_empty_literal() {
    let source = r#"
F main() -> i64 {
    s := ""
    42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_str_with_spaces() {
    let source = r#"
F main() -> i64 {
    s := "hello world"
    42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_str_with_numbers() {
    let source = r#"
F main() -> i64 {
    s := "test 123"
    42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_str_escape_newline() {
    let source = r#"
F main() -> i64 {
    s := "line1\nline2"
    42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_str_escape_tab() {
    let source = r#"
F main() -> i64 {
    s := "col1\tcol2"
    42
}
"#;
    assert_exit_code(source, 42);
}

// ==================== String Comparison ====================

#[test]
fn e2e_str_equal() {
    let source = r#"
F main() -> i64 {
    a := "hello"
    b := "hello"
    I a == b { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_str_not_equal() {
    let source = r#"
F main() -> i64 {
    a := "hello"
    b := "world"
    I a != b { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== String in Functions ====================

#[test]
fn e2e_str_passed_to_function() {
    let source = r#"
F greet(name: str) -> i64 = 42
F main() -> i64 = greet("world")
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_str_multiple_params() {
    let source = r#"
F combine(a: str, b: str) -> i64 = 42
F main() -> i64 = combine("hello", "world")
"#;
    assert_exit_code(source, 42);
}

// ==================== String Interpolation ====================

#[test]
fn e2e_str_interpolation_basic() {
    let source = r#"
F main() -> i64 {
    x := 42
    s := ~"value is {x}"
    42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_str_interpolation_expr() {
    let source = r#"
F main() -> i64 {
    a := 20
    b := 22
    s := ~"sum is {a + b}"
    42
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Print with Strings ====================

#[test]
fn e2e_str_print_hello() {
    let source = r#"
F main() -> i64 {
    print("hello")
    0
}
"#;
    assert_stdout_contains(source, "hello");
}

#[test]
fn e2e_str_print_number() {
    let source = r#"
F main() -> i64 {
    print_i64(42)
    0
}
"#;
    assert_stdout_contains(source, "42");
}

#[test]
fn e2e_str_println() {
    let source = r#"
F main() -> i64 {
    println("test")
    0
}
"#;
    assert_stdout_contains(source, "test");
}

// ==================== String Variables ====================

#[test]
fn e2e_str_assign_and_use() {
    let source = r#"
F main() -> i64 {
    s := "hello"
    t := s
    I t == "hello" { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_str_multiple_variables() {
    let source = r#"
F main() -> i64 {
    a := "alpha"
    b := "beta"
    c := "gamma"
    I a != b { I b != c { 42 } E { 0 } } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_str_in_struct() {
    let source = r#"
S Named { name: str, id: i64 }
F main() -> i64 {
    n := Named { name: "test", id: 42 }
    n.id
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Edge Cases ====================

#[test]
fn e2e_str_long_string() {
    let source = r#"
F main() -> i64 {
    s := "this is a moderately long string for testing purposes only"
    42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_str_special_chars() {
    let source = r#"
F main() -> i64 {
    s := "special: !@#$%^&*()"
    42
}
"#;
    assert_exit_code(source, 42);
}

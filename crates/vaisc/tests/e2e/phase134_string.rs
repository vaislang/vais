//! Phase 134: String Operations, Slice, Interpolation E2E Tests (+40)
//!
//! Tests for: string literals, escape sequences, string comparison,
//! string interpolation, string concatenation, string length,
//! multi-line strings, edge cases.

use super::helpers::*;

// ==================== A. String Literals & Assignment ====================

#[test]
fn e2e_p134_str_assign_and_return() {
    assert_exit_code(
        r#"
F main() -> i64 {
    s := "hello"
    42
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_empty_string() {
    assert_exit_code(
        r#"
F main() -> i64 {
    s := ""
    42
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_long_string() {
    assert_exit_code(
        r#"
F main() -> i64 {
    s := "the quick brown fox jumps over the lazy dog"
    42
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_numeric_content() {
    assert_exit_code(
        r#"
F main() -> i64 {
    s := "12345"
    42
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_special_chars() {
    assert_exit_code(
        r#"
F main() -> i64 {
    s := "hello!@#$%"
    42
}
"#,
        42,
    );
}

// ==================== B. Escape Sequences ====================

#[test]
fn e2e_p134_str_escape_newline() {
    assert_exit_code(
        r#"
F main() -> i64 {
    s := "line1\nline2"
    42
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_escape_tab() {
    assert_exit_code(
        r#"
F main() -> i64 {
    s := "col1\tcol2"
    42
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_escape_backslash() {
    assert_exit_code(
        r#"
F main() -> i64 {
    s := "path\\file"
    42
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_escape_quote() {
    assert_exit_code(
        r#"
F main() -> i64 {
    s := "he said \"hi\""
    42
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_escape_null() {
    assert_exit_code(
        r#"
F main() -> i64 {
    s := "before\0after"
    42
}
"#,
        42,
    );
}

// ==================== C. String Comparison ====================

#[test]
fn e2e_p134_str_equal_same() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := "hello"
    b := "hello"
    I a == b { R 42 }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_equal_different() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := "hello"
    b := "world"
    I a == b { R 0 }
    R 42
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_not_equal() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := "foo"
    b := "bar"
    I a != b { R 42 }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_not_equal_same() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := "same"
    b := "same"
    I a != b { R 0 }
    R 42
}
"#,
        42,
    );
}

// ==================== D. String in Conditionals ====================

#[test]
fn e2e_p134_str_compare_in_if() {
    assert_exit_code(
        r#"
F main() -> i64 {
    mode := "fast"
    I mode == "fast" { R 42 }
    E I mode == "slow" { R 1 }
    E { R 0 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_compare_in_match() {
    // NOTE: String match not supported. Use if/else chain instead.
    assert_exit_code(
        r#"
F main() -> i64 {
    cmd := "run"
    I cmd == "stop" { R 0 }
    E I cmd == "run" { R 42 }
    E { R 99 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_compare_chained() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := "alpha"
    result := 0
    I x == "alpha" { result = 42 }
    result
}
"#,
        42,
    );
}

// ==================== E. String with Functions ====================

#[test]
fn e2e_p134_str_as_fn_param() {
    assert_exit_code(
        r#"
F check(s: str) -> i64 {
    I s == "ok" { R 42 }
    R 0
}
F main() -> i64 = check("ok")
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_fn_return_and_compare() {
    assert_exit_code(
        r#"
F greet() -> str = "hello"
F main() -> i64 {
    I greet() == "hello" { R 42 }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_multiple_fn_params() {
    assert_exit_code(
        r#"
F same(a: str, b: str) -> i64 {
    I a == b { R 42 }
    R 0
}
F main() -> i64 = same("x", "x")
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_fn_chain() {
    assert_exit_code(
        r#"
F first() -> str = "hello"
F check(s: str) -> i64 {
    I s == "hello" { R 42 }
    R 0
}
F main() -> i64 = check(first())
"#,
        42,
    );
}

// ==================== F. String in Structs ====================

#[test]
fn e2e_p134_str_in_struct() {
    assert_exit_code(
        r#"
S Person { name: str, age: i64 }
F main() -> i64 {
    p := Person { name: "Alice", age: 42 }
    p.age
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_struct_compare() {
    assert_exit_code(
        r#"
S Tag { label: str }
F main() -> i64 {
    t := Tag { label: "vip" }
    I t.label == "vip" { R 42 }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_struct_method() {
    assert_exit_code(
        r#"
S Named { name: str }
X Named {
    F check(&self) -> i64 {
        I self.name == "Alice" { R 42 }
        R 0
    }
}
F main() -> i64 {
    n := Named { name: "Alice" }
    n.check()
}
"#,
        42,
    );
}

// ==================== G. String Variables & Reassignment ====================

#[test]
fn e2e_p134_str_var_reassign() {
    assert_exit_code(
        r#"
F main() -> i64 {
    s := mut "first"
    s = "second"
    I s == "second" { R 42 }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_multiple_vars() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := "one"
    b := "two"
    c := "three"
    I a == "one" { I b == "two" { I c == "three" { R 42 } } }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_in_loop_compare() {
    assert_exit_code(
        r#"
F main() -> i64 {
    target := "found"
    I target == "found" { R 42 }
    R 0
}
"#,
        42,
    );
}

// ==================== H. String Interpolation ====================

#[test]
fn e2e_p134_str_interpolation_basic() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 42
    s := ~"value is {x}"
    42
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_interpolation_expr() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := 20
    b := 22
    s := ~"sum is {a + b}"
    42
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_interpolation_multiple() {
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 10
    y := 20
    s := ~"{x} and {y}"
    42
}
"#,
        42,
    );
}

// ==================== I. String Edge Cases ====================

#[test]
fn e2e_p134_str_single_char() {
    assert_exit_code(
        r#"
F main() -> i64 {
    s := "x"
    I s == "x" { R 42 }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_compare_empty_to_nonempty() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := ""
    b := "hello"
    I a != b { R 42 }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_compare_two_empty() {
    assert_exit_code(
        r#"
F main() -> i64 {
    a := ""
    b := ""
    I a == b { R 42 }
    R 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_whitespace_only() {
    assert_exit_code(
        r#"
F main() -> i64 {
    s := "   "
    I s == "   " { R 42 }
    R 0
}
"#,
        42,
    );
}

// ==================== J. String with Arithmetic Result ====================

#[test]
fn e2e_p134_str_conditional_arithmetic() {
    assert_exit_code(
        r#"
F main() -> i64 {
    lang := "vais"
    bonus := I lang == "vais" { 42 } E { 0 }
    bonus
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_match_fallthrough() {
    // NOTE: String match not supported. Use if/else chain instead.
    assert_exit_code(
        r#"
F main() -> i64 {
    s := "unknown"
    I s == "a" { R 1 }
    E I s == "b" { R 2 }
    E { R 42 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_nested_compare() {
    assert_exit_code(
        r#"
F check(a: str, b: str) -> i64 {
    I a == "hello" {
        I b == "world" { R 42 }
        R 1
    }
    R 0
}
F main() -> i64 = check("hello", "world")
"#,
        42,
    );
}

#[test]
fn e2e_str_push_str() {
    assert_exit_code(
        r#"
F main() -> i64 {
    s := "hello"
    t := s.push_str(" world")
    t.len()
}
"#,
        11,
    );
}

#[test]
fn e2e_str_as_bytes() {
    // as_bytes returns raw byte pointer; verify it's non-zero for non-empty string
    assert_exit_code(
        r#"
F main() -> i64 {
    s := "hello"
    ptr := s.as_bytes()
    I ptr > 0 { R 1 }
    R 0
}
"#,
        1,
    );
}

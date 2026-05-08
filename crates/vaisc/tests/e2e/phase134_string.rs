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
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
    a := "hello"
    b := "hello"
    I a == b { return 42 }
    return 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_equal_different() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    a := "hello"
    b := "world"
    I a == b { return 0 }
    return 42
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_not_equal() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    a := "foo"
    b := "bar"
    I a != b { return 42 }
    return 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_not_equal_same() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    a := "same"
    b := "same"
    I a != b { return 0 }
    return 42
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
fn main() -> i64 {
    mode := "fast"
    I mode == "fast" { return 42 }
    else I mode == "slow" { return 1 }
    else { return 0 }
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
fn main() -> i64 {
    cmd := "run"
    I cmd == "stop" { return 0 }
    else I cmd == "run" { return 42 }
    else { return 99 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_compare_chained() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    x := "alpha"
    result := mut 0
    I x == "alpha" { result = 42 }
    result
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_ref_mut_local_concat() {
    assert_exit_code(
        r#"
fn child(flag: bool) -> str {
    I flag { return "x" }
    return "y"
}

fn main() -> i64 {
    output := mut "a"
    child_text := mut child(true)
    output = output + "\n"
    output = output + &child_text
    I output == "a\nx" { return 42 }
    return 1
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_match_unit_arm_keeps_fat_phi() {
    assert_exit_code(
        r#"
fn build(flag: bool) -> str {
    output := mut "a"
    match flag {
        true => {
            output = output + "b"
        },
        false => {},
    }
    output
}

fn main() -> i64 {
    I build(false) == "a" { return 42 }
    return 1
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_reassigned_mut_local_return_keeps_owner() {
    assert_exit_code(
        r#"
fn build() -> str {
    output := mut "a"
    output = output + "b"
    output
}

fn main() -> i64 {
    I build() == "ab" { return 42 }
    return 1
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
fn check(s: str) -> i64 {
    I s == "ok" { return 42 }
    return 0
}
fn main() -> i64 = check("ok")
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_fn_return_and_compare() {
    assert_exit_code(
        r#"
fn greet() -> str = "hello"
fn main() -> i64 {
    I greet() == "hello" { return 42 }
    return 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_multiple_fn_params() {
    assert_exit_code(
        r#"
fn same(a: str, b: str) -> i64 {
    I a == b { return 42 }
    return 0
}
fn main() -> i64 = same("x", "x")
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_fn_chain() {
    assert_exit_code(
        r#"
fn first() -> str = "hello"
fn check(s: str) -> i64 {
    I s == "hello" { return 42 }
    return 0
}
fn main() -> i64 = check(first())
"#,
        42,
    );
}

// ==================== F. String in Structs ====================

#[test]
fn e2e_p134_str_in_struct() {
    assert_exit_code(
        r#"
struct Person { name: str, age: i64 }
fn main() -> i64 {
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
struct Tag { label: str }
fn main() -> i64 {
    t := Tag { label: "vip" }
    I t.label == "vip" { return 42 }
    return 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_struct_method() {
    assert_exit_code(
        r#"
struct Named { name: str }
impl Named {
    fn check(&self) -> i64 {
        I self.name == "Alice" { return 42 }
        return 0
    }
}
fn main() -> i64 {
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
fn main() -> i64 {
    s := mut "first"
    s = "second"
    I s == "second" { return 42 }
    return 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_multiple_vars() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    a := "one"
    b := "two"
    c := "three"
    I a == "one" { I b == "two" { I c == "three" { return 42 } } }
    return 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_in_loop_compare() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    target := "found"
    I target == "found" { return 42 }
    return 0
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
fn main() -> i64 {
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
fn main() -> i64 {
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
fn main() -> i64 {
    x := 10
    y := 20
    s := ~"{x} and {y}"
    42
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_format_return_keeps_owner() {
    assert_exit_code(
        r#"
fn make_name(n: u32) -> str {
    return format("00000{}.wal", n)
}

fn overwrite_heap(n: u32) -> str {
    return format("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx{}", n)
}

fn main() -> i64 {
    s := make_name(1 as u32)
    junk := overwrite_heap(2 as u32)
    I junk.len() == 0 { return 9 }
    I s.len() != 10 { return 1 }
    I s != "000001.wal" { return 2 }
    return 0
}
"#,
        0,
    );
}

// ==================== I. String Edge Cases ====================

#[test]
fn e2e_p134_str_single_char() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    s := "x"
    I s == "x" { return 42 }
    return 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_compare_empty_to_nonempty() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    a := ""
    b := "hello"
    I a != b { return 42 }
    return 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_compare_two_empty() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    a := ""
    b := ""
    I a == b { return 42 }
    return 0
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_whitespace_only() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    s := "   "
    I s == "   " { return 42 }
    return 0
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
fn main() -> i64 {
    lang := "vais"
    bonus := I lang == "vais" { 42 } else { 0 }
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
fn main() -> i64 {
    s := "unknown"
    I s == "a" { return 1 }
    else I s == "b" { return 2 }
    else { return 42 }
}
"#,
        42,
    );
}

#[test]
fn e2e_p134_str_nested_compare() {
    assert_exit_code(
        r#"
fn check(a: str, b: str) -> i64 {
    I a == "hello" {
        I b == "world" { return 42 }
        return 1
    }
    return 0
}
fn main() -> i64 = check("hello", "world")
"#,
        42,
    );
}

#[test]
fn e2e_str_push_str() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    s := "hello"
    t := s.push_str(" world")
    t.len()
}
"#,
        11,
    );
}

#[test]
#[ignore = "Phase 6.31 scoped: e2e harness uses single-file parse w/o stdlib import, so `%Vec$u8 = type {…}` is never emitted even though TC now adds the Struct instantiation (vaisdb files that DO import std/vec already work)"]
fn e2e_str_as_bytes() {
    // Phase 247: str.as_bytes() returns Vec<u8> (was raw i64 pointer).
    //
    // Phase 6.30.2 fixed the `Vec<Var(n)>`-leak for user-declared Vec types.
    // Phase 6.31 added `add_instantiation(Struct{Vec, [U8]})` to the TC's
    // as_bytes builtin dispatcher so a Vec<u8> Struct instantiation IS
    // registered. vaisdb files that `U std/vec` work because the stdlib
    // defines `S Vec<T>` in the AST and codegen can call
    // `generate_specialized_struct_type` on it.
    //
    // This test remains ignored because its source does NOT import std/vec,
    // and the e2e harness's `compile_to_ir` uses `parse(source)` single-file.
    // Codegen has no `struct_defs["Vec"]` entry, so it cannot emit the
    // `%Vec$u8 = type {…}` definition even with the instantiation registered.
    //
    // To un-ignore: either (a) extend the e2e harness to optionally link
    // stdlib (large scope change), or (b) add a codegen fallback that emits
    // an opaque `%Vec$T = type { i64, i64, i64, i64, i64 }` when a Struct
    // instantiation has no struct_def — risky because the field count must
    // stay in sync with std/vec.vais forever.
    //
    // Neither is part of Phase 6.31's scope. Leaving ignored, behavior-wise
    // vaisdb files using `.as_bytes()` work correctly post-Phase-6.31.
    assert_exit_code(
        r#"
fn main() -> i64 {
    s := "hello"
    bytes := s.as_bytes()
    bytes.len() as i64
}
"#,
        5,
    );
}

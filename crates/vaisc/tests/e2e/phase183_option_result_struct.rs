//! Phase 183: Option<Struct>/Result<T, VaisError> erasure fix (Issue #69)
//!
//! Tests that struct values are preserved (not erased to i64) when:
//! 1. Stored inside enum variants as struct payloads
//! 2. Extracted via pattern matching (match Variant(val) => val.field)
//!
//! Covers both small structs (<=8 bytes, stored via bitcast into i64 payload)
//! and large structs (>8 bytes, heap-allocated with pointer in payload).
//!
//! Uses custom enums to avoid type checker limitations with built-in
//! Option<T>/Result<T,E> generic type parameter resolution.

use super::helpers::*;

// ==================== 1. Small struct (<=8 bytes) in enum ====================

#[test]
fn e2e_p183_enum_small_struct_payload() {
    // Small struct (1 i64 field = 8 bytes) stored in enum variant
    let source = r#"
struct Small {
    val: i64
}

enum Maybe {
    Nothing,
    Just(Small)
}

fn main() -> i64 {
    s := Small { val: 42 }
    opt := Just(s)
    match opt {
        Just(v) => v.val,
        Nothing => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 2. Large struct (>8 bytes) in enum ====================

#[test]
fn e2e_p183_enum_large_struct_payload() {
    // Large struct (2 i64 fields = 16 bytes) stored in enum variant
    let source = r#"
struct Point {
    x: i64,
    y: i64
}

enum Maybe {
    Nothing,
    Just(Point)
}

fn main() -> i64 {
    p := Point { x: 10, y: 32 }
    opt := Just(p)
    match opt {
        Just(val) => val.x + val.y,
        Nothing => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 3. Error struct in Err-like variant ====================

#[test]
fn e2e_p183_enum_err_struct_payload() {
    // Error struct (2 fields = 16 bytes) stored in error variant
    let source = r#"
struct MyError {
    code: i64,
    line: i64
}

enum Outcome {
    Success(i64),
    Failure(MyError)
}

fn fail_with(code: i64, line: i64) -> Outcome {
    Failure(MyError { code: code, line: line })
}

fn main() -> i64 {
    res := fail_with(40, 2)
    match res {
        Success(_v) => 0,
        Failure(e) => e.code + e.line
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 4. Struct in Ok-like variant ====================

#[test]
fn e2e_p183_enum_ok_struct_payload() {
    // Struct stored in success variant, extracted via match
    let source = r#"
struct Value {
    data: i64
}

enum Outcome {
    Success(Value),
    Failure(i64)
}

fn succeed_with(n: i64) -> Outcome {
    Success(Value { data: n })
}

fn main() -> i64 {
    res := succeed_with(42)
    match res {
        Success(v) => v.data,
        Failure(_e) => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 5. Triple-field struct in enum ====================

#[test]
fn e2e_p183_enum_three_field_struct() {
    // Struct with 3 fields (24 bytes) in enum
    let source = r#"
struct Triple {
    a: i64,
    b: i64,
    c: i64
}

enum Wrapper {
    Empty,
    Has(Triple)
}

fn main() -> i64 {
    t := Triple { a: 10, b: 20, c: 12 }
    w := Has(t)
    match w {
        Has(v) => v.a + v.b + v.c,
        Empty => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 6. None-like branch with struct type ====================

#[test]
fn e2e_p183_enum_none_branch_with_struct() {
    // Ensure the None-equivalent branch works when the other variant has struct payload
    let source = r#"
struct Data {
    x: i64
}

enum Maybe {
    Nothing,
    Just(Data)
}

fn main() -> i64 {
    opt := Nothing
    match opt {
        Just(d) => d.x,
        Nothing => 42
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 7. Struct payload round-trip through function ====================

#[test]
fn e2e_p183_enum_struct_via_function() {
    // Struct stored via function call, extracted in caller
    let source = r#"
struct Coord {
    x: i64,
    y: i64
}

enum Shape {
    Empty,
    Located(Coord)
}

fn make_shape(x: i64, y: i64) -> Shape {
    Located(Coord { x: x, y: y })
}

fn main() -> i64 {
    s := make_shape(20, 22)
    match s {
        Located(pos) => pos.x + pos.y,
        Empty => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 8. Both variants carry struct ====================

#[test]
fn e2e_p183_enum_both_variants_struct() {
    // Both variants carry structs — test correct tag dispatch
    let source = r#"
struct Good {
    value: i64
}

struct Bad {
    code: i64
}

enum Either {
    Left(Good),
    Right(Bad)
}

fn main() -> i64 {
    e := Left(Good { value: 42 })
    match e {
        Left(g) => g.value,
        Right(b) => b.code
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== B.1 note: Built-in Option<Struct> / Result<Struct, E> ====================
//
// B.1 (2026-04-21) fixed built-in Option/Result lowering for struct payloads
// in the **inkwell backend** (the default `vaisc build` path). The text-IR
// backend used by these e2e helpers has an independent implementation (see
// `vais-codegen/src/type_inference.rs:460-495` — monomorphization via
// `Option$T`). Fixing the text-IR backend is out of scope for B.1.
//
// Regression coverage for the inkwell backend lives in
// `docs/language/LIVING_SPEC/02_patterns/phase2_10_*_rewrap.vais` — those
// reproducers are exercised directly via `vaisc build` (inkwell default).

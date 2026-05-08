//! Phase 164: VaisDB test_btree — nested slice coercion, generic struct mono, slice open-end
//!
//! Tests for:
//! 1. Ref(Vec<Slice(T)>) → Slice(Slice(T)) nested coercion (TC)
//! 2. Generic struct field access after monomorphization (codegen)
//! 3. Slice source open-end slicing `slice[start..]` (codegen)

use crate::helpers::assert_exit_code;

// ==================== Task 1: Nested Slice Coercion ====================

/// &Vec<i64> should unify with &[i64] parameter via Ref(Vec<T>)↔Slice(T) coercion
#[test]
fn e2e_phase164_basic_vec_to_slice_coercion() {
    assert_exit_code(
        r#"
struct Vec<T> {
    data: i64,
    len: i64
}

impl Vec<T> {
    fn new() -> Vec<T> {
        Vec { data: 0, len: 0 }
    }
}

fn sum_slice(data: &[i64]) -> i64 {
    0
}

fn main() -> i64 {
    v := Vec.new()
    result := sum_slice(&v)
    result
}
"#,
        0,
    );
}

/// Nested slice type: function accepting &[&[i64]] — TC should parse and accept
#[test]
fn e2e_phase164_nested_slice_param_type() {
    assert_exit_code(
        r#"
fn process(data: &[&[i64]]) -> i64 {
    0
}

fn main() -> i64 {
    0
}
"#,
        0,
    );
}

// ==================== Task 3: Slice Open-End Slicing ====================

/// slice[start..] on a Slice/Ref(Slice) source should work
/// The ROADMAP pattern: data[offset..] where data is &[u8]
#[test]
fn e2e_phase164_slice_open_end_from_ref_slice() {
    assert_exit_code(
        r#"
fn process(data: &[i64], offset: i64) -> i64 {
    rest := data[offset..]
    0
}

fn main() -> i64 {
    0
}
"#,
        0,
    );
}

/// slice[start..] on a direct Slice source
#[test]
fn e2e_phase164_slice_open_end_direct_slice() {
    assert_exit_code(
        r#"
fn get_tail(s: &[i64]) -> i64 {
    tail := s[1..]
    0
}

fn main() -> i64 {
    0
}
"#,
        0,
    );
}

// ==================== Task 2: Generic Struct Field Access ====================

/// Generic struct with concrete type param: main-path struct literal now uses
/// the specialized layout %Entry$str (Phase 192 Group B).
#[test]
fn e2e_phase164_generic_struct_field_access() {
    assert_exit_code(
        r#"
struct Entry<T> {
    key: i64,
    value: T
}

fn get_key(e: Entry<str>) -> i64 {
    e.key
}

fn main() -> i64 {
    e := Entry { key: 42, value: "hello" }
    get_key(e)
}
"#,
        42,
    );
}

/// Generic function accessing fields of generic struct — requires monomorphization
/// (Phase 192 Group B).
#[test]
fn e2e_phase164_generic_fn_struct_field_access() {
    assert_exit_code(
        r#"
struct BTreeEntry<T> {
    key_off: i64,
    tid: T
}

fn get_key_off<T>(entry: BTreeEntry<T>) -> i64 {
    entry.key_off
}

struct Row {
    id: i64,
    name: str
}

fn main() -> i64 {
    e := BTreeEntry { key_off: 10, tid: Row { id: 1, name: "test" } }
    get_key_off(e)
}
"#,
        10,
    );
}

/// Generic struct field access where T is used in nested access (Phase 192 Group B).
#[test]
fn e2e_phase164_generic_struct_nested_field() {
    assert_exit_code(
        r#"
struct Wrapper<T> {
    inner: T,
    count: i64
}

struct Data {
    value: i64
}

fn extract<T>(w: Wrapper<T>) -> i64 {
    w.count
}

fn main() -> i64 {
    d := Data { value: 42 }
    w := Wrapper { inner: d, count: 5 }
    extract(w)
}
"#,
        5,
    );
}

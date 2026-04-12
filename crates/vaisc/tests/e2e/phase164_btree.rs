//! Phase 164: VaisDB test_btree — nested slice coercion, generic struct mono, slice open-end
//!
//! Tests for:
//! 1. Ref(Vec<Slice(T)>) → Slice(Slice(T)) nested coercion (TC)
//! 2. Generic struct field access after monomorphization (codegen)
//! 3. Slice source open-end slicing `slice[start..]` (codegen)

use crate::helpers::{assert_compiles, assert_exit_code};

// ==================== Task 1: Nested Slice Coercion ====================

/// &Vec<i64> should unify with &[i64] parameter via Ref(Vec<T>)↔Slice(T) coercion
#[test]
fn e2e_phase164_basic_vec_to_slice_coercion() {
    assert_exit_code(
        r#"
S Vec<T> {
    data: i64,
    len: i64
}

X Vec<T> {
    F new() -> Vec<T> {
        Vec { data: 0, len: 0 }
    }
}

F sum_slice(data: &[i64]) -> i64 {
    0
}

F main() -> i64 {
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
F process(data: &[&[i64]]) -> i64 {
    0
}

F main() -> i64 {
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
F process(data: &[i64], offset: i64) -> i64 {
    rest := data[offset..]
    0
}

F main() -> i64 {
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
F get_tail(s: &[i64]) -> i64 {
    tail := s[1..]
    0
}

F main() -> i64 {
    0
}
"#,
        0,
    );
}

// ==================== Task 2: Generic Struct Field Access ====================

/// Generic struct with concrete type param should allow field access after monomorphization
#[test]
fn e2e_phase164_generic_struct_field_access() {
    assert_compiles(
        r#"
S Entry<T> {
    key: i64,
    value: T
}

F get_key(e: Entry<str>) -> i64 {
    e.key
}

F main() -> i64 {
    e := Entry { key: 42, value: "hello" }
    get_key(e)
}
"#,
    );
}

/// Generic function accessing fields of generic struct — requires monomorphization
#[test]
fn e2e_phase164_generic_fn_struct_field_access() {
    assert_compiles(
        r#"
S BTreeEntry<T> {
    key_off: i64,
    tid: T
}

F get_key_off<T>(entry: BTreeEntry<T>) -> i64 {
    entry.key_off
}

S Row {
    id: i64,
    name: str
}

F main() -> i64 {
    e := BTreeEntry { key_off: 10, tid: Row { id: 1, name: "test" } }
    get_key_off(e)
}
"#,
    );
}

/// Generic struct field access where T is used in nested access
#[test]
fn e2e_phase164_generic_struct_nested_field() {
    assert_compiles(
        r#"
S Wrapper<T> {
    inner: T,
    count: i64
}

S Data {
    value: i64
}

F extract<T>(w: Wrapper<T>) -> i64 {
    w.count
}

F main() -> i64 {
    d := Data { value: 42 }
    w := Wrapper { inner: d, count: 5 }
    extract(w)
}
"#,
    );
}

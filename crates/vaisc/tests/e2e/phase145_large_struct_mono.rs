//! Phase 145: Large Struct Monomorphization Accuracy
//!
//! Tests for:
//! 1. load_typed/store_typed with large structs (>8 bytes) via memcpy
//! 2. compute_sizeof correctness for generic structs with concrete type args
//! 3. Manual Vec-like container with large struct elements
//! 4. type_size() inside generic context returning correct struct sizes
//! 5. Large struct round-trip through generic containers

use super::helpers::*;

// ==================== 1. type_size for large struct in generic function ====================

#[test]
fn e2e_phase145_type_size_two_field_struct() {
    // type_size() inside generic function should return 16 for a two-i64-field struct
    let source = r#"
S Point {
    x: i64,
    y: i64
}

F get_elem_size<T>(x: T) -> i64 {
    type_size()
}

F main() -> i64 {
    p := Point { x: 1, y: 2 }
    get_elem_size(p)
}
"#;
    assert_exit_code(source, 16);
}

#[test]
fn e2e_phase145_type_size_three_field_struct() {
    // type_size() returns 24 for a struct with three i64 fields
    let source = r#"
S Triple {
    a: i64,
    b: i64,
    c: i64
}

F get_size<T>(x: T) -> i64 {
    type_size()
}

F main() -> i64 {
    t := Triple { a: 1, b: 2, c: 3 }
    get_size(t)
}
"#;
    assert_exit_code(source, 24);
}

// ==================== 2. sizeof for various struct sizes ====================

#[test]
fn e2e_phase145_sizeof_two_i64_fields() {
    let source = r#"
S Pair {
    x: i64,
    y: i64
}

F main() -> i64 {
    p := Pair { x: 1, y: 2 }
    sizeof(p)
}
"#;
    assert_exit_code(source, 16);
}

#[test]
fn e2e_phase145_sizeof_four_i64_fields() {
    let source = r#"
S Wide {
    a: i64,
    b: i64,
    c: i64,
    d: i64
}

F main() -> i64 {
    w := Wide { a: 1, b: 2, c: 3, d: 4 }
    sizeof(w)
}
"#;
    assert_exit_code(source, 32);
}

// ==================== 3. Manual store_typed/load_typed for large struct ====================

#[test]
fn e2e_phase145_store_load_typed_16_byte_struct() {
    // Manually store and load a 16-byte struct using store_typed/load_typed
    // with generic substitution providing T = Pair
    let source = r#"
S Pair {
    x: i64,
    y: i64
}

F store_and_load<T>(value: T) -> T {
    es := type_size()
    ptr := malloc(es)
    store_typed(ptr, value)
    result := load_typed(ptr)
    free(ptr)
    result
}

F main() -> i64 {
    p := Pair { x: 10, y: 32 }
    q := store_and_load(p)
    q.x + q.y
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase145_store_load_typed_24_byte_struct() {
    // Store and load a 24-byte struct
    let source = r#"
S RGB {
    r: i64,
    g: i64,
    b: i64
}

F roundtrip<T>(value: T) -> T {
    es := type_size()
    ptr := malloc(es)
    store_typed(ptr, value)
    result := load_typed(ptr)
    free(ptr)
    result
}

F main() -> i64 {
    color := RGB { r: 10, g: 20, b: 12 }
    loaded := roundtrip(color)
    loaded.r + loaded.g + loaded.b
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 4. Overwrite verification ====================

#[test]
fn e2e_phase145_store_typed_overwrite() {
    // Store a value, overwrite it, verify the new value is loaded
    let source = r#"
S Data {
    key: i64,
    value: i64
}

F store_val<T>(ptr: i64, value: T) -> i64 {
    store_typed(ptr, value)
    0
}

F load_val<T>(ptr: i64, _dummy: T) -> T {
    load_typed(ptr)
}

F main() -> i64 {
    es := sizeof(Data { key: 0, value: 0 })
    ptr := malloc(es)
    store_val(ptr, Data { key: 1, value: 99 })
    store_val(ptr, Data { key: 2, value: 42 })
    d := load_val(ptr, Data { key: 0, value: 0 })
    free(ptr)
    d.value
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 7. elem_size / sizeof consistency ====================

#[test]
fn e2e_phase145_elem_size_consistency() {
    // sizeof(T) and type_size() should agree for large struct
    let source = r#"
S Wide {
    a: i64,
    b: i64,
    c: i64,
    d: i64
}

F check<T>(x: T) -> i64 {
    s1 := sizeof(x)
    s2 := type_size()
    I s1 == s2 && s1 == 32 { 42 } E { 0 }
}

F main() -> i64 {
    w := Wide { a: 1, b: 2, c: 3, d: 4 }
    check(w)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 8. Large struct identity through generic function ====================

#[test]
fn e2e_phase145_large_struct_identity() {
    // Pass a large struct through a generic identity function, verify fields preserved
    let source = r#"
S BigData {
    x: i64,
    y: i64,
    z: i64
}

F identity<T>(val: T) -> T { val }

F main() -> i64 {
    d := BigData { x: 10, y: 20, z: 12 }
    result := identity(d)
    result.x + result.y + result.z
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 9. Store/load 32-byte struct ====================

#[test]
fn e2e_phase145_store_load_32_byte_struct() {
    // 32-byte struct roundtrip through generic store/load
    let source = r#"
S Quad {
    a: i64,
    b: i64,
    c: i64,
    d: i64
}

F roundtrip<T>(value: T) -> T {
    es := type_size()
    ptr := malloc(es)
    store_typed(ptr, value)
    result := load_typed(ptr)
    free(ptr)
    result
}

F main() -> i64 {
    q := Quad { a: 10, b: 12, c: 8, d: 12 }
    loaded := roundtrip(q)
    loaded.a + loaded.b + loaded.c + loaded.d
}
"#;
    assert_exit_code(source, 42);
}

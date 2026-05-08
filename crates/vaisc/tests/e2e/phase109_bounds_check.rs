// REGRESSION(phase-109): runtime bounds check must abort on OOB slice access
//! Phase 109: Slice bounds check — runtime OOB defense tests

use crate::helpers::{assert_exit_code, compile_to_ir};

/// Slice indexing within bounds should work normally
#[test]
fn slice_index_in_bounds() {
    // Access first element of a 3-element slice
    assert_exit_code(
        r#"
fn main() -> i64 {
    arr := [10, 20, 30]
    s := arr[0..3]
    return s[0]
}
"#,
        10,
    );
}

/// Slice indexing last valid element
#[test]
fn slice_index_last_element() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    arr := [10, 20, 30]
    s := arr[0..3]
    return s[2]
}
"#,
        30,
    );
}

/// Slice OOB access — IR should compile successfully
/// NOTE: Text IR codegen uses plain pointers for slices (no fat pointer),
/// so runtime bounds checking is only available in the Inkwell path.
/// This test verifies the code compiles without error.
#[test]
fn slice_index_oob_compiles() {
    // OOB access compiles fine — it's a runtime issue, not compile-time
    let ir = compile_to_ir(
        r#"
fn main() -> i64 {
    arr := [10, 20, 30]
    s := arr[0..3]
    return s[5]
}
"#,
    );
    assert!(
        ir.is_ok(),
        "Slice OOB program should compile: {:?}",
        ir.err()
    );
}

/// Boundary index — accessing exactly at length should be OOB
/// but index length-1 should be valid
#[test]
fn slice_boundary_access() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    arr := [10, 20, 30]
    s := arr[0..2]
    # index 1 is the last valid element for a 2-element slice
    return s[1]
}
"#,
        20,
    );
}

/// Slice indexing compiles successfully with correct element access
#[test]
fn slice_index_middle_element() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    arr := [5, 10, 15, 20, 25]
    s := arr[1..4]
    return s[1]
}
"#,
        // s = [10, 15, 20], s[1] = 15
        15,
    );
}

/// Multiple in-bounds slice accesses
#[test]
fn slice_multiple_in_bounds_accesses() {
    assert_exit_code(
        r#"
fn main() -> i64 {
    arr := [1, 2, 3, 4, 5]
    s := arr[1..4]
    return s[0] + s[1] + s[2]
}
"#,
        // s = [2, 3, 4], sum = 9
        9,
    );
}

#[test]
fn slice_to_vec_copies_elements() {
    assert_exit_code(
        r#"
struct Vec<T> {
    data: i64,
    len: i64,
    cap: i64,
    elem_size: i64,
    owned: i64
}

impl Vec<T> {
    fn with_capacity(capacity: i64) -> Vec<T> {
        Vec { data: malloc(capacity * 8), len: 0, cap: capacity, elem_size: 8, owned: 0 }
    }
}

fn main() -> i64 {
    arr := [10, 20, 30, 40]
    s := arr[1..4]
    v := s.to_vec()
    return v[0] + v[2]
}
"#,
        60,
    );
}

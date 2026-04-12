//! Phase 166: Cross-module Vec→Slice coercion at call sites
//!
//! Tests that calling a function expecting &[T] with &Vec<T> works correctly
//! in cross-module codegen, where typed pointers can cause LLVM type mismatches
//! between Vec struct pointers and slice fat pointers.

use crate::helpers::{assert_compiles, assert_exit_code};

/// Basic: function expecting &[i64] called with &Vec<i64>
#[test]
fn e2e_phase166_vec_to_slice_arg_coercion() {
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
    sum_slice(&v)
}
"#,
        0,
    );
}

/// Nested slice: function expecting &[&[u8]] called with &Vec<&[u8]>
#[test]
fn e2e_phase166_nested_vec_to_slice_coercion() {
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

F encode_composite_key(parts: &[&[u8]]) -> i64 {
    0
}

F main() -> i64 {
    parts := Vec.new()
    encode_composite_key(&parts)
}
"#,
        0,
    );
}

/// Vec passed directly (not as &Vec) to slice parameter
#[test]
fn e2e_phase166_vec_direct_to_slice() {
    assert_compiles(
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

F process(data: &[i64]) -> i64 {
    0
}

F main() -> i64 {
    v := Vec.new()
    process(v)
}
"#,
    );
}

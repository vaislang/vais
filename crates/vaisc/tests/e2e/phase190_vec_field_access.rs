//! Phase 190 #3: `v[i].field` direct access without intermediate variable.
//!
//! Prior to this fix, chained index + field access on `Vec<S>`, `&[S]`, or
//! `[S; N]` triggered an ICE in `infer_struct_name` because `Expr::Index`
//! fell through to the catch-all error arm. Users had to write
//! `tmp := v[0]; tmp.field` as a workaround.
//!
//! These tests assert the ICE is gone — i.e. the program compiles. Runtime
//! offset correctness for indexing is a separate pre-existing concern
//! tracked outside of this task.

use super::helpers::*;

#[test]
fn e2e_phase190_slice_index_field_compiles() {
    assert_compiles(
        r#"
S Point { x: i64, y: i64 }

F get_x(arr: &[Point]) -> i64 {
  arr[0].x
}

F main() -> i64 {
  data := [Point { x: 11, y: 20 }, Point { x: 33, y: 40 }]
  get_x(&data)
}
"#,
    );
}

#[test]
fn e2e_phase190_slice_index_field_with_index_param_compiles() {
    assert_compiles(
        r#"
S Point { x: i64, y: i64 }

F get_x(arr: &[Point], i: i64) -> i64 {
  arr[i].x
}

F main() -> i64 {
  data := [Point { x: 1, y: 2 }, Point { x: 3, y: 4 }]
  get_x(&data, 1)
}
"#,
    );
}

#[test]
fn e2e_phase190_slice_mut_index_field_compiles() {
    assert_compiles(
        r#"
S Item { id: i64, qty: i64 }

F first_id(arr: &[Item]) -> i64 {
  arr[0].id
}

F main() -> i64 {
  data := [Item { id: 42, qty: 1 }, Item { id: 99, qty: 2 }]
  first_id(&data)
}
"#,
    );
}

//! Phase 190 #7: i64 → i1 coercion when passing a bool-typed value to an
//! i1 parameter.
//!
//! Codegen zext's comparison results (`i1`) to i64 "for consistency," but
//! the original type was bool — so when that value flows into a call whose
//! parameter is declared as `bool` (i1), the generated IR had an `i1 %tX`
//! operand whose definition was actually `i64`, which clang rejects as
//! `defined with type 'i64' but expected 'i1'`.
//!
//! These tests exercise the coercion by threading a comparison result
//! through a local variable and passing it into a bool-typed parameter.

use super::helpers::*;

#[test]
fn e2e_phase190_bool_local_to_bool_param_compiles() {
    assert_compiles(
        r#"
F takes_bool(flag: bool) -> i64 {
  I flag { 1 } E { 0 }
}

F main() -> i64 {
  x := 5
  is_big := x > 3
  takes_bool(is_big)
}
"#,
    );
}

#[test]
fn e2e_phase190_bool_inline_comparison_to_bool_param_compiles() {
    assert_compiles(
        r#"
F takes_bool(flag: bool) -> i64 {
  I flag { 42 } E { 0 }
}

F main() -> i64 {
  n := 10
  takes_bool(n > 5)
}
"#,
    );
}

#[test]
fn e2e_phase190_multiple_bool_params_compiles() {
    assert_compiles(
        r#"
F both(a: bool, b: bool) -> i64 {
  I a { I b { 3 } E { 2 } } E { I b { 1 } E { 0 } }
}

F main() -> i64 {
  x := 5
  y := 10
  both(x > 3, y > 3)
}
"#,
    );
}

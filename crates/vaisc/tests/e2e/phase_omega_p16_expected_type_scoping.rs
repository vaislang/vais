//! Phase Ω P1.6: function-return expected-type hint scoping
//!
//! Regression: prior to iter 132, `check_function` pushed the function's
//! declared return type onto `expected_type_stack` for the ENTIRE body
//! (both block and single-expr bodies). Inside block-bodied functions
//! returning `Vec<U>`, every `Vec.with_capacity(N)` for an unrelated
//! local accumulator had its element-type Var bound to `U` via the
//! make-and-bind path in `checker_expr::calls`. This silently
//! corrupted independent local Vec bindings and surfaced as confusing
//! "expected u8 found str" mismatches at later argument or push sites.
//!
//! The fix scopes the push to `FunctionBody::Expr` only — single tail
//! expressions where the body type IS the return type. Block bodies
//! rely on the body→return unification at the end of `check_function`
//! and the `pending_method_instantiations` deferred-recording mechanism
//! (Phase 193 R-1b).
//!
//! These tests pin the contract:
//!   1. Independent local Vec accumulators in a block-bodied function
//!      returning a Vec must keep distinct fresh element-type vars.
//!   2. The C16 case (`vec_new_t<T>() -> Vec<T>` body = single
//!      `Vec.with_capacity(8)` expr) must still bind T correctly.

use crate::helpers::assert_compiles;

/// Two unrelated local Vec accumulators in a block body returning Vec<str>
/// must NOT have their element types unified with str just because the
/// function returns Vec<str>. The compile must succeed even though one
/// accumulator is later used as `&[u8]` (via Vec→Slice coercion) and
/// another is pushed with `str` values.
#[test]
fn p16_block_body_expected_type_does_not_leak_into_local_vec() {
    assert_compiles(
        r#"
F from_utf8_lossy(bytes: &[u8]) -> str {
    bytes as str
}

F parse_csv_minimal(data: &[u8]) -> Vec<str> {
    fields := mut Vec.with_capacity(0u64)
    current_field := mut Vec.with_capacity(0u64)
    field_str := mut from_utf8_lossy(&current_field)
    fields.push(field_str)
    current_field.push(72u8)
    R fields
}

F main() -> i64 { R 0 }
"#,
    );
}

/// Loop + branch shape that originally trapped vaisdb's `parse_csv_line`.
/// The function constructs two `Vec.with_capacity(0u64)` accumulators with
/// different element types and uses both inside a while loop with three
/// if/else branches.
#[test]
fn p16_csv_loop_branch_shape_compiles() {
    assert_compiles(
        r#"
F from_utf8_lossy(bytes: &[u8]) -> str {
    bytes as str
}

F parse_csv(data: &[u8]) -> Vec<str> {
    fields := mut Vec.with_capacity(0u64)
    current_field := mut Vec.with_capacity(0u64)
    in_quotes := mut false
    i := mut 0
    LW i < data.len() {
        byte := mut data[i]
        I byte == 34u8 {
            in_quotes = !in_quotes
        } EL I byte == 44u8 && !in_quotes {
            field_str := mut from_utf8_lossy(&current_field)
            fields.push(field_str)
            current_field.clear()
        } EL {
            current_field.push(byte)
        }
        i = i + 1
    }
    R fields
}

F main() -> i64 { R 0 }
"#,
    );
}

/// C16 protection: a generic constructor whose body is a single
/// `Vec.with_capacity(8)` expression should still see the function's
/// `Vec<T>` return type as a hint and bind T into the fresh element-type
/// var so downstream monomorphization records the correct callee args.
/// (FunctionBody::Expr path retains the push.)
#[test]
fn p16_single_expr_body_still_consumes_return_hint() {
    assert_compiles(
        r#"
EN Color { Red, Green, Blue }

F vec_new_color() -> Vec<Color> {
    Vec.with_capacity(8)
}

F main() -> i64 {
    v := mut vec_new_color()
    v.push(Red)
    v.push(Green)
    R 0
}
"#,
    );
}

/// Multiple unrelated HashMap accumulators in a block body returning a
/// HashMap must not have their key/value vars cross-bound to the return
/// type's key/value.
#[test]
fn p16_hashmap_block_body_no_leak() {
    assert_compiles(
        r#"
F build() -> HashMap<str, i64> {
    out := mut HashMap.new()
    aux := mut HashMap.new()
    aux.insert(1u8, 2u8)
    out.insert("k", 1)
    R out
}

F main() -> i64 { R 0 }
"#,
    );
}

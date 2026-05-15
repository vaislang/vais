//! Phase Ω P1.7 (iter 134): closure-param hint propagation
//!
//! Stage A finding: `Vec<T>.sort_by(|a, b| a.field > b.field { ... })` failed
//! with E030 "no field 'field' on type '?N'" because the Lambda's `Type::Infer`
//! params resolved to fresh Vars without inheriting the receiver's element
//! type from the formal `cmp: fn(T, T) -> i64` signature.
//!
//! The fix introduces `lambda_param_hint_stack`: when a method-call path knows
//! the Fn/FnPtr param type, it pushes the first slot onto the stack before
//! recursing into the closure arg. The Lambda check pops the hint and unifies
//! it with each `Type::Infer`-param's fresh Var so the body sees the concrete
//! field type.
//!
//! Generic("T") in the formal signature is substituted with the receiver's
//! first generic argument (Vec<T>'s T) when the formal slot is itself a
//! Generic.

use crate::helpers::assert_compiles;

/// Vec<Struct>.sort_by(|a, b| a.field > b.field) — the canonical case from
/// vaisdb's planner/pipeline.vais:185 that the hint was designed for.
#[test]
fn p17_sort_by_struct_field_access() {
    assert_compiles(
        r#"
struct Item {
    score: f64,
    id: i64,
}

fn sort_items(items: &mut Vec<Item>) {
    items.sort_by(|a, b| {
        I b.score > a.score { 1 }
        else I b.score < a.score { -1 }
        else { 0 }
    })
}

fn main() -> i64 { return 0 }
"#,
    );
}

/// Generic-param case: when the receiver is `Vec<T>` (T is a generic param of
/// the enclosing function), the hint should still be applied — just via T
/// rather than a concrete struct name. This test pins that the substitution
/// path through `receiver_generics.first()` survives.
#[test]
fn p17_sort_by_with_generic_receiver() {
    assert_compiles(
        r#"
fn sort_any<T>(items: &mut Vec<T>, cmp: fn(T, T) -> i64) {
    items.sort_by(|a, b| cmp(a, b))
}

fn main() -> i64 { return 0 }
"#,
    );
}

/// Vec.sort with no closure (Unit return) — confirms the new code path
/// doesn't accidentally reject the no-arg sort case.
#[test]
fn p17_sort_no_closure_still_compiles() {
    assert_compiles(
        r#"
fn main() -> i64 {
    v: Vec<i64> := mut Vec.new()
    v.push(3)
    v.push(1)
    v.push(2)
    v.sort()
    return 0
}
"#,
    );
}

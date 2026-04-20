//! Phase 6.28.1 — Parser fix: block-terminated expressions (I / L / LW / LF /
//! M / Block) must NOT consume a following `*` as the Mul operator. The `*`
//! starts the next statement's Deref.
//!
//! Bug repro, pre-fix:
//!   LW cond { body }
//!   *used.get_mut(i) = v
//! parsed as `(LW{}) * used.get_mut(i) = v`, then surfaced as a confusing
//! E001 "expected numeric, found ()" on the LW span.
//!
//! These tests lock the correct parse by asserting the program compiles.

use crate::helpers::assert_compiles;

#[test]
fn e2e_phase6_28_lw_then_deref_assign() {
    assert_compiles(
        r#"
F main() -> i64 {
    ids := mut Vec.new();
    ids.push(1u64);
    used := mut Vec.new();
    used.push(false);

    LW 0 < ids.len() { }
    *used.get_mut(0) = true;
    0
}
"#,
    );
}

#[test]
fn e2e_phase6_28_if_then_deref_assign() {
    assert_compiles(
        r#"
F main() -> i64 {
    ids := mut Vec.new();
    ids.push(1u64);

    I ids.len() > 0 { }
    *ids.get_mut(0) = 42u64;
    0
}
"#,
    );
}

#[test]
fn e2e_phase6_28_match_then_deref_assign() {
    assert_compiles(
        r#"
F main() -> i64 {
    ids := mut Vec.new();
    ids.push(1u64);

    M ids.len() { _ => {} }
    *ids.get_mut(0) = 99u64;
    0
}
"#,
    );
}

/// Phase 6.28.3: `.ok_or_else(...)` on a literal `Optional(T)` receiver.
/// Prior to the fix, the Phase 271 fallback was inside the `receiver_named=
/// Some(Named)` block and never fired for bare Optional. Chains like
/// `guard.get(&k).ok_or_else(|| err)` failed with E004 "function 'ok_or_else'
/// is not defined". Fix added handling under Phase 311's Optional arm.
#[test]
fn e2e_phase6_28_ok_or_else_on_optional() {
    // Use an explicit Optional(...) via Vec.pop() which returns Option<T>.
    assert_compiles(
        r#"
partial F main() -> i64 {
    v := mut Vec.new();
    v.push(42i64);
    result := v.pop().ok_or_else(|| "empty".to_string())!;
    result
}
"#,
    );
}

#[test]
fn e2e_phase6_28_ok_or_on_optional() {
    assert_compiles(
        r#"
partial F main() -> i64 {
    v := mut Vec.new();
    v.push(7i64);
    result := v.pop().ok_or("none".to_string())!;
    result
}
"#,
    );
}

#[test]
fn e2e_phase6_28_nested_lw_windows_pattern() {
    // Real vaisdb window.vais shape: nested LW with inner deref-assign on a
    // sibling Vec<bool>. The trailing `j = j + 1` keeps the loop well-formed.
    assert_compiles(
        r#"
F merge(items: &Vec<i64>) -> i64 {
    used := mut Vec.new();
    k := mut 0u32;
    LW k < items.len() as u32 {
        used.push(false);
        k = k + 1;
    }

    i := mut 0u32;
    LW i < items.len() as u32 {
        j := mut i + 1;
        LW j < items.len() as u32 {
            *used.get_mut(j as u64) = true;
            j = j + 1;
        }
        i = i + 1;
    }
    0
}

F main() -> i64 {
    v: Vec<i64> = Vec.new();
    merge(&v)
}
"#,
    );
}

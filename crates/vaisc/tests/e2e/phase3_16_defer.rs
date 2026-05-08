//! Phase 3.16 — D (defer) scope-exit codegen verification.
//!
//! Confirms that defer currently works for: single defer, multiple defers
//! (reverse order), early return, observation via global state.

use super::helpers::*;

#[test]
fn defer_runs_after_return_value_evaluated() {
    // Defer writes 99 to x, but return value x is already captured.
    let source = r#"
fn compute() -> i64 {
    x := mut 10
    D { x = 99 }
    x
}
fn main() -> i64 { compute() }
"#;
    assert_exit_code(source, 10);
}

#[test]
fn defer_observable_via_global() {
    // Global OBSERVED = 99 after compute() returns → main reads post-defer state.
    let source = r#"
G OBSERVED: i64 = 0

fn compute() -> i64 {
    D { OBSERVED = 99 }
    10
}

fn main() -> i64 {
    _r := compute()
    OBSERVED
}
"#;
    assert_exit_code(source, 99);
}

#[test]
fn defer_multiple_reverse_order() {
    // Three defers — last one registered runs first (LIFO).
    // OBSERVED should end with whatever the FIRST-registered defer wrote
    // (because it runs last).
    let source = r#"
G OBSERVED: i64 = 0

fn compute() -> i64 {
    D { OBSERVED = 1 }
    D { OBSERVED = 2 }
    D { OBSERVED = 3 }
    0
}

fn main() -> i64 {
    _r := compute()
    OBSERVED
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn defer_with_early_return() {
    // Defer runs even when exiting via early R.
    let source = r#"
G OBSERVED: i64 = 0

fn compute(cond: bool) -> i64 {
    D { OBSERVED = 42 }
    I cond { return 7 }
    0
}

fn main() -> i64 {
    _r := compute(true)
    OBSERVED
}
"#;
    assert_exit_code(source, 42);
}

// ================ B.5: edge-case tests for existing defer behavior ================
//
// Vais semantics (per LANGUAGE_SPEC "Defer block (runs on scope exit)"):
//   - Defer runs AFTER return value is evaluated but BEFORE control transfers.
//   - Multiple defers run in LIFO order.
//   - Defers added before an early `R` path still fire.
//   - Nested function calls have independent defer stacks.
//   - Defers inside loop bodies run at function exit (current impl — function-scoped
//     stack), not per-iteration. Per-iteration is a Phase 4.x followup.

#[test]
fn defer_lifo_ordering_with_early_return() {
    // Three defers, one of which mutates OBSERVED via LIFO — the FIRST-registered
    // defer (OBSERVED=1) runs LAST, so OBSERVED ends at 1.
    let source = r#"
G OBSERVED: i64 = 0

fn compute() -> i64 {
    D { OBSERVED = 1 }
    D { OBSERVED = 2 }
    D { OBSERVED = 3 }
    I true { return 7 }
    0
}

fn main() -> i64 {
    _r := compute()
    OBSERVED
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn defer_nested_function_calls_independent_stacks() {
    // Defer stacks per function: compute() sets OBS=10 after finishing;
    // main's own defer overwrites OBS=20 after main's body runs.
    let source = r#"
G OBS: i64 = 0

fn compute() -> i64 {
    D { OBS = 10 }
    5
}

fn main() -> i64 {
    _r := compute()
    x := OBS
    D { OBS = 20 }
    x
}
"#;
    // compute() returns → OBS=10. x=10 captured. main defers fire on return →
    // OBS=20, but main returns x=10. The test observes main's return value.
    assert_exit_code(source, 10);
}

#[test]
fn defer_mutable_local_with_early_return() {
    // Defer runs after return value captured — mutation of local `x` via
    // defer is not visible to caller, matching Go's behavior.
    let source = r#"
fn compute(early: bool) -> i64 {
    x := mut 100
    D { x = 0 }
    I early { return x }
    x * 2
}

fn main() -> i64 {
    compute(true)
}
"#;
    // early=true → R x returns 100 (defer hasn't run yet at value evaluation).
    assert_exit_code(source, 100);
}

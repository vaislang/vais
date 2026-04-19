//! Phase 3.16 — D (defer) scope-exit codegen verification.
//!
//! Confirms that defer currently works for: single defer, multiple defers
//! (reverse order), early return, observation via global state.

use super::helpers::*;

#[test]
fn defer_runs_after_return_value_evaluated() {
    // Defer writes 99 to x, but return value x is already captured.
    let source = r#"
F compute() -> i64 {
    x := mut 10
    D { x = 99 }
    x
}
F main() -> i64 { compute() }
"#;
    assert_exit_code(source, 10);
}

#[test]
fn defer_observable_via_global() {
    // Global OBSERVED = 99 after compute() returns → main reads post-defer state.
    let source = r#"
G OBSERVED: i64 = 0

F compute() -> i64 {
    D { OBSERVED = 99 }
    10
}

F main() -> i64 {
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

F compute() -> i64 {
    D { OBSERVED = 1 }
    D { OBSERVED = 2 }
    D { OBSERVED = 3 }
    0
}

F main() -> i64 {
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

F compute(cond: bool) -> i64 {
    D { OBSERVED = 42 }
    I cond { R 7 }
    0
}

F main() -> i64 {
    _r := compute(true)
    OBSERVED
}
"#;
    assert_exit_code(source, 42);
}

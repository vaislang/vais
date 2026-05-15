//! Phase 4c.2 / Task #53 — totality gate (`partial` modifier) E2E tests.
//!
//! These tests cover the end-to-end compilation of the `partial F ...`
//! function-prefix modifier and the totality enforcement pass in
//! `crates/vais-types/src/totality.rs`.
//!
//! ## Test matrix
//!
//! The totality gate classifies expressions into panic vs non-panic. We
//! verify both sides:
//!
//! **Panic sources (non-partial ⇒ rejected)**
//!   - `panic(...)`  / `abort(...)` / `exit(...)` / `__panic(...)`  builtins
//!   - `assert(...)`  expression form
//!   - `expr!`  — the `Unwrap` operator
//!   - Transitively calling a non-partial function whose body contains
//!     any of the above
//!   - Calling a `partial`-marked function directly
//!
//! **Explicitly-accepted panic-adjacent ops (non-partial ⇒ still OK)**
//!   - `a / b`, `a % b`  — division/mod (delegated to refinement types)
//!   - `a /= b`, `a %= b`  — compound division
//!   - `arr[idx]`  — indexing (delegated to refinement types)
//!   - `expr?`  — `Try` operator (controlled Result/Option propagation)
//!
//! See the module-level doc comment on `crates/vais-types/src/totality.rs`
//! for the rationale on each case — in particular the empirical iter 10
//! measurement where the strict-gate form rejected 187/2526 programs
//! (~85% of those being legitimate arithmetic) and was narrowed to the
//! set above.

use super::helpers::*;

// ====================================================================
// Positive cases — programs that should compile under the totality gate
// ====================================================================

#[test]
fn e2e_phase4c2_partial_main_with_assert_compiles() {
    // A `partial F main` is explicitly allowed to contain `assert`.
    let source = r#"
partial fn main() -> i64 {
    x := 10
    assert(x > 0)
    return x
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_phase4c2_partial_main_with_unwrap_compiles() {
    // A `partial F main` is explicitly allowed to contain `!` unwrap.
    let source = r#"
enum Result { Ok(i64), Err(i64) }

fn get_value() -> Result {
    Ok(42)
}

partial fn main() -> i64 {
    get_value()!
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase4c2_total_main_with_try_operator_compiles() {
    // `expr?` is NOT a panic source — it is controlled Result/Option
    // early-return and a total function is explicitly allowed to use it.
    // This is the critical regression guard for the "Try is not panic"
    // design decision documented in `totality.rs`.
    let source = r#"
enum Result { Ok(i64), Err(i64) }

fn parse_num() -> Result {
    return Ok(7)
}

fn add_one(x: i64) -> i64 {
    return x + 1
}

fn get_number() -> Result {
    return Ok(add_one(parse_num()?))
}

fn main() -> i64 {
    match get_number() {
        Ok(v) => v,
        Err(_) => 99
    }
}
"#;
    assert_exit_code(source, 8);
}

#[test]
fn e2e_phase4c2_total_main_with_division_compiles() {
    // Division is explicitly NOT a panic source in the narrowed gate —
    // safety of the divisor is delegated to refinement types (Phase 4c.1
    // / `{b: i64 | b != 0}`). The codegen still emits its runtime
    // div-by-zero guard as a backstop.
    //
    // This is the critical regression guard for the "div is not panic"
    // design decision — without it, 101 existing E2E tests were broken
    // by the totality gate (iter 10 empirical measurement).
    let source = r#"
fn main() -> i64 {
    x := 42
    y := 7
    return x / y
}
"#;
    assert_exit_code(source, 6);
}

#[test]
fn e2e_phase4c2_total_main_with_modulo_compiles() {
    // Companion to the division regression guard: `%` is likewise NOT
    // a panic source. iter 10 `gcd` test was the motivating case.
    let source = r#"
fn gcd(a: i64, b: i64) -> i64 {
    I b == 0 { return a }
    return @(b, a % b)
}

fn main() -> i64 = gcd(126, 84)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase4c2_total_main_with_indexing_compiles() {
    // Indexing is explicitly NOT a panic source in the narrowed gate —
    // bounds safety lives in refinement types or `.get(idx)` APIs.
    // Companion regression guard to the division cases.
    let source = r#"
fn main() -> i64 {
    arr := [10, 20, 30, 40]
    return arr[1] + arr[2]
}
"#;
    assert_exit_code(source, 50);
}

#[test]
fn e2e_phase4c2_partial_helper_called_from_partial_main_compiles() {
    // `partial` propagates: a `partial` caller can call another
    // `partial` function without itself losing that marker.
    let source = r#"
partial fn ensure_positive(x: i64) -> i64 {
    assert(x > 0)
    return x
}

partial fn main() -> i64 {
    ensure_positive(11)
}
"#;
    assert_exit_code(source, 11);
}

// ====================================================================
// Negative cases — programs that should be rejected by the totality gate
// ====================================================================

#[test]
fn e2e_phase4c2_total_main_with_assert_rejected() {
    // Mirror of the positive `assert` test, minus the `partial` modifier.
    // Must be rejected with `TotalFunctionViolation`.
    let source = r#"
fn main() -> i64 {
    x := 10
    assert(x > 0)
    return x
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase4c2_total_main_with_unwrap_rejected() {
    // Mirror of the positive `!` test, minus `partial`.
    let source = r#"
enum Result { Ok(i64), Err(i64) }

fn get_value() -> Result {
    Ok(42)
}

fn main() -> i64 {
    get_value()!
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase4c2_total_main_with_panic_builtin_rejected() {
    // The `panic` / `abort` / `exit` / `__panic` / `assert` builtins are
    // all classified as panic sources. We test one of them — the rest
    // share the same dispatch in `totality.rs` PANIC_BUILTINS.
    let source = r#"
N "C" {
    fn abort() -> i64
}

fn main() -> i64 {
    abort()
    return 0
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase4c2_total_caller_of_partial_rejected() {
    // Transitive rejection: a total function that calls a `partial`
    // function is itself flagged as reachable-panic. This is the
    // primary mechanism by which non-local panic-reachability
    // propagates across the call graph.
    let source = r#"
partial fn dangerous() -> i64 {
    panic_marker := 0
    assert(panic_marker == 0)
    return panic_marker
}

fn main() -> i64 {
    dangerous()
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase4c2_total_caller_of_unwrapping_helper_rejected() {
    // Second transitive case: `helper` directly uses `!` (so it's
    // reachable-panic via the direct rule), and `main` calls it (so it
    // becomes reachable-panic via the worklist propagation rule).
    let source = r#"
enum Result { Ok(i64), Err(i64) }

fn get_ok() -> Result {
    Ok(5)
}

fn helper() -> i64 {
    get_ok()!
}

fn main() -> i64 {
    helper()
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase4c2_baseline_no_panic_source_compiles() {
    // Control test: a genuinely panic-free total function must compile
    // cleanly — this is the "0 false positives on simple code" guard.
    // If this test ever fails, it means a future totality-gate change
    // started flagging trivially safe code.
    let source = r#"
fn add_one(x: i64) -> i64 = x + 1

fn main() -> i64 {
    add_one(41)
}
"#;
    assert_exit_code(source, 42);
}

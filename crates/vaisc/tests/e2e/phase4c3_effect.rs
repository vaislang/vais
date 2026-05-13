//! Phase 4c.3 / Task #54 — effect purity gate (`pure`/`io`/`alloc`
//! prefix) E2E tests.
//!
//! These tests cover the end-to-end compilation of the Phase 4c.3
//! effect-prefix keywords and the enforcement pass in
//! `crates/vais-types/src/effect_purity.rs`.
//!
//! ## Design
//!
//! A function carrying `pure F`, `io F`, or `alloc F` has its body and
//! every transitively-reachable callee checked against the declared
//! ceiling:
//!
//! | prefix  | allowed effects            | forbidden   |
//! |---------|----------------------------|-------------|
//! | `pure`  | no IO, no Alloc            | IO, Alloc   |
//! | `io`    | IO (plus pure body)        | Alloc       |
//! | `alloc` | Alloc (plus pure body)     | IO          |
//!
//! Subtype: `pure ⊆ io`, `pure ⊆ alloc`; `io` and `alloc` are
//! incomparable (disjoint).
//!
//! Functions *without* a prefix are inferred — they do not participate
//! in the gate directly, but their effects propagate to any declared
//! caller that invokes them (transitive closure over the call graph).
//!
//! IO builtins currently tracked: `print`, `println`, `eprint`,
//! `eprintln`, `puts`, `putchar`, `printf`, and the file/network IO
//! set (`fopen`/`open`/`socket`/...). Alloc builtins: `malloc`,
//! `calloc`, `realloc`, `free`, `alloc`, `dealloc`. See
//! `effect_purity::{IO_BUILTINS, ALLOC_BUILTINS}` for the full list.
//!
//! ## Test matrix
//!
//! Positives (should compile + run):
//!   - `pure F` with pure arithmetic body
//!   - `io F` calling `println` directly
//!   - `io F` calling another `pure F`
//!   - `alloc F` calling `malloc`
//!   - Non-declared function calling `println` (no gate for it)
//!   - Legacy `#[pure]` attribute still works alongside the new keyword
//!
//! Negatives (should fail compilation):
//!   - `pure F` calling `println` directly
//!   - `pure F` calling `malloc` directly
//!   - `pure F` transitively reaching `println` through an untagged helper
//!   - `io F` calling `malloc` (alloc forbidden inside io)
//!   - `alloc F` calling `println` (io forbidden inside alloc)
//!
//! Interaction with other gates:
//!   - `partial pure F` composes — partial exempts totality; pure still
//!     enforces effect purity on the body.
//!   - `A pure F` would compose with async; deferred to a follow-up
//!     task since async + effect is largely untested in practice.

use super::helpers::*;

// ====================================================================
// Positive cases — declared-effect functions that should compile
// ====================================================================

#[test]
fn e2e_phase4c3_pure_arithmetic_compiles() {
    // The canonical pure function: no calls at all, just arithmetic.
    // This is the minimal smoke test for the lexer+parser+AST+TC
    // pipeline on the `pure` prefix.
    let source = r#"
pure F add(a: i64, b: i64) -> i64 = a + b

F main() -> i64 = add(2, 3)
"#;
    assert_exit_code(source, 5);
}

#[test]
fn e2e_phase4c3_pure_recursion_compiles() {
    // `pure F` with self-recursion via `@`. Exercises the call-graph
    // fixed point on a trivial cycle (fn -> self) — the worklist must
    // not get stuck.
    let source = r#"
pure F fact(n: i64) -> i64 {
    I n <= 1 { R 1 }
    R n * fact(n - 1)
}

F main() -> i64 = fact(5)
"#;
    assert_exit_code(source, 120);
}

#[test]
fn e2e_phase4c3_pure_calls_pure_compiles() {
    // pure ⊆ pure: one pure function calling another pure function.
    // Exercises the transitive propagation with a real edge in the
    // call graph (not just self-recursion).
    let source = r#"
pure F sq(x: i64) -> i64 = x * x

pure F sum_of_squares(a: i64, b: i64) -> i64 = sq(a) + sq(b)

F main() -> i64 = sum_of_squares(3, 4)
"#;
    assert_exit_code(source, 25);
}

#[test]
fn e2e_phase4c3_io_with_println_compiles() {
    // `io F` is allowed to call IO builtins. Critical regression
    // guard for the subtype rule `pure ⊆ io`.
    let source = r#"
io F greet() -> i64 {
    println("hi")
    R 0
}

F main() -> i64 = greet()
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase4c3_io_calling_pure_compiles() {
    // An io function may call a pure callee — classical subtype.
    let source = r#"
pure F double(x: i64) -> i64 = x * 2

io F log_doubled(x: i64) -> i64 {
    println("logging")
    R double(x)
}

F main() -> i64 = log_doubled(7)
"#;
    assert_exit_code(source, 14);
}

#[test]
fn e2e_phase4c3_alloc_with_malloc_compiles() {
    // `alloc F` is allowed to call allocation builtins.
    //
    // Note: we call `malloc` via `N F malloc(size: i64) -> i64` (the
    // extern declaration style already used elsewhere in the E2E
    // suite). The test is just about type checking reaching the
    // effect-purity gate; the returned pointer is cast to i64 and
    // not actually dereferenced.
    let source = r#"
N F malloc(size: i64) -> i64
N F free(ptr: i64) -> i64

alloc F make_buf(n: i64) -> i64 {
    ptr := malloc(n)
    free(ptr)
    R n
}

F main() -> i64 = make_buf(64)
"#;
    assert_exit_code(source, 64);
}

#[test]
fn e2e_phase4c3_untagged_caller_with_println_compiles() {
    // A function without any effect prefix is "inferred" and is not
    // gated directly — it freely mixes IO and arithmetic. This is
    // the regression guard that proves non-declared code keeps
    // compiling unchanged.
    let source = r#"
F maybe_log() -> i64 {
    println("just a log")
    R 42
}

F main() -> i64 = maybe_log()
"#;
    assert_exit_code(source, 42);
}

// Note: a `#[pure]` attribute backcompat test was intentionally NOT
// added. The legacy attribute path in `EffectInferrer::get_declared_effects`
// has zero production call sites today (see iter 11 recon notes); it
// exists only for source-level backwards compatibility. Furthermore
// the attribute parser uses `parse_ident()` for the attribute name,
// which rejects `Token::Pure` (a reserved keyword) outright. Wiring
// `#[pure]` through the attribute grammar is a separate concern
// orthogonal to Phase 4c.3 and is explicitly out of scope.

// ====================================================================
// Negative cases — declared-effect functions that should be rejected
// ====================================================================

#[test]
fn e2e_phase4c3_pure_calling_println_is_rejected() {
    // The core negative: a pure function directly calls an IO builtin.
    // The effect-purity gate must reject it.
    let source = r#"
pure F bad() -> i64 {
    println("hi")
    R 0
}

F main() -> i64 = bad()
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase4c3_pure_calling_malloc_is_rejected() {
    // `pure F` cannot allocate. Same shape as the println case but
    // exercising the Alloc flag rather than the IO flag.
    let source = r#"
N F malloc(size: i64) -> i64

pure F oops() -> i64 {
    R malloc(16)
}

F main() -> i64 = oops()
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase4c3_pure_transitive_io_is_rejected() {
    // Transitive propagation: a pure function calls an untagged
    // helper which itself calls `println`. The worklist must flag the
    // pure caller even though the pure body syntactically looks
    // effect-free.
    let source = r#"
F helper() -> i64 {
    println("sneaky")
    R 1
}

pure F top() -> i64 = helper()

F main() -> i64 = top()
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase4c3_io_calling_malloc_is_rejected() {
    // `io F` may call IO builtins but may NOT allocate — io and
    // alloc are disjoint in the subtype lattice.
    let source = r#"
N F malloc(size: i64) -> i64

io F oops() -> i64 {
    println("allocating now")
    R malloc(32)
}

F main() -> i64 = oops()
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase4c3_alloc_calling_println_is_rejected() {
    // `alloc F` may allocate but may NOT perform IO — symmetric
    // companion of the io-calling-malloc case.
    let source = r#"
N F malloc(size: i64) -> i64

alloc F oops() -> i64 {
    println("side effect")
    R malloc(8)
}

F main() -> i64 = oops()
"#;
    assert_compile_error(source);
}

// ====================================================================
// Interaction with other modifiers
// ====================================================================

#[test]
fn e2e_phase4c3_partial_pure_composes_compiles() {
    // `partial pure F` — partial disables the totality gate,
    // pure still enforces effect purity. The body contains an
    // `assert` (which the totality gate would normally reject in a
    // non-partial function) plus pure arithmetic. The effect-purity
    // gate should see no IO/Alloc and let it through.
    //
    // Note: `main` is also marked `partial` because totality
    // propagation treats any call to a `partial`-marked function
    // as a transitive panic source — a non-partial `main` calling
    // `checked_sqrt` would fail the totality gate, which is the
    // correct Phase 4c.2 behaviour but orthogonal to what this test
    // wants to exercise (Phase 4c.3 purity on a partial function).
    let source = r#"
partial pure F checked_sqrt(x: i64) -> i64 {
    assert(x >= 0)
    R x * x
}

partial F main() -> i64 = checked_sqrt(5)
"#;
    assert_exit_code(source, 25);
}

#[test]
fn e2e_phase4c3_pub_pure_compiles() {
    // `P pure F` — the canonical public-pure shape. Tests that
    // `pub` followed by the effect prefix parses correctly.
    let source = r#"
P pure F triple(x: i64) -> i64 = x * 3

F main() -> i64 = triple(7)
"#;
    assert_exit_code(source, 21);
}

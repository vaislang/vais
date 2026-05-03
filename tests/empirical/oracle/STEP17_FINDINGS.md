# Step 17 (I-6 MIR oracle expansion) findings

This file records empirical findings from Step 17 fuzz_mir_native_diff
work. Mirrors STEP7_FINDINGS / STEP11_FINDINGS structure.

## F-MIR-01 — vais-mir interpreter panics on Arbitrary-derived corrupted input

Discovered during Step 17 stage 1 (commit <pending>). Wiring Path A
of fuzz_mir_native_diff to real `interpret_function` — within seconds
libFuzzer drove the interpreter to a `deadly signal` exit (libFuzzer's
recipe for "the target panicked or aborted").

Crash artifacts written to `compiler/fuzz/crash-*` (intentionally
.gitignored — fuzz output is not committed). Reproducer pattern:
arbitrary high-byte sequences (e.g. `0xf5,0xf5,0xf5,...`) parsed and
type-checked as Vais surface, then lowered, then interpreted. Some
specific lowered MIR triggers a panic in
`vais_mir::interpreter::Interpreter::call` rather than returning
`MirInterpretError`.

**Implications**:

1. The interpreter's contract should be: `Result<MirValue,
   MirInterpretError>` covers ALL execution failures. Today some
   failure paths panic instead. That itself is a Step 17 deliverable
   — the oracle must be panic-free on any well-typed-and-lowered MIR
   so that a real diff finding is distinguishable from an oracle
   crash.

2. The Path A wiring in `fuzz_mir_native_diff.rs` short-circuits any
   `Err(_)` return, which would have classified these inputs as
   "InputInvalid" (out of scope for diff). But a panic escapes that
   match arm and reaches libFuzzer, which is interpreted as a finding
   even though it is an oracle bug, not a user-program bug.

**Stage-2 follow-up tasks**:
- Audit `vais_mir::interpreter` for `unwrap` / `expect` / `panic!`
  call sites; convert each to `MirInterpretError` returns.
- Add a `compiler/tests/integration/mir_interpreter_panic_free.rs`
  test that runs the same Arbitrary input space against
  `interpret_function` and asserts no panic — only Result variants.
- Once panic-free, re-enable Path A in fuzz mode and discover real
  MIR/native diffs (after Path B lands too).

## F-MIR-02 — `cargo test --bin fuzz_mir_native_diff` triggers libFuzzer

The libfuzzer-sys `fuzz_target!` macro expands to a `main` that
delegates to libFuzzer at startup. cargo test's per-bin runner calls
that main, so cargo test enters fuzz mode rather than running `#[test]`
functions.

Practical result: in-binary unit tests (`#[cfg(test)] mod tests
{ #[test] fn ... }`) inside a fuzz target are dead code at the cargo
test level. Earlier drafts of this fuzz target carried such tests;
they were removed in commit <pending> after this finding.

**Stage-2 alternatives**:
- Move shared logic (`compare_paths`, `RunOutput`, `PathOutcome`) into
  a tiny library crate (e.g. `vais-fuzz-core`) that the fuzz binary
  imports. Unit tests live in the library crate, run under cargo
  test normally. Fuzz binary stays a thin shim.
- Or: rely on `cargo fuzz run -- -runs=1` with seed corpus for
  smoke verification.

The latter is simpler today; the former is the right structural
fix for stage 2+ when more shared logic accumulates.

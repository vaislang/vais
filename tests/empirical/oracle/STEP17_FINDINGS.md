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

---

## F-MIR-01 RESOLVED — 2026-05-04 re-fuzz shows no panic

Stage 1 commit 309b3f47 removed the only `.unwrap()` in vais-mir
(lower.rs:892 enum_pattern_discriminant). At the time of that commit
we conservatively assumed remaining panics would surface from
upstream parser/types crates.

Re-running fuzz_mir_native_diff for 60 seconds (~75K iterations) on
2026-05-04 found **zero panics**. The source-level audit also shows
the panic surface in vais-mir is now empty (only test-only panic!
patterns remain). vais-types contains exactly 1 `unreachable!` in
checker_module/mod.rs:234 with a guard "ownership_errors was verified
non-empty"; that path is invariant-safe.

Conclusion: the oracle is panic-free under the current default-mode
strict A4 surface. Stage 2 dependency on F-MIR-01 is satisfied.

Remaining stage 2+ work (separate from panic-freedom):
- Wire Path B (native LLVM/clang execution) for actual differential
  finding capability (currently both paths short-circuit on
  parse/type errors, so the oracle never reports a diff).
- Refactor compare_paths into a vais-fuzz-core lib so unit tests are
  reachable under cargo test (per F-MIR-02).

---

## F-MIR-02 RESOLVED — 2026-05-04 lib refactor LANDED

Refactored `compiler/fuzz/` into a Rust crate with both a library
target and a thin binary target. Layout:

  fuzz/Cargo.toml — adds `[lib]` (implicitly via src/lib.rs) plus the
                     existing `[[bin]]` entries.
  fuzz/src/lib.rs — moved from fuzz_targets/fuzz_mir_native_diff.rs.
                     Contains VaisProgram + compare_paths + run_mir_path
                     + run_native_path + RunOutput + PathOutcome with
                     `pub` visibility. Plus unit tests at the bottom.
  fuzz/fuzz_targets/fuzz_mir_native_diff.rs — 20-line shim that
                     delegates to vais_fuzz::compare_paths.

`cargo test --lib -p vais-fuzz` now runs 4 #[test] functions (basic
compare_paths smoke + RunOutput equality). All pass. The original
F-MIR-02 problem (libFuzzer's main hijacking cargo test) is gone
because the lib has no main.

Verification

  cargo test --lib -p vais-fuzz:
    test tests::run_output_eq_basic ... ok
    test tests::compare_paths_invalid_source_does_not_panic ... ok
    test tests::compare_paths_empty_does_not_panic ... ok
    test tests::compare_paths_simple_main_does_not_panic ... ok
    4 passed; 0 failed.

  cargo check --bin fuzz_mir_native_diff: success.
  bash compiler/scripts/check-integrity.sh: INTEGRITY OK.

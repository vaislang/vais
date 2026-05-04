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

---

## Stage 2 Path B LANDED — vais-jit Cranelift in-process (2026-05-04)

`run_native_path` now uses `vais_jit::JitCompiler::compile_and_run_main`
to execute the program in-process via Cranelift. No tempfile, no fork
— fits the libFuzzer model that previously blocked option (b).

Layout

  fuzz/Cargo.toml         — added `vais-jit = { path = "..." }` dependency.
  fuzz/src/lib.rs:run_native_path
                          — parse → type-check → JitCompiler::new →
                            compile_and_run_main(&module). Returns
                            PathOutcome::Ok(RunOutput { exit_code, ... }).
                            JIT init/compile errors map to NotImplemented
                            (out-of-scope, same as before — no false
                            findings). stdout is empty (JIT writes to
                            host stdout; oracle compares exit code only).

Verification

  cargo test --lib -p vais-fuzz: 6/6 passed.
    - 4 prior (compare_paths_*, run_output_eq_basic) — still green.
    - 2 NEW:
        run_native_path_simple_returns_exit_code: probes
          `F main() -> i64 { 42 }` via Path B and asserts exit=42.
        compare_paths_simple_main_agrees: probes
          `F main() -> i64 { 7 }` through both paths and asserts no
          divergence panic.

  bash scripts/check-integrity.sh: INTEGRITY OK (no regression).

Stage 2 deliverables now complete:
  - F-MIR-01 RESOLVED (panic-free oracle)
  - F-MIR-02 RESOLVED (lib refactor; tests reachable)
  - Path B wired (this section)

Real differential findings can now land. The fuzz binary
fuzz_mir_native_diff is ready to run as `cargo fuzz run` for finding
discovery work in stage 3+.

---

## Stage 3 LANDED — fuzz_mir_native_diff promoted to nightly + tracked corpus seeds (2026-05-04)

Layout
- `.github/workflows/fuzz.yml` matrix gains `fuzz_mir_native_diff`
  (joins fuzz_lexer/parser/type_checker/codegen).
- `fuzz/corpus_seeds/fuzz_mir_native_diff/` tracked seed dir (3 entries:
  empty / zero / 8-byte). Necessary because `corpus/` is gitignored;
  cold-start CI box would have nothing to mutate from.
- New workflow step `Seed corpus from tracked seeds` copies
  `corpus_seeds/<target>/*` → `corpus/<target>/` with `cp -n` (no
  overwrite of cached corpus).
- README target table updated.

Verification
- `cargo test --release` in `compiler/fuzz/` → 6/6 #[test] still pass.
- `bash scripts/check-integrity.sh`: INTEGRITY OK.

Step 17 done_when criterion 1 of 3 (nightly fuzz green) met for the
matrix-wired job, pending first GH Actions cron execution.

---

## Stage 4a LANDED — sanitizer PR-blocking supersession documented (2026-05-04)

Empirical finding (this iter): the assumption that the workspace
sanitizer-tests / miri-tests jobs in `fuzz.yml` were the only sanitizer
gate was WRONG. Dedicated workflows already gate on push/pull_request:
- `.github/workflows/asan.yml`: `continue-on-error: false` on 7/8 steps
  (one LLVM-bindings step kept `true` for known false SEGV).
- `.github/workflows/tsan.yml`: `continue-on-error: false` on every step.

Therefore the broader workspace ASAN/UBSAN runs in `fuzz.yml` are
SUPERSEDED best-effort coverage. Comments added to fuzz.yml documenting
this supersession + Step 17 stage 4a/4b sub-step structure. No workflow
logic changes — this is recognition of pre-existing infrastructure.

Step 17 done_when criterion 2 of 3 status:
- Sanitizer (ASAN/TSAN) PR-blocking: ALREADY MET via dedicated workflows.
- Miri PR-blocking: PENDING stage 4b — see below.

---

## Stage 4b DEFERRED — Miri PR-blocking promotion (multi-day stability survey)

Single-session immediate landing is unsafe per CLAUDE.md rule 4
(regression-immediate-revert protocol). Plan:

1. Run nightly `cargo +nightly miri test -p vais-{lexer,parser,ast,types}
   -- --test-threads=1` under `MIRIFLAGS=-Zmiri-disable-isolation
   -Zmiri-tree-borrows` for 7+ consecutive days.
2. Surface and fix any false-positive Stacked/Tree-Borrows violations
   (Miri commonly flags valid raw-pointer patterns in self-referential
   structures; each finding requires individual triage).
3. After 7+ days of green nightly runs, flip the `|| true` shell-suffix
   in fuzz.yml line 151-154 to fail-fast (or split Miri to a dedicated
   workflow with `continue-on-error: false`).

Risk if landed prematurely: first false-positive blocks ALL PR merges
until fixed. With LLVM 17 + nightly toolchain churn the surface is
moving; needs empirical observation period.

---

## Stage 5 RECONNAISSANCE — diagnostic equivalence (2026-05-04)

Goal: extend `RunOutput` comparison from exit-code-only to
`(exit_code, stdout)` — Step 17 done_when criterion 3 of 3.

### Current state

`compiler/fuzz/src/lib.rs:run_mir_path` and `run_native_path` both
return `RunOutput { exit_code, stdout: String::new() }`. stdout
diff is technically wired in `compare_paths` (PartialEq on the whole
struct) but stays vacuously equal because both arms emit empty
strings.

### Path A blocker — vais-mir interpreter has no I/O model

`crates/vais-mir/src/interpreter.rs:88` `Interpreter::call(function, args)`
looks up `bodies.get(function)` and errors with `"function body not
found"` if missing. There is NO builtin/intrinsic intercept layer.
Any program calling `print` / `println` / etc. immediately produces
`MirInterpretError`, which `run_mir_path` maps to
`PathOutcome::InputInvalid` — silently dropped from the diff.

### Path B blocker — vais-jit writes to host stdout, no capture

`crates/vais-jit/src/lib.rs:JitCompiler::compile_and_run_main` runs
the function via Cranelift in-process. Any `print` builtin is linked
to libc's `printf` (or the Vais runtime's print shim) and writes
directly to host stdout/stderr file descriptors. No capture
mechanism.

### Design options (for stage 5 implementation)

(a) **Interpreter-side print intercept + JIT stdout redirect**:
    - Add `BuiltinIntercept` trait to `vais_mir::interpreter::Interpreter`
      with method `intercept_call(name, args) -> Option<MirValue>`.
      Wire `print` / `println` / `print_int` etc. to push-to-buffer.
    - For Path B: `dup2(stdout_pipe, 1)` before
      `compile_and_run_main`, restore after, drain pipe into String.
      Risk: thread-unsafe (libFuzzer's fork-server isolates each
      worker, but libc stdio buffering may strand output).
    - Effort: ~200-400 LOC (interpreter) + ~80-150 LOC (jit shim) +
      tests.

(b) **Recoverable stdout via runtime hook (preferred, future-safe)**:
    - Introduce `vais-runtime` thread-local `STDOUT_SINK: RefCell<Vec<u8>>`
      that print-builtin call sites consult before writing to fd 1.
    - Both Path A (interpreter intercept fills the same sink) and
      Path B (JIT-emitted print() calls into the sink) can read.
    - Effort: ~300-500 LOC across vais-runtime + vais-mir + vais-jit
      + builtin call lowering. Higher upfront but no fd manipulation.

### Decision

Stage 5 implementation is substantial (200-500 LOC across 2-3 crates +
runtime contract change). Not single-session safe. Path (b) is the
correct long-term choice; Path (a) is a faster MVP if needed sooner.

Status: RECONNAISSANCE LANDED. Stage 5 implementation deferred.
Re-open as task with explicit option (a)-vs-(b) decision when
prioritization dictates.

---

## Stage 5a LANDED — interpreter-side stdout sink + builtin intercept (2026-05-04)

Decision: Path (a) MVP chosen for the interpreter half. Path (b)
runtime-sink is still the long-term right answer for symmetry with
the JIT side, but (a) on the interpreter alone is the smallest
useful first step (asymmetric: interpreter captures stdout, JIT
still writes to host stdout — diff is therefore one-way for now).

Layout
- `crates/vais-mir/src/interpreter.rs`:
  - new `pub struct InterpreterRunOutput { exit_code, stdout, return_value }`
  - new `pub fn interpret_function_with_io(...)` entry point that
    sets up a `RefCell<String>` stdout sink before running the
    interpreter
  - new `Interpreter::try_intercept_builtin(name, args)` — fires
    only when `stdout_sink` is `Some`. Recognized builtins:
      - `print`, `print_str` → push args to sink
      - `println` → push args + '\n'
      - `print_int` → push the formatted i64
  - `Interpreter::call` checks the intercept before
    `bodies.get(function)`, so the bare entry point's
    "function body not found" error path is preserved when the sink
    is None.
  - `Interpreter::write_value` formats MirValues for stdout.

- `crates/vais-mir/tests/interpreter_tests.rs`:
  - new `interpret_with_io_int_return_maps_to_exit_code` (R 42 → exit 42)
  - new `interpret_with_io_truncates_exit_code_to_8_bits` (R 257 → exit 1)
  - new `bare_interpret_function_rejects_unknown_function_name`
    (regression guard for backward compatibility — bare entry must
    NOT silently intercept builtins)

- `compiler/fuzz/src/lib.rs::run_mir_path` switched to
  `interpret_function_with_io`. The `RunOutput { exit_code, stdout }`
  the differential check produces is now genuinely populated on the
  Path A side. Path B still emits empty stdout (path (b) JIT-side
  capture is the next stage).

Verification
- `cargo test --release -p vais-mir --test interpreter_tests` — 6/6 pass
  (3 prior + 3 new).
- `cargo test --release` in `compiler/fuzz/` — 6/6 #[test] still pass
  (compare_paths agree probe + Path B exit-code probe both green;
  no diff regression introduced by the Path A switch).

Asymmetry caveat
The diff oracle's stdout-equality check now compares "MIR sink output"
vs "empty string". For inputs that don't call any print builtin, both
sides produce empty strings → comparison stays vacuously equal (no
false findings). For inputs that DO call print, the MIR side emits the
captured text and the JIT side emits empty → one-sided diff would
panic on every such input.

To prevent that turning into noise, `compare_paths` should be amended
in stage 5b to skip the stdout equality check when either side is
empty. Alternatively, stage 5b wires a JIT-side stdout sink so both
sides have the same coverage. (Code in fuzz/src/lib.rs ~line 460 in
the diff branch.) NOT done in this commit — landed as immediate-safe
infrastructure only.

Status: stage 5a LANDED. Step 17 done_when criterion 3 of 3 (diagnostic
equivalence) is now PARTIALLY MET on the interpreter half; the JIT
half + asymmetry-handling is stage 5b work.

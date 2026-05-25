# Core Diagnostic Contract

## Purpose

Diagnostics are part of the Vais Core language surface. For AI-assisted
development, an invalid Core program must fail with a stable, machine-readable
error code instead of requiring an agent to guess from prose.

This contract is intentionally narrow. It covers the Core certification
fixtures listed in `tests/core/manifest.tsv`. Broader CLI presentation,
localization, recovery notes, and ecosystem diagnostics remain outside this
contract until they are promoted by fixtures.

## W1-C Diagnostic And CLI Readiness Contract

Status: T-506 selected W1-C as a bounded diagnostic/CLI readiness queue.

W1-C extends the diagnostic contract only where current source already exposes
a stable compiler-facing boundary or where a focused fixture can prove one. The
queue does not promote broad localization, editor/LSP behavior, package
registry behavior, release behavior, or production crash recovery.

Promoted or eligible W1-C surfaces:

- `vaisc check` parse and type failures that flow through
  `error_formatter::format_parse_error()` or
  `error_formatter::format_type_error()` and print a stable `error[CODE]`
  header before exiting nonzero.
- `vaisc build` type and codegen failures that flow through
  `format_type_error()`, `format_codegen_error()`, or
  `format_spanned_codegen_error()` before the top-level CLI exits nonzero.
- Explicit unsupported-backend or unsupported-mode CLI failures that are
  deterministic, bounded to one command path, and covered by focused evidence.
- The A2-03 invalid dyn trait construction/cast follow-up selected by T-507:
  the promoted target is a product-readable rejection before LLVM/runtime crash,
  not broader dyn trait semantic widening.

W1-C nonclaims:

- Program runtime exits from `vaisc run` remain user-program behavior, not
  compiler diagnostics, unless the failure occurs during compiler/build setup.
- Panic hook output and optional crash-report files remain internal compiler
  error handling, not stable user diagnostics.
- Timeout text, color styling, localized prose, help wording, shell-specific
  stderr ordering, editor/LSP diagnostics, package registry commands, release
  publishing, deployment, and production observability remain outside W1-C.
- No rejected feature becomes supported merely because its diagnostic wording is
  refreshed.

W1-C acceptance rule: every promoted diagnostic/CLI slice must name the command
path, expected exit class, stable diagnostic code or bounded error string, and
the exact focused gate that proves the behavior. If a failure can only be seen
as a panic, LLVM verifier error, linker failure, or unstructured runtime crash,
it must be fixed or explicitly carried as a blocker before W1-C can close.

### T-511 Runtime Crash-To-Diagnostic Boundary Audit

T-511 audits the current crash-like language/compiler surfaces and keeps W1-C
bounded to failures that already have focused evidence.

Promoted or deterministic W1-C surfaces:

- A2-03 invalid i64-as-`dyn Greet` now fails during `vaisc check` with
  `error[E001]` before build or run. The focused gate is
  `bash scripts/check-empirical.sh A2`, and the direct negative command reports
  expected `dyn Greet`, found `i64`.
- A4-06 integer truthy and A4-15 escape closure remain rejected at
  `vaisc check`; `crates/vaisc/tests/error_snapshot_tests.rs` pins the current
  diagnostic snapshots for those rejected surfaces.
- A4-08 Vec-to-reference permissiveness and A4-09 lifetime-ref erasure are
  historical runtime/linker failures in their metadata, but their current
  empirical runners reject at `vaisc check` with `E001`. W1-C may cite that
  current check-fails behavior; it does not promote the historical runtime
  crash or linker forms as acceptable user diagnostics.
- Parser and type diagnostic formatters are covered by focused unit tests that
  require stable `error[CODE]` headers and source filenames.

Deferred or nonclaim surfaces:

- Program exits, user program panics, assertions, unwraps, and dependent-check
  failures reached through `vaisc run` remain user-program behavior unless the
  compiler/build setup fails before execution.
- Panic-hook output, optional compiler crash-report files, Rust backtraces,
  LLVM verifier failures, clang/linker diagnostics, process timeout text, and
  shell-specific stderr ordering are not stable W1-C diagnostics.
- Historical empirical metadata may preserve older runtime-crash or build-fail
  provenance for A4 surfaces; W1-C only claims the current focused runner
  behavior named above.
- Runtime recovery, production crash handling, deployment observability, editor
  diagnostics, and package registry behavior remain outside this contract.

T-512 owns the next boundary: the user-facing `vaisc check`, `vaisc build`,
and `vaisc run` command envelopes and exit classes.

### T-512 CLI Check/Build/Run Failure Envelopes

T-512 pins selected CLI envelopes through
`crates/vaisc/tests/error_message_tests.rs`.

Promoted envelopes:

- `vaisc --no-update-check --timeout 0 check <invalid.vais>` exits 1 for a
  type error, writes a top-level `error: error[E001] Type mismatch` envelope to
  stderr, reports the expected/found types, and does not print the check
  success message.
- `vaisc --no-update-check --timeout 0 build <invalid.vais>` exits 1 for a
  type error, writes an `error[E001] Type mismatch` diagnostic to stderr, and
  ends with the bounded summary `error: 1 type error(s) found`.
- `vaisc --no-update-check --timeout 0 run <invalid.vais>` uses the same
  pre-execution compiler/build failure envelope as `build`: exit 1, stable
  `error[E001]` diagnostic, and bounded type-error summary.
- `vaisc --no-update-check --timeout 0 run <valid-program.vais>` propagates a
  nonzero user program exit code without wrapping it as a compiler diagnostic.
  The focused smoke uses exit 7 and asserts stderr does not contain `error:`.

Nonclaims:

- The tests do not promote full CLI prose, color/style, stdout ordering,
  localized text, timeout wording, JIT fallback wording, linker/clang wording,
  package registry commands, release/publish commands, deployment behavior, or
  production observability.
- Nonzero exits from a successfully built user program remain user-program
  behavior, not a compiler diagnostic contract.

### T-514 Docs/Help/Certification Sync

T-514 syncs W1-C documentation and generated certification inventory claims.

- `EXCLUDED_FEATURES.md` and the generated Master Plan section now display
  current A4 status/evidence beside historical discovery classes. This keeps
  A4-08 and A4-09 readable as current check-time `E001` rejections while
  preserving their original late-codegen/linker provenance.
- CLI help flags exist for `--no-update-check`, `--timeout`, and
  `--report-crash`, but help prose is not a stable diagnostic contract.
- The W1-C promoted command paths remain only the focused `check`, `build`, and
  pre-execution `run` envelopes named above plus the A2-03 empirical smoke.
- Package registry, release/publish, deploy, editor/LSP, production
  observability, panic/crash-report, timeout, and linker/clang output remain
  nonclaims unless a later focused fixture promotes them.

## Current Gate

`crates/vaisc/tests/core_certification.rs` enforces the contract through
`core_certification_manifest`:

- every negative Core fixture must declare an expected diagnostic code
- the code must use a stable family plus three digits, such as `P001` or `E002`
- `check` failures may use parser (`P`) or type (`E`) codes
- `codegen` and `run` failures may additionally use codegen (`C`) codes
- the compiler output must contain a structured diagnostic header
  `error[CODE]`

The manifest still records the expected stage and description. The diagnostic
header check makes the error code a real user-facing contract instead of a loose
substring that could be satisfied by unrelated text.

## Current Core Negative Fixtures

The active Core v0 manifest currently has these negative diagnostic classes:

| Fixture | Code | Meaning |
|---|---:|---|
| `negative/parse/missing_brace.vais` | `P001` | parse error |
| `negative/types/return_bool_as_i64.vais` | `E001` | type mismatch |
| `negative/types/undefined_var.vais` | `E002` | undefined variable |
| `negative/types/implicit_str_i64.vais` | `E001` | invalid implicit coercion |
| `negative/control/non_predicate_condition.vais` | `E001` | invalid condition type |
| `negative/control/branch_mismatch.vais` | `E001` | branch type mismatch |
| `negative/structs/unknown_field.vais` | `E002` | unknown field |

## Promotion Rule

When adding a new negative Core fixture:

1. add the fixture to `tests/core/manifest.tsv`,
2. declare the failure stage and expected diagnostic code,
3. make sure the compiler emits `error[CODE]` at that stage,
4. update this document if the new case adds a diagnostic class.

Do not promote a negative fixture whose failure only appears as an LLVM verifier
error, linker error, panic, or unstructured text dump. Those are compiler bugs
or non-Core surfaces until a stable diagnostic is added.

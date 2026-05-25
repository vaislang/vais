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

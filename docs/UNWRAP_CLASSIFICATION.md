# UNWRAP_CLASSIFICATION

Production `.unwrap()` audit for the certified Core compiler crates.

- Last verified: 2026-05-05 (re-audit during Category C → expect() conversion;
  Category C count corrected 37 → 23 after per-site re-classification of 14
  sites that turned out to live inside `#[cfg(test)]` blocks).
- Scope: `crates/{vais-codegen,vais-types,vais-parser,vais-lexer,vais-mir,vaisc}/src/**`
- Tests, benches, doc-tests outside `src/` are not in scope (panic on test
  failure is acceptable).

## Categories

- **A — Test/Doc**: site lives inside `#[cfg(test)] mod tests`, a `#[test]`
  function, or a doc-test. Panic on failure is the contract.
- **B — Infallible by construction**: an immediately preceding statement
  proves the `Some`/`Ok` (e.g. `vec.first().unwrap()` after `assert!(!vec.is_empty())`).
- **C — Infallible by invariant**: holds by a wider invariant the compiler
  cannot prove locally (e.g. an LLVM builder is in a known state, an AST node
  field is always populated by the parser). Low risk; an `expect("...invariant...")`
  is a reasonable hardening but is not a regression today.
- **D — User-input reachable**: a value derived from a user-controlled source
  (source code being compiled, CLI args, package manifest, registry payload,
  on-disk cache) can drive the panic. This is the only category that is a
  panic-as-DoS risk and the only category that requires a follow-up fix.

## Result

| Category | Sites | Share |
|---|---|---|
| A — Test/Doc | 301 | 92.9% |
| B — Infallible by construction | 0 | 0.0% |
| C — Infallible by invariant | 23 | 7.1% |
| **D — User-input reachable** | **0** | **0.0%** |
| Total | 324 | 100% |

The 2026-05-05 re-audit reclassified 14 sites that were originally counted
under Category C (in files like `index_access.rs`, `method_returns.rs`,
`bounds_check_elim.rs`, `alias_analysis.rs`, `data_layout.rs`,
`ir_passes/mod.rs`, `inlining.rs`) as Category A, because the unwrap sites
all live inside `#[cfg(test)] mod tests` blocks. The total of 324 is
unchanged.

`B = 0` is a definitional consequence of the spot-checks: every site that
could have qualified as B was already inside a `#[cfg(test)]` block (so it
was counted as A).

## Category C distribution

23 sites, all inside the certified compiler invariants. All sites converted
to `.expect("invariant: ...")` on 2026-05-05.

| File | Sites | Invariant |
|---|---|---|
| `crates/vais-codegen/src/inkwell/gen_stmt.rs` | 4 | builder positioned in basic block before deferred-free / alloc-cleanup codegen |
| `crates/vais-codegen/src/inkwell/gen_match.rs` | 2 | builder positioned before/after variant pattern branch |
| `crates/vais-codegen/src/inkwell/gen_aggregate.rs` | 4 | builder/function/entry-block valid during struct-malloc setup; malloc returns pointer |
| `crates/vais-codegen/src/inkwell/gen_expr/binary.rs` | 4 | builder/function/entry-block valid during string-concat alloca-slot setup |
| `crates/vais-codegen/src/inkwell/gen_expr/call.rs` | 5 | malloc fn available after or_else insert; malloc returns pointer value |
| `crates/vais-codegen/src/expr_helpers_call/method_call.rs` | 2 | len==1 guard; contains_key checked before get |
| `crates/vais-codegen/src/control_flow/pattern.rs` | 1 | is_some() confirmed in enclosing else-if guard |
| `crates/vais-types/src/checker_expr/control_flow.rs` | 1 | is_empty() guard above |

## Category D — none

The audit found **no `.unwrap()` on a value that user input can directly
drive to panic** in the in-scope crates.

The high-traffic candidates were spot-checked:

- `crates/vaisc/src/registry/version.rs` (57 unwraps) — all under
  `#[cfg(test)] mod tests` from line 493 onward. Production parsing returns
  `RegistryResult<Version>` with explicit `RegistryError::InvalidVersion`.
- `crates/vaisc/src/commands/build/cache.rs` (35 unwraps) — all under
  `#[cfg(test)] mod tests` from line 301 onward. Production cache I/O
  propagates errors through `?`.
- `crates/vaisc/src/incremental/graph.rs` (19 unwraps) — all under
  `#[cfg(test)] mod tests` from line 347 onward.
- `crates/vais-parser/src/ffi.rs` (20 unwraps) — all in doc-test snippets
  inside `///` comments illustrating the FFI API.

User-controllable inputs (source files, CLI args, manifest TOML, registry
payloads, on-disk caches) all flow through `Result`-returning APIs in the
production paths.

## Reconciliation with prior audits

`docs/SECURITY_AUDIT.md` reported "1,337 `.unwrap()` instances" as a single
number. That figure was a workspace-wide grep including tests, benches,
bindings (`vais-python`/`vais-node`), DAP, registry server, tutorial, and
similar non-Core crates. This document narrows scope to the certified Core
compiler `src/` only and classifies each site, which is why the count drops
to 324 and the user-input-reachable count is 0.

The earlier audit's recommendation (ban `.unwrap()` in production paths via
`clippy::unwrap_used = "deny"`) remains a defensible future hardening, but
the absence of any Category D site means it would be enforcement of an
already-met invariant rather than a fix for an open vulnerability.

## Action items

- None blocking. The certified Core has no Category D unwrap.
- ~~Optional, non-blocking: convert the 23 Category C sites to
  `expect("invariant: ...")` so the panic message documents the invariant.~~
  **DONE 2026-05-05** (all 23 sites converted; INTEGRITY OK preserved).
- Optional, non-blocking: add `#![warn(clippy::unwrap_used)]` to the in-scope
  crates so any new Category D site is rejected at review time. Scope this
  separately from the existing 287 Category A sites (which would need
  `#[allow(clippy::unwrap_used)]` on the test modules).

## Method

1. `find compiler/crates/{vais-codegen,vais-types,vais-parser,vais-lexer,vais-mir,vaisc}/src -name '*.rs' | xargs grep -n '\.unwrap()'`
   → 324 sites.
2. For each site, read ±5 lines and classify.
3. Spot-check the four highest-count files by hand to confirm the
   `#[cfg(test)] mod tests` boundaries: line 493 (`version.rs`), 301
   (`cache.rs`), 347 (`graph.rs`); `ffi.rs` examples are all in `///`
   doc-tests.
4. No code modified.

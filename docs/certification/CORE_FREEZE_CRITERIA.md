# Core Freeze Criteria

## Purpose

This document defines when the Vais Core compiler may be treated as frozen for
downstream product work. Freeze does not mean that every Vais feature is
complete. It means the currently certified Core language and compiler contract
is stable enough that `vaisdb`, `vais-server`, and `vais-web` work should stop
driving compiler fixes unless a promoted Core gate fails.

The freeze decision is a gate decision, not a confidence statement.
The active decision record is `CORE_FREEZE_DECISION.md`.

## Freeze Scope

In scope:

- the Core v0 source surface in `docs/certification/VAIS_CORE_V0.md`
- strict MIR fixtures and semantic-oracle rules in
  `docs/certification/MIR_CONTRACT.md`
- negative diagnostic rules in `docs/certification/DIAGNOSTIC_CONTRACT.md`
- exclusion and deferred-surface rules in
  `docs/certification/EXCLUDED_FEATURES.md` and
  `docs/certification/IGNORED_SURFACE_AUDIT.md`
- the canonical aggregate gate in `scripts/check-integrity.sh`

Out of scope:

- product-level completion for `vaisdb`, `vais-server`, or `vais-web`
- experimental crates outside the default trust path
- broad language features that are documented as Deferred or Experimental
- old roadmap counts or archived phase plans

## Required Freeze Evidence

All of the following must be true in the same working batch before declaring a
Core freeze:

| Gate | Required evidence |
|---|---|
| Full Rust-hosted compiler suite | `cargo test --release` exits with code `0` |
| Core certification | `bash scripts/core-certify.sh` reports `CORE_CERTIFICATION pass=16 fail=0 total=16` |
| Core aggregate gate | `bash scripts/check-integrity.sh` reports `CORE OK` |
| MIR semantic subset | `bash scripts/check-integrity.sh` reports `MIR OK` |
| Codegen invariant quarantine | `bash scripts/check-integrity.sh` reports `CODEGEN OK` |
| Ecosystem compilation surface | `bash scripts/check-integrity.sh` reports `ECOSYSTEM OK: syntax=200/? stages=14/? std=82/82 vaisdb=261/261` |
| Backend smoke | `bash scripts/check-integrity.sh` reports `BACKEND OK: phase158=18/18` |
| VaisDB runtime smoke | `bash scripts/check-integrity.sh` reports `VAISDB RUNTIME OK: smoke=34/34` |
| Whitespace sanity | `git diff --check` is clean in `compiler/` and `lang/` |

The narrative table above documents *what* each gate reports. The exact
pass/total numbers are *also* sourced from `GATE_MANIFEST.toml` and rendered
into the canonical baseline below by `python3 scripts/render-gate-tables.py`.
The narrative table above is the human-readable form (kept stable for the
`crates/vaisc/tests/core_certification.rs::core_freeze_criteria_doc_is_current`
freeze guard); the canonical baseline below is the machine-readable form
enforced by `bash scripts/check-gate-manifest.sh` M4. Both must agree — if
a count drifts in only one place, the freeze guard or M4 will fail.

### Canonical Gate Baseline

The table below between `gate-table:auto-start` / `gate-table:auto-end`
markers is generated from `GATE_MANIFEST.toml` by
`python3 scripts/render-gate-tables.py`. Do not hand-edit.

<!-- gate-table:auto-start -->
| Gate | Current status |
|---|---|
| Core certification | `CORE_CERTIFICATION pass=16 fail=0 total=16` |
| MIR strict gate | `MIR OK` |
| Codegen invariant gate | `CODEGEN OK` |
| Unsafe documentation audit | `UNSAFE AUDIT OK: vais-codegen undocumented_unsafe_blocks=0` |
| Ecosystem package codegen | `std=82/82`, `vaisdb=261/261` |
| Backend smoke | `phase158=18/18` |
| std/http_client runtime | `smoke=15/15` |
| std/tls runtime | `smoke=2/2` |
| VaisDB runtime | `smoke=34/34` |
| vais-server runtime | `smoke=20/20` |
| vais-web runtime | `smoke=61/77` |
| vais-web unit | `tests=390/390` |
| vais-web packages | `tests=3272/3272` |
| vais-web full-build | `packages=24/24` |
| Cross-package schema gate | `gate=15/15` |
| Multi-domain product gate | `gate=9/9` |
| Package full-build smoke | `smoke=2/2` |
<!-- gate-table:auto-end -->

If any number changes because a fixture is intentionally promoted, update
`GATE_MANIFEST.toml` (and the corresponding `INTEGRITY_*_MIN` threshold in
`scripts/check-integrity.sh`), then re-render. The stale guard in
`crates/vaisc/tests/core_certification.rs` must also be updated in the same
change when the Core certification count changes.

## Freeze Blockers

Do not declare a Core freeze if any of these are true:

- `tests/core/mir_deferred.tsv` contains a Core fixture entry.
- `tests/core/certification_exclusions.tsv` hides a Core failure without a
  documented reason.
- a negative Core fixture lacks a stable diagnostic code or no longer emits
  `error[CODE]`
- `lower_module_checked` accepts a promoted Core fixture by emitting a semantic
  placeholder
- codegen receives unresolved `Unknown`, `Var`, or inference placeholder types
  for a Core fixture
- a current failing gate is explained away by an archived roadmap count

## Allowed Work After Freeze

After a Core freeze, compiler work should be limited to:

- fixing a red Core/MIR/CODEGEN gate
- promoting one explicitly scoped new invariant
- correcting stale certification documentation
- adding downstream gates that consume the frozen Core without changing its
  contract

Product work may resume in this order:

1. VaisDB embedded durability scenario
2. VaisDB vector/HNSW correctness scenario
3. `vais-server` minimal runtime gate
4. `vais-server` plus VaisDB integration gate
5. `vais-web` plus `vais-server` end-to-end gate

Any downstream failure must first be classified as product/API drift, compiler
regression, or unsupported non-Core feature. Only compiler regressions should
modify the frozen Core compiler path.

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
| VaisDB runtime smoke | `bash scripts/check-integrity.sh` reports `VAISDB RUNTIME OK: smoke=28/28` |
| Whitespace sanity | `git diff --check` is clean in `compiler/` and `lang/` |

If any number changes because a fixture is intentionally promoted, this document
and the stale guard in `crates/vaisc/tests/core_certification.rs` must be
updated in the same change.

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

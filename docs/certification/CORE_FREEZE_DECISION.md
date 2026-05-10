# Core Freeze Decision

Date: 2026-05-01

Status: Frozen for downstream re-entry

## Decision

The certified Vais Core compiler is frozen for downstream re-entry.

This is not a claim that every Vais language feature, experimental compiler
crate, or product package is complete. It means the Core contract defined by
`VAIS_CORE_V0.md`, `MIR_CONTRACT.md`, `DIAGNOSTIC_CONTRACT.md`, and
`CORE_FREEZE_CRITERIA.md` has met its required gate bundle and should no longer
be changed to satisfy downstream product drift.

## Evidence

The freeze decision is based on the same-batch gate bundle required by
`CORE_FREEZE_CRITERIA.md`:

| Gate | Evidence |
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

## Frozen Scope

Frozen:

- Core v0 manifest behavior
- strict MIR promoted subset
- Core negative diagnostic shape
- grouped gate failure propagation
- active certification source-of-truth list

Not frozen:

- new explicitly scoped invariant promotions
- fixes for red Core/MIR/CODEGEN gates
- downstream product gates that consume the frozen Core
- documentation corrections that preserve the current gate evidence

Out of scope for this decision:

- product-complete `vaisdb`, `vais-server`, or `vais-web`
- self-hosting completion claims
- experimental crates outside the default trust path
- broad Deferred or Experimental language features

## Downstream Re-Entry

Product work may resume in this order while the frozen Core gates remain green:

1. VaisDB embedded durability scenario
2. VaisDB vector/HNSW correctness scenario
3. `vais-server` minimal runtime gate
4. `vais-server` plus VaisDB integration gate
5. `vais-web` plus `vais-server` end-to-end gate

Any downstream failure must be classified before changing compiler code:

- product/API drift
- compiler regression
- unsupported non-Core feature

Only compiler regressions should modify the frozen Core compiler path.

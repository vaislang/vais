# Phase 197 P197-V — vaisdb Audit

**Date**: 2026-04-18
**Compiler**: `/Users/sswoo/study/projects/vais/compiler/target/release/vaisc` (HEAD = 784d66e3)
**Method**: `vaisc check` each `src/*.vais` (261 files) with fresh cache removed first.

> **Important**: The audit agent was tool-budget-cut before writing this report. The numbers below were synthesized by the main agent from `/tmp/vaisdb_{all_vais,failures,check,full_check}.log` which the sub-agent produced before the cutoff.

## Source layout

- `src/` contains **261 `.vais` files** across 12 top-level modules:
  `client, fulltext, graph, main.vais, ops, planner, rag, security, server, sql, storage, vector`
- Build system: `vais.toml` (Vais package descriptor, no Cargo).

## Entry-point check (`src/main.vais`)

```
error: error[E004] Undefined function
  --> main.vais:36:14
   |
  36 |     M config.validate() {
   |              ^^^^^^^^ function 'validate' is not defined
```

`main.vais` fails immediately on a missing `validate` method call — so the project does not compile as a whole.

## Per-file check results

| Metric | Count |
|---|---|
| Total `.vais` files | 261 |
| Files with ≥1 error | 225 |
| Files clean | 36 (~14%) |

Many of the 225 report **only the first** error; once that fires, downstream ones are shadowed. True failure count is likely higher.

## Failure breakdown by error code

| Code | Meaning | Distinct files |
|---|---|---|
| **E004** | Undefined function | **110** |
| **P001** | Parser — unexpected token | **47** |
| **E002** | Undefined variable | **29** |
| **E030** | No such field on type | **17** |
| **E001** | Type mismatch | **14** |
| **E003** | Undefined type | **5** |
| **E022** | Use after move | **2** |
| **E008** | Duplicate definition | **1** |

### Category samples

**E004 (Undefined function, 110건)** — dominant category. Probably the vaisdb codebase references many functions that never got implemented or were removed from the stdlib / internal modules.
- `src/fulltext/index/compression.vais:26` — function missing
- `src/fulltext/index/posting.vais:29` — function missing
- `src/fulltext/search/bm25.vais:118` — function missing

**P001 (Parser, 47건)** — spread across modules, mostly deep lines (e.g. `mod.vais:665`, `concurrency.vais:204`). Suggests syntax that the parser used to accept but no longer does, or constructs the parser never supported.
- `src/fulltext/concurrency.vais:204:8`
- `src/fulltext/ddl.vais:109:21`
- `src/fulltext/mod.vais:665:18`

**E001 (Type mismatch, 14건)** — mix:
- "expected `Str`, found `str`" (4+ occurrences in `rag/` and `planner/`) — capital-S vs lowercase-s string type confusion; suggests two distinct string types in the typing universe.
- "expected `RwLock<T>`, found `RwLock`" in `ops/health.vais`, `ops/metrics.vais` — monomorphisation failure or generic-parameter dropping.
- `src/rag/chunking/hierarchy.vais` — i64 vs str crossover.

**E022 (Use after move, 2건)** — `src/planner/format.vais`, `src/rag/context/helpers.vais` — both on a variable named `init`. Consistent pattern; might be a stdlib factory method that moves the argument.

**E030 (No such field, 17건)** — `VaisError.code` / `VaisError.message` style field misses, or deeper struct evolutions.

## Phase 195/196 연관성 (hypotheses)

The **scale** of vaisdb failures (86% of files) far exceeds what Phase 195/196 could have caused on their own:
- Phase 196 touched inkwell codegen + type-checker for ConstArray / union fields / enum primitive payloads / PANIC_BUILTINS. None of those map to E004 (undefined function), which is the dominant category.
- The P001 parser failures at deep line numbers suggest vaisdb source predates several grammar tightenings (the `G X := mut Y` form removed in Phase 195, for instance, or forward-decl removal — both would show as P001).
- E001 "expected Str, found str" is new: the compiler now distinguishes capital-S string type from lowercase-s; vaisdb still types some values as `Str`.
- E030 / E003 suggest struct fields / types renamed or removed from stdlib between vaisdb's writing and HEAD.

**Conclusion**: vaisdb is far enough behind the current compiler that it's effectively a different dialect. Most regressions are **pre-existing**, not introduced by Phase 195/196 specifically. A handful (RwLock monomorphisation, some E001s) look like they might be Phase 195/196 contributions but are in the noise.

## Recommended action (for Phase 198)

Full fix is a multi-phase project, not a single-phase sweep. Prioritize:

1. **Block 1 (parser — P001 × 47)**: classify the exact syntax variants that fail; a small set of grammar changes will unblock most.
2. **Block 2 (typing — Str vs str, RwLock<T>)**: audit the compiler's string-type model; this is likely a single-line fix that affects many files.
3. **Block 3 (missing functions — E004 × 110, E002 × 29)**: likely a stdlib evolution (renames, removals, split into submodules). Needs a stdlib-vs-vaisdb diff, not a compiler change.
4. **Block 4 (everything else)**: E030/E022/E008/E003/C001 etc. — address individually after Blocks 1-3 collapse the noise.

## Files confirmed clean (sample)

The 36 files that compile cleanly are a mix of `src/storage/`, `src/vector/`, some `src/sql/` — the modules least affected by parser tightening.

## PROMISE

**PROMISE: COMPLETE** (synthesized from sub-agent logs after tool-budget cutoff)

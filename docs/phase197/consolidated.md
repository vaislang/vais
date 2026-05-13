# Phase 197 Consolidated — 3-package audit synthesis

**Date**: 2026-04-18
**Scope**: merge `vaisdb_report.md`, `vais-server_report.md`, `vais-web_report.md` into a single verdict + Phase 198 input.

## Per-package verdict

| Package | Green / Yellow / Red | Evidence |
|---|---|---|
| **vais-web (Rust crates)** | 🟢 GREEN | 271 tests pass, 0 failed. Independent of this repo's compiler. |
| **vais-server (main build)** | 🟢 GREEN | `src/main.vais` → IR → clang → Mach-O binary, all exit 0. Binary runs. |
| **vais-server (test suite)** | 🟡 YELLOW | 7/22 pass. All 15 failures predate Phase 195/196 (C-style for-loop, stub file, dup defs, API drift). |
| **vais-web (Node)** | 🟡 YELLOW | 1176+ tests pass; `packages/federation` throws during shutdown → `pnpm -r` short-circuits, `motion` and later packages unmeasured. Not a Vais compiler issue. |
| **vaisdb** | 🔴 RED | 225/261 source files fail `vaisc check` (86%). Massive dialect drift. Most errors pre-existing; a few may be Phase 196-adjacent. |

## Cross-package regression root cause groups

Organized by "one fix → N files" merge opportunity, mirroring Phase 195/196 strategy.

### Group A — Parser grammar drift (vaisdb × 47, vais-server × 7)

**Signature**: `error[P001] Unexpected token` at various deep line numbers.
**Dominant sub-pattern** (vais-server and likely some of vaisdb): C-style three-part `I i = 0; i < n; i = i + 1 { }` loops. Vais `I` only admits `I cond { body }`. C-style iteration should use `LF i:0..n` (for-each over range) or `LW cond { }` (while).
**Fix scope**: example/source migration, not a compiler change. The current parser is correct; these sources predate the grammar.
**Estimated files affected**: ~50 (vaisdb deep modules + 4 vais-server test dep files).

### Group B — String type split `Str` vs `str` (vaisdb × 4+)

**Signature**: `error[E001] Type mismatch — expected Str, found str`.
**Hypothesis**: the type checker now distinguishes `Str` (capital, possibly an alias or user type) from `str` (primitive). vaisdb sources interchange them.
**Fix scope**: **unknown — could be compiler or source**. First step: grep the compiler for where `Str` with a capital-S is defined or aliased. Possibly a stdlib type alias that went missing, or a strict-mode rule.
**Estimated files affected**: 4–10 in vaisdb.

### Group C — Missing stdlib/internal functions (vaisdb × 110 E004, × 29 E002)

**Signature**: `error[E004] Undefined function` / `error[E002] Undefined variable`.
**Hypothesis**: vaisdb calls functions (many internal helper names like `fnv1a_hash`, `btree_insert`, `fuse_results`, `analyze_query`) that have been renamed, moved, or never existed in the current stdlib layout.
**Fix scope**: source-side audit, not compiler. Needs a vaisdb-vs-current-stdlib diff to map old names → new names, or add missing exports.
**Estimated files affected**: 100+.

### Group D — `VaisError` struct API drift (vaisdb × 17 E030, vais-server × 2 tests)

**Signature**: `error[E030] No such field — .code`, `.message`.
**Fix scope**: two options
1. Add `code`, `message` fields (or getter methods) to `VaisError` — compiler/stdlib change.
2. Update callers to use the current API — source change.
Decide before touching anything. If `VaisError` is meant to be the single error type for the stdlib, Option 1 restores a lot of surface area cheaply.

### Group E — `RwLock<T>` monomorphisation drop (vais-server × 0, vaisdb × 2)

**Signature**: `error[E001] Type mismatch — expected RwLock<SystemMetrics>, found RwLock`.
**Hypothesis**: a generic type parameter is being dropped on the `RwLock<T>` instantiation somewhere in ownership/metrics code. **Plausibly Phase 196-adjacent** (P196-B2 touched generic instantiation / HashMap). Warrants compiler-side investigation.
**Fix scope**: small compiler change if the hypothesis holds; narrow source change otherwise.

### Group F — `M` (match) on `Result`-returning method calls (vais-server × 2 tests)

**Signature**: type errors on `M config.validate() { … }` patterns.
**Hypothesis**: match scrutinee is a `Result<T, E>` but the checker isn't peeling it. Might intersect with Phase 195 Option/Result improvements.
**Fix scope**: compiler change likely.

### Group G — Everything else (noise, each ≤5 occurrences)

- `E003` undefined type (vaisdb × 5)
- `E022` use-after-move on `init` (vaisdb × 2, same variable name — suspicious pattern)
- `E008` duplicate definition (vaisdb × 1, vais-server × 2)
- `P002` empty file (vais-server × 1 — comment-only stub)
- `federation` vitest thrown-error (vais-web Node × 1 — TypeScript bug, not Vais)

## Phase 195/196 impact assessment

**Direct Phase 195/196 responsibility**: low.
- Main builds (vais-server binary, vais-web Rust crates) are completely unaffected.
- vaisdb red state is a pre-existing backlog — the compiler moved faster than the downstream source. The Phase 195 parser tightenings (`G X := mut Y`, `spawn`, forward decls) and Phase 196 type-checker additions (ConstArray, strict `Str` vs `str`?) may account for a handful of files but nowhere near the bulk.

**Phase 196-plausible regressions worth verifying**:
- Group B (`Str` vs `str`) — did the `Str` alias go away?
- Group E (`RwLock<T>` instantiation dropping the param) — P196-B2 generic fix may have a blind spot.
- Group F (M-on-Result) — P195-3 / P196 type-checker Option/Result handling.

Each of these is a small spot-check (grep compiler main for `Str`, monomorphize a minimal `RwLock<i64>` locally, write a minimal `F f() -> Result<i64, i64> = Ok(1); F main() { M f() { … } }`). If any triggers, the compiler gets a targeted fix; otherwise the source migrates.

## Phase 198 task buckets (for next planning step)

Prioritized by (a) blast radius and (b) root-cause clarity:

| Bucket | Effort | Owner suggestion | Unblocks |
|---|---|---|---|
| 1. **Spot-check Group B/E/F** (compiler or source?) | 1 task, ~30 min | research-haiku recon → Opus direct | E/F possibly fixable in compiler; B determines stdlib policy |
| 2. **`VaisError` API decision** (Group D) | 1 task, decide + implement | Opus direct | Clears ~20 errors |
| 3. **Parser migration (Group A)** | Batch source edits on vaisdb + vais-server tests | impl-sonnet | ~54 P001 errors |
| 4. **Stdlib audit (Group C)** | Largest single bucket | research-haiku survey → impl-sonnet apply | 130+ errors |
| 5. **federation vitest fix** (vais-web Node) | Independent, JS-only | impl-sonnet | 1 failure; unblocks motion+ coverage |
| 6. **Tail cleanup (Group G)** | Per-file | impl-sonnet | ~10 errors |

Buckets 1/2/5 are small and parallelizable. Bucket 3 and 4 are where most of the vaisdb rot lives; they should land after Bucket 1's spot-check confirms the parser grammar is the right target.

## Final gate question

Is this Phase 197 exit criterion met?

- ✅ 3 리포트 작성됨 (실측 수치 포함)
- ✅ compiler crate 0 파일 변경
- ⏳ Phase 198 계획 작성 — Task #6 `Phase 198 계획 수립` 가 이걸 produces
- ⏳ docs/phase197/final_report.md — Task #8 `Final`

This consolidated view is enough input for Task #6 (Phase 198 planning) and Task #7 (user approval gate) to proceed.

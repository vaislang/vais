# vaisdb Cascade Survey — D.1 (2026-04-21)

> Baseline at session start: vaisdb = **237/261** (90.80%).
> Direct measurement via `./scripts/check-integrity.sh` + per-file re-run
> with `vaisc build --emit-ir --force-rebuild` for each of the 261 files.
> Results: **237 pass / 24 fail / 261 total** — no cascade from the
> 2026-04-21 compiler drive beyond what the prior baseline already captured.

## 1. Baseline check

```
INTEGRITY OK: syntax=200/? stages=14/? std=82/82 vaisdb=237/261 phase158=18/18
```

No delta from the prior drive's final baseline (commit `e1697c14`).
The 11 structural compiler fixes (A.1~A.3, B.1~B.6, C.1~C.2) landed before
the baseline was snapshotted, so additional cascade is **zero net** for this
measurement — expected.

## 2. Failure list (24 files)

```
fulltext/mod.vais
graph/mod.vais
planner/mod.vais
planner/pipeline.vais
rag/mod.vais
sql/catalog/manager.vais
sql/executor/dml.vais
sql/executor/join.vais
sql/executor/mod.vais
sql/executor/sort_agg.vais
sql/executor/window.vais
sql/parser/mod.vais
sql/parser/parser_expr.vais
storage/btree/insert.vais
storage/txn/deadlock.vais
vector/concurrency.vais
vector/filter.vais
vector/hnsw/bulk.vais
vector/hnsw/cow.vais
vector/hnsw/delete.vais
vector/hnsw/insert.vais
vector/hnsw/wal.vais
vector/mod.vais
vector/search.vais
```

Full error text: `/tmp/vaisdb-survey/fail-detail-v4.txt`.

## 3. Classification

Each file classified into:
- **(a)** compiler gap (TC/codegen limitation blocking well-formed code)
- **(b)** vaisdb code bug (Vais source is objectively wrong — arg count mismatch,
  undefined method, wrong field name, etc.)
- **(c)** SCOPED Phase 4.x dependency (construct the compiler deliberately
  doesn't support yet — trait objects with generic bounds, total-function
  panic-analysis, etc.)

### 3.1 (a) Compiler gap — **0 files** (revised 2026-04-21 D.3)

Originally D.1 classified `vector/search.vais:77` (`table_meta.columns[coli]` →
C003 "Cannot index into type 'i64'") as a compiler gap. **D.3 re-check shows
this is actually bucket (b)**: `S TableInfo` (defined in
`sql/catalog/schema.vais:52`) has **no `columns` field** — only
`column_count: u16`. The vaisdb code accesses a non-existent field; TC
falls back to `i64` for the unknown field; codegen rejects indexing i64.

This is identical-shape to other bucket (b) bugs (accessing a method or
field that doesn't exist on the receiving type). The cascade (planner/mod,
vector/concurrency, vector/filter) is a vaisdb-side cascade, not a
compiler cascade.

**(a) total (revised): 0 compiler gaps, 0 files. Previously counted 4 files
now belong to (b).**

### 3.2 (b) vaisdb code bugs — 16 files

These are plain Vais-code bugs: method signatures, undefined methods,
missing fields, arg-count mismatches. No compiler change would fix them —
the vaisdb code must be corrected.

| # | File | Primary bug pattern |
|---|------|--------------------|
| 1 | `fulltext/mod.vais` | 6 errors — `write_lock()` expects 1 arg but called with 0 (×3), `PhraseSearcher.new(0)` expects 0 got 1, `get_stats()` undefined |
| 2 | `graph/mod.vais` | 6 errors — `deserialize` expects 2 got 1, `write_page` expects 1 got 3, `lock_node` undefined (×3) |
| 3 | `rag/mod.vais` | 5 errors — `RagWalManager.new()` expects 1 got 0, `build_for_document` undefined, `log_memory_write` expects 6 got 2, `MemorySimilarityEntry` vs tuple mismatch, `remove` undefined |
| 4 | `sql/catalog/manager.vais` | 5 errors — sort_by closure field inference (`column_index`), `write_page` signature (expects 1 got 2, ×3) |
| 5 | `sql/executor/dml.vais` | ≥7 errors — `get_table_indexes` undefined (×3), `write_page` signature (×3+) |
| 6 | `sql/executor/join.vais` | 6+ errors — `.next(ctx)` on right_source undefined (×5), `combined_row` variable scope bug |
| 7 | `sql/executor/mod.vais` | 5 errors — inherits manager.vais bugs through import, + `load_from_disk` total-function |
| 8 | `sql/executor/sort_agg.vais` | 2 errors — assigning `Option<SqlValue>` to numeric slot, Result return type inference |
| 9 | `sql/parser/mod.vais` | 2 errors — `TokenKind` vs `Expr`, bare `Vec` vs `Vec<Expr>` |
| 10 | `sql/parser/parser_expr.vais` | 1 error — bare `Vec` vs `Vec<Expr>` |
| 11 | `storage/btree/insert.vais` | 2 errors — `.release()` and `.serialize_into()` undefined |
| 12 | `storage/txn/deadlock.vais` | 1 error — expects `Vec<u64>`, returns `i64` |
| 13 | `vector/hnsw/delete.vais` | 3 errors — `store.get_node` undefined (×3) |
| 14 | `vector/hnsw/insert.vais` | 2 errors — `visited.insert`, `meta.len()` undefined |
| 15 | `vector/hnsw/wal.vais` | 6+ errors — `insert_node`, `delete_node`, `get_node`, `get_page_data_mut` all undefined on `store` |
| 16 | `vector/mod.vais` | 9+ errors — `DistanceMetric` vs `u8`, `pin_layer`, `log_*` undefined, `write_page`/`allocate_page` arg counts, `BulkLoader.new` expects 1 got 7 |

**Pattern summary** (from 16 files):
- `write_page` arg-count mismatch: 9+ sites across catalog/dml/mod/graph/vector/mod
- `get_node` / `insert_node` / `delete_node` on `store`: 10+ sites across hnsw/wal, hnsw/delete
- `lock_node` on `lock_mgr`: 3 sites in graph/mod
- Constructor arg-count (`.new()`): 6+ sites
- Misc undefined methods: ~15 sites

**(b) total (revised 2026-04-21 D.3): 20 files (83.3%).**
Originally 16. The 4 previously-cascaded files from §3.1 (`vector/search`,
`planner/mod`, `vector/concurrency`, `vector/filter`) join (b) because the
root cause is a missing-field vaisdb bug. No compiler fix closes them.

### 3.3 (c) SCOPED Phase 4.x dependency — 4 files

| # | File | Blocker |
|---|------|---------|
| 1 | `sql/parser/mod.vais` | (also in (b)) Bare `Vec` unification — could be a TC inference gap, but listed (b) because adding explicit `<Expr>` is trivial |
| 2 | `sql/executor/window.vais` | E034 — `total function may panic` enforcement (`open` transitively calls `partition_rows` which unwraps `!`). Requires `partial F` annotation propagation — Phase 4.x purity-analysis feature |
| 3 | `vector/hnsw/bulk.vais` | 2 errors — generic bound `S: NodeStore` in scope but a local `node` fails the bound unification (TC bug OR missing trait impl for `S`); E034 panic |
| 4 | `vector/hnsw/cow.vais` | `ErrorCode.from_u32(...)` returns `ErrorCode`, passed where `str` is expected — vaisdb bug. E034 also present |

E034 "Total function may panic" appears in 4 files (manager/mod/bulk/cow/window).
Fix is **add `partial` keyword** to each offending F declaration — this is a
vaisdb code edit, so technically (b). But we group as (c) because changing
existing annotations at scale is a deliberate project choice, not a
compiler fix.

**(c) total: effectively 4 files with E034, 1 file (hnsw/bulk) with a generic-bound question. ~16.7%**

## 4. Representative examples (3 per bucket)

### (a) Compiler gap — 3 representatives

(Section obsolete — see §3.1 revision 2026-04-21 D.3. No compiler-gap
representatives exist; what was a.1 is actually `S TableInfo` missing a
`columns` field — a vaisdb code bug, not a codegen type-resolution bug.)

### (b) vaisdb code bugs — 3 representatives

**b.1** `vector/hnsw/wal.vais:238` — wrong method name
```vais
store.insert_node(&node)    # 'insert_node' not defined on the trait
```
`store` implements `NodeStore` trait; the trait likely has a different method
name (e.g. `put_node` or `add_node`). Fix by matching the trait's actual
method name.

**b.2** `sql/catalog/manager.vais:252` — arg count
```vais
pool.write_page(heap_frame, flushed)?;   # expected 1, got 2
```
`BufferPool.write_page` evolved to a 1-arg signature (frame → flushed implicit?),
but 10+ callsites still pass 2 args. Fix is mechanical refactor.

**b.3** `sql/parser/parser_expr.vais` — bare generic
```vais
let args: Vec<Expr> := some_call()  # some_call returns `Vec`
```
Either the callsite is missing an explicit generic or the callee lost its
generic annotation. Fix is to add `<Expr>` on the returning side.

### (c) SCOPED / purity dependency — 3 representatives

**c.1** `sql/executor/window.vais:79`
```vais
F open(mut self) -> Result<(), VaisError> {
    ...
    self.partition_rows()!   # partition_rows may panic, propagates to open
}
```
E034 fires because `partition_rows` transitively unwraps `!`. Fix:
annotate `F partial open(...)` or propagate Results all the way.

**c.2** `vector/hnsw/bulk.vais:420`
```vais
F build_graph<S: NodeStore>(
    store: &S,
    ...
) {
    self.store_neighbor(store, node, ...)   # line 456 — generic S fails NodeStore bound
}
```
TC message: `found type 'S' which does not implement 'NodeStore'` — despite
the where-clause. Either TC has a bound-propagation gap in nested calls, or
the receiving function's signature lost its own `S: NodeStore` bound.
Needs deeper investigation; possibly a legitimate TC bug (→ a).

**c.3** `vector/hnsw/cow.vais:362`
```vais
ErrorCode.from_u32(ERR_COW_INVALID_LAYER),   # expected str, found ErrorCode
```
Actually a vaisdb code bug — should wrap in `.message()` or similar.

## 5. Classification summary (revised 2026-04-21 D.3)

| Class | Count | % | Net file delta if fixed |
|-------|-------|---|-------------------------|
| (a) Compiler gap | **0 files** | **0%** | 0 (original candidate re-classified) |
| (b) vaisdb code bugs | **20 files** | **83.3%** | +20 (requires vaisdb edits) |
| (c) SCOPED / purity | 4 files (overlap with b) | 16.7% | +4 (requires `partial` annotations or Phase 4.x work) |

**Unique file count: 24.**

The re-check for the originally-classified compiler gap
(`vector/search.vais:77:40`) showed the `columns` field simply does not
exist on `S TableInfo`. That puts the file squarely in bucket (b) and
carries the three cascade files with it. The survey no longer proposes
any compiler-fix leverage on vaisdb from within this drive.

During D.2 a separate compiler gap was observed — struct-in-array-literal
(`[Point{...}, Point{...}]` emits a pointer into a struct slot) — but
**no vaisdb file uses that pattern** (zero leverage for this drive).
The gap is real and will surface when stdlib or user code starts using it;
filed for a future drive.

## 6. Decision guidance for D.3 — finalized

**Option (α) selected**: D.3 becomes a scope-documentation task rather than
a compiler fix. Rationale:

1. The largest bucket (b = 83.3%) is out-of-compiler-scope.
2. The original (a) candidate turned out to be (b) on closer inspection.
3. No other compiler-gap cluster surfaced in D.1's measurement.
4. D.2's Vec-literal fix is shipped and preserved baseline.
5. Drive-close with clear hand-off to next drive is more honest than
   spending iterations chasing diminishing returns.

**Out-of-drive items** (for E.1 to surface as next-drive candidates):
- vaisdb cleanup pass (20 files, bucket b) — separate repository, separate drive.
- Phase 4.x panic/purity analysis (4 files, bucket c).
- Struct-in-array-literal text-backend fix (`generate_array_expr` for Named
  element types) — complements D.2 when vaisdb or stdlib starts using
  `Vec<Struct> := [Struct{...}, ...]`.
- `Vec::new()` definition in std/vec (currently only `with_capacity`).

Baseline preservation floor held throughout D.1→D.3: **vaisdb = 237/261**,
**syntax = 200**, **stages = 14**, **std = 82/82**, **phase158 = 18/18**.

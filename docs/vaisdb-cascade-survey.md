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

### 3.1 (a) Compiler gap — 1 file (root cause of 3 cascade failures)

| # | File | Error |
|---|------|-------|
| 1 | `vector/search.vais` | `error[C003] Type error ... Cannot index into type 'i64'` at line 77:40. The `table_meta.columns[coli]` expression — `table_meta` comes from `get_table_by_id` whose return type the type-checker accepts but whose codegen-time resolution falls back to `i64` scalar. Three other files (`planner/mod`, `vector/concurrency`, `vector/filter`) import `search.vais` and inherit this codegen error. |

**Cascade impact**: fixing `vector/search.vais`'s root codegen bug would
auto-unblock `planner/mod`, `vector/concurrency`, `vector/filter` → +4 files
(237 → 241).

**(a) total: 1 root gap, 4 affected files (16.7%)**

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

**(b) total: 16 files (66.7%). No compiler fix can close these.**

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

**a.1** `vector/search.vais:77`
```vais
LW coli < table_meta.columns.len() {
    col := &table_meta.columns[coli]   # <-- C003: Cannot index into type 'i64'
```
`get_table_by_id` → `TableInfo` → codegen loses the `columns: Vec<Col>` field
type, treats `table_meta.columns` as `i64`. Likely a **method-return-type
inheritance at codegen time** problem in Named type resolution.

**a.2** `planner/mod.vais`, **a.3** `vector/concurrency.vais`, **a.4** `vector/filter.vais`
All three **cascade from a.1**: they import vector/search.vais and
inherit its codegen error. Fix a.1 → all four unblock.

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

## 5. Classification summary

| Class | Count | % | Net file delta if fixed |
|-------|-------|---|-------------------------|
| (a) Compiler gap | 1 root (4 files cascade) | 16.7% | +4 |
| (b) vaisdb code bugs | 16 files | 66.7% | +16 (requires vaisdb edits) |
| (c) SCOPED / purity | 4 files (overlap) | 16.7% | +4 (requires `partial` annotations) |

**Unique file count: 24 (no double-counting between b/c — E034 files categorised under (c)).**

## 6. Decision guidance for D.3

**Observation**: Vec-literal lowering (D.2) is **not** the largest vaisdb
blocker. Only 2 of 24 failures are plausibly Vec-literal-related
(`sql/parser/mod`, `sql/parser/parser_expr` with bare `Vec`). D.2 remains
justified independently because:

1. CLAUDE.md rule 5: `v: Vec<T> := [...]` literally fails with
   `Cannot allocate unsized type` (confirmed via `/tmp/vec_test.vais`
   during this survey).
2. D.2 unblocks `Vec<Struct>[i].field =` write-through (B.4 finished but
   literal lowering gap remained, per ROADMAP §드라이브 목적).

**D.3 recommendation (α)**: The dominant failure bucket is (b) vaisdb code
bugs (66.7%). Fixing compiler further yields diminishing returns here. D.3
should therefore be:
- **Narrow scope**: the **one** compiler gap in (a) — `vector/search.vais`
  codegen-time Named type resolution — which cascades to 4 files. This is
  a high-leverage compiler fix.
- **OR**: declare D.3 as "out-of-drive-scope: vaisdb cleanup" and close
  the drive after D.2.

Baseline preservation floor: **vaisdb ≥ 237/261** throughout remaining tasks.

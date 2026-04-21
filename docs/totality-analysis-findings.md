# Totality Analysis (E034) — Findings 2026-04-21 H.1

Investigation of whether E034 "Total function may panic" has precision
gaps that could be tightened without TC redesign.

## Current design (crates/vais-types/src/totality.rs)

The analyzer runs after module-level type checking. Steps:

1. Collect all free functions + impl methods.
2. For each body, walk the expression tree looking for direct panic sources:
   - Calls to `panic`/`abort`/`assert`/`__panic` builtins.
   - `Expr::Assert` form.
   - `expr!` unwrap.
   - (Division, modulo, indexing deliberately NOT flagged — delegated to
     refinement types / runtime guards.)
3. Seed reachable set with direct-panic fns + all `partial` fns.
4. Worklist fixpoint: any fn calling a reachable fn becomes reachable.
5. Report first violator alphabetically.

Design rationale (docs in totality.rs §"Why this scoping"): "catch what
a careful reviewer would catch". Empirical iter 10 measurement: narrowing
to `!` + panic-builtins + partial calls cut false-positives ~95%.

## E034 failures observed in vaisdb (D.1 survey)

4 files trigger E034:
- `sql/executor/window.vais::open` — transitively calls `partition_rows`
  which uses `!` after `contains_key → insert → get!`.
- `sql/catalog/manager.vais::load_from_disk` — direct `!` unwrap.
- `vector/hnsw/bulk.vais::build_graph` — direct `!` unwrap.
- `vector/hnsw/cow.vais::add_neighbor` — direct `!` unwrap.

All 4 are **legitimate panic sources** from the compiler's perspective.
The code uses `!` in total functions. The analyzer is correct to flag them.

## The `contains_key → insert → get!` pattern

`partition_rows` in window.vais:

```vais
I !partitions.contains_key(&hash) {
    partitions.insert(hash, Partition.new(...))
}
partition := mut partitions.get(&hash)!   # invariant: always Some
```

Here the `!` is provably safe because of the surrounding guard. A
**flow-sensitive** totality analyzer could track the invariant and accept
this pattern without `partial`. But this requires:

- Control-flow analysis beyond the current syntactic walk.
- Semantic understanding of `contains_key` / `insert` / `get` relationship.
- Regression safety against 12,000+ existing tests.

This is substantial TC work and lives squarely in Phase 4.x SCOPED.

## Alternative improvements considered

1. **Accept `?` as error-propagation vs panic**: already done. `Expr::Try`
   is not a panic source (totality.rs:418).

2. **Re-add `exit` exemption**: already done (Phase 196 P196-D).

3. **Per-variant unwrap analysis** (`expr!` on `Option::Some(_)` constant):
   constant folding at TC level — would need literal-option tracking.

4. **Allow `!` on values returned by known-never-None functions**: requires
   effect annotations or a `NonNull<T>` type marker — infrastructure doesn't exist.

5. **Warn instead of error for certain patterns**: changes public contract
   of E034, risky.

## Conclusion

H.1 as originally scoped (improve E034 precision) would require either:
- flow-sensitive analysis (weeks of TC work),
- or an annotation system for "safe after this guard" invariants.

Both fall outside the triple-drive scope and risk Phase 158-style yo-yo
(where multiple `unification.rs` changes were applied/reverted 5 times).

**Decision**: H.1 no-op in this drive. vaisdb files marked with E034 should
either (a) add `partial F` annotations where panic is intentional, or
(b) restructure to use `?` error propagation. Both are vaisdb-side edits,
aligning with the cascade survey §3.3 classification.

## What remains for a future TC drive

Worth doing eventually:
- Flow-sensitive panic analysis for the `contains_key → insert → get!`
  pattern (single most common vaisdb use case).
- Per-call-site reason reporting (current analyzer reports first
  alphabetical violator; multi-violator reporting would help triage).
- A `#[safe_unwrap(reason)]` attribute that silences E034 for a single
  `!` with a documented justification.

These are all Phase 4.x+ and should happen in a dedicated purity drive.

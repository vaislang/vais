# VaisDB relevance baseline (2026-08-08, whole-repo corpus)

Two layers keep search-ranking quality honest:

- `scripts/test-vaisdb-relevance.sh` — the deterministic regression
  gate: a role-controlled mini corpus (definition-site doc, saturated
  heavy user, partial match, noise flood, multibyte doc) locks exact
  top rows, tie bands, and an integer MRR100 floor (currently 70).
  Ranking changes must consciously move those assertions.
- `scripts/vaisdb-relevance-report.sh` — this advisory report: a
  curated query set with long-lived expected documents, run against
  the live `scripts/vaisdb-repo.sh` index (553 docs at capture time).
  Refresh the table below by hand whenever a ranking change ships.

## Captured 2026-08-08 (rarity-weighted -all, plain-sum OR)

| Mode | hits@1 | hits@3 | MRR100 |
| --- | --- | --- | --- |
| OR | 0/8 | 2/8 | 17 |
| `-all` | 2/8 | 3/8 | 37 |

Per-query expected-document ranks (top-10 window):

| Query | Expected doc | OR | `-all` |
| --- | --- | --- | --- |
| token_shard_at | e337 …/vaisdb/index | 6 | 4 |
| str_slice_raw | tools/vaisc_native | 4 | 9 |
| rename atomic | examples/e391_fs_rename_atomic_swap | 3 | 1 |
| fixpoint | compiler/self/fixpoint_full | — | — |
| is_word_byte | e337 …/vaisdb/index | — | 1 |
| front contract | tools/vaisc_front_check | 7 | 6 |
| shell sort desugar | tools/vaisc_native | 2 | 2 |
| release gates | scripts/test-release-gates | — | — |

Readings: `-all` roughly doubles both summary metrics over OR — the
all-terms filter plus rarity weighting is earning its keep on
identifier-style queries (is_word_byte jumps from absent to rank 1).
The residual weak spots are single-common-term queries (`fixpoint`,
`release gates`), where the working notes legitimately out-frequency
the expected file and bag-of-words has no counter-signal; a
definition-site or field-weighted signal would need index-format work
and stays demand-driven.

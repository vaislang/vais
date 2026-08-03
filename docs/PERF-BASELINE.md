# Vais toolchain performance baseline (2026-07-22, arm64 macOS)

Measured on the mainline at `82537706` with a warm `build/` cache; unit
builds are best-of-three, gates are single serial runs.

## Unit builds

| Operation | Time |
| --- | --- |
| `scripts/vaisc build` hello (full engine) | 174 ms |
| `scripts/vaisc build` hello (direct engine) | 193 ms |
| `scripts/vaisc package` e337 (vaisdb) | 139 ms |
| `scripts/vaisc package` e341 (vaisgrep) | 134 ms |
| `build/vaisc emit-ir` self-host core (22.9k lines → 4.4 MB .ll) | 444 ms |
| driver rebuild (`build-vaisc-native.sh`, clang -O2 of core .ll + driver C) | 11.9 s |

Per-invocation `scripts/vaisc` overhead includes two preflight tool runs
(manifest + import-graph); with cached tools this is inside the ~170 ms above.

## Gates (serial, one run each)

| Gate | Wall time |
| --- | --- |
| vaisfmt-check (fmt) | 23 s |
| test-vaisc-front | 144 s |
| test-vaisc-direct | 164 s |
| test-vais-check-vais | 36 s |
| test-fixpoint-full | 863 s |
| test.sh (value corpus) | 206 s |
| test-vaisc-parity | 205 s |
| test-vaisdb-workflow | 60 s |
| test-vaisc-native | 17 s |
| test-fixpoint-full-self | 272 s |
| test-release-gates | 2153 s |

Sum of the pre-dedup ladder chain: ~4143 s (~69 min).

## Findings

- `test-release-gates` internally re-runs front/direct/check/fixpoint/value/
  parity/workflow/native/selfhost (1967 s of its 2153 s) plus the release-only
  gates (manifest, import-graph, install, errors, host, embed, stage-IR,
  compiler, fixpoint 1/2, packaging) and ends with `git diff --check`.
  The ladder chain therefore duplicated ~33 minutes of work per run.
- `tools/gates.tasks` now defines `ladder = fmt + release`: a strict coverage
  superset of the old chain at roughly half the serial wall time (~36 min).
  Individual gate tasks remain for selective runs, and `quick`
  (fmt/front/direct/check, ~6 min) is unchanged for tight loops.
- Largest single gate: `test-fixpoint-full` (863 s serial) — every case
  embeds the 23k-line self-host core, builds that compiler, emits the case
  IR, clang-links, and runs it. 2026-07-22: cases are now stateless-hash
  sharded across `VAIS_FIXPOINT_SHARDS` parallel workers (default 8) with
  identical coverage (partition by construction; the only repeated log line
  is the per-shard embed-helper setup): **863 s → 320 s (2.7x)**. The
  sub-linear scaling is per-shard setup plus concurrent clang links of
  ~4.4 MB IR saturating memory bandwidth. 2026-07-22 (same day): `test.sh`
  and `test-vaisc-parity` received the same stateless-hash sharding
  (`VAIS_VALUE_SHARDS` / `VAIS_PARITY_SHARDS`, default 8; 1 keeps the serial
  path, single-name runs bypass sharding): **206 s → 129 s** and
  **205 s → 129 s**, with the shell wrappers summing per-shard counters into
  the canonical RESULT lines (pass=368 / native=368 unchanged). The 1.6x
  scaling is bounded by per-entry `scripts/vaisc` process spawns. The
  self-host gate's five independent probes (each embeds and builds its own
  first-generation compiler) now run as parallel phase workers with the
  stage1/stage2 comparison last (`VAIS_SELFHOST_PHASES=serial` preserves the
  single-process path): **272 s → 177 s**, bounded by the two heaviest
  probes. The ladder (fmt + perf + release) now lands around ~22 min from the
  original ~69 min; remaining costs are the release-only gates and the
  sequential heavy probes. The `perf` ladder task
  (`scripts/vaisbench-gate.sh 60000 2 bash scripts/test-vaisc-native.sh`)
  watches the native smoke gate under a 60 s median budget — ~3.5x the
  17 s baseline, so only real regressions fire — turning this document's
  resume trigger into an automated check.

## VaisDB large-corpus operations (2026-08-03, arm64 macOS)

Measured with `scripts/bench-vaisdb-corpus.sh` (deterministic corpus:
100 words/doc from a 3000-term vocabulary, phrase markers every 100th
doc; vaisbench median of 3, packaged binaries, query exits normalized).

Pre-batch-registry measurements (the scaling cliff this section was
created to catch — every `doc_src_path` / `doc_known` / `doc_mtime`
call re-read and re-split the whole `docs.txt`, so registry-per-doc
operations went superquadratic):

| Operation | 200 docs | 1000 docs | 5x-docs factor |
| --- | --- | --- | --- |
| reindex cold | 783 ms | 54,515 ms | x70 |
| reindex warm (all skip) | 1,957 ms | 236,976 ms | x121 |
| search term k=10 | 35 ms | 1,169 ms | x33 |
| msearch 2 indexes | 666 ms | 79,377 ms | x119 |
| phrase 2-word | 658 ms | 78,996 ms | x120 |
| similar doc k=5 | 1,071 ms | 20,998 ms | x20 |
| top k=10 | 952 ms | 9,980 ms | x10 |

After the batch registry landed (2026-08-04: `registry_load_into`
snapshots docs.txt once per operation into parallel id/src/stamp tables
plus a position map, reindex/msearch/phrase thread it down, and
`ingest_doc_known` takes the membership decision from the snapshot):

| Operation | 200 docs | 1000 docs | 5x factor | 1000-doc speedup |
| --- | --- | --- | --- | --- |
| reindex cold | 312 ms | 1,603 ms | x5.1 | x34 |
| reindex warm (all skip) | 12 ms | 98 ms | x8 | x2,418 |
| search term k=10 | 35 ms | 1,075 ms | x31 | ~1x |
| msearch 2 indexes | 24 ms | 416 ms | x17 | x191 |
| phrase 2-word | 16 ms | 120 ms | x7.5 | x658 |
| similar doc k=5 | 1,036 ms | 19,842 ms | x19 | ~1x |
| top k=10 | 960 ms | 9,966 ms | x10 | ~1x |

Cold reindex now scales linearly in docs (x5.1 for 5x). The remaining
heavies are shard-scan bound, not registry bound: similar issues one
shard scan per query term (~95 for a 100-word source) and top walks
every shard once — both honest O(data) paths, candidates for a later
cycle if they matter in practice.

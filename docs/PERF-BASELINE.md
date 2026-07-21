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
- Largest single gate: `test-fixpoint-full` (863 s) — the full-codegen case
  matrix compiles and clang-links per case. Recorded as the next optimization
  target if ladder time matters again.

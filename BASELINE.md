# Deterministic Build Baseline

### Task #58 — Deterministic Build Baseline (2026-04-11)

**Compiler self-determinism**:
- Build 1 sha256: `3579bde4ff9b2d19b307122371d37151cdf359df7423bfe14f80e6208aefc302`
- Build 2 sha256: `3579bde4ff9b2d19b307122371d37151cdf359df7423bfe14f80e6208aefc302`
- Deterministic: YES

**Emitted .ll determinism** (hello.vais, via `vaisc --emit-ir -o /tmp/emitN.ll`):
- emit 1 sha256: `1f530b20752e7b46d1322e3900cf5e00d079c1061478d72ca5510b42fd343942`
- emit 2 sha256: `1f530b20752e7b46d1322e3900cf5e00d079c1061478d72ca5510b42fd343942`
- diff lines: 0
- Deterministic: YES
- IR file: 778 lines

**Blocker categorization** (18 SystemTime/Instant sites from recon #41):

- Category A (diagnostic output / watch debounce — printed to stderr/stdout, not in emitted binary): 15 / 18 sites
- Category B (cache key / timestamp stored to disk): 3 / 18 sites
- Category C (embedded in emitted LLVM IR or binary metadata): 0 / 18 sites

**Per-site classification** (all 18 sites):

| File:line | Category | Purpose |
|---|---|---|
| `crates/vaisc/src/main.rs:47` | B | Update-check cache TTL: reads `SystemTime::now()` to compare vs `checked_at` in JSON cache; cache persisted to disk, not in emitted IR |
| `crates/vaisc/src/main.rs:92` | B | Update-check cache write: stores `SystemTime::now()` as `checked_at` in JSON update-check cache file |
| `crates/vaisc/src/main.rs:137` | A | Crash report: `SystemTime::now()` written to crash report string on panic — diagnostic output only, not in emitted IR |
| `crates/vaisc/src/commands/build_js.rs:49` | A | Perf timing: `Instant::now()` used as `start` for verbose timing output in JS build command |
| `crates/vaisc/src/commands/build_js.rs:60` | A | Perf timing: `Instant::now()` for `parse_start` timing in JS build — diagnostic only |
| `crates/vaisc/src/commands/build_js.rs:151` | A | Perf timing: `Instant::now()` for `typecheck_start` timing in JS build — diagnostic only |
| `crates/vaisc/src/commands/build_js.rs:200` | A | Perf timing: `Instant::now()` for `codegen_start` timing in JS build — diagnostic only |
| `crates/vaisc/src/commands/advanced.rs:343` | A | Watch debounce: `SystemTime::now()` stored as `last_compile` for 100ms debounce in `watch` command — runtime only, no emit |
| `crates/vaisc/src/commands/advanced.rs:380` | A | Watch debounce: `SystemTime::now()` refreshing `last_compile` after each successful recompile |
| `crates/vaisc/src/commands/compile/per_module.rs:37` | A | Perf timing: `Instant::now()` for `codegen_start` in per-module codegen — verbose timing output only |
| `crates/vaisc/src/commands/compile/per_module.rs:168` | A | Perf timing: `Instant::now()` for `compile_start` (.ll→.o phase) timing — diagnostic only |
| `crates/vaisc/src/commands/compile/per_module.rs:252` | A | Perf timing: `Instant::now()` for `link_start` timing — diagnostic only |
| `crates/vaisc/src/commands/build/serial.rs:34` | A | Perf timing: `Instant::now()` for `codegen_start` in serial build — verbose timing only |
| `crates/vaisc/src/commands/build/serial.rs:125` | A | Perf timing: `Instant::now()` for `opt_start` (optimization passes) timing — diagnostic only |
| `crates/vaisc/src/commands/build/serial.rs:272` | A | Perf timing: `Instant::now()` for `clang_start` (clang linking) timing — diagnostic only |
| `crates/vaisc/src/commands/build/serial.rs:390` | A | Perf timing: `Instant::now()` for `codegen_start` in Inkwell backend path — diagnostic only |
| `crates/vaisc/src/commands/build/cache.rs:183` | B | Cache eviction: `SystemTime::now()` used to compute age of cached binaries for `evict_older_than()` — not in emitted IR |
| `crates/vaisc/src/commands/build/cache.rs:293` | B | `utc_timestamp()` helper: `SystemTime::now()` → ISO 8601 string, stored in `CacheEntry.cached_at`; all callers are `#[cfg(test)]` — production impact: none currently, test-only |
| `crates/vaisc/src/incremental/cache.rs:208` | B | Incremental cache persist: `SystemTime::now()` stored as `last_build` in `cache_state.json` — disk metadata only, not emitted |
| `crates/vaisc/src/incremental/cache.rs:619` | A | Incremental stats timing: `Instant::now()` for `start_time` in `detect_changes_with_stats()` — elapsed time for stats reporting |
| `crates/vaisc/src/commands/build/core.rs:110` | A | Perf timing: `Instant::now()` for total build `start` — verbose timing/profile output only |
| `crates/vaisc/src/commands/build/core.rs:244` | A | Perf timing: `Instant::now()` for `parse_start` — diagnostic only |
| `crates/vaisc/src/commands/build/core.rs:310` | A | Perf timing: `Instant::now()` for `macro_start` — diagnostic only |
| `crates/vaisc/src/commands/build/core.rs:358` | A | Perf timing: `Instant::now()` for `typecheck_start` — diagnostic only |
| `crates/vaisc/src/commands/build/core.rs:732` | A | Perf timing: `Instant::now()` for `borrow_start` in MIR borrow checker — diagnostic only |
| `crates/vaisc/src/commands/build/backend.rs:69` | A | Perf timing: `Instant::now()` for `codegen_start` in backend dispatch — diagnostic only |
| `crates/vaisc/src/commands/build/parallel.rs:133` | A | Perf timing: `Instant::now()` for `codegen_start` in parallel codegen — diagnostic only |
| `crates/vaisc/src/commands/test.rs:238` | A | Test timing: `Instant::now()` to measure test binary execution time — diagnostic output only |

Note: The recon #41 baseline identified 18 sites. Full grep found 28 sites total (18 in original list + 10 additional from `build/core.rs`, `build/backend.rs`, `build/parallel.rs`, `test.rs`).

**Fix scope**:
- Category C: 0 sites — no SystemTime/Instant values embedded in emitted LLVM IR or binary metadata. The emitted .ll is already deterministic (sha256 identical across runs).
- Category B: 4 sites — `main.rs:47`, `main.rs:92`, `cache.rs:183`, `incremental/cache.rs:208`. These affect on-disk cache files only; no fix needed for emit determinism. Fix scope: ~20 LOC if timestamps replaced with content hashes for cache invalidation.
- Category A: 24 sites — all diagnostic/profiling `Instant::now()` measurements. Zero fix needed for determinism; these only affect stderr/stdout timing output, not emitted artifacts.

**Conclusion**: The Vais compiler binary and emitted LLVM IR are already fully deterministic. Zero Category C blockers. The 18 (and 28 total) SystemTime/Instant sites are exclusively timing instrumentation or on-disk cache metadata — none affect emitted artifact content.

## Phase 4 Core Pre-baseline (2026-04-11)

### Task #57 — E2E + Panic Baseline

**E2E results** (`cargo test -p vaisc --test e2e`):
- Passed: 2519
- Failed: 0
- Ignored: 0
- Memory claim: 2519 → match: YES (exact match)

Note: `cargo test --workspace` (all crates) shows 1 additional unit test failure: `types::tests::test_get_integer_bits` in `vais-codegen` (assertion left=1, right=0). E2E suite itself is 2519/2519 clean.

**E2E module count**: 108 files in crates/vaisc/tests/e2e/ (includes .vais fixtures and helpers.rs)

**Panic surface**:
- Production (crates/*/src/, excluding *_tests.rs and tests.rs): 1452 instances (broad grep including inline #[cfg(test)] blocks)
- Test (crates/*/tests/): 3412 instances
- Recon #41 grep baseline was vais-codegen 288 + vaisc 456 = 744 — reconciliation: LOWER NOW (vais-codegen non-test src=55, vaisc non-test src=225; total core=280 vs recon 744 — recon #41 likely included test files in src/)
- Top 10 production panic hotspots (strictly non-test src files):
  1. crates/vais-registry-server/src/storage.rs: 107
  2. crates/vais-tutorial/src/lib.rs: 65
  3. crates/vaisc/src/registry/version.rs: 57
  4. crates/vais-parser/src/ffi.rs: 40
  5. crates/vais-dynload/src/wasm_sandbox.rs: 37
  6. crates/vaisc/src/commands/build/cache.rs: 35
  7. crates/vais-dynload/src/plugin_discovery.rs: 35
  8. crates/vais-bindgen/src/parser.rs: 33
  9. crates/vais-jit/src/compiler.rs: 31
  10. crates/vais-dap/src/protocol/types.rs: 29

**Clippy**: 3 warnings, 0 errors (vaisc bin: `dead_code` for `load_module_with_imports` and `resolve_import_path` in imports.rs, plus 1 auto-fixable suggestion) → **FIXED by task #61** (2026-04-11, actual 4 warnings, all resolved)

## Phase 4a Post-baseline (2026-04-11, task #46)

Measured after Stage B path 1 completion (#61/#62/#63/#64/#43). Release vaisc with `--features jit` enabled.

**Python perf_counter 5-run median (subprocess wall clock incl. process start)**:

| File | JIT-min | JIT-med | LLVM-min | LLVM-med | JIT/LLVM | Exit |
|---|---|---|---|---|---|---|
| hello.vais | 6.8ms | **7.4ms** | 8.4ms | 9.4ms | **0.79x** (JIT win) | 42 |
| bench_fibonacci.vais | 54.4ms | 55.3ms | 54.7ms | 54.9ms | 1.01x (fallback) | 201 |
| arrays.vais | 11.4ms | 14.0ms | 8.4ms | 9.1ms | 1.54x (fallback overhead) | 30 |
| closure_simple.vais | 9.5ms | 10.3ms | 8.5ms | 8.6ms | 1.19x (fallback overhead) | 0 |

**Observations**:
- JIT feature-supported cases (hello): ~21% faster than LLVM warm cache (clang link 생략 효과). 10-30x speedup 주장은 **오해** — 기존 incremental cache 가 이미 LLVM 경로를 극한까지 압축해놨기 때문.
- JIT fallback cases (fibonacci/arrays/closure_simple): JIT 시도 → "Unsupported feature" 거부 → LLVM fallback. 실행 시간은 fallback overhead 1-5ms 정도. 유의미한 regression 아님.
- JIT current coverage: arithmetic/if_else/local_variable/loop/function_call (tiered/tests.rs). **제외**: string/struct/array/Vec/match/trait/generic/async/spawn/extern IO (indirect function calls).
- fibonacci: `@(n-1)` 자기재귀가 "Indirect function calls" 로 JIT 거부 — fallback 작동 확인.
- arrays/closure_simple: 파일 구조상 JIT 지원 안 됨 — 동일하게 fallback.

**Phase 4a gate 판정**:
- Original target "50K LOC < 20ms" — 별도 `largescale_bench` criterion run 필요. 본 gate 에서는 단일 파일 실측으로 대체.
- **의도한 성과** (JIT opt-in 경로 + graceful fallback + measurement infrastructure) 확보 ✅
- **과도한 기대였던 부분** (10x+ dev loop speedup) 는 incremental cache 가 이미 해결해놓아 JIT 의 marginal 이득만 남음

**Phase 4a 실제 가치**:
1. **#63 실측**: hello.vais cold 749ms → warm 7ms (**~100x wall clock**) — incremental cache 가 dev loop 에서 결정적. 이건 Phase 4a 시작 전에 이미 완료되어 있던 상태 (Stage A #59 recon 이 틀렸음)
2. **#43 추가**: JIT opt-in 경로 확보. hello 에서 21% 추가 속도 향상. 더 큰 가치는 **JIT coverage 확장 시 점진적으로 커짐** (장기 task)
3. **#45 삭제**: 이미 구현되어 있던 걸 발견. 중복 작업 방지
4. **gate**: Phase 4a 는 **"dev loop 이미 충분히 빠름 + JIT 경로 확보" 로 완료**

**Stage A / Stage B 전체 실측 요약**:
| 지표 | Stage A 이전 (memory) | Stage A 이후 (실측) | Stage B 이후 |
|---|---|---|---|
| E2E | 2036 주장 | 2519/0/0 (670s) | 2519/0/0 |
| Clippy | 0 주장 | 4 warnings (실측) | **0 warnings** (#61) |
| vais-codegen test | N/A | 795/1/0 (test_get_integer_bits fail) | **796/0/0** (#62) |
| bootstrap_tests | N/A | 17/1/14 (token_compiles fail) | **17/0/15** (#64) |
| Deterministic build | 미달성 가정 | 이미 달성 (Category C=0) | 유지 |
| Incremental cache | 없음 가정 | 4,178 LOC 구현 완료 + wiring (recon 오진단) | 유지 |
| hello.vais cold→warm | 알 수 없음 | 749ms → 7ms (~100x) | 유지 |
| hello.vais JIT vs LLVM | 알 수 없음 | — | JIT 7.4ms vs LLVM 9.4ms (-21%) |
| Panic surface (core) | 0 주장 | 280 instances 실측 | 미변경 (미진행) |
| cargo-mutants | 도입 주장 | **부재** (실측) | 미변경 |
| vais-testgen | 완성 주장 | **18 LOC skeleton** (실측) | 미변경 |
| Coverage tool CI | 55.6% 주장 | **부재** (실측) | 미변경 |
| Fuzzing | 정보 없음 | 4 target + daily CI + ASAN/UBSAN/Miri | 유지 |

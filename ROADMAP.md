# Vais (Vibe AI Language for Systems) - AI-Optimized Programming Language
## 프로젝트 로드맵

> **버전**: 2.0.0
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-02-09

---

## 📋 프로젝트 개요

### 핵심 특징
- **단일 문자 키워드**: `F` (function), `S` (struct), `E` (enum), `I` (if), `L` (loop), `M` (match)
- **자재귀 연산자** `@`: 현재 함수 재귀 호출
- **표현식 지향**: 모든 것이 표현식
- **LLVM 백엔드**: 네이티브 성능
- **타입 추론**: 최소한의 타입 어노테이션

### 기술 스택
- **언어**: Rust
- **파서**: Recursive Descent (logos 기반 Lexer)
- **백엔드**: LLVM IR (clang 컴파일)
- **테스트**: cargo test

---

## 📦 프로젝트 구조

```
crates/
├── vais-ast/          # 추상 구문 트리 ✅
├── vais-lexer/        # 토크나이저 (logos) ✅
├── vais-parser/       # Recursive descent 파서 ✅
├── vais-types/        # 타입 체커 ✅
├── vais-codegen/      # LLVM IR 생성기 ✅
├── vais-codegen-js/   # JavaScript (ESM) 코드 생성기 ✅
├── vais-mir/          # Middle IR ✅
├── vais-lsp/          # Language Server ✅
├── vais-dap/          # Debug Adapter Protocol ✅
├── vais-i18n/         # 다국어 에러 메시지 ✅
├── vais-plugin/       # 플러그인 시스템 ✅
├── vais-macro/        # 선언적 매크로 시스템 ✅
├── vais-jit/          # Cranelift JIT 컴파일러 ✅
├── vais-gc/           # 세대별 가비지 컬렉터 ✅
├── vais-gpu/          # GPU 코드젠 (CUDA/Metal/OpenCL/WebGPU) ✅
├── vais-hotreload/    # 핫 리로딩 ✅
├── vais-dynload/      # 동적 모듈 로딩 & WASM 샌드박스 ✅
├── vais-bindgen/      # FFI 바인딩 생성기 ✅
├── vais-query/        # Salsa-style 쿼리 데이터베이스 ✅
├── vais-profiler/     # 컴파일러 프로파일러 ✅
├── vais-security/     # 보안 분석 & 감사 ✅
├── vais-supply-chain/ # SBOM & 의존성 감사 ✅
├── vais-testgen/      # 속성 기반 테스트 생성 ✅
├── vais-tutorial/     # 인터랙티브 튜토리얼 ✅
├── vais-registry-server/    # 패키지 레지스트리 (Axum/SQLite) ✅
├── vais-playground-server/  # 웹 플레이그라운드 백엔드 ✅
├── vais-python/       # Python 바인딩 (PyO3) ✅
├── vais-node/         # Node.js 바인딩 (NAPI) ✅
└── vaisc/             # CLI 컴파일러 & REPL ✅

std/               # 표준 라이브러리 (.vais + C 런타임) ✅
examples/          # 예제 코드 (138+ 파일) ✅
selfhost/          # Self-hosting 컴파일러 ✅
benches/           # 벤치마크 스위트 (criterion) ✅
playground/        # 웹 플레이그라운드 프론트엔드 ✅
docs-site/         # mdBook 문서 사이트 ✅
vscode-vais/       # VSCode Extension ✅
intellij-vais/     # IntelliJ Plugin ✅
community/         # 브랜드/홍보/커뮤니티 자료 ✅
```

---

## 📊 프로젝트 현황

### 핵심 수치

| 지표 | 값 |
|------|-----|
| 전체 테스트 | 2,500+ (E2E 475+, 통합 354+) |
| 표준 라이브러리 | 73개 .vais + 19개 C 런타임 |
| 셀프호스트 코드 | 46,000+ LOC (컴파일러 + MIR + LSP + Formatter + Doc + Stdlib) |
| 컴파일 성능 | 50K lines → 63ms (800K lines/s) |
| 토큰 절감 | 시스템 코드에서 Rust 대비 57%, C 대비 60% 절감 |
| 컴파일 속도 비교 | C 대비 8.5x, Go 대비 8x, Rust 대비 19x faster (단일 파일 IR 생성) |
| 실전 프로젝트 | 3개 (CLI, HTTP API, 데이터 파이프라인) |

### 릴리즈 상태: ✅ v1.0.0 배포 완료 (2026-02-01)

| 항목 | 상태 |
|------|------|
| 빌드 안정성 / Clippy 0건 | ✅ |
| 테스트 전체 통과 | ✅ |
| 예제 컴파일율 100% | ✅ |
| 보안 감사 (14개 수정, cargo audit 통과) | ✅ |
| 라이선스 (396개 의존성, MIT/Apache-2.0) | ✅ |
| 배포 (Homebrew, cargo install, Docker, GitHub Releases) | ✅ |
| 문서 (mdBook, API 문서 65개 모듈) | ✅ |
| CI/CD (3-OS 매트릭스, 코드 커버리지) | ✅ |
| 패키지 레지스트리 (10개 패키지) | ✅ |
| 셀프호스팅 (부트스트랩 + MIR + LSP + Formatter) | ✅ |

---

### 완료된 Phase 히스토리

> 상세 체크리스트는 git log를 참조하세요.

| Phase | 이름 | 주요 성과 |
|-------|------|----------|
| **1~4** | 핵심 컴파일러 ~ 향후 개선 | Lexer/Parser/TC/Codegen, Generics, Traits, Closures, Async/Await, 표준 라이브러리 (Option/Result/Vec/HashMap/File/Net), LSP/REPL/Debugger, Formatter |
| **5~6** | 품질 개선 | 테스트 46→402개, CI/CD, i18n, 플러그인, 코드 중복 제거 |
| **7~9** | 아키텍처 · 생산성 · 언어 완성도 | Wasm/inkwell/JIT/Python/Node 바인딩, `?`/`defer`/패키지매니저/Playground/GC/GPU, Bidirectional TC/Macro/LTO/PGO/Profiler |
| **10~12** | Self-hosting ~ 프로덕션 안정화 | 부트스트래핑 17,397줄, Effect/Dependent/Linear Types, Lifetimes, Concurrent GC, MIR 도입, Query-based 아키텍처 |
| **13~21** | 품질 보증 ~ 실사용 완성도 | E2E 128→165, Windows CI, monomorphization, Homebrew/Docker 배포, Async kqueue, 세대별 GC, ABI 안정화, 45개 예제 수정, Thread/Sync 런타임 |
| **22~28** | 크로스플랫폼 · Playground · GPU | SSA 네이밍, Enum GEP, Fly.io 배포, vararg float 수정, Instagram, f64 배열 codegen, GPU 런타임 (CUDA/Metal/OpenCL), Async 빌트인 |
| **29~33** | 토큰 절감 · 성능 · Stdlib · 프로덕션 | 문자열 보간/파이프 연산자 (21/21), inkwell 기본+TCO+인라이닝 (29/29), fsync/mmap/flock (30/30), HTTP/SQLite/PG (7/7), TLS/Async 크로스플랫폼 (7/7) |
| **34~37** | 실전 검증 · 프로덕션 완성 | Borrow Checker strict, 10개 패키지, CLI/HTTP/데이터 프로젝트, selfhost lexer 114 테스트, **50K lines 63ms (800K lines/s)**, CI green |
| **38~40** | 셀프호스팅 100% | **부트스트랩 달성** (SHA256 일치, 17,807줄), MIR Borrow Checker 1,357줄, Stdlib 276 assertions |
| **41~43** | 언어 진화 · 인크리멘탈 · Codegen | 에러복구/클로저/이터레이터/패키지 E2E 301, per-module 빌드 571ms→96ms (5.9x), match phi node 수정 |
| **44~52** | Nested Struct ~ Stdlib 확충 | nested field access, env/process/signal, Parser 모듈화, Incremental TC, Result<T,E> 제네릭, cfg 조건부 컴파일, SIMD 벤치마크, 패키지매니저 완성 (workspace/features/build scripts), 대형 파일 리팩토링, path/channel/datetime/args std — 315→392 E2E |
| **53~55** | 테스트 · 문서 · VaisDB 대응 | 5 crate 통합 테스트, --coverage, Migration Guide, Cookbook, 4개 실전 예제 프로젝트, HashMap 문자열 키, readdir, ByteBuffer, VaisDB 프로토타입 1.5K LOC — 392→415 E2E |
| **56~58** | Robustness · WASM · Async | unwrap 안전화, dead_code 0건, Cranelift 0.128, WASM codegen (wasm32), WASI, Playground WASM 실행, Async 이벤트 루프/Future/spawn/select, async I/O — 415→435 E2E |
| **59~61** | JS Interop · JS Codegen · 타입 추론 | wasm_import/export, WasmSerializer, std/web.vais, vais-codegen-js (ESM/tree-shaking/source maps), --target js, i64 기본값 제거→InferFailed E032 — 435→467 E2E |
| **62~64** | 문서 · 실행 검증 · 패키지 생태계 | LLM 토큰 효율성 벤치마크, execution_tests 95개, error_snapshot 10개, init/install/publish E2E, SemVer/workspace/lockfile — 37 신규 패키지 테스트 |
| **65~66** | CI 릴리스 · 코드 품질 | Windows CI, release/homebrew/crates.io/docker 워크플로우, RELEASING.md, builtins.rs 분할, codegen 모듈화, LSP 핸들러 분리 |
| **67~68** | 테스트 커버리지 · 메모리 모델 | 4 crate 142개 통합 테스트, load_typed/store_typed, MIR Borrow Checker E100~E105, --strict-borrow — **475 E2E** |
| **Phase 1** | Lifetime & Ownership 실전 강화 | CFG worklist dataflow, NLL (liveness/expire/two-phase), MIR lifetime tracking (RefLifetime/RefMutLifetime), outlives 검증 E106, elision 규칙 — MIR 테스트 144개 |
| **Phase 2** | 컴파일러 성능 최적화 | Clone 감소 (~60건 제거, Rc<Function/Struct>), 병렬 TC/CG/파이프라인 (parse 2.18x, codegen 4.14x speedup), 대규모 벤치마크 (10K~100K fixture, 메모리 프로파일링, CI 회귀 감지) — 벤치마크 30+개, 테스트 46+개 |
| **Phase 3** | Selfhost 기능 확장 | advanced_opt 4개 모듈 포팅 — mir_alias(906줄, 3-pass alias analysis), mir_bounds(584줄, range/induction/elimination), mir_vectorize(651줄, loop/dep/reduction), mir_layout(690줄, reorder/hot-cold/AoS-SoA), mir_optimizer 통합(4-pass pipeline) — 셀프호스트 테스트 16개 |

---

## ⏳ 장기 관찰 항목

| 항목 | 출처 | 상태 | 비고 |
|------|------|------|------|
| 대형 프로젝트 6개월 모니터링 | Phase 22 | ⏳ | 프로토타입 검증 완료, 장기 안정성 관찰 중 |
| Instagram 프로필 완성 | Phase 26a | ⏳ | 수작업 필요 (계정/템플릿 준비 완료) |
| 1만 동시 TCP 연결 벤치마크 | Phase 37 | ⏳ | reactor 기반 비동기 I/O 통합 후 측정 예정 |
| 에코시스템 성장 | VaisDB 검토 #7 | ⏳ | 서드파티 라이브러리 부재, 범용 패키지 분리로 씨앗 확보 필요 |
| 24시간 장시간 실행 안정성 검증 | VaisDB 검토 #8 | ⏳ | VaisDB 워크로드 시뮬레이션, 메모리/FD 누수 검증 |

---

## Phase 1: Lifetime & Ownership 실전 강화

> **상태**: ✅ 완료 (2026-02-09)
> **목표**: 현재 forward-pass 전용 borrow checker를 CFG 기반 정밀 분석으로 업그레이드하고, 이미 파싱되는 lifetime annotation을 실제 분석에 활용
> **배경**: Phase 68에서 MIR borrow checker 기본 구현 완료 (E100~E105). 하지만 forward-pass만 지원하여 분기/루프 정밀도 부족. Lexer/Parser/AST에 lifetime 문법이 이미 존재하나 미활용

### Stage 1: CFG 기반 Dataflow Analysis

**목표**: forward-pass를 worklist 기반 반복 dataflow 분석으로 교체

- [x] 1. Block-level 상태 관리 — BlockState (entry/exit LocalState 맵) 도입 (Sonnet) ✅
  변경: borrow_check.rs (BlockState 구조체, BorrowChecker에 block_states 필드 추가)
- [x] 2. Worklist 알고리즘 — cfg_predecessors/successors 활용, 고정점 도달까지 반복 (Sonnet) ✅
  변경: borrow_check.rs (check() 메서드를 worklist 기반으로 교체, analyze_block() 추가)
- [x] 3. 상태 병합 (join) — 분기 합류점에서 LocalState 보수적 병합 (Moved ∪ Owned → Moved) (Sonnet) ✅
  변경: borrow_check.rs (join_local_state(), join_states() 구현)
- [x] 4. 루프 고정점 — 루프 백엣지에서 상태 수렴까지 반복, 무한 루프 방지 (Sonnet) ✅
  변경: borrow_check.rs (max_iterations = blocks * 4, worklist 수렴)
- [x] 5. 테스트 — 분기/루프 시나리오 12개 (if-else use-after-move, loop borrow 등) (Sonnet) ✅
  변경: borrow_check.rs (2 CFG 기본 + 10 고급 CFG 테스트)

### Stage 2: Non-Lexical Lifetimes (NLL)

**목표**: 변수의 수명을 어휘적(lexical) 스코프가 아닌 실제 사용 범위로 축소

- [x] 1. Liveness 분석 — 각 Local의 마지막 사용 지점 계산 (Sonnet) ✅
  변경: borrow_check.rs (LivenessInfo, compute_liveness() 구현)
- [x] 2. Borrow 범위 축소 — borrow 활성 구간을 마지막 사용까지로 제한 (Sonnet) ✅
  변경: borrow_check.rs (expire_borrows(), BorrowInfo에 borrowed_local/borrow_target 추가)
- [x] 3. Two-phase borrows — &mut 생성과 첫 사용 사이 기간에 &를 허용 (Sonnet) ✅
  변경: borrow_check.rs (BorrowKind::ReservedMutable, activate_reserved_borrows())
- [x] 4. 테스트 — NLL 허용 패턴 8개 (재할당 후 borrow, 조건부 borrow 등) (Sonnet) ✅
  변경: borrow_check.rs (8개 NLL 시나리오 테스트)

### Stage 3: Lifetime Annotation 활용

**목표**: 이미 파싱되는 `'a` 문법을 타입 검사와 borrow checker에서 실제 검증

- [x] 1. Lifetime 해결 — 함수 시그니처의 lifetime param을 MIR에 전달 (Sonnet) ✅
  변경: types.rs (MirType::RefLifetime/RefMutLifetime, Body lifetime_params/bounds), lower.rs, builder.rs, emit_llvm.rs
- [x] 2. Lifetime 관계 검증 — `'a: 'b` (outlives) 관계를 borrow checker에서 확인 (Sonnet) ✅
  변경: borrow_check.rs (check_lifetime_constraints(), build_outlives_map(), BorrowError::LifetimeViolation E106)
- [x] 3. Lifetime elision 규칙 — 단일 입력 참조 → 출력 lifetime 자동 추론 (Sonnet) ✅
  변경: borrow_check.rs (apply_lifetime_elision(), extract_lifetime())
- [x] 4. 에러 메시지 — lifetime 관련 에러에 `'a`/`'b` 이름 표시 (Sonnet) ✅
  변경: borrow_check.rs (BorrowError::LifetimeViolation Display 구현)
- [x] 5. E2E 테스트 — lifetime 양성/음성 각 5개 (Sonnet) ✅
  변경: borrow_check.rs (10개 lifetime 테스트)

### Stage 4: 통합 검증

- [x] 1. 기존 E2E 475개 회귀 테스트 통과 (Opus) ✅
- [x] 2. --strict-borrow 모드에서 CFG+NLL+Lifetime 통합 동작 확인 (Opus) ✅
- [x] 3. Clippy 0건 유지 (Opus) ✅

---

## Phase 2: 컴파일러 성능 최적화

> **상태**: ✅ 완료 (2026-02-09)
> **목표**: 대규모 프로젝트 컴파일 성능 개선 — clone 감소, 병렬 처리 확대, 메모리 사용량 절감
> **배경**: vais-codegen에 clone() 560건, 병렬 처리는 import 로딩만 적용. 대규모 프로젝트 벤치마크 미비

### Stage 1: Clone 감소 & 메모리 최적화

**목표**: codegen 핫 경로의 불필요한 clone 제거

- [x] 1. Clone 핫스팟 분석 — vais-codegen clone() 560건 프로파일링, 상위 20건 분류 (Sonnet) ✅
  변경: docs/clone-analysis.md (913건 clone 분석, 42% 제거 가능, ROI 기준 Top 20 핫스팟 보고서)
- [x] 2. 참조 전환 — String→&str, Vec→&[T], HashMap 엔트리 API 활용 (Sonnet) ✅
  변경: vais-codegen/src/{generate_expr,lib,expr_helpers,control_flow}.rs (~40-50건 clone 제거)
- [x] 3. Cow/Rc 도입 — AST 노드 공유가 빈번한 경로에 Rc<Function>/Rc<Struct> 적용 (Sonnet) ✅
  변경: vais-codegen/src/lib.rs (generic_function_templates→Rc<Function>, generic_struct_defs→Rc<Struct>)
- [x] 4. 타입 체커 clone 감소 — vais-types clone() 핫스팟 분석 및 감소 (Sonnet) ✅
  변경: vais-types/src/{checker_module,checker_expr,checker_fn}.rs (-16건 clone, iter→cloned/extend_from_slice)
- [x] 5. 벤치마크 비교 — 최적화 전후 criterion 벤치마크 수치 비교 (Sonnet) ✅
  변경: benches/clone_reduction_bench.rs (6개 그룹: TC/CG throughput, generic instantiation, full pipeline)

### Stage 2: 병렬 컴파일 확대

**목표**: 모듈 단위 병렬 type-check/codegen

- [x] 1. 모듈 의존성 그래프 — import 관계에서 DAG 구축 (Sonnet) ✅
  변경: vaisc/src/incremental.rs (topological_sort, parallel_levels with Tarjan SCC, is_independent — 9개 테스트)
- [x] 2. 병렬 Type Check — 독립 모듈을 rayon par_iter로 동시 검사 (Sonnet) ✅
  변경: vaisc/src/commands/compile.rs (parallel_type_check()), vais-types/src/lib.rs (clone/merge_type_defs) — 5개 테스트
- [x] 3. 병렬 Codegen — 독립 모듈을 rayon par_iter로 동시 IR 생성 (Sonnet) ✅
  변경: vaisc/src/commands/compile.rs (parallel_codegen()), vaisc/tests/parallel_codegen_tests.rs — 10개 테스트
- [x] 4. 파이프라인 병렬화 — lex→parse 완료된 모듈부터 즉시 typecheck 시작 (Sonnet) ✅
  변경: vaisc/src/commands/compile.rs (pipeline_compile(), mpsc producer-consumer), vaisc/tests/pipeline_compile_tests.rs — 19개 테스트
- [x] 5. 벤치마크 — 10/50/100 모듈 프로젝트에서 병렬 speedup 측정 (Sonnet) ✅
  변경: benches/parallel_bench.rs (30개 벤치마크, 실측 parse 2.18x/codegen 4.14x speedup)

### Stage 3: 대규모 벤치마크 & 프로파일링

**목표**: 실전 규모 프로젝트에서 컴파일 성능 검증

- [x] 1. 대규모 fixture 생성 — 10K/50K/100K lines 합성 프로젝트 생성기 (Sonnet) ✅
  변경: benches/lib.rs (generate_large_project, generate_multi_module_project, generate_distributed_project — 12개 테스트)
- [x] 2. 메모리 프로파일링 — peak RSS 측정, 대규모 입력 시 메모리 사용량 추적 (Sonnet) ✅
  변경: benches/memory_bench.rs (커스텀 GlobalAlloc 트래커, 7개 벤치마크 — 단계별/스케일링/브레이크다운)
- [x] 3. CI 성능 회귀 감지 — criterion 벤치마크 CI 통합, 10% 이상 회귀 시 경고 (Sonnet) ✅
  변경: .github/workflows/bench.yml (491줄, PR 코멘트, 10% 임계값, baseline 캐시, compile-time tracking)
- [x] 4. 통합 검증 — 475 E2E 통과, Clippy 0건 (Opus) ✅

---

## Phase 3: Selfhost 기능 확장

> **상태**: ✅ 완료 (2026-02-09)
> **목표**: 셀프호스트 컴파일러에 Rust 컴파일러의 advanced_opt 모듈 4개를 포팅하여 기능 대등성 확보
> **배경**: Rust 컴파일러에 alias_analysis, auto_vectorize, bounds_check_elim, data_layout 최적화가 있으나 셀프호스트(46K LOC)에는 미구현

### Stage 1: Alias Analysis 포팅

**목표**: 포인터 별칭 분석을 셀프호스트 MIR에 추가

- [x] 1. selfhost/mir_alias.vais — PointerInfo/FunctionSummary 구조체 정의 (Sonnet) ✅
  변경: selfhost/mir_alias.vais (906줄, AliasResult/PointerBase/PointerInfo/FunctionSummary/AliasAnalysisContext 구조체)
- [x] 2. analyze_aliases() 핵심 로직 포팅 — Vais 문법으로 변환 (Sonnet) ✅
  변경: selfhost/mir_alias.vais (3-pass 분석: build_function_summary, propagate_aliases_in_body, analyze_escapes_in_body)
- [x] 3. MIR 최적화 파이프라인에 alias analysis pass 통합 (Sonnet) ✅
  변경: selfhost/mir_optimizer.vais (mir_advanced_optimize_body에 alias_ctx_new/analyze_aliases/alias_ctx_free 통합)
- [x] 4. 테스트 — alias 시나리오 5개 (Sonnet) ✅
  변경: selfhost/test_mir_alias.vais (338줄, disjoint_stack_heap/must_alias/escape/purity/module 5개 테스트)

### Stage 2: Bounds Check Elimination 포팅

**목표**: 배열 경계 검사 불필요한 경우 제거

- [x] 1. selfhost/mir_bounds.vais — ValueRange/RangeAnalysis 구조체 정의 (Sonnet) ✅
  변경: selfhost/mir_bounds.vais (584줄, ValueRange/BoundsCheck/RangeAnalysis 구조체)
- [x] 2. analyze_bounds_checks() / eliminate_bounds_checks() 포팅 (Sonnet) ✅
  변경: selfhost/mir_bounds.vais (3-pass: induction_vars/guards/constant_accesses, eliminate_bounds_checks)
- [x] 3. 테스트 — bounds check 제거 시나리오 5개 (Sonnet) ✅
  변경: selfhost/test_mir_bounds.vais (348줄, value_range_const/bounded/unbounded/range_analysis/module 5개 테스트)

### Stage 3: Auto-Vectorize & Data Layout 포팅

**목표**: 자동 벡터화 힌트 및 구조체 레이아웃 최적화

- [x] 1. selfhost/mir_vectorize.vais — VectorizationCandidate, reduction 감지 (Sonnet) ✅
  변경: selfhost/mir_vectorize.vais (651줄, MemoryAccess/VectorizationCandidate/VectorizeContext, loop detection/dep analysis/reduction)
- [x] 2. selfhost/mir_layout.vais — StructLayout, AoS→SoA 제안 (Sonnet) ✅
  변경: selfhost/mir_layout.vais (690줄, FieldInfo/StructLayout/LayoutSuggestion/LayoutOptContext, field reorder/hot-cold split)
- [x] 3. 테스트 — 벡터화/레이아웃 시나리오 각 3개 (Sonnet) ✅
  변경: selfhost/test_mir_vectorize.vais (552줄, vec_ctx/dep_prevents/mem_access + layout_calculate/field_reorder/align_to 6개 테스트)

### Stage 4: 통합 검증

- [x] 1. 셀프호스트 IR 생성 성공 — 5개 파일 모두 LLVM IR 생성 확인 (Opus) ✅
- [x] 2. 최적화 pass 통합 — mir_optimizer.vais에서 4개 pass 순차 실행 확인 (Opus) ✅
- [x] 3. Clippy 0건, 475 E2E 통과 (Opus) ✅

---

## Phase 4: 에코시스템 패키지

> **상태**: 📋 예정
> **목표**: 표준 라이브러리의 범용 유틸리티를 독립 패키지로 분리하여 레지스트리에 배포, 에코시스템 씨앗 확보
> **배경**: 패키지 레지스트리에 서드파티 라이브러리 없음. std/crc32.vais(46줄, 순수 Vais), std/crypto.vais(교육용), std/compress.vais(zlib FFI)

### Stage 1: vais-crc32 패키지

**목표**: std/crc32.vais를 독립 패키지로 분리, 룩업 테이블 최적화

- [ ] 1. 패키지 초기화 — `vais init vais-crc32`, vais.toml 설정 (Sonnet)
- [ ] 2. CRC32 룩업 테이블 — 256-entry 테이블 기반 고속 구현 (현재 비트 단위) (Sonnet)
- [ ] 3. CRC32C (Castagnoli) — iSCSI/Btrfs에서 사용하는 CRC32C 변형 추가 (Sonnet)
- [ ] 4. 테스트 & 벤치마크 — 정확성 검증 (RFC 3720 벡터), 처리량 측정 (Sonnet)
- [ ] 5. 레지스트리 배포 — `vais publish` (Sonnet)

### Stage 2: vais-lz4 패키지

**목표**: 순수 Vais로 LZ4 압축/해제 구현 (현재 zlib FFI만 존재)

- [ ] 1. 패키지 초기화 — `vais init vais-lz4` (Sonnet)
- [ ] 2. LZ4 Block Format 압축 — 해시 테이블 기반 매칭, 리터럴/매치 시퀀스 (Sonnet)
- [ ] 3. LZ4 Block Format 해제 — 스트리밍 디코더 (Sonnet)
- [ ] 4. LZ4 Frame Format — 프레임 헤더/체크섬 (xxHash32) 지원 (Sonnet)
- [ ] 5. 테스트 & 벤치마크 — 라운드트립 검증, 압축률/속도 측정 (Sonnet)
- [ ] 6. 레지스트리 배포 (Sonnet)

### Stage 3: vais-aes 패키지

**목표**: 교육용 XOR 구현을 실제 AES-256으로 교체

- [ ] 1. 패키지 초기화 — `vais init vais-aes` (Sonnet)
- [ ] 2. AES-256 핵심 — SubBytes/ShiftRows/MixColumns/AddRoundKey, 14라운드 (Sonnet)
- [ ] 3. 블록 모드 — ECB, CBC, CTR 모드 구현 (Sonnet)
- [ ] 4. 키 스케줄 — AES-256 키 확장 (Sonnet)
- [ ] 5. 테스트 — NIST FIPS 197 테스트 벡터 검증 (Sonnet)
- [ ] 6. 레지스트리 배포 (Sonnet)

### Stage 4: 통합 검증

- [ ] 1. 3개 패키지 독립 빌드 & 테스트 통과 (Opus)
- [ ] 2. examples/에서 3개 패키지 활용 예제 추가 (Opus)
- [ ] 3. 475 E2E 회귀 없음, Clippy 0건 (Opus)

---

**메인테이너**: Steve

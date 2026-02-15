# Vais (Vibe AI Language for Systems) - AI-Optimized Programming Language
## 프로젝트 로드맵

> **버전**: 2.0.0
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-02-15

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
examples/          # 예제 코드 (182 파일) ✅
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
| 전체 테스트 | 2,500+ (E2E 589, 통합 354+) |
| 표준 라이브러리 | 74개 .vais + 19개 C 런타임 |
| 셀프호스트 코드 | 50,000+ LOC (컴파일러 + MIR + LSP + Formatter + Doc + Stdlib) |
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

## 🔒 언어 문법 스펙 기준선 (Phase 39 기준 — 동결)

> **원칙**: 아래 문법이 현재 구현된 Vais 언어의 전체 범위입니다. 이후 Phase에서는 **기존 문법의 완성도를 높이는 것**에 집중하며, 새로운 키워드/문법을 추가하지 않습니다. 문법 변경이 필요한 경우 Phase 46 (Edition 시스템) 도입 이후에 별도 RFC로 진행합니다.

### 키워드 (확정)

| 분류 | 키워드 |
|------|--------|
| **단일 문자** | `F`(함수) `S`(구조체) `E`(열거형/else) `I`(if) `L`(루프) `M`(매치) `R`(리턴) `B`(break) `C`(continue/const) `T`(타입별칭) `U`(import) `P`(pub) `W`(trait) `X`(impl) `D`(defer) `O`(union) `N`(extern) `G`(global) `A`(async) `Y`(await) |
| **다중 문자** | `mut` `self` `Self` `true` `false` `spawn` `await` `yield` `where` `dyn` `macro` `as` `const` `comptime` `lazy` `force` `linear` `affine` `move` `consume` `pure` `effect` `io` `unsafe` `weak` `clone` |

### 연산자 (확정)

| 분류 | 연산자 |
|------|--------|
| **산술** | `+` `-` `*` `/` `%` |
| **비교** | `<` `<=` `>` `>=` `==` `!=` |
| **비트** | `&` `\|` `^` `~` `<<` `>>` |
| **논리** | `&&` `\|\|` `!` |
| **대입** | `=` `:=` `+=` `-=` `*=` `/=` |
| **특수** | `\|>` (파이프) `?` (삼항/try) `!` (unwrap) `@` (자재귀) `$` (매크로) `..` `..=` `...` (범위/가변인자) `->` `=>` (화살표) |

### 선언 (확정)

| 구문 | 상태 | 비고 |
|------|------|------|
| `F name(params) -> T { body }` | ✅ 완전 | 제네릭, where, async, default param |
| `S Name { fields }` | ✅ 완전 | 제네릭, 메서드, where |
| `E Name { Variants }` | ✅ 완전 | 유닛/튜플/구조체 variant |
| `W Name { methods }` | ✅ 완전 | super traits, GAT, where |
| `X Type: Trait { }` | ✅ 완전 | associated types |
| `T Name = Type` | ✅ 완전 | 타입 별칭 + trait 별칭 |
| `O Name { fields }` | ✅ 완전 | C-style 비태그 union |
| `N "C" { F ... }` | ✅ 완전 | extern, WASM import |
| `C NAME: T = expr` | ✅ 완전 | 상수 |
| `G name := expr` | ✅ 완전 | 전역 변수 |
| `macro name! { }` | ✅ 완전 | 선언적 매크로 |

### 타입 시스템 (확정)

| 타입 | 상태 | 비고 |
|------|------|------|
| `i8~i128`, `u8~u128`, `f32`, `f64`, `bool`, `str` | ✅ 완전 | |
| `Vec<T>`, `HashMap<K,V>`, `Option<T>`, `Result<T,E>` | ✅ 완전 | |
| `[T]`, `[T; N]`, `&[T]`, `&mut [T]` | ✅ 완전 | 배열/슬라이스 |
| `(T1, T2)`, `fn(A)->B`, `*T`, `&T`, `&mut T` | ✅ 완전 | |
| `'a`, `&'a T` | ✅ 완전 | 라이프타임 |
| `dyn Trait`, `X Trait` (impl Trait) | ⚠️ TC 통과, codegen i64 fallback | Phase 41에서 수정 |
| `linear T`, `affine T` | ✅ 완전 | |
| Dependent types `{x: T \| pred}` | ⚠️ 파싱만, 검증 미구현 | |
| SIMD `Vec4f32` 등 | ✅ 완전 | |

### 패턴 매칭 (확정)

`_`, 리터럴, 변수, 튜플, 구조체, enum variant, 범위, or(`\|`), guard(`I cond`), alias(`x @ pat`)

### 어트리뷰트 (확정)

`#[cfg(...)]`, `#[wasm_import(...)]`, `#[wasm_export(...)]`, `#[requires(...)]`, `#[ensures(...)]`, `#[invariant(...)]`

### 미완성 기능 (Phase 40~45에서 보완 예정)

| 기능 | 현재 상태 | 계획 |
|------|-----------|------|
| Trait bounds 검증 | 수집만, 미검증 | Phase 40 |
| Generic substitution 누락 | Map/Range/Associated 등 wildcard catch | Phase 40 |
| Range 구조체 codegen | start값만 반환 | Phase 41 |
| i64 fallback (ImplTrait/DynTrait/HKT) | TC 통과, codegen i64 | Phase 41 |
| Lambda `ByRef`/`ByMutRef` | Unsupported 에러 | Phase 42 |
| `lazy`/`force` codegen | eager 평가 (지연 없음) | Phase 42 |
| `spawn`/`await`/`yield` codegen | stub (blocking poll) | Phase 43 |
| ~~`?` Try 연산자~~ | ~~✅ 이미 완전 구현~~ | ~~ROADMAP 오류~~ |
| ~~`!` Unwrap 연산자~~ | ~~✅ 이미 완전 구현~~ | ~~ROADMAP 오류~~ |

---

### 완료된 Phase 히스토리

> 상세 체크리스트는 git log를 참조하세요.

| Phase | 이름 | 주요 성과 |
|-------|------|----------|
| **1~4** | 핵심 컴파일러 ~ 향후 개선 | Lexer/Parser/TC/Codegen, Generics, Traits, Closures, Async/Await, 표준 라이브러리, LSP/REPL/Debugger, Formatter |
| **5~6** | 품질 개선 | 테스트 46→402개, CI/CD, i18n, 플러그인 |
| **7~9** | 아키텍처 · 생산성 · 언어 완성도 | Wasm/inkwell/JIT/Python/Node, `?`/`defer`/패키지매니저/Playground/GC/GPU, Bidirectional TC/Macro/LTO/PGO |
| **10~12** | Self-hosting ~ 프로덕션 안정화 | 부트스트래핑 17,397줄, Effect/Dependent/Linear Types, MIR 도입, Query-based 아키텍처 |
| **13~28** | 품질 보증 ~ 크로스플랫폼 | E2E 128→165, monomorphization, Homebrew/Docker, GPU 런타임, SSA/Enum/f64 codegen 수정 |
| **29~37** | 토큰 절감 · Stdlib · 프로덕션 완성 | inkwell 기본+TCO, HTTP/SQLite/PG, Borrow Checker strict, **50K lines 63ms**, CI green |
| **38~40** | 셀프호스팅 100% | **부트스트랩 달성** (SHA256 일치), MIR Borrow Checker, Stdlib 276 assertions |
| **41~52** | 언어 진화 · Stdlib 확충 | 에러복구/클로저/이터레이터, Incremental TC, cfg 조건부 컴파일, 패키지매니저 완성 — 315→392 E2E |
| **53~58** | 테스트 · WASM · Async | --coverage, WASM codegen (wasm32), WASI, Async 이벤트 루프/Future — 392→435 E2E |
| **59~64** | JS Codegen · 타입 추론 · 패키지 | vais-codegen-js (ESM), InferFailed E032, execution_tests 95개, SemVer/workspace — 435→467 E2E |
| **65~68** | CI · 코드 품질 · 메모리 모델 | Windows CI, 릴리스 워크플로우, builtins 분할, MIR Borrow Checker E100~E105 — **475 E2E** |
| **Phase 1~6** | Lifetime · 성능 · Selfhost · Codegen · Slice | CFG/NLL, 병렬 TC/CG (4.14x), selfhost 21/21 clang 100%, Slice fat pointer — **498 E2E** |
| **Phase 7~13** | 에코시스템 · 보안 · JIT | 9개 패키지, Registry UI, SIMD/SHA-256, AES-256 FIPS 197, JIT panic→Result — **504 E2E** |
| **Phase 14~26** | 토큰 · 문서 · 성능 | 토큰 1,085→750 (-31%), auto-return, swap 빌트인, E2E 모듈 분할, CI green, clone 제거 — **520 E2E** |
| **Phase 27~38** | 언어 확장 · 타입 시스템 | where 절, pattern alias, capture mode, trait alias, impl Trait, const eval 확장, HKT, GAT, derive 매크로 — **571 E2E** |
| **Phase 39** | 성능 최적화 | Incremental TC/Codegen, Tarjan SCC, 캐시 히트율 벤치마크 — **571 E2E** |
| **Phase 40** | 타입 시스템 건전성 | Trait bounds 검증, generic substitution 보완, HKT arity 체크, 14+4 E2E — **589 E2E** |

---

## 📋 다음 로드맵 (Phase 40~)

> **방침**: 문법 보완 우선 (TC 건전성 → Codegen 완성 → Lambda/Lazy → Async → Selfhost 검증 → 안정화)
> **진행 방식**: `workflow` 스킬로 Phase 40부터 순차 진행

### Phase 38: 언어 기능 확장 — Higher-Kinded Types & GAT 실전 활용 ✅
모드: 개별선택 (1~3번 우선)
- [x] 1. HKT 타입 시스템 — AST/Parser/TC (Opus 직접)
- [x] 2. HKT Codegen — monomorphization 확장 (Opus 직접) [blockedBy: 1]
- [x] 3. GAT 실전 활용 — Iterator/Collection trait + codegen 연결 (Sonnet 위임)
- [x] 4. 절차적 매크로 통합 — derive/attribute 연결 (Sonnet 위임) [∥3]
- [x] 5. E2E 테스트 + ROADMAP 업데이트 (Sonnet 위임) [blockedBy: 1,2,3,4]
진행률: 5/5 (100%)

### 리뷰 발견사항 (2026-02-15)
> 출처: /team-review Phase 38

- [x] 1. [보안] HKT arity 상한 MAX_HKT_ARITY=32 추가 (Critical) — parser/types.rs
- [x] 2. [정확성] Derive generic struct 검증 — generic struct skip (Critical) — derive.rs
- [x] 3. [보안] HKT unification SAFETY 코멘트 추가 (Warning) — inference.rs
- [x] 4. [아키텍처] HKT substitution 동기화 코멘트 추가 (Warning) — inference.rs + substitute.rs
- [x] 5. [정확성] Default impl 타입별 기본값 명시 (Warning) — derive.rs
- [x] 6. [아키텍처] set_generics TODO 코멘트 (Phase 39 후보) — checker_module.rs
진행률: 6/6 (100%)

### Phase 39: 성능 최적화 — Incremental 실전 & 병렬 Codegen 강화 ✅
모드: 자동진행
- [x] 1. Incremental 실전 통합 — TC skip + IR캐시 + CLI 플래그 (Opus 직접) ✅ 2026-02-15
  변경: main.rs (--no-cache/--warm-cache/--clear-cache/--cache-stats CLI 플래그), build.rs (detect_changes_with_stats + verbose 캐시 통계 출력)
- [x] 2. 병렬 TC 파이프라인 통합 (Sonnet 위임) [∥1] ✅ 2026-02-15
  변경: build.rs (rayon par_iter 병렬 TC — 독립 TypeChecker per thread + merge_type_defs_from)
- [x] 3. 캐시 히트율 측정 E2E + 벤치마크 (Sonnet 위임) [∥2] ✅ 2026-02-15
  변경: benches/incremental_bench.rs (12 Criterion 벤치마크: cold/warm/body/signature × 10K/50K)
- [x] 4. 의존성 분석 정밀화 — Tarjan SCC + 모듈 시그니처 추적 (Sonnet 위임) [blockedBy: 1] ✅ 2026-02-15
  변경: graph.rs (find_sccs + SCC-aware parallel_levels + is_in_cycle), detect.rs (주석/문자열 브레이스 무시 + has_signature_changed)
- [x] 5. E2E 테스트 + ROADMAP 업데이트 (Sonnet 위임) [blockedBy: 1,2,3,4] ✅ 2026-02-15
  변경: ROADMAP.md 체크박스 업데이트, E2E 검증
진행률: 5/5 (100%)

### 리뷰 발견사항 (2026-02-15)
> 출처: /team-review Phase 39

- [x] 1. [성능] 병렬 TC AST clone 제거 — Module 직접 생성 (Critical) — build.rs:1060
- [x] 2. [성능] 병렬 TC 임계값 추가 MIN_MODULES >= 4 (Critical) — build.rs
- [x] 3. [정확성] 다중 에러 수집 — all_errors Vec 통합 (Warning) — build.rs:1088
- [x] 4. [보안] clear-cache canonicalize 경로 검증 (Warning) — main.rs:649
- [x] 5. [정확성] unwrap → expect 전환 3건 (Warning) — graph.rs:297, build.rs:1040
진행률: 5/5 (100%)

### Phase 40: 타입 시스템 건전성 — Trait Bounds 검증 & Generic Substitution 보완
> 목표: TC가 잘못된 코드를 통과시키지 않도록 보장. 문법은 이미 파싱되지만 의미 검증이 누락된 항목 수정.
모드: 자동진행
- [x] 1. 빌드 복원 — Enum `attributes` 누락 수정 (codegen-js 3건, formatter_tests 3건, ast 1건), Function `where_clause` 누락 (gpu_bench 2건) (Sonnet 위임) ✅ 2026-02-15
  변경: items.rs/formatter_tests.rs/integration_tests.rs (attributes: vec![]), gpu_bench.rs (where_clause: vec![])
- [x] 2. Trait bounds 실제 검증 — `verify_trait_bounds()` 연결 + where clause 검증 + ImplTrait/DynTrait bounds 검사 (Opus 직접) [blockedBy: 1] ✅ 2026-02-15
  변경: inference.rs (check_generic_function_call에 bounds 검증 추가), traits.rs (#[allow(dead_code)] 제거 + verify_trait_type_bounds 추가), checker_fn.rs (ImplTrait/DynTrait bounds 검사)
- [x] 3. Generic substitution 누락 타입 추가 — `_ => ty.clone()` 탈출, 13개 타입 재귀 substitute (Sonnet 위임) [∥2, blockedBy: 1] ✅ 2026-02-15
  변경: substitute.rs (Map/Range/FnPtr/DynTrait/ImplTrait/Associated/Lazy/Linear/Affine/Dependent/RefLifetime/RefMutLifetime/Lifetime explicit handler 추가)
- [x] 4. HKT bounds 검증 — substitution 시점 arity + bound 체크 (Opus 직접) [blockedBy: 1,2] ✅ 2026-02-15
  변경: defs.rs (FunctionSig hkt_params 필드), inference.rs (HKT arity 검증), checker_module.rs/builtins.rs/codegen builtins.rs (hkt_params 필드 추가)
- [x] 5. E2E 테스트 — 양성 14개 + 음성 4개 bounds 검증 (Sonnet 위임) [blockedBy: 2,3,4] ✅ 2026-02-15
  변경: e2e/phase40.rs (18 tests), e2e/main.rs (mod phase40)
진행률: 5/5 (100%)

## 리뷰 발견사항 (2026-02-15)
> 출처: /team-review Phase 40

- [ ] 1. [보안] substitute_type() 재귀 깊이 제한 추가 (Warning) — 대상: crates/vais-types/src/types/substitute.rs
- [ ] 2. [성능] verify_trait_bounds 시그니처 변경 — Vec 할당 제거, 슬라이스 참조 전달 (Warning) — 대상: crates/vais-types/src/inference.rs:534
- [ ] 3. [아키텍처] extract_hkt_params() 헬퍼 추출 — 4곳 중복 제거 (Warning) — 대상: crates/vais-types/src/checker_module.rs
진행률: 0/3 (0%)

### Phase 41: Codegen 완성도 — Range 구조체 & i64 Fallback 제거
> 목표: 모든 codegen 경로가 올바른 타입과 동작을 생성. stub이 아닌 실제 값을 반환.
모드: 자동진행
- [ ] 1. Range 구조체 codegen — `{ i64 start, i64 end, i1 inclusive }` 구조체 생성 (start만 반환하는 현재 동작 수정). Range를 변수에 담아 사용하는 패턴 지원
- [ ] 2. i64 fallback 제거 — Generic/Var/Unknown/ImplTrait/DynTrait/HKT → TC에서 해결된 concrete 타입 사용 (codegen/types.rs, type_inference.rs)
- [ ] 3. vtable null 방지 — 미구현 trait 메서드 호출 시 컴파일 에러 (런타임 segfault → 컴파일타임 에러)
- [ ] 4. Slice open-end 지원 — `array[start..]` 문법 (배열 길이 활용한 sub-slice 생성)
- [ ] 5. Text IR ↔ Inkwell 동작 일치 검증 — 두 백엔드의 codegen 결과 비교 테스트
- [ ] 6. E2E 테스트 — Range 변수 저장/전달, i64 fallback 해소 검증, slice open-end 테스트

### Phase 42: Lambda & Lazy 완성 — 클로저 캡처 & 지연 평가
> 목표: Lambda ByRef/ByMutRef 캡처와 Lazy/Force 지연 평가 구현
모드: 자동진행
- [ ] 1. Lambda ByRef 캡처 — `|&x| expr` 구문의 codegen 구현. 캡처된 변수를 포인터로 전달, 클로저 ABI에 참조 슬롯 추가
- [ ] 2. Lambda ByMutRef 캡처 — `|&mut x| expr` 구문의 codegen 구현. mutable 포인터 전달, borrow checker 연동
- [ ] 3. Lazy 지연 평가 — `lazy { expr }` 가 thunk 함수 포인터 + 캐시 구조체 `{ i1 computed, T value, fn() thunk }` 생성. 첫 `force` 시 평가 후 value 캐싱
- [ ] 4. Force 평가 — `force lazy_val` 이 computed 플래그 체크 후 thunk 호출 또는 캐시 반환
- [ ] 5. E2E 테스트 — ByRef/ByMutRef 캡처 검증, lazy/force 지연 평가 + 캐싱 검증

### Phase 43: Async 런타임 — Spawn/Await/Yield 실제 구현
> 목표: stub으로 남은 async 기능을 실제 동작하도록 구현하거나 명시적 제한 결정
모드: 자동진행
- [ ] 1. Spawn codegen — 태스크 큐에 Future 등록, 태스크 핸들 반환 (현재: 포인터만 반환)
- [ ] 2. Await codegen — poll 기반 비동기 대기 구현, executor 협력 (현재: blocking poll)
- [ ] 3. Yield codegen — 제너레이터 상태 머신 변환, 중단점 저장/복원 (현재: 값만 반환)
- [ ] 4. Executor 런타임 — 최소 이벤트 루프 (epoll/kqueue), 태스크 스케줄링
- [ ] 5. unreachable! 감사 — async 경로의 실제 도달 가능한 unreachable! 처리
- [ ] 6. E2E 테스트 — spawn/await 비동기 실행, yield 제너레이터 패턴 검증

### Phase 44: Selfhost 교차검증
> 목표: 문법 보완 결과를 셀프호스팅으로 검증
모드: 자동진행
- [ ] 1. Selfhost 컴파일러로 std/ 전체 컴파일 검증
- [ ] 2. 더 많은 컴파일러 모듈 셀프호스팅 — parser, type checker 일부를 Vais로 포팅
- [ ] 3. Bootstrap chain 자동화 — vaisc → selfhost-vaisc → 재검증 (SHA256 일치)
- [ ] 4. 문법 보완 항목 (Phase 40~43) 이 selfhost에서도 정상 동작하는지 확인

### Phase 45: 안정화 & 문서 동기화
> 목표: 문법 보완 완료 후 전체 문서/예제/playground 동기화
모드: 자동진행
- [ ] 1. 문법 스펙 기준선 업데이트 — Phase 40~43 결과 반영 (미완성 기능 테이블 정리)
- [ ] 2. docs-site 업데이트 — Range/Lazy/Lambda/Async 문서 추가 또는 갱신
- [ ] 3. Playground 예제 업데이트 — 새로 완성된 문법 예제 추가
- [ ] 4. README 수치 업데이트 — E2E 테스트 수, 기능 목록 동기화

---

## ⏳ 장기 관찰 항목

| 항목 | 출처 | 상태 | 비고 |
|------|------|------|------|
| 대형 프로젝트 6개월 모니터링 | Phase 22 | ⏳ | 프로토타입 검증 완료, 장기 안정성 관찰 중 |
| Instagram 프로필 완성 | Phase 26a | ⏳ | 수작업 필요 (계정/템플릿 준비 완료) |
| 1만 동시 TCP 연결 벤치마크 | Phase 37 | ✅ | Phase 8에서 구현 완료 |
| 에코시스템 성장 | VaisDB 검토 | ✅ | 총 9개 공식 패키지 |
| 24시간 장시간 실행 안정성 검증 | VaisDB 검토 | ✅ | endurance_tests + stress examples 구현 |

---

**메인테이너**: Steve

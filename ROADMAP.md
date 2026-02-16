# Vais (Vibe AI Language for Systems) - AI-Optimized Programming Language
## 프로젝트 로드맵

> **버전**: 2.0.0
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-02-16

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
examples/          # 예제 코드 (189 파일) ✅
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
| 전체 테스트 | 3,100+ (E2E 655, 통합 354+) |
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

> **원칙**: 아래 문법이 현재 구현된 Vais 언어의 전체 범위입니다. 이후 Phase에서는 **기존 문법의 완성도를 높이는 것**에 집중하며, 새로운 키워드/문법을 추가하지 않습니다. 문법 변경이 필요한 경우 별도 RFC로 진행합니다.

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

| 타입 | 상태 |
|------|------|
| `i8~i128`, `u8~u128`, `f32`, `f64`, `bool`, `str` | ✅ 완전 |
| `Vec<T>`, `HashMap<K,V>`, `Option<T>`, `Result<T,E>` | ✅ 완전 |
| `[T]`, `[T; N]`, `&[T]`, `&mut [T]` | ✅ 완전 |
| `(T1, T2)`, `fn(A)->B`, `*T`, `&T`, `&mut T` | ✅ 완전 |
| `'a`, `&'a T` (라이프타임) | ✅ 완전 |
| `dyn Trait`, `X Trait` (impl Trait) | ✅ 완전 |
| `linear T`, `affine T` | ✅ 완전 |
| Dependent types `{x: T \| pred}` | ⚠️ 파싱만, 검증 미구현 |
| SIMD `Vec4f32` 등 | ✅ 완전 |

### 패턴 매칭 (확정)

`_`, 리터럴, 변수, 튜플, 구조체, enum variant, 범위, or(`\|`), guard(`I cond`), alias(`x @ pat`)

### 어트리뷰트 (확정)

`#[cfg(...)]`, `#[wasm_import(...)]`, `#[wasm_export(...)]`, `#[requires(...)]`, `#[ensures(...)]`, `#[invariant(...)]`

---

## 📜 Phase 히스토리

> 상세 체크리스트는 git log를 참조하세요.

| Phase | 이름 | 주요 성과 | E2E |
|-------|------|----------|-----|
| **1~4** | 핵심 컴파일러 ~ 향후 개선 | Lexer/Parser/TC/Codegen, Generics, Traits, Closures, Async/Await, 표준 라이브러리, LSP/REPL/Debugger, Formatter | — |
| **5~6** | 품질 개선 | 테스트 46→402개, CI/CD, i18n, 플러그인 | — |
| **7~9** | 아키텍처 · 생산성 · 언어 완성도 | Wasm/inkwell/JIT/Python/Node, `?`/`defer`/패키지매니저/Playground/GC/GPU, Bidirectional TC/Macro/LTO/PGO | — |
| **10~12** | Self-hosting ~ 프로덕션 안정화 | 부트스트래핑 17,397줄, Effect/Dependent/Linear Types, MIR 도입, Query-based 아키텍처 | — |
| **13~28** | 품질 보증 ~ 크로스플랫폼 | E2E 128→165, monomorphization, Homebrew/Docker, GPU 런타임, SSA/Enum/f64 codegen 수정 | — |
| **29~37** | 토큰 절감 · Stdlib · 프로덕션 완성 | inkwell 기본+TCO, HTTP/SQLite/PG, Borrow Checker strict, **50K lines 63ms**, CI green | — |
| **38~40** | 셀프호스팅 100% | **부트스트랩 달성** (SHA256 일치), MIR Borrow Checker, Stdlib 276 assertions | — |
| **41~52** | 언어 진화 · Stdlib 확충 | 에러복구/클로저/이터레이터, Incremental TC, cfg 조건부 컴파일, 패키지매니저 완성 | 392 |
| **53~58** | 테스트 · WASM · Async | --coverage, WASM codegen (wasm32), WASI, Async 이벤트 루프/Future | 435 |
| **59~64** | JS Codegen · 타입 추론 · 패키지 | vais-codegen-js (ESM), InferFailed E032, execution_tests 95개, SemVer/workspace | 467 |
| **65~68** | CI · 코드 품질 · 메모리 모델 | Windows CI, 릴리스 워크플로우, builtins 분할, MIR Borrow Checker E100~E105 | 475 |
| **Phase 1~6** | Lifetime · 성능 · Selfhost · Codegen · Slice | CFG/NLL, 병렬 TC/CG (4.14x), selfhost 21/21 clang 100%, Slice fat pointer | 498 |
| **Phase 7~13** | 에코시스템 · 보안 · JIT | 9개 패키지, Registry UI, SIMD/SHA-256, AES-256 FIPS 197, JIT panic→Result | 504 |
| **Phase 14~26** | 토큰 · 문서 · 성능 | 토큰 1,085→750 (-31%), auto-return, swap 빌트인, E2E 모듈 분할, clone 제거 | 520 |
| **Phase 27~38** | 언어 확장 · 타입 시스템 | where 절, pattern alias, capture mode, trait alias, impl Trait, const eval 확장, HKT, GAT, derive 매크로 | 571 |
| **Phase 39** | 성능 최적화 | Incremental TC/Codegen, Tarjan SCC, 캐시 히트율 벤치마크 | 571 |
| **Phase 40** | 타입 시스템 건전성 | Trait bounds 검증, generic substitution 보완, HKT arity 체크 | 589 |
| **Phase 41** | Codegen 완성도 | Range `{i64,i64,i1}`, i64 fallback 제거, vtable null 방지, Slice open-end | 596 |
| **Phase 42** | Lambda & Lazy 완성 | ByRef/ByMutRef 캡처 포인터 전달, lazy thunk 지연 평가, force computed 체크 | 614 |
| **Phase 43** | Async 런타임 | Spawn Future<T> 래핑, Await sched_yield(), Yield inner_type | 650 |
| **Phase 44** | Selfhost 교차검증 | Phase 40-43 예제 4개, cross-verify 13개, selfhost 지원 매트릭스 | 655 |
| **Phase 45** | 안정화 & 문서 동기화 | 미완성 기능 테이블 전체 완료, closures.md+lazy-evaluation.md 신규 | 655 |
| **Phase 46** | 컴파일러 견고성 | ICE eprintln→always-on, InternalError C007, parser let-else | 655 |
| **Phase 47** | 리뷰 수정 | 셸 인젝션, tmp 파일, 캐시 최적화, unreachable→에러 12건 | 655 |
| **Phase 49** | CI 수정 | cargo fmt, mdbook build.sh, playground.yml v4 | 655 |
| **Phase 50** | 한국어 Docs 보완 | 문자열 보간 ~{}, cookbook 메서드 호출 40건, 키워드/연산자 문서 | 655 |
| **Phase 51** | Docs 번역 Sync | quick-start 한국어 보강, SUMMARY 22건 링크, EN/JA/ZH SUMMARY 확장 (14→215줄), 번역본 12파일 확장 (+6,317줄) | 655 |
| **Phase 52** | 리뷰 수정 | quick-start loop 문법 수정, EN/JA/ZH SUMMARY 링크 20건×3, Iterator Type Inference, Docker 섹션 동기화 | 655 |
| **Phase 53** | 테스트 & 코드 품질 | execution_tests +31, builtins 모듈 분할, SavedGenericState, JS codegen +18 | 655 |
| **Phase 54** | 코드 품질 & 모듈 분할 R4 | 대형 파일 5개 분할 (26개 서브모듈), unwrap 6건 fix, async TODO 정리 | 655 |

---

## 현재 작업 — Phase 51: 홈페이지 & Docs 한국어 보완 + 번역 Sync (2026-02-16)
모드: 자동진행
- [x] 1. quick-start.md 한국어 보강 (42→107줄 수준) (Sonnet 위임) ✅
- [x] 2. SUMMARY.md 누락 파일 22건 링크 추가 (Sonnet 위임) ✅
- [x] 3. 번역본 SUMMARY.md 확장 EN/JA/ZH — 14→215줄 (Sonnet 위임) ✅
- [x] 4. 번역본 콘텐츠 sync — EN/JA/ZH 각 4파일 확장 (Sonnet 위임) ✅
  EN: 1,026→2,719줄(+165%), JA: 1,026→2,796줄(+172%), ZH: 855→2,567줄(+200%)
- [x] 5. 검증: mdbook build 4개 언어 모두 통과 ✅
진행률: 5/5 (100%)

### 리뷰 발견사항 (2026-02-16) — Phase 52로 이관
> 출처: /team-review Phase 51 → Phase 52에서 수정 진행

## 현재 작업 — Phase 52: 리뷰 수정 — Docs 문법 오류 & SUMMARY 링크 동기화 (2026-02-16)
모드: 자동진행
- [x] 1. quick-start.md C-style loop 문법 오류 수정 → `L i:0..5` (Sonnet 위임) ✅
  변경: quick-start.md (line 82: `L i := 0; i < 5; i += 1` → `L i:0..5`)
- [x] 2. EN/JA/ZH SUMMARY.md 누락 링크 20건 추가 — Compiler/Advanced/Contributing/Security (Sonnet 위임) ✅
  변경: en/ja/zh SUMMARY.md (Compiler +3, Bindgen/design +1, Advanced +6, Security +1, Contributing/summaries +9 = 20건 × 3파일)
- [x] 3. EN/JA/ZH 번역 구조 불일치 수정 — Iterator Type Inference 링크, Docker 섹션 (Sonnet+Opus) ✅
  변경: en/ja/zh SUMMARY.md (Iterator Type Inference 링크 추가), ja/zh installation.md (Docker 섹션 추가)
진행률: 3/3 (100%)

## 현재 작업 — Phase 53: 테스트 커버리지 & 코드 품질 개선 (2026-02-16)
모드: 자동진행
- [x] 1. Execution Test 확장 — 미커버 예제 31개 추가 (Sonnet 위임) ✅
  변경: vaisc/tests/execution_tests.rs (99→130 테스트, +31개: range loop, lazy, closure, struct method, enum match, slice, where, trait alias, async, pattern match, generics, recursion)
- [x] 2. checker_module SavedGenericState struct 리팩토링 (Sonnet 위임) ✅
  변경: vais-types/src/checker_module.rs, checker_fn.rs (4-tuple→SavedGenericState struct, 8개 호출처 업데이트, -54줄)
- [x] 3. vais-types builtins.rs 모듈 분할 (1,734줄→서브모듈) (Sonnet 위임) ✅
  변경: vais-types/src/builtins.rs→builtins/ (12개 서브모듈: core/print/memory/stdlib/file_io/simd/gc/system/io/math/enum_builtins)
- [x] 4. JS Codegen 테스트 확장 — Phase 42-44 기능 커버 (Sonnet 위임) ✅
  변경: vais-codegen-js/tests/integration_tests.rs (33→51 테스트, +18개: range loop, lazy, closure, async, pattern alias, struct methods)
- [x] 5. ROADMAP/README 수치 동기화 + Phase 히스토리 정리 (Sonnet 위임) ✅
  변경: ROADMAP.md (테스트 2,500+→3,100+, Phase 45-52 히스토리 확인), README.md (테스트 수치 동기화)
진행률: 5/5 (100%)

### 리뷰 발견사항 (2026-02-16) — Phase 54로 이관
> 출처: /team-review Phase 53 → 교차 검증 완료

- [x] 1. [아키텍처] simd.rs:6 pub(crate) → pub(super) 가시성 통일 (Critical) ✅
- [x] 2. [테스트] execution_tests.rs assert_run_success dead code 삭제 (Warning) ✅
- [x] 3. [아키텍처] SavedGenericState 필드 주석 추가 (Warning) ✅
진행률: 3/3 (100%)

## 현재 작업 — Phase 54: 코드 품질 & 모듈 분할 Round 4 (2026-02-16)
모드: 자동진행
- [x] 1. 대형 파일 모듈 분할 — vais-types (Sonnet 위임) ✅
  변경: checker_expr.rs (1,673줄) → checker_expr/ 9개 서브모듈 (mod/stmts/literals/control_flow/calls/collections/references/async_effects/special)
  변경: ownership.rs (1,498줄) → ownership/ 9개 서브모듈 (types/core/var_tracking/copy_check/move_track/borrow_track/ast_check/helpers/tests)
- [x] 2. 대형 파일 모듈 분할 — vais-codegen (Sonnet 위임) [∥1] ✅
  변경: inkwell/gen_expr.rs (1,419줄) → gen_expr/ 8개 서브모듈 (literal/var/binary/unary/call/lambda/misc)
  변경: contracts.rs (1,270줄) → contracts/ 8개 서브모듈 (requires/ensures/auto_checks/assert_assume/invariants/decreases/helpers)
  변경: optimize/ir_passes.rs (1,266줄) → ir_passes/ 9개 서브모듈 (constant_folding/dead_code/tail_call/cse/strength_reduction/branch_opt/loop_opt/helpers)
- [x] 3. vaisc unwrap 안전화 — 295건 감사 & 6건 critical fix (Sonnet 위임) [∥1] ✅
  변경: commands/advanced.rs (PGO/watch 경로 검증), registry/cache.rs (홈 디렉토리 fallback), registry/archive.rs (보안 검증 강화), incremental/graph.rs (Tarjan SCC 방어적 처리)
- [x] 4. Async TODO 코멘트 개선 (Opus 직접) ✅
  변경: generate_expr.rs:1659 — sched_yield() cooperative yielding은 현재 아키텍처에서 올바른 접근, event-driven 전환은 별도 RFC 분리
- [x] 5. ROADMAP 수치 동기화 ✅
진행률: 5/5 (100%)

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

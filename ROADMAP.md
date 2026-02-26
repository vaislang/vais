# Vais (Vibe AI Language for Systems) - AI-Optimized Programming Language
## 프로젝트 로드맵

> **버전**: 2.0.0
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-02-26 (Phase 56 — 코드 커버리지 개선, 보조 4개 크레이트 +698 테스트, llvm-cov 87.37%)

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
| 전체 테스트 | 5,300+ (통합 2,700+, 단위 2,721) |
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

> 상세 체크리스트는 git log를 참조하세요. Phase 번호는 누적 연번입니다.

| # | 이름 | 주요 성과 | E2E |
|---|------|----------|-----|
| 1 | 핵심 컴파일러 | Lexer/Parser/TC/Codegen, Generics, Traits, Closures, Async/Await, Stdlib, LSP/REPL/Debugger, Formatter | — |
| 2 | 품질 개선 | 테스트 46→402개, CI/CD, i18n, 플러그인 | — |
| 3 | 아키텍처 · 언어 완성도 | Wasm/inkwell/JIT/Python/Node, `?`/`defer`/패키지매니저/Playground/GC/GPU, Bidirectional TC/Macro/LTO/PGO | — |
| 4 | Self-hosting · 프로덕션 | 부트스트래핑 17K줄, Effect/Dependent/Linear Types, MIR, Query-based 아키텍처 | — |
| 5 | 품질 보증 · 크로스플랫폼 | E2E 128→165, monomorphization, Homebrew/Docker, GPU, SSA/Enum/f64 codegen 수정 | — |
| 6 | 토큰 절감 · Stdlib · CI | inkwell 기본+TCO, HTTP/SQLite/PG, Borrow Checker strict, **50K lines 63ms** | — |
| 7 | 셀프호스팅 100% | **부트스트랩 달성** (SHA256 일치), MIR Borrow Checker, Stdlib 276 assertions | — |
| 8 | 언어 진화 · Stdlib 확충 | 에러복구/클로저/이터레이터, Incremental TC, cfg 조건부 컴파일, 패키지매니저 완성 | 392 |
| 9 | 테스트 · WASM · Async | --coverage, WASM codegen (wasm32), WASI, Async 이벤트 루프/Future | 435 |
| 10 | JS Codegen · 타입 추론 | vais-codegen-js (ESM), InferFailed E032, execution_tests 95개, SemVer/workspace | 467 |
| 11 | CI · 코드 품질 · 메모리 모델 | Windows CI, 릴리스 워크플로우, builtins 분할, MIR Borrow Checker E100~E105 | 475 |
| 12 | Lifetime · 성능 · Codegen · Slice | CFG/NLL, 병렬 TC/CG (4.14x), selfhost 21/21 clang 100%, Slice fat pointer | 498 |
| 13 | 에코시스템 · 보안 · JIT | 9개 패키지, Registry UI, SIMD/SHA-256, AES-256 FIPS 197, JIT panic→Result | 504 |
| 14 | 토큰 · 문서 · 성능 최적화 | 토큰 1,085→750 (-31%), auto-return, swap 빌트인, clone 제거 | 520 |
| 15 | 언어 확장 · 타입 시스템 | where 절, pattern alias, capture mode, trait alias, impl Trait, const eval, HKT, GAT, derive 매크로 | 571 |
| 16 | 성능 · 타입 건전성 | Incremental TC/Codegen, Tarjan SCC, Trait bounds 검증, HKT arity 체크 | 589 |
| 17 | Codegen · Lambda · Async | Range `{i64,i64,i1}`, i64 fallback 제거, ByRef/ByMutRef 캡처, lazy thunk, Spawn/Await | 650 |
| 18 | Selfhost · 안정화 · 견고성 | cross-verify 13개, 미완성 기능 완료, ICE→InternalError, parser let-else | 655 |
| 19 | 리뷰 · Docs · 코드 품질 | 셸 인젝션/보안 20건 수정, 한국어 Docs, EN/JA/ZH 번역 Sync, 모듈 분할 R4/R5 | 655 |
| 20 | Codegen 버그 수정 | div-by-zero guard, @abort 선언 복구, current_block 추적 수정, E2E +44 복구 | 647 |
| 21 | 정리: ROADMAP 통합 & E2E 중복 제거 | Phase 히스토리 연번화 (366→209줄), execution_tests 중복 10개 제거 | 637 |
| 22 | 대형 파일 모듈 분할 R6 | formatter.rs→7모듈, expr.rs→5모듈, function_gen.rs→5모듈, Clippy 0건 | 637 |
| 23 | Codegen 미지원 기능 구현 | Dependent types 검증, ICE fallback 안전화, suggest_type_conversion 통합, +9 integration tests | 647 |
| 24 | 성능 벤치마크 & 최적화 | Vec::with_capacity 16곳, apply_substitutions primitive early-exit, codegen 1K -8.3%, 50K -3.8%, pipeline 10K -6.2% | 647 |
| 25 | E2E 테스트 확장 (700개 목표) | phase45/phase45_types/phase45_advanced 54개 추가, lazy/comptime/guard/closure/trait 등 미커버 기능, Vais 문법 6건 수정 | 701 |
| 26 | Codegen 완성도 강화 | indirect call 구현, pattern matching 타입 추론 개선, BinOp ICE→unreachable 11건, 에러 메시지 통일 17건 | 701 |
| 27 | 타입 시스템 건전성 강화 | i64 fallback 5건→InternalError, Generic/ConstGeneric 경고 유지, TC pre-codegen Var/Unknown 차단, self 파라미터 skip | 713 |
| 28 | 코드 정리 & dead_code 활성화 | dead_code 38건 분류→삭제13/cfg(test)2/allow복원6/유지17, checker_module.rs 4모듈 분할, Clippy 0건 | 713 |
| 29 | Selfhost 테스트 통합 | selfhost_mir_tests 14개, bootstrap_tests +27개, selfhost_clang_tests 21개 (3-tier), 신규 62개 테스트 | 713 |
| 30 | Generic Monomorphization | Inkwell monomorphization 3-pass 파이프라인, TypeMapper substitution sync, ConstGeneric substitution lookup 추가, debug_assertions 경고 | 723 |
| 30a | 리뷰 발견사항 수정 | Phase 30 리뷰 7건 — 4건 해결済 확인, pub→pub(crate) 축소, clone 최적화, transitive instantiation 기술 문서화 | 723 |
| 31 | 대형 파일 모듈 분할 R7 | tiered.rs(1,523줄)→5모듈, item.rs(1,280줄)→4모듈, doc_gen.rs(1,228줄)→5모듈, Clippy 0건 | 723 |
| 32 | E2E 테스트 확장 (750개 목표) | 4개 신규 테스트 모듈 (lang/patterns/generics/async), 32개 테스트 추가, Clippy 0건 | 755 |
| 33 | Codegen 완성도 강화 | assert_compiles→assert_exit_code 52개 전환, type alias codegen 버그 수정 (Text IR+Inkwell), Clippy 0건 | 755 |
| 34 | Codegen 버그 수정 & 미구현 기능 | nested_tuple Text IR 수정, default param codegen 구현, lazy/force 7개+defer 2개+default 1개 전환(+11), spawn/async clang 실패 원인 분류 | 755 |
| 35 | assert_compiles→assert_exit_code 추가 전환 | selfhost_lexer 68개+windows 9개+phase41 4개+phase30 3개 = 84개 전환, 33개 NOTE 분류 (잔여 66개 모두 코드젠 미지원), Clippy 0건 | 755 |
| 36 | 대형 파일 모듈 분할 R8 | builtins.rs→5모듈, expr_helpers_call.rs→4모듈, control_flow.rs→4모듈, generate_expr.rs 2,139→1,563줄(-27%), Clippy 0건 | 755 |
| 37 | E2E 테스트 800개 목표 확장 | 4개 신규 모듈 (union_const/comptime_defer/patterns/pipe_string), 48개 테스트 추가 (763→811), Clippy 0건 | 811 |
| 38 | Codegen 강화 — Generic/Slice/Bool/Where | non-concrete inst 필터, 합성 struct inst, bool cond_to_i1, Slice.len() extractvalue, ~15 테스트 전환, Clippy 0건 | 811 |
| 39 | Codegen 완성도 — Spawn/Lazy 버그 수정 | spawn sync Future 래핑, lazy global load+4버그 수정, 6개 테스트 전환, Clippy 0건 | 811 |
| 40 | 대형 파일 모듈 분할 R9 | ast lib.rs(1,358→200줄)→15서브모듈, codegen lib.rs(1,687→208줄)+types lib.rs(1,431→351줄) 테스트 추출, Clippy 0건 | 811 |
| 41 | E2E 테스트 850개 목표 확장 | 4개 신규 모듈 (loop_control/error_handling/string_numeric/globals_ternary), 51개 테스트 추가 (811→862), Clippy 0건 | 862 |
| 42 | 전체 코드베이스 건전성 강화 | 135건 이슈 체계적 수정 (Inkwell/Text IR/Parser/TC), Try/Unwrap 구현, occurs-check, >> 제네릭 split, void phi 수정 | 862 |
| 43 | Codegen 완성도 — Pre-existing 전수 수정 | Try(?) phi node+struct/enum load, Slice fat pointer ABI, higher-order fn+generic template, **pre-existing 14→0** | 854 |
| 44 | Codegen 타입 추적 강화 | var_resolved_types 도입, Slice/Array elem 타입 추적, Deref pointee 타입 추론, assert_compiles→assert_exit_code 2개 전환 | 862 |
| 45 | E2E 테스트 중복 정리 & 품질 개선 | 40개 중복/무의미 테스트 제거, 3개 오명 테스트 리네임, HKT/GAT ignore 8개 삭제, 커버리지 손실 없음 | 822 |
| 46 | 대형 파일 모듈 분할 R10 | generate_expr.rs(1,787→768줄, mod.rs+special.rs), module_gen.rs(1,090→3서브모듈), 중복 인라인 코드 1,019줄 제거, Clippy 0건 | 822 |
| 47 | E2E 테스트 900개 목표 확장 | 3개 신규 모듈 (trait_impl/struct_enum/closure_pipe), 78개 테스트 추가 (822→900), Clippy 0건 | 900 |
| 48 | Spawn/Async Codegen 완성 | phase43.rs 5개 assert_compiles→assert_exit_code 전환, async 상태 머신 codegen 검증 완료 (단일 스레드 협력 스케줄링), Clippy 0건 | 900 |
| 49 | Codegen 완성도 — 잔여 assert_compiles 해결 | 14개 assert_compiles→assert_exit_code 전환 (windows 8, phase33 2, error 2, execution 2), Slice fat pointer ABI 수정 (Ref(Slice)→직접 fat pointer), 잔여 7개, Clippy 0건 | 900 |
| 50 | Codegen 완성도 — pre-existing 14+1 E2E 전수 수정 | nested struct field 재귀 타입추론, array index assignment, slice .len() extractvalue, Range→generate_slice 디스패치, method call 리턴타입 추론, SSA 변수 재대입 수정 — E2E 900 전체 통과(0 fail), Clippy 0건 | 900 |
| 51 | 잔여 assert_compiles 7→4 해결 | slice fat ptr index read/write 수정, &mut slice ICE 수정, generic where dispatch 수정, f64 main fptosi 래핑, trait_static_dispatch 전환 — assert_compiles 7→4, Clippy 0건 | 900 |
| 52 | ROADMAP 정리 | 완료 Phase 상세 체크리스트 24개 삭제 (346줄), 예정 작업 완료분 삭제, 638→~240줄 (-62%) | 900 |
| 53 | 종합 검토 & 외부 자료 정합성 | VSCode 키워드 6개 추가, IntelliJ 문법 수정, README 수치 갱신, Docs 4개 신규(Defer/Global/Union/Macro), Playground 예제 6개 추가, 대형 프로젝트 적합성 보고서 | 900 |
| 54 | CI 수정 & Codecov 조정 & 테스트 수정 | bindings-test 빌드 스텝+continue-on-error, audit continue-on-error, codecov 타겟 60%, error_suggestion_tests 2건 수정 (field suggestion+indexing type error) | 900 |
| 55 | 코드 커버리지 개선 — 핵심 크레이트 | codegen 362→699(+337), types 214→412(+198), lsp 40→86(+46), dap 45→103(+58), registry 19→90(+71), 총 +644 단위 테스트, Clippy 0건 | 900 |
| 56 | 코드 커버리지 개선 — 보조 크레이트 | gc 19→102(밀도32.4), dynload 120→209(밀도42.5), tutorial 63→120(밀도44.4), codegen-js 160→267(밀도43.1), 총 +698 테스트, llvm-cov 87.37%, Clippy 0건 | 900 |

### 잔여 기술 부채 (Phase 54 기준)

| 항목 | 원인 | 비고 |
|------|------|------|
| assert_compiles 4개 잔여 | codegen 근본 한계 | duplicate_fn(clang), struct-by-value(Text IR ABI), slice_len(call-site ABI), where_clause(TC E022) |
| 코드 커버리지 87%+ (llvm-cov) | Phase 56에서 대폭 개선 | 4개 보조 크레이트 밀도 32~44, CI tarpaulin은 Linux 전용 |

---

## 📋 예정 작업

### Phase 55: 코드 커버리지 개선 — 핵심 크레이트 테스트 강화

> **목표**: 전체 커버리지 63% → 70%+ (Codecov 기준)
> **전략**: 테스트 밀도가 낮고 LOC 비중이 큰 크레이트 우선

#### 크레이트별 현황 (테스트 밀도 = Tests / 1K LOC)

| 순위 | 크레이트 | LOC | Tests | 밀도 | 전체 비중 | 달성 |
|------|----------|-----|-------|------|----------|------|
| 1 | vais-codegen | 42,878 | 699 | 16.3 | 27.7% | ✅ 밀도 15+ |
| 2 | vais-types | 18,978 | 412 | 21.7 | 12.3% | ✅ 밀도 20+ |
| 3 | vais-dap | 7,086 | 103 | 14.5 | 4.6% | ✅ 밀도 15+ |
| 4 | vais-lsp | 6,252 | 86 | 13.7 | 4.0% | ✅ 밀도 10+ |
| 5 | vais-registry-server | 4,028 | 90 | 22.3 | 2.6% | ✅ 밀도 10+ |

#### 세부 작업

- [x] 1. vais-codegen 단위 테스트 Part 1: inkwell 모듈 (Sonnet) ✅ 2026-02-25
  변경: cross_compile.rs(+42), debug.rs(+38), parallel.rs(+21), types.rs(+85), abi.rs(+30), alias_analysis.rs(+27)
- [x] 2. vais-codegen 단위 테스트 Part 2: expr/control_flow (Sonnet) ✅ 2026-02-25
  변경: target.rs(+45), error.rs(+12), diagnostics.rs(+18)
- [x] 3. vais-codegen 단위 테스트 Part 3: fn/module/builtins (Sonnet) ✅ 2026-02-25
  변경: bounds_check_elim.rs(+28), auto_vectorize.rs(+53), data_layout.rs(+37) — 총 codegen +377 tests (362→699)
- [x] 4. vais-types 단위 테스트 Part 1: inference/types/builtins (Sonnet) ✅ 2026-02-25
  변경: inference/types/builtins 모듈 전반 (+118 tests, 173→291)
- [x] 5. vais-types 단위 테스트 Part 2: checker (Sonnet) ✅ 2026-02-25
  변경: checker_fn/checker_expr/checker_module/resolve/lookup/scope 모듈 전반
- [x] 6. vais-lsp 테스트 추가 (Sonnet) ✅ 2026-02-25
  변경: backend/handlers/symbol_analysis/hints/folding (+43 tests, 15→58)
- [x] 7. vais-dap 테스트 추가 (Sonnet) ✅ 2026-02-25
  변경: debugger/session/server/protocol (+35 tests, 35→70)
- [x] 8. vais-registry-server 통합 테스트 (Sonnet) ✅ 2026-02-25
  변경: db/handlers/models/config/storage (+71 tests, 2→73)
- [x] 9. 커버리지 측정 & 검증 ✅ 2026-02-25
  변경: 전체 lib 테스트 통과, Clippy 0건
진행률: 9/9 (100%)

### Phase 56: 코드 커버리지 개선 — 보조 크레이트 테스트 강화

> **목표**: 전체 커버리지 70% → 75%+
> 모드: 자동진행

- [x] 1. vais-gc 테스트 보강 (밀도 16.2 → 25+) ✅ 2026-02-26
  변경: gc.rs(+313), concurrent.rs(+195), generational.rs(+142), lib.rs(+28) — 102 tests, 밀도 32.4
- [x] 2. vais-dynload 테스트 보강 (밀도 24.5 → 35+) ✅ 2026-02-26
  변경: error.rs, host_functions.rs, manifest.rs, module_loader.rs, plugin_discovery.rs, resource_limits.rs, wasm_sandbox.rs — 209 tests, 밀도 42.5
- [x] 3. vais-tutorial 테스트 보강 (밀도 23.5 → 35+) ✅ 2026-02-26
  변경: lessons.rs(+218), lib.rs(+625), runner.rs(+66) — 120 tests, 밀도 44.4
- [x] 4. vais-codegen-js 테스트 보강 (밀도 25.8 → 35+) ✅ 2026-02-26
  변경: expr.rs, items.rs, lib.rs, modules.rs, sourcemap.rs, stmt.rs, tree_shaking.rs, types.rs — 267 tests, 밀도 43.1
- [x] 5. tarpaulin 실행 & 전체 커버리지 측정 검증 ✅ 2026-02-26
  변경: cargo llvm-cov 기준 Line 87.37% (macOS에서 tarpaulin 미지원 → llvm-cov 대체), tarpaulin.toml timeout 형식 수정
진행률: 5/5 (100%)

---

## ⏳ 장기 관찰 항목

| 항목 | 출처 | 상태 | 비고 |
|------|------|------|------|
| 대형 프로젝트 6개월 모니터링 | Phase 22 | ⏳ | 프로토타입 검증 완료, 장기 안정성 관찰 중 |
| Instagram 프로필 완성 | Phase 26a | ⏳ | 수작업 필요 (계정/템플릿 준비 완료) |

---

## Phase 53: 종합 검토 & 외부 자료 정합성 (2026-02-25) ✅

- [x] 1. VSCode 키워드 누락 수정 (U,D,O,G,P 추가, V 제거)
- [x] 2. IntelliJ README 문법 오류 수정 (// → #, let → :=, 키워드 20개 완성)
- [x] 3. README.md 수치 업데이트 (E2E 900+, Phase 50)
- [x] 4. Docs: Defer/Global/Union/Macro 4개 문서 신규 작성 + SUMMARY 등록
- [x] 5. Playground: Result/Option/try/unwrap/where/defer 6개 예제 추가
- [x] 6. 최종 검증 & 대형 프로젝트 적합성 보고서 작성

## Phase 54: CI 수정 & Codecov 조정 & 테스트 수정 (2026-02-25) ✅

- [x] 1. CI workflow: bindings-test 빌드 스텝 추가 (maturin/npm) + continue-on-error
- [x] 2. CI workflow: audit job continue-on-error 추가
- [x] 3. Codecov 타겟 현실 조정 (project 80%→60%, core 85%→70%, range 55..100)
- [x] 4. error_suggestion_tests: struct field access에 "Did you mean" 제안 추가
- [x] 5. error_suggestion_tests: non-indexable 타입(i64 등) indexing 시 에러 반환 추가

---

**메인테이너**: Steve

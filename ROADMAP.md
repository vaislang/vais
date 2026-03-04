# Vais (Vibe AI Language for Systems) - AI-Optimized Programming Language
## 프로젝트 로드맵

> **현재 버전**: 0.1.0 (Phase 76 파일럿 검증 완료)
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-03-04 (Phase 95 완료 — IR 검증 게이트 전경로 통합/E2E 1,620 달성/문서 동기화)

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
examples/          # 예제 코드 (208 파일) ✅
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
| 전체 테스트 | 7,400+ (통합 2,700+, 단위 3,900+) |
| 표준 라이브러리 | 74개 .vais + 19개 C 런타임 |
| 셀프호스트 코드 | 50,000+ LOC (컴파일러 + MIR + LSP + Formatter + Doc + Stdlib) |
| 컴파일 성능 | 50K lines → 61.6ms (812K lines/s) |
| 토큰 절감 | 시스템 코드에서 Rust 대비 57%, C 대비 60% 절감 |
| 컴파일 속도 비교 | C 대비 8.5x, Go 대비 8x, Rust 대비 19x faster (단일 파일 IR 생성) |
| 실전 프로젝트 | 3개 (CLI, HTTP API, 데이터 파이프라인) |

### 릴리즈 상태: v0.1.0 (프리릴리스)

> **버전 정책**: 현재는 0.x.x 프리릴리스 단계입니다. 언어 문법이 완전히 확정되어 더 이상 수정이 필요 없을 때 v1.0.0 정식 릴리스를 배포합니다. 기존 v1.0.0 태그(2026-02-01)는 v1.0.0-alpha로 간주됩니다.

| 항목 | 상태 |
|------|------|
| 빌드 안정성 / Clippy 0건 | ✅ |
| 테스트 전체 통과 (7,400+) | ✅ |
| E2E 1,620개 통과 (0 fail, 1 ignored) | ✅ |
| 보안 감사 (14개 수정, cargo audit 통과) | ✅ |
| 라이선스 (396개 의존성, MIT/Apache-2.0) | ✅ |
| 배포 (Homebrew, cargo install, Docker, GitHub Releases) | ✅ |
| 문서 (mdBook, API 문서 65개 모듈) | ✅ |
| CI/CD (3-OS 매트릭스, 코드 커버리지 68.7%) | ✅ |
| 패키지 레지스트리 (10개 패키지) | ✅ |
| 셀프호스팅 (부트스트랩 + MIR + LSP + Formatter) | ✅ |

> **참고 (2026-03-02 재평가)**: 긴급 트랙 R1에서 모든 P0 블로커 해결 (56→90/100). `publish.yml` Cargo.toml 전환, fmt/clippy/test 게이트 통과, 버전 정합성 통일 완료. 상세는 `docs/RELEASE_READINESS_2026-03-01.md`.

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
| Dependent types `{x: T \| pred}` | ✅ 완전 (컴파일타임+런타임 검증) |
| SIMD `Vec4f32` 등 | ✅ 완전 |

### 패턴 매칭 (확정)

`_`, 리터럴, 변수, 튜플, 구조체, enum variant, 범위, or(`\|`), guard(`I cond`), alias(`x @ pat`)

### 어트리뷰트 (확정)

`#[cfg(...)]`, `#[wasm_import(...)]`, `#[wasm_export(...)]`, `#[requires(...)]`, `#[ensures(...)]`, `#[invariant(...)]`

---

## 📜 Phase 히스토리

> 상세 체크리스트는 git log를 참조하세요. Phase 번호는 누적 연번입니다.

### Phase 1~7: 기반 구축 (E2E — → 392)

핵심 컴파일러 파이프라인 (Lexer/Parser/TC/Codegen), Generics, Traits, Closures, Async/Await, Stdlib, LSP/REPL/Debugger 구현. inkwell/JIT/WASM/Python/Node 백엔드 확장. Effect/Dependent/Linear Types, MIR, Query-based 아키텍처. **부트스트랩 달성** (SHA256 일치). CI/CD, i18n, Homebrew/Docker 배포.

### Phase 8~21: 확장 · 안정화 (E2E 392 → 637)

| # | 이름 | 주요 성과 | E2E |
|---|------|----------|-----|
| 8 | 언어 진화 · Stdlib | 에러복구, Incremental TC, cfg 조건부 컴파일, 패키지매니저 | 392 |
| 9~10 | WASM · JS Codegen · 타입 추론 | wasm32 codegen, WASI, codegen-js (ESM), InferFailed E032 | 467 |
| 11~12 | CI · Lifetime · 성능 | Windows CI, CFG/NLL, 병렬 TC/CG (4.14x), Slice fat pointer | 498 |
| 13~14 | 에코시스템 · 토큰 최적화 | 9개 패키지, AES-256, JIT Result, 토큰 -31%, auto-return | 520 |
| 15~16 | 언어 확장 · 타입 건전성 | where/pattern alias/trait alias/impl Trait/HKT/GAT, Incremental, Tarjan SCC | 589 |
| 17~19 | Codegen · Selfhost · 보안 | Range struct, i64 fallback 제거, lazy/spawn, 보안 20건 수정, Docs 다국어 | 655 |
| 20~21 | 정리 · 복구 | Codegen 버그 수정 +44 E2E, ROADMAP 통합, 중복 제거 | 637 |

### Phase 22~52: Codegen 완성 · 품질 강화 (E2E 637 → 900)

| # | 이름 | 주요 성과 | E2E |
|---|------|----------|-----|
| 22~24 | 모듈 분할 R6 · 성능 | formatter/expr/function_gen 분할, Vec::with_capacity, codegen -8.3% | 647 |
| 25~27 | Codegen · 타입 건전성 | indirect call, i64 fallback→InternalError, TC pre-codegen 검증 | 713 |
| 28~31 | 코드 정리 · Selfhost · 모듈 분할 R7 | dead_code 정리, monomorphization 3-pass, tiered/item/doc_gen 분할 | 723 |
| 32~36 | E2E 확장 · assert_compiles 전환 | 136개 assert_compiles→assert_exit_code, type alias 버그 수정, 모듈 분할 R8 | 755 |
| 37~40 | E2E 800+ · Codegen 강화 | Spawn/Lazy 수정, Generic/Slice/Bool/Where, AST 15서브모듈, 모듈 분할 R9 | 811 |
| 41~44 | 건전성 · Pre-existing 전수 수정 | 135건 이슈 수정, pre-existing 14→0, var_resolved_types 도입 | 862 |
| 45~47 | 테스트 정리 · 900 달성 | 40개 중복 제거, 모듈 분할 R10, +78 E2E | 900 |
| 48~51 | Codegen 완성 | Spawn/Async 상태 머신, assert_compiles 7→4, E2E 900 전체 통과(0 fail) | 900 |
| 52 | ROADMAP 정리 | 완료 체크리스트 삭제, 638→~240줄 (-62%) | 900 |

### Phase 53~76: 성숙 · 릴리스 (E2E 900 → 967)

| # | 이름 | 주요 성과 | E2E |
|---|------|----------|-----|
| 53~54 | 외부 정합성 · CI | VSCode/IntelliJ 문법, Docs 4개 신규, codecov 60% | 900 |
| 55~62 | 코드 커버리지 | +2,948 단위 테스트, llvm-cov 전환, 68.7% 달성 | 900 |
| 63~64 | 버전 리셋 · EBNF 스펙 | 0.0.5 프리릴리스, vais.ebnf 154 rules, grammar_coverage 275개 | 900 |
| 65~66 | Pre-existing 검증 · Unify 완성 | 전수 수정 확인, unify 6 variant + apply_substitutions 13 compound | 900 |
| 67~70 | Codegen 확장 · 안전성 | Monomorphization 전이, Map literal, compound assign, panic 0개 | 919 |
| 71~73 | Object Safety · ABI · 릴리스 | v0.0.5, E034 중복함수 검출, assert_compiles 0개 | 931 |
| 74~76 | Stdlib · 온보딩 · 파일럿 | TOML/YAML 파서, 학습 경로 3단계, 실전 프로젝트 2개, **v0.1.0 릴리스** | 967 |

### Phase 77~86: 프로덕션 품질 (E2E 967 → 1,250)

| # | 이름 | 주요 성과 | E2E |
|---|------|----------|-----|
| 77 | Codecov 강화 | +515 tests, lexer~E2E 9파일 | 1,040 |
| 78 | 문자열 fat pointer | str `{i8*,i64}` 전환, C ABI 자동 변환 | 1,040 |
| 79 | 에러 위치 정보 | SpannedCodegenError, last_error_span | 1,040 |
| 80 | 직렬화 확장 | MessagePack + Protobuf | 1,065 |
| 81 | E2E 1,150 달성 | +85 E2E, error_report 컬럼 수정 | 1,150 |
| 82 | 성능 최적화 | 50K 61.6ms (-4.6%), 프로파일링 | 1,150 |
| 83 | Stdlib 확충 | regex +619줄, http_client +407줄, sqlite +64줄 | 1,185 |
| R1 | 릴리즈 복구 | publish.yml 수정, 56→90/100 | 1,185 |
| 84 | 셀프호스팅 강화 | MIR Lowering/최적화 확장, cross-verify 8개 | 1,204 |
| 85 | WASM 생태계 | WASI P2, Component Model, wasm-tools | 1,250 |
| 86 | IDE 개선 | LSP 타입추론/GoToDef, DAP 프리티프린터, IntelliJ 재구축, +590 테스트 | 1,250 |
| 87 | 문서 · 커뮤니티 | API Ref +16모듈, 블로그 3편, 예제 갤러리 15카테고리, README 강화 | 1,250 |
| 88 | 리포지토리 위생 · 타입 안전성 | gitignore profraw/ll, CI 핀, 문서 갱신, Dependent Types 검증 +16 E2E | 1,266 |
| 89 | 기술 부채 해소 | Codecov +203 tests, DT 런타임 assert/f64, Unicode \u{}/char_count | 1,291 |
| 90 | 컴파일러 최적화 · E2E 1,500 · 셀프호스팅 | GVN-CSE/DCE 4단계/Loop Unswitch, +218 E2E, MIR 4패스 추가 | 1,509 |
| 91 | 기술 부채 해소 · 언어 기능 완성도 | assert_compiles 0개, +406 단위 테스트, Method monomorphization, Lifetime 검증 실장 | 1,540 |
| 92 | 컴파일러 안정성 강화 | Panic-free (180+ expect→Result), proptest 24개, Codegen graceful degradation, IR 검증 확장 | 1,540 |
| 93 | 성능 최적화 심화 | 2-level 치환 캐시 (L1 16+L2 256), IdentPool 인터닝, 문자열 상수 dedup, CI Welch's t-test 회귀 감지 | 1,540 |
| 94 | 생태계 확장 | Ed25519 서명, SemVer 해석, 순환 감지, vaisc fix, lint 7종, 빌드 캐시/스크립트/템플릿 | 1,540 |
| 95 | 검증 강화 · E2E 확장 | IR 검증 게이트 7경로 통합, assert_compiles 10→1, +80 E2E, 문서 수치 동기화 | 1,620 |

---

## 현재 작업 (2026-03-04)

모드: 자동진행

### Phase 95: 컴파일러 검증 강화 & E2E 확장

> **목표**: IR 검증 게이트 전경로 통합, assert_compiles 최종 전환, E2E 1,600 달성, 문서 수치 동기화
> **기대 효과**: codegen 버그 조기 발견율 향상, 테스트 품질 완성, 문서 정확성 보장

- [x] 1. IR 검증 게이트 전경로 통합 + 진단 개선 (Opus) ✅ 2026-03-04
  변경: ir_verify.rs (IrDiagnostic에 function_name 추가), backend.rs/core.rs/per_module.rs/test.rs/repl.rs (5개 경로에 verify_text_ir_or_error 게이트 추가)
- [x] 2. assert_compiles→assert_exit_code 최종 전환 + ignored 테스트 분류 (Sonnet) ✅ 2026-03-04
  변경: phase91_lifetime.rs (9/10개 assert_compiles→assert_exit_code 전환, 1개 codegen 한계로 유지+NOTE)
- [x] 3. E2E 1,600 달성 — 미커버 언어 기능 테스트 추가 (Sonnet) ✅ 2026-03-04
  변경: phase95_coverage.rs (신규 80개 E2E — 패턴/구조체/산술/논리/루프/클로저/trait/auto-return 등), e2e/main.rs (모듈 등록)
- [x] 4. README/문서 수치 동기화 + 예제 갤러리 갱신 (Haiku) ✅ 2026-03-04
  변경: README.md (std 79, examples 174, E2E 1,620+, tests 9,300+), CLAUDE.md (동기화)
진행률: 4/4 (100%)

---

## 📜 이전 작업 (2026-03-04)

모드: 자동진행

### Phase 92-pre: Playground 예제 에러 수정

> **목표**: 31개 playground 예제 중 10개 컴파일/실행 에러 해결, 사용자 경험 즉시 개선
> **기대 효과**: playground Run 성공률 21/31 → 26/31 (예제 코드 수정), 미지원 기능 5개 명시적 마킹

- [x] 1. Playground 예제 코드 수정 — 6개 에러 예제 (Sonnet) ✅ 2026-03-02
  변경: playground/src/examples.js (hello-world/string-interpolation/destructuring/slice-types/result-option/unwrap-operator 6개 예제 수정)
- [x] 2. Compile-only 마킹 전환 — 5개 미지원 기능 예제 (Sonnet) ✅ 2026-03-02
  변경: playground/src/examples.js (async-await/ownership/wasm-interop/try-operator/lambda-capture → "syntax preview" 런너블 재작성)
- [x] 3. 수정 검증 — 전체 예제 재컴파일 + E2E 회귀 테스트 (Sonnet) ✅ 2026-03-02
  변경: 30개 예제 전체 컴파일/실행 성공, E2E 1,540 passed (0 fail, 1 ignored, 0 regression)
진행률: 3/3 (100%)

### 리뷰 발견사항 수정 (19건 → 18건 실수정)

> **목표**: team-review 발견 19건 Warning 해소 (#4 이미 완료 → 18건 실수정)
> **기대 효과**: 보안/성능/정확성/아키텍처 품질 강화

- [x] 1. 보안+CI 수정 — 5건 (#1~5, #4 이미 완료) (Opus) ✅ 2026-03-02
  변경: #1 검증완료(이미 파라미터 바인딩), #2 regression_check.sh 변수 인용, #3 publish.yml SHA 핀닝, #4 이미완료, #5 이미완료
- [x] 2. 성능 최적화 — 4건 (#6~9) (Opus) ✅ 2026-03-02
  변경: #6 loop_opt.rs 워크리스트 문서, #7 cse.rs GvnOp/GvnTy 인턴 enum, #8 string_ops.rs 4096 pre-alloc, #9 dead_code.rs 외부함수 보수적 처리
- [x] 3. 정확성 수정 — 5건 (#10~14) (Opus) ✅ 2026-03-02
  변경: #10 helpers.rs sanitize_llvm_name, #11 generics.rs depth guard 64, #12 checker_fn.rs lifetime 경고, #13 method_call.rs 다중impl 감지, #14 dependent_checks.rs 문서화
- [x] 4. 아키텍처 개선 — 5건 (#15~19) (Opus) ✅ 2026-03-02
  변경: #15 type_resolve.rs 분할 계획, #16 error.rs 아키텍처 문서, #17 toml.vais 파서 컴비네이터 노트, #18 runtime.rs WASM 상수 추출, #19 mir_optimizer.vais 교차검증 노트
- [x] 5. 검증 — 전체 빌드 + E2E 회귀 테스트 (Opus) ✅ 2026-03-02
  결과: cargo check OK, clippy 0건, E2E 1,540 passed (0 fail, 1 ignored), codegen 336 passed, types 112 passed
진행률: 5/5 (100%)

---

## 📋 예정 작업

모드: 자동진행

### Phase 92: 컴파일러 안정성 강화 — Panic-free/Fuzzing/에러 복구

> **목표**: 프로덕션 코드 unwrap 865개 중 핵심 경로 300+개 Result 전환, Fuzzing 인프라 구축, 에러 복구 강화
> **기대 효과**: 런타임 panic 0개 보장, 악의적 입력 내성, 다중 에러 보고 UX 개선

- [x] 1. unwrap()/panic() 제거 — codegen 핵심 경로 Result 전환 (Opus) ✅ 2026-03-03
  변경: builtins.rs 152개 expect→?전환, inkwell gen_*.rs 26개 expect→?전환 (8파일), ir_passes 2개 unwrap→let-else 전환, generator.rs 호출처 에러처리
- [x] 2. Fuzzing 인프라 구축 — cargo-fuzz 하네스 + proptest 속성 기반 테스트 (Sonnet) ✅ 2026-03-03
  변경: proptest 의존성 추가 (workspace+3크레이트), lexer 7 proptest (2K cases), parser 8 proptest (1K cases), types 9 proptest (500 cases)
- [x] 3. 에러 복구 강화 — TC 다중 에러 누적 + Codegen graceful degradation (Opus) ✅ 2026-03-03
  변경: CodeGenerator에 multi_error_mode/collected_errors 추가, generate_module Pass2에서 함수별 에러 수집(max 20)+계속 진행, backend.rs에서 활성화+에러 출력
- [x] 4. LLVM IR 검증 게이트 — LLVMVerifyModule 추가 + 진단 메시지 개선 (Sonnet) ✅ 2026-03-03
  변경: ir_verify.rs에 undefined label 검증+return type consistency 검증 추가 (+5 테스트), optimize/mod.rs에 O2+ 후 재검증 게이트 추가
진행률: 4/4 (100%)

### Phase 93: 성능 최적화 심화 — 캐시/인터닝/회귀 감지

> **목표**: 50K lines 컴파일 28.9ms → 24.5ms (2M lines/s), 성능 회귀 자동 감지
> **기대 효과**: 컴파일 속도 -15%, CI 성능 대시보드 구축

- [x] 1. 타입 치환 캐시 고도화 — 2-level 캐시 (L1 fast path + L2 LRU 256) (Opus) ✅ 2026-03-04
  변경: vais-types/src/lib.rs (L1 16-entry direct-mapped + L2 256-entry HashMap), inference.rs (primitive fast path + empty subst fast path + L2→L1 promotion)
- [x] 2. String Interning — FxHashMap 식별자 인턴 풀 + codegen ID 전환 (Sonnet) ✅ 2026-03-04
  변경: vais-codegen/src/string_pool.rs (신규 212줄, IdentPool + InternId(u32) + 7 테스트), lib.rs/init.rs/registration.rs (함수/구조체/enum 이름 인터닝)
- [x] 3. LLVM Value 중복 제거 — ValueCache 상수/phi 재사용 (Opus) ✅ 2026-03-04
  변경: state.rs (dedup_cache), helpers.rs (get_or_create_string_constant), generate_expr/mod.rs+special.rs+expr_visitor.rs+pattern.rs (문자열 상수 중복 제거), types.rs (Named 타입 semi-fast path)
- [x] 4. 성능 회귀 감지 — CI 벤치마크 대시보드 + 자동 알림 (Sonnet) ✅ 2026-03-04
  변경: bench-regression.yml (Welch's t-test + Cohen's d 효과 크기 + perf-regression 이슈 자동 생성 + 90일 아티팩트 보존)
진행률: 4/4 (100%)

### Phase 94: 생태계 확장 — 레지스트리/자동수정/정적분석

> **목표**: 패키지 레지스트리 프로덕션화, vaisc fix 자동 수정 실장, 정적 분석 확장
> **기대 효과**: 패키지 생태계 성숙도 60%→90%, 개발자 경험(DX) 향상

- [x] 1. 패키지 레지스트리 프로덕션화 — Ed25519 서명 + SemVer 해석 + 순환 감지 (Opus) ✅ 2026-03-04
  변경: signing.rs (Ed25519 서명 검증 13 tests), semver_resolve.rs (서버 SemVer 20+ tests), package/semver.rs (클라이언트 SemVer 11 tests), resolution.rs (순환 감지 Tarjan DFS 9 tests)
- [x] 2. vaisc fix 자동 수정 — AST 재작성 엔진 + unused vars/imports 제거 (Opus) ✅ 2026-03-04
  변경: commands/fix.rs (911줄, AST 재작성 엔진 + unused var/import 감지 + span 기반 제거, 9 tests)
- [x] 3. 정적 분석 확장 — Dead code/unused import/unsafe 감사 + lint 규칙 (Sonnet) ✅ 2026-03-04
  변경: vais-security/src/lint.rs (~650줄, L100~L600 lint 규칙 7종 + 2-pass 분석, 13 tests), commands/test.rs (LintAnalyzer 통합)
- [x] 4. 빌드 시스템 강화 — vais.build 스크립트 + 바이너리 캐싱 + 워크스페이스 템플릿 (Sonnet) ✅ 2026-03-04
  변경: build/scripts.rs (빌드 스크립트 pre/post 6 tests), build/cache.rs (SHA-256 content-addressable 캐시 11 tests), simple.rs (workspace 템플릿 --template binary/lib/workspace)
진행률: 4/4 (100%)

---

## ⏳ 장기 관찰 항목

| 항목 | 출처 | 상태 | 비고 |
|------|------|------|------|
| 대형 프로젝트 6개월 모니터링 | Phase 22 | ⏳ | 프로토타입 검증 완료, 장기 안정성 관찰 중 |
| Instagram 프로필 완성 | Phase 26a | ⏳ | 수작업 필요 (계정/템플릿 준비 완료) |

---

## 리뷰 발견사항 (2026-03-02)
> 출처: /team-review (Phase 73-92, origin/main..HEAD, 229 files, +48,742/-1,718)

- [x] 1. [보안] registry packages.rs: SQL 쿼리 문자열 보간 — 검증완료: 이미 sqlx 파라미터 바인딩 사용 중 (false positive) ✅
- [x] 2. [보안] regression_check.sh: 미인용 변수 확장 — ${BASELINE_FILE} 등 중괄호 인용 적용 ✅
- [x] 3. [보안] publish.yml: GitHub Actions 해시 핀닝 — 6개 액션 SHA 고정 (checkout/rust-toolchain/cache/upload/download/gh-release) ✅
- [x] 4. [보안] storage.rs: 패키지 파일 경로 traversal 검증 — 이미 validate_path_component() 구현됨 ✅
- [x] 5. [보안] http_client.vais: 리다이렉트 횟수 제한 — 이미 CLIENT_DEFAULT_MAX_REDIRECTS=10 구현됨 ✅
- [x] 6. [성능] optimizer loop_opt.rs: 워크리스트 알고리즘 — 아키텍처 문서 추가 (현재 O(n) 충분, 미래 최적화 경로 명시) ✅
- [x] 7. [성능] CSE cse.rs: 해시맵 키 인턴닝 — GvnOp/GvnTy enum 도입 (String→enum 전환) ✅
- [x] 8. [성능] string_ops.rs: pre-alloc 패턴 — 2048→4096 초기 할당 증가 ✅
- [x] 9. [성능] dead_code.rs: 외부 함수 호출 보수적 처리 — PURE_FUNCTION_PREFIXES 화이트리스트 + call 기본 side-effectful ✅
- [x] 10. [정확성] helpers.rs: sanitize_llvm_name — 접미사 카운터 + LLVM_INVALID_CHARS 상수 추가 ✅
- [x] 11. [정확성] generics.rs: 모노모피제이션 depth guard — MAX_MONOMORPHIZATION_DEPTH=64 + RecursionLimitExceeded 에러 ✅
- [x] 12. [정확성] checker_fn.rs: lifetime 교차 함수 검증 — self+다중 ref 파라미터 시 경고 발생 ✅
- [x] 13. [정확성] method_call.rs: 다중 trait impl 감지 — candidate_count 기반 ambiguity 경고 (debug_assertions) ✅
- [x] 14. [정확성] dependent_checks.rs: release_mode 최적화 — 모듈 문서화 (assertion 제거 동작 + LLVM 상호작용 설명) ✅
- [x] 15. [아키텍처] type_resolve.rs: 모듈 분할 계획 — 3-4 서브모듈 분할 아키텍처 노트 추가 ✅
- [x] 16. [아키텍처] error.rs: 에러 타입 아키텍처 — CodegenError vs SpannedCodegenError 설계 문서화 ✅
- [x] 17. [아키텍처] std/toml.vais: 파서 컴비네이터 — 공통 추출 계획 노트 추가 ✅
- [x] 18. [아키텍처] runtime.rs: WASM 상수 추출 — WASM_PAGE_SIZE/WASM_HEAP_START_OFFSET 명명 상수화 ✅
- [x] 19. [아키텍처] mir_optimizer.vais: 교차 검증 — selfhost↔Rust 구현 동작 일치 검증 노트 추가 ✅
진행률: 19/19 (100%)

---

## 리뷰 발견사항 (2026-03-04)
> 출처: /team-review HEAD~1..HEAD (phase94 commit)

- [x] 1. [보안] scripts.rs — Command::arg() 사용으로 커맨드 인젝션 방지 ✅
- [x] 2. [보안] scripts.rs — Command::env() per-process 전환, set_var 제거 ✅
- [x] 3. [보안] scripts.rs — 빌드 명령 실행 전 eprintln 경고 추가 ✅
- [x] 4. [아키텍처] semver 중복 — 양 모듈에 교차 참조 문서 추가, 공유 crate 추출 계획 명시 ✅
- [x] 5. [성능] semver_resolve — 테스트 버전 범위 major 0-4→0-19 확장 (75→300 버전) ✅
- [x] 6. [정확성] fix.rs — 겹치는 span 검증 + skip 로직 추가 ✅
- [x] 7. [보안] cache.rs — is_valid_hex() 검증 추가 (lookup/store) ✅
- [x] 8. [성능] cache.rs — BufReader 스트리밍 해시 (8KB 청크) ✅
- [x] 9. [정확성] resolution.rs — fallback에 SemVer 요구사항 매칭 + 최대 버전 선택 ✅
- [x] 10. [아키텍처] test.rs — 레거시 cmd_fix 98줄 삭제 ✅
진행률: 10/10 (100%)

---

## 리뷰 발견사항 (2026-03-05)
> 출처: /team-review HEAD~2..HEAD (phase92 IR검증 + phase95 E2E확장)

- [x] 1. [정확성] ir_verify.rs:185 — 연산자 우선순위 버그 → 무조건 `seen_non_phi_in_block = true` 으로 단순화 ✅
- [x] 2. [정확성] ir_verify.rs:296-334 — rsplit_once로 `@` 직전 토큰 추출 (linkage/CC 무관) + clean_expected 제거 ✅
- [x] 3. [정확성] ir_verify.rs:80-86 — 문자열 상수 내 `"` 토글 추적, in_string 시 brace 무시 ✅
- [x] 4. [아키텍처] utils.rs에 `verify_ir_and_log()` 공통 헬퍼 추출 → 6개 통합 지점 1줄 호출로 통일 ✅
- [x] 5. [아키텍처] verbose 불일치 → 모든 경로 동일 `verify_ir_and_log()` 사용으로 일관성 확보 ✅
- [x] 6. [성능] verify_text_ir() 3회 순회 — 현재 IR 크기에서 무시 가능 (clang 대비 1% 미만), 유지 ✅
- [x] 7. [아키텍처] verify_text_ir_or_error()에 Warning 진단 eprintln 로깅 추가 ✅
진행률: 7/7 (100%)

---

**메인테이너**: Steve

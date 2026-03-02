# Vais (Vibe AI Language for Systems) - AI-Optimized Programming Language
## 프로젝트 로드맵

> **현재 버전**: 0.1.0 (Phase 76 파일럿 검증 완료)
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-03-02 (Phase 91 완료 — 기술 부채 해소/커버리지 75%+/Monomorphization Method/Lifetime 검증)

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
| E2E 1,509개 통과 (0 fail, 1 ignored) | ✅ |
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

---

## 📋 예정 작업

모드: 자동진행

### Phase 91: 기술 부채 해소 + 언어 기능 완성도 강화

> **목표**: 잔여 기술 부채 해소 (assert_compiles 9→0, 커버리지 68.7→75%+) + 언어 기능 완성도 (Monomorphization const generic/method, Lifetime 검증 실장)
> **기대 효과**: 코드 품질 향상, 타입 시스템 건전성 강화, 커버리지 목표 달성

- [x] 1. assert_compiles 완전 제거 — struct-by-value ABI 수정 + global/defer/extern 전환 (Opus) ✅ 2026-03-02
  변경: phase90_structs.rs (6개 struct-by-value→assert_exit_code), phase77_coverage.rs (3개 global/defer/extern→assert_exit_code), helpers.rs (+assert_compiles_only), inkwell generator.rs (struct param alloca+store 패턴)
- [x] 2. 코드 커버리지 75% 달성 — 저커버리지 크레이트 단위 테스트 +406 (Sonnet) ✅ 2026-03-02
  변경: vais-mir/tests (+113: types/emit_llvm/optimize), vais-types/tests (+159: checker_fn/traits/resolve/checker_expr/free_vars), vais-codegen/tests (+44), vais-lsp/tests (+51: semantic/diagnostics), vais-codegen-js/tests (+39), lib.rs pub mod 전환
- [x] 3. Monomorphization 확장 — InstantiationKind::Method 구현 + generic method E2E 12개 추가 (Opus) ✅ 2026-03-02
  변경: inkwell/generator.rs (+96줄 Method 핸들링), module_gen/instantiations.rs (+145줄 Method 핸들링), phase91_monomorphization.rs (+12 E2E)
- [x] 4. Lifetime 검증 실장 — _resolution 결과 적용 + 위반 에러 보고 + E2E 19개 (Opus) ✅ 2026-03-02
  변경: checker_fn.rs (_resolution→resolution, return lifetime 검증, orphan lifetime 검출), lifetime.rs (explicit annotation bypass, dedup same-named lifetimes, validate_return_lifetime 강화), phase91_lifetime.rs (+19 E2E)
진행률: 4/4 (100%)

### Phase 90: 컴파일러 최적화 + E2E 1,500 + 셀프호스팅 강화

> **목표**: 컴파일러 최적화 패스 확장, E2E 1,291→1,500개 달성, 셀프호스팅 MIR 최적화 강화
> **기대 효과**: 컴파일 성능 개선, 테스트 커버리지 +16%, 셀프호스팅 완성도 향상

- [x] 1. 컴파일러 최적화 강화 — LLVM 최적화 패스 확장/인라이닝/DCE/CSE 개선 (Opus) ✅ 2026-03-02
  변경: inlining.rs (복합 점수 휴리스틱, leaf 우선, 루프 내 가중치), dead_code.rs (unreachable block/pure call/store-load 4단계 DCE), cse.rs (GVN 기반 CSE, 교환법칙 감지), loop_opt.rs (Loop Unswitching/Strength Reduction), mod.rs (+11 벤치마크 테스트), +1,601줄/29 tests
- [x] 2. E2E 테스트 1,500개 달성 — 미커버 언어 기능 테스트 추가 +218개 (Sonnet) ✅ 2026-03-02
  변경: e2e/ +10 모듈 (arithmetic/bitwise/closures/control_flow/enums/recursion/strings/structs/variables/functions), 1,291→1,509개 (+218)
- [x] 3. 셀프호스팅 컴파일러 강화 — MIR 최적화 확장/Stage1 보완/코드 생성 개선 (Opus) ✅ 2026-03-02
  변경: optimize.rs (+4 패스: copy propagation/loop unrolling/escape analysis/tail call detection, +827줄), emit_llvm.rs (OptLevel/debug metadata), mir_optimizer.vais (+3 패스: copy propagation/algebraic simplification/loop unrolling, +501줄), lexer_s1.vais (float literal), constants.vais (토큰 ID 동기화), tests.rs (+7 cross-verify)
진행률: 3/3 (100%)

### Phase 89: 기술 부채 해소 — Codecov/Dependent Types/Unicode

> **목표**: 잔여 기술 부채 3건 체계적 해소 — 커버리지 72%+, DT 런타임 검증, Unicode 문자 인식
> **기대 효과**: 커버리지 +3.3%p, 타입 안전성 강화, Unicode 사용성 개선

- [x] 1. Codecov 강화 — LSP/CLI/Registry 단위 테스트 +203 (Sonnet) ✅ 2026-03-02
  변경: vais-lsp/type_resolve.rs (+151 tests), vais-registry-server/handlers/packages.rs (+35), storage.rs (+17)
- [x] 2. Dependent Types 런타임 검증 — assert 삽입 + f64 지원 + 리턴타입 (Opus) ✅ 2026-03-02
  변경: function_gen/dependent_checks.rs (신규, 런타임 assert 삽입), resolve.rs (f64 predicate 평가), types.rs (Dependent/Linear/Affine 핸들링), lib.rs (escape_llvm_string UTF-8 수정), +15 E2E
- [x] 3. Unicode 지원 강화 — \u{XXXX} 이스케이프 + char_count() + case 확장 (Sonnet) ✅ 2026-03-02
  변경: vais-lexer/lib.rs (\u{XXXX} 파싱), std/string.vais (char_count/Latin-1 case), +10 E2E
진행률: 3/3 (100%)

### Phase 87: 문서 · 커뮤니티 — API Reference · 블로그

> **목표**: 프로젝트 외부 가시성 향상 및 커뮤니티 성장 기반 구축
> **기대 효과**: GitHub Stars 100+, 외부 기여자 유입

- [x] 1. API Reference 자동 생성 확장 — std/ 74개 전체 doc_gen (Sonnet) ✅ 2026-03-02
  변경: docs-site/src/api/ (+16 .md: env,error,iter,process,signal,async_http/io/net,msgpack,protobuf,simd,toml,yaml,wasm,wasi_p2,web) + SUMMARY.md 동기화
- [x] 2. 블로그 포스트 3편 작성 — 설계철학/성능비교/셀프호스팅 (Sonnet) ✅ 2026-03-02
  변경: website/blog/ (+3 .html: why-single-char-keywords, performance-comparison, self-hosting-journey) + index.html 업데이트
- [x] 3. 예제 갤러리 구축 — 카테고리별 분류 + docs-site 페이지 (Sonnet) ✅ 2026-03-02
  변경: docs-site/src/examples/gallery.md (174개 예제 15카테고리 분류) + SUMMARY.md 갤러리 섹션
- [x] 4. README 커뮤니티 섹션 강화 — SNS/블로그/배지 추가 (Haiku) ✅ 2026-03-02
  변경: README.md (배지 3개, SNS 링크 3개, 블로그 포스트 3개, Discord placeholder)
진행률: 4/4 (100%)

### Phase 88: 리포지토리 위생 · 타입 안전성 — gitignore/CI 핀/Dependent Types

> **목표**: 201개 profraw 파일 정리, CI 안정성, 오래된 문서 갱신, Dependent Types 기초 검증
> **기대 효과**: git status 깨끗, CI 안정, 타입 안전성 향상 +15 E2E

- [x] 1. .gitignore 보강 + profraw 정리 (Haiku) ✅ 2026-03-02
  변경: .gitignore (+*.profraw, *.profdata, packages/**/*.ll), 640개 profraw 삭제
- [x] 2. CI 워크플로우 안정화 (Haiku) ✅ 2026-03-02
  변경: deploy-playground-server.yml (flyctl @master→@v1)
- [x] 3. 오래된 문서 갱신 (Sonnet) ✅ 2026-03-02
  변경: docs/STABILITY.md (v1.0.0→0.1.0-pre), docs/PACKAGE_GUIDELINES.md (vais pkg→vaisc pkg)
- [x] 4. Dependent Types 기초 검증 (Opus) ✅ 2026-03-02
  변경: checker_fn.rs (validate_dependent_type 호출), +16 E2E (phase86_dependent_types.rs)
진행률: 4/4 (100%)

---

## ⏳ 장기 관찰 항목

| 항목 | 출처 | 상태 | 비고 |
|------|------|------|------|
| 대형 프로젝트 6개월 모니터링 | Phase 22 | ⏳ | 프로토타입 검증 완료, 장기 안정성 관찰 중 |
| Instagram 프로필 완성 | Phase 26a | ⏳ | 수작업 필요 (계정/템플릿 준비 완료) |

---

**메인테이너**: Steve

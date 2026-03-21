# Vais (Vibe AI Language for Systems) - AI-Optimized Programming Language
## 프로젝트 로드맵

> **현재 버전**: 0.1.0 (Phase 144 완료, Pre-existing 39건 해결)
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-03-21 (Phase 144 완료)

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
| 전체 테스트 | 10,400+ (E2E 2,383+, 단위 8,400+) |
| 표준 라이브러리 | 74개 .vais + 19개 C 런타임 |
| 셀프호스트 코드 | 50,000+ LOC (컴파일러 + MIR + LSP + Formatter + Doc + Stdlib) |
| 컴파일 성능 | 50K lines → 58.8ms (850K lines/s) |
| 토큰 절감 | 시스템 코드에서 Rust 대비 57%, C 대비 60% 절감 |
| 컴파일 속도 비교 | C 대비 8.5x, Go 대비 8x, Rust 대비 19x faster (단일 파일 IR 생성) |
| 실전 프로젝트 | 3개 (CLI, HTTP API, 데이터 파이프라인) |

### 코드 건강도 (2026-03-10 감사)

| 지표 | 값 | 상태 |
|------|-----|------|
| TODO/FIXME | 0개 | ✅ |
| Clippy 경고 | 0건 | ✅ |
| 프로덕션 panic/expect | 0개 | ✅ |
| 에러 처리 | Result 패턴 일관, bare unwrap 없음 | ✅ |
| 대형 파일 (>1000줄) | 13개 (R14에서 comptime/concurrent 분할) | ✅ |
| unsafe SAFETY 주석 | 44/44 문서화 (100%) | ✅ |
| 의존성 버전 | 전부 최신 안정 버전 | ✅ |
| 보안 (입력 검증/인젝션/시크릿) | 이슈 없음 | ✅ |
| pre-existing 테스트 실패 | E2E 1건, 단위 5건 (Phase 144에서 39건 해결) | ⚠️ |

### 릴리즈 상태: v0.1.0 (프리릴리스)

> **버전 정책**: 현재는 0.x.x 프리릴리스 단계입니다. 언어 문법이 완전히 확정되어 더 이상 수정이 필요 없을 때 v1.0.0 정식 릴리스를 배포합니다.

| 항목 | 상태 |
|------|------|
| 빌드 안정성 / Clippy 0건 | ✅ |
| 테스트 전체 통과 (9,500+) | ✅ |
| E2E 2,036개 통과 (0 fail, 0 ignored) | ✅ |
| 보안 감사 (cargo audit 통과) | ✅ |
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

### Phase 77~98: 프로덕션 품질 (E2E 967 → 1,620)

| # | 이름 | 주요 성과 | E2E |
|---|------|----------|-----|
| 77~78 | Codecov · str fat pointer | +515 tests, str `{i8*,i64}` 전환, C ABI 자동 변환 | 1,040 |
| 79~81 | 에러 위치 · 직렬화 · E2E 확장 | SpannedCodegenError, MessagePack/Protobuf, E2E 1,150 | 1,150 |
| 82~83 | 성능 · Stdlib | 50K 61.6ms (-4.6%), regex/http_client/sqlite 확충 | 1,185 |
| R1 | 릴리즈 복구 | publish.yml 수정, 56→90/100 | 1,185 |
| 84~86 | Selfhost · WASM · IDE | MIR Lowering, WASI P2/Component Model, LSP/DAP/IntelliJ +590 tests | 1,250 |
| 87~89 | 문서 · 위생 · 기술부채 | API Ref +16모듈, gitignore, Dependent Types 검증, Unicode, Codecov +203 | 1,291 |
| 90~91 | 최적화 · E2E 1,500 | GVN-CSE/DCE/Loop Unswitch, +218 E2E, MIR 4패스, Method mono, Lifetime 실장 | 1,540 |
| 92~94 | 안정성 · 성능 · 생태계 | Panic-free 180+ expect→Result, proptest, 2-level 캐시, Ed25519, vaisc fix, lint 7종 | 1,540 |
| 95~96 | 검증 · 기술부채 | IR 검증 게이트 7경로, +80 E2E, toml 1.0/Cranelift 0.129, LSP 모듈화 | 1,620 |
| 97~98 | CI 복구 | cargo fmt 65파일, MIR +58/JS +39 tests, Security Audit SHA, clang-17 명시 | 1,620 |
| 99~103 | 안정성 · 정리 | expect→Result 61개, 모듈 분할 R11, 테스트 커버리지, Inkwell ABI 수정 | 1,620 |
| 104~108 | 감사 · 분할 R12 · 0 ignored | expect/panic 전수 감사(프로덕션 0건), 9파일 모듈 분할, E2E 0 ignored | 1,620 |
| 109~113 | v1.0 블로커 해결 | Slice bounds check, scope-based auto free, 에러 테스트 +31, ownership 강화 44 tests | 1,667 |
| 114~117 | 완성도 · 검증 | Monomorphization 경고, WASM E2E 44개, 벤치마크 갱신 61.0ms, Codecov 80%+ | 1,723 |
| 118~122 | 성능 · 타입 · 에코 · 문서 · 커버리지 | clone 축소, Text IR 일관성, ConstGeneric mono, tutorial 24 lessons, examples 188, Codecov 85% | 1,745 |
| 99~125 | 안정성 · 완성도 · 타입 정확성 | expect/panic 0건, 모듈 분할 R11-R12, bounds check, auto free, Codecov 85%, strict_type_mode, unit_value() 중앙화 | 1,789 |
| 126~128 | 커버리지 · 타입 강화 · E2E 2K | +309 단위 테스트, strict_type_mode 기본화, +235 E2E | 2,036 |
| 129~130 | 성능 최적화 · 모듈 분할 R13 | Lexer -29.8%, write_ir! 619건, Parser -9.9%, 대형 파일 3개 분할 | 2,036 |
| 131~133 | 커버리지 · ICE 정리 · unsafe 감사 | +150 단위 테스트, eprintln→Error 8건, SAFETY 주석 29건 | 2,052 |
| 134~136 | E2E 2,345 · 성능 R2 · Stdlib 강화 | +262 E2E, Result 표준화, Vec/String/HashMap 메서드 확충 | 2,345 |
| 137~139 | 감사 기반 개선 | SAFETY 주석 44건, 모듈 분할 R14 (comptime/concurrent), async recursive ICE 수정 | 2,345 |
| 140 | 코드 커버리지 강화 | 6개 crate 단위/통합 테스트 추가, 전체 11,357 tests, 0 fail | 2,345 |
| 141 | R1 Generic Monomorphization | C8 타입 전달, type_size 정확도, specialized struct codegen, +27 E2E | 2,372 |
| 142 | R2 IR Type Tracking Phase 1 | temp_var_types 레지스트리, void call naming, integer width mismatch 수정 | 2,383 |
| 143 | R2/R1/R4 근본 문제 해결 | store/load/call/ret 타입 추적, elem_size 전파, Drop auto-call, large struct memcpy | 2,383 |

## 📋 예정 작업


### Phase 137: unsafe SAFETY 주석 완전 문서화 — 44건 미문서화 블록 해소

> **목표**: 44개 unsafe 블록에 SAFETY 주석 추가 (codegen GEP 28건, FFI 10건, GC 4건, JIT 1건, 기타 1건)
> **기대 효과**: 감사 추적성 100%, 코드 리뷰 품질 향상

- [x] 1. unsafe SAFETY 주석 — codegen GEP 28건 문서화 (Sonnet) ✅ 2026-03-10
  변경: simd.rs(21), gen_aggregate.rs(7), gen_advanced.rs(1), binary.rs(1) — 전수 SAFETY 문서화
- [x] 2. unsafe SAFETY 주석 — FFI/GC/JIT 16건 문서화 (Sonnet) ✅ 2026-03-10
  변경: loader.rs(6), module_loader.rs(2), gc.rs(2), concurrent.rs(1), generational.rs(1), compiler.rs(1), dylib_loader.rs(1)
진행률: 2/2 (100%)

### Phase 138: 대형 파일 분할 R14 — comptime.rs & concurrent.rs 모듈화

> **목표**: 1,100줄+ 대형 파일 2개를 서브모듈로 분할 (15→13개)
> **기대 효과**: 모듈 응집력 향상, 테스트 격리 용이

- [x] 3. comptime.rs 모듈 분할 (1,142줄 → mod/evaluator/operators/builtins/tests) (Sonnet) ✅ 2026-03-10
- [x] 4. concurrent.rs 모듈 분할 (1,136줄 → mod/mark/sweep/barrier/worker/tests) (Sonnet) ✅ 2026-03-10
진행률: 2/2 (100%)

### Phase 139: Pre-existing 테스트 실패 해결 — async recursive ICE 수정

> **목표**: async recursive await on non-Future ICE (phase32_async::e2e_phase32_async_recursive) 근본 수정
> **기대 효과**: E2E 0 fail 달성, async codegen 완성도 향상

- [x] 5. async recursive ICE 수정 — __poll 접미사 제거로 @ 자재귀 해결 (Opus) ✅ 2026-03-10
  변경: type_inference.rs, expr_visitor.rs — __poll suffix stripping으로 async 내 @ 호출 정상 해결
- [x] 6. 검증: E2E 2,345 pass / 0 fail / 0 regression, Clippy 0건 (Opus) ✅ 2026-03-10
진행률: 2/2 (100%)

### Phase 140: 코드 커버리지 강화 — 68% → 80%+ 목표

> **목표**: 커버리지 낮은 6개 crate에 단위/통합 테스트 추가, 전체 커버리지 80%+ 달성
> **기대 효과**: Codecov 12%+ 상승, 프로덕션 품질 기준 충족

모드: 중단 (unknown)
- [x] 1. vais-codegen advanced_opt/ 단위 테스트 추가 (Opus 직접) ✅ 2026-03-11
  변경: crates/vais-codegen/tests/advanced_opt_tests.rs (dead_code/inline/const_fold/loop_unroll 등 27 테스트)
- [x] 2. vais-lsp 핸들러 단위 테스트 추가 (Opus 직접) ✅ 2026-03-11
  변경: crates/vais-lsp/tests/handler_tests.rs (completion/symbols/goto_def/references/formatting 등 27 테스트)
- [x] 3. vais-registry-server API 테스트 추가 (Opus 직접) ✅ 2026-03-11
  변경: crates/vais-registry-server/tests/api_coverage_tests.rs (unyank/categories/owners/web/auth 등 27 테스트)
- [x] 4. vais-dynload WASM 샌드박스 테스트 추가 (Opus 직접) ✅ 2026-03-11
  변경: crates/vais-dynload/src/wasm_sandbox.rs (sandbox config/capabilities/wasm instance 등 테스트 추가)
- [x] 5. vais-macro 단위 테스트 추가 (Opus 직접) ✅ 2026-03-11
  변경: crates/vais-macro/tests/coverage_tests.rs (macro expander/hygiene/declarative macro 테스트 추가)
- [x] 6. vais-gpu 백엔드별 테스트 추가 (Opus 직접) ✅ 2026-03-11
  변경: crates/vais-gpu/tests/gpu_tests.rs (CUDA/Metal/OpenCL/WebGPU/SIMD 백엔드별 29 테스트)
- [x] 7. 검증: cargo test 전체 통과 + 커버리지 측정 (Opus 직접) ✅ 2026-03-11
  변경: 전체 11,357 passed / 0 failed / 131 ignored, Clippy 0건
진행률: 7/7 (100%)

### Phase 141: R1 Generic Monomorphization — C8 타입 전달 + type_size 정확도

> **목표**: codegen이 generic T를 항상 i64로 처리하는 근본 문제 수정. Vec<MyStruct> 등에서 실제 타입 크기 추적.
> **기대 효과**: VaisDB test_graph 82%→100%, IR postprocessor 의존성 감소, 구조체 인자 정확한 타입 전달

- [x] 1. C8 Fix: method call argument type lookup — Path B(인스턴스) + C(스태틱) 파라미터 타입 조회 (Opus 직접) ✅ 2026-03-20
  변경: call_gen.rs, method_call.rs — resolved_function_sigs fallback 3경로 추가
- [x] 2. type_size<T> monomorphization — 실제 struct 크기 반환 (Opus 직접) ✅ 2026-03-20
  변경: conversion.rs — compute_sizeof에 Optional/Result/Generic/mangled struct 정확 크기 계산, estimate_type_size에 %struct 레지스트리 조회
- [x] 3. Specialized struct type codegen — Result$T, Option$T 필드 타입 정확도 (Opus 직접) ✅ 2026-03-20
  변경: conversion.rs — type_to_llvm에서 mangled name 우선 조회 (generated_structs 포함), compute_alignof에 Optional/Result/Generic 처리
- [x] 4. E2E tests: generic monomorphization 정확성 검증 (Opus 직접) ✅ 2026-03-20
  변경: phase141_generic_mono.rs — 27개 E2E 테스트 (sizeof struct, type_size generic, specialization, method arg types, nested generics)
- [x] 5. 검증: E2E 2330 passed + 40 pre-existing + 0 regression, Clippy 0 new warnings (Opus 직접) ✅ 2026-03-20
진행률: 5/5 (100%)

### Phase 142: R2 IR Type Tracking Phase 1 — temp_var_types 레지스트리 + void/width 수정

> **목표**: codegen에서 모든 임시 변수의 LLVM 타입을 추적하는 인프라 구축. void call naming + integer width mismatch 수정.
> **기대 효과**: IR postprocessor Fix 4b(void) + Fix 5(width) 제거, ~60-90건 수정 자동화
> **설계**: FunctionContext에 temp_var_types: HashMap<String, ResolvedType> 추가 (Option B — 시그니처 변경 없이)

모드: 자동진행
- [x] 1. temp_var_types 레지스트리를 FunctionContext에 추가 (impl-sonnet) ✅ 2026-03-21
- [x] 2. core generate_expr 경로에서 temp_var_types 채우기 (impl-sonnet) ✅ 2026-03-21
- [x] 3. void call naming 수정: %var = call void 제거 (impl-sonnet) ✅ 2026-03-21
- [x] 4. integer width mismatch 수정: store/binary/icmp (impl-sonnet) ✅ 2026-03-21
- [x] 5. 검증: E2E 2341 passed + 40 pre-existing + 0 regression, Clippy 0 new warnings (Opus 직접) ✅ 2026-03-21
진행률: 5/5 (100%)

### Phase 143: R2/R1/R4/R3/R5 — Codegen 근본 문제 순차 해결

> **목표**: R2 IR 타입 정확성 확장, R1 Monomorphization 강화, R4 Drop codegen, R3 Per-Module, R5 Trait Dispatch
> **기대 효과**: IR postprocessor 완전 제거, generic 컨테이너 정확한 타입 크기, RAII 지원, 모듈별 컴파일, vtable dispatch

모드: 자동진행
- [x] 1. R2: store/load 타입 추적 — llvm_type_of + coerce_int_width 활용 확장 (Opus 직접) ✅ 2026-03-21
  변경: if_else.rs, expr_helpers_control.rs, stmt.rs, expr_helpers.rs — phi coercion, alloca width coerce, index GEP 타입 정확성
- [x] 2. R2: call/ret 타입 정확성 — 함수 시그니처 기반 타입 매칭 (Opus 직접) ✅ 2026-03-21
  변경: call_gen.rs, method_call.rs — ret_resolved 추적 + register_temp_type으로 downstream 전파
- [x] 3. R2: extractvalue/insertvalue 타입 정확성 (Opus 직접) ✅ 2026-03-21
  변경: stmt.rs — tuple extractvalue 후 elem type 등록, expr_helpers_misc.rs — Try/Unwrap 결과 타입 등록
- [x] 4. R1: Vec/HashMap elem_size 실제 타입 크기 전파 (Opus 직접) ✅ 2026-03-21
  변경: vec.vais — es>8 클램프 제거, method_call.rs — generic substitution 설정, conversion.rs — enum/mangled struct sizeof 개선
- [x] 5. R1: monomorphized function 생성 — 기존 인프라 활용 확인 (Opus 직접) ✅ 2026-03-21
  변경: 기존 generate_module_with_instantiations + Task 4 substitution 설정으로 Vec_push$T 정확한 타입 크기 사용
- [x] 6. R4: Drop trait 자동 호출 codegen (Opus 직접) ✅ 2026-03-21
  변경: state.rs — drop_registry, trait_dispatch.rs — Drop impl 등록, function_gen/codegen.rs + stmt.rs + stmt_visitor.rs — 모든 return점에서 auto-drop cleanup
- [x] 7. E2E 검증 + ROADMAP 업데이트 (Opus 직접) ✅ 2026-03-21
  결과: E2E 2341 passed + 40 pre-existing + 0 regression, Clippy 0 new warnings
진행률: 7/7 (100%)

> **참고**: R3(Per-Module)와 R5(VTable Dispatch)는 분석 결과 이미 완전 구현됨 (module_gen/ 600줄, vtable.rs 681줄). 작업 목록에서 제외.

### Phase 144: Pre-existing E2E 실패 39건 해결 — TC 강화 + R2 잔여 + Option 수정 ✅

> **결과**: E2E 2380 passed (+35), 1 failed (pre-existing bytebuffer), 2 ignored
> **단위 테스트**: 343 passed (+10), 5 failed (pre-existing float/complex type)

모드: 자동진행
- [x] 1. TC 타입 불일치 검출 강화 — Bool/Str coercion 제거 + if-branch mismatch + empty-body check
- [x] 2. R2 잔여 IR 타입 오류 수정 — 중복 integer width coercion 제거 (7/8 해결, 1건 pre-existing)
- [x] 3. Option codegen 수정 — Some variant tag 0→1 (동적 lookup으로 전환)
- [x] 4. E2E 검증 + ROADMAP 업데이트
진행률: 4/4 (100%)

**변경 파일**:
- `vais-types/src/inference/unification.rs` — Bool 제거 from is_integer_type(), Str↔I64 coercion 제거
- `vais-types/src/checker_expr/control_flow.rs` — if-branch type mismatch 에러 (both non-Unit)
- `vais-types/src/checker_fn.rs` — explicit return type with empty body → mismatch 에러
- `vais-codegen/src/generate_expr_call.rs` — 중복 trunc 제거, Some tag 동적 lookup
- `vais-codegen/src/expr_helpers_call/call_gen.rs` — Some/Ok/Err tag 동적 lookup

---

## 🔴 Codegen 근본 문제 (VaisDB 실전 컴파일에서 발견, 2026-03-20)

> **배경**: VaisDB (RAG-native hybrid DB, ~200파일 순수 Vais) 컴파일 과정에서 발견된 컴파일러 한계.
> C1-C8 근본 수정 완료 (커밋 bcf1be5), TC 에러 674→5 (-99%), test_graph 37/45 통과 (82%).
> 아래는 **아직 미해결인 구조적 문제**로, 범용 컴파일러로 사용하려면 반드시 해결 필요.

### R1. Generic Monomorphization 미완성 ★★★

**증상**: `Vec<MyStruct>` (8바이트 초과 struct)에서 데이터 손실, SIGSEGV
**원인**: codegen이 generic T를 항상 `i64` (8바이트)로 처리
- `Vec.push(struct)` → elem_size=8로 첫 8바이트만 저장
- `Result<LargeStruct>` → payload를 `{ i64 }` 한 필드로 축소
- `HashMap<K, V>` → value를 i64로 저장

**영향**: test_graph 8건 crash, test_btree Vec<BTreeLeafEntry> stride 불일치, test_fulltext Vec<TokenInfo> 데이터 손실
**해결 방향**:
1. `type_size()` 제네릭 전파 — codegen에서 T의 실제 크기 참조
2. monomorphized function 생성 — `Vec_push$MyStruct` 등 타입별 함수
3. Result/Option 타입 정의에 실제 payload 크기 반영

### R2. IR Postprocessor 의존성 ★★★

**증상**: 컴파일러 생성 IR에 490+건 타입 오류 → Python 스크립트로 후처리 필수
**원인**: codegen 타입 시스템이 LLVM IR 타입 규칙과 불일치
- `store i64 %str_var` (str은 `{ i8*, i64 }` 16바이트)
- `extractvalue { i8*, i64 } %i64_var` (i64에서 struct 추출 시도)
- `%t = call void @func()` (void 반환에 이름 할당)
- `ret %Result$i64 %var` (함수 반환 타입과 불일치)
- phi 노드에 pointer/value 혼합

**해결 방향**: codegen에서 모든 변수의 LLVM 타입을 정확히 추적하는 타입 맵 구현

### R3. Per-Module Codegen 미작동 ★★

**증상**: `VAIS_SINGLE_MODULE=1` 없이 컴파일하면 cross-module 참조 실패
**원인**: 모듈별 분리 컴파일 시 std의 impl 블록이 사용측 모듈에 전파 안 됨
- `Vec.push()` → std/vec.vais의 impl이 test 모듈에서 미해석
- GenericInstantiation이 모듈 경계를 넘지 못함

**영향**: 대형 프로젝트에서 컴파일 시간 선형 증가, 증분 컴파일 불가
**해결 방향**: 모듈 인터페이스 파일 (.vai) 생성 또는 monolithic 빌드 최적화

### R4. RAII / Drop Codegen 미구현 ★★

**증상**: `Mutex.lock()` → guard 반환 → guard.drop() 미호출 → deadlock
**원인**: 스코프 종료 시 drop 함수 자동 호출 codegen 없음
**영향**: test_wal 3건 timeout (Mutex deadlock), 메모리 누수 가능성
**해결 방향**: scope exit 시 alloca 변수의 Drop trait 구현 자동 호출

### R5. Trait 기반 다형성 Codegen 제한 ★★

**증상**: `W Display { F fmt(...) }` trait 정의는 파싱되지만 vtable/dispatch codegen 없음
**원인**: trait method dispatch가 hardcoded name mangling에 의존
**해결 방향**: vtable 생성 또는 monomorphization 기반 static dispatch

### R6. TC NONFATAL 모드 의존 ★

**증상**: `VAIS_TC_NONFATAL=1` 없이 5개 타입 에러로 IR 생성 중단
**원인**: cross-module impl 메서드 스코프 해석 불완전 (R3과 연관)
**영향**: 타입 안전성 미보장 상태로 IR 생성
**해결 방향**: R3 해결 시 자연 해소 예상

### 우선순위 요약

| 순위 | 이슈 | 영향 | 난이도 |
|------|------|------|--------|
| 1 | R1: Generic Monomorphization | 82%→100% 통과율 핵심 | 상 |
| 2 | R2: IR Postprocessor 제거 | 정상적인 컴파일러 조건 | 상 |
| 3 | R3: Per-Module Codegen | 확장성, 증분 컴파일 | 중 |
| 4 | R4: RAII/Drop | 리소스 관리 안전성 | 중 |
| 5 | R5: Trait Dispatch | 다형성 지원 | 중 |
| 6 | R6: TC NONFATAL 제거 | 타입 안전성 | 하 (R3 후 자동 해소) |

### 참고 자료
- VaisDB ROADMAP: `/Users/sswoo/study/projects/vaisdb/ROADMAP.md` — 테스트별 상세 실패 분석
- COMPILER_AUDIT: `/Users/sswoo/study/projects/vais/COMPILER_AUDIT.md` — C1-C8 수정 분류
- IR Postprocessor: `/tmp/ir_postprocess.py` — 490+ fix 카테고리 목록

---

## ⏳ 장기 관찰 항목

| 항목 | 출처 | 상태 | 비고 |
|------|------|------|------|
| 대형 프로젝트 6개월 모니터링 | Phase 22 | ⏳ | 프로토타입 검증 완료, 장기 안정성 관찰 중 |
| Instagram 프로필 완성 | Phase 26a | ⏳ | 수작업 필요 (계정/템플릿 준비 완료) |

---

**메인테이너**: Steve

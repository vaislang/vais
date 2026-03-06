# Vais (Vibe AI Language for Systems) - AI-Optimized Programming Language
## 프로젝트 로드맵

> **현재 버전**: 0.1.0 (Phase 100 완료)
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-03-06 (Phase 100 — LSP 대형 파일 모듈 분할 R11)

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
| 전체 테스트 | 9,500+ (E2E 1,620+, 단위 7,900+) |
| 표준 라이브러리 | 74개 .vais + 19개 C 런타임 |
| 셀프호스트 코드 | 50,000+ LOC (컴파일러 + MIR + LSP + Formatter + Doc + Stdlib) |
| 컴파일 성능 | 50K lines → 61.6ms (812K lines/s) |
| 토큰 절감 | 시스템 코드에서 Rust 대비 57%, C 대비 60% 절감 |
| 컴파일 속도 비교 | C 대비 8.5x, Go 대비 8x, Rust 대비 19x faster (단일 파일 IR 생성) |
| 실전 프로젝트 | 3개 (CLI, HTTP API, 데이터 파이프라인) |

### 코드 건강도 (2026-03-06 분석)

| 지표 | 값 | 상태 |
|------|-----|------|
| TODO/FIXME | 0개 | ✅ |
| Clippy 경고 | 0건 | ✅ |
| assert_compiles 잔여 | 1개 (lifetime codegen 한계) | ✅ |
| expect() 핫스팟 | parser 38, gc 25, mir 18 | ⚠️ |
| panic! 호출 | 96개 | ⚠️ |
| 대형 파일 (>1000줄) | 32개 | ⚠️ |
| 최소 테스트 크레이트 (1파일) | 17개 | ⚠️ |

### 릴리즈 상태: v0.1.0 (프리릴리스)

> **버전 정책**: 현재는 0.x.x 프리릴리스 단계입니다. 언어 문법이 완전히 확정되어 더 이상 수정이 필요 없을 때 v1.0.0 정식 릴리스를 배포합니다.

| 항목 | 상태 |
|------|------|
| 빌드 안정성 / Clippy 0건 | ✅ |
| 테스트 전체 통과 (9,500+) | ✅ |
| E2E 1,620개 통과 (0 fail, 1 ignored) | ✅ |
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

---

## 📋 예정 작업

모드: 자동진행

### Phase 99: 컴파일러 안정성 — expect→Result 전환 & panic 축소

> **목표**: 프로덕션 핵심 경로 expect() 81개→Result 전환, panic! 96개 중 50+개 축소
> **기대 효과**: 컴파일러 런타임 panic 경로 60% 축소, 에러 메시지 품질 향상

- [x] 1. Parser expect→Result — expr/primary.rs 38개, item/declarations.rs 20개 (Opus) ✅ 2026-03-06
  변경: Parser의 self.expect()는 이미 ParseResult 반환 메서드 — 변환 불필요 확인
- [x] 2. GC/MIR expect→Result — gc/ffi.rs 25개, mir/lib.rs 18개, mir/lower.rs 15개 (Sonnet) ✅ 2026-03-06
  변경: gc/ffi.rs 25개 lock_or_abort() 헬퍼 전환, MIR expect()는 #[cfg(test)] 내부 — 유지
- [x] 3. panic! 축소 — 96개 중 프로덕션 경로 50+개를 Result/에러 반환으로 전환 (Opus) ✅ 2026-03-06
  변경: 14파일 61개 expect/panic 전환 (dap, incremental, types, codegen-js, repl, package, registry)
- [x] 4. 검증 — cargo test 전체 통과 + clippy 0건 확인 (Sonnet) ✅ 2026-03-06
  변경: E2E 1618 passed, 0 failed, 3 ignored, clippy 0건

### Phase 100: LSP 대형 파일 모듈 분할 R11

> **목표**: 1,500줄+ 대형 파일 6개 모듈 분할
> **기대 효과**: 유지보수성 향상, 코드 탐색 용이

- [x] 1. vais-lsp type_resolve/mod.rs (2,461줄) → 4개 서브모듈 분할 (Sonnet) ✅ 2026-03-06
  변경: mod.rs(125줄) + context.rs(241줄) + inference.rs(224줄) + tests.rs(1,886줄)
- [x] 2. vais-lsp backend/mod.rs (1,935줄) → 3개 서브모듈 분할 (Sonnet) ✅ 2026-03-06
  변경: mod.rs(127줄) + language_server.rs(1,019줄) + tests.rs(792줄)
- [x] 3. vais-codegen types.rs (1,697줄) → 3개 서브모듈 분할 (Sonnet) ✅ 2026-03-06
  변경: types/mod.rs(222줄) + conversion.rs(763줄) + tests.rs(721줄)
- [x] 4. vais-codegen-js expr.rs (1,676줄) → 3개 모듈 분할 (Sonnet) ✅ 2026-03-06
  변경: expr.rs(714줄) + expr_helpers.rs(100줄) + expr_tests.rs(870줄)
- [x] 5. 검증 — E2E 전체 통과 + clippy 0건 확인 (Sonnet) ✅ 2026-03-06
  변경: E2E 1618 passed, 0 failed, 3 ignored, clippy 0건

### Phase 101: 테스트 커버리지 확장 — 최소 테스트 크레이트 강화

> **목표**: 17개 최소 테스트 크레이트 중 핵심 8개에 통합 테스트 추가
> **기대 효과**: 코드 커버리지 향상, 회귀 방지

- [ ] 1. 런타임 핵심 — vais-plugin, vais-macro, vais-hotreload 통합 테스트 (Sonnet)
- [ ] 2. 성능 핵심 — vais-gpu, vais-jit 통합 테스트 (Sonnet)
- [ ] 3. 인프라 — vais-dynload, vais-bindgen, vais-query 통합 테스트 (Sonnet)
- [ ] 4. 보안/품질 — vais-security, vais-supply-chain 통합 테스트 (Sonnet)
- [ ] 5. 검증 — 전체 테스트 통과 + 커버리지 수치 갱신 (Sonnet)

### Phase 102: Untracked 정리 & 코드 위생

> **목표**: 11개 untracked 파일 정돈, proptest regression 파일 관리
> **기대 효과**: 깔끔한 git status, 재현 가능한 빌드

- [ ] 1. proptest regression 파일 .gitignore 추가 + 예제 파일 commit/정리 (Sonnet)
- [ ] 2. pilot 프로젝트 디렉토리 정돈 (examples/pilot_*) (Sonnet)
- [ ] 3. selfhost 예제 파일 정리 (examples/selfhost_*.vais) (Sonnet)

### Phase 103: Codegen 완성도 — 잔여 이슈 해결

> **목표**: ignored E2E 2개 해결, assert_compiles 1개 해결 시도
> **기대 효과**: E2E 0 ignored 달성

- [ ] 1. clang-17 crash E2E — num_convert codegen phi 버그 수정 (Opus)
- [ ] 2. exit code 불일치 E2E — codegen 로직 추적 및 수정 (Opus)
- [ ] 3. lifetime codegen assert_compiles — IR 생성 정확성 개선 (Opus)

---

## ⏳ 장기 관찰 항목

| 항목 | 출처 | 상태 | 비고 |
|------|------|------|------|
| 대형 프로젝트 6개월 모니터링 | Phase 22 | ⏳ | 프로토타입 검증 완료, 장기 안정성 관찰 중 |
| Instagram 프로필 완성 | Phase 26a | ⏳ | 수작업 필요 (계정/템플릿 준비 완료) |

---

**메인테이너**: Steve

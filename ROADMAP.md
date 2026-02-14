# Vais (Vibe AI Language for Systems) - AI-Optimized Programming Language
## 프로젝트 로드맵

> **버전**: 2.0.0
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-02-14

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
| 전체 테스트 | 2,500+ (E2E 571, 통합 354+) |
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

---

## 📋 다음 로드맵 (Phase 39~)

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

### Phase 39: 성능 최적화 — Incremental 실전 & 병렬 Codegen 강화
- Incremental compilation 실전 검증: 대규모 프로젝트에서 캐시 히트율 측정 및 개선
- 병렬 codegen 추가 최적화: 모듈 간 의존성 분석 정밀화
- 컴파일 속도 벤치마크 자동 회귀 감지 강화

### Phase 40: 에코시스템 성장 — 패키지 확대 & 커뮤니티
- 추가 공식 패키지 10개+ (HTTP client, CLI framework, testing framework, logging, config, etc.)
- 커뮤니티 기여 가이드 (CONTRIBUTING.md, good first issues)
- 튜토리얼 확대: 실전 프로젝트 가이드 (CLI 도구, REST API, WASM 앱)

### Phase 41: Codegen 고도화 — Monomorphization & ImplTrait 완성
- Monomorphization 완성: ImplTrait 리턴 시 실제 타입별 코드 생성 (i64 fallback 제거)
- Trait object vtable 최적화
- Dead code elimination 강화
- Codegen test pre-existing 14 에러 해결

### Phase 42: Selfhost 강화 — 컴파일러 자체 확장
- 더 많은 컴파일러 모듈 셀프호스팅 (parser, type checker 일부)
- Selfhost 컴파일러로 std/ 컴파일 검증
- Bootstrap chain 자동화 (vaisc → selfhost-vaisc → 재검증)

### Phase 43: WASM/JS 타겟 강화 — Component Model & JS Interop
- WASM Component Model 실전: 복합 타입 직렬화, 리소스 핸들
- JS interop 개선: TypeScript 타입 자동 생성, Promise 브리징
- wasm-opt 통합 (바이너리 크기 최적화)

### Phase 44: 언어 진화 인프라 — Edition & Migration 시스템
- Edition 시스템 도입: `edition = "2026"` in vais.toml (breaking change 격리)
- `vaisc migrate` 도구: 소스 코드 자동 변환 (AST 기반 리팩토링)
- SemVer 기반 컴파일러 호환성 보장: 이전 edition 코드 무수정 컴파일
- `vais update` 안전 업데이트: lockfile + registry 연동 완성

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

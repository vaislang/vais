# Vais (Vibe AI Language for Systems) - AI-Optimized Programming Language
## 프로젝트 로드맵

> **버전**: 1.0.0
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-02-07

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
examples/          # 예제 코드 (110+ 파일) ✅
selfhost/          # Self-hosting 컴파일러 ✅
benches/           # 벤치마크 스위트 (criterion) ✅
playground/        # 웹 플레이그라운드 프론트엔드 ✅
docs-site/         # mdBook 문서 사이트 ✅
vscode-vais/       # VSCode Extension ✅
intellij-vais/     # IntelliJ Plugin ✅
community/         # 브랜드/홍보/커뮤니티 자료 ✅
```

---

## 📊 전체 진행률 요약

| Phase | 이름 | 상태 | 비고 |
|-------|------|------|------|
| 1~12 | 핵심 컴파일러 ~ 프로덕션 안정화 | ✅ 완료 | 100% |
| 13~21 | 품질 보증 ~ 실사용 완성도 | ✅ 완료 | 100% |
| 22 | 대형 프로젝트 도입 전략 | ⏳ 장기 관찰 | 11/12 (92%) — 6개월 모니터링 잔여 |
| 23~25 | 크로스플랫폼 · Playground · Vararg | ✅ 완료 | 100% |
| 26a | 홍보 & 커뮤니티 성장 | ⏳ 수작업 대기 | 3/4 (75%) — Instagram 프로필 완성 잔여 |
| 26b~28 | 기술 부채 · GPU · Async | ✅ 완료 | 100% |
| 29 | 토큰 절감 강화 | ✅ 완료 | 21/21 (100%) — Rust 대비 30%+ 절감 |
| 30 | 성능 최적화 | ✅ 완료 | 29/29 (100%) — inkwell 기본 + TCO + 인라이닝 |
| 31 | 표준 라이브러리 시스템 프로그래밍 보강 | ✅ 완료 | 30/30 (100%) — fsync/mmap/flock/ByteBuffer 등 |
| 32 | 표준 라이브러리 웹/DB/템플릿 | ✅ 완료 | 7/7 (100%) — HTTP/SQLite/PG/WebSocket/ORM |
| 33 | 프로덕션 블로커 해소 | ✅ 완료 | 7/7 (100%) — TLS/Async 크로스플랫폼/로깅/압축 |
| 34 | 실전 검증 & 에코시스템 | ✅ 완료 | 8/8 (100%) — CLI·HTTP·데이터 실전 프로젝트 3개 |
| 35 | 프로덕션 완성 & 커뮤니티 런칭 | ✅ 완료 | 7/7 (100%) — CI/CD, API 문서, 커뮤니티 인프라 |
| 36 | Production Readiness | ✅ 완료 | 8/8 (100%) — 50K lines 79ms, 641K lines/s |
| 37 | 프로덕션 갭 해소 | ✅ 완료 | 7/7 (100%) — CI green, 원커맨드 빌드, 레지스트리 배포 |
| 38 | 셀프호스팅 100% 달성 | ✅ 완료 | 부트스트랩 달성! (SHA256 일치, 17,807줄) |
| 39 | 셀프호스트 MIR Borrow Checker | ✅ 완료 | mir_borrow.vais 1,357줄 + 8,000+ LOC MIR 전체 |
| 40 | 셀프호스트 Stdlib 테스트 강화 | ✅ 완료 | 276 assertions, 6 모듈 + Rust E2E (버그 3건 수정) |
| **41** | **v2.0 언어 진화** | **📋 예정** | **아래 상세** |
| | *VaisDB 본체 → 별도 repo (`vaisdb`)에서 진행* | | |

---

## 📊 완료된 Phase 요약 (Phase 1~12)

> 아래는 완료된 Phase의 간략 요약입니다. 상세 이력은 git log를 참조하세요.

| Phase | 이름 | 주요 성과 | 완료일 |
|-------|------|----------|--------|
| **Phase 1** | 핵심 컴파일러 | Lexer, Parser, Type Checker, Code Generator, Generics, Traits, Closures, Async/Await, Pattern Matching, Module System | 2026-01-20 |
| **Phase 2** | 표준 라이브러리 | Option, Result, Vec, String, HashMap, File, Iterator, Future, Rc, Box, Arena, Math, IO, Set, Deque, Net (TCP/UDP, IPv6) | 2026-01-21 |
| **Phase 3** | 개발자 도구 | LSP Server, REPL, Optimization Passes (6종), VSCode Extension, Doc Generator, Formatter, Debugger (DWARF) | 2026-01-21 |
| **Phase 4** | 향후 개선 | 표현식 디버그 메타데이터, IPv6 소켓, PriorityQueue, BTreeMap, Regex, JSON, 인라이닝/루프 최적화 | 2026-01-20 |
| **Phase 5** | 품질 개선 | 테스트 46→245개, 엣지케이스 100+, 통합 테스트 47개, vais-codegen/vais-types 모듈 분리, CI/CD, i18n, 플러그인 시스템 | 2026-01-20 |
| **Phase 6** | 후속 개선 | 테스트 302→402개, import 보안 강화, 코드 중복 제거, LSP 캐싱, Architecture.md, LSP/플러그인/Formatter 통합 테스트 | 2026-01-21 |
| **Phase 7** | 아키텍처 개선 | Parser 모듈 분해, Codegen Visitor 패턴, Wasm 타겟, 증분 컴파일, IntelliJ 플러그인, inkwell 통합, Python/Node.js 바인딩, JIT 컴파일, Self-hosting (7개 모듈) | 2026-01-22 |
| **Phase 8** | 생산성 향상 | `?` 연산자, `defer` 문, 패키지 매니저 (vais.toml), 패키지 레지스트리, Const generics, SIMD intrinsics, Union types, Comptime evaluation, Playground, GC, Hot reloading, GPU 타겟 | 2026-01-22 |
| **Phase 9** | 언어 완성도 | Bidirectional Type Checking, Dynamic Dispatch, Macro System, Thread/Sync/Http 모듈, LTO, PGO, 증분 빌드 고도화, Profiler, Test Framework | 2026-01-22 |
| **Phase 10** | Self-hosting | Stage 1+2 부트스트래핑 완료 (17,397줄 동일 IR 검증), 에러 복구, Macro Runtime, LSP 고도화, 패키지 레지스트리 서버, FFI 고도화, 크로스 컴파일 16개 타겟, DAP 서버, inkwell 완전 전환 | 2026-01-26 |
| **Phase 11** | 프로덕션 준비 | Effect System, Dependent/Linear Types, Lifetimes, Associated Types, Tiered JIT, Concurrent GC, Lazy evaluation, FFI bindgen, GPU 백엔드 (CUDA/Metal/AVX-512/NEON), 동적 모듈 로딩, WASM 샌드박싱 | 2026-01-27 |
| **Phase 12** | 프로덕션 안정화 | dead_code/clippy 정리, inkwell for loop, auto_vectorize, 에러 복구 강화, Async Traits/Structured Concurrency/Async Drop, GAT/Const Traits/Variance, MIR 도입, Query-based 아키텍처, mdBook 문서 사이트 | 2026-01-29 |

---

## 📊 완료된 Phase 요약 (Phase 13~40)

> Phase 13~40의 상세 체크리스트는 git log를 참조하세요. 아래는 각 Phase의 핵심 성과 요약입니다.

| Phase | 이름 | 주요 성과 | 완료일 |
|-------|------|----------|--------|
| **Phase 13** | 품질 보증 및 프로덕션 검증 | E2E 89→128개, Windows CI, Python 바인딩, Const Generics/Named Arguments/Procedural Macros, PGO/병렬 컴파일 | 2026-01-29 |
| **Phase 14** | 프로덕션 배포 및 커뮤니티 구축 | 제네릭 monomorphization + vtable, Homebrew/Docker/Windows 배포, 웹사이트, SNS, YouTube 튜토리얼 5편 | 2026-01-31 |
| **Phase 15** | v1.0 출시 준비 | Async 런타임(kqueue), 세대별 GC, 라이프타임/소유권 검사, ABI 안정화, GAT/Specialization, 보안 감사 14개 수정 | 2026-01-31 |
| **Phase 16** | 실사용 검증 버그 수정 | 45개 예제 실패 → 0개 (105/105 100%), Option/Vec self 타입, 클로저 IR, GC 링킹 | 2026-01-31 |
| **Phase 17** | 런타임 버그 수정 | printf 포맷 검증, if-else 타입 추론, GC 런타임 검증, clippy 경고 수정, CLAUDE.md 생성 | 2026-01-31 |
| **Phase 18** | 코드젠 심층 버그 수정 | mutable 구조체 segfault, LLVM float 상수 포매팅, float 연산, sin/cos/exp/log extern | 2026-01-31 |
| **Phase 19** | 대형 프로젝트 도입 준비 | unwrap→Result 전환, Borrow Checker 3-모드, HTTP/JSON/Regex 런타임, 증분 컴파일 | 2026-02-01 |
| **Phase 20** | 근본적 문제 해결 | pthread Thread 런타임, f64 포인터 역참조 codegen, 파서 재귀 깊이 안전장치 | 2026-02-01 |
| **Phase 21** | 실사용 완성도 강화 | Sync 런타임 (Mutex/RwLock/Condvar/Barrier/Semaphore/Atomics), E2E 152→165개 | 2026-02-01 |
| **Phase 22** | 대형 프로젝트 도입 전략 | 프로토타입 검증(239줄), 중형 프로젝트 패턴 검증(5종), C 대비 벤치마크. **⏳ 6개월 모니터링 대기** | 2026-02-01 |
| **Phase 23** | 크로스플랫폼 호환성 | Extern 함수 포인터, Enum 패턴 매칭 LLVM 타입, ExprVisitor float 분기 | 2026-02-01 |
| **Phase 24** | Playground Linux 배포 | SSA 네이밍, Enum GEP 인덱싱, Match phi, Fly.io 배포, vaislang org 이전 | 2026-02-01 |
| **Phase 25** | Vararg float 버그 수정 | vararg float→i64 제거, Ternary/If/Match/Cast 타입 추론, float printf E2E | 2026-02-01 |
| **Phase 26a** | 홍보 & 커뮤니티 | Instagram 계정, 코드 카드 템플릿. **⏳ Instagram 프로필 완성 대기 (수작업)** | 2026-02-01 |
| **Phase 26b** | 기술 부채 해결 | f64 배열 codegen, GPU 커널 타입 추론, std 런타임 검증 | 2026-02-01 |
| **Phase 27** | GPU & Async 완성 | Metal CLI, 호스트 코드 생성, GPU E2E 9개, Async 빌트인(poll/kqueue/pipe/time) | 2026-02-01 |
| **Phase 28** | GPU 런타임 실행 지원 | CUDA/Metal/OpenCL 런타임 통합, Unified Memory, 스트림/비동기, 다중 GPU, 프로파일링 (27/27) | 2026-02-02 |
| **Phase 29** | 토큰 절감 강화 | 문자열 보간, 파라미터 타입 추론, `~`/`Y`/`\|>`/`..` 연산자, 암시적 self 생략 (21/21) | 2026-02-02 |
| **Phase 30** | 성능 최적화 | inkwell 기본 전환 (E2E 210개 통과, 36% 빠름), TCO, 인라이닝/PGO, MIR 최적화, 경계 검사 제거, 벤치마크 스위트 (29/29) | 2026-02-03 |
| **Phase 31** | 시스템 프로그래밍 보강 | fsync/mmap/flock, 할당자, StringMap/OwnedString, 디렉토리 조작, ByteBuffer/CRC32, `?`/`!` 연산자 완성 (30/30) | 2026-02-03 |
| **Phase 32** | 웹/DB/템플릿 확장 | HTTP 서버/클라이언트, SQLite/PostgreSQL, 템플릿 엔진, WebSocket, ORM (7/7) | 2026-02-04 |
| **Phase 33** | 프로덕션 블로커 해소 | TLS/HTTPS, Async 크로스플랫폼(epoll/IOCP/kqueue), 패키지 레지스트리 배포, DAP 검증, 구조화 로깅, gzip 압축 (7/7) | 2026-02-04 |
| **Phase 34** | 실전 검증 & 에코시스템 | Borrow Checker strict, Windows E2E, API 문서 사이트, 온보딩 가이드, 10개 공개 패키지, CLI/HTTP/데이터 실전 프로젝트 3개 (8/8) | 2026-02-04 |
| **Phase 35** | 프로덕션 완성 | selfhost lexer 검증 (114 테스트), 패키지 publish/install 라운드트립 (60 E2E), CI/CD 파이프라인, 에러 메시지 개선, API 문서 전체, 커뮤니티 인프라, v1.0 릴리스 (7/7) | 2026-02-05 |
| **Phase 36** | Production Readiness | 모듈 시스템 수정, i18n 에러 메시지, 크로스 모듈 타입, selfhost 75%, 소유권 strict, CI/CD, **50K lines 79ms (641K lines/s)** (8/8) | 2026-02-07 |
| **Phase 37** | 프로덕션 갭 해소 | CI green 복구, 통합 빌드 시스템 (`vaisc build`), 의존성 보안, 레지스트리 Fly.io 배포, selfhost lexer 100% (45,640 토큰), 프로덕션 가이드 (7/7) | 2026-02-07 |
| **Phase 38** | 셀프호스팅 100% | Parser/AST/TC/Codegen/Module 100%, **부트스트랩 달성** (Stage1→Stage2→Stage3 fixed point, SHA256 일치, 17,807줄) | 2026-02-06 |
| **Phase 39** | 셀프호스트 MIR | MIR 전체 파이프라인 (8,000+ LOC): 구조체/빌더/lowering/LLVM emission/optimizer/analysis/borrow checker + 파이프라인 통합 | 2026-02-07 |
| **Phase 40** | Stdlib 테스트 강화 | 276 assertions (Vec 103, String 58, HashMap 50, Option 32, File I/O 12, Print 21), Rust E2E 6개, 버그 3건 수정 | 2026-02-07 |

---

## 📊 릴리즈 준비 상태 평가

### 릴리즈 배포: ✅ v1.0.0 배포 완료 (2026-02-01)

| 항목 | 상태 | 비고 |
|------|------|------|
| 빌드 안정성 | ✅ | cargo build/clippy 클린 |
| 테스트 통과율 | ✅ | 2,007 테스트 전체 통과, 241+ E2E |
| 예제 컴파일율 | ✅ | 110/111 (100%) + 1개 의도적 실패 |
| 보안 감사 | ✅ | 14개 이슈 전수 수정, cargo audit 통과 |
| 라이선스 | ✅ | 396개 의존성 감사, MIT/Apache-2.0 호환 |
| 배포 인프라 | ✅ | Homebrew, cargo install, .deb/.rpm, Docker, GitHub Releases |
| 문서화 | ✅ | mdBook, Quickstart, API 문서 65개 모듈, CLAUDE.md |
| CI/CD | ✅ | 3-OS 매트릭스, 코드 커버리지, cargo audit, 퍼즈 테스트 |
| 패키지 레지스트리 | ✅ | https://vais-registry.fly.dev — 10개 패키지 배포 |
| 셀프호스팅 | ✅ | 부트스트랩 달성 + MIR + LSP + Formatter + Doc Generator |

### 핵심 수치

| 지표 | 값 |
|------|-----|
| 전체 테스트 | 2,017+ (E2E 258+, 통합 256+) |
| 표준 라이브러리 | 65개 .vais + 19개 C 런타임 |
| 셀프호스트 코드 | 30,000+ LOC (컴파일러 + MIR + LSP + Formatter + Doc + Stdlib) |
| 컴파일 성능 | 50K lines → 79ms (641K lines/s) |
| 토큰 절감 | Rust 대비 30%+ |
| 실전 프로젝트 | 3개 (CLI, HTTP API, 데이터 파이프라인) |

---

## ⏳ 장기 관찰 항목

> 아래 항목은 시간 경과나 수작업이 필요하여 별도 관리합니다.

| 항목 | 출처 | 상태 | 비고 |
|------|------|------|------|
| 대형 프로젝트 6개월 모니터링 | Phase 22 | ⏳ | 프로토타입 검증 완료, 장기 안정성 관찰 중 |
| Instagram 프로필 완성 | Phase 26a | ⏳ | 수작업 필요 (계정/템플릿 준비 완료) |
| 1만 동시 TCP 연결 벤치마크 | Phase 37 Stage 6 | ⏳ | reactor 기반 비동기 I/O 통합 후 측정 예정 |
| 24시간 장시간 실행 메모리 누수 테스트 | Phase 37 Stage 6 | ⏳ | 인프라 구축 후 진행 |

---

## 🚀 Phase 41: v2.0 언어 진화

> **상태**: ✅ 완료
> **목표**: v1.0 안정 기반 위에서 언어 표현력, 생태계, 실전 활용도를 한 단계 끌어올림
> **방향**: 사용자 피드백 기반 개선 + 언어 기능 확장 + 생태계 성숙

### Stage 1: 셀프호스트 에러 복구 파서 ✅ 2026-02-07

**목표**: 여러 에러를 한번에 보고하는 에러 복구 모드

- [x] 파서 에러 복구 전략 구현 (synchronization tokens 기반) ✅
- [x] 첫 에러에서 중단하지 않고 최대 20개 에러 수집 ✅
- [x] 에러 위치 정보(줄:열) 정확도 개선 ✅
- [x] 에러 복구 E2E 테스트 10개 ✅
- **난이도**: 중 | **모델**: Opus 직접

### Stage 2: 클로저 캡처 개선 & 고차 함수 표준 패턴 ✅ 2026-02-07

**목표**: 클로저/람다의 실전 활용도 향상

- [x] selfhost parser에서 클로저 구문 완전 지원 (선택적 타입 어노테이션 포함) ✅
- [x] 표준 라이브러리 고차 함수 패턴: `map`, `filter`, `fold`, `for_each` (+any, all, find) ✅
- [x] 클로저 타입 추론 강화 (파라미터 타입 생략 가능: `|x| x * 2`) ✅
- [x] 클로저 E2E 테스트 10개 (268개 전부 통과) ✅
- **난이도**: 중 | **모델**: Opus 직접

### Stage 3: 에러 타입 체계화 & 에러 체이닝 ✅

**목표**: 프로덕션 수준 에러 처리 패턴

- [x] `Error` 트레이트 표준화 (message, source, backtrace) — `std/result.vais` W Error trait ✅
- [x] 에러 체이닝: `Result<T, E>.map_err()`, `context()` — map_err/and_then/or_else/context ✅
- [x] 사용자 정의 에러 타입 derive 매크로 — derive(Error) 매크로 등록 ✅
- [x] `anyhow`/`thiserror` 스타일 패키지 — `std/error.vais` AppError/ErrorChain ✅
- [x] enum impl 블록 지원 (EnumDef.methods + register_impl + MethodCall) ✅
- [x] E2E 테스트 11개 (에러/Result 관련) 전부 통과 ✅
- **난이도**: 중 | **모델**: Opus 직접

### Stage 4: 이터레이터 프로토콜 & 제너레이터 ✅

**목표**: 지연 평가 기반 데이터 처리 파이프라인

- [x] `Iterator` 트레이트 정의 (W Iterator + Range/VecIter/SliceIter) — `std/iter.vais` ✅
- [x] 이터레이터 어댑터: `iter_map`, `iter_filter`, `iter_take`, `iter_skip`, `iter_chain`, `iter_zip`, `iter_enumerate` ✅
- [x] `collect` 어댑터 (`collect_range` 패턴 + `iter_fold`) ✅
- [x] `L i:start..end { body }` 범위 for-루프 문법 ✅
- [x] `yield` 키워드 — lexer/parser/AST/type checker/codegen 전체 파이프라인 ✅
- [x] 소비 함수: `iter_sum`, `iter_product`, `iter_min`, `iter_max`, `iter_contains`, `iter_any`, `iter_all`, `iter_find`, `iter_position` ✅
- [x] E2E 테스트 16개 (범위루프, 어댑터, 클로저, 제너레이터 등) 전부 통과 ✅
- **난이도**: 상 | **모델**: Opus 직접

### Stage 5: 패키지 에코시스템 활성화 ✅

**목표**: 외부 기여자가 패키지를 만들고 공유할 수 있는 환경

- [x] `vaisc new <name>` 프로젝트 스캐폴딩 (vais.toml + src/main.vais + tests/ + .gitignore) ✅
- [x] `vaisc test` 명령 — 프로젝트 내 테스트 자동 탐색·실행·필터링·결과 요약 ✅
- [x] 패키지 의존성 트리 시각화 (`vaisc pkg tree` — 트리 포맷, 전이 의존성, depth 제한) ✅
- [x] 패키지 문서 자동 생성 (`vaisc pkg doc` — markdown/html, 인덱스 자동 생성) ✅
- [x] E2E 테스트 14개 추가 (new/test/pkg tree/pkg doc) 전부 통과 ✅
- **난이도**: 중 | **모델**: Opus 직접

### Stage 6: E2E 테스트 300개 달성 ✅

**목표**: 엣지 케이스 및 신규 기능 테스트 커버리지 확장

- [x] 에러 처리 E2E 11개 (Result, is_ok/is_err, unwrap_or, context, ensure) ✅
- [x] 이터레이터/고차 함수 E2E 16개 (map, filter, fold, take, skip, chain, zip, enumerate, any/all/find, position, collect) ✅
- [x] 추가 범용 E2E 9개 (재귀, @연산자, 비트연산, 다중 반환, 클로저 합성, 구조체 메서드, enum 매칭, 파이프라인) ✅
- [x] **총 301개 E2E 테스트**, 전부 통과 ✅
- **난이도**: 하 | **모델**: Opus 직접

### Stage 7: 성능 회귀 CI 자동화 ✅

**목표**: PR마다 성능 벤치마크를 자동 실행하여 회귀 방지

- [x] GitHub Actions에 criterion 벤치마크 자동 실행 (bench.yml + bench-regression.yml) ✅
- [x] PR 코멘트에 성능 비교 테이블 자동 게시 (actions/github-script PR 코멘트) ✅
- [x] 10% 이상 회귀 시 CI 실패 처리 (BENCH_THRESHOLD: 10) ✅
- [x] 벤치마크 히스토리 대시보드 (GitHub Pages — benchmark-action/github-action-benchmark) ✅
- **난이도**: 중 | **모델**: Sonnet 위임

### 검증 기준

| Stage | 검증 항목 |
|-------|----------|
| Stage 1 | 구문 에러 3개 있는 파일에서 3개 모두 보고 |
| Stage 2 | `[1,2,3].map(\|x\| x * 2)` E2E 통과 |
| Stage 3 | `?` 체이닝 3단계 에러 전파 + 커스텀 에러 타입 동작 |
| Stage 4 | `for x in vec.iter() { ... }` E2E 통과 |
| Stage 5 | `vaisc new` → `vaisc test` → `vaisc pkg tree` → `vaisc pkg doc` E2E 14개 통과 ✅ |
| Stage 6 | E2E 테스트 300개 이상, 전부 통과 |
| Stage 7 | PR에 성능 비교 테이블 자동 게시 |

### 우선순위

```
Stage 1 (에러 복구) ──→ Stage 6 (E2E 확장)
Stage 2 (클로저) ─────→ Stage 4 (이터레이터)
Stage 3 (에러 체계) ──→ Stage 5 (패키지)
                        Stage 7 (성능 CI)
```

### 완료 후 기대 효과
- 에러 복구로 개발자 경험 대폭 향상
- 이터레이터/고차 함수로 현대적 데이터 처리 가능
- 에러 타입 체계화로 프로덕션 에러 처리 패턴 완성
- 패키지 에코시스템 활성화로 커뮤니티 성장 기반
- E2E 300개 + 성능 CI로 품질 보증 강화

---

**메인테이너**: Steve

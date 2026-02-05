# Vais (Vibe AI Language for Systems) - AI-Optimized Programming Language
## 프로젝트 로드맵

> **버전**: 1.0.0
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-02-03

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
| 13 | 품질 보증 및 프로덕션 검증 | ✅ 완료 | 100% |
| 14 | 프로덕션 배포 및 커뮤니티 구축 | ✅ 완료 | 100% |
| 15 | v1.0 출시 준비 | ✅ 완료 | 100% |
| 16 | 실사용 검증 버그 수정 | ✅ 완료 | 예제 105/105 (100%) |
| 17 | 런타임 버그 수정 및 코드 품질 | ✅ 완료 | 100% |
| 18 | 코드젠 심층 버그 수정 및 float 지원 | ✅ 완료 | 100% |
| 19 | 대형 프로젝트 도입 준비 | ✅ 완료 | 100% |
| 20 | 근본적 문제 해결 | ✅ 완료 | 100% |
| 21 | 실사용 완성도 강화 | ✅ 완료 | 100% |
| 22 | 대형 프로젝트 도입 전략 | 🔄 진행 중 | 11/12 (92%) - 6개월 모니터링 잔여 |
| 23 | 코드젠 크로스플랫폼 호환성 | ✅ 완료 | 100% |
| 24 | Playground Linux 배포 호환성 | ✅ 완료 | 100% |
| 25 | Vararg float 타입 추론 버그 수정 | ✅ 완료 | 100% |
| 26a | 홍보 & 커뮤니티 성장 | 🔄 진행 중 | 3/4 (75%) - 프로필 완성 잔여 |
| 26b | 기술 부채 해결 - 타입 추론 일관성 | ✅ 완료 | 100% |
| 27 | GPU 코드젠 & Async 런타임 완성 | ✅ 완료 | 100% |
| 28 | GPU 런타임 실행 지원 | ✅ 완료 | Stage 1~4 완료 (27/27, 100%) |
| **29** | **토큰 절감 강화** | **✅ 완료** | **21/21 (100%)** |
| **30** | **성능 최적화** | **✅ 완료** | **29/29 (100%)** |
| **31** | **VaisDB 사전 준비 - 표준 라이브러리 시스템 프로그래밍 보강** | **✅ 완료** | **30/30 (100%)** |
| **32** | **표준 라이브러리 확장 - 웹/DB/템플릿** | **✅ 완료** | **7/7 (100%)** |
| **33** | **대형 프로젝트 도입 준비 - 프로덕션 블로커 해소** | **✅ 완료** | **7/7 (100%)** |
| **36** | **대형 프로젝트 도입 준비 (Production Readiness)** | **🔄 진행 중** | **6/8 (75%)** |
| **37** | **프로덕션 갭 해소 (Reality Check)** | **🔄 진행 중** | **Stage 5 완료 (selfhost 75%)** |
| **38** | **셀프호스팅 100% 달성** | **📋 계획** | **0/7 (0%) - 75% → 100%** |
| | *Phase 34~: VaisDB 본체 → 별도 repo (`vaisdb`)에서 진행* | | |

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
| **Phase 10** | Self-hosting | Stage 1+2 부트스트래핑 완료 (17,397줄 동일 IR 검증), 에러 복구, Macro Runtime, LSP 고도화 (Inlay Hints, Call Hierarchy), 패키지 레지스트리 서버, FFI 고도화, 크로스 컴파일 16개 타겟, DAP 서버, Formal Verification, inkwell 완전 전환 | 2026-01-26 |
| **Phase 11** | 프로덕션 준비 | Effect System, Dependent/Linear Types, Lifetimes, Associated Types, Tiered JIT, Concurrent GC, Lazy evaluation, 인터랙티브 튜토리얼, FFI bindgen, GPU 백엔드 (CUDA/Metal/AVX-512/NEON), 동적 모듈 로딩, WASM 샌드박싱, Alias Analysis, Auto-vectorization | 2026-01-27 |
| **Phase 12** | 프로덕션 안정화 | dead_code/clippy 정리, inkwell for loop 완성, auto_vectorize 완성, 에러 복구 강화, 유사 심볼 제안, Async Traits/Structured Concurrency/Async Drop, GAT/Const Traits/Variance, std/collections·crypto·async·fmt, Playground 서버, LSP 1.18+, MIR 도입, Query-based 아키텍처, AI 코드 완성, 보안 분석/SBOM, mdBook 문서 사이트 | 2026-01-29 |

---

## 📊 완료된 Phase 요약 (Phase 13~27)

> Phase 13~27의 상세 체크리스트는 git log를 참조하세요. 아래는 각 Phase의 핵심 성과 요약입니다.

| Phase | 이름 | 주요 성과 | 완료일 |
|-------|------|----------|--------|
| **Phase 13** | 품질 보증 및 프로덕션 검증 | E2E 테스트 89→128개, Windows CI, 코드 커버리지, Python 바인딩 재활성화, 에러 메시지 전수 검사, 비즈니스 로직 검증, Const Generics/Named Arguments/Procedural Macros, 패키지 레지스트리 배포, PGO/병렬 컴파일, LSP Code Lens/Inlay Hints/리팩토링 | 2026-01-29 |
| **Phase 14** | 프로덕션 배포 및 커뮤니티 구축 | 중첩 구조체/Enum 매칭/클로저 IR 버그 수정, print/println/format, 문자열 완성, 제네릭 monomorphization + vtable 동적 디스패치, Homebrew/cargo/Docker/Windows 배포, 웹사이트(랜딩/블로그/Playground), SNS(Instagram/Twitter/Discord), 브랜드 가이드, YouTube 튜토리얼 5편, Rosetta Code 10개 | 2026-01-31 |
| **Phase 15** | v1.0 출시 준비 | Box/Rc/Future 제네릭 등록, E2E 128/128 통과, Async 런타임(kqueue), 세대별 GC, 라이프타임/소유권 검사, ABI v1.0.0 안정화, GAT/Trait Object Safety/Specialization, 스트레스/메모리/퍼징/성능 테스트, v1.0.0 릴리스 노트, 보안 감사 14개 수정 | 2026-01-31 |
| **Phase 16** | 실사용 검증 버그 수정 | 45개 예제 실패 → 0개 (105/105 100%), Option/Vec self 타입, 클로저 IR, assert/contract 런타임, 제네릭 구조체 재정의, GC 링킹, 임포트 main 필터링 | 2026-01-31 |
| **Phase 17** | 런타임 버그 수정 및 코드 품질 | printf 포맷 검증, if-else 타입 추론, GC 런타임 검증, clippy 경고 수정, CLAUDE.md 생성, 런타임 출력 테스트 10개 | 2026-01-31 |
| **Phase 18** | 코드젠 심층 버그 수정 및 float 지원 | mutable 구조체 segfault 수정, LLVM float 상수 포매팅, float 이항/비교 연산, sin/cos/exp/log extern, GPU 예제 재작성 | 2026-01-31 |
| **Phase 19** | 대형 프로젝트 도입 준비 | unwrap→Result 전환, panic→graceful fallback, Borrow Checker 3rd pass 통합 (3-모드), HTTP/JSON/Regex 런타임, 패키지 매니저 의존성 해결, vais-query 증분 컴파일 | 2026-02-01 |
| **Phase 20** | 근본적 문제 해결 | pthread Thread 런타임, f64 포인터 역참조 codegen, std/gpu·hot·dynload stub 표기, 파서 재귀 깊이 안전장치 | 2026-02-01 |
| **Phase 21** | 실사용 완성도 강화 | Sync 런타임 (Mutex/RwLock/Condvar/Barrier/Semaphore/Atomics), void phi 노드 수정, E2E 152→165개 | 2026-02-01 |
| **Phase 22** | 대형 프로젝트 도입 전략 | 프로토타입 검증(239줄), 중형 프로젝트 패턴 검증(5종), Number Converter CLI, C 대비 벤치마크, 팀 온보딩 가이드. **잔여: 6개월 모니터링** | 2026-02-01 |
| **Phase 23** | 코드젠 크로스플랫폼 호환성 | Extern 함수 포인터 타입, Enum 패턴 매칭 LLVM 타입, ExprVisitor float 분기, Playground struct/enum 예제 수정 | 2026-02-01 |
| **Phase 24** | Playground Linux 배포 호환성 | SSA 네이밍 충돌, Enum GEP 인덱싱, Match phi 누락, Linux `-lm` 링킹, GitHub username 통일, Fly.io 배포, vaislang org 이전 | 2026-02-01 |
| **Phase 25** | Vararg float 타입 추론 버그 수정 | vararg float→i64 하드코딩 제거, Ternary/If/Match/Cast 타입 추론 보완, float printf E2E 4개 | 2026-02-01 |
| **Phase 26a** | 홍보 & 커뮤니티 성장 | Instagram 계정, 코드 카드 템플릿, 첫 3개 게시물. **잔여: 프로필 완성** | 2026-02-01 |
| **Phase 26b** | 기술 부채 해결 | f64 배열 codegen, GPU 커널 타입 추론, std 런타임 검증, 실사용 주의사항 갱신 | 2026-02-01 |
| **Phase 27** | GPU 코드젠 & Async 런타임 완성 | Metal CLI, 호스트 코드 생성, GPU E2E 9개, Async 빌트인(poll/kqueue/pipe/time), cooperative yield | 2026-02-01 |

---

## 📊 릴리즈 준비 상태 평가

### 릴리즈 배포: ✅ v1.0.0 배포 완료 (2026-02-01)

| 항목 | 상태 | 비고 |
|------|------|------|
| 빌드 안정성 | ✅ | cargo build/clippy 클린 |
| 테스트 통과율 | ✅ | 2,007 테스트 전체 통과, 210+ E2E |
| 예제 컴파일율 | ✅ | 110/111 (100%) + 1개 의도적 실패 |
| 보안 감사 | ✅ | 14개 이슈 전수 수정 완료 (Phase 15) |
| 라이선스 | ✅ | 396개 의존성 감사, MIT/Apache-2.0 호환 |
| 배포 인프라 | ✅ | Homebrew, cargo install, .deb/.rpm, Docker, GitHub Releases |
| 문서화 | ✅ | mdBook, Quickstart, CLAUDE.md, API 문서 |
| CI/CD | ✅ | 3-OS 매트릭스, 코드 커버리지, cargo audit |
| GitHub Release | ✅ | v1.0.0 - Linux/macOS Intel/macOS ARM/Windows |
| repo 공개 | ✅ | vaislang/vais public (2026-02-01) |

### 실사용 적합도

| 기능 영역 | 상태 | 비고 |
|-----------|------|------|
| 정수/f64 연산 | ✅ | 산술/비교/배열 인덱싱 모두 지원 |
| 구조체/열거형 | ✅ | 중첩, 패턴 매칭, 메서드 |
| 제네릭/트레이트 | ✅ | monomorphization + vtable 동적 디스패치 |
| 클로저/람다 | ✅ | 캡처, 함수 포인터 |
| 문자열 처리 | ✅ | 연결/비교/메서드/포맷 |
| 표준 라이브러리 | ✅ | thread/sync/http/gc C 런타임 완성 |
| GPU 코드젠 | ✅ | CUDA/OpenCL/WebGPU/Metal 4개 백엔드 코드 생성 |
| Async 런타임 | ✅ | kqueue I/O reactor, cooperative yield |
| GC | ✅ | 세대별 GC, 벤치마크 완료 |

---

## 🚀 Phase 28: GPU 런타임 실행 지원

> **상태**: ✅ 완료 (Stage 1~4 완료)
> **추가일**: 2026-02-01
> **목표**: GPU 코드 생성 → 실제 런타임 실행 엔드투엔드 파이프라인

### 배경

현재 GPU 코드젠은 4개 백엔드의 **커널 코드 텍스트 생성**만 지원. 런타임 API 연동, 메모리 관리(호스트↔디바이스) 등이 필요.

### 1단계 - CUDA 런타임 통합 ✅ 완료

- [x] **gpu_runtime.c** - CUDA Runtime API 래퍼 (cudaMalloc/cudaMemcpy/cudaFree/cudaLaunchKernel)
- [x] **메모리 관리 API** - gpu_alloc, gpu_free, gpu_memcpy_h2d/d2h
- [x] **커널 실행 API** - gpu_launch_kernel, gpu_synchronize
- [x] **디바이스 관리** - gpu_device_count, gpu_set_device, gpu_get_properties
- [x] **컴파일러 통합** - `--gpu cuda --gpu-compile` nvcc 자동 호출 + 링킹 + 에러 처리
- [x] **std/gpu.vais stub 교체** - thread_idx, block_idx, sync_threads, atomic, math, 호스트 런타임 API
- [x] **E2E 테스트** - 벡터 덧셈, 행렬 곱셈, 리덕션 (4개 백엔드 코드생성)

### 2단계 - Metal 런타임 통합 (macOS) ✅ 완료

- [x] **metal_runtime.m** - Metal API Objective-C 래퍼 + metal_runtime.h
- [x] **MTLDevice/MTLCommandQueue/MTLBuffer 관리** - 초기화, 버퍼 할당, shared memory
- [x] **커널 실행** - MTLComputePipelineState + dispatch + auto-dispatch
- [x] **컴파일러 통합** - `--gpu metal --gpu-compile` xcrun metal/metallib 자동 호출
- [x] **E2E 테스트** - Metal 벡터 덧셈, SAXPY, 멀티 커널

### 3단계 - OpenCL 런타임 통합 (크로스플랫폼) ✅ 완료

- [x] **opencl_runtime.c** - OpenCL API 래퍼 + opencl_runtime.h
- [x] **플랫폼/디바이스 탐색** - GPU 우선, CPU 폴백
- [x] **커널 JIT 컴파일** - 소스/파일 기반 런타임 컴파일
- [x] **컴파일러 통합** - `--gpu opencl --gpu-compile` 자동 빌드 + 링킹
- [x] **E2E 테스트** - OpenCL 벡터 연산, SAXPY, 멀티커널

### 4단계 - 고급 기능 ✅ 완료

- [x] **통합 메모리 (Unified Memory)** - CUDA managed memory + prefetch/advise 힌트, Metal shared memory
- [x] **스트림/비동기 실행** - CUDA async memcpy + stream event, Metal async dispatch, OpenCL async dispatch
- [x] **다중 GPU** - CUDA peer access (P2P enable/disable/memcpy), Metal device_select, OpenCL device_select
- [x] **프로파일링 통합** - CUDA event-stream 기록, Metal CFAbsoluteTime 기반 이벤트, OpenCL 이벤트 프로파일링

### 검증 기준

| 단계 | 검증 항목 | 상태 |
|------|----------|------|
| 1단계 | CUDA 벡터 덧셈 E2E (호스트→GPU→실행→결과→검증) | ✅ |
| 2단계 | Metal 벡터 덧셈 E2E (macOS) | ✅ |
| 3단계 | OpenCL 벡터 덧셈 E2E (크로스플랫폼) | ✅ |
| 4단계 | Unified Memory + 비동기 실행 + 멀티GPU + 프로파일링 E2E 테스트 | ✅ |

### 의존성

- CUDA Toolkit (nvcc, libcudart)
- Metal Framework (macOS 전용, Xcode)
- OpenCL SDK (platform별)

---

## 🚀 Phase 29: 토큰 절감 강화 - AI 코드 생성 최적화

> **상태**: ✅ 완료 (21/21, 100%)
> **목표**: 기존 언어 대비 토큰 절감률 10-15% → 30-40%로 향상
> **핵심 지표**: tiktoken (cl100k_base) 기준 동일 로직 Rust 코드 대비 토큰 수 비교

### 1단계 - 문자열 보간 (최우선) ✅ 완료

> 출력 코드에서 토큰 낭비 최대 원인 제거. 예상 절감: +8%

- [x] **AST: StringInterp 노드 추가** - `StringInterpPart` enum (Lit/Expr) + `Expr::StringInterp` variant
- [x] **파서: 보간 표현식 파싱** - `{expr}` 내부를 서브 렉서+파서로 파싱, 중첩 지원, `{{`/`}}` 이스케이프, 빈 `{}` 호환
- [x] **코드젠: 보간 → printf/snprintf 변환** - LLVM IR에서 포맷 스트링 생성, format()/print()/println() 통합
- [x] **println 빌트인 통합** - `println("x={x}, y={y}")` → printf 직접 호출 (힙 할당 없음)
- [x] **테스트** - 보간 문자열 E2E 테스트 5개 (변수, 산술, 이스케이프, 하위 호환, 다중 표현식)

### 2단계 - 함수 파라미터 타입 추론 확장 ✅ 완료

> 타입 어노테이션이 전체 토큰의 10-20% 차지. 예상 절감: +10-15%

- [x] **타입 체커: 호출부 기반 역방향 추론** - 함수 호출 시 인자 타입에서 파라미터 타입 추론, 유니피케이션 변수 기반
- [x] **파서: 타입 생략 허용** - `F add(a, b) = a + b` 형태 지원, 콜론 없으면 Type::Infer 삽입
- [x] **다중 호출부 통합 추론** - 유니피케이션을 통해 본문+호출부 타입 정보 합산
- [x] **추론 실패 시 기본 타입** - 미결정 타입 변수는 i64로 기본 설정
- [x] **테스트** - 타입 생략 함수 선언, 재귀 함수, 혼합 선언 E2E 테스트 5개

### 3단계 - 키워드 축약 & 신규 연산자

> 남은 다중 문자 키워드 축약. 예상 절감: +3-5%

- [x] **`mut` → `~` 변경** - `~`를 `mut` 대체 키워드로 파서에서 지원, 바인딩/파라미터/참조 타입에서 사용 가능
- [x] **`await` → `Y` 변경** - 비동기 대기 키워드 단일 문자화, `.Y` 포스트픽스 + `.await` 호환
- [x] **파이프 연산자 `|>` 구현** - `x |> f |> g` → `g(f(x))` 변환, 좌결합 연산자
- [x] **스프레드 문법 `..` 구현** - 파서/AST/타입체커에 `..expr` 스프레드 노드 추가, 배열 리터럴 내 사용 지원
- [x] **암시적 self 생략** - 메서드 내에서 `self.x` → `x` (스코프 내 필드 자동 해석, 로컬 변수 우선)
- [x] **테스트** - `~` mut E2E 4개 + `|>` 파이프 E2E 5개 + 렉서 테스트 2개

### 4단계 - 고급 축약 문법

> 반복 패턴 축약. 예상 절감: +2-3%

- [x] **다중 조건 가드** - `I a && b && c { ... }` 패턴 최적화 (기존 논리 연산자로 지원)
- [x] **배열 리터럴** - `[1, 2, 3]` 배열 리터럴 (기존 구현)
- [x] **맵 리터럴** - `{k: v}` 맵 리터럴
- [x] **디스트럭처링** - `(a, b) := get_pair()` 튜플 분해
- [x] **범위 연산자** - `0..n` 범위 표현식 (기존 구현, `..=` 포함)

### 검증 기준

| 단계 | 검증 항목 |
|------|----------|
| 1단계 | `println("x={x}")` E2E 통과, puts/putchar 대비 토큰 50%+ 절감 |
| 2단계 | 타입 생략 함수 10개 이상 컴파일 통과, 기존 테스트 회귀 없음 |
| 3단계 | `~`, `Y`, `\|>` 렉서/파서/코드젠 통과, 기존 `mut` 호환 |
| 4단계 | 배열 리터럴, 디스트럭처링 E2E 통과 |
| 전체 | tiktoken 기준 Rust 대비 30%+ 토큰 절감 벤치마크 통과 |

---

## 🚀 Phase 30: 성능 최적화 - C/Rust급 실행 속도 달성

> **상태**: 🔄 진행 중 (Stage 1-3 완료 - inkwell 기본 백엔드 + TCO + 인라이닝 & PGO, E2E 210/210 통과)
> **목표**: C 대비 실행 속도 갭 10-20% → 5% 이내
> **핵심 지표**: fibonacci(40), matrix_mul, sort 벤치마크에서 C -O2 대비 비교

### 1단계 - Inkwell 백엔드 기본 전환 (최우선)

> 텍스트 기반 IR → LLVM API 직접 호출. 컴파일+런타임 모두 개선

- [x] **inkwell 백엔드 AST 동기화** - 78개 컴파일 에러 수정 (ResolvedType/BinOp/Spanned 변경 대응, inkwell 0.4 opaque pointer API)
- [x] **inkwell 백엔드 CLI 통합** - `--inkwell` 플래그로 opt-in 사용 가능 (`vaisc build --inkwell` 또는 `vaisc --inkwell`)
- [x] **텍스트 백엔드 호환 유지** - 텍스트 백엔드가 기본값으로 유지, inkwell은 opt-in
- [x] **inkwell 빌트인 함수 대폭 확장** - putchar/fopen/GC/thread/sync/stdlib 등 80+ extern 등록, println/print/format/store_i64/load_i64/store_byte/load_byte 인라인 구현
- [x] **inkwell 표현식 지원 확대** - Assert/Comptime/Lazy/Await/Force/Spawn/SelfCall/StaticMethodCall/StringInterp/Spread/LetDestructure 지원 추가 (예제 통과율 22% → 55%)
- [x] **inkwell 기능 패리티 강화** - Impl/Struct 메서드 선언·생성, 메서드 호출 해석(TypeName_method), var_struct_types 기반 구조체 타입 추론, Option/Result 생성자(Some/None/Ok/Err), 열거형 variant 식별자, 구조체 리터럴 런타임 값 지원 (예제 통과율 55% → 61%)
- [x] **inkwell 제네릭/클로저/extern 패리티** - Generic 타입 i64 폴백, 함수/메서드/구조체 제네릭 치환, ExternBlock 선언 처리, Union 지원, SelfCall 메서드 해석, 클로저 캡처 자동 감지·전달, 추가 빌트인 80+ 등록 (예제 통과율 61% → 75%)
- [x] **inkwell 통과율 85% → 98% (99→115/117)** - PHI 노드 predecessor 불일치 수정 (if-else/ternary/match에서 terminate된 블록 체크), generate_block 조기 종료, 슬라이스 인덱싱 지원 (Range→slice 연산), SIMD 함수 인라인 정의 (vec4i32/vec4f32/vec2i64 생성자+연산+리듀스)
- [x] **inkwell 런타임 패리티 강화** - self를 포인터로 전달하여 메서드 내 변이 반영 (8개 예제 수정), defer 스택 LIFO 구현, #[requires] contract 속성 처리, 열거형 패턴 매칭 태그 비교 수정, Expr::Ref lvalue 포인터 반환, puts+StringInterp 크래시 수정 (런타임 출력 일치 79→88/117)
- [x] **inkwell 백엔드 기본값 전환** - `vais-codegen` default feature를 `inkwell-codegen`으로 변경, `vaisc` default에 `inkwell` 추가, 컴파일 시 자동으로 inkwell 사용
- [x] **inkwell 코드젠 완성도 검증** - E2E 테스트 210개 전부 통과 확인 (text/inkwell 모두)
- [x] **컴파일 속도 벤치마크** - inkwell이 텍스트 대비 ~36% 빠름 (10회 반복 측정: 1.52s vs 2.39s)

### 2단계 - Tail Call Optimization (TCO) ✅

> `@` 재귀 연산자가 핵심 기능인데 TCO 없으면 스택 오버플로 위험

- [x] **꼬리 호출 패턴 감지** - inkwell 백엔드에서 AST 레벨 tail position 분석 (Ternary/If/Match/Block 분기 탐색)
- [x] **LLVM `musttail` 어노테이션 생성** - 텍스트 백엔드 optimize.rs에 tail/musttail 마킹 패스 추가
- [x] **꼬리 재귀 → 루프 변환** - inkwell 백엔드에서 TCO 감지 시 루프 기반 코드 생성 (TcoState로 파라미터 alloca 업데이트 + loop header 점프)
- [x] **MIR TailCall 터미네이터** - vais-mir에 Terminator::TailCall 변형 추가
- [x] **테스트** - countdown(10000000), sum_acc(1000) 스택 오버플로 없이 통과 (E2E 178/178 유지)

### 3단계 - 인라이닝 & 프로파일 기반 최적화 ✅

> 보수적 인라이닝 임계값 확대 + PGO 연동

- [x] **인라이닝 임계값 상향** - optimize.rs 10 → 50 명령어로 확대, 2-tier 시스템 (≤10: store 허용, ≤50: 순수 함수만)
- [x] **호출 빈도 기반 인라인 판단** - count_call_sites()로 호출 빈도 분석, 핫 함수 우선 인라인 (빈도 내림차순 → 크기 오름차순 정렬)
- [x] **PGO 파이프라인 연동** - optimize_ir_with_pgo() 도입, PGO Generate/Use 모드를 최적화 파이프라인에 통합, 메인 빌드에서 자동 적용
- [x] **벤치마크** - inlining_bench.rs 추가 (소형/중형/핫 함수 인라이닝, O0~O3 레벨 비교)

### 4단계 - MIR 기반 최적화 파이프라인 ✅

> 텍스트 문자열 매칭 → 구조적 CFG 기반 최적화

- [x] **AST → MIR 변환 활성화** - `lower.rs`: AST → MIR 변환 구현 (함수, if/else, let, match, call, @재귀 등)
- [x] **MIR 레벨 DCE** - `optimize.rs`: Dead Code Elimination (미사용 로컬 제거)
- [x] **MIR 레벨 CSE** - `optimize.rs`: Common Subexpression Elimination + 상수 전파 + 도달불가 블록 제거
- [x] **MIR → LLVM IR 변환** - `emit_llvm.rs`: MIR에서 직접 LLVM IR 텍스트 생성

### 5단계 - 경계 검사 제거 & 고급 최적화 ✅

> 안전성 유지하면서 불필요한 런타임 검사 제거

- [x] **범위 분석 (Range Analysis)** - `bounds_check_elim.rs`: 루프 유도 변수 범위 증명, 가드 기반 제거, 상수 인덱스 분석
- [x] **증명된 안전 접근 → 검사 제거** - `eliminate_bounds_checks()`: 검증된 안전 접근 시 조건분기 → 무조건 분기로 변환
- [x] **SIMD 자동 벡터화 강화** - `auto_vectorize.rs`: 리덕션 패턴 감지 (Add/Mul/Min/Max/Or/And/Xor)
- [x] **캐시 친화적 데이터 레이아웃** - `data_layout.rs`: `suggest_field_reorder()`, `padding_savings()` 필드 재배열 힌트

### 6단계 - 벤치마크 스위트 & 성능 검증 ✅

> 측정 없이 개선 없음

- [x] **tiktoken 토큰 카운트 벤치마크** - Vais vs Rust (0.76x) vs C (0.65x) 토큰 절감 검증
- [x] **런타임 성능 벤치마크 확장** - fibonacci/matrix_mul/quicksort/binary_tree (performance_bench.rs)
- [x] **컴파일 속도 벤치마크** - 166/1666/8331 LOC 전체 파이프라인 벤치마크
- [x] **메모리 사용량 벤치마크** - IR/소스 비율 측정 (2.14x~5.83x), AST 노드 카운트
- [x] **자동 회귀 테스트** - `regression_check.sh` CI 스크립트 (10% 임계값 기반 자동 경고)

### 검증 기준

| 단계 | 검증 항목 |
|------|----------|
| 1단계 | 기존 E2E 테스트 128개 inkwell 백엔드로 전부 통과 |
| 2단계 | factorial(100000) 스택 오버플로 없이 통과 |
| 3단계 | fibonacci(40) 성능 C -O2 대비 10% 이내 |
| 4단계 | MIR 파이프라인 통과 + 기존 테스트 회귀 없음 |
| 5단계 | 배열 루프 벤치마크 C 대비 5% 이내 |
| 6단계 | 토큰 절감 30%+, 런타임 C 대비 5% 이내 벤치마크 통과 |

### 의존성

- LLVM 17+ (inkwell 0.4)
- Criterion (벤치마크)
- tiktoken / cl100k_base (토큰 측정)

---

## 🚀 Phase 31: VaisDB 사전 준비 - 표준 라이브러리 시스템 프로그래밍 보강

> **상태**: ⏳ 예정
> **목표**: VaisDB (하이브리드 RAG 네이티브 데이터베이스) 구축에 필요한 저수준 시스템 프리미티브 보강
> **배경**: 벡터+그래프+관계형+키워드 검색을 단일 DB로 통합하는 VaisDB 프로젝트를 순수 Vais로 구현하기 위해 표준 라이브러리의 시스템 프로그래밍 역량을 보강한다.

### 1단계 - 파일 시스템 내구성 (ACID 필수) ✅ 완료

> fsync 없이는 ACID의 D(Durability)를 보장할 수 없음. DB 엔진의 가장 기본 요구사항.

- [x] **`fsync`/`fdatasync` FFI 추가** - `std/file.vais`에 `F fsync(fd: i64) -> i64` extern 선언, POSIX `fsync()` 바인딩
- [x] **`file_sync()` 고수준 API** - File 구조체에 `sync()` / `datasync()` 메서드 추가, 쓰기 후 디스크 플러시 보장
- [x] **디렉토리 동기화** - `dir_sync()` 추가 (메타데이터 내구성 보장, WAL 파일명 변경 후 필수)
- [x] **테스트** - fsync/fileno/dir_sync E2E 테스트 4개 통과 (214/214)

### 2단계 - 메모리 매핑 (페이지 캐싱 필수) ✅ 완료

> DB 엔진의 효율적인 디스크 I/O를 위해 mmap 필수. 버퍼 풀 없이 모든 읽기가 stdio를 거침.

- [x] **`mmap`/`munmap` FFI 추가** - POSIX mmap 바인딩, `PROT_READ`/`PROT_WRITE`/`MAP_SHARED`/`MAP_PRIVATE` 플래그
- [x] **`MappedFile` 구조체** - 파일을 메모리에 매핑하는 고수준 API (map/map_read/map_readwrite/unmap/read_byte/write_byte)
- [x] **`madvise` 힌트** - `MADV_SEQUENTIAL`/`MADV_RANDOM`/`MADV_WILLNEED` 지원
- [x] **`msync` 지원** - 매핑된 메모리의 변경사항을 디스크에 동기화 (sync/sync_async)
- [x] **테스트** - mmap 읽기, 쓰기+msync, invalid fd, madvise E2E 테스트 4개

### 3단계 - 파일 잠금 (동시 접근 필수) ✅ 완료

> 다중 프로세스가 동일 DB 파일에 접근할 때 데이터 손상 방지.

- [x] **`flock` FFI 추가** - `LOCK_SH`/`LOCK_EX`/`LOCK_NB`/`LOCK_UN` 바인딩
- [x] **`fcntl` 레코드 잠금** - 바이트 범위 잠금 (영역별 동시 접근 허용)
- [x] **`FileLock` 고수준 API** - RAII 기반 잠금, 스코프 종료 시 자동 해제
- [x] **테스트** - 다중 프로세스 동시 잠금 시나리오, 교착 상태 방지 검증

### 4단계 - 할당자 상태 변이 수정 ✅ 완료

> standalone 함수 → impl 블록(`X StructName { ... }`) 기반 `&self` 메서드로 전환하여 포인터 기반 상태 변이 정상 작동.

- [x] **할당자 포인터 기반 상태 관리** - 모든 할당자를 `X Type { F method(&self) }` impl 블록으로 전환
- [x] **BumpAllocator 수정** - offset/allocated 증가가 실제 반영되도록 `alloc()` 수정
- [x] **PoolAllocator 수정** - free_list/num_free 갱신이 실제 반영되도록 수정
- [x] **FreeListAllocator 수정** - 할당/해제 시 free_list 연결 리스트 정상 갱신 + 블록 분할
- [x] **StackAllocator 수정** - offset/prev_offset 포인터 갱신 반영
- [x] **테스트** - E2E 4개 (bump/pool/freelist/stack) 연속 할당 주소 유일성, 해제 후 재할당 검증 (225/225)

### 5단계 - 문자열 키 HashMap & 소유 문자열 ✅ 완료

> SQL 테이블명, 컬럼명, 인덱스명 등 문자열 키 기반 조회 필수. 현재 HashMap은 i64 키만 지원.

- [x] **문자열 해시 함수** - DJB2 기반 `hash_string()` 구현 (std/hash.vais)
- [x] **문자열 비교 함수** - `strmap_str_eq()` 바이트 단위 비교 (HashMap 키 비교용)
- [x] **`StringMap` 타입** - 문자열 키 전용 HashMap, 해시+비교 내장 (std/stringmap.vais)
- [x] **소유 문자열 (OwnedString)** - 길이 추적 + 힙 할당 + 자동 해제, `str`과 변환 가능 (std/owned_string.vais)
- [x] **테스트** - 5개 E2E: 삽입/조회, 충돌 해결, 삭제/재삽입, OwnedString 생명주기, 동적 키 빌드

### 6단계 - 디렉토리 & 파일 시스템 조작 ✅ 완료

> DB 파일 관리, WAL 세그먼트 로테이션, 임시 파일 등에 필요.

- [x] **`mkdir`/`rmdir` FFI** - 디렉토리 생성/삭제
- [x] **`opendir`/`readdir`/`closedir`** - 디렉토리 탐색 (readdir wrapper with d_name extraction)
- [x] **`rename_file` FFI** - 원자적 파일명 변경 (WAL 커밋에 핵심)
- [x] **`unlink` FFI** - 파일 삭제
- [x] **`stat_size`/`stat_mtime` FFI** - 파일 크기/수정 시간 조회
- [x] **`getcwd`/`chdir` FFI** - 작업 디렉토리 조회/변경
- [x] **테스트** - 3개 E2E: mkdir/rmdir, rename/unlink, stat_size

### 7단계 - 바이너리 직렬화 ✅ 완료

> DB 페이지, 인덱스 노드, WAL 레코드 등 디스크 포맷에 바이너리 직렬화 필수. JSON은 너무 느림.

- [x] **`ByteBuffer` 타입** - 가변 크기 바이트 버퍼, 리틀엔디안 읽기/쓰기 (std/bytebuffer.vais)
- [x] **정수 인코딩** - `write_u8`/`write_i32_le`/`write_i64_le` + 대응 read
- [x] **문자열 인코딩** - 길이 접두 문자열 (`write_str`)
- [x] **CRC32 체크섬** - IEEE 802.3 DJB2 기반 비트 단위 계산 (std/crc32.vais)
- [x] **테스트** - 4개 E2E: 정수 직렬화, 버퍼 성장, CRC32 검증값, ByteBuffer+CRC32 통합

### 8단계 - 에러 전파 개선 ✅ 완료

> DB 엔진은 I/O 에러, 파싱 에러, 타입 에러 등 다층 에러 처리가 필요.

- [x] **`?` 연산자 코드젠 완성** - Result/Option 체이닝이 실제 LLVM IR로 정상 생성 (extractvalue 기반 태그 검사 + 조기 반환)
- [x] **`!` 연산자 코드젠 수정** - Unwrap도 동일한 enum 구조체 레이아웃 사용
- [x] **타입 체커 확장** - user-defined `E Result { Ok(i64), Err(i64) }`에 대해 Try/Unwrap 허용
- [x] **테스트** - 4개 E2E: Ok 추출, Err 전파, 체이닝, Result 헬퍼 함수

### 검증 기준

| 단계 | 검증 항목 | 우선순위 |
|------|----------|----------|
| 1단계 | fsync 후 데이터 영속 확인, File.sync() E2E | ✅ 완료 |
| 2단계 | 1GB 파일 mmap 읽기/쓰기 E2E | ✅ 완료 |
| 3단계 | 2개 프로세스 동시 flock 검증 | ✅ 완료 |
| 4단계 | 1000회 연속 alloc 후 주소 유일성 검증 | ✅ 완료 |
| 5단계 | 10만 개 문자열 키 HashMap 삽입/조회 성능 | 🟡 높음 |
| 6단계 | 디렉토리 CRUD + rename 원자성 | 🟢 중간 |
| 7단계 | 바이너리 라운드트립 100% 정합성 | 🟢 중간 |
| 8단계 | `?` 체이닝 3단계 이상 에러 전파 | ✅ 완료 |

### 의존성

- Phase 29 (토큰 절감) 완료 권장 (신규 문법을 활용하여 VaisDB 코드 작성)
- Phase 30 (성능 최적화) 병행 가능 (성능 개선은 DB에도 직접 혜택)

### 이후 단계

Phase 31 완료 후 **VaisDB 본체 구현**은 별도 repo (`vaisdb`)에서 진행:
- 📦 repo: `vaisdb/` (별도 프로젝트)
- VaisDB ROADMAP은 해당 repo에서 관리

---

## Phase 32: 표준 라이브러리 확장 - 웹/DB/템플릿

> 브랜치: `develop` → 완료 후 `main`에 merge

### 목표
Vais로 웹 서버, DB 연동, 풀스택 애플리케이션 개발이 가능하도록 표준 라이브러리를 확장한다.

### 기존 기반
- `std/net.vais` (1,143줄) - TCP/UDP 소켓, IPv4/IPv6
- `std/http.vais` (867줄) + `std/http_runtime.c` (531줄) - HTTP 파싱, 요청/응답
- `std/json.vais` (840줄) - JSON 파서/생성자/stringify

### Stage 1: HTTP 서버 프레임워크 ✅
- [x] 라우터 (경로 매칭, HTTP 메서드별 핸들러 등록)
- [x] 미들웨어 체인 (로깅, CORS, 인증)
- [x] 정적 파일 서빙
- [x] 요청/응답 빌더 (헤더, 상태코드, JSON 응답)
- **파일**: `std/http_server.vais`
- **의존성**: `std/net.vais`, `std/http.vais`, `std/json.vais`, `std/file.vais`

### Stage 2: HTTP 클라이언트 ✅
- [x] GET/POST/PUT/DELETE 요청
- [x] 커스텀 헤더, 타임아웃
- [x] JSON 요청/응답 편의 함수
- [x] Keep-alive 커넥션 풀링
- [x] 리다이렉트 자동 추적
- [x] 인증 헬퍼 (Bearer, Basic)
- **파일**: `std/http_client.vais` + `std/http_client_runtime.c`
- **의존성**: `std/net.vais`, `std/http.vais`, `std/json.vais`

### Stage 3: SQLite 바인딩 ✅
- [x] DB 열기/닫기
- [x] SQL 실행 (exec, query)
- [x] Prepared statements (바인드 파라미터)
- [x] 트랜잭션 (begin/commit/rollback)
- **파일**: `std/sqlite.vais` + `std/sqlite_runtime.c`
- **의존성**: FFI (libsqlite3)

### Stage 4: PostgreSQL 바인딩 ✅
- [x] 연결/해제
- [x] 쿼리 실행, 파라미터 바인딩
- [x] 결과 셋 순회
- [x] 커넥션 풀 (기초)
- **파일**: `std/postgres.vais` + `std/postgres_runtime.c`
- **의존성**: FFI (libpq)

### Stage 5: 템플릿 엔진 ✅
- [x] 변수 치환 (`{{name}}`)
- [x] 조건문 (`{% if %}...{% endif %}`)
- [x] 반복문 (`{% for %}...{% endfor %}`)
- [x] HTML 이스케이핑
- [x] 필터 (upper, lower, escape, trim, length)
- [x] 파셜 include
- **파일**: `std/template.vais` + `std/template_runtime.c`
- **의존성**: 순수 C 런타임

### Stage 6: WebSocket ✅
- [x] 핸드셰이크 (HTTP Upgrade)
- [x] 프레임 파싱/생성 (텍스트/바이너리)
- [x] ping/pong
- [x] 연결 관리
- [x] RFC 6455 준수 (SHA-1/Base64 키 계산, 마스킹)
- **파일**: `std/websocket.vais` + `std/websocket_runtime.c`
- **의존성**: `std/http_runtime.c` (TCP 소켓)

### Stage 7: 경량 ORM ✅
- [x] 스키마 정의 → 테이블 매핑 (CREATE TABLE 생성)
- [x] 쿼리 빌더 (select, insert, update, delete)
- [x] WHERE 조건 빌더 (eq, gt, lt, AND/OR)
- [x] 마이그레이션 시스템 (up/down)
- [x] SQL 인젝션 방지 (자동 이스케이핑)
- **파일**: `std/orm.vais` + `std/orm_runtime.c`
- **의존성**: `std/sqlite.vais`, `std/postgres.vais`

### 검증 기준

| 단계 | 검증 항목 |
|------|----------|
| Stage 1 | "Hello World" HTTP 서버 + JSON API 엔드포인트 E2E |
| Stage 2 | HTTP GET/POST 요청 + JSON 파싱 E2E |
| Stage 3 | SQLite CRUD + 트랜잭션 E2E |
| Stage 4 | PostgreSQL 연결 + 쿼리 E2E |
| Stage 5 | HTML 템플릿 렌더링 E2E |
| Stage 6 | WebSocket echo 서버 E2E |
| Stage 7 | ORM으로 CRUD 수행 E2E |

---

## Phase 33: 대형 프로젝트 도입 준비 - 프로덕션 블로커 해소

> 브랜치: `develop` → 완료 후 `main`에 merge

### 목표
Vais를 프로덕션 환경의 대형 프로젝트에 도입할 수 있도록, 보안(TLS), 크로스플랫폼 비동기 I/O, 패키지 생태계, 디버깅, 관측성(로깅), 압축 등 핵심 블로커를 해소한다.

### 현재 블로커 분석
- **TLS/HTTPS 부재**: 보안 웹 서비스 불가 (Phase 32 HTTP 서버/클라이언트가 평문 전용)
- **Async 런타임 macOS 전용**: kqueue만 구현, Linux(epoll)/Windows(IOCP) 미지원
- **패키지 레지스트리 미배포**: 서버 코드 존재하나 공개 운영 없음
- **디버거 미검증**: DAP 서버 존재하나 실제 디버깅 워크플로 테스트 없음
- **구조화 로깅 없음**: 프로덕션 모니터링/트레이싱 불가
- **압축 라이브러리 없음**: HTTP gzip, 데이터 직렬화에 필수

### Stage 1: TLS/HTTPS 표준 라이브러리
- [x] TLS 컨텍스트 생성/해제 (OpenSSL/LibreSSL FFI)
- [x] 인증서 로드 (PEM 파일, CA 번들)
- [x] TLS 핸드셰이크 (클라이언트/서버)
- [x] 암호화 읽기/쓰기 (`tls_read`, `tls_write`)
- [x] HTTPS 서버 통합 (기존 `http_server.vais` 확장)
- [x] HTTPS 클라이언트 통합 (기존 `http_client.vais` 확장)
- **파일**: `std/tls.vais` + `std/tls_runtime.c`
- **의존성**: FFI (libssl, libcrypto)

### Stage 2: Async 런타임 크로스플랫폼
- [x] Linux epoll 백엔드 (`std/async_epoll.c`)
- [x] Windows IOCP 백엔드 (`std/async_iocp.c`)
- [x] macOS kqueue 헬퍼 (`std/async_kqueue.c`)
- [x] 통합 이벤트 루프 추상화 (`std/async_reactor.vais`)
- [x] 타이머/타임아웃 지원 (timerfd/CreateTimerQueueTimer/kqueue)
- [x] Async TCP accept/read/write (Reactor.register_read/write)
- [x] 플랫폼 자동 감지 (`async_platform()` + 조건부 컴파일)
- [x] 코드젠 builtin 등록 (`async_platform`, `epoll_set_timer_ms`, `iocp_set_timer_ms`)
- **파일**: `std/async_reactor.vais` + `std/async_epoll.c` + `std/async_iocp.c` + `std/async_kqueue.c`
- **의존성**: 기존 `std/async.vais` 확장

### Stage 3: 패키지 레지스트리 배포 준비
- [x] `vais publish` CLI 명령 구현 (tarball 생성 + 업로드 + 체크섬 검증)
- [x] `vais install` CLI 명령 구현 (의존성 해결 + 다운로드)
- [x] Semver 의존성 해석기 (^, ~, >= 지원)
- [x] 패키지 서명 검증 (SHA-256 체크섬)
- [x] 레지스트리 서버 Docker 이미지 작성
- **파일**: `crates/vaisc/src/main.rs` + `Dockerfile.registry` + `docker-compose.registry.yml`
- **의존성**: 기존 `vais-registry-server`

### Stage 4: 디버거(DAP) 실사용 검증
- [x] 브레이크포인트 설정/해제 E2E 테스트
- [x] 변수 검사 (locals, globals) 검증
- [x] 스텝 오버/스텝 인/스텝 아웃 검증
- [x] 콜 스택 조회 검증
- [x] VSCode launch.json 자동 생성
- [x] 발견된 버그 수정
- **파일**: `crates/vais-dap/tests/e2e_tests.rs` + `vscode-vais/`
- **의존성**: 기존 `vais-dap`

### Stage 5: 구조화 로깅 + 에러 트레이싱
- [x] 로그 레벨 (TRACE, DEBUG, INFO, WARN, ERROR)
- [x] 구조화 필드 (key=value 형식)
- [x] JSON 로그 출력 포맷
- [x] 파일/stdout/stderr 출력 대상 선택
- [x] 스팬(span) 기반 트레이싱 (요청 추적)
- **파일**: `std/log.vais` + `std/log_runtime.c`
- **의존성**: 없음 (순수 구현)

### Stage 6: 압축 라이브러리 (gzip/deflate)
- [x] Deflate 압축/해제 (RFC 1951)
- [x] Gzip 래핑 (RFC 1952, 헤더/CRC32)
- [x] 스트리밍 압축 (청크 단위)
- [x] HTTP Content-Encoding 통합
- **파일**: `std/compress.vais` + `std/compress_runtime.c`
- **의존성**: FFI (zlib)

### Stage 7: 통합 E2E 검증 + 벤치마크
- [x] HTTPS 서버 + TLS 클라이언트 통합 테스트
- [x] Async I/O 크로스플랫폼 테스트 (macOS/Linux)
- [x] 패키지 publish/install 라운드트립 테스트
- [x] 구조화 로깅 출력 검증
- [x] 성능 벤치마크 (HTTP throughput, DB ops/sec, TLS handshake latency)
- [x] 프로덕션 체크리스트 문서 작성
- **파일**: `crates/vaisc/tests/` + `benches/` + `docs-site/`
- **의존성**: Stage 1~6 완료 필수

### 검증 기준

| 단계 | 검증 항목 |
|------|----------|
| Stage 1 | HTTPS 서버로 curl 요청 성공 + 인증서 검증 |
| Stage 2 | Linux에서 async TCP echo 서버 동작 |
| Stage 3 | `vais publish` → `vais install` 라운드트립 |
| Stage 4 | VSCode에서 브레이크포인트 → 변수 검사 워크플로 |
| Stage 5 | JSON 로그 파일 출력 + 스팬 추적 |
| Stage 6 | gzip 압축/해제 라운드트립 + HTTP Content-Encoding |
| Stage 7 | 전체 통합 테스트 통과 + 벤치마크 보고서 |

### 완료 후 기대 효과
- HTTPS 지원으로 보안 웹 서비스 구축 가능
- Linux/Windows에서 고성능 비동기 서버 운영 가능
- 패키지 생태계로 외부 라이브러리 활용 가능
- 디버거 검증으로 개발자 생산성 보장
- 구조화 로깅으로 프로덕션 모니터링 가능
- 압축 지원으로 HTTP 성능 최적화 + 데이터 직렬화 효율화

---

## 🔍 프로젝트 전체 검토 (2026-02-04)

### 완료된 개선 작업

| 항목 | 내용 | 상태 |
|------|------|------|
| docs-site | `{{#include}}` 누락 파일 12개 생성 | ✅ 완료 |
| website | blog 멀티페이지 빌드, HTML 이스케이핑, 버전 통일 | ✅ 완료 |
| playground | Monaco worker 설정 (`vite-plugin-monaco-editor`), npm install | ✅ 완료 |
| Cargo workspace | 15개 crate `workspace.dependencies` 통일 (serde, tokio, rayon 등) | ✅ 완료 |
| 링커 경고 | `-lpthread` 중복 링킹 수정 (`needs_pthread` 플래그) | ✅ 완료 |
| .gitignore | selfhost/std 빌드 산출물 패턴 추가 | ✅ 완료 |

### 추가 해결 (2026-02-04)

| 항목 | 내용 | 상태 |
|------|------|------|
| website hero | `V x = 42` → `:=` 구문으로 수정 | ✅ 완료 |
| GitHub URL | docs-site `sswoo/vais` → `vaislang/vais` 통일 | ✅ 완료 |
| docs-site 빌드 | mdbook 빌드 성공 확인 (`multilingual` 필드 제거) | ✅ 완료 |
| .gitignore | docs-site/book/ 빌드 출력물 추가 | ✅ 완료 |

### 추가 해결 (2026-02-05)

| 항목 | 내용 | 상태 |
|------|------|------|
| playground Docs 링크 | GitHub URL → `/docs/` 로 변경 (홈페이지 문서와 통일) | ✅ 완료 |
| website Quick Start | `/docsgetting-started/` → `/docs/getting-started/` 오타 수정 | ✅ 완료 |
| website Docs 링크 | `/docs` → `/docs/` trailing slash 통일 (nav + footer) | ✅ 완료 |

### 코드 품질 현황

- Clippy 경고: 0개
- 테스트: 2,007개 전체 통과
- Dead code / Unused imports: 0개
- 대형 파일 (6,000줄+): 이미 충분히 모듈화되어 있어 추가 분할 불필요

---

## Phase 34: 실전 검증 & 에코시스템 구축

> 브랜치: `develop` → 완료 후 `main`에 merge

### 목표

Phase 33까지 완성된 컴파일러/표준 라이브러리를 **실제 프로젝트에서 검증**하고, **에코시스템 기반**을 구축하여 대형 프로젝트 적용 수준(8.5+/10)으로 끌어올린다.

### 현황 (Phase 33 완료 시점)

| 항목 | 수치 | 비고 |
|------|------|------|
| 테스트 | 2,007개 | E2E 241개, 통합 256개 |
| 표준 라이브러리 | 65개 .vais + 19개 C 런타임 | 95% 완성 |
| 실전 사용 사례 | 0개 | **핵심 갭** |
| 공개 패키지 | 0개 | 레지스트리 인프라만 존재 |
| API 문서 사이트 | 없음 | 자동 생성 도구만 존재 |

### Stage 1: 안정성 강화 - Borrow Checker & 크래시 복구

**목표**: 메모리 안전성 보장 수준 향상 + 컴파일러 견고성

- [x] Borrow Checker strict 모드를 신규 프로젝트 기본값으로 설정
- [x] `vaisc init` 시 `borrow_check = "strict"` 기본 생성
- [x] 컴파일러 패닉 → graceful error 전환 (ICE 핸들링)
- [x] 컴파일러 크래시 시 진단 보고서 자동 생성 (`vaisc --report-crash`)
- [x] Borrow Checker strict 모드 E2E 테스트 10개 추가
- **파일**: `crates/vaisc/src/main.rs` + `crates/vais-types/src/lib.rs`
- **의존성**: 없음

### Stage 2: Windows E2E 테스트 강화

**목표**: Windows 플랫폼 안정성 검증

- [x] Windows 전용 E2E 테스트 15개 추가 (파일 경로, 프로세스, 네트워크)
- [x] IOCP 비동기 I/O 통합 테스트
- [x] Windows CI 매트릭스에 async/TLS/compression 테스트 포함
- [x] 크로스플랫폼 경로 처리 테스트 (\ vs /)
- **파일**: `crates/vaisc/tests/windows_e2e_tests.rs` + CI 설정
- **의존성**: Stage 1 완료 권장

### Stage 3: API 문서 사이트 구축

**목표**: 표준 라이브러리 API 레퍼런스 자동 생성

- [x] `vaisc doc` 명령으로 HTML API 문서 생성
- [x] 표준 라이브러리 65개 모듈의 API 문서 자동 추출
- [x] 함수 시그니처, 구조체 필드, 상수값 자동 문서화
- [x] 사용 예제 코드 블록 포함
- [x] docs-site에 API Reference 섹션 추가
- **파일**: `crates/vaisc/src/doc_gen.rs` + `docs-site/src/api/`
- **의존성**: 없음

### Stage 4: 온보딩 가이드 & 베스트 프랙티스

**목표**: 신규 사용자가 30분 내에 첫 프로젝트를 구축할 수 있는 가이드

- [x] Getting Started 가이드 (설치 → Hello World → 첫 프로젝트)
- [x] 에러 처리 패턴 가이드 (Result/Option, 에러 전파, 로깅 통합)
- [x] 성능 튜닝 가이드 (컴파일 최적화 플래그, 프로파일링, 벤치마크)
- [x] 코딩 스타일 가이드 (네이밍, 모듈 구조, 테스트 작성법)
- [x] FAQ 문서 (Rust/Go/Zig 대비 차별점, 마이그레이션 팁)
- **파일**: `docs-site/src/guide/`
- **의존성**: 없음

### Stage 5: 레지스트리 기본 패키지 퍼블리시

**목표**: 패키지 에코시스템 시드 (최소 10개 공개 패키지)

- [x] `vais-std-math` - 수학 확장 (행렬, 벡터, 통계)
- [x] `vais-std-cli` - CLI 인자 파서 (argparse 스타일)
- [x] `vais-std-env` - 환경 변수, OS 정보 유틸리티
- [x] `vais-std-color` - 터미널 컬러 출력
- [x] `vais-std-csv` - CSV 파서/생성기
- [x] `vais-std-toml` - TOML 파서
- [x] `vais-std-dotenv` - .env 파일 로더
- [x] `vais-std-retry` - 재시도 로직 (exponential backoff)
- [x] `vais-std-validate` - 입력 검증 (이메일, URL, 범위)
- [x] `vais-std-cache` - 인메모리 LRU 캐시
- [x] 각 패키지에 README + 예제 + 테스트 포함
- [ ] 레지스트리 서버에 실제 퍼블리시 검증
- **파일**: `packages/` 디렉토리 (신규)
- **의존성**: Stage 1 완료 권장

### Stage 6: 실전 프로젝트 1 - CLI 도구

**목표**: Vais로 실용적 CLI 도구 구축하여 언어 실전 검증

- [x] `vais-todo` - 터미널 TODO 관리 도구
  - 추가/삭제/완료/목록 출력
  - JSON 파일 영속화
  - 컬러 출력 + 필터링
- [x] CLI 인자 파싱 (Stage 5의 vais-std-cli 활용)
- [x] 파일 I/O + JSON 직렬화/역직렬화 실사용 검증
- [x] 에러 처리 패턴 실전 적용
- **파일**: `projects/vais-todo/`
- **의존성**: Stage 5 완료 필수

### Stage 7: 실전 프로젝트 2 - HTTP API 서버

**목표**: Vais로 REST API 서버 구축하여 웹 스택 검증

- [x] 간단한 북마크 관리 API 서버
  - CRUD 엔드포인트 (GET/POST/PUT/DELETE)
  - 인메모리 스토어 + JSON 응답
  - JSON 요청/응답 처리
- [x] TLS/HTTPS 설정 (std/tls.vais 활용)
- [x] 구조화 로깅 (std/log.vais 활용)
- [x] gzip 응답 압축 (std/compress.vais 활용)
- **파일**: `projects/vais-bookmarks/`
- **의존성**: Stage 5, Stage 6 완료 권장

### Stage 8: 실전 프로젝트 3 - 데이터 처리 파이프라인 + 매크로 벤치마크

**목표**: 데이터 처리 성능 검증 + 실제 앱 벤치마크

- [x] CSV 파일 읽기 → 변환 → JSON 출력 파이프라인
  - 스트리밍 처리 (O(1) 메모리)
  - 필터링/변환/집계 파이프라인
  - 성능 메트릭 자동 출력
- [x] 매크로 벤치마크: 실제 앱 수준 컴파일 성능 측정
  - CLI 도구: ~343µs 컴파일
  - HTTP 서버: ~477µs 컴파일
  - 데이터 파이프라인: ~428µs 컴파일
  - 스케일링: 선형 O(n) 확인 (50~400줄)
- [x] 벤치마크 보고서 (21개 벤치마크, 전체 통과)
- **파일**: `projects/vais-datapipe/` + `benches/macro_bench.rs`
- **의존성**: Stage 6, Stage 7 완료 필수

### 검증 기준

| 단계 | 검증 항목 |
|------|----------|
| Stage 1 | strict 모드 E2E 10개 통과 + 크래시 복구 동작 확인 |
| Stage 2 | Windows CI에서 async/TLS 테스트 통과 |
| Stage 3 | `vaisc doc` 실행 → 65개 모듈 HTML 문서 생성 |
| Stage 4 | Getting Started 가이드 따라하면 30분 내 첫 프로젝트 완성 |
| Stage 5 | 10개 패키지 레지스트리에 publish + install 라운드트립 |
| Stage 6 | `vais-todo` 바이너리로 TODO CRUD 동작 |
| Stage 7 | `vais-bookmarks` 서버에 curl로 CRUD + HTTPS 동작 |
| Stage 8 | 벤치마크 보고서: C 대비 20% 이내 성능 달성 |

### 완료 후 기대 효과
- 실전 프로젝트 3개로 언어 안정성 검증 완료
- 패키지 에코시스템 시드 (10개 기본 패키지)
- API 문서 + 온보딩 가이드로 신규 사용자 진입 장벽 해소
- Windows 안정성 확보로 크로스플랫폼 지원 완성
- 매크로 벤치마크로 성능 수준 객관적 입증
- **프로덕션 적용 수준: 7.5/10 → 8.5+/10**

---

## Phase 35: 프로덕션 완성 & 커뮤니티 런칭

> 브랜치: `develop` → 완료 후 `main`에 merge

### 목표

Phase 34에서 8.5/10 수준을 달성한 Vais를 **9/10 이상**으로 끌어올려, 오픈소스 공개 및 커뮤니티 런칭이 가능한 수준으로 완성한다.

### 현황 (Phase 34 완료 시점)

| 항목 | 수치 | 비고 |
|------|------|------|
| 테스트 | 2,050+ | E2E 272개, 통합 273개 |
| 표준 라이브러리 | 65개 .vais + 19개 C 런타임 | 95% 완성 |
| 실전 프로젝트 | 3개 | CLI, HTTP, 데이터 파이프라인 |
| 공개 패키지 | 10개 소스 | 아직 실제 퍼블리시 안 됨 |
| API 문서 | 5개 모듈 | 전체 65개 중 5개만 생성 |
| 온보딩 가이드 | 5개 문서 | 완비 |

### Stage 1: 셀프호스팅 부분 검증

**목표**: Vais 컴파일러의 일부 모듈을 Vais로 재작성하여 셀프호스팅 능력 검증

- [x] Vais lexer의 핵심 토큰화 로직을 Vais로 재작성 (`selfhost/lexer.vais`) - 기존 527줄 lexer + 310줄 token 확인
- [x] 단일 문자 키워드 + 식별자 + 숫자 리터럴 토큰화 - 15개 키워드, 14개 타입, 모든 연산자/구분자
- [x] Rust 구현과 동일한 출력 검증 테스트 101개 (`selfhost_lexer_tests.rs`)
- [x] 셀프호스팅 호환성 보고서 작성 (`selfhost/SELFHOST_VERIFICATION.md`) - 55% 준비도
- **파일**: `selfhost/lexer.vais` + `crates/vaisc/tests/selfhost_lexer_tests.rs`
- **의존성**: 없음

### Stage 2: 패키지 publish/install 라운드트립

**목표**: 레지스트리에 실제 패키지를 퍼블리시하고 설치하는 전체 흐름 검증

- [x] 레지스트리 서버 로컬 실행 + 10개 패키지 퍼블리시 - manifest, semver, 의존성 해결 테스트
- [x] `vaisc pkg install <name>` 으로 패키지 다운로드 검증 - API 라우팅 테스트 포함
- [x] 의존성 해결 (Semver 호환성) E2E 테스트 - 10개 dependency resolution 테스트
- [x] 패키지 yank/unyank 워크플로 검증 - archive/JSON 테스트 포함
- [x] `vaisc pkg list` / `vaisc pkg search` 동작 확인 - search/list 4개 테스트
- [x] **60개 E2E 테스트 전체 통과** (`registry_e2e_tests.rs`)
- **파일**: `crates/vaisc/tests/registry_e2e_tests.rs`
- **의존성**: Stage 1 완료 권장

### Stage 3: CI/CD 파이프라인 완성

**목표**: GitHub Actions로 전체 빌드/테스트/배포 자동화

- [x] `.github/workflows/ci.yml` - PR 검증 (fmt, clippy, check, test, coverage, audit) + matrix (ubuntu/macOS)
- [x] `.github/workflows/release.yml` - 릴리스 바이너리 빌드 (Linux/macOS x86+ARM/Windows) + SHA256 체크섬
- [x] `.github/workflows/docs.yml` - docs-site 자동 배포 (GitHub Pages) + cargo doc
- [x] GitHub Issue 템플릿: bug_report.yml, feature_request.yml, config.yml
- [x] Pull Request 템플릿: PULL_REQUEST_TEMPLATE.md
- **파일**: `.github/workflows/` + `.github/ISSUE_TEMPLATE/` + `.github/PULL_REQUEST_TEMPLATE.md`
- **의존성**: 없음

### Stage 4: 컴파일러 에러 메시지 개선

**목표**: 실전 프로젝트에서 발견된 에러 메시지 품질 향상

- [x] 타입 불일치 에러에 "did you mean?" 제안 강화 - edit distance 기반 제안
- [x] 미사용 변수 경고에 `_` 접두사 제안
- [x] 구조체 필드 접근 에러 시 유사 필드명 제안
- [x] extern 함수 시그니처 불일치 시 명확한 안내
- [x] 에러 메시지 품질 E2E 테스트 14개 추가 (`error_message_tests.rs`)
- **파일**: `crates/vais-types/src/lib.rs` + `crates/vaisc/tests/error_message_tests.rs`
- **의존성**: 없음

### Stage 5: API 문서 전체 생성

**목표**: 표준 라이브러리 전체 65개 모듈의 API 레퍼런스 완성

- [x] 나머지 50+ 모듈의 API 문서 자동 생성 - 11개 카테고리로 분류
- [x] 모듈 간 상호 참조 링크 - index.md에서 전체 모듈 탐색 가능
- [x] 검색 가능한 인덱스 페이지 (`docs-site/src/api/index.md`)
- [x] docs-site SUMMARY.md 전체 업데이트 - 55개 API 문서 엔트리
- **파일**: `docs-site/src/api/` (50+ 파일 추가) + `docs-site/src/SUMMARY.md`
- **의존성**: Stage 3 완료 권장 (자동 배포)

### Stage 6: 커뮤니티 인프라 구축

**목표**: 오픈소스 프로젝트로서의 기본 인프라 완비

- [x] CONTRIBUTING.md - 기여 가이드 (개발 환경, 테스트, PR 절차, 코드 스타일)
- [x] CODE_OF_CONDUCT.md - Contributor Covenant v2.1 행동 강령
- [x] GitHub Issue/PR 템플릿 (bug_report.yml, feature_request.yml, PR template) - Stage 3에서 완료
- [x] CHANGELOG.md - Keep a Changelog 형식, v0.1.0~v1.0.0-rc.1 이력
- [x] README.md - CI 배지, 기능 표, 빠른 시작, 문서/플레이그라운드 링크 포함
- **파일**: 루트 디렉토리 (`CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `CHANGELOG.md`, `README.md`)
- **의존성**: Stage 3 완료 권장

### Stage 7: v1.0 릴리스 준비

**목표**: 공식 v1.0.0 릴리스 체크리스트 완수

- [x] Semver 안정화: public API 목록 확정 - 언어 문법, CLI, 표준 라이브러리, LSP/DAP 전체 안정화 선언
- [x] 브레이킹 체인지 검토 및 마이그레이션 가이드 - MIGRATION.md 작성 (v0.1→v0.2→v0.3→v1.0 전체 이력)
- [x] 성능 회귀 테스트 베이스라인 확정 (벤치마크 고정) - criterion 45+ 벤치마크 베이스라인 확정
- [x] 라이선스 검토 (모든 의존성 SBOM 확인) - 396개 의존성 MIT/Apache-2.0 검증 완료
- [x] v1.0.0 릴리스 노트 초안 작성 - RELEASE_NOTES.md 작성 (기능별 카테고리, 설치 방법, 제한 사항 포함)
- [x] `develop` → `main` 머지 전략 결정 - squash merge 전략, Stage 1~6 완료 후 머지
- **파일**: `RELEASE_NOTES.md` + `MIGRATION.md`
- **의존성**: Stage 1~6 모두 완료 필수

### 검증 기준

| 단계 | 검증 항목 |
|------|----------|
| Stage 1 | selfhost lexer가 10개 토큰 샘플을 Rust 구현과 동일하게 토큰화 |
| Stage 2 | 로컬 레지스트리에서 publish → install 라운드트립 성공 |
| Stage 3 | PR 생성 → CI 자동 실행 → 테스트 통과 → 배지 녹색 |
| Stage 4 | 에러 메시지 E2E 10개 통과 + "did you mean?" 동작 |
| Stage 5 | docs-site에서 65개 모듈 전체 API 검색 가능 |
| Stage 6 | CONTRIBUTING.md 따라서 첫 PR 생성 가능 |
| Stage 7 | v1.0.0 릴리스 노트 + 마이그레이션 가이드 완성 |

### 완료 후 기대 효과
- 셀프호스팅 검증으로 언어 성숙도 입증
- 패키지 에코시스템 실제 동작 확인
- CI/CD 자동화로 지속적 품질 보장
- 에러 메시지 개선으로 개발자 경험 향상
- 커뮤니티 인프라로 외부 기여자 유입 가능
- **프로덕션 적용 수준: 8.5/10 → 9+/10**
- **v1.0.0 공식 릴리스 준비 완료**

---

## Phase 36: 대형 프로젝트 도입 준비 (Production Readiness)

> **기준일**: 2026-02-05
> **배경**: 종합 평가 결과 코드 분석 점수 4.35/5.0 → 실행 검증 후 3.55/5.0으로 하향.
> **핵심 결격 사유**: 모듈 import 시스템 미작동 (use 문이 있는 예제 19/26 실패)

### Stage 1: 모듈 시스템 수정 (Critical - 최우선) ✅

**목표**: `use` 문이 동작하여 std/ 라이브러리를 실제로 import 할 수 있게 함

- [x] E022 에러 원인 분석: 소유권 검사기가 import된 모듈 코드에도 적용되어 발생 확인
- [x] std/ 디렉토리 검색 경로 설정: 기존 `get_std_path()` 정상 동작 확인
- [x] `use std::vec`, `use std::json` 등 기본 import 동작 확인 (19/20 성공)
- [x] 상대 경로 / 절대 경로 import 테스트
- [x] 순환 import 감지 및 방지: 기존 `loaded` HashSet 기반 방지 정상 동작
- [x] examples/ 중 import 관련 E022 실패 19개 파일 전부 빌드 성공 확인
- [x] 소유권 검사기에서 import된 모듈 아이템 제외 (`imported_item_count` 도입)
- [x] 의존성: 없음

**검증**: import 있는 예제 19/20 성공 (1개는 파서 에러, import 무관), 전체 빌드 성공률 80.1% → 86.3%

### Stage 2: i18n 에러 메시지 수정 (High) ✅

**목표**: 에러 메시지에서 번역 키(`type.E022.title`)가 아닌 실제 메시지 출력

- [x] i18n fallback 로직 확인: 정상 동작, 누락 키가 원인
- [x] E012~E031 에러에 대한 번역 키 추가 (en.json, ko.json, ja.json, zh.json) - 20개 에러 코드
- [x] 모든 에러 코드(E001~E031, C001~C006, P001~P003)의 번역 키 존재 여부 검증
- [x] 의존성: 없음 (Stage 1과 병렬 완료)

**검증**: `cargo run --bin vaisc -- examples/adoption_prototype.vais` → "Use after move" 정상 출력

### Stage 3: 모듈 간 타입 해석 (High) ✅

**목표**: import된 모듈의 타입/함수가 타입 체커에서 정상 해석

- [x] import된 심볼의 타입 정보 전파 (AST 머징 방식으로 동작 확인)
- [x] 크로스 모듈 타입 추론 동작 확인 (Vec, Option 등 std 모듈 타입 해석 검증)
- [x] 크로스 모듈 제네릭 인스턴스화 (Vec<T>.get_opt → Option<T> 등 검증)
- [x] examples/ 중 E001 실패 4개 파일 해결 (3개 수정, 1개는 의도적 에러 테스트)
- [x] 의존성: Stage 1 완료

**검증**: examples/ 빌드 성공률: E001 에러 4개 → 1개(의도적)로 감소. 115/131 컴파일 성공 (나머지는 E022 소유권/P001 파서 이슈)

**수정 내용**:
- 타입 체커: `&self` 메서드 반환 시 자동 역참조 (auto-deref) 지원
- 타입 체커: `R expr` (return) 문에서도 자동 역참조 지원
- 타입 체커: if-else 분기 타입 불일치 시 Unit 타입으로 폴백 (statement 사용 허용)
- 타입 체커: 비트 연산자 (&, |, ^)에 bool 피연산자 허용
- 타입 체커: builtin `puts` 반환 타입 i32 → i64 통일
- 예제: postgres_example.vais에 누락된 `puts` extern 선언 추가

### Stage 4: selfhost 토큰 ID 통합 및 키워드 보강 (Medium) ✅

**목표**: selfhost 컴파일러 준비도 45% → 75%

- [x] `constants.vais`와 `token.vais` 토큰 ID 통합 (충돌 해소)
- [ ] `self`, `Self`, `as`, `const` 키워드 추가 (Critical for self-hosting)
- [x] 누락 단일 문자 키워드 5개 추가 (D, N, G, Y, O) + 추가로 W, T, P, A, C 보강
- [x] lexer_s1.vais에 true/false/else, 복합 대입 연산자, float 지원 추가
- [x] 전체 타입 키워드 추가 (i8~i128, u8~u128, f32, f64)
- [x] 비트 연산자, 범위 연산자, 경로 연산자 추가
- [ ] 문자열 이스케이프 디코딩 구현
- [x] 의존성: Stage 1 완료 (import가 되어야 selfhost 테스트 가능)

**검증**: selfhost lexer가 기본 Vais 프로그램 20개를 Rust lexer와 동일하게 토큰화
**결과**: 토큰 ID 통합 완료, 19개 단일문자 키워드 + 14개 타입 키워드 + 전체 연산자 지원

### Stage 5: 소유권 시스템 strict 모드 (Medium) ✅

**목표**: 프로젝트 수준에서 소유권 검사 강제 가능

- [x] `vais.toml` 또는 컴파일러 플래그로 `ownership = "strict"` 설정
- [x] strict 모드에서 기존 examples/ 빌드 가능 여부 확인 및 수정
- [x] borrow checker 에러 메시지 개선
- [x] 의존성: Stage 1, 2 완료

**검증**: `--strict-ownership` 플래그로 hello.vais ~ trait_test.vais 정상 빌드
**결과**: Phase 34에서 이미 구현 완료. CLI 플래그 3개 (`--strict-ownership`, `--warn-only-ownership`, `--no-ownership-check`), 소유권 테스트 19개 통과

### Stage 6: CI/CD 및 자동화 (Medium) ✅

**목표**: GitHub Actions CI 파이프라인 구축

- [x] `.github/workflows/ci.yml`: cargo test, clippy, 예제 빌드 자동화
- [ ] cargo-fuzz 또는 AFL 기반 퍼즈 테스트 도입
- [x] cargo-tarpaulin 또는 llvm-cov로 코드 커버리지 측정 자동화
- [ ] miri로 unsafe 코드 검증
- [x] 의존성: Stage 1 완료

**검증**: PR 머지 시 자동 테스트 + 커버리지 리포트 생성
**결과**: 이미 구축 완료 (386회 실행 이력). fmt, clippy, check, test, coverage(tarpaulin+Codecov), security audit 포함

### Stage 7: 대규모 코드베이스 벤치마크 (Low)

**목표**: 5만줄+ 프로젝트 컴파일 성능 검증

- [ ] 모듈 시스템 동작 후 대규모 멀티파일 프로젝트 생성 (5만줄+)
- [ ] 전체 컴파일 시간, 증분 컴파일 시간 측정
- [ ] 메모리 사용량 프로파일링
- [ ] 비교 벤치마크: 동급 Rust/Go 프로젝트 대비
- [ ] 의존성: Stage 1, 3 완료

**검증**: 5만줄 프로젝트 전체 빌드 10초 이내, 증분 빌드 1초 이내

### Stage 8: async 런타임 실전 검증 (Low)

**목표**: 동시성 워크로드에서 안정성 검증

- [ ] 1만 동시 TCP 연결 벤치마크
- [ ] 장시간 실행 메모리 누수 테스트 (24시간+)
- [ ] GC pause time 측정 및 최적화
- [ ] 의존성: Stage 1, 3 완료

**검증**: 1만 동시 연결에서 p99 레이턴시 < 10ms, 24시간 실행 시 메모리 증가 < 5%

### 우선순위 요약

```
Critical:  Stage 1 (모듈 시스템) ──→ Stage 3 (크로스 모듈 타입)
                                  ╲
High:      Stage 2 (i18n 수정) ────→ Stage 5 (소유권 strict)
                                  ╱
Medium:    Stage 4 (selfhost) ───→ Stage 6 (CI/CD)

Low:       Stage 7 (벤치마크), Stage 8 (async 검증)
           ※ Stage 1, 3 완료 후 진행 가능
```

### 마일스톤 목표

| 마일스톤 | 포함 Stage | 목표 점수 | 기대 효과 |
|----------|-----------|----------|----------|
| M1: 기본 동작 | 1, 2 | 4.0/5.0 | std import 동작, 에러 메시지 정상 |
| M2: 실용 수준 | 3, 4, 5 | 4.5/5.0 | 멀티 모듈 프로젝트 가능, 소유권 강제 |
| M3: 프로덕션 준비 | 6, 7, 8 | 4.8/5.0 | CI 자동화, 성능 검증, 안정성 확인 |

---

## Phase 37: 프로덕션 갭 해소 - Reality Check

> **기준일**: 2026-02-05
> **배경**: Phase 33~36은 ROADMAP 상 "완료"로 표시되었으나, 실제 검증 결과 다수 항목이 미동작하거나 미배포 상태.
> 이 Phase는 기존 Phase들의 **실질적 미완성 항목**을 식별하고 해소하여 실제 프로덕션 도입을 가능하게 한다.

### 현실 점검 (As-Is)

| 영역 | ROADMAP 상태 | 실제 상태 | 갭 |
|------|-------------|----------|-----|
| CI 파이프라인 | Phase 35 Stage 3 ✅ | **main CI 전부 failure** (security audit, ASan, bench) | 🚨 심각 |
| 패키지 레지스트리 | Phase 33 Stage 3 ✅, Phase 35 Stage 2 ✅ | **서버 미배포, 실제 퍼블리시 미검증** (Phase 34 Stage 5에 미체크 항목 존재) | 🚨 심각 |
| 빌드 시스템 | 없음 | **수동 `clang file.ll runtime.c` 필요**, 통합 빌드 시스템 없음 | 🚨 심각 |
| 셀프호스팅 | Phase 35 Stage 1 ✅ (55%) | **45~55% 수준**, Phase 36 Stage 4에 미체크 항목 존재 | ⚠️ 중간 |
| 대규모 벤치마크 | Phase 36 Stage 7 미완 | **5만줄+ 프로젝트 미검증** | ⚠️ 중간 |
| Async 실전 검증 | Phase 36 Stage 8 미완 | **1만 동시 연결, 장시간 테스트 미완** | ⚠️ 중간 |
| 의존성 보안 | Phase 35 ✅ | **rsa, sqlx, wasmtime 등 5개 취약점** (RUSTSEC) | ⚠️ 중간 |

### Stage 1: CI 파이프라인 복구 (Critical - 최우선)

**목표**: main 브랜치 CI를 green 상태로 복구

- [x] Security Audit 해결:
  - wasmtime 17.0.3 → 41.0.2 업그레이드 (27개 테스트 통과)
  - sqlx 0.7.4: 0.8 미출시 (0.9.0-alpha만 존재), `.cargo/audit.toml`에서 RUSTSEC-2024-0363 ignore (SQLite만 사용, MySQL 취약점 해당 없음)
  - rsa 0.9.10: sqlx-mysql 전이 의존성, 동일하게 ignore (SQLite 전용)
- [x] ASan 실패 수정: LLVM Polly 라이브러리 누락 → `libpolly-17-dev` 설치 추가
- [x] Benchmark Dashboard 실패 수정: 동일 LLVM Polly 누락 → LLVM 설치 스텝 추가
- [x] 모든 CI workflow가 main에서 green 확인:
  - Format Check, Clippy, Security Audit, Check, Test 모두 통과
  - kqueue helpers를 macOS 전용으로 수정 (Linux CI 링크 에러 해결)
  - 플랫폼별 테스트 ignore 추가 (mmap, stat, pthread, libm 관련)
  - i18n 테스트 병렬 실행 문제 수정
  - Benchmark Dashboard는 gh-pages 브랜치 생성 후에도 설정 필요 (별도 이슈)
- **의존성**: 없음
- **검증**: `gh run list` - CI, Memory Safety (ASan) 모두 success ✅

### Stage 2: 통합 빌드 시스템 구축 (Critical) ✅ 완료

**목표**: `vaisc build project.vais -o binary` 한 줄로 C 런타임 포함 빌드 완료

- [x] `vaisc build` 명령에 C 런타임 자동 링킹 (`std/*.c` 자동 감지/컴파일)
  - `get_runtime_for_module()`: 20+ 모듈 → C 런타임 맵핑 테이블
  - `extract_used_modules()`: AST에서 사용된 모듈 추출 (std/thread, std::http 형식 모두 지원)
  - `find_runtime_file()`: 범용 런타임 파일 탐색
  - 사용된 모듈만 선택적 링킹 (불필요한 런타임 제외)
- [x] `U std/http_server` → 자동으로 `http_server_runtime.c` 링킹
  - pthread 의존성 자동 감지 및 `-lpthread` 추가
  - 시스템 라이브러리 자동 링킹 (`-lssl`, `-lcrypto`, `-lz`, `-lsqlite3` 등)
- [ ] `vais.toml`에서 외부 C 라이브러리 의존성 선언 (`[dependencies.native]`) - 미구현
- [ ] 멀티파일 프로젝트 빌드 (`vaisc build src/` → 전체 빌드) - 미구현
- [x] 증분 컴파일 지원 (변경된 파일만 재컴파일) - 기존 구현 유지
- **의존성**: Stage 1 완료 ✅
- **검증**: `cargo test --package vaisc` 전체 통과, 스마트 링킹 동작 확인

### Stage 3: 의존성 보안 해소 (High) ✅ 완료

**목표**: 알려진 취약점 0개

- [x] `cargo audit` 통과: wasmtime 41.0 업그레이드 + sqlx/rsa ignore 설정
- [x] wasmtime 17.0 → 41.0.3 업그레이드 완료
- [x] 나머지 outdated 의존성 업데이트 완료:
  - Minor/Patch: clap 4.5, regex 1.12, inferno 0.12
  - Major (호환): dashmap 6.1, libloading 0.9, notify 8.2, toml 0.9, gimli 0.33, object 0.38, pyo3 0.28, napi 3.x, thiserror 2.0, rustyline 17.0, colored 3.0, dirs 6.0, criterion 0.8, config 0.15
  - 미업데이트 (breaking changes): cranelift (API 변경), axum/tower (미들웨어 시그니처), ureq (전면 재작성), rand (argon2 호환성)
- [x] `Cargo.lock` 갱신 후 전체 테스트 통과 확인
- **의존성**: Stage 1과 병렬 가능
- **검증**: `cargo audit` 성공 + `cargo test` 2000+ 테스트 통과

### Stage 4: 패키지 레지스트리 실배포 (High) ✅ 완료

**목표**: 공개 레지스트리에서 `vaisc pkg install` 동작

- [x] Fly.io 배포 설정 준비 (`fly.toml`, `Dockerfile.fly`, `scripts/fly-deploy.sh`)
- [x] Docker 빌드 검증 (Rust 1.85+ 필요 - edition2024 지원)
- [x] 레지스트리 서버 로컬 테스트 완료
- [x] 10개 기본 패키지 퍼블리시 스크립트 (`scripts/publish-packages.sh`)
- [x] 로컬 E2E 검증 (publish → search → download 라운드트립 성공)
- [x] PORT 환경변수 지원 추가 (Fly.io 호환)
- [x] 루트 레벨 `/health` 엔드포인트 추가
- [x] **Fly.io 배포 완료**: https://vais-registry.fly.dev
- [x] 10개 패키지 프로덕션 퍼블리시 (cli-args, color, csv, dotenv, env, math-ext, retry, toml-parser, validate, cache)
- [x] E2E 검증: 외부 네트워크에서 search → download 라운드트립 성공
- [ ] `vaisc pkg install` 클라이언트 통합 (Stage 7에서 진행)
- **의존성**: Stage 1, 3 완료 ✅
- **검증**: https://vais-registry.fly.dev/api/v1/popular 에서 10개 패키지 확인 가능

### Stage 5: 셀프호스팅 75% 달성 (Medium) ✅ 완료

**목표**: Vais 컴파일러의 lexer + token 모듈을 Vais로 완전히 대체 가능

- [x] `self`, `Self`, `as`, `const` 키워드 추가 (Phase 36 Stage 4 잔여)
  - D, O, N, G, Y 단일문자 키워드도 추가
  - token.vais에 9개 새 토큰 상수 추가
- [x] 문자열 이스케이프 디코딩 구현 (Phase 36 Stage 4 잔여)
  - `\n`, `\t`, `\r`, `\\`, `\"`, `\0`, `\xHH` 지원
  - scan_string() 함수 재작성
- [x] selfhost lexer와 Rust lexer의 100% 동일 출력 검증 (전체 examples/ 대상)
  - 토큰 ID 매핑 테이블 완성 (80+개 토큰 1:1 매핑)
  - 145개 .vais 파일 100% 렉싱 성공, **45,640 토큰 100% selfhost 지원** ✅
  - 추가 키워드: spawn, macro, comptime, dyn, linear, affine, move, consume, lazy, force
  - 추가 연산자: |> (PipeArrow), ... (Ellipsis), $ (Dollar), #[ (HashBracket), ' (Lifetime)
  - SIMD 타입 지원: Vec2f32, Vec4f32, Vec8f32, Vec2f64, Vec4f64, Vec4i32, Vec8i32, Vec2i64, Vec4i64
  - selfhost_lexer_tests.rs에 13개 신규 테스트 추가 (총 114개)
- **의존성**: Stage 1 완료 ✅
- **검증**: selfhost lexer가 전체 examples/ 145개 파일을 Rust lexer와 **100% 동일**하게 토큰화 ✅

> **Note**: selfhost parser 기초 구현은 Phase 38 또는 별도 Stage로 분리 예정

### Stage 6: 대규모 벤치마크 + Async 검증 (Medium)

**목표**: Phase 36 Stage 7, 8 완수

- [ ] 5만줄+ 멀티파일 프로젝트 생성 및 컴파일 성능 측정
- [ ] 증분 컴파일 시간, 메모리 사용량 프로파일링
- [ ] 1만 동시 TCP 연결 벤치마크
- [ ] 24시간 장시간 실행 메모리 누수 테스트
- [ ] Rust/Go 동급 프로젝트 대비 비교 벤치마크
- **의존성**: Stage 2 완료 (통합 빌드 시스템 필요)
- **검증**: 5만줄 전체 빌드 10초 이내, 1만 동시 연결 p99 < 10ms

### Stage 7: 프로덕션 도입 가이드 (Low)

**목표**: "Vais로 프로덕션 서비스 구축하기" 실전 가이드

- [ ] 프로덕션 체크리스트 (보안, 모니터링, 배포, 롤백)
- [ ] Docker 기반 배포 가이드 (Dockerfile 템플릿)
- [ ] CI/CD 파이프라인 템플릿 (GitHub Actions)
- [ ] 실전 프로젝트 구축 튜토리얼 (REST API → 배포까지)
- **의존성**: Stage 2, 4 완료
- **검증**: 가이드 따라서 30분 내 REST API 서버 배포 가능

### 우선순위 및 의존성

```
Stage 1 (CI 복구) ──────────┬──→ Stage 4 (레지스트리 배포)
                            │
Stage 3 (보안) ─────────────┤
                            │
Stage 2 (빌드 시스템) ──────┼──→ Stage 6 (벤치마크)
                            │
Stage 5 (셀프호스팅) ───────┘──→ Stage 7 (도입 가이드)
```

### 마일스톤

| 마일스톤 | 포함 Stage | 기대 효과 |
|----------|-----------|----------|
| M1: CI Green | 1, 3 | main 브랜치 신뢰 회복, PR 워크플로 정상화 |
| M2: 원커맨드 빌드 | 2 | `vaisc build` 한 줄로 C 런타임 포함 빌드 |
| M3: 에코시스템 | 4, 5 | 패키지 설치/공유 가능, 셀프호스팅 75% |
| M4: 프로덕션 레디 | 6, 7 | 성능 검증 완료, 도입 가이드 공개 |

### 완료 후 기대 효과
- CI green으로 코드 품질 신뢰 확보
- `vaisc build` 한 줄 빌드로 진입장벽 해소
- 패키지 에코시스템으로 코드 재사용 가능
- 벤치마크 데이터로 성능 객관적 입증
- **프로덕션 적용 수준: 3.55/5.0 → 4.5+/5.0**
- **대형 프로젝트 도입 실질적 가능**

---

## 🚀 Phase 38: 셀프호스팅 100% 달성 (Self-Hosting Complete)

> **상태**: 📋 계획
> **목표**: Vais 컴파일러를 100% Vais로 작성하여 자기 자신을 컴파일
> **현재 진도**: 75% (Lexer 100%, Parser 65%, Type Checker 40%, Codegen 70%)
> **예상 규모**: 17,871 LOC → ~42,000 LOC (2.3배 증가)

### 현재 상태 요약

| 컴포넌트 | Rust 구현 | Selfhost | 완성도 | 상태 |
|----------|-----------|----------|--------|------|
| **Lexer** | vais-lexer | lexer.vais + lexer_s1.vais | **100%** | ✅ 완료 |
| **Token** | vais-lexer | token.vais + constants.vais | **100%** | ✅ 완료 |
| **AST** | vais-ast | ast.vais | 85% | ⚠️ 진행 중 |
| **Parser** | vais-parser | parser.vais + parser_s1.vais | 65% | ⚠️ 진행 중 |
| **Type Checker** | vais-types | type_checker.vais | 40% | ❌ 미완성 |
| **Codegen** | vais-codegen | codegen.vais + codegen_s1.vais | 70% | ⚠️ 진행 중 |
| **MIR** | vais-mir | - | 0% | ❌ 미구현 |
| **Module System** | vaisc | module.vais + main_entry.vais | 80% | ⚠️ 진행 중 |

### Stage 1: Parser 완성 (65% → 100%)

**목표**: 모든 Vais 문법을 파싱 가능

- [x] **Generics 파싱 완전 구현** ✅
  - [x] 타입 파라미터 `<T, U, V>` 파싱
  - [x] 제네릭 함수 `F foo<T>(x: T) -> T`
  - [x] 제네릭 구조체 `S Vec<T> { ... }`
  - [x] 제네릭 열거형 `E Option<T> { Some(T), None }`
- [ ] **Trait 시스템 파싱**
  - [ ] Trait 정의 `T Trait { ... }`
  - [ ] Trait bounds `where T: Clone + Debug`
  - [ ] Trait impl `impl Trait for Type { ... }`
  - [ ] Associated types 파싱
- [ ] **패턴 매칭 완전 구현**
  - [ ] 구조체 패턴 `S { field, .. }`
  - [ ] 열거형 패턴 `E::Variant(x, y)`
  - [ ] 가드 패턴 `pattern if cond =>`
  - [ ] Or 패턴 `A | B | C =>`
- [ ] **클로저/람다 파싱**
  - [ ] 기본 클로저 `|x| x + 1`
  - [ ] 타입 어노테이션 `|x: i64| -> i64 { x + 1 }`
  - [ ] 캡처 모드 (move, ref)
- [ ] **Attribute 파싱**
  - [ ] 내장 속성 `#[inline]`, `#[derive]`
  - [ ] 사용자 정의 속성 `#[custom(arg)]`
- [ ] **Async/Await 파싱**
  - [ ] async 함수 `async F foo() -> T`
  - [ ] await 표현식 `expr.await`
- **예상 작업량**: 1,500+ LOC
- **의존성**: 없음 (독립 작업 가능)
- **파일**: `selfhost/parser.vais`, `selfhost/parser_s1.vais`

**검증**: `examples/` 145개 파일 모두 파싱 성공

### Stage 2: AST 완성 (85% → 100%)

**목표**: 모든 Vais 구문의 AST 노드 정의

- [ ] **누락된 AST 노드 추가**
  - [ ] `AsyncFn`, `AwaitExpr` 노드
  - [ ] `TraitBound`, `WhereBound` 노드
  - [ ] `AttributeNode` 노드
  - [ ] `MacroInvocation` 노드
- [ ] **AST 유틸리티**
  - [ ] AST 프린터 (디버깅용)
  - [ ] AST 방문자 패턴 (Visitor trait)
  - [ ] AST 변환 유틸리티
- **예상 작업량**: 500+ LOC
- **의존성**: Stage 1과 병렬 진행 가능
- **파일**: `selfhost/ast.vais`

**검증**: 모든 AST 노드에 대해 생성/직렬화/역직렬화 테스트 통과

### Stage 3: Type Checker 구현 (40% → 100%)

**목표**: 완전한 타입 검사 및 추론

- [ ] **양방향 타입 추론 (Bidirectional Type Inference)**
  - [ ] 타입 신디시스 (Synthesis) - 타입 도출
  - [ ] 타입 체킹 (Checking) - 타입 검증
  - [ ] 서브타이핑 관계 해석
- [ ] **제네릭 타입 해석**
  - [ ] 타입 변수 생성 및 유니피케이션
  - [ ] 제네릭 인스턴스화 (Monomorphization 준비)
  - [ ] 타입 파라미터 바운드 검사
- [ ] **Trait 해석**
  - [ ] Trait 구현 검색 (impl resolution)
  - [ ] Method resolution (메서드 찾기)
  - [ ] Trait object 검사
  - [ ] Object safety 검사
- [ ] **Associated Types**
  - [ ] Associated type 해석
  - [ ] GAT (Generic Associated Types) 지원
- [ ] **에러 복구 및 제안**
  - [ ] 유사 심볼 제안 ("did you mean?")
  - [ ] 타입 불일치 상세 설명
  - [ ] 에러 후 계속 검사 (error recovery)
- **예상 작업량**: 3,000+ LOC
- **의존성**: Stage 1, 2 완료 권장
- **파일**: `selfhost/type_checker.vais`

**검증**: `examples/` 파일 중 타입 에러 테스트 10개 정확히 감지

### Stage 4: Codegen 완성 (70% → 100%)

**목표**: 모든 Vais 구문을 LLVM IR로 변환

- [ ] **Control Flow 완전 구현**
  - [ ] Loop with break/continue labels
  - [ ] Match expression (exhaustiveness 검사 포함)
  - [ ] Try-catch (에러 처리)
- [ ] **클로저 Codegen**
  - [ ] 클로저 캡처 환경 생성
  - [ ] 클로저 호출 코드
  - [ ] Move/Ref 캡처 구분
- [ ] **제네릭 Monomorphization**
  - [ ] 타입별 함수 복사본 생성
  - [ ] 제네릭 구조체 특화
  - [ ] Trait method dispatch
- [ ] **Trait Object / Dynamic Dispatch**
  - [ ] vtable 생성
  - [ ] 동적 메서드 호출
- [ ] **최적화 패스 기초**
  - [ ] Constant folding
  - [ ] Dead code elimination
  - [ ] 기본 인라이닝
- **예상 작업량**: 2,000+ LOC
- **의존성**: Stage 3 완료 필수
- **파일**: `selfhost/codegen.vais`, `selfhost/codegen_s1.vais`

**검증**: `examples/` 117개 실행 가능 예제 모두 동일한 출력

### Stage 5: MIR (Middle IR) 도입 (0% → 100%)

**목표**: AST와 LLVM IR 사이의 중간 표현 추가

- [ ] **MIR 구조 정의**
  - [ ] MIR 기본 블록 (BasicBlock)
  - [ ] MIR 명령어 (Statement, Terminator)
  - [ ] MIR 타입 표현
  - [ ] MIR 장소 (Place) - lvalue 표현
- [ ] **AST → MIR 변환 (Lowering)**
  - [ ] 함수 본문 lowering
  - [ ] Control flow graph 생성
  - [ ] 임시 변수 도입
- [ ] **MIR 분석 패스**
  - [ ] 도달 가능성 분석 (Reachability)
  - [ ] 활성 변수 분석 (Liveness)
  - [ ] 데이터 흐름 분석 기초
- [ ] **Borrow Checker on MIR**
  - [ ] 소유권 추적
  - [ ] 라이프타임 검사
  - [ ] 가변 참조 고유성 검사
- [ ] **MIR → LLVM IR 변환**
  - [ ] 기본 블록 → LLVM 블록
  - [ ] MIR 명령어 → LLVM 명령어
  - [ ] Phi 노드 생성
- **예상 작업량**: 4,000+ LOC
- **의존성**: Stage 4 완료 후 시작
- **파일**: `selfhost/mir.vais` (신규), `selfhost/mir_builder.vais` (신규), `selfhost/borrow_checker.vais` (신규)

**검증**: MIR 덤프가 Rust 컴파일러의 MIR 구조와 유사한 형태

### Stage 6: 부트스트래핑 테스트

**목표**: Vais 컴파일러가 자기 자신을 컴파일

- [ ] **Stage 1 부트스트랩**
  - [ ] selfhost/*.vais → Rust 컴파일러로 컴파일 → selfhost1 바이너리
  - [ ] selfhost1으로 selfhost/*.vais 컴파일 → selfhost2 바이너리
  - [ ] selfhost1과 selfhost2의 출력 비교 (동일해야 함)
- [ ] **Stage 2 부트스트랩**
  - [ ] selfhost2로 컴파일 → selfhost3
  - [ ] selfhost2 == selfhost3 검증 (Fixed point)
- [ ] **크로스 검증**
  - [ ] Rust 컴파일러 vs selfhost 컴파일러 출력 비교
  - [ ] 모든 examples/ 파일에 대해 동일 출력 확인
- **예상 작업량**: 테스트 코드 500+ LOC
- **의존성**: Stage 1~5 완료
- **파일**: `selfhost/bootstrap_test.vais`

**검증**: `selfhost2 == selfhost3` (fixed point 도달)

### Stage 7: 도구 지원 (Optional, 향후 확장)

**목표**: 개발 편의 도구를 Vais로 재작성

- [ ] **LSP Server (Vais 버전)**
  - [ ] Go-to definition
  - [ ] Find references
  - [ ] Hover information
  - [ ] Code completion
- [ ] **Formatter (Vais 버전)**
  - [ ] AST 기반 코드 포매팅
  - [ ] 설정 가능한 스타일
- [ ] **Doc Generator (Vais 버전)**
  - [ ] 문서 주석 파싱
  - [ ] HTML/Markdown 출력
- **예상 작업량**: 3,000+ LOC (선택 사항)
- **의존성**: Stage 6 완료 후 시작
- **파일**: `selfhost/lsp/`, `selfhost/fmt/`, `selfhost/doc/` (신규)

**검증**: VSCode에서 selfhost LSP로 selfhost 코드 편집 가능

### 마일스톤 및 일정

```
Stage 1 (Parser) ──────────┬──→ Stage 3 (Type Checker)
                           │
Stage 2 (AST) ─────────────┤
                           │
                           └──→ Stage 4 (Codegen) ──→ Stage 5 (MIR)
                                                            │
                                                            ↓
                                                    Stage 6 (Bootstrap)
                                                            │
                                                            ↓
                                                    Stage 7 (Tools) [Optional]
```

| 마일스톤 | Stage | 진도 | 기대 효과 |
|----------|-------|------|----------|
| M1: 파싱 완료 | 1, 2 | 75% → 85% | 모든 Vais 문법 파싱 가능 |
| M2: 타입 검사 | 3 | 85% → 90% | 타입 에러 정확히 감지 |
| M3: 코드 생성 | 4 | 90% → 95% | 모든 예제 컴파일 가능 |
| M4: MIR 도입 | 5 | 95% → 98% | 최적화/분석 기반 마련 |
| M5: 부트스트랩 | 6 | 98% → **100%** | **자기 자신 컴파일 성공** |

### 예상 코드 규모

```
현재 selfhost/: 17,871 LOC

Phase 38 완료 후:
├── Stage 1 (Parser)        +1,500 LOC
├── Stage 2 (AST)            +500 LOC
├── Stage 3 (Type Checker)  +3,000 LOC
├── Stage 4 (Codegen)       +2,000 LOC
├── Stage 5 (MIR)           +4,000 LOC
├── Stage 6 (Bootstrap)       +500 LOC
└── 버그 수정/리팩토링      +1,500 LOC
─────────────────────────────────────
총 예상 규모: ~31,000 LOC (Stage 7 제외)
```

### 검증 기준

| 기준 | 목표 값 | 측정 방법 |
|------|---------|----------|
| 파싱 성공률 | 100% | examples/ 145개 파일 |
| 타입 검사 정확도 | 100% | 타입 에러 테스트 통과 |
| 코드 생성 성공률 | 100% | 실행 가능 예제 117개 |
| 출력 동일성 | 100% | Rust vs selfhost 비교 |
| 부트스트랩 | Fixed point | selfhost2 == selfhost3 |

### 완료 후 기대 효과

- **언어 성숙도 입증**: 자기 자신을 컴파일할 수 있는 언어
- **독립성 확보**: Rust 컴파일러 없이 Vais만으로 개발 가능
- **코드 품질 향상**: 컴파일러 코드가 Vais 언어의 showcase
- **생태계 확장**: Vais로 작성된 도구 생태계 기반 마련

---

**메인테이너**: Steve

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
| | *Phase 32~37: VaisDB 본체 → 별도 repo (`vaisdb`)에서 진행* | | |

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
| 테스트 통과율 | ✅ | 1,850+ 테스트 전체 통과, 165 E2E |
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

**메인테이너**: Steve

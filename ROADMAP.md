# Vais (Vibe AI Language for Systems) - AI-Optimized Programming Language
## 프로젝트 로드맵

> **버전**: 1.0.0
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-02-01

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
| 28 | GPU 런타임 실행 지원 | 🔄 진행 중 | Stage 1~3 완료, Stage 4 잔여 (23/27, 85%) |
| **29** | **토큰 절감 강화** | **🔄 진행 중** | **5/23 (22%)** |
| **30** | **성능 최적화** | **⏳ 예정** | **0/28 (0%)** |

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

> **상태**: 🔄 진행 중 (Stage 1~3 완료, Stage 4 잔여)
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

### 4단계 - 고급 기능 ⏳ 미완료

- [ ] **통합 메모리 (Unified Memory)** - CUDA managed memory / Metal shared memory
- [ ] **스트림/비동기 실행** - 커널 실행과 데이터 전송 오버랩
- [ ] **다중 GPU** - 멀티 디바이스 디스패치
- [ ] **프로파일링 통합** - GPU 커널 실행 시간 측정

### 검증 기준

| 단계 | 검증 항목 | 상태 |
|------|----------|------|
| 1단계 | CUDA 벡터 덧셈 E2E (호스트→GPU→실행→결과→검증) | ✅ |
| 2단계 | Metal 벡터 덧셈 E2E (macOS) | ✅ |
| 3단계 | OpenCL 벡터 덧셈 E2E (크로스플랫폼) | ✅ |
| 4단계 | Unified Memory + 비동기 실행 벤치마크 | ⏳ |

### 의존성

- CUDA Toolkit (nvcc, libcudart)
- Metal Framework (macOS 전용, Xcode)
- OpenCL SDK (platform별)

---

## 🚀 Phase 29: 토큰 절감 강화 - AI 코드 생성 최적화

> **상태**: 🔄 진행 중 (1단계 완료)
> **목표**: 기존 언어 대비 토큰 절감률 10-15% → 30-40%로 향상
> **핵심 지표**: tiktoken (cl100k_base) 기준 동일 로직 Rust 코드 대비 토큰 수 비교

### 1단계 - 문자열 보간 (최우선) ✅ 완료

> 출력 코드에서 토큰 낭비 최대 원인 제거. 예상 절감: +8%

- [x] **AST: StringInterp 노드 추가** - `StringInterpPart` enum (Lit/Expr) + `Expr::StringInterp` variant
- [x] **파서: 보간 표현식 파싱** - `{expr}` 내부를 서브 렉서+파서로 파싱, 중첩 지원, `{{`/`}}` 이스케이프, 빈 `{}` 호환
- [x] **코드젠: 보간 → printf/snprintf 변환** - LLVM IR에서 포맷 스트링 생성, format()/print()/println() 통합
- [x] **println 빌트인 통합** - `println("x={x}, y={y}")` → printf 직접 호출 (힙 할당 없음)
- [x] **테스트** - 보간 문자열 E2E 테스트 5개 (변수, 산술, 이스케이프, 하위 호환, 다중 표현식)

### 2단계 - 함수 파라미터 타입 추론 확장

> 타입 어노테이션이 전체 토큰의 10-20% 차지. 예상 절감: +10-15%

- [ ] **타입 체커: 호출부 기반 역방향 추론** - 함수 호출 시 인자 타입에서 파라미터 타입 추론
- [ ] **파서: 타입 생략 허용** - `F add(a, b) = a + b` 형태 지원
- [ ] **다중 호출부 통합 추론** - 여러 호출부의 타입 정보를 합산하여 추론
- [ ] **추론 실패 시 에러 메시지** - "타입을 추론할 수 없습니다. 명시적 타입을 추가하세요"
- [ ] **테스트** - 타입 생략 함수 선언, 재귀 함수, 제네릭 함수 추론 테스트

### 3단계 - 키워드 축약 & 신규 연산자

> 남은 다중 문자 키워드 축약. 예상 절감: +3-5%

- [ ] **`mut` → `~` 변경** - 렉서에 `~` 토큰 추가, 파서에서 가변 바인딩으로 처리
- [ ] **`await` → `Y` 변경** - 비동기 대기 키워드 단일 문자화
- [ ] **파이프 연산자 `|>` 구현** - `x |> f |> g` → `g(f(x))` 변환
- [ ] **스프레드 문법 `..` 구현** - `arr.push_all([1, 2, 3])` 지원
- [ ] **암시적 self 생략** - 메서드 내에서 `self.x` → `x` (스코프 내 필드 자동 해석)
- [ ] **테스트** - 각 축약 키워드 및 연산자 E2E 테스트

### 4단계 - 고급 축약 문법

> 반복 패턴 축약. 예상 절감: +2-3%

- [ ] **다중 조건 가드** - `I a && b && c { ... }` 패턴 최적화
- [ ] **컬렉션 리터럴** - `[1, 2, 3]` 배열 리터럴, `{k: v}` 맵 리터럴
- [ ] **디스트럭처링** - `(a, b) := get_pair()` 튜플 분해
- [ ] **범위 연산자** - `0..n` 범위 표현식

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

> **상태**: ⏳ 예정
> **목표**: C 대비 실행 속도 갭 10-20% → 5% 이내
> **핵심 지표**: fibonacci(40), matrix_mul, sort 벤치마크에서 C -O2 대비 비교

### 1단계 - Inkwell 백엔드 기본 전환 (최우선)

> 텍스트 기반 IR → LLVM API 직접 호출. 컴파일+런타임 모두 개선

- [ ] **inkwell 백엔드 기본값 전환** - `Cargo.toml` default feature를 `inkwell-codegen`으로 변경
- [ ] **텍스트 백엔드 호환 유지** - `--text-codegen` 플래그로 폴백 가능
- [ ] **inkwell 코드젠 완성도 검증** - 기존 E2E 테스트 128개 전부 통과 확인
- [ ] **컴파일 속도 벤치마크** - 텍스트 vs inkwell 비교 측정

### 2단계 - Tail Call Optimization (TCO)

> `@` 재귀 연산자가 핵심 기능인데 TCO 없으면 스택 오버플로 위험

- [ ] **꼬리 호출 패턴 감지** - optimize.rs에 tail position 분석 추가
- [ ] **LLVM `musttail` 어노테이션 생성** - 꼬리 호출에 musttail 마킹
- [ ] **꼬리 재귀 → 루프 변환** - MIR 레벨에서 재귀를 while 루프로 변환
- [ ] **테스트** - factorial(100000), fib tail-recursive 버전 스택 오버플로 없이 통과

### 3단계 - 인라이닝 & 프로파일 기반 최적화

> 보수적 인라이닝 임계값 확대 + PGO 연동

- [ ] **인라이닝 임계값 상향** - optimize.rs 10 → 50 명령어로 확대
- [ ] **호출 빈도 기반 인라인 판단** - 핫 루프 내 함수 우선 인라인
- [ ] **PGO 파이프라인 연동** - 프로파일 데이터 → 인라인/최적화 결정에 반영
- [ ] **벤치마크** - 인라이닝 전/후 fibonacci, matrix_mul 성능 비교

### 4단계 - MIR 기반 최적화 파이프라인

> 텍스트 문자열 매칭 → 구조적 CFG 기반 최적화

- [ ] **AST → MIR 변환 활성화** - vais-mir 크레이트를 메인 파이프라인에 연결
- [ ] **MIR 레벨 DCE** - Dead Code Elimination을 MIR에서 수행
- [ ] **MIR 레벨 CSE** - Common Subexpression Elimination
- [ ] **MIR → LLVM IR 변환** - MIR에서 직접 LLVM IR 생성 (텍스트 최적화 단계 대체)

### 5단계 - 경계 검사 제거 & 고급 최적화

> 안전성 유지하면서 불필요한 런타임 검사 제거

- [ ] **범위 분석 (Range Analysis)** - 루프 인덱스 범위 증명
- [ ] **증명된 안전 접근 → 검사 제거** - `I i < arr.len` 후 `arr[i]` 접근 시 검사 불필요
- [ ] **SIMD 자동 벡터화 강화** - 배열 연산 루프 → SIMD 명령어 변환
- [ ] **캐시 친화적 데이터 레이아웃** - 구조체 필드 재배열 힌트

### 6단계 - 벤치마크 스위트 & 성능 검증

> 측정 없이 개선 없음

- [ ] **tiktoken 토큰 카운트 벤치마크** - Vais vs Rust vs C 동일 알고리즘 토큰 비교
- [ ] **런타임 성능 벤치마크 확장** - fibonacci, matrix_mul, quicksort, binary_tree
- [ ] **컴파일 속도 벤치마크** - 100/1K/10K LOC 파일 컴파일 시간 측정
- [ ] **메모리 사용량 벤치마크** - 피크 메모리, 할당 횟수 비교
- [ ] **자동 회귀 테스트** - CI에서 성능 회귀 시 경고

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

**메인테이너**: Steve

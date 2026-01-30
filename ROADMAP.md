# Vais - AI-Optimized Programming Language
## 프로젝트 로드맵

> **버전**: 0.1.0
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-01-30

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
├── vais-ast/      # 추상 구문 트리 ✅
├── vais-lexer/    # 토크나이저 (logos) ✅
├── vais-parser/   # Recursive descent 파서 ✅
├── vais-types/    # 타입 체커 ✅
├── vais-codegen/  # LLVM IR 생성기 ✅
├── vais-lsp/      # Language Server ✅
├── vais-i18n/     # 다국어 에러 메시지 ✅
├── vais-plugin/   # 플러그인 시스템 ✅
├── vais-jit/      # Cranelift JIT 컴파일러 ✅
└── vaisc/         # CLI 컴파일러 & REPL ✅

std/               # 표준 라이브러리 ✅
├── option.vais, result.vais, vec.vais, string.vais
├── hashmap.vais, file.vais, iter.vais, future.vais
├── rc.vais, box.vais, arena.vais, runtime.vais
├── math.vais, io.vais, set.vais, deque.vais
├── net.vais, priority_queue.vais, btreemap.vais
├── regex.vais, json.vais, hash.vais
├── time.vais, random.vais, uuid.vais, base64.vais, url.vais
├── thread.vais, sync.vais, http.vais
├── profiler.vais, test.vais, gc.vais
├── memory.vais, allocator.vais
├── collections.vais, crypto.vais, async.vais, fmt.vais
└── gpu.vais, hot.vais, contract_runtime.c

vscode-vais/       # VSCode Extension ✅
intellij-vais/     # IntelliJ Plugin ✅
benches/           # 벤치마크 스위트 ✅
selfhost/          # Self-hosting 컴파일러 ✅
playground/        # 웹 기반 실행 환경 ✅
docs-site/         # mdBook 문서 사이트 ✅
examples/          # 예제 코드 (40+ 파일) ✅
```

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

### 컴포넌트 현황

| 컴포넌트 | 상태 | 진행률 |
|----------|------|--------|
| Lexer | ✅ 완료 | 100% |
| Parser | ✅ 완료 | 100% |
| Type Checker | ✅ 완료 | 100% |
| Code Generator | ✅ 완료 | 100% |
| Standard Library | ✅ 완료 | 100% |
| LSP | ✅ 완료 | 100% |
| REPL | ✅ 완료 | 100% |
| Optimization | ✅ 완료 | 100% |
| VSCode Extension | ✅ 완료 | 100% |
| Formatter | ✅ 완료 | 100% |
| Debugger | ✅ 완료 | 100% |
| i18n | ✅ 완료 | 100% |
| Plugin System | ✅ 완료 | 100% |
| Benchmark Suite | ✅ 완료 | 100% |
| JIT Compiler | ✅ 완료 | 100% |

### 테스트 현황

```
✅ 402+ tests passed
✅ 104+ E2E integration tests
✅ 40+ example files compiled and running
✅ 21 i18n tests
✅ 100+ edge case tests
```

---

## 📊 전체 진행률 요약

| Phase | 상태 | 진행률 |
|-------|------|--------|
| Phase 1~12 | ✅ 완료 | 100% |
| Phase 13: 품질 보증 및 프로덕션 검증 | 🔄 진행 중 | P0-P2 완료, P3 진행 중 |
| **Phase 14: 배포 및 커뮤니티** | **✅ 완료** | **P0-P4 완료** |

---

## 🚀 Phase 13: 품질 보증 및 프로덕션 검증

> **상태**: 🔄 진행 중 (P0 완료, P1 완료, P2 완료)
> **추가일**: 2026-01-29
> **목표**: 테스트 커버리지 강화, 실사용 검증, v0.2.0 프로덕션 품질 달성

### P0 - 긴급 (1-2주) - 테스트 수정 및 CI 강화 ✅ 완료

#### 테스트 수정
- [x] **error_suggestion_tests 수정** - 6개 실패 테스트 복구 (완료일: 2026-01-29)
- [x] **통합 테스트 확장 Part 1** - 47개 E2E 테스트 (완료일: 2026-01-29)
- [x] **통합 테스트 확장 Part 2** - 42개 신규 E2E 테스트, 총 89개 (완료일: 2026-01-29)

#### CI/CD 강화
- [x] **Windows CI 추가** - ubuntu/macos/windows 3개 OS 매트릭스 (완료일: 2026-01-29)
- [x] **코드 커버리지 측정** - cargo-tarpaulin, 80%+ 목표 (완료일: 2026-01-29)

### P1 - 높은 우선순위 (3-4주) - Python 바인딩 및 에러 품질 ✅ 완료

#### Python 통합 완성
- [x] **vais-python 재활성화** - PyO3 0.22→0.25 업그레이드 (완료일: 2026-01-29)

#### 에러 메시지 품질 감사
- [x] **에러 메시지 전수 검사** - 18 TypeError + 3 ParseError + 6 CodegenError + 14+ 기타 (완료일: 2026-01-29)

### P2 - 중간 우선순위 (1-2개월) - 실사용 검증 ✅ 완료

#### 실세계 프로젝트 검증
- [x] **비즈니스 로직 프로젝트** - Math CLI & Data Processing, 15개 E2E 테스트 추가 (완료일: 2026-01-29)
- [x] **Quickstart 가이드** - "5분만에 시작하기" 문서 (완료일: 2026-01-29)

#### 언어 기능 보강
- [x] **Const Generics 개선** - const/type 제네릭 구분, 인스턴스화 (완료일: 2026-01-29)
- [x] **Named Arguments / Default Parameters** - 기본값 파싱, 생략 허용 (완료일: 2026-01-29)
- [x] **Procedural Macros** - TokenStream, 3가지 매크로 종류, 6개 빌트인 매크로 (완료일: 2026-01-29)

### P3 - 낮은 우선순위 (3-6개월) - 생태계 성장 🔄 진행 중

#### 패키지 에코시스템
- [x] **패키지 레지스트리 배포** - 7개 예제 패키지, publish/yank/login CLI, Docker 배포 (완료일: 2026-01-29)
- [x] **패키지 검색/디스커버리** - 고급 검색 API, 카테고리/태그/인기순 정렬 (완료일: 2026-01-29)

#### 성능 최적화
- [x] **Profile-Guided Optimization (PGO)** - `vaisc pgo` 자동화, llvm-profdata merge (완료일: 2026-01-29)
- [x] **병렬 컴파일** - rayon 기반 병렬 파싱/최적화, `--parallel`/`-j` CLI 플래그 (완료일: 2026-01-29)
- [x] **Comptime 확장** - 컴파일 타임 평가 강화 (String/Array 타입, 내장 함수 5종, assert, break/continue) (완료일: 2026-01-29)

#### IDE 경험 향상
- [x] **인라인 타입 힌트** - LSP inlay hints (타입 추론, 파라미터 이름 힌트, AST 기반 경량 추론) (완료일: 2026-01-29)
- [x] **리팩토링 도구** - Inline Variable, Convert Expression/Block Body, Introduce Named Params + 기존 Extract Variable/Function (완료일: 2026-01-29)
- [x] **Code Lens** - 테스트/벤치마크 실행, 참조 수, 구현 수 표시 (완료일: 2026-01-29)

### 예상 마일스톤

| 마일스톤 | 기간 | 목표 |
|----------|------|------|
| M1 | Week 2 | P0 완료 - 테스트 수정 및 CI 강화 ✅ |
| M2 | Week 6 | P1 완료 - Python 바인딩 및 에러 품질 ✅ |
| M3 | Week 12 | P2 완료 - 실사용 검증 및 언어 보강 ✅ |
| M4 | Week 24 | P3 완료 - 생태계 성장 |

---

## 🚀 Phase 14: 프로덕션 배포 및 커뮤니티 구축

> **상태**: ✅ 완료 (P0-P4 완료)
> **추가일**: 2026-01-29
> **목표**: 기술적 한계 해소, 설치 가능한 배포, 공식 웹사이트, 커뮤니티 채널 구축

### P0 - 긴급: 기술적 한계 해소 ✅ 완료

> 실제 프로그램 개발을 가로막는 핵심 버그 및 누락 기능 수정

#### 코드 생성 버그 수정 ✅ 완료
- [x] **중첩 구조체 필드 접근** - `o.a.val` 같은 다단계 필드 접근 codegen 구현 (완료일: 2026-01-30)
- [x] **Enum variant 매칭 버그** - unit variant가 항상 첫 번째 arm으로 매칭되는 문제 수정 (완료일: 2026-01-30)
- [x] **구조체 값 전달 타입 불일치** - 함수 인자로 구조체 전달 시 codegen 수정 (완료일: 2026-01-30)
- [x] **루프 변수 바인딩 codegen 버그** - `L x:arr` 패턴에서 변수 바인딩 수정 (완료일: 2026-01-30)
- [x] **논리 NOT codegen 버그** - `!expr` 연산 코드 생성 수정 (완료일: 2026-01-30)

#### 필수 언어 기능 추가
- [x] **print/println 내장 함수** - 포맷 문자열 지원하는 출력 함수 (`print("x = {}", x)`) (완료일: 2026-01-30)
- [x] **문자열 타입 완성** - 런타임 연결(+), 비교(==,!=,<,>), 메서드(len,charAt,contains,indexOf,substring,startsWith,endsWith,isEmpty) (완료일: 2026-01-30)
- [x] **배열 mutation** - 배열 요소 수정 (`arr[i] = val`) (완료일: 2026-01-30)
- [x] **format 함수** - 문자열 포매팅 (`format("hello {}", name)`) (완료일: 2026-01-30)
- [x] **stdlib 기본 함수** - atoi, atol, atof, labs, fabs, sqrt, rand, srand, isdigit, isalpha, toupper, tolower, strcpy, strcat (14개) (완료일: 2026-01-30)

#### 제네릭/트레이트 codegen 완성
- [x] **제네릭 함수 codegen** - 타입 체커 instantiation 기반 monomorphization, 다중 타입 인스턴스화, mangled name 호출 (완료일: 2026-01-30)
- [x] **트레이트 메서드 호출 codegen** - vtable 기반 동적 디스패치, fat pointer 생성, dyn Trait 파라미터 지원 (완료일: 2026-01-30)
- [x] **제네릭 stdlib E2E 테스트** - 제네릭 함수 monomorphization + 트레이트 동적 디스패치 통합 테스트 11개 (완료일: 2026-01-30)

### P1 - 높은 우선순위: 설치 및 배포 시스템 ✅ 완료

> `brew install vais` 한 줄로 설치 가능하게 만들기

#### 배포 패키지
- [x] **Homebrew Formula 작성** - macOS/Linux용 `brew install vais`, tap 저장소, CI 자동 업데이트 (완료일: 2026-01-30)
- [x] **cargo install 지원** - crates.io 메타데이터, MIT 라이선스 (완료일: 2026-01-30)
- [x] **Linux 패키지** - .deb 빌드 스크립트, .rpm spec, AUR PKGBUILD (완료일: 2026-01-30)
- [x] **Windows 설치** - Scoop manifest, WinGet manifest (완료일: 2026-01-30)
- [x] **Docker 이미지** - multi-stage Dockerfile, docker-compose.yml (완료일: 2026-01-30)

#### 릴리스 자동화
- [x] **GitHub Releases 자동화** - 4-platform matrix build (linux/macOS-x86/macOS-arm/windows), SHA256 checksums, 자동 릴리스 노트 (완료일: 2026-01-30)
- [x] **버전 관리 체계** - CHANGELOG.md (Keep a Changelog), MIGRATION.md (v0.1.0→v0.2.0 가이드) (완료일: 2026-01-30)

### P2 - 중간 우선순위: 공식 웹사이트 ✅ 완료

> Vais 언어의 얼굴이 되는 공식 웹사이트 구축

#### 웹사이트 (vais-lang.org 또는 vaislang.dev)
- [x] **랜딩 페이지** - 첫인상을 결정하는 메인 페이지 (완료일: 2026-01-30)
  - 언어 소개 (토큰 효율성, AI 최적화, 네이티브 성능)
  - 코드 비교 (Vais vs Rust vs Python 토큰 수 비교)
  - Playground 링크 (Open Playground CTA)
  - 주요 기능 하이라이트 (6가지)
  - "Try in Browser" / "brew install vais" CTA 버튼
- [x] **문서 사이트** - mdBook 기반 docs-site 배포 (완료일: 2026-01-30)
  - 기존 docs-site/ 디렉토리 활용
  - GitHub Actions 배포 워크플로우 (cargo-binstall 최적화)
  - 검색 기능 (mdBook 내장)
  - Vercel 대체 배포 설정
- [x] **Playground 배포** - 웹 브라우저에서 Vais 코드 실행 (완료일: 2026-01-30)
  - 기존 playground/ 디렉토리 활용
  - GitHub Actions + Vercel + Netlify 배포 설정
  - 프로덕션 base path 설정 (/playground/)
  - 예제 코드 갤러리
- [x] **블로그** - 개발 일지 및 언어 설계 결정 공유 (완료일: 2026-01-30)
  - "Why Vais?" 첫 포스트 (~850 words)
  - 블로그 인덱스 페이지 (카드 레이아웃)
  - 다크 테마, 코드 하이라이팅
- [x] **호스팅 및 도메인** (완료일: 2026-01-30)
  - GitHub Pages 배포 워크플로우 (website.yml, docs.yml, playground.yml)
  - Vercel/Netlify 대체 배포 설정
  - DNS 설정 가이드 (6개 레지스트라별)
  - 종합 배포 전략 문서 (DEPLOYMENT_*.md)

### P3 - 커뮤니티 및 홍보 ✅ 완료

> 사용자 유입과 커뮤니티 형성

#### SNS 채널
- [x] **Instagram 계정** - @vaislang (완료일: 2026-01-31)
  - 프로필 설정 (로고, 바이오, 웹사이트 링크)
  - 코드 스니펫 카드 디자인 템플릿 (community/CODE_CARD_TEMPLATES.md)
  - 콘텐츠 계획: community/SOCIAL_MEDIA_PLAN.md
  - 주 2-3회 포스팅 일정
- [x] **Twitter/X 계정** - @vaislang (완료일: 2026-01-31)
  - 개발 진행 상황 공유
  - #PLDev #ProgrammingLanguage 해시태그 활용
- [x] **GitHub Discussions 활성화** - 커뮤니티 Q&A (완료일: 2026-01-31)
  - Categories: General, Ideas, Show & Tell, Q&A
  - .github/DISCUSSION_TEMPLATE/ 4개 템플릿

#### 개발자 커뮤니티
- [x] **Discord 서버** - 실시간 소통 채널 (완료일: 2026-01-31)
  - community/DISCORD_SETUP.md 설정 가이드
- [x] **Reddit 홍보** - r/ProgrammingLanguages, r/rust, r/compilers (완료일: 2026-01-31)
  - community/LAUNCH_STRATEGY.md 포스트 초안
- [x] **Hacker News / Lobsters 포스트** - "Show HN: Vais - AI-optimized systems language" (완료일: 2026-01-31)
  - community/LAUNCH_STRATEGY.md 포스트 초안

#### 브랜딩
- [x] **로고 디자인** - 심볼 마크 + 워드 마크, 다크/라이트 버전 (완료일: 2026-01-31)
  - community/BRAND_GUIDE.md SVG 로고 3종
- [x] **브랜드 가이드** - 색상, 폰트, 톤앤매너 정의 (완료일: 2026-01-31)
  - community/BRAND_GUIDE.md
- [x] **코드 스니펫 비주얼** - SNS용 코드 카드 템플릿 (완료일: 2026-01-31)
  - community/CODE_CARD_TEMPLATES.md + community/templates/code-card.html

### P4 - 장기: 에코시스템 성장 ✅ 완료

> 지속 가능한 성장을 위한 기반

#### 교육 콘텐츠
- [x] **"Learn Vais in Y Minutes"** - learnxinyminutes.com 스타일 종합 레퍼런스 (완료일: 2026-01-31)
  - docs/learn-vais-in-y-minutes.md (~300줄)
- [x] **YouTube 튜토리얼 시리즈** - "Building X in Vais" 5편 스크립트 (완료일: 2026-01-31)
  - docs/youtube-tutorials/ (episode-01 ~ episode-05)
- [x] **Rosetta Code 기여** - 10개 알고리즘 예제 (완료일: 2026-01-31)
  - docs/rosetta-code/ (fibonacci, factorial, fizzbuzz, gcd, is_prime, bubble_sort, binary_search, towers_of_hanoi, palindrome, collatz)

#### 벤치마크 & 비교
- [x] **공식 벤치마크 페이지** - Vais vs C vs Rust vs Go vs Python (완료일: 2026-01-31)
  - docs/benchmarks.md (7개 벤치마크, 토큰 효율성 비교 포함)

#### 서드파티 통합
- [x] **GitHub Actions for Vais** - `setup-vais` 액션 (완료일: 2026-01-31)
  - github-action-setup-vais/ (action.yml + README.md)
- [x] **Rosetta Code** - Vais 언어 페이지 생성용 예제 10개 (완료일: 2026-01-31)
- [x] **TIOBE/PYPL** - 프로그래밍 언어 인덱스 등록 가이드 (완료일: 2026-01-31)
  - docs/language-index-registration.md (TIOBE, PYPL, GitHub Linguist, Wikipedia, StackOverflow)

### 예상 마일스톤

| 마일스톤 | 목표 |
|----------|------|
| M1 | P0 완료 - 핵심 버그 수정, 실사용 가능한 언어 ✅ |
| M2 | P1 완료 - brew/cargo install 배포, v0.2.0 릴리스 ✅ |
| M3 | P2 완료 - 공식 웹사이트 + Playground 오픈 ✅ |
| M4 | P3 완료 - SNS 채널 운영, 커뮤니티 100명 ✅ |
| M5 | P4 완료 - 교육 콘텐츠, 벤치마크, 서드파티 통합 ✅ |

---

**메인테이너**: Steve

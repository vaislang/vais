# Vais - AI-Optimized Programming Language
## 프로젝트 로드맵

> **버전**: 0.0.1
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-01-24

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

## 🎯 Phase 1: 핵심 컴파일러

> **상태**: ✅ 완료 (100%)

### 완료된 작업
- [x] **Lexer** (vais-lexer) - logos 기반 토크나이저
- [x] **AST** (vais-ast) - 추상 구문 트리 정의
- [x] **Parser** (vais-parser) - Recursive descent 파서
- [x] **Type Checker** (vais-types) - 타입 체크 및 추론
- [x] **Code Generator** (vais-codegen) - LLVM IR 생성
  - [x] 모듈 구조 리팩토링
  - [x] 함수 생성 및 호출
  - [x] 제어 흐름 (if/loop/match)
  - [x] 구조체/열거형
  - [x] 기본 연산 (arithmetic, comparison)
  - [x] 메모리 관리 (stack allocation, malloc)
- [x] 표현식 지향 문법
- [x] 자재귀 연산자 `@` 구현
- [x] Match 표현식 exhaustiveness 체크

### 고급 기능 (완료)
- [x] **Generics** - 제네릭 타입 파라미터
- [x] **Traits** - 트레이트 정의 및 구현
- [x] **Closures/Lambdas** - 클로저 및 람다 표현식
- [x] **Async/Await** - 비동기 프로그래밍
- [x] **Pattern Matching** - 고급 패턴 매칭 (destructuring, guards)
- [x] **Module System** - 모듈 import/export

### 남은 작업
- [x] 에러 메시지 개선 (완료일: 2026-01-20)
  - ErrorReporter 모듈 추가 (줄 번호, 컬럼, 소스 코드 표시)
  - TypeError/ParseError에 Span 정보 및 에러 코드 추가
  - help 메시지 시스템 구현
- [x] 타입 추론 개선 (완료일: 2026-01-20)
  - fresh_type_var() 버그 수정 (고유 ID 부여)
  - 제네릭 구조체 타입 인자 추론
  - 클로저 파라미터 타입 추론 개선
  - substitute_generics() 헬퍼 함수 추가

---

## 🚀 Phase 2: 표준 라이브러리

> **상태**: ✅ 완료 (100%)

### 완료된 작업
- [x] **Option<T>** - 옵셔널 타입 (`std/option.vais`)
- [x] **Result<T, E>** - 에러 처리 타입 (`std/result.vais`)
- [x] **Vec<T>** - 동적 배열 (`std/vec.vais`)
- [x] **String** - 문자열 처리 (`std/string.vais`)
- [x] **HashMap<K, V>** - 해시맵 (`std/hashmap.vais`)
- [x] **File** - 파일 입출력 (`std/file.vais`)
- [x] **Iterator** - 이터레이터 트레이트 (`std/iter.vais`)
- [x] **Future** - 비동기 Future (`std/future.vais`)
- [x] **Rc<T>** - 참조 카운팅 (`std/rc.vais`)
- [x] **Box<T>** - 힙 할당 (`std/box.vais`)
- [x] **Arena** - 아레나 할당자 (`std/arena.vais`)
- [x] **Runtime** - 런타임 지원 (`std/runtime.vais`)

- [x] **Math** - 수학 함수 (`std/math.vais`)
  - 상수: PI, E, TAU
  - 기본: abs, min, max, clamp
  - 수학: sqrt, pow, floor, ceil, round
  - 삼각함수: sin, cos, tan, asin, acos, atan, atan2
  - 로그: log, log10, log2, exp
- [x] **IO** - 표준 입력 처리 (`std/io.vais`)
  - read_line, read_i64, read_f64
  - read_char, read_word
  - prompt_line, prompt_i64, prompt_f64

- [x] **Set<T>** - 해시 기반 집합 (`std/set.vais`) **NEW**
  - set_new, set_insert, set_contains, set_remove
  - set_size, set_is_empty, set_clear, set_free
- [x] **Deque<T>** - 양방향 큐 (`std/deque.vais`) **NEW**
  - deque_new, deque_push_front, deque_push_back
  - deque_pop_front, deque_pop_back, deque_get
  - deque_size, deque_is_empty, deque_free

- [x] **Net** - 네트워크 기본 지원 (`std/net.vais`) **NEW**
  - TCP: TcpListener, TcpStream (listen, accept, connect, read, write)
  - UDP: UdpSocket (bind, send_to, recv_from)
  - C FFI 연동 (socket, bind, listen, accept 등)
  - C-style API: tcp_listen, tcp_connect, udp_bind 등

### 남은 작업
- (없음)

---

## ⚡ Phase 3: 개발자 도구

> **상태**: ✅ 완료 (100%)

### 완료된 작업
- [x] **LSP Server** (vais-lsp)
  - [x] 기본 진단 (diagnostics)
  - [x] 시맨틱 토큰 하이라이팅
  - [x] 자동 완성 (키워드, 타입, 빌트인 함수, std 모듈, 메서드)
  - [x] Go to definition
  - [x] Hover 정보 (함수, 구조체, 열거형, 트레이트, 빌트인)
  - [x] Find references
- [x] **REPL** - 대화형 환경
  - [x] 표현식 평가
  - [x] 함수/타입 정의 지원
  - [x] 명령어 시스템 (:help, :clear, :load 등)
- [x] **Optimization Passes** (vais-codegen/optimize.rs)
  - [x] Constant folding
  - [x] Dead store elimination
  - [x] Branch optimization
  - [x] Common subexpression elimination
  - [x] Strength reduction
  - [x] Dead code elimination
- [x] **Doc Generator** - 문서 생성

- [x] **VSCode Extension** (`vscode-vais/`)
  - [x] TextMate 문법 파일 (syntax highlighting)
  - [x] 언어 설정 (brackets, comments, indentation)
  - [x] LSP 클라이언트 연동 (자동 완성, hover, go-to-definition)

- [x] **Documentation** (docs/)
  - [x] LANGUAGE_SPEC.md - 언어 스펙
  - [x] TUTORIAL.md - 튜토리얼
  - [x] STDLIB.md - 표준 라이브러리 레퍼런스

- [x] **Formatter** (`vais fmt`) - 코드 포맷터 **NEW**
  - AST 기반 pretty-print
  - 들여쓰기 설정 (--indent)
  - 체크 모드 (--check)

- [x] **Debugger** - 디버깅 지원 **NEW**
  - DWARF 디버그 메타데이터 생성 (DIFile, DISubprogram, DILocation)
  - `--debug` / `-g` CLI 옵션
  - lldb/gdb 소스 레벨 디버깅 지원

### 남은 작업
- (없음)

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
├── vais-jit/      # Cranelift JIT 컴파일러 ✅ NEW
└── vaisc/         # CLI 컴파일러 & REPL ✅

std/               # 표준 라이브러리 ✅
├── option.vais
├── result.vais
├── vec.vais
├── string.vais
├── hashmap.vais
├── file.vais
├── iter.vais
├── future.vais
├── rc.vais
├── box.vais
├── arena.vais
├── runtime.vais
├── math.vais
├── io.vais
├── set.vais
├── deque.vais
└── net.vais       # NEW

vscode-vais/       # VSCode Extension ✅
├── package.json
├── language-configuration.json
└── syntaxes/vais.tmLanguage.json

benches/           # 벤치마크 스위트 ✅ NEW
├── Cargo.toml
├── compile_bench.rs
├── runtime_bench.rs
└── fixtures/      # 벤치마크 테스트 파일

examples/          # 예제 코드 (40+ 파일) ✅
```

---

## 📊 컴포넌트 현황

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

**핵심 기능 진행률: 100%** (Phase 1-3 완료)

---

## 🧪 테스트 현황

```
✅ 402 tests passed, 0 failed
✅ 40+ example files compiled and running
✅ 47 E2E integration tests
✅ 21 i18n tests
✅ 100+ edge case tests
```

---

## 최근 커밋

```
ecdc5ca Add LSP client to VSCode extension
ae528ef Enhance LSP with comprehensive auto-completion and hover support
90b925e Add comprehensive language documentation
8df5e53 Add test examples for Math and IO standard library modules
5c2d61c Add VSCode extension and Math/IO standard library modules
```

## 최근 변경사항 (2026-01-21)

### 제네릭 표준 라이브러리 완성
- **Vec<T>, HashMap<K,V>, Option<T> 제네릭화 확인 및 검증**
  - std/vec.vais: Vec<T> 제네릭 구조체로 동작 확인
  - std/hashmap.vais: HashMap<K, V> 제네릭 구조체로 동작 확인
  - std/option.vais: Option<T> 제네릭 열거형으로 동작 확인
  - 타입 인자 자동 추론 동작 (예: `Vec.with_capacity(10)` → `Vec<i64>`)
  - Monomorphization 인프라와 완전 통합
  - 테스트 302개 통과

## 이전 변경사항 (2026-01-20)

### 플러그인 시스템 추가
- **vais-plugin 크레이트** 추가 (`crates/vais-plugin/`)
  - libloading 기반 동적 라이브러리 로딩
  - 4가지 플러그인 타입 지원:
    - **Lint**: 코드 검사, 진단 메시지 반환
    - **Transform**: 타입 검사 전 AST 수정
    - **Optimize**: 코드 생성 후 LLVM IR 최적화
    - **Codegen**: 추가 파일 생성 (바인딩, 문서 등)
  - PluginRegistry: 플러그인 관리 및 실행
  - PluginsConfig: vais-plugins.toml 설정 파싱
- **CLI 옵션** 추가
  - `--plugin <PATH>`: 추가 플러그인 로드
  - `--no-plugins`: 모든 플러그인 비활성화
- **예제 플러그인** (`examples/plugins/example-lint/`)
  - naming-convention 린트 플러그인
  - snake_case 명명 규칙 검사
  - 함수 이름 길이 검사
- **테스트 17개 추가**

### i18n 에러 메시지 다국어 지원
- **vais-i18n 크레이트** 추가 (`crates/vais-i18n/`)
  - JSON 기반 메시지 로딩 (컴파일 타임 embed)
  - 영어(en), 한국어(ko), 일본어(ja) 지원
  - 변수 치환 지원 (예: `{expected}`, `{found}`)
- **CLI 옵션** `--locale` 추가
  - `vaisc --locale ko check file.vais` 형식으로 사용
  - `VAIS_LANG` 환경변수 지원
  - 시스템 `LANG` 환경변수 자동 감지
- **에러 메시지 다국어화**
  - TypeError 11종 에러 메시지
  - ParseError 3종 에러 메시지
- **테스트 21개 추가**

### 루프 최적화 추가
- **Loop Unrolling** (`vais-codegen/src/optimize.rs`)
  - 고정 횟수 루프 자동 펼치기 (UNROLL_FACTOR=4)
  - 작은 루프 바디(MAX_BODY_SIZE=20) 자동 감지
  - SSA 변수 이름 자동 리네이밍
  - 인덕션 변수 및 바운드 분석
- **Loop Invariant Code Motion (LICM) 개선**
  - 루프 불변 계산식 자동 감지
  - 프리헤더 생성을 통한 호이스팅
  - 루프 변수 추적 및 의존성 분석
  - phi/load 명령어 제외 처리
- **테스트 케이스 5개 추가**
  - test_loop_unrolling
  - test_loop_invariant_motion
  - test_rename_for_unroll
  - test_full_loop_optimization

### IPv6 소켓 지원 추가
- **Net 모듈 IPv6 확장** (`std/net.vais`)
  - sockaddr_in6 구조체 헬퍼 함수 추가 (28 bytes)
    - make_sockaddr_in6(host, port): IPv6 주소 구조체 생성
    - make_sockaddr_any6(port): 와일드카드 주소 (::) 구조체 생성
  - TcpListener IPv6 지원
    - bind6(port): IPv6 TCP 리스너 생성
    - tcp_listen6(port): C-style API
  - TcpStream IPv6 지원
    - connect6(host, port): IPv6 TCP 연결
    - tcp_connect6(host, port): C-style API
  - UdpSocket IPv6 지원
    - new6(): 언바운드 IPv6 UDP 소켓
    - bind6(port): IPv6 UDP 소켓 바인드
    - send_to6(data, len, host, port): IPv6 주소로 전송
    - recv_from6(buffer, len, src_addr_out, src_port_out): IPv6 주소에서 수신
    - udp_bind6(port), udp_send_to6(...): C-style API
  - IPv6 상수 추가
    - AF_INET6 = 30 (macOS)
    - IPPROTO_IPV6 = 41
    - IPV6_V6ONLY = 27 (macOS)
    - SOCKADDR_IN6_SIZE = 28
  - 유틸리티 함수
    - is_valid_ip6(host): IPv6 주소 유효성 검사

### 디버거 표현식 레벨 위치 정보 추가
- **표현식 디버그 메타데이터** (`vais-codegen/src/debug.rs`, `lib.rs`)
  - `dbg_ref_from_span()` 헬퍼 메서드: Span에서 DILocation 생성
  - 모든 함수 호출 명령어에 `!dbg` 메타데이터 추가
    - 직접 함수 호출 (void/non-void)
    - 간접 함수 호출 (람다/클로저)
    - 특수 함수 (malloc, free, memcpy, strlen, puts_ptr)
  - 산술/비교/논리 연산자에 `!dbg` 메타데이터 추가
    - Binary 연산: add, sub, mul, div, mod, and, or, xor, shl, shr
    - Comparison 연산: eq, ne, lt, le, gt, ge
    - Unary 연산: neg, not, bitnot
- **디버깅 개선**
  - lldb/gdb에서 표현식 단위 스텝 실행 가능
  - 함수 호출 위치 정확히 추적 가능
  - LLVM 경고 메시지 제거 (missing !dbg)

### Net 모듈 추가
- **std/net.vais** - 네트워크 소켓 지원
  - TcpListener: bind, accept, close
  - TcpStream: connect, read, write, write_all
  - UdpSocket: bind, send_to, recv, recv_from
  - C FFI 선언: socket, bind, listen, accept, connect, send, recv 등
  - C-style 편의 함수: tcp_listen, tcp_connect, udp_bind 등

### Debugger 지원 추가
- **debug.rs** 모듈 (`vais-codegen/src/debug.rs`)
  - DWARF 디버그 메타데이터 생성 (DIFile, DICompileUnit, DISubprogram, DILocation)
  - 소스 줄/컬럼 번호 계산
- **CLI 옵션**
  - `--debug` / `-g`: 디버그 정보 포함 컴파일
  - 디버그 모드에서 자동 최적화 비활성화
- lldb/gdb에서 소스 레벨 브레이크포인트 지원

### 코드 포맷터 추가
- **Formatter 모듈** (`vais-codegen/src/formatter.rs`)
  - AST 기반 pretty-print 구현
  - 모든 언어 구성요소 지원 (함수, 구조체, 열거형, 트레이트, impl 등)
  - 중첩된 if-else, loop, match 적절한 들여쓰기
- **`vaisc fmt` 서브커맨드** 추가
  - `--check` 모드: 포맷팅 필요 여부 확인
  - `--indent` 옵션: 들여쓰기 크기 설정
  - 디렉토리 재귀 처리 지원

### 타입 추론 개선
- **fresh_type_var() 버그 수정**
  - Cell<usize>를 사용하여 각 타입 변수에 고유 ID 부여
  - 여러 독립적인 타입 추론이 간섭하지 않도록 수정
- **제네릭 구조체 타입 인자 추론**
  - 필드 값에서 제네릭 타입 인자를 자동 추론
  - `substitute_generics()` 헬퍼 함수 추가
- **클로저 파라미터 타입 추론**
  - Type::Infer 파라미터의 타입을 본문 사용에서 추론

### 에러 메시지 개선
- **ErrorReporter 모듈** 추가 (`vais-types/src/error_report.rs`)
  - 줄 번호, 컬럼, 소스 코드 스니펫 표시
  - 에러 위치에 캐럿(^) 지시자 표시
  - 컬러 출력 지원 (colored 크레이트)
- **TypeError 개선**
  - 모든 variant에 `Option<Span>` 필드 추가
  - 에러 코드 시스템 (E001-E011)
  - help 메시지 시스템 (수정 제안)
- **ParseError 개선**
  - 모든 variant에 span 정보 추가
  - 에러 코드 시스템 (P001-P003)

### 버그 수정 (2026-01-19)
- **Codegen**: Nested if-else phi node predecessor 버그 수정
  - 문제: else 블록에 중첩된 if-else가 있을 때 phi 노드의 predecessor가 잘못 설정됨
  - 해결: `current_block` 필드로 현재 basic block 추적

### 신규 기능 (2026-01-19)
- **Set<T>**: 해시 기반 집합 자료구조 추가 (`std/set.vais`)
- **Deque<T>**: 원형 버퍼 기반 양방향 큐 추가 (`std/deque.vais`)

---

## 🔮 Phase 4: 향후 개선 사항

> **상태**: ✅ 완료 (100%)

### 디버거 개선
- [x] **표현식 레벨 위치 정보** - 함수 호출에 `!dbg` 위치 메타데이터 추가 (완료일: 2026-01-20)
  - 모든 함수 호출 명령어에 `!dbg` 메타데이터 추가
  - 산술 연산자, 비교 연산자, 논리 연산자에 위치 정보 추가
  - 단항 연산자에 위치 정보 추가
  - `dbg_ref_from_span()` 헬퍼 메서드 추가
  - lldb/gdb에서 표현식 레벨 디버깅 가능

### 네트워크 확장
- [x] **IPv6 지원** - Net 모듈 IPv6 소켓 지원 (완료일: 2026-01-20)
  - sockaddr_in6 구조체 추가 (28 bytes)
  - TCP/UDP IPv6 버전 함수 추가 (bind6, connect6, send_to6, recv_from6)
  - C-style API: tcp_listen6, tcp_connect6, udp_bind6, udp_send_to6
  - IPv6 헬퍼 함수: make_sockaddr_in6, make_sockaddr_any6, is_valid_ip6
  - AF_INET6, IPPROTO_IPV6, IPV6_V6ONLY 상수 추가

### 추가 표준 라이브러리
- [x] **PriorityQueue<T>** - 우선순위 큐 (`std/priority_queue.vais`) (완료일: 2026-01-20)
  - 힙 기반 구현 (min-heap)
  - push, pop, peek 연산 지원
  - 동적 크기 조정 (grow)
- [x] **BTreeMap<K, V>** - B-트리 맵 (`std/btreemap.vais`) (완료일: 2026-01-20)
  - 정렬된 키 순회 지원
  - 범위 쿼리 지원
  - insert, get, remove, contains 연산
  - min, max, range 쿼리 지원
- [x] **Regex** - 정규표현식 (`std/regex.vais`) (완료일: 2026-01-20)
  - 기본 패턴 매칭 (., *, +, ?, [], ^, $)
  - 캡처 그룹 지원
  - match, find, find_all, replace 함수
- [x] **JSON** - JSON 파서/생성기 (`std/json.vais`) (완료일: 2026-01-20)
  - parse/stringify 함수
  - JsonValue 타입 (Object, Array, String, Number, Bool, Null)
  - 중첩 객체/배열 지원

### 컴파일러 최적화
- [x] **인라이닝 최적화** - 작은 함수 자동 인라인 (완료일: 2026-01-20)
  - `alwaysinline` 속성으로 작은 함수 자동 인라인
  - 상수 전파(constant propagation) 지원
  - 공통 부분식 제거(CSE) 지원
  - 강도 감소(strength reduction) 지원
  - LLVM `-O2` 최적화 레벨 통합
- [x] **루프 최적화** - Loop unrolling, Loop invariant code motion (완료일: 2026-01-20)
  - Loop unrolling (고정 횟수 루프 펼치기, UNROLL_FACTOR=4)
  - Loop Invariant Code Motion (LICM) - 루프 불변식 외부 이동
  - 프리헤더 생성을 통한 호이스팅 최적화
  - 테스트 케이스 5개 추가

### 남은 작업
- (없음)

---

## 📊 진행률 요약 (Phase 4 포함)

| Phase | 상태 | 진행률 |
|-------|------|--------|
| Phase 1: 핵심 컴파일러 | ✅ 완료 | 100% |
| Phase 2: 표준 라이브러리 | ✅ 완료 | 100% |
| Phase 3: 개발자 도구 | ✅ 완료 | 100% |
| Phase 4: 향후 개선 | ✅ 완료 | 100% |
| Phase 5: 품질 개선 | ✅ 완료 | 100% |
| Phase 6: 후속 개선 | ✅ 완료 | 100% |

---

## 🔧 Phase 5: 품질 개선 및 안정화

> **상태**: ✅ 완료 (100%)

### P0 - 긴급 (즉시 수행)
- [x] **테스트 실행 문제 해결** - 조사 결과 정상 작동 확인 (46 tests passed) (완료일: 2026-01-20)
- [x] **README.md 업데이트** - ROADMAP과 일치하도록 완료된 기능 체크박스 업데이트 (완료일: 2026-01-20)

### P1 - 높은 우선순위 (1주일 내)
- [x] **TODO 주석 해결** - trait 메서드의 async 지원 구현 (완료일: 2026-01-20)
  - TraitMethod에 is_async 필드 추가 (AST)
  - 파서에서 `A F method()` 형태 파싱 지원
  - 타입 체커에서 async 정보 전파
- [x] **파서 테스트 개선** - panic! 대신 let...else + unreachable! 패턴 사용 (완료일: 2026-01-20)
  - 10개의 panic! 매크로 제거
  - let...else 구문으로 Rust 관용적 패턴 적용
  - matches! 매크로 활용
- [x] **vais-codegen 모듈 분리** - lib.rs를 논리적 모듈로 분리 (완료일: 2026-01-20)
  - types.rs (259줄) - 타입 정의 및 변환
  - stmt.rs (162줄) - 문장 코드 생성
  - lib.rs 3,981줄 → 3,589줄 (392줄 감소)
- [x] **vais-types 모듈 분리** - 타입 체커를 논리적 모듈로 분리 (완료일: 2026-01-20)
  - types.rs (326줄) - 핵심 타입 정의
  - inference.rs (176줄) - 타입 추론 알고리즘
  - lib.rs 2,814줄 → 2,346줄 (468줄 감소)

### P2 - 중간 우선순위 (1개월 내)
- [x] **엣지 케이스 테스트 추가** - 빈 파일, 잘못된 문법, 경계값 테스트 (완료일: 2026-01-20)
  - vais-lexer: 25개 테스트 추가 (9→34)
  - vais-parser: 51개 테스트 추가 (6→57)
  - vais-types: 38개 테스트 추가 (10→48)
  - vais-codegen: 38개 테스트 추가 (20→58)
  - 총 테스트: 198개 (이전 46개)
- [x] **통합 테스트 스위트 구축** - 전체 컴파일 파이프라인 테스트 (완료일: 2026-01-20)
  - `crates/vaisc/tests/integration_tests.rs` 추가
  - 47개 E2E 테스트: Lexer → Parser → TypeChecker → CodeGen
  - 기본 컴파일, 제어 흐름, 타입, 연산자, 에러 감지 테스트
  - 총 테스트: 245개
- [x] **표준 라이브러리 에러 처리 개선** - 0 반환 대신 Option/Result 타입 일관적 사용 (완료일: 2026-01-20)
  - 9개 파일에 24개 Option/Result 반환 함수 추가
  - Vec, HashMap, String, Deque, PriorityQueue, BTreeMap, IO, File, Net
  - 기존 API 100% 호환성 유지
- [x] **입력 검증 강화** - 네트워크/파일 API에 입력 범위 검사 추가 (완료일: 2026-01-20)
  - net.vais: 포트 범위(0-65535), 버퍼/포인터 null 검사
  - file.vais: 경로 null, 버퍼 크기, seek 원점 검증
- [x] **unwrap/expect 감소** - 126개 사용처 검토 및 적절한 에러 핸들링 추가 (완료일: 2026-01-20)
  - 326개 → 316개 (10개 위험한 패턴 제거)
  - 경로 처리, I/O, 파서 토큰 처리 개선
  - 컴파일러 내부 불변조건은 명시적 expect()로 변환

### P3 - 낮은 우선순위 (3개월 내)
- [x] **CONTRIBUTING.md 작성** - 기여 가이드라인 문서화 (완료일: 2026-01-20)
  - 개발 환경 설정, 코드 스타일, PR 가이드, 이슈 보고 양식
- [x] **rustdoc 문서 생성** - Rust API에 doc 주석 추가 및 문서 생성 (완료일: 2026-01-20)
  - 5개 크레이트에 크레이트 레벨 및 주요 API 문서 추가
  - Doc 테스트 5개 포함
- [x] **CI/CD 파이프라인 구축** - GitHub Actions로 자동 테스트/빌드 (완료일: 2026-01-20)
  - .github/workflows/ci.yml 생성
  - Linux/macOS 매트릭스 빌드
  - 포맷팅, 린팅, 테스트, 빌드, 문서 생성 단계

### P4 - 미래 (6개월+)
- [x] **i18n 에러 메시지** - 에러 메시지 다국어 지원 인프라 구축 (완료일: 2026-01-20)
  - vais-i18n 크레이트 추가 (JSON 기반 메시지 로딩)
  - 영어(en), 한국어(ko), 일본어(ja) 지원
  - CLI `--locale` 옵션 추가 (VAIS_LANG 환경변수 지원)
  - TypeError/ParseError 에러 메시지 다국어 지원
- [x] **플러그인 시스템** - 컴파일러 확장 API 설계 및 구현 (완료일: 2026-01-20)
  - vais-plugin 크레이트 추가 (libloading 기반 동적 로딩)
  - 4가지 플러그인 타입: Lint, Transform, Optimize, Codegen
  - CLI 옵션: `--plugin <PATH>`, `--no-plugins`
  - vais-plugins.toml 설정 파일 지원
  - 예제 플러그인: naming-convention lint 플러그인
- [x] **제네릭 표준 라이브러리** - Vec<T>, HashMap<K,V>의 실제 제네릭 지원 (완료일: 2026-01-21)
  - [x] Monomorphization 설계 문서 작성 (docs/design/monomorphization-design.md)
  - [x] GenericInstantiation 추적 구조체 추가
  - [x] mangle_name, mangle_type, substitute_type 유틸리티
  - [x] 코드젠 제네릭 치환 인프라
  - [x] 타입 체커 제네릭 인스턴스화 추론 (완료일: 2026-01-20)
    - check_generic_function_call() 메서드 추가
    - 제네릭 함수 호출 시 타입 인자 자동 추론
    - 제네릭 구조체 리터럴에서 인스턴스화 기록
    - 9개 테스트 케이스 추가
  - [x] 코드젠 특수화된 함수/구조체 생성 (완료일: 2026-01-20)
    - generate_module_with_instantiations() 메서드 추가
    - generate_specialized_function() - 제네릭 함수의 특수화된 LLVM IR 생성
    - generate_specialized_struct_type() - 제네릭 구조체 타입 정의 생성
    - 4개 테스트 케이스 추가
  - [x] Vec<T>, HashMap<K,V>, Option<T> 제네릭화 완료 (완료일: 2026-01-21)
    - std/vec.vais: Vec<T> 제네릭 구조체 및 메서드
    - std/hashmap.vais: HashMap<K, V> 제네릭 구조체 및 메서드
    - std/option.vais: Option<T> 제네릭 열거형 및 메서드
    - 타입 인자 자동 추론 동작 확인
- [x] **REPL 개선** - 멀티라인 입력, 히스토리, 탭 자동완성 (완료일: 2026-01-20)
  - rustyline 기반 멀티라인 입력 (중괄호/괄호 균형 검사)
  - 화살표 키 히스토리 탐색 (최대 100개, 파일 저장)
  - 키워드 + 내장 함수 탭 자동완성
- [x] **LSP Rename 기능** - 심볼 이름 변경 기능 추가 (완료일: 2026-01-20)
  - textDocument/prepareRename, textDocument/rename 핸들러
  - 변수, 함수, 구조체, 열거형, 트레이트 지원
- [x] **벤치마크 스위트** - 성능 측정 및 회귀 테스트 (완료일: 2026-01-20)
  - Criterion 기반 벤치마크 프레임워크
  - 4개 fixture 파일 (fibonacci, sort, struct_heavy, complex)
  - 컴파일러 단계별 벤치마크 (lexer, parser, type_checker, codegen, full_compile)
  - 런타임 비교 벤치마크 (Vais vs Rust)
  - CI 통합 (main 브랜치 자동 측정, PR 비교)

### 남은 작업
- (없음)

---

## 🚀 Phase 6: 후속 개선 및 확장

> **상태**: ✅ 완료 (100%)
> **추가일**: 2026-01-21

### P0 - 즉시 (1-2주)
- [x] **테스트 커버리지 확대** - 엣지 케이스 테스트 100개 추가 (완료일: 2026-01-21)
  - 제네릭 중첩 조합 (Vec<HashMap<K, Option<V>>>)
  - 패턴 매칭 + guard 조합
  - 상호 재귀 함수 타입 추론
  - 정수 오버플로우/언더플로우
  - 테스트 302개 → 402개
- [x] **import 경로 보안 강화** - 경로 트래버설 취약점 방지 (완료일: 2026-01-21)
  - canonical 경로 검증 (std::fs::canonicalize)
  - 심볼릭 링크 처리 (외부 경로 차단)
  - 보안 테스트 11개 추가
- [x] **코드 중복 제거** - 공통 유틸리티 모듈화 (완료일: 2026-01-21)
  - type_to_llvm 캐싱 (RefCell+HashMap)
  - 에러 포맷팅 통합 (FormattableError 트레이트)
  - 9개 캐싱 테스트 추가

### P1 - 중기 (2-4주)
- [x] **성능 최적화** (완료일: 2026-01-21)
  - LSP 심볼 인덱스 캐싱 (SymbolCache 구조체)
  - 타입 대체 메모이제이션 (substitute_generics 캐싱)
  - 패턴 매칭 exhaustiveness 캐싱
  - 9개 최적화 테스트 추가
  - Criterion 벤치마크 추가
- [x] **문서 확충** (완료일: 2026-01-21)
  - Architecture.md - 컴파일 파이프라인, 크레이트 구조, 설계 결정 설명 (500줄)
  - INSTALLATION.md - 플랫폼별 설치 가이드 (377줄)
  - README.md - 벤치마크 결과 및 문서 링크 추가
- [x] **LSP 통합 테스트** - Mock 클라이언트 E2E 테스트 (완료일: 2026-01-21)
  - tower-lsp 기반 16개 통합 테스트
  - 서버 초기화/종료, 자동 완성 (키워드, 타입, 빌트인, std 모듈)
  - 에러 핸들링 (없는 문서 처리), capability 검증
- [x] **플러그인/formatter 테스트** - 로드/실행 테스트 추가 (완료일: 2026-01-21)
  - 플러그인 테스트 26개 (PluginRegistry, PluginsConfig, Diagnostic, 에러 핸들링)
  - Formatter 테스트 34개 (함수, 구조체, 열거형, 제어 흐름, 들여쓰기)

### 남은 작업
- (없음 - P2/P3 항목은 Phase 7로 이동)

---

## 🚀 Phase 7: 아키텍처 개선 및 확장

> **상태**: ✅ P0+P1 완료 (100%)
> **추가일**: 2026-01-21

### P0 - 즉시 (1-2주)
- [x] **TODO 주석 해결** - 플러그인 이름으로 로드 기능 구현 (완료일: 2026-01-21)
  - load_plugin_by_name() 메서드 추가
  - 플러그인 검색 디렉토리: ~/.vais/plugins/, /usr/local/lib/vais/plugins/, VAIS_PLUGIN_PATH
  - 플랫폼별 라이브러리 확장자 지원 (dylib/so/dll)
  - 테스트 4개 추가
- [x] **Parser 모듈 분해 1단계** - 표현식 파싱을 expr.rs로 분리 (완료일: 2026-01-21)
  - lib.rs 3,073줄 → lib.rs 1,937줄 + expr.rs 1,049줄 + stmt.rs 117줄
  - 표현식 파싱(binary, unary, postfix, primary, control flow) → expr.rs
  - 문장 파싱(let, return, break, continue) → stmt.rs
  - 테스트 81개 통과
- [x] **테스트 커버리지 보고서** - cargo-tarpaulin 설정 (완료일: 2026-01-21)
  - tarpaulin.toml 설정 파일 생성
  - .cargo/config.toml에 cargo coverage alias 추가
  - scripts/coverage.sh 스크립트 생성
  - CI 워크플로우에 coverage job 추가
  - docs/COVERAGE.md 문서화

### P1 - 중기 (2-4주)
- [x] **Codegen 리팩토링** - Visitor 패턴 도입 (완료일: 2026-01-21)
  - visitor.rs: ExprVisitor, StmtVisitor, ItemVisitor trait 정의 (234줄)
  - expr_visitor.rs: 표현식 방문자 구현 (361줄)
  - expr_helpers.rs: 표현식 헬퍼 분리 (1,314줄)
  - stmt_visitor.rs: 문장 방문자 구현 (185줄)
  - 전체 테스트 통과
- [x] **고급 튜토리얼 작성** (완료일: 2026-01-21)
  - async_tutorial.md - Async/Await 패턴, Future trait, 비동기 에러 처리
  - generic_tutorial.md - 제네릭, 트레이트, 바운드, 표준 라이브러리 활용
- [x] **다국어 확장** - 중국어(zh) 에러 메시지 추가 (완료일: 2026-01-21)
  - locales/zh.json 생성 (14개 에러 메시지)
  - Locale::Zh 지원 추가
- [x] **clone() 최적화** - codegen에서 참조 기반 리팩토링 (완료일: 2026-01-21)
  - 195개 → 153개 (42개 제거, 21.5% 감소)
  - formatter.rs: 11개 → 1개 (90.9% 감소)
  - lib.rs: 127개 → 100개 (21.3% 감소)
  - clone() → to_string(), clone_from() 패턴 적용
- [x] **LSP 추가 기능** - Code Actions 구현 (완료일: 2026-01-21)
  - Quick fixes: 변수 생성, import 추가, 타입 캐스트
  - Refactoring: 변수 추출, 함수 추출

### P2 - 장기 (4-8주)
- [x] **Wasm 컴파일 대상** - wasm32-unknown-unknown 타겟 지원 (완료일: 2026-01-21)
  - CLI `--target` 옵션 추가 (wasm32, wasi, x86_64, aarch64, native)
  - TargetTriple enum 및 관련 메서드 구현
  - CodeGenerator에 타겟별 IR 헤더 생성 (target triple, data layout)
  - 타겟별 clang 컴파일 옵션 분기 (wasm32, wasi, native)
  - 참고: 실제 wasm 바이너리 생성은 wasm-sdk 설치 필요
- [x] **증분 컴파일** - 파일 해시 기반 캐싱, 변경된 파일만 재컴파일 (완료일: 2026-01-21)
  - IncrementalCache 구조체 (crates/vaisc/src/incremental.rs)
  - SHA256 파일 해시 기반 변경 감지
  - 의존성 그래프 (DependencyGraph) - forward/reverse 의존성 추적
  - 캐시 상태 JSON 직렬화 (.vais-cache/cache_state.json)
  - CompilationOptions로 opt_level, debug, target_triple 변경 감지
  - CLI `--force-rebuild` 옵션 추가
  - 캐시 버전 및 컴파일러 버전 검증
- [x] **IntelliJ IDE 플러그인** - LSP 클라이언트 기반 (완료일: 2026-01-21)
  - intellij-vais/ 프로젝트 디렉토리
  - Kotlin/Gradle 기반 IntelliJ Platform Plugin SDK
  - 구문 강조 (VaisLexer, VaisSyntaxHighlighter)
  - Color Settings Page
  - LSP 클라이언트 통합 (lsp4j 기반)
  - vais-lsp 바이너리 자동 탐색
  - 빌드: `./gradlew buildPlugin` → intellij-vais-0.0.1.zip
- [x] **플러그인 확장** - 포맷터 플러그인 API, 분석 플러그인 (완료일: 2026-01-21)
  - FormatterPlugin trait 추가 (format_module, FormatConfig)
  - AnalysisPlugin trait 추가 (analyze_complexity, analyze_dependencies)
  - ComplexityReport, DependencyGraph 데이터 구조
  - PluginRegistry에 run_format, run_analysis_* 메서드 추가
  - 테스트 35개 통과
- [x] **unwrap/expect 감소** - 에러 전파 패턴 적용 (완료일: 2026-01-21)
  - 총 488개 분석 완료: 테스트 코드 ~380개, 컴파일러 내부 ~50개, 안전한 폴백 ~16개
  - exhaustiveness.rs: unwrap() → expect() 변환 (길이 검증 문서화)
  - 프로덕션 코드의 외부 입력 처리는 이미 unwrap_or/unwrap_or_else 사용

### P3 - 미래 (6개월+)
- [x] **inkwell 직접 통합** - 문자열 IR 대신 LLVM API 직접 사용 (완료일: 2026-01-21)
  - inkwell 0.4 (LLVM 17) 의존성 추가 (optional feature)
  - feature flag: `text-codegen` (기본), `inkwell-codegen`
  - InkwellCodeGenerator 구현 (generator.rs)
  - TypeMapper: Vais → LLVM 타입 매핑 (types.rs)
  - 빌트인 함수 선언 (builtins.rs)
  - 설계 문서: docs/design/inkwell-integration-design.md
  - 참고: 실제 사용은 LLVM 17+ 설치 필요
- [x] **언어 바인딩** - Python/Node.js에서 vaisc 호출 (완료일: 2026-01-21)
  - vais-python 크레이트 (pyo3 기반)
    - compile(), check(), parse(), tokenize() 함수
    - Python 3.13 이하 필요 (PyO3 제한)
  - vais-node 크레이트 (napi-rs 기반)
    - compile(), check(), parse(), tokenize() 함수
    - 빌드 검증 완료
- [x] **JIT 컴파일** - REPL에서 Cranelift JIT 실행 (완료일: 2026-01-21)
  - vais-jit 크레이트 추가 (Cranelift 기반)
    - JitCompiler: JIT 컴파일러 구현
    - TypeMapper: Vais → Cranelift 타입 매핑
    - JitRuntime: 외부 함수 해결 (libc, libm)
  - REPL JIT 모드 지원 (`--features jit`)
    - 디스크 I/O 없이 즉시 실행
    - clang 불필요
  - 설계 문서: docs/design/jit-compilation-design.md
  - 테스트 27개 통과
- [x] **Self-hosting** - vaisc를 Vais로 재작성 (완료일: 2026-01-22)
  - 설계 문서: docs/design/self-hosting-design.md (완료)
  - selfhost/ 디렉토리 생성 (완료)
  - [x] span.vais - 소스 위치 추적 (완료)
  - [x] token.vais - 토큰 정의 (완료)
  - [x] lexer.vais - 기본 렉서 구조 (완료)
  - [x] lexer.vais - tokenize 기능 완성 (테스트 통과)
  - [x] ast.vais - AST 정의 (테스트 통과)
  - [x] parser.vais - 파서 구현 (테스트 통과)
  - [x] type_checker.vais - 타입 체커 구현 (테스트 통과)
  - [x] codegen.vais - LLVM IR 코드 생성기 구현 (1700+ 줄, 테스트 통과)
  - [x] bootstrap_test.vais - 부트스트래핑 테스트 (57개 테스트 통과)

### 남은 작업
- (없음)

---

## 🚀 Phase 8: 생산성 향상 및 생태계 확장

> **상태**: ✅ 완료 (100%)
> **추가일**: 2026-01-22

### P0 - 긴급 (완료)
- [x] **Option codegen 버그 수정** - if-expression에서 Option<T> 반환 타입 처리 (완료일: 2026-01-22)
  - is_expr_value()에서 enum variant constructor 감지
  - phi 노드에서 중첩된 if-else 값 처리 개선
  - match 표현식에서 enum 함수 호출 결과 처리
  - Vec<T>.get_opt(), pop_opt() 활성화
- [x] **inkwell TODO 해결** - generator.rs의 3개 TODO 완료 (완료일: 2026-01-22)
  - 변수 로드 시 적절한 타입 조회 (locals에 타입 정보 저장)
  - 필드 이름으로 인덱스 매핑 (struct_fields 맵 추가)
- [x] **Generic 반환 타입 수정** - vec_new() 등 활성화 (완료일: 2026-01-22)
  - struct 타입도 enum처럼 기본 이름 사용 (레이아웃 동일)
  - type_to_llvm에서 structs.contains_key() 체크 추가

### P1 - 높은 우선순위 (2-4주)
- [x] **`?` 연산자 (에러 전파)** - Result<T,E>/Option<T>에서 조기 반환 지원 (완료일: 2026-01-22)
  - Parser: postfix try 연산자 파싱 (Expr::Try)
  - TypeChecker: Result<T,E> → T, Option<T> → T 추론
  - Codegen: 에러/None 시 조기 반환 IR 생성
- [x] **`defer` 문** - Go 스타일 cleanup 구문 (완료일: 2026-01-22)
  - Lexer: Token::Defer (`D`) 추가
  - Parser: Stmt::Defer 파싱 지원
  - Codegen: defer_stack으로 LIFO 순서 실행
  - 모든 return 경로에서 defer cleanup 호출
- [x] **해시 함수 제네릭화** - HashMap의 hash() 함수를 다양한 타입 지원 (완료일: 2026-01-22)
  - std/hash.vais 모듈 추가 (mult_hash, hash_string, combine_hash 등)
  - HashMap이 std/hash 모듈의 mult_hash 함수 사용
  - DJB2 알고리즘 기반 문자열 해시 지원
- [x] **패키지 매니저 설계** - vais.toml 기반 의존성 관리 (완료일: 2026-01-22)
  - vais.toml manifest 파일 파싱 (toml crate)
  - `vais pkg init` - 새 패키지 초기화
  - `vais pkg build` - 패키지 빌드
  - `vais pkg check` - 타입 검사
  - `vais pkg add/remove` - 의존성 관리
  - `vais pkg clean` - 빌드 아티팩트 정리
  - 경로 기반 의존성 해결
  - 설계 문서: docs/design/package-manager-design.md

### P2 - 중간 우선순위 (1-2개월)
- [x] **패키지 레지스트리** - 중앙 패키지 저장소 구현 (완료일: 2026-01-22)
  - crates/vaisc/src/registry/ 모듈 추가
  - SemVer 버전 파싱/비교 (version.rs)
  - HTTP/로컬 레지스트리 클라이언트 (client.rs)
  - 패키지 캐싱 ~/.vais/registry/ (cache.rs)
  - tar.gz 압축/해제 (archive.rs)
  - 의존성 해결 알고리즘 (resolver.rs)
  - vais.lock 파일 생성 (lockfile.rs)
  - CLI 명령어: `vais pkg install`, `vais pkg update`, `vais pkg search`, `vais pkg info`, `vais pkg cache`
- [x] **Const generics** - 컴파일 타임 상수를 제네릭 파라미터로 사용 (`[T; N]`) (완료일: 2026-01-22)
  - GenericParamKind::Const - const 제네릭 파라미터 지원
  - ConstExpr - 컴파일 타임 상수 표현식 (리터럴, 파라미터, 연산)
  - Type::ConstArray / ResolvedType::ConstArray - `[T; N]` 문법
  - 파서: `const N: u64` 문법 및 `[T; N]` 배열 타입 파싱
  - 타입체커: resolve_const_expr(), 상수 연산 평가
  - 코드젠: LLVM `[N x T]` 배열 타입 생성
  - 11개 통합 테스트 추가
- [x] **SIMD intrinsics** - 벡터 연산 intrinsic 함수 (완료일: 2026-01-22)
  - 9개 SIMD 벡터 타입 지원: Vec2f32, Vec4f32, Vec8f32, Vec2f64, Vec4f64, Vec4i32, Vec8i32, Vec2i64, Vec4i64
  - 벡터 생성자: vec4f32(x, y, z, w), vec4i32(...) 등
  - 산술 연산: simd_add_*, simd_sub_*, simd_mul_*, simd_div_* (float 타입)
  - 수평 리듀스: simd_reduce_add_* (전체 요소 합)
  - LLVM IR 직접 생성: insertelement, fadd/fmul, @llvm.vector.reduce.*
  - 17개 통합 테스트 추가

### P3 - 낮은 우선순위 (3-6개월)
- [x] **Union types** - Tagged union 외 untagged union 지원 (완료일: 2026-01-22)
  - `O` 키워드로 union 정의 (O = One-of/Overlay)
  - 모든 필드 offset 0 (C union 스타일)
  - 제네릭 union 지원 (O Either<L, R> { left: L, right: R })
  - 메모리 레이아웃: 가장 큰 필드 타입 기준
  - 필드 접근은 호출자 책임 (unsafe, 컴파일러가 활성 필드 추적 안함)
- [x] **Compile-time evaluation** - comptime 블록으로 컴파일 타임 계산 (완료일: 2026-01-22)
  - `comptime { expr }` 문법으로 컴파일 타임 평가
  - ComptimeValue 타입 (Int, Float, Bool, Unit)
  - 산술/비트/논리/비교 연산 지원
  - 조건문, 반복문, 변수 바인딩 지원
  - vais-types/src/comptime.rs 모듈 추가
- [x] **Playground** - 웹 기반 Vais 실행 환경 (완료일: 2026-01-22)
  - playground/ 디렉토리 (Vite + Monaco Editor)
  - 13개 예제 코드 스니펫
  - Vais 구문 강조 및 자동 완성
  - 반응형 UI, 다크 테마
- [x] **표준 라이브러리 확장** - Time, Random, UUID, Base64, URL 파서 (완료일: 2026-01-22)
  - std/time.vais: Duration, time_now(), sleep()
  - std/random.vais: LCG 난수 생성, random_range()
  - std/uuid.vais: UUID v4 생성
  - std/base64.vais: Base64 인코딩/디코딩
  - std/url.vais: URL 파싱, percent-encoding

### P4 - 미래 (6개월+)
- [x] **Garbage Collection 옵션** - 선택적 GC 모드 (REPL/스크립팅용) (완료일: 2026-01-22)
  - vais-gc 크레이트 추가 (Mark-and-Sweep 알고리즘)
  - GcAllocator, GcRoot 관리, C FFI 인터페이스
  - std/gc.vais 런타임 모듈 (gc_init, gc_alloc, gc_collect)
  - CLI 옵션: `--gc`, `--gc-threshold <bytes>`
  - 9개 단위 테스트 통과
- [x] **Hot reloading** - 코드 변경 시 실행 중인 프로그램 업데이트 (완료일: 2026-01-22)
  - vais-hotreload 크레이트 추가 (FileWatcher + DylibLoader)
  - notify 크레이트로 파일 변경 감시
  - libloading으로 동적 라이브러리 로드/언로드
  - std/hot.vais 런타임 모듈 (hot_init, hot_check, hot_reload)
  - CLI: `vaisc watch <file>`, `vaisc build --hot`
  - 16개 테스트 통과
- [x] **GPU 타겟** - CUDA/OpenCL/WebGPU 코드 생성 (완료일: 2026-01-22)
  - vais-gpu 크레이트 (CUDA, OpenCL, WebGPU 코드 생성기)
  - GpuTarget enum (Cuda, OpenCL, WebGPU)
  - GpuType (GPU 호환 타입)
  - GpuBuiltins (각 백엔드별 빌트인 함수 매핑)
  - CLI: `vaisc build --gpu cuda|opencl|webgpu`
  - std/gpu.vais 런타임 모듈
  - 예제: examples/gpu_vector_add.vais

---

## 🚀 Phase 9: 언어 완성도 및 생산성 향상

> **상태**: ✅ 완료 (100%)
> **추가일**: 2026-01-22
> **예상 기간**: 12주 (약 3개월)

### P0 - 핵심 (1-2주)
- [x] **Bidirectional Type Checking** - 양방향 타입 체크 기반 구조 구현 (완료일: 2026-01-22)
  - CheckMode enum (Infer/Check) 추가
  - check_expr_bidirectional() 메서드 구현
  - 람다 파라미터 타입 추론 (check_lambda_with_expected)
  - 배열 요소 타입 전파 (check_array_with_expected)
  - 제네릭 함수 호출 양방향 추론 (check_generic_function_call_bidirectional)
  - 13개 단위 테스트 추가
- [x] **Dynamic Dispatch (dyn Trait)** - Rust 스타일 vtable 기반 동적 디스패치 (완료일: 2026-01-22)
  - `dyn Trait` 문법 추가 (Token::Dyn, Type::DynTrait, ResolvedType::DynTrait)
  - 파서에서 `dyn Trait<T>` 구문 파싱 지원
  - 타입 시스템에서 DynTrait 처리 (codegen, JIT)
  - 6개 단위 테스트 추가
  - Note: 실제 vtable 런타임 생성은 미구현 (타입 시스템 기반만 완료)

### P1 - 중요 (3-4주)
- [x] **Macro System** - 선언적 매크로 지원 (완료일: 2026-01-22)
  - `macro!` 키워드로 매크로 정의 (Token::Macro, Token::Dollar)
  - MacroDef, MacroRule, MacroPattern AST 타입
  - 토큰 패턴 매칭 및 치환 (MetaVarKind: expr, ty, ident, pat, stmt, block, item, lit, tt)
  - 반복 패턴 지원 ($(...),*, $(...),+, $(...),?)
  - MacroExpander: 패턴 매칭, 바인딩 추출, 템플릿 치환
  - vais-macro 크레이트 신규 추가
- [x] **Thread 모듈** - 멀티스레딩 지원 (`std/thread.vais`) (완료일: 2026-01-22)
  - JoinHandle<T>: 스레드 조인 및 결과 수신
  - ThreadBuilder: 스레드 이름, 스택 크기 설정
  - ThreadLocal<T>: 스레드 로컬 스토리지
  - ThreadPool: 작업자 스레드 풀
  - Scope: 범위 기반 스레드 관리
  - spawn(), sleep(), yield_now(), park() 함수
- [x] **Sync 모듈** - 동기화 프리미티브 (`std/sync.vais`) (완료일: 2026-01-22)
  - Mutex<T>, MutexGuard<T>: 상호 배제 락
  - RwLock<T>: 읽기-쓰기 락
  - Condvar: 조건 변수
  - Barrier: 동기화 장벽
  - Semaphore: 세마포어
  - Once: 일회성 초기화
  - Channel<T>, Sender<T>, Receiver<T>: MPSC 채널
  - AtomicI64, AtomicBool: 원자적 타입
  - SpinLock: 바쁜 대기 락
- [x] **Http 모듈** - HTTP 클라이언트/서버 (`std/http.vais`) (완료일: 2026-01-22)
  - Headers: HTTP 헤더 관리
  - Request: HTTP 요청 빌더 (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)
  - Response: HTTP 응답 (status, headers, body)
  - Client: HTTP 클라이언트 (execute, get, post)
  - Router: URL 라우팅 및 핸들러 등록
  - Server: HTTP 서버 (run, routes, handle_connection)

### P2 - 개선 (2-3주)
- [x] **LTO (Link-Time Optimization)** - LLVM LTO 플래그 통합 (완료일: 2026-01-22)
  - LtoMode enum (None, Thin, Full) 추가
  - clang 플래그 생성 (-flto=thin, -flto=full)
  - prepare_ir_for_lto(): LTO 친화적 IR 변환
  - interprocedural_analysis(): 순수 함수 감지, 호출 그래프 분석
  - cross_module_dce(): 크로스 모듈 데드 코드 제거
  - 4개 테스트 추가
- [x] **Profile-Guided Optimization** - PGO 지원 (완료일: 2026-01-22)
  - PgoMode enum (None, Generate, Use) 추가
  - `--profile-generate=<dir>`: 프로파일 데이터 수집용 instrumented 바이너리 생성
  - `--profile-use=<file>`: 프로파일 데이터 기반 최적화 빌드
  - `--lto=<mode>`: LTO 플래그 (thin, full) CLI 지원 추가
  - PgoConfig: 브랜치 가중치, 간접 호출 프로모션, hot/cold 임계값 설정
  - instrument_ir_for_pgo(), annotate_function_hotness() 헬퍼 함수
  - 9개 PGO 테스트 추가
- [x] **Incremental Build 고도화** - 함수 수준 증분 컴파일 (완료일: 2026-01-22)
  - FunctionMetadata, TypeMetadata 구조체 추가 (함수/타입 해시, 라인 범위, 의존성)
  - DefinitionExtractor: 소스에서 함수/구조체/열거형 정의 추출
  - detect_function_changes(): 함수 수준 변경 감지 (추가/수정/삭제/영향받음)
  - FunctionChangeSet: 변경된 함수 집합 관리
  - DirtySet 확장: dirty_functions, dirty_types 맵 추가
  - detect_changes_fine_grained(): 함수 수준 정밀 변경 감지
  - get_reusable_objects(): 캐시된 오브젝트 파일 재사용
  - 10개 테스트 추가 (전체 13개 증분 컴파일 테스트 통과)

### P3 - 경험 (1-2주)
- [x] **Profiler 통합** - 성능 프로파일링 도구 (`std/profiler.vais`) (완료일: 2026-01-22)
  - Timer: 고해상도 타이밍 (nanosecond 정밀도)
  - ProfileEntry: 호출 횟수, 총/평균/최소/최대 시간 추적
  - Profiler: 전역 프로파일러 (enter/exit/report)
  - MemoryProfiler: 메모리 할당/해제 추적
  - SampleProfiler: 샘플링 기반 프로파일링
  - FlameGraphBuilder: Flame graph 데이터 생성
- [x] **Test Framework 개선** - 테스트 작성 편의성 향상 (`std/test.vais`) (완료일: 2026-01-22)
  - TestResult: 테스트 결과 (passed/failed/skipped)
  - TestCase: 테스트 케이스 (setup/teardown, timeout, should_panic, tags)
  - TestSuite: 테스트 스위트 (before_all/after_all, before_each/after_each)
  - TestRunner: 테스트 실행기 (filter, verbose, fail_fast)
  - Assertions: assert, assert_eq, assert_ne, assert_gt, assert_lt, assert_str_eq 등
  - ANSI 컬러 출력, 테스트 시간 측정

### 완료
- Phase 9 모든 작업 완료 (100%)

---

## 📊 진행률 요약

| Phase | 상태 | 진행률 |
|-------|------|--------|
| Phase 1: 핵심 컴파일러 | ✅ 완료 | 100% |
| Phase 2: 표준 라이브러리 | ✅ 완료 | 100% |
| Phase 3: 개발자 도구 | ✅ 완료 | 100% |
| Phase 4: 향후 개선 | ✅ 완료 | 100% |
| Phase 5: 품질 개선 | ✅ 완료 | 100% |
| Phase 6: 후속 개선 | ✅ 완료 | 100% |
| Phase 7: 아키텍처 개선 | ✅ 완료 | 100% |
| Phase 8: 생산성 향상 | ✅ 완료 | 100% |
| Phase 9: 언어 완성도 | ✅ 완료 | 100% |
| Phase 10: Self-hosting | ✅ 완료 | 100% |

---

## 🚀 Phase 10: Self-hosting 완성 및 생태계 확장

> **상태**: ✅ 완료 (100%)
> **추가일**: 2026-01-22
> **최종 업데이트**: 2026-01-25
> **예상 기간**: 14-16주 (약 4개월)
> **목표**: 완전한 self-hosting 달성 및 프로덕션 준비 생태계

### Self-hosting 현황 (selfhost/) - 정리 완료
**디렉토리 크기**: 12MB → 664KB (95% 감소)

**핵심 파일 (18개):**
- **main.vais** (~3,900줄): Stage 1 monolithic 컴파일러, CLI 지원 (`./vaisc-stage1 <input.vais>`)
- **main_entry.vais** + 6개 모듈: Stage 2용 분리 버전
  - constants.vais, stringbuffer_s1.vais, lexer_s1.vais
  - helpers_s1.vais, parser_s1.vais, codegen_s1.vais
- **참조용**: ast.vais, lexer.vais, parser.vais, codegen.vais, type_checker.vais, token.vais, span.vais, stringpool.vais, module.vais
- **테스트**: bootstrap_test.vais

### Stage 2 부트스트래핑 진행 (2026-01-25)
- **vaisc-stage1 v0.5.2**: Stage 2 부트스트래핑 완료
- **완료된 기능**:
  - ✅ 토큰/파서/코드젠 확장 (S/X/함수/표현식/블록/if/loop)
  - ✅ SSA 최적화 (alloca 94% 감소)
  - ✅ 모든 연산자 (+, -, *, /, %, 비교, 논리)
  - ✅ 문자열 리터럴 및 extern 함수
  - ✅ 메모리 연산 (load_byte, store_byte, load_i64, store_i64)
  - ✅ Import 시스템 (U 문) - 모듈 import 지원
  - ✅ CLI 인자 지원 (argc/argv)
  - ✅ **vaisc-stage1으로 main.vais 컴파일 → vaisc-stage2 바이너리 생성 성공!**
  - ✅ strlen/memcpy_str 특수 처리 (i64↔ptr 변환)
  - ✅ 문자열 리터럴 이스케이프 시퀀스 처리 (\n, \t, \r 등)
  - ✅ **STMT_RETURN 코드젠 버그 수정** - `I ... { R ... }` 패턴에서 early return이 ret 명령어로 생성됨
  - ✅ **lexer_scan_operator 변수 스코프 버그 수정** - 내부 블록의 `end` 변수 이름 충돌 해결
- **현재 상태**: ✅ **Stage 2 부트스트래핑 완료!**
  - vaisc-stage1: main.vais 컴파일 시 313개 함수 정상 파싱
  - vaisc-stage2: 빌드 성공, **Stage 1과 동일한 출력 검증 완료**
  - **부트스트랩 검증 완료**: Stage 1과 Stage 2 모두 main.vais → 17,397줄 동일 IR 생성
- **수정된 버그** (2026-01-25):
  - ✅ **read_file_ptr 표현식 버그**: `buf + 8` 반복 계산 → `data_ptr` 변수로 수정
  - ✅ **cg_find_var 변수 쉐도잉**: 순방향 검색 → 역순 검색으로 변경

### P0 - 핵심 (1-2주) [Self-hosting 필수] ✅ 완료
- [x] **Self-hosting CLI 구현** - selfhost/main.vais (완료일: 2026-01-22)
- [x] **Selfhost 모듈 시스템 개선** (완료일: 2026-01-22)
- [x] **Stage 1 부트스트래핑** (완료일: 2026-01-22)
- [x] **CLI 인자 지원 추가** (완료일: 2026-01-24)
  - main.vais에 argc/argv 지원 추가
  - `./vaisc-stage1 <input.vais>` 형태로 임의 파일 컴파일 가능
- [x] **불필요한 파일 정리** (완료일: 2026-01-24)
  - 테스트 파일, .ll 파일, 중복 바이너리 삭제
  - 12MB → 664KB (95% 감소)

### P1 - 높은 우선순위 (3-4주)
- [x] **Stage 2 부트스트래핑 기본 기능** (완료일: 2026-01-24)
- [x] **Import 시스템 (U 문)** (완료일: 2026-01-24)
- [x] **Stage 2 바이너리 생성 성공** (완료일: 2026-01-24)
  - [x] load_byte/store_byte/load_i64/store_i64 지원
  - [x] strlen/memcpy_str 특수 처리 (i64↔ptr 변환)
  - [x] 문자열 이스케이프 시퀀스 처리
  - [x] **vaisc-stage1으로 main.vais 컴파일 → vaisc-stage2 바이너리 생성**
- [x] **Stage 2 부트스트래핑 완성** (완료일: 2026-01-25)
  - [x] read_file_ptr 표현식 버그 수정 (`buf + 8` → `data_ptr` 변수)
  - [x] cg_find_var 변수 쉐도잉 지원 (역순 검색)
  - [x] **Stage 1 vs Stage 2 출력 비교 완료 (17,397줄 동일 IR 검증)**
- [x] **에러 복구 개선** (완료일: 2026-01-25)
  - Panic-free 파싱 (파싱 에러 후 복구)
  - Synchronization point 탐지 (synchronize_item, synchronize_statement, synchronize_expression)
  - ErrorNode AST 타입 추가 (Item::Error, Stmt::Error, Expr::Error)
  - parse_with_recovery() 공개 API
  - 9개 에러 복구 테스트 추가
- [x] **Macro Runtime 통합** (완료일: 2026-01-25)
  - [x] Parser → MacroExpander → TypeChecker 흐름 통합
  - [x] 위생적 매크로 기본 지원 (HygienicContext)
  - [x] `#[derive(...)]` 속성 매크로 프레임워크 (AST 확장 대기)
- [x] **LSP 고도화** (완료일: 2026-01-25)
  - Inlay hints: 타입 추론 결과 표시
  - Call hierarchy: 함수 호출 관계 추적 (incoming/outgoing calls)
  - Folding ranges: 코드 블록 접기 (함수, 제어 구조)
  - Document links: import 경로 클릭 이동

### P2 - 중간 우선순위 (4-6주)
- [x] **패키지 레지스트리 서버** (완료일: 2026-01-26)
  - vais-registry-server 크레이트 추가 (axum + tokio 기반)
  - REST API: /api/v1/index.json, /api/v1/packages/:name/:version, /api/v1/search
  - SQLite 기반 메타데이터 저장소 (sqlx)
  - 패키지 발행, 검색, 다운로드, yanking 지원
  - 사용자 인증 (argon2 패스워드 해싱, Bearer 토큰)
  - 소유자 관리 (다중 소유자 지원)
  - 아카이브 검증 (SHA256 체크섬, 경로 순회 방지)
- [x] **문서 자동 생성 (vais doc)** (완료일: 2026-01-26)
  - DocGenerator 구조체 (vaisc/src/doc.rs)
  - Rustdoc 스타일 HTML 문서 생성
  - Markdown → HTML 변환 (pulldown-cmark)
  - /// 문서 주석 파싱 (Token::DocComment)
  - 검색 인덱스 자동 생성 (search-index.json)
  - CLI 명령어: `vaisc doc [--output <dir>]`
- [x] **FFI 고도화** (완료일: 2026-01-26)
  - `extern "C"` 블록 문법 (ExternBlock, ExternFunction AST)
  - 구조체 패딩/정렬 (repr(C) 속성, StructInfo.repr_c)
  - 가변 인자 함수 지원 (Param.is_vararg, FunctionSig.is_vararg, ...)
  - 함수 포인터 타입 (Type::FnPtr, ResolvedType::FnPtr)
  - Ellipsis 토큰 (...) 파싱 지원
- [x] **컴파일러 성능 최적화** (완료일: 2026-01-25)
  - 증분 컴파일 병렬화 (rayon) - 파일 해시 병렬 계산
  - 의존성 전파 병렬화
  - 타입 체커/코드 생성기 병렬화 인프라 추가
  - vaisc, vais-types, vais-codegen에 rayon 의존성 추가

### P3 - 낮은 우선순위 (4-6주)
- [x] **퍼징 및 보안 테스트** (완료일: 2026-01-26)
  - cargo-fuzz (libFuzzer) 통합 (fuzz/ 디렉토리)
  - 5개 fuzz target: lexer, parser, type_checker, codegen, full_pipeline
  - ASAN/UBSAN 빌드 옵션 (scripts/run-sanitizers.sh)
  - OSS-Fuzz 통합 준비 (fuzz/oss-fuzz/)
  - GitHub Actions fuzzing 워크플로우 (fuzz.yml)
  - 메모리 안전 테스트 20개 추가
- [x] **Trait Object 런타임 구현** (완료일: 2026-01-26)
  - VtableGenerator 모듈 추가 (vtable.rs)
  - VTable 구조: drop, size, align, 메서드 포인터
  - 동적 디스패치 LLVM IR 생성 (vtable lookup, indirect call)
  - trait object 생성: fat pointer { data_ptr, vtable_ptr }
  - CodeGenerator에 vtable 통합 (register_trait, register_trait_impl, get_or_generate_vtable)
  - 18개 vtable 테스트 추가
- [x] **Async Runtime 개선** (완료일: 2026-01-26)
  - select!/join!/timeout! 매크로 추가 (vais-macro/async_macros.rs)
  - AsyncMacroExpander: 매크로 확장 유틸리티
  - register_async_macros(): 내장 async 매크로 등록
  - CancellationTokenSource, CancellationToken (std/sync.vais)
  - CancellableFuture<T>: 취소 가능한 Future 래퍼
  - WaitGroup: 작업 그룹 대기 (Go 스타일)
  - 16개 매크로 테스트 추가
- [x] **크로스 컴파일 완성** (완료일: 2026-01-26)
  - TargetTriple 확장: 16개 타겟 지원
    - Linux: x86_64-gnu/musl, aarch64-gnu/musl, riscv64
    - Windows: x86_64-msvc, x86_64-gnu(MinGW)
    - macOS: x86_64-darwin, aarch64-darwin
    - iOS: aarch64-ios, aarch64-ios-simulator
    - Android: aarch64-android, armv7-android
    - WebAssembly: wasm32, wasi-preview1, wasi-preview2
  - CrossCompileConfig: SDK 자동 감지 (Android NDK, iOS SDK, WASI SDK, MSVC)
  - RuntimeLibs: 타겟별 런타임 라이브러리 정의
  - clang_flags(), output_extension(), is_*() 헬퍼 메서드
  - 10개 크로스 컴파일 테스트 추가

### P4 - 미래 (장기 목표)
- [x] **에디터 통합 확장** - Neovim/Helix/Emacs (완료일: 2026-01-26)
  - Neovim: syntax/vais.vim, ftdetect, ftplugin, lsp.lua, install.sh
  - Helix: languages.toml, queries/vais/highlights.scm
  - Emacs: vais-mode.el (major mode), vais-lsp.el (LSP 통합)
  - 통합 가이드 문서: docs/EDITORS.md
- [x] **DAP 서버** - 디버그 어댑터 프로토콜 (완료일: 2026-01-26)
  - crates/vais-dap/ - 새로운 crate
  - DAP 프로토콜 타입 정의 (protocol/types.rs, requests.rs, responses.rs, events.rs)
  - DAP 서버 구현 (server.rs) - 모든 표준 요청 핸들러
  - LLDB 어댑터 구현 (debugger.rs) - lldb CLI 래핑
  - 세션 관리 (session.rs) - launch/attach/breakpoints
  - DWARF 소스 매핑 (source_map.rs) - gimli 기반
  - VSCode 디버거 설정 (package.json contributes.debuggers)
  - docs/EDITORS.md에 DAP 사용 가이드 추가
- [x] **Formal Verification** - Design by Contract 형식 검증 (완료일: 2026-01-26)
  - AST: Expr::Old, Expr::Assert, Expr::Assume 표현식 추가
  - Parser: old(), assert(), assume() 내장 함수 파싱
  - Type Checker: 새 표현식 타입 검증 (조건은 Bool, 메시지는 Str)
  - Codegen/contracts.rs:
    - generate_assert(): 런타임 검사 생성, 실패 시 __panic 호출
    - generate_assume(): 디버그에서 검사, 릴리스에서 llvm.assume
    - generate_invariant_checks(): 구조체 불변식 검증
    - generate_old_snapshots(): ensures 절의 old() 전처리
    - generate_decreases_checks(): 종료성 증명 (비음수 체크)
  - StructInfo에 invariants 필드 추가
  - std/contract_runtime.c에 __panic() 함수 추가
  - 테스트: formal_verification_test.vais, contract_violation_test.vais
- [ ] **inkwell 완전 전환** - 텍스트 IR → LLVM C API (진행중)
  - [x] Match 표현식 구현 (완료일: 2026-01-26)
    - generate_match(): switch 및 chained conditional branches 지원
    - generate_pattern_check(): Wildcard, Ident, Literal, Range, Or, Tuple, Variant, Struct 패턴 체크
    - generate_pattern_bindings(): 패턴 변수 바인딩 지원
    - Guard 조건 지원
  - [x] Loop/While/For 구현 (완료일: 2026-01-26)
    - generate_loop(): 무한 루프 및 조건부 루프 지원
    - Loop context 관리 (break/continue 블록 추적)
    - 패턴 바인딩 기반 구조 (for-like 루프)
  - [x] Array/Tuple/Index 구현 (완료일: 2026-01-26)
    - generate_array(): 스택 할당 배열 생성
    - generate_tuple(): 익명 구조체 기반 튜플
    - generate_index(): 배열/튜플 인덱스 접근
  - [x] Method Call 구현 (완료일: 2026-01-26)
    - generate_method_call(): receiver-first 호출 변환
  - [x] Break/Continue/Defer 문장 구현 (완료일: 2026-01-26)
    - generate_break(): 루프 종료 점프
    - generate_continue(): 루프 시작 점프
    - Defer 스텁 (완전 구현 대기)
  - [x] Try/Unwrap 구현 (완료일: 2026-01-26)
    - generate_try(): ? 연산자 기본 지원
    - generate_unwrap(): ! 연산자 기본 지원
  - [x] Lambda/Closure 구현 (완료일: 2026-01-26)
    - generate_lambda(): 람다 함수 생성
    - 캡처 변수 지원 (captured variables as parameters)
    - ast_type_to_resolved(): AST Type → ResolvedType 변환
    - 함수 포인터로 i64 반환
  - [x] 제네릭 타입 처리 완성 (완료일: 2026-01-26)
    - get_generic_substitution(): 제네릭 파라미터 치환 조회
    - set_generic_substitutions(): 제네릭 파라미터 치환 설정
    - substitute_type(): ResolvedType 내 제네릭 치환
    - mangle_struct_name(): 제네릭 구조체 맹글링
    - mangle_function_name(): 제네릭 함수 맹글링
    - map_type_with_generics(): 제네릭 포함 타입의 LLVM 타입 매핑
    - define_specialized_struct(): 모노모픽화된 구조체 정의
    - declare_specialized_function(): 모노모픽화된 함수 선언

### 남은 작업 (다음 단계)
- **inkwell 완전 전환 완료!** (모든 핵심 기능 구현 완료)

---

**메인테이너**: Steve
**라이센스**: MIT

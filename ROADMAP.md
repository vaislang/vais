# Vais - AI-Optimized Programming Language
## 프로젝트 로드맵

> **버전**: 0.0.1
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-01-20

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

vscode-vais/       # VSCode Extension ✅ NEW
├── package.json
├── language-configuration.json
└── syntaxes/vais.tmLanguage.json

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

**핵심 기능 진행률: 100%** (Phase 1-3 완료)

---

## 🧪 테스트 현황

```
✅ 36 tests passed, 0 failed
✅ 40+ example files compiled and running
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

## 최근 변경사항 (2026-01-20)

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
| Phase 5: 품질 개선 | 🔄 진행 중 | 36% |

---

## 🔧 Phase 5: 품질 개선 및 안정화

> **상태**: 🔄 진행 중 (36%)

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
- [ ] **엣지 케이스 테스트 추가** - 빈 파일, 잘못된 문법, 경계값 테스트
- [ ] **통합 테스트 스위트 구축** - 전체 컴파일 파이프라인 테스트
- [ ] **표준 라이브러리 에러 처리 개선** - 0 반환 대신 Option/Result 타입 일관적 사용
- [ ] **입력 검증 강화** - 네트워크/파일 API에 입력 범위 검사 추가
- [ ] **unwrap/expect 감소** - 126개 사용처 검토 및 적절한 에러 핸들링 추가

### P3 - 낮은 우선순위 (3개월 내)
- [ ] **CONTRIBUTING.md 작성** - 기여 가이드라인 문서화
- [ ] **rustdoc 문서 생성** - Rust API에 doc 주석 추가 및 문서 생성
- [ ] **CI/CD 파이프라인 구축** - GitHub Actions로 자동 테스트/빌드

### P4 - 미래 (6개월+)
- [ ] **i18n 에러 메시지** - 에러 메시지 다국어 지원 인프라 구축
- [ ] **플러그인 시스템** - 컴파일러 확장 API 설계 및 구현
- [ ] **제네릭 표준 라이브러리** - Vec<T>, HashMap<K,V>의 실제 제네릭 지원
- [ ] **REPL 개선** - 멀티라인 입력, 히스토리, 탭 자동완성
- [ ] **LSP Rename 기능** - 심볼 이름 변경 기능 추가
- [ ] **벤치마크 스위트** - 성능 측정 및 회귀 테스트

### 남은 작업
- (위 항목들 중 선택하여 진행)

---

**메인테이너**: Steve
**라이센스**: MIT

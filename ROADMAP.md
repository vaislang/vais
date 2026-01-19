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

## 📊 진행률 요약

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

**전체 진행률: 100%**

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

## 🎯 다음 목표

모든 주요 기능이 완료되었습니다! 향후 개선 사항:
- 디버거 표현식 레벨 위치 정보 개선
- IPv6 지원
- 추가 표준 라이브러리 모듈

---

**메인테이너**: Steve
**라이센스**: MIT

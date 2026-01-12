# AOEL Development Roadmap

## Project Overview

**AOEL (AI-Optimized Executable Language)**
AI가 가장 효율적으로 생성, 수정, 실행할 수 있는 프로그래밍 언어

---

## Current Status

| Phase | Status | Progress |
|-------|--------|----------|
| Phase 0: Prototype (Python) | **DONE** | 100% |
| Phase 1: Foundation (Rust) | **IN PROGRESS** | 90% |
| Phase 2: Execution | NOT STARTED | 0% |
| Phase 3: Optimization | NOT STARTED | 0% |
| Phase 4: Native Compile | NOT STARTED | 0% |
| Phase 5: Ecosystem | NOT STARTED | 0% |

**Last Updated:** 2026-01-12

---

## Phase 0: Prototype (Python) - COMPLETED

Python으로 개념 검증 완료

- [x] 문법 스펙 v0.2
- [x] Lexer (토큰화)
- [x] Parser (구문 분석)
- [x] AST 정의
- [x] Validator (의미 검증)
- [x] 10개 예제 파싱 성공

**Location:** `/parser/` (Python prototype)

---

## Phase 1: Foundation (Rust) - IN PROGRESS

Rust로 컴파일러 프론트엔드 구현

### Tasks

- [x] **1.1 Project Setup**
  - [x] Cargo workspace 구성
  - [x] 모듈 구조 설계
  - [ ] CI/CD 설정

- [x] **1.2 Lexer** (`aoel-lexer` crate)
  - [x] Token 정의 (100+ 토큰 타입)
  - [x] Lexer 구현 (logos 기반)
  - [x] 에러 처리 (위치 정보 포함, ariadne)
  - [x] 테스트

- [x] **1.3 AST** (`aoel-ast` crate)
  - [x] 노드 타입 정의 (Unit, Block, Expr, Type)
  - [x] Visitor 패턴
  - [ ] Pretty printer

- [x] **1.4 Parser** (`aoel-parser` crate)
  - [x] Recursive descent parser
  - [x] 모든 블록 파싱 (META, INPUT, OUTPUT, INTENT, CONSTRAINT, FLOW, EXECUTION, VERIFY)
  - [x] 에러 처리 (ariadne 리포트)
  - [x] 통합 테스트 (12개 테스트 케이스)

- [x] **1.5 CLI** (`aoel-cli` crate)
  - [x] `aoel check <file>` - 파일 파싱 및 검증
  - [x] `aoel ast <file>` - AST 출력
  - [x] `aoel tokens <file>` - 토큰 목록 출력

- [x] **1.6 Type Checker** (`aoel-typeck` crate) - NEW!
  - [x] 타입 정의 (aoel-ast에서 완료)
  - [x] 타입 체커 구현
  - [x] 타입 추론 (표현식)
  - [x] CLI 통합 (`aoel check`에 타입 체크 추가)
  - [x] 테스트 (11개 테스트 케이스)

- [x] **1.7 Semantic Analysis** (aoel-typeck에 포함)
  - [x] 심볼 테이블 (INPUT/OUTPUT 필드, FLOW 노드)
  - [x] 스코프 분석 (Input, Output, Flow 스코프)
  - [x] 참조 해결 (input.*, output.*, 노드 참조)
  - [x] 제약 조건 검증 (CONSTRAINT/VERIFY가 BOOL인지 확인)
  - [x] FLOW 에지 유효성 검증
  - [x] 빌트인 함수 지원 (LEN, SUM, COUNT 등)

- [ ] **1.8 CI/CD**
  - [ ] GitHub Actions 설정
  - [ ] 자동 테스트
  - [ ] 릴리스 자동화

### Completed Files

```
aoel-rs/
├── Cargo.toml                    # Workspace 설정
├── crates/
│   ├── aoel-lexer/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── token.rs          # 100+ 토큰 정의
│   │       ├── lexer.rs          # 렉서 구현
│   │       ├── error.rs          # 에러 타입
│   │       └── tests.rs          # 단위 테스트 (12개)
│   │
│   ├── aoel-ast/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs          # 타입 시스템
│   │       ├── expr.rs           # 표현식 AST
│   │       ├── stmt.rs           # 블록/문장 AST
│   │       ├── unit.rs           # Unit (최상위 노드)
│   │       └── visitor.rs        # Visitor 패턴
│   │
│   ├── aoel-parser/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── parser.rs         # 재귀 하향 파서 (1500+ 라인)
│   │   │   └── error.rs          # 파서 에러
│   │   └── tests/
│   │       └── integration_tests.rs  # 12개 통합 테스트
│   │
│   ├── aoel-cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs           # CLI (check, ast, tokens 명령)
│   │
│   ├── aoel-typeck/              # NEW! 타입 체커
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # 공개 API: check()
│   │       ├── error.rs          # TypeCheckError (14개 에러 타입)
│   │       ├── symbol.rs         # SymbolTable, Symbol, ScopeLevel
│   │       ├── types.rs          # 타입 유틸리티
│   │       ├── infer.rs          # 표현식 타입 추론
│   │       ├── checker.rs        # TypeChecker 메인 로직
│   │       └── tests.rs          # 단위 테스트 (11개)
│   │
│   ├── aoel-ir/                  # Phase 2 placeholder
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   │
│   └── aoel-vm/                  # Phase 2 placeholder
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
```

### Deliverables
- `aoel` CLI로 `.aoel` 파일 파싱 및 타입 체크 가능
- 명확한 에러 메시지 (ariadne 기반)

---

## Phase 2: Execution

AOEL 코드 실행 환경 구현

### Tasks

- [ ] **2.1 AOEL IR (Intermediate Representation)**
  - [ ] IR 명세 설계
  - [ ] AST → IR 변환
  - [ ] IR 직렬화/역직렬화

- [ ] **2.2 Virtual Machine**
  - [ ] 스택 기반 VM
  - [ ] 명령어 세트 정의
  - [ ] 메모리 모델
  - [ ] FLOW 그래프 실행 엔진

- [ ] **2.3 Built-in Operations**
  - [ ] 산술 연산
  - [ ] 문자열 처리
  - [ ] 컬렉션 (Array, Map)
  - [ ] FLOW 연산 (MAP, FILTER, REDUCE, etc.)

- [ ] **2.4 Runtime**
  - [ ] 메모리 관리 (GC 또는 RC)
  - [ ] 에러 처리
  - [ ] 스택 트레이스

### Deliverables
- `aoel run example.aoel`로 실행 가능
- 모든 예제 실행 성공

---

## Phase 3: Optimization

성능 최적화

### Tasks

- [ ] **3.1 IR Optimization Passes**
  - [ ] 상수 폴딩 (Constant Folding)
  - [ ] 상수 전파 (Constant Propagation)
  - [ ] 데드 코드 제거 (Dead Code Elimination)
  - [ ] 공통 부분식 제거 (CSE)

- [ ] **3.2 FLOW Optimization**
  - [ ] 노드 인라이닝
  - [ ] 병렬화 분석
  - [ ] 파이프라인 최적화

- [ ] **3.3 VM Optimization**
  - [ ] 명령어 융합
  - [ ] 캐싱
  - [ ] (선택) JIT 컴파일

### Deliverables
- Python 프로토타입 대비 10배+ 성능 향상

---

## Phase 4: Native Compile

네이티브 바이너리 생성

### Tasks

- [ ] **4.1 Backend Selection**
  - [ ] LLVM vs Cranelift 평가
  - [ ] 백엔드 선택 및 통합

- [ ] **4.2 Code Generation**
  - [ ] IR → LLVM IR / Cranelift IR
  - [ ] 함수 생성
  - [ ] 메모리 레이아웃

- [ ] **4.3 AOT Compilation**
  - [ ] 실행 파일 생성
  - [ ] 크로스 컴파일 지원
  - [ ] 링킹

- [ ] **4.4 WASM Target**
  - [ ] WASM 백엔드
  - [ ] 웹 런타임

### Deliverables
- `aoel build example.aoel -o example` → 네이티브 바이너리
- `aoel build example.aoel --target wasm` → .wasm 파일

---

## Phase 5: Ecosystem

실제 사용 가능한 생태계

### Tasks

- [ ] **5.1 Standard Library**
  - [ ] I/O
  - [ ] 네트워킹
  - [ ] JSON/데이터 처리
  - [ ] 시간/날짜
  - [ ] 암호화

- [ ] **5.2 Package Manager (aoelpm)**
  - [ ] 패키지 포맷
  - [ ] 레지스트리
  - [ ] 의존성 해결
  - [ ] 버전 관리

- [ ] **5.3 FFI (Foreign Function Interface)**
  - [ ] C 바인딩
  - [ ] Rust 바인딩
  - [ ] Python 바인딩

- [ ] **5.4 Developer Tools**
  - [ ] LSP (Language Server Protocol)
  - [ ] 디버거
  - [ ] 프로파일러
  - [ ] REPL
  - [ ] Formatter

- [ ] **5.5 Documentation**
  - [ ] 언어 스펙 문서
  - [ ] 튜토리얼
  - [ ] API 문서
  - [ ] 예제 프로젝트

### Deliverables
- 완전한 프로그래밍 언어 생태계

---

## Directory Structure

```
aoel/
├── ROADMAP.md              # 이 파일
├── AOEL_MASTER_SPEC.md     # 원본 스펙
├── AOEL_GRAMMAR_SPEC_v0.2.md  # 문법 스펙
│
├── parser/                 # Python 프로토타입 (Phase 0)
│   ├── lexer.py
│   ├── parser.py
│   ├── ast_nodes.py
│   └── validator.py
│
├── examples/               # AOEL 예제 파일
│   ├── 01_hello_world.aoel
│   ├── 02_add_numbers.aoel
│   └── ...
│
├── aoel-rs/               # Rust 구현 (Phase 1+)
│   ├── Cargo.toml
│   ├── crates/
│   │   ├── aoel-lexer/    # 토큰화
│   │   ├── aoel-ast/      # AST 정의
│   │   ├── aoel-parser/   # 파서
│   │   ├── aoel-typeck/   # 타입 체커 (NEW!)
│   │   ├── aoel-ir/       # IR (Phase 2)
│   │   ├── aoel-vm/       # VM (Phase 2)
│   │   └── aoel-cli/      # CLI
│   └── tests/
│
└── docs/                  # 문서
    ├── spec/
    ├── tutorial/
    └── api/
```

---

## Change Log

### 2026-01-12
- **Phase 1 진행 (90%)**
  - `aoel-typeck` crate 추가 (타입 체커)
    - TypeCheckError: 14개 에러 타입 정의
    - SymbolTable: INPUT/OUTPUT 필드, FLOW 노드 관리
    - 스코프 분석: Input, Output, Flow 스코프
    - 참조 해결: input.*, output.*, 노드 참조 유효성 검증
    - 타입 추론: 리터럴, 연산자, 함수 호출 타입 추론
    - CONSTRAINT/VERIFY 표현식이 BOOL인지 검증
    - FLOW 에지 소스/타겟 유효성 검증
    - 빌트인 함수: LEN, SUM, COUNT, AVG, MIN, MAX, CONTAINS, MATCH 등
    - 11개 테스트 케이스 추가
  - CLI 업데이트
    - `aoel check`가 파싱 + 타입 체크를 모두 수행
  - 전체 테스트 통과 (37개)
    - Lexer: 12개
    - Parser: 14개 (단위 + 통합)
    - Typeck: 11개

### 2026-01-11 (Update 3)
- **Phase 1 진행 (75%)**
  - Rust 1.92.0 설치 및 빌드 환경 구성
  - `aoel-cli` crate 완성
    - `check`, `ast`, `tokens` 명령어 구현
    - 예제 파일 파싱 성공
  - `aoel-ir`, `aoel-vm` placeholder crate 생성
  - 파서 버그 수정
    - META 값에서 qualified name 지원 (예: `examples.basic`)
    - VOID 리터럴 지원
    - Goal spec 괄호 선택적 지원
    - OUTPUT/INPUT 키워드를 식별자로 사용 가능
  - 전체 테스트 통과 (26개)
    - Lexer: 12개
    - Parser: 2개 단위 테스트
    - Integration: 12개

### 2026-01-11 (Update 2)
- **Phase 1 진행 (60%)**
  - `aoel-lexer` crate 완성
    - 100+ 토큰 타입 정의 (logos 기반)
    - Span 및 SourceMap 구현
    - 에러 리포팅 (ariadne)
  - `aoel-ast` crate 완성
    - 완전한 타입 시스템 (Primitive, Array, Optional, Struct, Union)
    - 표현식 AST (Literal, Binary, Unary, FieldAccess, Index, FunctionCall, Quantifier)
    - 모든 블록 AST (Meta, Input, Output, Intent, Constraint, Flow, Execution, Verify)
    - Visitor 패턴
  - `aoel-parser` crate 완성
    - 재귀 하향 파서 (1500+ 라인)
    - 모든 블록 파싱 구현
    - ParseError 타입 및 ariadne 리포트
    - 12개 통합 테스트

### 2026-01-11
- Phase 0 (Python Prototype) 완료
- Phase 1 시작
- ROADMAP.md 생성

# AOEL Development Roadmap

## Project Overview

**AOEL (AI-Optimized Executable Language)**
AI가 가장 효율적으로 생성, 수정, 실행할 수 있는 프로그래밍 언어

---

## Current Status

| Phase | Status | Progress |
|-------|--------|----------|
| Phase 0: Prototype (Python) | **DONE** | 100% |
| Phase 1: Foundation (Rust) | **DONE** | 100% |
| Phase 2: Execution | **DONE** | 100% |
| Phase 3: Optimization | **IN PROGRESS** | 50% |
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

## Phase 1: Foundation (Rust) - COMPLETED

Rust로 컴파일러 프론트엔드 구현

### Tasks

- [x] **1.1 Project Setup**
  - [x] Cargo workspace 구성
  - [x] 모듈 구조 설계
  - [x] CI/CD 설정

- [x] **1.2 Lexer** (`aoel-lexer` crate)
  - [x] Token 정의 (100+ 토큰 타입)
  - [x] Lexer 구현 (logos 기반)
  - [x] 에러 처리 (위치 정보 포함, ariadne)
  - [x] 테스트 (12개)

- [x] **1.3 AST** (`aoel-ast` crate)
  - [x] 노드 타입 정의 (Unit, Block, Expr, Type)
  - [x] Visitor 패턴

- [x] **1.4 Parser** (`aoel-parser` crate)
  - [x] Recursive descent parser
  - [x] 모든 블록 파싱 (META, INPUT, OUTPUT, INTENT, CONSTRAINT, FLOW, EXECUTION, VERIFY)
  - [x] 에러 처리 (ariadne 리포트)
  - [x] 통합 테스트 (12개 테스트 케이스)

- [x] **1.5 CLI** (`aoel-cli` crate)
  - [x] `aoel check <file>` - 파일 파싱 및 검증
  - [x] `aoel ast <file>` - AST 출력
  - [x] `aoel tokens <file>` - 토큰 목록 출력

- [x] **1.6 Type Checker** (`aoel-typeck` crate)
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

- [x] **1.8 CI/CD**
  - [x] GitHub Actions 설정 (`.github/workflows/ci.yml`)
  - [x] 자동 테스트 (push/PR 시 테스트, lint, format 체크)
  - [x] 릴리스 자동화 (`.github/workflows/release.yml`)
  - [x] Dependabot 설정 (`.github/dependabot.yml`)

### Deliverables
- `aoel` CLI로 `.aoel` 파일 파싱 및 타입 체크 가능
- 명확한 에러 메시지 (ariadne 기반)

---

## Phase 2: Execution - COMPLETED

AOEL 코드 실행 환경 구현

### Tasks

- [x] **2.1 AOEL IR (Intermediate Representation)** (`aoel-ir` crate)
  - [x] IR 명세 설계
    - Value 타입: Void, Bool, Int, Float, String, Bytes, Array, Map, Struct, Optional, Error
    - OpCode: 40+ 명령어 (스택, 변수, 산술, 비교, 논리, 컬렉션, 제어 흐름, 빌트인)
    - NodeIR/EdgeIR: FLOW 그래프 표현
    - NodeOpType: Transform, Map, Filter, Reduce, Branch, Merge, Fetch, Store, Validate
  - [x] AST → IR 변환 (lowering.rs)
  - [x] IR 직렬화/역직렬화 (serde JSON)
  - [x] 테스트 (5개)

- [x] **2.2 Virtual Machine** (`aoel-vm` crate)
  - [x] 스택 기반 VM
  - [x] 명령어 세트 정의 (40+ opcodes)
  - [x] 메모리 모델 (Runtime: stack, locals, inputs, outputs)
  - [x] FLOW 그래프 실행 엔진
    - 위상 정렬 기반 노드 실행
    - Map/Filter/Reduce 지원
  - [x] 테스트 (15개)

- [x] **2.3 Built-in Operations** (builtins.rs)
  - [x] 산술 연산 (Add, Sub, Mul, Div, Neg, ABS, MIN, MAX)
  - [x] 문자열 처리 (LEN, UPPER, LOWER, TRIM, CONTAINS, STARTS_WITH, ENDS_WITH, CONCAT)
  - [x] 컬렉션 (FIRST, LAST, REVERSE, FLATTEN, SUM, AVG, COUNT)
  - [x] FLOW 연산 (MAP, FILTER, REDUCE)
  - [x] 논리 연산 (IN, MATCH, XOR, IMPLIES)
  - [x] 타입 변환 (TO_STRING, TO_INT, TO_FLOAT)

- [x] **2.4 Runtime** (runtime.rs)
  - [x] 스택 관리
  - [x] 에러 처리 (RuntimeError: 12가지 에러 타입)
  - [x] 입출력 관리

- [x] **2.5 CLI 확장**
  - [x] `aoel compile <file>` - IR로 컴파일 (JSON 출력)
  - [x] `aoel run <file> --input '{json}'` - 실행

### Completed Files

```
aoel-rs/crates/
├── aoel-ir/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # 공개 API
│       ├── value.rs            # 런타임 값 타입
│       ├── instruction.rs      # OpCode, NodeIR, EdgeIR
│       ├── module.rs           # Module, Function
│       └── lowering.rs         # AST → IR 변환
│
└── aoel-vm/
    ├── Cargo.toml
    └── src/
        ├── lib.rs              # 공개 API: execute()
        ├── error.rs            # RuntimeError (12가지 에러)
        ├── runtime.rs          # 런타임 상태 관리
        ├── builtins.rs         # 30+ 빌트인 함수
        └── vm.rs               # VM 실행 엔진
```

### Deliverables
- `aoel run example.aoel`로 실행 가능
- `aoel compile example.aoel`로 IR JSON 출력

---

## Phase 3: Optimization - IN PROGRESS

성능 최적화

### Tasks

- [x] **3.1 IR Optimization Passes** (`aoel-ir/src/optimize.rs`)
  - [x] 상수 폴딩 (Constant Folding)
    - 컴파일 타임에 상수 표현식 평가
    - 산술, 비교, 논리, 문자열 연결 지원
  - [x] 데드 코드 제거 (Dead Code Elimination)
    - NOP 제거
    - 불필요한 push-pop 패턴 제거
    - Dup-Pop 패턴 제거
  - [x] CLI 통합: `aoel compile -O <level>`
    - `-O0`: 최적화 없음
    - `-O1`: 기본 최적화 (기본값)
    - `-O2`: 공격적 최적화
  - [ ] 상수 전파 (Constant Propagation)
  - [ ] 공통 부분식 제거 (CSE)

- [ ] **3.2 FLOW Optimization**
  - [ ] 노드 인라이닝
  - [ ] 병렬화 분석
  - [ ] 파이프라인 최적화

- [ ] **3.3 VM Optimization**
  - [ ] 명령어 융합
  - [ ] 캐싱
  - [ ] (선택) JIT 컴파일

### Completed Files

```
aoel-rs/crates/aoel-ir/src/
├── optimize.rs            # 최적화 패스
│   ├── constant_folding() # 상수 폴딩
│   ├── dead_code_elimination() # 데드 코드 제거
│   └── OptLevel enum      # 최적화 레벨
```

### Deliverables
- [x] `aoel compile -O1 example.aoel`로 최적화된 IR 생성
- [ ] Python 프로토타입 대비 10배+ 성능 향상

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
├── aoel-rs/               # Rust 구현 (Phase 1-2)
│   ├── Cargo.toml
│   ├── crates/
│   │   ├── aoel-lexer/    # 토큰화
│   │   ├── aoel-ast/      # AST 정의
│   │   ├── aoel-parser/   # 파서
│   │   ├── aoel-typeck/   # 타입 체커
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

## Test Summary

| Crate | Tests | Description |
|-------|-------|-------------|
| aoel-lexer | 13 | 토큰화 테스트 |
| aoel-parser | 14 | 파싱 테스트 (단위 + 통합) |
| aoel-typeck | 11 | 타입 체크 테스트 |
| aoel-ir | 13 | IR 변환 + 최적화 테스트 |
| aoel-vm | 15 | VM 실행 테스트 |
| **Total** | **66** | |

---

## Change Log

### 2026-01-12 (Update 5)
- **파서 수정 - 모든 예제 파싱 성공**
  - Lexer 수정 (`aoel-lexer/src/token.rs`)
    - Slash(`/`) 연산자와 Regex 리터럴 충돌 해결
    - Regex 패턴 임시 비활성화 (Slash 우선)
    - Slash 연산자 테스트 추가
  - 파서 수정 (`aoel-parser/src/parser.rs`)
    - `REQUIRE WITHIN 10s` 문법 지원 (CONSTRAINT 블록)
    - Duration 리터럴 표현식 파싱 지원 (`5s`, `10m`, `100ms`)
    - Size 리터럴 표현식 파싱 지원 (`256MB`, `1GB`)
    - EDGE 파라미터 `(key=value)` 문법 지원
    - `parse_edge_target_expr()` 함수 추가 (함수 호출과 edge params 구분)
  - AST 수정 (`aoel-ast/src/stmt.rs`)
    - `FlowEdge`에 `params: Vec<NodeParam>` 필드 추가
  - IR 수정 (`aoel-ir/src/lowering.rs`)
    - `FlowEdge` 생성 시 `params` 필드 추가
  - 예제 수정 (`examples/08_validate_email.aoel`)
    - Regex 리터럴을 문자열로 변경 (임시)
  - **예제 파싱 결과**
    - 파싱 + 타입체크 통과: 01, 02, 03, 06 (4개)
    - 파싱 통과, 타입체크 실패: 04, 05, 07, 08, 09, 10 (6개)
    - 모든 10개 예제 파싱 성공!
  - 전체 테스트 통과 (66개)

### 2026-01-12 (Update 4)
- **Phase 3 진행 (50%)**
  - IR 최적화 패스 구현 (`aoel-ir/src/optimize.rs`)
    - 상수 폴딩 (Constant Folding)
      - 산술 연산: Add, Sub, Mul, Div
      - 비교 연산: Eq, Neq, Lt, Gt, Lte, Gte
      - 논리 연산: And, Or, Not
      - 문자열 연결
      - 단항 연산: Neg, Not
    - 데드 코드 제거 (Dead Code Elimination)
      - NOP 명령어 제거
      - Const-Pop 패턴 제거
      - Dup-Pop 패턴 제거
    - 7개 테스트 추가
  - CLI 최적화 옵션 추가
    - `aoel compile -O0/1/2 <file>`: 최적화 레벨 지정
  - lowering 로직 개선
    - TRANSFORM 노드의 value 파라미터 → OUTPUT 포트 연결
    - TRANSFORM 노드의 op/left/right 패턴 지원 (ADD, SUB, MUL, DIV)
  - GOAL 구문 파서 수정
    - 괄호 없이 쉼표로 구분된 여러 입력 지원
    - `GOAL TRANSFORM: input.a, input.b -> output.sum`
  - 타입 체커 개선
    - ADD, SUB, MUL, DIV 등 연산자 키워드를 특수 처리
  - 예제 실행 테스트
    - `01_hello_world.aoel`: ✓ 실행 성공
    - `02_add_numbers.aoel`: ✓ 실행 성공
  - 전체 테스트 통과 (65개)

### 2026-01-12 (Update 3)
- **Phase 2 완료 (100%)**
  - `aoel-ir` crate 구현
    - Value: 11가지 런타임 값 타입
    - OpCode: 40+ VM 명령어
    - NodeIR/EdgeIR: FLOW 그래프 IR 표현
    - Module/Function: IR 모듈 구조
    - lowering.rs: AST → IR 변환
    - 5개 테스트
  - `aoel-vm` crate 구현
    - 스택 기반 VM 실행 엔진
    - Runtime: 스택, 로컬 변수, 입출력 관리
    - 30+ 빌트인 함수 (LEN, SUM, UPPER, CONTAINS 등)
    - Map/Filter/Reduce 지원
    - 15개 테스트
  - CLI 확장
    - `aoel compile <file>` - IR JSON 출력
    - `aoel run <file> --input '{json}'` - 실행
  - 전체 테스트 통과 (57개)
    - Lexer: 12개
    - Parser: 14개
    - Typeck: 11개
    - IR: 5개
    - VM: 15개

### 2026-01-12 (Update 2)
- **Phase 1 완료 (100%)**
  - CI/CD 설정 완료
    - `.github/workflows/ci.yml`: 자동 테스트, lint, format 체크
    - `.github/workflows/release.yml`: 릴리스 자동화 (멀티 플랫폼 빌드)
    - `.github/dependabot.yml`: 의존성 자동 업데이트
  - Phase 1 (Foundation) 완료!

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

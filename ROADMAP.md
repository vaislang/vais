# AOEL Development Roadmap

## Project Overview

**AOEL (AI-Optimized Executable Language)**
AI가 가장 효율적으로 생성, 수정, 실행할 수 있는 프로그래밍 언어

---

## ⚠️ Architecture Update (2026-01-12)

**v6b 문법이 AOEL 메인 문법으로 통합되었습니다.**

기존 블록 기반 문법 (META, INPUT, OUTPUT, FLOW 등)에서 **함수형 스크립트 문법**으로 전환:

```aoel
// 기존 (블록 기반)
META { name: "add" }
INPUT { a: INT, b: INT }
OUTPUT { result: INT }
FLOW { ... }

// 현재 (함수형 - v6b 통합)
add(a, b) = a + b
```

### 주요 특징
- **함수형 스타일**: `name(params) = body`
- **컬렉션 연산자**: `.@` (map), `.?` (filter), `./` (reduce)
- **자기 재귀**: `$()` 연산자
- **Hindley-Milner 타입 추론**
- **모듈 시스템**: `use path.to.module`

---

## Current Status

| Phase | Status | Progress |
|-------|--------|----------|
| Phase 0: Prototype (Python) | **DONE** | 100% |
| Phase 1: Foundation (Rust) | **DONE** | 100% |
| Phase 2: Execution | **DONE** | 100% |
| Phase 3: Optimization | **DONE** | 100% |
| Phase 4: Native Compile | **IN PROGRESS** | 70% |
| Phase 5: Ecosystem | **IN PROGRESS** | 40% |

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
  - [x] 컬렉션 연산자 (.@, .?, ./, .&, .<>)
  - [x] 테스트 (11개)

- [x] **1.3 AST** (`aoel-ast` crate)
  - [x] 노드 타입 정의
  - [x] 함수형 AST (FunctionDef, Expr, Pattern)
  - [x] 컬렉션 연산 AST (MapOp, FilterOp, ReduceOp)

- [x] **1.4 Parser** (`aoel-parser` crate)
  - [x] Recursive descent parser
  - [x] 함수 정의 파싱: `name(params) = body`
  - [x] 컬렉션 연산 파싱
  - [x] Let 바인딩, Match 표현식
  - [x] 모듈 시스템 (`use`, `mod`)
  - [x] 테스트 (10개)

- [x] **1.5 CLI** (`aoel-cli` crate)
  - [x] `aoel check <file>` - 파일 파싱 및 타입 체크
  - [x] `aoel ast <file>` - AST 출력
  - [x] `aoel tokens <file>` - 토큰 목록 출력
  - [x] `aoel eval <expr>` - 표현식 평가
  - [x] `aoel repl` - 대화형 REPL

- [x] **1.6 Type Checker** (`aoel-typeck` crate)
  - [x] Hindley-Milner 타입 추론
  - [x] 타입 통일 (Unification)
  - [x] 재귀 함수 타입 추론
  - [x] 컬렉션 연산 타입 체크
  - [x] 테스트 (11개)

### Deliverables
- `aoel` CLI로 `.aoel` 파일 파싱 및 타입 체크 가능

---

## Phase 2: Execution - COMPLETED

AOEL 코드 실행 환경 구현

### Tasks

- [x] **2.1 AOEL IR** (`aoel-ir` crate)
  - [x] Value 타입: Void, Bool, Int, Float, String, Array, Map, Closure
  - [x] OpCode: 40+ 명령어
  - [x] AST → IR 변환 (lowering)
  - [x] 테스트 (8개)

- [x] **2.2 Virtual Machine** (`aoel-vm` crate)
  - [x] 스택 기반 VM
  - [x] 클로저 지원
  - [x] 컬렉션 연산 (Map, Filter, Reduce)
  - [x] 자기 재귀 ($) 지원
  - [x] 테스트 (6개)

- [x] **2.3 Built-in Functions** (50+ 함수)
  - [x] 컬렉션: len, first, last, reverse, concat, range
  - [x] 수학: sqrt, sin, cos, tan, log, pow, abs, min, max
  - [x] 문자열: upper, lower, trim, split, join, replace, substr
  - [x] 타입 변환: int, float, str, bool

- [x] **2.4 CLI 확장**
  - [x] `aoel run <file>` - 실행
  - [x] `aoel compile <file>` - IR 출력

### Deliverables
- `aoel run example.aoel`로 실행 가능

---

## Phase 3: Optimization - COMPLETED

성능 최적화

### Tasks

- [x] **3.1 IR Optimization Passes** (`aoel-ir/src/optimize.rs`)
  - [x] 상수 폴딩 (Constant Folding)
  - [x] 데드 코드 제거 (Dead Code Elimination)
  - [x] CLI 통합: `aoel compile -O <level>`
  - [x] 상수 전파 (Constant Propagation)
  - [x] 공통 부분식 제거 (CSE)

- [x] **3.2 VM Optimization**
  - [x] 명령어 융합 (Instruction Fusion)
  - [x] 꼬리 재귀 최적화 (TCO)
  - [x] TailSelfCall 명령어 및 VM 지원

- [x] **3.3 Type System Improvements**
  - [x] 배열 연결 타입 추론 수정
  - [x] 재귀 함수 타입 추론 개선

### Deliverables
- [x] `aoel compile -O1 example.aoel`로 최적화된 IR 생성
- [x] `aoel compile -O2 example.aoel`로 고급 최적화 (상수 전파, CSE, 명령어 융합, TCO)

---

## Phase 4: Native Compile - IN PROGRESS (70%)

네이티브 바이너리 생성

### Tasks

- [x] **4.1 C Code Generation** (`aoel-codegen` crate)
  - [x] IR → C 코드 변환
  - [x] 기본 함수 생성
  - [x] 배열/맵 지원
  - [x] 테스트 (5개)

- [x] **4.2 Build Command**
  - [x] `aoel build <file>` - 네이티브 바이너리 생성
  - [x] 시스템 C 컴파일러 (gcc/clang) 사용
  - [x] `--keep-c` 옵션
  - [x] `--target` 옵션 (c, wasm)

- [x] **4.3 WASM Target**
  - [x] WAT (WebAssembly Text Format) 생성
  - [x] 기본 연산 지원 (산술, 비교, 논리)
  - [x] 함수 호출 및 재귀 지원
  - [x] WASI import 지원 준비
  - [x] 테스트 (4개)

- [ ] **4.4 Advanced Backends (Future)**
  - [ ] LLVM 백엔드
  - [ ] Cranelift 백엔드

### Deliverables
- [x] `aoel build example.aoel -o example` → 네이티브 바이너리
- [x] `aoel build example.aoel --target wasm` → WAT/WASM 파일

---

## Phase 5: Ecosystem - IN PROGRESS (40%)

실제 사용 가능한 생태계

### Tasks

- [x] **5.1 Standard Library** (부분 완료)
  - [x] 기본 빌트인 (50+ 함수)
  - [x] 확장 배열 함수: push, pop, take, drop, zip, flatten, sort, unique, index_of
  - [x] 확장 수학 함수: exp, log2, asin, acos, atan, atan2, clamp
  - [x] 확장 문자열 함수: chars, pad_left, pad_right, repeat
  - [x] 타입 체크 함수: is_int, is_float, is_string, is_bool, is_array, is_map
  - [ ] I/O (파일 읽기/쓰기)
  - [ ] 네트워킹
  - [ ] JSON 파싱
  - [ ] 시간/날짜

- [x] **5.2 Module System**
  - [x] `use path.to.module` 문법
  - [x] `pub` 가시성
  - [x] 선택적 import: `use module.{a, b}`
  - [x] 순환 의존성 감지
  - [ ] 패키지 매니저

- [x] **5.3 Developer Tools** (부분 완료)
  - [x] LSP 기본 구현 (`aoel-lsp` crate)
  - [x] REPL (`aoel repl`)
  - [ ] 디버거
  - [ ] 프로파일러
  - [ ] Formatter

- [ ] **5.4 FFI**
  - [ ] C 바인딩
  - [ ] Rust 바인딩
  - [ ] Python 바인딩

### Deliverables
- [x] 모듈 import로 코드 재사용 가능
- [x] 50+ 빌트인 함수

---

## Directory Structure

```
aoel/
├── ROADMAP.md              # 이 파일
├── AOEL_MASTER_SPEC.md     # 원본 스펙
│
├── parser/                 # Python 프로토타입 (Phase 0, deprecated)
│
├── aoel-rs/                # Rust 구현 (메인)
│   ├── Cargo.toml
│   ├── crates/
│   │   ├── aoel-lexer/     # 토큰화
│   │   ├── aoel-ast/       # AST 정의
│   │   ├── aoel-parser/    # 파서 + 모듈 시스템
│   │   ├── aoel-typeck/    # Hindley-Milner 타입 체커
│   │   ├── aoel-ir/        # IR + 최적화
│   │   ├── aoel-lowering/  # AST → IR 변환
│   │   ├── aoel-vm/        # 스택 기반 VM
│   │   ├── aoel-codegen/   # C 코드 생성
│   │   ├── aoel-lsp/       # Language Server
│   │   └── aoel-cli/       # CLI
│   └── examples/
│       └── aoel/           # 예제 파일
│
└── docs/                   # 문서
```

---

## Test Summary

| Crate | Tests | Description |
|-------|-------|-------------|
| aoel-lexer | 11 | 토큰화 테스트 |
| aoel-parser | 10 | 파싱 + 모듈 테스트 |
| aoel-typeck | 11 | 타입 체크 테스트 |
| aoel-ir | 20 | IR 변환 + 최적화 테스트 (상수 전파, CSE, 명령어 융합, TCO 포함) |
| aoel-lowering | 3 | Lowering 테스트 |
| aoel-vm | 6 | VM 실행 테스트 (TCO 지원) |
| aoel-codegen | 9 | C/WASM 코드 생성 테스트 (5 C + 4 WASM) |
| **Total** | **71** | |

---

## Change Log

### 2026-01-12 (Update 9) - Phase 4 WASM 타겟 추가
- **WebAssembly 코드 생성기** 추가 (`aoel-codegen/src/wasm_codegen.rs`)
  - WAT (WebAssembly Text Format) 출력
  - 기본 연산 지원: 산술, 비교, 논리
  - 함수 호출 및 재귀 지원
  - WASI import 준비

- **CLI 업데이트**
  - `--target` 옵션 추가 (c, wasm)
  - `aoel build file.aoel --target wasm` → WAT 파일 생성
  - wat2wasm 자동 호출 (설치된 경우)

- **테스트 추가**
  - 4개 WASM 코드 생성 테스트

### 2026-01-12 (Update 8) - Phase 3 Optimization 완료
- **상수 전파 (Constant Propagation)** 구현
  - 변수에 할당된 상수 값 추적
  - Load 명령어를 상수로 대체

- **공통 부분식 제거 (CSE)** 구현
  - 동일한 표현식 감지 및 재사용
  - 임시 변수로 결과 저장

- **명령어 융합 (Instruction Fusion)** 구현
  - 항등원 제거: x + 0, x * 1, x - 0, x / 1
  - 영원 최적화: x * 0 = 0
  - 강도 감소: x + x → 2 * x
  - 이중 부정/부정 제거

- **꼬리 재귀 최적화 (TCO)** 구현
  - TailSelfCall 명령어 추가
  - VM에서 루프 기반 TCO 실행
  - 스택 오버플로 없는 재귀 지원

- **테스트 추가**
  - 20개 최적화 테스트 (기존 8개 → 20개)

### 2026-01-12 (Update 7) - v6b → AOEL 통합
- **v6b 문법을 AOEL 메인으로 통합**
  - 기존 블록 기반 문법 deprecated
  - 함수형 스크립트 문법이 메인
  - crate 이름 변경: `aoel-v6b-*` → `aoel-*`

- **타입 체커 버그 수정**
  - 배열 연결 (`+`) 시 타입 변수 통일 버그 수정
  - quicksort 등 재귀 함수 타입 체크 통과

- **모듈 시스템 구현** (`aoel-parser/src/module.rs`)
  - `use path.to.module` 문법
  - `pub` 함수/타입만 import
  - 선택적 import: `use module.{item1, item2}`
  - CLI 통합 (check, run, compile, build)

- **표준 라이브러리 확장** (20+ 함수 추가)
  - 배열: push, pop, take, drop, zip, flatten, sort, unique, index_of
  - 수학: exp, log2, asin, acos, atan, atan2, clamp
  - 문자열: chars, pad_left, pad_right, repeat
  - 타입 체크: is_int, is_float, is_string, is_bool, is_array, is_map

- **테스트 결과**
  - 55개 단위 테스트 통과
  - 15개 예제 파일 통과

### 2026-01-12 (Update 6)
- v6b crate를 aoel로 이름 변경
- v1 crate 제거
- CLI 통합

### 2026-01-12 (Update 5)
- 파서 수정 - 모든 예제 파싱 성공
- 전체 테스트 통과 (66개)

### 2026-01-12 (Update 4)
- Phase 3 진행 (50%)
- IR 최적화 패스 구현

### 2026-01-12 (Update 3)
- Phase 2 완료 (100%)
- VM 및 IR 구현

### 2026-01-12 (Update 2)
- Phase 1 완료 (100%)
- CI/CD 설정

### 2026-01-11
- Phase 0 완료
- Phase 1 시작

---

## Next Steps (Phase 4 & 5 진행을 위해)

### Phase 4: Native Compile
1. **LLVM 백엔드** 구현
   - IR → LLVM IR 변환
   - 최적화된 네이티브 코드 생성

2. **WASM 타겟** 지원
   - 웹 브라우저 실행 지원
   - WASI 지원

### Phase 5: Ecosystem
1. **I/O 함수** 구현
   - 파일 읽기/쓰기
   - JSON 파싱

2. **LSP 개선**
   - 자동 완성
   - Go to Definition
   - Hover 정보

3. **패키지 매니저**
   - 의존성 관리
   - 모듈 배포

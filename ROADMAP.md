# Vais - AI-Optimized Programming Language
## 프로젝트 로드맵

> **버전**: 0.0.1
> **목표**: AI 코드 생성에 최적화된 토큰 효율적 시스템 프로그래밍 언어
> **최종 업데이트**: 2026-01-19

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

> **상태**: ✅ 완료 (95%)

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
- [ ] 에러 메시지 개선
- [ ] 더 많은 타입 추론 케이스

---

## 🚀 Phase 2: 표준 라이브러리

> **상태**: 🔄 진행 중 (70%)

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

### 남은 작업
- [ ] **Math** - 수학 함수 (sin, cos, sqrt 등)
- [ ] **IO** - 표준 입력 처리
- [ ] **Collections** - Set, Deque 등 추가 컬렉션
- [ ] **Net** - 네트워크 기본 지원

---

## ⚡ Phase 3: 개발자 도구

> **상태**: 🔄 진행 중 (60%)

### 완료된 작업
- [x] **LSP Server** (vais-lsp)
  - [x] 기본 진단 (diagnostics)
  - [x] 시맨틱 토큰 하이라이팅
  - [ ] 자동 완성 (부분 구현)
  - [ ] Go to definition
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

### 남은 작업
- [ ] **VSCode Extension** - 문법 강조, 자동 완성
- [ ] **Formatter** - 코드 포맷터
- [ ] **Debugger** - 디버깅 지원
- [ ] **Documentation** - 언어 스펙 문서

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
└── runtime.vais

examples/          # 예제 코드 (40+ 파일) ✅
```

---

## 📊 진행률 요약

| 컴포넌트 | 상태 | 진행률 |
|----------|------|--------|
| Lexer | ✅ 완료 | 100% |
| Parser | ✅ 완료 | 100% |
| Type Checker | ✅ 완료 | 100% |
| Code Generator | ✅ 완료 | 95% |
| Standard Library | 🔄 진행 중 | 70% |
| LSP | 🔄 진행 중 | 50% |
| REPL | ✅ 완료 | 100% |
| Optimization | ✅ 완료 | 100% |

**전체 진행률: ~85%**

---

## 🧪 테스트 현황

```
✅ 8 tests passed, 0 failed
✅ 40+ example files compiled and running
```

---

## 최근 커밋

```
570e6bd Refactor codegen into modules, add exhaustiveness checking and REPL
5a2b713 Phase 3 completion: Memory management, LSP enhancement, optimization passes
f5cd20c Add async/await support, LSP server, and optimization passes
cd266a6 Add module system, fix generics, improve std library
e33bfa5 Add standard library and comprehensive examples
```

---

## 🎯 다음 목표

1. **Phase 2 완료**: Math, IO 모듈 추가
2. **LSP 개선**: 자동 완성, Go to definition 구현
3. **VSCode Extension**: 기본 문법 강조
4. **문서화**: 언어 스펙 및 튜토리얼

---

**메인테이너**: Steve
**라이센스**: MIT

# Vais Self-Hosting Compiler (Stage 1) Roadmap

## Current Status: v0.7.0 — Bootstrap Achieved! 🎉

Stage 1 컴파일러가 다중 파라미터 함수를 포함한 Vais 프로그램을 안정적으로 컴파일할 수 있음.
Import 시스템 지원 완료.
제네릭 타입 해석 (type_checker.vais) 완료.
Bitwise 연산자, Index expression, Array 리터럴 지원 완료.

---

## 최근 완료 (2024-01)

### v0.4.1 - 다중 파라미터 함수 버그 수정 ✅
- [x] `cg_gen_function_multi` 함수 추가 - 모든 파라미터를 LLVM IR 시그니처에 포함
- [x] `cg_gen_function_item`이 `cg_gen_function_multi` 사용하도록 수정
- [x] 식별자 해결 순서 변경 - 변수 테이블 우선 확인
- [x] **SIGBUS 크래시 해결** - 대용량 파일 (98+ 함수) 컴파일 가능

### v0.4.0 - Import 시스템 ✅
- [x] `@"path"` import 문법 파싱
- [x] 재귀적 모듈 로딩 (`load_module_with_imports`)
- [x] 중복 import 방지 (`loaded_modules` 트래킹)
- [x] 모듈 분리: constants, stringbuffer, lexer, helpers, parser, codegen, main_entry

---

## 현재 작업: Stage 2 부트스트래핑

### 목표
vaisc-stage1 (Vais로 작성, Rust vaisc로 컴파일) → main.vais 컴파일 → vaisc-stage2

### 남은 작업

#### 1. 런타임 함수 선언 추가 ✅
- [x] `memcpy` 선언 추가
- [x] `memcmp` 선언 추가
- [x] `realloc` 선언 추가

#### 2. Stage 2 컴파일 테스트
- [ ] stage1으로 main.vais 컴파일 시도
- [ ] 생성된 LLVM IR 검증
- [ ] stage2 바이너리 빌드 및 테스트

#### 3. 부트스트랩 검증
- [ ] stage1과 stage2가 동일한 출력 생성하는지 비교
- [ ] 기능적 동등성 테스트

---

## Implemented Features

### Keywords
- [x] F (function)
- [x] S (struct)
- [x] X (impl)
- [x] I (if)
- [x] E (else)
- [x] L (loop)
- [x] R (return)
- [x] B (break)
- [x] M (match)
- [x] mut

### Types
- [x] i64
- [x] str
- [x] bool
- [ ] Custom struct types (partial - parsing only)

### Expressions
- [x] Integer literals
- [x] String literals
- [x] Identifiers
- [x] Binary operators (+, -, *, /, %, <, >, <=, >=, ==, !=, &&, ||, &, |, ^, <<, >>)
- [x] Unary operators (-)
- [x] Function calls (다중 파라미터 지원 ✅)
- [x] Method calls (.method())
- [x] Field access (.field)
- [x] Self calls (@, @.method())
- [x] Block expressions { ... }
- [x] If expressions (I cond { } E { })
- [x] Loop expressions (L { })
- [x] Match expressions (M expr { pattern => body })
- [x] Assignment (=)
- [x] Struct literals (Name { field: value })

### Statements
- [x] Let bindings (name := expr)
- [x] Typed let (name: Type = expr)
- [x] Mutable let (name: mut Type = expr)
- [x] Expression statements
- [x] Return (R)
- [x] Break (B)

### Items
- [x] Functions (F name(params) -> Type { body })
- [x] Structs (S Name { fields })
- [x] Impl blocks (X Name { methods })
- [x] Import (@"path") ✅

---

## Phase 1: Core Language (Current)

### Completed
- [x] Index expressions [i] ✅
- [x] Bitwise operators (&, |, ^, <<, >>) ✅
- [x] Unary not (!) ✅ (이미 구현됨)
- [x] Continue statement (C) ✅ (이미 구현됨)
- [x] Array literal [e1, e2, ...] ✅ (parser_s1.vais, parser.vais, codegen_s1.vais, codegen.vais)

### In Progress

### Todo
- [ ] While loop sugar
- [ ] Negative numbers in lexer

---

## Phase 2: Advanced Features

### Pattern Matching Enhancements
- [ ] Wildcard pattern (_)
- [ ] Variable binding patterns
- [ ] Multiple patterns (1 | 2 => ...)
- [ ] Guard expressions (pattern if cond => ...)

### Type System
- [x] Generic types <T> parsing ✅ (parser.vais, parser_s1.vais)
- [x] Generic type resolution ✅ (type_checker.vais)
  - [x] Generic binding management (add/get/clear_generic_binding)
  - [x] Type instantiation (instantiate_type)
  - [x] Generic inference from argument types (infer_generic_from_types)
  - [x] Generic struct field access with type arguments
- [x] Trait resolution ✅ (type_checker.vais)
  - [x] TraitDefInfo/TraitImplInfo structures
  - [x] add_trait/find_trait/register_trait
  - [x] add_trait_impl/find_trait_impl/type_implements_trait
  - [x] check_trait with super trait validation
  - [x] check_impl with trait verification
- [x] Type mismatch detailed descriptions ✅ (format_type, print_errors)
  - [x] format_type: ResolvedType → human-readable string
  - [x] print_errors: formatted error output with type names
  - [x] mismatch calls pass actual expected/found type info
- [x] Error recovery (continue checking after errors) ✅
- [ ] Type inference improvements
- [ ] Option<T> / Result<T, E>

### Memory
- [ ] Pointers (*T)
- [ ] References (&T, &mut T)
- [ ] Defer statement

---

## Phase 3: Standard Library

### Core
- [ ] Vec<T>
- [ ] String
- [ ] HashMap<K, V>

### I/O
- [ ] File operations (partial - fopen, fread, fwrite exist)
- [ ] Better print functions

---

## Phase 4: Self-Compilation

### Goal: Stage 1 컴파일러가 자기 자신을 컴파일

### Requirements
- [ ] All features used in main.vais must be supported
- [ ] Stable code generation
- [ ] Full bootstrap test

### Progress
- [x] 다중 파라미터 함수 지원
- [x] Import 시스템
- [ ] 완전한 main.vais 컴파일
- [ ] Stage 2 생성 및 검증

---

## Known Limitations (Updated)

1. ~~**다중 파라미터 버그**~~ - **해결됨** ✅
2. **Match scrutinee**: Must be simple identifier (not complex expression) due to `{` ambiguity
3. ~~**No generics**~~ - **타입 해석 완료** ✅ (파싱 및 타입 추론 지원, 코드젠은 추후)
4. **memcpy 등 런타임 함수**: Header에 선언 추가 필요

---

## Build Instructions

```bash
# Compile main.vais with Rust compiler
cargo run --package vaisc -- selfhost/main.vais --emit-ir -o /tmp/main_stage1.ll

# Build Stage 1 binary
clang -O0 /tmp/main_stage1.ll -o selfhost/vaisc-stage1 -lm

# Run Stage 1 compiler (compiles /tmp/test_import.vais)
./selfhost/vaisc-stage1

# Output written to selfhost/main_output.ll
```

---

## Version History

- **v0.7.0** - 부트스트랩 달성! 🎉
  - Stage1→Stage2→Stage3 fixed point 도달 (SHA256: e14776a6..., 17,807줄 일치)
  - Inkwell 빌트인: fopen_ptr/memcpy_str 래퍼 함수 + realloc 선언 추가
  - Stage1: 124KB, Stage2: 134KB (arm64 macOS)
- **v0.6.1** - Phase 38 TC 100% + E001 해결
  - Type Checker 100%: check_enum, check_type_alias 추가 (7개 아이템 타입 전부 커버)
  - E001 해결: Rust unify()에 Ref(T)↔T auto-deref 추가 + memcpy_str Inkwell 등록
  - codegen.vais 클린 컴파일 달성 (E001 제거)
  - E2E 241개 + selfhost lexer 114개 전부 통과
- **v0.6.0** - Phase 38 진도 (TC 95%+, Codegen 100%, Module 100%)
  - Type Checker: 9개 누락 식 핸들러 추가 (Array, Range, Await, Try, Unwrap, Ref, Deref, AssignOp, Spawn)
  - Codegen: Await/Spawn codegen + method dispatch 개선 (impl type prefix, infer_receiver_type)
  - Module: parser.vais parse_use 구현 (U ident/ident 파싱)
  - Type mismatch 상세 설명 (format_type + print_errors)
  - E2E 241개 + selfhost lexer 114개 전부 통과
- **v0.5.2** - Array 리터럴 지원
  - Array literal [e1, e2, ...] 파싱 (parser_s1.vais, parser.vais)
  - Array literal 코드젠 (codegen_s1.vais, codegen.vais)
- **v0.5.1** - Bitwise 연산자 및 Index expression 지원
  - Bitwise operators (&, |, ^, <<, >>) 파싱 완성
  - Index expression [i] 파싱 및 코드젠 구현
- **v0.5.0** - 제네릭 타입 해석 (type_checker.vais)
  - Generic binding management
  - Type instantiation for function calls and struct fields
  - Generic inference from argument types
- **v0.4.1** - 다중 파라미터 함수 버그 수정, SIGBUS 크래시 해결
- **v0.4.0** - Import 시스템 (@"path"), 모듈 분리
- **v0.3.0** - Match expressions (M expr { pattern => body })
- **v0.2.0** - Multi-function compilation, structs, impl blocks
- **v0.1.0** - Basic single-function compilation

---

## Next Steps (for /workflow)

1. **런타임 함수 선언 추가**: `memcpy`, `memcmp` 등을 stage1 codegen header에 추가
2. **Stage 2 컴파일 테스트**: stage1으로 main.vais 전체 컴파일 시도
3. **부트스트랩 완성**: stage2 생성 및 동등성 검증

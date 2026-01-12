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
| Phase 3: Optimization | **DONE** | 100% |
| Phase 4: Native Compile | **DONE** | 100% |
| Phase 5: Ecosystem | **IN PROGRESS** | 40% |

**Last Updated:** 2026-01-12

---

## Quick Start

```bash
# 빌드
cd aoel-rs
cargo build --release

# 실행
./target/release/aoel run examples/factorial.aoel

# 네이티브 컴파일
./target/release/aoel build examples/factorial.aoel --target llvm

# JIT 실행 (Cranelift)
cargo build --release --features cranelift
./target/release/aoel jit examples/simple.aoel
```

---

## Language Syntax

```aoel
// 함수 정의
add(a, b) = a + b
factorial(n) = n < 2 ? 1 : n * $(n - 1)

// 컬렉션 연산
[1,2,3].@(_ * 2)      // map: [2,4,6]
[1,2,3].?(_ > 1)      // filter: [2,3]
[1,2,3]./(0, _ + _)   // reduce: 6

// 모듈
use math.{sin, cos}
```

---

## Completed Phases

### Phase 0-1: Foundation
- Python 프로토타입 → Rust 재구현
- Lexer, Parser, AST, Type Checker (Hindley-Milner)
- 76개 테스트 통과

### Phase 2: Execution
- 스택 기반 VM
- 50+ 빌트인 함수
- 클로저, 컬렉션 연산, 자기 재귀 ($)

### Phase 3: Optimization
- 상수 폴딩/전파, DCE, CSE
- 명령어 융합, 꼬리 재귀 최적화 (TCO)

### Phase 4: Native Compile
| Backend | Command | 특징 |
|---------|---------|------|
| C | `aoel build file.aoel` | 가장 호환성 좋음 |
| WASM | `aoel build file.aoel --target wasm` | 웹 지원 |
| LLVM | `aoel build file.aoel --target llvm` | 최적화 우수 |
| Cranelift | `aoel jit file.aoel` | 빠른 컴파일 |

---

## Phase 5: Ecosystem (진행 중)

### 완료
- [x] 50+ 빌트인 함수
- [x] 모듈 시스템 (`use`, `pub`)
- [x] LSP 기본 구현
- [x] REPL

### TODO
- [ ] I/O 함수 (파일, JSON, 네트워킹)
- [ ] LSP 개선 (자동완성, Go to Definition)
- [ ] 패키지 매니저
- [ ] FFI (C, Rust, Python 바인딩)
- [ ] 디버거, 프로파일러, Formatter

---

## Project Structure

```
aoel-rs/crates/
├── aoel-lexer/      # 토큰화
├── aoel-ast/        # AST 정의
├── aoel-parser/     # 파서 + 모듈
├── aoel-typeck/     # 타입 체커
├── aoel-ir/         # IR + 최적화
├── aoel-lowering/   # AST → IR
├── aoel-vm/         # 스택 VM
├── aoel-codegen/    # C/WASM/LLVM/Cranelift
├── aoel-lsp/        # Language Server
└── aoel-cli/        # CLI
```

---

## Test Summary

| Crate | Tests |
|-------|-------|
| aoel-lexer | 11 |
| aoel-parser | 10 |
| aoel-typeck | 11 |
| aoel-ir | 20 |
| aoel-lowering | 3 |
| aoel-vm | 6 |
| aoel-codegen | 14 |
| **Total** | **76** |

---

## Change Log (Recent)

### 2026-01-12 - Phase 4 완료
- Cranelift JIT 백엔드 추가
- LLVM Jump/JumpIfNot 완전 구현
- 4가지 백엔드 지원 (C, WASM, LLVM, Cranelift)

### 2026-01-12 - Phase 3 완료
- 최적화 패스: 상수 전파, CSE, 명령어 융합, TCO
- v6b 문법을 AOEL 메인으로 통합

### 2026-01-11 - Phase 0-2 완료
- Python 프로토타입
- Rust 컴파일러 프론트엔드
- VM 및 IR 구현

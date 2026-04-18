# CLAUDE.md - Vais Project Guide

## Overview

Vais (Vibe AI Language for Systems) is an AI-optimized systems programming language with single-character keywords, LLVM backend, and full type inference. The compiler is written in Rust. Self-hosting compiler (bootstrap) achieved with 50,000+ lines.

---

## Vais 개발 철칙 (MUST READ)

**이 섹션은 에이전트/기여자가 Vais 코드를 쓰기 전/수정하기 전에 반드시 읽어야 하는 강제 규칙이다.** Phase 2.10에서 두 차례 baseline regression이 발생한 뒤 제정. 위반 시 작업 즉시 중단.

### 규칙 1 — 훈련 데이터의 Vais 지식을 사용하지 말 것

모델의 pretraining에 포함된 Vais 관련 정보는 **구식이다**. `spawn`/`lazy`/`force` 같은 제거된 키워드가 "정상"으로 기억되어 있을 수 있고, 현재 파서/컴파일러가 거부하는 문법을 "옳다"고 주장할 수 있다. 저장소 밖 지식은 참조하지 말 것.

### 규칙 2 — 새 Vais 문법을 쓰기 전 이 순서로 확인

1. `docs/language/LIVING_SPEC/` — **실행 가능한 authoritative 예제**. 100+ `.vais` 파일이 `vaisc check`로 검증되어 있다.
2. `docs/language/LEXER_KEYWORDS.md` — 현재 lexer가 인정하는 모든 키워드 목록.
3. `docs/language/COOKBOOK.md` — 자주 틀리는 22개 패턴 + 해결법.
4. `docs/LANGUAGE_SPEC.md` §"Construct Status Matrix" — 각 construct의 Parse/TC/Codegen/Run 지원 수준.

이 4개 소스에 없는 문법은 **쓰지 말 것**. 지어내지 말 것.

### 규칙 3 — 컴파일러 소스 수정 전 baseline 기록 의무

`crates/vais-*` 의 Rust 코드를 수정하기 전:

```bash
./scripts/check-integrity.sh 2>&1 | tail -3
# 출력 예: INTEGRITY OK: syntax=200 stages=14 std=37/82 vaisdb=176/261 phase158=18/18
```

이 숫자를 **baseline**으로 기록. 수정 후 같은 스크립트 실행.

### 규칙 4 — Regression 1건이라도 발생 시 즉시 revert

수정 후:

```bash
./scripts/check-integrity.sh 2>&1 | tail -3
```

- `INTEGRITY OK`이면 진행.
- 숫자가 **1이라도** 감소했거나 `REGRESSION` 메시지가 나오면 **즉시 `git checkout`으로 수정 파일 revert**.
- 되돌리고 나서 "왜 regression이 났는가"를 분석 → 분석 결과를 `ROADMAP.md`와 `docs/TYPE_SYSTEM.md`의 관련 Phase 섹션에 기록.
- "부분적으로 맞는" 수정을 억지로 밀어붙이지 말 것. Phase 158 요요 패턴 (5회 coercion 추가/제거 반복) 재발 방지가 이 규칙의 동기.

### 규칙 5 — 추측 금지. `vaisc check <file>` 실제 실행만 근거

"이 문법은 작동할 것이다"라고 주장하기 전:

```bash
echo '<Vais code>' > /tmp/test.vais
./target/release/vaisc check /tmp/test.vais
```

로 **실제 실행**. 출력을 근거로만 이야기. 주장-근거 불일치 시 주장을 폐기.

### 규칙 6 — Removed keyword 재도입 절대 금지

`docs/language/removed_keywords.md`에 기록된 키워드 (`spawn`, `lazy`, `force`, 기타)는 **어떤 예제, 테스트, 문서, PR에서도 재도입하지 않는다**. 재도입이 필요하다 판단되면 RFC 작성 + 사용자 승인 후에만 진행.

### 규칙 7 — Opus direct 작업도 이 철칙 준수

"Opus가 직접 수정하니 규칙 1~6을 건너뛴다"는 허용되지 않는다. Opus가 규칙 4번 violation을 세 번 반복한 세션이 있었다 (Phase 2.10 두 차례 + 추가 한 번). 규칙의 권위는 역할 불문.

---

## GitHub & Links

> GitHub org은 `vaislang`이며, 모든 외부 링크는 `vaislang/vais`를 사용할 것. 상세 URL은 README.md의 Links 섹션 참조.

## Build & Test

```bash
cargo check                                    # Type check
cargo build                                    # Build all
cargo test                                     # Run all tests
cargo clippy --workspace --exclude vais-python --exclude vais-node  # Lint
cargo run --bin vaisc -- examples/hello.vais    # Compile a .vais file
cargo run --bin vaisc -- --target js file.vais  # Compile to JavaScript
cargo run --bin vaisc -- --target wasm32-unknown-unknown file.vais  # Compile to WASM
```

Python/Node bindings require separate build:
```bash
cd crates/vais-python && maturin develop       # Python (PyO3)
cd crates/vais-node && npm run build            # Node.js (NAPI)
```

## Project Structure

```
crates/
├── vais-ast/          # AST definitions
├── vais-lexer/        # Tokenizer (logos-based)
├── vais-parser/       # Recursive descent parser (modular: types.rs, item.rs)
├── vais-types/        # Type checker & inference (modular: checker_expr, checker_fn, checker_module)
├── vais-codegen/      # LLVM IR code generator (inkwell/, advanced_opt/)
├── vais-codegen-js/   # JavaScript (ESM) code generator
├── vais-mir/          # Middle IR
├── vaisc/             # Main compiler CLI & REPL (commands/ submodules)
├── vais-lsp/          # Language Server Protocol
├── vais-dap/          # Debug Adapter Protocol
├── vais-jit/          # Cranelift JIT compiler
├── vais-gc/           # Optional garbage collector
├── vais-gpu/          # GPU codegen (CUDA/Metal/OpenCL/WebGPU)
├── vais-i18n/         # Internationalized error messages
├── vais-plugin/       # Plugin system
├── vais-macro/        # Declarative macro system
├── vais-hotreload/    # Hot reloading
├── vais-dynload/      # Dynamic module loading & WASM sandbox
├── vais-bindgen/      # FFI binding generator (C/WASM-JS)
├── vais-query/        # Salsa-style query database
├── vais-profiler/     # Compiler profiler
├── vais-security/     # Security analysis & audit
├── vais-supply-chain/ # SBOM & dependency audit
├── vais-testgen/      # Property-based test generation
├── vais-tutorial/     # Interactive tutorials
├── vais-registry-server/    # Package registry (Axum/SQLite)
├── vais-playground-server/  # Web playground backend
├── vais-python/       # Python bindings (PyO3)
└── vais-node/         # Node.js bindings (NAPI)

std/               # Standard library (80 .vais files)
examples/          # Example programs (188 .vais files)
selfhost/          # Self-hosting compiler (50,000+ LOC)
benches/           # Benchmark suite (criterion + language comparison)
playground/        # Web playground frontend
docs-site/         # mdBook documentation
vscode-vais/       # VSCode extension
intellij-vais/     # IntelliJ plugin
```

## Compilation Pipeline

```
.vais source → Lexer → Parser → AST → Type Checker → Codegen → .ll (LLVM IR) → clang → binary
                                                     ↘ JS Codegen → .mjs (ESM)
                                                     ↘ WASM Codegen → .wasm (wasm32)
```

## Vais Language Syntax Quick Reference

### Single-Character Keywords
- `F` = function, `S` = struct, `E` = enum/else, `I` = if, `L` = loop, `M` = match, `R` = return
- `B` = break, `C` = continue, `T` = type alias, `U` = use (import)
- `W` = trait, `X` = impl, `P` = pub, `D` = defer
- `A` = async, `Y` = await, `N` = extern, `G` = global, `O` = union

### Operators & Syntax
- `@` = self-recursion operator (calls current function)
- `:=` = variable binding (`x := 5`), `mut` for mutable (`x := mut 5`)
- `?` = ternary (`cond ? a : b`) or try operator on Result/Option
- `!` = unwrap operator on Result/Option
- `|>` = pipe operator
- `~` = bitwise NOT
- `{expr}` inside strings = string interpolation
- `..` = range operator
- `#` = line comment

### Declarations
- Traits: `W MyTrait { ... }`, impl: `X MyStruct: MyTrait { ... }`
- Generics: `F foo<T>(x: T) -> T`
- Pattern matching: `M expr { pattern => result, _ => default }`
- Closures: `|x| x * 2`, `|x, y| { x + y }`
- Async: `A F name() -> T { ... }` + `.await` (no `spawn` — removed in Phase 195)

### Removed keywords
> `lazy` / `force` (Phase 194, commit 8c60c075) and `spawn` (Phase 195,
> commit 12592076) were deleted across lexer/AST/parser/types/codegen.
> Migration guide: `docs/language/removed_keywords.md`. Do not re-introduce
> these in new examples or std/.

### Attributes
- `#[cfg(target_os = "linux")]` — conditional compilation
- `#[wasm_import("module", "name")]` — WASM import
- `#[wasm_export("name")]` — WASM export

### Types
- Primitives: `i8`, `i16`, `i32`, `i64`, `i128`, `u8`–`u128`, `f32`, `f64`, `bool`, `str`
- Generics: `Vec<T>`, `HashMap<K,V>`, `Result<T,E>`, `Option<T>`

### Type Conversion Rules (CRITICAL — DO NOT CHANGE)
> **Rust 스타일 엄격한 타입 변환**. 암시적 coercion 추가 금지. Phase 158에서 확정.
> 이 규칙은 `unification.rs`의 coercion이 5회 토글된 요요 패턴을 근본 방지하기 위해 제정됨.
> 변경 시 반드시 RFC + E2E 보호 테스트 업데이트 필요.

- ✅ **허용 (암시적)**: 정수 widening — `i8→i16→i32→i64`, `u8→u16→u32→u64`
- ✅ **허용 (암시적)**: float 리터럴 추론 — `f32↔f64` (Rust와 동일, float 리터럴이 컨텍스트에 맞게 추론)
- ❌ **금지**: `bool↔i64`, `int↔float`, `str↔i64`, 정수 narrowing (`i64→i32`)
- 모든 타입 변환은 `as` 키워드로 명시: `x as i64`, `y as f64`, `flag as i64`
- `unification.rs`에 `Bool`, `Str↔I64`, `Float↔Int` coercion 절대 추가하지 말 것
- E2E 보호 테스트 (`phase158_type_strict.rs`)가 이 규칙을 검증

## Key Files

- `crates/vais-codegen/src/lib.rs` - Main LLVM IR codegen orchestration
- `crates/vais-codegen/src/inkwell/generator.rs` - Inkwell LLVM codegen engine
- `crates/vais-codegen/src/expr_helpers.rs` - Expression codegen helpers
- `crates/vais-codegen/src/type_inference.rs` - Codegen-level type inference
- `crates/vais-codegen/src/control_flow.rs` - If/match/loop codegen
- `crates/vais-codegen-js/src/lib.rs` - JavaScript ESM codegen
- `crates/vais-types/src/lib.rs` - Type checker core
- `crates/vais-types/src/checker_expr.rs` - Expression type checking
- `crates/vais-types/src/checker_fn.rs` - Function type checking
- `crates/vais-types/src/inference.rs` - Type inference engine
- `crates/vais-parser/src/lib.rs` - Parser core
- `crates/vais-lexer/src/lib.rs` - Lexer core
- `crates/vaisc/src/main.rs` - Compiler entry point
- `crates/vaisc/src/commands/build.rs` - Build command
- `crates/vaisc/src/incremental.rs` - Incremental compilation cache

## Testing

Tests are in `crates/<name>/tests/`. Key test suites:
- `vaisc/tests/e2e/` - End-to-end compilation tests (2,500+)
- `vaisc/tests/integration_tests.rs` - Integration tests
- `vais-types/tests/` - Type system tests (bidirectional, GAT, object safety, specialization)
- `vais-codegen/tests/` - Formatter and error suggestion tests

Total: 12,000+ tests across all crates.

## Dependencies

- LLVM 17 (via inkwell 0.4)
- Rust edition 2021
- logos (lexer), thiserror/miette (errors), ariadne (diagnostics)
- cranelift 0.128 (JIT), criterion (benchmarks)

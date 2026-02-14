# CLAUDE.md - Vais Project Guide

## Overview

Vais (Vibe AI Language for Systems) is an AI-optimized systems programming language with single-character keywords, LLVM backend, and full type inference. The compiler is written in Rust. Self-hosting compiler (bootstrap) achieved with 50,000+ lines.

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

std/               # Standard library (74 .vais files)
examples/          # Example programs (182 files)
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
- `~` = string interpolation
- `..` = range operator
- `#` = line comment

### Declarations
- Traits: `W MyTrait { ... }`, impl: `X MyTrait for MyStruct { ... }`
- Generics: `F foo<T>(x: T) -> T`
- Pattern matching: `M expr { pattern => result, _ => default }`
- Closures: `|x| x * 2`, `|x, y| { x + y }`
- Spawn: `spawn expr`, Yield: `yield expr`

### Attributes
- `#[cfg(target_os = "linux")]` — conditional compilation
- `#[wasm_import("module", "name")]` — WASM import
- `#[wasm_export("name")]` — WASM export

### Types
- Primitives: `i8`, `i16`, `i32`, `i64`, `i128`, `u8`–`u128`, `f32`, `f64`, `bool`, `str`
- Generics: `Vec<T>`, `HashMap<K,V>`, `Result<T,E>`, `Option<T>`

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
- `vaisc/tests/e2e_tests.rs` - End-to-end compilation tests (538)
- `vaisc/tests/integration_tests.rs` - Integration tests
- `vais-types/tests/` - Type system tests (bidirectional, GAT, object safety, specialization)
- `vais-codegen/tests/` - Formatter and error suggestion tests

Total: 2,500+ tests across all crates.

## Dependencies

- LLVM 17 (via inkwell 0.4)
- Rust edition 2021
- logos (lexer), thiserror/miette (errors), ariadne (diagnostics)
- cranelift 0.128 (JIT), criterion (benchmarks)

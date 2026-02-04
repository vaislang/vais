# CLAUDE.md - Vais Project Guide

## Overview

Vais (Vibe AI Language for Systems) is an AI-optimized systems programming language with single-character keywords, LLVM backend, and full type inference. The compiler is written in Rust.

## GitHub & Links

> GitHub org은 `vaislang`이며, 모든 외부 링크는 `vaislang/vais`를 사용할 것. 상세 URL은 README.md의 Links 섹션 참조.

## Build & Test

```bash
cargo check                                    # Type check
cargo build                                    # Build all
cargo test                                     # Run all tests
cargo clippy --workspace --exclude vais-python --exclude vais-node  # Lint
cargo run --bin vaisc -- examples/hello.vais    # Compile a .vais file
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
├── vais-parser/       # Recursive descent parser
├── vais-types/        # Type checker & inference
├── vais-codegen/      # LLVM IR code generator
├── vais-mir/          # Middle IR
├── vaisc/             # Main compiler CLI & REPL
├── vais-lsp/          # Language Server Protocol
├── vais-dap/          # Debug Adapter Protocol
├── vais-jit/          # Cranelift JIT compiler
├── vais-gc/           # Optional garbage collector
├── vais-gpu/          # GPU codegen (CUDA/Metal)
├── vais-i18n/         # Internationalized error messages
├── vais-plugin/       # Plugin system
├── vais-macro/        # Declarative macro system
├── vais-hotreload/    # Hot reloading
├── vais-dynload/      # Dynamic module loading & WASM sandbox
├── vais-bindgen/      # FFI binding generator
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

std/               # Standard library (.vais files)
examples/          # Example programs (105+ files)
selfhost/          # Self-hosting compiler
benches/           # Benchmark suite (criterion)
playground/        # Web playground frontend
docs-site/         # mdBook documentation
vscode-vais/       # VSCode extension
intellij-vais/     # IntelliJ plugin
```

## Compilation Pipeline

```
.vais source → Lexer → Parser → AST → Type Checker → Codegen → .ll (LLVM IR) → clang → binary
```

## Vais Language Syntax Quick Reference

- `F` = function, `S` = struct, `E` = enum, `I` = if, `E` = else, `L` = loop, `M` = match, `R` = return
- `@` = self-recursion operator
- `:=` = variable binding (`x := 5`), `mut` for mutable (`x := mut 5`)
- `?` ternary: `cond ? a : b`
- `#` = line comment
- Traits: `T MyTrait { ... }`, impl: `impl MyTrait for MyStruct { ... }`
- Generics: `F foo<T>(x: T) -> T`
- Pattern matching: `M expr { pattern => result, _ => default }`

## Key Files

- `crates/vais-codegen/src/lib.rs` - Main LLVM IR codegen
- `crates/vais-codegen/src/expr_helpers.rs` - Expression codegen helpers
- `crates/vais-codegen/src/type_inference.rs` - Codegen-level type inference
- `crates/vais-types/src/lib.rs` - Type checker core
- `crates/vais-parser/src/lib.rs` - Parser core
- `crates/vais-lexer/src/lib.rs` - Lexer core
- `crates/vaisc/src/main.rs` - Compiler entry point

## Testing

Tests are in `crates/<name>/tests/`. Key test suites:
- `vaisc/tests/e2e_tests.rs` - End-to-end compilation tests (128+)
- `vaisc/tests/integration_tests.rs` - Integration tests
- `vais-types/tests/` - Type system tests (bidirectional, GAT, object safety, specialization)
- `vais-codegen/tests/` - Formatter and error suggestion tests

## Dependencies

- LLVM 17 (via inkwell 0.4)
- Rust edition 2021
- logos (lexer), thiserror/miette (errors), ariadne (diagnostics)

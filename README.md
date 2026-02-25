# Vais 1.0.0 (Vibe AI Language for Systems)

**AI-optimized systems programming language with token-efficient syntax.**

[![CI](https://github.com/vaislang/vais/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/vaislang/vais/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/vaislang/vais/branch/main/graph/badge.svg)](https://codecov.io/gh/vaislang/vais)

Vais is designed to minimize token usage while maximizing code expressiveness, making it ideal for AI-assisted development and LLM code generation.

## Key Features

- **Single-letter keywords** - `F` (function), `S` (struct), `E` (enum/else), `I` (if), `L` (loop), `M` (match)
- **Self-recursion operator** `@` - Call the current function recursively
- **Expression-oriented** - Everything is an expression
- **LLVM backend** - Native performance with LLVM 17
- **Type inference** - Minimal type annotations with full constraint solving
- **Memory Safety** - Borrow checker with Non-Lexical Lifetimes (NLL), `--strict-borrow` mode
- **Slice Types** - `&[T]` / `&mut [T]` with fat pointer implementation
- **Parallel Compilation** - DAG-based parallel type-check and codegen (2-4x speedup)
- **Self-Hosting** - 50,000+ LOC bootstrap compiler, 21/21 clang 100% success
- **Rich Ecosystem** - 28+ crates, 74 stdlib modules, growing package ecosystem

## Quick Example

```vais
# Fibonacci with self-recursion
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)

# Struct definition
S Point { x:f64, y:f64 }

# Sum with loop
F sum(arr:[i64])->i64 {
    s := 0
    L x:arr { s += x }
    s
}
```

## Syntax Overview

| Keyword | Meaning | Example |
|---------|---------|---------|
| `F` | Function | `F add(a:i64,b:i64)->i64=a+b` |
| `S` | Struct | `S Point{x:f64,y:f64}` |
| `E` | Enum/Else | `E Option<T>{Some(T),None}` |
| `I` | If | `I x>0{1}E{-1}` |
| `L` | Loop | `L i:0..10{print(i)}` |
| `M` | Match | `M opt{Some(v)=>v,None=>0}` |
| `@` | Self-call | `@(n-1)` (recursive call) |
| `:=` | Infer & assign | `x := 42` |

## Project Structure

```
crates/
├── vais-ast/          # AST definitions
├── vais-lexer/        # Tokenizer (logos-based)
├── vais-parser/       # Recursive descent parser
├── vais-types/        # Type checker & inference
├── vais-codegen/      # LLVM IR code generator (inkwell/, advanced_opt/)
├── vais-codegen-js/   # JavaScript (ESM) code generator
├── vais-mir/          # Middle IR
├── vaisc/             # Main compiler CLI & REPL
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

std/               # Standard library (74 modules)
selfhost/          # Self-hosting compiler (50,000+ LOC)
vscode-vais/       # VSCode extension
intellij-vais/     # IntelliJ plugin
docs-site/         # mdBook documentation
examples/          # Example programs (189 files)
benches/           # Benchmark suite (criterion + language comparison)
playground/        # Web playground frontend
```

## Building

```bash
cargo build --release
cargo test                                     # Run all 4,000+ tests
cargo test -p vaisc                            # Run vaisc tests (900+ E2E tests)
cargo clippy --workspace --exclude vais-python --exclude vais-node
```

## Test Coverage

This project uses **cargo-tarpaulin** to measure test coverage. Coverage reports are generated automatically in the CI pipeline.

### Local Coverage Measurement

To generate coverage reports locally:

```bash
# Install cargo-tarpaulin (one-time setup)
cargo install cargo-tarpaulin

# Generate coverage reports (HTML and Lcov)
cargo tarpaulin --config tarpaulin.toml

# Or use the convenience alias
cargo coverage

# Generate HTML report only
cargo coverage-html

# Generate Lcov format for CI integration
cargo coverage-lcov
```

Coverage reports are saved to `target/coverage/`:
- `index.html` - Interactive HTML coverage report
- `lcov.info` - Lcov format for codecov integration

### CI Integration

Coverage is measured automatically on every push and pull request to `main` and `develop` branches. Reports are:
- Uploaded as GitHub Actions artifacts
- Sent to Codecov for tracking trends
- Available for 30 days in the CI artifacts

## Usage

```bash
# Compile a Vais file
./target/release/vaisc build hello.vais -o hello

# Run directly
./target/release/vaisc run hello.vais

# Start REPL
./target/release/vaisc repl

# Format code
./target/release/vaisc fmt src/

# Check for errors
./target/release/vaisc check hello.vais
```

## Status

- [x] Lexer (logos-based tokenizer)
- [x] Parser (recursive descent)
- [x] Type checker (generics, traits, type inference, GATs, object safety)
- [x] Code generator (LLVM IR via inkwell, JavaScript ESM, WASM)
- [x] Standard library (74 modules: Vec, HashMap, String, File, Net, Async, GPU, etc.)
- [x] Borrow checker (Non-Lexical Lifetimes, CFG-based dataflow, `--strict-borrow`)
- [x] Slice types (`&[T]` / `&mut [T]` with fat pointers)
- [x] Parallel compilation (DAG-based dependency resolution, 2-4x speedup)
- [x] Self-hosting compiler (50,000+ LOC, 21/21 clang success, Bootstrap Phase 50)
- [x] LSP support (diagnostics, completion, hover, go-to-definition, references, rename)
- [x] REPL (interactive environment)
- [x] VSCode extension + IntelliJ plugin (syntax highlighting, LSP integration)
- [x] Optimizer (constant folding, DCE, CSE, loop unrolling, LICM, alias analysis, vectorization)
- [x] Formatter (`vaisc fmt`)
- [x] Debugger (DWARF metadata, lldb/gdb support)
- [x] Ecosystem packages (vais-aes, vais-base64, vais-crc32, vais-csv, vais-json, vais-lz4, vais-regex, vais-sha256, vais-uuid)

## Performance

Vais is designed for both compilation speed and runtime performance.

### Compilation Speed

| Phase | Time (avg) | Throughput | Notes |
|-------|------------|------------|-------|
| Lexer | ~0.5ms/1K LOC | ~2M tokens/sec | |
| Parser | ~1.2ms/1K LOC | ~800K AST nodes/sec | 2.18x speedup with parallel |
| Type Checker | ~2.5ms/1K LOC | ~400K types/sec | DAG-based parallel |
| Code Generator | ~3.0ms/1K LOC | ~300K IR lines/sec | 4.14x speedup with parallel |
| **Full Pipeline** | **~1.25ms/1K LOC** | **~800K lines/sec** | **50K lines → 63ms** |

**Self-Hosting Bootstrap:** 50,000+ LOC, 21/21 clang compilation success (100%)

### Runtime Performance

Fibonacci(35) benchmark (Apple M-series ARM64, 2026-02-11):

| Language | Time | Relative |
|----------|------|----------|
| C (clang -O3) | 32ms | 0.94x |
| Rust (release) | 33ms | 0.97x |
| **Vais** (clang -O2) | **34ms** | **1.0x** |
| Python | 3200ms | ~94x slower |

### Running Benchmarks

```bash
# Compile-time benchmarks
cargo bench -p vais-benches --bench compile_bench

# Runtime comparison benchmarks
cargo bench -p vais-benches --bench runtime_bench
```

## Documentation

### Official Documentation Site

The comprehensive documentation is available as an interactive mdBook site:

```bash
# Build and view the documentation
cd docs-site
./serve.sh
```

Visit the [online documentation](https://vais.dev/docs/) or browse the individual files:

- [LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md) - Complete language specification
- [STDLIB.md](docs/STDLIB.md) - Standard library reference
- [TUTORIAL.md](docs/TUTORIAL.md) - Getting started tutorial
- [Architecture.md](docs/Architecture.md) - Compiler architecture and design
- [INSTALLATION.md](docs/INSTALLATION.md) - Installation guide
- [COVERAGE.md](docs/COVERAGE.md) - Test coverage measurement guide
- [MEMORY_SAFETY.md](docs/MEMORY_SAFETY.md) - Memory safety testing and guarantees
- [ROADMAP.md](ROADMAP.md) - Project roadmap and progress

### Memory Safety Testing

Vais ensures memory safety through Rust's ownership system and comprehensive testing:

```bash
# Run memory safety tests (without AddressSanitizer)
cargo test -p vaisc --test memory_safety_tests

# Run with AddressSanitizer (requires Rust nightly)
./scripts/asan-test.sh

# Run all sanitizers (ASan, UBSan, etc.)
./scripts/run-sanitizers.sh all
```

See [MEMORY_SAFETY.md](docs/MEMORY_SAFETY.md) for detailed information on memory safety guarantees and testing.

## Installation

### Homebrew (macOS/Linux) - No Rust required

```bash
brew tap vaislang/tap
brew install vais
```

### Pre-built Binaries

Download from [Releases](https://github.com/vaislang/vais/releases/tag/v1.0.0) (Linux, macOS Intel/ARM, Windows):

```bash
# macOS ARM
curl -LO https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-aarch64-apple-darwin.tar.gz
tar -xzf vais-v1.0.0-aarch64-apple-darwin.tar.gz
./vais/vaisc --version
```

### From Source (requires Rust)

```bash
git clone https://github.com/vaislang/vais.git
cd vais && cargo build --release
```

### Docker

```bash
docker run -it vaislang/vais:latest
```

## Links

| Resource | URL |
|----------|-----|
| **GitHub Org** | https://github.com/vaislang |
| **Repository** | https://github.com/vaislang/vais |
| **Documentation** | https://vais.dev/docs/ |
| **Playground** | https://vais.dev/playground/ |
| **Website** | https://vais.dev/ |
| **Docker Hub** | `vaislang/vais` |
| **Homebrew Tap** | `vaislang/tap` |
| **Ecosystem Packages** | https://github.com/vaislang/vais/tree/main/packages (9 packages: vais-aes, vais-base64, vais-crc32, vais-csv, vais-json, vais-lz4, vais-regex, vais-sha256, vais-uuid) |

## Community

- [GitHub Discussions](https://github.com/vaislang/vais/discussions) - Questions, ideas, show & tell
- [Contributing Guide](CONTRIBUTING.md) - How to contribute
- [CHANGELOG](CHANGELOG.md) - Release history

## Legacy

The prototype implementation is available on the [`proto`](https://github.com/vaislang/vais/tree/proto) branch.

## License

MIT License

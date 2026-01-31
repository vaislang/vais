# Vais 0.0.1

**AI-optimized systems programming language with token-efficient syntax.**

[![CI](https://github.com/sswoo88/vais/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/sswoo88/vais/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/sswoo88/vais/graph/badge.svg?token=YOUR_CODECOV_TOKEN)](https://codecov.io/gh/sswoo88/vais)

Vais is designed to minimize token usage while maximizing code expressiveness, making it ideal for AI-assisted development and LLM code generation.

## Key Features

- **Single-letter keywords** - `F` (function), `S` (struct), `E` (enum/else), `I` (if), `L` (loop), `M` (match)
- **Self-recursion operator** `@` - Call the current function recursively
- **Expression-oriented** - Everything is an expression
- **LLVM backend** - Native performance
- **Type inference** - Minimal type annotations

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
├── vais-ast/      # Abstract Syntax Tree
├── vais-lexer/    # Tokenizer (logos-based)
├── vais-parser/   # Recursive descent parser
├── vais-types/    # Type checker
├── vais-codegen/  # LLVM IR generator
├── vais-lsp/      # Language Server Protocol
└── vaisc/         # CLI compiler & REPL

std/               # Standard library (24 modules)
vscode-vais/       # VSCode extension
docs/              # Documentation
examples/          # Example programs (40+ files)
```

## Building

```bash
cargo build --release
cargo test
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
- [x] Type checker (generics, traits, type inference)
- [x] Code generator (LLVM IR)
- [x] Standard library (24 modules: Vec, HashMap, String, File, Net, etc.)
- [x] LSP support (diagnostics, completion, hover, go-to-definition, references, rename)
- [x] REPL (interactive environment)
- [x] VSCode extension (syntax highlighting, LSP integration)
- [x] Optimizer (constant folding, DCE, CSE, loop unrolling, LICM)
- [x] Formatter (`vaisc fmt`)
- [x] Debugger (DWARF metadata, lldb/gdb support)

## Performance

Vais is designed for both compilation speed and runtime performance.

### Compilation Speed

| Phase | Time (avg) | Throughput |
|-------|------------|------------|
| Lexer | ~0.5ms/1K LOC | ~2M tokens/sec |
| Parser | ~1.2ms/1K LOC | ~800K AST nodes/sec |
| Type Checker | ~2.5ms/1K LOC | ~400K types/sec |
| Code Generator | ~3.0ms/1K LOC | ~300K IR lines/sec |
| **Full Pipeline** | ~7.5ms/1K LOC | ~130 files/sec |

### Runtime Performance

Fibonacci(35) benchmark:

| Language | Time | Relative |
|----------|------|----------|
| **Vais** (optimized) | 48ms | 1.0x |
| Rust (release) | 45ms | 0.94x |
| C (gcc -O3) | 44ms | 0.92x |
| Python | 3200ms | 67x |

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

Visit the [online documentation](https://sswoo.github.io/vais/) or browse the individual files:

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

## Legacy

The prototype implementation is available on the [`proto`](https://github.com/sswoo88/vais/tree/proto) branch.

## License

MIT License

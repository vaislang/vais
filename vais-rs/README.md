# Vais - Vibe AI Script Language

[![Version](https://img.shields.io/badge/version-0.0.6-blue.svg)](https://github.com/vais-lang/vais)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-616%2B%20passed-brightgreen.svg)]()

**Vais** (**V**ibe **AI** **S**cript) is a modern programming language designed for AI-assisted development ("vibe coding"). It combines the simplicity of Python with performance that rivals compiled languages.

## Features

- **Fast**: JIT compilation with Cranelift (15-75x faster than Python for compute-heavy tasks)
- **Concise**: Minimal syntax designed for low token usage with LLMs
- **Functional**: First-class functions, pattern matching, immutable by default
- **Safe**: Strong type inference, no null pointer exceptions
- **Extensible**: FFI support, package system, rich standard library

## Quick Start

### Installation

```bash
# Build from source
git clone https://github.com/vais-lang/vais.git
cd vais/vais-rs
cargo build --release

# Add to PATH
export PATH="$PATH:$(pwd)/target/release"
```

### Hello World

```vais
// hello.vais
println("Hello, World!")
```

```bash
vais run hello.vais
```

### Examples

```vais
// Factorial with recursion
factorial(n) = n <= 1 ? 1 : n * $(n - 1)
println(factorial(10))  // 3628800

// Fibonacci (with memoization for O(n) complexity)
#[memo]
fib(n) = n <= 1 ? n : $(n-1) + $(n-2)
println(fib(30))  // 832040 (instant with memo)

// Array operations
numbers = [1, 2, 3, 4, 5]
doubled = numbers.@(x => x * 2)      // Map: [2, 4, 6, 8, 10]
evens = numbers.?(x => x % 2 == 0)   // Filter: [2, 4]
sum = numbers./+                      // Reduce: 15

// Pattern matching
classify(n) = match n {
  0 => "zero",
  1..10 => "small",
  _ => "large"
}
```

## Tools

| Tool | Command | Description |
|------|---------|-------------|
| Run | `vais run file.vais` | Execute Vais file |
| REPL | `vais repl` | Interactive shell |
| Check | `vais check file.vais` | Type check without running |
| Format | `vais format file.vais` | Format source code |
| Debug | `vais debug file.vais` | Debug with breakpoints |
| Profile | `vais profile file.vais` | Profile execution time |
| Doc | `vais doc file.vais` | Generate documentation |

### JIT Compilation

```bash
# Run with JIT for maximum performance
vais run --jit compute_heavy.vais
```

## VS Code Extension

Install the Vais extension for syntax highlighting, LSP support, and snippets:

```bash
cd editors/vscode
npm install
npm run compile
# Then install the .vsix file
```

## Project Structure

```
vais-rs/
├── crates/
│   ├── vais-lexer/      # Tokenizer
│   ├── vais-parser/     # Parser (Pratt)
│   ├── vais-ast/        # Abstract Syntax Tree
│   ├── vais-typeck/     # Type checker
│   ├── vais-lowering/   # AST → IR
│   ├── vais-ir/         # Intermediate Representation
│   ├── vais-vm/         # Virtual Machine
│   ├── vais-jit/        # JIT Compiler (Cranelift)
│   ├── vais-codegen/    # Code generation (C, WASM, LLVM)
│   ├── vais-tools/      # Debugger, Profiler, Formatter
│   ├── vais-lsp/        # Language Server
│   ├── vais-cli/        # Command Line Interface
│   └── vais-playground/ # Web Playground
├── docs/                # Documentation
├── editors/             # Editor integrations
└── examples/            # Example programs
```

## Documentation

- [Getting Started](docs/GETTING_STARTED.md)
- [Language Reference](docs/LANGUAGE_REFERENCE.md)
- [Contributing](docs/CONTRIBUTING.md)
- [Roadmap](ROADMAP.md)
- [Changelog](CHANGELOG.md)

## Token Efficiency (LLM-Friendly)

Vais is designed for AI-assisted "vibe coding" with minimal token usage:

| Example | Python Tokens | Vais Tokens | Reduction |
|---------|---------------|-------------|-----------|
| Fibonacci | ~50 | ~19 | 62% |
| Factorial | ~42 | ~15 | 64% |
| Map/Filter/Reduce | ~85 | ~28 | 67% |
| Error Handling | ~65 | ~22 | 66% |
| **Average** | | | **~58%** |

## Performance

### fibonacci(30) Benchmark

| VM Type | Time (ms) | vs Python |
|---------|-----------|-----------|
| CPython 3.14 | 110 ms | baseline |
| PyPy 7.3 (JIT) | 8.7 ms | 12.8x faster |
| Vais Standard VM | 510 ms | 4.6x slower |
| Vais FastVM | 198 ms | 1.8x slower |
| Vais FastVM + SelfCall | 90 ms | **1.2x faster** |
| **Vais JIT** | **7.1 ms** | **15.5x faster** |
| **Vais VM + @memo** | **5 ms** | **22x faster** |

### Memoization (`#[memo]`)

Use `#[memo]` attribute for automatic function result caching:

```vais
#[memo]
fib(n) = n <= 1 ? n : $(n-1) + $(n-2)
fib(30)  // 832040 - executes in ~5ms instead of ~4.5s
```

| Benchmark | Without @memo | With @memo | Speedup |
|-----------|---------------|------------|---------|
| fib(30) | 4.5s | 5ms | **900x** |
| fib(35) | 50s | 5ms | **10000x** |

### Other Benchmarks

| Operation | Python | Vais VM | Vais JIT |
|-----------|--------|---------|----------|
| Map (1000 elements) | 27.4µs | 24.7µs | - |
| Filter (1000 elements) | 28.0µs | 24.0µs | - |
| Factorial(20) | 1030ns | - | 48ns (21x faster) |
| Fibonacci(20) | 922µs | - | 60µs (15x faster) |

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

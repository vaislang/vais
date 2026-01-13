# AOEL - AI-Optimized Executable Language

[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/aoel-lang/aoel)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-522%2B%20passed-brightgreen.svg)]()

AOEL is a modern programming language designed for AI-assisted development ("vibe coding"). It combines the simplicity of Python with performance that rivals compiled languages.

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
git clone https://github.com/aoel-lang/aoel.git
cd aoel/aoel-rs
cargo build --release

# Add to PATH
export PATH="$PATH:$(pwd)/target/release"
```

### Hello World

```aoel
// hello.aoel
println("Hello, World!")
```

```bash
aoel run hello.aoel
```

### Examples

```aoel
// Factorial with recursion
factorial(n) = n <= 1 ? 1 : n * $(n - 1)
println(factorial(10))  // 3628800

// Fibonacci
fib(n) = n <= 1 ? n : $(n-1) + $(n-2)
println(fib(20))  // 6765

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
| Run | `aoel run file.aoel` | Execute AOEL file |
| REPL | `aoel repl` | Interactive shell |
| Check | `aoel check file.aoel` | Type check without running |
| Format | `aoel format file.aoel` | Format source code |
| Debug | `aoel debug file.aoel` | Debug with breakpoints |
| Profile | `aoel profile file.aoel` | Profile execution time |
| Doc | `aoel doc file.aoel` | Generate documentation |

### JIT Compilation

```bash
# Run with JIT for maximum performance
aoel run --jit compute_heavy.aoel
```

## VS Code Extension

Install the AOEL extension for syntax highlighting, LSP support, and snippets:

```bash
cd editors/vscode
npm install
npm run compile
# Then install the .vsix file
```

## Project Structure

```
aoel-rs/
├── crates/
│   ├── aoel-lexer/      # Tokenizer
│   ├── aoel-parser/     # Parser (Pratt)
│   ├── aoel-ast/        # Abstract Syntax Tree
│   ├── aoel-typeck/     # Type checker
│   ├── aoel-lowering/   # AST → IR
│   ├── aoel-ir/         # Intermediate Representation
│   ├── aoel-vm/         # Virtual Machine
│   ├── aoel-jit/        # JIT Compiler (Cranelift)
│   ├── aoel-codegen/    # Code generation (C, WASM, LLVM)
│   ├── aoel-tools/      # Debugger, Profiler, Formatter
│   ├── aoel-lsp/        # Language Server
│   ├── aoel-cli/        # Command Line Interface
│   └── aoel-playground/ # Web Playground
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

## Performance

| Operation | Python | AOEL VM | AOEL JIT |
|-----------|--------|---------|----------|
| Map (1000 elements) | 27.4µs | 24.7µs | - |
| Filter (1000 elements) | 28.0µs | 24.0µs | - |
| Factorial(20) | 1030ns | - | 48ns (21x faster) |
| Fibonacci(20) | 922µs | - | 60µs (15x faster) |

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

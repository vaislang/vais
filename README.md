# Vais - Vibe AI Script

[![Build Status](https://github.com/sswoo88/vais/actions/workflows/ci.yml/badge.svg)](https://github.com/sswoo88/vais/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Vais** is a programming language designed for AI to generate, modify, and execute code most efficiently. It features a concise syntax optimized for token efficiency while maintaining full expressiveness.

[한국어 문서](README.ko.md) | [Language Guide](docs/syntax.md) | [API Reference](docs/api.md) | [Examples](docs/examples.md)

## Features

- **Token-Efficient Syntax** - 30-60% fewer tokens compared to Python
- **Functional-First** - First-class functions, closures, and collection operations
- **Self-Recursion** - `$` operator for elegant recursive definitions
- **Collection Operators** - `.@` (map), `.?` (filter), `./` (reduce)
- **Multiple Backends** - Interpreter, JIT (50-75x faster), C, WASM, LLVM
- **Rich Ecosystem** - LSP, Package Manager, Debugger, Formatter, Profiler
- **Web Playground** - Run Vais in your browser via WebAssembly

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/sswoo88/vais.git
cd vais/vais-rs

# Build
cargo build --release

# Add to PATH (optional)
export PATH="$PATH:$(pwd)/target/release"
```

### Hello World

```bash
echo 'print("Hello, Vais!")' > hello.vais
./target/release/vais run hello.vais
```

### REPL

```bash
./target/release/vais repl
```

## Language Overview

### Functions

```vais
// Simple function
add(a, b) = a + b

// Recursive with self-call ($)
factorial(n) = n < 2 ? 1 : n * $(n - 1)

// Fibonacci
fib(n) = n < 2 ? n : $(n - 1) + $(n - 2)
```

### Collection Operations

```vais
numbers = [1, 2, 3, 4, 5]

// Map: double each element
doubled = numbers.@(_ * 2)        // [2, 4, 6, 8, 10]

// Filter: keep evens
evens = numbers.?(_ % 2 == 0)     // [2, 4]

// Reduce: sum all
sum = numbers./+(0, _ + _)        // 15

// Chain operations
result = [1..10].?(_ % 2 == 0).@(_ * _)  // [4, 16, 36, 64]
```

### Ternary Expressions

```vais
// Ternary operator
max(a, b) = a > b ? a : b

// Nested ternary
grade(score) = score >= 90 ? "A" : score >= 80 ? "B" : score >= 70 ? "C" : "F"
```

### Modules

```vais
// Import specific functions
use math.{sin, cos, pi}

// Public function (exportable)
pub calculate(x) = sin(x) * cos(x)
```

## Execution Modes

### Interpreter (Default)

```bash
vais run program.vais
```

### JIT Compilation (50-75x faster)

```bash
# Build with JIT support
cargo build --release --features jit

# Run with JIT
vais run program.vais --jit
```

### Native Compilation

```bash
# Compile to C
vais build program.vais --target c

# Compile to WebAssembly
vais build program.vais --target wasm

# Compile to LLVM IR
vais build program.vais --target llvm
```

## Built-in Functions

### Core (20+)
`print`, `println`, `len`, `type`, `str`, `int`, `float`, `range`, `abs`, `sqrt`, `pow`, `sin`, `cos`, `tan`, `log`, `exp`, `floor`, `ceil`, `round`, `min`, `max`

### Collections (15+)
`head`, `tail`, `init`, `last`, `reverse`, `sort`, `unique`, `concat`, `flatten`, `zip`, `enumerate`, `take`, `drop`, `slice`, `sum`, `product`

### Strings (10+)
`split`, `join`, `trim`, `upper`, `lower`, `contains`, `replace`, `starts_with`, `ends_with`, `substring`

### File I/O - std.io (25+)
`read_file`, `write_file`, `append_file`, `read_lines`, `path_join`, `path_exists`, `list_dir`, `create_dir`, `remove_file`, `cwd`, `env_get`

### JSON - std.json (14)
`json_parse`, `json_stringify`, `json_get`, `json_set`, `json_keys`, `json_values`, `json_has`, `json_remove`, `json_merge`, `json_type`

### HTTP - std.net (10)
`http_get`, `http_post`, `http_put`, `http_delete`, `http_get_json`, `http_post_json`, `url_encode`, `url_decode`

## Development Tools

### Language Server (LSP)

Full IDE support with:
- Auto-completion
- Hover documentation
- Go to Definition
- Find References
- Rename Symbol
- Signature Help

```bash
# Start LSP server
vais lsp
```

### Package Manager

```bash
# Initialize a new project
vais init my-project

# Add dependencies
vais add utils

# Install dependencies
vais install

# Publish to registry
vais publish
```

### Code Formatting

```bash
# Format to stdout
vais format program.vais

# Format in place
vais format program.vais --write

# Check formatting
vais format program.vais --check
```

### Profiler

```bash
# Profile execution
vais profile program.vais

# JSON output
vais profile program.vais --format json
```

## Web Playground

Try Vais in your browser without installation:

```bash
cd vais-rs/crates/vais-playground
wasm-pack build --target web --out-dir www/pkg
cd www && python3 -m http.server 8080
# Open http://localhost:8080
```

## Project Structure

```
vais-rs/crates/
├── vais-lexer/      # Tokenizer
├── vais-ast/        # AST definitions
├── vais-parser/     # Parser + modules
├── vais-typeck/     # Type checker (Hindley-Milner)
├── vais-ir/         # IR + optimizations
├── vais-lowering/   # AST → IR
├── vais-vm/         # Stack-based VM
├── vais-jit/        # Adaptive JIT (Cranelift)
├── vais-codegen/    # C/WASM/LLVM backends
├── vais-tools/      # Formatter, Profiler, Debugger
├── vais-lsp/        # Language Server
├── vais-playground/ # Web Playground (WASM)
└── vais-cli/        # CLI interface
```

## Performance

| Benchmark | Interpreter | JIT | Speedup |
|-----------|-------------|-----|---------|
| add(100, 200) | 769 ns | 15 ns | 51x |
| calc(50, 30) | 875 ns | 14 ns | 62x |
| math(100) | 961 ns | 13 ns | 74x |

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with Rust
- JIT powered by [Cranelift](https://cranelift.dev/)
- Inspired by functional programming languages

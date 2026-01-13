# AOEL - AI-Optimized Executable Language

[![Build Status](https://github.com/sswoo88/aoel/actions/workflows/ci.yml/badge.svg)](https://github.com/sswoo88/aoel/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**AOEL** is a programming language designed for AI to generate, modify, and execute code most efficiently. It features a concise syntax optimized for token efficiency while maintaining full expressiveness.

[한국어 문서](README.ko.md) | [Language Guide](docs/syntax.md) | [API Reference](docs/api.md) | [Examples](docs/examples.md)

## Features

- **Token-Efficient Syntax** - 30-60% fewer tokens compared to Python
- **Functional-First** - First-class functions, closures, and collection operations
- **Self-Recursion** - `$` operator for elegant recursive definitions
- **Collection Operators** - `.@` (map), `.?` (filter), `./` (reduce)
- **Multiple Backends** - Interpreter, JIT (50-75x faster), C, WASM, LLVM
- **Rich Ecosystem** - LSP, Package Manager, Debugger, Formatter, Profiler
- **Web Playground** - Run AOEL in your browser via WebAssembly

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/sswoo88/aoel.git
cd aoel/aoel-rs

# Build
cargo build --release

# Add to PATH (optional)
export PATH="$PATH:$(pwd)/target/release"
```

### Hello World

```bash
echo 'print("Hello, AOEL!")' > hello.aoel
./target/release/aoel run hello.aoel
```

### REPL

```bash
./target/release/aoel repl
```

## Language Overview

### Functions

```aoel
// Simple function
add(a, b) = a + b

// Recursive with self-call ($)
factorial(n) = n < 2 ? 1 : n * $(n - 1)

// Fibonacci
fib(n) = n < 2 ? n : $(n - 1) + $(n - 2)
```

### Collection Operations

```aoel
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

```aoel
// Ternary operator
max(a, b) = a > b ? a : b

// Nested ternary
grade(score) = score >= 90 ? "A" : score >= 80 ? "B" : score >= 70 ? "C" : "F"
```

### Modules

```aoel
// Import specific functions
use math.{sin, cos, pi}

// Public function (exportable)
pub calculate(x) = sin(x) * cos(x)
```

## Execution Modes

### Interpreter (Default)

```bash
aoel run program.aoel
```

### JIT Compilation (50-75x faster)

```bash
# Build with JIT support
cargo build --release --features jit

# Run with JIT
aoel run program.aoel --jit
```

### Native Compilation

```bash
# Compile to C
aoel build program.aoel --target c

# Compile to WebAssembly
aoel build program.aoel --target wasm

# Compile to LLVM IR
aoel build program.aoel --target llvm
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
aoel lsp
```

### Package Manager

```bash
# Initialize a new project
aoel init my-project

# Add dependencies
aoel add utils

# Install dependencies
aoel install

# Publish to registry
aoel publish
```

### Code Formatting

```bash
# Format to stdout
aoel format program.aoel

# Format in place
aoel format program.aoel --write

# Check formatting
aoel format program.aoel --check
```

### Profiler

```bash
# Profile execution
aoel profile program.aoel

# JSON output
aoel profile program.aoel --format json
```

## Web Playground

Try AOEL in your browser without installation:

```bash
cd aoel-rs/crates/aoel-playground
wasm-pack build --target web --out-dir www/pkg
cd www && python3 -m http.server 8080
# Open http://localhost:8080
```

## Project Structure

```
aoel-rs/crates/
├── aoel-lexer/      # Tokenizer
├── aoel-ast/        # AST definitions
├── aoel-parser/     # Parser + modules
├── aoel-typeck/     # Type checker (Hindley-Milner)
├── aoel-ir/         # IR + optimizations
├── aoel-lowering/   # AST → IR
├── aoel-vm/         # Stack-based VM
├── aoel-jit/        # Adaptive JIT (Cranelift)
├── aoel-codegen/    # C/WASM/LLVM backends
├── aoel-tools/      # Formatter, Profiler, Debugger
├── aoel-lsp/        # Language Server
├── aoel-playground/ # Web Playground (WASM)
└── aoel-cli/        # CLI interface
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

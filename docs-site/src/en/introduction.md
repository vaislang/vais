# Vais Programming Language

**Vais** (Vibe AI Language for Systems) is an AI-optimized systems programming language designed for maximum token efficiency and developer productivity.

## Key Features

### Token-Efficient Syntax

- **Single-Character Keywords**: `F` for function, `S` for struct, `E` for enum/else, `I` for if, `L` for loop, `M` for match, `R` for return
- **Self-Recursion Operator `@`**: Call the current function recursively with minimal tokens
- **Concise Operators**: `:=` for variable binding, `?` for error propagation, `!` for unwrap, `|>` for piping
- **50-70% fewer tokens** compared to Rust/C++ in AI-generated code

### Modern Type System

- **Full Type Inference**: Minimal type annotations with complete constraint solving
- **Generics**: Generic functions, structs, and enums with type parameters
- **Traits**: Interface-based polymorphism with trait bounds
- **Memory Safety**: Borrow checker with Non-Lexical Lifetimes (NLL) and `--strict-borrow` mode
- **Slice Types**: `&[T]` and `&mut [T]` with fat pointer implementation

### Performance & Compilation

- **LLVM Backend**: Native performance with LLVM 17 code generation
- **Parallel Compilation**: DAG-based parallel type-check and codegen (2-4x speedup)
- **774K lines/second** compilation speed (50K LOC in 64.6ms)
- **Multiple Targets**: Native binaries, JavaScript (ESM), WebAssembly (WASM)

### Advanced Features

- **Expression-Oriented**: Everything returns a value, no statements vs expressions distinction
- **Pattern Matching**: Exhaustive matching with `M` keyword
- **Error Handling**: `Result<T,E>` and `Option<T>` types with `?` try operator
- **Async/Await**: First-class async support with `A` and `Y` keywords
- **Macro System**: Declarative macros for metaprogramming

## Why Vais?

Vais is built from the ground up to excel in AI-assisted development:

- **Token Efficiency**: Single-character keywords minimize token usage in LLM context windows
- **Self-Hosting**: 50,000+ LOC bootstrap compiler written in Vais itself
- **Production-Ready**: 2,500+ tests across 28+ crates, 655 E2E tests
- **Rich Ecosystem**: 74 standard library modules, 9 official packages
- **Fast Iteration**: JIT compiler, REPL, hot reloading, incremental compilation

## Project Structure

Vais includes a comprehensive set of compiler components and tools:

```
crates/
├── vais-ast/          # AST definitions
├── vais-lexer/        # Tokenizer (logos-based)
├── vais-parser/       # Recursive descent parser
├── vais-types/        # Type checker & inference
├── vais-codegen/      # LLVM IR code generator
├── vais-codegen-js/   # JavaScript (ESM) code generator
├── vais-mir/          # Middle IR
├── vaisc/             # Main compiler CLI & REPL
├── vais-lsp/          # Language Server Protocol
├── vais-dap/          # Debug Adapter Protocol
├── vais-jit/          # Cranelift JIT compiler
├── vais-bindgen/      # FFI binding generator
├── vais-registry-server/    # Package registry
└── vais-playground-server/  # Web playground backend

std/               # Standard library (74 modules)
selfhost/          # Self-hosting compiler (50,000+ LOC)
vscode-vais/       # VSCode extension
intellij-vais/     # IntelliJ plugin
docs-site/         # mdBook documentation
examples/          # Example programs (189 files)
```

## Compilation Pipeline

Vais uses a multi-stage compilation pipeline:

```
.vais source → Lexer → Parser → AST → Type Checker → Codegen → Output

Outputs:
  → LLVM IR (.ll) → clang → native binary
  → JavaScript ESM (.mjs)
  → WebAssembly (.wasm)
```

**Key Stages:**
- **Lexer**: Tokenizes source code using logos library (~2M tokens/sec)
- **Parser**: Builds AST with recursive descent parser (~800K nodes/sec)
- **Type Checker**: Performs type inference and constraint solving (~400K types/sec)
- **Codegen**: Generates LLVM IR, JavaScript, or WASM (~300K IR lines/sec)

## Quick Examples

### Hello World

```vais
F main() {
    puts("Hello, Vais!")
}
```

### Self-Recursion

```vais
# Fibonacci with @ operator
F fib(n: i64) -> i64 = n < 2 ? n : @(n-1) + @(n-2)

F main() {
    puts("fib(10) = ")
    print_i64(fib(10))  # Prints: 55
}
```

### Structs and Methods

```vais
S Point { x: f64, y: f64 }

X Point {
    F distance(self) -> f64 {
        sqrt(self.x * self.x + self.y * self.y)
    }
}

F main() {
    p := Point { x: 3.0, y: 4.0 }
    print_f64(p.distance())  # Prints: 5.0
}
```

### Pattern Matching

```vais
E Option<T> { Some(T), None }

F unwrap_or<T>(opt: Option<T>, default: T) -> T {
    M opt {
        Some(v) => v,
        None => default
    }
}
```

### Error Handling

```vais
E Result<T, E> { Ok(T), Err(E) }

F divide(a: i64, b: i64) -> Result<i64, str> {
    I b == 0 { R Err("Division by zero") }
    Ok(a / b)
}

F compute() -> Result<i64, str> {
    x := divide(10, 2)?  # Propagates error if Err
    y := divide(x, 0)?   # Returns Err("Division by zero")
    Ok(y)
}
```

## Performance

### Compilation Speed

| Phase | Time (avg) | Throughput |
|-------|------------|------------|
| Lexer | ~0.5ms/1K LOC | ~2M tokens/sec |
| Parser | ~1.2ms/1K LOC | ~800K AST nodes/sec |
| Type Checker | ~2.5ms/1K LOC | ~400K types/sec |
| Code Generator | ~3.0ms/1K LOC | ~300K IR lines/sec |
| **Full Pipeline** | **~1.25ms/1K LOC** | **~800K lines/sec** |

**Self-Hosting Bootstrap:** 50,000+ LOC compiles to LLVM IR with 21/21 clang success (100%)

### Runtime Performance

Fibonacci(35) benchmark (Apple M-series ARM64):

| Language | Time | Relative |
|----------|------|----------|
| C (clang -O3) | 32ms | 0.94x |
| Rust (release) | 33ms | 0.97x |
| **Vais** (clang -O2) | **34ms** | **1.0x** |
| Python | 3200ms | ~94x slower |

## Getting Started

1. **[Install Vais](./getting-started/installation.md)** - Download pre-built binaries or build from source
2. **[Quick Start](./getting-started/quick-start.md)** - Write your first program in 5 minutes
3. **[Tutorial](./getting-started/tutorial.md)** - Complete guide from basics to advanced features
4. **[Language Specification](./language/language-spec.md)** - Full syntax and semantics reference

## Ecosystem

### Official Packages

- `vais-aes` - AES-256 encryption (FIPS 197)
- `vais-base64` - Base64 encoding/decoding
- `vais-crc32` - CRC32 checksums
- `vais-csv` - CSV parsing and generation
- `vais-json` - JSON serialization
- `vais-lz4` - LZ4 compression
- `vais-regex` - Regular expressions
- `vais-sha256` - SHA-256 hashing (FIPS 180-4)
- `vais-uuid` - UUID generation

### Standard Library

74 modules covering:
- Collections (Vec, HashMap, HashSet, LinkedList, BTree)
- I/O (File, Network, Async I/O)
- Concurrency (Thread, Channel, Mutex, RwLock)
- GPU computing (CUDA, Metal, OpenCL, WebGPU)
- Cryptography (AES, SHA, CRC)
- Web APIs (DOM, Fetch, Canvas)

## Community & Support

- **GitHub**: [vaislang/vais](https://github.com/vaislang/vais)
- **Documentation**: [vais.dev/docs](https://vais.dev/docs/)
- **Playground**: [vais.dev/playground](https://vais.dev/playground/)
- **Issues**: [Report bugs and request features](https://github.com/vaislang/vais/issues)
- **Discussions**: [GitHub Discussions](https://github.com/vaislang/vais/discussions)

## License

Vais is open source under the MIT license.

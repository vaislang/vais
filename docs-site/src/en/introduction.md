# Vais Programming Language

**Vais** (Vibe AI Language for Systems) is an AI-optimized systems programming language designed for clear generated code, native execution, and gate-backed public claims.

> **Current public status:** Vais is currently presented as a certified Core
> compiler plus named promoted runtime gates, not a product-complete v1.0
> release. See [`PUBLIC_STATUS.md`](https://github.com/vaislang/vais/blob/main/PUBLIC_STATUS.md)
> in the repository for the gate-backed claim boundary.

## Key Features

### Canonical Syntax

- **Canonical Keywords**: `fn`, `struct`, `enum`, `else`, `match`, `return`, `use`, and `pub` are the current forms. Retired forms `F/S/E/EN/EL/M/R/T/U/P/W/X` are no longer accepted.
- **Self-Recursion Operator `@`**: Call the current function recursively with minimal tokens
- **Concise Operators**: `:=` for variable binding, `?` for error propagation, `!` for unwrap, `|>` for piping
- **Gate-backed claims**: syntax, runtime, package, and product statements are tied to named gates rather than historical phase claims

### Modern Type System

- **Type Inference**: Minimal annotations on the certified Core surface, with
  broader inference work under active hardening
- **Generics**: Generic functions, structs, and enums where covered by named
  fixtures; broader specialization semantics remain gate-bound
- **Traits**: Interface-based polymorphism with trait bounds
- **Memory Safety**: Borrow checker with Non-Lexical Lifetimes (NLL) and `--strict-borrow` mode
- **Slice Types**: `&[T]` and `&mut [T]` with fat pointer implementation

### Performance & Compilation

- **LLVM Backend**: Promoted native codegen path with LLVM 17
- **Parallel Compilation**: DAG-based type-check and codegen workbench
- **Benchmarked compilation**: performance numbers are useful regression context
  and should be read with the corresponding benchmark date
- **Targets**: Native is the promoted path; JavaScript and WebAssembly remain
  experimental unless a page names a gate

### Advanced Features

- **Expression-Oriented**: Everything returns a value, no statements vs expressions distinction
- **Pattern Matching**: Exhaustive matching with `match`
- **Error Handling**: `Result<T,E>` and `Option<T>` types with `?` try operator
- **Async/Await**: First-class async support with `A` and `Y` keywords
- **Macro System**: Declarative macros for metaprogramming

## Why Vais?

Vais is built from the ground up to excel in AI-assisted development:

- **Clarity for AI-generated code**: canonical keywords align generated code with the current lexer and public docs
- **Self-Hosting Workbench**: 50,000+ lines of Vais compiler sources used for
  bootstrap and conformance work
- **Gate-backed status**: Current guarantees are the certified Core and promoted
  runtime gates listed in `PUBLIC_STATUS.md`
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

std/               # Standard library (80 modules)
selfhost/          # Self-hosting compiler (51,190 LOC, 58 .vais files)
vscode-vais/       # VSCode extension
intellij-vais/     # IntelliJ plugin
docs-site/         # mdBook documentation
examples/          # Example programs (188 files)
```

## Compilation Pipeline

Vais uses a multi-stage compilation pipeline:

```
.vais source → Lexer → Parser → AST → Type Checker → Codegen → Output

Outputs:
  → LLVM IR (.ll) → clang → native binary
  → JavaScript ESM (.mjs)
  → WebAssembly (.wasm, experimental unless gated)
```

**Key Stages:**
- **Lexer**: Tokenizes source code using logos library (~2M tokens/sec)
- **Parser**: Builds AST with recursive descent parser (~800K nodes/sec)
- **Type Checker**: Performs type inference and constraint solving (~400K types/sec)
- **Codegen**: Generates LLVM IR on the promoted native path; JavaScript and
  WASM paths are experimental unless a page names a gate

## Quick Examples

### Hello World

```vais
fn main() {
    puts("Hello, Vais!")
}
```

### Self-Recursion

```vais
# Fibonacci with @ operator
fn fib(n: i64) -> i64 = n < 2 ? n : @(n-1) + @(n-2)

fn main() {
    puts("fib(10) = ")
    print_i64(fib(10))  # Prints: 55
}
```

### Structs and Methods

```vais
struct Point { x: f64, y: f64 }

impl Point {
    fn distance(self) -> f64 {
        sqrt(self.x * self.x + self.y * self.y)
    }
}

fn main() {
    p := Point { x: 3.0, y: 4.0 }
    print_f64(p.distance())  # Prints: 5.0
}
```

### Pattern Matching

```vais
enum Option<T> { Some(T), None }

fn unwrap_or<T>(opt: Option<T>, default: T) -> T {
    match opt {
        Some(v) => v,
        None => default
    }
}
```

### Error Handling

```vais
enum Result<T, E> { Ok(T), Err(E) }

fn divide(a: i64, b: i64) -> Result<i64, str> {
    I b == 0 { return Err("Division by zero") }
    Ok(a / b)
}

fn compute() -> Result<i64, str> {
    x := divide(10, 2)?  # Propagates error if Err
    y := divide(x, 0)?   # Returns Err("Division by zero")
    Ok(y)
}
```

## Performance

### Compilation Speed

Current single-file compile-speed benchmark (`benches/lang-comparison/compile_bench.sh`,
Hyperfine, 2026-05-13, Apple ARM64/macOS) averages 6.3ms for Vais
`--emit-ir` across four benchmark programs. This compares Vais LLVM IR emission
against full binary compilation for the other toolchains.

| Phase | Time (avg) | Throughput |
|-------|------------|------------|
| Lexer | ~0.5ms/1K LOC | ~2M tokens/sec |
| Parser | ~1.2ms/1K LOC | ~800K AST nodes/sec |
| Type Checker | ~2.5ms/1K LOC | ~400K types/sec |
| Code Generator | ~3.0ms/1K LOC | ~300K IR lines/sec |
| **Full Pipeline** | **~1.25ms/1K LOC** | **~800K lines/sec** |

**Self-Hosting:** The repository contains 50,000+ lines of Vais compiler
sources used for bootstrap and conformance work. Current correctness is judged
by the certified Core gate and promoted runtime fixtures.

### Runtime Performance

Historical Fibonacci(35) runtime snapshot (Apple M-series ARM64). These
numbers are retained as scoped evidence until the runtime benchmark suite is
refreshed on the current compiler:

| Language | Time | Relative |
|----------|------|----------|
| C (clang -O3) | 32ms | 0.94x |
| Rust (release) | 33ms | 0.97x |
| **Vais** (clang -O2) | **34ms** | **1.0x** |
| Python | 3200ms | ~94x slower |

## Getting Started

1. **[Install Vais](./getting-started/installation.md)** - Download pre-built binaries or build from source
2. **[Quick Start](./getting-started/quick-start.md)** - Write your first program in 5 minutes
3. **[Tutorial](./getting-started/tutorial.md)** - Guided path from basics to
   advanced workbench topics
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

80 modules covering:
- Collections (Vec, HashMap, HashSet, LinkedList, BTree)
- I/O (File, Network, Async I/O)
- Concurrency (Thread, Channel, Mutex, RwLock)
- GPU computing (CUDA, Metal, OpenCL, WebGPU)
- Cryptography (AES, SHA, CRC)
- Web APIs (DOM, Fetch, Canvas)

## Community & Support

- **GitHub**: [vaislang/vais](https://github.com/vaislang/vais)
- **Documentation**: [vaislang.dev/docs](https://vaislang.dev/docs/)
- **Playground**: [vaislang.dev/playground](https://vaislang.dev/playground/)
- **Issues**: [Report bugs and request features](https://github.com/vaislang/vais/issues)
- **Discussions**: [GitHub Discussions](https://github.com/vaislang/vais/discussions)

## License

Vais is open source under the MIT license.

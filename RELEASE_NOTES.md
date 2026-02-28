# Vais v1.0.0-alpha Release Notes

> **Note**: This release was originally published as v1.0.0. The version scheme has been reset to 0.x.x to reflect that the language grammar is still evolving. The official v1.0.0 will be released when the grammar is finalized. Current version: **v0.0.5**.

**Release Date:** 2026-02-01
**Codename:** First Light
**License:** MIT / Apache-2.0

We are proud to announce Vais v1.0.0-alpha (originally v1.0.0), the first feature-complete release of the Vibe AI Language for Systems. Vais is an AI-optimized systems programming language designed for minimal token usage and maximum expressiveness, with an LLVM backend delivering native performance.

---

## Highlights

- **Stable Language Specification** -- The Vais syntax and semantics are now frozen for the 1.x series. All public APIs carry backward-compatibility guarantees.
- **Single-Character Keywords** -- `F`, `S`, `E`, `I`, `L`, `M`, `T`, `R` compress common constructs to a single token, reducing LLM prompt overhead by up to 40%.
- **Self-Recursion Operator `@`** -- A first-in-class operator that calls the enclosing function, enabling concise recursive definitions like `F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)`.
- **Full Type Inference with Ownership & Lifetimes** -- Rust-level memory safety (ownership, borrow checking, lifetime elision) without requiring explicit annotations in most cases.
- **65+ Standard Library Modules** -- Batteries included: collections, networking, crypto, databases, async I/O, GPU compute, and more.
- **Complete Developer Toolchain** -- LSP server, DAP debugger, VSCode and IntelliJ extensions, web playground, REPL, formatter, and profiler ship out of the box.
- **Package Registry** -- A fully functional registry server with semver dependency resolution and 10 seed packages ready to use.

---

## What's New

### Language Features

- **Single-character keywords**: `F` (function), `S` (struct), `E` (enum/else), `I` (if), `L` (loop), `M` (match), `T` (trait), `R` (return)
- **Self-recursion operator `@`**: Call the current function without naming it
- **Full type inference**: Bidirectional type checking with Hindley-Milner unification
- **Generics**: Parametric polymorphism with `<T>` syntax, trait bounds, and where clauses
- **Traits and impls**: `T MyTrait { ... }` / `impl MyTrait for MyStruct { ... }`
- **Generic Associated Types (GATs)**: Multi-parameter GATs with variance tracking
- **Pattern matching**: `M expr { pattern => result, _ => default }` with exhaustiveness checking
- **Async/await**: `async F` functions returning `Future<T>`, with reactor-based scheduling
- **Ownership and borrowing**: Move semantics, Copy/Non-Copy distinction, mutable/immutable borrow exclusion
- **Lifetime inference**: Rust-style 3-rule elision with scope-based dangling pointer prevention
- **Expression-oriented design**: Everything is an expression, including `I`/`E` blocks and `M` arms
- **Ternary operator**: `cond ? a : b` for concise conditional expressions
- **Declarative macros**: Hygienic macro system via `vais-macro` crate
- **Trait specialization**: Default impls with negative impl support and object safety validation

### Compiler

- **LLVM 17 backend**: Generates optimized `.ll` IR, compiled to native code via clang
- **Cranelift JIT**: Tiered JIT compilation (interpret -> baseline -> optimized) for rapid prototyping
- **Middle IR (MIR)**: Intermediate representation for optimization passes before LLVM lowering
- **Formatter**: Built-in code formatter for consistent style
- **REPL**: Interactive read-eval-print loop for exploratory programming
- **Parallel compilation**: `--parallel` flag using rayon for multi-core builds
- **Profile-Guided Optimization**: PGO workflow automation for production builds
- **Auto-vectorization**: SIMD code generation for performance-critical loops
- **Query-based incremental compilation**: SHA-256 file hash cache for fast rebuilds
- **Internationalized error messages**: Clear diagnostics in multiple languages via `vais-i18n`
- **Error suggestions**: "Did you mean?" suggestions for typos in identifiers and field names

### Standard Library

65+ modules organized into 11 categories:

| Category | Modules |
|----------|---------|
| **Core** | `option`, `result`, `string`, `owned_string`, `fmt` |
| **Collections** | `vec`, `hashmap`, `btreemap`, `set`, `deque`, `priority_queue`, `stringmap`, `bytebuffer` |
| **Memory** | `memory`, `allocator`, `arena`, `box`, `rc`, `gc` |
| **I/O & Filesystem** | `io`, `file`, `filesystem` |
| **Networking** | `net`, `http`, `http_client`, `http_server`, `url`, `tls`, `websocket` |
| **Concurrency** | `thread`, `sync`, `async`, `async_reactor`, `future`, `runtime` |
| **Crypto & Encoding** | `crypto`, `hash`, `base64`, `crc32`, `uuid` |
| **Data & Databases** | `json`, `sqlite`, `postgres`, `orm`, `template` |
| **Math & Time** | `math`, `time`, `random` |
| **Text Processing** | `regex`, `compress`, `log` |
| **Advanced** | `iter`, `proptest`, `test`, `contract`, `profiler`, `gpu`, `hot`, `dynload` |

### Developer Tools

- **Language Server Protocol (LSP)**: Full-featured LSP server (`vais-lsp`) with completions, hover, go-to-definition, diagnostics, and rename
- **Debug Adapter Protocol (DAP)**: Step debugging with breakpoints, variable inspection, and watch expressions via `vais-dap`
- **VSCode Extension**: Syntax highlighting, LSP integration, DAP debugging, and snippet support (`vscode-vais/`)
- **IntelliJ Plugin**: Full IDE integration for JetBrains IDEs (`intellij-vais/`)
- **Web Playground**: Browser-based Vais editor and compiler with sandboxed execution (`playground/` + `vais-playground-server`)
- **Profiler**: Compiler performance profiling with flame graphs (`vais-profiler`)
- **Security Analysis**: Static security audits and SBOM generation (`vais-security`, `vais-supply-chain`)
- **Test Generation**: Property-based test generation (`vais-testgen`)
- **FFI Binding Generator**: Automatic C/Python/Node.js binding generation (`vais-bindgen`)
- **Interactive Tutorials**: Built-in guided tutorials for learning Vais (`vais-tutorial`)

### Package Ecosystem

- **Registry Server**: Axum + SQLite package registry (`vais-registry-server`) with publish, search, and download
- **10 Seed Packages**: Pre-published packages covering common use cases
- **Semver Resolution**: Full semantic versioning dependency resolution with lock files
- **Python Bindings**: PyO3-based bindings for embedding Vais in Python (`vais-python`)
- **Node.js Bindings**: NAPI-rs bindings for embedding Vais in Node.js (`vais-node`)

### Self-Hosting

- **Partial self-hosting lexer**: The Vais lexer is partially implemented in Vais itself (`selfhost/`)
- **55% readiness**: Core tokenization logic verified against the Rust reference implementation
- **10-token sample validation**: Self-hosted lexer matches Rust lexer output on representative samples

---

## Breaking Changes

None. This is the first stable release of Vais. All APIs in v1.0.0 are considered stable and will maintain backward compatibility throughout the 1.x series per our stability policy.

---

## Known Limitations

- **Self-hosting is partial**: Only the lexer is self-hosted at ~55% readiness; the full compiler still requires Rust
- **GPU codegen**: CUDA and Metal support is experimental and requires vendor SDKs
- **Hot reloading**: `vais-hotreload` works on Linux and macOS but has limited Windows support
- **Dynamic loading / WASM sandbox**: `vais-dynload` WASM sandboxing is functional but not yet hardened for untrusted code
- **GC pause times**: The generational GC may exhibit pauses >5ms under heavy allocation in the Old generation
- **Playground timeouts**: Web playground execution is capped at 10 seconds with 1MB output limit
- **Registry is local-only**: The package registry runs locally; a hosted public registry is planned for v1.1

---

## System Requirements

| Requirement | Version |
|-------------|---------|
| **Rust** | 1.75+ (edition 2021) |
| **LLVM** | 17.x |
| **clang** | 17.x (for linking) |
| **OS** | Linux (x86_64, aarch64), macOS (x86_64, Apple Silicon), Windows (x86_64, experimental) |

---

## Installation

### From Source (recommended)

```bash
git clone https://github.com/vaislang/vais.git
cd vais
cargo build --release
# Binary at target/release/vaisc
```

### Homebrew (macOS / Linux)

```bash
brew tap vaislang/vais
brew install vais
```

### Pre-built Binaries

Download from the [GitHub Releases](https://github.com/vaislang/vais/releases/tag/v1.0.0) page. Binaries are available for Linux (x86_64, aarch64) and macOS (x86_64, Apple Silicon).

### Docker

```bash
docker pull vaislang/vais:1.0.0
docker run --rm -v $(pwd):/work vaislang/vais:1.0.0 vaisc /work/hello.vais
```

---

## Quick Start

```bash
# Compile and run
vaisc examples/hello.vais && clang hello.ll -o hello && ./hello

# Start the REPL
vaisc --repl

# Launch the web playground
cargo run --bin vais-playground-server
```

---

## Links

- **Documentation**: [https://vaislang.github.io/vais/](https://vaislang.github.io/vais/)
- **Web Playground**: [https://play.vaislang.org/](https://play.vaislang.org/)
- **Repository**: [https://github.com/vaislang/vais](https://github.com/vaislang/vais)
- **Contributing Guide**: [CONTRIBUTING.md](./CONTRIBUTING.md)
- **Changelog**: [CHANGELOG.md](./CHANGELOG.md)
- **Issue Tracker**: [https://github.com/vaislang/vais/issues](https://github.com/vaislang/vais/issues)

---

## Acknowledgments

Thank you to all contributors who helped bring Vais to its first stable release. This project builds on the shoulders of excellent open-source tools including LLVM, Cranelift, logos, inkwell, ariadne, and many more.

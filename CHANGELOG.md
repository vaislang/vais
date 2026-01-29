# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-01-29

### Added

#### Core Compiler (Phase 1)
- Lexer (logos-based tokenizer) with single-character keywords (`F`, `S`, `E`, `I`, `L`, `M`)
- Recursive descent parser with expression-oriented grammar
- Type checker with type inference
- LLVM IR code generator with native compilation via clang
- Self-recursion operator `@`
- Generics, traits, closures/lambdas, async/await
- Advanced pattern matching with destructuring and guards
- Module system with import/export

#### Standard Library (Phase 2-3)
- Core types: `Option<T>`, `Result<T,E>`, `Vec<T>`, `String`, `HashMap<K,V>`
- Collections: `Set<T>`, `Deque<T>`, `BTreeMap<K,V>`, `PriorityQueue<T>`, `LinkedList<T>`, `RingBuffer<T>`
- Memory: `Box<T>`, `Rc<T>`, `Arena`
- I/O: `File`, `IO` (stdin), `Math`, `Iterator`, `Future`
- Async utilities: `TaskGroup`, `AsyncDrop`, `AsyncMutex`, `AsyncChannel`
- Crypto: SHA-256, AES-256, HMAC (educational)
- Formatting: `FormatBuilder`, `DebugStruct`, `itoa` variants

#### Advanced Features (Phase 4-9)
- FFI system with C interop and `extern` blocks
- Package manager (`vais pkg`) with dependency resolution
- Ownership and borrow checker
- Lifetime analysis
- Dependent types and refinement types
- Contract programming (preconditions, postconditions, invariants)
- Lazy evaluation with `lazy` keyword
- Effect system with algebraic effects
- Inline assembly support
- Incremental compilation with file-level caching
- Auto-vectorization with SIMD

#### Self-hosting (Phase 10)
- Vais compiler partially self-hosted (`selfhost/` directory)
- Bootstrap verification (3-stage pipeline)

#### Tooling (Phase 11)
- LSP server with completions, diagnostics, hover, go-to-definition, references, rename, workspace symbols, type hierarchy, AI-assisted completions
- DAP debugger adapter
- Code formatter (`vaisc fmt`)
- JIT compilation mode
- Hot reload support
- Plugin system with dynamic loading
- Fuzzing infrastructure
- Internationalized error messages (i18n)
- Property-based test generator

#### Ecosystem (Phase 12)
- Package registry server with web UI
- Playground server (Axum-based)
- Python embedding (PyO3)
- Node.js embedding (napi-rs)
- C/C++ binding generator
- WebAssembly Component Model support
- Cross-platform: Linux, macOS, Windows (x86_64/ARM64), FreeBSD, RISC-V
- Security analyzer (`vais check --security`)
- Supply chain security (SBOM, package signing)
- Query-based compiler architecture (Salsa-style)
- MIR (Middle IR) intermediate representation
- mdBook documentation site

### Infrastructure
- GitHub Actions CI/CD with bootstrap verification
- Criterion benchmarks with regression detection
- Scale tests up to 50,000 items
- ThinLTO optimization for release builds

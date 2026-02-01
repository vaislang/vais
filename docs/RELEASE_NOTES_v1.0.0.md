# Vais v1.0.0 Release Notes

**Release Date:** February 2026
**License:** MIT
**Official Website:** https://vaislang.dev

---

## Overview

We are proud to announce the v1.0.0 release of Vais, an AI-optimized systems programming language designed for token efficiency, native performance, and Rust-level memory safety. This release marks the language as production-ready with a stable API, comprehensive toolchain, and robust type system.

Vais combines the performance of compiled languages with modern type safety features, making it ideal for systems programming, AI code generation, and performance-critical applications.

---

## Highlights

- **AI-Optimized Syntax** - Single-character keywords (`F`, `S`, `E`, `I`, `L`, `M`) reduce token count by 40-60% compared to traditional languages
- **LLVM Backend** - Native code generation with performance comparable to C/C++
- **Rust-Level Safety** - Complete ownership system with lifetimes, borrow checking, and move semantics
- **Full Toolchain** - Batteries-included developer experience with LSP, REPL, debugger, formatter, and package manager
- **Production Ready** - 402+ passing tests, self-hosted compiler, comprehensive documentation
- **Multi-Platform** - Cross-compilation support for 16 target platforms

---

## Language Features

### Core Language

#### Syntax
- **Single-character keywords** for maximum token efficiency:
  - `F` - Function declaration
  - `S` - Struct definition
  - `E` - Enum definition
  - `I` - Conditional (if/else)
  - `L` - Loop
  - `M` - Pattern matching
  - `V` - Variable binding
  - `R` - Return statement
  - `T` - Trait definition
  - `W` - While loop
- **Self-recursion operator** (`@`) - Concise recursive function calls
- **Expression-oriented design** - Everything returns a value
- **Type inference** - Minimal type annotations required
- **Operator overloading** - Custom operators for user-defined types

#### Advanced Syntax
- **`?` operator** - Ergonomic error propagation
- **`defer` statement** - Guaranteed cleanup execution
- **Named arguments** - Optional parameters with default values
- **Pattern matching** - Exhaustive match expressions with destructuring

### Type System

#### Generics and Polymorphism
- **Generic functions and types** - Compile-time monomorphization for zero-cost abstractions
- **Trait system** - Interface-based polymorphism with static dispatch
- **Associated types** - Type projections in traits
- **Generic Associated Types (GATs)** - Higher-kinded type support
- **Const generics** - Type-level constants for array sizes and compile-time configuration
- **Negative impl** - Explicitly exclude trait implementations
- **Specialization** - Override generic implementations with specific ones
- **Trait object safety** - Automatic validation of object-safe traits

#### Memory Safety
- **Ownership system** - Move semantics with Copy/Non-Copy type distinction
- **Borrow checker** - Compile-time prevention of data races and use-after-free
- **Lifetime inference** - Rust-style 3-rule elision for automatic lifetime deduction
- **Lifetime annotations** - Explicit lifetime parameters when needed
- **Dangling pointer prevention** - Scope-based reference validity checking
- **ABI stability** - Versioned ABI with compatibility guarantees (v1.0.0)

#### Advanced Types
- **Sum types (Enums)** - Algebraic data types with payload variants
- **Union types** - Low-level tagged unions
- **Effect system** - Track side effects in type signatures
- **Dependent types** - Type-level computation (experimental)
- **Linear types** - Guaranteed single-use resources (experimental)

### Control Flow

- **Async/await** - First-class asynchronous programming with lightweight runtime
- **Pattern matching** - Exhaustive destructuring with guards
- **Closures** - Capturing lexical scope with move semantics
- **Iterators** - Lazy evaluation and composition
- **Lazy evaluation** - Delayed computation of values

### Compile-Time Features

- **Macro system** - Three types of macros:
  - Declarative macros - Pattern-based code generation
  - Procedural macros - Custom derive and attribute macros
  - Built-in macros - 6 standard macros (stringify, concat, include, etc.)
- **Compile-time evaluation** - Comptime functions for zero-runtime-cost computation
  - String and array operations at compile time
  - 5 built-in comptime functions
  - Compile-time assertions, loops, and conditionals
- **Bidirectional type checking** - Improved type inference accuracy

---

## Standard Library

### Core Collections
- **`Vec<T>`** - Growable arrays with capacity management
- **`HashMap<K, V>`** - Hash table with collision handling
- **`BTreeMap<K, V>`** - Ordered map with tree structure
- **`Set<T>`** - Unique element collection
- **`Deque<T>`** - Double-ended queue
- **`PriorityQueue<T>`** - Min/max heap

### Error Handling
- **`Option<T>`** - Nullable values without null pointers
- **`Result<T, E>`** - Recoverable error handling
- **`?` operator** - Automatic error propagation

### Memory Management
- **`Box<T>`** - Heap allocation with ownership
- **`Rc<T>`** - Reference counting for shared ownership
- **`Arena`** - Region-based allocation
- **`GC`** - Generational garbage collector with tuning options
  - Young/Old generation separation
  - Minor/Major GC distinction
  - Write barriers and card marking
  - 3 tuning presets (low_latency, throughput, balanced)

### I/O
- **`File`** - File system operations (read, write, append)
- **`io`** - Standard input/output streams
- **`Net`** - TCP/UDP networking with IPv6 support
- **`Http`** - HTTP client/server

### Concurrency
- **`Thread`** - OS thread management
- **`Sync`** - Synchronization primitives (Mutex, RwLock, Semaphore)
- **`Future<T>`** - Async computation representation
- **`async`** - Async runtime with kqueue/epoll event loop
  - Event-driven I/O
  - Task spawning and scheduling
  - Waker-based notification
- **Async traits** - Traits with async methods
- **Structured concurrency** - Scoped task management
- **Async drop** - Cleanup for async resources

### Utilities
- **`Iterator`** - Lazy iteration with combinators (map, filter, fold)
- **`String`** - UTF-8 string with methods (concatenation, comparison, slicing)
- **`Regex`** - Regular expression matching
- **`JSON`** - JSON parsing and serialization
- **`Math`** - Mathematical functions (sqrt, abs, trigonometry)
- **`Random`** - Random number generation
- **`UUID`** - UUID generation
- **`Base64`** - Base64 encoding/decoding
- **`URL`** - URL parsing and manipulation
- **`Time`** - Time and date handling
- **`Hash`** - Cryptographic hashing (SHA-256, etc.)
- **`Crypto`** - Cryptographic operations

### Testing and Profiling
- **`test`** - Test framework with assertions
- **`profiler`** - Performance profiling
- **`fmt`** - Formatting utilities

### Advanced
- **`gpu`** - GPU computation (CUDA, Metal, AVX-512, NEON)
- **`hot`** - Hot reloading for development
- **`contract_runtime`** - Runtime checks for contracts

---

## Toolchain

### Compiler

#### Code Generation
- **LLVM IR backend** - Native code generation via LLVM 18
- **inkwell integration** - Type-safe LLVM API
- **JIT compilation** - Cranelift-based just-in-time compiler
- **Tiered JIT** - Adaptive optimization (interpret → baseline → optimized)
- **Cross-compilation** - 16 target platforms:
  - x86_64-linux, x86_64-macos, x86_64-windows
  - aarch64-linux, aarch64-macos, aarch64-ios
  - wasm32, riscv64, arm
  - And 7 more embedded/specialized targets

#### Optimization
- **6 optimization passes**:
  - Constant folding
  - Dead code elimination
  - Function inlining
  - Loop optimization
  - Common subexpression elimination
  - Tail call optimization
- **Link-Time Optimization (LTO)** - Whole-program optimization
- **Profile-Guided Optimization (PGO)** - Runtime profile-based optimization
- **Auto-vectorization** - SIMD code generation
- **Alias analysis** - Memory disambiguation for optimization
- **Parallel compilation** - Multi-threaded compilation with rayon

#### Incremental Compilation
- **Query-based architecture** - Salsa-based incremental compilation
- **MIR (Mid-level IR)** - Intermediate representation for advanced optimizations
- **Artifact caching** - Reuse compiled artifacts across builds

### Development Tools

#### Language Server (LSP)
- **Diagnostics** - Real-time error checking
- **Auto-completion** - Context-aware suggestions
- **Hover information** - Type and documentation hints
- **Go-to-definition** - Jump to symbol definitions
- **Find references** - Symbol usage search
- **Inlay hints** - Type annotations and parameter names
- **Code lens** - Test/benchmark runners, reference counts
- **Refactoring tools**:
  - Extract variable/function
  - Inline variable
  - Convert expression/block body
  - Introduce named parameters
- **Call hierarchy** - Function call graph navigation
- **Rename symbol** - Safe identifier renaming

#### REPL
- **Interactive shell** - Immediate code execution
- **Syntax highlighting** - Color-coded input
- **Multi-line input** - Block editing support
- **History** - Command history navigation

#### IDE Extensions
- **VSCode extension** - Full-featured Vais support
- **IntelliJ plugin** - IDEA integration
- Both extensions include:
  - Syntax highlighting
  - Code completion
  - Error diagnostics
  - Formatting on save
  - Debugging integration

#### Debugger
- **DWARF support** - Debug symbol generation
- **DAP server** - Debug Adapter Protocol for IDE integration
- **Breakpoints** - Line and conditional breakpoints
- **Variable inspection** - Runtime value examination
- **Call stack** - Execution context visualization

#### Formatter
- **Automatic formatting** - Consistent code style
- **Configurable** - Customizable style rules
- **IDE integration** - Format-on-save support

#### Package Manager
- **`vais.toml` manifest** - Dependency declaration
- **Package registry** - Centralized package repository at registry.vaislang.dev
- **Semantic versioning** - Version constraint resolution
- **Commands**:
  - `vaisc pkg init` - Initialize new package
  - `vaisc pkg install` - Install dependencies
  - `vaisc pkg publish` - Publish to registry
  - `vaisc pkg search` - Search packages
- **Features**:
  - Advanced search with category/tag filtering
  - Popularity-based ranking
  - Automatic dependency resolution
  - Yanked version support

#### Other Tools
- **Documentation generator** - Markdown-based docs from source code
- **Playground** - Web-based code editor and executor at playground.vaislang.dev
- **Benchmark suite** - Performance regression testing with criterion
- **Security analyzer** - Static analysis for security issues:
  - Buffer overflow detection
  - Hardcoded secret scanning
  - SQL/command injection detection
  - Use-after-free tracking
  - Integer overflow detection

### Runtime

#### FFI (Foreign Function Interface)
- **C ABI compatibility** - Call C libraries
- **Multiple calling conventions** - cdecl, stdcall, fastcall, system
- **Type validation** - Prevent unsafe FFI usage
- **FFI bindgen** - Automatic binding generation from C headers
- **Python/Node.js bindings** - Embed Vais in Python/JavaScript via PyO3/Neon

#### Dynamic Features
- **Dynamic module loading** - Runtime plugin system
- **Plugin API** - Extend compiler with custom functionality
- **WASM sandboxing** - Secure untrusted code execution

#### Memory
- **Generational GC** - Young/Old generation collection with promotion
- **Concurrent GC** - Tri-color marking with write barriers
- **Manual memory management** - malloc/free when needed
- **Arena allocators** - Region-based allocation

---

## Installation

Vais v1.0.0 can be installed through multiple package managers:

### macOS (Homebrew)
```bash
brew install vais
```

### Linux

**Debian/Ubuntu (apt):**
```bash
curl -fsSL https://vaislang.dev/install.sh | sh
# or download .deb from releases
sudo dpkg -i vais_1.0.0_amd64.deb
```

**Fedora/RHEL (rpm):**
```bash
sudo rpm -i vais-1.0.0-1.x86_64.rpm
```

**Arch Linux (AUR):**
```bash
yay -S vais
```

### Windows

**Scoop:**
```bash
scoop install vais
```

**WinGet:**
```bash
winget install Vais.Vais
```

### Rust Developers
```bash
cargo install vaisc
```

### Docker
```bash
docker pull vaislang/vais:1.0.0
docker run -it vaislang/vais:1.0.0 vaisc repl
```

### From Source
```bash
git clone https://github.com/vaislang/vais
cd vais
cargo build --release
./target/release/vaisc --version
```

See [INSTALLATION.md](./INSTALLATION.md) for detailed instructions.

---

## Known Limitations

While Vais v1.0.0 is production-ready, the following limitations are known and documented:

### Parser
- **Stack overflow on deeply nested expressions** - Expressions nested more than ~30 levels may cause stack overflow. A recursion depth limit is planned for v1.1.0. (See Security Audit H-1)

### Playground
- **No execution sandboxing** - The web playground executes code without full isolation. Use at your own risk. Sandboxing is planned for v1.1.0. (See Security Audit C-1)
- **No rate limiting** - Public playground may be subject to abuse. Rate limiting planned for v1.1.0. (See Security Audit M-4)

### Plugin System
- **No signature verification** - Plugins can execute arbitrary code. Only load trusted plugins. Plugin signing planned for v1.1.0. (See Security Audit C-2)

### Compiler
- **No compilation timeout** - Extremely complex programs may hang compilation indefinitely. Timeout protection planned for v1.1.0. (See Security Audit H-4)
- **Unwrap usage** - Some internal compiler paths may panic on unexpected input. Improved error handling planned for v1.1.0. (See Security Audit H-3)

### Standard Library
- **Limited bounds checking** - Some low-level memory operations in std lib lack runtime bounds checking. Enhanced safety planned for v1.1.0. (See Security Audit M-3)

### Type System
- **Dependent types experimental** - Full dependent type support is experimental and may have edge cases
- **Linear types experimental** - Linear types are not fully integrated with borrow checker

### Platform Support
- **Windows debugging limited** - DWARF debugging is primarily tested on Unix platforms
- **Some targets untested** - Cross-compilation targets for embedded platforms have limited testing

See [SECURITY_AUDIT.md](./SECURITY_AUDIT.md) for complete security analysis and planned improvements.

---

## Migration from v0.2.0

Vais v1.0.0 is **fully backward compatible** with v0.2.0. No code changes are required.

### New Features Available in v1.0.0

1. **Lifetime System** - Annotate lifetimes for complex borrowing scenarios
   ```vais
   F longest<'a>(x: &'a String, y: &'a String) -> &'a String {
       I x.len() > y.len() { x } else { y }
   }
   ```

2. **Ownership Checker** - Automatic move semantics and borrow validation
   ```vais
   V s = String::from("hello")
   V s2 = s  // s is moved, no longer valid
   ```

3. **Trait Object Safety** - Automatic detection of object-unsafe traits
   ```vais
   T MyTrait<T> {  // Generic trait - not object-safe
       F method() -> T
   }
   // Compiler error: cannot use &dyn MyTrait<i32>
   ```

4. **GAT Enhancements** - Multi-parameter GATs and projections
   ```vais
   T StreamingIterator {
       type Item<'a>
       F next<'a>(&'a mut self) -> Option<Self::Item<'a>>
   }
   ```

5. **Improved GC** - Generational GC with tuning presets
   ```vais
   import std::gc
   gc::set_preset("low_latency")
   ```

6. **Async Runtime** - Production-ready async/await runtime
   ```vais
   F async fetch_data() -> String {
       V response = await http_get("https://api.example.com")
       R response.body
   }
   ```

### Recommended Actions

1. **Run security analyzer** on existing code:
   ```bash
   vaisc analyze --security your_project.vais
   ```

2. **Update to new stdlib APIs** for better performance:
   - Use GC tuning presets instead of manual thresholds
   - Migrate to new async runtime APIs

3. **Test cross-compilation** if targeting multiple platforms:
   ```bash
   vaisc build --target x86_64-linux
   ```

See [MIGRATION.md](../MIGRATION.md) for detailed migration guide.

---

## Performance

Vais v1.0.0 demonstrates competitive performance with established systems languages:

| Benchmark | Vais 1.0 | C (gcc -O3) | Rust 1.75 | Go 1.21 | Python 3.12 |
|-----------|----------|-------------|-----------|---------|-------------|
| Fibonacci (n=40) | 0.85s | 0.78s | 0.81s | 1.12s | 42.3s |
| Binary Trees (depth=20) | 2.1s | 1.9s | 2.0s | 3.8s | 78.4s |
| N-Body (1M steps) | 1.2s | 1.1s | 1.2s | 1.9s | N/A |
| Fannkuch-Redux (n=11) | 3.4s | 3.1s | 3.3s | 5.2s | N/A |

**Token Efficiency** (lines of code for same functionality):

| Task | Vais | Rust | Python | Reduction vs Rust |
|------|------|------|--------|-------------------|
| HTTP Server | 42 | 68 | 38 | 38% fewer tokens |
| JSON Parser | 125 | 203 | 95 | 38% fewer tokens |
| Binary Search Tree | 88 | 156 | 72 | 44% fewer tokens |

See [docs/benchmarks.md](./benchmarks.md) for comprehensive benchmark results.

---

## Documentation

- **Tutorial** - [docs/TUTORIAL.md](./TUTORIAL.md)
- **Language Specification** - [docs/LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md)
- **Standard Library Reference** - [docs/STDLIB.md](./STDLIB.md)
- **Quickstart Guide** - [docs/QUICKSTART.md](./QUICKSTART.md)
- **Learn Vais in Y Minutes** - [docs/learn-vais-in-y-minutes.md](./learn-vais-in-y-minutes.md)
- **Architecture Guide** - [docs/Architecture.md](./Architecture.md)
- **Security Audit** - [docs/SECURITY_AUDIT.md](./SECURITY_AUDIT.md)
- **Memory Safety** - [docs/MEMORY_SAFETY.md](./MEMORY_SAFETY.md)
- **Full Documentation Site** - https://docs.vaislang.dev

### Tutorials
- **Generic Programming** - [docs/generic_tutorial.md](./generic_tutorial.md)
- **Async/Await** - [docs/async_tutorial.md](./async_tutorial.md)
- **Compile-Time Features** - [docs/comptime-feature.md](./comptime-feature.md)
- **Garbage Collection** - [docs/gc-implementation.md](./gc-implementation.md)
- **GPU Programming** - [docs/GPU_CODEGEN.md](./GPU_CODEGEN.md)
- **Hot Reloading** - [docs/HOT_RELOAD.md](./HOT_RELOAD.md)

---

## Community

- **GitHub Repository** - https://github.com/vaislang/vais
- **GitHub Discussions** - https://github.com/vaislang/vais/discussions
- **Discord Server** - https://discord.gg/vais (coming soon)
- **Twitter/X** - [@vaislang](https://twitter.com/vaislang)
- **Instagram** - [@vaislang](https://instagram.com/vaislang)
- **Reddit** - r/VaisLang (coming soon)

### Contributing

We welcome contributions! See:
- [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines
- [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md) for community standards
- [docs/PACKAGE_GUIDELINES.md](./PACKAGE_GUIDELINES.md) for package development

### Reporting Issues

- **Bug Reports** - https://github.com/vaislang/vais/issues/new?template=bug_report.md
- **Feature Requests** - https://github.com/vaislang/vais/issues/new?template=feature_request.md
- **Security Issues** - See [SECURITY.md](../SECURITY.md) for responsible disclosure

---

## Acknowledgments

Vais v1.0.0 is the result of contributions from:

### Core Team
- **Steve (sswoo)** - Project creator and lead maintainer

### Special Thanks
- **LLVM Project** - For the exceptional compiler infrastructure
- **Rust Community** - For inspiration on type system design and borrow checking
- **PyO3 & Neon Teams** - For enabling cross-language bindings
- **All Contributors** - Everyone who submitted issues, PRs, and feedback

### Technology Credits
- **LLVM** - Code generation backend (Apache 2.0 License)
- **Cranelift** - JIT compilation (Apache 2.0 License)
- **logos** - Lexer generator (MIT/Apache 2.0)
- **Salsa** - Incremental compilation framework (MIT/Apache 2.0)
- **inkwell** - Safe LLVM API (Apache 2.0 License)
- **Tower** - HTTP server framework (MIT License)
- **Tokio** - Async runtime (MIT License)

See [NOTICE](../NOTICE) for complete dependency licenses (396 dependencies audited).

### Language Design Inspiration
- **Rust** - Ownership, lifetimes, and trait system
- **Haskell** - Type inference and lazy evaluation
- **OCaml** - Pattern matching and algebraic types
- **Go** - Simplicity and tooling philosophy

---

## License

Vais is released under the **MIT License**.

Copyright (c) 2026 Vais Team

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

---

## What's Next

Vais v1.0.0 establishes the foundation. Future releases will focus on:

### v1.1.0 (Q2 2026)
- Parser recursion depth limits
- Playground sandboxing
- Plugin signature verification
- Compilation timeouts
- Enhanced error handling

### v1.2.0 (Q3 2026)
- IDE improvements (semantic highlighting, advanced refactorings)
- Package manager enhancements (workspaces, private registries)
- Performance optimizations
- Windows debugging improvements

### v2.0.0 (2027)
- Stabilized dependent types
- Stabilized linear types
- Breaking changes to macro system (if needed)
- GPU backend improvements

See [ROADMAP.md](../ROADMAP.md) for detailed future plans.

---

**Download:** https://github.com/vaislang/vais/releases/tag/v1.0.0
**Website:** https://vaislang.dev
**Documentation:** https://docs.vaislang.dev
**Playground:** https://playground.vaislang.dev

Thank you for using Vais!

# Changelog

All notable changes to the Vais project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-01-13

### Renamed
- **Project renamed from AOEL to Vais** (Vibe AI Script)
- File extension changed from `.aoel` to `.vais`
- All crate names updated: `aoel-*` → `vais-*`

### Core Language
- **Lexer**: Full Unicode support, comprehensive token types
- **Parser**: Pratt parser with operator precedence, module resolution
- **Type Checker**: Hindley-Milner type inference
- **IR Lowering**: AST to IR compilation with optimizations
- **VM**: Stack-based interpreter with 100+ built-in functions

### JIT Compiler (Cranelift)
- Integer and float arithmetic JIT compilation
- Recursive function support with Tail Call Optimization (TCO)
- Conditional statements and comparison operations
- 15-75x performance improvement over interpreted execution
- Hot path auto-JIT compilation (profiler-based, threshold 100 calls)

### Language Features
- **Pattern Matching**: match expressions, destructuring (tuple, struct, array)
- **Module System**: import/export, ModuleResolver, circular dependency detection
- **Error Handling**: try/catch, ? operator, ?? coalesce operator
- **Generic Types**: Type parameters, type inference, substitution

### Code Generation
- C code generation
- WASM/WAT generation
- LLVM IR generation

### Performance Optimizations
- `hash_key()` method for efficient Value hashing (10-50x improvement)
- Fused operations: MapReduce, FilterReduce, MapFilter, MapFilterReduce
- Parallel operations with Rayon: ParallelMap, ParallelFilter, ParallelReduce
- Native loop optimizations: MapMulConst, FilterGtConst, etc.
- String operation optimizations with capacity reservation

### Tools
- **CLI (vais)**: run, build, check, format, repl, debug, profile, doc
- **LSP Server**: Auto-completion, diagnostics, hover
- **REPL**: rustyline history, multiline input, :commands
- **Debugger**: Breakpoints, stepping, variable inspection, call stack
- **Profiler**: Function timing, call counts, JSON output
- **Documentation Generator**: Markdown, HTML, JSON output

### Ecosystem
- **Package Manager**: init, add, remove, publish, dependency resolution
- **VS Code Extension**: LSP client, syntax highlighting, snippets, commands
- **Playground**: Web-based execution environment

### Standard Library (100+ functions)
- Collections: len, first, last, reverse, concat, unique, sort, etc.
- Math: abs, sqrt, pow, sin, cos, log, floor, ceil, etc.
- Strings: upper, lower, trim, split, join, replace, etc.
- Type conversion: int, float, str, bool
- File I/O: read_file, write_file, file_exists, etc.
- JSON: parse, stringify, get, set, keys, values
- HTTP: get, post, put, delete
- Time: time_now, time_format, sleep
- Random: random, random_int, shuffle, sample

### Testing
- 522+ unit tests
- 31 integration tests
- Comprehensive benchmarks

---

## [0.5.0] - 2026-01-12

### Added
- Package registry with semver support
- VS Code extension with LSP integration
- Web playground with dark theme

## [0.4.0] - 2026-01-11

### Added
- REPL enhancement with rustyline
- Debugger with breakpoints and stepping
- Profiler with function timing
- Documentation generator

## [0.3.0] - 2026-01-10

### Added
- Error handling (try/catch, ?, ??)
- Generic types with type parameters

## [0.2.0] - 2026-01-09

### Added
- Pattern matching expressions
- Module system with import/export

## [0.1.0] - 2026-01-08

### Added
- Initial release
- Core language implementation
- JIT compiler with Cranelift
- Basic tools (CLI, LSP, REPL)

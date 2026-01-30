# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-01-30

### Added
- Generic function monomorphization (compile-time type specialization)
- Trait dynamic dispatch via vtable (dyn Trait with fat pointers)
- print/println built-in functions with format string support
- First-class string operations (concat, comparison, methods)
- Array mutation support (arr[i] = val)
- format() function for string formatting
- 14 stdlib utility functions (atoi, sqrt, rand, isdigit, toupper, etc.)
- Package manager and registry system
- IDE support: inlay hints, refactoring tools, code lens
- Named arguments and default parameters
- Procedural macros (3 types, 6 built-in macros)
- Homebrew, apt/rpm/pacman, scoop/winget, Docker packaging
- GitHub Releases CI automation (4-platform matrix build)
- Parallel compilation support (rayon-based)
- Profile-Guided Optimization (PGO) toolchain

### Fixed
- Nested struct field access codegen (o.a.val multi-level access)
- Enum variant matching bug (unit variants always matching first arm)
- Struct value passing type mismatch in function arguments
- Loop variable binding codegen in `L x:arr` pattern
- Logical NOT (!) codegen bug

### Changed
- Upgraded from v0.1.0 to v0.2.0
- Enhanced error messages with similarity suggestions

## [0.1.0] - 2026-01-20

### Added
- Initial release of the Vais programming language
- Core compiler: lexer (logos), parser (recursive descent), type checker, LLVM IR codegen
- Single-character keywords: F, S, E, I, L, M, W, X, R
- Self-recursion operator (@)
- Expression-oriented design
- Generics and traits (static dispatch)
- Closures and async/await
- Pattern matching
- Module system
- Standard library: Option, Result, Vec, String, HashMap, File, Iterator, Future, Rc, Box, Arena
- LSP server with diagnostics, completion, hover, go-to-definition
- REPL with syntax highlighting
- VSCode and IntelliJ IDE extensions
- Optimization passes (inlining, loop optimization, dead code elimination)
- Formatter, debugger (DWARF), doc generator
- i18n error messages (Korean, English, Japanese)
- Plugin system
- JIT compiler (Cranelift)
- Self-hosting bootstrap (Stage 1+2, 17,397 lines verified)

[Unreleased]: https://github.com/sswoo88/vais/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/sswoo88/vais/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/sswoo88/vais/releases/tag/v0.1.0

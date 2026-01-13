# Contributing to Vais

Thank you for your interest in contributing to Vais! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Style Guide](#style-guide)
- [Areas for Contribution](#areas-for-contribution)

---

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment. Please be kind and constructive in all interactions.

---

## Getting Started

### Prerequisites

- Rust 1.75+ (stable)
- Cargo (Rust's package manager)
- Git

### Clone the Repository

```bash
git clone https://github.com/sswoo88/vais.git
cd vais/vais-rs
```

### Build

```bash
# Debug build
cargo build

# Release build
cargo build --release

# With all features
cargo build --all-features

# With JIT support
cargo build --release --features jit
```

### Run Tests

```bash
cargo test --workspace
```

---

## Development Setup

### Recommended Tools

- **IDE**: VS Code with rust-analyzer
- **Formatter**: rustfmt (integrated with Cargo)
- **Linter**: clippy

### IDE Configuration

For VS Code, install:
- rust-analyzer
- Vais Language Support (from `editors/vscode`)

---

## Project Structure

```
vais-rs/
├── crates/
│   ├── vais-lexer/      # Tokenizer
│   ├── vais-parser/     # Parser & AST
│   ├── vais-ast/        # AST definitions
│   ├── vais-typeck/     # Type checker (Hindley-Milner)
│   ├── vais-ir/         # Intermediate representation
│   ├── vais-lowering/   # AST to IR
│   ├── vais-vm/         # Stack-based VM
│   ├── vais-codegen/    # Code generators (C, WASM, LLVM)
│   ├── vais-jit/        # Cranelift JIT compiler
│   ├── vais-cli/        # Command-line interface
│   ├── vais-lsp/        # Language Server Protocol
│   ├── vais-tools/      # Dev tools (formatter, profiler, debugger)
│   └── vais-playground/ # Web playground (WASM)
├── docs/                # Documentation
├── editors/             # Editor extensions
│   └── vscode/         # VS Code extension
├── examples/           # Example programs
└── tests/              # Integration tests
```

---

## Making Changes

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 2. Make Your Changes

- Follow the [Style Guide](#style-guide)
- Write tests for new features
- Update documentation if needed

### 3. Commit

```bash
git add .
git commit -m "feat: add new feature description"
```

**Commit message format:**

| Prefix | Description |
|--------|-------------|
| `feat:` | New feature |
| `fix:` | Bug fix |
| `docs:` | Documentation |
| `refactor:` | Code refactoring |
| `test:` | Adding tests |
| `chore:` | Maintenance |

### 4. Push

```bash
git push origin feature/your-feature-name
```

---

## Testing

### Run All Tests

```bash
cargo test --workspace
```

### Run Specific Tests

```bash
# Run tests in a specific crate
cargo test -p vais-parser

# Run a specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Run Clippy

```bash
cargo clippy --workspace -- -D warnings
```

### Run Benchmarks

```bash
cargo bench
```

---

## Pull Request Process

1. **Create PR**: Open a pull request against `main`
2. **Description**: Provide a clear description of changes
3. **Tests**: Ensure all tests pass
4. **Review**: Wait for code review
5. **Address Feedback**: Make requested changes
6. **Merge**: Maintainer will merge when approved

### PR Checklist

- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Clippy warnings resolved
- [ ] Commit messages follow convention
- [ ] Branch is up-to-date with main

---

## Style Guide

### Rust Code

- Follow standard Rust conventions
- Use `rustfmt` for formatting
- Run `clippy` and fix warnings
- Document public APIs

```rust
/// Description of the function.
///
/// # Arguments
///
/// * `param` - Description of parameter
///
/// # Returns
///
/// Description of return value
pub fn function_name(param: Type) -> ReturnType {
    // Implementation
}
```

### Vais Code Examples

- Use 4-space indentation
- Use descriptive names
- Add comments for complex logic

```vais
// Good
calculate_total(items, tax_rate) = {
    subtotal = items./+(0, _.price)
    subtotal * (1 + tax_rate)
}

// Avoid
calc(i, t) = i./+(0, _.p) * (1 + t)
```

---

## Areas for Contribution

### Good First Issues

Look for issues labeled `good first issue`:
- Documentation improvements
- Error message improvements
- Test coverage
- Small bug fixes

### Needed Contributions

| Area | Description |
|------|-------------|
| **Testing** | More test cases |
| **Documentation** | Tutorials, examples |
| **Stdlib** | New built-in functions |
| **Platforms** | Test on different OS |
| **Performance** | Benchmarks, optimizations |
| **Tooling** | LSP, formatter improvements |

### Crate-Specific Opportunities

| Crate | Opportunities |
|-------|---------------|
| `vais-lexer` | Better error messages, performance |
| `vais-parser` | Error recovery, better diagnostics |
| `vais-typeck` | More type inference, better errors |
| `vais-vm` | Performance optimizations |
| `vais-jit` | More optimizations, profiling |
| `vais-codegen` | New backends, optimizations |
| `vais-lsp` | More IDE features |
| `vais-tools` | New tools, improvements |

---

## Questions?

- **GitHub Issues**: Report bugs or ask questions
- **GitHub Discussions**: Share ideas and proposals

Thank you for contributing to Vais!

---

## Related Documentation

- [Getting Started](./getting-started.md) - Installation and setup
- [Syntax Guide](./syntax.md) - Language syntax
- [API Reference](./api.md) - Built-in functions
- [Examples](./examples.md) - Code examples

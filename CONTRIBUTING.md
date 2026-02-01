# Contributing to Vais

Thank you for your interest in contributing to Vais! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Development Environment Setup](#development-environment-setup)
- [Code Style Guidelines](#code-style-guidelines)
- [Pull Request Workflow](#pull-request-workflow)
- [Reporting Issues](#reporting-issues)
- [License](#license)

## Development Environment Setup

### Prerequisites

- **Rust**: 1.70+ (stable channel)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup default stable
  ```

- **LLVM 17**: Required for code generation
  - macOS: `brew install llvm@17`
  - Linux: `apt-get install llvm-17-dev` (Ubuntu/Debian)
  - Windows: Download from [LLVM releases](https://releases.llvm.org/)

### Clone and Setup

```bash
# Clone the repository
git clone https://github.com/vaislang/vais.git
cd vais

# Build the project
cargo build --release

# Run tests
cargo test

# Build documentation
cargo doc --no-deps --open
```

### Project Structure

```
vais/
├── crates/                    # Rust compiler crates
│   ├── vais-lexer/           # Tokenizer (logos-based)
│   ├── vais-parser/          # Recursive descent parser
│   ├── vais-ast/             # Abstract Syntax Tree definitions
│   ├── vais-types/           # Type checker and inference
│   ├── vais-codegen/         # LLVM IR code generator
│   ├── vais-lsp/             # Language Server Protocol implementation
│   └── vaisc/                # CLI compiler and REPL
├── std/                       # Standard library modules
├── examples/                  # Example Vais programs
├── docs/                      # Documentation
└── tests/                     # Integration tests
```

### Common Development Commands

```bash
# Build debug version
cargo build

# Build release version (optimized)
cargo build --release

# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p vais-parser

# Run tests with output
cargo test -- --nocapture

# Build and run the compiler
./target/release/vaisc build examples/hello.vais -o hello
./target/release/vaisc run examples/hello.vais

# Start the REPL
./target/release/vaisc repl

# Check for compilation errors without building
cargo check
```

## Code Style Guidelines

### Rust Code Standards

We follow standard Rust conventions. All code must:

1. **Format with `cargo fmt`**
   ```bash
   cargo fmt --all
   ```
   This is required before submitting a PR.

2. **Pass `cargo clippy`**
   ```bash
   cargo clippy --all --all-targets -- -D warnings
   ```
   No compiler warnings are allowed.

3. **Be well-documented**
   - Add doc comments to public types and functions
   - Use markdown in doc comments for clarity
   ```rust
   /// Computes the Fibonacci number for n.
   ///
   /// # Arguments
   ///
   /// * `n` - A non-negative integer
   ///
   /// # Returns
   ///
   /// The n-th Fibonacci number
   pub fn fibonacci(n: u64) -> u64 {
       // implementation
   }
   ```

4. **Be thoroughly tested**
   - Write unit tests for new functionality
   - Add integration tests for language features
   - Aim for high code coverage in critical paths

### Naming Conventions

- **Functions and variables**: `snake_case`
- **Types and structs**: `PascalCase`
- **Constants**: `UPPER_SNAKE_CASE`
- **Private helpers**: prefix with `_` if unused

### Example Code Guidelines

Example `.vais` files should:
- Demonstrate clear, idiomatic Vais code
- Include comments explaining key concepts
- Be executable and tested
- Follow the [LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md)

## Pull Request Workflow

### Branch Naming

Use descriptive branch names following this pattern:

```
<type>/<scope>/<description>
```

Examples:
- `feat/parser/support-async-syntax`
- `fix/codegen/overflow-bug`
- `docs/tutorial/add-examples`
- `test/types/generic-constraints`

Valid types:
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `test` - Test additions/improvements
- `refactor` - Code refactoring
- `perf` - Performance improvements
- `chore` - Build, dependencies, etc.

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Subject line:**
- Use imperative mood ("add" not "adds" or "added")
- Don't capitalize the first letter
- No period at the end
- Maximum 50 characters

**Body (optional):**
- Explain what and why, not how
- Wrap at 72 characters
- Separate from subject with a blank line

**Footer (optional):**
- Reference issues: `Fixes #123`
- Breaking changes: `BREAKING CHANGE: <description>`

**Examples:**

```
feat(parser): add support for async/await syntax

Implement async function parsing and type checking.
Enables programs to use async blocks and await expressions.

Fixes #45
```

```
fix(codegen): prevent register allocation overflow

Store temporary values on stack when register pressure is high.
This fixes crashes on deeply nested expressions.

Fixes #234
BREAKING CHANGE: Register allocation strategy has changed
```

### Creating a Pull Request

1. **Create a feature branch**
   ```bash
   git checkout -b feat/your-feature
   ```

2. **Make your changes**
   - Keep commits logical and atomic
   - Reference issues in commit messages
   - Test locally: `cargo test`

3. **Format and lint**
   ```bash
   cargo fmt --all
   cargo clippy --all --all-targets -- -D warnings
   ```

4. **Push your branch**
   ```bash
   git push origin feat/your-feature
   ```

5. **Open a Pull Request**
   - Use a clear title matching Conventional Commits
   - Fill out the PR template completely
   - Link related issues with "Fixes #123"
   - Request review from maintainers

### PR Checklist

Before submitting your PR, ensure:

- [ ] Code builds without errors: `cargo build`
- [ ] All tests pass: `cargo test`
- [ ] Code is formatted: `cargo fmt --all`
- [ ] No clippy warnings: `cargo clippy --all -- -D warnings`
- [ ] Commit messages follow Conventional Commits
- [ ] Documentation is updated if needed
- [ ] Added tests for new functionality
- [ ] No unnecessary dependencies added

### Code Review Process

- Maintainers will review within 2-3 days
- Address feedback with new commits (don't rebase)
- Once approved, a maintainer will merge your PR

## Reporting Issues

### Bug Reports

Please use the following template:

**Title:** `[BUG] Brief description`

**Description:**

```markdown
## Environment
- Vais version: (output of `vaisc --version`)
- OS: (macOS, Linux, Windows)
- Rust version: (output of `rustc --version`)

## Reproduction Steps
1. ...
2. ...
3. ...

## Expected Behavior
Description of what should happen

## Actual Behavior
Description of what actually happens

## Minimal Reproducible Example
```vais
# Minimal Vais code that triggers the bug
F main() {
    # ...
}
```

## Additional Context
Any other relevant information (error messages, logs, etc.)
```

### Feature Requests

Please use the following template:

**Title:** `[FEATURE] Brief description`

**Description:**

```markdown
## Motivation
Explain why this feature would be useful

## Proposed Solution
Describe how the feature should work

## Example Usage
```vais
# Example of using the feature
```

## Alternatives Considered
Other approaches to solving this problem

## Additional Context
Links to relevant discussions, issues, or examples
```

## License

By contributing to Vais, you agree that your contributions will be licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

## Questions?

- Join our discussions on GitHub
- Check out the [documentation](docs/)
- Review the [LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md) for language details
- Look at [STDLIB.md](docs/STDLIB.md) for standard library reference

Thank you for contributing to Vais!

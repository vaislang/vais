# Vais Documentation

Welcome to the official documentation for **Vais** (Vibe AI Script), a programming language designed for AI to generate, modify, and execute code most efficiently.

## Documentation Index

| Document | Description |
|----------|-------------|
| [Getting Started](./getting-started.md) | Installation and first steps |
| [Syntax Guide](./syntax.md) | Complete language syntax reference |
| [API Reference](./api.md) | Built-in functions and standard library |
| [Examples](./examples.md) | Practical code examples |
| [Contributing](./contributing.md) | How to contribute to Vais |

## Quick Overview

### What is Vais?

Vais is a functional-first programming language optimized for:

- **Token Efficiency** - 30-60% fewer tokens compared to Python
- **AI Code Generation** - Syntax designed for LLM comprehension
- **High Performance** - Interpreter, JIT (50-75x faster), and native compilation

### Key Features

- **Self-Recursion Operator (`$`)** - Elegant recursive definitions
- **Collection Operators** - `.@` (map), `.?` (filter), `./` (reduce)
- **First-Class Functions** - Closures, lambdas, and higher-order functions
- **Multiple Backends** - Interpreter, JIT, C, WebAssembly, LLVM
- **Rich Tooling** - LSP, Package Manager, Formatter, Profiler, Debugger

### Quick Example

```vais
// Factorial with self-recursion
factorial(n) = n < 2 ? 1 : n * $(n - 1)

// Collection operations pipeline
result = [1..10]
    .?(_ % 2 == 0)      // Filter evens
    .@(_ * _)           // Square each
    ./+(0, _ + _)       // Sum all
```

## Getting Help

- **[GitHub Repository](https://github.com/sswoo88/vais)** - Source code and issues
- **[Language Guide](./syntax.md)** - Complete syntax documentation
- **[API Reference](./api.md)** - Built-in functions

## License

Vais is licensed under the MIT License. See the [LICENSE](https://github.com/sswoo88/vais/blob/main/LICENSE) file for details.

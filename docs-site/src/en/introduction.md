# Vais Programming Language

**Vais** is an AI-optimized systems programming language designed for maximum token efficiency and developer productivity.

## Key Features

- **Single-Character Keywords**: `F` for function, `S` for struct, `I` for if, `M` for match
- **Expression-Oriented**: Everything returns a value
- **Self-Recursion Operator**: `@` for concise recursive functions
- **Advanced Type System**: Full type inference, generics, traits
- **LLVM Backend**: Native performance with optimized code generation
- **Modern Features**: Async/await, pattern matching, error handling with `?` and `!`

## Why Vais?

Vais minimizes token usage in AI-generated code while maintaining full systems programming capabilities:

- **50-70% fewer tokens** compared to Rust/C++
- **Self-hosting compiler** with 50,000+ lines of Vais code
- **774K lines/second** compilation speed
- **2,500+ tests** across all components

## Example

```vais
# Fibonacci with self-recursion
F fib(n: i64) -> i64 {
    I n <= 1 { R n }
    @(n-1) + @(n-2)
}

F main() {
    puts("fib(10) = ")
    print_i64(fib(10))
}
```

## Getting Started

- [Install Vais](./getting-started/installation.md) on your system
- Follow the [Quick Start](./getting-started/quick-start.md) guide
- Learn from the [Tutorial](./getting-started/tutorial.md)
- Read the [Language Specification](./language/language-spec.md)

## Community & Support

- **GitHub**: [vaislang/vais](https://github.com/vaislang/vais)
- **Issues**: Report bugs and request features
- **Documentation**: Browse the full docs

## License

Vais is open source under the MIT license.

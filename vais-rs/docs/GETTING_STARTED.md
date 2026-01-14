# Getting Started with Vais

Welcome to Vais (Vibe AI Script)! This guide will help you get started with Vais programming.

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/vais-lang/vais.git
cd vais/vais-rs

# Build with Cargo
cargo build --release

# Add to PATH
export PATH="$PATH:$(pwd)/target/release"
```

### Verify Installation

```bash
vais --version
# Output: vais 0.1.0
```

## Your First Program

Create a file called `hello.vais`:

```vais
// hello.vais
greet(name) = "Hello, " ++ name ++ "!"

print(greet("World"))
```

Run it:

```bash
vais run hello.vais
# Output: Hello, World!
```

## Language Basics

### Functions

Functions use a simple mathematical notation:

```vais
// Simple function
add(a, b) = a + b

// Conditional with ternary operator
max(a, b) = a > b ? a : b

// Nested ternary for multiple conditions
abs(n) = n < 0 ? -n : n
```

### Self-Recursion ($)

Use `$` to refer to the current function:

```vais
factorial(n) = n < 2 ? 1 : n * $(n - 1)
fibonacci(n) = n < 2 ? n : $(n-1) + $(n-2)
```

### Collections

```vais
// Arrays
numbers = [1, 2, 3, 4, 5]

// Map with .@
doubled = numbers.@(_ * 2)  // [2, 4, 6, 8, 10]

// Filter with .?
evens = numbers.?(_ % 2 == 0)  // [2, 4]

// Reduce with ./+, ./*
sum = numbers./+      // 15
product = numbers./*  // 120

// Chain operations
result = numbers
    .?(_ > 2)     // filter: [3, 4, 5]
    .@(_ * 2)     // map: [6, 8, 10]
    ./+           // reduce: 24
```

### Pattern Matching

```vais
classify(n) = match n {
    0 => "zero",
    1 => "one",
    x if x < 0 => "negative",
    _ => "other"
}
```

### Chaining Operations

```vais
// Chain collection operators for data transformation
data = [1, 2, 3, 4, 5]

// Double -> filter evens -> sum
result = data.@(_ * 2).?(_ % 2 == 0)./+

// Long words to uppercase
words = ["hello", "world", "hi"]
result = words.?(len(_) > 3).@(upper(_))  // ["HELLO", "WORLD"]
```

## Development Tools

### REPL

Interactive shell for experimenting:

```bash
vais repl
```

Commands:
- `:help` - Show help
- `:type <expr>` - Show expression type
- `:ast <expr>` - Show AST
- `:quit` - Exit

### Check & Format

```bash
# Type check a file
vais check myfile.vais

# Format code
vais format myfile.vais --write
```

### Build

Compile to native executables:

```bash
# Build to C + compile
vais build program.vais

# Build to WebAssembly
vais build program.vais --target wasm

# Build to LLVM IR
vais build program.vais --target llvm
```

## Project Setup

Create a new project:

```bash
vais init my-project
cd my-project
```

This creates:
- `vais.toml` - Project manifest
- `src/main.vais` - Entry point
- `.gitignore` - Git ignore file

### Dependencies

```bash
# Add a dependency
vais add utils

# Install all dependencies
vais install

# List dependencies
vais list
```

## Next Steps

- **[Language Reference](./LANGUAGE_REFERENCE.md)** - Complete language documentation
- **[Standard Library](./STDLIB.md)** - Built-in functions
- **[Examples](../examples/)** - Code examples
- **[Playground](https://vais-lang.github.io/playground)** - Try Vais in browser

## Getting Help

- **GitHub Issues**: Report bugs or request features
- **Discussions**: Ask questions and share ideas
- **Discord**: Join our community chat

Happy coding with Vais! 🚀

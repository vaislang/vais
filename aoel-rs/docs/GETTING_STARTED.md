# Getting Started with Vais

Welcome to Vais (AI-Optimized Executable Language)! This guide will help you get started with Vais programming.

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

// With type annotations (optional)
multiply(x: Int, y: Int) -> Int = x * y

// Conditional expression
max(a, b) = if a > b then a else b

// Ternary shorthand
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

// Reduce with ./
sum = numbers./+(0, _ + _)  // 15

// Chain operations
result = numbers
    .?(_ > 2)      // filter
    .@(_ * 2)      // map
    ./+(0, _ + _)  // reduce
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

### Pipeline Operator

```vais
// Instead of nested calls
result = sum(filter(map(data, double), is_even))

// Use pipelines
result = data |> @(double) |> ?(is_even) |> sum
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

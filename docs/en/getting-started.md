# Getting Started with Vais

Welcome to Vais! This guide will help you install Vais and write your first program.

## Table of Contents

- [Installation](#installation)
- [Your First Program](#your-first-program)
- [Language Basics](#language-basics)
- [Running Programs](#running-programs)
- [Development Tools](#development-tools)
- [Project Setup](#project-setup)
- [Next Steps](#next-steps)

---

## Installation

### Prerequisites

- Rust 1.75+ (stable)
- Cargo (Rust's package manager)
- Git

### Building from Source

```bash
# Clone the repository
git clone https://github.com/sswoo88/vais.git
cd vais/vais-rs

# Build release version
cargo build --release

# Add to PATH (optional)
export PATH="$PATH:$(pwd)/target/release"
```

### Verify Installation

```bash
vais --version
# Output: vais 1.0.0
```

### Building with JIT Support

For optimal performance, build with JIT compilation:

```bash
cargo build --release --features jit
```

---

## Your First Program

### Hello World

Create a file named `hello.vais`:

```vais
print("Hello, Vais!")
```

Run it:

```bash
vais run hello.vais
# Output: Hello, Vais!
```

### A More Complete Example

Create `greeting.vais`:

```vais
// Define a greeting function
greet(name) = "Hello, " ++ name ++ "!"

// Use the function
message = greet("World")
print(message)
```

Run:

```bash
vais run greeting.vais
# Output: Hello, World!
```

---

## Language Basics

### Variables

```vais
// Variables are immutable by default
name = "Vais"
version = 1.0
is_ready = true
numbers = [1, 2, 3, 4, 5]
```

### Functions

Functions use a mathematical notation:

```vais
// Simple function
add(a, b) = a + b

// With type annotations (optional)
multiply(x: Int, y: Int): Int = x * y

// Conditional expression
max(a, b) = a > b ? a : b
```

### Self-Recursion with `$`

The `$` operator calls the current function recursively:

```vais
// Factorial
factorial(n) = n < 2 ? 1 : n * $(n - 1)

// Fibonacci
fib(n) = n < 2 ? n : $(n - 1) + $(n - 2)

// Usage
print(factorial(5))   // 120
print(fib(10))        // 55
```

### Collection Operations

Vais provides concise operators for functional programming:

```vais
numbers = [1, 2, 3, 4, 5]

// Map (.@) - transform each element
doubled = numbers.@(_ * 2)        // [2, 4, 6, 8, 10]

// Filter (.?) - keep matching elements
evens = numbers.?(_ % 2 == 0)     // [2, 4]

// Reduce (./) - fold into single value
sum = numbers./+(0, _ + _)        // 15

// Chain operations
result = [1..10]
    .?(_ % 2 == 0)    // filter evens
    .@(_ * _)         // square each
    ./+(0, _ + _)     // sum all
```

### Modules

```vais
// Import specific functions
use math.{sin, cos, pi}

// Public function (exportable)
pub calculate(x) = sin(x) * cos(x)

// Private function (default)
helper(x) = x * 2
```

---

## Running Programs

### Interpreter (Default)

```bash
vais run program.vais
```

### JIT Compilation (50-75x Faster)

```bash
# Requires --features jit during build
vais run program.vais --jit
```

### REPL (Interactive Mode)

```bash
vais repl
```

REPL commands:
- `:help` - Show help
- `:type <expr>` - Show expression type
- `:ast <expr>` - Show AST
- `:quit` - Exit

### Native Compilation

```bash
# Compile to C
vais build program.vais --target c

# Compile to WebAssembly
vais build program.vais --target wasm

# Compile to LLVM IR
vais build program.vais --target llvm
```

---

## Development Tools

### Language Server (LSP)

Full IDE support with:
- Auto-completion
- Hover documentation
- Go to Definition
- Find References
- Rename Symbol

```bash
vais lsp
```

### Code Formatting

```bash
# Format to stdout
vais format program.vais

# Format in place
vais format program.vais --write

# Check formatting
vais format program.vais --check
```

### Type Checking

```bash
vais check program.vais
```

### Profiling

```bash
# Profile execution
vais profile program.vais

# JSON output
vais profile program.vais --format json
```

---

## Project Setup

### Create a New Project

```bash
vais init my-project
cd my-project
```

This creates:
- `vais.toml` - Project manifest
- `src/main.vais` - Entry point
- `.gitignore` - Git ignore file

### Project Structure

```
my-project/
├── vais.toml
├── src/
│   ├── main.vais
│   └── utils.vais
└── lib/
    └── math.vais
```

### Managing Dependencies

```bash
# Add a dependency
vais add utils

# Install all dependencies
vais install

# List dependencies
vais list

# Publish to registry
vais publish
```

---

## Next Steps

Now that you have Vais installed and running, explore more:

- **[Syntax Guide](./syntax.md)** - Complete language syntax reference
- **[API Reference](./api.md)** - All built-in functions
- **[Examples](./examples.md)** - Practical code examples
- **[Contributing](./contributing.md)** - How to contribute to Vais

## Getting Help

- **GitHub Issues** - Report bugs or request features
- **GitHub Discussions** - Ask questions and share ideas

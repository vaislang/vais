# Quick Start

Get up and running with Vais in minutes.

## Installation

```bash
# macOS / Linux (Homebrew)
brew tap vaislang/tap && brew install vais

# Or via Cargo
cargo install vaisc
```

> To build from source, see the [installation guide](./installation.md).

## Your First Program

Create a file named `hello.vais`:

```vais
fn main() {
    puts("Hello, Vais!")
}
```

## Compile and Run

```bash
# Compile
vaisc build hello.vais -o hello
./hello

# Or run directly
vaisc run hello.vais
```

**Output:**
```
Hello, Vais!
```

## Basic Syntax

### Variables

```vais
fn main() {
    x := 42              # Type inferred as i64
    y := 3.14            # Type inferred as f64
    name := "Alice"      # Type inferred as str
    flag := true         # Type inferred as bool

    puts("Variables declared!")
}
```

### Functions

```vais
fn add(a: i64, b: i64) -> i64 {
    a + b  # Last expression is return value
}

fn main() {
    result := add(10, 20)
    print_i64(result)  # Prints: 30
}
```

### Control Flow

```vais
fn main() {
    x := 10

    # If expression
    msg := I x > 5 { "big" } else { "small" }
    puts(msg)

    # Loop
    L i := 0; i < 5; i += 1 {
        print_i64(i)
    }
}
```

### Self-Recursion

Use `@` to call the current function:

```vais
fn factorial(n: i64) -> i64 {
    I n <= 1 { return 1 }
    n * @(n - 1)
}

fn main() {
    print_i64(factorial(5))  # Prints: 120
}
```

## Next Steps

- [Tutorial](./tutorial.md) - Learn Vais in depth
- [Language Specification](../language/language-spec.md) - Full syntax reference
- [Example Programs](https://github.com/vaislang/vais/tree/main/examples) - Browse code samples

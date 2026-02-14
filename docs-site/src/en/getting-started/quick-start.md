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
F main() {
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
F main() {
    x := 42              # Type inferred as i64
    y := 3.14            # Type inferred as f64
    name := "Alice"      # Type inferred as str
    flag := true         # Type inferred as bool

    puts("Variables declared!")
}
```

### Functions

```vais
F add(a: i64, b: i64) -> i64 {
    a + b  # Last expression is return value
}

F main() {
    result := add(10, 20)
    print_i64(result)  # Prints: 30
}
```

### Control Flow

```vais
F main() {
    x := 10

    # If expression
    msg := I x > 5 { "big" } E { "small" }
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
F factorial(n: i64) -> i64 {
    I n <= 1 { R 1 }
    n * @(n - 1)
}

F main() {
    print_i64(factorial(5))  # Prints: 120
}
```

## Next Steps

- [Tutorial](./tutorial.md) - Learn Vais in depth
- [Language Specification](../language/language-spec.md) - Full syntax reference
- [Example Programs](https://github.com/vaislang/vais/tree/main/examples) - Browse code samples

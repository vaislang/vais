# Building X in Vais - Episode 01: Getting Started with Vais

**Duration:** 8 minutes
**Difficulty:** Beginner
**Series:** Building X in Vais

## Introduction

Welcome to the first episode of "Building X in Vais"! In this series, we'll explore Vais, a modern programming language designed for concise, expressive code generation. Vais uses ultra-compact keywords like `F` for functions and `@` for self-recursion, making it perfect for AI-assisted development.

In this episode, we'll get you set up and writing your first Vais programs. By the end, you'll understand the basics of functions, variables, and output in Vais.

## Prerequisites

Before we start, make sure you have:
- Rust toolchain installed
- LLVM 17+ and Clang
- A text editor (VS Code with Vais extension recommended)

## Step 1: Installing Vais (1 minute)

Let's clone and build the Vais compiler:

```bash
git clone https://github.com/sswoo88/vais.git
cd vais
cargo build --release
```

The compiler will be at `./target/release/vaisc`. Let's verify it works:

```bash
./target/release/vaisc --version
```

You should see: `Vais 0.0.1`

## Step 2: Hello World (2 minutes)

Create a file called `hello.vais`:

```vais
# Hello World example
F main() -> i64 {
    puts("Hello, Vais!")
    0
}
```

Let's break this down:
- `F` is the keyword for function definition
- `main()` is the entry point with no parameters
- `-> i64` means it returns a 64-bit integer
- `puts()` prints a string with a newline
- `0` is the return value (convention: 0 means success)

Compile and run:

```bash
./target/release/vaisc hello.vais --emit-ir -o hello.ll
clang hello.ll -o hello
./hello
```

Output: `Hello, Vais!`

## Step 3: Variables and Basic Math (2 minutes)

Let's do some calculations. Create `calculator.vais`:

```vais
F main() -> i64 {
    # Variable binding with type inference
    x := 20
    y := 22
    sum := x + y

    puts("The answer is:")
    sum
}
```

Notice:
- `:=` creates a variable with inferred type
- Last expression is the return value (no explicit `return` needed)
- `#` starts a comment

The program returns 42. Compile and run it to see!

## Step 4: Your First Function (2 minutes)

Functions in Vais are concise. Here's a simple addition function:

```vais
# Single-line function using = syntax
F add(a: i64, b: i64) -> i64 = a + b

# Multi-line function using { } blocks
F greet(name: str) -> i64 {
    puts("Hello, ")
    puts(name)
    puts("!")
    0
}

F main() -> i64 {
    result := add(15, 27)
    greet("Vais")
    result
}
```

Key points:
- Single-expression functions use `= expr` syntax
- Multi-line functions use `{ }` blocks
- Parameters require type annotations like `a: i64`
- Return type follows `->` arrow

## Step 5: The Self-Recursion Operator @ (1 minute)

Vais has a unique feature: the `@` operator for self-recursion. Instead of writing the function name, just use `@`:

```vais
# Factorial using @
F factorial(n: i64) -> i64 = n < 2 ? 1 : n * @(n - 1)

# Countdown function
F countdown(n: i64) -> i64 {
    I n <= 0 {
        puts("Blast off!")
        0
    } E {
        puts("Counting...")
        @(n - 1)
    }
}

F main() -> i64 {
    result := factorial(5)  # Returns 120
    countdown(3)
    result
}
```

The `@` operator:
- Calls the current function recursively
- More concise than typing the function name
- Clear signal that recursion is happening

## Key Takeaways

1. **Compact Keywords**: `F` for functions, `I/E` for if/else, `M` for match
2. **Type Inference**: Use `:=` for automatic types, `:` for explicit
3. **Expression-Based**: Last expression is the return value
4. **Self-Recursion**: `@` operator for recursive calls
5. **Simple Output**: `puts()` for strings, return values for data

## Next Episode Preview

In Episode 02, we'll build a practical Fibonacci calculator that demonstrates:
- The power of the `@` self-recursion operator
- Ternary expressions for compact logic
- Performance comparisons with iterative approaches
- Building reusable math utilities

## Try It Yourself

Challenge: Write a function that:
1. Takes a name as parameter
2. Counts from 1 to 5
3. Greets the person that many times

Hint: You'll need recursion and string output!

## Resources

- Vais GitHub: https://github.com/sswoo88/vais
- Quickstart Guide: `docs/QUICKSTART.md`
- Tutorial: `docs/TUTORIAL.md`
- Examples: `examples/` directory

---

See you in Episode 02 where we build a Fibonacci calculator!

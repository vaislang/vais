# Vais Quickstart Guide

Get started with Vais in 5 minutes. This guide takes you from zero to running your first program.

## 1. Install Prerequisites

### macOS

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install LLVM and Clang
brew install llvm@17 cmake
```

### Linux (Ubuntu/Debian)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install LLVM and Clang
sudo apt install llvm-17 clang-17 cmake
```

### Windows

```powershell
# Install Rust from https://rustup.rs
# Install LLVM from https://releases.llvm.org or:
choco install llvm cmake
```

## 2. Build the Compiler

```bash
git clone https://github.com/sswoo88/vais.git
cd vais
cargo build --release
```

The compiler binary is at `./target/release/vaisc`.

## 3. Write Your First Program

Create `hello.vais`:

```vais
F main() -> i64 {
    puts("Hello, Vais!")
    0
}
```

## 4. Compile and Run

```bash
# Compile to LLVM IR
./target/release/vaisc hello.vais --emit-ir -o hello.ll

# Build executable
clang hello.ll -o hello

# Run
./hello
# Output: Hello, Vais!
```

That's it! You've compiled and run your first Vais program.

---

## Language Basics

### Functions

Functions use `F` keyword. The last expression is the return value:

```vais
# One-liner function
F double(x: i64) -> i64 = x * 2

# Multi-line function
F add(a: i64, b: i64) -> i64 {
    result := a + b
    result
}

F main() -> i64 = add(20, 22)
```

### Variables

```vais
F main() -> i64 {
    # Immutable binding
    x := 42

    # Mutable variable
    y := mut 0
    y = 10

    x + y
}
```

### Control Flow

```vais
# If/Else (I/E keywords)
F max(a: i64, b: i64) -> i64 = I a > b { a } E { b }

# Ternary
F abs(x: i64) -> i64 = x < 0 ? 0 - x : x

# Match (M keyword)
F describe(n: i64) -> i64 = M n {
    0 => 10,
    1 => 20,
    _ => 42
}
```

### Self-Recursion (`@`)

The `@` operator calls the current function recursively:

```vais
F fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)
F factorial(n: i64) -> i64 = n < 2 ? 1 : n * @(n - 1)
```

### Structs

```vais
S Point { x: i64, y: i64 }

F main() -> i64 {
    p := Point { x: 40, y: 2 }
    p.x + p.y
}
```

### Arrays

```vais
F main() -> i64 {
    arr: *i64 = [10, 20, 30, 42]
    arr[3]
}
```

### Output

```vais
F main() -> i64 {
    puts("Hello!")          # Print string with newline
    putchar(65)             # Print character 'A'
    0
}
```

---

## Cookbook / Recipes

### Recipe 1: Fibonacci Sequence

```vais
F fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)
F main() -> i64 = fib(10)   # Returns 55
```

### Recipe 2: GCD (Greatest Common Divisor)

```vais
F gcd(a: i64, b: i64) -> i64 = I b == 0 { a } E { gcd(b, a % b) }
F main() -> i64 = gcd(48, 18)   # Returns 6
```

### Recipe 3: Prime Check

```vais
F is_prime_helper(n: i64, d: i64) -> i64 =
    I d * d > n { 1 }
    E I n % d == 0 { 0 }
    E { @(n, d + 2) }

F is_prime(n: i64) -> i64 =
    I n < 2 { 0 }
    E I n < 4 { 1 }
    E I n % 2 == 0 { 0 }
    E { is_prime_helper(n, 3) }

F main() -> i64 = is_prime(97)   # Returns 1 (true)
```

### Recipe 4: Array Statistics

```vais
F array_sum(arr: *i64, len: i64, idx: i64) -> i64 =
    I idx == len { 0 }
    E { arr[idx] + @(arr, len, idx + 1) }

F main() -> i64 {
    data: *i64 = [10, 20, 30, 40]
    array_sum(data, 4, 0)   # Returns 100
}
```

### Recipe 5: Binary Search

```vais
F binary_search(arr: *i64, target: i64, lo: i64, hi: i64) -> i64 =
    I lo > hi { 0 - 1 }
    E {
        mid := (lo + hi) / 2;
        I arr[mid] == target { mid }
        E I arr[mid] < target { @(arr, target, mid + 1, hi) }
        E { @(arr, target, lo, mid - 1) }
    }

F main() -> i64 {
    sorted: *i64 = [10, 20, 30, 40, 50]
    binary_search(sorted, 30, 0, 4)   # Returns 2
}
```

### Recipe 6: Struct with Logic

```vais
S Counter { count: i64, step: i64 }

F advance(count: i64, step: i64, times: i64) -> i64 =
    I times == 0 { count } E { @(count + step, step, times - 1) }

F main() -> i64 {
    c := Counter { count: 0, step: 7 }
    advance(c.count, c.step, 6)   # Returns 42
}
```

### Recipe 7: Print Numbers

```vais
F print_digit(d: i64) -> i64 { putchar(d + 48); 0 }

F print_num(n: i64) -> i64 =
    I n < 10 { print_digit(n) }
    E { print_num(n / 10); print_digit(n % 10) }

F main() -> i64 {
    print_num(12345)
    putchar(10)
    0
}
```

---

## CLI Commands

```bash
# Compile to LLVM IR
vaisc hello.vais --emit-ir -o hello.ll

# Build and run in one step
vaisc run hello.vais

# Type-check only (no codegen)
vaisc check hello.vais

# Format source code
vaisc fmt hello.vais

# Show compilation timings
vaisc hello.vais --time

# Start interactive REPL
vaisc repl

# Show tokens (debugging)
vaisc hello.vais --show-tokens

# Show AST (debugging)
vaisc hello.vais --show-ast
```

---

## Next Steps

- [Full Tutorial](TUTORIAL.md) - In-depth language guide
- [Language Spec](LANGUAGE_SPEC.md) - Complete language reference
- [Standard Library](STDLIB.md) - Available stdlib modules
- [Editor Support](EDITORS.md) - IDE/editor integration
- [Examples](../examples/) - 100+ example programs

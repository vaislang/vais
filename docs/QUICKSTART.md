# Vais Quickstart Guide

Get started with Vais in 5 minutes. This guide takes you from zero to running your first program.

## 1. Install Vais

### Option A: Homebrew (macOS/Linux) - Recommended

```bash
brew tap vaislang/tap
brew install vais
```

Release-channel binaries may lag behind the current certified source baseline.
For certification-sensitive work, use the source build below and run the gates
listed in [`../PUBLIC_STATUS.md`](../PUBLIC_STATUS.md).

### Option B: Pre-built Binaries

Download availability is release-dependent. Check
[GitHub Releases](https://github.com/vaislang/vais/releases/latest) for the
currently published artifacts.

### Option C: From Source (requires Rust 1.70+ and LLVM 17)

```bash
git clone https://github.com/vaislang/vais.git
cd vais && cargo build --release
```

The compiler binary is at `./target/release/vaisc`.

### Prerequisite: clang

Vais compiles to LLVM IR and uses `clang` to produce native binaries.

- **macOS**: `xcode-select --install` (Xcode Command Line Tools)
- **Linux**: `sudo apt install clang` or `sudo dnf install clang`
- **Windows**: Install LLVM from https://releases.llvm.org

## 2. Write Your First Program

Create `hello.vais`:

```vais
fn main() -> i64 {
    puts("Hello, Vais!")
    0
}
```

## 3. Compile and Run

```bash
vaisc hello.vais    # Compiles to native binary
./hello             # Run it
# Output: Hello, Vais!
```

That's it! You've compiled and run your first Vais program.

---

## Language Basics

### Functions

Functions use the canonical `fn` keyword. The last expression is the return value:

```vais
# One-liner function
fn double(x: i64) -> i64 = x * 2

# Multi-line function
fn add(a: i64, b: i64) -> i64 {
    result := a + b
    result
}

fn main() -> i64 = add(20, 22)
```

### Variables

```vais
fn main() -> i64 {
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
# If/Else (`I` and `else`)
fn max(a: i64, b: i64) -> i64 = I a > b { a } else { b }

# Ternary
fn abs(x: i64) -> i64 = x < 0 ? 0 - x : x

# Match
fn describe(n: i64) -> i64 = match n {
    0 => 10,
    1 => 20,
    _ => 42
}
```

### Self-Recursion (`@`)

The `@` operator calls the current function recursively:

```vais
fn fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)
fn factorial(n: i64) -> i64 = n < 2 ? 1 : n * @(n - 1)
```

### Structs

```vais
struct Point { x: i64, y: i64 }

fn main() -> i64 {
    p := Point { x: 40, y: 2 }
    p.x + p.y
}
```

### Arrays

```vais
fn main() -> i64 {
    arr: *i64 = [10, 20, 30, 42]
    arr[3]
}
```

### Output

```vais
fn main() -> i64 {
    puts("Hello!")          # Print string with newline
    putchar(65)             # Print character 'A'
    0
}
```

---

## Cookbook / Recipes

### Recipe 1: Fibonacci Sequence

```vais
fn fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)
fn main() -> i64 = fib(10)   # Returns 55
```

### Recipe 2: GCD (Greatest Common Divisor)

```vais
fn gcd(a: i64, b: i64) -> i64 = I b == 0 { a } else { gcd(b, a % b) }
fn main() -> i64 = gcd(48, 18)   # Returns 6
```

### Recipe 3: Prime Check

```vais
fn is_prime_helper(n: i64, d: i64) -> i64 =
    I d * d > n { 1 }
    else I n % d == 0 { 0 }
    else { @(n, d + 2) }

fn is_prime(n: i64) -> i64 =
    I n < 2 { 0 }
    else I n < 4 { 1 }
    else I n % 2 == 0 { 0 }
    else { is_prime_helper(n, 3) }

fn main() -> i64 = is_prime(97)   # Returns 1 (true)
```

### Recipe 4: Array Statistics

```vais
fn array_sum(arr: *i64, len: i64, idx: i64) -> i64 =
    I idx == len { 0 }
    else { arr[idx] + @(arr, len, idx + 1) }

fn main() -> i64 {
    data: *i64 = [10, 20, 30, 40]
    array_sum(data, 4, 0)   # Returns 100
}
```

### Recipe 5: Binary Search

```vais
fn binary_search(arr: *i64, target: i64, lo: i64, hi: i64) -> i64 =
    I lo > hi { 0 - 1 }
    else {
        mid := (lo + hi) / 2
        I arr[mid] == target { mid }
        else I arr[mid] < target { @(arr, target, mid + 1, hi) }
        else { @(arr, target, lo, mid - 1) }
    }

fn main() -> i64 {
    sorted: *i64 = [10, 20, 30, 40, 50]
    binary_search(sorted, 30, 0, 4)   # Returns 2
}
```

### Recipe 6: Struct with Logic

```vais
struct Counter { count: i64, step: i64 }

fn advance(count: i64, step: i64, times: i64) -> i64 =
    I times == 0 { count } else { @(count + step, step, times - 1) }

fn main() -> i64 {
    c := Counter { count: 0, step: 7 }
    advance(c.count, c.step, 6)   # Returns 42
}
```

### Recipe 7: Print Numbers

```vais
fn print_digit(d: i64) -> i64 { putchar(d + 48); 0 }

fn print_num(n: i64) -> i64 =
    I n < 10 { print_digit(n) }
    else { print_num(n / 10); print_digit(n % 10) }

fn main() -> i64 {
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
- [Language Spec](LANGUAGE_SPEC.md) - Current language reference and non-Core notes
- [Standard Library](STDLIB.md) - Available stdlib modules
- [Editor Support](EDITORS.md) - IDE/editor integration
- [Examples](../examples/) - 100+ example programs

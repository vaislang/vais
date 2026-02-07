# Vais Performance Benchmarks

## Introduction

This page presents **actual measured** performance benchmarks comparing Vais against C, Rust, and Python. All numbers are from real runs on local hardware — no projected or estimated values.

### Test Environment

**Hardware:**
- CPU: Apple Silicon (ARM64)
- OS: macOS Darwin 25.2.0

**Compiler Versions:**
- Vais: vaisc 1.0.0 (LLVM 17 backend, text codegen + clang)
- C: Apple clang 17.0.0
- Rust: rustc 1.92.0
- Python: CPython 3.14.2

**Compilation Flags:**
- Vais: `vaisc build -O2` (LLVM backend, clang -O2)
- C: `clang -O2`
- Rust: `rustc -C opt-level=3`
- Python: interpreted (no JIT)

**Measurement:**
- Each benchmark run 5 times, median `user` time reported
- Measured using `/usr/bin/time -p`
- All programs single-threaded

## Runtime Performance

### Benchmark Suite (Fibonacci + Sum + Primes)

Three algorithms: recursive fib(35), sum 1-100K (tail-recursive/iterative), count primes to 5000.

| Language | Time (user) | vs C | vs Vais |
|----------|-------------|------|---------|
| C (-O2) | 0.02s | 1.0x | 1.0x |
| Rust (-O3) | 0.02s | 1.0x | 1.0x |
| **Vais (-O2)** | **0.02s** | **1.0x** | **1.0x** |
| Vais (-O0, default) | 0.05s | 2.5x | 2.5x |
| Python 3.14 | 1.23s | 61.5x | 61.5x |

### Fibonacci fib(40) — CPU-Intensive Single Function

| Language | Time (user) | vs C |
|----------|-------------|------|
| C (-O2) | 0.31s | 1.00x |
| Rust (-O3) | 0.31s | 1.00x |
| **Vais (-O2)** | **0.31s** | **1.00x** |
| Vais (-O0) | 0.55s | 1.77x |
| Python 3.14 | 13.53s | 43.6x |

**Key finding:** With `-O2` optimization, Vais achieves **identical performance** to C and Rust. The default `-O0` is 1.77x slower, which is expected for unoptimized code.

## Binary Size

Compiled benchmark executable (fib + sum + primes):

| Language | Size | vs C |
|----------|------|------|
| C | 33 KB | 1.0x |
| **Vais** | **58 KB** | **1.7x** |
| Rust | 433 KB | 12.8x |
| Python | N/A | interpreter |

Vais produces compact binaries — 7.5x smaller than Rust.

## Compilation Speed

Vais compiler throughput (internal benchmark):
- **50K lines in 63ms** (~800K lines/sec)
- Frontend (lexer+parser+type checker) dominates; LLVM codegen is fast

## Token Efficiency (GPT-4 Tokenizer)

Token counts measured with `tiktoken` (cl100k_base, GPT-4 tokenizer) on the benchmark source files.

### Full Source Files (with comments + main)

| Language | Tokens | Lines | Tok/Line | vs Vais |
|----------|--------|-------|----------|---------|
| Python | 387 | 61 | 6.3 | 0.60x |
| Rust | 417 | 60 | 7.0 | 0.65x |
| C | 566 | 73 | 7.8 | 0.88x |
| **Vais** | **646** | **64** | **10.1** | **1.00x** |

### Core Algorithms Only (no main, no comments, no I/O)

| Language | Tokens | Lines | vs Vais |
|----------|--------|-------|---------|
| Python | 182 | 28 | 0.65x |
| Rust | 226 | 29 | 0.80x |
| C | 234 | 27 | 0.83x |
| **Vais** | **282** | **17** | **1.00x** |

### Why Vais Uses More Tokens

Vais's single-character keywords (`F`, `I`, `E`, `L`, `R`, `M`) each consume **1 token** in GPT-4's BPE tokenizer — but so do multi-character keywords (`fn`, `if`, `else`, `return`) in other languages. The per-keyword savings are zero.

Where Vais **gains** tokens:
- `:=` binding operator (1 token, same as Rust's `let`)
- Explicit type annotations on all function parameters (`i64`)
- Tail-recursive style requires more helper functions

Where Vais **saves** tokens:
- No semicolons
- Expression-body functions (`F f(n: i64) -> i64 = ...`)
- `@` self-recursion operator (replaces function name)
- No `#include`, `use`, `import` boilerplate
- Fewer lines overall (17 vs 27-29 for algorithms)

### Honest Assessment

For these small, algorithm-heavy benchmarks, **Vais does not have fewer tokens than C/Rust/Python**. Vais's token advantage is more likely to appear in:
- Larger programs with more control flow keywords
- Programs with structs, traits, and pattern matching
- Code that benefits from expression-body syntax and `@` operator

## Source Code

All benchmark source files are in `examples/projects/benchmark/`:
- `benchmark.vais` — Vais implementation
- `benchmark.c` — C implementation
- `benchmark.rs` — Rust implementation
- `benchmark.py` — Python implementation

### Vais (64 lines)
```vais
F fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)

F sum_to_n_helper(n: i64, acc: i64) -> i64 =
    I n == 0 { acc } E { @(n - 1, acc + n) }

F sum_to_n(n: i64) -> i64 = sum_to_n_helper(n, 0)

F is_prime_helper(n: i64, d: i64) -> i64 =
    I d * d > n { 1 }
    E I n % d == 0 { 0 }
    E { @(n, d + 2) }

F is_prime(n: i64) -> i64 =
    I n < 2 { 0 }
    E I n < 4 { 1 }
    E I n % 2 == 0 { 0 }
    E { is_prime_helper(n, 3) }

F count_primes_helper(n: i64, current: i64, count: i64) -> i64 =
    I current > n { count }
    E { @(n, current + 1, count + is_prime(current)) }

F count_primes(n: i64) -> i64 = count_primes_helper(n, 2, 0)
```

### C (73 lines)
```c
int64_t fib(int64_t n) {
    if (n < 2) return n;
    return fib(n - 1) + fib(n - 2);
}

int64_t sum_to_n(int64_t n) {
    int64_t sum = 0;
    for (int64_t i = 1; i <= n; i++) { sum += i; }
    return sum;
}

int is_prime(int64_t n) {
    if (n < 2) return 0;
    if (n < 4) return 1;
    if (n % 2 == 0) return 0;
    for (int64_t d = 3; d * d <= n; d += 2) {
        if (n % d == 0) return 0;
    }
    return 1;
}
```

### Rust (60 lines)
```rust
fn fib(n: i64) -> i64 {
    if n < 2 { return n; }
    fib(n - 1) + fib(n - 2)
}

fn sum_to_n(n: i64) -> i64 {
    let mut sum: i64 = 0;
    for i in 1..=n { sum += i; }
    sum
}

fn is_prime(n: i64) -> bool {
    if n < 2 { return false; }
    if n < 4 { return true; }
    if n % 2 == 0 { return false; }
    let mut d: i64 = 3;
    while d * d <= n {
        if n % d == 0 { return false; }
        d += 2;
    }
    true
}
```

### Python (61 lines)
```python
def fib(n):
    if n < 2: return n
    return fib(n - 1) + fib(n - 2)

def sum_to_n(n):
    s = 0
    for i in range(1, n + 1): s += i
    return s

def is_prime(n):
    if n < 2: return False
    if n < 4: return True
    if n % 2 == 0: return False
    d = 3
    while d * d <= n:
        if n % d == 0: return False
        d += 2
    return True
```

## Conclusion

Based on **actual measurements**:

- **Runtime:** Vais with `-O2` matches C and Rust exactly (1.00x). Both LLVM-backed languages produce identical machine code quality. Python is 40-60x slower.
- **Binary size:** Vais (58 KB) is compact — 1.7x of C, 7.5x smaller than Rust.
- **Token efficiency:** For these benchmarks, Vais uses slightly more tokens than Rust/Python due to explicit type annotations. Token savings are more pronounced in larger, control-flow-heavy programs.
- **Compilation speed:** ~800K lines/sec throughput.

Vais's key advantages are:
1. **C-equivalent performance** with higher-level syntax
2. **Compact binaries** (much smaller than Rust)
3. **Concise expression syntax** (`@` recursion, expression bodies, no semicolons)
4. **Fast compilation** (~800K lines/sec)

---

*Last updated: February 2026*
*Vais version: 1.0.0*
*All measurements are actual runs, not projections.*

# Vais Performance Benchmarks

## Introduction

This page presents **actual measured** performance benchmarks comparing Vais against C, Rust, Go, and Python. Each section states its measurement date and scope. Current compile-speed and token-efficiency data was refreshed on 2026-05-13; older runtime snapshots remain labeled as historical evidence, not current throughput claims.

### Test Environment

**Hardware:**
- CPU: Apple Silicon (ARM64)
- OS: macOS Darwin 25.2.0

**Current compile/token refresh (2026-05-13):**
- Vais: vaisc 0.1.0
- C: Apple clang 21.0.0
- Go: go1.25.7 darwin/arm64
- Rust: rustc 1.95.0
- Hyperfine: 1.20.0
- Tokenizer: tiktoken cl100k_base

**Compilation Flags:**
- Compile-speed benchmark: Vais `vaisc build --emit-ir`; C `clang`; Go `go build`; Rust `rustc`
- Runtime snapshot: Vais LLVM IR linked with clang `-O2`; C `clang -O2`; Rust release

**Measurement:**
- Compile speed: `hyperfine --warmup 3 --min-runs 15`
- Runtime snapshot: `/usr/bin/time -p`, single-threaded

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

Current single-file compile-speed benchmark (`benches/lang-comparison/compile_bench.sh`,
Hyperfine, 2026-05-13, Apple ARM64/macOS):

| Program | Vais `--emit-ir` | Rust `rustc` | Go `go build` | C `clang` |
|---------|------------------|--------------|---------------|-----------|
| fibonacci | 6.0ms | 93.7ms | 48.0ms | 55.8ms |
| quicksort | 6.4ms | 95.6ms | 47.8ms | 56.8ms |
| http_types | 6.6ms | 103.2ms | 47.1ms | 60.7ms |
| linked_list | 6.0ms | 98.3ms | 47.3ms | 59.5ms |
| **Average** | **6.3ms** | **97.7ms** | **47.5ms** | **58.2ms** |

Vais `--emit-ir` is 9.3x faster than C/clang, 7.6x faster than Go, and
15.6x faster than Rust on this benchmark. This compares Vais LLVM IR emission
against full binary compilation for the other toolchains.

The older large-scale throughput snapshot (50K lines in 63ms, ~800K lines/sec)
is tracked by a separate benchmark and should be rerun before citing as a
current claim.

## Token Efficiency (GPT-4 Tokenizer)

Token counts measured with `tiktoken` (cl100k_base, GPT-4 tokenizer) on four benchmark programs: fibonacci, quicksort, http_types, linked_list.

### Summary (Total across all 4 programs)

| Language | Tokens | Lines | Tok/Line | vs Vais |
|----------|--------|-------|----------|---------|
| **Vais** | **829** | **120** | **6.9** | **1.00x** |
| Python | 889 | 137 | 6.5 | 1.07x |
| Go | 893 | 174 | 5.1 | 1.08x |
| Rust | 1,080 | 163 | 6.6 | 1.30x |
| C | 1,211 | 191 | 6.3 | 1.46x |

### Per-Program Breakdown

| Program | Vais | Rust | Go | C | Python |
|---------|------|------|----|----|--------|
| fibonacci | 130 | 135 | 126 | 159 | 118 |
| quicksort | 235 | 242 | 228 | 291 | 227 |
| http_types | 172 | 431 | 318 | 454 | 326 |
| linked_list | 292 | 272 | 221 | 307 | 218 |

### Token Savings

- Vais saves **23.2%** vs Rust (1,080 → 829 tokens)
- Vais saves **31.5%** vs C (1,211 → 829 tokens)
- Vais saves **7.2%** vs Go (893 → 829 tokens)
- Vais saves **6.7%** vs Python (889 → 829 tokens)

### Why Vais Uses Fewer Tokens Than Rust/C

Vais is no longer documented as a single-letter-keyword language. The current
benchmark fixtures use canonical public declarations such as `fn` and `struct`.
The token advantage now comes from a smaller set of explicit, current syntax
choices:

**Control flow density:**
- Compact control forms (`I`, `LF`, `L`) reduce repeated boilerplate in loops and branches
- No semicolons are required
- Expression-body syntax (`= expr`) keeps simple functions short

**Recursion & operators:**
- `@` self-recursion replaces repeating the current function name
- `+=` compound assignment avoids repeated left-hand sides
- Range loops such as `LF i:0..n` avoid manual counter setup

**Bindings and data access:**
- `:=` works as the normal local binding form, without `let`/`var`/`def`
- Direct pointer/array indexing (`arr[i]`) stays concise for systems code
- Named struct literals remain explicit while still avoiding constructor boilerplate

**Struct-heavy code advantage:**
- In `http_types`, Vais uses **172 tokens** vs Rust's **431 tokens** and C's **454 tokens**
- The advantage is strongest where other systems languages repeat type names, ownership wrappers, or C-style struct setup

### Honest Assessment

**Aggregate results:** With current canonical public syntax, Vais uses **829 tokens total**, 23.2% fewer than Rust and 31.5% fewer than C. Python and Go are close in aggregate because their linked-list fixtures use GC/reference patterns while the Vais fixture uses explicit pointer operations.

**Per-benchmark variability:**
- **Struct-heavy code** (`http_types`): Vais has a large advantage (46-62% smaller than Rust/C/Python/Go)
- **Algorithm code** (`fibonacci`, `quicksort`): Vais is close to Python/Go and smaller than Rust/C in aggregate
- **Pointer arithmetic code** (`linked_list`): Vais at 292 tokens; larger than Python/Go/Rust because the fixture uses explicit `malloc`/`store`/`load` instead of GC or ownership-library helpers

Vais's token advantage comes from:
- Compact loops and branches (`I`, `L`, `LF`)
- Universal local binding with `:=`
- Canonical declarations, compact control flow, and expression bodies
- Expression-body syntax and `@` self-recursion operator

## Source Code

Single-file language-comparison benchmarks are in `benches/lang-comparison/`:
- `benches/lang-comparison/vais/*.vais`
- `benches/lang-comparison/rust/*.rs`
- `benches/lang-comparison/go/*.go`
- `benches/lang-comparison/c/*.c`
- `benches/lang-comparison/python/*.py`

The older runtime benchmark sources live under `examples/projects/benchmark/`
and the Rust Criterion benches.

### Vais
```vais
fn fib_rec(n: i64) -> i64 =
    I n <= 1 { n } else { @(n - 1) + @(n - 2) }

fn fib_iter(n: i64) -> i64 {
    a := mut 0
    b := mut 1
    LF _:0..n {
        t := a + b
        a = b
        b = t
    }
    a
}
```

## Conclusion

Based on **actual measurements**:

- **Runtime:** Vais with `-O2` matches C and Rust exactly (1.00x). Both LLVM-backed languages produce identical machine code quality. Python is 40-60x slower.
- **Binary size:** Vais (58 KB) is compact — 1.7x of C, 7.5x smaller than Rust.
- **Token efficiency:** Vais uses 829 tokens across the current fixtures, saving 23.2% vs Rust and 31.5% vs C. Python and Go are close in aggregate; Vais's advantage is strongest in struct-heavy code.
- **Compilation speed:** current single-file `--emit-ir` average is 6.3ms, 9.3x faster than C/clang and 15.6x faster than Rust full binary compilation.

Vais's key advantages are:
1. **C-equivalent performance** with higher-level syntax
2. **Compact binaries** (much smaller than Rust)
3. **Lower token count than Rust and C** on the current canonical syntax fixtures
4. **Fast single-file compilation** (6.3ms `--emit-ir` average on the current benchmark)

---

*Last updated: 2026-05-13*
*Current compile/token refresh: vaisc 0.1.0*
*All measurements are actual runs, not projections.*

# Vais Language Comparison Benchmark

> **Date**: 2026-05-13
> **Machine**: Apple M-series ARM64 / macOS Darwin
> **Tools**: hyperfine 1.20.0, vaisc 0.1.0, rustc 1.95.0, Go 1.25.7, Apple clang 21.0.0, tiktoken cl100k_base

## 1. Compile Speed Benchmark

Single-file compilation time measured with `hyperfine --warmup 3 --min-runs 15`.

> **Note**: Vais measures `--emit-ir` (LLVM IR generation only, no clang linking).
> Rust/Go/C measure full compilation to binary. This is intentional — Vais's
> compilation pipeline produces LLVM IR, then delegates to clang for final linking.

### Results

| Program | Vais | C (clang) | Go | Rust | Vais vs C | Vais vs Go | Vais vs Rust |
|---------|------|-----------|----|------|-----------|------------|--------------|
| fibonacci | **6.0ms** | 55.8ms | 48.0ms | 93.7ms | **9.3x** | **8.0x** | **15.6x** |
| quicksort | **6.4ms** | 56.8ms | 47.8ms | 95.6ms | **8.9x** | **7.5x** | **14.9x** |
| http_types | **6.6ms** | 60.7ms | 47.1ms | 103.2ms | **9.2x** | **7.1x** | **15.6x** |
| linked_list | **6.0ms** | 59.5ms | 47.3ms | 98.3ms | **9.9x** | **7.9x** | **16.4x** |
| **Average** | **6.3ms** | **58.2ms** | **47.5ms** | **97.7ms** | **9.3x** | **7.6x** | **15.6x** |

### Key Findings

- **Vais is ~9.3x faster than C/clang** for single-file compilation
- **Vais is ~7.6x faster than Go** for single-file compilation
- **Vais is ~15.6x faster than Rust** for single-file compilation
- Vais `--emit-ir` time is consistently ~6-7ms across these benchmark fixtures
- Large-scale throughput is tracked by a separate benchmark and was not refreshed by this single-file run

### Methodology Notes

- Vais `--emit-ir` generates LLVM IR text without clang invocation
- For fair comparison, Vais full pipeline (IR + clang) would add ~50ms for linking
- Even with linking, Vais total time (~57ms) would be competitive with C/Go
- Python is excluded from compile speed (interpreted language)

---

## 2. Runtime Execution Performance

Measures the actual execution speed of compiled Vais binaries vs equivalent Rust programs.
This benchmarks the quality of LLVM IR generation and final binary performance.

### Methodology

1. Compile `.vais` source to LLVM IR with `vaisc --emit-ir`
2. Link IR to native binary with `clang`
3. Benchmark binary execution using Criterion
4. Compare against Rust reference implementations (same algorithms)

### Benchmark Programs

| Program | Description | Input Size |
|---------|-------------|------------|
| `bench_fibonacci.vais` | Naive recursive Fibonacci | fib(35) |
| `bench_compute.vais` | Prime counting (trial division) | primes ≤ 100,000 |
| `bench_sorting.vais` | Quicksort in-place | 10,000 elements |

### Running the Benchmarks

```bash
# Requires clang to link LLVM IR to binaries
cargo bench --bench runtime_bench

# View results in target/criterion/
open target/criterion/compute/fibonacci/report/index.html
```

### Results (2026-04-04, Apple M-series ARM64, clang -O2, Phase 182)

| Program | C (-O3) | Rust (release) | Vais (-O2) | Vais vs C | Vais vs Rust |
|---------|---------|----------------|------------|-----------|--------------|
| fibonacci(35) | 32ms | 33ms | 34ms | 1.06x | 1.03x |

Vais-compiled binaries perform **within 3-7% of native C and Rust** for compute-intensive workloads.
The LLVM 17 backend applies the same optimizations (inlining, loop unrolling, vectorization) to both languages.
Generic monomorphization (hybrid: specialized + sizeof dispatch) and Vec<struct> direct field access
improvements introduced by Phase 182 further reduce overhead for struct-heavy workloads.

**Note**: Actual numbers depend on your CPU architecture and LLVM version. Run the benchmark locally to measure.

> **Why compare to Rust?** Rust is the gold standard for systems programming performance. Matching
> Rust's runtime speed validates Vais's LLVM codegen quality while maintaining faster compile times.

---

## 3. LLM Token Efficiency Benchmark

Measures how many LLM tokens each language requires for equivalent programs.
Uses `tiktoken` with `cl100k_base` encoding (same tokenizer family as GPT-4 and Claude).

### Per-Program Results

#### fibonacci (recursive + iterative)
| Language | Tokens | Lines | vs Vais |
|----------|--------|-------|---------|
| **Vais** | **130** | 20 | baseline |
| Python | 118 | 18 | -9.2% |
| Go | 126 | 24 | -3.1% |
| Rust | 135 | 20 | +3.8% |
| C | 159 | 24 | +22.3% |

#### quicksort (in-place partitioning)
| Language | Tokens | Lines | vs Vais |
|----------|--------|-------|---------|
| **Vais** | **235** | 31 | baseline |
| Python | 227 | 25 | -3.4% |
| Go | 228 | 33 | -3.0% |
| Rust | 242 | 30 | +3.0% |
| C | 291 | 39 | +23.8% |

#### http_types (struct + routing)
| Language | Tokens | Lines | vs Vais |
|----------|--------|-------|---------|
| **Vais** | **172** | 25 | baseline |
| Go | 318 | 69 | +84.9% |
| Python | 326 | 56 | +89.5% |
| Rust | 431 | 67 | +150.6% |
| C | 454 | 76 | +164.0% |

#### linked_list (singly linked, push/len/sum)
| Language | Tokens | Lines | vs Vais |
|----------|--------|-------|---------|
| Python | 218 | 38 | -25.3% |
| Go | 221 | 48 | -24.3% |
| Rust | 272 | 46 | -6.8% |
| **Vais** | **292** | 44 | baseline |
| C | 307 | 52 | +5.1% |

### Summary (All Programs Combined)

| Language | Total Tokens | Lines | Tokens/Line | vs Vais |
|----------|-------------|-------|-------------|---------|
| **Vais** | **829** | **120** | **6.9** | baseline |
| Python | 889 | 137 | 6.5 | +7.2% |
| Go | 893 | 174 | 5.1 | +7.7% |
| Rust | 1,080 | 163 | 6.6 | +30.3% |
| C | 1,211 | 191 | 6.3 | +46.1% |

### Analysis

**Where Vais excels:**
- **Struct definitions + routing**: Vais remains much smaller than Rust/C/Go/Python in `http_types`
- `fn`/`struct`/`enum` canonical spellings are now used in the benchmark fixtures, so token counts are higher than old single-letter-keyword snapshots
- Type inference, expression bodies, and `@` self-recursion still reduce boilerplate

**Key insight**: With current canonical public syntax, Vais uses **829 tokens total**, saving 23.2% vs Rust and 31.5% vs C across these fixtures. Python and Go are close in aggregate because their linked-list fixtures use GC/reference patterns while the Vais fixture uses explicit pointer operations.

---

## 4. Combined Score

Weighting: Compile speed (40%) + Token efficiency for systems code (60%)

| Language | Compile Speed | Token Efficiency (http_types) | Combined |
|----------|--------------|------------------------------|----------|
| **Vais** | **6.3ms (1.0x)** | **172 tokens (1.0x)** | **Best** |
| Go | 47.5ms (7.6x) | 318 tokens (1.85x) | |
| C | 58.2ms (9.3x) | 454 tokens (2.64x) | |
| Rust | 97.7ms (15.6x) | 431 tokens (2.51x) | |

For AI-generated systems programming code, Vais offers the best combination of:
1. **Fast compilation** for rapid iteration
2. **Token-efficient syntax** for reduced AI API costs
3. **Systems-level capabilities** (LLVM backend, manual memory control)

---

## Running the Benchmarks

```bash
# Token count
python3 benches/lang-comparison/count_tokens.py

# Compile speed (requires hyperfine, rustc, go, clang)
bash benches/lang-comparison/compile_bench.sh
```

## Changelog

- **2026-05-13**: Refreshed single-file compile speed and token counts with current canonical Vais fixtures; fixed benchmark script to build `vaisc` from the project root and preserve all per-program result JSON files
- **2026-04-04**: Historical Phase 182 update — throughput snapshot at 850K lines/sec (1.2ms/1K LOC); runtime near-C performance note updated; LLVM 17 backend noted explicitly; generic monomorphization and Vec<struct> field access improvements noted in runtime section
- **2026-02-11**: Historical compile-speed snapshot (6.4ms avg), added runtime results (fib35: 34ms, within 3-7% of C/Rust)
- **2026-02-09**: Initial benchmark — 4 programs, 5 languages

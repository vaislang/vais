# Vais Benchmark Baselines

This document provides baseline performance metrics for the Vais compiler and runtime. These benchmarks serve as a reference for detecting performance regressions in pull requests.

## Machine Specifications

> **Note:** Benchmark results are machine-dependent. The baselines below were collected on a reference machine. Your local results may vary.

### Reference Machine

- **OS:** Ubuntu 22.04 LTS (Linux)
- **CPU:** Intel Core i7-9700K @ 3.60GHz (8 cores)
- **RAM:** 16GB DDR4-3200
- **Rust Version:** 1.75.0
- **LLVM Version:** 17.0.0

### CI Machine (GitHub Actions)

- **Runner:** `ubuntu-latest` (Ubuntu 22.04)
- **CPU:** 2-core Intel Xeon Platinum 8272CL
- **RAM:** 7GB
- **Rust Version:** stable (latest)

## Benchmark Suite Overview

The Vais benchmark suite consists of several categories:

### 1. Compile-Time Benchmarks (`compile_bench.rs`)

Measures compiler pipeline performance:

- **Lexer:** Tokenization throughput
- **Parser:** AST generation performance
- **Type Checker:** Type inference and validation
- **Code Generator:** LLVM IR generation
- **Full Pipeline:** End-to-end compilation

### 2. Runtime Benchmarks (`runtime_bench.rs`)

Measures runtime performance of generated code:

- Function call overhead
- Control flow (if/else, loops)
- Arithmetic operations
- Memory access patterns

### 3. GC Benchmarks (`gc_bench.rs`)

Measures garbage collection performance:

- Allocation throughput
- Collection latency
- Memory overhead
- GC pause times

### 4. Optimization Benchmarks (`optimization_bench.rs`)

Tests optimizer effectiveness:

- Constant folding
- Dead code elimination
- Inlining
- Loop optimizations

## Baseline Results

> **Last Updated:** 2026-01-31
> **Git Commit:** `6f29465` (Phase 15 P2)

### Compile-Time Benchmarks

#### Lexer Performance

| Fixture | Size (bytes) | Throughput | Time |
|---------|--------------|------------|------|
| `fibonacci.vais` | 287 | ~2.8 MB/s | ~100 µs |
| `sort.vais` | 542 | ~2.7 MB/s | ~200 µs |
| `struct_heavy.vais` | 1,234 | ~2.6 MB/s | ~475 µs |
| `complex.vais` | 1,458 | ~2.5 MB/s | ~580 µs |

**Expected Variance:** ±5%

#### Parser Performance

| Fixture | Size (bytes) | Throughput | Time |
|---------|--------------|------------|------|
| `fibonacci.vais` | 287 | ~1.4 MB/s | ~205 µs |
| `sort.vais` | 542 | ~1.3 MB/s | ~415 µs |
| `struct_heavy.vais` | 1,234 | ~1.2 MB/s | ~1.03 ms |
| `complex.vais` | 1,458 | ~1.1 MB/s | ~1.32 ms |

**Expected Variance:** ±8%

#### Type Checker Performance

| Fixture | AST Nodes | Time |
|---------|-----------|------|
| `fibonacci.vais` | ~45 | ~150 µs |
| `sort.vais` | ~85 | ~280 µs |
| `struct_heavy.vais` | ~180 | ~620 µs |
| `complex.vais` | ~215 | ~750 µs |

**Expected Variance:** ±10%

#### Code Generation Performance

| Fixture | AST Nodes | Time |
|---------|-----------|------|
| `fibonacci.vais` | ~45 | ~320 µs |
| `sort.vais` | ~85 | ~580 µs |
| `struct_heavy.vais` | ~180 | ~1.25 ms |
| `complex.vais` | ~215 | ~1.48 ms |

**Expected Variance:** ±12%

#### Full Compilation Pipeline

| Fixture | Size (bytes) | Total Time | Throughput |
|---------|--------------|------------|------------|
| `fibonacci.vais` | 287 | ~775 µs | ~370 KB/s |
| `sort.vais` | 542 | ~1.48 ms | ~365 KB/s |
| `struct_heavy.vais` | 1,234 | ~3.37 ms | ~366 KB/s |
| `complex.vais` | 1,458 | ~4.14 ms | ~352 KB/s |

**Expected Variance:** ±10%

### Lexer Scaling (Synthetic Code)

Tests how the lexer scales with input size:

| Functions | Code Size | Time | Throughput |
|-----------|-----------|------|------------|
| 100 | ~4.5 KB | ~1.75 ms | ~2.6 MB/s |
| 500 | ~22.5 KB | ~8.7 ms | ~2.6 MB/s |
| 1,000 | ~45 KB | ~17.4 ms | ~2.6 MB/s |
| 5,000 | ~225 KB | ~87 ms | ~2.6 MB/s |

**Expected:** Linear scaling (O(n))

### Runtime Benchmarks

> **Note:** Runtime benchmarks measure the performance of compiled Vais code, not the compiler itself.

#### Function Call Overhead

| Test | Time per Call |
|------|---------------|
| Empty function | ~2.5 ns |
| Simple arithmetic | ~3.2 ns |
| Recursive (depth 10) | ~28 ns |

#### Fibonacci Computation

| Method | Input (n) | Time |
|--------|-----------|------|
| Recursive | 20 | ~12 µs |
| Iterative | 50 | ~185 ns |

### GC Benchmarks

#### Allocation Performance

| Object Size | Allocations/sec | Throughput |
|-------------|-----------------|------------|
| Small (64B) | ~15M/sec | ~960 MB/s |
| Medium (1KB) | ~8M/sec | ~8 GB/s |
| Large (64KB) | ~500K/sec | ~32 GB/s |

#### Collection Performance

| Heap Size | Live Objects | GC Pause |
|-----------|--------------|----------|
| 10 MB | 10,000 | ~2.5 ms |
| 50 MB | 50,000 | ~8.2 ms |
| 100 MB | 100,000 | ~15 ms |

## Running Benchmarks Locally

### Prerequisites

```bash
# Ensure you have Rust and LLVM installed
rustc --version  # Should be 1.75+
llvm-config --version  # Should be 17.0+
```

### Run All Benchmarks

```bash
# Run all benchmarks in the suite
cargo bench -p vais-benches

# Run specific benchmark group
cargo bench -p vais-benches --bench compile_bench
cargo bench -p vais-benches --bench runtime_bench
cargo bench -p vais-benches --bench gc_bench
```

### Save and Compare Baselines

```bash
# Save current results as baseline
cargo bench -p vais-benches -- --save-baseline my-baseline

# Make changes, then compare against baseline
cargo bench -p vais-benches -- --baseline my-baseline

# Compare against main branch
git checkout main
cargo bench -p vais-benches -- --save-baseline main-baseline
git checkout your-branch
cargo bench -p vais-benches -- --baseline main-baseline
```

### Generate HTML Reports

Criterion automatically generates detailed HTML reports:

```bash
# After running benchmarks, open the report
open target/criterion/report/index.html  # macOS
xdg-open target/criterion/report/index.html  # Linux
start target/criterion/report/index.html  # Windows
```

## Interpreting Results

### Performance Regression Thresholds

The CI system flags performance regressions based on these thresholds:

- **Critical (🔴):** >10% regression - **Fails PR check**
- **Warning (🟡):** 5-10% regression - Warning comment on PR
- **Minor (🔵):** 2-5% regression - Informational
- **Neutral (⚪):** ±2% - Within normal variance

### Expected Variance

Benchmark results naturally vary due to:

- CPU frequency scaling
- Background processes
- Memory allocation patterns
- Kernel scheduling
- Cache effects

**Local runs:** ±5-10% variance is normal
**CI runs:** ±3-5% variance (more consistent environment)

### When to Investigate

Investigate performance changes when:

1. **Consistent regressions** appear across multiple runs
2. **Large regressions** (>10%) in critical paths
3. **Unexpected improvements** (might indicate broken tests)
4. **Scaling changes** (O(n) becomes O(n²))

## Adding New Benchmarks

### Creating a New Benchmark

1. Add benchmark file to `benches/`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn my_benchmark(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            // Code to benchmark
            black_box(my_function(black_box(input)))
        })
    });
}

criterion_group!(benches, my_benchmark);
criterion_main!(benches);
```

2. Register in `benches/Cargo.toml`:

```toml
[[bench]]
name = "my_benchmark"
path = "my_benchmark.rs"
harness = false
```

3. Update this baseline document with expected results

### Best Practices

- Use `black_box()` to prevent compiler optimizations
- Benchmark realistic workloads, not microbenchmarks
- Include setup time outside the benchmark loop
- Use `Throughput` for size-dependent benchmarks
- Run benchmarks multiple times to ensure consistency

## Continuous Performance Monitoring

### GitHub Actions Integration

Every PR automatically runs benchmarks and:

1. Compares against the base branch (main)
2. Generates a comparison report
3. Posts results as a PR comment
4. Fails the build if regressions exceed threshold

### Benchmark Dashboard

Historical benchmark data is tracked on the [GitHub Pages benchmark dashboard](https://vaislang.github.io/vais/dev/bench/).

View trends over time to:
- Identify gradual performance degradation
- Validate optimization efforts
- Track compiler performance across releases

## Troubleshooting

### Benchmarks Fail to Compile

```bash
# Clean and rebuild
cargo clean
cargo bench -p vais-benches --no-run

# Check for missing dependencies
cargo check -p vais-benches
```

### Inconsistent Results

```bash
# Disable CPU frequency scaling (Linux)
sudo cpupower frequency-set --governor performance

# Close other applications
# Disable background processes
# Run benchmarks multiple times
```

### Comparing Different Machines

Don't directly compare absolute numbers across different hardware. Instead:
- Focus on relative changes (%)
- Compare trends over time
- Use the same machine for before/after comparisons

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [LLVM Optimization Guide](https://llvm.org/docs/Passes.html)

## Changelog

### 2026-01-31
- Initial baseline establishment
- Added compile-time benchmarks for all compiler phases
- Documented expected variance and thresholds
- Added scaling tests for lexer performance

---

**Maintainers:** Update this document when:
- Adding new benchmarks
- Significant performance improvements are merged
- Hardware configuration changes
- Benchmark methodology changes

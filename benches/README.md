# Vais Benchmark Suite

This directory contains comprehensive performance benchmarks for the Vais compiler and runtime. The benchmark suite is designed to detect performance regressions automatically in pull requests.

## Quick Start

```bash
# Run all benchmarks
cargo bench -p vais-benches

# Run specific benchmark suite
cargo bench -p vais-benches --bench compile_bench
cargo bench -p vais-benches --bench compiler_benchmarks
cargo bench -p vais-benches --bench runtime_bench
cargo bench -p vais-benches --bench gc_bench

# Run benchmarks matching a pattern
cargo bench -p vais-benches -- lexer
cargo bench -p vais-benches -- typecheck
```

## Benchmark Suites

### 1. `compile_bench.rs` - Core Compiler Benchmarks

Original benchmark suite measuring:
- **Lexer Performance:** Tokenization throughput for different file sizes
- **Parser Performance:** AST generation speed
- **Type Checker:** Type inference and validation
- **Code Generator:** LLVM IR generation
- **Full Pipeline:** End-to-end compilation
- **Lexer Scaling:** Performance with varying input sizes

### 2. `compiler_benchmarks.rs` - Comprehensive Compiler Tests

NEW! Extended compiler benchmarks including:
- **Medium File Processing:** Realistic file size benchmarks
- **Scaling Tests:** Performance across different file sizes
- **Complexity Tests:** Type checking with various code patterns
- **Code Generation Patterns:** Different code structures (recursion, iteration, structs)
- **Incremental Parsing:** Simulates LSP edit scenarios
- **Cache Effects:** Cold vs warm compilation
- **Error Handling:** Performance impact of error detection
- **Large Functions:** Compilation of functions with many statements
- **Many Functions:** Modules with numerous function definitions

### 3. `runtime_bench.rs` - Runtime Performance

Measures performance of compiled Vais code:
- Function call overhead
- Control flow performance
- Arithmetic operations
- Memory access patterns

### 4. `gc_bench.rs` - Garbage Collection

Tests GC performance:
- Allocation throughput
- Collection latency
- Memory overhead
- GC pause times

### 5. `optimization_bench.rs` - Optimizer Effectiveness

Validates optimization passes:
- Constant folding
- Dead code elimination
- Function inlining
- Loop optimizations

## Benchmark Fixtures

Test files in `fixtures/`:

- **`fibonacci.vais`** (287 bytes): Simple recursive and iterative functions
- **`sort.vais`** (542 bytes): Array operations and iteration
- **`struct_heavy.vais`** (1,234 bytes): Complex struct operations
- **`complex.vais`** (1,458 bytes): Mixed features for realistic workload

## Performance Regression Testing

### Automated PR Checks

Every pull request automatically:

1. **Runs benchmarks** on both PR and base branch
2. **Compares results** using criterion baselines
3. **Posts a comment** on the PR with performance diff
4. **Fails the build** if regressions exceed 10% threshold

### Workflow Files

- **`.github/workflows/bench.yml`**: Main benchmark dashboard (runs on push to main)
- **`.github/workflows/bench-regression.yml`**: PR regression checks (runs on PRs)

### Comparison Script

The `scripts/bench-compare.sh` script:
- Parses criterion benchmark JSON output
- Calculates percentage differences
- Generates markdown reports with status indicators
- Exits with error code if regressions exceed threshold

```bash
# Run comparison manually
./scripts/bench-compare.sh

# Set custom threshold (default: 10%)
BENCH_THRESHOLD=5 ./scripts/bench-compare.sh

# Compare specific git refs
BASE_REF=main CURRENT_REF=feature-branch ./scripts/bench-compare.sh
```

## Running Benchmarks Locally

### Basic Usage

```bash
# Run all benchmarks
cargo bench -p vais-benches

# Run with HTML reports (automatically generated)
cargo bench -p vais-benches
open target/criterion/report/index.html  # macOS
xdg-open target/criterion/report/index.html  # Linux
```

### Comparing Changes

```bash
# Save current performance as baseline
cargo bench -p vais-benches -- --save-baseline before

# Make your changes...

# Compare against baseline
cargo bench -p vais-benches -- --baseline before

# Compare against main branch
git checkout main
cargo bench -p vais-benches -- --save-baseline main
git checkout your-feature-branch
cargo bench -p vais-benches -- --baseline main
```

### Advanced Options

```bash
# Run benchmarks with sample size control
cargo bench -p vais-benches -- --sample-size 50

# Run warm-up iterations
cargo bench -p vais-benches -- --warm-up-time 5

# Save results to specific baseline
cargo bench -p vais-benches -- --save-baseline my-baseline

# List available benchmarks
cargo bench -p vais-benches -- --list

# Run benchmarks matching a filter
cargo bench -p vais-benches -- parse
cargo bench -p vais-benches -- "lexer.*fibonacci"
```

## Interpreting Results

### Criterion Output

Criterion provides detailed statistics:

```
parse_medium/complex_file
                        time:   [1.32 ms 1.35 ms 1.38 ms]
                        change: [+2.45% +3.12% +3.89%] (p = 0.00 < 0.05)
                        Performance has regressed.
```

- **time:** Mean time with confidence interval
- **change:** Performance change vs baseline (if comparing)
- **p-value:** Statistical significance

### Status Indicators

In PR comments and reports:

- 🔴 **Critical Regression:** >10% slower (fails CI)
- 🟡 **Minor Regression:** 5-10% slower (warning)
- 🔵 **Minor Improvement:** 2-10% faster
- 🟢 **Significant Improvement:** >10% faster
- ⚪ **Neutral:** Change within ±5% (normal variance)

### Normal Variance

Expect some variance due to:
- CPU frequency scaling
- Background processes
- Memory allocation patterns
- Cache effects
- OS scheduling

**Typical variance:** ±3-5% on CI, ±5-10% locally

## Baseline Documentation

See [`BASELINE.md`](./BASELINE.md) for:
- Reference performance numbers
- Machine specifications
- Expected variance ranges
- Historical trends

## Adding New Benchmarks

### 1. Create Benchmark Function

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn my_benchmark(c: &mut Criterion) {
    c.bench_function("my_test", |b| {
        b.iter(|| {
            // Code to benchmark
            black_box(my_function(black_box(input)))
        })
    });
}

criterion_group!(benches, my_benchmark);
criterion_main!(benches);
```

### 2. Register in Cargo.toml

```toml
[[bench]]
name = "my_benchmark"
path = "my_benchmark.rs"
harness = false
```

### 3. Update Documentation

- Add expected results to `BASELINE.md`
- Update this README with benchmark description

### Best Practices

1. **Use `black_box()`** to prevent compiler optimizations
2. **Separate setup** from benchmarked code
3. **Use realistic inputs** that represent actual usage
4. **Set appropriate sample sizes** (default: 100)
5. **Document expected performance** in BASELINE.md
6. **Test for regressions** before committing changes

## Troubleshooting

### Benchmarks Won't Compile

```bash
# Clean and rebuild
cargo clean
cargo bench -p vais-benches --no-run

# Check dependencies
cargo check -p vais-benches
```

### Inconsistent Results

```bash
# Close other applications
# Disable CPU frequency scaling (Linux):
sudo cpupower frequency-set --governor performance

# Run multiple times
for i in {1..5}; do
  cargo bench -p vais-benches -- my_benchmark
done
```

### Criterion Cache Issues

```bash
# Clear criterion cache
rm -rf target/criterion

# Clear all baselines
rm -rf target/criterion/*/*/base/
```

### Comparing Across Machines

Don't compare absolute numbers across different hardware. Instead:
- Focus on **relative changes** (percentage)
- Compare **trends over time** on the same machine
- Use **CI results** as the source of truth for PRs

## CI Integration

### GitHub Actions

Benchmark workflows run automatically:

**On Pull Requests:**
- Compares PR branch vs base branch
- Posts results as PR comment
- Fails if regressions exceed threshold

**On Push to Main:**
- Saves results to benchmark dashboard
- Tracks historical performance
- Publishes to GitHub Pages

### Local CI Testing

Test the CI workflow locally:

```bash
# Install act (GitHub Actions local runner)
brew install act  # macOS
# or
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Run benchmark workflow
act pull_request -W .github/workflows/bench-regression.yml
```

## Performance Optimization Tips

When fixing regressions:

1. **Profile first:** Use `cargo flamegraph` or `perf` to identify bottlenecks
2. **Measure impact:** Run benchmarks before and after changes
3. **Target hot paths:** Focus on code executed frequently
4. **Check algorithms:** O(n) vs O(n²) matters more than micro-optimizations
5. **Validate with benchmarks:** Ensure optimizations actually help

## Resources

- [Criterion.rs Book](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [LLVM Optimization Passes](https://llvm.org/docs/Passes.html)
- [Benchmarking Best Practices](https://easyperf.net/blog/)

## Contact

For questions about benchmarks:
- Check existing baselines in `BASELINE.md`
- Review criterion HTML reports in `target/criterion/report/`
- Ask in GitHub issues or PR comments

---

**Last Updated:** 2026-01-31
**Maintained by:** Vais Team

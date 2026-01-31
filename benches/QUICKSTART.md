# Performance Benchmarks - Quick Start Guide

## TL;DR

```bash
# Run all benchmarks
cargo bench -p vais-benches

# Compare your branch against main
./scripts/run-bench-comparison.sh main

# View detailed HTML reports
open target/criterion/report/index.html
```

## Common Tasks

### Running Benchmarks

```bash
# All benchmarks
cargo bench -p vais-benches

# Specific suite
cargo bench -p vais-benches --bench compile_bench
cargo bench -p vais-benches --bench compiler_benchmarks

# Specific benchmark group
cargo bench -p vais-benches -- lexer
cargo bench -p vais-benches -- typecheck
```

### Before Creating a PR

```bash
# 1. Run comparison against main
./scripts/run-bench-comparison.sh main

# 2. If regressions found:
#    - Investigate the cause
#    - Profile if needed: cargo flamegraph --bench compile_bench
#    - Fix and re-test

# 3. Check results
open target/criterion/report/index.html
```

### After PR Comments

Your PR will automatically get a comment like this:

```
📊 Benchmark Comparison Report

| Status | Benchmark | Base | PR | Change |
|--------|-----------|------|-----|--------|
| 🔴 | lexer/tokenize/complex | 580 µs | 650 µs | +12.1% |
| ⚪ | parser/parse/fibonacci | 205 µs | 210 µs | +2.4% |
| 🟢 | typecheck/check/sort | 280 µs | 245 µs | -12.5% |
```

**What to do:**

- 🔴 **Critical Regression (>10%):** Must fix before merge
  ```bash
  # Reproduce locally
  ./scripts/run-bench-comparison.sh main

  # Profile to find issue
  cargo flamegraph --bench compile_bench -- lexer

  # Fix and verify
  cargo bench -p vais-benches -- lexer
  ```

- 🟡 **Minor Regression (5-10%):** Investigate
  - Is it expected? (e.g., added features)
  - Can it be optimized?
  - Document in PR if intentional

- 🟢 **Improvement:** Great!
  - Verify it's real (run multiple times)
  - Ensure tests still pass

- ⚪ **Neutral (±5%):** Normal variance, OK to merge

### Understanding Results

**Criterion Output:**
```
parse_medium/complex_file
    time:   [1.32 ms 1.35 ms 1.38 ms]
    change: [+2.45% +3.12% +3.89%]
```

- First line: benchmark name
- `time:` = [lower bound, mean, upper bound] with 95% confidence
- `change:` = performance difference vs baseline

## Quick Reference

### Status Emojis

| Symbol | Meaning | Action |
|--------|---------|--------|
| 🔴 | >10% slower | **Fix required** |
| 🟡 | 5-10% slower | Investigate |
| ⚪ | ±5% | Normal |
| 🔵 | 2-10% faster | Good! |
| 🟢 | >10% faster | Excellent! |

### Benchmark Suites

| Suite | What It Tests |
|-------|---------------|
| `compile_bench` | Core compiler (lexer, parser, typechecker, codegen) |
| `compiler_benchmarks` | Extended compiler tests (scaling, patterns, etc.) |
| `runtime_bench` | Generated code performance |
| `gc_bench` | Garbage collection |

### Useful Commands

```bash
# Save current performance
cargo bench -p vais-benches -- --save-baseline before

# Make changes...

# Compare against saved baseline
cargo bench -p vais-benches -- --baseline before

# Run specific benchmark
cargo bench -p vais-benches -- "lexer.*fibonacci"

# Profile a benchmark
cargo flamegraph --bench compile_bench -- lexer
```

## Troubleshooting

**Benchmarks fail to compile:**
```bash
cargo clean
cargo check -p vais-benches --benches
```

**Inconsistent results:**
```bash
# Close other apps, run multiple times
for i in {1..3}; do
  cargo bench -p vais-benches -- my_benchmark
done
```

**Need help:**
- Read [`README.md`](./README.md) for detailed guide
- Check [`BASELINE.md`](./BASELINE.md) for expected performance
- See [`../docs/PERFORMANCE_TESTING.md`](../docs/PERFORMANCE_TESTING.md) for complete guide

## Pro Tips

1. **Always compare against main before creating PR**
2. **Focus on CI results** (local can vary)
3. **Use `black_box()` when writing new benchmarks**
4. **Check HTML reports for detailed statistics**
5. **Document intentional performance trade-offs in PR**

---

**More Info:** See [README.md](./README.md) | [BASELINE.md](./BASELINE.md) | [Performance Testing Guide](../docs/PERFORMANCE_TESTING.md)

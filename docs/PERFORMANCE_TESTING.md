# Performance Regression Testing Guide

This document describes the automated performance regression testing infrastructure for the Vais compiler.

## Overview

The Vais project includes a comprehensive benchmark suite that automatically detects performance regressions in pull requests. Every PR is tested against the base branch (usually `main`), and any performance degradation beyond the threshold triggers a CI failure.

## Quick Reference

### For Contributors

**Before creating a PR:**
```bash
# Compare your changes against main
./scripts/run-bench-comparison.sh main
```

**Interpreting PR comments:**
- 🔴 **Critical Regression** (>10%): Must be fixed before merge
- 🟡 **Minor Regression** (5-10%): Should be investigated
- 🟢 **Significant Improvement** (>10%): Great work!
- ⚪ **Neutral** (±5%): Normal variance

**Fixing regressions:**
1. Run benchmarks locally to reproduce
2. Profile the affected code path
3. Fix the issue and verify improvement
4. Push updated code to trigger new benchmark run

### For Reviewers

**What to look for:**
- Critical regressions (🔴) should block merge
- Multiple minor regressions may indicate a systemic issue
- Large improvements deserve validation (not broken tests)
- Check if performance trade-offs are intentional

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                     Performance Testing                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐      ┌──────────────┐                    │
│  │   Benchmark  │      │   Benchmark  │                    │
│  │    Suites    │      │   Fixtures   │                    │
│  │              │      │              │                    │
│  │ • compile    │      │ • fibonacci  │                    │
│  │ • compiler   │      │ • sort       │                    │
│  │ • runtime    │      │ • struct     │                    │
│  │ • gc         │      │ • complex    │                    │
│  └──────┬───────┘      └──────┬───────┘                    │
│         │                     │                            │
│         └──────────┬──────────┘                            │
│                    │                                       │
│         ┌──────────▼──────────┐                            │
│         │   Criterion.rs      │                            │
│         │   (Benchmark Tool)  │                            │
│         └──────────┬──────────┘                            │
│                    │                                       │
│         ┌──────────▼──────────┐                            │
│         │   CI Workflows      │                            │
│         │                     │                            │
│         │ • bench.yml         │                            │
│         │ • bench-regression  │                            │
│         └──────────┬──────────┘                            │
│                    │                                       │
│         ┌──────────▼──────────┐                            │
│         │  Comparison Script  │                            │
│         │  (bench-compare.sh) │                            │
│         └──────────┬──────────┘                            │
│                    │                                       │
│         ┌──────────▼──────────┐                            │
│         │   PR Comment        │                            │
│         │   • Results table   │                            │
│         │   • Status emojis   │                            │
│         │   • Recommendations │                            │
│         └─────────────────────┘                            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Benchmark Suites

#### 1. `compile_bench.rs` - Core Compilation
- Lexer throughput
- Parser performance
- Type checker speed
- Code generator
- Full compilation pipeline
- Scaling tests

#### 2. `compiler_benchmarks.rs` - Extended Tests
- Medium file processing
- Different file sizes
- Type checking complexity
- Code generation patterns
- Incremental parsing (LSP simulation)
- Cache effects
- Error handling overhead
- Large functions
- Many functions

#### 3. `runtime_bench.rs` - Runtime Performance
- Function call overhead
- Control flow
- Arithmetic operations
- Memory access

#### 4. `gc_bench.rs` - Garbage Collection
- Allocation throughput
- Collection latency
- Memory overhead
- GC pause times

### Test Fixtures

Located in `benches/fixtures/`:

| File | Size | Features |
|------|------|----------|
| `fibonacci.vais` | 287 B | Recursion, tail-call optimization |
| `sort.vais` | 542 B | Arrays, iteration, comparisons |
| `struct_heavy.vais` | 1.2 KB | Structs, enums, pattern matching |
| `complex.vais` | 1.5 KB | Mixed features, realistic code |

## CI Workflows

### `bench.yml` - Main Dashboard

**Triggers:** Push to `main` branch

**Actions:**
1. Run full benchmark suite
2. Save results to GitHub Pages
3. Track historical performance
4. Alert on significant changes

**Output:**
- Benchmark dashboard at `https://vaislang.github.io/vais/dev/bench/`
- Performance trends over time
- Downloadable results artifacts

### `bench-regression.yml` - PR Checks

**Triggers:** Pull requests to `main`

**Actions:**
1. Checkout PR branch
2. Run benchmarks → save as "pr" baseline
3. Checkout base branch
4. Run benchmarks → save as "base" baseline
5. Compare results
6. Post PR comment with diff
7. Fail build if regressions exceed threshold

**Configuration:**
- `BENCH_THRESHOLD`: 10% (configurable)
- Timeout: 30 minutes
- Runs on: `ubuntu-latest`

## Thresholds and Interpretation

### Regression Severity

| Change | Status | Symbol | Action |
|--------|--------|--------|--------|
| >10% slower | Critical | 🔴 | **Fails CI** - must fix |
| 5-10% slower | Minor Regression | 🟡 | Investigate before merge |
| ±5% | Neutral | ⚪ | Normal variance |
| 2-10% faster | Minor Improvement | 🔵 | Good! |
| >10% faster | Significant Improvement | 🟢 | Excellent! Verify not broken |

### Expected Variance

**CI Environment (GitHub Actions):**
- Normal: ±3-5%
- Acceptable: ±5-8%
- Concerning: >8%

**Local Development:**
- Normal: ±5-10%
- Depends on: background processes, CPU scaling, thermal throttling

### Statistical Significance

Criterion uses:
- **Sample size:** 100 measurements (default)
- **Confidence interval:** 95%
- **Outlier detection:** Automated
- **Warm-up:** 3 seconds

## Usage Guide

### Running Benchmarks Locally

**All benchmarks:**
```bash
cargo bench -p vais-benches
```

**Specific suite:**
```bash
cargo bench -p vais-benches --bench compile_bench
cargo bench -p vais-benches --bench compiler_benchmarks
```

**Filtered benchmarks:**
```bash
cargo bench -p vais-benches -- lexer
cargo bench -p vais-benches -- "typecheck.*complex"
```

### Comparing Against Base Branch

**Using the comparison script:**
```bash
./scripts/run-bench-comparison.sh main
```

**Manual comparison:**
```bash
# Benchmark base branch
git checkout main
cargo bench -p vais-benches -- --save-baseline main

# Benchmark your changes
git checkout your-feature-branch
cargo bench -p vais-benches -- --baseline main
```

### Analyzing Results

**HTML reports:**
```bash
# After running benchmarks
open target/criterion/report/index.html  # macOS
xdg-open target/criterion/report/index.html  # Linux
```

**JSON data:**
```bash
# Estimate files contain raw statistics
cat target/criterion/*/base/estimates.json | jq .
```

**Analysis script:**
```bash
BENCH_THRESHOLD=5 ./scripts/bench-compare.sh
```

## Adding New Benchmarks

### Step 1: Create Benchmark File

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn my_benchmark(c: &mut Criterion) {
    c.bench_function("my_test", |b| {
        b.iter(|| {
            // Setup
            let input = create_test_input();

            // Benchmark this part
            black_box(function_to_test(black_box(input)))
        })
    });
}

criterion_group!(benches, my_benchmark);
criterion_main!(benches);
```

### Step 2: Register in Cargo.toml

```toml
# benches/Cargo.toml

[[bench]]
name = "my_benchmark"
path = "my_benchmark.rs"
harness = false
```

### Step 3: Document Expected Performance

Add to `benches/BASELINE.md`:

```markdown
### My Benchmark

| Test Case | Expected Time | Variance |
|-----------|---------------|----------|
| Small input | ~100 µs | ±5% |
| Large input | ~1.2 ms | ±8% |
```

### Step 4: Verify

```bash
# Test that it compiles
cargo bench -p vais-benches --bench my_benchmark --no-run

# Run it
cargo bench -p vais-benches --bench my_benchmark
```

## Best Practices

### Writing Good Benchmarks

**DO:**
- Use `black_box()` to prevent compiler optimizations
- Separate setup from measured code
- Use realistic input data
- Test edge cases (empty, small, large)
- Document expected performance
- Run multiple times to verify consistency

**DON'T:**
- Benchmark trivial operations (< 1µs)
- Include setup time in measurements
- Use hardcoded constants that get optimized away
- Test without representative data
- Ignore variance warnings

### Interpreting Results

**When results show regression:**

1. **Reproduce locally:**
   ```bash
   ./scripts/run-bench-comparison.sh main
   ```

2. **Profile the code:**
   ```bash
   # Install flamegraph
   cargo install flamegraph

   # Profile
   cargo flamegraph --bench compile_bench
   ```

3. **Check for algorithmic changes:**
   - O(n) → O(n log n) → O(n²)?
   - Extra allocations?
   - Unnecessary copying?

4. **Verify with micro-benchmarks:**
   - Isolate the slow part
   - Create focused benchmark
   - Test different approaches

**When results show improvement:**

1. **Verify it's real:**
   - Run multiple times
   - Check different inputs
   - Ensure tests still pass

2. **Understand why:**
   - Algorithm change?
   - Better caching?
   - Compiler optimization?

3. **Document it:**
   - Update BASELINE.md
   - Note in commit message
   - Mention in PR description

## Troubleshooting

### Problem: Inconsistent Results Locally

**Solution:**
```bash
# Close other applications
# Disable CPU frequency scaling (Linux)
sudo cpupower frequency-set --governor performance

# Run multiple times
for i in {1..5}; do
  cargo bench -p vais-benches -- my_benchmark
done
```

### Problem: CI Fails but Local Passes

**Causes:**
- Different CPU architecture
- Different load patterns
- CI has less RAM/CPU
- Timing-dependent test

**Solution:**
- Focus on CI results as source of truth
- Run on same OS as CI (Ubuntu 22.04)
- Use Docker to match CI environment
- Check for non-deterministic behavior

### Problem: Benchmarks Won't Compile

**Solution:**
```bash
# Clean build
cargo clean
cargo bench -p vais-benches --no-run

# Check for errors
cargo check -p vais-benches --benches

# Update dependencies
cargo update
```

### Problem: Results Show High Variance

**Causes:**
- Non-deterministic code
- Random number generation
- Hash maps (random seed)
- Timing-dependent logic

**Solution:**
- Use deterministic seeds
- Use BTreeMap instead of HashMap
- Increase sample size
- Add warm-up iterations

## Performance Optimization Workflow

### 1. Identify Regression

PR comment shows:
```
🔴 lexer/tokenize/complex: +15.3%
```

### 2. Reproduce Locally

```bash
./scripts/run-bench-comparison.sh main
```

### 3. Profile

```bash
# Install tools
cargo install flamegraph
cargo install cargo-profdata

# Profile
cargo flamegraph --bench compile_bench -- --bench lexer
```

### 4. Analyze

Look for:
- Hot functions (>10% time)
- Unexpected allocations
- Excessive copying
- Cache misses

### 5. Fix

Common optimizations:
- Use references instead of clones
- Pre-allocate collections
- Use iterators instead of loops
- Cache expensive computations
- Avoid string allocations

### 6. Verify

```bash
# Benchmark fix
cargo bench -p vais-benches -- --baseline before

# Should see improvement
```

### 7. Document

Update PR with:
- Root cause analysis
- Fix description
- Performance impact
- Benchmark results

## References

### Documentation

- [Criterion.rs Book](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Benchmarking Best Practices](https://easyperf.net/blog/)

### Project Files

- `benches/BASELINE.md` - Expected performance baselines
- `benches/README.md` - Benchmark suite documentation
- `scripts/bench-compare.sh` - Comparison script
- `scripts/run-bench-comparison.sh` - Local comparison helper

### GitHub Actions

- `.github/workflows/bench.yml` - Main benchmark dashboard
- `.github/workflows/bench-regression.yml` - PR regression checks

## FAQ

**Q: Why did my PR fail with a performance regression?**

A: Your changes caused a benchmark to run >10% slower than the base branch. Review the PR comment for specific benchmarks affected.

**Q: The regression is intentional (e.g., more features). What do I do?**

A: Document the trade-off in your PR description and get reviewer approval. Consider if the regression can be mitigated.

**Q: Can I change the threshold?**

A: The 10% threshold is set in workflow files. Changes should be discussed with maintainers.

**Q: Why are local results different from CI?**

A: Different hardware, OS, and load. CI results are more consistent and should be considered authoritative.

**Q: How do I add a new benchmark?**

A: See "Adding New Benchmarks" section above. Remember to document expected performance.

**Q: Can I disable benchmark checks for my PR?**

A: No, performance regression checks run on all PRs affecting the compiler.

---

**Last Updated:** 2026-01-31
**Maintained by:** Vais Team
**Questions?** Open an issue or ask in PR comments

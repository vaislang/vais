# Performance Regression Testing Infrastructure - Setup Summary

This document summarizes the complete performance regression testing infrastructure that has been implemented for the Vais programming language project.

## Overview

A comprehensive automated performance regression testing system has been set up to:
- Automatically benchmark every pull request
- Compare PR performance against the base branch
- Flag regressions exceeding 10% threshold
- Generate detailed performance reports
- Track performance trends over time

## What Was Created

### 1. Benchmark Suites

#### New Benchmark File
**File:** `benches/compiler_benchmarks.rs`

A comprehensive compiler benchmark suite with 11 benchmark groups:
- `bench_parse_medium_file` - Medium-sized file parsing
- `bench_typecheck_medium_file` - Type checking performance
- `bench_full_compile_medium` - Full compilation pipeline
- `bench_parse_file_sizes` - Scaling across different file sizes
- `bench_typecheck_complexity` - Type checking with various patterns
- `bench_codegen_patterns` - Code generation for different structures
- `bench_incremental_parse` - Incremental parsing (LSP simulation)
- `bench_compilation_warmup` - Cold vs warm compilation
- `bench_error_handling` - Error detection overhead
- `bench_large_functions` - Large function compilation
- `bench_many_functions` - Modules with many functions

**Total Benchmarks:** 30+ individual benchmark cases

#### Updated Configuration
**File:** `benches/Cargo.toml`

Added registration for new compiler benchmarks:
```toml
[[bench]]
name = "compiler_benchmarks"
path = "compiler_benchmarks.rs"
harness = false
```

### 2. CI/CD Workflows

#### New PR Regression Check Workflow
**File:** `.github/workflows/bench-regression.yml`

A complete GitHub Actions workflow that:
1. Checks out PR branch and runs benchmarks
2. Checks out base branch and runs benchmarks
3. Compares results using Python script
4. Posts detailed comparison as PR comment
5. Fails build if regressions exceed 10% threshold
6. Includes compile-time tracking job

**Key Features:**
- Automatic baseline comparison
- Statistical analysis
- Markdown table generation
- Status emojis (🔴🟡🟢🔵⚪)
- Artifact upload for historical tracking
- Updates existing comments (no spam)

#### Existing Dashboard Workflow
**File:** `.github/workflows/bench.yml` (already exists)

Complements the PR checks with:
- Historical tracking on main branch
- GitHub Pages dashboard
- Long-term trend analysis

### 3. Comparison Scripts

#### Detailed Comparison Script
**File:** `scripts/bench-compare.sh`

A robust bash/jq-based benchmark comparison tool:
- Parses criterion JSON output
- Calculates percentage changes
- Generates markdown reports
- Color-coded terminal output
- Configurable threshold (default: 10%)
- Exit codes for CI integration
- Handles missing benchmarks gracefully

**Features:**
- Human-readable time formatting (ns/µs/ms/s)
- Statistical summaries
- Regression categorization
- Improvement detection

#### Local Development Helper
**File:** `scripts/run-bench-comparison.sh`

A convenient script for developers:
- Compares current branch vs main
- Handles git operations automatically
- Uses Python for comparison
- Clean terminal output
- Interactive workflow

**Usage:**
```bash
./scripts/run-bench-comparison.sh main
```

### 4. Documentation

#### Baseline Documentation
**File:** `benches/BASELINE.md`

Comprehensive baseline performance documentation:
- Reference machine specifications
- CI machine specifications
- Expected performance for all benchmarks
- Variance ranges
- Historical data
- Instructions for running benchmarks
- Troubleshooting guide
- Adding new benchmarks

**Contents:**
- Lexer performance baselines
- Parser performance baselines
- Type checker baselines
- Code generator baselines
- Full pipeline baselines
- Runtime benchmarks
- GC benchmarks
- Expected variance ranges

#### Benchmark Suite Documentation
**File:** `benches/README.md`

Complete user guide for the benchmark suite:
- Quick start guide
- Benchmark suite descriptions
- Running benchmarks locally
- Comparing changes
- Interpreting results
- Adding new benchmarks
- Best practices
- Troubleshooting
- CI integration

#### Performance Testing Guide
**File:** `docs/PERFORMANCE_TESTING.md`

High-level guide for contributors and reviewers:
- Overview of regression testing
- Architecture diagram
- Quick reference for contributors
- Quick reference for reviewers
- Threshold interpretation
- Usage examples
- Optimization workflow
- FAQ

## Directory Structure

```
vais/
├── .github/
│   └── workflows/
│       ├── bench.yml                    # Existing: main branch tracking
│       └── bench-regression.yml         # NEW: PR regression checks
│
├── benches/
│   ├── fixtures/
│   │   ├── fibonacci.vais              # Existing test files
│   │   ├── sort.vais
│   │   ├── struct_heavy.vais
│   │   └── complex.vais
│   ├── compile_bench.rs                 # Existing: core benchmarks
│   ├── compiler_benchmarks.rs           # NEW: comprehensive compiler tests
│   ├── runtime_bench.rs                 # Existing: runtime benchmarks
│   ├── gc_bench.rs                      # Existing: GC benchmarks
│   ├── optimization_bench.rs            # Existing: optimizer tests
│   ├── lib.rs                           # Existing: utilities
│   ├── Cargo.toml                       # Updated: new benchmark registration
│   ├── BASELINE.md                      # NEW: baseline documentation
│   └── README.md                        # NEW: benchmark suite guide
│
├── scripts/
│   ├── bench-compare.sh                 # NEW: detailed comparison tool
│   └── run-bench-comparison.sh          # NEW: local development helper
│
├── docs/
│   └── PERFORMANCE_TESTING.md           # NEW: performance testing guide
│
└── PERFORMANCE_REGRESSION_SETUP.md      # This file
```

## How It Works

### PR Workflow

1. **Developer creates PR**
   - CI automatically triggers `bench-regression.yml`

2. **Benchmark PR branch**
   - Checks out PR code
   - Runs all benchmarks
   - Saves results as "pr" baseline

3. **Benchmark base branch**
   - Checks out main/base
   - Runs same benchmarks
   - Saves results as "base" baseline

4. **Compare and report**
   - Python script compares results
   - Calculates percentage changes
   - Generates markdown table
   - Posts as PR comment

5. **Check thresholds**
   - If any benchmark regresses >10%: ❌ Fail build
   - Otherwise: ✅ Pass

6. **Developer response**
   - Fix regression if critical (🔴)
   - Investigate if concerning (🟡)
   - Celebrate if improvement (🟢)

### Local Development Workflow

1. **Make changes**
   ```bash
   # Edit code...
   ```

2. **Compare against main**
   ```bash
   ./scripts/run-bench-comparison.sh main
   ```

3. **Review results**
   - Check for regressions
   - Investigate if needed
   - Fix and retest

4. **Create PR**
   - Automated checks will run
   - Results posted as comment

## Benchmark Coverage

### Compiler Phases

| Phase | Benchmarks | Coverage |
|-------|------------|----------|
| Lexing | 8 | Small/medium/large files, scaling |
| Parsing | 8 | Different complexities, file sizes |
| Type Checking | 6 | Various code patterns |
| Code Generation | 6 | Different IR patterns |
| Full Pipeline | 5 | End-to-end compilation |
| Error Handling | 2 | Valid vs invalid code |
| Scaling | 10 | Large functions, many functions |

**Total:** 45+ individual benchmark cases

### Input Sizes

| Category | Size Range | Examples |
|----------|------------|----------|
| Small | 100-500 bytes | Simple functions |
| Medium | 500-1500 bytes | Realistic modules |
| Large | Generated code | Stress tests |
| Scaling | 10-200 functions | Module complexity |

## Performance Thresholds

### Regression Severity

| Status | Change | Symbol | CI Behavior |
|--------|--------|--------|-------------|
| Critical Regression | >10% slower | 🔴 | **FAIL** |
| Minor Regression | 5-10% slower | 🟡 | Pass with warning |
| Neutral | ±5% | ⚪ | Pass |
| Minor Improvement | 2-10% faster | 🔵 | Pass |
| Significant Improvement | >10% faster | 🟢 | Pass |

### Configuration

```yaml
env:
  BENCH_THRESHOLD: 10  # Percentage for critical regression
```

## Usage Examples

### For Contributors

**Before submitting PR:**
```bash
# Compare your changes
./scripts/run-bench-comparison.sh main

# If regressions found, investigate
cargo bench -p vais-benches -- --baseline main

# Profile to find bottleneck
cargo install flamegraph
cargo flamegraph --bench compile_bench
```

**Responding to PR comments:**
```bash
# Reproduce locally
./scripts/run-bench-comparison.sh main

# Fix issue

# Verify fix
cargo bench -p vais-benches -- my_benchmark

# Push update (triggers new benchmark run)
git push
```

### For Reviewers

**Evaluating performance impact:**
```bash
# Check out PR branch
gh pr checkout 123

# Run comparison
./scripts/run-bench-comparison.sh main

# Review detailed HTML reports
open target/criterion/report/index.html
```

## Customization

### Changing Threshold

Edit `.github/workflows/bench-regression.yml`:
```yaml
env:
  BENCH_THRESHOLD: 15  # Change from 10 to 15
```

### Adding New Benchmarks

1. Create benchmark function in `benches/compiler_benchmarks.rs`
2. Add to `criterion_group!()` macro
3. Document expected performance in `benches/BASELINE.md`
4. Test locally: `cargo bench -p vais-benches -- new_benchmark`

### Excluding Files from Triggers

Edit `.github/workflows/bench-regression.yml`:
```yaml
on:
  pull_request:
    paths:
      - 'crates/**'
      - 'benches/**'
      # Add exclusions:
      - '!docs/**'
      - '!**.md'
```

## Verification

### Check Everything Works

```bash
# 1. Verify benchmarks compile
cargo check -p vais-benches --benches

# 2. Run quick test
cargo bench -p vais-benches --bench compiler_benchmarks -- --test

# 3. Test comparison script
./scripts/bench-compare.sh

# 4. Test local comparison
./scripts/run-bench-comparison.sh main

# 5. Validate workflow syntax
# (requires act: https://github.com/nektos/act)
act -l -W .github/workflows/bench-regression.yml
```

### Expected Output

**Benchmark test mode:**
```
Testing parse_medium/complex_file
Success
Testing typecheck_medium/complex_file
Success
...
```

**Comparison script:**
```
====================================
Vais Benchmark Comparison
====================================
...
✅ PASSED: No regressions exceed 10% threshold
```

## Maintenance

### Regular Tasks

**Weekly:**
- Review benchmark trends on dashboard
- Check for gradual degradation

**Per Release:**
- Update `benches/BASELINE.md` with new baselines
- Review and adjust thresholds if needed
- Add benchmarks for new features

**As Needed:**
- Investigate persistent variance
- Update fixtures for new language features
- Optimize slow benchmarks

### Monitoring

**GitHub Actions:**
- Check workflow runs: https://github.com/vaislang/vais/actions
- Review failed runs for patterns
- Monitor execution time

**Benchmark Dashboard:**
- View trends: https://vaislang.github.io/vais/dev/bench/
- Look for gradual degradation
- Validate optimizations

## Troubleshooting

### Common Issues

**Issue:** Benchmarks fail to compile
```bash
cargo clean
cargo check -p vais-benches --benches
```

**Issue:** High variance in results
- Close background applications
- Run multiple times
- Check CPU temperature
- Use consistent power settings

**Issue:** CI passes but local fails
- CI environment is different
- Focus on CI results as authoritative
- Match CI OS (Ubuntu 22.04)

**Issue:** Workflow doesn't trigger
- Check file paths in trigger configuration
- Ensure PR targets main branch
- Verify workflow file syntax

## Resources

### Documentation
- `benches/BASELINE.md` - Performance baselines
- `benches/README.md` - Benchmark usage guide
- `docs/PERFORMANCE_TESTING.md` - Testing guide

### External Resources
- [Criterion.rs Book](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [GitHub Actions Documentation](https://docs.github.com/actions)

## Summary

The performance regression testing infrastructure provides:

✅ **Automated PR Checks** - Every PR is benchmarked automatically
✅ **Detailed Comparisons** - Clear performance diff with status indicators
✅ **Local Development Tools** - Easy comparison scripts for developers
✅ **Comprehensive Coverage** - 45+ benchmarks across all compiler phases
✅ **Configurable Thresholds** - 10% default, easily adjustable
✅ **Complete Documentation** - Guides for contributors, reviewers, and maintainers
✅ **Historical Tracking** - Dashboard for long-term trend analysis
✅ **Developer-Friendly** - Clear output, actionable feedback

## Next Steps

1. **Merge this PR** to enable performance regression testing
2. **Establish baselines** by running benchmarks on main branch
3. **Monitor first few PRs** to validate thresholds
4. **Adjust if needed** based on real-world usage
5. **Add more benchmarks** as new features are developed

---

**Created:** 2026-01-31
**Status:** Ready for review and merge
**Maintainer:** Vais Team

For questions or issues, please open a GitHub issue or ask in PR comments.

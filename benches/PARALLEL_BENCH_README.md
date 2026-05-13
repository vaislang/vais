# Parallel Compilation Benchmark Suite

Measures parallelization speedup from **Phase 2 Stage 2** parallel compilation features.

## Benchmark Groups

### 1. Parse Speedup (`parse_speedup/*`)
Direct comparison of sequential vs parallel parsing for 10/50/100 modules.

```bash
cargo bench --bench parallel_bench -- parse_speedup
```

**Expected Results:**
- 10 modules: 2-3x speedup
- 50 modules: 4-6x speedup  
- 100 modules: 6-8x speedup

### 2. Type Check Speedup (`typecheck_speedup/*`)
Direct comparison of sequential vs parallel type checking for pre-parsed ASTs.

```bash
cargo bench --bench parallel_bench -- typecheck_speedup
```

**Expected Results:**
- 10 modules: 2-3x speedup
- 50 modules: 5-7x speedup
- 100 modules: 7-9x speedup

### 3. Codegen Speedup (`codegen_speedup/*`)
Direct comparison of sequential vs parallel IR generation for type-checked ASTs.

```bash
cargo bench --bench parallel_bench -- codegen_speedup
```

**Expected Results:**
- 10 modules: 3-4x speedup (highest single-stage speedup)
- 50 modules: 6-8x speedup
- 100 modules: 8-10x speedup

### 4. Full Pipeline (`sequential_full_pipeline/*` vs `parallel_full_pipeline/*`)
End-to-end pipeline: lex → parse → typecheck → codegen.

```bash
cargo bench --bench parallel_bench -- full_pipeline
```

**Expected Results:**
- 10 modules: 2-3x speedup (overhead dominates)
- 50 modules: 4-6x speedup
- 100 modules: 6-8x speedup

### 5. Individual Stage Benchmarks

#### Sequential Parse (`sequential_parse/*`)
```bash
cargo bench --bench parallel_bench -- sequential_parse
```

#### Parallel Parse (`parallel_parse/*`)
```bash
cargo bench --bench parallel_bench -- parallel_parse
```

#### Sequential Parse + Typecheck (`sequential_parse_typecheck/*`)
```bash
cargo bench --bench parallel_bench -- sequential_parse_typecheck
```

#### Parallel Parse + Typecheck (`parallel_parse_typecheck/*`)
```bash
cargo bench --bench parallel_bench -- parallel_parse_typecheck
```

## Test Scenarios

| Scenario | Modules | Lines/Module | Total Lines | Total Bytes |
|----------|---------|--------------|-------------|-------------|
| Small    | 10      | 500          | 5,000       | ~57 KiB     |
| Medium   | 50      | 500          | 25,000      | ~286 KiB    |
| Large    | 100     | 500          | 50,000      | ~572 KiB    |

## Running Benchmarks

### Quick Test (all benchmarks, reduced samples)
```bash
cargo bench --bench parallel_bench -- --quick
```

### Full Benchmark Suite (20+ minutes)
```bash
cargo bench --bench parallel_bench
```

### Compare Specific Scenarios
```bash
# Only 100-module scenarios (highest speedup)
cargo bench --bench parallel_bench -- 100_modules

# Only parse stage
cargo bench --bench parallel_bench -- parse

# Only speedup comparisons
cargo bench --bench parallel_bench -- speedup
```

### Save Baseline for Future Comparison
```bash
# Before Phase 2
cargo bench --bench parallel_bench -- --save-baseline before-phase2

# After Phase 2 optimizations
cargo bench --bench parallel_bench -- --save-baseline after-phase2

# Compare
cargo bench --bench parallel_bench -- --baseline before-phase2
```

## Output Interpretation

### Console Output
```
parse_speedup/sequential/10_modules
                        time:   [1.6061 ms 1.6073 ms 1.6123 ms]
                        thrpt:  [35.436 MiB/s 35.545 MiB/s 35.572 MiB/s]

parse_speedup/parallel/10_modules
                        time:   [731.76 µs 735.64 µs 751.13 µs]
                        thrpt:  [76.062 MiB/s 77.664 MiB/s 78.075 MiB/s]
```

**Speedup Calculation:**
```
Speedup = Sequential Time / Parallel Time
        = 1.607 ms / 0.736 ms
        = 2.18x
```

### HTML Reports
After running benchmarks, open:
```
target/criterion/report/index.html
```

This provides:
- Interactive charts
- Throughput graphs  
- Statistical distributions
- Outlier analysis
- Historical comparison (if baselines exist)

## Performance Characteristics

### Scaling by Module Count

| Modules | Parse Speedup | Typecheck Speedup | Codegen Speedup | Full Pipeline |
|---------|---------------|-------------------|-----------------|---------------|
| 10      | 2.2x          | 2.5x              | 4.1x            | 2.8x          |
| 50      | 5.1x          | 6.2x              | 7.3x            | 5.7x          |
| 100     | 7.2x          | 8.1x              | 9.4x            | 7.8x          |

*(Expected values on 8-core system)*

### Why Different Speedups?

1. **Parse** (moderate speedup): I/O-bound, text processing overhead
2. **Typecheck** (high speedup): CPU-bound, minimal cross-module dependencies
3. **Codegen** (highest speedup): CPU-intensive LLVM IR generation, fully independent
4. **Full Pipeline** (moderate): Overhead from inter-stage coordination

## Troubleshooting

### Low Speedup (<2x)
- Check CPU core count: `sysctl -n hw.ncpu` (macOS) or `nproc` (Linux)
- Verify rayon thread pool: benchmarks use all available cores by default
- System load: close other CPU-intensive applications

### Variance in Results
- Run with `--sample-size 100` for more stable results
- Disable CPU frequency scaling (performance mode)
- Close background applications

### Out of Memory
- Reduce module count in `generate_multi_module_project()` calls
- Use `--sample-size 10` for large scenarios (100 modules)

## Architecture Notes

### Parallel Strategy (Phase 2 Stage 2)

1. **Parallel Parse**: `rayon::par_iter()` on module sources
   - Each thread gets independent `.vais` file
   - No shared state during parsing
   - Speedup limited by I/O and memory bandwidth

2. **Parallel Typecheck**: `rayon::par_iter()` on parsed ASTs
   - Each module type-checked independently
   - Cross-module references resolved in second pass
   - Good speedup for independent modules

3. **Parallel Codegen**: `rayon::par_iter()` on type-checked ASTs
   - LLVM IR generation per module
   - Fully independent (highest speedup)
   - Memory-intensive (each thread has CodeGenerator)

4. **Pipeline Parallelism**: Overlapping stages
   - Parse module N while type-checking module N-1
   - Limited by data dependencies
   - Future optimization opportunity

## Related Files

- **Implementation**: `crates/vais-codegen/src/parallel.rs`
- **CLI Integration**: `crates/vaisc/src/commands/compile.rs`
- **Helper Functions**: `benches/lib.rs` (`generate_multi_module_project`)
- **Phase 2 Roadmap**: `ROADMAP.md` (Phase 2 Stage 2)

## Contributing

When adding new parallel features:

1. Add corresponding benchmarks to this suite
2. Update expected speedup values in this README
3. Run baseline comparison to verify improvement
4. Document any new parallelization strategies

---

**Last Updated**: Phase 2 Stage 2 (2026-02-09)
**Benchmark Count**: 30 individual benchmarks
**Total Runtime**: ~20 minutes (full suite), ~5 minutes (--quick)

# Vais vs C Performance Benchmark

This benchmark compares the performance of Vais against C for common algorithms.

## Benchmarks

1. **Fibonacci(35)** - Recursive fibonacci to test function call overhead
2. **Sum 1 to 100,000** - Iterative summation (tail recursive in Vais, loop in C)
3. **Count primes up to 5,000** - Prime counting using trial division

## Files

- `benchmark.vais` - Vais implementation
- `benchmark.c` - C implementation (compiled with `-O2`)
- `run_bench.sh` - Shell script to compile and run both benchmarks

## Usage

```bash
# Run the complete benchmark suite
./run_bench.sh

# Or compile and run manually:

# Vais version
cargo run --bin vaisc -- examples/projects/benchmark/benchmark.vais
./examples/projects/benchmark/benchmark

# C version
clang -O2 -o examples/projects/benchmark/benchmark_c examples/projects/benchmark/benchmark.c
./examples/projects/benchmark/benchmark_c
```

## Expected Results

Both implementations should produce identical results:
- Fibonacci(35) = 9227465
- Sum 1 to 100,000 = 5000050000
- Count primes up to 5,000 = 669

## Performance Notes

The benchmark compares:
- Vais compiled to LLVM IR, then to native binary via clang
- C compiled directly with clang -O2

Vais demonstrates competitive performance with C, especially considering that:
- Vais uses single-character keywords for AI optimization
- The compiler implements full type inference
- It's a young language at Phase 22 Stage 3 (production deployment readiness)

## Limitations

Note that the summation benchmark uses tail recursion in Vais (since Vais doesn't have traditional loop constructs), while C uses a standard for-loop. The benchmark size is limited to 100,000 to avoid stack overflow in the recursive implementation.

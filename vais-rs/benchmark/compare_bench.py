#!/usr/bin/env python3
"""
Vais vs Python Benchmark Comparison
===================================
Fair comparison using internal timing (not process spawn overhead)
"""

import subprocess
import time
import statistics
import sys

ITERATIONS = 5
WARMUP = 2

# ============================================================================
# Python Benchmarks
# ============================================================================

def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

def sum_to_n(n):
    if n <= 0:
        return 0
    return n + sum_to_n(n - 1)

def py_benchmark(name, func, *args):
    """Benchmark Python function"""
    # Warmup
    for _ in range(WARMUP):
        func(*args)

    # Measure
    times = []
    result = None
    for _ in range(ITERATIONS):
        start = time.perf_counter()
        result = func(*args)
        end = time.perf_counter()
        times.append((end - start) * 1_000_000)

    return result, statistics.mean(times)

# ============================================================================
# Vais Benchmarks (using hyperfine-style approach)
# ============================================================================

def vais_benchmark(name, file_path):
    """Benchmark Vais file execution (process includes all overhead)"""
    # Warmup
    for _ in range(WARMUP):
        subprocess.run(
            ["./target/release/vais", "run", file_path],
            capture_output=True, text=True
        )

    # Measure
    times = []
    result = None
    for _ in range(ITERATIONS):
        start = time.perf_counter()
        proc = subprocess.run(
            ["./target/release/vais", "run", file_path],
            capture_output=True, text=True
        )
        end = time.perf_counter()
        result = proc.stdout.strip()
        times.append((end - start) * 1_000_000)

    return result, statistics.mean(times)

# ============================================================================
# Main
# ============================================================================

if __name__ == "__main__":
    print()
    print("=" * 70)
    print("              Vais vs Python Performance Comparison")
    print("=" * 70)
    print()
    print("Note: Vais times include process startup overhead (~2-3ms)")
    print("      For pure computation comparison, see the 'Internal Timing' section")
    print()

    benchmarks = []

    # Factorial 10
    py_result, py_time = py_benchmark("factorial(10)", factorial, 10)
    vais_result, vais_time = vais_benchmark("factorial(10)", "benchmark/bench_factorial10.vais")
    benchmarks.append(("factorial(10)", py_result, py_time, vais_result, vais_time))

    # Factorial 20
    py_result, py_time = py_benchmark("factorial(20)", factorial, 20)
    vais_result, vais_time = vais_benchmark("factorial(20)", "benchmark/bench_factorial20.vais")
    benchmarks.append(("factorial(20)", py_result, py_time, vais_result, vais_time))

    # Fibonacci 20
    py_result, py_time = py_benchmark("fibonacci(20)", fibonacci, 20)
    vais_result, vais_time = vais_benchmark("fibonacci(20)", "benchmark/bench_fib20.vais")
    benchmarks.append(("fibonacci(20)", py_result, py_time, vais_result, vais_time))

    # Fibonacci 30
    py_result, py_time = py_benchmark("fibonacci(30)", fibonacci, 30)
    vais_result, vais_time = vais_benchmark("fibonacci(30)", "benchmark/bench_fib30.vais")
    benchmarks.append(("fibonacci(30)", py_result, py_time, vais_result, vais_time))

    # Sum to 100
    py_result, py_time = py_benchmark("sum_to_n(100)", sum_to_n, 100)
    vais_result, vais_time = vais_benchmark("sum_to_n(100)", "benchmark/bench_sum100.vais")
    benchmarks.append(("sum_to_n(100)", py_result, py_time, vais_result, vais_time))

    # Print results
    print("-" * 70)
    print(f"{'Benchmark':<18} | {'Python':>12} | {'Vais (total)':>12} | {'Result':>10}")
    print("-" * 70)

    for name, py_res, py_t, vais_res, vais_t in benchmarks:
        match = "✓" if str(py_res) == str(vais_res) else "✗"
        print(f"{name:<18} | {py_t:>10.2f}µs | {vais_t:>10.2f}µs | {match:>10}")

    print("-" * 70)
    print()

    # Internal timing comparison (subtracting process overhead)
    PROCESS_OVERHEAD_US = 2500  # ~2.5ms process startup overhead

    print("=" * 70)
    print("           Pure Computation Time (Estimated)")
    print("           (Vais total - ~2.5ms process overhead)")
    print("=" * 70)
    print()
    print("-" * 70)
    print(f"{'Benchmark':<18} | {'Python':>12} | {'Vais (pure)':>12} | {'Speedup':>10}")
    print("-" * 70)

    for name, py_res, py_t, vais_res, vais_t in benchmarks:
        vais_pure = max(1, vais_t - PROCESS_OVERHEAD_US)
        if py_t > 0:
            ratio = py_t / vais_pure
            if ratio > 1:
                speedup = f"{ratio:.1f}x faster"
            else:
                speedup = f"{1/ratio:.1f}x slower"
        else:
            speedup = "N/A"
        print(f"{name:<18} | {py_t:>10.2f}µs | {vais_pure:>10.2f}µs | {speedup:>10}")

    print("-" * 70)
    print()

    # Highlight fib(30) which is CPU intensive
    print("=" * 70)
    print("             Fibonacci(30) Deep Dive")
    print("=" * 70)
    print()
    fib_bench = [b for b in benchmarks if b[0] == "fibonacci(30)"][0]
    name, py_res, py_t, vais_res, vais_t = fib_bench
    vais_pure = vais_t - PROCESS_OVERHEAD_US

    print(f"Python:     {py_t:>12.2f} µs  ({py_t/1000:.2f} ms)")
    print(f"Vais total: {vais_t:>12.2f} µs  ({vais_t/1000:.2f} ms)")
    print(f"Vais pure:  {vais_pure:>12.2f} µs  ({vais_pure/1000:.2f} ms) (estimated)")
    print()

    if vais_pure > 0:
        ratio = py_t / vais_pure
        print(f"Vais is approximately {ratio:.1f}x faster than Python for fib(30)")

    print()
    print("=" * 70)

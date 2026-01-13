#!/usr/bin/env python3
"""
Vais vs Python Full Benchmark Comparison
========================================
"""

import subprocess
import time
import statistics
import sys
from functools import lru_cache

print()
print("=" * 70)
print("           VAIS vs PYTHON PERFORMANCE BENCHMARK")
print("=" * 70)
print()

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

def map_double(arr):
    return [x * 2 for x in arr]

def filter_even(arr):
    return [x for x in arr if x % 2 == 0]

def reduce_sum(arr):
    total = 0
    for x in arr:
        total += x
    return total

def chain_ops(arr):
    """filter > 2, double, sum"""
    return sum(x * 2 for x in arr if x > 2)

def benchmark_py(name, func, *args, iterations=100):
    """Benchmark Python function"""
    # Warmup
    for _ in range(10):
        func(*args)

    times = []
    result = None
    for _ in range(iterations):
        start = time.perf_counter()
        result = func(*args)
        end = time.perf_counter()
        times.append((end - start) * 1_000_000)

    return result, statistics.mean(times)

# ============================================================================
# Vais Benchmarks
# ============================================================================

def benchmark_vais(name, file_path, iterations=5):
    """Benchmark Vais file (includes process overhead)"""
    # Warmup
    for _ in range(3):
        subprocess.run(["./target/release/vais", "run", file_path], capture_output=True)

    times = []
    result = None
    for _ in range(iterations):
        start = time.perf_counter()
        proc = subprocess.run(["./target/release/vais", "run", file_path], capture_output=True, text=True)
        end = time.perf_counter()
        result = proc.stdout.strip()
        times.append((end - start) * 1_000_000)

    return result, statistics.mean(times)

# ============================================================================
# Run Benchmarks
# ============================================================================

# Estimate process overhead
print("Measuring Vais process startup overhead...")
overhead_times = []
for _ in range(10):
    start = time.perf_counter()
    subprocess.run(["./target/release/vais", "run", "benchmark/bench_chain.vais"], capture_output=True)
    end = time.perf_counter()
    overhead_times.append((end - start) * 1_000_000)

OVERHEAD = min(overhead_times)  # Minimum as baseline
print(f"Process overhead (min): {OVERHEAD:.0f} µs ({OVERHEAD/1000:.1f} ms)")
print()

results = []

# Factorial 10
py_res, py_time = benchmark_py("factorial(10)", factorial, 10, iterations=1000)
vais_res, vais_time = benchmark_vais("factorial(10)", "benchmark/bench_factorial10.vais")
results.append(("factorial(10)", py_res, py_time, vais_res, vais_time))

# Factorial 20
py_res, py_time = benchmark_py("factorial(20)", factorial, 20, iterations=1000)
vais_res, vais_time = benchmark_vais("factorial(20)", "benchmark/bench_factorial20.vais")
results.append(("factorial(20)", py_res, py_time, vais_res, vais_time))

# Fibonacci 20
py_res, py_time = benchmark_py("fibonacci(20)", fibonacci, 20, iterations=100)
vais_res, vais_time = benchmark_vais("fibonacci(20)", "benchmark/bench_fib20.vais")
results.append(("fibonacci(20)", py_res, py_time, vais_res, vais_time))

# Fibonacci 30 (CPU intensive)
print("Running fibonacci(30)... this may take a moment")
py_res, py_time = benchmark_py("fibonacci(30)", fibonacci, 30, iterations=3)
vais_res, vais_time = benchmark_vais("fibonacci(30)", "benchmark/bench_fib30.vais", iterations=3)
results.append(("fibonacci(30)", py_res, py_time, vais_res, vais_time))

# Sum to 100
py_res, py_time = benchmark_py("sum_to_n(100)", sum_to_n, 100, iterations=1000)
vais_res, vais_time = benchmark_vais("sum_to_n(100)", "benchmark/bench_sum100.vais")
results.append(("sum_to_n(100)", py_res, py_time, vais_res, vais_time))

# Collection operations
arr_1000 = list(range(1, 1001))

py_res, py_time = benchmark_py("map 1000", map_double, arr_1000, iterations=1000)
vais_res, vais_time = benchmark_vais("map 1000", "benchmark/bench_map1000.vais")
results.append(("map (* 2) 1000", f"[sum={sum(py_res)}]", py_time, vais_res, vais_time))

py_res, py_time = benchmark_py("filter 1000", filter_even, arr_1000, iterations=1000)
vais_res, vais_time = benchmark_vais("filter 1000", "benchmark/bench_filter1000.vais")
results.append(("filter even 1000", f"[len={len(py_res)}]", py_time, vais_res, vais_time))

py_res, py_time = benchmark_py("reduce 1000", reduce_sum, arr_1000, iterations=1000)
vais_res, vais_time = benchmark_vais("reduce 1000", "benchmark/bench_reduce1000.vais")
results.append(("reduce sum 1000", py_res, py_time, vais_res, vais_time))

# Chain
py_res, py_time = benchmark_py("chain", chain_ops, [1,2,3,4,5], iterations=1000)
vais_res, vais_time = benchmark_vais("chain", "benchmark/bench_chain.vais")
results.append(("chain ops", py_res, py_time, vais_res, vais_time))

# ============================================================================
# Print Results
# ============================================================================

print()
print("=" * 75)
print("                         BENCHMARK RESULTS")
print("=" * 75)
print()
print("-" * 75)
print(f"{'Benchmark':<20} | {'Python':>12} | {'Vais (total)':>14} | {'Vais (pure)*':>12} | {'Ratio':>8}")
print("-" * 75)

for name, py_res, py_time, vais_res, vais_time in results:
    vais_pure = max(1, vais_time - OVERHEAD)
    if py_time > 0:
        ratio = vais_pure / py_time
        ratio_str = f"{ratio:.1f}x"
    else:
        ratio_str = "N/A"
    print(f"{name:<20} | {py_time:>10.1f}µs | {vais_time:>12.1f}µs | {vais_pure:>10.1f}µs | {ratio_str:>8}")

print("-" * 75)
print()
print("* Vais (pure) = Vais (total) - process overhead")
print()

# Summary for fibonacci(30) - most meaningful comparison
print("=" * 75)
print("                    FIBONACCI(30) COMPARISON")
print("=" * 75)
print()

fib30 = [r for r in results if r[0] == "fibonacci(30)"][0]
name, py_res, py_time, vais_res, vais_time = fib30
vais_pure = vais_time - OVERHEAD

print(f"Python:              {py_time/1000:>10.2f} ms")
print(f"Vais (with startup): {vais_time/1000:>10.2f} ms")
print(f"Vais (pure compute): {vais_pure/1000:>10.2f} ms")
print()

ratio = vais_pure / py_time
print(f"Vais VM is {ratio:.1f}x {'slower' if ratio > 1 else 'faster'} than Python for recursive fibonacci")
print()
print("Note: Vais is an interpreted VM. For maximum performance,")
print("      use the JIT compiler feature (--features jit)")
print()
print("=" * 75)

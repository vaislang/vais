#!/usr/bin/env python3
"""
Vais vs Python Benchmark
========================
Performance comparison between Vais and Python
"""

import time
import statistics

# Number of iterations for timing
ITERATIONS = 5

def benchmark(name, func, *args):
    """Run a benchmark and return average time in microseconds"""
    times = []
    for _ in range(ITERATIONS):
        start = time.perf_counter()
        result = func(*args)
        end = time.perf_counter()
        times.append((end - start) * 1_000_000)  # Convert to microseconds

    avg = statistics.mean(times)
    return result, avg

# ============================================================================
# Benchmark Functions
# ============================================================================

def factorial(n):
    """Recursive factorial"""
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def fibonacci(n):
    """Recursive fibonacci (naive)"""
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

def sum_to_n(n):
    """Sum from 1 to n (recursive)"""
    if n <= 0:
        return 0
    return n + sum_to_n(n - 1)

def map_double(arr):
    """Map: double each element"""
    return [x * 2 for x in arr]

def filter_even(arr):
    """Filter: keep even numbers"""
    return [x for x in arr if x % 2 == 0]

def reduce_sum(arr):
    """Reduce: sum all elements"""
    total = 0
    for x in arr:
        total += x
    return total

def map_filter_reduce(arr):
    """Chain: filter > 2, double, sum"""
    filtered = [x for x in arr if x > 2]
    doubled = [x * 2 for x in filtered]
    return sum(doubled)

# ============================================================================
# Run Benchmarks
# ============================================================================

if __name__ == "__main__":
    print("=" * 60)
    print("Python Benchmark Results")
    print("=" * 60)
    print()

    # Recursive benchmarks
    print("--- Recursive Functions ---")

    result, time_us = benchmark("factorial(10)", factorial, 10)
    print(f"factorial(10)     = {result:>12}  |  {time_us:>10.2f} µs")

    result, time_us = benchmark("factorial(20)", factorial, 20)
    print(f"factorial(20)     = {result:>12}  |  {time_us:>10.2f} µs")

    result, time_us = benchmark("fibonacci(20)", fibonacci, 20)
    print(f"fibonacci(20)     = {result:>12}  |  {time_us:>10.2f} µs")

    result, time_us = benchmark("fibonacci(30)", fibonacci, 30)
    print(f"fibonacci(30)     = {result:>12}  |  {time_us:>10.2f} µs")

    result, time_us = benchmark("sum_to_n(100)", sum_to_n, 100)
    print(f"sum_to_n(100)     = {result:>12}  |  {time_us:>10.2f} µs")

    print()
    print("--- Collection Operations (1000 elements) ---")

    arr_1000 = list(range(1, 1001))

    result, time_us = benchmark("map_double", map_double, arr_1000)
    print(f"map (* 2)         = [len={len(result):>5}]  |  {time_us:>10.2f} µs")

    result, time_us = benchmark("filter_even", filter_even, arr_1000)
    print(f"filter (even)     = [len={len(result):>5}]  |  {time_us:>10.2f} µs")

    result, time_us = benchmark("reduce_sum", reduce_sum, arr_1000)
    print(f"reduce (sum)      = {result:>12}  |  {time_us:>10.2f} µs")

    print()
    print("--- Chained Operations ---")

    arr_small = list(range(1, 6))  # [1, 2, 3, 4, 5]
    result, time_us = benchmark("map_filter_reduce", map_filter_reduce, arr_small)
    print(f"filter>map>reduce = {result:>12}  |  {time_us:>10.2f} µs")

    print()
    print("=" * 60)

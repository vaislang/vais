#!/usr/bin/env python3
"""
Vais Benchmark Runner
=====================
Measures Vais execution times for comparison with Python
"""

import subprocess
import time
import statistics
import os

# Number of iterations
ITERATIONS = 5

def run_vais(code):
    """Run Vais code and measure execution time"""
    vais_path = os.path.join(os.path.dirname(__file__), "..", "target", "release", "vais")

    start = time.perf_counter()
    result = subprocess.run(
        [vais_path, "run", "-e", code],
        capture_output=True,
        text=True
    )
    end = time.perf_counter()

    return result.stdout.strip(), (end - start) * 1_000_000  # microseconds

def benchmark_vais(name, code):
    """Run Vais benchmark multiple times and return average"""
    times = []
    result = None

    for _ in range(ITERATIONS):
        result, elapsed = run_vais(code)
        times.append(elapsed)

    avg = statistics.mean(times)
    return result, avg

if __name__ == "__main__":
    print("=" * 60)
    print("Vais Benchmark Results")
    print("=" * 60)
    print()

    print("--- Recursive Functions ---")

    result, time_us = benchmark_vais(
        "factorial(10)",
        "factorial(n) = n <= 1 ? 1 : n * factorial(n - 1); factorial(10)"
    )
    print(f"factorial(10)     = {result:>12}  |  {time_us:>10.2f} µs")

    result, time_us = benchmark_vais(
        "factorial(20)",
        "factorial(n) = n <= 1 ? 1 : n * factorial(n - 1); factorial(20)"
    )
    print(f"factorial(20)     = {result:>12}  |  {time_us:>10.2f} µs")

    result, time_us = benchmark_vais(
        "fibonacci(20)",
        "fib(n) = n <= 1 ? n : fib(n - 1) + fib(n - 2); fib(20)"
    )
    print(f"fibonacci(20)     = {result:>12}  |  {time_us:>10.2f} µs")

    result, time_us = benchmark_vais(
        "fibonacci(30)",
        "fib(n) = n <= 1 ? n : fib(n - 1) + fib(n - 2); fib(30)"
    )
    print(f"fibonacci(30)     = {result:>12}  |  {time_us:>10.2f} µs")

    result, time_us = benchmark_vais(
        "sum_to_n(100)",
        "sum(n) = n <= 0 ? 0 : n + sum(n - 1); sum(100)"
    )
    print(f"sum_to_n(100)     = {result:>12}  |  {time_us:>10.2f} µs")

    print()
    print("--- Collection Operations (1000 elements) ---")

    # Map operation
    result, time_us = benchmark_vais(
        "map (* 2)",
        "(1..1001).@(_ * 2)./+"
    )
    print(f"map (* 2) sum     = {result:>12}  |  {time_us:>10.2f} µs")

    # Filter operation
    result, time_us = benchmark_vais(
        "filter (even)",
        "#((1..1001).?(_ % 2 == 0))"
    )
    print(f"filter (even) len = {result:>12}  |  {time_us:>10.2f} µs")

    # Reduce operation
    result, time_us = benchmark_vais(
        "reduce (sum)",
        "(1..1001)./+"
    )
    print(f"reduce (sum)      = {result:>12}  |  {time_us:>10.2f} µs")

    print()
    print("--- Chained Operations ---")

    result, time_us = benchmark_vais(
        "filter>map>sum",
        "[1, 2, 3, 4, 5].?(_ > 2).@(_ * 2)./+"
    )
    print(f"filter>map>reduce = {result:>12}  |  {time_us:>10.2f} µs")

    print()
    print("=" * 60)

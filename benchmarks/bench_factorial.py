#!/usr/bin/env python3
"""Factorial benchmark for Python vs Vais comparison"""
import time

def factorial_recursive(n):
    if n <= 1:
        return 1
    return n * factorial_recursive(n - 1)

def factorial_iterative(n):
    result = 1
    for i in range(2, n + 1):
        result *= i
    return result

def benchmark(name, func, arg, iterations=1000):
    """Run benchmark and return average time"""
    start = time.perf_counter()
    for _ in range(iterations):
        result = func(arg)
    end = time.perf_counter()
    avg_time = (end - start) / iterations
    return result, avg_time

if __name__ == "__main__":
    print("=" * 60)
    print("Python Factorial Benchmark")
    print("=" * 60)

    # Recursive Factorial
    print("\nRecursive:")
    for n in [10, 20, 50, 100]:
        result, elapsed = benchmark(f"factorial({n})", factorial_recursive, n)
        print(f"factorial({n:3d}) | Time: {elapsed*1000000:10.2f} us | Result digits: {len(str(result))}")

    print("\nIterative:")
    for n in [10, 20, 50, 100]:
        result, elapsed = benchmark(f"factorial({n})", factorial_iterative, n)
        print(f"factorial({n:3d}) | Time: {elapsed*1000000:10.2f} us | Result digits: {len(str(result))}")

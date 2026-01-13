#!/usr/bin/env python3
"""Fibonacci benchmark for Python vs Vais comparison"""
import time

def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def fib_iterative(n):
    if n <= 1:
        return n
    a, b = 0, 1
    for _ in range(2, n + 1):
        a, b = b, a + b
    return b

def benchmark(name, func, arg, iterations=1):
    """Run benchmark and return average time"""
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        result = func(arg)
        end = time.perf_counter()
        times.append(end - start)
    avg_time = sum(times) / len(times)
    return result, avg_time

if __name__ == "__main__":
    print("=" * 60)
    print("Python Fibonacci Benchmark")
    print("=" * 60)

    # Recursive Fibonacci
    for n in [20, 25, 30, 35]:
        result, elapsed = benchmark(f"fib({n})", fib, n)
        print(f"fib({n:2d}) = {result:12d} | Time: {elapsed*1000:10.2f} ms")

    print()
    print("Iterative (for reference):")
    for n in [20, 25, 30, 35, 40]:
        result, elapsed = benchmark(f"fib_iter({n})", fib_iterative, n)
        print(f"fib({n:2d}) = {result:12d} | Time: {elapsed*1000:10.4f} ms")

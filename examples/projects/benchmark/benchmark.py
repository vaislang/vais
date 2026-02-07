# Performance Benchmark Suite - Python version for comparison
# Implements the same algorithms as the Vais/C/Rust versions:
# 1. Fibonacci (recursive)
# 2. Sum 1 to N (iterative)
# 3. Prime counting

import sys
sys.setrecursionlimit(200000)

def fib(n):
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)

def sum_to_n(n):
    s = 0
    for i in range(1, n + 1):
        s += i
    return s

def is_prime(n):
    if n < 2:
        return False
    if n < 4:
        return True
    if n % 2 == 0:
        return False
    d = 3
    while d * d <= n:
        if n % d == 0:
            return False
        d += 2
    return True

def count_primes(n):
    count = 0
    for i in range(2, n + 1):
        if is_prime(i):
            count += 1
    return count

if __name__ == "__main__":
    print("=== Python Performance Benchmark Suite ===")
    print()

    print("Benchmark 1: Fibonacci(35)")
    result1 = fib(35)
    print(f"Result: {result1}")
    print()

    print("Benchmark 2: Sum 1 to 100,000")
    result2 = sum_to_n(100000)
    print(f"Result: {result2}")
    print()

    print("Benchmark 3: Count primes up to 5,000")
    result3 = count_primes(5000)
    print(f"Result: {result3}")
    print()

    print("All benchmarks completed!")

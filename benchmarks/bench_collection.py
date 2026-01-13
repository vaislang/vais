#!/usr/bin/env python3
"""Collection operations benchmark for Python vs Vais comparison"""
import time
from functools import reduce

def benchmark(name, func, iterations=1000):
    """Run benchmark and return average time"""
    start = time.perf_counter()
    for _ in range(iterations):
        result = func()
    end = time.perf_counter()
    avg_time = (end - start) / iterations
    return result, avg_time

# Test data
small_arr = list(range(1, 101))      # 100 elements
medium_arr = list(range(1, 1001))    # 1000 elements
large_arr = list(range(1, 10001))    # 10000 elements

def map_double_small():
    return [x * 2 for x in small_arr]

def map_double_medium():
    return [x * 2 for x in medium_arr]

def map_double_large():
    return [x * 2 for x in large_arr]

def filter_even_small():
    return [x for x in small_arr if x % 2 == 0]

def filter_even_medium():
    return [x for x in medium_arr if x % 2 == 0]

def filter_even_large():
    return [x for x in large_arr if x % 2 == 0]

def reduce_sum_small():
    return sum(small_arr)

def reduce_sum_medium():
    return sum(medium_arr)

def reduce_sum_large():
    return sum(large_arr)

def chain_small():
    """Filter even, double, sum"""
    return sum(x * 2 for x in small_arr if x % 2 == 0)

def chain_medium():
    return sum(x * 2 for x in medium_arr if x % 2 == 0)

def chain_large():
    return sum(x * 2 for x in large_arr if x % 2 == 0)

if __name__ == "__main__":
    print("=" * 70)
    print("Python Collection Operations Benchmark")
    print("=" * 70)

    print("\n--- Map (double each element) ---")
    result, elapsed = benchmark("map_small", map_double_small, 10000)
    print(f"100 elements   | Time: {elapsed*1000000:10.2f} us | Result len: {len(result)}")
    result, elapsed = benchmark("map_medium", map_double_medium, 1000)
    print(f"1000 elements  | Time: {elapsed*1000000:10.2f} us | Result len: {len(result)}")
    result, elapsed = benchmark("map_large", map_double_large, 100)
    print(f"10000 elements | Time: {elapsed*1000000:10.2f} us | Result len: {len(result)}")

    print("\n--- Filter (keep even numbers) ---")
    result, elapsed = benchmark("filter_small", filter_even_small, 10000)
    print(f"100 elements   | Time: {elapsed*1000000:10.2f} us | Result len: {len(result)}")
    result, elapsed = benchmark("filter_medium", filter_even_medium, 1000)
    print(f"1000 elements  | Time: {elapsed*1000000:10.2f} us | Result len: {len(result)}")
    result, elapsed = benchmark("filter_large", filter_even_large, 100)
    print(f"10000 elements | Time: {elapsed*1000000:10.2f} us | Result len: {len(result)}")

    print("\n--- Reduce (sum) ---")
    result, elapsed = benchmark("reduce_small", reduce_sum_small, 10000)
    print(f"100 elements   | Time: {elapsed*1000000:10.2f} us | Result: {result}")
    result, elapsed = benchmark("reduce_medium", reduce_sum_medium, 1000)
    print(f"1000 elements  | Time: {elapsed*1000000:10.2f} us | Result: {result}")
    result, elapsed = benchmark("reduce_large", reduce_sum_large, 100)
    print(f"10000 elements | Time: {elapsed*1000000:10.2f} us | Result: {result}")

    print("\n--- Chain (filter even -> double -> sum) ---")
    result, elapsed = benchmark("chain_small", chain_small, 10000)
    print(f"100 elements   | Time: {elapsed*1000000:10.2f} us | Result: {result}")
    result, elapsed = benchmark("chain_medium", chain_medium, 1000)
    print(f"1000 elements  | Time: {elapsed*1000000:10.2f} us | Result: {result}")
    result, elapsed = benchmark("chain_large", chain_large, 100)
    print(f"10000 elements | Time: {elapsed*1000000:10.2f} us | Result: {result}")

/*
 * Performance Benchmark Suite - C version for comparison
 * Implements the same algorithms as the Vais version:
 * 1. Fibonacci (recursive)
 * 2. Sum 1 to N (iterative)
 * 3. Prime counting
 */

#include <stdio.h>
#include <stdint.h>
#include <inttypes.h>

/* Algorithm 1: Fibonacci (Recursive) */
int64_t fib(int64_t n) {
    if (n < 2) return n;
    return fib(n - 1) + fib(n - 2);
}

/* Algorithm 2: Sum 1 to N (Iterative with Loop) */
int64_t sum_to_n(int64_t n) {
    int64_t sum = 0;
    for (int64_t i = 1; i <= n; i++) {
        sum += i;
    }
    return sum;
}

/* Algorithm 3: Prime Counting */
int is_prime(int64_t n) {
    if (n < 2) return 0;
    if (n < 4) return 1;
    if (n % 2 == 0) return 0;

    for (int64_t d = 3; d * d <= n; d += 2) {
        if (n % d == 0) return 0;
    }
    return 1;
}

int64_t count_primes(int64_t n) {
    int64_t count = 0;
    for (int64_t i = 2; i <= n; i++) {
        count += is_prime(i);
    }
    return count;
}

/* Main Benchmark Runner */
int main(void) {
    printf("=== C Performance Benchmark Suite ===\n");
    printf("\n");

    /* Benchmark 1: Fibonacci(35) */
    printf("Benchmark 1: Fibonacci(35)\n");
    int64_t result1 = fib(35);
    printf("Result: %" PRId64 "\n", result1);
    printf("\n");

    /* Benchmark 2: Sum 1 to 100,000 */
    printf("Benchmark 2: Sum 1 to 100,000\n");
    int64_t result2 = sum_to_n(100000);
    printf("Result: %" PRId64 "\n", result2);
    printf("\n");

    /* Benchmark 3: Count primes up to 5,000 */
    printf("Benchmark 3: Count primes up to 5,000\n");
    int64_t result3 = count_primes(5000);
    printf("Result: %" PRId64 "\n", result3);
    printf("\n");

    printf("All benchmarks completed!\n");
    return 0;
}

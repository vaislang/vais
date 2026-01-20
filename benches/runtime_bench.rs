//! Runtime benchmarks for generated Vais code
//!
//! Compares execution performance of generated code against native Rust.
//! Note: Vais binary benchmarks are prepared for future comparison.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Native Rust implementations for comparison

fn rust_fibonacci(n: i64) -> i64 {
    if n <= 1 {
        n
    } else {
        rust_fibonacci(n - 1) + rust_fibonacci(n - 2)
    }
}

fn rust_fibonacci_iter(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }
    let (mut a, mut b) = (0i64, 1i64);
    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    b
}

fn rust_is_prime(n: i64) -> bool {
    if n <= 1 {
        return false;
    }
    if n <= 3 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut i = 3;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 2;
    }
    true
}

fn rust_count_primes(limit: i64) -> i64 {
    (2..=limit).filter(|&n| rust_is_prime(n)).count() as i64
}

fn rust_factorial(n: i64) -> i64 {
    if n <= 1 {
        1
    } else {
        n * rust_factorial(n - 1)
    }
}

fn rust_gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a
    } else {
        rust_gcd(b, a % b)
    }
}

/// Benchmark: Fibonacci comparison
fn bench_fibonacci_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");

    // Rust implementation
    group.bench_function("rust_recursive", |b| {
        b.iter(|| rust_fibonacci(black_box(20)))
    });

    group.bench_function("rust_iterative", |b| {
        b.iter(|| rust_fibonacci_iter(black_box(50)))
    });

    group.finish();
}

/// Benchmark: Prime counting comparison
fn bench_prime_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("primes");

    group.bench_function("rust_count_100", |b| {
        b.iter(|| rust_count_primes(black_box(100)))
    });

    group.bench_function("rust_count_1000", |b| {
        b.iter(|| rust_count_primes(black_box(1000)))
    });

    group.finish();
}

/// Benchmark: Factorial comparison
fn bench_factorial_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("factorial");

    for n in [10, 15, 20] {
        group.bench_with_input(BenchmarkId::new("rust", n), &n, |b, &n| {
            b.iter(|| rust_factorial(black_box(n)))
        });
    }

    group.finish();
}

/// Benchmark: GCD comparison
fn bench_gcd_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("gcd");

    let test_cases = [(48, 18), (12345, 67890), (1000000007, 998244353)];

    for (a, b) in test_cases {
        group.bench_with_input(BenchmarkId::new("rust", format!("{}_{}", a, b)), &(a, b), |bench, &(a, b)| {
            bench.iter(|| rust_gcd(black_box(a), black_box(b)))
        });
    }

    group.finish();
}

/// Benchmark: Algorithm comparison (Vais vs Rust)
fn bench_algorithm_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("algorithms");

    // Test various algorithm sizes
    let sizes = [10, 20, 30];

    for size in sizes {
        group.bench_with_input(
            BenchmarkId::new("fib_rust", size),
            &size,
            |b, &n| b.iter(|| rust_fibonacci(black_box(n))),
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_fibonacci_comparison,
    bench_prime_comparison,
    bench_factorial_comparison,
    bench_gcd_comparison,
    bench_algorithm_comparison,
);

criterion_main!(benches);

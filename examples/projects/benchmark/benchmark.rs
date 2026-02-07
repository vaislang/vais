// Performance Benchmark Suite - Rust version for comparison
// Implements the same algorithms as the Vais/C versions:
// 1. Fibonacci (recursive)
// 2. Sum 1 to N (iterative)
// 3. Prime counting

fn fib(n: i64) -> i64 {
    if n < 2 { return n; }
    fib(n - 1) + fib(n - 2)
}

fn sum_to_n(n: i64) -> i64 {
    let mut sum: i64 = 0;
    for i in 1..=n {
        sum += i;
    }
    sum
}

fn is_prime(n: i64) -> bool {
    if n < 2 { return false; }
    if n < 4 { return true; }
    if n % 2 == 0 { return false; }
    let mut d: i64 = 3;
    while d * d <= n {
        if n % d == 0 { return false; }
        d += 2;
    }
    true
}

fn count_primes(n: i64) -> i64 {
    let mut count: i64 = 0;
    for i in 2..=n {
        if is_prime(i) { count += 1; }
    }
    count
}

fn main() {
    println!("=== Rust Performance Benchmark Suite ===");
    println!();

    println!("Benchmark 1: Fibonacci(35)");
    let result1 = fib(35);
    println!("Result: {}", result1);
    println!();

    println!("Benchmark 2: Sum 1 to 100,000");
    let result2 = sum_to_n(100000);
    println!("Result: {}", result2);
    println!();

    println!("Benchmark 3: Count primes up to 5,000");
    let result3 = count_primes(5000);
    println!("Result: {}", result3);
    println!();

    println!("All benchmarks completed!");
}

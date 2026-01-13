//! FastVM Benchmark - NaN-boxing performance test
//!
//! Run with: cargo run --release --example fast_bench

use std::time::Instant;
use vais_ir::{Value, NanBoxedValue};
use vais_lowering::Lowerer;
use vais_vm::{execute_function, execute_fast};

fn benchmark_standard(name: &str, source: &str, func: &str, args: Vec<Value>, iterations: u32) -> f64 {
    let program = vais_parser::parse(source).expect("Parse failed");
    let mut lowerer = Lowerer::new();
    let functions = lowerer.lower_program(&program).expect("Lowering failed");

    // Warmup
    for _ in 0..3 {
        let _ = execute_function(functions.clone(), func, args.clone());
    }

    // Measure
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = execute_function(functions.clone(), func, args.clone());
    }
    let elapsed = start.elapsed();

    let avg_us = elapsed.as_micros() as f64 / iterations as f64;
    println!("{:<20} (Standard VM): {:>12.2} µs", name, avg_us);
    avg_us
}

fn benchmark_fast(name: &str, source: &str, func: &str, args: Vec<Value>, iterations: u32) -> f64 {
    let program = vais_parser::parse(source).expect("Parse failed");
    let mut lowerer = Lowerer::new();
    let functions = lowerer.lower_program(&program).expect("Lowering failed");

    // Warmup
    for _ in 0..3 {
        let _ = execute_fast(functions.clone(), func, args.clone());
    }

    // Measure
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = execute_fast(functions.clone(), func, args.clone());
    }
    let elapsed = start.elapsed();

    let avg_us = elapsed.as_micros() as f64 / iterations as f64;
    println!("{:<20} (FastVM):      {:>12.2} µs", name, avg_us);
    avg_us
}

fn main() {
    println!("=".repeat(70));
    println!("            FastVM (NaN-boxing) vs Standard VM Benchmark");
    println!("=".repeat(70));
    println!();

    let benchmarks = vec![
        ("factorial(10)", "factorial(n) = n <= 1 ? 1 : n * factorial(n - 1)", "factorial", vec![Value::Int(10)], 1000),
        ("factorial(20)", "factorial(n) = n <= 1 ? 1 : n * factorial(n - 1)", "factorial", vec![Value::Int(20)], 1000),
        ("fibonacci(20)", "fib(n) = n <= 1 ? n : fib(n - 1) + fib(n - 2)", "fib", vec![Value::Int(20)], 100),
        ("fibonacci(25)", "fib(n) = n <= 1 ? n : fib(n - 1) + fib(n - 2)", "fib", vec![Value::Int(25)], 10),
        ("sum_to(100)", "sum_to(n) = n <= 0 ? 0 : n + sum_to(n - 1)", "sum_to", vec![Value::Int(100)], 1000),
    ];

    println!("-".repeat(70));
    println!("{:<20} | {:>15} | {:>15} | {:>10}", "Benchmark", "Standard VM", "FastVM", "Speedup");
    println!("-".repeat(70));

    for (name, source, func, args, iterations) in benchmarks {
        let std_time = benchmark_standard(name, source, func, args.clone(), iterations);
        let fast_time = benchmark_fast(name, source, func, args, iterations);
        let speedup = std_time / fast_time;
        println!("{:<20} | {:>12.2} µs | {:>12.2} µs | {:>9.2}x", name, std_time, fast_time, speedup);
        println!();
    }

    println!("-".repeat(70));
    println!();

    // Fibonacci(30) comparison
    println!("=".repeat(70));
    println!("                    FIBONACCI(30) DETAILED COMPARISON");
    println!("=".repeat(70));
    println!();

    let source = "fib(n) = n <= 1 ? n : fib(n - 1) + fib(n - 2)";
    let args = vec![Value::Int(30)];

    println!("Running fibonacci(30) with Standard VM...");
    let start = Instant::now();
    let program = vais_parser::parse(source).expect("Parse failed");
    let mut lowerer = Lowerer::new();
    let functions = lowerer.lower_program(&program).expect("Lowering failed");
    let result = execute_function(functions.clone(), "fib", args.clone());
    let std_time = start.elapsed();
    println!("Result: {:?}", result);
    println!("Time: {:.2} ms", std_time.as_millis());
    println!();

    println!("Running fibonacci(30) with FastVM...");
    let start = Instant::now();
    let result = execute_fast(functions, "fib", args);
    let fast_time = start.elapsed();
    println!("Result: {:?}", result);
    println!("Time: {:.2} ms", fast_time.as_millis());
    println!();

    let speedup = std_time.as_micros() as f64 / fast_time.as_micros() as f64;
    println!("FastVM is {:.2}x {} than Standard VM",
        if speedup > 1.0 { speedup } else { 1.0 / speedup },
        if speedup > 1.0 { "faster" } else { "slower" }
    );
    println!();
    println!("=".repeat(70));
}

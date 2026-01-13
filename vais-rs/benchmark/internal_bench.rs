//! Internal Vais Benchmark
//!
//! Measures pure VM execution time without process startup overhead

use std::time::Instant;
use vais_ir::Value;
use vais_lowering::Lowerer;
use vais_vm::execute_function;

fn benchmark(name: &str, source: &str, func: &str, args: Vec<Value>, iterations: u32) {
    let program = vais_parser::parse(source).expect("Parse failed");
    let mut lowerer = Lowerer::new();
    let functions = lowerer.lower_program(&program).expect("Lowering failed");

    // Warmup
    for _ in 0..5 {
        let _ = execute_function(functions.clone(), func, args.clone());
    }

    // Measure
    let start = Instant::now();
    let mut result = Value::Void;
    for _ in 0..iterations {
        result = execute_function(functions.clone(), func, args.clone()).expect("Execution failed");
    }
    let elapsed = start.elapsed();

    let avg_us = elapsed.as_micros() as f64 / iterations as f64;
    println!("{:<20} = {:>15}  |  {:>12.2} µs", name, format!("{:?}", result), avg_us);
}

fn main() {
    println!("=".repeat(60));
    println!("Vais Internal Benchmark (Pure VM Execution)");
    println!("=".repeat(60));
    println!();

    println!("--- Recursive Functions ---");

    benchmark(
        "factorial(10)",
        "factorial(n) = n <= 1 ? 1 : n * factorial(n - 1)",
        "factorial",
        vec![Value::Int(10)],
        1000,
    );

    benchmark(
        "factorial(20)",
        "factorial(n) = n <= 1 ? 1 : n * factorial(n - 1)",
        "factorial",
        vec![Value::Int(20)],
        1000,
    );

    benchmark(
        "fibonacci(20)",
        "fib(n) = n <= 1 ? n : fib(n - 1) + fib(n - 2)",
        "fib",
        vec![Value::Int(20)],
        100,
    );

    benchmark(
        "fibonacci(30)",
        "fib(n) = n <= 1 ? n : fib(n - 1) + fib(n - 2)",
        "fib",
        vec![Value::Int(30)],
        5,
    );

    benchmark(
        "sum_to_n(100)",
        "sum_to(n) = n <= 0 ? 0 : n + sum_to(n - 1)",
        "sum_to",
        vec![Value::Int(100)],
        1000,
    );

    println!();
    println!("=".repeat(60));
}

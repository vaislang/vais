//! FastVM Benchmark - NaN-boxing performance test
//!
//! Run with: cargo run --release --example fast_bench

use std::time::Instant;
use vais_ir::Value;
use vais_lowering::Lowerer;
use vais_vm::{execute_function, execute_fast};

fn main() {
    println!("{}", "=".repeat(70));
    println!("            FastVM (NaN-boxing) vs Standard VM Benchmark");
    println!("{}", "=".repeat(70));
    println!();

    // Fibonacci(30) comparison
    println!("Running fibonacci(30) comparison...");
    println!();

    let source = "fib(n) = n <= 1 ? n : fib(n - 1) + fib(n - 2)";
    let args = vec![Value::Int(30)];

    let program = vais_parser::parse(source).expect("Parse failed");
    let mut lowerer = Lowerer::new();
    let functions = lowerer.lower_program(&program).expect("Lowering failed");

    // Standard VM
    println!("Standard VM:");
    let start = Instant::now();
    let result = execute_function(functions.clone(), "fib", args.clone());
    let std_time = start.elapsed();
    println!("  Result: {:?}", result);
    println!("  Time: {:.2} ms", std_time.as_secs_f64() * 1000.0);
    println!();

    // FastVM
    println!("FastVM (NaN-boxing):");
    let start = Instant::now();
    let result = execute_fast(functions.clone(), "fib", args.clone());
    let fast_time = start.elapsed();
    println!("  Result: {:?}", result);
    println!("  Time: {:.2} ms", fast_time.as_secs_f64() * 1000.0);
    println!();

    // Comparison
    let speedup = std_time.as_secs_f64() / fast_time.as_secs_f64();
    println!("{}", "-".repeat(70));
    if speedup > 1.0 {
        println!("FastVM is {:.2}x FASTER than Standard VM", speedup);
    } else {
        println!("FastVM is {:.2}x slower than Standard VM", 1.0 / speedup);
    }
    println!("{}", "-".repeat(70));
    println!();

    // Python comparison
    println!("Python (for reference): ~110 ms");
    println!();

    let python_time = 110.0; // ms
    let fast_time_ms = fast_time.as_secs_f64() * 1000.0;
    let vs_python = fast_time_ms / python_time;

    if vs_python < 1.0 {
        println!("FastVM is {:.2}x FASTER than Python!", 1.0 / vs_python);
    } else {
        println!("FastVM is {:.2}x slower than Python", vs_python);
    }

    println!();
    println!("{}", "=".repeat(70));
}

//! FastVM Benchmark with SelfCall optimization
//!
//! Run with: cargo run --release --example fast_bench_selfcall

use std::time::Instant;
use vais_ir::Value;
use vais_lowering::Lowerer;
use vais_vm::{execute_function, execute_fast};

fn main() {
    println!("{}", "=".repeat(70));
    println!("     FastVM SelfCall Optimization Benchmark");
    println!("{}", "=".repeat(70));
    println!();

    // Fibonacci with explicit self-call syntax $()
    let source_selfcall = "fib(n) = n <= 1 ? n : $(n - 1) + $(n - 2)";
    let source_normal = "fib(n) = n <= 1 ? n : fib(n - 1) + fib(n - 2)";

    let args = vec![Value::Int(30)];

    // Print IR for selfcall version
    println!("SelfCall IR:");
    {
        let program = vais_parser::parse(source_selfcall).expect("Parse failed");
        let mut lowerer = Lowerer::new();
        let functions = lowerer.lower_program(&program).expect("Lowering failed");
        for instr in &functions[0].instructions {
            println!("  {:?}", instr.opcode);
        }
    }
    println!();

    // Normal version
    println!("Normal Call (fib):");
    let program = vais_parser::parse(source_normal).expect("Parse failed");
    let mut lowerer = Lowerer::new();
    let functions = lowerer.lower_program(&program).expect("Lowering failed");

    let start = Instant::now();
    let result = execute_fast(functions.clone(), "fib", args.clone());
    let time_normal = start.elapsed();
    println!("  FastVM Result: {:?}", result);
    println!("  Time: {:.2} ms", time_normal.as_secs_f64() * 1000.0);
    println!();

    // SelfCall version
    println!("SelfCall ($):");
    let program = vais_parser::parse(source_selfcall).expect("Parse failed");
    let mut lowerer = Lowerer::new();
    let functions = lowerer.lower_program(&program).expect("Lowering failed");

    let start = Instant::now();
    let result = execute_fast(functions.clone(), "fib", args.clone());
    let time_selfcall = start.elapsed();
    println!("  FastVM Result: {:?}", result);
    println!("  Time: {:.2} ms", time_selfcall.as_secs_f64() * 1000.0);
    println!();

    // Comparison
    let speedup = time_normal.as_secs_f64() / time_selfcall.as_secs_f64();
    println!("{}", "-".repeat(70));
    if speedup > 1.0 {
        println!("SelfCall is {:.2}x FASTER than normal Call", speedup);
    } else {
        println!("SelfCall is {:.2}x slower than normal Call", 1.0 / speedup);
    }
    println!("{}", "-".repeat(70));

    // Compare with Python
    let python_time = 110.0;
    let selfcall_ms = time_selfcall.as_secs_f64() * 1000.0;
    if selfcall_ms < python_time {
        println!("SelfCall FastVM is {:.2}x FASTER than Python!", python_time / selfcall_ms);
    } else {
        println!("SelfCall FastVM is {:.2}x slower than Python", selfcall_ms / python_time);
    }

    println!("{}", "=".repeat(70));
}

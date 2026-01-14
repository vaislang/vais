//! Fibonacci Benchmark with configurable N
//!
//! Run with: cargo run --release --example fib_compare --features jit -- 35

use std::time::Instant;
use std::env;
use vais_ir::Value;
use vais_lowering::Lowerer;
use vais_vm::{execute_function, execute_fast};

#[cfg(feature = "jit")]
use vais_vm::{JitVm, JitConfig};

fn main() {
    let n: i64 = env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    println!("============================================================");
    println!("                    fib({}) Benchmark", n);
    println!("============================================================\n");

    let source_call = "fib(n) = n <= 1 ? n : fib(n - 1) + fib(n - 2)";
    let source_selfcall = "fib(n) = n <= 1 ? n : $(n - 1) + $(n - 2)";
    let args = vec![Value::Int(n)];

    // Standard VM
    let program = vais_parser::parse(source_call).expect("Parse failed");
    let mut lowerer = Lowerer::new();
    let functions = lowerer.lower_program(&program).expect("Lowering failed");

    let start = Instant::now();
    let result = execute_function(functions.clone(), "fib", args.clone());
    let std_ms = start.elapsed().as_secs_f64() * 1000.0;
    let result_val = result.unwrap();
    println!("Standard VM:        {:>12.2} ms  (result: {:?})", std_ms, result_val);

    // FastVM
    let start = Instant::now();
    let _result = execute_fast(functions.clone(), "fib", args.clone());
    let fast_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("FastVM:             {:>12.2} ms  ({:.2}x faster)", fast_ms, std_ms / fast_ms);

    // FastVM + SelfCall
    let program = vais_parser::parse(source_selfcall).expect("Parse failed");
    let mut lowerer = Lowerer::new();
    let functions_selfcall = lowerer.lower_program(&program).expect("Lowering failed");

    let start = Instant::now();
    let _result = execute_fast(functions_selfcall.clone(), "fib", args.clone());
    let fast_sc_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("FastVM+SelfCall:    {:>12.2} ms  ({:.2}x faster)", fast_sc_ms, std_ms / fast_sc_ms);

    // JIT VM
    #[cfg(feature = "jit")]
    {
        let program = vais_parser::parse(source_call).expect("Parse failed");
        let mut lowerer = Lowerer::new();
        let functions = lowerer.lower_program(&program).expect("Lowering failed");

        let mut jit_vm = JitVm::with_config(JitConfig {
            enabled: true,
            auto_jit: false,
            profiling: false,
            threshold: 1,
        });
        jit_vm.load_functions(functions);
        let _ = jit_vm.compile_function("fib");

        let start = Instant::now();
        let _result = jit_vm.call_function("fib", args.clone());
        let jit_ms = start.elapsed().as_secs_f64() * 1000.0;
        println!("JIT VM:             {:>12.2} ms  ({:.2}x faster)", jit_ms, std_ms / jit_ms);

        // JIT + SelfCall
        let program = vais_parser::parse(source_selfcall).expect("Parse failed");
        let mut lowerer = Lowerer::new();
        let functions_selfcall = lowerer.lower_program(&program).expect("Lowering failed");

        let mut jit_vm = JitVm::with_config(JitConfig {
            enabled: true,
            auto_jit: false,
            profiling: false,
            threshold: 1,
        });
        jit_vm.load_functions(functions_selfcall);
        let _ = jit_vm.compile_function("fib");

        let start = Instant::now();
        let _result = jit_vm.call_function("fib", args.clone());
        let jit_sc_ms = start.elapsed().as_secs_f64() * 1000.0;
        println!("JIT+SelfCall:       {:>12.2} ms  ({:.2}x faster)", jit_sc_ms, std_ms / jit_sc_ms);
    }

    println!("\n============================================================");
}

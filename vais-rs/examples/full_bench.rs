//! Full Performance Benchmark
//!
//! Compares: Python vs Standard VM vs FastVM vs JIT VM
//!
//! Run with: cargo run --release --example full_bench --features jit

use std::time::Instant;
use vais_ir::Value;
use vais_lowering::Lowerer;
use vais_vm::{execute_function, execute_fast};

#[cfg(feature = "jit")]
use vais_vm::{JitVm, JitConfig};

fn main() {
    println!("{}", "=".repeat(75));
    println!("              Vais VM Performance Comparison");
    println!("              fibonacci(30) Benchmark");
    println!("{}", "=".repeat(75));
    println!();

    // Python reference time
    let python_time_ms = 110.0;
    println!("Python (CPython 3.x reference): ~{:.0} ms", python_time_ms);
    println!();

    // Source code variants
    let source_call = "fib(n) = n <= 1 ? n : fib(n - 1) + fib(n - 2)";
    let source_selfcall = "fib(n) = n <= 1 ? n : $(n - 1) + $(n - 2)";
    let args = vec![Value::Int(30)];

    println!("{}", "-".repeat(75));
    println!("{:<30} | {:>12} | {:>12} | {:>12}", "VM Type", "Time (ms)", "vs Python", "vs Std VM");
    println!("{}", "-".repeat(75));

    // 1. Standard VM (normal call)
    let program = vais_parser::parse(source_call).expect("Parse failed");
    let mut lowerer = Lowerer::new();
    let functions = lowerer.lower_program(&program).expect("Lowering failed");

    let start = Instant::now();
    let result = execute_function(functions.clone(), "fib", args.clone());
    let std_time = start.elapsed();
    let std_ms = std_time.as_secs_f64() * 1000.0;

    assert!(result.is_ok());
    println!("{:<30} | {:>12.2} | {:>11.2}x | {:>12}",
        "Standard VM",
        std_ms,
        std_ms / python_time_ms,
        "baseline"
    );

    // 2. FastVM (normal call)
    let start = Instant::now();
    let result = execute_fast(functions.clone(), "fib", args.clone());
    let fast_time = start.elapsed();
    let fast_ms = fast_time.as_secs_f64() * 1000.0;

    assert!(result.is_ok());
    println!("{:<30} | {:>12.2} | {:>11.2}x | {:>11.2}x",
        "FastVM (NaN-boxing)",
        fast_ms,
        fast_ms / python_time_ms,
        std_ms / fast_ms
    );

    // 3. FastVM with SelfCall
    let program = vais_parser::parse(source_selfcall).expect("Parse failed");
    let mut lowerer = Lowerer::new();
    let functions_selfcall = lowerer.lower_program(&program).expect("Lowering failed");

    let start = Instant::now();
    let result = execute_fast(functions_selfcall.clone(), "fib", args.clone());
    let fast_selfcall_time = start.elapsed();
    let fast_selfcall_ms = fast_selfcall_time.as_secs_f64() * 1000.0;

    assert!(result.is_ok());
    println!("{:<30} | {:>12.2} | {:>11.2}x | {:>11.2}x",
        "FastVM + SelfCall",
        fast_selfcall_ms,
        fast_selfcall_ms / python_time_ms,
        std_ms / fast_selfcall_ms
    );

    // JIT benchmarks (only available with jit feature)
    #[cfg(feature = "jit")]
    let (jit_ms, jit_selfcall_ms) = {
        // 4. JIT VM (normal call)
        let program = vais_parser::parse(source_call).expect("Parse failed");
        let mut lowerer = Lowerer::new();
        let functions = lowerer.lower_program(&program).expect("Lowering failed");

        let mut jit_vm = JitVm::with_config(JitConfig {
            enabled: true,
            auto_jit: false,  // Manual compile
            profiling: false,
            threshold: 1,
        });
        jit_vm.load_functions(functions.clone());

        // Pre-compile the function
        if let Err(e) = jit_vm.compile_function("fib") {
            println!("JIT compilation failed: {:?}", e);
        }

        let start = Instant::now();
        let result = jit_vm.call_function("fib", args.clone());
        let jit_time = start.elapsed();
        let jit_ms = jit_time.as_secs_f64() * 1000.0;

        match result {
            Ok(_) => {
                println!("{:<30} | {:>12.2} | {:>11.2}x | {:>11.2}x",
                    "JIT VM (Cranelift)",
                    jit_ms,
                    jit_ms / python_time_ms,
                    std_ms / jit_ms
                );
            }
            Err(e) => {
                println!("{:<30} | {:>12} | {:>12} | {:>12}",
                    "JIT VM (Cranelift)",
                    format!("Error: {:?}", e),
                    "-",
                    "-"
                );
            }
        }

        // 5. JIT VM with SelfCall (if applicable)
        let program = vais_parser::parse(source_selfcall).expect("Parse failed");
        let mut lowerer = Lowerer::new();
        let functions_selfcall = lowerer.lower_program(&program).expect("Lowering failed");

        let mut jit_vm_selfcall = JitVm::with_config(JitConfig {
            enabled: true,
            auto_jit: false,
            profiling: false,
            threshold: 1,
        });
        jit_vm_selfcall.load_functions(functions_selfcall.clone());

        if let Err(e) = jit_vm_selfcall.compile_function("fib") {
            println!("JIT (SelfCall) compilation failed: {:?}", e);
        }

        let start = Instant::now();
        let result = jit_vm_selfcall.call_function("fib", args.clone());
        let jit_selfcall_time = start.elapsed();
        let jit_selfcall_ms = jit_selfcall_time.as_secs_f64() * 1000.0;

        match result {
            Ok(_) => {
                println!("{:<30} | {:>12.2} | {:>11.2}x | {:>11.2}x",
                    "JIT VM + SelfCall",
                    jit_selfcall_ms,
                    jit_selfcall_ms / python_time_ms,
                    std_ms / jit_selfcall_ms
                );
            }
            Err(e) => {
                println!("{:<30} | {:>12} | {:>12} | {:>12}",
                    "JIT VM + SelfCall",
                    format!("Err: {:?}", e),
                    "-",
                    "-"
                );
            }
        }

        (jit_ms, jit_selfcall_ms)
    };

    #[cfg(not(feature = "jit"))]
    let (jit_ms, jit_selfcall_ms) = {
        println!("{:<30} | {:>12} | {:>12} | {:>12}",
            "JIT VM (Cranelift)",
            "N/A",
            "(needs --features jit)",
            "-"
        );
        println!("{:<30} | {:>12} | {:>12} | {:>12}",
            "JIT VM + SelfCall",
            "N/A",
            "(needs --features jit)",
            "-"
        );
        (f64::MAX, f64::MAX)
    };

    println!("{}", "-".repeat(75));
    println!();

    // Summary
    println!("{}", "=".repeat(75));
    println!("                           SUMMARY");
    println!("{}", "=".repeat(75));
    println!();

    let best_time = fast_selfcall_ms.min(jit_ms.min(jit_selfcall_ms));
    let best_name = if best_time == fast_selfcall_ms {
        "FastVM + SelfCall"
    } else if best_time == jit_selfcall_ms {
        "JIT VM + SelfCall"
    } else {
        "JIT VM"
    };

    if best_time < python_time_ms {
        println!("Best: {} at {:.2} ms", best_name, best_time);
        println!("      {:.2}x FASTER than Python!", python_time_ms / best_time);
    } else {
        println!("Best: {} at {:.2} ms", best_name, best_time);
        println!("      {:.2}x slower than Python", best_time / python_time_ms);
    }

    println!();
    println!("Optimization techniques applied:");
    println!("  - NaN-boxing: All values in 64-bit (O(1) copy)");
    println!("  - Type Specialization: Fast path for integers");
    println!("  - Trampoline Execution: No Rust recursion overhead");
    println!("  - SelfCall: Cached function reference (no HashMap lookup)");
    println!("  - JIT: Native code via Cranelift");
    println!();
    println!("{}", "=".repeat(75));
}

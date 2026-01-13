//! JIT vs VM vs Python benchmark
//! Run: cargo run --release --example jit_bench --features jit

#[cfg(feature = "jit")]
use aoel_ir::{Instruction, OpCode, Value};
#[cfg(feature = "jit")]
use aoel_jit::JitCompiler;
#[cfg(feature = "jit")]
use aoel_lowering::{CompiledFunction, Lowerer};
#[cfg(feature = "jit")]
use aoel_vm::execute_function;
#[cfg(feature = "jit")]
use std::time::Instant;

#[cfg(feature = "jit")]
fn make_instruction(opcode: OpCode) -> Instruction {
    Instruction::new(opcode)
}

#[cfg(not(feature = "jit"))]
fn main() {
    println!("JIT feature가 필요합니다. --features jit 옵션으로 실행하세요.");
}

#[cfg(feature = "jit")]
fn main() {
    println!("{}", "=".repeat(70));
    println!("AOEL JIT vs VM vs Python 벤치마크");
    println!("{}", "=".repeat(70));
    println!();

    // 1. Factorial benchmark
    bench_factorial();

    // 2. Fibonacci benchmark
    bench_fibonacci();
}

#[cfg(feature = "jit")]
fn bench_factorial() {
    println!("📌 Factorial(20)");
    println!("{}", "-".repeat(50));

    // Compile AOEL source
    let source = "fact(n) = n < 2 ? 1 : n * $(n - 1)";
    let program = aoel_parser::parse(source).unwrap();
    let functions = Lowerer::new().lower_program(&program).unwrap();

    let n = 20i64;
    let iterations = 10000;

    // VM benchmark
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = execute_function(functions.clone(), "fact", vec![Value::Int(n)]);
    }
    let vm_time = start.elapsed().as_nanos() as f64 / iterations as f64;

    // JIT benchmark - need to create JIT-compatible function
    let jit_func = CompiledFunction {
        name: "fact".to_string(),
        params: vec!["n".to_string()],
        instructions: vec![
            make_instruction(OpCode::Load("n".to_string())),    // 0
            make_instruction(OpCode::Const(Value::Int(2))),     // 1
            make_instruction(OpCode::Lt),                        // 2
            make_instruction(OpCode::JumpIfNot(3)),              // 3
            make_instruction(OpCode::Const(Value::Int(1))),     // 4
            make_instruction(OpCode::Return),                    // 5
            make_instruction(OpCode::Load("n".to_string())),    // 6
            make_instruction(OpCode::Load("n".to_string())),    // 7
            make_instruction(OpCode::Const(Value::Int(1))),     // 8
            make_instruction(OpCode::Sub),                       // 9
            make_instruction(OpCode::SelfCall(1)),               // 10
            make_instruction(OpCode::Mul),                       // 11
            make_instruction(OpCode::Return),                    // 12
        ],
        local_count: 1,
    };

    let mut jit = JitCompiler::new().unwrap();
    jit.compile_function_int(&jit_func).unwrap();

    // Warmup
    for _ in 0..100 {
        unsafe { let _ = jit.call_int("fact", &[n]); }
    }

    let start = Instant::now();
    for _ in 0..iterations {
        unsafe { let _ = jit.call_int("fact", &[n]); }
    }
    let jit_time = start.elapsed().as_nanos() as f64 / iterations as f64;

    // Python reference (measured separately): ~1.03 µs
    let python_time = 1030.0; // ns

    println!("  VM:     {:>10.2} ns", vm_time);
    println!("  JIT:    {:>10.2} ns", jit_time);
    println!("  Python: {:>10.2} ns (참고)", python_time);
    println!();
    println!("  JIT vs VM:     {:>6.1}x 빠름", vm_time / jit_time);
    println!("  JIT vs Python: {:>6.1}x 빠름", python_time / jit_time);
    println!();
}

#[cfg(feature = "jit")]
fn bench_fibonacci() {
    println!("📌 Fibonacci(20)");
    println!("{}", "-".repeat(50));

    // Compile AOEL source
    let source = "fib(n) = n < 2 ? n : $(n - 1) + $(n - 2)";
    let program = aoel_parser::parse(source).unwrap();
    let functions = Lowerer::new().lower_program(&program).unwrap();

    let n = 20i64;
    let iterations = 100; // Fewer iterations since fib(20) is slow

    // VM benchmark
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = execute_function(functions.clone(), "fib", vec![Value::Int(n)]);
    }
    let vm_time = start.elapsed().as_nanos() as f64 / iterations as f64;

    // JIT benchmark
    let jit_func = CompiledFunction {
        name: "fib".to_string(),
        params: vec!["n".to_string()],
        instructions: vec![
            make_instruction(OpCode::Load("n".to_string())),    // 0
            make_instruction(OpCode::Const(Value::Int(2))),     // 1
            make_instruction(OpCode::Lt),                        // 2
            make_instruction(OpCode::JumpIfNot(3)),              // 3
            make_instruction(OpCode::Load("n".to_string())),    // 4
            make_instruction(OpCode::Return),                    // 5
            make_instruction(OpCode::Load("n".to_string())),    // 6
            make_instruction(OpCode::Const(Value::Int(1))),     // 7
            make_instruction(OpCode::Sub),                       // 8
            make_instruction(OpCode::SelfCall(1)),               // 9
            make_instruction(OpCode::Load("n".to_string())),    // 10
            make_instruction(OpCode::Const(Value::Int(2))),     // 11
            make_instruction(OpCode::Sub),                       // 12
            make_instruction(OpCode::SelfCall(1)),               // 13
            make_instruction(OpCode::Add),                       // 14
            make_instruction(OpCode::Return),                    // 15
        ],
        local_count: 1,
    };

    let mut jit = JitCompiler::new().unwrap();
    jit.compile_function_int(&jit_func).unwrap();

    // Warmup
    for _ in 0..10 {
        unsafe { let _ = jit.call_int("fib", &[n]); }
    }

    let start = Instant::now();
    for _ in 0..iterations {
        unsafe { let _ = jit.call_int("fib", &[n]); }
    }
    let jit_time = start.elapsed().as_nanos() as f64 / iterations as f64;

    // Python reference: ~922 µs
    let python_time = 922350.0; // ns

    println!("  VM:     {:>12.2} ns ({:.2} ms)", vm_time, vm_time / 1_000_000.0);
    println!("  JIT:    {:>12.2} ns ({:.2} ms)", jit_time, jit_time / 1_000_000.0);
    println!("  Python: {:>12.2} ns ({:.2} ms, 참고)", python_time, python_time / 1_000_000.0);
    println!();
    println!("  JIT vs VM:     {:>6.1}x 빠름", vm_time / jit_time);
    println!("  JIT vs Python: {:>6.1}x 빠름", python_time / jit_time);
    println!();
}

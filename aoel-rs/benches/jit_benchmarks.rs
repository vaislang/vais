//! AOEL JIT vs 인터프리터 성능 벤치마크
//!
//! JIT 컴파일이 활성화된 경우와 인터프리터의 성능을 비교합니다.
//!
//! 실행: cargo bench --features jit --bench jit_benchmarks

#![cfg(feature = "jit")]

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use aoel_ir::{Instruction, OpCode, Value};
use aoel_lowering::CompiledFunction;
use aoel_vm::{execute_function, JitVm, JitConfig};
use aoel_jit::JitCompiler;

/// 간단한 더하기 함수 생성
fn make_add_function() -> CompiledFunction {
    CompiledFunction {
        name: "add".to_string(),
        params: vec!["a".to_string(), "b".to_string()],
        instructions: vec![
            Instruction::new(OpCode::Load("a".to_string())),
            Instruction::new(OpCode::Load("b".to_string())),
            Instruction::new(OpCode::Add),
            Instruction::new(OpCode::Return),
        ],
    }
}

/// 복잡한 산술 함수: (a + b) * (a - b)
fn make_calc_function() -> CompiledFunction {
    CompiledFunction {
        name: "calc".to_string(),
        params: vec!["a".to_string(), "b".to_string()],
        instructions: vec![
            // a + b
            Instruction::new(OpCode::Load("a".to_string())),
            Instruction::new(OpCode::Load("b".to_string())),
            Instruction::new(OpCode::Add),
            // a - b
            Instruction::new(OpCode::Load("a".to_string())),
            Instruction::new(OpCode::Load("b".to_string())),
            Instruction::new(OpCode::Sub),
            // (a + b) * (a - b)
            Instruction::new(OpCode::Mul),
            Instruction::new(OpCode::Return),
        ],
    }
}

/// 여러 연산이 포함된 수학 함수: ((x * 2 + 3) * 4 - 5) / 2
fn make_math_function() -> CompiledFunction {
    CompiledFunction {
        name: "math".to_string(),
        params: vec!["x".to_string()],
        instructions: vec![
            // x * 2
            Instruction::new(OpCode::Load("x".to_string())),
            Instruction::new(OpCode::Const(Value::Int(2))),
            Instruction::new(OpCode::Mul),
            // + 3
            Instruction::new(OpCode::Const(Value::Int(3))),
            Instruction::new(OpCode::Add),
            // * 4
            Instruction::new(OpCode::Const(Value::Int(4))),
            Instruction::new(OpCode::Mul),
            // - 5
            Instruction::new(OpCode::Const(Value::Int(5))),
            Instruction::new(OpCode::Sub),
            // / 2
            Instruction::new(OpCode::Const(Value::Int(2))),
            Instruction::new(OpCode::Div),
            Instruction::new(OpCode::Return),
        ],
    }
}

/// 인터프리터 벤치마크
fn bench_interpreter(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpreter");

    // add
    let add_func = make_add_function();
    group.bench_function("add", |b| {
        b.iter(|| {
            execute_function(
                vec![add_func.clone()],
                "add",
                vec![black_box(Value::Int(100)), black_box(Value::Int(200))],
            ).unwrap()
        });
    });

    // calc
    let calc_func = make_calc_function();
    group.bench_function("calc", |b| {
        b.iter(|| {
            execute_function(
                vec![calc_func.clone()],
                "calc",
                vec![black_box(Value::Int(50)), black_box(Value::Int(30))],
            ).unwrap()
        });
    });

    // math
    let math_func = make_math_function();
    group.bench_function("math", |b| {
        b.iter(|| {
            execute_function(
                vec![math_func.clone()],
                "math",
                vec![black_box(Value::Int(100))],
            ).unwrap()
        });
    });

    group.finish();
}

/// JIT 컴파일러 직접 호출 벤치마크 (순수 JIT 성능)
fn bench_jit_direct(c: &mut Criterion) {
    let mut group = c.benchmark_group("jit_direct");

    // add - JIT 컴파일 후 직접 호출
    let add_func = make_add_function();
    let mut jit = JitCompiler::new().unwrap();
    jit.compile_function_int(&add_func).unwrap();

    group.bench_function("add", |b| {
        b.iter(|| {
            unsafe { jit.call_int("add", &[black_box(100), black_box(200)]).unwrap() }
        });
    });

    // calc - JIT 컴파일 후 직접 호출
    let calc_func = make_calc_function();
    let mut jit_calc = JitCompiler::new().unwrap();
    jit_calc.compile_function_int(&calc_func).unwrap();

    group.bench_function("calc", |b| {
        b.iter(|| {
            unsafe { jit_calc.call_int("calc", &[black_box(50), black_box(30)]).unwrap() }
        });
    });

    // math - JIT 컴파일 후 직접 호출
    let math_func = make_math_function();
    let mut jit_math = JitCompiler::new().unwrap();
    jit_math.compile_function_int(&math_func).unwrap();

    group.bench_function("math", |b| {
        b.iter(|| {
            unsafe { jit_math.call_int("math", &[black_box(100)]).unwrap() }
        });
    });

    group.finish();
}

/// JitVm 통합 벤치마크 (명시적 JIT 컴파일)
fn bench_jit_vm(c: &mut Criterion) {
    let mut group = c.benchmark_group("jit_vm");

    // add - JitVm으로 명시적 컴파일 후 호출
    let add_func = make_add_function();
    let mut vm = JitVm::with_config(JitConfig {
        enabled: true,
        auto_jit: false,
        profiling: false,
        threshold: 100,
    });
    vm.load_functions(vec![add_func]);
    vm.compile_function("add").unwrap();

    group.bench_function("add", |b| {
        b.iter(|| {
            vm.call_function("add", vec![black_box(Value::Int(100)), black_box(Value::Int(200))]).unwrap()
        });
    });

    // calc
    let calc_func = make_calc_function();
    let mut vm_calc = JitVm::with_config(JitConfig {
        enabled: true,
        auto_jit: false,
        profiling: false,
        threshold: 100,
    });
    vm_calc.load_functions(vec![calc_func]);
    vm_calc.compile_function("calc").unwrap();

    group.bench_function("calc", |b| {
        b.iter(|| {
            vm_calc.call_function("calc", vec![black_box(Value::Int(50)), black_box(Value::Int(30))]).unwrap()
        });
    });

    // math
    let math_func = make_math_function();
    let mut vm_math = JitVm::with_config(JitConfig {
        enabled: true,
        auto_jit: false,
        profiling: false,
        threshold: 100,
    });
    vm_math.load_functions(vec![math_func]);
    vm_math.compile_function("math").unwrap();

    group.bench_function("math", |b| {
        b.iter(|| {
            vm_math.call_function("math", vec![black_box(Value::Int(100))]).unwrap()
        });
    });

    group.finish();
}

/// 인터프리터 vs JIT 비교
fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison");

    let funcs = vec![
        ("add", make_add_function(), vec![Value::Int(100), Value::Int(200)]),
        ("calc", make_calc_function(), vec![Value::Int(50), Value::Int(30)]),
        ("math", make_math_function(), vec![Value::Int(100)]),
    ];

    for (name, func, args) in funcs {
        // 인터프리터
        let func_clone = func.clone();
        let args_clone = args.clone();
        group.bench_with_input(
            BenchmarkId::new("interpreter", name),
            &(func_clone, args_clone),
            |b, (f, a)| {
                b.iter(|| {
                    execute_function(vec![f.clone()], name, a.clone()).unwrap()
                });
            },
        );

        // JIT (직접 호출)
        let mut jit = JitCompiler::new().unwrap();
        jit.compile_function_int(&func).unwrap();
        let int_args: Vec<i64> = args.iter()
            .filter_map(|v| match v {
                Value::Int(n) => Some(*n),
                _ => None,
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("jit_direct", name),
            &int_args,
            |b, a| {
                b.iter(|| {
                    unsafe { jit.call_int(name, black_box(a)).unwrap() }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_interpreter,
    bench_jit_direct,
    bench_jit_vm,
    bench_comparison,
);

criterion_main!(benches);

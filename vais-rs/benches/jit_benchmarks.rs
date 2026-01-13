//! Vais JIT vs 인터프리터 성능 벤치마크
//!
//! JIT 컴파일이 활성화된 경우와 인터프리터의 성능을 비교합니다.
//!
//! 실행: cargo bench --features jit --bench jit_benchmarks

#![cfg(feature = "jit")]

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use vais_ir::{Instruction, OpCode, Value};
use vais_lowering::CompiledFunction;
use vais_vm::{execute_function, JitVm, JitConfig};
use vais_jit::JitCompiler;

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
        local_count: 2,
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
        local_count: 2,
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
        local_count: 1,
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

/// Factorial 함수 (재귀)
fn make_factorial_function() -> CompiledFunction {
    // factorial(n) = n <= 1 ? 1 : n * factorial(n - 1)
    CompiledFunction {
        name: "factorial".to_string(),
        params: vec!["n".to_string()],
        instructions: vec![
            Instruction::new(OpCode::Load("n".to_string())),    // 0
            Instruction::new(OpCode::Const(Value::Int(1))),     // 1
            Instruction::new(OpCode::Lte),                       // 2
            Instruction::new(OpCode::JumpIfNot(3)),              // 3
            Instruction::new(OpCode::Const(Value::Int(1))),     // 4
            Instruction::new(OpCode::Return),                    // 5
            Instruction::new(OpCode::Load("n".to_string())),    // 6
            Instruction::new(OpCode::Load("n".to_string())),    // 7
            Instruction::new(OpCode::Const(Value::Int(1))),     // 8
            Instruction::new(OpCode::Sub),                       // 9
            Instruction::new(OpCode::SelfCall(1)),               // 10
            Instruction::new(OpCode::Mul),                       // 11
            Instruction::new(OpCode::Return),                    // 12
        ],
        local_count: 1,
    }
}

/// Tail-recursive sum 함수 (TCO)
fn make_sum_tail_function() -> CompiledFunction {
    // sum_tail(n, acc) = n <= 0 ? acc : sum_tail(n - 1, acc + n)
    CompiledFunction {
        name: "sum_tail".to_string(),
        params: vec!["n".to_string(), "acc".to_string()],
        instructions: vec![
            Instruction::new(OpCode::Load("n".to_string())),    // 0
            Instruction::new(OpCode::Const(Value::Int(0))),     // 1
            Instruction::new(OpCode::Lte),                       // 2
            Instruction::new(OpCode::JumpIfNot(3)),              // 3
            Instruction::new(OpCode::Load("acc".to_string())),  // 4
            Instruction::new(OpCode::Return),                    // 5
            Instruction::new(OpCode::Load("n".to_string())),    // 6
            Instruction::new(OpCode::Const(Value::Int(1))),     // 7
            Instruction::new(OpCode::Sub),                       // 8
            Instruction::new(OpCode::Load("acc".to_string())),  // 9
            Instruction::new(OpCode::Load("n".to_string())),    // 10
            Instruction::new(OpCode::Add),                       // 11
            Instruction::new(OpCode::TailSelfCall(2)),           // 12
        ],
        local_count: 2,
    }
}

/// 재귀 함수 JIT 벤치마크
fn bench_recursive_jit(c: &mut Criterion) {
    let mut group = c.benchmark_group("recursive_jit");

    // Factorial
    let fact_func = make_factorial_function();
    let mut jit_fact = JitCompiler::new().unwrap();
    jit_fact.compile_function_int(&fact_func).unwrap();

    group.bench_function("factorial_10", |b| {
        b.iter(|| {
            unsafe { jit_fact.call_int("factorial", black_box(&[10])).unwrap() }
        });
    });

    group.bench_function("factorial_15", |b| {
        b.iter(|| {
            unsafe { jit_fact.call_int("factorial", black_box(&[15])).unwrap() }
        });
    });

    // Sum with TCO
    let sum_func = make_sum_tail_function();
    let mut jit_sum = JitCompiler::new().unwrap();
    jit_sum.compile_function_int(&sum_func).unwrap();

    group.bench_function("sum_100", |b| {
        b.iter(|| {
            unsafe { jit_sum.call_int("sum_tail", black_box(&[100, 0])).unwrap() }
        });
    });

    group.bench_function("sum_1000", |b| {
        b.iter(|| {
            unsafe { jit_sum.call_int("sum_tail", black_box(&[1000, 0])).unwrap() }
        });
    });

    // 큰 값 - TCO 덕분에 스택 오버플로우 없음
    group.bench_function("sum_10000", |b| {
        b.iter(|| {
            unsafe { jit_sum.call_int("sum_tail", black_box(&[10000, 0])).unwrap() }
        });
    });

    group.finish();
}

// Note: 인터프리터 vs JIT 비교 벤치마크는 VM의 SelfCall 처리와
// 벤치마크 IR 구조 간의 불일치로 인해 현재 비활성화됨.

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

/// double(x) = x * 2
fn make_double_function() -> CompiledFunction {
    CompiledFunction {
        name: "double".to_string(),
        params: vec!["x".to_string()],
        instructions: vec![
            Instruction::new(OpCode::Load("x".to_string())),
            Instruction::new(OpCode::Const(Value::Int(2))),
            Instruction::new(OpCode::Mul),
            Instruction::new(OpCode::Return),
        ],
        local_count: 1,
    }
}

/// quadruple(x) = double(double(x)) = x * 4
fn make_quadruple_function() -> CompiledFunction {
    CompiledFunction {
        name: "quadruple".to_string(),
        params: vec!["x".to_string()],
        instructions: vec![
            Instruction::new(OpCode::Load("x".to_string())),
            Instruction::new(OpCode::Call("double".to_string(), 1)),
            Instruction::new(OpCode::Call("double".to_string(), 1)),
            Instruction::new(OpCode::Return),
        ],
        local_count: 1,
    }
}

/// add_helper(a, b) = a + b
fn make_add_helper_function() -> CompiledFunction {
    CompiledFunction {
        name: "add_helper".to_string(),
        params: vec!["a".to_string(), "b".to_string()],
        instructions: vec![
            Instruction::new(OpCode::Load("a".to_string())),
            Instruction::new(OpCode::Load("b".to_string())),
            Instruction::new(OpCode::Add),
            Instruction::new(OpCode::Return),
        ],
        local_count: 2,
    }
}

/// sum_with_helper(n, acc) = n <= 0 ? acc : sum_with_helper(n-1, add_helper(acc, n))
fn make_sum_with_helper_function() -> CompiledFunction {
    CompiledFunction {
        name: "sum_with_helper".to_string(),
        params: vec!["n".to_string(), "acc".to_string()],
        instructions: vec![
            Instruction::new(OpCode::Load("n".to_string())),     // 0
            Instruction::new(OpCode::Const(Value::Int(0))),      // 1
            Instruction::new(OpCode::Lte),                        // 2
            Instruction::new(OpCode::JumpIfNot(3)),               // 3
            Instruction::new(OpCode::Load("acc".to_string())),   // 4
            Instruction::new(OpCode::Return),                     // 5
            Instruction::new(OpCode::Load("n".to_string())),     // 6
            Instruction::new(OpCode::Const(Value::Int(1))),      // 7
            Instruction::new(OpCode::Sub),                        // 8
            Instruction::new(OpCode::Load("acc".to_string())),   // 9
            Instruction::new(OpCode::Load("n".to_string())),     // 10
            Instruction::new(OpCode::Call("add_helper".to_string(), 2)), // 11
            Instruction::new(OpCode::TailSelfCall(2)),            // 12
        ],
        local_count: 2,
    }
}

/// 함수 간 호출 JIT 벤치마크
fn bench_multi_function_jit(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_function_jit");

    // 단순 함수 체이닝: double -> quadruple
    let double_func = make_double_function();
    let quadruple_func = make_quadruple_function();

    let mut jit_chain = JitCompiler::new().unwrap();
    jit_chain.compile_functions_batch(&[double_func, quadruple_func]).unwrap();

    group.bench_function("quadruple", |b| {
        b.iter(|| {
            unsafe { jit_chain.call_int("quadruple", black_box(&[100])).unwrap() }
        });
    });

    // 헬퍼 함수를 사용하는 재귀: add_helper + sum_with_helper
    let add_helper = make_add_helper_function();
    let sum_with_helper = make_sum_with_helper_function();

    let mut jit_helper = JitCompiler::new().unwrap();
    jit_helper.compile_functions_batch(&[add_helper, sum_with_helper]).unwrap();

    group.bench_function("sum_100_with_helper", |b| {
        b.iter(|| {
            unsafe { jit_helper.call_int("sum_with_helper", black_box(&[100, 0])).unwrap() }
        });
    });

    group.bench_function("sum_1000_with_helper", |b| {
        b.iter(|| {
            unsafe { jit_helper.call_int("sum_with_helper", black_box(&[1000, 0])).unwrap() }
        });
    });

    // 직접 tail call (헬퍼 없음) vs 헬퍼 함수 사용 비교
    // sum_tail은 위에서 정의한 것 사용
    let sum_direct = make_sum_tail_function();
    let mut jit_direct = JitCompiler::new().unwrap();
    jit_direct.compile_function_int(&sum_direct).unwrap();

    group.bench_function("sum_100_direct", |b| {
        b.iter(|| {
            unsafe { jit_direct.call_int("sum_tail", black_box(&[100, 0])).unwrap() }
        });
    });

    group.bench_function("sum_1000_direct", |b| {
        b.iter(|| {
            unsafe { jit_direct.call_int("sum_tail", black_box(&[1000, 0])).unwrap() }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_interpreter,
    bench_jit_direct,
    bench_jit_vm,
    bench_comparison,
    bench_recursive_jit,
    bench_multi_function_jit,
);

criterion_main!(benches);

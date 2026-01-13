//! Vais Benchmarks
//!
//! Performance benchmarks for the Vais compiler and runtime.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use vais_ir::Value;
use vais_lowering::Lowerer;
use vais_vm::execute_function;

/// Helper to compile Vais source to functions
fn compile(source: &str) -> Vec<vais_lowering::CompiledFunction> {
    let program = vais_parser::parse(source).unwrap();
    let mut lowerer = Lowerer::new();
    lowerer.lower_program(&program).unwrap()
}

/// Benchmark: Lexer performance
fn bench_lexer(c: &mut Criterion) {
    let sources = vec![
        ("simple", "add(a, b) = a + b"),
        ("factorial", "fact(n) = n < 2 ? 1 : n * $(n - 1)"),
        ("complex", r#"
            double(arr) = arr.@(_ * 2)
            filter_pos(arr) = arr.?(_ > 0)
            sum(arr) = arr./+
            process(arr) = arr.@(_ * 2).?(_ > 5)./+
        "#),
    ];

    let mut group = c.benchmark_group("lexer");
    for (name, source) in sources {
        group.bench_with_input(BenchmarkId::new("tokenize", name), source, |b, src| {
            b.iter(|| {
                let mut lexer = vais_lexer::Lexer::new(black_box(src));
                let _ = lexer.tokenize();
            });
        });
    }
    group.finish();
}

/// Benchmark: Parser performance
fn bench_parser(c: &mut Criterion) {
    let sources = vec![
        ("simple", "add(a, b) = a + b"),
        ("factorial", "fact(n) = n < 2 ? 1 : n * $(n - 1)"),
        ("multiple_funcs", r#"
            add(a, b) = a + b
            sub(a, b) = a - b
            mul(a, b) = a * b
            div(a, b) = a / b
            fact(n) = n < 2 ? 1 : n * $(n - 1)
            fib(n) = n < 2 ? n : $(n - 1) + $(n - 2)
        "#),
    ];

    let mut group = c.benchmark_group("parser");
    for (name, source) in sources {
        group.bench_with_input(BenchmarkId::new("parse", name), source, |b, src| {
            b.iter(|| {
                let _ = vais_parser::parse(black_box(src));
            });
        });
    }
    group.finish();
}

/// Benchmark: Lowering (AST to IR)
fn bench_lowering(c: &mut Criterion) {
    let sources = vec![
        ("simple", "add(a, b) = a + b"),
        ("factorial", "fact(n) = n < 2 ? 1 : n * $(n - 1)"),
        ("collection_ops", r#"
            double(arr) = arr.@(_ * 2)
            filter_pos(arr) = arr.?(_ > 0)
            sum(arr) = arr./+
        "#),
    ];

    let mut group = c.benchmark_group("lowering");
    for (name, source) in sources {
        let program = vais_parser::parse(source).unwrap();
        group.bench_with_input(BenchmarkId::new("lower", name), &program, |b, prog| {
            b.iter(|| {
                let mut lowerer = Lowerer::new();
                let _ = lowerer.lower_program(black_box(prog));
            });
        });
    }
    group.finish();
}

/// Benchmark: VM execution - Factorial
fn bench_factorial(c: &mut Criterion) {
    let source = "fact(n) = n < 2 ? 1 : n * $(n - 1)";
    let functions = compile(source);

    let mut group = c.benchmark_group("factorial");
    for n in [5, 10, 15, 20] {
        group.bench_with_input(BenchmarkId::new("compute", n), &n, |b, &n| {
            b.iter(|| {
                execute_function(functions.clone(), "fact", vec![Value::Int(black_box(n))]).unwrap()
            });
        });
    }
    group.finish();
}

/// Benchmark: VM execution - Fibonacci
fn bench_fibonacci(c: &mut Criterion) {
    let source = "fib(n) = n < 2 ? n : $(n - 1) + $(n - 2)";
    let functions = compile(source);

    let mut group = c.benchmark_group("fibonacci");
    // Smaller values since fibonacci is exponential without memoization
    for n in [5, 10, 15, 20] {
        group.bench_with_input(BenchmarkId::new("compute", n), &n, |b, &n| {
            b.iter(|| {
                execute_function(functions.clone(), "fib", vec![Value::Int(black_box(n))]).unwrap()
            });
        });
    }
    group.finish();
}

/// Benchmark: Collection operations - Map
fn bench_map(c: &mut Criterion) {
    let source = "double(arr) = arr.@(_ * 2)";
    let functions = compile(source);

    let mut group = c.benchmark_group("map");
    for size in [10, 100, 1000, 10000] {
        let arr = Value::Array((0..size).map(Value::Int).collect());
        group.bench_with_input(BenchmarkId::new("double", size), &arr, |b, arr| {
            b.iter(|| {
                execute_function(functions.clone(), "double", vec![black_box(arr.clone())]).unwrap()
            });
        });
    }
    group.finish();
}

/// Benchmark: Collection operations - Filter
fn bench_filter(c: &mut Criterion) {
    let source = "evens(arr) = arr.?(_ % 2 == 0)";
    let functions = compile(source);

    let mut group = c.benchmark_group("filter");
    for size in [10, 100, 1000, 10000] {
        let arr = Value::Array((0..size).map(Value::Int).collect());
        group.bench_with_input(BenchmarkId::new("evens", size), &arr, |b, arr| {
            b.iter(|| {
                execute_function(functions.clone(), "evens", vec![black_box(arr.clone())]).unwrap()
            });
        });
    }
    group.finish();
}

/// Benchmark: Collection operations - Reduce
fn bench_reduce(c: &mut Criterion) {
    let source = "sum(arr) = arr./+";
    let functions = compile(source);

    let mut group = c.benchmark_group("reduce");
    for size in [10, 100, 1000, 10000] {
        let arr = Value::Array((0..size).map(Value::Int).collect());
        group.bench_with_input(BenchmarkId::new("sum", size), &arr, |b, arr| {
            b.iter(|| {
                execute_function(functions.clone(), "sum", vec![black_box(arr.clone())]).unwrap()
            });
        });
    }
    group.finish();
}

/// Benchmark: Chained operations
fn bench_chained_operations(c: &mut Criterion) {
    let source = "process(arr) = arr.@(_ * 2).?(_ > 50)./+";
    let functions = compile(source);

    let mut group = c.benchmark_group("chained");
    for size in [10, 100, 1000] {
        let arr = Value::Array((0..size).map(Value::Int).collect());
        group.bench_with_input(BenchmarkId::new("map_filter_reduce", size), &arr, |b, arr| {
            b.iter(|| {
                execute_function(functions.clone(), "process", vec![black_box(arr.clone())]).unwrap()
            });
        });
    }
    group.finish();
}

/// Benchmark: VM builtin functions (via Vais functions)
fn bench_builtins(c: &mut Criterion) {
    let mut group = c.benchmark_group("builtins");

    // Array length via Vais
    let len_funcs = compile("len_fn(arr) = #arr");
    let arr = Value::Array((0..100).map(Value::Int).collect());
    group.bench_function("len", |b| {
        b.iter(|| {
            execute_function(len_funcs.clone(), "len_fn", vec![black_box(arr.clone())]).unwrap()
        });
    });

    // Reverse via Vais
    let rev_funcs = compile("rev(arr) = reverse(arr)");
    group.bench_function("reverse", |b| {
        b.iter(|| {
            execute_function(rev_funcs.clone(), "rev", vec![black_box(arr.clone())]).unwrap()
        });
    });

    // Sort via Vais
    let sort_funcs = compile("sort_fn(arr) = sort(arr)");
    let unsorted = Value::Array(vec![
        Value::Int(5), Value::Int(2), Value::Int(8), Value::Int(1),
        Value::Int(9), Value::Int(3), Value::Int(7), Value::Int(4),
    ]);
    group.bench_function("sort", |b| {
        b.iter(|| {
            execute_function(sort_funcs.clone(), "sort_fn", vec![black_box(unsorted.clone())]).unwrap()
        });
    });

    // String concat via Vais
    let concat_funcs = compile("concat_fn(a, b) = a + b");
    let s1 = Value::String("Hello, ".to_string());
    let s2 = Value::String("World!".to_string());
    group.bench_function("concat", |b| {
        b.iter(|| {
            execute_function(concat_funcs.clone(), "concat_fn", vec![black_box(s1.clone()), black_box(s2.clone())]).unwrap()
        });
    });

    // Abs via Vais
    let abs_funcs = compile("abs_fn(n) = abs(n)");
    group.bench_function("abs", |b| {
        b.iter(|| {
            execute_function(abs_funcs.clone(), "abs_fn", vec![black_box(Value::Int(-42))]).unwrap()
        });
    });

    // Sqrt via Vais
    let sqrt_funcs = compile("sqrt_fn(x) = sqrt(x)");
    group.bench_function("sqrt", |b| {
        b.iter(|| {
            execute_function(sqrt_funcs.clone(), "sqrt_fn", vec![black_box(Value::Float(144.0))]).unwrap()
        });
    });

    group.finish();
}

/// Benchmark: Full pipeline (parse -> lower -> execute)
fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");

    // Simple function
    group.bench_function("simple_add", |b| {
        let source = "add(a, b) = a + b";
        b.iter(|| {
            let program = vais_parser::parse(black_box(source)).unwrap();
            let mut lowerer = Lowerer::new();
            let functions = lowerer.lower_program(&program).unwrap();
            execute_function(functions, "add", vec![Value::Int(3), Value::Int(5)]).unwrap()
        });
    });

    // Factorial
    group.bench_function("factorial_10", |b| {
        let source = "fact(n) = n < 2 ? 1 : n * $(n - 1)";
        b.iter(|| {
            let program = vais_parser::parse(black_box(source)).unwrap();
            let mut lowerer = Lowerer::new();
            let functions = lowerer.lower_program(&program).unwrap();
            execute_function(functions, "fact", vec![Value::Int(10)]).unwrap()
        });
    });

    // Collection operation
    group.bench_function("map_100_elements", |b| {
        let source = "double(arr) = arr.@(_ * 2)";
        let arr = Value::Array((0..100).map(Value::Int).collect());
        b.iter(|| {
            let program = vais_parser::parse(black_box(source)).unwrap();
            let mut lowerer = Lowerer::new();
            let functions = lowerer.lower_program(&program).unwrap();
            execute_function(functions, "double", vec![arr.clone()]).unwrap()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_lexer,
    bench_parser,
    bench_lowering,
    bench_factorial,
    bench_fibonacci,
    bench_map,
    bench_filter,
    bench_reduce,
    bench_chained_operations,
    bench_builtins,
    bench_full_pipeline,
);

criterion_main!(benches);

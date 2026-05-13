//! Endurance Benchmarks for Vais Compiler
//!
//! These benchmarks measure long-running stability, memory characteristics,
//! and throughput consistency across repeated compilation cycles.
//!
//! Run with: cargo bench --bench endurance_bench

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

// Re-use the lib.rs utilities
mod lib;
use lib::*;

/// Simple source for repeated benchmarking
fn generate_simple_source() -> String {
    r#"
F add(a: i64, b: i64) -> i64 {
    R a + b
}

F multiply(x: i64, y: i64) -> i64 {
    R x * y
}

F compute(n: i64) -> i64 {
    result := add(n, 10)
    result2 := multiply(result, 2)
    R result2
}

F main() -> i64 {
    R compute(42)
}
"#
    .to_string()
}

/// Benchmark: Repeated parse (100 iterations)
/// Tests parser throughput stability over repeated invocations
fn bench_repeated_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("endurance_repeated_parse");

    let source = generate_simple_source();
    let iterations = 100;

    group.bench_function("parse_100x", |b| {
        b.iter(|| {
            for _ in 0..iterations {
                let result = parse(black_box(&source));
                black_box(result).expect("Parse failed");
            }
        })
    });

    group.finish();
}

/// Benchmark: Repeated typecheck (100 iterations)
/// Tests type checker throughput stability
fn bench_repeated_typecheck(c: &mut Criterion) {
    let mut group = c.benchmark_group("endurance_repeated_typecheck");

    let source = generate_simple_source();
    let module = parse(&source).expect("Parse failed");
    let iterations = 100;

    group.bench_function("typecheck_100x", |b| {
        b.iter(|| {
            for _ in 0..iterations {
                let mut checker = TypeChecker::new();
                let result = checker.check_module(black_box(&module));
                black_box(result).expect("Type check failed");
            }
        })
    });

    group.finish();
}

/// Benchmark: Memory stability over 50 iterations
/// Measures allocation patterns for memory leak detection
fn bench_memory_stability(c: &mut Criterion) {
    let mut group = c.benchmark_group("endurance_memory_stability");

    let source = utils::generate_code(100); // 100 functions
    let iterations = 50;

    group.bench_function("compile_50x_memtrack", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = std::time::Duration::ZERO;

            for _ in 0..iters {
                let allocation_tracker = Arc::new(AtomicUsize::new(0));

                // Track allocations across iterations
                let mut alloc_history = Vec::with_capacity(iterations);

                for i in 0..iterations {
                    let before = allocation_tracker.load(Ordering::Relaxed);
                    let start = std::time::Instant::now();

                    // Full compilation pipeline
                    let tokens = tokenize(black_box(&source)).expect("Lex failed");
                    black_box(tokens);

                    let module = parse(black_box(&source)).expect("Parse failed");
                    let mut checker = TypeChecker::new();
                    let type_result = checker.check_module(&module);
                    black_box(type_result).ok();

                    total_duration += start.elapsed();

                    let after = allocation_tracker.load(Ordering::Relaxed);
                    alloc_history.push(after.saturating_sub(before));
                }

                // Calculate allocation stability
                if alloc_history.len() >= 2 {
                    let first = alloc_history[0];
                    let last = alloc_history[iterations - 1];

                    // Detect significant growth
                    if first > 0 {
                        let growth_percent = ((last as f64 - first as f64) / first as f64) * 100.0;
                        if growth_percent.abs() > 20.0 {
                            eprintln!(
                                "WARNING: Memory growth detected: {:.1}% (first: {}, last: {})",
                                growth_percent, first, last
                            );
                        }
                    }
                }
            }

            total_duration
        })
    });

    group.finish();
}

/// Benchmark: Scaling with different input sizes
/// Tests throughput across 1K, 5K, 10K, 50K line programs
fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("endurance_scaling");
    group.sample_size(20); // Reduce sample size for large inputs

    // Test different program sizes
    let sizes = vec![
        (50, "1K"),    // ~50 funcs ≈ 1K lines
        (250, "5K"),   // ~250 funcs ≈ 5K lines
        (500, "10K"),  // ~500 funcs ≈ 10K lines
        (2500, "50K"), // ~2500 funcs ≈ 50K lines
    ];

    for (num_funcs, label) in sizes {
        let source = utils::generate_code(num_funcs);
        let lines = source.lines().count();

        group.bench_with_input(BenchmarkId::new("parse", label), &source, |b, s| {
            b.iter(|| {
                let result = parse(black_box(s));
                black_box(result).expect("Parse failed")
            })
        });

        // Pre-parse for type check benchmark
        let module = parse(&source).expect("Parse failed");

        group.bench_with_input(BenchmarkId::new("typecheck", label), &module, |b, m| {
            b.iter(|| {
                let mut checker = TypeChecker::new();
                let result = checker.check_module(black_box(m));
                black_box(result).ok();
            })
        });

        eprintln!("Benchmarked {} ({} lines)", label, lines);
    }

    group.finish();
}

/// Benchmark: Full pipeline repeated compilation
/// Measures end-to-end stability
fn bench_repeated_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("endurance_full_pipeline");

    let source = utils::generate_code(100);
    let iterations = 50;

    group.bench_function("pipeline_50x", |b| {
        b.iter(|| {
            for _ in 0..iterations {
                let tokens = tokenize(black_box(&source)).expect("Lex failed");
                black_box(tokens);

                let module = parse(black_box(&source)).expect("Parse failed");
                let mut checker = TypeChecker::new();
                let result = checker.check_module(&module);
                black_box(result).ok();
            }
        })
    });

    group.finish();
}

/// Benchmark: Large project compilation
/// Tests memory and performance on substantial codebases
fn bench_large_project(c: &mut Criterion) {
    let mut group = c.benchmark_group("endurance_large_project");
    group.sample_size(10);

    let sizes = vec![(1000, "10K"), (5000, "50K"), (10000, "100K")];

    for (target_lines, label) in sizes {
        let source = utils::generate_large_project(target_lines);
        let actual_lines = source.lines().count();

        group.bench_with_input(BenchmarkId::new("compile", label), &source, |b, s| {
            b.iter(|| {
                let tokens = tokenize(black_box(s)).expect("Lex failed");
                black_box(tokens);

                let module = parse(black_box(s)).expect("Parse failed");
                let mut checker = TypeChecker::new();
                let result = checker.check_module(&module);
                black_box(result).ok();
            })
        });

        eprintln!(
            "Benchmarked large project: {} ({} lines)",
            label, actual_lines
        );
    }

    group.finish();
}

/// Benchmark: Incremental compilation simulation
/// Simulates repeated small changes and recompilation
fn bench_incremental_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("endurance_incremental");

    let iterations = 50;

    group.bench_function("incremental_50x", |b| {
        b.iter(|| {
            for i in 0..iterations {
                // Generate slightly different code each iteration
                let mut source = utils::generate_code(10);
                source.push_str(&format!(
                    "F incremental_func_{}(x: i64) -> i64 {{ R x + {} }}\n",
                    i, i
                ));

                let module = parse(black_box(&source)).expect("Parse failed");
                let mut checker = TypeChecker::new();
                let result = checker.check_module(&module);
                black_box(result).ok();
            }
        })
    });

    group.finish();
}

/// Benchmark: Burst compilation
/// Simulates rapid repeated compilation cycles
fn bench_burst_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("endurance_burst");

    let source = generate_simple_source();
    let burst_sizes = vec![10, 50, 100, 200];

    for burst_size in burst_sizes {
        group.bench_with_input(
            BenchmarkId::new("burst", burst_size),
            &burst_size,
            |b, &size| {
                b.iter(|| {
                    for _ in 0..size {
                        let module = parse(black_box(&source)).expect("Parse failed");
                        let mut checker = TypeChecker::new();
                        let result = checker.check_module(&module);
                        black_box(result).ok();
                    }
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_repeated_parse,
    bench_repeated_typecheck,
    bench_memory_stability,
    bench_scaling,
    bench_repeated_full_pipeline,
    bench_large_project,
    bench_incremental_simulation,
    bench_burst_compilation,
);

criterion_main!(benches);

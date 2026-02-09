//! Parallel compilation speedup benchmarks for Vais
//!
//! Measures parallelization speedup from Phase 2 Stage 2:
//! - Parallel parsing with rayon par_iter
//! - Parallel type checking
//! - Parallel codegen
//! - Full pipeline parallelization
//!
//! ## Test Scenarios
//!
//! - **10 modules** × 500 lines = 5K total lines
//! - **50 modules** × 500 lines = 25K total lines
//! - **100 modules** × 500 lines = 50K total lines
//!
//! ## Running
//!
//! ```bash
//! # Run all parallel benchmarks
//! cargo bench --bench parallel_bench
//!
//! # Run specific benchmark group
//! cargo bench --bench parallel_bench -- parse_speedup
//! cargo bench --bench parallel_bench -- typecheck_speedup
//! cargo bench --bench parallel_bench -- codegen_speedup
//! cargo bench --bench parallel_bench -- full_pipeline
//!
//! # Compare sequential vs parallel for 100 modules
//! cargo bench --bench parallel_bench -- 100_modules
//!
//! # Generate baseline for comparison
//! cargo bench --bench parallel_bench -- --save-baseline phase2
//! ```
//!
//! ## Output
//!
//! Criterion will generate:
//! - Console output with mean/median/stddev times
//! - HTML reports in `target/criterion/`
//! - Speedup ratios (compare sequential vs parallel timings)
//!
//! Expected speedup on multi-core systems:
//! - **10 modules**: 2-3x speedup (overhead dominates)
//! - **50 modules**: 4-6x speedup (good parallelism)
//! - **100 modules**: 6-8x speedup (near-optimal parallelism)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rayon::prelude::*;
use std::hint::black_box as std_black_box;
use vais_ast::Module;
use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

// Re-use helper from benches/lib.rs
fn generate_multi_module_project(
    num_modules: usize,
    lines_per_module: usize,
) -> Vec<(String, String)> {
    // Import helper from lib.rs
    vais_benches::utils::generate_multi_module_project(num_modules, lines_per_module)
}

/// Benchmark: Sequential parse (baseline)
fn bench_sequential_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_parse");

    for &(num_modules, lines_per_module) in &[(10, 500), (50, 500), (100, 500)] {
        let modules = generate_multi_module_project(num_modules, lines_per_module);
        let total_bytes: usize = modules.iter().map(|(_, src)| src.len()).sum();

        group.throughput(Throughput::Bytes(total_bytes as u64));
        group.bench_with_input(
            BenchmarkId::new("modules", num_modules),
            &modules,
            |b, modules| {
                b.iter(|| {
                    // Sequential iteration
                    let _results: Vec<_> = modules
                        .iter()
                        .map(|(name, src)| {
                            let tokens = tokenize(black_box(src)).expect("Lex failed");
                            let ast = parse(black_box(src)).expect("Parse failed");
                            (name.clone(), tokens, ast)
                        })
                        .collect();
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Parallel parse with rayon
fn bench_parallel_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_parse");

    for &(num_modules, lines_per_module) in &[(10, 500), (50, 500), (100, 500)] {
        let modules = generate_multi_module_project(num_modules, lines_per_module);
        let total_bytes: usize = modules.iter().map(|(_, src)| src.len()).sum();

        group.throughput(Throughput::Bytes(total_bytes as u64));
        group.bench_with_input(
            BenchmarkId::new("modules", num_modules),
            &modules,
            |b, modules| {
                b.iter(|| {
                    // Parallel iteration with rayon
                    let _results: Vec<_> = modules
                        .par_iter()
                        .map(|(name, src)| {
                            let tokens = tokenize(black_box(src)).expect("Lex failed");
                            let ast = parse(black_box(src)).expect("Parse failed");
                            (name.clone(), tokens, ast)
                        })
                        .collect();
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Sequential parse + typecheck (baseline)
fn bench_sequential_parse_typecheck(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_parse_typecheck");

    for &(num_modules, lines_per_module) in &[(10, 500), (50, 500), (100, 500)] {
        let modules = generate_multi_module_project(num_modules, lines_per_module);
        let total_bytes: usize = modules.iter().map(|(_, src)| src.len()).sum();

        group.throughput(Throughput::Bytes(total_bytes as u64));
        group.bench_with_input(
            BenchmarkId::new("modules", num_modules),
            &modules,
            |b, modules| {
                b.iter(|| {
                    // Sequential: parse then typecheck
                    let asts: Vec<_> = modules
                        .iter()
                        .map(|(name, src)| {
                            let ast = parse(black_box(src)).expect("Parse failed");
                            (name.clone(), ast)
                        })
                        .collect();

                    // Type check each module sequentially
                    for (_name, ast) in &asts {
                        let mut checker = TypeChecker::new();
                        let _ = checker.check_module(std_black_box(ast));
                    }
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Parallel parse + parallel typecheck
fn bench_parallel_parse_typecheck(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_parse_typecheck");

    for &(num_modules, lines_per_module) in &[(10, 500), (50, 500), (100, 500)] {
        let modules = generate_multi_module_project(num_modules, lines_per_module);
        let total_bytes: usize = modules.iter().map(|(_, src)| src.len()).sum();

        group.throughput(Throughput::Bytes(total_bytes as u64));
        group.bench_with_input(
            BenchmarkId::new("modules", num_modules),
            &modules,
            |b, modules| {
                b.iter(|| {
                    // Parallel parse
                    let asts: Vec<_> = modules
                        .par_iter()
                        .map(|(name, src)| {
                            let ast = parse(black_box(src)).expect("Parse failed");
                            (name.clone(), ast)
                        })
                        .collect();

                    // Parallel type check
                    let _checked: Vec<_> = asts
                        .par_iter()
                        .map(|(_name, ast)| {
                            let mut checker = TypeChecker::new();
                            checker.check_module(std_black_box(ast))
                        })
                        .collect();
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Sequential full pipeline (lex → parse → typecheck → codegen)
fn bench_sequential_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_full_pipeline");
    group.sample_size(10); // Fewer samples for expensive codegen

    for &(num_modules, lines_per_module) in &[(10, 500), (50, 500), (100, 500)] {
        let modules = generate_multi_module_project(num_modules, lines_per_module);
        let total_bytes: usize = modules.iter().map(|(_, src)| src.len()).sum();

        group.throughput(Throughput::Bytes(total_bytes as u64));
        group.bench_with_input(
            BenchmarkId::new("modules", num_modules),
            &modules,
            |b, modules| {
                b.iter(|| {
                    // Sequential: parse → typecheck → codegen
                    let asts: Vec<_> = modules
                        .iter()
                        .map(|(name, src)| {
                            let ast = parse(black_box(src)).expect("Parse failed");
                            (name.clone(), ast)
                        })
                        .collect();

                    // Sequential typecheck
                    let checked_asts: Vec<_> = asts
                        .iter()
                        .map(|(name, ast)| {
                            let mut checker = TypeChecker::new();
                            let _ = checker.check_module(std_black_box(ast));
                            (name.clone(), ast)
                        })
                        .collect();

                    // Sequential codegen
                    for (name, ast) in &checked_asts {
                        let mut codegen = CodeGenerator::new(name);
                        let _ = codegen.generate_module(std_black_box(ast));
                    }
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Parallel full pipeline (lex → parse → typecheck → codegen)
fn bench_parallel_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_full_pipeline");
    group.sample_size(10); // Fewer samples for expensive codegen

    for &(num_modules, lines_per_module) in &[(10, 500), (50, 500), (100, 500)] {
        let modules = generate_multi_module_project(num_modules, lines_per_module);
        let total_bytes: usize = modules.iter().map(|(_, src)| src.len()).sum();

        group.throughput(Throughput::Bytes(total_bytes as u64));
        group.bench_with_input(
            BenchmarkId::new("modules", num_modules),
            &modules,
            |b, modules| {
                b.iter(|| {
                    // Parallel parse
                    let asts: Vec<_> = modules
                        .par_iter()
                        .map(|(name, src)| {
                            let ast = parse(black_box(src)).expect("Parse failed");
                            (name.clone(), ast)
                        })
                        .collect();

                    // Parallel typecheck
                    let checked_asts: Vec<_> = asts
                        .par_iter()
                        .map(|(name, ast)| {
                            let mut checker = TypeChecker::new();
                            let _ = checker.check_module(std_black_box(ast));
                            (name.clone(), ast)
                        })
                        .collect();

                    // Parallel codegen
                    let _ir_results: Vec<_> = checked_asts
                        .par_iter()
                        .map(|(name, ast)| {
                            let mut codegen = CodeGenerator::new(name);
                            codegen.generate_module(std_black_box(ast))
                        })
                        .collect();
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Parse-only speedup comparison (10 modules)
fn bench_parse_speedup_10modules(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_speedup");
    let modules = generate_multi_module_project(10, 500);
    let total_bytes: usize = modules.iter().map(|(_, src)| src.len()).sum();

    group.throughput(Throughput::Bytes(total_bytes as u64));

    group.bench_with_input(
        BenchmarkId::new("sequential", "10_modules"),
        &modules,
        |b, modules| {
            b.iter(|| {
                let _results: Vec<_> = modules
                    .iter()
                    .map(|(name, src)| {
                        let ast = parse(black_box(src)).expect("Parse failed");
                        (name.clone(), ast)
                    })
                    .collect();
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("parallel", "10_modules"),
        &modules,
        |b, modules| {
            b.iter(|| {
                let _results: Vec<_> = modules
                    .par_iter()
                    .map(|(name, src)| {
                        let ast = parse(black_box(src)).expect("Parse failed");
                        (name.clone(), ast)
                    })
                    .collect();
            })
        },
    );

    group.finish();
}

/// Benchmark: Parse-only speedup comparison (50 modules)
fn bench_parse_speedup_50modules(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_speedup");
    let modules = generate_multi_module_project(50, 500);
    let total_bytes: usize = modules.iter().map(|(_, src)| src.len()).sum();

    group.throughput(Throughput::Bytes(total_bytes as u64));

    group.bench_with_input(
        BenchmarkId::new("sequential", "50_modules"),
        &modules,
        |b, modules| {
            b.iter(|| {
                let _results: Vec<_> = modules
                    .iter()
                    .map(|(name, src)| {
                        let ast = parse(black_box(src)).expect("Parse failed");
                        (name.clone(), ast)
                    })
                    .collect();
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("parallel", "50_modules"),
        &modules,
        |b, modules| {
            b.iter(|| {
                let _results: Vec<_> = modules
                    .par_iter()
                    .map(|(name, src)| {
                        let ast = parse(black_box(src)).expect("Parse failed");
                        (name.clone(), ast)
                    })
                    .collect();
            })
        },
    );

    group.finish();
}

/// Benchmark: Parse-only speedup comparison (100 modules)
fn bench_parse_speedup_100modules(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_speedup");
    let modules = generate_multi_module_project(100, 500);
    let total_bytes: usize = modules.iter().map(|(_, src)| src.len()).sum();

    group.throughput(Throughput::Bytes(total_bytes as u64));

    group.bench_with_input(
        BenchmarkId::new("sequential", "100_modules"),
        &modules,
        |b, modules| {
            b.iter(|| {
                let _results: Vec<_> = modules
                    .iter()
                    .map(|(name, src)| {
                        let ast = parse(black_box(src)).expect("Parse failed");
                        (name.clone(), ast)
                    })
                    .collect();
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("parallel", "100_modules"),
        &modules,
        |b, modules| {
            b.iter(|| {
                let _results: Vec<_> = modules
                    .par_iter()
                    .map(|(name, src)| {
                        let ast = parse(black_box(src)).expect("Parse failed");
                        (name.clone(), ast)
                    })
                    .collect();
            })
        },
    );

    group.finish();
}

/// Benchmark: Typecheck-only speedup comparison (10 modules)
fn bench_typecheck_speedup_10modules(c: &mut Criterion) {
    let mut group = c.benchmark_group("typecheck_speedup");
    let modules = generate_multi_module_project(10, 500);

    // Pre-parse all modules
    let asts: Vec<(String, Module)> = modules
        .iter()
        .map(|(name, src)| {
            let ast = parse(src).expect("Parse failed");
            (name.clone(), ast)
        })
        .collect();

    group.bench_with_input(
        BenchmarkId::new("sequential", "10_modules"),
        &asts,
        |b, asts| {
            b.iter(|| {
                for (_name, ast) in asts {
                    let mut checker = TypeChecker::new();
                    let _ = checker.check_module(std_black_box(ast));
                }
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("parallel", "10_modules"),
        &asts,
        |b, asts| {
            b.iter(|| {
                let _results: Vec<_> = asts
                    .par_iter()
                    .map(|(_name, ast)| {
                        let mut checker = TypeChecker::new();
                        checker.check_module(std_black_box(ast))
                    })
                    .collect();
            })
        },
    );

    group.finish();
}

/// Benchmark: Typecheck-only speedup comparison (50 modules)
fn bench_typecheck_speedup_50modules(c: &mut Criterion) {
    let mut group = c.benchmark_group("typecheck_speedup");
    let modules = generate_multi_module_project(50, 500);

    // Pre-parse all modules
    let asts: Vec<(String, Module)> = modules
        .iter()
        .map(|(name, src)| {
            let ast = parse(src).expect("Parse failed");
            (name.clone(), ast)
        })
        .collect();

    group.bench_with_input(
        BenchmarkId::new("sequential", "50_modules"),
        &asts,
        |b, asts| {
            b.iter(|| {
                for (_name, ast) in asts {
                    let mut checker = TypeChecker::new();
                    let _ = checker.check_module(std_black_box(ast));
                }
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("parallel", "50_modules"),
        &asts,
        |b, asts| {
            b.iter(|| {
                let _results: Vec<_> = asts
                    .par_iter()
                    .map(|(_name, ast)| {
                        let mut checker = TypeChecker::new();
                        checker.check_module(std_black_box(ast))
                    })
                    .collect();
            })
        },
    );

    group.finish();
}

/// Benchmark: Typecheck-only speedup comparison (100 modules)
fn bench_typecheck_speedup_100modules(c: &mut Criterion) {
    let mut group = c.benchmark_group("typecheck_speedup");
    let modules = generate_multi_module_project(100, 500);

    // Pre-parse all modules
    let asts: Vec<(String, Module)> = modules
        .iter()
        .map(|(name, src)| {
            let ast = parse(src).expect("Parse failed");
            (name.clone(), ast)
        })
        .collect();

    group.bench_with_input(
        BenchmarkId::new("sequential", "100_modules"),
        &asts,
        |b, asts| {
            b.iter(|| {
                for (_name, ast) in asts {
                    let mut checker = TypeChecker::new();
                    let _ = checker.check_module(std_black_box(ast));
                }
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("parallel", "100_modules"),
        &asts,
        |b, asts| {
            b.iter(|| {
                let _results: Vec<_> = asts
                    .par_iter()
                    .map(|(_name, ast)| {
                        let mut checker = TypeChecker::new();
                        checker.check_module(std_black_box(ast))
                    })
                    .collect();
            })
        },
    );

    group.finish();
}

/// Benchmark: Codegen-only speedup comparison (10 modules)
fn bench_codegen_speedup_10modules(c: &mut Criterion) {
    let mut group = c.benchmark_group("codegen_speedup");
    group.sample_size(10); // Fewer samples for expensive codegen

    let modules = generate_multi_module_project(10, 500);

    // Pre-parse and typecheck all modules
    let checked_asts: Vec<(String, Module)> = modules
        .iter()
        .map(|(name, src)| {
            let ast = parse(src).expect("Parse failed");
            let mut checker = TypeChecker::new();
            let _ = checker.check_module(&ast);
            (name.clone(), ast)
        })
        .collect();

    group.bench_with_input(
        BenchmarkId::new("sequential", "10_modules"),
        &checked_asts,
        |b, asts| {
            b.iter(|| {
                for (name, ast) in asts {
                    let mut codegen = CodeGenerator::new(name);
                    let _ = codegen.generate_module(std_black_box(ast));
                }
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("parallel", "10_modules"),
        &checked_asts,
        |b, asts| {
            b.iter(|| {
                let _results: Vec<_> = asts
                    .par_iter()
                    .map(|(name, ast)| {
                        let mut codegen = CodeGenerator::new(name);
                        codegen.generate_module(std_black_box(ast))
                    })
                    .collect();
            })
        },
    );

    group.finish();
}

/// Benchmark: Codegen-only speedup comparison (50 modules)
fn bench_codegen_speedup_50modules(c: &mut Criterion) {
    let mut group = c.benchmark_group("codegen_speedup");
    group.sample_size(10); // Fewer samples for expensive codegen

    let modules = generate_multi_module_project(50, 500);

    // Pre-parse and typecheck all modules
    let checked_asts: Vec<(String, Module)> = modules
        .iter()
        .map(|(name, src)| {
            let ast = parse(src).expect("Parse failed");
            let mut checker = TypeChecker::new();
            let _ = checker.check_module(&ast);
            (name.clone(), ast)
        })
        .collect();

    group.bench_with_input(
        BenchmarkId::new("sequential", "50_modules"),
        &checked_asts,
        |b, asts| {
            b.iter(|| {
                for (name, ast) in asts {
                    let mut codegen = CodeGenerator::new(name);
                    let _ = codegen.generate_module(std_black_box(ast));
                }
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("parallel", "50_modules"),
        &checked_asts,
        |b, asts| {
            b.iter(|| {
                let _results: Vec<_> = asts
                    .par_iter()
                    .map(|(name, ast)| {
                        let mut codegen = CodeGenerator::new(name);
                        codegen.generate_module(std_black_box(ast))
                    })
                    .collect();
            })
        },
    );

    group.finish();
}

/// Benchmark: Codegen-only speedup comparison (100 modules)
fn bench_codegen_speedup_100modules(c: &mut Criterion) {
    let mut group = c.benchmark_group("codegen_speedup");
    group.sample_size(10); // Fewer samples for expensive codegen

    let modules = generate_multi_module_project(100, 500);

    // Pre-parse and typecheck all modules
    let checked_asts: Vec<(String, Module)> = modules
        .iter()
        .map(|(name, src)| {
            let ast = parse(src).expect("Parse failed");
            let mut checker = TypeChecker::new();
            let _ = checker.check_module(&ast);
            (name.clone(), ast)
        })
        .collect();

    group.bench_with_input(
        BenchmarkId::new("sequential", "100_modules"),
        &checked_asts,
        |b, asts| {
            b.iter(|| {
                for (name, ast) in asts {
                    let mut codegen = CodeGenerator::new(name);
                    let _ = codegen.generate_module(std_black_box(ast));
                }
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("parallel", "100_modules"),
        &checked_asts,
        |b, asts| {
            b.iter(|| {
                let _results: Vec<_> = asts
                    .par_iter()
                    .map(|(name, ast)| {
                        let mut codegen = CodeGenerator::new(name);
                        codegen.generate_module(std_black_box(ast))
                    })
                    .collect();
            })
        },
    );

    group.finish();
}

criterion_group!(
    parallel_benches,
    // Individual stage benchmarks (10/50/100 modules)
    bench_sequential_parse,
    bench_parallel_parse,
    bench_sequential_parse_typecheck,
    bench_parallel_parse_typecheck,
    bench_sequential_full_pipeline,
    bench_parallel_full_pipeline,
    // Direct speedup comparison benchmarks
    bench_parse_speedup_10modules,
    bench_parse_speedup_50modules,
    bench_parse_speedup_100modules,
    bench_typecheck_speedup_10modules,
    bench_typecheck_speedup_50modules,
    bench_typecheck_speedup_100modules,
    bench_codegen_speedup_10modules,
    bench_codegen_speedup_50modules,
    bench_codegen_speedup_100modules,
);

criterion_main!(parallel_benches);

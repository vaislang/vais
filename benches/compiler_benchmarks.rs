//! Comprehensive compiler benchmarks for Vais
//!
//! This benchmark suite focuses on measuring the performance of individual
//! compiler phases and the full compilation pipeline with realistic code.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;

use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Load a fixture file from the benches/fixtures directory
fn load_fixture(name: &str) -> String {
    // Try multiple paths to support different working directories
    let paths = [
        format!("benches/fixtures/{}.vais", name),
        format!("fixtures/{}.vais", name),
    ];

    for path in &paths {
        if let Ok(content) = fs::read_to_string(path) {
            return content;
        }
    }

    // Try using CARGO_MANIFEST_DIR
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = format!("{}/fixtures/{}.vais", manifest_dir, name);
        if let Ok(content) = fs::read_to_string(&manifest_path) {
            return content;
        }
    }

    panic!("Failed to load fixture: {}", name)
}

/// Benchmark: Lexer with medium-sized files
fn bench_parse_medium_file(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_medium");

    // Use complex.vais as a medium-sized representative file
    let source = load_fixture("complex");
    let bytes = source.len() as u64;

    group.throughput(Throughput::Bytes(bytes));
    group.bench_function("complex_file", |b| {
        b.iter(|| tokenize(black_box(&source)))
    });

    group.finish();
}

/// Benchmark: Type-check medium-sized files
fn bench_typecheck_medium_file(c: &mut Criterion) {
    let mut group = c.benchmark_group("typecheck_medium");

    // Pre-parse the complex file
    let source = load_fixture("complex");
    let ast = parse(&source).expect("Parse failed");

    group.bench_function("complex_file", |b| {
        b.iter(|| {
            let mut checker = TypeChecker::new();
            checker.check_module(black_box(&ast))
        })
    });

    group.finish();
}

/// Benchmark: Full compilation pipeline with medium files
fn bench_full_compile_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("compile_pipeline_medium");

    let source = load_fixture("complex");
    let bytes = source.len() as u64;

    group.throughput(Throughput::Bytes(bytes));
    group.bench_function("complex_file", |b| {
        b.iter(|| {
            // Full compilation pipeline
            let _tokens = tokenize(black_box(&source)).expect("Tokenization failed");
            let ast = parse(black_box(&source)).expect("Parse failed");

            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("Type check failed");

            let mut codegen = CodeGenerator::new("bench");
            codegen.generate_module(&ast)
        })
    });

    group.finish();
}

/// Benchmark: Parse performance across different file sizes
fn bench_parse_file_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_scaling");

    let fixtures = [
        ("small", "fibonacci"),
        ("medium", "sort"),
        ("large", "struct_heavy"),
        ("xlarge", "complex"),
    ];

    for (size_label, fixture) in fixtures {
        let source = load_fixture(fixture);
        let bytes = source.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(
            BenchmarkId::new("parse", size_label),
            &source,
            |b, src| {
                b.iter(|| parse(black_box(src)))
            },
        );
    }

    group.finish();
}

/// Benchmark: Type checking performance across different complexities
fn bench_typecheck_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("typecheck_complexity");

    let test_cases = [
        ("simple_recursion", "fibonacci"),
        ("array_ops", "sort"),
        ("struct_operations", "struct_heavy"),
        ("mixed_features", "complex"),
    ];

    for (label, fixture) in test_cases {
        let source = load_fixture(fixture);
        let ast = parse(&source).expect("Parse failed");

        group.bench_with_input(BenchmarkId::new("check", label), &ast, |b, ast| {
            b.iter(|| {
                let mut checker = TypeChecker::new();
                checker.check_module(black_box(ast))
            })
        });
    }

    group.finish();
}

/// Benchmark: Code generation for different code patterns
fn bench_codegen_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("codegen_patterns");

    let test_cases = [
        ("recursion", "fibonacci"),
        ("iteration", "sort"),
        ("structs", "struct_heavy"),
        ("mixed", "complex"),
    ];

    for (label, fixture) in test_cases {
        let source = load_fixture(fixture);
        let ast = parse(&source).expect("Parse failed");

        // Type check first (required for codegen)
        let mut checker = TypeChecker::new();
        checker.check_module(&ast).expect("Type check failed");

        group.bench_with_input(
            BenchmarkId::new("generate", label),
            &ast,
            |b, ast| {
                b.iter(|| {
                    let mut codegen = CodeGenerator::new(label);
                    codegen.generate_module(black_box(ast))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Incremental parsing (simulating LSP scenarios)
fn bench_incremental_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental");

    let source = load_fixture("complex");

    // Simulate small edits to the file
    let variants = [
        ("original", source.clone()),
        ("small_edit", source.clone() + "\nF dummy()->i64 = 42"),
        (
            "medium_edit",
            source.clone() + "\nF helper(x: i64)->i64 = x * 2\nF helper2(x: i64)->i64 = @(x) + 1",
        ),
    ];

    for (label, variant) in variants {
        group.bench_with_input(BenchmarkId::new("parse", label), &variant, |b, src| {
            b.iter(|| parse(black_box(src)))
        });
    }

    group.finish();
}

/// Benchmark: Cold vs warm compilation (cache effects)
fn bench_compilation_warmup(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation_warmup");

    let source = load_fixture("complex");

    // Cold compilation (first time)
    group.bench_function("cold_compile", |b| {
        b.iter(|| {
            let ast = parse(black_box(&source)).expect("Parse failed");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("Type check failed");
            let mut codegen = CodeGenerator::new("bench");
            codegen.generate_module(&ast)
        })
    });

    // Warm compilation (repeated - should benefit from cache)
    // Pre-warm
    for _ in 0..3 {
        let ast = parse(&source).expect("Parse failed");
        let mut checker = TypeChecker::new();
        checker.check_module(&ast).expect("Type check failed");
        let mut codegen = CodeGenerator::new("bench");
        let _ = codegen.generate_module(&ast);
    }

    group.bench_function("warm_compile", |b| {
        b.iter(|| {
            let ast = parse(black_box(&source)).expect("Parse failed");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("Type check failed");
            let mut codegen = CodeGenerator::new("bench");
            codegen.generate_module(&ast)
        })
    });

    group.finish();
}

/// Benchmark: Error handling overhead
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    // Valid code (no errors)
    let valid_source = load_fixture("fibonacci");

    group.bench_function("valid_code", |b| {
        b.iter(|| {
            let ast = parse(black_box(&valid_source)).expect("Parse succeeded");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast)
        })
    });

    // Invalid code (syntax error)
    let invalid_syntax = "F broken(x: i64 = x +";

    group.bench_function("syntax_error", |b| {
        b.iter(|| {
            let _ = parse(black_box(invalid_syntax));
        })
    });

    group.finish();
}

/// Benchmark: Large function compilation
fn bench_large_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_functions");

    // Generate functions with different sizes
    let sizes = [10, 50, 100];

    for size in sizes {
        let source = generate_large_function(size);
        let bytes = source.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(
            BenchmarkId::new("compile", format!("{}_stmts", size)),
            &source,
            |b, src| {
                b.iter(|| {
                    let ast = parse(black_box(src)).expect("Parse failed");
                    let mut checker = TypeChecker::new();
                    checker.check_module(&ast).expect("Type check failed");
                    let mut codegen = CodeGenerator::new("bench");
                    codegen.generate_module(&ast)
                })
            },
        );
    }

    group.finish();
}

/// Generate a large function with N nested operations
fn generate_large_function(n: usize) -> String {
    let mut code = String::from("F large(x: i64)->i64 = ");

    for i in 0..n {
        if i > 0 {
            code.push_str(" + ");
        }
        code.push_str(&format!("(x * {} + {})", i % 10, i));
    }

    code.push('\n');
    code.push_str("F main()->i64 = large(42)\n");
    code
}

/// Benchmark: Module with many functions
fn bench_many_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("many_functions");

    let function_counts = [10, 50, 100, 200];

    for count in function_counts {
        let source = generate_module_with_functions(count);
        let bytes = source.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(
            BenchmarkId::new("compile", format!("{}_funcs", count)),
            &source,
            |b, src| {
                b.iter(|| {
                    let ast = parse(black_box(src)).expect("Parse failed");
                    let mut checker = TypeChecker::new();
                    checker.check_module(&ast).expect("Type check failed");
                    let mut codegen = CodeGenerator::new("bench");
                    codegen.generate_module(&ast)
                })
            },
        );
    }

    group.finish();
}

/// Generate a module with N simple functions
fn generate_module_with_functions(n: usize) -> String {
    let mut code = String::new();

    for i in 0..n {
        code.push_str(&format!(
            "F func{}(x: i64)->i64 = x * {} + {}\n",
            i,
            i % 10,
            i
        ));
    }

    // Main function that calls some of them
    code.push_str("F main()->i64 = func0(42)");
    if n > 1 {
        code.push_str(" + func1(10)");
    }
    if n > 2 {
        code.push_str(" + func2(5)");
    }
    code.push('\n');

    code
}

criterion_group!(
    compiler_benches,
    bench_parse_medium_file,
    bench_typecheck_medium_file,
    bench_full_compile_medium,
    bench_parse_file_sizes,
    bench_typecheck_complexity,
    bench_codegen_patterns,
    bench_incremental_parse,
    bench_compilation_warmup,
    bench_error_handling,
    bench_large_functions,
    bench_many_functions,
);

criterion_main!(compiler_benches);

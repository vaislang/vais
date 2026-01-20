//! Compile-time benchmarks for Vais compiler
//!
//! Measures performance of lexing, parsing, type checking, and code generation.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;

use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Load a fixture file
fn load_fixture(name: &str) -> String {
    // Try relative path first (when running from project root)
    let relative_path = format!("benches/fixtures/{}.vais", name);
    if let Ok(content) = fs::read_to_string(&relative_path) {
        return content;
    }

    // Try path relative to benches directory
    let bench_relative = format!("fixtures/{}.vais", name);
    if let Ok(content) = fs::read_to_string(&bench_relative) {
        return content;
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

/// Benchmark: Lexer throughput
fn bench_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");

    for fixture in ["fibonacci", "sort", "struct_heavy", "complex"] {
        let source = load_fixture(fixture);
        let bytes = source.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(BenchmarkId::new("tokenize", fixture), &source, |b, s| {
            b.iter(|| tokenize(black_box(s)))
        });
    }

    group.finish();
}

/// Benchmark: Parser throughput
fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");

    for fixture in ["fibonacci", "sort", "struct_heavy", "complex"] {
        let source = load_fixture(fixture);
        let bytes = source.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(BenchmarkId::new("parse", fixture), &source, |b, s| {
            b.iter(|| parse(black_box(s)))
        });
    }

    group.finish();
}

/// Benchmark: Type checker
fn bench_type_checker(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_checker");

    for fixture in ["fibonacci", "sort", "struct_heavy", "complex"] {
        let source = load_fixture(fixture);
        let ast = parse(&source).expect("Parse failed");

        group.bench_with_input(BenchmarkId::new("check", fixture), &ast, |b, ast| {
            b.iter(|| {
                let mut checker = TypeChecker::new();
                checker.check_module(black_box(ast))
            })
        });
    }

    group.finish();
}

/// Benchmark: Code generator
fn bench_codegen(c: &mut Criterion) {
    let mut group = c.benchmark_group("codegen");

    for fixture in ["fibonacci", "sort", "struct_heavy", "complex"] {
        let source = load_fixture(fixture);
        let ast = parse(&source).expect("Parse failed");

        // Type check first (required for codegen)
        let mut checker = TypeChecker::new();
        checker.check_module(&ast).expect("Type check failed");

        group.bench_with_input(BenchmarkId::new("generate", fixture), &ast, |b, ast| {
            b.iter(|| {
                let mut codegen = CodeGenerator::new(fixture);
                codegen.generate_module(black_box(ast))
            })
        });
    }

    group.finish();
}

/// Benchmark: Full compilation pipeline
fn bench_full_compile(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_compile");

    for fixture in ["fibonacci", "sort", "struct_heavy", "complex"] {
        let source = load_fixture(fixture);
        let bytes = source.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(
            BenchmarkId::new("compile", fixture),
            &source,
            |b, source| {
                b.iter(|| {
                    // Lex
                    let _tokens = tokenize(black_box(source)).expect("Lex failed");

                    // Parse
                    let ast = parse(black_box(source)).expect("Parse failed");

                    // Type check
                    let mut checker = TypeChecker::new();
                    checker.check_module(&ast).expect("Type check failed");

                    // Generate IR
                    let mut codegen = CodeGenerator::new("bench");
                    codegen.generate_module(&ast)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Lexer with varying input sizes
fn bench_lexer_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_scaling");

    // Generate code of different sizes
    let sizes = [100, 500, 1000, 5000];

    for size in sizes {
        let source = generate_code(size);
        let bytes = source.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(
            BenchmarkId::new("tokenize", format!("{}_funcs", size)),
            &source,
            |b, s| b.iter(|| tokenize(black_box(s))),
        );
    }

    group.finish();
}

/// Generate synthetic Vais code with N functions
fn generate_code(num_funcs: usize) -> String {
    let mut code = String::new();

    for i in 0..num_funcs {
        code.push_str(&format!(
            "F func{}(x: i64)->i64 = x * {} + {}\n",
            i,
            i % 10,
            i
        ));
    }

    code.push_str("F main()->i64 = func0(42)\n");
    code
}

criterion_group!(
    benches,
    bench_lexer,
    bench_parser,
    bench_type_checker,
    bench_codegen,
    bench_full_compile,
    bench_lexer_scaling,
);

criterion_main!(benches);

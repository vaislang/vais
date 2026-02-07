//! Large-scale compiler benchmarks for Vais
//!
//! Measures compilation performance with 1K ~ 50K+ line synthetic projects.
//! Covers lexer, parser, type checker, and full pipeline scaling.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;

use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Generate a realistic Vais module with structs, enums, traits, and functions.
/// Returns (source_code, approximate_line_count).
fn generate_large_project(target_lines: usize) -> String {
    let mut code = String::with_capacity(target_lines * 40);
    let mut lines = 0;

    let funcs_per_module = 20;
    let structs_per_module = 5;
    let enums_per_module = 3;
    let module_count =
        target_lines / (funcs_per_module * 3 + structs_per_module * 5 + enums_per_module * 6 + 10);
    let module_count = module_count.max(1);

    for m in 0..module_count {
        if lines >= target_lines {
            break;
        }

        // Comment header
        code.push_str(&format!(
            "# Module {} — auto-generated benchmark code\n\n",
            m
        ));
        lines += 2;

        // Structs
        for s in 0..structs_per_module {
            code.push_str(&format!("S Mod{}Struct{} {{\n", m, s));
            code.push_str(&format!("    field_a: i64,\n"));
            code.push_str(&format!("    field_b: i64,\n"));
            code.push_str(&format!("    field_c: bool\n"));
            code.push_str("}\n\n");
            lines += 6;

            if lines >= target_lines {
                break;
            }
        }

        // Enums
        for e in 0..enums_per_module {
            code.push_str(&format!("E Mod{}Result{} {{\n", m, e));
            code.push_str(&format!("    Ok(i64),\n"));
            code.push_str(&format!("    Err(i64),\n"));
            code.push_str(&format!("    None\n"));
            code.push_str("}\n\n");
            lines += 6;

            if lines >= target_lines {
                break;
            }
        }

        // Functions — varied complexity
        for f in 0..funcs_per_module {
            if lines >= target_lines {
                break;
            }

            match f % 5 {
                0 => {
                    // Simple arithmetic
                    code.push_str(&format!("F mod{}_func{}(x: i64, y: i64) -> i64 {{\n", m, f));
                    code.push_str(&format!("    a := x * {} + y\n", f + 1));
                    code.push_str(&format!("    b := a - {} * x\n", f % 7 + 1));
                    code.push_str("    R a + b\n");
                    code.push_str("}\n\n");
                    lines += 6;
                }
                1 => {
                    // Recursive
                    code.push_str(&format!("F mod{}_rec{}(n: i64) -> i64 {{\n", m, f));
                    code.push_str(&format!("    I n <= 1 {{ R {} }}\n", f % 3 + 1));
                    code.push_str(&format!("    R n * @(n - 1)\n"));
                    code.push_str("}\n\n");
                    lines += 5;
                }
                2 => {
                    // Conditional chain
                    code.push_str(&format!("F mod{}_cond{}(x: i64) -> i64 {{\n", m, f));
                    code.push_str(&format!("    I x < {} {{ R x * 2 }}\n", f * 3));
                    code.push_str(&format!("    I x < {} {{ R x + {} }}\n", f * 10, f));
                    code.push_str("    R x\n");
                    code.push_str("}\n\n");
                    lines += 6;
                }
                3 => {
                    // Loop-based
                    code.push_str(&format!("F mod{}_loop{}(n: i64) -> i64 {{\n", m, f));
                    code.push_str("    sum := mut 0\n");
                    code.push_str("    i := mut 0\n");
                    code.push_str("    L {\n");
                    code.push_str("        I i >= n { R sum }\n");
                    code.push_str(&format!("        sum = sum + i * {}\n", f % 5 + 1));
                    code.push_str("        i = i + 1\n");
                    code.push_str("    }\n");
                    code.push_str("}\n\n");
                    lines += 10;
                }
                _ => {
                    // Multi-expression
                    code.push_str(&format!(
                        "F mod{}_multi{}(a: i64, b: i64, c: i64) -> i64 {{\n",
                        m, f
                    ));
                    code.push_str(&format!("    x := a * {} + b\n", f));
                    code.push_str(&format!("    y := b * {} - c\n", f % 4 + 1));
                    code.push_str(&format!("    z := x + y + c * {}\n", f % 3));
                    code.push_str("    R x + y + z\n");
                    code.push_str("}\n\n");
                    lines += 7;
                }
            }
        }
    }

    // Entry point
    code.push_str("F main() -> i64 {\n");
    code.push_str("    result := mod0_func0(1, 2)\n");
    code.push_str("    R result\n");
    code.push_str("}\n");

    code
}

/// Benchmark: Full pipeline scaling from 1K to 50K lines
fn bench_full_pipeline_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("largescale_full_pipeline");
    group.sample_size(10); // Fewer samples for large inputs

    for &target_lines in &[1_000, 5_000, 10_000, 25_000, 50_000] {
        let source = generate_large_project(target_lines);
        let actual_lines = source.lines().count();
        let bytes = source.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(
            BenchmarkId::new("compile", format!("{}lines", actual_lines)),
            &source,
            |b, source| {
                b.iter(|| {
                    let _tokens = tokenize(black_box(source)).expect("Lex failed");
                    let ast = parse(black_box(source)).expect("Parse failed");
                    let mut checker = TypeChecker::new();
                    let _ = checker.check_module(&ast);
                    let mut codegen = CodeGenerator::new("bench_large");
                    codegen.generate_module(&ast)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Lexer scaling with large input
fn bench_lexer_large_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("largescale_lexer");
    group.sample_size(10);

    for &target_lines in &[1_000, 5_000, 10_000, 25_000, 50_000] {
        let source = generate_large_project(target_lines);
        let actual_lines = source.lines().count();
        let bytes = source.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(
            BenchmarkId::new("tokenize", format!("{}lines", actual_lines)),
            &source,
            |b, source| b.iter(|| tokenize(black_box(source))),
        );
    }

    group.finish();
}

/// Benchmark: Parser scaling with large input
fn bench_parser_large_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("largescale_parser");
    group.sample_size(10);

    for &target_lines in &[1_000, 5_000, 10_000, 25_000, 50_000] {
        let source = generate_large_project(target_lines);
        let actual_lines = source.lines().count();
        let bytes = source.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(
            BenchmarkId::new("parse", format!("{}lines", actual_lines)),
            &source,
            |b, source| b.iter(|| parse(black_box(source))),
        );
    }

    group.finish();
}

/// Benchmark: Type checker scaling with large input
fn bench_typechecker_large_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("largescale_typechecker");
    group.sample_size(10);

    for &target_lines in &[1_000, 5_000, 10_000, 25_000, 50_000] {
        let source = generate_large_project(target_lines);
        let actual_lines = source.lines().count();
        let ast = parse(&source).expect("Parse failed");

        group.bench_with_input(
            BenchmarkId::new("check", format!("{}lines", actual_lines)),
            &ast,
            |b, ast| {
                b.iter(|| {
                    let mut checker = TypeChecker::new();
                    checker.check_module(black_box(ast))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Codegen scaling with large input
fn bench_codegen_large_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("largescale_codegen");
    group.sample_size(10);

    for &target_lines in &[1_000, 5_000, 10_000, 25_000, 50_000] {
        let source = generate_large_project(target_lines);
        let actual_lines = source.lines().count();
        let ast = parse(&source).expect("Parse failed");
        let mut checker = TypeChecker::new();
        let _ = checker.check_module(&ast);

        group.bench_with_input(
            BenchmarkId::new("generate", format!("{}lines", actual_lines)),
            &ast,
            |b, ast| {
                b.iter(|| {
                    let mut codegen = CodeGenerator::new("bench_large");
                    codegen.generate_module(black_box(ast))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Incremental-like compilation (compile → modify → recompile)
fn bench_incremental_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("largescale_incremental");
    group.sample_size(10);

    let source = generate_large_project(10_000);
    let bytes = source.len() as u64;

    // Baseline: full compile
    group.throughput(Throughput::Bytes(bytes));
    group.bench_function("full_10k", |b| {
        b.iter(|| {
            let _tokens = tokenize(black_box(&source)).expect("Lex failed");
            let ast = parse(black_box(&source)).expect("Parse failed");
            let mut checker = TypeChecker::new();
            let _ = checker.check_module(&ast);
            let mut codegen = CodeGenerator::new("bench");
            codegen.generate_module(&ast)
        })
    });

    // Simulate incremental: compile twice (measuring second compile)
    group.bench_function("second_compile_10k", |b| {
        // First compile (warmup state)
        let _tokens = tokenize(&source).expect("Lex failed");
        let ast = parse(&source).expect("Parse failed");
        let mut checker = TypeChecker::new();
        let _ = checker.check_module(&ast);

        b.iter(|| {
            let _tokens = tokenize(black_box(&source)).expect("Lex failed");
            let ast = parse(black_box(&source)).expect("Parse failed");
            let mut checker = TypeChecker::new();
            let _ = checker.check_module(&ast);
            let mut codegen = CodeGenerator::new("bench");
            codegen.generate_module(&ast)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_full_pipeline_scaling,
    bench_lexer_large_scaling,
    bench_parser_large_scaling,
    bench_typechecker_large_scaling,
    bench_codegen_large_scaling,
    bench_incremental_simulation,
);

criterion_main!(benches);

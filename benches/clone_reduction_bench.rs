//! Clone Reduction Optimization Benchmarks
//!
//! Measures the performance impact of clone reduction in Phase 2 Stage 1:
//! - vais-codegen: Rc<Function>/Rc<Struct> introduction (~40-50 clones removed)
//! - vais-types: ~16 clones removed
//!
//! This benchmark suite focuses on:
//! 1. Type checker throughput on large files (50K lines)
//! 2. Codegen throughput on the same files
//! 3. Generic instantiation performance (Rc optimization impact)
//! 4. Full compilation pipeline (lex→parse→typecheck→codegen)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vais_codegen::CodeGenerator;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Generate synthetic code with heavy generic usage.
/// This tests the impact of Rc<Function>/Rc<Struct> on generic instantiation.
///
/// # Arguments
/// * `num_generics` - Number of generic functions/structs to generate and instantiate
///
/// # Returns
/// Valid Vais source code with multiple generic instantiations
fn generate_generic_heavy_code(num_generics: usize) -> String {
    let mut code = String::with_capacity(num_generics * 500);

    // Generate generic struct definitions
    for i in 0..num_generics {
        code.push_str(&format!(
            "S Container{}<T> {{\n    value: T,\n    count: i64\n}}\n\n",
            i
        ));
    }

    // Generate generic function definitions
    for i in 0..num_generics {
        code.push_str(&format!(
            "F process{}<T>(x: T) -> T {{\n    R x\n}}\n\n",
            i
        ));
    }

    // Generate trait definitions
    for i in 0..num_generics / 2 {
        code.push_str(&format!(
            "W Processor{} {{\n    F compute(x: i64) -> i64\n}}\n\n",
            i
        ));
    }

    // Generate generic functions with multiple type parameters
    for i in 0..num_generics / 3 {
        code.push_str(&format!(
            "F combine{}<T, U>(a: T, b: U) -> i64 {{\n    R {}\n}}\n\n",
            i, i * 100
        ));
    }

    // Generate impl blocks for generic structs
    for i in 0..num_generics / 4 {
        code.push_str(&format!(
            "X Container{}<i64> {{\n    F new(v: i64) -> Container{}<i64> {{\n        R Container{} {{ value: v, count: 1 }}\n    }}\n}}\n\n",
            i, i, i
        ));
    }

    // Generate code that instantiates all these generics
    code.push_str("F main() -> i64 {\n");
    code.push_str("    sum := mut 0\n");

    // Instantiate generic structs
    for i in 0..(num_generics.min(20)) {
        code.push_str(&format!(
            "    c{} := Container{} {{ value: {}, count: {} }}\n",
            i,
            i,
            i * 10,
            i
        ));
        code.push_str(&format!("    sum = sum + c{}.value + c{}.count\n", i, i));
    }

    // Call generic functions
    for i in 0..(num_generics.min(20)) {
        code.push_str(&format!("    r{} := process{}({})\n", i, i, i * 5));
        code.push_str(&format!("    sum = sum + r{}\n", i));
    }

    // Call multi-parameter generic functions
    for i in 0..(num_generics / 3).min(10) {
        code.push_str(&format!(
            "    x{} := combine{}({}, {})\n",
            i,
            i,
            i * 10,
            i * 20
        ));
        code.push_str(&format!("    sum = sum + x{}\n", i));
    }

    code.push_str("    R sum\n");
    code.push_str("}\n");

    code
}

/// Generate a large file targeting a specific line count.
/// Uses diverse constructs: functions, structs, enums, loops, conditionals, matches.
fn generate_large_file(target_lines: usize) -> String {
    let mut code = String::with_capacity(target_lines * 50);
    let mut lines = 0;

    // Calculate distribution
    let structs_count = target_lines / 100;
    let enums_count = target_lines / 150;
    let funcs_count = target_lines / 10;

    // Generate structs
    for s in 0..structs_count {
        if lines >= target_lines {
            break;
        }
        code.push_str(&format!(
            "S Point{} {{\n    x: i64,\n    y: i64,\n    z: i64,\n    w: i64\n}}\n\n",
            s
        ));
        lines += 7;
    }

    // Generate enums
    for e in 0..enums_count {
        if lines >= target_lines {
            break;
        }
        code.push_str(&format!(
            "E Result{} {{\n    Ok(i64),\n    Err(i64),\n    Pending,\n    Cancelled\n}}\n\n",
            e
        ));
        lines += 7;
    }

    // Generate functions with diverse patterns
    for f in 0..funcs_count {
        if lines >= target_lines {
            break;
        }

        match f % 10 {
            0 => {
                // Simple arithmetic
                code.push_str(&format!("F compute_{}(x: i64, y: i64) -> i64 {{\n", f));
                code.push_str(&format!("    a := x * {} + y\n", f + 1));
                code.push_str(&format!("    b := a - {} * x\n", f % 7 + 1));
                code.push_str(&format!("    c := b + {} * y\n", f % 5 + 2));
                code.push_str("    R a + b + c\n");
                code.push_str("}\n\n");
                lines += 7;
            }
            1 => {
                // Recursive function
                code.push_str(&format!("F recursive_{}(n: i64) -> i64 {{\n", f));
                code.push_str("    I n <= 1 {\n");
                code.push_str("        R 1\n");
                code.push_str("    }\n");
                code.push_str(&format!("    R n * @(n - {})\n", f % 3 + 1));
                code.push_str("}\n\n");
                lines += 7;
            }
            2 => {
                // Conditional chain
                code.push_str(&format!("F conditional_{}(x: i64) -> i64 {{\n", f));
                code.push_str(&format!("    I x < {} {{\n", f * 5));
                code.push_str(&format!("        R x * {}\n", f % 4 + 2));
                code.push_str(&format!("    }} E I x < {} {{\n", f * 10));
                code.push_str(&format!("        R x + {}\n", f));
                code.push_str("    } E {\n");
                code.push_str("        R x\n");
                code.push_str("    }\n");
                code.push_str("}\n\n");
                lines += 10;
            }
            3 => {
                // Loop with accumulator
                code.push_str(&format!("F loop_{}(n: i64) -> i64 {{\n", f));
                code.push_str("    sum := mut 0\n");
                code.push_str("    i := mut 0\n");
                code.push_str("    L {\n");
                code.push_str("        I i >= n {\n");
                code.push_str("            B\n");
                code.push_str("        }\n");
                code.push_str(&format!("        sum = sum + i * {}\n", f % 6 + 1));
                code.push_str("        i = i + 1\n");
                code.push_str("    }\n");
                code.push_str("    R sum\n");
                code.push_str("}\n\n");
                lines += 13;
            }
            4 => {
                // Match expression
                code.push_str(&format!("F match_{}(x: i64) -> i64 {{\n", f));
                code.push_str("    M x {\n");
                code.push_str(&format!("        {} => x * 2,\n", f % 10));
                code.push_str(&format!("        {} => x * 3,\n", (f + 1) % 10));
                code.push_str(&format!("        {} => x * 4,\n", (f + 2) % 10));
                code.push_str("        _ => x\n");
                code.push_str("    }\n");
                code.push_str("}\n\n");
                lines += 8;
            }
            5 => {
                // Ternary operator chain
                code.push_str(&format!("F ternary_{}(a: i64, b: i64) -> i64 {{\n", f));
                code.push_str(&format!(
                    "    x := a > b ? a * {} : b * {}\n",
                    f + 1,
                    f + 2
                ));
                code.push_str(&format!(
                    "    y := x > {} ? x - {} : x + {}\n",
                    f * 10,
                    f,
                    f * 2
                ));
                code.push_str("    R x + y\n");
                code.push_str("}\n\n");
                lines += 6;
            }
            6 => {
                // Struct construction and field access
                code.push_str(&format!("F struct_{}(v: i64) -> i64 {{\n", f));
                let struct_idx = f % structs_count;
                code.push_str(&format!(
                    "    p := Point{} {{ x: v, y: v * {}, z: v + {}, w: v - {} }}\n",
                    struct_idx,
                    f % 5 + 1,
                    f,
                    f % 3
                ));
                code.push_str("    R p.x + p.y + p.z + p.w\n");
                code.push_str("}\n\n");
                lines += 5;
            }
            7 => {
                // Multi-variable computation
                code.push_str(&format!(
                    "F compute_multi_{}(a: i64, b: i64, c: i64) -> i64 {{\n",
                    f
                ));
                code.push_str(&format!("    x := a * {} + b * {}\n", f + 1, f % 3 + 1));
                code.push_str(&format!("    y := b * {} - c * {}\n", f % 4 + 1, f % 2 + 1));
                code.push_str(&format!("    z := c * {} + x\n", f % 5 + 1));
                code.push_str("    w := x + y - z\n");
                code.push_str("    R (x + y + z + w) / 4\n");
                code.push_str("}\n\n");
                lines += 8;
            }
            8 => {
                // Nested conditionals
                code.push_str(&format!("F nested_{}(x: i64, y: i64) -> i64 {{\n", f));
                code.push_str("    I x > 0 {\n");
                code.push_str("        I y > 0 {\n");
                code.push_str(&format!("            R x + y + {}\n", f));
                code.push_str("        } E {\n");
                code.push_str(&format!("            R x - y + {}\n", f * 2));
                code.push_str("        }\n");
                code.push_str("    } E {\n");
                code.push_str("        I y > 0 {\n");
                code.push_str(&format!("            R y - x + {}\n", f * 3));
                code.push_str("        } E {\n");
                code.push_str(&format!("            R {}\n", f * 4));
                code.push_str("        }\n");
                code.push_str("    }\n");
                code.push_str("}\n\n");
                lines += 15;
            }
            _ => {
                // Complex expression
                code.push_str(&format!("F complex_{}(n: i64) -> i64 {{\n", f));
                code.push_str(&format!(
                    "    a := n > {} ? @(n - {}) : n * {}\n",
                    f * 10,
                    f % 5 + 1,
                    f % 3 + 2
                ));
                code.push_str(&format!("    b := a + {} * n\n", f % 7 + 1));
                code.push_str("    M b % 3 {\n");
                code.push_str("        0 => b * 2,\n");
                code.push_str("        1 => b + 10,\n");
                code.push_str("        _ => b - 5\n");
                code.push_str("    }\n");
                code.push_str("}\n\n");
                lines += 10;
            }
        }
    }

    // Main entry point
    code.push_str("F main() -> i64 {\n");
    code.push_str("    result := compute_0(42, 13)\n");
    code.push_str("    R result\n");
    code.push_str("}\n");

    code
}

/// Benchmark: Type checker throughput on large files
///
/// Measures the impact of clone reduction in vais-types (~16 clones removed).
/// Tests with files of increasing size to measure scaling behavior.
fn bench_type_checker_large_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone_reduction/type_checker");

    // Test different file sizes
    let sizes = [10_000, 25_000, 50_000];

    for size in sizes {
        let source = generate_large_file(size);
        let ast = parse(&source).expect("Parse failed");
        let bytes = source.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(
            BenchmarkId::new("large_file", format!("{}lines", size)),
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

/// Benchmark: Codegen throughput on large files
///
/// Measures the impact of clone reduction in vais-codegen (~40-50 clones removed).
/// Tests Rc<Function>/Rc<Struct> optimization impact on IR generation.
fn bench_codegen_large_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone_reduction/codegen");

    // Test different file sizes
    let sizes = [10_000, 25_000, 50_000];

    for size in sizes {
        let source = generate_large_file(size);
        let ast = parse(&source).expect("Parse failed");
        let bytes = source.len() as u64;

        // Type check first (required for codegen)
        let mut checker = TypeChecker::new();
        checker.check_module(&ast).expect("Type check failed");

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(
            BenchmarkId::new("large_file", format!("{}lines", size)),
            &ast,
            |b, ast| {
                b.iter(|| {
                    let mut codegen = CodeGenerator::new("clone_bench");
                    codegen.generate_module(black_box(ast))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Generic instantiation performance
///
/// Measures the impact of Rc<Function>/Rc<Struct> on code with heavy generic usage.
/// This is where clone reduction has the most impact, as generics are frequently
/// instantiated and cloned during type checking and code generation.
fn bench_generic_instantiation(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone_reduction/generic_instantiation");

    // Test different numbers of generic definitions
    let generic_counts = [50, 100, 200];

    for count in generic_counts {
        let source = generate_generic_heavy_code(count);
        let ast = parse(&source).expect("Parse failed");

        // Benchmark type checking
        group.bench_with_input(
            BenchmarkId::new("typecheck", format!("{}generics", count)),
            &ast,
            |b, ast| {
                b.iter(|| {
                    let mut checker = TypeChecker::new();
                    checker.check_module(black_box(ast))
                })
            },
        );

        // Benchmark codegen
        let mut checker = TypeChecker::new();
        checker.check_module(&ast).expect("Type check failed");

        group.bench_with_input(
            BenchmarkId::new("codegen", format!("{}generics", count)),
            &ast,
            |b, ast| {
                b.iter(|| {
                    let mut codegen = CodeGenerator::new("generic_bench");
                    codegen.generate_module(black_box(ast))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Full compilation pipeline
///
/// Measures end-to-end compilation time (lex→parse→typecheck→codegen).
/// This benchmark captures the cumulative impact of all clone reductions.
fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone_reduction/full_pipeline");

    // Test with different code patterns
    let test_cases = [
        ("large_file_10k", generate_large_file(10_000)),
        ("large_file_50k", generate_large_file(50_000)),
        ("generic_heavy_100", generate_generic_heavy_code(100)),
        ("generic_heavy_200", generate_generic_heavy_code(200)),
    ];

    for (name, source) in test_cases {
        let bytes = source.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(BenchmarkId::new("compile", name), &source, |b, source| {
            b.iter(|| {
                // Parse
                let ast = parse(black_box(source)).expect("Parse failed");

                // Type check
                let mut checker = TypeChecker::new();
                checker.check_module(&ast).expect("Type check failed");

                // Generate IR
                let mut codegen = CodeGenerator::new("pipeline_bench");
                codegen.generate_module(&ast)
            })
        });
    }

    group.finish();
}

/// Benchmark: Struct-heavy workloads
///
/// Tests code with many struct definitions and field accesses.
/// Rc<Struct> optimization should show benefits here.
fn bench_struct_heavy(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone_reduction/struct_heavy");

    // Generate code with many structs
    let num_structs = 500;
    let mut code = String::new();

    // Define structs
    for s in 0..num_structs {
        code.push_str(&format!(
            "S Data{} {{\n    a: i64,\n    b: i64,\n    c: i64,\n    d: i64,\n    e: i64\n}}\n\n",
            s
        ));
    }

    // Functions that use structs
    for f in 0..100 {
        let struct_idx = f % num_structs;
        code.push_str(&format!("F use_struct_{}(v: i64) -> i64 {{\n", f));
        code.push_str(&format!(
            "    s := Data{} {{ a: v, b: v+1, c: v+2, d: v+3, e: v+4 }}\n",
            struct_idx
        ));
        code.push_str("    R s.a + s.b + s.c + s.d + s.e\n");
        code.push_str("}\n\n");
    }

    code.push_str("F main() -> i64 {\n");
    code.push_str("    R use_struct_0(42)\n");
    code.push_str("}\n");

    let ast = parse(&code).expect("Parse failed");

    // Benchmark type checking
    group.bench_function("typecheck", |b| {
        b.iter(|| {
            let mut checker = TypeChecker::new();
            checker.check_module(black_box(&ast))
        })
    });

    // Benchmark codegen
    let mut checker = TypeChecker::new();
    checker.check_module(&ast).expect("Type check failed");

    group.bench_function("codegen", |b| {
        b.iter(|| {
            let mut codegen = CodeGenerator::new("struct_bench");
            codegen.generate_module(black_box(&ast))
        })
    });

    group.finish();
}

/// Benchmark: Function-heavy workloads
///
/// Tests code with many function definitions and calls.
/// Rc<Function> optimization should show benefits here.
fn bench_function_heavy(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone_reduction/function_heavy");

    // Generate code with many functions
    let num_funcs = 1000;
    let mut code = String::new();

    // Define functions
    for f in 0..num_funcs {
        code.push_str(&format!(
            "F func_{}(x: i64, y: i64) -> i64 {{\n    R x * {} + y * {}\n}}\n\n",
            f,
            f % 10 + 1,
            f % 7 + 1
        ));
    }

    // Function that calls many others
    code.push_str("F call_all(n: i64) -> i64 {\n");
    code.push_str("    sum := mut 0\n");
    for f in 0..20 {
        code.push_str(&format!("    sum = sum + func_{}(n, n+{})\n", f, f));
    }
    code.push_str("    R sum\n");
    code.push_str("}\n\n");

    code.push_str("F main() -> i64 {\n");
    code.push_str("    R call_all(42)\n");
    code.push_str("}\n");

    let ast = parse(&code).expect("Parse failed");

    // Benchmark type checking
    group.bench_function("typecheck", |b| {
        b.iter(|| {
            let mut checker = TypeChecker::new();
            checker.check_module(black_box(&ast))
        })
    });

    // Benchmark codegen
    let mut checker = TypeChecker::new();
    checker.check_module(&ast).expect("Type check failed");

    group.bench_function("codegen", |b| {
        b.iter(|| {
            let mut codegen = CodeGenerator::new("func_bench");
            codegen.generate_module(black_box(&ast))
        })
    });

    group.finish();
}

criterion_group!(
    clone_reduction_benches,
    bench_type_checker_large_files,
    bench_codegen_large_files,
    bench_generic_instantiation,
    bench_full_pipeline,
    bench_struct_heavy,
    bench_function_heavy,
);

criterion_main!(clone_reduction_benches);

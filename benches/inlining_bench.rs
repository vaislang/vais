//! Inlining optimization benchmarks
//!
//! Measures the performance impact of the inlining optimization pass,
//! including the expanded threshold (50 instructions) and call-frequency
//! based prioritization.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vais_codegen::optimize::{optimize_ir_with_pgo, OptLevel, PgoMode};

/// Generate synthetic LLVM IR with small helper functions to test inlining
fn generate_ir_with_small_functions(num_helpers: usize) -> String {
    let mut ir = String::new();

    // Preamble
    ir.push_str("; ModuleID = 'inlining_bench'\n");
    ir.push_str("target triple = \"x86_64-unknown-linux-gnu\"\n\n");
    ir.push_str("declare i64 @print_i64(i64)\n\n");

    // Generate small helper functions (2-5 instructions each)
    for i in 0..num_helpers {
        ir.push_str(&format!(
            "define i64 @helper_{}(i64 %x, i64 %y) {{\n\
             entry:\n\
             \t%sum = add i64 %x, %y\n\
             \t%result = mul i64 %sum, 2\n\
             \tret i64 %result\n\
             }}\n\n",
            i
        ));
    }

    // Generate main function that calls all helpers
    ir.push_str("define i64 @main() {\nentry:\n");
    for i in 0..num_helpers {
        ir.push_str(&format!(
            "\t%r{} = call i64 @helper_{}(i64 {}, i64 {})\n",
            i,
            i,
            i,
            i + 1
        ));
    }
    // Sum results
    if num_helpers > 0 {
        ir.push_str(&format!("\tret i64 %r{}\n", num_helpers - 1));
    } else {
        ir.push_str("\tret i64 0\n");
    }
    ir.push_str("}\n");

    ir
}

/// Generate IR with hot functions (called many times) to test frequency-based inlining
fn generate_ir_with_hot_functions() -> String {
    let mut ir = String::new();

    ir.push_str("; ModuleID = 'hot_function_bench'\n");
    ir.push_str("target triple = \"x86_64-unknown-linux-gnu\"\n\n");
    ir.push_str("declare i64 @print_i64(i64)\n\n");

    // Hot function - called many times
    ir.push_str(
        "define i64 @hot_add(i64 %x, i64 %y) {\n\
         entry:\n\
         \t%sum = add i64 %x, %y\n\
         \tret i64 %sum\n\
         }\n\n",
    );

    // Cold function - called once
    ir.push_str(
        "define i64 @cold_compute(i64 %a, i64 %b, i64 %c) {\n\
         entry:\n\
         \t%t1 = add i64 %a, %b\n\
         \t%t2 = mul i64 %t1, %c\n\
         \t%t3 = sub i64 %t2, %a\n\
         \t%t4 = add i64 %t3, 1\n\
         \tret i64 %t4\n\
         }\n\n",
    );

    // Main function with many calls to hot function
    ir.push_str("define i64 @main() {\nentry:\n");
    for i in 0..50 {
        ir.push_str(&format!(
            "\t%hot_{} = call i64 @hot_add(i64 {}, i64 {})\n",
            i,
            i,
            i + 1
        ));
    }
    ir.push_str("\t%cold_0 = call i64 @cold_compute(i64 1, i64 2, i64 3)\n");
    ir.push_str("\tret i64 %hot_49\n");
    ir.push_str("}\n");

    ir
}

/// Generate IR with medium-sized functions (10-50 instructions)
fn generate_ir_with_medium_functions() -> String {
    let mut ir = String::new();

    ir.push_str("; ModuleID = 'medium_func_bench'\n");
    ir.push_str("target triple = \"x86_64-unknown-linux-gnu\"\n\n");

    // Medium function (~20 instructions, no side effects)
    ir.push_str("define i64 @compute(i64 %x, i64 %y) {\nentry:\n");
    for i in 0..18 {
        if i == 0 {
            ir.push_str(&format!("\t%t{} = add i64 %x, %y\n", i));
        } else if i % 3 == 0 {
            ir.push_str(&format!("\t%t{} = mul i64 %t{}, 2\n", i, i - 1));
        } else if i % 3 == 1 {
            ir.push_str(&format!("\t%t{} = add i64 %t{}, %x\n", i, i - 1));
        } else {
            ir.push_str(&format!("\t%t{} = sub i64 %t{}, 1\n", i, i - 1));
        }
    }
    ir.push_str("\tret i64 %t17\n}\n\n");

    // Main function
    ir.push_str("define i64 @main() {\nentry:\n");
    for i in 0..10 {
        ir.push_str(&format!(
            "\t%r{} = call i64 @compute(i64 {}, i64 {})\n",
            i,
            i,
            i + 1
        ));
    }
    ir.push_str("\tret i64 %r9\n}\n");

    ir
}

/// Benchmark: Inlining with varying number of small functions
fn bench_inline_small_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("inline_small_functions");

    for num_helpers in [5, 10, 20, 50] {
        let ir = generate_ir_with_small_functions(num_helpers);
        group.bench_function(format!("{}_helpers", num_helpers), |b| {
            b.iter(|| optimize_ir_with_pgo(black_box(&ir), OptLevel::O3, &PgoMode::None))
        });
    }

    group.finish();
}

/// Benchmark: Call-frequency-based inlining priority
fn bench_inline_hot_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("inline_hot_functions");

    let ir = generate_ir_with_hot_functions();

    // Compare O2 (no inlining) vs O3 (with inlining)
    group.bench_function("O2_no_inline", |b| {
        b.iter(|| optimize_ir_with_pgo(black_box(&ir), OptLevel::O2, &PgoMode::None))
    });

    group.bench_function("O3_with_inline", |b| {
        b.iter(|| optimize_ir_with_pgo(black_box(&ir), OptLevel::O3, &PgoMode::None))
    });

    group.finish();
}

/// Benchmark: Medium function inlining (expanded 50-instruction threshold)
fn bench_inline_medium_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("inline_medium_functions");

    let ir = generate_ir_with_medium_functions();

    group.bench_function("O3_medium_inline", |b| {
        b.iter(|| optimize_ir_with_pgo(black_box(&ir), OptLevel::O3, &PgoMode::None))
    });

    group.finish();
}

/// Benchmark: Full optimization pipeline at different levels
fn bench_optimization_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_levels");

    let ir = generate_ir_with_small_functions(20);

    for (name, level) in [
        ("O0", OptLevel::O0),
        ("O1", OptLevel::O1),
        ("O2", OptLevel::O2),
        ("O3", OptLevel::O3),
    ] {
        group.bench_function(name, |b| {
            b.iter(|| optimize_ir_with_pgo(black_box(&ir), level, &PgoMode::None))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_inline_small_functions,
    bench_inline_hot_functions,
    bench_inline_medium_functions,
    bench_optimization_levels,
);

criterion_main!(benches);

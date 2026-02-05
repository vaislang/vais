//! Performance Benchmark Suite (Phase 30 Stage 6)
//!
//! Comprehensive benchmarks covering:
//! 1. Token count comparison (Vais vs Rust vs C)
//! 2. Runtime algorithm benchmarks (matrix_mul, quicksort, binary_tree)
//! 3. Compile speed scaling (100/1K/10K LOC)
//! 4. Memory usage estimation

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

// ============================================================
// 1. Token Count Comparison: Vais vs Rust vs C
// ============================================================

/// Count tokens in source code using the Vais lexer
fn count_vais_tokens(source: &str) -> usize {
    match tokenize(source) {
        Ok(tokens) => tokens.len(),
        Err(_) => 0,
    }
}

/// Approximate token count by whitespace-separated words and operators
fn approximate_token_count(source: &str) -> usize {
    let mut count = 0;
    let mut in_string = false;
    let mut prev_char = ' ';

    for ch in source.chars() {
        if ch == '"' && prev_char != '\\' {
            in_string = !in_string;
            count += 1;
            prev_char = ch;
            continue;
        }
        if in_string {
            prev_char = ch;
            continue;
        }
        if ch.is_alphanumeric() || ch == '_' {
            if !prev_char.is_alphanumeric() && prev_char != '_' {
                count += 1;
            }
        } else if !ch.is_whitespace() && ch != '\n' {
            count += 1;
        }
        prev_char = ch;
    }
    count
}

// Fibonacci implementations in each language (as strings for token counting)
const FIBONACCI_VAIS: &str = r#"F fib(n: i64) -> i64 = I n <= 1 ? n : @(n - 1) + @(n - 2)
F main() -> i64 = fib(35)
"#;

const FIBONACCI_RUST: &str = r#"fn fib(n: i64) -> i64 {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}
fn main() {
    println!("{}", fib(35));
}
"#;

const FIBONACCI_C: &str = r#"#include <stdio.h>
long fib(long n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}
int main() {
    printf("%ld\n", fib(35));
    return 0;
}
"#;

// Quicksort implementations
const QUICKSORT_VAIS: &str = r#"F partition(arr: [i64], lo: i64, hi: i64) -> i64 {
    pivot := arr[hi]
    i := mut lo - 1
    j := mut lo
    L j < hi {
        I arr[j] <= pivot {
            i = i + 1
            arr[i] = arr[j]
        }
        j = j + 1
    }
    arr[i + 1] = arr[hi]
    R i + 1
}
F qsort(arr: [i64], lo: i64, hi: i64) -> i64 {
    I lo < hi {
        p := partition(arr, lo, hi)
        qsort(arr, lo, p - 1)
        qsort(arr, p + 1, hi)
    }
    R 0
}
F main() -> i64 = 0
"#;

const QUICKSORT_RUST: &str = r#"fn partition(arr: &mut [i64], lo: usize, hi: usize) -> usize {
    let pivot = arr[hi];
    let mut i = lo;
    for j in lo..hi {
        if arr[j] <= pivot {
            arr.swap(i, j);
            i += 1;
        }
    }
    arr.swap(i, hi);
    i
}
fn quicksort(arr: &mut [i64], lo: usize, hi: usize) {
    if lo < hi {
        let p = partition(arr, lo, hi);
        if p > 0 { quicksort(arr, lo, p - 1); }
        quicksort(arr, p + 1, hi);
    }
}
fn main() {
    let mut arr = vec![5, 3, 8, 4, 2, 7, 1, 6];
    let len = arr.len();
    quicksort(&mut arr, 0, len - 1);
}
"#;

const QUICKSORT_C: &str = r#"#include <stdio.h>
void swap(long *a, long *b) { long t = *a; *a = *b; *b = t; }
long partition(long arr[], long lo, long hi) {
    long pivot = arr[hi];
    long i = lo - 1;
    for (long j = lo; j < hi; j++) {
        if (arr[j] <= pivot) {
            i++;
            swap(&arr[i], &arr[j]);
        }
    }
    swap(&arr[i + 1], &arr[hi]);
    return i + 1;
}
void quicksort(long arr[], long lo, long hi) {
    if (lo < hi) {
        long p = partition(arr, lo, hi);
        quicksort(arr, lo, p - 1);
        quicksort(arr, p + 1, hi);
    }
}
int main() {
    long arr[] = {5, 3, 8, 4, 2, 7, 1, 6};
    quicksort(arr, 0, 7);
    return 0;
}
"#;

fn bench_token_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_count");

    // Fibonacci token count comparison
    let vais_fib_tokens = count_vais_tokens(FIBONACCI_VAIS);
    let rust_fib_tokens = approximate_token_count(FIBONACCI_RUST);
    let c_fib_tokens = approximate_token_count(FIBONACCI_C);

    // Print the comparison
    eprintln!("\n=== Token Count Comparison (Fibonacci) ===");
    eprintln!(
        "Vais:  {} tokens ({} chars)",
        vais_fib_tokens,
        FIBONACCI_VAIS.len()
    );
    eprintln!(
        "Rust:  {} tokens ({} chars)",
        rust_fib_tokens,
        FIBONACCI_RUST.len()
    );
    eprintln!(
        "C:     {} tokens ({} chars)",
        c_fib_tokens,
        FIBONACCI_C.len()
    );
    eprintln!(
        "Vais/Rust ratio: {:.2}x",
        vais_fib_tokens as f64 / rust_fib_tokens as f64
    );
    eprintln!(
        "Vais/C ratio:    {:.2}x",
        vais_fib_tokens as f64 / c_fib_tokens as f64
    );

    // Quicksort token count
    let vais_qs_tokens = count_vais_tokens(QUICKSORT_VAIS);
    let rust_qs_tokens = approximate_token_count(QUICKSORT_RUST);
    let c_qs_tokens = approximate_token_count(QUICKSORT_C);

    eprintln!("\n=== Token Count Comparison (Quicksort) ===");
    eprintln!(
        "Vais:  {} tokens ({} chars)",
        vais_qs_tokens,
        QUICKSORT_VAIS.len()
    );
    eprintln!(
        "Rust:  {} tokens ({} chars)",
        rust_qs_tokens,
        QUICKSORT_RUST.len()
    );
    eprintln!(
        "C:     {} tokens ({} chars)",
        c_qs_tokens,
        QUICKSORT_C.len()
    );
    eprintln!(
        "Vais/Rust ratio: {:.2}x",
        vais_qs_tokens as f64 / rust_qs_tokens as f64
    );
    eprintln!(
        "Vais/C ratio:    {:.2}x",
        vais_qs_tokens as f64 / c_qs_tokens as f64
    );

    group.bench_function("vais_fibonacci", |b| {
        b.iter(|| count_vais_tokens(black_box(FIBONACCI_VAIS)))
    });
    group.bench_function("vais_quicksort", |b| {
        b.iter(|| count_vais_tokens(black_box(QUICKSORT_VAIS)))
    });

    group.finish();
}

// ============================================================
// 2. Runtime Algorithm Benchmarks (native Rust for comparison)
// ============================================================

fn rust_matrix_mul(n: usize) -> Vec<Vec<i64>> {
    let a: Vec<Vec<i64>> = (0..n)
        .map(|i| (0..n).map(|j| (i * n + j) as i64).collect())
        .collect();
    let b: Vec<Vec<i64>> = (0..n)
        .map(|i| (0..n).map(|j| ((i + j) % n) as i64).collect())
        .collect();
    let mut c = vec![vec![0i64; n]; n];
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                c[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    c
}

fn rust_quicksort(arr: &mut [i64]) {
    if arr.len() <= 1 {
        return;
    }
    let pivot = arr[arr.len() - 1];
    let mut i = 0;
    for j in 0..arr.len() - 1 {
        if arr[j] <= pivot {
            arr.swap(i, j);
            i += 1;
        }
    }
    let len = arr.len();
    arr.swap(i, len - 1);
    let (left, right) = arr.split_at_mut(i);
    rust_quicksort(left);
    if right.len() > 1 {
        rust_quicksort(&mut right[1..]);
    }
}

struct BinaryTree {
    left: Option<Box<BinaryTree>>,
    right: Option<Box<BinaryTree>>,
}

fn make_tree(depth: u32) -> Option<Box<BinaryTree>> {
    if depth == 0 {
        return None;
    }
    Some(Box::new(BinaryTree {
        left: make_tree(depth - 1),
        right: make_tree(depth - 1),
    }))
}

fn count_nodes(tree: &Option<Box<BinaryTree>>) -> u64 {
    match tree {
        None => 0,
        Some(node) => 1 + count_nodes(&node.left) + count_nodes(&node.right),
    }
}

fn rust_fibonacci(n: i64) -> i64 {
    if n <= 1 {
        n
    } else {
        rust_fibonacci(n - 1) + rust_fibonacci(n - 2)
    }
}

fn bench_runtime_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("runtime_algorithms");

    // Fibonacci
    for n in [20, 30, 35] {
        group.bench_with_input(BenchmarkId::new("fibonacci", n), &n, |b, &n| {
            b.iter(|| rust_fibonacci(black_box(n)))
        });
    }

    // Matrix multiplication
    for size in [8, 16, 32, 64] {
        group.bench_with_input(BenchmarkId::new("matrix_mul", size), &size, |b, &n| {
            b.iter(|| rust_matrix_mul(black_box(n)))
        });
    }

    // Quicksort
    for size in [100, 1000, 10000] {
        let mut data: Vec<i64> = (0..size).rev().collect();
        group.bench_with_input(BenchmarkId::new("quicksort", size), &data, |b, d| {
            let mut arr = d.clone();
            b.iter(|| {
                arr.copy_from_slice(d);
                rust_quicksort(black_box(&mut arr))
            })
        });
    }

    // Binary tree
    for depth in [10, 15, 18] {
        group.bench_with_input(BenchmarkId::new("binary_tree", depth), &depth, |b, &d| {
            b.iter(|| {
                let tree = make_tree(black_box(d));
                count_nodes(&tree)
            })
        });
    }

    group.finish();
}

// ============================================================
// 3. Compile Speed Scaling
// ============================================================

/// Generate synthetic Vais code with approximately `loc` lines of code
fn generate_vais_code(loc: usize) -> String {
    let mut code = String::new();
    let funcs_needed = loc / 3; // ~3 lines per function

    for i in 0..funcs_needed {
        code.push_str(&format!(
            "F func_{i}(x: i64, y: i64) -> i64 {{\n  z := x * {m} + y\n  R z + {c}\n}}\n\n",
            i = i,
            m = (i % 7) + 1,
            c = i % 100,
        ));
    }

    code.push_str("F main() -> i64 = func_0(1, 2)\n");
    code
}

fn bench_compile_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("compile_scaling");
    group.sample_size(10); // Larger inputs take longer

    for loc in [100, 1000, 5000] {
        let source = generate_vais_code(loc);
        let actual_loc = source.lines().count();
        let bytes = source.len() as u64;

        group.throughput(Throughput::Bytes(bytes));

        // Lex only
        group.bench_with_input(
            BenchmarkId::new("lex", format!("{}_LOC", actual_loc)),
            &source,
            |b, s| b.iter(|| tokenize(black_box(s))),
        );

        // Lex + Parse
        group.bench_with_input(
            BenchmarkId::new("parse", format!("{}_LOC", actual_loc)),
            &source,
            |b, s| b.iter(|| parse(black_box(s))),
        );

        // Full pipeline (lex + parse + typecheck + codegen)
        group.bench_with_input(
            BenchmarkId::new("full", format!("{}_LOC", actual_loc)),
            &source,
            |b, s| {
                b.iter(|| {
                    let ast = parse(black_box(s)).expect("parse");
                    let mut checker = TypeChecker::new();
                    checker.check_module(&ast).expect("typecheck");
                    let mut codegen = CodeGenerator::new("bench");
                    codegen.generate_module(&ast)
                })
            },
        );
    }

    group.finish();
}

// ============================================================
// 4. Memory Usage Estimation
// ============================================================

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_estimation");

    for loc in [100, 1000, 5000] {
        let source = generate_vais_code(loc);

        // Measure the size of generated IR as a proxy for memory usage
        group.bench_with_input(BenchmarkId::new("ir_size", loc), &source, |b, s| {
            b.iter(|| {
                let ast = parse(black_box(s)).expect("parse");
                let mut checker = TypeChecker::new();
                checker.check_module(&ast).expect("typecheck");
                let mut codegen = CodeGenerator::new("bench");
                let ir = codegen.generate_module(&ast).expect("codegen");
                ir.len() // Return IR size as proxy for memory
            })
        });

        // Measure AST node count as allocation proxy
        group.bench_with_input(BenchmarkId::new("ast_size", loc), &source, |b, s| {
            b.iter(|| {
                let ast = parse(black_box(s)).expect("parse");
                // Count items as rough allocation metric
                ast.items.len()
            })
        });
    }

    eprintln!("\n=== Memory Usage Estimation ===");
    for loc in [100, 1000, 5000] {
        let source = generate_vais_code(loc);
        let ast = parse(&source).expect("parse");
        let mut checker = TypeChecker::new();
        checker.check_module(&ast).expect("typecheck");
        let mut codegen = CodeGenerator::new("bench");
        let ir = codegen.generate_module(&ast).expect("codegen");

        eprintln!(
            "{} LOC: source={} bytes, AST items={}, IR={} bytes, IR/source ratio={:.2}x",
            source.lines().count(),
            source.len(),
            ast.items.len(),
            ir.len(),
            ir.len() as f64 / source.len() as f64
        );
    }

    group.finish();
}

// ============================================================
// Main
// ============================================================

criterion_group!(
    benches,
    bench_token_count,
    bench_runtime_algorithms,
    bench_compile_scaling,
    bench_memory_usage,
);

criterion_main!(benches);

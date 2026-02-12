//! Runtime execution benchmarks for compiled Vais binaries
//!
//! Compares execution performance of Vais-compiled programs against native Rust.
//! This measures the quality of generated LLVM IR and final binary performance.
//!
//! Architecture:
//! 1. Compile .vais source to LLVM IR (--emit-ir)
//! 2. Use clang to link IR to native binary
//! 3. Benchmark binary execution time
//! 4. Compare against equivalent Rust implementations

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Check if clang is available on the system
fn is_clang_available() -> bool {
    Command::new("clang")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Compile a Vais source file to native binary
/// Returns the path to the compiled binary, or None if compilation failed
fn compile_vais_to_binary(source_path: &Path) -> Option<PathBuf> {
    if !is_clang_available() {
        eprintln!("Skipping Vais binary compilation: clang not found");
        return None;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_path = PathBuf::from(&manifest_dir);
    let project_root = manifest_path.parent().expect("Failed to get project root");

    let source_abs = project_root.join(source_path);
    if !source_abs.exists() {
        eprintln!("Source file not found: {}", source_abs.display());
        return None;
    }

    let stem = source_abs
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("Invalid file name");

    let ir_path = project_root
        .join("target")
        .join("bench_ir")
        .join(format!("{}.ll", stem));
    let binary_path = project_root.join("target").join("bench_bin").join(stem);

    // Create output directories
    fs::create_dir_all(ir_path.parent().unwrap()).ok();
    fs::create_dir_all(binary_path.parent().unwrap()).ok();

    // Step 1: Compile Vais to LLVM IR
    let vaisc_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "vaisc",
            "--quiet",
            "--",
            "--emit-ir",
            source_abs.to_str().unwrap(),
        ])
        .current_dir(project_root)
        .output();

    if let Err(e) = vaisc_output {
        eprintln!("Failed to run vaisc: {}", e);
        return None;
    }

    let vaisc_result = vaisc_output.unwrap();
    if !vaisc_result.status.success() {
        eprintln!(
            "vaisc compilation failed for {}: {}",
            source_path.display(),
            String::from_utf8_lossy(&vaisc_result.stderr)
        );
        return None;
    }

    // vaisc --emit-ir writes to <source_name>.ll in the same directory
    let default_ir = source_abs.with_extension("ll");
    if !default_ir.exists() {
        eprintln!("IR file not generated: {}", default_ir.display());
        return None;
    }

    // Move IR to our target directory
    if fs::copy(&default_ir, &ir_path).is_err() {
        eprintln!("Failed to copy IR to {}", ir_path.display());
        return None;
    }
    let _ = fs::remove_file(&default_ir); // Clean up original IR

    // Step 2: Link IR to binary with clang
    let clang_output = Command::new("clang")
        .args([
            ir_path.to_str().unwrap(),
            "-o",
            binary_path.to_str().unwrap(),
            "-lm", // Link math library (required for some builtins)
        ])
        .output();

    if let Err(e) = clang_output {
        eprintln!("Failed to run clang: {}", e);
        return None;
    }

    let clang_result = clang_output.unwrap();
    if !clang_result.status.success() {
        eprintln!(
            "clang linking failed for {}: {}",
            ir_path.display(),
            String::from_utf8_lossy(&clang_result.stderr)
        );
        return None;
    }

    if !binary_path.exists() {
        eprintln!("Binary not generated: {}", binary_path.display());
        return None;
    }

    Some(binary_path)
}

/// Execute a compiled binary and return its exit code
fn execute_binary(binary_path: &Path) -> Option<i32> {
    let output = Command::new(binary_path).output().ok()?;

    if !output.status.success() {
        eprintln!(
            "Binary execution failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return None;
    }

    // Extract exit code
    Some(output.status.code().unwrap_or(0))
}

//
// Rust reference implementations (for comparison)
//

fn rust_fibonacci(n: i64) -> i64 {
    if n <= 1 {
        n
    } else {
        rust_fibonacci(n - 1) + rust_fibonacci(n - 2)
    }
}

fn rust_is_prime(n: i64) -> bool {
    if n <= 1 {
        return false;
    }
    if n <= 3 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut i = 3;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 2;
    }
    true
}

fn rust_count_primes(limit: i64) -> i64 {
    (2..=limit).filter(|&n| rust_is_prime(n)).count() as i64
}

fn rust_quicksort(arr: &mut [i64]) {
    if arr.len() <= 1 {
        return;
    }
    let pivot_idx = rust_partition(arr);
    rust_quicksort(&mut arr[..pivot_idx]);
    rust_quicksort(&mut arr[pivot_idx + 1..]);
}

fn rust_partition(arr: &mut [i64]) -> usize {
    let len = arr.len();
    let pivot = arr[len - 1];
    let mut i = 0;

    for j in 0..len - 1 {
        if arr[j] < pivot {
            arr.swap(i, j);
            i += 1;
        }
    }
    arr.swap(i, len - 1);
    i
}

fn rust_checksum(arr: &[i64]) -> i64 {
    arr.iter().sum()
}

//
// Benchmark functions
//

/// Benchmark: Fibonacci computation (fib(35) recursive)
fn bench_fibonacci(c: &mut Criterion) {
    let mut group = c.benchmark_group("compute/fibonacci");

    // Rust baseline
    group.bench_function("rust_fib35", |b| b.iter(|| rust_fibonacci(black_box(35))));

    // Vais compiled binary
    if let Some(binary) = compile_vais_to_binary(Path::new("examples/bench_fibonacci.vais")) {
        group.bench_function("vais_fib35", |b| {
            b.iter(|| {
                execute_binary(&binary);
            })
        });
    } else {
        eprintln!("Skipping Vais fibonacci benchmark: compilation failed");
    }

    group.finish();
}

/// Benchmark: Prime counting (count primes up to 100000)
fn bench_primes(c: &mut Criterion) {
    let mut group = c.benchmark_group("compute/primes");

    // Rust baseline
    group.bench_function("rust_count_100k", |b| {
        b.iter(|| rust_count_primes(black_box(100000)))
    });

    // Vais compiled binary
    if let Some(binary) = compile_vais_to_binary(Path::new("examples/bench_compute.vais")) {
        group.bench_function("vais_count_100k", |b| {
            b.iter(|| {
                execute_binary(&binary);
            })
        });
    } else {
        eprintln!("Skipping Vais primes benchmark: compilation failed");
    }

    group.finish();
}

/// Benchmark: Quicksort (10000 elements)
fn bench_sorting(c: &mut Criterion) {
    let mut group = c.benchmark_group("compute/sorting");

    // Rust baseline
    group.bench_function("rust_quicksort_10k", |b| {
        b.iter(|| {
            let mut arr: Vec<i64> = (0..10000).rev().collect();
            rust_quicksort(&mut arr);
            rust_checksum(&arr)
        })
    });

    // Vais compiled binary
    if let Some(binary) = compile_vais_to_binary(Path::new("examples/bench_sorting.vais")) {
        group.bench_function("vais_quicksort_10k", |b| {
            b.iter(|| {
                execute_binary(&binary);
            })
        });
    } else {
        eprintln!("Skipping Vais sorting benchmark: compilation failed");
    }

    group.finish();
}

/// Benchmark: Comparison across different input sizes
fn bench_fibonacci_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling/fibonacci");

    for n in [20, 25, 30, 35] {
        group.bench_with_input(BenchmarkId::new("rust", n), &n, |b, &n| {
            b.iter(|| rust_fibonacci(black_box(n)))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_fibonacci,
    bench_primes,
    bench_sorting,
    bench_fibonacci_scaling,
);

criterion_main!(benches);

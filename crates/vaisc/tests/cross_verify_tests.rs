//! Cross-Verification Tests: Rust compiler vs Selfhost compiler
//!
//! These tests verify that the selfhost compiler (vaisc-stage1) produces
//! the same execution results as the Rust compiler (vaisc) for a set of
//! example programs. Each test:
//!   1. Compiles the .vais file with the Rust compiler → IR → binary → run
//!   2. Compiles the same .vais file with selfhost → IR → binary → run
//!   3. Asserts that stdout and exit codes match

use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn selfhost_dir() -> PathBuf {
    project_root().join("selfhost")
}

fn examples_dir() -> PathBuf {
    project_root().join("examples")
}

fn vaisc_bin() -> PathBuf {
    project_root().join("target/release/vaisc")
}

fn stage1_bin() -> PathBuf {
    selfhost_dir().join("vaisc-stage1")
}

fn runtime_o() -> PathBuf {
    selfhost_dir().join("runtime.o")
}

/// Check if prerequisites are available
fn prerequisites_met() -> bool {
    vaisc_bin().exists() && stage1_bin().exists() && runtime_o().exists()
}

struct CompileRunResult {
    exit_code: i32,
    stdout: String,
}

/// Compile a .vais file with the Rust compiler, then link and run
fn compile_run_rust(vais_file: &Path, tmp: &Path) -> Result<CompileRunResult, String> {
    let ir_path = tmp.join("rust_output.ll");
    let exe_path = tmp.join("rust_exe");

    // Compile with Rust vaisc
    let output = Command::new(vaisc_bin())
        .arg(vais_file)
        .arg("--emit-ir")
        .arg("-o")
        .arg(&ir_path)
        .output()
        .map_err(|e| format!("Failed to run vaisc: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Rust compile failed: {}", stderr));
    }

    // Link with clang
    let output = Command::new("clang")
        .arg(&ir_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("-Wno-override-module")
        .arg("-lm")
        .output()
        .map_err(|e| format!("clang failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Rust link failed: {}", stderr));
    }

    // Run
    let output = Command::new(&exe_path)
        .output()
        .map_err(|e| format!("Rust exe failed: {}", e))?;

    Ok(CompileRunResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
    })
}

/// Compile a .vais file with the selfhost compiler, then link and run
fn compile_run_selfhost(vais_file: &Path, tmp: &Path) -> Result<CompileRunResult, String> {
    // Selfhost always writes to selfhost/main_output.ll regardless of -o flag
    let fixed_ir_path = selfhost_dir().join("main_output.ll");
    let ir_path = tmp.join("selfhost_output.ll");
    let exe_path = tmp.join("selfhost_exe");

    // Compile with selfhost (run from project root so paths resolve)
    let output = Command::new(stage1_bin())
        .arg(vais_file)
        .arg("--emit-ir")
        .arg("--no-ownership-check")
        .current_dir(project_root())
        .output()
        .map_err(|e| format!("Failed to run stage1: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Selfhost compile failed: {}", stderr));
    }

    // Copy IR from fixed location to temp dir
    std::fs::copy(&fixed_ir_path, &ir_path)
        .map_err(|e| format!("Failed to copy IR from {}: {}", fixed_ir_path.display(), e))?;

    // Link with clang + runtime.o
    let output = Command::new("clang")
        .arg(&ir_path)
        .arg(&runtime_o())
        .arg("-o")
        .arg(&exe_path)
        .arg("-Wno-override-module")
        .arg("-lm")
        .output()
        .map_err(|e| format!("clang failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Selfhost link failed: {}", stderr));
    }

    // Run
    let output = Command::new(&exe_path)
        .output()
        .map_err(|e| format!("Selfhost exe failed: {}", e))?;

    Ok(CompileRunResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
    })
}

/// Core cross-verify function: compile with both compilers, compare results
fn cross_verify(example_name: &str) {
    if !prerequisites_met() {
        eprintln!(
            "SKIP: prerequisites not met (need release vaisc, vaisc-stage1, runtime.o)"
        );
        return;
    }

    let vais_file = examples_dir().join(example_name);
    assert!(
        vais_file.exists(),
        "Example file not found: {}",
        vais_file.display()
    );

    let tmp = TempDir::new().expect("Failed to create temp dir");

    let rust_result = compile_run_rust(&vais_file, tmp.path())
        .unwrap_or_else(|e| panic!("[{}] Rust compiler: {}", example_name, e));

    let selfhost_result = compile_run_selfhost(&vais_file, tmp.path())
        .unwrap_or_else(|e| panic!("[{}] Selfhost compiler: {}", example_name, e));

    assert_eq!(
        rust_result.exit_code, selfhost_result.exit_code,
        "[{}] Exit code mismatch: Rust={}, Selfhost={}",
        example_name, rust_result.exit_code, selfhost_result.exit_code
    );

    assert_eq!(
        rust_result.stdout, selfhost_result.stdout,
        "[{}] Stdout mismatch:\n  Rust:     {:?}\n  Selfhost: {:?}",
        example_name, rust_result.stdout, selfhost_result.stdout
    );
}

// === Cross-verification tests for each passing example ===

#[test]
#[ignore] // Run with: cargo test --test cross_verify_tests -- --ignored
fn cross_verify_hello() {
    cross_verify("hello.vais");
}

#[test]
#[ignore]
fn cross_verify_hello_world() {
    cross_verify("hello_world.vais");
}

#[test]
#[ignore]
fn cross_verify_fib() {
    cross_verify("fib.vais");
}

#[test]
#[ignore]
fn cross_verify_control_flow() {
    cross_verify("control_flow.vais");
}

#[test]
#[ignore]
fn cross_verify_putchar_var() {
    cross_verify("putchar_var.vais");
}

#[test]
#[ignore]
fn cross_verify_malloc_test() {
    cross_verify("malloc_test.vais");
}

#[test]
#[ignore]
fn cross_verify_enum_test() {
    cross_verify("enum_test.vais");
}

#[test]
#[ignore]
fn cross_verify_tco_tail_call() {
    cross_verify("tco_tail_call.vais");
}

#[test]
#[ignore]
fn cross_verify_opt_test() {
    cross_verify("opt_test.vais");
}

/// Run all passing cross-verification tests in one go
#[test]
#[ignore]
fn cross_verify_all_passing() {
    let passing = [
        "hello.vais",
        "hello_world.vais",
        "fib.vais",
        "control_flow.vais",
        "putchar_var.vais",
        "malloc_test.vais",
        "enum_test.vais",
        "tco_tail_call.vais",
        "opt_test.vais",
    ];

    if !prerequisites_met() {
        eprintln!(
            "SKIP: prerequisites not met (need release vaisc, vaisc-stage1, runtime.o)"
        );
        return;
    }

    let mut passed = 0;
    let mut failed = Vec::new();

    for name in &passing {
        let vais_file = examples_dir().join(name);
        if !vais_file.exists() {
            failed.push(format!("{}: file not found", name));
            continue;
        }

        let tmp = TempDir::new().expect("Failed to create temp dir");

        let rust = match compile_run_rust(&vais_file, tmp.path()) {
            Ok(r) => r,
            Err(e) => {
                failed.push(format!("{}: Rust error: {}", name, e));
                continue;
            }
        };

        let selfhost = match compile_run_selfhost(&vais_file, tmp.path()) {
            Ok(r) => r,
            Err(e) => {
                failed.push(format!("{}: Selfhost error: {}", name, e));
                continue;
            }
        };

        if rust.exit_code != selfhost.exit_code {
            failed.push(format!(
                "{}: exit code Rust={} Selfhost={}",
                name, rust.exit_code, selfhost.exit_code
            ));
        } else if rust.stdout != selfhost.stdout {
            failed.push(format!(
                "{}: stdout mismatch",
                name
            ));
        } else {
            passed += 1;
        }
    }

    eprintln!(
        "\nCross-verify results: {}/{} passed",
        passed,
        passing.len()
    );

    if !failed.is_empty() {
        panic!(
            "Cross-verification failures:\n  {}",
            failed.join("\n  ")
        );
    }
}

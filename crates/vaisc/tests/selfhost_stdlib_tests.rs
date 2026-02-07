//! Selfhost Stdlib E2E Tests
//!
//! These tests verify the selfhost standard library modules (Vec, String, HashMap,
//! Option/Result, File I/O, Print) by compiling and running their test suites
//! through the full vaisc CLI pipeline.
//!
//! Each test:
//! 1. Compiles a selfhost test .vais file with `vaisc --no-ownership-check`
//! 2. Runs the resulting executable
//! 3. Verifies exit code 0 (all assertions passed)

use std::process::Command;
use tempfile::TempDir;

/// Compile a selfhost .vais file via vaisc CLI and run the resulting binary.
/// Returns (exit_code, stdout, stderr).
fn compile_and_run_selfhost(test_file: &str) -> (i32, String, String) {
    let project_root = env!("CARGO_MANIFEST_DIR");
    let workspace_root = format!("{}/../..", project_root);
    let source_path = format!("{}/selfhost/{}", workspace_root, test_file);

    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let exe_path = tmp_dir.path().join("test_exe");

    // Build vaisc first (ensure it's up to date)
    let build = Command::new("cargo")
        .args(["build", "--bin", "vaisc"])
        .current_dir(&workspace_root)
        .output()
        .expect("Failed to build vaisc");

    assert!(
        build.status.success(),
        "cargo build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );

    // Compile the test file
    let compile = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "vaisc",
            "--",
            &source_path,
            "-o",
            exe_path.to_str().unwrap(),
            "--no-ownership-check",
        ])
        .current_dir(&workspace_root)
        .output()
        .expect("Failed to run vaisc");

    if !compile.status.success() {
        let stderr = String::from_utf8_lossy(&compile.stderr);
        panic!(
            "Compilation of {} failed:\n{}",
            test_file, stderr
        );
    }

    // Run the compiled test
    let run = Command::new(exe_path.to_str().unwrap())
        .output()
        .expect("Failed to run test executable");

    let exit_code = run.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&run.stdout).to_string();
    let stderr = String::from_utf8_lossy(&run.stderr).to_string();

    (exit_code, stdout, stderr)
}

#[test]
fn selfhost_stdlib_vec_tests() {
    let (exit_code, stdout, _stderr) = compile_and_run_selfhost("test_vec.vais");
    assert!(
        stdout.contains("All tests PASSED"),
        "Vec tests should all pass.\nstdout:\n{}",
        stdout
    );
    assert_eq!(exit_code, 0, "Vec test suite should exit with 0");
}

#[test]
fn selfhost_stdlib_string_tests() {
    let (exit_code, stdout, _stderr) = compile_and_run_selfhost("test_string.vais");
    assert!(
        stdout.contains("All tests PASSED"),
        "String tests should all pass.\nstdout:\n{}",
        stdout
    );
    assert_eq!(exit_code, 0, "String test suite should exit with 0");
}

#[test]
fn selfhost_stdlib_hashmap_tests() {
    let (exit_code, stdout, _stderr) = compile_and_run_selfhost("test_hashmap.vais");
    assert!(
        stdout.contains("50 tests passed"),
        "HashMap tests should all pass.\nstdout:\n{}",
        stdout
    );
    assert_eq!(exit_code, 0, "HashMap test suite should exit with 0");
}

#[test]
fn selfhost_stdlib_option_tests() {
    let (exit_code, stdout, _stderr) = compile_and_run_selfhost("test_option.vais");
    assert!(
        stdout.contains("32 / 32"),
        "Option tests should all pass.\nstdout:\n{}",
        stdout
    );
    assert_eq!(exit_code, 0, "Option test suite should exit with 0");
}

#[test]
fn selfhost_stdlib_file_io_tests() {
    let (exit_code, stdout, _stderr) = compile_and_run_selfhost("test_file_io.vais");
    assert!(
        stdout.contains("All tests passed"),
        "File I/O tests should all pass.\nstdout:\n{}",
        stdout
    );
    assert_eq!(exit_code, 0, "File I/O test suite should exit with 0");
}

#[test]
fn selfhost_stdlib_print_tests() {
    let (exit_code, stdout, _stderr) = compile_and_run_selfhost("test_print.vais");
    assert!(
        stdout.contains("All tests PASSED"),
        "Print tests should all pass.\nstdout:\n{}",
        stdout
    );
    assert_eq!(exit_code, 0, "Print test suite should exit with 0");
}

#[test]
fn selfhost_pointer_ref_tests() {
    let (exit_code, stdout, _stderr) = compile_and_run_selfhost("test_pointers.vais");
    assert!(
        stdout.contains("6/6 passed"),
        "Pointer/ref tests should all pass.\nstdout:\n{}",
        stdout
    );
    assert_eq!(exit_code, 0, "Pointer/ref test suite should exit with 0");
}

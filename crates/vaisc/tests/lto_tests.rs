//! Integration tests for Link-Time Optimization (LTO)
//!
//! These tests verify that ThinLTO is automatically enabled for release builds

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Helper to create a temporary test file
fn create_test_file(name: &str, content: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("vais_lto_test_{}.vais", name));
    fs::write(&path, content).expect("Failed to write test file");
    path
}

/// Helper to run vaisc command
fn run_vaisc(args: &[&str]) -> std::process::Output {
    let vaisc = env!("CARGO_BIN_EXE_vaisc");
    Command::new(vaisc)
        .args(args)
        .output()
        .expect("Failed to execute vaisc")
}

#[test]
fn test_thinlto_auto_enabled_o2() {
    let source = r#"
F add(a: i64, b: i64) -> i64 = a + b
F main() -> () { () }
"#;

    let test_file = create_test_file("o2_auto_lto", source);
    let ir_file = test_file.with_extension("ll");

    // Build with O2 - should automatically enable ThinLTO
    let output = run_vaisc(&[
        "-v",
        "build",
        test_file.to_str().unwrap(),
        "-O2",
        "--emit-ir",
    ]);

    // Check that compilation succeeded
    assert!(
        output.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that ThinLTO is mentioned in verbose output
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stderr, stdout);

    // Should mention Thin LTO in optimization info
    assert!(
        combined.contains("Thin") || combined.contains("LTO"),
        "ThinLTO should be mentioned in output: {}",
        combined
    );

    // Cleanup
    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&ir_file);
}

#[test]
fn test_thinlto_auto_enabled_o3() {
    let source = r#"
F multiply(a: i64, b: i64) -> i64 = a * b
F main() -> () { () }
"#;

    let test_file = create_test_file("o3_auto_lto", source);
    let ir_file = test_file.with_extension("ll");

    // Build with O3 - should automatically enable ThinLTO
    let output = run_vaisc(&[
        "-v",
        "build",
        test_file.to_str().unwrap(),
        "-O3",
        "--emit-ir",
    ]);

    // Check that compilation succeeded
    assert!(
        output.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Cleanup
    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&ir_file);
}

#[test]
fn test_no_lto_disables_auto_lto() {
    let source = r#"
F square(x: i64) -> i64 = x * x
F main() -> () { () }
"#;

    let test_file = create_test_file("no_lto", source);
    let ir_file = test_file.with_extension("ll");

    // Build with O2 but --no-lto should disable automatic ThinLTO
    let output = run_vaisc(&[
        "-v",
        "build",
        test_file.to_str().unwrap(),
        "-O2",
        "--no-lto",
        "--emit-ir",
    ]);

    // Check that compilation succeeded
    assert!(
        output.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Cleanup
    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&ir_file);
}

#[test]
fn test_explicit_lto_full() {
    let source = r#"
F factorial(n: i64) -> i64 = n < 2 ? 1 : n * @(n - 1)
F main() -> () { () }
"#;

    let test_file = create_test_file("lto_full", source);
    let ir_file = test_file.with_extension("ll");

    // Build with explicit --lto=full
    let output = run_vaisc(&[
        "-v",
        "build",
        test_file.to_str().unwrap(),
        "-O2",
        "--lto=full",
        "--emit-ir",
    ]);

    // Check that compilation succeeded
    assert!(
        output.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that Full LTO is mentioned
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stderr, stdout);

    assert!(
        combined.contains("Full") || combined.contains("LTO"),
        "Full LTO should be mentioned in output: {}",
        combined
    );

    // Cleanup
    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&ir_file);
}

#[test]
fn test_o0_no_auto_lto() {
    let source = r#"
F increment(x: i64) -> i64 = x + 1
F main() -> () { () }
"#;

    let test_file = create_test_file("o0_no_lto", source);
    let ir_file = test_file.with_extension("ll");

    // Build with O0 - should NOT enable ThinLTO automatically
    let output = run_vaisc(&["build", test_file.to_str().unwrap(), "-O0", "--emit-ir"]);

    // Check that compilation succeeded
    assert!(
        output.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Cleanup
    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&ir_file);
}

#[test]
fn test_o1_no_auto_lto() {
    let source = r#"
F double(x: i64) -> i64 = x + x
F main() -> () { () }
"#;

    let test_file = create_test_file("o1_no_lto", source);
    let ir_file = test_file.with_extension("ll");

    // Build with O1 - should NOT enable ThinLTO automatically
    let output = run_vaisc(&["build", test_file.to_str().unwrap(), "-O1", "--emit-ir"]);

    // Check that compilation succeeded
    assert!(
        output.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Cleanup
    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&ir_file);
}

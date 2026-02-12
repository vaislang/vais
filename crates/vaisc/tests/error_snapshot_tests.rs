//! Error Message Snapshot Testing Framework
//!
//! This test suite captures compiler error messages as snapshots and compares
//! them against stored snapshots to prevent regressions in error quality.
//!
//! Features:
//! - Compiles invalid Vais source that should produce errors
//! - Captures the error message string
//! - Compares against stored snapshots in tests/snapshots/
//! - Creates snapshots on first run or when UPDATE_SNAPSHOTS=1 is set
//! - Fails with clear diff when snapshots don't match
//!
//! Usage:
//!   cargo test --test error_snapshot_tests              # Run tests
//!   UPDATE_SNAPSHOTS=1 cargo test --test error_snapshot_tests  # Update snapshots

use std::fs;
use std::path::PathBuf;
use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Compile Vais source to LLVM IR, returning error string if any stage fails
fn compile_to_ir(source: &str) -> Result<String, String> {
    let _tokens = tokenize(source).map_err(|e| format!("Lexer error: {:?}", e))?;
    let module = parse(source).map_err(|e| format!("Parser error: {:?}", e))?;
    let mut checker = TypeChecker::new();
    checker
        .check_module(&module)
        .map_err(|e| format!("Type error: {:?}", e))?;
    let mut gen = CodeGenerator::new("snapshot_test");
    gen.set_resolved_functions(checker.get_all_functions().clone());
    gen.generate_module(&module)
        .map_err(|e| format!("Codegen error: {:?}", e))
}

/// Get the snapshot directory path
fn snapshot_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("tests").join("snapshots")
}

/// Assert that the compilation error message matches the snapshot.
/// If UPDATE_SNAPSHOTS=1 env var is set, update the snapshot instead of failing.
/// If the snapshot file doesn't exist, create it on first run.
fn assert_error_snapshot(test_name: &str, source: &str) {
    let error = match compile_to_ir(source) {
        Ok(_) => panic!(
            "Expected compilation to fail for snapshot '{}', but it succeeded",
            test_name
        ),
        Err(e) => e,
    };

    // Normalize the error: trim whitespace
    let normalized = error.trim().to_string();

    let snap_dir = snapshot_dir();
    let snap_file = snap_dir.join(format!("{}.snap", test_name));

    // Create or update snapshot if env var is set or file doesn't exist
    if std::env::var("UPDATE_SNAPSHOTS").is_ok() || !snap_file.exists() {
        fs::create_dir_all(&snap_dir).expect("Failed to create snapshot dir");
        fs::write(&snap_file, &normalized).expect("Failed to write snapshot");
        eprintln!(
            "[snapshot] {} snapshot: {}",
            if snap_file.exists() {
                "Updated"
            } else {
                "Created"
            },
            snap_file.display()
        );
        return;
    }

    // Compare with existing snapshot
    let expected = fs::read_to_string(&snap_file)
        .unwrap_or_else(|e| panic!("Failed to read snapshot {}: {}", snap_file.display(), e));

    if normalized != expected.trim() {
        panic!(
            "\n\nSnapshot mismatch for '{}'!\n\
             \n--- Expected (from {}) ---\n{}\n\
             \n--- Actual ---\n{}\n\
             \nRun with UPDATE_SNAPSHOTS=1 to update snapshots.\n",
            test_name,
            snap_file.display(),
            expected.trim(),
            normalized
        );
    }
}

// ==================== Snapshot Tests ====================

#[test]
fn snapshot_undefined_variable() {
    let source = "F main() -> i64 = xyz";
    assert_error_snapshot("undefined_variable", source);
}

#[test]
fn snapshot_undefined_function() {
    // Define a function with a similar name to get a predictable suggestion
    let source = r#"
F greet(x: i64) -> i64 = x
F main() -> i64 = gret(1)
"#;
    assert_error_snapshot("undefined_function", source);
}

#[test]
fn snapshot_type_mismatch() {
    let source = "F main() -> bool = 42";
    assert_error_snapshot("type_mismatch", source);
}

#[test]
fn snapshot_wrong_arg_count() {
    let source = r#"
F f(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = f(1)
"#;
    assert_error_snapshot("wrong_arg_count", source);
}

#[test]
fn snapshot_parser_unexpected_token() {
    let source = "F main( { }";
    assert_error_snapshot("parser_unexpected_token", source);
}

#[test]
fn snapshot_unconstrained_params() {
    // Phase 61: Parameters without type annotations and no way to infer should fail
    let source = "F add(a, b) { a + b }";
    assert_error_snapshot("unconstrained_params", source);
}

#[test]
fn snapshot_recursive_no_return_type() {
    // Phase 61: Recursive functions using @ without explicit return type should fail
    let source = "F fib(n: i64) = I n < 2 { R n } E { R @(n-1) + @(n-2) }";
    assert_error_snapshot("recursive_no_return_type", source);
}

#[test]
fn snapshot_invalid_field_access() {
    // Accessing a non-existent field on a struct
    let source = r#"
S User { name: str, age: i64 }
F main() -> i64 {
    u := User { name: "Alice", age: 30 }
    R u.invalid_field
}
"#;
    assert_error_snapshot("invalid_field_access", source);
}

#[test]
fn snapshot_binary_op_type_error() {
    // Binary operation with incompatible types
    let source = r#"F main() -> i64 = 5 + true"#;
    assert_error_snapshot("binary_op_type_error", source);
}

#[test]
fn snapshot_return_type_mismatch() {
    // Function body returns wrong type
    let source = r#"
F get_number() -> i64 {
    R "not a number"
}
F main() -> i64 = 0
"#;
    assert_error_snapshot("return_type_mismatch", source);
}

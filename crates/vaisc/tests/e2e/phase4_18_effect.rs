//! Phase 4.18 — Effect system (pure/io/partial) TC verification.
//!
//! Existing EffectInferrer already enforces effect constraints.
//! These tests capture the current working behavior to prevent regression.

use super::helpers::*;

fn check_only_ok(source: &str) {
    use tempfile::TempDir;
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("ef.vais");
    std::fs::write(&path, source).expect("write");
    let vaisc = env!("CARGO_BIN_EXE_vaisc");
    let out = std::process::Command::new(vaisc)
        .arg("check")
        .arg(&path)
        .output()
        .expect("spawn vaisc");
    assert!(
        out.status.success(),
        "vaisc check failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn check_only_fails_with(source: &str, expected_substring: &str) {
    use tempfile::TempDir;
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("ef.vais");
    std::fs::write(&path, source).expect("write");
    let vaisc = env!("CARGO_BIN_EXE_vaisc");
    let out = std::process::Command::new(vaisc)
        .arg("check")
        .arg(&path)
        .output()
        .expect("spawn vaisc");
    assert!(
        !out.status.success(),
        "vaisc check should have failed for:\n{}",
        source
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains(expected_substring),
        "error missing expected substring '{}':\n{}",
        expected_substring,
        stderr
    );
}

#[test]
fn pure_function_accepts_pure_call() {
    check_only_ok(
        r#"
pure F add(a: i64, b: i64) -> i64 { a + b }
pure F double(n: i64) -> i64 { add(n, n) }
F main() -> i64 { double(5) }
"#,
    );
}

#[test]
fn io_function_can_call_io() {
    check_only_ok(
        r#"
io F log(msg: str) -> i64 { puts(msg); 0 }
io F app() -> i64 { log("hi") }
F main() -> i64 { app() }
"#,
    );
}

#[test]
fn pure_calling_io_is_rejected() {
    // Pure function calling io function should produce an effect error.
    check_only_fails_with(
        r#"
io F log(msg: str) -> i64 { puts(msg); 0 }
pure F bad() -> i64 { log("hi") }
F main() -> i64 { bad() }
"#,
        "pure function cannot call impure function",
    );
}

#[test]
fn total_function_calling_unwrap_is_rejected() {
    // E034 totality: unmarked function with unwrap `!` is flagged.
    check_only_fails_with(
        r#"
F might_panic(opt: Option<i64>) -> i64 { opt! }
F main() -> i64 { might_panic(None) }
"#,
        "may panic",
    );
}

#[test]
fn partial_function_can_unwrap() {
    // `partial` modifier opts in to panic-permitting semantics.
    check_only_ok(
        r#"
partial F unwrap_force(opt: Option<i64>) -> i64 { opt! }
partial F main() -> i64 { unwrap_force(Some(42)) }
"#,
    );
}

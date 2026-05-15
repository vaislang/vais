//! Skeleton tests for `vaisc fix --explicit` (Master Plan v16 Order Step 2).
//!
//! These tests verify:
//!   1. A4-01 detection on the empirical probe fixture (probe.vais exits non-zero,
//!      diagnostic mentions "A4-01" and the binding line number).
//!   2. Requesting an unimplemented site (A4-02) emits a "stage 0" message and
//!      exits non-zero.
//!   3. --dry-run does not modify the input file.

use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

// ---------------------------------------------------------------------------
// Helper: path to the compiled vaisc binary
// ---------------------------------------------------------------------------

fn vaisc_binary() -> PathBuf {
    // Cargo injects CARGO_BIN_EXE_vaisc when running `cargo test --bin vaisc`
    // or `cargo test -p vaisc`.
    let built = env!("CARGO_BIN_EXE_vaisc");
    if !built.is_empty() {
        return PathBuf::from(built);
    }
    if let Ok(p) = std::env::var("VAISC") {
        return PathBuf::from(p);
    }
    PathBuf::from("target/debug/vaisc")
}

// ---------------------------------------------------------------------------
// Helper: path to the A4-01 probe fixture (read-only, never modified by tests)
// ---------------------------------------------------------------------------

fn a4_01_probe() -> PathBuf {
    // The fixture lives in compiler/tests/empirical/A4/A4-01_unit_i64/probe.vais.
    // During `cargo test` the working directory is the package root
    // (compiler/crates/vaisc).  We need to go up two levels.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("../../tests/empirical/A4/A4-01_unit_i64/probe.vais")
}

// ---------------------------------------------------------------------------
// Test 1 — A4-01 detection: non-zero exit, diagnostic names site and line
// ---------------------------------------------------------------------------

#[test]
fn fix_explicit_a4_01_detects_binding() {
    let vaisc = vaisc_binary();
    let probe = a4_01_probe();

    // Confirm the fixture exists before running.
    assert!(
        probe.exists(),
        "A4-01 probe fixture not found at {}",
        probe.display()
    );

    let output = Command::new(&vaisc)
        .args(["fix", "--explicit", probe.to_str().unwrap()])
        // Disable the update check so the test output is deterministic.
        .env("VAIS_NO_UPDATE_CHECK", "1")
        .output()
        .expect("failed to run vaisc");

    // The tool must exit non-zero when a finding is detected.
    assert_ne!(
        output.status.code().unwrap_or(0),
        0,
        "expected non-zero exit when A4-01 issue is present"
    );

    // The diagnostic must mention "A4-01".
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("A4-01"),
        "expected diagnostic to contain 'A4-01'; got:\n{}",
        stderr
    );

    // The probe fixture has `x: i64 = void_fn()` on line 6.
    // The diagnostic must reference line 6.
    assert!(
        stderr.contains("line 6") || stderr.contains(":6"),
        "expected diagnostic to reference line 6; got:\n{}",
        stderr
    );
}

// ---------------------------------------------------------------------------
// Test 2 — unimplemented site: non-zero exit, message says "stage 0"
// ---------------------------------------------------------------------------

#[test]
fn fix_explicit_unimplemented_site_errors() {
    let vaisc = vaisc_binary();
    let probe = a4_01_probe();

    assert!(
        probe.exists(),
        "A4-01 probe not found at {}",
        probe.display()
    );

    let output = Command::new(&vaisc)
        .args(["fix", "--explicit", "--site=A4-02", probe.to_str().unwrap()])
        .env("VAIS_NO_UPDATE_CHECK", "1")
        .output()
        .expect("failed to run vaisc");

    // Must exit non-zero.
    assert_ne!(
        output.status.code().unwrap_or(0),
        0,
        "expected non-zero exit for unimplemented A4-02 site"
    );

    // The error message must contain "stage 0".
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("stage 0"),
        "expected 'stage 0' in error message; got:\n{}",
        stderr
    );
}

// ---------------------------------------------------------------------------
// Test 3 — --dry-run does not modify the source file
// ---------------------------------------------------------------------------

#[test]
fn fix_explicit_dry_run_no_modification() {
    let vaisc = vaisc_binary();
    let probe = a4_01_probe();

    assert!(
        probe.exists(),
        "A4-01 probe not found at {}",
        probe.display()
    );

    // Record the file modification time and content hash before the run.
    let before_mtime = std::fs::metadata(&probe)
        .expect("cannot stat probe")
        .modified()
        .unwrap_or(SystemTime::UNIX_EPOCH);
    let before_content = std::fs::read_to_string(&probe).expect("cannot read probe before dry-run");

    let output = Command::new(&vaisc)
        .args(["fix", "--explicit", "--dry-run", probe.to_str().unwrap()])
        .env("VAIS_NO_UPDATE_CHECK", "1")
        .output()
        .expect("failed to run vaisc");

    // The tool may exit non-zero (finding detected) — that is expected.
    // What we care about is that the file was not changed.
    let after_content = std::fs::read_to_string(&probe).expect("cannot read probe after dry-run");
    let after_mtime = std::fs::metadata(&probe)
        .expect("cannot stat probe after dry-run")
        .modified()
        .unwrap_or(SystemTime::UNIX_EPOCH);

    assert_eq!(
        before_content, after_content,
        "--dry-run must not modify the source file content"
    );
    assert_eq!(
        before_mtime, after_mtime,
        "--dry-run must not change the source file mtime"
    );

    // The stderr should say "dry-run" somewhere so the user knows the mode.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("dry-run"),
        "expected 'dry-run' in output; got:\n{}",
        stderr
    );
}

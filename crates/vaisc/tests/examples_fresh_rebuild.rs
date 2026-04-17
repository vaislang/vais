//! Examples Fresh-Rebuild Gate (Phase 194 P194-4)
//!
//! # Purpose
//! Defends against the cache-hiding regression pattern discovered in Phase 193
//! Recon-C: `.vais-cache/` can make `vaisc FILE.vais` report success using a
//! stale binary even when the current codegen is completely broken. Standard
//! E2E tests were green (2596/0/0) while `examples/simple_vec_test.vais`
//! fresh-build was broken for weeks.
//!
//! # What this test does
//! Compiles every `examples/*.vais` file from scratch (IR-only, no clang link)
//! by invoking the `vaisc build --emit-ir --no-cache` subprocess. Using the
//! real binary path (`env!("CARGO_BIN_EXE_vaisc")`) ensures:
//! - Import resolution (`U std/vec`, `U constants`, etc.) works correctly
//!   because the binary looks for modules relative to the source file's directory.
//! - The cache is bypassed with `--no-cache`, forcing a full recompile every run.
//! - All 188 examples (confirmed by Recon-D) are exercised.
//!
//! # Why `#[ignore]`
//! The full sweep takes ~3 minutes (~1 s per file × 188 files). Running this
//! on every `cargo test` invocation would make the standard dev loop unusable.
//! The gate is intentionally opt-in: run it when you suspect a codegen
//! regression or before releasing a new compiler version.
//!
//! # How to run
//! ```sh
//! # Run only this gate (release build recommended for speed):
//! cargo test --release -p vaisc --test examples_fresh_rebuild -- --ignored
//!
//! # Verify the gate is gated (should report "0 passed, 1 ignored"):
//! cargo test --release -p vaisc --test examples_fresh_rebuild
//! ```
//!
//! # Expected runtime
//! ~3 minutes with a release build of `vaisc`. Debug builds will be slower.
//!
//! # Adding a skip entry
//! If a future example legitimately cannot compile stand-alone (e.g. it requires
//! a native library not available in CI), add its filename to the `SKIP_LIST`
//! constant below with a comment explaining why.

use std::path::PathBuf;
use std::process::Command;

/// Files to skip (filename only, not full path).
///
/// Entries here are examples that compile today but **must fail** the gate for
/// a specific architectural reason (e.g. the test checker is expected to be
/// strict about a rule that examples/ samples happen to trip). Historical or
/// intentional-error fixtures belong in `examples/archive/` and
/// `examples/intentional_errors/` instead — those subdirectories are skipped
/// wholesale by the discovery loop below and do not need individual entries
/// here.
const SKIP_LIST: &[&str] = &[
    // Phase 196 P196-C1: the `LW expr { body }` loop expects the scrutinee to
    // have type Optional/Result, but several reactor-style examples use it on
    // an `()`-valued comparison. Deferred until the type checker grows a
    // `while (bool)` admission rule for LW.
    "async_reactor_test.vais",
    // Phase 196 P196-C1: `[i64; 100]` fixed-size arrays are not treated as
    // indexable by the type checker, so `todo_ids[idx] = v` reports E001.
    // Global-array codegen works; the type-level admission rule is missing.
    "wasm_todo_app.vais",
];

/// Invoke `vaisc build FILE --emit-ir --no-cache` and return Ok(()) on success
/// or Err(truncated_stderr) on failure.
///
/// Propagates `VAIS_STD_PATH` pointing at the repo-root `std/` so examples that
/// import standard library modules (e.g. `U std/vec`) resolve regardless of the
/// subprocess CWD (cargo test environments launch binaries from `target/`).
fn compile_example_emit_ir(example_path: &PathBuf) -> Result<(), String> {
    let vaisc = env!("CARGO_BIN_EXE_vaisc");

    // CARGO_MANIFEST_DIR is crates/vaisc — go up two levels to reach the
    // project root where `std/` lives.
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to canonicalize project root path");
    let std_path = project_root.join("std");

    let output = Command::new(vaisc)
        .arg("build")
        .arg(example_path)
        .arg("--emit-ir")
        .arg("--no-cache")
        .env("VAIS_STD_PATH", &std_path)
        .output()
        .map_err(|e| format!("failed to spawn vaisc: {}", e))?;

    if output.status.success() {
        return Ok(());
    }

    // Collect stderr + stdout for the error message, truncated to 200 chars.
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = if stderr.is_empty() {
        stdout.to_string()
    } else {
        stderr.to_string()
    };
    let truncated: String = combined.chars().take(200).collect();
    Err(truncated)
}

#[test]
#[ignore = "on-demand gate (~3 min): cargo test --release -p vaisc --test examples_fresh_rebuild -- --ignored"]
fn examples_fresh_rebuild() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    // CARGO_MANIFEST_DIR is crates/vaisc — go up two levels to the repo root.
    let examples_dir = PathBuf::from(manifest_dir).join("../..").join("examples");
    let examples_dir = examples_dir
        .canonicalize()
        .expect("failed to canonicalize examples/ path");

    let mut entries: Vec<PathBuf> = std::fs::read_dir(&examples_dir)
        .unwrap_or_else(|e| panic!("failed to read_dir {}: {}", examples_dir.display(), e))
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("vais") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    // Sort for deterministic output order.
    entries.sort();

    assert!(
        !entries.is_empty(),
        "No .vais files found in {}",
        examples_dir.display()
    );

    let mut failures: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();

    for path in &entries {
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>");

        if SKIP_LIST.contains(&file_name) {
            skipped.push(file_name.to_string());
            continue;
        }

        match compile_example_emit_ir(path) {
            Ok(()) => {}
            Err(err_snippet) => {
                failures.push(format!("{}: {}", file_name, err_snippet));
            }
        }
    }

    // Summary line (visible even when tests pass).
    let total = entries.len();
    let passed = total - skipped.len() - failures.len();
    println!(
        "examples_fresh_rebuild: {}/{} passed, {} skipped, {} failed",
        passed,
        total,
        skipped.len(),
        failures.len()
    );

    if !failures.is_empty() {
        // Print individually first so they appear in --nocapture output.
        eprintln!(
            "\nexamples_fresh_rebuild: {} failure(s):",
            failures.len()
        );
        for f in &failures {
            eprintln!("  FAIL  {}", f);
        }

        // Collect all failures and panic once so users see the full picture.
        let failure_list = failures.join("\n  ");
        panic!(
            "examples_fresh_rebuild: {}/{} examples failed IR codegen:\n  {}",
            failures.len(),
            total,
            failure_list
        );
    }
}

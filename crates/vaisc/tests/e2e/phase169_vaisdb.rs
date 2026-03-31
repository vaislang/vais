//! Phase 169: VaisDB cross-module compile + run regression test
//!
//! Compiles examples/projects/vaisdb/main.vais (a multi-module project with
//! page, row, btree, storage modules) end-to-end through the vaisc build
//! command and verifies the resulting binary exits with code 0 and prints
//! "All VaisDB tests passed!".

use std::path::PathBuf;
use std::process::Command;

/// Returns the absolute path to the repository root (two levels above the
/// crates/ directory that contains this test binary).
fn repo_root() -> PathBuf {
    // The test binary lives under target/…; use CARGO_MANIFEST_DIR which is
    // set by cargo to the vaisc crate directory during test compilation.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent() // crates/
        .unwrap()
        .parent() // repo root
        .unwrap()
        .to_path_buf()
}

#[test]
fn e2e_phase169_vaisdb_compiles_and_runs() {
    let root = repo_root();
    let vaisdb_main = root.join("examples/projects/vaisdb/main.vais");
    assert!(
        vaisdb_main.exists(),
        "VaisDB source not found at {}",
        vaisdb_main.display()
    );

    // Output binary goes to a temp path that is unique per test run.
    let out_bin = std::env::temp_dir().join(format!(
        "vaisdb_e2e_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));

    // --- Compilation step ---
    let compile_status = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "vaisc",
            "--",
            "build",
            vaisdb_main.to_str().unwrap(),
            "-o",
            out_bin.to_str().unwrap(),
        ])
        .current_dir(&root)
        .status()
        .expect("failed to spawn cargo run vaisc build");

    assert!(
        compile_status.success(),
        "VaisDB compilation failed with status: {}",
        compile_status
    );
    assert!(
        out_bin.exists(),
        "Expected compiled binary at {} but it was not created",
        out_bin.display()
    );

    // --- Execution step ---
    let run_output = Command::new(&out_bin)
        .output()
        .expect("failed to run VaisDB binary");

    let stdout = String::from_utf8_lossy(&run_output.stdout);
    let stderr = String::from_utf8_lossy(&run_output.stderr);
    let exit_code = run_output.status.code().unwrap_or(-1);

    assert_eq!(
        exit_code, 0,
        "VaisDB binary exited with code {} (expected 0).\nstdout: {}\nstderr: {}",
        exit_code, stdout, stderr
    );
    assert!(
        stdout.contains("All VaisDB tests passed!"),
        "Expected stdout to contain 'All VaisDB tests passed!' but got: {:?}",
        stdout
    );

    // Clean up the temporary binary.
    let _ = std::fs::remove_file(&out_bin);
}

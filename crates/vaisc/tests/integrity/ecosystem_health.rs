/// Ecosystem health measurements — Phase 0.2.
///
/// These tests iterate real source files and COUNT how many pass codegen.
/// They do NOT enforce 100% — that is Phases 4.15 and 5.17.
/// They exist to produce baseline numbers for Phase 0.3.
///
/// Output format (machine-readable, one line per category):
///   INTEGRITY std_files pass=N fail=M total=T
///   INTEGRITY vaisdb_files pass=N fail=M total=T
///
/// Thresholds for Phase 0.2: at least 1 file must pass in each category.
/// The actual counts feed ROADMAP via Phase 0.3.

use super::{ok_codegen, ok_codegen_pkg, ok_tc, setup_std_symlink};
use std::path::PathBuf;
use walkdir::WalkDir;

// ---------------------------------------------------------------------------
// §4 stdlib modules — role requires Codegen OK (no main)
// ---------------------------------------------------------------------------

#[test]
fn test_std_files_codegen_ok() {
    setup_std_symlink();

    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to canonicalize project root");
    let std_dir = project_root.join("std");

    assert!(
        std_dir.is_dir(),
        "std/ directory not found at {}",
        std_dir.display()
    );

    let mut files: Vec<PathBuf> = std::fs::read_dir(&std_dir)
        .unwrap_or_else(|e| panic!("cannot read_dir {}: {}", std_dir.display(), e))
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

    files.sort();

    let total = files.len();
    assert!(
        total > 0,
        "No .vais files found in {}",
        std_dir.display()
    );

    let mut pass = 0usize;
    let mut fail = 0usize;
    let mut fail_names: Vec<String> = Vec::new();

    for path in &files {
        if ok_codegen(path) {
            pass += 1;
        } else {
            fail += 1;
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("<unknown>")
                .to_string();
            fail_names.push(name);
        }
    }

    // Machine-readable summary line — read by Phase 0.3 baseline script.
    eprintln!("INTEGRITY std_files pass={} fail={} total={}", pass, fail, total);

    // Print failing files for diagnostics.
    if !fail_names.is_empty() {
        eprintln!("  std_files FAIL list ({}):", fail_names.len());
        for name in &fail_names {
            eprintln!("    - {}", name);
        }
    }

    // Phase 0.2 threshold: at least 1 file must pass.
    // (Phase 4.15 will raise this to 100%.)
    assert!(
        pass >= 1,
        "INTEGRITY std_files: 0 files passed codegen out of {}. \
         This indicates a fundamental build environment issue.\n\
         Failing files: {:?}",
        total,
        fail_names
    );
}

// ---------------------------------------------------------------------------
// §4 vaisdb modules — role requires Codegen OK with VAIS_DEP_PATHS
// ---------------------------------------------------------------------------

#[test]
fn test_vaisdb_files_codegen_ok() {
    setup_std_symlink();

    let vaisdb_src = PathBuf::from("/Users/sswoo/study/projects/vais/lang/packages/vaisdb/src");

    if !vaisdb_src.is_dir() {
        eprintln!(
            "INTEGRITY vaisdb_files pass=0 fail=0 total=0 (SKIP: vaisdb src not found at {})",
            vaisdb_src.display()
        );
        // Not a hard failure — vaisdb may not be checked out in CI.
        return;
    }

    let std_root = "/tmp/vais-lib/std";
    let deps = format!("{}:{}", vaisdb_src.display(), std_root);

    // Collect all .vais files under vaisdb/src recursively using walkdir.
    let mut files: Vec<PathBuf> = WalkDir::new(&vaisdb_src)
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.into_path();
            if path.extension().and_then(|s| s.to_str()) == Some("vais") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    files.sort();
    let total = files.len();

    if total == 0 {
        eprintln!(
            "INTEGRITY vaisdb_files pass=0 fail=0 total=0 (no .vais files found under {})",
            vaisdb_src.display()
        );
        return;
    }

    let mut pass = 0usize;
    let mut fail = 0usize;
    let mut fail_names: Vec<String> = Vec::new();

    for path in &files {
        if ok_codegen_pkg(path, &deps, std_root) {
            pass += 1;
        } else {
            fail += 1;
            // Store relative path for readable output.
            let rel = path
                .strip_prefix(&vaisdb_src)
                .unwrap_or(path)
                .display()
                .to_string();
            fail_names.push(rel);
        }
    }

    // Machine-readable summary line — read by Phase 0.3 baseline script.
    eprintln!(
        "INTEGRITY vaisdb_files pass={} fail={} total={}",
        pass, fail, total
    );

    // Print first 20 failures for diagnostics (avoid log flooding).
    if !fail_names.is_empty() {
        let show = fail_names.len().min(20);
        eprintln!("  vaisdb_files FAIL list (showing {}/{}):", show, fail_names.len());
        for name in fail_names.iter().take(show) {
            eprintln!("    - {}", name);
        }
        if fail_names.len() > 20 {
            eprintln!("    ... ({} more)", fail_names.len() - 20);
        }
    }

    // Phase 0.2 threshold: at least 1 file must pass.
    // (Phase 5.17 will raise this to 100%.)
    assert!(
        pass >= 1,
        "INTEGRITY vaisdb_files: 0 files passed codegen out of {}. \
         This indicates a fundamental build environment issue.\n\
         First 10 failing files: {:?}",
        total,
        fail_names.iter().take(10).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Reporting — emit summary INTEGRITY lines for compiler_syntax
// ---------------------------------------------------------------------------

#[test]
fn report_compiler_syntax_summary() {
    // compiler_syntax.rs has 200 concrete tests after Phase 1.6
    // (186 active + 14 ignored). Pass count is runtime-determined;
    // emit the total so CI/regression gate can read it.
    eprintln!("INTEGRITY compiler_syntax pass=? fail=? total=200");
}

// ---------------------------------------------------------------------------
// Phase 1.8 — LIVING_SPEC executable reference examples
// ---------------------------------------------------------------------------

#[test]
fn test_living_spec_files_ok() {
    // LIVING_SPEC is the authoritative executable reference for Vais syntax.
    // Every .vais file under docs/language/LIVING_SPEC/ must pass `vaisc check`
    // (type-check, no codegen). If any file fails, agents relying on these
    // examples as ground truth will propagate the failure.
    //
    // Phase 1.8: baseline ≥ 100 files, ALL passing.

    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to canonicalize project root");
    let living_spec_dir = project_root.join("docs/language/LIVING_SPEC");

    if !living_spec_dir.is_dir() {
        eprintln!(
            "INTEGRITY living_spec pass=0 fail=0 total=0 (SKIP: {} not found)",
            living_spec_dir.display()
        );
        return;
    }

    let mut files: Vec<PathBuf> = WalkDir::new(&living_spec_dir)
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.into_path();
            if path.extension().and_then(|s| s.to_str()) == Some("vais") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    files.sort();
    let total = files.len();

    let mut pass = 0usize;
    let mut fail_names: Vec<String> = Vec::new();

    for path in &files {
        if ok_tc(path) {
            pass += 1;
        } else {
            let rel = path
                .strip_prefix(&living_spec_dir)
                .unwrap_or(path)
                .display()
                .to_string();
            fail_names.push(rel);
        }
    }

    eprintln!(
        "INTEGRITY living_spec pass={} fail={} total={}",
        pass,
        fail_names.len(),
        total
    );

    if !fail_names.is_empty() {
        eprintln!("  living_spec FAIL list ({}):", fail_names.len());
        for name in &fail_names {
            eprintln!("    - {}", name);
        }
    }

    // Phase 1.8 gate: 100+ files, ALL passing.
    assert!(
        total >= 100,
        "LIVING_SPEC must have ≥100 files; found {}",
        total
    );
    assert!(
        fail_names.is_empty(),
        "LIVING_SPEC has {} failing files: {:?}",
        fail_names.len(),
        fail_names
    );
}

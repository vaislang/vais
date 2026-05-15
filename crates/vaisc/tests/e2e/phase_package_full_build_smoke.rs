//! Package full-build smoke gate (Phase 1 100% Gap close).
//!
//! Builds each lang/packages/<pkg>/src/main.vais entry-point with the actual
//! `vaisc build` driver. Catches production-cascade bugs that runtime-smoke
//! gates (which use inline fixture source) silently miss (LESSONS L-018).
//!
//! This gate is part of the "100% Gap" Phase 1 close — the user's mandate
//! that compiler stability MUST reach operational 100% (entry-point full
//! builds GREEN for every lang/packages/* package) before web/db/server
//! feature progress (Phase β) is allowed.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Build `<repo_root>/lang/packages/<pkg>/src/main.vais` with `vaisc build`.
/// Returns Ok(()) on success, Err(stderr) on failure.
fn try_build_pkg_entry(pkg: &str, entry: &Path) -> Result<(), String> {
    let compiler_root = compiler_root();
    if !entry.exists() {
        return Err(format!(
            "package entry not found: {} (pkg={})",
            entry.display(),
            pkg
        ));
    }

    let std_path = compiler_root.join("std");
    let vaisc = compiler_root.join("target").join("release").join("vaisc");
    if !vaisc.exists() {
        // Fall back to debug build if release isn't there (shouldn't happen in CI).
        let debug_vaisc = compiler_root.join("target").join("debug").join("vaisc");
        if !debug_vaisc.exists() {
            return Err(format!(
                "vaisc binary not found at {} or {}",
                vaisc.display(),
                debug_vaisc.display()
            ));
        }
    }
    let vaisc_path = if vaisc.exists() {
        vaisc
    } else {
        compiler_root.join("target").join("debug").join("vaisc")
    };

    let tmp_out = std::env::temp_dir().join(format!("vais_pkg_full_build_{}", pkg));
    let _ = std::fs::remove_file(&tmp_out);

    let output = Command::new(&vaisc_path)
        .env("VAIS_STD_PATH", &std_path)
        .arg("build")
        .arg(entry)
        .arg("-o")
        .arg(&tmp_out)
        .output()
        .map_err(|e| format!("failed to spawn vaisc: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!(
            "vaisc build failed for {} (pkg={}):\n{}",
            entry.display(),
            pkg,
            stderr
        ));
    }
    Ok(())
}

fn package_entry(pkg: &str) -> Option<PathBuf> {
    let entry = lang_root()?
        .join("packages")
        .join(pkg)
        .join("src")
        .join("main.vais");
    if !entry.exists() {
        eprintln!(
            "SKIP: package entry not found: {} (pkg={})",
            entry.display(),
            pkg
        );
        return None;
    }
    Some(entry)
}

fn lang_root() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("VAIS_LANG_ROOT") {
        let path = PathBuf::from(path);
        if path.is_dir() {
            return Some(path);
        }
    }

    for candidate in [
        workspace_root().join("lang"),
        compiler_root().join("vais-lang"),
        compiler_root().join("lang"),
    ] {
        if candidate.is_dir() {
            return Some(candidate);
        }
    }

    eprintln!("SKIP: vais-lang root not found; set VAIS_LANG_ROOT to run package full-build smoke");
    None
}

fn compiler_root() -> PathBuf {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set in tests");
    let path = Path::new(&manifest_dir);
    path.parent()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
        .expect("could not derive compiler root from CARGO_MANIFEST_DIR")
}

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points to compiler/crates/vaisc; the workspace root
    // (containing both compiler/ and lang/) is two levels up plus one.
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set in tests");
    let path = Path::new(&manifest_dir);
    // compiler/crates/vaisc → compiler → workspace_root
    path.parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(PathBuf::from)
        .expect("could not derive workspace root from CARGO_MANIFEST_DIR")
}

#[test]
fn e2e_pkg_full_build_vais_server_main() {
    let Some(entry) = package_entry("vais-server") else {
        return;
    };
    match try_build_pkg_entry("vais-server", &entry) {
        Ok(()) => {}
        Err(e) => panic!("vais-server full build failed:\n{}", e),
    }
}

#[test]
fn e2e_pkg_full_build_vaisdb_main() {
    let Some(entry) = package_entry("vaisdb") else {
        return;
    };
    match try_build_pkg_entry("vaisdb", &entry) {
        Ok(()) => {}
        Err(e) => panic!("vaisdb full build failed:\n{}", e),
    }
}

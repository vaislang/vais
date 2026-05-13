//! Utility functions for build operations.

use std::path::PathBuf;

/// Find the Vais standard library directory (for gpu_runtime.c etc.)
pub(crate) fn find_std_dir() -> Option<PathBuf> {
    // Check relative to executable
    if let Ok(exe) = std::env::current_exe() {
        let exe_dir = exe.parent()?;
        // Check ../std/ (installed layout)
        let std_dir = exe_dir.join("../std");
        if std_dir.exists() {
            return Some(std_dir.canonicalize().unwrap_or(std_dir));
        }
        // Check ../../std/ (cargo build layout)
        let std_dir = exe_dir.join("../../std");
        if std_dir.exists() {
            return Some(std_dir.canonicalize().unwrap_or(std_dir));
        }
    }
    // Check VAIS_STD_DIR environment variable
    if let Ok(dir) = std::env::var("VAIS_STD_DIR") {
        let path = PathBuf::from(&dir);
        if path.exists() {
            return Some(path);
        }
    }
    // Check current directory's std/
    let cwd_std = PathBuf::from("std");
    if cwd_std.exists() {
        return Some(cwd_std.canonicalize().unwrap_or(cwd_std));
    }
    None
}

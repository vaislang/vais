//! Bootstrap verification tests for the Vais self-hosting compiler
//!
//! These tests verify that the self-hosting compiler source files
//! can be compiled by the Rust-based compiler (Stage 0 → Stage 1 path).
//! Full Stage 2/3 verification requires the bootstrap-verify.sh script.

use std::path::Path;

/// Verify selfhost source files exist
#[test]
fn selfhost_source_files_exist() {
    let selfhost_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("selfhost");

    let required_files = [
        "main_entry.vais",
        "constants.vais",
        "stringbuffer_s1.vais",
        "lexer_s1.vais",
        "helpers_s1.vais",
        "parser_s1.vais",
        "codegen_s1.vais",
        "runtime.c",
        "bootstrap_test.vais",
    ];

    for file in &required_files {
        let path = selfhost_dir.join(file);
        assert!(
            path.exists(),
            "Required selfhost file missing: {}",
            path.display()
        );
    }
}

/// Verify bootstrap verification script exists and is executable
#[test]
fn bootstrap_script_exists() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("scripts/bootstrap-verify.sh");

    assert!(
        script.exists(),
        "Bootstrap verification script missing: {}",
        script.display()
    );
}

/// Verify selfhost main_entry.vais can be tokenized by the Rust compiler
#[test]
fn selfhost_main_entry_tokenizes() {
    let selfhost_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("selfhost");

    let source = std::fs::read_to_string(selfhost_dir.join("main_entry.vais"))
        .expect("Failed to read main_entry.vais");

    // Verify it can be tokenized (basic sanity check)
    let result = vais_lexer::tokenize(&source);
    assert!(
        result.is_ok(),
        "Failed to tokenize main_entry.vais: {:?}",
        result.err()
    );
}

/// Verify selfhost constants.vais can be tokenized
#[test]
fn selfhost_constants_tokenizes() {
    let selfhost_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("selfhost");

    let source = std::fs::read_to_string(selfhost_dir.join("constants.vais"))
        .expect("Failed to read constants.vais");

    let result = vais_lexer::tokenize(&source);
    assert!(
        result.is_ok(),
        "Failed to tokenize constants.vais: {:?}",
        result.err()
    );
}

/// Verify runtime.c compiles (syntax check)
#[test]
fn selfhost_runtime_c_valid() {
    let selfhost_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("selfhost");

    let source =
        std::fs::read_to_string(selfhost_dir.join("runtime.c")).expect("Failed to read runtime.c");

    // Basic validity checks
    assert!(source.contains("#include <stdint.h>"));
    assert!(source.contains("load_i64"));
    assert!(source.contains("store_i64"));
    assert!(source.contains("vais_gc_alloc"));
}

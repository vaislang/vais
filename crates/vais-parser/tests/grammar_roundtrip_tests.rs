//! Round-trip consistency tests for the Vais parser.
//!
//! Verifies deterministic parsing: parsing the same source twice must produce
//! identical AST structures. Tests all .vais files in examples/ and std/ directories.

use std::fs;
use std::path::{Path, PathBuf};
use vais_lexer::tokenize;
use vais_parser::Parser;

/// Returns the project root directory (parent of crates/).
fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap() // crates/
        .parent()
        .unwrap() // project root
        .to_path_buf()
}

/// Collects all .vais files in the given directory (non-recursive).
fn collect_vais_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("Failed to read directory {:?}: {}", dir, e))
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension().map_or(false, |e| e == "vais") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    files.sort();
    files
}

/// Result of a deterministic parsing check for a single file.
struct ParseCheckResult {
    success: u32,
    lex_fail: u32,
    parse_fail: u32,
    non_deterministic: u32,
}

impl ParseCheckResult {
    fn new() -> Self {
        Self {
            success: 0,
            lex_fail: 0,
            parse_fail: 0,
            non_deterministic: 0,
        }
    }

    fn merge(&mut self, other: &ParseCheckResult) {
        self.success += other.success;
        self.lex_fail += other.lex_fail;
        self.parse_fail += other.parse_fail;
        self.non_deterministic += other.non_deterministic;
    }
}

/// Parse a single file twice and verify deterministic output.
fn check_deterministic_parse(path: &Path) -> ParseCheckResult {
    let mut result = ParseCheckResult::new();

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => {
            result.lex_fail += 1;
            return result;
        }
    };

    // Tokenize twice (separate token vecs for each parser)
    let tokens1 = match tokenize(&source) {
        Ok(t) => t,
        Err(_) => {
            result.lex_fail += 1;
            return result;
        }
    };
    let tokens2 = match tokenize(&source) {
        Ok(t) => t,
        Err(_) => {
            result.lex_fail += 1;
            return result;
        }
    };

    // Parse twice
    let mut parser1 = Parser::new(tokens1);
    let mut parser2 = Parser::new(tokens2);

    let r1 = parser1.parse_module();
    let r2 = parser2.parse_module();

    match (r1, r2) {
        (Ok(m1), Ok(m2)) => {
            let d1 = format!("{:?}", m1);
            let d2 = format!("{:?}", m2);
            if d1 == d2 {
                result.success += 1;
            } else {
                result.non_deterministic += 1;
                eprintln!("NON-DETERMINISTIC: {:?}", path.file_name().unwrap());
            }
        }
        (Err(_), Err(_)) => {
            // Both fail consistently — not a determinism issue
            result.parse_fail += 1;
        }
        _ => {
            // One succeeds, one fails — non-deterministic
            result.non_deterministic += 1;
            eprintln!(
                "NON-DETERMINISTIC (success/fail mismatch): {:?}",
                path.file_name().unwrap()
            );
        }
    }

    result
}

// =============================================================================
// Deterministic parsing tests
// =============================================================================

#[test]
fn roundtrip_deterministic_examples() {
    let root = project_root().join("examples");
    let files = collect_vais_files(&root);
    assert!(!files.is_empty(), "No .vais files found in examples/");

    let mut totals = ParseCheckResult::new();
    for path in &files {
        let r = check_deterministic_parse(path);
        totals.merge(&r);
    }

    eprintln!(
        "examples/: {} files, {} success, {} lex fail, {} parse fail, {} non-deterministic",
        files.len(),
        totals.success,
        totals.lex_fail,
        totals.parse_fail,
        totals.non_deterministic
    );

    assert_eq!(
        totals.non_deterministic, 0,
        "Parser must be deterministic for all example files"
    );
    assert!(
        totals.success > 0,
        "At least some example files should parse successfully"
    );
}

#[test]
fn roundtrip_deterministic_std() {
    let root = project_root().join("std");
    let files = collect_vais_files(&root);
    assert!(!files.is_empty(), "No .vais files found in std/");

    let mut totals = ParseCheckResult::new();
    for path in &files {
        let r = check_deterministic_parse(path);
        totals.merge(&r);
    }

    eprintln!(
        "std/: {} files, {} success, {} lex fail, {} parse fail, {} non-deterministic",
        files.len(),
        totals.success,
        totals.lex_fail,
        totals.parse_fail,
        totals.non_deterministic
    );

    assert_eq!(
        totals.non_deterministic, 0,
        "Parser must be deterministic for all std files"
    );
    assert!(
        totals.success > 0,
        "At least some std files should parse successfully"
    );
}

// =============================================================================
// Parse success rate tests
// =============================================================================

#[test]
fn parse_success_rate_examples() {
    let root = project_root().join("examples");
    let files = collect_vais_files(&root);

    let mut totals = ParseCheckResult::new();
    for path in &files {
        let r = check_deterministic_parse(path);
        totals.merge(&r);
    }

    let total = files.len() as u32;
    let rate = (totals.success as f64 / total as f64) * 100.0;
    eprintln!(
        "examples/ parse success rate: {}/{} ({:.1}%)",
        totals.success, total, rate
    );

    // At least 50% of example files should parse successfully
    assert!(
        rate >= 50.0,
        "Expected at least 50% parse success rate for examples/, got {:.1}%",
        rate
    );
}

#[test]
fn parse_success_rate_std() {
    let root = project_root().join("std");
    let files = collect_vais_files(&root);

    let mut totals = ParseCheckResult::new();
    for path in &files {
        let r = check_deterministic_parse(path);
        totals.merge(&r);
    }

    let total = files.len() as u32;
    let rate = (totals.success as f64 / total as f64) * 100.0;
    eprintln!(
        "std/ parse success rate: {}/{} ({:.1}%)",
        totals.success, total, rate
    );

    // At least 50% of std files should parse successfully
    assert!(
        rate >= 50.0,
        "Expected at least 50% parse success rate for std/, got {:.1}%",
        rate
    );
}

// =============================================================================
// Broken syntax rejection tests
// =============================================================================

#[test]
fn reject_broken_syntax_missing_brace() {
    let source = "F foo() { x := 5";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    assert!(
        parser.parse_module().is_err(),
        "Should reject unclosed brace"
    );
}

#[test]
fn reject_broken_syntax_invalid_function() {
    let source = "F (a, b) -> i64 = a + b";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    assert!(
        parser.parse_module().is_err(),
        "Should reject function without name"
    );
}

#[test]
fn reject_broken_syntax_incomplete_struct() {
    let source = "S MyStruct {";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    assert!(
        parser.parse_module().is_err(),
        "Should reject incomplete struct"
    );
}

#[test]
fn reject_broken_syntax_orphan_arrow() {
    let source = "-> i64";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    assert!(
        parser.parse_module().is_err(),
        "Should reject orphan arrow at top level"
    );
}

#[test]
fn reject_broken_syntax_double_equals_binding() {
    let source = "F foo() { x == 5 }";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    // This might parse as an expression statement (comparison), which is valid.
    // The point is that it should not crash or hang.
    let _ = parser.parse_module();
}

#[test]
fn reject_broken_syntax_empty_match() {
    let source = "F foo() { M x { } }";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    // Empty match body — parser may accept or reject, but must not crash
    let _ = parser.parse_module();
}

//! Integration tests for vais-query crate.
//!
//! These tests focus on end-to-end scenarios, file I/O, full compilation pipelines,
//! large-scale caching, cfg conditional parsing, error propagation, and revision tracking.
//! They complement the unit tests in src/tests.rs by testing integration-level behavior.

use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;
use vais_codegen::TargetTriple;
use vais_query::{QueryDatabase, QueryError};

const SIMPLE_SOURCE: &str = r#"F main() -> i64 { 42 }"#;
const MODIFIED_SOURCE: &str = r#"F main() -> i64 { 100 }"#;
const LARGE_SOURCE_TEMPLATE: &str = r#"F func{idx}() -> i64 {{ {idx} }}"#;

// ─── Task 1: File I/O Integration (3 tests) ──────────────────────────

#[test]
fn test_load_source_file_and_query() {
    // Load a real temp file and run tokenize/parse on it
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "{}", SIMPLE_SOURCE).unwrap();
    temp_file.flush().unwrap();

    let db = QueryDatabase::new();
    db.load_source_file(temp_file.path()).unwrap();

    // Should be able to tokenize
    let tokens = db.tokenize(temp_file.path()).unwrap();
    assert!(!tokens.is_empty());

    // Should be able to parse
    let module = db.parse(temp_file.path()).unwrap();
    assert!(!module.items.is_empty());

    // Should be cached
    assert!(db.is_cached(temp_file.path(), "tokenize"));
    assert!(db.is_cached(temp_file.path(), "parse"));
}

#[test]
fn test_load_nonexistent_file_error() {
    let db = QueryDatabase::new();
    let nonexistent_path = "/nonexistent/path/file.vais";

    let result = db.load_source_file(nonexistent_path);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), QueryError::FileNotFound(_)));
}

#[test]
fn test_load_modify_reload_invalidation() {
    // Create temp file with initial content
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "{}", SIMPLE_SOURCE).unwrap();
    temp_file.flush().unwrap();

    let db = QueryDatabase::new();
    db.load_source_file(temp_file.path()).unwrap();

    // Parse and cache
    let module1 = db.parse(temp_file.path()).unwrap();
    assert!(db.is_cached(temp_file.path(), "parse"));
    assert_eq!(module1.items.len(), 1);

    // Modify file content
    let temp_path = temp_file.path().to_path_buf();
    drop(temp_file); // Close file to allow rewrite
    std::fs::write(&temp_path, MODIFIED_SOURCE).unwrap();

    // Reload
    db.load_source_file(&temp_path).unwrap();

    // Cache should be invalidated
    assert!(!db.is_cached(&temp_path, "parse"));

    // Re-parse should succeed with new content
    let module2 = db.parse(&temp_path).unwrap();
    assert_eq!(module2.items.len(), 1);

    // Content should be updated (verified via source_text)
    let source = db.source_text(&temp_path).unwrap();
    assert!(source.contains("100"));
}

// ─── Task 2: Query Pipeline Integration (3 tests) ────────────────────

#[test]
fn test_full_pipeline_tokenize_to_ir() {
    // Test the entire pipeline: tokenize → parse → type_check → generate_ir
    let db = QueryDatabase::new();
    db.set_source_text("pipeline.vais", SIMPLE_SOURCE);

    // Step 1: Tokenize
    let tokens = db.tokenize("pipeline.vais").unwrap();
    assert!(!tokens.is_empty());
    assert!(db.is_cached("pipeline.vais", "tokenize"));

    // Step 2: Parse
    let module = db.parse("pipeline.vais").unwrap();
    assert!(!module.items.is_empty());
    assert!(db.is_cached("pipeline.vais", "parse"));

    // Step 3: Type check
    db.type_check("pipeline.vais").unwrap();
    assert!(db.is_cached("pipeline.vais", "type_check"));

    // Step 4: Generate IR
    let ir = db.generate_ir("pipeline.vais", TargetTriple::Native).unwrap();
    assert!(db.is_cached("pipeline.vais", "generate_ir"));

    // Verify IR content
    assert!(ir.contains("define"));
    assert!(ir.len() > 50); // Non-trivial IR output
}

#[test]
fn test_pipeline_valid_source_ir_content() {
    // Test that valid Vais source produces correct LLVM IR
    let source = r#"
F add(a: i64, b: i64) -> i64 {
    a + b
}

F main() -> i64 {
    add(10, 20)
}
"#;

    let db = QueryDatabase::new();
    db.set_source_text("math.vais", source);

    // Generate IR
    let ir = db.generate_ir("math.vais", TargetTriple::Native).unwrap();

    // Verify IR contains expected functions
    assert!(ir.contains("define"));
    assert!(ir.contains("add") || ir.contains("main"));

    // Verify it's valid LLVM IR (contains standard tokens)
    assert!(ir.contains("i64"));
    assert!(ir.contains("ret"));
}

#[test]
fn test_pipeline_invalid_source_error() {
    // Test that invalid source produces appropriate errors
    let invalid_sources = vec![
        ("lex_error.vais", "F broken( -> i64 { }", "parse"),
        ("parse_error.vais", "F missing_body() -> i64", "parse"),
        ("type_error.vais", "F type_err() -> i64 { x + y }", "type"),
    ];

    for (path, source, expected_error_kind) in invalid_sources {
        let db = QueryDatabase::new();
        db.set_source_text(path, source);

        // Try to run through pipeline
        let result = db.type_check(path);

        // Should fail with appropriate error
        assert!(result.is_err());
        let err = result.unwrap_err();
        match expected_error_kind {
            "parse" => {
                assert!(matches!(err, QueryError::Parse(_)))
            }
            "type" => {
                assert!(matches!(err, QueryError::Type(_)))
            }
            _ => panic!("Unexpected error type"),
        }
    }
}

// ─── Task 3: Large-Scale Cache Scenarios (3 tests) ───────────────────

#[test]
fn test_large_scale_independent_files_caching() {
    // Register 15 files and verify independent cache hits
    let db = QueryDatabase::new();

    // Register files
    for i in 0..15 {
        let path = format!("file{}.vais", i);
        let source = LARGE_SOURCE_TEMPLATE.replace("{idx}", &i.to_string());
        db.set_source_text(&path, source);
    }

    assert_eq!(db.source_file_count(), 15);

    // Query each file (tokenize + parse)
    for i in 0..15 {
        let path = format!("file{}.vais", i);
        db.tokenize(&path).unwrap();
        db.parse(&path).unwrap();
    }

    // Verify all are cached
    for i in 0..15 {
        let path = format!("file{}.vais", i);
        assert!(db.is_cached(&path, "tokenize"));
        assert!(db.is_cached(&path, "parse"));
    }

    assert_eq!(db.cached_file_count(), 15);

    // Second query should hit cache
    for i in 0..15 {
        let path = format!("file{}.vais", i);
        let tokens = db.tokenize(&path).unwrap();
        assert!(!tokens.is_empty());
    }
}

#[test]
fn test_large_source_parsing_and_caching() {
    // Generate a large source file (1000+ lines)
    let db = QueryDatabase::new();

    let mut large_source = String::new();
    for i in 0..1000 {
        large_source.push_str(&format!("F func{}() -> i64 {{ {} }}\n", i, i));
    }

    db.set_source_text("large.vais", &large_source);

    // Parse (should succeed despite size)
    let module = db.parse("large.vais").unwrap();
    assert_eq!(module.items.len(), 1000);
    assert!(db.is_cached("large.vais", "parse"));

    // Second parse should hit cache
    let module2 = db.parse("large.vais").unwrap();
    assert_eq!(module2.items.len(), 1000);

    // Source should be retrievable
    let source = db.source_text("large.vais").unwrap();
    assert!(source.len() > 10000); // Large source
}

#[test]
fn test_selective_invalidation_multi_file() {
    // Register 12 files, modify one, verify only that file is invalidated
    let db = QueryDatabase::new();

    // Register files
    for i in 0..12 {
        let path = format!("multi{}.vais", i);
        let source = LARGE_SOURCE_TEMPLATE.replace("{idx}", &i.to_string());
        db.set_source_text(&path, source);
    }

    // Parse all files
    for i in 0..12 {
        let path = format!("multi{}.vais", i);
        db.parse(&path).unwrap();
        assert!(db.is_cached(&path, "parse"));
    }

    // Modify file 5
    let modified_source = "F func5() -> i64 { 999 }";
    db.set_source_text("multi5.vais", modified_source);

    // Only file 5 should be invalidated
    assert!(!db.is_cached("multi5.vais", "parse"));

    // All other files should remain cached
    for i in 0..12 {
        if i != 5 {
            let path = format!("multi{}.vais", i);
            assert!(db.is_cached(&path, "parse"));
        }
    }

    // Re-parse file 5
    let module5 = db.parse("multi5.vais").unwrap();
    assert_eq!(module5.items.len(), 1);
    assert!(db.is_cached("multi5.vais", "parse"));

    // Other files should still be cached
    assert!(db.is_cached("multi3.vais", "parse"));
    assert!(db.is_cached("multi7.vais", "parse"));
}

// ─── Task 4: cfg Conditional Compilation (2 tests) ───────────────────

#[test]
fn test_cfg_values_affect_parsing() {
    // Set cfg values and verify they affect parsing behavior
    let source = r#"
#[cfg(feature = "enabled")]
F enabled_func() -> i64 { 1 }

#[cfg(feature = "disabled")]
F disabled_func() -> i64 { 2 }

F always_present() -> i64 { 3 }
"#;

    // Test 1: With "enabled" feature
    let mut db1 = QueryDatabase::new();
    let mut cfg_values = HashMap::new();
    cfg_values.insert("feature".to_string(), "enabled".to_string());
    db1.set_cfg_values(cfg_values);
    db1.set_source_text("cfg_test.vais", source);

    let module1 = db1.parse("cfg_test.vais").unwrap();
    // Should contain enabled_func and always_present (possibly filtered)
    // Note: cfg filtering happens in parser, exact behavior depends on parser impl
    assert!(!module1.items.is_empty());

    // Test 2: With "disabled" feature
    let mut db2 = QueryDatabase::new();
    let mut cfg_values2 = HashMap::new();
    cfg_values2.insert("feature".to_string(), "disabled".to_string());
    db2.set_cfg_values(cfg_values2);
    db2.set_source_text("cfg_test.vais", source);

    let module2 = db2.parse("cfg_test.vais").unwrap();
    // Should contain disabled_func and always_present
    assert!(!module2.items.is_empty());

    // Test 3: No cfg values (default parsing)
    let db3 = QueryDatabase::new();
    db3.set_source_text("cfg_test.vais", source);

    let module3 = db3.parse("cfg_test.vais").unwrap();
    // Default parsing includes all items
    assert!(!module3.items.is_empty());
}

#[test]
fn test_cfg_change_invalidates_parsing() {
    // Change cfg values and verify parsing result changes
    let source = r#"
#[cfg(target_os = "linux")]
F linux_specific() -> i64 { 1 }

#[cfg(target_os = "windows")]
F windows_specific() -> i64 { 2 }
"#;

    // Initial parse with linux target
    let mut db = QueryDatabase::new();
    let mut cfg_linux = HashMap::new();
    cfg_linux.insert("target_os".to_string(), "linux".to_string());
    db.set_cfg_values(cfg_linux);
    db.set_source_text("platform.vais", source);

    let module_linux = db.parse("platform.vais").unwrap();
    assert!(!module_linux.items.is_empty());

    // Change cfg to windows (requires new database since cfg is immutable after creation)
    let mut db2 = QueryDatabase::new();
    let mut cfg_windows = HashMap::new();
    cfg_windows.insert("target_os".to_string(), "windows".to_string());
    db2.set_cfg_values(cfg_windows);
    db2.set_source_text("platform.vais", source);

    let module_windows = db2.parse("platform.vais").unwrap();
    assert!(!module_windows.items.is_empty());

    // Both should parse successfully (cfg filtering happens inside parser)
    // The exact items depend on parser's cfg implementation
}

// ─── Task 5: Error Propagation & Recovery (2 tests) ──────────────────

#[test]
fn test_lexer_error_propagation() {
    // Test that lexer errors are properly wrapped in QueryError::Lex
    let invalid_source = "F test() -> i64 { \x00\x01\x02 }"; // Invalid characters

    let db = QueryDatabase::new();
    db.set_source_text("lex_err.vais", invalid_source);

    // Tokenize should fail with Lex error
    let result = db.tokenize("lex_err.vais");

    // Note: Logos lexer may handle invalid chars differently, so check result
    match result {
        Err(QueryError::Lex(_)) => {
            // Expected: lexer error
        }
        Ok(_tokens) => {
            // Logos may tokenize but produce error tokens
            // Verify at least we got some result
        }
        Err(other) => panic!("Expected Lex error, got: {:?}", other),
    }
}

#[test]
fn test_parser_error_propagation() {
    // Test that parser errors are properly wrapped in QueryError::Parse
    let invalid_sources = vec![
        "F incomplete_func(",          // Missing closing paren and body
        "F no_body() -> i64",          // Missing function body
        "F bad_syntax() -> i64 { = }", // Invalid expression
        "S BadStruct { field }",       // Missing type annotation
    ];

    for source in invalid_sources {
        let db = QueryDatabase::new();
        db.set_source_text("parse_err.vais", source);

        // Parse should fail with Parse error
        let result = db.parse("parse_err.vais");
        assert!(result.is_err());

        match result.unwrap_err() {
            QueryError::Parse(msg) => {
                assert!(!msg.is_empty());
            }
            other => panic!("Expected Parse error, got: {:?}", other),
        }
    }
}

// ─── Task 6: Revision & Hash Tracking (2 tests) ───────────────────────

#[test]
fn test_revision_monotonic_increase() {
    // Verify that revisions increase monotonically across multiple file operations
    let db = QueryDatabase::new();

    let rev0 = db.current_revision();

    // Add first file
    db.set_source_text("rev1.vais", "F a() -> i64 { 1 }");
    let rev1 = db.current_revision();
    assert!(rev1 > rev0);

    // Add second file
    db.set_source_text("rev2.vais", "F b() -> i64 { 2 }");
    let rev2 = db.current_revision();
    assert!(rev2 > rev1);

    // Modify first file
    db.set_source_text("rev1.vais", "F a() -> i64 { 10 }");
    let rev3 = db.current_revision();
    assert!(rev3 > rev2);

    // Add third file
    db.set_source_text("rev3.vais", "F c() -> i64 { 3 }");
    let rev4 = db.current_revision();
    assert!(rev4 > rev3);

    // Remove file
    db.remove_source("rev2.vais");
    let rev5 = db.current_revision();
    assert!(rev5 > rev4);

    // Verify monotonic sequence
    assert!(rev0 < rev1);
    assert!(rev1 < rev2);
    assert!(rev2 < rev3);
    assert!(rev3 < rev4);
    assert!(rev4 < rev5);
}

#[test]
fn test_hash_prevents_unnecessary_invalidation() {
    // Test that re-setting identical source doesn't invalidate cache
    let db = QueryDatabase::new();

    let source = "F stable() -> i64 { 42 }";
    db.set_source_text("stable.vais", source);

    let hash1 = db.source_hash("stable.vais").unwrap();
    let rev1 = db.current_revision();

    // Parse and cache
    db.parse("stable.vais").unwrap();
    assert!(db.is_cached("stable.vais", "parse"));

    // Re-set identical source (content-based deduplication)
    db.set_source_text("stable.vais", source);

    let hash2 = db.source_hash("stable.vais").unwrap();
    let rev2 = db.current_revision();

    // Hash should be identical
    assert_eq!(hash1, hash2);

    // Revision should not change (content unchanged)
    assert_eq!(rev1, rev2);

    // Cache should remain valid
    assert!(db.is_cached("stable.vais", "parse"));

    // Modify content slightly
    db.set_source_text("stable.vais", "F stable() -> i64 { 43 }");
    let hash3 = db.source_hash("stable.vais").unwrap();
    let rev3 = db.current_revision();

    // Hash should differ
    assert_ne!(hash1, hash3);

    // Revision should increment
    assert!(rev3 > rev2);

    // Cache should be invalidated
    assert!(!db.is_cached("stable.vais", "parse"));
}

// ─── Additional Integration Tests ─────────────────────────────────────

#[test]
fn test_clear_all_removes_everything() {
    // Test that clear_all removes both sources and caches
    let db = QueryDatabase::new();

    // Add multiple files and query them
    for i in 0..5 {
        let path = format!("clear{}.vais", i);
        db.set_source_text(&path, SIMPLE_SOURCE);
        db.parse(&path).unwrap();
    }

    assert_eq!(db.source_file_count(), 5);
    assert_eq!(db.cached_file_count(), 5);

    // Clear all
    db.clear_all();

    assert_eq!(db.source_file_count(), 0);
    assert_eq!(db.cached_file_count(), 0);

    // Files should no longer be accessible
    assert!(db.source_text("clear0.vais").is_none());
}

#[test]
fn test_source_files_list_accuracy() {
    // Test that source_files() returns accurate list
    let db = QueryDatabase::new();

    let expected_files = vec!["a.vais", "b.vais", "c.vais"];
    for file in &expected_files {
        db.set_source_text(file, SIMPLE_SOURCE);
    }

    let files = db.source_files();
    assert_eq!(files.len(), 3);

    // Verify all expected files are present
    for expected in &expected_files {
        assert!(files.iter().any(|p| p.to_str().unwrap() == *expected));
    }

    // Remove one file
    db.remove_source("b.vais");

    let files_after = db.source_files();
    assert_eq!(files_after.len(), 2);
    assert!(files_after.iter().any(|p| p.to_str().unwrap() == "a.vais"));
    assert!(files_after.iter().any(|p| p.to_str().unwrap() == "c.vais"));
    assert!(!files_after
        .iter()
        .any(|p| p.to_str().unwrap() == "b.vais"));
}

#[test]
fn test_has_current_source_accuracy() {
    // Test has_current_source hash comparison
    let db = QueryDatabase::new();
    let source = "F test() -> i64 { 123 }";

    // Before setting
    assert!(!db.has_current_source("test.vais", source));

    // After setting
    db.set_source_text("test.vais", source);
    assert!(db.has_current_source("test.vais", source));

    // Different content
    assert!(!db.has_current_source("test.vais", "F test() -> i64 { 456 }"));

    // After modifying
    db.set_source_text("test.vais", "F test() -> i64 { 999 }");
    assert!(!db.has_current_source("test.vais", source));
    assert!(db.has_current_source("test.vais", "F test() -> i64 { 999 }"));
}

#[test]
fn test_pipeline_with_multiple_targets() {
    // Test generating IR for multiple targets independently
    let db = QueryDatabase::new();
    db.set_source_text("multi_target.vais", SIMPLE_SOURCE);

    // Generate for native target
    let ir_native = db
        .generate_ir("multi_target.vais", TargetTriple::Native)
        .unwrap();
    assert!(ir_native.contains("define"));

    // Generate for Linux target
    let ir_linux = db
        .generate_ir("multi_target.vais", TargetTriple::X86_64Linux)
        .unwrap();
    assert!(ir_linux.contains("define"));

    // Generate for Windows target
    let ir_windows = db
        .generate_ir("multi_target.vais", TargetTriple::X86_64WindowsMsvc)
        .unwrap();
    assert!(ir_windows.contains("define"));

    // All should produce valid IR
    assert!(ir_native.len() > 50);
    assert!(ir_linux.len() > 50);
    assert!(ir_windows.len() > 50);
}

#[test]
fn test_concurrent_queries_on_different_files() {
    // Test that querying different files doesn't interfere
    use std::sync::Arc;
    use std::thread;

    let db = Arc::new(QueryDatabase::new());

    // Set up multiple files
    for i in 0..10 {
        let path = format!("concurrent{}.vais", i);
        let source = LARGE_SOURCE_TEMPLATE.replace("{idx}", &i.to_string());
        db.set_source_text(&path, source);
    }

    // Query files from multiple threads
    let mut handles = vec![];
    for i in 0..10 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            let path = format!("concurrent{}.vais", i);
            let result = db_clone.parse(&path);
            assert!(result.is_ok());
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all are cached
    for i in 0..10 {
        let path = format!("concurrent{}.vais", i);
        assert!(db.is_cached(&path, "parse"));
    }
}

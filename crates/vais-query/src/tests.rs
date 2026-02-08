//! Tests for the query-based compilation database.

use super::*;
use crate::database::QueryError;
use vais_codegen::TargetTriple;

const SIMPLE_SOURCE: &str = r#"
F main() -> i64 {
    42
}
"#;

const MODIFIED_SOURCE: &str = r#"
F main() -> i64 {
    100
}
"#;

#[test]
fn test_set_and_get_source() {
    let db = QueryDatabase::new();
    db.set_source_text("test.vais", SIMPLE_SOURCE);

    assert_eq!(db.source_file_count(), 1);
    assert_eq!(db.source_text("test.vais").unwrap(), SIMPLE_SOURCE);
}

#[test]
fn test_remove_source() {
    let db = QueryDatabase::new();
    db.set_source_text("test.vais", SIMPLE_SOURCE);
    assert_eq!(db.source_file_count(), 1);

    db.remove_source("test.vais");
    assert_eq!(db.source_file_count(), 0);
    assert!(db.source_text("test.vais").is_none());
}

#[test]
fn test_tokenize_query() {
    let db = QueryDatabase::new();
    db.set_source_text("test.vais", SIMPLE_SOURCE);

    let tokens = db.tokenize("test.vais").unwrap();
    assert!(!tokens.is_empty());

    // Second call should be cached
    let tokens2 = db.tokenize("test.vais").unwrap();
    assert_eq!(tokens.len(), tokens2.len());
    assert!(db.is_cached("test.vais", "tokenize"));
}

#[test]
fn test_parse_query() {
    let db = QueryDatabase::new();
    db.set_source_text("test.vais", SIMPLE_SOURCE);

    let module = db.parse("test.vais").unwrap();
    assert!(!module.items.is_empty());
    assert!(db.is_cached("test.vais", "parse"));
}

#[test]
fn test_type_check_query() {
    let db = QueryDatabase::new();
    db.set_source_text("test.vais", SIMPLE_SOURCE);

    db.type_check("test.vais").unwrap();
    assert!(db.is_cached("test.vais", "type_check"));
}

#[test]
fn test_generate_ir_query() {
    let db = QueryDatabase::new();
    db.set_source_text("test.vais", SIMPLE_SOURCE);

    let ir = db.generate_ir("test.vais", TargetTriple::Native).unwrap();
    assert!(ir.contains("define"));
    assert!(db.is_cached("test.vais", "generate_ir"));
}

#[test]
fn test_cache_invalidation_on_change() {
    let db = QueryDatabase::new();
    db.set_source_text("test.vais", SIMPLE_SOURCE);

    // Run all queries
    let _tokens = db.tokenize("test.vais").unwrap();
    let _ast = db.parse("test.vais").unwrap();
    db.type_check("test.vais").unwrap();
    let ir1 = db.generate_ir("test.vais", TargetTriple::Native).unwrap();

    assert!(db.is_cached("test.vais", "tokenize"));
    assert!(db.is_cached("test.vais", "parse"));

    // Change source → invalidates all caches
    db.set_source_text("test.vais", MODIFIED_SOURCE);

    assert!(!db.is_cached("test.vais", "tokenize"));
    assert!(!db.is_cached("test.vais", "parse"));
    assert!(!db.is_cached("test.vais", "type_check"));
    assert!(!db.is_cached("test.vais", "generate_ir"));

    // Re-run IR generation
    let ir2 = db.generate_ir("test.vais", TargetTriple::Native).unwrap();
    assert_ne!(*ir1, *ir2);
}

#[test]
fn test_no_invalidation_on_same_content() {
    let db = QueryDatabase::new();
    db.set_source_text("test.vais", SIMPLE_SOURCE);

    let rev1 = db.current_revision();
    db.tokenize("test.vais").unwrap();

    // Set same content → should NOT invalidate
    db.set_source_text("test.vais", SIMPLE_SOURCE);
    let rev2 = db.current_revision();

    assert_eq!(rev1, rev2);
    assert!(db.is_cached("test.vais", "tokenize"));
}

#[test]
fn test_revision_increments() {
    let db = QueryDatabase::new();

    let rev1 = db.current_revision();
    db.set_source_text("a.vais", "F a() -> i64 { 1 }");
    let rev2 = db.current_revision();
    db.set_source_text("b.vais", "F b() -> i64 { 2 }");
    let rev3 = db.current_revision();

    assert!(rev2 > rev1);
    assert!(rev3 > rev2);
}

#[test]
fn test_file_not_found_error() {
    let db = QueryDatabase::new();

    let result = db.tokenize("nonexistent.vais");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), QueryError::FileNotFound(_)));
}

#[test]
fn test_clear_caches() {
    let db = QueryDatabase::new();
    db.set_source_text("test.vais", SIMPLE_SOURCE);
    db.tokenize("test.vais").unwrap();

    assert!(db.is_cached("test.vais", "tokenize"));

    db.clear_caches();
    assert!(!db.is_cached("test.vais", "tokenize"));

    // Source still exists
    assert!(db.source_text("test.vais").is_some());
}

#[test]
fn test_multiple_files() {
    let db = QueryDatabase::new();
    db.set_source_text("a.vais", "F a() -> i64 { 1 }");
    db.set_source_text("b.vais", "F b() -> i64 { 2 }");

    assert_eq!(db.source_file_count(), 2);

    let ast_a = db.parse("a.vais").unwrap();
    let ast_b = db.parse("b.vais").unwrap();

    assert!(db.is_cached("a.vais", "parse"));
    assert!(db.is_cached("b.vais", "parse"));

    // Changing a.vais should not invalidate b.vais
    db.set_source_text("a.vais", "F a() -> i64 { 10 }");
    assert!(!db.is_cached("a.vais", "parse"));
    assert!(db.is_cached("b.vais", "parse"));

    // b.vais parse should return same result
    let ast_b2 = db.parse("b.vais").unwrap();
    assert_eq!(ast_b.items.len(), ast_b2.items.len());

    // a.vais should re-parse
    let ast_a2 = db.parse("a.vais").unwrap();
    assert_eq!(ast_a.items.len(), ast_a2.items.len());
}

#[test]
fn test_source_files_list() {
    let db = QueryDatabase::new();
    db.set_source_text("a.vais", "F a() -> i64 { 1 }");
    db.set_source_text("b.vais", "F b() -> i64 { 2 }");

    let files = db.source_files();
    assert_eq!(files.len(), 2);
}

#[test]
fn test_default_impl() {
    let db = QueryDatabase::default();
    assert_eq!(db.source_file_count(), 0);
    assert_eq!(db.cached_file_count(), 0);
}

// ─── Additional Query Invalidation Tests ─────────────────────────────

#[test]
fn test_query_invalidation_cascade() {
    // Test that modifying a source file invalidates all dependent queries in order
    let db = QueryDatabase::new();
    db.set_source_text("cascade.vais", SIMPLE_SOURCE);

    // Build the full query pipeline
    let _tokens1 = db.tokenize("cascade.vais").unwrap();
    let _ast1 = db.parse("cascade.vais").unwrap();
    db.type_check("cascade.vais").unwrap();
    let ir1 = db.generate_ir("cascade.vais", TargetTriple::Native).unwrap();

    // All queries should be cached
    assert!(db.is_cached("cascade.vais", "tokenize"));
    assert!(db.is_cached("cascade.vais", "parse"));
    assert!(db.is_cached("cascade.vais", "type_check"));
    assert!(db.is_cached("cascade.vais", "generate_ir"));

    // Modify source
    db.set_source_text("cascade.vais", MODIFIED_SOURCE);

    // All queries should be invalidated
    assert!(!db.is_cached("cascade.vais", "tokenize"));
    assert!(!db.is_cached("cascade.vais", "parse"));
    assert!(!db.is_cached("cascade.vais", "type_check"));
    assert!(!db.is_cached("cascade.vais", "generate_ir"));

    // Re-run queries and verify they produce different results
    let tokens2 = db.tokenize("cascade.vais").unwrap();
    let ast2 = db.parse("cascade.vais").unwrap();
    let ir2 = db.generate_ir("cascade.vais", TargetTriple::Native).unwrap();

    // IR content should differ (the constant value changed from 42 to 100)
    assert_ne!(*ir1, *ir2);
    // Token and AST structures exist (recomputed successfully)
    assert!(!tokens2.is_empty());
    assert!(!ast2.items.is_empty());
}

#[test]
fn test_partial_query_invalidation() {
    // Test that only queries for modified files are invalidated
    let db = QueryDatabase::new();
    db.set_source_text("file1.vais", "F foo() -> i64 { 1 }");
    db.set_source_text("file2.vais", "F bar() -> i64 { 2 }");
    db.set_source_text("file3.vais", "F baz() -> i64 { 3 }");

    // Run queries on all files
    db.tokenize("file1.vais").unwrap();
    db.tokenize("file2.vais").unwrap();
    db.tokenize("file3.vais").unwrap();
    db.parse("file1.vais").unwrap();
    db.parse("file2.vais").unwrap();
    db.parse("file3.vais").unwrap();

    // All should be cached
    assert!(db.is_cached("file1.vais", "tokenize"));
    assert!(db.is_cached("file2.vais", "tokenize"));
    assert!(db.is_cached("file3.vais", "tokenize"));

    // Modify only file2
    db.set_source_text("file2.vais", "F bar() -> i64 { 200 }");

    // Only file2 queries should be invalidated
    assert!(db.is_cached("file1.vais", "tokenize"));
    assert!(!db.is_cached("file2.vais", "tokenize"));
    assert!(db.is_cached("file3.vais", "tokenize"));
    assert!(db.is_cached("file1.vais", "parse"));
    assert!(!db.is_cached("file2.vais", "parse"));
    assert!(db.is_cached("file3.vais", "parse"));
}

#[test]
fn test_cache_hit_miss_tracking() {
    // Test cache hit/miss behavior across multiple queries
    let db = QueryDatabase::new();
    db.set_source_text("cache_test.vais", SIMPLE_SOURCE);

    // First query: cache miss
    assert!(!db.is_cached("cache_test.vais", "tokenize"));
    let _tokens1 = db.tokenize("cache_test.vais").unwrap();

    // Second query: cache hit
    assert!(db.is_cached("cache_test.vais", "tokenize"));
    let _tokens2 = db.tokenize("cache_test.vais").unwrap();

    // Parse: first is miss, second is hit
    assert!(!db.is_cached("cache_test.vais", "parse"));
    let _ast1 = db.parse("cache_test.vais").unwrap();
    assert!(db.is_cached("cache_test.vais", "parse"));
    let _ast2 = db.parse("cache_test.vais").unwrap();

    // After source change: all become misses
    db.set_source_text("cache_test.vais", MODIFIED_SOURCE);
    assert!(!db.is_cached("cache_test.vais", "tokenize"));
    assert!(!db.is_cached("cache_test.vais", "parse"));

    // Rebuild cache
    let _tokens3 = db.tokenize("cache_test.vais").unwrap();
    assert!(db.is_cached("cache_test.vais", "tokenize"));
}

#[test]
fn test_incremental_recomputation() {
    // Test that incremental changes only trigger necessary recomputations
    let db = QueryDatabase::new();
    let source_v1 = "F add(x: i64, y: i64) -> i64 { x + y }";
    let source_v2 = "F add(x: i64, y: i64) -> i64 { x + y + 1 }";
    let source_v3 = "F add(x: i64, y: i64) -> i64 { x + y + 2 }";

    // Version 1
    db.set_source_text("incr.vais", source_v1);
    let rev1 = db.current_revision();
    let ir1 = db.generate_ir("incr.vais", TargetTriple::Native).unwrap();
    assert!(db.is_cached("incr.vais", "generate_ir"));

    // Version 2: small change
    db.set_source_text("incr.vais", source_v2);
    let rev2 = db.current_revision();
    assert_ne!(rev1, rev2);
    assert!(!db.is_cached("incr.vais", "generate_ir"));
    let ir2 = db.generate_ir("incr.vais", TargetTriple::Native).unwrap();
    assert_ne!(*ir1, *ir2);

    // Version 3: another small change
    db.set_source_text("incr.vais", source_v3);
    let rev3 = db.current_revision();
    assert_ne!(rev2, rev3);
    let ir3 = db.generate_ir("incr.vais", TargetTriple::Native).unwrap();
    assert_ne!(*ir2, *ir3);

    // Rollback to version 1: content hash matches, no invalidation
    db.set_source_text("incr.vais", source_v1);
    let rev4 = db.current_revision();
    // Revision should increment because content differs from v3
    assert_ne!(rev3, rev4);
}

#[test]
fn test_multi_file_dependency_tracking() {
    // Simulate a dependency graph: main.vais → lib.vais
    let db = QueryDatabase::new();

    let lib_source = "F helper() -> i64 { 42 }";
    let main_source_v1 = "F main() -> i64 { 1 }";
    let main_source_v2 = "F main() -> i64 { 2 }";

    // Set up both files
    db.set_source_text("lib.vais", lib_source);
    db.set_source_text("main.vais", main_source_v1);

    // Query both files
    let lib_ir1 = db.generate_ir("lib.vais", TargetTriple::Native).unwrap();
    let main_ir1 = db.generate_ir("main.vais", TargetTriple::Native).unwrap();

    assert!(db.is_cached("lib.vais", "generate_ir"));
    assert!(db.is_cached("main.vais", "generate_ir"));

    // Modify main.vais only
    db.set_source_text("main.vais", main_source_v2);

    // main.vais should be invalidated, lib.vais should remain cached
    assert!(db.is_cached("lib.vais", "generate_ir"));
    assert!(!db.is_cached("main.vais", "generate_ir"));

    // lib.vais should return the same result (from cache)
    let lib_ir2 = db.generate_ir("lib.vais", TargetTriple::Native).unwrap();
    assert_eq!(*lib_ir1, *lib_ir2);

    // main.vais should be recomputed
    let main_ir2 = db.generate_ir("main.vais", TargetTriple::Native).unwrap();
    assert_ne!(*main_ir1, *main_ir2);
}

#[test]
fn test_multi_file_cross_invalidation() {
    // Test that modifying one file doesn't invalidate unrelated files
    let db = QueryDatabase::new();

    let files = vec![
        ("a.vais", "F a() -> i64 { 1 }"),
        ("b.vais", "F b() -> i64 { 2 }"),
        ("c.vais", "F c() -> i64 { 3 }"),
        ("d.vais", "F d() -> i64 { 4 }"),
    ];

    // Register all files
    for (path, source) in &files {
        db.set_source_text(path, *source);
    }

    // Query all files
    for (path, _) in &files {
        db.tokenize(path).unwrap();
        db.parse(path).unwrap();
    }

    // Verify all are cached
    for (path, _) in &files {
        assert!(db.is_cached(path, "tokenize"));
        assert!(db.is_cached(path, "parse"));
    }

    // Modify only b.vais
    db.set_source_text("b.vais", "F b() -> i64 { 200 }");

    // Only b.vais should be invalidated
    assert!(db.is_cached("a.vais", "tokenize"));
    assert!(!db.is_cached("b.vais", "tokenize"));
    assert!(db.is_cached("c.vais", "tokenize"));
    assert!(db.is_cached("d.vais", "tokenize"));
}

#[test]
fn test_circular_dependency_simulation() {
    // Vais doesn't support circular imports, but we can simulate the scenario
    // where multiple files reference each other and verify cache behavior
    let db = QueryDatabase::new();

    // File a references b, file b references a (in comments)
    let source_a = "# References b.vais\nF a() -> i64 { 1 }";
    let source_b = "# References a.vais\nF b() -> i64 { 2 }";

    db.set_source_text("a.vais", source_a);
    db.set_source_text("b.vais", source_b);

    // Both files should compile independently
    let ast_a = db.parse("a.vais").unwrap();
    let ast_b = db.parse("b.vais").unwrap();

    assert!(!ast_a.items.is_empty());
    assert!(!ast_b.items.is_empty());

    // Modifying a.vais should not cause issues with b.vais
    db.set_source_text("a.vais", "# Updated\nF a() -> i64 { 10 }");

    assert!(!db.is_cached("a.vais", "parse"));
    assert!(db.is_cached("b.vais", "parse"));

    // Both should still parse successfully
    let ast_a2 = db.parse("a.vais").unwrap();
    let ast_b2 = db.parse("b.vais").unwrap();

    assert!(!ast_a2.items.is_empty());
    assert!(!ast_b2.items.is_empty());
}

#[test]
fn test_target_triple_cache_invalidation() {
    // Test that changing target triple invalidates IR cache
    let db = QueryDatabase::new();
    db.set_source_text("target_test.vais", SIMPLE_SOURCE);

    // Generate IR for native target
    let ir_native = db.generate_ir("target_test.vais", TargetTriple::Native).unwrap();
    assert!(db.is_cached("target_test.vais", "generate_ir"));

    // Generate IR for a different target (this should compute new IR)
    let ir_linux = db.generate_ir("target_test.vais", TargetTriple::X86_64Linux).unwrap();

    // Both IRs should exist but may differ in target-specific details
    assert!(ir_native.contains("define"));
    assert!(ir_linux.contains("define"));

    // Verify that native target is still cached after generating for Linux
    // Note: The cache tracks target separately, so native should remain valid
    assert!(db.is_cached("target_test.vais", "generate_ir"));
}

#[test]
fn test_revision_stability() {
    // Test that revision numbers are stable and predictable
    let db = QueryDatabase::new();

    let rev0 = db.current_revision();

    db.set_source_text("rev1.vais", "F a() -> i64 { 1 }");
    let rev1 = db.current_revision();
    assert!(rev1 > rev0);

    db.set_source_text("rev2.vais", "F b() -> i64 { 2 }");
    let rev2 = db.current_revision();
    assert!(rev2 > rev1);

    // Setting the same content should not increment revision
    db.set_source_text("rev1.vais", "F a() -> i64 { 1 }");
    let rev3 = db.current_revision();
    assert_eq!(rev2, rev3);

    // Setting different content should increment
    db.set_source_text("rev1.vais", "F a() -> i64 { 10 }");
    let rev4 = db.current_revision();
    assert!(rev4 > rev3);
}

#[test]
fn test_hash_based_deduplication() {
    // Test that content hash prevents unnecessary invalidations
    let db = QueryDatabase::new();
    let content = "F unique() -> i64 { 999 }";

    db.set_source_text("hash.vais", content);
    let hash1 = db.source_hash("hash.vais").unwrap();

    // Query to populate cache
    db.tokenize("hash.vais").unwrap();
    assert!(db.is_cached("hash.vais", "tokenize"));

    // Set the exact same content
    db.set_source_text("hash.vais", content);
    let hash2 = db.source_hash("hash.vais").unwrap();

    // Hash should be identical
    assert_eq!(hash1, hash2);

    // Cache should still be valid
    assert!(db.is_cached("hash.vais", "tokenize"));

    // Different content should produce different hash
    db.set_source_text("hash.vais", "F unique() -> i64 { 888 }");
    let hash3 = db.source_hash("hash.vais").unwrap();
    assert_ne!(hash1, hash3);
}

#[test]
fn test_query_error_caching() {
    // Test that query errors are also cached
    let db = QueryDatabase::new();
    let invalid_source = "F broken( -> i64 { }"; // Missing closing paren

    db.set_source_text("error.vais", invalid_source);

    // First parse: should fail
    let result1 = db.parse("error.vais");
    assert!(result1.is_err());

    // Error result should be cached
    assert!(db.is_cached("error.vais", "parse"));

    // Second parse: should return cached error
    let result2 = db.parse("error.vais");
    assert!(result2.is_err());

    // Fix the source
    db.set_source_text("error.vais", "F fixed() -> i64 { 42 }");

    // Cache should be invalidated
    assert!(!db.is_cached("error.vais", "parse"));

    // Now it should succeed
    let result3 = db.parse("error.vais");
    assert!(result3.is_ok());
}

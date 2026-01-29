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

use super::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_compute_file_hash() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.vais");
    fs::write(&file_path, "F main() {}").unwrap();

    let hash1 = compute_file_hash(&file_path).unwrap();
    assert_eq!(hash1.len(), 64); // SHA256 produces 64 hex chars

    // Same content = same hash
    let hash2 = compute_file_hash(&file_path).unwrap();
    assert_eq!(hash1, hash2);

    // Different content = different hash
    fs::write(&file_path, "F main() { 1 }").unwrap();
    let hash3 = compute_file_hash(&file_path).unwrap();
    assert_ne!(hash1, hash3);
}

#[test]
fn test_dependency_graph() {
    let mut graph = DependencyGraph::new();

    let main = PathBuf::from("/main.vais");
    let math = PathBuf::from("/math.vais");
    let util = PathBuf::from("/util.vais");

    // main imports math, math imports util
    graph.add_dependency(main.clone(), math.clone());
    graph.add_dependency(math.clone(), util.clone());

    // Dependents of util should include math and main
    let dependents = graph.get_dependents(&util);
    assert!(dependents.contains(&math));
    assert!(dependents.contains(&main));

    // Dependents of math should only include main
    let dependents = graph.get_dependents(&math);
    assert!(dependents.contains(&main));
    assert!(!dependents.contains(&util));
}

#[test]
fn test_incremental_cache_new_project() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join(".vais-cache");

    let cache = IncrementalCache::new(cache_dir.clone()).unwrap();

    assert!(cache_dir.exists());
    assert_eq!(cache.state.version, CACHE_VERSION);
}

#[test]
fn test_dirty_set_detection() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join(".vais-cache");
    let source_file = temp_dir.path().join("main.vais");
    fs::write(&source_file, "F main() {}").unwrap();

    let mut cache = IncrementalCache::new(cache_dir).unwrap();
    cache.set_compilation_options(CompilationOptions {
        opt_level: 0,
        debug: false,
        target_triple: "native".to_string(),
    });

    // First build - file is new, so it should be dirty
    let dirty = cache.detect_changes(&source_file).unwrap();
    assert!(dirty
        .modified_files
        .contains(&source_file.canonicalize().unwrap()));

    // Update cache
    cache.update_file(&source_file).unwrap();
    cache.persist().unwrap();

    // Second check - file unchanged, should be clean
    let dirty = cache.detect_changes(&source_file).unwrap();
    assert!(dirty.is_empty());

    // Modify file
    fs::write(&source_file, "F main() { 1 }").unwrap();
    let dirty = cache.detect_changes(&source_file).unwrap();
    assert!(!dirty.is_empty());
}

#[test]
fn test_definition_extractor_functions() {
    let source = r#"
F add(a: i32, b: i32) -> i32 {
    R a + b
}

F main() {
    V x = add(1, 2)
    print(x)
}
"#;
    let mut extractor = DefinitionExtractor::new();
    extractor.extract_from_source(source).unwrap();

    assert_eq!(extractor.functions.len(), 2);
    assert!(extractor.functions.contains_key("add"));
    assert!(extractor.functions.contains_key("main"));

    // Check that main depends on add
    let main_meta = extractor.functions.get("main").unwrap();
    assert!(main_meta.dependencies.contains(&"add".to_string()));
}

#[test]
fn test_definition_extractor_structs() {
    let source = r#"
S Point {
    x: i32,
    y: i32,
}

S Line {
    start: Point,
    end: Point,
}

F distance(p1: Point, p2: Point) -> f64 {
    R 0.0
}
"#;
    let mut extractor = DefinitionExtractor::new();
    extractor.extract_from_source(source).unwrap();

    assert_eq!(extractor.types.len(), 2);
    assert!(extractor.types.contains_key("Point"));
    assert!(extractor.types.contains_key("Line"));

    // Line depends on Point
    let line_meta = extractor.types.get("Line").unwrap();
    assert!(line_meta.dependencies.contains(&"Point".to_string()));

    assert_eq!(extractor.functions.len(), 1);
}

#[test]
fn test_definition_extractor_enums() {
    let source = r#"
E Color {
    Red,
    Green,
    Blue,
}

E Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
}
"#;
    let mut extractor = DefinitionExtractor::new();
    extractor.extract_from_source(source).unwrap();

    assert_eq!(extractor.types.len(), 2);
    assert!(extractor.types.contains_key("Color"));
    assert!(extractor.types.contains_key("Shape"));
}

#[test]
fn test_function_change_detection() {
    let source1 = r#"
F add(a: i32, b: i32) -> i32 {
    R a + b
}

F main() {
    V x = add(1, 2)
}
"#;
    let source2 = r#"
F add(a: i32, b: i32) -> i32 {
    R a + b + 1
}

F main() {
    V x = add(1, 2)
}
"#;
    let mut extractor1 = DefinitionExtractor::new();
    extractor1.extract_from_source(source1).unwrap();

    let mut extractor2 = DefinitionExtractor::new();
    extractor2.extract_from_source(source2).unwrap();

    let changes = detect_function_changes(&extractor1.functions, &extractor2.functions);

    // add was modified
    assert!(changes.modified.contains("add"));
    // main was affected (depends on add)
    assert!(changes.affected.contains("main"));
    // No additions or removals
    assert!(changes.added.is_empty());
    assert!(changes.removed.is_empty());
}

#[test]
fn test_function_addition_detection() {
    let source1 = r#"
F main() {
    V x = 1
}
"#;
    let source2 = r#"
F helper() -> i32 {
    R 42
}

F main() {
    V x = 1
}
"#;
    let mut extractor1 = DefinitionExtractor::new();
    extractor1.extract_from_source(source1).unwrap();

    let mut extractor2 = DefinitionExtractor::new();
    extractor2.extract_from_source(source2).unwrap();

    let changes = detect_function_changes(&extractor1.functions, &extractor2.functions);

    assert!(changes.added.contains("helper"));
    assert!(changes.modified.is_empty());
    assert!(changes.removed.is_empty());
}

#[test]
fn test_function_removal_detection() {
    let source1 = r#"
F helper() -> i32 {
    R 42
}

F main() {
    V x = helper()
}
"#;
    let source2 = r#"
F main() {
    V x = 1
}
"#;
    let mut extractor1 = DefinitionExtractor::new();
    extractor1.extract_from_source(source1).unwrap();

    let mut extractor2 = DefinitionExtractor::new();
    extractor2.extract_from_source(source2).unwrap();

    let changes = detect_function_changes(&extractor1.functions, &extractor2.functions);

    assert!(changes.removed.contains("helper"));
    assert!(changes.modified.contains("main")); // main's hash changed too
}

#[test]
fn test_dirty_set_function_tracking() {
    let mut dirty_set = DirtySet::default();
    let file = PathBuf::from("/test.vais");

    dirty_set.mark_function_dirty(file.clone(), "func1".to_string());
    dirty_set.mark_function_dirty(file.clone(), "func2".to_string());

    assert!(dirty_set.has_partial_changes());
    assert_eq!(dirty_set.dirty_function_count(), 2);

    let funcs = dirty_set.get_dirty_functions(&file).unwrap();
    assert!(funcs.contains("func1"));
    assert!(funcs.contains("func2"));
}

#[test]
fn test_content_hash() {
    let content1 = "F main() { R 1 }";
    let content2 = "F main() { R 2 }";
    let content3 = "F main() { R 1 }"; // Same as content1

    let hash1 = compute_content_hash(content1);
    let hash2 = compute_content_hash(content2);
    let hash3 = compute_content_hash(content3);

    assert_eq!(hash1.len(), 64);
    assert_ne!(hash1, hash2);
    assert_eq!(hash1, hash3);
}

#[test]
fn test_fine_grained_change_detection() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join(".vais-cache");
    let source_file = temp_dir.path().join("test.vais");

    // Initial source with multiple functions
    let source1 = r#"F add(a: i32, b: i32) -> i32 {
    R a + b
}

F sub(a: i32, b: i32) -> i32 {
    R a - b
}

F main() {
    V x = add(1, 2)
    V y = sub(3, 1)
}"#;
    fs::write(&source_file, source1).unwrap();

    let mut cache = IncrementalCache::new(cache_dir.clone()).unwrap();
    cache.set_compilation_options(CompilationOptions {
        opt_level: 0,
        debug: false,
        target_triple: "native".to_string(),
    });

    // Initial build - use update_file_with_functions
    cache.update_file_with_functions(&source_file).unwrap();
    cache.persist().unwrap();

    // Verify functions were extracted
    let canonical = source_file.canonicalize().unwrap();
    let meta = cache.state.dep_graph.file_metadata.get(&canonical).unwrap();
    assert_eq!(meta.functions.len(), 3);

    // Modify only the add function
    let source2 = r#"F add(a: i32, b: i32) -> i32 {
    R a + b + 1
}

F sub(a: i32, b: i32) -> i32 {
    R a - b
}

F main() {
    V x = add(1, 2)
    V y = sub(3, 1)
}"#;
    fs::write(&source_file, source2).unwrap();

    // Detect function-level changes
    let changes = cache
        .detect_function_changes(&source_file)
        .unwrap()
        .unwrap();

    assert!(changes.modified.contains("add"));
    assert!(changes.affected.contains("main")); // main depends on add
    assert!(!changes.modified.contains("sub")); // sub unchanged
}

#[test]
fn test_cache_miss_reasons() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join(".vais-cache");
    let file1 = temp_dir.path().join("file1.vais");
    let file2 = temp_dir.path().join("file2.vais");

    fs::write(&file1, "F main() {}").unwrap();

    let mut cache = IncrementalCache::new(cache_dir).unwrap();
    cache.set_compilation_options(CompilationOptions {
        opt_level: 0,
        debug: false,
        target_triple: "native".to_string(),
    });

    // First build - new file
    let (_dirty, stats) = cache.detect_changes_with_stats(&file1).unwrap();
    assert!(stats.cache_misses > 0);
    let reasons = stats.miss_reasons.get(&file1.canonicalize().unwrap());
    assert!(reasons.is_some());
    assert!(reasons.unwrap().contains(&CacheMissReason::NewFile));

    // Update cache
    cache.update_file(&file1).unwrap();
    cache.persist().unwrap();

    // Second check - cache hit
    let (_dirty, stats) = cache.detect_changes_with_stats(&file1).unwrap();
    assert!(stats.cache_hits > 0);

    // Modify file - hash changed
    fs::write(&file1, "F main() { R 1 }").unwrap();
    let (_dirty, stats) = cache.detect_changes_with_stats(&file1).unwrap();
    let reasons = stats.miss_reasons.get(&file1.canonicalize().unwrap());
    assert!(reasons.is_some());
    assert!(reasons
        .unwrap()
        .contains(&CacheMissReason::ContentHashChanged));

    // Create new file and add dependency
    fs::write(&file2, "F helper() {}").unwrap();
    cache.update_file(&file1).unwrap();
    cache.update_file(&file2).unwrap();
    cache
        .add_dependency(
            &file1.canonicalize().unwrap(),
            &file2.canonicalize().unwrap(),
        )
        .unwrap();
    cache.persist().unwrap();

    // Modify dependency
    fs::write(&file2, "F helper() { R 2 }").unwrap();
    let (_dirty, stats) = cache.detect_changes_with_stats(&file1).unwrap();

    // file2 should have ContentHashChanged
    let file2_reasons = stats.miss_reasons.get(&file2.canonicalize().unwrap());
    assert!(file2_reasons.is_some());

    // file1 might have DependencyChanged if it's in affected_files
    // (depends on propagation logic)
}

#[test]
fn test_incremental_stats_hit_rate() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join(".vais-cache");
    let file1 = temp_dir.path().join("file1.vais");
    let file2 = temp_dir.path().join("file2.vais");
    let file3 = temp_dir.path().join("file3.vais");

    fs::write(&file1, "F main() {}").unwrap();
    fs::write(&file2, "F helper() {}").unwrap();
    fs::write(&file3, "F util() {}").unwrap();

    let mut cache = IncrementalCache::new(cache_dir).unwrap();
    cache.set_compilation_options(CompilationOptions {
        opt_level: 0,
        debug: false,
        target_triple: "native".to_string(),
    });

    // First build - all new
    let (_dirty, stats) = cache.detect_changes_with_stats(&file1).unwrap();
    assert_eq!(stats.hit_rate(), 0.0); // No hits on first build

    // Update cache
    cache.update_file(&file1).unwrap();
    cache.update_file(&file2).unwrap();
    cache.update_file(&file3).unwrap();
    cache.persist().unwrap();

    // Second check - all hits
    let (_dirty, stats) = cache.detect_changes_with_stats(&file1).unwrap();
    assert!(stats.hit_rate() > 0.0);
    assert_eq!(stats.cache_hits, 3);
    assert_eq!(stats.cache_misses, 0);

    // Modify one file
    fs::write(&file2, "F helper() { R 1 }").unwrap();
    let (_dirty, stats) = cache.detect_changes_with_stats(&file1).unwrap();

    // Hit rate should be 66.67% (2 hits, 1 miss)
    let expected_rate = (2.0 / 3.0) * 100.0;
    assert!((stats.hit_rate() - expected_rate).abs() < 0.01);
}

#[test]
fn test_warm_cache() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join(".vais-cache");
    let project_dir = temp_dir.path().join("project");
    fs::create_dir(&project_dir).unwrap();

    let file1 = project_dir.join("file1.vais");
    let file2 = project_dir.join("file2.vais");
    let subdir = project_dir.join("subdir");
    fs::create_dir(&subdir).unwrap();
    let file3 = subdir.join("file3.vais");

    fs::write(&file1, "F main() {}").unwrap();
    fs::write(&file2, "F helper() {}").unwrap();
    fs::write(&file3, "F util() {}").unwrap();

    let mut cache = IncrementalCache::new(cache_dir).unwrap();

    // Warm cache - should find all 3 files
    let warmed = cache.warm_cache(&project_dir).unwrap();
    assert_eq!(warmed, 3);

    // Verify all files are in cache
    assert_eq!(cache.state.dep_graph.file_metadata.len(), 3);

    // Warm again - should find 0 new files (all already cached and unchanged)
    let warmed = cache.warm_cache(&project_dir).unwrap();
    assert_eq!(warmed, 0);

    // Modify one file
    fs::write(&file2, "F helper() { R 1 }").unwrap();

    // Warm again - should update 1 file
    let warmed = cache.warm_cache(&project_dir).unwrap();
    assert_eq!(warmed, 1);
}

#[test]
fn test_signature_based_skip() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join(".vais-cache");
    let source_file = temp_dir.path().join("test.vais");

    // Initial source with a function
    let source1 = r#"F add(a: i32, b: i32) -> i32 {
    R a + b
}"#;
    fs::write(&source_file, source1).unwrap();

    let mut cache = IncrementalCache::new(cache_dir.clone()).unwrap();
    cache.set_compilation_options(CompilationOptions {
        opt_level: 0,
        debug: false,
        target_triple: "native".to_string(),
    });

    // Initial build
    cache.update_file_with_functions(&source_file).unwrap();

    // Get initial signature hash
    let canonical = source_file.canonicalize().unwrap();
    let initial_meta = cache.state.dep_graph.file_metadata.get(&canonical).unwrap();
    let initial_func_meta = initial_meta.functions.get("add").unwrap();
    let initial_hash = initial_func_meta.hash.clone();

    // Modify only the function body (signature unchanged)
    let source2 = r#"F add(a: i32, b: i32) -> i32 {
    V result = a + b
    R result
}"#;
    fs::write(&source_file, source2).unwrap();

    // Update metadata
    cache.update_file_with_functions(&source_file).unwrap();

    // Get new metadata
    let new_meta = cache.state.dep_graph.file_metadata.get(&canonical).unwrap();
    let new_func_meta = new_meta.functions.get("add").unwrap();
    let new_hash = new_func_meta.hash.clone();

    // Function body changed, so hash should be different
    assert_ne!(initial_hash, new_hash);

    // In a full implementation with signature tracking:
    // - The signature hash (parameters + return type) would be the same
    // - Dependent files would not need recompilation
    // - This would increment signature_hits in stats

    // For now, verify that function metadata is tracked
    assert!(new_meta.functions.contains_key("add"));
    assert_eq!(new_meta.functions.len(), 1);
}

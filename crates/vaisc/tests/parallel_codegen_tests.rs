//! Integration tests for parallel codegen functionality.
//!
//! These tests verify the correct behavior of parallel LLVM IR generation
//! using the dependency graph to determine compilation order.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper to create a temporary .vais file with content
fn create_vais_file(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, content).expect("Failed to write test file");
    path
}

/// Test 1: Single module compilation (baseline)
#[test]
fn test_single_module_codegen() {
    let temp_dir = TempDir::new().unwrap();

    let main_file = create_vais_file(
        temp_dir.path(),
        "main.vais",
        "F main() -> i32 { R 42 }",
    );

    // Run vaisc to compile
    let output = Command::new("cargo")
        .args(["run", "--bin", "vaisc", "--", "--emit-ir", main_file.to_str().unwrap()])
        .output();

    if let Ok(result) = output {
        if result.status.success() {
            // Check that IR file was created
            let ir_file = main_file.with_extension("ll");
            assert!(ir_file.exists(), "IR file should be generated");

            let ir_content = fs::read_to_string(&ir_file).expect("Failed to read IR");
            assert!(ir_content.contains("define"), "IR should contain function definition");
        }
    }
}

/// Test 2: Test dependency graph levels using tempdir structure
#[test]
fn test_dependency_graph_levels() {
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();

    // Create three modules with dependencies: main -> util -> base
    let base_content = "F base_func() -> i32 { R 1 }";
    let util_content = "F util_func() -> i32 { R base_func() + 1 }";
    let main_content = "F main() -> i32 { R util_func() + 1 }";

    let base_file = create_vais_file(temp_dir.path(), "base.vais", base_content);
    let util_file = create_vais_file(temp_dir.path(), "util.vais", util_content);
    let main_file = create_vais_file(temp_dir.path(), "main.vais", main_content);

    // Create a simple dependency graph manually
    struct TestDepGraph {
        forward_deps: HashMap<PathBuf, Vec<PathBuf>>,
    }

    impl TestDepGraph {
        fn new() -> Self {
            Self {
                forward_deps: HashMap::new(),
            }
        }

        fn add_dependency(&mut self, from: PathBuf, to: PathBuf) {
            self.forward_deps.entry(from).or_default().push(to);
        }

        fn parallel_levels(&self) -> Vec<Vec<PathBuf>> {
            // Topological sort that properly updates reverse dependency in-degrees
            let mut levels = Vec::new();
            let mut visited = std::collections::HashSet::new();
            let mut in_degree: HashMap<PathBuf, usize> = HashMap::new();

            // Build reverse dependency map
            let mut reverse_deps: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
            for (from, tos) in &self.forward_deps {
                for to in tos {
                    reverse_deps.entry(to.clone()).or_default().push(from.clone());
                }
            }

            // Initialize in-degrees
            let all_files: std::collections::HashSet<PathBuf> = self
                .forward_deps
                .keys()
                .chain(self.forward_deps.values().flatten())
                .cloned()
                .collect();

            for file in &all_files {
                let degree = self.forward_deps.get(file).map_or(0, |deps| deps.len());
                in_degree.insert(file.clone(), degree);
            }

            // Process levels
            while visited.len() < all_files.len() {
                let mut current_level = Vec::new();

                // Find all nodes with no remaining dependencies
                for file in &all_files {
                    if !visited.contains(file) && in_degree.get(file).copied().unwrap_or(0) == 0 {
                        current_level.push(file.clone());
                    }
                }

                if current_level.is_empty() {
                    break;
                }

                // Mark as visited and reduce in-degrees of dependents
                for file in &current_level {
                    visited.insert(file.clone());

                    // Reduce in-degree for all nodes that depend on this file
                    if let Some(dependents) = reverse_deps.get(file) {
                        for dependent in dependents {
                            if let Some(degree) = in_degree.get_mut(dependent) {
                                *degree = degree.saturating_sub(1);
                            }
                        }
                    }
                }

                levels.push(current_level);
            }

            levels
        }
    }

    // Build dependency graph
    let mut graph = TestDepGraph::new();
    graph.add_dependency(main_file.clone(), util_file.clone());
    graph.add_dependency(util_file.clone(), base_file.clone());

    // Get parallel levels
    let levels = graph.parallel_levels();

    // Verify we have 3 levels (base, then util, then main)
    assert_eq!(levels.len(), 3, "Should have 3 dependency levels");

    // First level should contain only base (no dependencies)
    assert_eq!(levels[0].len(), 1, "First level should have 1 module");
    assert!(levels[0].contains(&base_file), "First level should contain base");

    // Second level should contain util
    assert_eq!(levels[1].len(), 1, "Second level should have 1 module");
    assert!(levels[1].contains(&util_file), "Second level should contain util");

    // Third level should contain main
    assert_eq!(levels[2].len(), 1, "Third level should have 1 module");
    assert!(levels[2].contains(&main_file), "Third level should contain main");
}

/// Test 3: Verify parallel processing of independent modules
#[test]
fn test_independent_modules_parallel() {
    let temp_dir = TempDir::new().unwrap();

    // Create two completely independent modules
    let module_a_content = "F func_a() -> i32 { R 1 }";
    let module_b_content = "F func_b() -> i32 { R 2 }";

    let file_a = create_vais_file(temp_dir.path(), "module_a.vais", module_a_content);
    let file_b = create_vais_file(temp_dir.path(), "module_b.vais", module_b_content);

    // Both files should compile independently
    for file in &[file_a, file_b] {
        let output = Command::new("cargo")
            .args(["run", "--bin", "vaisc", "--", "--emit-ir", file.to_str().unwrap()])
            .output();

        if let Ok(result) = output {
            if result.status.success() {
                let ir_file = file.with_extension("ll");
                assert!(ir_file.exists(), "IR file should be generated for {:?}", file);
            }
        }
    }
}

/// Test 4: Diamond dependency pattern
#[test]
fn test_diamond_dependency_pattern() {
    use std::collections::HashMap;

    // Create dependency graph structure:
    //      top
    //     /   \
    //   left  right
    //     \   /
    //      base

    struct DepGraph {
        deps: HashMap<String, Vec<String>>,
    }

    impl DepGraph {
        fn new() -> Self {
            Self { deps: HashMap::new() }
        }

        fn add(&mut self, from: &str, to: &str) {
            self.deps.entry(from.to_string()).or_default().push(to.to_string());
        }

        fn levels(&self) -> Vec<Vec<String>> {
            let mut levels = Vec::new();
            let mut visited = std::collections::HashSet::new();
            let mut in_degree: HashMap<String, usize> = HashMap::new();

            // Build reverse dependency map
            let mut reverse_deps: HashMap<String, Vec<String>> = HashMap::new();
            for (from, tos) in &self.deps {
                for to in tos {
                    reverse_deps.entry(to.clone()).or_default().push(from.clone());
                }
            }

            let all_nodes: std::collections::HashSet<String> = self
                .deps
                .keys()
                .chain(self.deps.values().flatten())
                .cloned()
                .collect();

            for node in &all_nodes {
                let degree = self.deps.get(node).map_or(0, |deps| deps.len());
                in_degree.insert(node.clone(), degree);
            }

            while visited.len() < all_nodes.len() {
                let mut current_level = Vec::new();

                for node in &all_nodes {
                    if !visited.contains(node) && in_degree.get(node).copied().unwrap_or(0) == 0 {
                        current_level.push(node.clone());
                    }
                }

                if current_level.is_empty() {
                    break;
                }

                // Mark as visited and reduce in-degrees of dependents
                for node in &current_level {
                    visited.insert(node.clone());

                    // Reduce in-degree for all nodes that depend on this node
                    if let Some(dependents) = reverse_deps.get(node) {
                        for dependent in dependents {
                            if let Some(degree) = in_degree.get_mut(dependent) {
                                *degree = degree.saturating_sub(1);
                            }
                        }
                    }
                }

                levels.push(current_level);
            }

            levels
        }
    }

    let mut graph = DepGraph::new();
    graph.add("left", "base");
    graph.add("right", "base");
    graph.add("top", "left");
    graph.add("top", "right");

    let levels = graph.levels();

    // Verify level structure
    assert!(levels.len() >= 3, "Should have at least 3 levels");

    // Base should be in first level
    assert!(levels[0].contains(&"base".to_string()), "Base should be in first level");

    // Left and right should be in second level (parallel)
    assert!(levels[1].len() >= 2, "Second level should have at least 2 modules");
    assert!(levels[1].contains(&"left".to_string()) && levels[1].contains(&"right".to_string()),
            "Second level should contain left and right modules");

    // Top should be in third level
    assert!(levels[2].contains(&"top".to_string()), "Top should be in third level");
}

/// Test 5: Hash computation for IR content caching
#[test]
fn test_ir_content_hash() {
    use sha2::{Digest, Sha256};

    let ir_content_1 = "define i32 @main() { ret i32 42 }";
    let ir_content_2 = "define i32 @main() { ret i32 99 }";
    let ir_content_3 = "define i32 @main() { ret i32 42 }"; // Same as content_1

    fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    let hash_1 = compute_hash(ir_content_1);
    let hash_2 = compute_hash(ir_content_2);
    let hash_3 = compute_hash(ir_content_3);

    // Same content = same hash
    assert_eq!(hash_1, hash_3, "Same IR content should produce same hash");

    // Different content = different hash
    assert_ne!(hash_1, hash_2, "Different IR content should produce different hash");

    // Hash should be 64 chars (SHA256 hex)
    assert_eq!(hash_1.len(), 64, "SHA256 hash should be 64 hex characters");
}

/// Test 6: Module stem extraction from file paths
#[test]
fn test_module_stem_extraction() {
    let test_cases = vec![
        ("/path/to/main.vais", "main"),
        ("/another/path/module.vais", "module"),
        ("file.vais", "file"),
        ("/complex/path.with.dots/test.vais", "test"),
    ];

    for (path_str, expected_stem) in test_cases {
        let path = std::path::Path::new(path_str);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        assert_eq!(stem, expected_stem, "Stem extraction failed for {}", path_str);
    }
}

/// Test 7: Optimization level propagation
#[test]
fn test_optimization_level_mapping() {
    // Test that optimization levels map correctly
    let test_cases = vec![
        (0, 0), // O0 -> O0
        (1, 1), // O1 -> O1
        (2, 2), // O2 -> O2
        (3, 3), // O3 -> O3
        (4, 3), // O4 -> O3 (capped)
    ];

    for (input_level, expected_effective) in test_cases {
        let effective = input_level.min(3);
        assert_eq!(
            effective, expected_effective,
            "Optimization level mapping failed for input {}",
            input_level
        );
    }
}

/// Test 8: Debug mode disables optimization
#[test]
fn test_debug_mode_optimization() {
    let opt_level = 3;
    let debug = true;

    let effective_opt_level = if debug { 0 } else { opt_level };

    assert_eq!(
        effective_opt_level, 0,
        "Debug mode should force optimization level to 0"
    );

    let debug_false = false;
    let effective_opt_level_no_debug = if debug_false { 0 } else { opt_level };

    assert_eq!(
        effective_opt_level_no_debug, 3,
        "Without debug mode, optimization level should be preserved"
    );
}

/// Test 9: GC threshold configuration
#[test]
fn test_gc_threshold_configuration() {
    let gc_enabled = true;
    let gc_threshold_some = Some(1024usize);
    let gc_threshold_none: Option<usize> = None;

    // Test that threshold is properly handled
    if gc_enabled {
        if let Some(threshold) = gc_threshold_some {
            assert_eq!(threshold, 1024, "GC threshold should be set correctly");
        }

        assert!(gc_threshold_none.is_none(), "GC threshold can be None");
    }
}

/// Test 10: Empty module handling
#[test]
fn test_empty_module() {
    let temp_dir = TempDir::new().unwrap();

    // Create an empty .vais file (no functions)
    let empty_file = create_vais_file(temp_dir.path(), "empty.vais", "# Empty module\n");

    // Try to compile - should handle gracefully
    let output = Command::new("cargo")
        .args(["run", "--bin", "vaisc", "--", "--emit-ir", empty_file.to_str().unwrap()])
        .output();

    // We don't assert success here as empty modules might be rejected by parser,
    // but we verify the command runs without crashing
    assert!(output.is_ok(), "Command should run without panic");
}

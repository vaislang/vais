//! Unit tests for parallel type-checking functionality.
//!
//! These tests verify the dependency graph topological sorting and parallel level computation,
//! which are the foundation for parallel type-checking.

use std::collections::HashSet;
use std::path::PathBuf;

// Re-create DependencyGraph locally for testing since vaisc is a binary crate
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
struct DependencyGraph {
    forward_deps: HashMap<PathBuf, Vec<PathBuf>>,
    reverse_deps: HashMap<PathBuf, Vec<PathBuf>>,
}

impl DependencyGraph {
    fn new() -> Self {
        Self::default()
    }

    fn add_dependency(&mut self, from: PathBuf, to: PathBuf) {
        self.forward_deps
            .entry(from.clone())
            .or_default()
            .push(to.clone());
        self.reverse_deps.entry(to).or_default().push(from);
    }

    fn topological_sort(&self) -> Vec<Vec<PathBuf>> {
        let mut levels: Vec<Vec<PathBuf>> = Vec::new();
        let mut visited: HashSet<PathBuf> = HashSet::new();
        let mut in_degree: HashMap<PathBuf, usize> = HashMap::new();

        let all_files: HashSet<PathBuf> = self
            .forward_deps
            .keys()
            .chain(self.reverse_deps.keys())
            .cloned()
            .collect();

        for file in &all_files {
            let degree = self.forward_deps.get(file).map_or(0, |deps| deps.len());
            in_degree.insert(file.clone(), degree);
        }

        while visited.len() < all_files.len() {
            let mut current_level: Vec<PathBuf> = Vec::new();

            for file in &all_files {
                if !visited.contains(file) && in_degree.get(file).copied().unwrap_or(0) == 0 {
                    current_level.push(file.clone());
                }
            }

            if current_level.is_empty() {
                for file in &all_files {
                    if !visited.contains(file) {
                        current_level.push(file.clone());
                    }
                }
            }

            for file in &current_level {
                visited.insert(file.clone());

                if let Some(dependents) = self.reverse_deps.get(file) {
                    for dependent in dependents {
                        if let Some(degree) = in_degree.get_mut(dependent) {
                            *degree = degree.saturating_sub(1);
                        }
                    }
                }
            }

            if !current_level.is_empty() {
                levels.push(current_level);
            } else {
                break;
            }
        }

        levels
    }

    fn parallel_levels(&self) -> Vec<Vec<PathBuf>> {
        self.topological_sort()
    }
}

/// Test topological_sort on DependencyGraph with linear chain
#[test]
fn test_dependency_graph_topological_sort() {
    let mut dep_graph = DependencyGraph::new();

    let path_a = PathBuf::from("/test/a.vais");
    let path_b = PathBuf::from("/test/b.vais");
    let path_c = PathBuf::from("/test/c.vais");
    let path_d = PathBuf::from("/test/d.vais");

    // Dependencies: D -> C -> B -> A
    // A is leaf (no dependencies)
    // B depends on A
    // C depends on B
    // D depends on C
    dep_graph.add_dependency(path_b.clone(), path_a.clone());
    dep_graph.add_dependency(path_c.clone(), path_b.clone());
    dep_graph.add_dependency(path_d.clone(), path_c.clone());

    let levels = dep_graph.topological_sort();

    assert_eq!(levels.len(), 4, "Should have 4 levels");

    // Level 0 should contain A (no dependencies)
    assert!(
        levels[0].contains(&path_a),
        "Level 0 should contain leaf module A"
    );

    // Level 1 should contain B (depends only on A)
    assert!(
        levels[1].contains(&path_b),
        "Level 1 should contain module B"
    );

    // Level 2 should contain C
    assert!(
        levels[2].contains(&path_c),
        "Level 2 should contain module C"
    );

    // Level 3 should contain D
    assert!(
        levels[3].contains(&path_d),
        "Level 3 should contain module D"
    );
}

/// Test parallel_levels method with diamond dependency
#[test]
fn test_dependency_graph_parallel_levels() {
    let mut dep_graph = DependencyGraph::new();

    // Create a diamond dependency pattern:
    //     A
    //    / \
    //   B   C
    //    \ /
    //     D
    let path_a = PathBuf::from("/test/a.vais");
    let path_b = PathBuf::from("/test/b.vais");
    let path_c = PathBuf::from("/test/c.vais");
    let path_d = PathBuf::from("/test/d.vais");

    dep_graph.add_dependency(path_b.clone(), path_a.clone());
    dep_graph.add_dependency(path_c.clone(), path_a.clone());
    dep_graph.add_dependency(path_d.clone(), path_b.clone());
    dep_graph.add_dependency(path_d.clone(), path_c.clone());

    let levels = dep_graph.parallel_levels();

    assert_eq!(levels.len(), 3, "Should have 3 levels");

    // Level 0: A (no dependencies)
    assert_eq!(levels[0].len(), 1);
    assert!(levels[0].contains(&path_a));

    // Level 1: B and C can be processed in parallel
    assert_eq!(levels[1].len(), 2);
    assert!(levels[1].contains(&path_b));
    assert!(levels[1].contains(&path_c));

    // Level 2: D (depends on both B and C)
    assert_eq!(levels[2].len(), 1);
    assert!(levels[2].contains(&path_d));
}

/// Test empty dependency graph
#[test]
fn test_dependency_graph_empty() {
    let dep_graph = DependencyGraph::new();
    let levels = dep_graph.topological_sort();
    assert_eq!(levels.len(), 0, "Empty graph should produce 0 levels");
}

/// Test independent modules (no dependencies)
#[test]
fn test_dependency_graph_independent_modules() {
    let mut dep_graph = DependencyGraph::new();

    let path_a = PathBuf::from("/test/a.vais");
    let path_b = PathBuf::from("/test/b.vais");
    let path_c = PathBuf::from("/test/c.vais");

    // No dependencies - all modules are independent
    // Just add them as nodes by adding empty dependencies
    dep_graph.forward_deps.insert(path_a.clone(), vec![]);
    dep_graph.forward_deps.insert(path_b.clone(), vec![]);
    dep_graph.forward_deps.insert(path_c.clone(), vec![]);

    let levels = dep_graph.topological_sort();

    assert_eq!(levels.len(), 1, "Independent modules should be in one level");
    assert_eq!(
        levels[0].len(),
        3,
        "All 3 modules should be in the same level"
    );
    assert!(levels[0].contains(&path_a));
    assert!(levels[0].contains(&path_b));
    assert!(levels[0].contains(&path_c));
}

/// Test complex dependency graph with multiple parallel opportunities
#[test]
fn test_dependency_graph_complex() {
    let mut dep_graph = DependencyGraph::new();

    //     A   B
    //    /|\ /|
    //   C D E F
    //    \|/
    //     G
    let path_a = PathBuf::from("/test/a.vais");
    let path_b = PathBuf::from("/test/b.vais");
    let path_c = PathBuf::from("/test/c.vais");
    let path_d = PathBuf::from("/test/d.vais");
    let path_e = PathBuf::from("/test/e.vais");
    let path_f = PathBuf::from("/test/f.vais");
    let path_g = PathBuf::from("/test/g.vais");

    dep_graph.add_dependency(path_c.clone(), path_a.clone());
    dep_graph.add_dependency(path_d.clone(), path_a.clone());
    dep_graph.add_dependency(path_e.clone(), path_a.clone());
    dep_graph.add_dependency(path_e.clone(), path_b.clone());
    dep_graph.add_dependency(path_f.clone(), path_b.clone());
    dep_graph.add_dependency(path_g.clone(), path_c.clone());
    dep_graph.add_dependency(path_g.clone(), path_d.clone());
    dep_graph.add_dependency(path_g.clone(), path_e.clone());

    let levels = dep_graph.topological_sort();

    assert_eq!(levels.len(), 3, "Should have 3 levels");

    // Level 0: A and B (no dependencies)
    assert_eq!(levels[0].len(), 2);
    assert!(levels[0].contains(&path_a));
    assert!(levels[0].contains(&path_b));

    // Level 1: C, D, E, F (all depend only on level 0)
    assert_eq!(levels[1].len(), 4);
    assert!(levels[1].contains(&path_c));
    assert!(levels[1].contains(&path_d));
    assert!(levels[1].contains(&path_e));
    assert!(levels[1].contains(&path_f));

    // Level 2: G (depends on C, D, E)
    assert_eq!(levels[2].len(), 1);
    assert!(levels[2].contains(&path_g));
}

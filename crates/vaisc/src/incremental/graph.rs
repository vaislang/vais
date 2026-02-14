use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::types::FileMetadata;

/// Dependency graph for tracking file relationships
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// File -> files it imports (forward deps)
    pub forward_deps: HashMap<PathBuf, Vec<PathBuf>>,
    /// File -> files that import it (reverse deps)
    pub reverse_deps: HashMap<PathBuf, Vec<PathBuf>>,
    /// File metadata (hash, timestamp, size)
    pub file_metadata: HashMap<PathBuf, FileMetadata>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a dependency: `from` imports `to`
    pub fn add_dependency(&mut self, from: PathBuf, to: PathBuf) {
        // Forward dependency
        self.forward_deps
            .entry(from.clone())
            .or_default()
            .push(to.clone());

        // Reverse dependency
        self.reverse_deps.entry(to).or_default().push(from);
    }

    /// Get all files that depend on the given file (directly or indirectly)
    pub fn get_dependents(&self, file: &Path) -> HashSet<PathBuf> {
        let mut dependents = HashSet::new();
        let mut queue = vec![file.to_path_buf()];

        while let Some(current) = queue.pop() {
            if let Some(deps) = self.reverse_deps.get(&current) {
                for dep in deps {
                    if !dependents.contains(dep) {
                        dependents.insert(dep.clone());
                        queue.push(dep.clone());
                    }
                }
            }
        }

        dependents
    }

    /// Update file metadata
    pub fn update_file_metadata(&mut self, path: PathBuf, metadata: FileMetadata) {
        self.file_metadata.insert(path, metadata);
    }

    /// Clear all dependencies for a file (before re-adding)
    pub fn clear_file_deps(&mut self, file: &Path) {
        // Remove from forward deps
        if let Some(imports) = self.forward_deps.remove(file) {
            // Remove from reverse deps of imported files
            for imported in imports {
                if let Some(importers) = self.reverse_deps.get_mut(&imported) {
                    importers.retain(|p| p != file);
                }
            }
        }
    }

    /// Compute topological sort of modules (for sequential processing).
    /// Returns levels where modules in the same level can be processed in parallel.
    /// Each level contains modules that only depend on modules from previous levels.
    ///
    /// # Returns
    ///
    /// A vector of levels, where each level is a vector of file paths that can be
    /// processed in parallel. Dependencies are satisfied from outer to inner levels.
    pub fn topological_sort(&self) -> Vec<Vec<PathBuf>> {
        let mut levels: Vec<Vec<PathBuf>> = Vec::new();
        let mut visited: HashSet<PathBuf> = HashSet::new();
        let mut in_degree: HashMap<PathBuf, usize> = HashMap::new();

        // Initialize in-degrees for all files
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

        // Process levels until all files are visited
        while visited.len() < all_files.len() {
            let mut current_level: Vec<PathBuf> = Vec::new();

            // Find all files with in-degree 0 (no unprocessed dependencies)
            for file in &all_files {
                if !visited.contains(file) && in_degree.get(file).copied().unwrap_or(0) == 0 {
                    current_level.push(file.clone());
                }
            }

            // If no files can be processed, we have a circular dependency
            if current_level.is_empty() {
                // Collect remaining files into a single level (SCC handling)
                for file in &all_files {
                    if !visited.contains(file) {
                        current_level.push(file.clone());
                    }
                }
            }

            // Mark current level as visited and update in-degrees
            for file in &current_level {
                visited.insert(file.clone());

                // Reduce in-degree for files that depend on this file
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
                break; // Avoid infinite loop
            }
        }

        levels
    }

    /// Compute parallel levels with SCC (Strongly Connected Component) grouping.
    /// This is similar to topological_sort but explicitly handles cycles.
    ///
    /// # Returns
    ///
    /// A vector of levels, where each level contains modules that can be processed
    /// in parallel. Circular dependencies are grouped into a single level entry.
    pub fn parallel_levels(&self) -> Vec<Vec<PathBuf>> {
        let sccs = self.find_sccs();

        // Build a condensation graph: SCC nodes with edges between SCCs
        let mut scc_index_map: HashMap<PathBuf, usize> = HashMap::new();
        for (scc_id, scc) in sccs.iter().enumerate() {
            for file in scc {
                scc_index_map.insert(file.clone(), scc_id);
            }
        }

        // Build edges between SCCs
        let mut scc_forward_deps: HashMap<usize, HashSet<usize>> = HashMap::new();
        let mut scc_in_degree: HashMap<usize, usize> = HashMap::new();

        for (scc_id, _scc) in sccs.iter().enumerate() {
            scc_in_degree.insert(scc_id, 0);
            scc_forward_deps.insert(scc_id, HashSet::new());
        }

        for (from_file, to_files) in &self.forward_deps {
            if let Some(&from_scc) = scc_index_map.get(from_file) {
                for to_file in to_files {
                    if let Some(&to_scc) = scc_index_map.get(to_file) {
                        if from_scc != to_scc {
                            // from_file -> to_file means from_file depends on to_file
                            // So to_scc must be processed before from_scc
                            // This means we add an edge to_scc -> from_scc in the condensation
                            // (reverse the edge direction for processing order)
                            if scc_forward_deps.get_mut(&to_scc).unwrap().insert(from_scc) {
                                *scc_in_degree.get_mut(&from_scc).unwrap() += 1;
                            }
                        }
                    }
                }
            }
        }

        // Topological sort of SCCs
        let mut levels: Vec<Vec<PathBuf>> = Vec::new();
        let mut visited: HashSet<usize> = HashSet::new();

        while visited.len() < sccs.len() {
            let mut current_level: Vec<PathBuf> = Vec::new();

            // Find all SCCs with in-degree 0
            for (scc_id, scc) in sccs.iter().enumerate() {
                if !visited.contains(&scc_id) && *scc_in_degree.get(&scc_id).unwrap() == 0 {
                    current_level.extend(scc.clone());
                }
            }

            if current_level.is_empty() {
                break; // Should not happen if SCCs are correct
            }

            // Mark SCCs as visited and reduce in-degrees
            let visited_sccs: Vec<usize> = current_level
                .iter()
                .filter_map(|file| scc_index_map.get(file).copied())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            for &scc_id in &visited_sccs {
                visited.insert(scc_id);
                if let Some(deps) = scc_forward_deps.get(&scc_id) {
                    for &dep_scc in deps {
                        if let Some(degree) = scc_in_degree.get_mut(&dep_scc) {
                            *degree = degree.saturating_sub(1);
                        }
                    }
                }
            }

            levels.push(current_level);
        }

        levels
    }

    /// Find strongly connected components using Tarjan's algorithm.
    ///
    /// # Returns
    ///
    /// A vector of SCCs, where each SCC is a vector of file paths.
    /// Files within the same SCC are mutually reachable (circular dependency).
    pub fn find_sccs(&self) -> Vec<Vec<PathBuf>> {
        let mut state = TarjanState::new();
        let all_files: HashSet<PathBuf> = self
            .forward_deps
            .keys()
            .chain(self.reverse_deps.keys())
            .cloned()
            .collect();

        for file in all_files {
            if !state.index.contains_key(&file) {
                self.tarjan_visit(&file, &mut state);
            }
        }

        state.sccs
    }

    /// Check if a file is part of a circular dependency.
    pub fn is_in_cycle(&self, file: &Path) -> bool {
        let sccs = self.find_sccs();
        for scc in sccs {
            if scc.len() > 1 && scc.iter().any(|f| f == file) {
                return true;
            }
        }
        false
    }

    /// Tarjan's SCC algorithm - recursive visit
    fn tarjan_visit(&self, file: &PathBuf, state: &mut TarjanState) {
        let current_index = state.index_counter;
        state.index.insert(file.clone(), current_index);
        state.low_link.insert(file.clone(), current_index);
        state.index_counter += 1;
        state.stack.push(file.clone());
        state.on_stack.insert(file.clone());

        // Visit all dependencies
        if let Some(deps) = self.forward_deps.get(file) {
            for dep in deps {
                if !state.index.contains_key(dep) {
                    // Recurse
                    self.tarjan_visit(dep, state);
                    let dep_low = *state.low_link.get(dep).unwrap();
                    let file_low = state.low_link.get_mut(file).unwrap();
                    *file_low = (*file_low).min(dep_low);
                } else if state.on_stack.contains(dep) {
                    // Back edge
                    let dep_index = *state.index.get(dep).unwrap();
                    let file_low = state.low_link.get_mut(file).unwrap();
                    *file_low = (*file_low).min(dep_index);
                }
            }
        }

        // If this is a root node, pop the SCC
        if state.low_link.get(file) == state.index.get(file) {
            let mut scc = Vec::new();
            loop {
                let node = state.stack.pop()
                    .expect("BUG: Tarjan stack underflow - algorithm invariant violated");
                state.on_stack.remove(&node);
                scc.push(node.clone());
                if node == *file {
                    break;
                }
            }
            state.sccs.push(scc);
        }
    }
}

/// State for Tarjan's SCC algorithm
struct TarjanState {
    index: HashMap<PathBuf, usize>,
    low_link: HashMap<PathBuf, usize>,
    index_counter: usize,
    stack: Vec<PathBuf>,
    on_stack: HashSet<PathBuf>,
    sccs: Vec<Vec<PathBuf>>,
}

impl TarjanState {
    fn new() -> Self {
        Self {
            index: HashMap::new(),
            low_link: HashMap::new(),
            index_counter: 0,
            stack: Vec::new(),
            on_stack: HashSet::new(),
            sccs: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_sccs_no_cycle() {
        let mut graph = DependencyGraph::new();
        let a = PathBuf::from("a.vais");
        let b = PathBuf::from("b.vais");
        let c = PathBuf::from("c.vais");

        graph.add_dependency(a.clone(), b.clone());
        graph.add_dependency(b.clone(), c.clone());

        let sccs = graph.find_sccs();

        // Each file should be in its own SCC
        assert_eq!(sccs.len(), 3);
        for scc in &sccs {
            assert_eq!(scc.len(), 1);
        }
    }

    #[test]
    fn test_find_sccs_simple_cycle() {
        let mut graph = DependencyGraph::new();
        let a = PathBuf::from("a.vais");
        let b = PathBuf::from("b.vais");

        graph.add_dependency(a.clone(), b.clone());
        graph.add_dependency(b.clone(), a.clone());

        let sccs = graph.find_sccs();

        // Should have 1 SCC with 2 files
        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0].len(), 2);
        assert!(sccs[0].contains(&a));
        assert!(sccs[0].contains(&b));
    }

    #[test]
    fn test_find_sccs_complex() {
        let mut graph = DependencyGraph::new();
        let a = PathBuf::from("a.vais");
        let b = PathBuf::from("b.vais");
        let c = PathBuf::from("c.vais");
        let d = PathBuf::from("d.vais");
        let e = PathBuf::from("e.vais");

        // Cycle: A -> B -> C -> A
        graph.add_dependency(a.clone(), b.clone());
        graph.add_dependency(b.clone(), c.clone());
        graph.add_dependency(c.clone(), a.clone());

        // D -> A
        graph.add_dependency(d.clone(), a.clone());

        // D -> E
        graph.add_dependency(d.clone(), e.clone());

        let sccs = graph.find_sccs();

        // Should have 3 SCCs: {A,B,C}, {D}, {E}
        assert_eq!(sccs.len(), 3);

        // Find the SCC with A, B, C
        let abc_scc = sccs.iter().find(|scc| scc.contains(&a)).unwrap();
        assert_eq!(abc_scc.len(), 3);
        assert!(abc_scc.contains(&a));
        assert!(abc_scc.contains(&b));
        assert!(abc_scc.contains(&c));

        // D and E should each be in their own SCC
        let d_scc = sccs.iter().find(|scc| scc.contains(&d)).unwrap();
        assert_eq!(d_scc.len(), 1);

        let e_scc = sccs.iter().find(|scc| scc.contains(&e)).unwrap();
        assert_eq!(e_scc.len(), 1);
    }

    #[test]
    fn test_parallel_levels_with_scc() {
        let mut graph = DependencyGraph::new();
        let a = PathBuf::from("a.vais");
        let b = PathBuf::from("b.vais");
        let c = PathBuf::from("c.vais");
        let d = PathBuf::from("d.vais");

        // Cycle: A -> B -> A
        graph.add_dependency(a.clone(), b.clone());
        graph.add_dependency(b.clone(), a.clone());

        // C -> A
        graph.add_dependency(c.clone(), a.clone());

        // D -> C
        graph.add_dependency(d.clone(), c.clone());

        let levels = graph.parallel_levels();

        // The levels depend on the dependency direction
        // A -> B means A depends on B (A imports B), so B should come first
        // Level 0: Files with no dependencies (or SCC with external deps)
        // Level 1: Files depending on level 0
        // etc.

        // Given the graph:
        // - A and B are in a cycle (SCC)
        // - C depends on A (so C needs A)
        // - D depends on C (so D needs C)
        // The SCC {A, B} should be at level 0
        // C should be at level 1
        // D should be at level 2

        assert!(levels.len() >= 2, "Expected at least 2 levels, got {}", levels.len());

        // Find which level contains A and B
        let ab_level = levels.iter().position(|level| level.contains(&a) && level.contains(&b));
        assert!(ab_level.is_some(), "A and B should be in the same level");

        // Find which level contains C
        let c_level = levels.iter().position(|level| level.contains(&c));
        assert!(c_level.is_some(), "C should be in a level");

        // Find which level contains D
        let d_level = levels.iter().position(|level| level.contains(&d));
        assert!(d_level.is_some(), "D should be in a level");

        // C should be after the AB SCC
        assert!(c_level.unwrap() > ab_level.unwrap(),
                "C (level {}) should be after A/B (level {})",
                c_level.unwrap(), ab_level.unwrap());

        // D should be after C
        assert!(d_level.unwrap() > c_level.unwrap(),
                "D (level {}) should be after C (level {})",
                d_level.unwrap(), c_level.unwrap());
    }

    #[test]
    fn test_is_in_cycle() {
        let mut graph = DependencyGraph::new();
        let a = PathBuf::from("a.vais");
        let b = PathBuf::from("b.vais");
        let c = PathBuf::from("c.vais");

        // Cycle: A -> B -> A
        graph.add_dependency(a.clone(), b.clone());
        graph.add_dependency(b.clone(), a.clone());

        // C -> A (not in cycle)
        graph.add_dependency(c.clone(), a.clone());

        assert!(graph.is_in_cycle(&a));
        assert!(graph.is_in_cycle(&b));
        assert!(!graph.is_in_cycle(&c));
    }
}

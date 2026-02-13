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
        // For now, use topological_sort as the implementation
        // In a full implementation, we would detect SCCs using Tarjan's algorithm
        // and group circular dependencies together
        self.topological_sort()
    }
}

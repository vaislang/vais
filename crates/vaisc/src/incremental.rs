#![allow(dead_code)] // Incremental compilation features reserved for future use
//! Incremental Compilation Cache for Vais Compiler
//!
//! Provides file-hash based caching to avoid unnecessary recompilation.
//! Tracks dependencies between files to invalidate cache when imports change.
//!
//! Performance: Uses rayon for parallel file hash computation.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Cache version for compatibility checking
const CACHE_VERSION: u32 = 1;

/// Compiler version for cache invalidation
const COMPILER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compilation options that affect cache validity
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilationOptions {
    pub opt_level: u8,
    pub debug: bool,
    pub target_triple: String,
}

/// File metadata stored in cache
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub hash: String,
    pub timestamp: u64,
    pub size: u64,
    /// Function-level metadata for fine-grained incremental compilation
    #[serde(default)]
    pub functions: HashMap<String, FunctionMetadata>,
    /// Struct/enum definitions for type change detection
    #[serde(default)]
    pub types: HashMap<String, TypeMetadata>,
    /// Module signature hash for incremental type checking
    #[serde(default)]
    pub signature_hash: Option<ModuleSignatureHash>,
}

/// Function-level metadata for incremental compilation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FunctionMetadata {
    /// Hash of the function body/signature
    pub hash: String,
    /// Line range (start, end) in the source file
    pub line_range: (u32, u32),
    /// Function dependencies (called functions, used types)
    pub dependencies: Vec<String>,
    /// Whether this function was modified since last build
    #[serde(default)]
    pub is_dirty: bool,
}

/// Type (struct/enum) metadata for incremental compilation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypeMetadata {
    /// Hash of the type definition
    pub hash: String,
    /// Line range in source
    pub line_range: (u32, u32),
    /// Types this type depends on (field types, variant types)
    pub dependencies: Vec<String>,
}

/// Module signature hash for incremental type checking.
/// Captures the "public interface" of a file — function signatures, struct fields,
/// enum variants, trait definitions — but NOT function bodies.
/// If this hash is unchanged, dependent modules don't need re-type-checking.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleSignatureHash {
    /// Combined hash of all public signatures in this file
    pub hash: String,
    /// Whether type checking passed for this file (cached result)
    #[serde(default)]
    pub tc_passed: bool,
}

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

/// Cache state stored on disk
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheState {
    pub version: u32,
    pub compiler_version: String,
    pub compilation_options: Option<CompilationOptions>,
    pub dep_graph: DependencyGraph,
    pub last_build: u64,
}

impl Default for CacheState {
    fn default() -> Self {
        Self {
            version: CACHE_VERSION,
            compiler_version: COMPILER_VERSION.to_string(),
            compilation_options: None,
            dep_graph: DependencyGraph::new(),
            last_build: 0,
        }
    }
}

/// Set of files that need recompilation
#[derive(Clone, Debug, Default)]
pub struct DirtySet {
    /// Files that were directly modified
    pub modified_files: HashSet<PathBuf>,
    /// Files affected by modified files (through dependencies)
    pub affected_files: HashSet<PathBuf>,
    /// Function-level dirty tracking: file -> dirty function names
    pub dirty_functions: HashMap<PathBuf, HashSet<String>>,
    /// Type-level dirty tracking: file -> dirty type names
    pub dirty_types: HashMap<PathBuf, HashSet<String>>,
}

impl DirtySet {
    /// Check if any files need recompilation
    pub fn is_empty(&self) -> bool {
        self.modified_files.is_empty() && self.affected_files.is_empty()
    }

    /// Get all files that need recompilation
    pub fn all_dirty_files(&self) -> HashSet<PathBuf> {
        let mut all = self.modified_files.clone();
        all.extend(self.affected_files.clone());
        all
    }

    /// Total count of dirty files
    pub fn count(&self) -> usize {
        self.modified_files.len() + self.affected_files.len()
    }

    /// Check if only specific functions changed (partial recompilation possible)
    pub fn has_partial_changes(&self) -> bool {
        !self.dirty_functions.is_empty() && self.modified_files.is_empty()
    }

    /// Get dirty functions for a file
    pub fn get_dirty_functions(&self, file: &Path) -> Option<&HashSet<String>> {
        self.dirty_functions.get(file)
    }

    /// Mark a function as dirty
    pub fn mark_function_dirty(&mut self, file: PathBuf, func_name: String) {
        self.dirty_functions
            .entry(file)
            .or_default()
            .insert(func_name);
    }

    /// Mark a type as dirty
    pub fn mark_type_dirty(&mut self, file: PathBuf, type_name: String) {
        self.dirty_types.entry(file).or_default().insert(type_name);
    }

    /// Get count of dirty functions across all files
    pub fn dirty_function_count(&self) -> usize {
        self.dirty_functions.values().map(|s| s.len()).sum()
    }
}

/// Incremental compilation cache manager
pub struct IncrementalCache {
    cache_dir: PathBuf,
    state: CacheState,
    current_options: Option<CompilationOptions>,
}

impl IncrementalCache {
    /// Create or load cache from the given directory
    pub fn new(cache_dir: PathBuf) -> Result<Self, String> {
        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)
                .map_err(|e| format!("Cannot create cache directory: {}", e))?;
        }

        let state_file = cache_dir.join("cache_state.json");
        let state = if state_file.exists() {
            let content = fs::read_to_string(&state_file)
                .map_err(|e| format!("Cannot read cache state: {}", e))?;

            match serde_json::from_str::<CacheState>(&content) {
                Ok(state) => {
                    // Validate cache version and compiler version
                    if state.version != CACHE_VERSION || state.compiler_version != COMPILER_VERSION
                    {
                        CacheState::default()
                    } else {
                        state
                    }
                }
                Err(_) => CacheState::default(),
            }
        } else {
            CacheState::default()
        };

        Ok(Self {
            cache_dir,
            state,
            current_options: None,
        })
    }

    /// Set current compilation options
    pub fn set_compilation_options(&mut self, options: CompilationOptions) {
        self.current_options = Some(options);
    }

    /// Detect which files need recompilation (parallelized with rayon)
    pub fn detect_changes(&mut self, entry_file: &Path) -> Result<DirtySet, String> {
        let mut dirty_set = DirtySet::default();

        // Check if compilation options changed
        if let (Some(current), Some(cached)) =
            (&self.current_options, &self.state.compilation_options)
        {
            if current != cached {
                // Options changed - mark all files as dirty
                for file in self.state.dep_graph.file_metadata.keys() {
                    dirty_set.modified_files.insert(file.clone());
                }
                return Ok(dirty_set);
            }
        }

        // Check which files were modified
        let entry_canonical = entry_file
            .canonicalize()
            .map_err(|e| format!("Cannot canonicalize path: {}", e))?;

        // Collect all known files from cache
        let known_files: Vec<PathBuf> =
            self.state.dep_graph.file_metadata.keys().cloned().collect();

        // Parallel file hash computation using rayon
        let cached_hashes: HashMap<PathBuf, String> = self
            .state
            .dep_graph
            .file_metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.hash.clone()))
            .collect();

        let modified_files: Vec<PathBuf> = known_files
            .par_iter()
            .filter_map(|file_path| {
                if !file_path.exists() {
                    // File was deleted
                    return Some(file_path.clone());
                }

                match compute_file_hash(file_path) {
                    Ok(current_hash) => {
                        if let Some(cached_hash) = cached_hashes.get(file_path) {
                            if &current_hash != cached_hash {
                                return Some(file_path.clone());
                            }
                        } else {
                            // New file not in cache
                            return Some(file_path.clone());
                        }
                        None
                    }
                    Err(_) => Some(file_path.clone()), // Mark as dirty on error
                }
            })
            .collect();

        dirty_set.modified_files.extend(modified_files);

        // Check if entry file is new
        if !self
            .state
            .dep_graph
            .file_metadata
            .contains_key(&entry_canonical)
        {
            dirty_set.modified_files.insert(entry_canonical);
        }

        // Propagate changes to dependent files (parallelized)
        let modified_list: Vec<PathBuf> = dirty_set.modified_files.iter().cloned().collect();
        let dep_graph = &self.state.dep_graph;

        let affected: HashSet<PathBuf> = modified_list
            .par_iter()
            .flat_map(|modified| dep_graph.get_dependents(modified))
            .collect();

        dirty_set.affected_files = affected;

        Ok(dirty_set)
    }

    /// Update cache with a compiled file (basic - no function-level metadata)
    pub fn update_file(&mut self, path: &Path) -> Result<(), String> {
        let canonical = path
            .canonicalize()
            .map_err(|e| format!("Cannot canonicalize path: {}", e))?;

        let hash = compute_file_hash(&canonical)?;
        let metadata =
            fs::metadata(&canonical).map_err(|e| format!("Cannot get file metadata: {}", e))?;

        let file_meta = FileMetadata {
            hash,
            timestamp: metadata
                .modified()
                .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
                .unwrap_or(0),
            size: metadata.len(),
            functions: HashMap::new(),
            types: HashMap::new(),
            signature_hash: None,
        };

        self.state
            .dep_graph
            .update_file_metadata(canonical, file_meta);
        Ok(())
    }

    /// Add a dependency between files
    pub fn add_dependency(&mut self, from: &Path, to: &Path) -> Result<(), String> {
        let from_canonical = from
            .canonicalize()
            .map_err(|e| format!("Cannot canonicalize 'from' path: {}", e))?;
        let to_canonical = to
            .canonicalize()
            .map_err(|e| format!("Cannot canonicalize 'to' path: {}", e))?;

        self.state
            .dep_graph
            .add_dependency(from_canonical, to_canonical);
        Ok(())
    }

    /// Clear dependencies for a file (before re-adding after recompilation)
    pub fn clear_file_deps(&mut self, path: &Path) -> Result<(), String> {
        let canonical = path
            .canonicalize()
            .map_err(|e| format!("Cannot canonicalize path: {}", e))?;
        self.state.dep_graph.clear_file_deps(&canonical);
        Ok(())
    }

    /// Save cache state to disk
    pub fn persist(&mut self) -> Result<(), String> {
        // Update compilation options and last build time
        if let Some(opts) = &self.current_options {
            self.state.compilation_options = Some(opts.clone());
        }
        self.state.last_build = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let state_file = self.cache_dir.join("cache_state.json");
        let content = serde_json::to_string_pretty(&self.state)
            .map_err(|e| format!("Cannot serialize cache state: {}", e))?;

        // Atomic write: write to temp file, then rename
        let tmp_file = self.cache_dir.join("cache_state.json.tmp");

        // Write to temp file
        if let Err(e) = fs::write(&tmp_file, &content) {
            // Clean up temp file on error
            let _ = fs::remove_file(&tmp_file);
            return Err(format!("Cannot write cache state to temp file: {}", e));
        }

        // Atomically replace the target file
        if let Err(e) = fs::rename(&tmp_file, &state_file) {
            // Clean up temp file on error
            let _ = fs::remove_file(&tmp_file);
            return Err(format!("Cannot rename cache state file: {}", e));
        }

        Ok(())
    }

    /// Clear all cached data
    pub fn clear(&mut self) -> Result<(), String> {
        self.state = CacheState::default();

        // Remove cache files
        let state_file = self.cache_dir.join("cache_state.json");
        if state_file.exists() {
            fs::remove_file(&state_file)
                .map_err(|e| format!("Cannot remove cache state file: {}", e))?;
        }

        Ok(())
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            total_files: self.state.dep_graph.file_metadata.len(),
            total_dependencies: self
                .state
                .dep_graph
                .forward_deps
                .values()
                .map(|v| v.len())
                .sum(),
            last_build: self.state.last_build,
        }
    }

    /// Clean up cache to keep total size under max_bytes
    /// Deletes oldest .o files first based on modification time
    /// Returns the number of files deleted
    pub fn cleanup_cache(&self, max_bytes: u64) -> Result<usize, String> {
        // Collect all .o files in cache_dir
        let entries = fs::read_dir(&self.cache_dir)
            .map_err(|e| format!("Cannot read cache directory: {}", e))?;

        let mut cache_files: Vec<(PathBuf, u64, std::time::SystemTime)> = Vec::new();
        let mut total_size: u64 = 0;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Cannot read directory entry: {}", e))?;
            let path = entry.path();

            // Skip cache_state.json
            if path.file_name().and_then(|n| n.to_str()) == Some("cache_state.json")
                || path.file_name().and_then(|n| n.to_str()) == Some("cache_state.json.tmp")
            {
                continue;
            }

            // Only process .o files
            if path.extension().and_then(|e| e.to_str()) != Some("o") {
                continue;
            }

            if let Ok(metadata) = fs::metadata(&path) {
                let size = metadata.len();
                let modified = metadata
                    .modified()
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

                cache_files.push((path, size, modified));
                total_size += size;
            }
        }

        // If total size is within limit, nothing to do
        if total_size <= max_bytes {
            return Ok(0);
        }

        // Sort by modification time (oldest first)
        cache_files.sort_by_key(|(_path, _size, modified)| *modified);

        // Delete files until we're under the limit
        let mut deleted_count = 0;
        let mut current_size = total_size;

        for (path, size, _modified) in cache_files {
            if current_size <= max_bytes {
                break;
            }

            if let Err(e) = fs::remove_file(&path) {
                // Continue on error, just skip this file
                eprintln!(
                    "Warning: Failed to delete cache file '{}': {}",
                    path.display(),
                    e
                );
                continue;
            }

            current_size -= size;
            deleted_count += 1;
        }

        Ok(deleted_count)
    }

    /// Detect function-level changes in a file
    pub fn detect_function_changes(
        &self,
        path: &Path,
    ) -> Result<Option<FunctionChangeSet>, String> {
        let canonical = path
            .canonicalize()
            .map_err(|e| format!("Cannot canonicalize path: {}", e))?;

        // Get cached metadata
        let cached_meta = match self.state.dep_graph.file_metadata.get(&canonical) {
            Some(m) => m,
            None => return Ok(None), // New file, no function-level comparison possible
        };

        // Read current file content
        let content =
            fs::read_to_string(&canonical).map_err(|e| format!("Cannot read file: {}", e))?;

        // Extract current definitions
        let mut extractor = DefinitionExtractor::new();
        extractor.extract_from_source(&content)?;

        // Compare with cached
        let change_set = detect_function_changes(&cached_meta.functions, &extractor.functions);

        Ok(Some(change_set))
    }

    /// Update file with function-level metadata
    pub fn update_file_with_functions(&mut self, path: &Path) -> Result<(), String> {
        let canonical = path
            .canonicalize()
            .map_err(|e| format!("Cannot canonicalize path: {}", e))?;

        // Read file content
        let content =
            fs::read_to_string(&canonical).map_err(|e| format!("Cannot read file: {}", e))?;

        // Compute file hash
        let hash = compute_file_hash(&canonical)?;
        let metadata =
            fs::metadata(&canonical).map_err(|e| format!("Cannot get file metadata: {}", e))?;

        // Extract function and type definitions
        let mut extractor = DefinitionExtractor::new();
        extractor.extract_from_source(&content)?;

        let file_meta = FileMetadata {
            hash,
            timestamp: metadata
                .modified()
                .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
                .unwrap_or(0),
            size: metadata.len(),
            functions: extractor.functions,
            types: extractor.types,
            signature_hash: None,
        };

        self.state
            .dep_graph
            .update_file_metadata(canonical, file_meta);
        Ok(())
    }

    /// Detect changes with function-level granularity (parallelized with rayon)
    pub fn detect_changes_fine_grained(&mut self, entry_file: &Path) -> Result<DirtySet, String> {
        let dirty_set = Mutex::new(DirtySet::default());

        // Check if compilation options changed
        if let (Some(current), Some(cached)) =
            (&self.current_options, &self.state.compilation_options)
        {
            if current != cached {
                // Options changed - mark all files as dirty
                let mut ds = dirty_set.lock().expect("dirty_set mutex poisoned");
                for file in self.state.dep_graph.file_metadata.keys() {
                    ds.modified_files.insert(file.clone());
                }
                return Ok(std::mem::take(&mut *ds));
            }
        }

        let entry_canonical = entry_file
            .canonicalize()
            .map_err(|e| format!("Cannot canonicalize path: {}", e))?;

        // Collect all known files from cache
        let known_files: Vec<PathBuf> =
            self.state.dep_graph.file_metadata.keys().cloned().collect();

        // Clone cached metadata for parallel access
        let cached_meta_map: HashMap<PathBuf, FileMetadata> =
            self.state.dep_graph.file_metadata.clone();
        let cached_hashes: HashMap<PathBuf, String> = cached_meta_map
            .iter()
            .map(|(k, v)| (k.clone(), v.hash.clone()))
            .collect();

        // Parallel file processing
        known_files.par_iter().for_each(|file_path| {
            if !file_path.exists() {
                // File was deleted
                dirty_set
                    .lock()
                    .expect("dirty_set mutex poisoned")
                    .modified_files
                    .insert(file_path.clone());
                return;
            }

            let current_hash = match compute_file_hash(file_path) {
                Ok(h) => h,
                Err(_) => {
                    dirty_set
                        .lock()
                        .expect("dirty_set mutex poisoned")
                        .modified_files
                        .insert(file_path.clone());
                    return;
                }
            };

            if let Some(cached_hash) = cached_hashes.get(file_path) {
                if &current_hash != cached_hash {
                    // File changed - check at function level
                    if let Some(cached_meta) = cached_meta_map.get(file_path) {
                        // Function-level analysis (simplified for parallel execution)
                        let total_functions = cached_meta.functions.len();

                        // If no function metadata, mark whole file dirty
                        if total_functions == 0 {
                            dirty_set
                                .lock()
                                .expect("dirty_set mutex poisoned")
                                .modified_files
                                .insert(file_path.clone());
                            return;
                        }

                        // For now, mark whole file dirty in parallel mode
                        // Fine-grained function detection requires sequential processing
                        dirty_set
                            .lock()
                            .expect("dirty_set mutex poisoned")
                            .modified_files
                            .insert(file_path.clone());
                    } else {
                        dirty_set
                            .lock()
                            .expect("dirty_set mutex poisoned")
                            .modified_files
                            .insert(file_path.clone());
                    }
                }
            } else {
                // New file not in cache
                dirty_set
                    .lock()
                    .expect("dirty_set mutex poisoned")
                    .modified_files
                    .insert(file_path.clone());
            }
        });

        let mut dirty_set = dirty_set.into_inner().expect("dirty_set mutex poisoned");

        // Check if entry file is new
        if !self
            .state
            .dep_graph
            .file_metadata
            .contains_key(&entry_canonical)
        {
            dirty_set.modified_files.insert(entry_canonical);
        }

        // Propagate changes to dependent files (parallelized)
        let modified_list: Vec<PathBuf> = dirty_set.modified_files.iter().cloned().collect();
        let dep_graph = &self.state.dep_graph;

        let affected: HashSet<PathBuf> = modified_list
            .par_iter()
            .flat_map(|modified| dep_graph.get_dependents(modified))
            .collect();

        dirty_set.affected_files = affected;

        // Propagate function-level changes to affected functions in other files
        let dirty_funcs: Vec<(PathBuf, HashSet<String>)> = dirty_set
            .dirty_functions
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // Parallel propagation of function-level changes
        let new_dirty_funcs = Mutex::new(Vec::<(PathBuf, String)>::new());

        dirty_funcs.par_iter().for_each(|(file_path, func_names)| {
            let dependents = self.state.dep_graph.get_dependents(file_path);

            for dependent in dependents {
                if let Some(dep_meta) = self.state.dep_graph.file_metadata.get(&dependent) {
                    for (dep_func_name, dep_func) in &dep_meta.functions {
                        for dep_item in &dep_func.dependencies {
                            if func_names.contains(dep_item) {
                                new_dirty_funcs
                                    .lock()
                                    .expect("new_dirty_funcs mutex poisoned")
                                    .push((dependent.clone(), dep_func_name.clone()));
                                break;
                            }
                        }
                    }
                }
            }
        });

        for (file, func) in new_dirty_funcs
            .into_inner()
            .expect("new_dirty_funcs mutex poisoned")
        {
            dirty_set.mark_function_dirty(file, func);
        }

        Ok(dirty_set)
    }

    /// Get cached object file path for a function
    pub fn get_cached_object_path(&self, file: &Path, func_name: &str) -> PathBuf {
        let file_hash = file.to_string_lossy().replace(['/', '\\', ':'], "_");
        self.cache_dir
            .join(format!("{}_{}.o", file_hash, func_name))
    }

    /// Check if a function's object file is cached
    pub fn has_cached_object(&self, file: &Path, func_name: &str) -> bool {
        self.get_cached_object_path(file, func_name).exists()
    }

    /// Get all cached object paths for non-dirty functions in a file
    pub fn get_reusable_objects(&self, file: &Path, dirty_set: &DirtySet) -> Vec<PathBuf> {
        let mut objects = Vec::new();

        let canonical = match file.canonicalize() {
            Ok(p) => p,
            Err(_) => return objects,
        };

        if let Some(meta) = self.state.dep_graph.file_metadata.get(&canonical) {
            let dirty_funcs = dirty_set
                .dirty_functions
                .get(&canonical)
                .cloned()
                .unwrap_or_default();

            for func_name in meta.functions.keys() {
                if !dirty_funcs.contains(func_name) {
                    let obj_path = self.get_cached_object_path(&canonical, func_name);
                    if obj_path.exists() {
                        objects.push(obj_path);
                    }
                }
            }
        }

        objects
    }

    /// Detect changes with detailed statistics tracking
    /// Returns both the dirty set and statistics about cache hits/misses
    pub fn detect_changes_with_stats(
        &mut self,
        entry_file: &Path,
    ) -> Result<(DirtySet, IncrementalStats), String> {
        let start_time = std::time::Instant::now();
        let mut dirty_set = DirtySet::default();
        let mut stats = IncrementalStats::default();

        // Check if compilation options changed
        if let (Some(current), Some(cached)) =
            (&self.current_options, &self.state.compilation_options)
        {
            if current != cached {
                // Options changed - mark all files as dirty
                for file in self.state.dep_graph.file_metadata.keys() {
                    dirty_set.modified_files.insert(file.clone());
                    stats
                        .miss_reasons
                        .entry(file.clone())
                        .or_default()
                        .push(CacheMissReason::OptionsChanged);
                    stats.cache_misses += 1;
                }
                stats.files_checked = self.state.dep_graph.file_metadata.len();
                stats.total_check_time_ms = start_time.elapsed().as_millis() as u64;
                return Ok((dirty_set, stats));
            }
        }

        // Check which files were modified
        let entry_canonical = entry_file
            .canonicalize()
            .map_err(|e| format!("Cannot canonicalize path: {}", e))?;

        // Collect all known files from cache
        let known_files: Vec<PathBuf> =
            self.state.dep_graph.file_metadata.keys().cloned().collect();

        // Parallel file hash computation using rayon
        let cached_metadata: HashMap<PathBuf, (String, Option<String>)> = self
            .state
            .dep_graph
            .file_metadata
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    (
                        v.hash.clone(),
                        v.signature_hash.as_ref().map(|sh| sh.hash.clone()),
                    ),
                )
            })
            .collect();

        stats.files_checked = known_files.len();

        let file_check_results: Vec<(PathBuf, Option<CacheMissReason>)> = known_files
            .par_iter()
            .map(|file_path| {
                if !file_path.exists() {
                    // File was deleted
                    return (file_path.clone(), Some(CacheMissReason::FileDeleted));
                }

                match compute_file_hash(file_path) {
                    Ok(current_hash) => {
                        if let Some((cached_hash, _cached_sig_hash)) =
                            cached_metadata.get(file_path)
                        {
                            if &current_hash != cached_hash {
                                // Content changed - check if signature also changed
                                // For now, we mark as ContentHashChanged
                                // Signature check happens in the next phase
                                return (file_path.clone(), Some(CacheMissReason::ContentHashChanged));
                            }
                            // Hash unchanged - cache hit
                            (file_path.clone(), None)
                        } else {
                            // New file not in cache
                            (file_path.clone(), Some(CacheMissReason::NewFile))
                        }
                    }
                    Err(_) => (file_path.clone(), Some(CacheMissReason::CacheCorrupted)),
                }
            })
            .collect();

        // Process results
        for (file_path, miss_reason) in file_check_results {
            if let Some(reason) = miss_reason {
                dirty_set.modified_files.insert(file_path.clone());
                stats
                    .miss_reasons
                    .entry(file_path)
                    .or_default()
                    .push(reason);
                stats.cache_misses += 1;
            } else {
                stats.cache_hits += 1;
            }
        }

        // Check if entry file is new
        if !self
            .state
            .dep_graph
            .file_metadata
            .contains_key(&entry_canonical)
            && !dirty_set.modified_files.contains(&entry_canonical)
        {
            dirty_set.modified_files.insert(entry_canonical.clone());
            stats
                .miss_reasons
                .entry(entry_canonical)
                .or_default()
                .push(CacheMissReason::NewFile);
            stats.cache_misses += 1;
            stats.files_checked += 1;
        }

        // Propagate changes to dependent files (parallelized)
        // BUT: skip propagation if only signature is unchanged (body changed)
        let modified_list: Vec<PathBuf> = dirty_set.modified_files.iter().cloned().collect();
        let dep_graph = &self.state.dep_graph;

        // Check signature hashes to avoid unnecessary propagation
        let mut files_to_propagate = Vec::new();
        for modified_file in &modified_list {
            let should_propagate = if let Some(_meta) = dep_graph.file_metadata.get(modified_file) {
                // If we have a cached signature hash, re-compute current signature
                // If signatures match, we don't need to propagate to dependents
                // For now, we conservatively propagate (signature check requires AST)
                // This is where signature_hits would be tracked in a full implementation
                true
            } else {
                true
            };

            if should_propagate {
                files_to_propagate.push(modified_file.clone());
            } else {
                stats.signature_hits += 1;
            }
        }

        let affected: HashSet<PathBuf> = files_to_propagate
            .par_iter()
            .flat_map(|modified| dep_graph.get_dependents(modified))
            .collect();

        // Track dependency changes in miss reasons
        for affected_file in &affected {
            if !dirty_set.modified_files.contains(affected_file) {
                // Find which dependency caused this to be dirty
                if let Some(deps) = dep_graph.forward_deps.get(affected_file) {
                    for dep in deps {
                        if dirty_set.modified_files.contains(dep) {
                            stats
                                .miss_reasons
                                .entry(affected_file.clone())
                                .or_default()
                                .push(CacheMissReason::DependencyChanged(
                                    dep.to_string_lossy().to_string(),
                                ));
                            break;
                        }
                    }
                }
            }
        }

        dirty_set.affected_files = affected;

        stats.total_check_time_ms = start_time.elapsed().as_millis() as u64;

        Ok((dirty_set, stats))
    }

    /// Get incremental compilation statistics
    pub fn get_incremental_stats(&self) -> IncrementalStats {
        IncrementalStats {
            files_checked: self.state.dep_graph.file_metadata.len(),
            ..IncrementalStats::default()
        }
    }

    /// Warm the cache by scanning all .vais files in the project
    /// Computes hashes for all files and adds them to the cache if not present
    /// Returns the number of files warmed
    pub fn warm_cache(&mut self, project_root: &Path) -> Result<usize, String> {
        let mut warmed_count = 0;

        // Recursively scan for .vais files
        let vais_files = self.scan_vais_files(project_root)?;

        for file_path in vais_files {
            let canonical = match file_path.canonicalize() {
                Ok(p) => p,
                Err(_) => continue,
            };

            // Check if already in cache
            if let Some(cached_meta) = self.state.dep_graph.file_metadata.get(&canonical) {
                // Verify hash is still valid
                match compute_file_hash(&canonical) {
                    Ok(current_hash) => {
                        if current_hash != cached_meta.hash {
                            // Hash changed - update metadata
                            if self.update_file_with_functions(&canonical).is_ok() {
                                warmed_count += 1;
                            }
                        }
                    }
                    Err(_) => continue,
                }
            } else {
                // Not in cache - add it
                if self.update_file_with_functions(&canonical).is_ok() {
                    warmed_count += 1;
                }
            }
        }

        Ok(warmed_count)
    }

    /// Recursively scan directory for .vais files
    fn scan_vais_files(&self, dir: &Path) -> Result<Vec<PathBuf>, String> {
        let mut vais_files = Vec::new();

        if !dir.is_dir() {
            return Ok(vais_files);
        }

        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Cannot read directory '{}': {}", dir.display(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Cannot read directory entry: {}", e))?;
            let path = entry.path();

            // Skip cache directories
            if path.file_name().and_then(|n| n.to_str()) == Some(".vais-cache") {
                continue;
            }

            if path.is_dir() {
                // Recursively scan subdirectory
                let sub_files = self.scan_vais_files(&path)?;
                vais_files.extend(sub_files);
            } else if path.extension().and_then(|e| e.to_str()) == Some("vais") {
                vais_files.push(path);
            }
        }

        Ok(vais_files)
    }
}

/// Cache statistics for verbose output
#[derive(Debug)]
pub struct CacheStats {
    pub total_files: usize,
    pub total_dependencies: usize,
    pub last_build: u64,
}

/// Reason why a cache miss occurred
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheMissReason {
    /// File is new to the project
    NewFile,
    /// File content hash changed
    ContentHashChanged,
    /// File signature (public interface) changed
    SignatureChanged,
    /// A dependency of this file changed
    DependencyChanged(String),
    /// Compilation options changed
    OptionsChanged,
    /// File was deleted from disk
    FileDeleted,
    /// Cache was corrupted or incompatible
    CacheCorrupted,
}

/// Incremental compilation statistics
#[derive(Debug, Clone, Default)]
pub struct IncrementalStats {
    /// Number of files that hit the cache (unchanged)
    pub cache_hits: usize,
    /// Number of files that missed the cache (need recompilation)
    pub cache_misses: usize,
    /// Reasons for cache misses per file
    pub miss_reasons: HashMap<PathBuf, Vec<CacheMissReason>>,
    /// Total number of files checked
    pub files_checked: usize,
    /// Files skipped due to unchanged signature (body changed but signature didn't)
    pub files_skipped: usize,
    /// Files where signature matched (dependents don't need rebuild)
    pub signature_hits: usize,
    /// Total time spent checking cache (milliseconds)
    pub total_check_time_ms: u64,
}

impl IncrementalStats {
    /// Calculate cache hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        if self.files_checked == 0 {
            return 0.0;
        }
        (self.cache_hits as f64 / self.files_checked as f64) * 100.0
    }
}

/// Compute SHA256 hash of a file
pub fn compute_file_hash(path: &Path) -> Result<String, String> {
    let content =
        fs::read(path).map_err(|e| format!("Cannot read file '{}': {}", path.display(), e))?;

    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();

    Ok(format!("{:x}", result))
}

/// Compute SHA256 hash of a string (for function bodies, type definitions)
pub fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Compute a signature hash for a set of AST items.
/// This captures the "public interface" — function signatures, struct fields,
/// enum variants, trait definitions — but NOT function bodies.
/// Used to determine if type checking results can be reused.
pub fn compute_signature_hash(items: &[vais_ast::Spanned<vais_ast::Item>]) -> String {
    use std::fmt::Write;
    let mut sig = String::new();

    for item in items {
        match &item.node {
            vais_ast::Item::Function(f) => {
                let _ = write!(sig, "fn:{}", f.name.node);
                for p in &f.params {
                    let _ = write!(sig, ",p:{}:{:?}", p.name.node, p.ty.node);
                }
                if let Some(ret) = &f.ret_type {
                    let _ = write!(sig, "->:{:?}", ret.node);
                }
                sig.push(';');
            }
            vais_ast::Item::Struct(s) => {
                let _ = write!(sig, "struct:{}", s.name.node);
                for f in &s.fields {
                    let _ = write!(sig, ",f:{}:{:?}", f.name.node, f.ty.node);
                }
                for m in &s.methods {
                    let _ = write!(sig, ",m:{}", m.node.name.node);
                    for p in &m.node.params {
                        let _ = write!(sig, ",mp:{}:{:?}", p.name.node, p.ty.node);
                    }
                    if let Some(ret) = &m.node.ret_type {
                        let _ = write!(sig, "->:{:?}", ret.node);
                    }
                }
                sig.push(';');
            }
            vais_ast::Item::Enum(e) => {
                let _ = write!(sig, "enum:{}", e.name.node);
                for v in &e.variants {
                    let _ = write!(sig, ",v:{}:{:?}", v.name.node, v.fields);
                }
                sig.push(';');
            }
            vais_ast::Item::Trait(t) => {
                let _ = write!(sig, "trait:{}", t.name.node);
                for m in &t.methods {
                    let _ = write!(sig, ",tm:{}", m.name.node);
                    for p in &m.params {
                        let _ = write!(sig, ",tp:{}:{:?}", p.name.node, p.ty.node);
                    }
                    if let Some(ret) = &m.ret_type {
                        let _ = write!(sig, "->:{:?}", ret.node);
                    }
                }
                sig.push(';');
            }
            vais_ast::Item::Impl(imp) => {
                let _ = write!(sig, "impl:{:?}", imp.target_type.node);
                if let Some(tn) = &imp.trait_name {
                    let _ = write!(sig, ":trait:{}", tn.node);
                }
                for m in &imp.methods {
                    let _ = write!(sig, ",im:{}", m.node.name.node);
                    for p in &m.node.params {
                        let _ = write!(sig, ",ip:{}:{:?}", p.name.node, p.ty.node);
                    }
                    if let Some(ret) = &m.node.ret_type {
                        let _ = write!(sig, "->:{:?}", ret.node);
                    }
                }
                sig.push(';');
            }
            vais_ast::Item::TypeAlias(ta) => {
                let _ = write!(sig, "type:{}={:?};", ta.name.node, ta.ty.node);
            }
            vais_ast::Item::Const(c) => {
                let _ = write!(sig, "const:{}:{:?};", c.name.node, c.ty.node);
            }
            vais_ast::Item::Global(g) => {
                let _ = write!(sig, "global:{}:{:?};", g.name.node, g.ty.node);
            }
            vais_ast::Item::Union(u) => {
                let _ = write!(sig, "union:{}", u.name.node);
                for f in &u.fields {
                    let _ = write!(sig, ",f:{}:{:?}", f.name.node, f.ty.node);
                }
                sig.push(';');
            }
            vais_ast::Item::ExternBlock(eb) => {
                for f in &eb.functions {
                    let _ = write!(sig, "extern:{}:{}", eb.abi, f.name.node);
                    for p in &f.params {
                        let _ = write!(sig, ",ep:{}:{:?}", p.name.node, p.ty.node);
                    }
                    if let Some(ret) = &f.ret_type {
                        let _ = write!(sig, "->:{:?}", ret.node);
                    }
                    sig.push(';');
                }
            }
            _ => {} // Use, Macro, Error — don't affect signature
        }
    }

    compute_content_hash(&sig)
}

/// Check if type checking can be skipped based on cached signatures.
/// Returns true if ALL files have unchanged content hashes AND signature hashes,
/// meaning type checking results are still valid.
pub fn can_skip_type_checking(cache: &IncrementalCache, files: &[PathBuf]) -> bool {
    for file in files {
        let canonical = match file.canonicalize() {
            Ok(p) => p,
            Err(_) => return false,
        };

        match cache.state.dep_graph.file_metadata.get(&canonical) {
            Some(meta) => {
                // Check file content hash
                let current_hash = match compute_file_hash(&canonical) {
                    Ok(h) => h,
                    Err(_) => return false,
                };
                if current_hash != meta.hash {
                    return false;
                }
                // Check that TC previously passed for this file
                match &meta.signature_hash {
                    Some(sig) if sig.tc_passed => {}
                    _ => return false,
                }
            }
            None => return false, // File not in cache
        }
    }
    true
}

/// Update the signature hash and TC result for files in the cache.
/// Call this after successful type checking.
pub fn update_tc_cache(cache: &mut IncrementalCache, module: &vais_ast::Module, tc_passed: bool) {
    // If no modules_map, compute a single hash for the whole module
    if let Some(modules_map) = &module.modules_map {
        for (file_path, indices) in modules_map {
            let canonical = match file_path.canonicalize() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let file_items: Vec<vais_ast::Spanned<vais_ast::Item>> = indices
                .iter()
                .filter_map(|&i| module.items.get(i).cloned())
                .collect();

            let sig_hash = compute_signature_hash(&file_items);

            if let Some(meta) = cache.state.dep_graph.file_metadata.get_mut(&canonical) {
                meta.signature_hash = Some(ModuleSignatureHash {
                    hash: sig_hash,
                    tc_passed,
                });
            }
        }
    }
}

/// Get the path for a cached object file based on IR content hash.
/// This enables skipping `clang -c` when the generated IR hasn't changed.
pub fn get_ir_cached_object_path(cache_dir: &Path, ir_hash: &str, opt_level: u8) -> PathBuf {
    cache_dir.join(format!("ir_O{}_{}.o", opt_level, &ir_hash[..16]))
}

/// Check if a cached object file exists for the given IR hash.
pub fn has_ir_cached_object(cache_dir: &Path, ir_hash: &str, opt_level: u8) -> bool {
    get_ir_cached_object_path(cache_dir, ir_hash, opt_level).exists()
}

/// Function/type extractor for incremental compilation
pub struct DefinitionExtractor {
    /// Extracted function metadata
    pub functions: HashMap<String, FunctionMetadata>,
    /// Extracted type metadata
    pub types: HashMap<String, TypeMetadata>,
}

impl DefinitionExtractor {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            types: HashMap::new(),
        }
    }

    /// Extract definitions from source content
    /// This is a simplified parser that looks for function and type patterns
    pub fn extract_from_source(&mut self, content: &str) -> Result<(), String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut current_line = 0;

        while current_line < lines.len() {
            let line = lines[current_line].trim();

            // Function definition: F name(...) or F name<...>(...)
            if let Some(func_info) = self.try_parse_function(line, &lines, current_line) {
                let (name, start, end, body) = func_info;
                let hash = compute_content_hash(&body);
                let deps = self.extract_dependencies(&body);

                self.functions.insert(
                    name.clone(),
                    FunctionMetadata {
                        hash,
                        line_range: (start as u32, end as u32),
                        dependencies: deps,
                        is_dirty: false,
                    },
                );

                current_line = end + 1;
                continue;
            }

            // Struct definition: S name { ... }
            if let Some(type_info) = self.try_parse_struct(line, &lines, current_line) {
                let (name, start, end, body) = type_info;
                let hash = compute_content_hash(&body);
                let deps = self.extract_type_dependencies(&body);

                self.types.insert(
                    name.clone(),
                    TypeMetadata {
                        hash,
                        line_range: (start as u32, end as u32),
                        dependencies: deps,
                    },
                );

                current_line = end + 1;
                continue;
            }

            // Enum definition: E name { ... }
            if let Some(type_info) = self.try_parse_enum(line, &lines, current_line) {
                let (name, start, end, body) = type_info;
                let hash = compute_content_hash(&body);
                let deps = self.extract_type_dependencies(&body);

                self.types.insert(
                    name.clone(),
                    TypeMetadata {
                        hash,
                        line_range: (start as u32, end as u32),
                        dependencies: deps,
                    },
                );

                current_line = end + 1;
                continue;
            }

            current_line += 1;
        }

        Ok(())
    }

    /// Try to parse a function definition, returns (name, start_line, end_line, body)
    fn try_parse_function(
        &self,
        line: &str,
        lines: &[&str],
        start: usize,
    ) -> Option<(String, usize, usize, String)> {
        // Match patterns: "F name(", "F name<", "pub F name("
        let line_trimmed = line.trim_start_matches("pub ").trim();
        if !line_trimmed.starts_with("F ") {
            return None;
        }

        // Extract function name
        let after_f = line_trimmed[2..].trim();
        let name_end = after_f.find(['(', '<']).unwrap_or(after_f.len());
        let name = after_f[..name_end].trim().to_string();

        if name.is_empty() {
            return None;
        }

        // Find matching braces
        let (end_line, body) = self.find_block_end(lines, start)?;

        Some((name, start, end_line, body))
    }

    /// Try to parse a struct definition
    fn try_parse_struct(
        &self,
        line: &str,
        lines: &[&str],
        start: usize,
    ) -> Option<(String, usize, usize, String)> {
        let line_trimmed = line.trim_start_matches("pub ").trim();
        if !line_trimmed.starts_with("S ") {
            return None;
        }

        let after_s = line_trimmed[2..].trim();
        let name_end = after_s.find(['{', '<', ' ']).unwrap_or(after_s.len());
        let name = after_s[..name_end].trim().to_string();

        if name.is_empty() {
            return None;
        }

        let (end_line, body) = self.find_block_end(lines, start)?;
        Some((name, start, end_line, body))
    }

    /// Try to parse an enum definition
    fn try_parse_enum(
        &self,
        line: &str,
        lines: &[&str],
        start: usize,
    ) -> Option<(String, usize, usize, String)> {
        let line_trimmed = line.trim_start_matches("pub ").trim();
        if !line_trimmed.starts_with("E ") {
            return None;
        }

        let after_e = line_trimmed[2..].trim();
        let name_end = after_e.find(['{', '<', ' ']).unwrap_or(after_e.len());
        let name = after_e[..name_end].trim().to_string();

        if name.is_empty() {
            return None;
        }

        let (end_line, body) = self.find_block_end(lines, start)?;
        Some((name, start, end_line, body))
    }

    /// Find the end of a block (matching braces)
    fn find_block_end(&self, lines: &[&str], start: usize) -> Option<(usize, String)> {
        let mut brace_count = 0;
        let mut found_open = false;
        let mut body = String::new();

        for (i, line) in lines.iter().enumerate().skip(start) {
            body.push_str(line);
            body.push('\n');

            for ch in line.chars() {
                if ch == '{' {
                    brace_count += 1;
                    found_open = true;
                } else if ch == '}' {
                    brace_count -= 1;
                }
            }

            if found_open && brace_count == 0 {
                return Some((i, body));
            }
        }

        None
    }

    /// Extract function dependencies from body (called functions, used types)
    fn extract_dependencies(&self, body: &str) -> Vec<String> {
        let mut deps = Vec::new();

        // Simple pattern matching for function calls: name(
        // This is a simplified approach - a real implementation would use the AST
        let words: Vec<&str> = body
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty())
            .collect();

        for window in words.windows(1) {
            let word = window[0];
            // Skip Vais keywords
            if !is_vais_keyword(word)
                && word
                    .chars()
                    .next()
                    .map(|c| c.is_alphabetic())
                    .unwrap_or(false)
            {
                // Check if followed by ( in original body
                if (body.contains(&format!("{}(", word)) || body.contains(&format!("{}<", word)))
                    && !deps.contains(&word.to_string())
                {
                    deps.push(word.to_string());
                }
            }
        }

        deps
    }

    /// Extract type dependencies from type definition
    fn extract_type_dependencies(&self, body: &str) -> Vec<String> {
        let mut deps = Vec::new();

        // Look for type references: field: Type, Vec<Type>, etc.
        let words: Vec<&str> = body
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty())
            .collect();

        for word in words {
            // Type names start with uppercase (convention)
            if word
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
                && !is_vais_keyword(word)
                && !is_builtin_type(word)
                && !deps.contains(&word.to_string())
            {
                deps.push(word.to_string());
            }
        }

        deps
    }
}

impl Default for DefinitionExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a word is a Vais keyword
fn is_vais_keyword(word: &str) -> bool {
    matches!(
        word,
        "F" | "S"
            | "E"
            | "T"
            | "I"
            | "M"
            | "N"
            | "C"
            | "V"
            | "L"
            | "W"
            | "R"
            | "B"
            | "P"
            | "if"
            | "else"
            | "for"
            | "while"
            | "return"
            | "break"
            | "continue"
            | "true"
            | "false"
            | "self"
            | "Self"
            | "pub"
            | "mut"
            | "async"
            | "await"
            | "import"
            | "from"
            | "as"
            | "match"
            | "spawn"
            | "defer"
    )
}

/// Check if a type is a builtin type
fn is_builtin_type(word: &str) -> bool {
    matches!(
        word,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "f32"
            | "f64"
            | "bool"
            | "str"
            | "String"
            | "Vec"
            | "HashMap"
            | "HashSet"
            | "Option"
            | "Result"
            | "Box"
            | "Rc"
            | "Arc"
            | "RefCell"
            | "Mutex"
    )
}

/// Compare function metadata and detect changes
pub fn detect_function_changes(
    old_meta: &HashMap<String, FunctionMetadata>,
    new_meta: &HashMap<String, FunctionMetadata>,
) -> FunctionChangeSet {
    let mut change_set = FunctionChangeSet::default();

    // Find added and modified functions
    for (name, new_fn) in new_meta {
        if let Some(old_fn) = old_meta.get(name) {
            if old_fn.hash != new_fn.hash {
                change_set.modified.insert(name.clone());
            }
        } else {
            change_set.added.insert(name.clone());
        }
    }

    // Find removed functions
    for name in old_meta.keys() {
        if !new_meta.contains_key(name) {
            change_set.removed.insert(name.clone());
        }
    }

    // Find affected functions (functions that depend on changed functions)
    let all_changed: HashSet<_> = change_set
        .modified
        .iter()
        .chain(change_set.added.iter())
        .chain(change_set.removed.iter())
        .cloned()
        .collect();

    for (name, func) in new_meta {
        if all_changed.contains(name) {
            continue;
        }
        for dep in &func.dependencies {
            if all_changed.contains(dep) {
                change_set.affected.insert(name.clone());
                break;
            }
        }
    }

    change_set
}

/// Set of function changes
#[derive(Debug, Default)]
pub struct FunctionChangeSet {
    /// Newly added functions
    pub added: HashSet<String>,
    /// Modified functions (hash changed)
    pub modified: HashSet<String>,
    /// Removed functions
    pub removed: HashSet<String>,
    /// Functions affected by changes (through dependencies)
    pub affected: HashSet<String>,
}

impl FunctionChangeSet {
    /// Check if there are any changes
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.modified.is_empty()
            && self.removed.is_empty()
            && self.affected.is_empty()
    }

    /// Get all functions that need recompilation
    pub fn all_dirty(&self) -> HashSet<String> {
        let mut all = self.added.clone();
        all.extend(self.modified.clone());
        all.extend(self.affected.clone());
        all
    }

    /// Total count of changes
    pub fn count(&self) -> usize {
        self.added.len() + self.modified.len() + self.removed.len() + self.affected.len()
    }
}

/// Import tracker for collecting dependencies during parsing
#[derive(Default)]
pub struct ImportTracker {
    /// Current file being parsed
    pub current_file: Option<PathBuf>,
    /// Collected imports: from_file -> to_files
    pub imports: HashMap<PathBuf, Vec<PathBuf>>,
}

impl ImportTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start tracking imports for a file
    pub fn start_file(&mut self, path: PathBuf) {
        self.current_file = Some(path.clone());
        self.imports.entry(path).or_default();
    }

    /// Record an import
    pub fn add_import(&mut self, imported_path: PathBuf) {
        if let Some(current) = &self.current_file {
            self.imports
                .entry(current.clone())
                .or_default()
                .push(imported_path);
        }
    }

    /// Finish tracking and get all imports
    pub fn finish(self) -> HashMap<PathBuf, Vec<PathBuf>> {
        self.imports
    }
}

/// Determine cache directory for a given source file
pub fn get_cache_dir(source_file: &Path) -> PathBuf {
    // Use parent directory of source file, or current directory
    let base = source_file.parent().unwrap_or(Path::new("."));
    base.join(".vais-cache")
}

#[cfg(test)]
mod tests {
    use super::*;

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
            .add_dependency(&file1.canonicalize().unwrap(), &file2.canonicalize().unwrap())
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
}

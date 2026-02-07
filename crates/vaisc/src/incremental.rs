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
                let mut ds = dirty_set.lock().unwrap();
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
                    .unwrap()
                    .modified_files
                    .insert(file_path.clone());
                return;
            }

            let current_hash = match compute_file_hash(file_path) {
                Ok(h) => h,
                Err(_) => {
                    dirty_set
                        .lock()
                        .unwrap()
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
                                .unwrap()
                                .modified_files
                                .insert(file_path.clone());
                            return;
                        }

                        // For now, mark whole file dirty in parallel mode
                        // Fine-grained function detection requires sequential processing
                        dirty_set
                            .lock()
                            .unwrap()
                            .modified_files
                            .insert(file_path.clone());
                    } else {
                        dirty_set
                            .lock()
                            .unwrap()
                            .modified_files
                            .insert(file_path.clone());
                    }
                }
            } else {
                // New file not in cache
                dirty_set
                    .lock()
                    .unwrap()
                    .modified_files
                    .insert(file_path.clone());
            }
        });

        let mut dirty_set = dirty_set.into_inner().unwrap();

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
                                    .unwrap()
                                    .push((dependent.clone(), dep_func_name.clone()));
                                break;
                            }
                        }
                    }
                }
            }
        });

        for (file, func) in new_dirty_funcs.into_inner().unwrap() {
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
}

/// Cache statistics for verbose output
#[derive(Debug)]
pub struct CacheStats {
    pub total_files: usize,
    pub total_dependencies: usize,
    pub last_build: u64,
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
}

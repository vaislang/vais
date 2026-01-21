//! Incremental Compilation Cache for Vais Compiler
//!
//! Provides file-hash based caching to avoid unnecessary recompilation.
//! Tracks dependencies between files to invalidate cache when imports change.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
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
                    if state.version != CACHE_VERSION {
                        CacheState::default()
                    } else if state.compiler_version != COMPILER_VERSION {
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

    /// Detect which files need recompilation
    pub fn detect_changes(&mut self, entry_file: &Path) -> Result<DirtySet, String> {
        let mut dirty_set = DirtySet::default();

        // Check if compilation options changed
        if let (Some(current), Some(cached)) = (&self.current_options, &self.state.compilation_options) {
            if current != cached {
                // Options changed - mark all files as dirty
                for file in self.state.dep_graph.file_metadata.keys() {
                    dirty_set.modified_files.insert(file.clone());
                }
                return Ok(dirty_set);
            }
        }

        // Check which files were modified
        let entry_canonical = entry_file.canonicalize()
            .map_err(|e| format!("Cannot canonicalize path: {}", e))?;

        // Collect all known files from cache
        let known_files: Vec<PathBuf> = self.state.dep_graph.file_metadata.keys().cloned().collect();

        for file_path in known_files {
            if !file_path.exists() {
                // File was deleted
                dirty_set.modified_files.insert(file_path);
                continue;
            }

            let current_hash = compute_file_hash(&file_path)?;
            if let Some(cached_meta) = self.state.dep_graph.file_metadata.get(&file_path) {
                if current_hash != cached_meta.hash {
                    dirty_set.modified_files.insert(file_path);
                }
            } else {
                // New file not in cache
                dirty_set.modified_files.insert(file_path);
            }
        }

        // Check if entry file is new
        if !self.state.dep_graph.file_metadata.contains_key(&entry_canonical) {
            dirty_set.modified_files.insert(entry_canonical);
        }

        // Propagate changes to dependent files
        for modified in dirty_set.modified_files.clone() {
            let dependents = self.state.dep_graph.get_dependents(&modified);
            dirty_set.affected_files.extend(dependents);
        }

        Ok(dirty_set)
    }

    /// Update cache with a compiled file
    pub fn update_file(&mut self, path: &Path) -> Result<(), String> {
        let canonical = path.canonicalize()
            .map_err(|e| format!("Cannot canonicalize path: {}", e))?;

        let hash = compute_file_hash(&canonical)?;
        let metadata = fs::metadata(&canonical)
            .map_err(|e| format!("Cannot get file metadata: {}", e))?;

        let file_meta = FileMetadata {
            hash,
            timestamp: metadata.modified()
                .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
                .unwrap_or(0),
            size: metadata.len(),
        };

        self.state.dep_graph.update_file_metadata(canonical, file_meta);
        Ok(())
    }

    /// Add a dependency between files
    pub fn add_dependency(&mut self, from: &Path, to: &Path) -> Result<(), String> {
        let from_canonical = from.canonicalize()
            .map_err(|e| format!("Cannot canonicalize 'from' path: {}", e))?;
        let to_canonical = to.canonicalize()
            .map_err(|e| format!("Cannot canonicalize 'to' path: {}", e))?;

        self.state.dep_graph.add_dependency(from_canonical, to_canonical);
        Ok(())
    }

    /// Clear dependencies for a file (before re-adding after recompilation)
    pub fn clear_file_deps(&mut self, path: &Path) -> Result<(), String> {
        let canonical = path.canonicalize()
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

        fs::write(&state_file, content)
            .map_err(|e| format!("Cannot write cache state: {}", e))?;

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

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            total_files: self.state.dep_graph.file_metadata.len(),
            total_dependencies: self.state.dep_graph.forward_deps.values()
                .map(|v| v.len())
                .sum(),
            last_build: self.state.last_build,
        }
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
    let content = fs::read(path)
        .map_err(|e| format!("Cannot read file '{}': {}", path.display(), e))?;

    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();

    Ok(format!("{:x}", result))
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
    use std::io::Write;
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
        assert!(dirty.modified_files.contains(&source_file.canonicalize().unwrap()));

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
}

use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use super::detect::{compute_file_hash, DefinitionExtractor, detect_function_changes, FunctionChangeSet};
use super::stats::{CacheMissReason, CacheStats, IncrementalStats};
use super::types::{CacheState, CompilationOptions, DirtySet, FileMetadata};

/// Incremental compilation cache manager
pub struct IncrementalCache {
    cache_dir: PathBuf,
    pub(super) state: CacheState,
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
                    if state.version != super::CACHE_VERSION || state.compiler_version != super::COMPILER_VERSION
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
                                return (
                                    file_path.clone(),
                                    Some(CacheMissReason::ContentHashChanged),
                                );
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

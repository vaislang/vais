use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

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

/// Cache state stored on disk
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheState {
    pub version: u32,
    pub compiler_version: String,
    pub compilation_options: Option<CompilationOptions>,
    pub dep_graph: super::graph::DependencyGraph,
    pub last_build: u64,
}

impl Default for CacheState {
    fn default() -> Self {
        Self {
            version: super::CACHE_VERSION,
            compiler_version: super::COMPILER_VERSION.to_string(),
            compilation_options: None,
            dep_graph: super::graph::DependencyGraph::new(),
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
    pub fn get_dirty_functions(&self, file: &std::path::Path) -> Option<&HashSet<String>> {
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

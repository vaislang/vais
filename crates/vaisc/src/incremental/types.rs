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

    /// Get count of dirty types across all files
    pub fn dirty_type_count(&self) -> usize {
        self.dirty_types.values().map(|s| s.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CompilationOptions tests ──

    #[test]
    fn test_compilation_options_equality() {
        let a = CompilationOptions {
            opt_level: 2,
            debug: false,
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
        };
        let b = CompilationOptions {
            opt_level: 2,
            debug: false,
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
        };
        assert_eq!(a, b);
    }

    #[test]
    fn test_compilation_options_inequality_opt_level() {
        let a = CompilationOptions {
            opt_level: 0,
            debug: false,
            target_triple: "native".to_string(),
        };
        let b = CompilationOptions {
            opt_level: 2,
            debug: false,
            target_triple: "native".to_string(),
        };
        assert_ne!(a, b);
    }

    #[test]
    fn test_compilation_options_inequality_debug() {
        let a = CompilationOptions {
            opt_level: 0,
            debug: false,
            target_triple: "native".to_string(),
        };
        let b = CompilationOptions {
            opt_level: 0,
            debug: true,
            target_triple: "native".to_string(),
        };
        assert_ne!(a, b);
    }

    #[test]
    fn test_compilation_options_inequality_target() {
        let a = CompilationOptions {
            opt_level: 0,
            debug: false,
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
        };
        let b = CompilationOptions {
            opt_level: 0,
            debug: false,
            target_triple: "aarch64-unknown-linux-gnu".to_string(),
        };
        assert_ne!(a, b);
    }

    #[test]
    fn test_compilation_options_clone() {
        let a = CompilationOptions {
            opt_level: 3,
            debug: true,
            target_triple: "wasm32-unknown-unknown".to_string(),
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_compilation_options_serde_roundtrip() {
        let opts = CompilationOptions {
            opt_level: 2,
            debug: true,
            target_triple: "x86_64-apple-darwin".to_string(),
        };
        let json = serde_json::to_string(&opts).unwrap();
        let parsed: CompilationOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(opts, parsed);
    }

    // ── FileMetadata tests ──

    #[test]
    fn test_file_metadata_serde_roundtrip() {
        let meta = FileMetadata {
            hash: "abc123".to_string(),
            timestamp: 1000,
            size: 512,
            functions: HashMap::new(),
            types: HashMap::new(),
            signature_hash: None,
        };
        let json = serde_json::to_string(&meta).unwrap();
        let parsed: FileMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.hash, "abc123");
        assert_eq!(parsed.timestamp, 1000);
        assert_eq!(parsed.size, 512);
    }

    #[test]
    fn test_file_metadata_with_functions() {
        let mut functions = HashMap::new();
        functions.insert(
            "main".to_string(),
            FunctionMetadata {
                hash: "hash1".to_string(),
                line_range: (1, 5),
                dependencies: vec!["helper".to_string()],
                is_dirty: false,
            },
        );
        let meta = FileMetadata {
            hash: "filehash".to_string(),
            timestamp: 2000,
            size: 1024,
            functions,
            types: HashMap::new(),
            signature_hash: None,
        };
        assert_eq!(meta.functions.len(), 1);
        assert_eq!(meta.functions["main"].line_range, (1, 5));
        assert!(meta.functions["main"].dependencies.contains(&"helper".to_string()));
    }

    #[test]
    fn test_file_metadata_with_types() {
        let mut types = HashMap::new();
        types.insert(
            "Point".to_string(),
            TypeMetadata {
                hash: "typehash".to_string(),
                line_range: (10, 15),
                dependencies: vec![],
            },
        );
        let meta = FileMetadata {
            hash: "filehash".to_string(),
            timestamp: 3000,
            size: 2048,
            functions: HashMap::new(),
            types,
            signature_hash: None,
        };
        assert_eq!(meta.types.len(), 1);
        assert_eq!(meta.types["Point"].line_range, (10, 15));
    }

    #[test]
    fn test_file_metadata_with_signature_hash() {
        let meta = FileMetadata {
            hash: "filehash".to_string(),
            timestamp: 4000,
            size: 4096,
            functions: HashMap::new(),
            types: HashMap::new(),
            signature_hash: Some(ModuleSignatureHash {
                hash: "sighash".to_string(),
                tc_passed: true,
            }),
        };
        assert!(meta.signature_hash.is_some());
        let sig = meta.signature_hash.unwrap();
        assert_eq!(sig.hash, "sighash");
        assert!(sig.tc_passed);
    }

    #[test]
    fn test_file_metadata_defaults() {
        let json = r#"{"hash":"h","timestamp":0,"size":0}"#;
        let meta: FileMetadata = serde_json::from_str(json).unwrap();
        assert!(meta.functions.is_empty());
        assert!(meta.types.is_empty());
        assert!(meta.signature_hash.is_none());
    }

    // ── FunctionMetadata tests ──

    #[test]
    fn test_function_metadata_equality() {
        let a = FunctionMetadata {
            hash: "hash1".to_string(),
            line_range: (1, 10),
            dependencies: vec!["dep1".to_string()],
            is_dirty: false,
        };
        let b = FunctionMetadata {
            hash: "hash1".to_string(),
            line_range: (1, 10),
            dependencies: vec!["dep1".to_string()],
            is_dirty: false,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn test_function_metadata_inequality_hash() {
        let a = FunctionMetadata {
            hash: "hash1".to_string(),
            line_range: (1, 10),
            dependencies: vec![],
            is_dirty: false,
        };
        let b = FunctionMetadata {
            hash: "hash2".to_string(),
            line_range: (1, 10),
            dependencies: vec![],
            is_dirty: false,
        };
        assert_ne!(a, b);
    }

    #[test]
    fn test_function_metadata_is_dirty_default() {
        let json = r#"{"hash":"h","line_range":[1,5],"dependencies":[]}"#;
        let meta: FunctionMetadata = serde_json::from_str(json).unwrap();
        assert!(!meta.is_dirty);
    }

    // ── TypeMetadata tests ──

    #[test]
    fn test_type_metadata_equality() {
        let a = TypeMetadata {
            hash: "h1".to_string(),
            line_range: (5, 20),
            dependencies: vec!["Other".to_string()],
        };
        let b = TypeMetadata {
            hash: "h1".to_string(),
            line_range: (5, 20),
            dependencies: vec!["Other".to_string()],
        };
        assert_eq!(a, b);
    }

    #[test]
    fn test_type_metadata_serde_roundtrip() {
        let meta = TypeMetadata {
            hash: "typehash".to_string(),
            line_range: (10, 30),
            dependencies: vec!["A".to_string(), "B".to_string()],
        };
        let json = serde_json::to_string(&meta).unwrap();
        let parsed: TypeMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(meta, parsed);
    }

    // ── ModuleSignatureHash tests ──

    #[test]
    fn test_module_signature_hash_equality() {
        let a = ModuleSignatureHash {
            hash: "sig1".to_string(),
            tc_passed: true,
        };
        let b = ModuleSignatureHash {
            hash: "sig1".to_string(),
            tc_passed: true,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn test_module_signature_hash_tc_default() {
        let json = r#"{"hash":"sig"}"#;
        let sig: ModuleSignatureHash = serde_json::from_str(json).unwrap();
        assert!(!sig.tc_passed);
    }

    // ── CacheState tests ──

    #[test]
    fn test_cache_state_default() {
        let state = CacheState::default();
        assert_eq!(state.version, super::super::CACHE_VERSION);
        assert!(!state.compiler_version.is_empty());
        assert!(state.compilation_options.is_none());
        assert_eq!(state.last_build, 0);
    }

    #[test]
    fn test_cache_state_serde_roundtrip() {
        let state = CacheState {
            version: 1,
            compiler_version: "0.1.0".to_string(),
            compilation_options: Some(CompilationOptions {
                opt_level: 2,
                debug: false,
                target_triple: "native".to_string(),
            }),
            dep_graph: super::super::graph::DependencyGraph::new(),
            last_build: 12345,
        };
        let json = serde_json::to_string(&state).unwrap();
        let parsed: CacheState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.compiler_version, "0.1.0");
        assert!(parsed.compilation_options.is_some());
        assert_eq!(parsed.last_build, 12345);
    }

    // ── DirtySet tests ──

    #[test]
    fn test_dirty_set_default_is_empty() {
        let ds = DirtySet::default();
        assert!(ds.is_empty());
        assert_eq!(ds.count(), 0);
        assert_eq!(ds.dirty_function_count(), 0);
        assert_eq!(ds.dirty_type_count(), 0);
        assert!(!ds.has_partial_changes());
    }

    #[test]
    fn test_dirty_set_modified_files() {
        let mut ds = DirtySet::default();
        ds.modified_files.insert(PathBuf::from("a.vais"));
        assert!(!ds.is_empty());
        assert_eq!(ds.count(), 1);
    }

    #[test]
    fn test_dirty_set_affected_files() {
        let mut ds = DirtySet::default();
        ds.affected_files.insert(PathBuf::from("b.vais"));
        assert!(!ds.is_empty());
        assert_eq!(ds.count(), 1);
    }

    #[test]
    fn test_dirty_set_all_dirty_files_combines() {
        let mut ds = DirtySet::default();
        ds.modified_files.insert(PathBuf::from("a.vais"));
        ds.affected_files.insert(PathBuf::from("b.vais"));
        let all = ds.all_dirty_files();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&PathBuf::from("a.vais")));
        assert!(all.contains(&PathBuf::from("b.vais")));
    }

    #[test]
    fn test_dirty_set_all_dirty_files_deduplicates() {
        let mut ds = DirtySet::default();
        ds.modified_files.insert(PathBuf::from("a.vais"));
        ds.affected_files.insert(PathBuf::from("a.vais"));
        let all = ds.all_dirty_files();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_dirty_set_count_sums_both() {
        let mut ds = DirtySet::default();
        ds.modified_files.insert(PathBuf::from("a.vais"));
        ds.modified_files.insert(PathBuf::from("b.vais"));
        ds.affected_files.insert(PathBuf::from("c.vais"));
        assert_eq!(ds.count(), 3);
    }

    #[test]
    fn test_dirty_set_has_partial_changes_true() {
        let mut ds = DirtySet::default();
        ds.dirty_functions
            .entry(PathBuf::from("a.vais"))
            .or_default()
            .insert("foo".to_string());
        assert!(ds.has_partial_changes());
    }

    #[test]
    fn test_dirty_set_has_partial_changes_false_when_modified() {
        let mut ds = DirtySet::default();
        ds.modified_files.insert(PathBuf::from("a.vais"));
        ds.dirty_functions
            .entry(PathBuf::from("a.vais"))
            .or_default()
            .insert("foo".to_string());
        assert!(!ds.has_partial_changes());
    }

    #[test]
    fn test_dirty_set_has_partial_changes_false_when_empty() {
        let ds = DirtySet::default();
        assert!(!ds.has_partial_changes());
    }

    #[test]
    fn test_dirty_set_mark_function_dirty() {
        let mut ds = DirtySet::default();
        let file = PathBuf::from("test.vais");
        ds.mark_function_dirty(file.clone(), "func1".to_string());
        ds.mark_function_dirty(file.clone(), "func2".to_string());

        assert_eq!(ds.dirty_function_count(), 2);
        let funcs = ds.get_dirty_functions(&file).unwrap();
        assert!(funcs.contains("func1"));
        assert!(funcs.contains("func2"));
    }

    #[test]
    fn test_dirty_set_mark_function_dirty_idempotent() {
        let mut ds = DirtySet::default();
        let file = PathBuf::from("test.vais");
        ds.mark_function_dirty(file.clone(), "func1".to_string());
        ds.mark_function_dirty(file.clone(), "func1".to_string());
        assert_eq!(ds.dirty_function_count(), 1);
    }

    #[test]
    fn test_dirty_set_mark_function_dirty_multiple_files() {
        let mut ds = DirtySet::default();
        ds.mark_function_dirty(PathBuf::from("a.vais"), "f1".to_string());
        ds.mark_function_dirty(PathBuf::from("b.vais"), "f2".to_string());
        ds.mark_function_dirty(PathBuf::from("b.vais"), "f3".to_string());
        assert_eq!(ds.dirty_function_count(), 3);
    }

    #[test]
    fn test_dirty_set_get_dirty_functions_none() {
        let ds = DirtySet::default();
        assert!(ds.get_dirty_functions(Path::new("nonexistent.vais")).is_none());
    }

    #[test]
    fn test_dirty_set_mark_type_dirty() {
        let mut ds = DirtySet::default();
        let file = PathBuf::from("test.vais");
        ds.mark_type_dirty(file.clone(), "Point".to_string());
        ds.mark_type_dirty(file.clone(), "Line".to_string());
        assert_eq!(ds.dirty_type_count(), 2);
    }

    #[test]
    fn test_dirty_set_mark_type_dirty_idempotent() {
        let mut ds = DirtySet::default();
        let file = PathBuf::from("test.vais");
        ds.mark_type_dirty(file.clone(), "Point".to_string());
        ds.mark_type_dirty(file.clone(), "Point".to_string());
        assert_eq!(ds.dirty_type_count(), 1);
    }

    #[test]
    fn test_dirty_set_mark_type_dirty_multiple_files() {
        let mut ds = DirtySet::default();
        ds.mark_type_dirty(PathBuf::from("a.vais"), "TypeA".to_string());
        ds.mark_type_dirty(PathBuf::from("b.vais"), "TypeB".to_string());
        assert_eq!(ds.dirty_type_count(), 2);
    }

    #[test]
    fn test_dirty_set_combined_scenario() {
        let mut ds = DirtySet::default();
        let file_a = PathBuf::from("a.vais");
        let file_b = PathBuf::from("b.vais");

        ds.modified_files.insert(file_a.clone());
        ds.affected_files.insert(file_b.clone());
        ds.mark_function_dirty(file_a.clone(), "main".to_string());
        ds.mark_type_dirty(file_b.clone(), "Config".to_string());

        assert!(!ds.is_empty());
        assert_eq!(ds.count(), 2);
        assert_eq!(ds.dirty_function_count(), 1);
        assert_eq!(ds.dirty_type_count(), 1);
        assert!(!ds.has_partial_changes()); // modified_files not empty
    }
}

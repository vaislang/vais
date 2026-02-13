#![allow(dead_code, unused_imports)] // Incremental compilation features reserved for future use
//! Incremental Compilation Cache for Vais Compiler
//!
//! Provides file-hash based caching to avoid unnecessary recompilation.
//! Tracks dependencies between files to invalidate cache when imports change.
//!
//! Performance: Uses rayon for parallel file hash computation.

/// Cache version for compatibility checking
pub(crate) const CACHE_VERSION: u32 = 1;

/// Compiler version for cache invalidation
pub(crate) const COMPILER_VERSION: &str = env!("CARGO_PKG_VERSION");

mod types;
mod graph;
mod cache;
mod detect;
mod stats;

#[cfg(test)]
mod tests;

// Re-export all public items
pub use types::{
    CacheState, CompilationOptions, DirtySet, FileMetadata, FunctionMetadata,
    ModuleSignatureHash, TypeMetadata,
};
pub use graph::DependencyGraph;
pub use cache::IncrementalCache;
pub use detect::{
    can_skip_type_checking, compute_content_hash, compute_file_hash, compute_signature_hash,
    detect_function_changes, DefinitionExtractor, FunctionChangeSet, get_cache_dir,
    get_ir_cached_object_path, has_ir_cached_object, ImportTracker, update_tc_cache,
};
pub use stats::{CacheMissReason, CacheStats, IncrementalStats};

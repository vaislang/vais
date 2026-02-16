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

mod cache;
mod detect;
mod graph;
mod stats;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public items
pub use cache::IncrementalCache;
pub use detect::{
    can_skip_type_checking, compute_content_hash, compute_file_hash, compute_signature_hash,
    detect_function_changes, get_cache_dir, get_ir_cached_object_path, has_ir_cached_object,
    update_tc_cache, DefinitionExtractor, FunctionChangeSet, ImportTracker,
};
pub use graph::DependencyGraph;
pub use stats::{CacheMissReason, CacheStats, IncrementalStats};
pub use types::{
    CacheState, CompilationOptions, DirtySet, FileMetadata, FunctionMetadata, ModuleSignatureHash,
    TypeMetadata,
};

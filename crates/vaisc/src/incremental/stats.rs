use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

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

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ── CacheStats tests ──

    #[test]
    fn test_cache_stats_creation() {
        let stats = CacheStats {
            total_files: 10,
            total_dependencies: 25,
            last_build: 1234567890,
        };
        assert_eq!(stats.total_files, 10);
        assert_eq!(stats.total_dependencies, 25);
        assert_eq!(stats.last_build, 1234567890);
    }

    // ── CacheMissReason tests ──

    #[test]
    fn test_cache_miss_reason_equality() {
        assert_eq!(CacheMissReason::NewFile, CacheMissReason::NewFile);
        assert_eq!(
            CacheMissReason::ContentHashChanged,
            CacheMissReason::ContentHashChanged
        );
        assert_ne!(CacheMissReason::NewFile, CacheMissReason::FileDeleted);
    }

    #[test]
    fn test_cache_miss_reason_dependency_changed() {
        let a = CacheMissReason::DependencyChanged("math.vais".to_string());
        let b = CacheMissReason::DependencyChanged("math.vais".to_string());
        let c = CacheMissReason::DependencyChanged("util.vais".to_string());
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_cache_miss_reason_clone() {
        let reason = CacheMissReason::DependencyChanged("dep.vais".to_string());
        let cloned = reason.clone();
        assert_eq!(reason, cloned);
    }

    #[test]
    fn test_cache_miss_reason_serde_roundtrip() {
        let reasons = vec![
            CacheMissReason::NewFile,
            CacheMissReason::ContentHashChanged,
            CacheMissReason::SignatureChanged,
            CacheMissReason::DependencyChanged("dep.vais".to_string()),
            CacheMissReason::OptionsChanged,
            CacheMissReason::FileDeleted,
            CacheMissReason::CacheCorrupted,
        ];
        for reason in &reasons {
            let json = serde_json::to_string(reason).unwrap();
            let parsed: CacheMissReason = serde_json::from_str(&json).unwrap();
            assert_eq!(*reason, parsed);
        }
    }

    // ── IncrementalStats tests ──

    #[test]
    fn test_incremental_stats_default() {
        let stats = IncrementalStats::default();
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.files_checked, 0);
        assert_eq!(stats.files_skipped, 0);
        assert_eq!(stats.signature_hits, 0);
        assert_eq!(stats.total_check_time_ms, 0);
        assert!(stats.miss_reasons.is_empty());
    }

    #[test]
    fn test_hit_rate_zero_files_checked() {
        let stats = IncrementalStats::default();
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_hit_rate_all_hits() {
        let stats = IncrementalStats {
            cache_hits: 10,
            cache_misses: 0,
            files_checked: 10,
            ..Default::default()
        };
        assert!((stats.hit_rate() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_hit_rate_all_misses() {
        let stats = IncrementalStats {
            cache_hits: 0,
            cache_misses: 5,
            files_checked: 5,
            ..Default::default()
        };
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_hit_rate_mixed() {
        let stats = IncrementalStats {
            cache_hits: 2,
            cache_misses: 1,
            files_checked: 3,
            ..Default::default()
        };
        let expected = (2.0 / 3.0) * 100.0;
        assert!((stats.hit_rate() - expected).abs() < 0.01);
    }

    #[test]
    fn test_hit_rate_50_percent() {
        let stats = IncrementalStats {
            cache_hits: 5,
            cache_misses: 5,
            files_checked: 10,
            ..Default::default()
        };
        assert!((stats.hit_rate() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_incremental_stats_with_miss_reasons() {
        let mut stats = IncrementalStats::default();
        stats.cache_misses = 2;
        stats.files_checked = 3;
        stats.cache_hits = 1;

        let file = PathBuf::from("test.vais");
        stats
            .miss_reasons
            .entry(file.clone())
            .or_default()
            .push(CacheMissReason::ContentHashChanged);

        assert_eq!(stats.miss_reasons.len(), 1);
        assert_eq!(stats.miss_reasons[&file].len(), 1);
    }

    #[test]
    fn test_incremental_stats_clone() {
        let mut stats = IncrementalStats {
            cache_hits: 5,
            cache_misses: 3,
            files_checked: 8,
            files_skipped: 1,
            signature_hits: 2,
            total_check_time_ms: 42,
            miss_reasons: HashMap::new(),
        };
        stats
            .miss_reasons
            .insert(PathBuf::from("a.vais"), vec![CacheMissReason::NewFile]);

        let cloned = stats.clone();
        assert_eq!(cloned.cache_hits, 5);
        assert_eq!(cloned.miss_reasons.len(), 1);
    }

    #[test]
    fn test_hit_rate_single_file() {
        let stats = IncrementalStats {
            cache_hits: 1,
            cache_misses: 0,
            files_checked: 1,
            ..Default::default()
        };
        assert!((stats.hit_rate() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_hit_rate_large_numbers() {
        let stats = IncrementalStats {
            cache_hits: 999,
            cache_misses: 1,
            files_checked: 1000,
            ..Default::default()
        };
        assert!((stats.hit_rate() - 99.9).abs() < 0.01);
    }

    #[test]
    fn test_incremental_stats_multiple_miss_reasons() {
        let mut stats = IncrementalStats::default();
        let file = PathBuf::from("test.vais");
        stats
            .miss_reasons
            .entry(file.clone())
            .or_default()
            .push(CacheMissReason::ContentHashChanged);
        stats
            .miss_reasons
            .entry(file.clone())
            .or_default()
            .push(CacheMissReason::DependencyChanged("dep.vais".to_string()));

        assert_eq!(stats.miss_reasons[&file].len(), 2);
    }

    #[test]
    fn test_cache_miss_reason_all_variants() {
        let variants = vec![
            CacheMissReason::NewFile,
            CacheMissReason::ContentHashChanged,
            CacheMissReason::SignatureChanged,
            CacheMissReason::DependencyChanged("x".to_string()),
            CacheMissReason::OptionsChanged,
            CacheMissReason::FileDeleted,
            CacheMissReason::CacheCorrupted,
        ];
        assert_eq!(variants.len(), 7);
        // Ensure all are distinct
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn test_hit_rate_one_hit_one_miss() {
        let stats = IncrementalStats {
            cache_hits: 1,
            cache_misses: 1,
            files_checked: 2,
            ..Default::default()
        };
        assert!((stats.hit_rate() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_incremental_stats_check_time() {
        let stats = IncrementalStats {
            total_check_time_ms: 150,
            ..Default::default()
        };
        assert_eq!(stats.total_check_time_ms, 150);
    }

    #[test]
    fn test_incremental_stats_signature_hits() {
        let stats = IncrementalStats {
            signature_hits: 10,
            ..Default::default()
        };
        assert_eq!(stats.signature_hits, 10);
    }

    #[test]
    fn test_incremental_stats_files_skipped() {
        let stats = IncrementalStats {
            files_skipped: 5,
            ..Default::default()
        };
        assert_eq!(stats.files_skipped, 5);
    }

    #[test]
    fn test_hit_rate_only_hits() {
        let stats = IncrementalStats {
            cache_hits: 42,
            cache_misses: 0,
            files_checked: 42,
            ..Default::default()
        };
        assert!((stats.hit_rate() - 100.0).abs() < 0.001);
    }
}

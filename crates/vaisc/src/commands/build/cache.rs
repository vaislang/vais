//! Binary caching for compiled artifacts
//!
//! Provides SHA-256 content-addressable caching for compiled binaries.
//! When source files haven't changed, the cached binary is reused instead
//! of recompiling.

#![allow(dead_code)]
//!
//! Cache layout:
//! ```text
//! .vais-cache/
//!   bin/
//!     <sha256-of-sources>.bin  -> compiled binary
//!     <sha256-of-sources>.meta -> JSON metadata (source hash, timestamp, opts)
//! ```

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Binary cache for compiled artifacts
pub struct BinaryCache {
    /// Root directory for the cache (default: .vais-cache/)
    cache_dir: PathBuf,
}

/// Metadata stored alongside each cached binary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry {
    /// SHA-256 hash of all source inputs
    pub source_hash: String,
    /// Original source file path
    pub source_path: String,
    /// Timestamp when cached
    pub cached_at: String,
    /// Optimization level used
    pub opt_level: u8,
    /// Target triple
    pub target: String,
    /// Whether debug info was included
    pub debug: bool,
}

impl BinaryCache {
    /// Create a new binary cache at the given directory
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Create a cache at the default location (.vais-cache/ in project root)
    pub fn default_for_project(project_dir: &Path) -> Self {
        Self {
            cache_dir: project_dir.join(".vais-cache"),
        }
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Check if a cached binary exists for the given source hash
    pub fn lookup(&self, source_hash: &str) -> Option<PathBuf> {
        let bin_path = self.bin_dir().join(format!("{}.bin", source_hash));
        let meta_path = self.bin_dir().join(format!("{}.meta", source_hash));

        if bin_path.exists() && meta_path.exists() {
            Some(bin_path)
        } else {
            None
        }
    }

    /// Store a compiled binary in the cache
    pub fn store(
        &self,
        source_hash: &str,
        binary_path: &Path,
        entry: &CacheEntry,
    ) -> Result<(), String> {
        let bin_dir = self.bin_dir();
        fs::create_dir_all(&bin_dir)
            .map_err(|e| format!("failed to create cache directory: {}", e))?;

        let cached_bin = bin_dir.join(format!("{}.bin", source_hash));
        let meta_path = bin_dir.join(format!("{}.meta", source_hash));

        fs::copy(binary_path, &cached_bin)
            .map_err(|e| format!("failed to cache binary: {}", e))?;

        let meta_json = serde_json::to_string_pretty(entry)
            .map_err(|e| format!("failed to serialize cache metadata: {}", e))?;
        fs::write(&meta_path, meta_json)
            .map_err(|e| format!("failed to write cache metadata: {}", e))?;

        Ok(())
    }

    /// Copy a cached binary to the target output location
    pub fn restore(&self, source_hash: &str, output_path: &Path) -> Result<(), String> {
        let cached_bin = self.bin_dir().join(format!("{}.bin", source_hash));
        fs::copy(&cached_bin, output_path)
            .map_err(|e| format!("failed to restore cached binary: {}", e))?;

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o755);
            let _ = fs::set_permissions(output_path, perms);
        }

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let bin_dir = self.bin_dir();
        if !bin_dir.exists() {
            return CacheStats::default();
        }

        let mut stats = CacheStats::default();
        if let Ok(entries) = fs::read_dir(&bin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    match ext {
                        "bin" => {
                            stats.entries += 1;
                            if let Ok(meta) = fs::metadata(&path) {
                                stats.total_size += meta.len();
                            }
                        }
                        "meta" => {} // counted with .bin
                        _ => {}
                    }
                }
            }
        }
        stats
    }

    /// Remove all cached artifacts
    pub fn clear(&self) -> Result<usize, String> {
        let bin_dir = self.bin_dir();
        if !bin_dir.exists() {
            return Ok(0);
        }

        let mut removed = 0;
        if let Ok(entries) = fs::read_dir(&bin_dir) {
            for entry in entries.flatten() {
                if fs::remove_file(entry.path()).is_ok() {
                    removed += 1;
                }
            }
        }

        Ok(removed)
    }

    /// Remove cache entries older than the given age in seconds
    pub fn evict_older_than(&self, max_age_secs: u64) -> Result<usize, String> {
        let bin_dir = self.bin_dir();
        if !bin_dir.exists() {
            return Ok(0);
        }

        let now = std::time::SystemTime::now();
        let mut removed = 0;

        if let Ok(entries) = fs::read_dir(&bin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(meta) = fs::metadata(&path) {
                    if let Ok(modified) = meta.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age.as_secs() > max_age_secs
                                && fs::remove_file(&path).is_ok()
                            {
                                removed += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(removed)
    }

    fn bin_dir(&self) -> PathBuf {
        self.cache_dir.join("bin")
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cached entries
    pub entries: usize,
    /// Total size in bytes
    pub total_size: u64,
}

impl CacheStats {
    /// Format total size in human-readable form
    pub fn human_size(&self) -> String {
        if self.total_size < 1024 {
            format!("{} B", self.total_size)
        } else if self.total_size < 1024 * 1024 {
            format!("{:.1} KB", self.total_size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.total_size as f64 / (1024.0 * 1024.0))
        }
    }
}

/// Compute a SHA-256 hash of all source files that contribute to a build
pub fn compute_source_hash(
    source_files: &[PathBuf],
    opt_level: u8,
    target: &str,
    debug: bool,
) -> Result<String, String> {
    let mut hasher = Sha256::new();

    // Sort paths for deterministic hashing
    let mut sorted_files: Vec<_> = source_files.to_vec();
    sorted_files.sort();

    // Hash each source file's contents
    let mut file_hashes = BTreeMap::new();
    for file in &sorted_files {
        let content = fs::read(file)
            .map_err(|e| format!("failed to read '{}' for hashing: {}", file.display(), e))?;
        let mut file_hasher = Sha256::new();
        file_hasher.update(&content);
        let hash = hex::encode(file_hasher.finalize());
        file_hashes.insert(file.display().to_string(), hash);
    }

    // Hash the deterministic map
    for (path, hash) in &file_hashes {
        Digest::update(&mut hasher, path.as_bytes());
        Digest::update(&mut hasher, b":");
        Digest::update(&mut hasher, hash.as_bytes());
        Digest::update(&mut hasher, b"\n");
    }

    // Include build parameters in the hash
    Digest::update(&mut hasher, format!("opt:{}\n", opt_level).as_bytes());
    Digest::update(&mut hasher, format!("target:{}\n", target).as_bytes());
    Digest::update(&mut hasher, format!("debug:{}\n", debug).as_bytes());

    Ok(hex::encode(hasher.finalize()))
}

/// Get the current UTC timestamp as ISO 8601 string
pub fn utc_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple ISO-ish format without chrono dependency
    format!("{}Z", secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_cache_lookup_miss() {
        let dir = tempfile::tempdir().unwrap();
        let cache = BinaryCache::new(dir.path().to_path_buf());
        assert!(cache.lookup("nonexistent").is_none());
    }

    #[test]
    fn test_cache_store_and_lookup() {
        let dir = tempfile::tempdir().unwrap();
        let cache = BinaryCache::new(dir.path().to_path_buf());

        // Create a fake binary
        let mut fake_bin = tempfile::NamedTempFile::new().unwrap();
        fake_bin.write_all(b"fake binary content").unwrap();

        let entry = CacheEntry {
            source_hash: "abc123".to_string(),
            source_path: "main.vais".to_string(),
            cached_at: utc_timestamp(),
            opt_level: 0,
            target: "native".to_string(),
            debug: false,
        };

        cache.store("abc123", fake_bin.path(), &entry).unwrap();

        let result = cache.lookup("abc123");
        assert!(result.is_some());
    }

    #[test]
    fn test_cache_restore() {
        let dir = tempfile::tempdir().unwrap();
        let cache = BinaryCache::new(dir.path().to_path_buf());

        let mut fake_bin = tempfile::NamedTempFile::new().unwrap();
        fake_bin.write_all(b"binary data").unwrap();

        let entry = CacheEntry {
            source_hash: "def456".to_string(),
            source_path: "main.vais".to_string(),
            cached_at: utc_timestamp(),
            opt_level: 2,
            target: "x86_64".to_string(),
            debug: true,
        };

        cache.store("def456", fake_bin.path(), &entry).unwrap();

        let output = dir.path().join("output_binary");
        cache.restore("def456", &output).unwrap();

        let content = fs::read(&output).unwrap();
        assert_eq!(content, b"binary data");
    }

    #[test]
    fn test_cache_stats_empty() {
        let dir = tempfile::tempdir().unwrap();
        let cache = BinaryCache::new(dir.path().to_path_buf());
        let stats = cache.stats();
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.total_size, 0);
    }

    #[test]
    fn test_cache_stats_with_entries() {
        let dir = tempfile::tempdir().unwrap();
        let cache = BinaryCache::new(dir.path().to_path_buf());

        let mut fake_bin = tempfile::NamedTempFile::new().unwrap();
        fake_bin.write_all(b"some data").unwrap();

        let entry = CacheEntry {
            source_hash: "hash1".to_string(),
            source_path: "a.vais".to_string(),
            cached_at: utc_timestamp(),
            opt_level: 0,
            target: "native".to_string(),
            debug: false,
        };

        cache.store("hash1", fake_bin.path(), &entry).unwrap();

        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
        assert!(stats.total_size > 0);
    }

    #[test]
    fn test_cache_clear() {
        let dir = tempfile::tempdir().unwrap();
        let cache = BinaryCache::new(dir.path().to_path_buf());

        let mut fake_bin = tempfile::NamedTempFile::new().unwrap();
        fake_bin.write_all(b"data").unwrap();

        let entry = CacheEntry {
            source_hash: "hash2".to_string(),
            source_path: "b.vais".to_string(),
            cached_at: utc_timestamp(),
            opt_level: 0,
            target: "native".to_string(),
            debug: false,
        };

        cache.store("hash2", fake_bin.path(), &entry).unwrap();
        assert!(cache.lookup("hash2").is_some());

        let removed = cache.clear().unwrap();
        assert!(removed > 0);
        assert!(cache.lookup("hash2").is_none());
    }

    #[test]
    fn test_compute_source_hash_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        let file1 = dir.path().join("a.vais");
        let file2 = dir.path().join("b.vais");
        fs::write(&file1, "F main() -> i64 { 0 }").unwrap();
        fs::write(&file2, "F helper() -> i64 { 1 }").unwrap();

        let hash1 = compute_source_hash(&[file1.clone(), file2.clone()], 0, "native", false).unwrap();
        let hash2 = compute_source_hash(&[file2, file1], 0, "native", false).unwrap();

        // Order shouldn't matter (sorted internally)
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_source_hash_changes_with_content() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("main.vais");

        fs::write(&file, "F main() -> i64 { 0 }").unwrap();
        let hash1 = compute_source_hash(&[file.clone()], 0, "native", false).unwrap();

        fs::write(&file, "F main() -> i64 { 1 }").unwrap();
        let hash2 = compute_source_hash(&[file], 0, "native", false).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_compute_source_hash_changes_with_opts() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("main.vais");
        fs::write(&file, "F main() -> i64 { 0 }").unwrap();

        let hash_o0 = compute_source_hash(&[file.clone()], 0, "native", false).unwrap();
        let hash_o2 = compute_source_hash(&[file], 2, "native", false).unwrap();

        assert_ne!(hash_o0, hash_o2);
    }

    #[test]
    fn test_human_size() {
        assert_eq!(CacheStats { entries: 0, total_size: 0 }.human_size(), "0 B");
        assert_eq!(CacheStats { entries: 0, total_size: 512 }.human_size(), "512 B");
        assert_eq!(CacheStats { entries: 0, total_size: 2048 }.human_size(), "2.0 KB");
        assert_eq!(CacheStats { entries: 0, total_size: 1_500_000 }.human_size(), "1.4 MB");
    }

    #[test]
    fn test_utc_timestamp() {
        let ts = utc_timestamp();
        assert!(ts.ends_with('Z'));
        assert!(ts.len() > 5);
    }
}

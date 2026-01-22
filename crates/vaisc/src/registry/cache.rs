//! Package cache management
//!
//! Handles caching of downloaded packages in ~/.vais/registry/

use super::error::{RegistryError, RegistryResult};
use super::version::Version;
use std::fs;
use std::path::{Path, PathBuf};

/// Package cache location and operations
pub struct PackageCache {
    /// Root cache directory (e.g., ~/.vais/registry)
    root: PathBuf,
}

impl PackageCache {
    /// Create a new package cache with default location (~/.vais/registry)
    pub fn new() -> RegistryResult<Self> {
        let home = dirs::home_dir().ok_or_else(|| RegistryError::ReadError {
            path: PathBuf::from("~"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "home directory not found"),
        })?;

        let root = home.join(".vais").join("registry");
        Self::with_root(root)
    }

    /// Create a cache with a custom root directory
    pub fn with_root(root: PathBuf) -> RegistryResult<Self> {
        // Ensure cache directories exist
        fs::create_dir_all(&root).map_err(|e| RegistryError::CreateDirError {
            path: root.clone(),
            source: e,
        })?;

        fs::create_dir_all(root.join("cache")).map_err(|e| RegistryError::CreateDirError {
            path: root.join("cache"),
            source: e,
        })?;

        fs::create_dir_all(root.join("index")).map_err(|e| RegistryError::CreateDirError {
            path: root.join("index"),
            source: e,
        })?;

        Ok(Self { root })
    }

    /// Get the root cache directory
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get the cache directory for a specific package
    pub fn package_dir(&self, name: &str) -> PathBuf {
        self.root.join("cache").join(name)
    }

    /// Get the path for a specific version's archive
    pub fn archive_path(&self, name: &str, version: &Version) -> PathBuf {
        self.package_dir(name).join(format!("{}.tar.gz", version))
    }

    /// Get the path for extracted package source
    pub fn extracted_path(&self, name: &str, version: &Version) -> PathBuf {
        self.package_dir(name)
            .join(version.to_string())
            .join("extracted")
    }

    /// Check if a package version is cached (archive exists)
    pub fn has_archive(&self, name: &str, version: &Version) -> bool {
        self.archive_path(name, version).exists()
    }

    /// Check if a package version is extracted
    pub fn has_extracted(&self, name: &str, version: &Version) -> bool {
        self.extracted_path(name, version).exists()
    }

    /// Store an archive in the cache
    pub fn store_archive(&self, name: &str, version: &Version, data: &[u8]) -> RegistryResult<PathBuf> {
        let pkg_dir = self.package_dir(name);
        fs::create_dir_all(&pkg_dir).map_err(|e| RegistryError::CreateDirError {
            path: pkg_dir.clone(),
            source: e,
        })?;

        let archive_path = self.archive_path(name, version);
        fs::write(&archive_path, data).map_err(|e| RegistryError::WriteError {
            path: archive_path.clone(),
            source: e,
        })?;

        Ok(archive_path)
    }

    /// Read an archive from the cache
    pub fn read_archive(&self, name: &str, version: &Version) -> RegistryResult<Vec<u8>> {
        let path = self.archive_path(name, version);
        fs::read(&path).map_err(|e| RegistryError::ReadError { path, source: e })
    }

    /// Mark a version as extracted (create the extraction directory)
    pub fn mark_extracted(&self, name: &str, version: &Version) -> RegistryResult<PathBuf> {
        let path = self.extracted_path(name, version);
        fs::create_dir_all(&path).map_err(|e| RegistryError::CreateDirError {
            path: path.clone(),
            source: e,
        })?;
        Ok(path)
    }

    /// Remove a specific version from cache
    pub fn remove_version(&self, name: &str, version: &Version) -> RegistryResult<()> {
        let version_dir = self.package_dir(name).join(version.to_string());
        if version_dir.exists() {
            fs::remove_dir_all(&version_dir).map_err(|e| RegistryError::WriteError {
                path: version_dir,
                source: e,
            })?;
        }

        let archive = self.archive_path(name, version);
        if archive.exists() {
            fs::remove_file(&archive).map_err(|e| RegistryError::WriteError {
                path: archive,
                source: e,
            })?;
        }

        Ok(())
    }

    /// Remove all versions of a package
    pub fn remove_package(&self, name: &str) -> RegistryResult<()> {
        let pkg_dir = self.package_dir(name);
        if pkg_dir.exists() {
            fs::remove_dir_all(&pkg_dir).map_err(|e| RegistryError::WriteError {
                path: pkg_dir,
                source: e,
            })?;
        }
        Ok(())
    }

    /// List all cached packages
    pub fn list_packages(&self) -> RegistryResult<Vec<String>> {
        let cache_dir = self.root.join("cache");
        if !cache_dir.exists() {
            return Ok(Vec::new());
        }

        let mut packages = Vec::new();
        for entry in fs::read_dir(&cache_dir).map_err(|e| RegistryError::ReadError {
            path: cache_dir.clone(),
            source: e,
        })? {
            let entry = entry.map_err(|e| RegistryError::ReadError {
                path: cache_dir.clone(),
                source: e,
            })?;
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    packages.push(name.to_string());
                }
            }
        }
        Ok(packages)
    }

    /// List all cached versions of a package
    pub fn list_versions(&self, name: &str) -> RegistryResult<Vec<Version>> {
        let pkg_dir = self.package_dir(name);
        if !pkg_dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();
        for entry in fs::read_dir(&pkg_dir).map_err(|e| RegistryError::ReadError {
            path: pkg_dir.clone(),
            source: e,
        })? {
            let entry = entry.map_err(|e| RegistryError::ReadError {
                path: pkg_dir.clone(),
                source: e,
            })?;
            let path = entry.path();

            // Check for .tar.gz files
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Some(version_str) = name.strip_suffix(".tar.gz") {
                        if let Ok(version) = Version::parse(version_str) {
                            versions.push(version);
                        }
                    }
                }
            }
        }

        versions.sort();
        Ok(versions)
    }

    /// Get cache statistics
    pub fn stats(&self) -> RegistryResult<CacheStats> {
        let packages = self.list_packages()?;
        let mut total_versions = 0;
        let mut total_size = 0u64;

        for pkg in &packages {
            let versions = self.list_versions(pkg)?;
            total_versions += versions.len();

            for version in &versions {
                let archive = self.archive_path(pkg, version);
                if archive.exists() {
                    if let Ok(metadata) = fs::metadata(&archive) {
                        total_size += metadata.len();
                    }
                }
            }
        }

        Ok(CacheStats {
            packages: packages.len(),
            versions: total_versions,
            size_bytes: total_size,
        })
    }

    /// Clear entire cache
    pub fn clear(&self) -> RegistryResult<()> {
        let cache_dir = self.root.join("cache");
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir).map_err(|e| RegistryError::WriteError {
                path: cache_dir.clone(),
                source: e,
            })?;
            fs::create_dir_all(&cache_dir).map_err(|e| RegistryError::CreateDirError {
                path: cache_dir,
                source: e,
            })?;
        }
        Ok(())
    }

    /// Get index cache path for a registry
    pub fn index_path(&self, registry_name: &str) -> PathBuf {
        self.root.join("index").join(registry_name)
    }

    /// Store index data
    pub fn store_index(&self, registry_name: &str, data: &[u8]) -> RegistryResult<()> {
        let index_dir = self.root.join("index");
        fs::create_dir_all(&index_dir).map_err(|e| RegistryError::CreateDirError {
            path: index_dir.clone(),
            source: e,
        })?;

        let path = self.index_path(registry_name);
        fs::write(&path, data).map_err(|e| RegistryError::WriteError { path, source: e })
    }

    /// Read cached index
    pub fn read_index(&self, registry_name: &str) -> RegistryResult<Vec<u8>> {
        let path = self.index_path(registry_name);
        fs::read(&path).map_err(|e| RegistryError::ReadError { path, source: e })
    }
}

impl Default for PackageCache {
    fn default() -> Self {
        Self::new().expect("Failed to create default package cache")
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub packages: usize,
    pub versions: usize,
    pub size_bytes: u64,
}

impl CacheStats {
    /// Format size for display
    pub fn size_display(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size_bytes >= GB {
            format!("{:.2} GB", self.size_bytes as f64 / GB as f64)
        } else if self.size_bytes >= MB {
            format!("{:.2} MB", self.size_bytes as f64 / MB as f64)
        } else if self.size_bytes >= KB {
            format!("{:.2} KB", self.size_bytes as f64 / KB as f64)
        } else {
            format!("{} bytes", self.size_bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cache_operations() {
        let dir = tempdir().unwrap();
        let cache = PackageCache::with_root(dir.path().to_path_buf()).unwrap();

        let version = Version::new(1, 0, 0);

        // Store archive
        cache
            .store_archive("test-pkg", &version, b"test data")
            .unwrap();
        assert!(cache.has_archive("test-pkg", &version));

        // Read archive
        let data = cache.read_archive("test-pkg", &version).unwrap();
        assert_eq!(data, b"test data");

        // List versions
        let versions = cache.list_versions("test-pkg").unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0], version);

        // Remove version
        cache.remove_version("test-pkg", &version).unwrap();
        assert!(!cache.has_archive("test-pkg", &version));
    }

    #[test]
    fn test_cache_stats() {
        let dir = tempdir().unwrap();
        let cache = PackageCache::with_root(dir.path().to_path_buf()).unwrap();

        cache
            .store_archive("pkg1", &Version::new(1, 0, 0), b"data1")
            .unwrap();
        cache
            .store_archive("pkg1", &Version::new(1, 1, 0), b"data2")
            .unwrap();
        cache
            .store_archive("pkg2", &Version::new(0, 1, 0), b"data3")
            .unwrap();

        let stats = cache.stats().unwrap();
        assert_eq!(stats.packages, 2);
        assert_eq!(stats.versions, 3);
    }
}

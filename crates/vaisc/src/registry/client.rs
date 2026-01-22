//! Registry client for fetching packages
//!
//! Supports HTTP and local filesystem registries.

use super::archive::{sha256_hex, unpack_from_bytes, verify_checksum};
use super::cache::PackageCache;
use super::error::{RegistryError, RegistryResult};
use super::index::{PackageIndex, PackageMetadata, VersionEntry};
use super::source::RegistrySource;
use super::version::{Version, VersionReq};
use std::fs;
use std::path::{Path, PathBuf};

/// Registry client for fetching and managing packages
pub struct RegistryClient {
    source: RegistrySource,
    cache: PackageCache,
    index: Option<PackageIndex>,
}

impl RegistryClient {
    /// Create a new registry client
    pub fn new(source: RegistrySource) -> RegistryResult<Self> {
        let cache = PackageCache::new()?;
        Ok(Self {
            source,
            cache,
            index: None,
        })
    }

    /// Create with custom cache
    pub fn with_cache(source: RegistrySource, cache: PackageCache) -> Self {
        Self {
            source,
            cache,
            index: None,
        }
    }

    /// Get the cache
    pub fn cache(&self) -> &PackageCache {
        &self.cache
    }

    /// Fetch and update the package index
    pub fn update_index(&mut self) -> RegistryResult<()> {
        // Clone values to avoid borrow issues
        let (url, token, path_opt) = match &self.source {
            RegistrySource::Http { url, token } => {
                (Some(url.clone()), token.clone(), None)
            }
            RegistrySource::Local { path } => {
                (None, None, Some(path.clone()))
            }
            RegistrySource::Git { url, branch } => {
                let index_url = format!("{}/raw/{}/index.json", url.trim_end_matches(".git"), branch);
                (Some(index_url), None, None)
            }
        };

        if let Some(path) = path_opt {
            self.load_local_index(&path)?;
        } else if let Some(url) = url {
            self.fetch_http_index(&url, token.as_deref())?;
        }
        Ok(())
    }

    /// Fetch HTTP index
    fn fetch_http_index(&mut self, url: &str, token: Option<&str>) -> RegistryResult<()> {
        let index_url = format!("{}/index.json", url.trim_end_matches('/'));

        let mut request = ureq::get(&index_url);
        if let Some(t) = token {
            request = request.set("Authorization", &format!("Bearer {}", t));
        }

        let response = request.call().map_err(|e| match e {
            ureq::Error::Status(code, _) => RegistryError::HttpError {
                status: code,
                message: format!("failed to fetch index from {}", index_url),
            },
            _ => RegistryError::RegistryUnreachable {
                url: index_url.clone(),
            },
        })?;

        let body = response
            .into_string()
            .map_err(|_| RegistryError::RegistryUnreachable { url: index_url })?;

        // Parse and cache index
        self.index = Some(PackageIndex::from_json(&body)?);

        // Store in cache
        let registry_name = self.source.name();
        self.cache.store_index(&registry_name, body.as_bytes())?;

        Ok(())
    }

    /// Load local index
    fn load_local_index(&mut self, path: &Path) -> RegistryResult<()> {
        let index_path = path.join("index.json");
        let content = fs::read_to_string(&index_path).map_err(|e| RegistryError::ReadError {
            path: index_path,
            source: e,
        })?;

        self.index = Some(PackageIndex::from_json(&content)?);
        Ok(())
    }

    /// Load cached index (if available)
    pub fn load_cached_index(&mut self) -> RegistryResult<bool> {
        let registry_name = self.source.name();
        match self.cache.read_index(&registry_name) {
            Ok(data) => {
                let content = String::from_utf8_lossy(&data);
                self.index = Some(PackageIndex::from_json(&content)?);
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    /// Search for packages
    pub fn search(&self, query: &str) -> RegistryResult<Vec<&PackageMetadata>> {
        let index = self.index.as_ref().ok_or_else(|| RegistryError::ResolutionError {
            message: "index not loaded, run update first".to_string(),
        })?;

        Ok(index.search(query))
    }

    /// Get package metadata
    pub fn get_package(&self, name: &str) -> RegistryResult<&PackageMetadata> {
        let index = self.index.as_ref().ok_or_else(|| RegistryError::ResolutionError {
            message: "index not loaded, run update first".to_string(),
        })?;

        index
            .get(name)
            .ok_or_else(|| RegistryError::PackageNotFound { name: name.to_string() })
    }

    /// Find best matching version
    pub fn find_version(&self, name: &str, req: &VersionReq) -> RegistryResult<&VersionEntry> {
        let pkg = self.get_package(name)?;
        let versions: Vec<Version> = pkg
            .available_versions()
            .iter()
            .map(|v| v.version.clone())
            .collect();

        let best = req.best_match(&versions).ok_or_else(|| RegistryError::NoMatchingVersion {
            name: name.to_string(),
            req: req.to_string(),
        })?;

        pkg.get_version(best).ok_or_else(|| RegistryError::VersionNotFound {
            name: name.to_string(),
            version: best.to_string(),
        })
    }

    /// Download a package version
    pub fn download(&self, name: &str, version: &Version) -> RegistryResult<PathBuf> {
        // Check cache first
        if self.cache.has_extracted(name, version) {
            return Ok(self.cache.extracted_path(name, version));
        }

        let pkg = self.get_package(name)?;
        let entry = pkg.get_version(version).ok_or_else(|| RegistryError::VersionNotFound {
            name: name.to_string(),
            version: version.to_string(),
        })?;

        // Download archive
        let data = self.fetch_archive(name, version, entry)?;

        // Verify checksum
        if !verify_checksum(&data, &entry.checksum) {
            return Err(RegistryError::ChecksumMismatch {
                name: name.to_string(),
                version: version.to_string(),
                expected: entry.checksum.clone(),
                actual: sha256_hex(&data),
            });
        }

        // Store and extract
        self.cache.store_archive(name, version, &data)?;
        let extract_path = self.cache.mark_extracted(name, version)?;
        unpack_from_bytes(&data, &extract_path)?;

        Ok(extract_path)
    }

    /// Fetch archive data
    fn fetch_archive(&self, name: &str, version: &Version, entry: &VersionEntry) -> RegistryResult<Vec<u8>> {
        match &self.source {
            RegistrySource::Http { url, token } => {
                let archive_url = entry
                    .download_url
                    .clone()
                    .unwrap_or_else(|| format!("{}/packages/{}/{}.tar.gz", url, name, version));

                let mut request = ureq::get(&archive_url);
                if let Some(t) = token {
                    request = request.set("Authorization", &format!("Bearer {}", t));
                }

                let response = request.call().map_err(|e| match e {
                    ureq::Error::Status(code, _) => RegistryError::HttpError {
                        status: code,
                        message: format!("failed to download {}", archive_url),
                    },
                    _ => RegistryError::RegistryUnreachable { url: archive_url.clone() },
                })?;

                let mut data = Vec::new();
                response
                    .into_reader()
                    .read_to_end(&mut data)
                    .map_err(|_| RegistryError::ArchiveError {
                        message: "failed to read response body".to_string(),
                    })?;

                Ok(data)
            }
            RegistrySource::Local { path } => {
                let archive_path = path.join("packages").join(name).join(format!("{}.tar.gz", version));
                fs::read(&archive_path).map_err(|e| RegistryError::ReadError {
                    path: archive_path,
                    source: e,
                })
            }
            RegistrySource::Git { url, branch } => {
                // For git, use raw file access
                let archive_url = format!(
                    "{}/raw/{}/packages/{}/{}.tar.gz",
                    url.trim_end_matches(".git"),
                    branch,
                    name,
                    version
                );

                let response = ureq::get(&archive_url).call().map_err(|e| match e {
                    ureq::Error::Status(code, _) => RegistryError::HttpError {
                        status: code,
                        message: format!("failed to download {}", archive_url),
                    },
                    _ => RegistryError::RegistryUnreachable { url: archive_url.clone() },
                })?;

                let mut data = Vec::new();
                response
                    .into_reader()
                    .read_to_end(&mut data)
                    .map_err(|_| RegistryError::ArchiveError {
                        message: "failed to read response body".to_string(),
                    })?;

                Ok(data)
            }
        }
    }

    /// Install a package (download and extract if needed)
    pub fn install(&self, name: &str, req: &VersionReq) -> RegistryResult<InstalledPackage> {
        let entry = self.find_version(name, req)?;
        let version = entry.version.clone();
        let path = self.download(name, &version)?;

        Ok(InstalledPackage {
            name: name.to_string(),
            version,
            path,
        })
    }

    /// Check if a package version is installed
    pub fn is_installed(&self, name: &str, version: &Version) -> bool {
        self.cache.has_extracted(name, version)
    }

    /// Get installed path for a package
    pub fn installed_path(&self, name: &str, version: &Version) -> Option<PathBuf> {
        if self.is_installed(name, version) {
            Some(self.cache.extracted_path(name, version))
        } else {
            None
        }
    }
}

/// Information about an installed package
#[derive(Debug, Clone)]
pub struct InstalledPackage {
    pub name: String,
    pub version: Version,
    pub path: PathBuf,
}

use std::io::Read;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_index(path: &Path) {
        let index_content = r#"[
            {
                "name": "test-pkg",
                "description": "A test package",
                "versions": [
                    {
                        "version": "1.0.0",
                        "checksum": "abc123",
                        "dependencies": {}
                    }
                ],
                "keywords": ["test"]
            }
        ]"#;

        fs::write(path.join("index.json"), index_content).unwrap();
    }

    #[test]
    fn test_local_registry() {
        let registry_dir = tempdir().unwrap();
        let cache_dir = tempdir().unwrap();

        create_test_index(registry_dir.path());

        let source = RegistrySource::local(registry_dir.path());
        let cache = PackageCache::with_root(cache_dir.path().to_path_buf()).unwrap();
        let mut client = RegistryClient::with_cache(source, cache);

        client.update_index().unwrap();

        let results = client.search("test").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test-pkg");
    }
}

//! Package archive storage

use crate::error::{ServerError, ServerResult};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};

/// Maximum uncompressed archive size (100MB)
const MAX_UNCOMPRESSED_SIZE: u64 = 100 * 1024 * 1024;
/// Maximum number of files in an archive
const MAX_FILE_COUNT: usize = 10_000;

/// Package storage manager
pub struct PackageStorage {
    root: PathBuf,
}

impl PackageStorage {
    /// Create a new storage manager
    pub fn new(root: PathBuf) -> ServerResult<Self> {
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    /// Validate a path component for safety (no path traversal)
    fn validate_path_component(s: &str) -> ServerResult<()> {
        if s.contains("..") || s.contains('/') || s.contains('\\') || s.contains('\0') {
            return Err(ServerError::Archive("Invalid path component".to_string()));
        }
        Ok(())
    }

    /// Get the path to a package directory
    pub fn package_dir(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    /// Get the path to a specific version archive
    pub fn archive_path(&self, name: &str, version: &str) -> PathBuf {
        self.package_dir(name).join(format!("{}.tar.gz", version))
    }

    /// Store a package archive
    pub fn store_archive(&self, name: &str, version: &str, data: &[u8]) -> ServerResult<String> {
        // Validate path components to prevent path traversal
        Self::validate_path_component(name)?;
        Self::validate_path_component(version)?;

        let pkg_dir = self.package_dir(name);
        fs::create_dir_all(&pkg_dir)?;

        let archive_path = self.archive_path(name, version);

        // Calculate checksum before writing
        let checksum = sha256_hex(data);

        // Write the archive
        let mut file = File::create(&archive_path)?;
        file.write_all(data)?;

        Ok(checksum)
    }

    /// Read a package archive
    pub fn read_archive(&self, name: &str, version: &str) -> ServerResult<Vec<u8>> {
        // Validate path components to prevent path traversal
        Self::validate_path_component(name)?;
        Self::validate_path_component(version)?;

        let archive_path = self.archive_path(name, version);

        if !archive_path.exists() {
            return Err(ServerError::VersionNotFound(
                name.to_string(),
                version.to_string(),
            ));
        }

        let mut file = File::open(&archive_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        Ok(data)
    }

    /// Check if a version archive exists
    pub fn archive_exists(&self, name: &str, version: &str) -> bool {
        // Validate path components (if invalid, return false)
        if Self::validate_path_component(name).is_err()
            || Self::validate_path_component(version).is_err()
        {
            return false;
        }
        self.archive_path(name, version).exists()
    }

    /// Delete a version archive
    pub fn delete_archive(&self, name: &str, version: &str) -> ServerResult<bool> {
        // Validate path components to prevent path traversal
        Self::validate_path_component(name)?;
        Self::validate_path_component(version)?;

        let archive_path = self.archive_path(name, version);

        if archive_path.exists() {
            fs::remove_file(&archive_path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get the size of an archive
    pub fn archive_size(&self, name: &str, version: &str) -> ServerResult<u64> {
        // Validate path components to prevent path traversal
        Self::validate_path_component(name)?;
        Self::validate_path_component(version)?;

        let archive_path = self.archive_path(name, version);

        if !archive_path.exists() {
            return Err(ServerError::VersionNotFound(
                name.to_string(),
                version.to_string(),
            ));
        }

        let metadata = fs::metadata(&archive_path)?;
        Ok(metadata.len())
    }

    /// Verify archive checksum
    pub fn verify_checksum(&self, name: &str, version: &str, expected: &str) -> ServerResult<bool> {
        // Path validation is done in read_archive
        let data = self.read_archive(name, version)?;
        let actual = sha256_hex(&data);
        Ok(actual == expected)
    }

    /// List all versions of a package
    pub fn list_versions(&self, name: &str) -> ServerResult<Vec<String>> {
        // Validate path component to prevent path traversal
        Self::validate_path_component(name)?;

        let pkg_dir = self.package_dir(name);

        if !pkg_dir.exists() {
            return Ok(vec![]);
        }

        let mut versions = Vec::new();

        for entry in fs::read_dir(&pkg_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("gz") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // Remove .tar from the stem
                    if let Some(version) = stem.strip_suffix(".tar") {
                        versions.push(version.to_string());
                    }
                }
            }
        }

        Ok(versions)
    }

    /// Get total storage size
    pub fn total_size(&self) -> ServerResult<u64> {
        let mut total = 0u64;

        for entry in walkdir(&self.root)? {
            if entry.is_file() {
                total += entry.metadata()?.len();
            }
        }

        Ok(total)
    }

    /// Extract archive contents to a temporary directory for validation
    pub fn extract_to_temp(&self, data: &[u8]) -> ServerResult<tempfile::TempDir> {
        let temp_dir = tempfile::TempDir::new()?;

        let decoder = GzDecoder::new(data);
        let mut archive = Archive::new(decoder);

        let mut total_size = 0u64;
        let mut file_count = 0usize;

        // Security: check for path traversal and archive bombs
        for entry in archive.entries()? {
            let entry = entry?;
            let path = entry.path()?;

            // Check for absolute paths or path traversal
            if path.is_absolute() {
                return Err(ServerError::Archive(
                    "Archive contains absolute path".to_string(),
                ));
            }

            for component in path.components() {
                if matches!(component, std::path::Component::ParentDir) {
                    return Err(ServerError::Archive(
                        "Archive contains path traversal".to_string(),
                    ));
                }
            }

            // Archive bomb protection: check file count
            file_count += 1;
            if file_count > MAX_FILE_COUNT {
                return Err(ServerError::Archive(format!(
                    "Archive contains too many files (max: {})",
                    MAX_FILE_COUNT
                )));
            }

            // Archive bomb protection: check total uncompressed size
            total_size += entry.size();
            if total_size > MAX_UNCOMPRESSED_SIZE {
                return Err(ServerError::Archive(format!(
                    "Archive too large (max: {} bytes)",
                    MAX_UNCOMPRESSED_SIZE
                )));
            }
        }

        // Re-read and extract
        let decoder = GzDecoder::new(data);
        let mut archive = Archive::new(decoder);
        archive.unpack(temp_dir.path())?;

        Ok(temp_dir)
    }

    /// Validate archive contents (check for vais.toml, etc.)
    pub fn validate_archive(&self, data: &[u8]) -> ServerResult<PackageManifest> {
        let temp_dir = self.extract_to_temp(data)?;

        // Look for vais.toml
        let manifest_path = temp_dir.path().join("vais.toml");

        if !manifest_path.exists() {
            return Err(ServerError::Archive(
                "Archive missing vais.toml manifest".to_string(),
            ));
        }

        let manifest_content = fs::read_to_string(&manifest_path)?;
        let manifest: PackageManifest =
            toml::from_str(&manifest_content).map_err(|e| ServerError::Archive(e.to_string()))?;

        Ok(manifest)
    }
}

/// Simple manifest structure for validation
#[derive(Debug, serde::Deserialize)]
pub struct PackageManifest {
    pub package: PackageInfo,
    #[serde(default)]
    pub dependencies: std::collections::HashMap<String, toml::Value>,
    #[serde(default)]
    pub dev_dependencies: std::collections::HashMap<String, toml::Value>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub documentation: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
}

/// Calculate SHA256 hex digest
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Create a tar.gz archive from a directory
pub fn create_archive(source_dir: &Path) -> ServerResult<Vec<u8>> {
    let mut buffer = Vec::new();

    {
        let encoder = GzEncoder::new(&mut buffer, Compression::default());
        let mut archive = Builder::new(encoder);

        // Add files from source directory
        add_dir_to_archive(&mut archive, source_dir, Path::new(""))?;

        let encoder = archive.into_inner()?;
        encoder.finish()?;
    }

    Ok(buffer)
}

fn add_dir_to_archive<W: Write>(
    archive: &mut Builder<W>,
    source_dir: &Path,
    prefix: &Path,
) -> ServerResult<()> {
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();

        // Skip hidden files and common build directories
        if let Some(name_str) = name.to_str() {
            if name_str.starts_with('.')
                || name_str == "target"
                || name_str == "node_modules"
                || name_str == ".vais-cache"
            {
                continue;
            }
        }

        let archive_path = prefix.join(&name);

        if path.is_dir() {
            add_dir_to_archive(archive, &path, &archive_path)?;
        } else if path.is_file() {
            let mut file = File::open(&path)?;
            archive.append_file(&archive_path, &mut file)?;
        }
    }

    Ok(())
}

/// Walk directory recursively
fn walkdir(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut result = Vec::new();

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                result.extend(walkdir(&path)?);
            } else {
                result.push(path);
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let data = b"hello world";
        let hash = sha256_hex(data);
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_sha256_empty() {
        let hash = sha256_hex(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_deterministic() {
        let data = b"reproducible";
        let h1 = sha256_hex(data);
        let h2 = sha256_hex(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_storage_operations() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();

        // Store archive
        let data = b"test archive data";
        let checksum = storage.store_archive("test-pkg", "1.0.0", data).unwrap();

        // Verify it exists
        assert!(storage.archive_exists("test-pkg", "1.0.0"));

        // Read it back
        let read_data = storage.read_archive("test-pkg", "1.0.0").unwrap();
        assert_eq!(read_data, data);

        // Verify checksum
        assert!(storage
            .verify_checksum("test-pkg", "1.0.0", &checksum)
            .unwrap());

        // List versions
        let versions = storage.list_versions("test-pkg").unwrap();
        assert_eq!(versions, vec!["1.0.0"]);

        // Delete
        assert!(storage.delete_archive("test-pkg", "1.0.0").unwrap());
        assert!(!storage.archive_exists("test-pkg", "1.0.0"));
    }

    #[test]
    fn test_storage_new_creates_directory() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("nested").join("storage");
        let _storage = PackageStorage::new(storage_path.clone()).unwrap();
        assert!(storage_path.exists());
    }

    #[test]
    fn test_archive_not_exists() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        assert!(!storage.archive_exists("nonexistent", "1.0.0"));
    }

    #[test]
    fn test_read_nonexistent_archive() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        let result = storage.read_archive("nonexistent", "1.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_nonexistent_archive() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        let result = storage.delete_archive("nonexistent", "1.0.0").unwrap();
        assert!(!result);
    }

    #[test]
    fn test_archive_size() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        let data = b"some data here";
        storage.store_archive("pkg", "1.0.0", data).unwrap();
        let size = storage.archive_size("pkg", "1.0.0").unwrap();
        assert_eq!(size, data.len() as u64);
    }

    #[test]
    fn test_archive_size_nonexistent() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        let result = storage.archive_size("nonexistent", "1.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_versions() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();

        storage.store_archive("pkg", "1.0.0", b"v1").unwrap();
        storage.store_archive("pkg", "2.0.0", b"v2").unwrap();
        storage.store_archive("pkg", "3.0.0", b"v3").unwrap();

        assert!(storage.archive_exists("pkg", "1.0.0"));
        assert!(storage.archive_exists("pkg", "2.0.0"));
        assert!(storage.archive_exists("pkg", "3.0.0"));

        let mut versions = storage.list_versions("pkg").unwrap();
        versions.sort();
        assert_eq!(versions, vec!["1.0.0", "2.0.0", "3.0.0"]);
    }

    #[test]
    fn test_list_versions_no_package() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        let versions = storage.list_versions("nonexistent").unwrap();
        assert!(versions.is_empty());
    }

    #[test]
    fn test_checksum_mismatch() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        storage.store_archive("pkg", "1.0.0", b"data").unwrap();
        let result = storage
            .verify_checksum("pkg", "1.0.0", "wrong_checksum")
            .unwrap();
        assert!(!result);
    }

    #[test]
    fn test_path_traversal_store() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        let result = storage.store_archive("../escape", "1.0.0", b"data");
        assert!(result.is_err());
    }

    #[test]
    fn test_path_traversal_read() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        let result = storage.read_archive("../escape", "1.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_path_traversal_version() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        let result = storage.store_archive("pkg", "../escape", b"data");
        assert!(result.is_err());
    }

    #[test]
    fn test_path_traversal_slash() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        assert!(!storage.archive_exists("foo/bar", "1.0.0"));
        assert!(!storage.archive_exists("pkg", "1/0"));
    }

    #[test]
    fn test_path_traversal_backslash() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        assert!(!storage.archive_exists("foo\\bar", "1.0.0"));
    }

    #[test]
    fn test_path_traversal_null_byte() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        assert!(!storage.archive_exists("foo\0bar", "1.0.0"));
    }

    #[test]
    fn test_package_dir() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        let dir = storage.package_dir("my-package");
        assert!(dir.ends_with("my-package"));
    }

    #[test]
    fn test_archive_path() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        let path = storage.archive_path("my-package", "1.0.0");
        assert!(path.ends_with("my-package/1.0.0.tar.gz"));
    }

    #[test]
    fn test_total_size_empty() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        let size = storage.total_size().unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_total_size_with_archives() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        storage.store_archive("a", "1.0.0", b"hello").unwrap();
        storage.store_archive("b", "1.0.0", b"world!").unwrap();
        let size = storage.total_size().unwrap();
        assert_eq!(size, 11); // 5 + 6
    }

    #[test]
    fn test_store_and_overwrite() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();
        storage.store_archive("pkg", "1.0.0", b"old").unwrap();
        storage.store_archive("pkg", "1.0.0", b"new data").unwrap();
        let data = storage.read_archive("pkg", "1.0.0").unwrap();
        assert_eq!(data, b"new data");
    }

    // ========== PackageManifest serde tests ==========

    #[test]
    fn test_manifest_minimal() {
        let toml = r#"
            [package]
            name = "my-pkg"
            version = "1.0.0"
        "#;
        let manifest: PackageManifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.package.name, "my-pkg");
        assert_eq!(manifest.package.version, "1.0.0");
        assert!(manifest.package.description.is_none());
        assert!(manifest.package.authors.is_empty());
        assert!(manifest.dependencies.is_empty());
    }

    #[test]
    fn test_manifest_full() {
        let toml = r#"
            [package]
            name = "my-pkg"
            version = "2.0.0"
            description = "A test package"
            authors = ["alice", "bob"]
            license = "MIT"
            homepage = "https://example.com"
            repository = "https://github.com/example/pkg"
            documentation = "https://docs.example.com"
            keywords = ["test", "vais"]
            categories = ["web"]

            [dependencies]
            foo = "^1.0"
        "#;
        let manifest: PackageManifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.package.name, "my-pkg");
        assert_eq!(manifest.package.version, "2.0.0");
        assert_eq!(
            manifest.package.description,
            Some("A test package".to_string())
        );
        assert_eq!(manifest.package.authors, vec!["alice", "bob"]);
        assert_eq!(manifest.package.license, Some("MIT".to_string()));
        assert_eq!(
            manifest.package.homepage,
            Some("https://example.com".to_string())
        );
        assert_eq!(manifest.package.keywords, vec!["test", "vais"]);
        assert_eq!(manifest.package.categories, vec!["web"]);
        assert_eq!(manifest.dependencies.len(), 1);
    }

    #[test]
    fn test_manifest_with_dev_dependencies() {
        let toml = r#"
            [package]
            name = "my-pkg"
            version = "1.0.0"

            [dev_dependencies]
            test-helper = "^0.1"
        "#;
        let manifest: PackageManifest = toml::from_str(toml).unwrap();
        assert!(manifest.dependencies.is_empty());
        assert_eq!(manifest.dev_dependencies.len(), 1);
    }

    #[test]
    fn test_manifest_empty_package_name() {
        let toml = r#"
            [package]
            name = ""
            version = "1.0.0"
        "#;
        let manifest: PackageManifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.package.name, "");
    }

    // ========== validate_path_component tests ==========

    #[test]
    fn test_validate_path_component_valid() {
        assert!(PackageStorage::validate_path_component("my-pkg").is_ok());
        assert!(PackageStorage::validate_path_component("1.0.0").is_ok());
        assert!(PackageStorage::validate_path_component("a").is_ok());
    }

    #[test]
    fn test_validate_path_component_dotdot() {
        assert!(PackageStorage::validate_path_component("..").is_err());
        assert!(PackageStorage::validate_path_component("a..b").is_err());
    }

    #[test]
    fn test_validate_path_component_slash() {
        assert!(PackageStorage::validate_path_component("a/b").is_err());
    }

    #[test]
    fn test_validate_path_component_backslash() {
        assert!(PackageStorage::validate_path_component("a\\b").is_err());
    }

    #[test]
    fn test_validate_path_component_null() {
        assert!(PackageStorage::validate_path_component("a\0b").is_err());
    }

    // ========== sha256_hex additional tests ==========

    #[test]
    fn test_sha256_different_inputs() {
        let h1 = sha256_hex(b"hello");
        let h2 = sha256_hex(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_sha256_length() {
        let hash = sha256_hex(b"test");
        assert_eq!(hash.len(), 64); // SHA-256 = 32 bytes = 64 hex chars
    }

    #[test]
    fn test_sha256_hex_chars() {
        let hash = sha256_hex(b"test");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ========== create_archive tests ==========

    #[test]
    fn test_create_archive_basic() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("hello.txt");
        std::fs::write(&file_path, "hello world").unwrap();

        let archive_data = create_archive(temp_dir.path()).unwrap();
        assert!(!archive_data.is_empty());
    }

    #[test]
    fn test_create_archive_empty_dir() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let archive_data = create_archive(temp_dir.path()).unwrap();
        // Should succeed even for empty directory
        assert!(!archive_data.is_empty());
    }

    #[test]
    fn test_create_archive_skips_hidden() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join(".hidden"), "secret").unwrap();
        std::fs::write(temp_dir.path().join("visible.txt"), "hello").unwrap();

        let archive_data = create_archive(temp_dir.path()).unwrap();
        // Verify archive doesn't contain .hidden
        let decoder = GzDecoder::new(&archive_data[..]);
        let mut archive = Archive::new(decoder);
        let paths: Vec<String> = archive
            .entries()
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(paths.iter().any(|p| p.contains("visible")));
        assert!(!paths.iter().any(|p| p.contains(".hidden")));
    }

    #[test]
    fn test_create_archive_skips_target() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let target_dir = temp_dir.path().join("target");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(target_dir.join("build.o"), "binary").unwrap();
        std::fs::write(temp_dir.path().join("src.vais"), "code").unwrap();

        let archive_data = create_archive(temp_dir.path()).unwrap();
        let decoder = GzDecoder::new(&archive_data[..]);
        let mut archive = Archive::new(decoder);
        let paths: Vec<String> = archive
            .entries()
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(!paths.iter().any(|p| p.contains("target")));
    }

    // ========== extract_to_temp and validate_archive tests ==========

    #[test]
    fn test_validate_archive_with_manifest() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().join("storage")).unwrap();

        // Create a valid package directory
        let pkg_dir = temp_dir.path().join("pkg");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(
            pkg_dir.join("vais.toml"),
            r#"
[package]
name = "my-pkg"
version = "1.0.0"
"#,
        )
        .unwrap();

        let archive_data = create_archive(&pkg_dir).unwrap();
        let manifest = storage.validate_archive(&archive_data).unwrap();
        assert_eq!(manifest.package.name, "my-pkg");
        assert_eq!(manifest.package.version, "1.0.0");
    }

    #[test]
    fn test_validate_archive_missing_manifest() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().join("storage")).unwrap();

        let pkg_dir = temp_dir.path().join("pkg");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(pkg_dir.join("lib.vais"), "F main() -> i64 { 0 }").unwrap();

        let archive_data = create_archive(&pkg_dir).unwrap();
        let result = storage.validate_archive(&archive_data);
        assert!(result.is_err());
    }

    // ========== Multiple packages storage tests ==========

    #[test]
    fn test_multiple_packages() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();

        storage.store_archive("alpha", "1.0.0", b"a1").unwrap();
        storage.store_archive("beta", "1.0.0", b"b1").unwrap();
        storage.store_archive("alpha", "2.0.0", b"a2").unwrap();

        assert!(storage.archive_exists("alpha", "1.0.0"));
        assert!(storage.archive_exists("alpha", "2.0.0"));
        assert!(storage.archive_exists("beta", "1.0.0"));
        assert!(!storage.archive_exists("beta", "2.0.0"));

        let alpha_versions = storage.list_versions("alpha").unwrap();
        assert_eq!(alpha_versions.len(), 2);

        let beta_versions = storage.list_versions("beta").unwrap();
        assert_eq!(beta_versions.len(), 1);
    }

    #[test]
    fn test_checksum_consistency() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = PackageStorage::new(temp_dir.path().to_path_buf()).unwrap();

        let data = b"deterministic content";
        let checksum1 = storage.store_archive("pkg", "1.0.0", data).unwrap();
        let checksum2 = sha256_hex(data);
        assert_eq!(checksum1, checksum2);
    }
}

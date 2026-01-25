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
        self.archive_path(name, version).exists()
    }

    /// Delete a version archive
    pub fn delete_archive(&self, name: &str, version: &str) -> ServerResult<bool> {
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
        let data = self.read_archive(name, version)?;
        let actual = sha256_hex(&data);
        Ok(actual == expected)
    }

    /// List all versions of a package
    pub fn list_versions(&self, name: &str) -> ServerResult<Vec<String>> {
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

        // Security: check for path traversal
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
}

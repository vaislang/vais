//! Archive handling for packages
//!
//! Handles packing and unpacking of .tar.gz package archives.

use super::error::{RegistryError, RegistryResult};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tar::{Archive, Builder};

/// Pack a directory into a tar.gz archive
pub fn pack_package(source_dir: &Path, output_path: &Path) -> RegistryResult<()> {
    let output_file = File::create(output_path).map_err(|e| RegistryError::WriteError {
        path: output_path.to_path_buf(),
        source: e,
    })?;

    let encoder = GzEncoder::new(output_file, Compression::default());
    let mut builder = Builder::new(encoder);

    // Add all files from source directory
    add_directory_to_archive(&mut builder, source_dir, Path::new(""))?;

    let encoder = builder.into_inner().map_err(|e| RegistryError::ArchiveError {
        message: format!("failed to finish tar archive: {}", e),
    })?;

    encoder.finish().map_err(|e| RegistryError::ArchiveError {
        message: format!("failed to finish gzip compression: {}", e),
    })?;

    Ok(())
}

/// Recursively add a directory to the tar archive
fn add_directory_to_archive<W: Write>(
    builder: &mut Builder<W>,
    source_dir: &Path,
    prefix: &Path,
) -> RegistryResult<()> {
    for entry in fs::read_dir(source_dir).map_err(|e| RegistryError::ReadError {
        path: source_dir.to_path_buf(),
        source: e,
    })? {
        let entry = entry.map_err(|e| RegistryError::ReadError {
            path: source_dir.to_path_buf(),
            source: e,
        })?;

        let path = entry.path();
        let name = entry.file_name();
        let archive_path = prefix.join(&name);

        // Skip hidden files and common ignored directories
        if let Some(name_str) = name.to_str() {
            if name_str.starts_with('.') || name_str == "target" || name_str == "node_modules" {
                continue;
            }
        }

        if path.is_dir() {
            add_directory_to_archive(builder, &path, &archive_path)?;
        } else if path.is_file() {
            let mut file = File::open(&path).map_err(|e| RegistryError::ReadError {
                path: path.clone(),
                source: e,
            })?;

            builder
                .append_file(&archive_path, &mut file)
                .map_err(|e| RegistryError::ArchiveError {
                    message: format!("failed to add file {}: {}", path.display(), e),
                })?;
        }
    }

    Ok(())
}

/// Unpack a tar.gz archive to a directory
pub fn unpack_package(archive_path: &Path, output_dir: &Path) -> RegistryResult<()> {
    // Create output directory
    fs::create_dir_all(output_dir).map_err(|e| RegistryError::CreateDirError {
        path: output_dir.to_path_buf(),
        source: e,
    })?;

    // Open and decompress archive
    let file = File::open(archive_path).map_err(|e| RegistryError::ReadError {
        path: archive_path.to_path_buf(),
        source: e,
    })?;

    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    // Extract with security checks
    for entry in archive.entries().map_err(|e| RegistryError::ArchiveError {
        message: format!("failed to read archive entries: {}", e),
    })? {
        let mut entry = entry.map_err(|e| RegistryError::ArchiveError {
            message: format!("failed to read entry: {}", e),
        })?;

        let path = entry.path().map_err(|e| RegistryError::ArchiveError {
            message: format!("invalid entry path: {}", e),
        })?.to_path_buf();

        // Security: check for path traversal
        if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err(RegistryError::InvalidArchive {
                message: format!("path traversal detected: {}", path.display()),
            });
        }

        let target = output_dir.join(&path);

        // Security: ensure target is within output directory
        let canonical_output = output_dir.canonicalize().unwrap_or_else(|_| output_dir.to_path_buf());
        if let Ok(canonical_target) = target.canonicalize() {
            if !canonical_target.starts_with(&canonical_output) {
                return Err(RegistryError::InvalidArchive {
                    message: format!("path escapes output directory: {}", path.display()),
                });
            }
        }

        // Create parent directories
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| RegistryError::CreateDirError {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        // Extract entry
        entry.unpack(&target).map_err(|e| RegistryError::ArchiveError {
            message: format!("failed to extract {}: {}", path.display(), e),
        })?;
    }

    Ok(())
}

/// Unpack from in-memory data
pub fn unpack_from_bytes(data: &[u8], output_dir: &Path) -> RegistryResult<()> {
    // Create output directory
    fs::create_dir_all(output_dir).map_err(|e| RegistryError::CreateDirError {
        path: output_dir.to_path_buf(),
        source: e,
    })?;

    // Decompress
    let decoder = GzDecoder::new(data);
    let mut archive = Archive::new(decoder);

    // Extract with security checks
    for entry in archive.entries().map_err(|e| RegistryError::ArchiveError {
        message: format!("failed to read archive entries: {}", e),
    })? {
        let mut entry = entry.map_err(|e| RegistryError::ArchiveError {
            message: format!("failed to read entry: {}", e),
        })?;

        let path = entry.path().map_err(|e| RegistryError::ArchiveError {
            message: format!("invalid entry path: {}", e),
        })?.to_path_buf();

        // Security: check for path traversal
        if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err(RegistryError::InvalidArchive {
                message: format!("path traversal detected: {}", path.display()),
            });
        }

        let target = output_dir.join(&path);

        // Create parent directories
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| RegistryError::CreateDirError {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        // Extract entry
        entry.unpack(&target).map_err(|e| RegistryError::ArchiveError {
            message: format!("failed to extract {}: {}", path.display(), e),
        })?;
    }

    Ok(())
}

/// Calculate SHA256 checksum of data
pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex_encode(&result)
}

/// Encode bytes as hex string
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Verify archive checksum
pub fn verify_checksum(data: &[u8], expected: &str) -> bool {
    sha256_hex(data) == expected
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_pack_unpack() {
        let source_dir = tempdir().unwrap();
        let archive_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        // Create some files in source
        fs::write(source_dir.path().join("file1.txt"), "content1").unwrap();
        fs::create_dir(source_dir.path().join("subdir")).unwrap();
        fs::write(
            source_dir.path().join("subdir").join("file2.txt"),
            "content2",
        )
        .unwrap();

        // Pack
        let archive_path = archive_dir.path().join("test.tar.gz");
        pack_package(source_dir.path(), &archive_path).unwrap();
        assert!(archive_path.exists());

        // Unpack
        unpack_package(&archive_path, output_dir.path()).unwrap();

        // Verify
        assert!(output_dir.path().join("file1.txt").exists());
        assert!(output_dir.path().join("subdir").join("file2.txt").exists());

        let content1 = fs::read_to_string(output_dir.path().join("file1.txt")).unwrap();
        assert_eq!(content1, "content1");
    }

    #[test]
    fn test_checksum() {
        let data = b"test data for checksum";
        let hash = sha256_hex(data);
        assert!(verify_checksum(data, &hash));
        assert!(!verify_checksum(data, "invalid"));
    }
}

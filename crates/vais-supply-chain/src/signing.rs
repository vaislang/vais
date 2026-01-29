use crate::{Result, SupplyChainError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageSignature {
    /// SHA-256 hash of the package
    pub hash: String,

    /// Algorithm used for hashing
    pub algorithm: String,

    /// Timestamp when signature was created
    pub timestamp: DateTime<Utc>,

    /// Signer information
    pub signer: SignerInfo,

    /// Additional metadata
    pub metadata: SignatureMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignerInfo {
    /// Signer name or identifier
    pub name: String,

    /// Signer email (optional)
    pub email: Option<String>,

    /// Organization (optional)
    pub organization: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignatureMetadata {
    /// Package name
    pub package_name: String,

    /// Package version
    pub package_version: String,

    /// File size in bytes
    pub file_size: u64,

    /// Tool version used for signing
    pub tool_version: String,
}

pub struct PackageSigner {
    pub signer_info: SignerInfo,
    pub tool_version: String,
}

impl PackageSigner {
    pub fn new(name: String, email: Option<String>, organization: Option<String>) -> Self {
        Self {
            signer_info: SignerInfo {
                name,
                email,
                organization,
            },
            tool_version: "0.0.1".to_string(),
        }
    }

    /// Sign a package file and return the signature
    pub fn sign_package<P: AsRef<Path>>(
        &self,
        path: P,
        package_name: String,
        package_version: String,
    ) -> Result<PackageSignature> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(SupplyChainError::PackageNotFound(
                path.display().to_string(),
            ));
        }

        // Read file contents
        let content = fs::read(path)?;
        let file_size = content.len() as u64;

        // Compute SHA-256 hash
        let hash = self.compute_sha256(&content);

        Ok(PackageSignature {
            hash,
            algorithm: "SHA-256".to_string(),
            timestamp: Utc::now(),
            signer: self.signer_info.clone(),
            metadata: SignatureMetadata {
                package_name,
                package_version,
                file_size,
                tool_version: self.tool_version.clone(),
            },
        })
    }

    /// Sign directory (compute hash of all files)
    pub fn sign_directory<P: AsRef<Path>>(
        &self,
        path: P,
        package_name: String,
        package_version: String,
    ) -> Result<PackageSignature> {
        let path = path.as_ref();

        if !path.exists() || !path.is_dir() {
            return Err(SupplyChainError::PackageNotFound(
                path.display().to_string(),
            ));
        }

        // Collect all files in directory
        let mut files = Vec::new();
        self.collect_files(path, &mut files)?;

        // Sort files for deterministic hashing
        files.sort();

        // Compute combined hash
        let mut hasher = Sha256::new();
        let mut total_size = 0u64;

        for file_path in &files {
            let content = fs::read(file_path)?;
            total_size += content.len() as u64;
            hasher.update(&content);
        }

        let hash = format!("{:x}", hasher.finalize());

        Ok(PackageSignature {
            hash,
            algorithm: "SHA-256".to_string(),
            timestamp: Utc::now(),
            signer: self.signer_info.clone(),
            metadata: SignatureMetadata {
                package_name,
                package_version,
                file_size: total_size,
                tool_version: self.tool_version.clone(),
            },
        })
    }

    /// Verify a package signature
    pub fn verify_signature<P: AsRef<Path>>(
        &self,
        path: P,
        signature: &PackageSignature,
    ) -> Result<bool> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(SupplyChainError::PackageNotFound(
                path.display().to_string(),
            ));
        }

        let content = if path.is_file() {
            fs::read(path)?
        } else {
            // Directory - collect all files
            let mut files = Vec::new();
            self.collect_files(path, &mut files)?;
            files.sort();

            let mut combined = Vec::new();
            for file_path in &files {
                let file_content = fs::read(file_path)?;
                combined.extend_from_slice(&file_content);
            }
            combined
        };

        let computed_hash = self.compute_sha256(&content);
        Ok(computed_hash == signature.hash)
    }

    /// Verify signature and throw error if invalid
    pub fn verify_signature_strict<P: AsRef<Path>>(
        &self,
        path: P,
        signature: &PackageSignature,
    ) -> Result<()> {
        if self.verify_signature(path, signature)? {
            Ok(())
        } else {
            Err(SupplyChainError::SignatureVerificationFailed(
                "Hash mismatch".to_string(),
            ))
        }
    }

    /// Compute SHA-256 hash
    fn compute_sha256(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Recursively collect all files in a directory
    fn collect_files(&self, dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                self.collect_files(&path, files)?;
            }
        }
        Ok(())
    }
}

impl PackageSignature {
    /// Serialize signature to JSON
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Write signature to file
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = self.to_json()?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load signature from JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// Check if signature is recent (within specified duration)
    pub fn is_recent(&self, max_age: chrono::Duration) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.timestamp);
        age <= max_age
    }
}

impl Default for PackageSigner {
    fn default() -> Self {
        Self::new(
            "vais-builder".to_string(),
            Some("build@vais-lang.org".to_string()),
            Some("Vais Team".to_string()),
        )
    }
}

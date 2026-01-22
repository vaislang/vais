//! Lock file management
//!
//! Handles vais.lock file for reproducible builds.

use super::error::{RegistryError, RegistryResult};
use super::version::Version;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Lock file for reproducible dependency resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    /// Lock file format version
    pub version: u32,
    /// Locked packages
    #[serde(default)]
    pub packages: BTreeMap<String, LockedPackage>,
}

/// A locked package entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    /// Exact version
    #[serde(with = "version_serde")]
    pub version: Version,
    /// SHA256 checksum
    pub checksum: String,
    /// Source (registry name or path)
    pub source: String,
    /// Locked dependencies
    #[serde(default)]
    pub dependencies: Vec<String>,
}

impl LockFile {
    /// Create a new empty lock file
    pub fn new() -> Self {
        Self {
            version: 1,
            packages: BTreeMap::new(),
        }
    }

    /// Load lock file from path
    pub fn load(path: &Path) -> RegistryResult<Self> {
        let content = fs::read_to_string(path).map_err(|e| RegistryError::ReadError {
            path: path.to_path_buf(),
            source: e,
        })?;

        Self::parse(&content)
    }

    /// Parse lock file from TOML string
    pub fn parse(content: &str) -> RegistryResult<Self> {
        toml::from_str(content).map_err(|e| RegistryError::LockFileError {
            message: e.to_string(),
        })
    }

    /// Save lock file to path
    pub fn save(&self, path: &Path) -> RegistryResult<()> {
        let content = self.to_string()?;
        fs::write(path, content).map_err(|e| RegistryError::WriteError {
            path: path.to_path_buf(),
            source: e,
        })
    }

    /// Serialize to TOML string
    pub fn to_string(&self) -> RegistryResult<String> {
        toml::to_string_pretty(self).map_err(|e| RegistryError::LockFileError {
            message: e.to_string(),
        })
    }

    /// Add or update a locked package
    pub fn insert(&mut self, name: String, package: LockedPackage) {
        self.packages.insert(name, package);
    }

    /// Get a locked package
    pub fn get(&self, name: &str) -> Option<&LockedPackage> {
        self.packages.get(name)
    }

    /// Check if a package is locked
    pub fn contains(&self, name: &str) -> bool {
        self.packages.contains_key(name)
    }

    /// Remove a package
    pub fn remove(&mut self, name: &str) -> Option<LockedPackage> {
        self.packages.remove(name)
    }

    /// Get all locked packages
    pub fn iter(&self) -> impl Iterator<Item = (&String, &LockedPackage)> {
        self.packages.iter()
    }

    /// Number of locked packages
    pub fn len(&self) -> usize {
        self.packages.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// Merge with another lock file (other takes precedence)
    pub fn merge(&mut self, other: &LockFile) {
        for (name, pkg) in &other.packages {
            self.packages.insert(name.clone(), pkg.clone());
        }
    }

    /// Check if lock file is up to date with dependencies
    pub fn is_current(&self, deps: &[(String, String)]) -> bool {
        for (name, _req) in deps {
            if !self.packages.contains_key(name) {
                return false;
            }
        }
        true
    }
}

impl Default for LockFile {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom serde for Version
mod version_serde {
    use super::Version;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(version: &Version, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&version.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Version::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockfile_roundtrip() {
        let mut lock = LockFile::new();
        lock.insert(
            "test-pkg".to_string(),
            LockedPackage {
                version: Version::new(1, 2, 3),
                checksum: "abc123".to_string(),
                source: "registry".to_string(),
                dependencies: vec!["other-pkg".to_string()],
            },
        );

        let toml = lock.to_string().unwrap();
        let parsed = LockFile::parse(&toml).unwrap();

        assert_eq!(parsed.packages.len(), 1);
        let pkg = parsed.get("test-pkg").unwrap();
        assert_eq!(pkg.version.to_string(), "1.2.3");
        assert_eq!(pkg.checksum, "abc123");
    }

    #[test]
    fn test_lockfile_format() {
        let toml = r#"
version = 1

[packages.json-parser]
version = "1.0.0"
checksum = "abc123def456"
source = "registry"
dependencies = []

[packages.utils]
version = "0.5.0"
checksum = "xyz789"
source = "registry"
dependencies = []
"#;

        let lock = LockFile::parse(toml).unwrap();
        assert_eq!(lock.packages.len(), 2);
        assert!(lock.contains("json-parser"));
        assert!(lock.contains("utils"));
    }
}

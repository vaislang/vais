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

    #[test]
    fn test_lockfile_new() {
        let lock = LockFile::new();
        assert_eq!(lock.version, 1);
        assert!(lock.packages.is_empty());
        assert!(lock.is_empty());
        assert_eq!(lock.len(), 0);
    }

    #[test]
    fn test_lockfile_default() {
        let lock = LockFile::default();
        assert_eq!(lock.version, 1);
        assert!(lock.is_empty());
    }

    #[test]
    fn test_lockfile_insert_and_get() {
        let mut lock = LockFile::new();
        lock.insert(
            "my-pkg".to_string(),
            LockedPackage {
                version: Version::new(1, 0, 0),
                checksum: "abc123".to_string(),
                source: "registry".to_string(),
                dependencies: vec![],
            },
        );

        assert!(!lock.is_empty());
        assert_eq!(lock.len(), 1);
        assert!(lock.contains("my-pkg"));
        assert!(!lock.contains("other-pkg"));

        let pkg = lock.get("my-pkg").unwrap();
        assert_eq!(pkg.version.to_string(), "1.0.0");
        assert_eq!(pkg.checksum, "abc123");
    }

    #[test]
    fn test_lockfile_get_nonexistent() {
        let lock = LockFile::new();
        assert!(lock.get("nonexistent").is_none());
    }

    #[test]
    fn test_lockfile_remove() {
        let mut lock = LockFile::new();
        lock.insert(
            "my-pkg".to_string(),
            LockedPackage {
                version: Version::new(1, 0, 0),
                checksum: "abc".to_string(),
                source: "registry".to_string(),
                dependencies: vec![],
            },
        );

        let removed = lock.remove("my-pkg");
        assert!(removed.is_some());
        assert!(lock.is_empty());

        // Removing again returns None
        assert!(lock.remove("my-pkg").is_none());
    }

    #[test]
    fn test_lockfile_iter() {
        let mut lock = LockFile::new();
        lock.insert(
            "a".to_string(),
            LockedPackage {
                version: Version::new(1, 0, 0),
                checksum: "a".to_string(),
                source: "registry".to_string(),
                dependencies: vec![],
            },
        );
        lock.insert(
            "b".to_string(),
            LockedPackage {
                version: Version::new(2, 0, 0),
                checksum: "b".to_string(),
                source: "registry".to_string(),
                dependencies: vec![],
            },
        );

        let names: Vec<&String> = lock.iter().map(|(n, _)| n).collect();
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_lockfile_merge() {
        let mut lock1 = LockFile::new();
        lock1.insert(
            "a".to_string(),
            LockedPackage {
                version: Version::new(1, 0, 0),
                checksum: "old".to_string(),
                source: "registry".to_string(),
                dependencies: vec![],
            },
        );

        let mut lock2 = LockFile::new();
        lock2.insert(
            "a".to_string(),
            LockedPackage {
                version: Version::new(2, 0, 0),
                checksum: "new".to_string(),
                source: "registry".to_string(),
                dependencies: vec![],
            },
        );
        lock2.insert(
            "b".to_string(),
            LockedPackage {
                version: Version::new(1, 0, 0),
                checksum: "b".to_string(),
                source: "registry".to_string(),
                dependencies: vec![],
            },
        );

        lock1.merge(&lock2);

        assert_eq!(lock1.len(), 2);
        // "a" should be overwritten by lock2's version
        assert_eq!(lock1.get("a").unwrap().version.to_string(), "2.0.0");
        assert!(lock1.contains("b"));
    }

    #[test]
    fn test_lockfile_is_current_all_present() {
        let mut lock = LockFile::new();
        lock.insert(
            "a".to_string(),
            LockedPackage {
                version: Version::new(1, 0, 0),
                checksum: "".to_string(),
                source: "registry".to_string(),
                dependencies: vec![],
            },
        );
        lock.insert(
            "b".to_string(),
            LockedPackage {
                version: Version::new(2, 0, 0),
                checksum: "".to_string(),
                source: "registry".to_string(),
                dependencies: vec![],
            },
        );

        let deps = vec![
            ("a".to_string(), "^1.0".to_string()),
            ("b".to_string(), "^2.0".to_string()),
        ];
        assert!(lock.is_current(&deps));
    }

    #[test]
    fn test_lockfile_is_current_missing_dep() {
        let mut lock = LockFile::new();
        lock.insert(
            "a".to_string(),
            LockedPackage {
                version: Version::new(1, 0, 0),
                checksum: "".to_string(),
                source: "registry".to_string(),
                dependencies: vec![],
            },
        );

        let deps = vec![
            ("a".to_string(), "^1.0".to_string()),
            ("b".to_string(), "^2.0".to_string()),
        ];
        assert!(!lock.is_current(&deps));
    }

    #[test]
    fn test_lockfile_is_current_empty() {
        let lock = LockFile::new();
        let deps: Vec<(String, String)> = vec![];
        assert!(lock.is_current(&deps));
    }

    #[test]
    fn test_lockfile_insert_overwrites() {
        let mut lock = LockFile::new();
        lock.insert(
            "a".to_string(),
            LockedPackage {
                version: Version::new(1, 0, 0),
                checksum: "old".to_string(),
                source: "registry".to_string(),
                dependencies: vec![],
            },
        );
        lock.insert(
            "a".to_string(),
            LockedPackage {
                version: Version::new(2, 0, 0),
                checksum: "new".to_string(),
                source: "registry".to_string(),
                dependencies: vec![],
            },
        );

        assert_eq!(lock.len(), 1);
        assert_eq!(lock.get("a").unwrap().checksum, "new");
    }

    #[test]
    fn test_lockfile_with_dependencies() {
        let mut lock = LockFile::new();
        lock.insert(
            "app".to_string(),
            LockedPackage {
                version: Version::new(1, 0, 0),
                checksum: "abc".to_string(),
                source: "registry".to_string(),
                dependencies: vec!["lib-a".to_string(), "lib-b".to_string()],
            },
        );

        let pkg = lock.get("app").unwrap();
        assert_eq!(pkg.dependencies.len(), 2);
        assert!(pkg.dependencies.contains(&"lib-a".to_string()));
    }

    #[test]
    fn test_lockfile_parse_invalid_toml() {
        let result = LockFile::parse("this is not toml {{{}}}");
        assert!(result.is_err());
    }

    #[test]
    fn test_lockfile_btreemap_ordering() {
        let mut lock = LockFile::new();
        let pkg = LockedPackage {
            version: Version::new(1, 0, 0),
            checksum: "".to_string(),
            source: "registry".to_string(),
            dependencies: vec![],
        };
        lock.insert("z-pkg".to_string(), pkg.clone());
        lock.insert("a-pkg".to_string(), pkg.clone());
        lock.insert("m-pkg".to_string(), pkg);

        let names: Vec<&String> = lock.iter().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["a-pkg", "m-pkg", "z-pkg"]);
    }
}

//! Package index and metadata
//!
//! Represents the registry index structure for package discovery.

use super::error::{RegistryError, RegistryResult};
use super::version::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Package metadata from the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    /// Package name
    pub name: String,
    /// Package description
    #[serde(default)]
    pub description: Option<String>,
    /// Available versions
    pub versions: Vec<VersionEntry>,
    /// Package authors
    #[serde(default)]
    pub authors: Vec<String>,
    /// Homepage URL
    #[serde(default)]
    pub homepage: Option<String>,
    /// Repository URL
    #[serde(default)]
    pub repository: Option<String>,
    /// License
    #[serde(default)]
    pub license: Option<String>,
    /// Keywords for search
    #[serde(default)]
    pub keywords: Vec<String>,
}

impl PackageMetadata {
    /// Get the latest non-prerelease version
    pub fn latest_version(&self) -> Option<&VersionEntry> {
        self.versions
            .iter()
            .filter(|v| !v.version.is_prerelease() && !v.yanked)
            .max_by(|a, b| a.version.cmp(&b.version))
    }

    /// Get a specific version
    pub fn get_version(&self, version: &Version) -> Option<&VersionEntry> {
        self.versions.iter().find(|v| &v.version == version)
    }

    /// Get all non-yanked versions
    pub fn available_versions(&self) -> Vec<&VersionEntry> {
        self.versions.iter().filter(|v| !v.yanked).collect()
    }
}

/// Version entry in the package index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    /// Parsed version
    #[serde(with = "version_serde")]
    pub version: Version,
    /// SHA256 checksum of the archive
    pub checksum: String,
    /// Dependencies for this version
    #[serde(default)]
    pub dependencies: HashMap<String, VersionDependency>,
    /// Whether this version is yanked
    #[serde(default)]
    pub yanked: bool,
    /// Download URL (relative or absolute)
    #[serde(default)]
    pub download_url: Option<String>,
    /// Size in bytes
    #[serde(default)]
    pub size: Option<u64>,
}

/// Dependency specification in version entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDependency {
    /// Version requirement string
    pub req: String,
    /// Optional features to enable
    #[serde(default)]
    pub features: Vec<String>,
    /// Whether this is optional
    #[serde(default)]
    pub optional: bool,
    /// Target-specific dependency
    #[serde(default)]
    pub target: Option<String>,
}

/// Package index - in-memory representation of registry index
#[derive(Debug, Default)]
pub struct PackageIndex {
    packages: HashMap<String, PackageMetadata>,
}

impl PackageIndex {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    /// Add or update a package
    pub fn insert(&mut self, metadata: PackageMetadata) {
        self.packages.insert(metadata.name.clone(), metadata);
    }

    /// Get package metadata
    pub fn get(&self, name: &str) -> Option<&PackageMetadata> {
        self.packages.get(name)
    }

    /// Check if package exists
    pub fn contains(&self, name: &str) -> bool {
        self.packages.contains_key(name)
    }

    /// Search packages by name or keyword
    pub fn search(&self, query: &str) -> Vec<&PackageMetadata> {
        let query = query.to_lowercase();
        self.packages
            .values()
            .filter(|pkg| {
                pkg.name.to_lowercase().contains(&query)
                    || pkg
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query))
                        .unwrap_or(false)
                    || pkg.keywords.iter().any(|k| k.to_lowercase().contains(&query))
            })
            .collect()
    }

    /// Get all packages
    pub fn all(&self) -> impl Iterator<Item = &PackageMetadata> {
        self.packages.values()
    }

    /// Load from JSON
    pub fn from_json(json: &str) -> RegistryResult<Self> {
        let packages: Vec<PackageMetadata> = serde_json::from_str(json)?;
        let mut index = Self::new();
        for pkg in packages {
            index.insert(pkg);
        }
        Ok(index)
    }

    /// Load a single package from JSON
    pub fn load_package(json: &str) -> RegistryResult<PackageMetadata> {
        serde_json::from_str(json).map_err(RegistryError::from)
    }
}

/// Custom serde for Version to serialize as string
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
    fn test_package_metadata() {
        let json = r#"{
            "name": "json-parser",
            "description": "A JSON parser for Vais",
            "versions": [
                {
                    "version": "1.0.0",
                    "checksum": "abc123",
                    "dependencies": {}
                },
                {
                    "version": "1.1.0",
                    "checksum": "def456",
                    "dependencies": {
                        "utils": { "req": "^0.5.0" }
                    }
                }
            ],
            "keywords": ["json", "parser"]
        }"#;

        let pkg: PackageMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(pkg.name, "json-parser");
        assert_eq!(pkg.versions.len(), 2);
        assert_eq!(pkg.latest_version().unwrap().version.to_string(), "1.1.0");
    }

    #[test]
    fn test_package_search() {
        let mut index = PackageIndex::new();
        index.insert(PackageMetadata {
            name: "json-parser".to_string(),
            description: Some("JSON parsing library".to_string()),
            versions: vec![],
            authors: vec![],
            homepage: None,
            repository: None,
            license: None,
            keywords: vec!["json".to_string()],
        });
        index.insert(PackageMetadata {
            name: "xml-parser".to_string(),
            description: Some("XML parsing library".to_string()),
            versions: vec![],
            authors: vec![],
            homepage: None,
            repository: None,
            license: None,
            keywords: vec!["xml".to_string()],
        });

        let results = index.search("json");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "json-parser");

        let results = index.search("parser");
        assert_eq!(results.len(), 2);
    }
}

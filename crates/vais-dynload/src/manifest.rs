//! Plugin manifest parsing and validation
//!
//! Handles `plugin.toml` files that describe plugin metadata,
//! capabilities, dependencies, and configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::{DynloadError, Result};

/// Plugin manifest loaded from `plugin.toml`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin metadata section
    pub plugin: PluginMetadata,

    /// Required capabilities
    #[serde(default)]
    pub capabilities: Vec<PluginCapability>,

    /// Plugin dependencies
    #[serde(default)]
    pub dependencies: Vec<PluginDependency>,

    /// Exported functions
    #[serde(default)]
    pub exports: Vec<PluginExport>,

    /// Configuration schema
    #[serde(default)]
    pub config: HashMap<String, ConfigField>,

    /// Platform-specific settings
    #[serde(default)]
    pub platform: PlatformConfig,
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name (unique identifier)
    pub name: String,

    /// Plugin version (semver)
    pub version: String,

    /// Human-readable description
    #[serde(default)]
    pub description: String,

    /// Plugin authors
    #[serde(default)]
    pub authors: Vec<String>,

    /// Plugin license
    #[serde(default)]
    pub license: Option<String>,

    /// Plugin homepage URL
    #[serde(default)]
    pub homepage: Option<String>,

    /// Plugin repository URL
    #[serde(default)]
    pub repository: Option<String>,

    /// Plugin type (wasm, native, vais)
    #[serde(default = "default_plugin_format")]
    pub format: PluginFormat,

    /// Entry point file
    #[serde(default)]
    pub entry: Option<String>,

    /// Minimum Vais version required
    #[serde(default)]
    pub min_vais_version: Option<String>,

    /// Maximum Vais version supported
    #[serde(default)]
    pub max_vais_version: Option<String>,
}

fn default_plugin_format() -> PluginFormat {
    PluginFormat::Wasm
}

/// Plugin format type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginFormat {
    /// WebAssembly plugin
    Wasm,
    /// Native dynamic library
    Native,
    /// Vais source module
    Vais,
}

/// Capability that a plugin can request or provide
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapability {
    /// File system read access
    FsRead,
    /// File system write access
    FsWrite,
    /// Network access
    Network,
    /// Environment variable access
    Env,
    /// Process spawning
    Process,
    /// Timer/clock access
    Time,
    /// Random number generation
    Random,
    /// Console/logging output
    Console,
    /// Memory allocation beyond default
    ExtendedMemory,
    /// Multi-threading support
    Threading,
    /// GPU/compute access
    Gpu,
    /// Custom capability with name
    Custom(String),
}

impl PluginCapability {
    /// Check if this capability is considered dangerous
    pub fn is_dangerous(&self) -> bool {
        matches!(
            self,
            PluginCapability::FsWrite
                | PluginCapability::Network
                | PluginCapability::Process
                | PluginCapability::Env
        )
    }

    /// Get human-readable description
    pub fn description(&self) -> &str {
        match self {
            PluginCapability::FsRead => "Read files from the filesystem",
            PluginCapability::FsWrite => "Write files to the filesystem",
            PluginCapability::Network => "Make network connections",
            PluginCapability::Env => "Read environment variables",
            PluginCapability::Process => "Spawn child processes",
            PluginCapability::Time => "Access system time",
            PluginCapability::Random => "Generate random numbers",
            PluginCapability::Console => "Write to console/logs",
            PluginCapability::ExtendedMemory => "Use more than default memory",
            PluginCapability::Threading => "Create threads",
            PluginCapability::Gpu => "Access GPU for compute",
            PluginCapability::Custom(_) => "Custom capability",
        }
    }
}

/// Plugin dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Dependency name
    pub name: String,

    /// Version requirement (semver)
    pub version: String,

    /// Whether dependency is optional
    #[serde(default)]
    pub optional: bool,

    /// Features to enable
    #[serde(default)]
    pub features: Vec<String>,
}

/// Exported function from plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExport {
    /// Function name
    pub name: String,

    /// Function description
    #[serde(default)]
    pub description: String,

    /// Parameter types
    #[serde(default)]
    pub params: Vec<ParamSpec>,

    /// Return type
    #[serde(default)]
    pub returns: Option<String>,
}

/// Parameter specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamSpec {
    /// Parameter name
    pub name: String,
    /// Parameter type
    #[serde(rename = "type")]
    pub param_type: String,
    /// Whether optional
    #[serde(default)]
    pub optional: bool,
}

/// Configuration field schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    /// Field type
    #[serde(rename = "type")]
    pub field_type: String,
    /// Default value
    #[serde(default)]
    pub default: Option<toml::Value>,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Required flag
    #[serde(default)]
    pub required: bool,
}

/// Platform-specific configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// Windows-specific settings
    #[serde(default)]
    pub windows: Option<PlatformSettings>,
    /// macOS-specific settings
    #[serde(default)]
    pub macos: Option<PlatformSettings>,
    /// Linux-specific settings
    #[serde(default)]
    pub linux: Option<PlatformSettings>,
}

/// Platform-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformSettings {
    /// Entry point override
    #[serde(default)]
    pub entry: Option<String>,
    /// Additional library paths
    #[serde(default)]
    pub library_paths: Vec<String>,
    /// Extra environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl PluginManifest {
    /// Load manifest from a file path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
            DynloadError::ManifestError(format!(
                "Failed to read manifest '{}': {}",
                path.display(),
                e
            ))
        })?;

        Self::parse(&content)
    }

    /// Parse manifest from TOML string
    pub fn parse(content: &str) -> Result<Self> {
        let manifest: PluginManifest =
            toml::from_str(content).map_err(|e| DynloadError::ManifestError(e.to_string()))?;

        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate the manifest
    pub fn validate(&self) -> Result<()> {
        // Validate plugin name
        if self.plugin.name.is_empty() {
            return Err(DynloadError::ManifestError(
                "Plugin name cannot be empty".to_string(),
            ));
        }

        // Validate version is valid semver
        semver::Version::parse(&self.plugin.version).map_err(|e| {
            DynloadError::ManifestError(format!("Invalid version '{}': {}", self.plugin.version, e))
        })?;

        // Validate min_vais_version if present
        if let Some(ref min_version) = self.plugin.min_vais_version {
            semver::VersionReq::parse(min_version).map_err(|e| {
                DynloadError::ManifestError(format!(
                    "Invalid min_vais_version '{}': {}",
                    min_version, e
                ))
            })?;
        }

        // Validate dependencies
        for dep in &self.dependencies {
            semver::VersionReq::parse(&dep.version).map_err(|e| {
                DynloadError::ManifestError(format!(
                    "Invalid dependency version '{}' for '{}': {}",
                    dep.version, dep.name, e
                ))
            })?;
        }

        Ok(())
    }

    /// Check if plugin is compatible with the given Vais version
    pub fn is_compatible_with(&self, vais_version: &str) -> Result<bool> {
        let version = semver::Version::parse(vais_version)?;

        // Check minimum version
        if let Some(ref min_version) = self.plugin.min_vais_version {
            let req = semver::VersionReq::parse(min_version)?;
            if !req.matches(&version) {
                return Ok(false);
            }
        }

        // Check maximum version
        if let Some(ref max_version) = self.plugin.max_vais_version {
            let req = semver::VersionReq::parse(max_version)?;
            if !req.matches(&version) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get all required (non-optional) dependencies
    pub fn required_dependencies(&self) -> Vec<&PluginDependency> {
        self.dependencies.iter().filter(|d| !d.optional).collect()
    }

    /// Check if plugin requires any dangerous capabilities
    pub fn has_dangerous_capabilities(&self) -> bool {
        self.capabilities.iter().any(|c| c.is_dangerous())
    }

    /// Get entry point for current platform
    pub fn entry_point(&self) -> Option<&str> {
        // Check platform-specific entry first
        #[cfg(target_os = "windows")]
        if let Some(ref settings) = self.platform.windows {
            if let Some(ref entry) = settings.entry {
                return Some(entry);
            }
        }

        #[cfg(target_os = "macos")]
        if let Some(ref settings) = self.platform.macos {
            if let Some(ref entry) = settings.entry {
                return Some(entry);
            }
        }

        #[cfg(target_os = "linux")]
        if let Some(ref settings) = self.platform.linux {
            if let Some(ref entry) = settings.entry {
                return Some(entry);
            }
        }

        // Fall back to default entry
        self.plugin.entry.as_deref()
    }
}

impl Default for PluginManifest {
    fn default() -> Self {
        Self {
            plugin: PluginMetadata {
                name: "unnamed".to_string(),
                version: "0.0.0".to_string(),
                description: String::new(),
                authors: vec![],
                license: None,
                homepage: None,
                repository: None,
                format: PluginFormat::Wasm,
                entry: None,
                min_vais_version: None,
                max_vais_version: None,
            },
            capabilities: vec![],
            dependencies: vec![],
            exports: vec![],
            config: HashMap::new(),
            platform: PlatformConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_manifest() {
        let toml = r#"
[plugin]
name = "my-plugin"
version = "1.0.0"
"#;

        let manifest = PluginManifest::parse(toml).unwrap();
        assert_eq!(manifest.plugin.name, "my-plugin");
        assert_eq!(manifest.plugin.version, "1.0.0");
        assert_eq!(manifest.plugin.format, PluginFormat::Wasm);
    }

    #[test]
    fn test_parse_full_manifest() {
        let toml = r#"
[plugin]
name = "example-plugin"
version = "1.2.3"
description = "An example plugin"
authors = ["Alice", "Bob"]
license = "MIT"
format = "wasm"
entry = "plugin.wasm"
min_vais_version = ">=0.1.0"

[[dependencies]]
name = "helper-lib"
version = ">=0.5.0"
optional = false

[[exports]]
name = "process"
description = "Process data"
returns = "i64"

[config]
max_items = { type = "integer", default = 100, description = "Maximum items to process" }
"#;

        let manifest = PluginManifest::parse(toml).unwrap();
        assert_eq!(manifest.plugin.name, "example-plugin");
        assert_eq!(manifest.plugin.license, Some("MIT".to_string()));
        assert_eq!(manifest.dependencies.len(), 1);
        assert_eq!(manifest.exports.len(), 1);
    }

    #[test]
    fn test_validate_invalid_version() {
        let toml = r#"
[plugin]
name = "test"
version = "invalid"
"#;

        let result = PluginManifest::parse(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_name() {
        let toml = r#"
[plugin]
name = ""
version = "1.0.0"
"#;

        let result = PluginManifest::parse(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_capability_is_dangerous() {
        assert!(PluginCapability::FsWrite.is_dangerous());
        assert!(PluginCapability::Network.is_dangerous());
        assert!(PluginCapability::Process.is_dangerous());
        assert!(!PluginCapability::Console.is_dangerous());
        assert!(!PluginCapability::Time.is_dangerous());
    }

    #[test]
    fn test_version_compatibility() {
        let toml = r#"
[plugin]
name = "test"
version = "1.0.0"
min_vais_version = ">=0.1.0"
"#;

        let manifest = PluginManifest::parse(toml).unwrap();
        assert!(manifest.is_compatible_with("0.1.0").unwrap());
        assert!(manifest.is_compatible_with("1.0.0").unwrap());
    }

    #[test]
    fn test_required_dependencies() {
        let toml = r#"
[plugin]
name = "test"
version = "1.0.0"

[[dependencies]]
name = "required-dep"
version = ">=1.0.0"
optional = false

[[dependencies]]
name = "optional-dep"
version = ">=1.0.0"
optional = true
"#;

        let manifest = PluginManifest::parse(toml).unwrap();
        let required = manifest.required_dependencies();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0].name, "required-dep");
    }
}

//! Type definitions for package management

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Package manifest (vais.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    #[serde(default)]
    pub package: PackageInfo,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, Dependency>,
    #[serde(default, rename = "native-dependencies")]
    pub native_dependencies: HashMap<String, NativeDependency>,
    #[serde(default)]
    pub build: BuildConfig,
    /// Feature flags configuration
    #[serde(default)]
    pub features: Option<FeatureConfig>,
    /// Workspace configuration (only in workspace root)
    #[serde(default)]
    pub workspace: Option<WorkspaceConfig>,
}

/// Package information section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
}

/// Dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    /// Simple version string (for future registry support)
    Version(String),
    /// Detailed dependency spec
    Detailed(DetailedDependency),
}

/// Detailed dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedDependency {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
    /// Registry name (None = default registry)
    #[serde(default)]
    pub registry: Option<String>,
    /// Use workspace dependency definition
    #[serde(default)]
    pub workspace: Option<bool>,
}

/// Native (C/system) dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NativeDependency {
    /// Simple: just a library name to pass as -l flag
    /// e.g. openssl = "ssl"
    Simple(String),
    /// Detailed specification
    Detailed(NativeDependencyDetail),
}

/// Detailed native dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeDependencyDetail {
    /// Library name(s) for -l flags (e.g. ["ssl", "crypto"])
    #[serde(default)]
    pub libs: Vec<String>,
    /// Header search path for -I flag
    #[serde(default)]
    pub include: Option<String>,
    /// Library search path for -L flag
    #[serde(default)]
    pub lib_path: Option<String>,
    /// Whether this is a system library (found via pkg-config)
    #[serde(default)]
    pub system: Option<bool>,
    /// C source files to compile and link
    #[serde(default)]
    pub sources: Vec<String>,
}

impl NativeDependency {
    /// Get the -l library flags for this native dependency
    pub fn lib_flags(&self) -> Vec<String> {
        match self {
            NativeDependency::Simple(lib) => vec![format!("-l{}", lib)],
            NativeDependency::Detailed(d) => d.libs.iter().map(|l| format!("-l{}", l)).collect(),
        }
    }

    /// Get the -I include path flag, if any
    pub fn include_flag(&self) -> Option<String> {
        match self {
            NativeDependency::Simple(_) => None,
            NativeDependency::Detailed(d) => d.include.as_ref().map(|p| format!("-I{}", p)),
        }
    }

    /// Get the -L library search path flag, if any
    pub fn lib_path_flag(&self) -> Option<String> {
        match self {
            NativeDependency::Simple(_) => None,
            NativeDependency::Detailed(d) => d.lib_path.as_ref().map(|p| format!("-L{}", p)),
        }
    }

    /// Get source files to compile, if any
    pub fn source_files(&self) -> &[String] {
        match self {
            NativeDependency::Simple(_) => &[],
            NativeDependency::Detailed(d) => &d.sources,
        }
    }
}

/// Build configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildConfig {
    #[serde(default)]
    pub opt_level: Option<u8>,
    #[serde(default)]
    pub debug: Option<bool>,
    #[serde(default)]
    pub target: Option<String>,
    /// Borrow check mode: "strict" (default), "warn", or "off"
    #[serde(default = "default_borrow_check")]
    pub borrow_check: Option<String>,
}

pub(crate) fn default_borrow_check() -> Option<String> {
    Some("strict".to_string())
}

/// Resolved dependency with its path
#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    pub name: String,
    pub path: PathBuf,
    pub _manifest: PackageManifest,
}

/// Package manager errors
#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("could not find `vais.toml` in `{0}`")]
    ManifestNotFound(PathBuf),

    #[error("failed to read `{path}`: {source}")]
    ReadError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse `{path}`: {message}")]
    ParseError { path: PathBuf, message: String },

    #[error("dependency `{name}` not found: path `{path}` does not exist")]
    DependencyNotFound { name: String, path: PathBuf },

    #[error("registry dependency `{name}` (version {version}) is not installed; run `vais pkg install` first")]
    RegistryDepNotInstalled { name: String, version: String },

    #[error("cyclic dependency detected: {cycle}")]
    CyclicDependency { cycle: String },

    #[error("failed to write file `{path}`: {source}")]
    WriteError {
        path: PathBuf,
        source: std::io::Error,
    },
}

pub type PackageResult<T> = Result<T, PackageError>;

#[cfg(test)]
mod tests {
    use super::*;

    // ── NativeDependency::Simple tests ──

    #[test]
    fn test_native_dep_simple_lib_flags() {
        let dep = NativeDependency::Simple("ssl".to_string());
        assert_eq!(dep.lib_flags(), vec!["-lssl"]);
    }

    #[test]
    fn test_native_dep_simple_include_flag() {
        let dep = NativeDependency::Simple("ssl".to_string());
        assert!(dep.include_flag().is_none());
    }

    #[test]
    fn test_native_dep_simple_lib_path_flag() {
        let dep = NativeDependency::Simple("ssl".to_string());
        assert!(dep.lib_path_flag().is_none());
    }

    #[test]
    fn test_native_dep_simple_source_files() {
        let dep = NativeDependency::Simple("ssl".to_string());
        assert!(dep.source_files().is_empty());
    }

    // ── NativeDependency::Detailed tests ──

    #[test]
    fn test_native_dep_detailed_lib_flags() {
        let dep = NativeDependency::Detailed(NativeDependencyDetail {
            libs: vec!["ssl".to_string(), "crypto".to_string()],
            include: None,
            lib_path: None,
            system: None,
            sources: vec![],
        });
        assert_eq!(dep.lib_flags(), vec!["-lssl", "-lcrypto"]);
    }

    #[test]
    fn test_native_dep_detailed_empty_libs() {
        let dep = NativeDependency::Detailed(NativeDependencyDetail {
            libs: vec![],
            include: None,
            lib_path: None,
            system: None,
            sources: vec![],
        });
        assert!(dep.lib_flags().is_empty());
    }

    #[test]
    fn test_native_dep_detailed_include_flag() {
        let dep = NativeDependency::Detailed(NativeDependencyDetail {
            libs: vec![],
            include: Some("/usr/include/openssl".to_string()),
            lib_path: None,
            system: None,
            sources: vec![],
        });
        assert_eq!(
            dep.include_flag(),
            Some("-I/usr/include/openssl".to_string())
        );
    }

    #[test]
    fn test_native_dep_detailed_no_include_flag() {
        let dep = NativeDependency::Detailed(NativeDependencyDetail {
            libs: vec![],
            include: None,
            lib_path: None,
            system: None,
            sources: vec![],
        });
        assert!(dep.include_flag().is_none());
    }

    #[test]
    fn test_native_dep_detailed_lib_path_flag() {
        let dep = NativeDependency::Detailed(NativeDependencyDetail {
            libs: vec![],
            include: None,
            lib_path: Some("/usr/lib".to_string()),
            system: None,
            sources: vec![],
        });
        assert_eq!(dep.lib_path_flag(), Some("-L/usr/lib".to_string()));
    }

    #[test]
    fn test_native_dep_detailed_no_lib_path_flag() {
        let dep = NativeDependency::Detailed(NativeDependencyDetail {
            libs: vec![],
            include: None,
            lib_path: None,
            system: None,
            sources: vec![],
        });
        assert!(dep.lib_path_flag().is_none());
    }

    #[test]
    fn test_native_dep_detailed_source_files() {
        let dep = NativeDependency::Detailed(NativeDependencyDetail {
            libs: vec![],
            include: None,
            lib_path: None,
            system: None,
            sources: vec!["vendor/lib.c".to_string(), "vendor/util.c".to_string()],
        });
        assert_eq!(dep.source_files(), &["vendor/lib.c", "vendor/util.c"]);
    }

    #[test]
    fn test_native_dep_detailed_no_source_files() {
        let dep = NativeDependency::Detailed(NativeDependencyDetail {
            libs: vec![],
            include: None,
            lib_path: None,
            system: None,
            sources: vec![],
        });
        assert!(dep.source_files().is_empty());
    }

    // ── Serde tests ──

    #[test]
    fn test_package_info_default() {
        let info = PackageInfo::default();
        assert!(info.name.is_empty());
        assert!(info.version.is_empty());
        assert!(info.authors.is_empty());
        assert!(info.description.is_none());
        assert!(info.license.is_none());
    }

    #[test]
    fn test_package_info_serde() {
        let info = PackageInfo {
            name: "my-pkg".to_string(),
            version: "1.0.0".to_string(),
            authors: vec!["Author".to_string()],
            description: Some("A test package".to_string()),
            license: Some("MIT".to_string()),
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: PackageInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "my-pkg");
        assert_eq!(parsed.license, Some("MIT".to_string()));
    }

    #[test]
    fn test_build_config_default() {
        let config = BuildConfig::default();
        assert!(config.opt_level.is_none());
        assert!(config.debug.is_none());
        assert!(config.target.is_none());
    }

    #[test]
    fn test_default_borrow_check() {
        let val = default_borrow_check();
        assert_eq!(val, Some("strict".to_string()));
    }

    #[test]
    fn test_dependency_version_serde() {
        let dep = Dependency::Version("1.0.0".to_string());
        let json = serde_json::to_string(&dep).unwrap();
        let parsed: Dependency = serde_json::from_str(&json).unwrap();
        match parsed {
            Dependency::Version(v) => assert_eq!(v, "1.0.0"),
            _ => panic!("Expected Version variant"),
        }
    }

    #[test]
    fn test_dependency_detailed_serde() {
        let dep = Dependency::Detailed(DetailedDependency {
            version: Some("1.0.0".to_string()),
            path: Some("../local".to_string()),
            features: vec!["json".to_string()],
            registry: None,
            workspace: None,
        });
        let json = serde_json::to_string(&dep).unwrap();
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn test_package_manifest_default_deps() {
        let toml_str = r#"
[package]
name = "test"
version = "0.1.0"
"#;
        let manifest: PackageManifest = toml::from_str(toml_str).unwrap();
        assert!(manifest.dependencies.is_empty());
        assert!(manifest.dev_dependencies.is_empty());
        assert!(manifest.native_dependencies.is_empty());
    }

    #[test]
    fn test_package_manifest_with_deps() {
        let toml_str = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
json = "1.0.0"
"#;
        let manifest: PackageManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.dependencies.len(), 1);
        assert!(manifest.dependencies.contains_key("json"));
    }

    #[test]
    fn test_native_dependency_simple_serde() {
        let toml_str = r#"
[package]
name = "test"
version = "0.1.0"

[native-dependencies]
zlib = "z"
"#;
        let manifest: PackageManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.native_dependencies.len(), 1);
        let zlib = &manifest.native_dependencies["zlib"];
        assert_eq!(zlib.lib_flags(), vec!["-lz"]);
    }

    #[test]
    fn test_package_error_manifest_not_found() {
        let err = PackageError::ManifestNotFound(PathBuf::from("/path/to/project"));
        let msg = err.to_string();
        assert!(msg.contains("vais.toml"));
        assert!(msg.contains("/path/to/project"));
    }

    #[test]
    fn test_package_error_dependency_not_found() {
        let err = PackageError::DependencyNotFound {
            name: "my-dep".to_string(),
            path: PathBuf::from("../my-dep"),
        };
        let msg = err.to_string();
        assert!(msg.contains("my-dep"));
        assert!(msg.contains("not found"));
    }

    #[test]
    fn test_package_error_registry_dep_not_installed() {
        let err = PackageError::RegistryDepNotInstalled {
            name: "json".to_string(),
            version: "1.0.0".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("json"));
        assert!(msg.contains("not installed"));
    }

    #[test]
    fn test_package_error_cyclic_dependency() {
        let err = PackageError::CyclicDependency {
            cycle: "a -> b -> a".to_string(),
        };
        assert!(err.to_string().contains("cyclic"));
    }

    #[test]
    fn test_package_error_parse_error() {
        let err = PackageError::ParseError {
            path: PathBuf::from("vais.toml"),
            message: "invalid syntax".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("vais.toml"));
        assert!(msg.contains("invalid syntax"));
    }

    #[test]
    fn test_detailed_dependency_defaults() {
        let dep = DetailedDependency {
            version: None,
            path: None,
            features: vec![],
            registry: None,
            workspace: None,
        };
        assert!(dep.version.is_none());
        assert!(dep.path.is_none());
        assert!(dep.features.is_empty());
    }

    #[test]
    fn test_package_manifest_features_optional() {
        let toml_str = r#"
[package]
name = "test"
version = "0.1.0"
"#;
        let manifest: PackageManifest = toml::from_str(toml_str).unwrap();
        assert!(manifest.features.is_none());
    }

    #[test]
    fn test_package_manifest_workspace_optional() {
        let toml_str = r#"
[package]
name = "test"
version = "0.1.0"
"#;
        let manifest: PackageManifest = toml::from_str(toml_str).unwrap();
        assert!(manifest.workspace.is_none());
    }
}

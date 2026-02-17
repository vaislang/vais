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

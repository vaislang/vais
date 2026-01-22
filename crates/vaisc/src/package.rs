//! Package manager for Vais
//!
//! Handles vais.toml parsing, dependency resolution, and package builds.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Package manifest (vais.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub package: PackageInfo,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    pub build: BuildConfig,
}

/// Package information section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
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
    ParseError {
        path: PathBuf,
        message: String,
    },

    #[error("dependency `{name}` not found: path `{path}` does not exist")]
    DependencyNotFound {
        name: String,
        path: PathBuf,
    },

    #[error("cyclic dependency detected: {cycle}")]
    CyclicDependency {
        cycle: String,
    },

    #[error("failed to write file `{path}`: {source}")]
    WriteError {
        path: PathBuf,
        source: std::io::Error,
    },
}

pub type PackageResult<T> = Result<T, PackageError>;

/// Load a package manifest from a directory
pub fn load_manifest(dir: &Path) -> PackageResult<PackageManifest> {
    let manifest_path = dir.join("vais.toml");

    if !manifest_path.exists() {
        return Err(PackageError::ManifestNotFound(dir.to_path_buf()));
    }

    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| PackageError::ReadError {
            path: manifest_path.clone(),
            source: e,
        })?;

    let manifest: PackageManifest = toml::from_str(&content)
        .map_err(|e| PackageError::ParseError {
            path: manifest_path,
            message: e.to_string(),
        })?;

    Ok(manifest)
}

/// Find vais.toml by searching current dir and parents
pub fn find_manifest(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        let manifest = current.join("vais.toml");
        if manifest.exists() {
            return Some(current);
        }

        if !current.pop() {
            return None;
        }
    }
}

/// Initialize a new package in the given directory
pub fn init_package(dir: &Path, name: Option<&str>) -> PackageResult<()> {
    let manifest_path = dir.join("vais.toml");

    if manifest_path.exists() {
        return Err(PackageError::ParseError {
            path: manifest_path,
            message: "vais.toml already exists".to_string(),
        });
    }

    // Use directory name if no name provided
    let pkg_name = name
        .map(String::from)
        .or_else(|| dir.file_name().and_then(|n| n.to_str()).map(String::from))
        .unwrap_or_else(|| "my-package".to_string());

    let manifest = PackageManifest {
        package: PackageInfo {
            name: pkg_name,
            version: "0.1.0".to_string(),
            authors: Vec::new(),
            description: None,
            license: Some("MIT".to_string()),
        },
        dependencies: HashMap::new(),
        dev_dependencies: HashMap::new(),
        build: BuildConfig::default(),
    };

    let content = toml::to_string_pretty(&manifest)
        .map_err(|e| PackageError::ParseError {
            path: manifest_path.clone(),
            message: e.to_string(),
        })?;

    fs::write(&manifest_path, content)
        .map_err(|e| PackageError::WriteError {
            path: manifest_path,
            source: e,
        })?;

    // Create src directory
    let src_dir = dir.join("src");
    if !src_dir.exists() {
        fs::create_dir_all(&src_dir)
            .map_err(|e| PackageError::WriteError {
                path: src_dir.clone(),
                source: e,
            })?;
    }

    // Create main.vais if it doesn't exist
    let main_path = src_dir.join("main.vais");
    if !main_path.exists() {
        let main_content = r#"# Main entry point

F main() -> i64 {
    puts("Hello, Vais!")
    0
}
"#;
        fs::write(&main_path, main_content)
            .map_err(|e| PackageError::WriteError {
                path: main_path,
                source: e,
            })?;
    }

    Ok(())
}

/// Resolved dependency with its path
#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    pub name: String,
    pub path: PathBuf,
    pub manifest: PackageManifest,
}

/// Resolve all dependencies for a package
pub fn resolve_dependencies(
    manifest: &PackageManifest,
    base_dir: &Path,
) -> PackageResult<Vec<ResolvedDependency>> {
    let mut resolved = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut stack = Vec::new();

    resolve_deps_recursive(
        manifest,
        base_dir,
        &mut resolved,
        &mut visited,
        &mut stack,
    )?;

    Ok(resolved)
}

fn resolve_deps_recursive(
    manifest: &PackageManifest,
    base_dir: &Path,
    resolved: &mut Vec<ResolvedDependency>,
    visited: &mut std::collections::HashSet<PathBuf>,
    stack: &mut Vec<String>,
) -> PackageResult<()> {
    for (name, dep) in &manifest.dependencies {
        let dep_path = match dep {
            Dependency::Detailed(d) if d.path.is_some() => {
                let path = d.path.as_ref().unwrap();
                base_dir.join(path)
            }
            _ => continue, // Skip non-path dependencies for now
        };

        let canonical = dep_path.canonicalize()
            .map_err(|_| PackageError::DependencyNotFound {
                name: name.clone(),
                path: dep_path.clone(),
            })?;

        // Check for cycles
        if stack.contains(name) {
            let cycle = format!("{} -> {}", stack.join(" -> "), name);
            return Err(PackageError::CyclicDependency { cycle });
        }

        // Skip if already processed
        if visited.contains(&canonical) {
            continue;
        }

        visited.insert(canonical.clone());
        stack.push(name.clone());

        // Load dependency's manifest
        let dep_manifest = load_manifest(&canonical)?;

        // Recursively resolve transitive dependencies
        resolve_deps_recursive(
            &dep_manifest,
            &canonical,
            resolved,
            visited,
            stack,
        )?;

        stack.pop();

        // Add this dependency after its own dependencies
        resolved.push(ResolvedDependency {
            name: name.clone(),
            path: canonical,
            manifest: dep_manifest,
        });
    }

    Ok(())
}

/// Add a dependency to the manifest
pub fn add_dependency(
    manifest_path: &Path,
    name: &str,
    path: Option<&str>,
    version: Option<&str>,
) -> PackageResult<()> {
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| PackageError::ReadError {
            path: manifest_path.to_path_buf(),
            source: e,
        })?;

    let mut manifest: PackageManifest = toml::from_str(&content)
        .map_err(|e| PackageError::ParseError {
            path: manifest_path.to_path_buf(),
            message: e.to_string(),
        })?;

    let dep = if let Some(p) = path {
        Dependency::Detailed(DetailedDependency {
            path: Some(p.to_string()),
            version: version.map(String::from),
            features: Vec::new(),
            registry: None,
        })
    } else if let Some(v) = version {
        // Registry dependency with version
        Dependency::Version(v.to_string())
    } else {
        return Err(PackageError::ParseError {
            path: manifest_path.to_path_buf(),
            message: "dependency must have either path or version".to_string(),
        });
    };

    manifest.dependencies.insert(name.to_string(), dep);

    let new_content = toml::to_string_pretty(&manifest)
        .map_err(|e| PackageError::ParseError {
            path: manifest_path.to_path_buf(),
            message: e.to_string(),
        })?;

    fs::write(manifest_path, new_content)
        .map_err(|e| PackageError::WriteError {
            path: manifest_path.to_path_buf(),
            source: e,
        })?;

    Ok(())
}

/// Remove a dependency from the manifest
pub fn remove_dependency(manifest_path: &Path, name: &str) -> PackageResult<()> {
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| PackageError::ReadError {
            path: manifest_path.to_path_buf(),
            source: e,
        })?;

    let mut manifest: PackageManifest = toml::from_str(&content)
        .map_err(|e| PackageError::ParseError {
            path: manifest_path.to_path_buf(),
            message: e.to_string(),
        })?;

    if manifest.dependencies.remove(name).is_none() {
        return Err(PackageError::ParseError {
            path: manifest_path.to_path_buf(),
            message: format!("dependency `{}` not found", name),
        });
    }

    let new_content = toml::to_string_pretty(&manifest)
        .map_err(|e| PackageError::ParseError {
            path: manifest_path.to_path_buf(),
            message: e.to_string(),
        })?;

    fs::write(manifest_path, new_content)
        .map_err(|e| PackageError::WriteError {
            path: manifest_path.to_path_buf(),
            source: e,
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_init_package() {
        let dir = tempdir().unwrap();
        init_package(dir.path(), Some("test-pkg")).unwrap();

        let manifest = load_manifest(dir.path()).unwrap();
        assert_eq!(manifest.package.name, "test-pkg");
        assert_eq!(manifest.package.version, "0.1.0");

        assert!(dir.path().join("src/main.vais").exists());
    }

    #[test]
    fn test_load_manifest() {
        let dir = tempdir().unwrap();
        let toml_content = r#"
[package]
name = "my-pkg"
version = "1.0.0"
description = "A test package"

[dependencies]
other-lib = { path = "../other" }

[build]
opt_level = 2
"#;
        fs::write(dir.path().join("vais.toml"), toml_content).unwrap();

        let manifest = load_manifest(dir.path()).unwrap();
        assert_eq!(manifest.package.name, "my-pkg");
        assert_eq!(manifest.package.version, "1.0.0");
        assert_eq!(manifest.dependencies.len(), 1);
        assert_eq!(manifest.build.opt_level, Some(2));
    }

    #[test]
    fn test_add_remove_dependency() {
        let dir = tempdir().unwrap();
        init_package(dir.path(), Some("test-pkg")).unwrap();

        let manifest_path = dir.path().join("vais.toml");

        // Add dependency
        add_dependency(&manifest_path, "my-lib", Some("../my-lib"), None).unwrap();

        let manifest = load_manifest(dir.path()).unwrap();
        assert!(manifest.dependencies.contains_key("my-lib"));

        // Remove dependency
        remove_dependency(&manifest_path, "my-lib").unwrap();

        let manifest = load_manifest(dir.path()).unwrap();
        assert!(!manifest.dependencies.contains_key("my-lib"));
    }

    #[test]
    fn test_find_manifest() {
        let dir = tempdir().unwrap();
        init_package(dir.path(), Some("test-pkg")).unwrap();

        // Create nested directory
        let nested = dir.path().join("src/nested");
        fs::create_dir_all(&nested).unwrap();

        // Should find manifest from nested dir
        let found = find_manifest(&nested);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), dir.path());
    }
}

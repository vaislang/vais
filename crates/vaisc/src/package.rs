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
    /// Borrow check mode: "strict" (default), "warn", or "off"
    #[serde(default = "default_borrow_check")]
    pub borrow_check: Option<String>,
}

fn default_borrow_check() -> Option<String> {
    Some("strict".to_string())
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

/// Load a package manifest from a directory
pub fn load_manifest(dir: &Path) -> PackageResult<PackageManifest> {
    let manifest_path = dir.join("vais.toml");

    if !manifest_path.exists() {
        return Err(PackageError::ManifestNotFound(dir.to_path_buf()));
    }

    let content = fs::read_to_string(&manifest_path).map_err(|e| PackageError::ReadError {
        path: manifest_path.clone(),
        source: e,
    })?;

    let manifest: PackageManifest =
        toml::from_str(&content).map_err(|e| PackageError::ParseError {
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

    let content = toml::to_string_pretty(&manifest).map_err(|e| PackageError::ParseError {
        path: manifest_path.clone(),
        message: e.to_string(),
    })?;

    fs::write(&manifest_path, content).map_err(|e| PackageError::WriteError {
        path: manifest_path,
        source: e,
    })?;

    // Create src directory
    let src_dir = dir.join("src");
    if !src_dir.exists() {
        fs::create_dir_all(&src_dir).map_err(|e| PackageError::WriteError {
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
        fs::write(&main_path, main_content).map_err(|e| PackageError::WriteError {
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
    pub _manifest: PackageManifest,
}

/// Resolve only path dependencies for a package (legacy behavior)
#[allow(dead_code)]
pub fn resolve_dependencies(
    manifest: &PackageManifest,
    base_dir: &Path,
) -> PackageResult<Vec<ResolvedDependency>> {
    resolve_all_dependencies(manifest, base_dir, None)
}

/// Resolve all dependencies (path + registry) for a package.
///
/// When `cache_root` is provided, registry dependencies (version-only or detailed
/// without a path) are looked up in the cache directory (`~/.vais/registry/cache/`).
/// If the package is not found in the cache, an error is returned directing the
/// user to run `vais pkg install`.
pub fn resolve_all_dependencies(
    manifest: &PackageManifest,
    base_dir: &Path,
    cache_root: Option<&Path>,
) -> PackageResult<Vec<ResolvedDependency>> {
    let mut resolved = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut stack = Vec::new();

    resolve_deps_recursive(
        manifest,
        base_dir,
        cache_root,
        &mut resolved,
        &mut visited,
        &mut stack,
    )?;

    Ok(resolved)
}

/// Look up a registry dependency in the cache directory.
///
/// The cache layout is: `<cache_root>/cache/<name>/<version>/extracted/`
/// We find the best matching version directory that contains an "extracted" subdirectory.
pub fn find_cached_registry_dep(
    cache_root: &Path,
    name: &str,
    version_str: &str,
) -> Option<PathBuf> {
    let pkg_cache_dir = cache_root.join("cache").join(name);
    if !pkg_cache_dir.exists() {
        return None;
    }

    // Try exact version match first
    let exact_path = pkg_cache_dir.join(version_str).join("extracted");
    if exact_path.exists() {
        return Some(exact_path);
    }

    // Try stripping leading version operators (^, ~, >=, etc.) for a simple match
    let stripped = version_str
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim_start_matches(">=")
        .trim_start_matches("<=")
        .trim_start_matches('>')
        .trim_start_matches('<')
        .trim_start_matches('=')
        .trim();

    if stripped != version_str {
        let stripped_path = pkg_cache_dir.join(stripped).join("extracted");
        if stripped_path.exists() {
            return Some(stripped_path);
        }
    }

    // Scan available versions in the cache directory
    if let Ok(entries) = fs::read_dir(&pkg_cache_dir) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                let extracted = entry_path.join("extracted");
                if extracted.exists() {
                    return Some(extracted);
                }
            }
        }
    }

    None
}

/// Get the default registry cache root path (`~/.vais/registry`)
pub fn default_registry_cache_root() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".vais").join("registry"))
}

fn resolve_deps_recursive(
    manifest: &PackageManifest,
    base_dir: &Path,
    cache_root: Option<&Path>,
    resolved: &mut Vec<ResolvedDependency>,
    visited: &mut std::collections::HashSet<PathBuf>,
    stack: &mut Vec<String>,
) -> PackageResult<()> {
    for (name, dep) in &manifest.dependencies {
        let dep_path = match dep {
            Dependency::Detailed(d) if d.path.is_some() => {
                // Path dependency
                let path = d.path.as_ref().unwrap();
                base_dir.join(path)
            }
            Dependency::Version(v) => {
                // Registry dependency with version string
                if let Some(cache) = cache_root {
                    match find_cached_registry_dep(cache, name, v) {
                        Some(path) => path,
                        None => {
                            return Err(PackageError::RegistryDepNotInstalled {
                                name: name.clone(),
                                version: v.clone(),
                            })
                        }
                    }
                } else {
                    continue; // No cache root provided, skip registry deps
                }
            }
            Dependency::Detailed(d) => {
                // Detailed dependency without path = registry dependency
                let version = d.version.as_deref().unwrap_or("*");
                if let Some(cache) = cache_root {
                    match find_cached_registry_dep(cache, name, version) {
                        Some(path) => path,
                        None => {
                            return Err(PackageError::RegistryDepNotInstalled {
                                name: name.clone(),
                                version: version.to_string(),
                            })
                        }
                    }
                } else {
                    continue; // No cache root provided, skip registry deps
                }
            }
        };

        let canonical = dep_path
            .canonicalize()
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

        // Load dependency's manifest if it exists (registry deps might not have one)
        let dep_manifest = match load_manifest(&canonical) {
            Ok(m) => m,
            Err(_) => {
                // For registry deps without a manifest at the extracted root,
                // create a minimal manifest so we can still track the dependency
                PackageManifest {
                    package: PackageInfo {
                        name: name.clone(),
                        version: "0.0.0".to_string(),
                        authors: Vec::new(),
                        description: None,
                        license: None,
                    },
                    dependencies: HashMap::new(),
                    dev_dependencies: HashMap::new(),
                    build: BuildConfig::default(),
                }
            }
        };

        // Recursively resolve transitive dependencies
        resolve_deps_recursive(
            &dep_manifest,
            &canonical,
            cache_root,
            resolved,
            visited,
            stack,
        )?;

        stack.pop();

        // Add this dependency after its own dependencies
        resolved.push(ResolvedDependency {
            name: name.clone(),
            path: canonical,
            _manifest: dep_manifest,
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
    let content = fs::read_to_string(manifest_path).map_err(|e| PackageError::ReadError {
        path: manifest_path.to_path_buf(),
        source: e,
    })?;

    let mut manifest: PackageManifest =
        toml::from_str(&content).map_err(|e| PackageError::ParseError {
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

    let new_content = toml::to_string_pretty(&manifest).map_err(|e| PackageError::ParseError {
        path: manifest_path.to_path_buf(),
        message: e.to_string(),
    })?;

    fs::write(manifest_path, new_content).map_err(|e| PackageError::WriteError {
        path: manifest_path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}

/// Remove a dependency from the manifest
pub fn remove_dependency(manifest_path: &Path, name: &str) -> PackageResult<()> {
    let content = fs::read_to_string(manifest_path).map_err(|e| PackageError::ReadError {
        path: manifest_path.to_path_buf(),
        source: e,
    })?;

    let mut manifest: PackageManifest =
        toml::from_str(&content).map_err(|e| PackageError::ParseError {
            path: manifest_path.to_path_buf(),
            message: e.to_string(),
        })?;

    if manifest.dependencies.remove(name).is_none() {
        return Err(PackageError::ParseError {
            path: manifest_path.to_path_buf(),
            message: format!("dependency `{}` not found", name),
        });
    }

    let new_content = toml::to_string_pretty(&manifest).map_err(|e| PackageError::ParseError {
        path: manifest_path.to_path_buf(),
        message: e.to_string(),
    })?;

    fs::write(manifest_path, new_content).map_err(|e| PackageError::WriteError {
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

    #[test]
    fn test_resolve_all_dependencies_with_path_dep() {
        let root = tempdir().unwrap();

        // Create main package
        let main_dir = root.path().join("main-pkg");
        fs::create_dir_all(&main_dir).unwrap();
        init_package(&main_dir, Some("main-pkg")).unwrap();

        // Create dependency package
        let dep_dir = root.path().join("my-dep");
        fs::create_dir_all(&dep_dir).unwrap();
        init_package(&dep_dir, Some("my-dep")).unwrap();

        // Add path dependency to main package
        let manifest_path = main_dir.join("vais.toml");
        add_dependency(&manifest_path, "my-dep", Some("../my-dep"), None).unwrap();

        let manifest = load_manifest(&main_dir).unwrap();
        let deps = resolve_all_dependencies(&manifest, &main_dir, None).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "my-dep");
    }

    #[test]
    fn test_resolve_registry_dep_from_cache() {
        let root = tempdir().unwrap();

        // Create a fake registry cache with an extracted package
        let cache_root = root.path().join("registry");
        let extracted = cache_root
            .join("cache")
            .join("json-parser")
            .join("1.0.0")
            .join("extracted");
        fs::create_dir_all(&extracted).unwrap();

        // Create a minimal vais.toml in the extracted package
        let dep_manifest = r#"
[package]
name = "json-parser"
version = "1.0.0"
"#;
        fs::write(extracted.join("vais.toml"), dep_manifest).unwrap();
        fs::create_dir_all(extracted.join("src")).unwrap();
        fs::write(extracted.join("src/lib.vais"), "# json-parser lib\n").unwrap();

        // Create main package with a registry dependency
        let main_dir = root.path().join("main-pkg");
        fs::create_dir_all(&main_dir).unwrap();
        init_package(&main_dir, Some("main-pkg")).unwrap();

        // Write manifest with registry dep
        let toml_content = r#"
[package]
name = "main-pkg"
version = "0.1.0"

[dependencies]
json-parser = "1.0.0"
"#;
        fs::write(main_dir.join("vais.toml"), toml_content).unwrap();

        let manifest = load_manifest(&main_dir).unwrap();
        let deps = resolve_all_dependencies(&manifest, &main_dir, Some(&cache_root)).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "json-parser");
        assert!(deps[0].path.to_string_lossy().contains("extracted"));
    }

    #[test]
    fn test_resolve_registry_dep_not_installed() {
        let root = tempdir().unwrap();

        // Empty cache
        let cache_root = root.path().join("registry");
        fs::create_dir_all(&cache_root).unwrap();

        // Create main package with a registry dependency
        let main_dir = root.path().join("main-pkg");
        fs::create_dir_all(&main_dir).unwrap();

        let toml_content = r#"
[package]
name = "main-pkg"
version = "0.1.0"

[dependencies]
nonexistent-pkg = "2.0.0"
"#;
        fs::write(main_dir.join("vais.toml"), toml_content).unwrap();
        fs::create_dir_all(main_dir.join("src")).unwrap();
        fs::write(main_dir.join("src/main.vais"), "").unwrap();

        let manifest = load_manifest(&main_dir).unwrap();
        let result = resolve_all_dependencies(&manifest, &main_dir, Some(&cache_root));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not installed"));
        assert!(err.contains("nonexistent-pkg"));
    }

    #[test]
    fn test_resolve_mixed_path_and_registry_deps() {
        let root = tempdir().unwrap();

        // Create path dependency
        let path_dep_dir = root.path().join("local-lib");
        fs::create_dir_all(&path_dep_dir).unwrap();
        init_package(&path_dep_dir, Some("local-lib")).unwrap();

        // Create registry cache with another package
        let cache_root = root.path().join("registry");
        let extracted = cache_root
            .join("cache")
            .join("remote-lib")
            .join("0.5.0")
            .join("extracted");
        fs::create_dir_all(&extracted).unwrap();
        let dep_manifest = r#"
[package]
name = "remote-lib"
version = "0.5.0"
"#;
        fs::write(extracted.join("vais.toml"), dep_manifest).unwrap();
        fs::create_dir_all(extracted.join("src")).unwrap();
        fs::write(extracted.join("src/lib.vais"), "").unwrap();

        // Create main package with both types of deps
        let main_dir = root.path().join("main-pkg");
        fs::create_dir_all(&main_dir).unwrap();

        let toml_content = r#"
[package]
name = "main-pkg"
version = "0.1.0"

[dependencies]
local-lib = { path = "../local-lib" }
remote-lib = "0.5.0"
"#;
        fs::write(main_dir.join("vais.toml"), toml_content).unwrap();
        fs::create_dir_all(main_dir.join("src")).unwrap();
        fs::write(main_dir.join("src/main.vais"), "").unwrap();

        let manifest = load_manifest(&main_dir).unwrap();
        let deps = resolve_all_dependencies(&manifest, &main_dir, Some(&cache_root)).unwrap();
        assert_eq!(deps.len(), 2);

        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"local-lib"));
        assert!(names.contains(&"remote-lib"));
    }

    #[test]
    fn test_find_cached_registry_dep_with_version_prefix() {
        let root = tempdir().unwrap();

        // Create cache with version 1.2.3
        let cache_root = root.path().join("registry");
        let extracted = cache_root
            .join("cache")
            .join("my-pkg")
            .join("1.2.3")
            .join("extracted");
        fs::create_dir_all(&extracted).unwrap();

        // Should find with exact version
        assert!(find_cached_registry_dep(&cache_root, "my-pkg", "1.2.3").is_some());

        // Should find with ^ prefix (stripped)
        assert!(find_cached_registry_dep(&cache_root, "my-pkg", "^1.2.3").is_some());

        // Should find with ~ prefix (stripped)
        assert!(find_cached_registry_dep(&cache_root, "my-pkg", "~1.2.3").is_some());

        // Should not find nonexistent package
        assert!(find_cached_registry_dep(&cache_root, "no-such-pkg", "1.0.0").is_none());
    }

    #[test]
    fn test_registry_dep_without_cache_root_skipped() {
        let root = tempdir().unwrap();

        let main_dir = root.path().join("main-pkg");
        fs::create_dir_all(&main_dir).unwrap();

        let toml_content = r#"
[package]
name = "main-pkg"
version = "0.1.0"

[dependencies]
some-registry-pkg = "1.0.0"
"#;
        fs::write(main_dir.join("vais.toml"), toml_content).unwrap();
        fs::create_dir_all(main_dir.join("src")).unwrap();
        fs::write(main_dir.join("src/main.vais"), "").unwrap();

        let manifest = load_manifest(&main_dir).unwrap();
        // Without cache_root, registry deps should be skipped (not cause an error)
        let deps = resolve_all_dependencies(&manifest, &main_dir, None).unwrap();
        assert_eq!(deps.len(), 0);
    }
}

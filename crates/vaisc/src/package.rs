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

/// Feature flags configuration section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureConfig {
    /// Default features enabled when no --no-default-features is passed
    #[serde(default)]
    pub default: Vec<String>,
    /// All available features and their dependencies
    /// Each feature maps to a list of other features/optional deps it enables
    #[serde(flatten)]
    pub features: HashMap<String, Vec<String>>,
}

impl FeatureConfig {
    /// Resolve the full set of enabled features given user selections
    pub fn resolve_features(&self, selected: &[String], use_defaults: bool) -> Vec<String> {
        let mut enabled = std::collections::HashSet::new();
        let mut stack: Vec<String> = selected.to_vec();

        if use_defaults {
            stack.extend(self.default.clone());
        }

        while let Some(feat) = stack.pop() {
            if enabled.insert(feat.clone()) {
                // Add transitive feature dependencies
                if let Some(deps) = self.features.get(&feat) {
                    for dep in deps {
                        if !enabled.contains(dep) {
                            stack.push(dep.clone());
                        }
                    }
                }
            }
        }

        let mut result: Vec<String> = enabled.into_iter().collect();
        result.sort();
        result
    }

    /// Get all defined feature names
    pub fn all_features(&self) -> Vec<String> {
        let mut all: Vec<String> = self.features.keys().cloned().collect();
        all.sort();
        all
    }
}

/// Workspace configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Member package paths (supports glob patterns like "crates/*")
    #[serde(default)]
    pub members: Vec<String>,
    /// Shared dependency versions for workspace members
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
}

/// A resolved workspace with all member packages.
/// Used by workspace resolution functions for multi-package builds.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ResolvedWorkspace {
    /// Root directory of the workspace
    pub root: PathBuf,
    /// Root manifest
    pub root_manifest: PackageManifest,
    /// Resolved member packages (directory path, manifest)
    pub members: Vec<WorkspaceMember>,
}

/// A member of a workspace
#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    /// Directory containing the member's vais.toml
    pub path: PathBuf,
    /// Parsed manifest with workspace dependencies resolved
    pub manifest: PackageManifest,
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
        native_dependencies: HashMap::new(),
        build: BuildConfig::default(),
        features: None,
        workspace: None,
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

/// Resolve only path dependencies for a package (legacy behavior).
/// Superseded by SemVer resolver; kept for backward compatibility.
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
                    native_dependencies: HashMap::new(),
                    build: BuildConfig::default(),
                    features: None,
                    workspace: None,
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
            workspace: None,
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

/// Find workspace root by searching for a vais.toml with [workspace] section
pub fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        let manifest_path = current.join("vais.toml");
        if manifest_path.exists() {
            if let Ok(content) = fs::read_to_string(&manifest_path) {
                if let Ok(manifest) = toml::from_str::<PackageManifest>(&content) {
                    if manifest.workspace.is_some() {
                        return Some(current);
                    }
                }
            }
        }

        if !current.pop() {
            return None;
        }
    }
}

/// Resolve all workspace members from glob patterns
pub fn resolve_workspace_members(workspace_root: &Path) -> PackageResult<ResolvedWorkspace> {
    let manifest = load_manifest(workspace_root)?;

    let workspace_config = manifest
        .workspace
        .as_ref()
        .ok_or_else(|| PackageError::ParseError {
            path: workspace_root.join("vais.toml"),
            message: "no [workspace] section found".to_string(),
        })?;

    let mut members = Vec::new();

    for pattern in &workspace_config.members {
        let full_pattern = workspace_root.join(pattern).display().to_string();

        let matched_dirs: Vec<PathBuf> = glob::glob(&full_pattern)
            .map_err(|e| PackageError::ParseError {
                path: workspace_root.join("vais.toml"),
                message: format!("invalid glob pattern '{}': {}", pattern, e),
            })?
            .filter_map(|entry| entry.ok())
            .filter(|p| p.is_dir() && p.join("vais.toml").exists())
            .collect();

        for member_dir in matched_dirs {
            let mut member_manifest = load_manifest(&member_dir)?;

            // Resolve workspace = true dependencies
            resolve_workspace_deps(&mut member_manifest, workspace_config);

            members.push(WorkspaceMember {
                path: member_dir,
                manifest: member_manifest,
            });
        }
    }

    // Also check if workspace root itself is a package (has [package] section with name)
    if !manifest.package.name.is_empty() {
        let mut root_manifest = manifest.clone();
        resolve_workspace_deps(&mut root_manifest, workspace_config);
        // Only add if not already in members
        let root_canonical = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());
        let already_added = members
            .iter()
            .any(|m| m.path.canonicalize().unwrap_or_else(|_| m.path.clone()) == root_canonical);
        if !already_added {
            members.insert(
                0,
                WorkspaceMember {
                    path: workspace_root.to_path_buf(),
                    manifest: root_manifest,
                },
            );
        }
    }

    Ok(ResolvedWorkspace {
        root: workspace_root.to_path_buf(),
        root_manifest: manifest,
        members,
    })
}

/// Resolve dependencies with `workspace = true` using workspace-level dependency definitions
fn resolve_workspace_deps(manifest: &mut PackageManifest, workspace_config: &WorkspaceConfig) {
    let ws_deps = &workspace_config.dependencies;

    let mut resolved_deps = HashMap::new();
    for (name, dep) in &manifest.dependencies {
        if let Dependency::Detailed(d) = dep {
            if d.workspace == Some(true) {
                // Look up in workspace dependencies
                if let Some(ws_dep) = ws_deps.get(name) {
                    resolved_deps.insert(name.clone(), ws_dep.clone());
                    continue;
                }
            }
        }
        resolved_deps.insert(name.clone(), dep.clone());
    }
    manifest.dependencies = resolved_deps;
}

/// Resolve path dependencies between workspace members automatically.
/// If a member depends on another member by name (without path), add the path.
pub fn resolve_inter_workspace_deps(workspace: &mut ResolvedWorkspace) {
    // Build a name -> path map of all members
    let member_paths: HashMap<String, PathBuf> = workspace
        .members
        .iter()
        .map(|m| (m.manifest.package.name.clone(), m.path.clone()))
        .collect();

    for member in &mut workspace.members {
        let member_base = member.path.clone();
        let mut updated_deps = HashMap::new();

        for (name, dep) in &member.manifest.dependencies {
            match dep {
                Dependency::Version(_)
                | Dependency::Detailed(DetailedDependency { path: None, .. }) => {
                    // Check if this dependency name matches a workspace member
                    if let Some(dep_path) = member_paths.get(name) {
                        // Calculate relative path from this member to the dependency
                        let rel_path = pathdiff_relative(&member_base, dep_path);
                        updated_deps.insert(
                            name.clone(),
                            Dependency::Detailed(DetailedDependency {
                                path: Some(rel_path),
                                version: match dep {
                                    Dependency::Version(v) => Some(v.clone()),
                                    Dependency::Detailed(d) => d.version.clone(),
                                },
                                features: match dep {
                                    Dependency::Detailed(d) => d.features.clone(),
                                    _ => Vec::new(),
                                },
                                registry: None,
                                workspace: None,
                            }),
                        );
                        continue;
                    }
                }
                _ => {}
            }
            updated_deps.insert(name.clone(), dep.clone());
        }

        member.manifest.dependencies = updated_deps;
    }
}

/// Compute relative path from `from_dir` to `to_dir`
fn pathdiff_relative(from: &Path, to: &Path) -> String {
    // Use canonicalized paths for accuracy
    let from_abs = from.canonicalize().unwrap_or_else(|_| from.to_path_buf());
    let to_abs = to.canonicalize().unwrap_or_else(|_| to.to_path_buf());

    // Find common prefix
    let from_parts: Vec<_> = from_abs.components().collect();
    let to_parts: Vec<_> = to_abs.components().collect();

    let common_len = from_parts
        .iter()
        .zip(to_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let up_count = from_parts.len() - common_len;
    let mut result = String::new();
    for _ in 0..up_count {
        if !result.is_empty() {
            result.push('/');
        }
        result.push_str("..");
    }

    for part in &to_parts[common_len..] {
        if !result.is_empty() {
            result.push('/');
        }
        result.push_str(&part.as_os_str().to_string_lossy());
    }

    if result.is_empty() {
        ".".to_string()
    } else {
        result
    }
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
    fn test_native_dependencies_simple() {
        let dir = tempdir().unwrap();
        let toml_content = r#"
[package]
name = "my-pkg"
version = "1.0.0"

[native-dependencies]
openssl = "ssl"
zlib = "z"
"#;
        fs::write(dir.path().join("vais.toml"), toml_content).unwrap();

        let manifest = load_manifest(dir.path()).unwrap();
        assert_eq!(manifest.native_dependencies.len(), 2);

        let ssl = &manifest.native_dependencies["openssl"];
        assert_eq!(ssl.lib_flags(), vec!["-lssl"]);
        assert!(ssl.include_flag().is_none());

        let z = &manifest.native_dependencies["zlib"];
        assert_eq!(z.lib_flags(), vec!["-lz"]);
    }

    #[test]
    fn test_native_dependencies_detailed() {
        let dir = tempdir().unwrap();
        let toml_content = r#"
[package]
name = "my-pkg"
version = "1.0.0"

[native-dependencies.openssl]
libs = ["ssl", "crypto"]
include = "/usr/include/openssl"
lib_path = "/usr/lib"
system = true

[native-dependencies.custom]
libs = ["mylib"]
sources = ["vendor/mylib.c"]
"#;
        fs::write(dir.path().join("vais.toml"), toml_content).unwrap();

        let manifest = load_manifest(dir.path()).unwrap();
        assert_eq!(manifest.native_dependencies.len(), 2);

        let ssl = &manifest.native_dependencies["openssl"];
        assert_eq!(ssl.lib_flags(), vec!["-lssl", "-lcrypto"]);
        assert_eq!(
            ssl.include_flag(),
            Some("-I/usr/include/openssl".to_string())
        );
        assert_eq!(ssl.lib_path_flag(), Some("-L/usr/lib".to_string()));

        let custom = &manifest.native_dependencies["custom"];
        assert_eq!(custom.lib_flags(), vec!["-lmylib"]);
        assert_eq!(custom.source_files(), &["vendor/mylib.c"]);
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

    #[test]
    fn test_workspace_manifest_parsing() {
        let dir = tempdir().unwrap();
        let toml_content = r#"
[workspace]
members = ["crates/*"]

[workspace.dependencies]
json = "1.0.0"

[package]
name = "workspace-root"
version = "0.1.0"
"#;
        fs::write(dir.path().join("vais.toml"), toml_content).unwrap();

        let manifest = load_manifest(dir.path()).unwrap();
        assert!(manifest.workspace.is_some());

        let ws = manifest.workspace.as_ref().unwrap();
        assert_eq!(ws.members, vec!["crates/*"]);
        assert!(ws.dependencies.contains_key("json"));
    }

    #[test]
    fn test_find_workspace_root() {
        let root = tempdir().unwrap();

        // Create workspace root
        let ws_toml = r#"
[workspace]
members = ["crates/*"]

[package]
name = "ws-root"
version = "0.1.0"
"#;
        fs::write(root.path().join("vais.toml"), ws_toml).unwrap();

        // Create a member directory
        let member_dir = root.path().join("crates").join("my-lib");
        fs::create_dir_all(&member_dir).unwrap();
        let member_toml = r#"
[package]
name = "my-lib"
version = "0.1.0"
"#;
        fs::write(member_dir.join("vais.toml"), member_toml).unwrap();

        // Should find workspace root from member directory
        let found = find_workspace_root(&member_dir);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), root.path().to_path_buf());
    }

    #[test]
    fn test_resolve_workspace_members() {
        let root = tempdir().unwrap();

        // Create workspace root
        let ws_toml = r#"
[workspace]
members = ["crates/*"]
"#;
        fs::write(root.path().join("vais.toml"), ws_toml).unwrap();

        // Create two member packages
        for name in &["lib-a", "lib-b"] {
            let dir = root.path().join("crates").join(name);
            fs::create_dir_all(dir.join("src")).unwrap();
            let toml = format!(
                r#"[package]
name = "{}"
version = "0.1.0"
"#,
                name
            );
            fs::write(dir.join("vais.toml"), toml).unwrap();
            fs::write(
                dir.join("src/lib.vais"),
                format!("# {}\nF greet() -> i64 {{ 0 }}\n", name),
            )
            .unwrap();
        }

        let workspace = resolve_workspace_members(root.path()).unwrap();
        assert_eq!(workspace.members.len(), 2);

        let names: Vec<&str> = workspace
            .members
            .iter()
            .map(|m| m.manifest.package.name.as_str())
            .collect();
        assert!(names.contains(&"lib-a"));
        assert!(names.contains(&"lib-b"));
    }

    #[test]
    fn test_workspace_dependency_resolution() {
        let root = tempdir().unwrap();

        // Create workspace root with shared dependency version
        let ws_toml = r#"
[workspace]
members = ["crates/*"]

[workspace.dependencies]
json = "2.0.0"
"#;
        fs::write(root.path().join("vais.toml"), ws_toml).unwrap();

        // Create a member that uses workspace = true
        let dir = root.path().join("crates").join("my-app");
        fs::create_dir_all(dir.join("src")).unwrap();
        let member_toml = r#"
[package]
name = "my-app"
version = "0.1.0"

[dependencies]
json = { workspace = true }
"#;
        fs::write(dir.join("vais.toml"), member_toml).unwrap();
        fs::write(dir.join("src/main.vais"), "F main() -> i64 { 0 }\n").unwrap();

        let workspace = resolve_workspace_members(root.path()).unwrap();
        assert_eq!(workspace.members.len(), 1);

        // The dependency should be resolved from workspace
        let member = &workspace.members[0];
        let json_dep = member.manifest.dependencies.get("json").unwrap();
        match json_dep {
            Dependency::Version(v) => assert_eq!(v, "2.0.0"),
            _ => panic!("expected Version dependency, got {:?}", json_dep),
        }
    }

    #[test]
    fn test_inter_workspace_path_deps() {
        let root = tempdir().unwrap();

        // Create workspace root
        let ws_toml = r#"
[workspace]
members = ["crates/*"]
"#;
        fs::write(root.path().join("vais.toml"), ws_toml).unwrap();

        // Create lib-core
        let core_dir = root.path().join("crates").join("lib-core");
        fs::create_dir_all(core_dir.join("src")).unwrap();
        fs::write(
            core_dir.join("vais.toml"),
            r#"[package]
name = "lib-core"
version = "0.1.0"
"#,
        )
        .unwrap();
        fs::write(core_dir.join("src/lib.vais"), "F core_fn() -> i64 { 42 }\n").unwrap();

        // Create my-app that depends on lib-core (by name, no path)
        let app_dir = root.path().join("crates").join("my-app");
        fs::create_dir_all(app_dir.join("src")).unwrap();
        fs::write(
            app_dir.join("vais.toml"),
            r#"[package]
name = "my-app"
version = "0.1.0"

[dependencies]
lib-core = "0.1.0"
"#,
        )
        .unwrap();
        fs::write(app_dir.join("src/main.vais"), "F main() -> i64 { 0 }\n").unwrap();

        let mut workspace = resolve_workspace_members(root.path()).unwrap();
        resolve_inter_workspace_deps(&mut workspace);

        // Find my-app member
        let app_member = workspace
            .members
            .iter()
            .find(|m| m.manifest.package.name == "my-app")
            .unwrap();

        // lib-core dependency should now have a path
        let dep = app_member.manifest.dependencies.get("lib-core").unwrap();
        match dep {
            Dependency::Detailed(d) => {
                assert!(
                    d.path.is_some(),
                    "expected path to be set for workspace member dep"
                );
                let path = d.path.as_ref().unwrap();
                assert!(
                    path.contains("lib-core"),
                    "path should reference lib-core: {}",
                    path
                );
            }
            _ => panic!("expected Detailed dependency with path"),
        }
    }

    #[test]
    fn test_feature_config_parsing() {
        let dir = tempdir().unwrap();
        let toml_content = r#"
[package]
name = "my-pkg"
version = "1.0.0"

[features]
default = ["json"]
json = []
async = ["json"]
full = ["json", "async"]
"#;
        fs::write(dir.path().join("vais.toml"), toml_content).unwrap();

        let manifest = load_manifest(dir.path()).unwrap();
        assert!(manifest.features.is_some());

        let fc = manifest.features.as_ref().unwrap();
        assert_eq!(fc.default, vec!["json"]);
        assert!(fc.features.contains_key("json"));
        assert!(fc.features.contains_key("async"));
        assert!(fc.features.contains_key("full"));
    }

    #[test]
    fn test_feature_resolve_defaults() {
        let fc = FeatureConfig {
            default: vec!["json".to_string()],
            features: {
                let mut m = HashMap::new();
                m.insert("json".to_string(), vec![]);
                m.insert("async".to_string(), vec!["json".to_string()]);
                m
            },
        };

        // With defaults
        let resolved = fc.resolve_features(&[], true);
        assert!(resolved.contains(&"json".to_string()));
        assert!(!resolved.contains(&"async".to_string()));

        // Without defaults
        let resolved = fc.resolve_features(&[], false);
        assert!(resolved.is_empty());
    }

    #[test]
    fn test_feature_resolve_transitive() {
        let fc = FeatureConfig {
            default: vec![],
            features: {
                let mut m = HashMap::new();
                m.insert("json".to_string(), vec![]);
                m.insert("async".to_string(), vec!["json".to_string()]);
                m.insert(
                    "full".to_string(),
                    vec!["json".to_string(), "async".to_string()],
                );
                m
            },
        };

        // Selecting "async" should also enable "json"
        let resolved = fc.resolve_features(&["async".to_string()], false);
        assert!(resolved.contains(&"async".to_string()));
        assert!(resolved.contains(&"json".to_string()));

        // Selecting "full" should enable all
        let resolved = fc.resolve_features(&["full".to_string()], false);
        assert_eq!(resolved.len(), 3);
    }

    #[test]
    fn test_feature_all_features() {
        let fc = FeatureConfig {
            default: vec!["json".to_string()],
            features: {
                let mut m = HashMap::new();
                m.insert("json".to_string(), vec![]);
                m.insert("async".to_string(), vec![]);
                m.insert("full".to_string(), vec![]);
                m
            },
        };

        let all = fc.all_features();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&"async".to_string()));
        assert!(all.contains(&"full".to_string()));
        assert!(all.contains(&"json".to_string()));
    }
}

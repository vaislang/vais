//! Dependency resolution logic

use super::*;
use std::fs;
use std::path::{Path, PathBuf};

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
                let path = d
                    .path
                    .as_ref()
                    .expect("path is Some - checked in match guard");
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
                    dependencies: std::collections::HashMap::new(),
                    dev_dependencies: std::collections::HashMap::new(),
                    native_dependencies: std::collections::HashMap::new(),
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

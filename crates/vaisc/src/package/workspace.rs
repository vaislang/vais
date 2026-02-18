//! Workspace management and resolution

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
pub struct ResolvedWorkspace {
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
        // safe: fallback to original path if canonicalization fails
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

    Ok(ResolvedWorkspace { members })
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
    // safe: fallback to original paths if canonicalization fails
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

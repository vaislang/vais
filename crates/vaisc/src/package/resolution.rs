//! Dependency resolution logic

use super::*;
use std::collections::HashMap;
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

    // Scan available versions in the cache directory, picking the best match
    // that satisfies the version requirement
    if let Ok(entries) = fs::read_dir(&pkg_cache_dir) {
        let mut best: Option<(String, PathBuf)> = None;
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                let extracted = entry_path.join("extracted");
                if extracted.exists() {
                    let dir_name = entry_path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    // Use the semver module to check if this version satisfies the requirement
                    if super::semver::version_satisfies(dir_name, stripped) {
                        if let Some((ref best_ver, _)) = best {
                            if super::semver::compare_versions(dir_name, best_ver)
                                == Some(std::cmp::Ordering::Greater)
                            {
                                best = Some((dir_name.to_string(), extracted));
                            }
                        } else {
                            best = Some((dir_name.to_string(), extracted));
                        }
                    }
                }
            }
        }
        if let Some((_, path)) = best {
            return Some(path);
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

/// Build a dependency graph from a manifest and detect all cycles.
///
/// Returns a list of all cycles found, where each cycle is a vector
/// of package names forming the cycle. An empty result means no cycles.
#[allow(dead_code)] // Public API reserved for `vaisc build --check-cycles` integration
pub fn detect_all_cycles(
    manifest: &PackageManifest,
    base_dir: &Path,
    cache_root: Option<&Path>,
) -> Vec<Vec<String>> {
    let graph = build_dependency_graph(manifest, base_dir, cache_root);
    find_all_cycles(&graph)
}

/// Build an adjacency list representation of the dependency graph.
///
/// Each key is a package name, and the value is a list of its direct
/// dependency names.
#[allow(dead_code)] // Used by detect_all_cycles (reserved public API)
fn build_dependency_graph(
    manifest: &PackageManifest,
    base_dir: &Path,
    cache_root: Option<&Path>,
) -> HashMap<String, Vec<String>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    let mut visited = std::collections::HashSet::new();

    fn collect_edges(
        manifest: &PackageManifest,
        base_dir: &Path,
        cache_root: Option<&Path>,
        graph: &mut HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
        pkg_name: &str,
    ) {
        if visited.contains(pkg_name) {
            return;
        }
        visited.insert(pkg_name.to_string());

        let deps: Vec<String> = manifest.dependencies.keys().cloned().collect();
        graph.insert(pkg_name.to_string(), deps.clone());

        for dep_name in &deps {
            let dep = &manifest.dependencies[dep_name];
            let dep_path = match dep {
                Dependency::Detailed(d) if d.path.is_some() => {
                    // safe: guard `d.path.is_some()` ensures this branch only matches Some
                    let path = d.path.as_ref().expect("guarded by is_some()");
                    Some(base_dir.join(path))
                }
                Dependency::Version(v) => cache_root
                    .and_then(|c| find_cached_registry_dep(c, dep_name, v)),
                Dependency::Detailed(d) => {
                    let version = d.version.as_deref().unwrap_or("*");
                    cache_root
                        .and_then(|c| find_cached_registry_dep(c, dep_name, version))
                }
            };

            if let Some(path) = dep_path {
                if let Ok(canonical) = path.canonicalize() {
                    if let Ok(dep_manifest) = load_manifest(&canonical) {
                        collect_edges(
                            &dep_manifest,
                            &canonical,
                            cache_root,
                            graph,
                            visited,
                            dep_name,
                        );
                    }
                }
            }
        }
    }

    let root_name = &manifest.package.name;
    collect_edges(
        manifest,
        base_dir,
        cache_root,
        &mut graph,
        &mut visited,
        root_name,
    );

    graph
}

/// Find all cycles in a directed graph using DFS.
///
/// Uses Tarjan-inspired coloring: white (unvisited), gray (in progress), black (done).
#[allow(dead_code)] // Used by detect_all_cycles (reserved public API)
fn find_all_cycles(graph: &HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        White,
        Gray,
        Black,
    }

    let mut colors: HashMap<String, Color> = HashMap::new();
    let mut cycles = Vec::new();
    let mut stack = Vec::new();

    for node in graph.keys() {
        colors.insert(node.clone(), Color::White);
    }

    fn dfs(
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        colors: &mut HashMap<String, Color>,
        stack: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        if let Some(&Color::Black) = colors.get(node) {
            return;
        }
        if let Some(&Color::Gray) = colors.get(node) {
            // Found a cycle: extract it from the stack
            if let Some(pos) = stack.iter().position(|n| n == node) {
                let mut cycle: Vec<String> = stack[pos..].to_vec();
                cycle.push(node.to_string());
                cycles.push(cycle);
            }
            return;
        }

        colors.insert(node.to_string(), Color::Gray);
        stack.push(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                dfs(neighbor, graph, colors, stack, cycles);
            }
        }

        stack.pop();
        colors.insert(node.to_string(), Color::Black);
    }

    for node in graph.keys() {
        if colors.get(node) == Some(&Color::White) {
            dfs(node, graph, &mut colors, &mut stack, &mut cycles);
        }
    }

    cycles
}

/// Format cycle information for user-friendly error messages.
#[allow(dead_code)] // Public API reserved for `vaisc build --check-cycles` integration
pub fn format_cycles(cycles: &[Vec<String>]) -> String {
    if cycles.is_empty() {
        return "No dependency cycles detected.".to_string();
    }

    let mut result = format!("Found {} dependency cycle(s):\n", cycles.len());
    for (i, cycle) in cycles.iter().enumerate() {
        result.push_str(&format!("  {}. {}\n", i + 1, cycle.join(" -> ")));
    }
    result
}

#[cfg(test)]
mod cycle_tests {
    use super::*;

    #[test]
    fn test_no_cycles() {
        let mut graph = HashMap::new();
        graph.insert("a".to_string(), vec!["b".to_string()]);
        graph.insert("b".to_string(), vec!["c".to_string()]);
        graph.insert("c".to_string(), vec![]);

        let cycles = find_all_cycles(&graph);
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_simple_cycle() {
        let mut graph = HashMap::new();
        graph.insert("a".to_string(), vec!["b".to_string()]);
        graph.insert("b".to_string(), vec!["a".to_string()]);

        let cycles = find_all_cycles(&graph);
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_self_cycle() {
        let mut graph = HashMap::new();
        graph.insert("a".to_string(), vec!["a".to_string()]);

        let cycles = find_all_cycles(&graph);
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_three_node_cycle() {
        let mut graph = HashMap::new();
        graph.insert("a".to_string(), vec!["b".to_string()]);
        graph.insert("b".to_string(), vec!["c".to_string()]);
        graph.insert("c".to_string(), vec!["a".to_string()]);

        let cycles = find_all_cycles(&graph);
        assert!(!cycles.is_empty());
        // The cycle should contain a, b, c
        let cycle = &cycles[0];
        assert!(cycle.contains(&"a".to_string()));
        assert!(cycle.contains(&"b".to_string()));
        assert!(cycle.contains(&"c".to_string()));
    }

    #[test]
    fn test_diamond_no_cycle() {
        let mut graph = HashMap::new();
        graph.insert("a".to_string(), vec!["b".to_string(), "c".to_string()]);
        graph.insert("b".to_string(), vec!["d".to_string()]);
        graph.insert("c".to_string(), vec!["d".to_string()]);
        graph.insert("d".to_string(), vec![]);

        let cycles = find_all_cycles(&graph);
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_format_cycles_empty() {
        let result = format_cycles(&[]);
        assert!(result.contains("No dependency cycles"));
    }

    #[test]
    fn test_format_cycles_one() {
        let cycles = vec![vec![
            "a".to_string(),
            "b".to_string(),
            "a".to_string(),
        ]];
        let result = format_cycles(&cycles);
        assert!(result.contains("1 dependency cycle"));
        assert!(result.contains("a -> b -> a"));
    }

    #[test]
    fn test_empty_graph() {
        let graph: HashMap<String, Vec<String>> = HashMap::new();
        let cycles = find_all_cycles(&graph);
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_isolated_nodes() {
        let mut graph = HashMap::new();
        graph.insert("a".to_string(), vec![]);
        graph.insert("b".to_string(), vec![]);
        graph.insert("c".to_string(), vec![]);

        let cycles = find_all_cycles(&graph);
        assert!(cycles.is_empty());
    }
}

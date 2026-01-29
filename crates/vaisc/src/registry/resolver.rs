//! Dependency resolution algorithm
//!
//! Implements a simple dependency resolver with:
//! - Transitive dependency resolution
//! - Version conflict detection
//! - Lock file generation

use super::client::RegistryClient;
use super::error::{RegistryError, RegistryResult};
use super::lockfile::{LockFile, LockedPackage};
use super::version::{Version, VersionReq};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

/// A resolved package ready for installation
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: Version,
    pub checksum: String,
    pub source: String,
    pub dependencies: Vec<String>,
    pub path: Option<PathBuf>,
}

/// Dependency resolver
pub struct DependencyResolver<'a> {
    client: &'a RegistryClient,
    lock: Option<&'a LockFile>,
    resolved: HashMap<String, ResolvedPackage>,
    pending: VecDeque<(String, VersionReq)>,
    visiting: HashSet<String>,
}

impl<'a> DependencyResolver<'a> {
    /// Create a new resolver
    pub fn new(client: &'a RegistryClient) -> Self {
        Self {
            client,
            lock: None,
            resolved: HashMap::new(),
            pending: VecDeque::new(),
            visiting: HashSet::new(),
        }
    }

    /// Use an existing lock file for resolution
    pub fn with_lock(mut self, lock: &'a LockFile) -> Self {
        self.lock = Some(lock);
        self
    }

    /// Add a root dependency to resolve
    pub fn add(&mut self, name: &str, req: &str) -> RegistryResult<()> {
        let version_req = VersionReq::parse(req)?;
        self.pending.push_back((name.to_string(), version_req));
        Ok(())
    }

    /// Resolve all dependencies
    pub fn resolve(&mut self) -> RegistryResult<Vec<ResolvedPackage>> {
        while let Some((name, req)) = self.pending.pop_front() {
            // Skip if already resolved
            if self.resolved.contains_key(&name) {
                // Check version compatibility
                let resolved = &self.resolved[&name];
                if !req.matches(&resolved.version) {
                    return Err(RegistryError::ResolutionError {
                        message: format!(
                            "version conflict: {} requires {}, but {} is already resolved",
                            name, req, resolved.version
                        ),
                    });
                }
                continue;
            }

            // Check for cycles
            if self.visiting.contains(&name) {
                return Err(RegistryError::CyclicDependency {
                    cycle: format!("{} -> ...", name),
                });
            }
            self.visiting.insert(name.clone());

            // Try lock file first
            let resolved = if let Some(lock) = self.lock {
                if let Some(locked) = lock.get(&name) {
                    if req.matches(&locked.version) {
                        self.resolve_from_lock(&name, locked)?
                    } else {
                        self.resolve_from_registry(&name, &req)?
                    }
                } else {
                    self.resolve_from_registry(&name, &req)?
                }
            } else {
                self.resolve_from_registry(&name, &req)?
            };

            // Queue transitive dependencies
            for dep_name in &resolved.dependencies {
                if !self.resolved.contains_key(dep_name) {
                    // Get dependency version requirement
                    let pkg = self.client.get_package(&name)?;
                    let entry = pkg.get_version(&resolved.version).ok_or_else(|| {
                        RegistryError::VersionNotFound {
                            name: name.clone(),
                            version: resolved.version.to_string(),
                        }
                    })?;

                    if let Some(dep_info) = entry.dependencies.get(dep_name) {
                        let dep_req = VersionReq::parse(&dep_info.req)?;
                        self.pending.push_back((dep_name.clone(), dep_req));
                    }
                }
            }

            self.visiting.remove(&name);
            self.resolved.insert(name, resolved);
        }

        // Return in dependency order (dependencies before dependents)
        self.topological_sort()
    }

    /// Resolve from locked version
    fn resolve_from_lock(&self, name: &str, locked: &LockedPackage) -> RegistryResult<ResolvedPackage> {
        let path = self.client.installed_path(name, &locked.version);

        Ok(ResolvedPackage {
            name: name.to_string(),
            version: locked.version.clone(),
            checksum: locked.checksum.clone(),
            source: locked.source.clone(),
            dependencies: locked.dependencies.clone(),
            path,
        })
    }

    /// Resolve from registry
    fn resolve_from_registry(&self, name: &str, req: &VersionReq) -> RegistryResult<ResolvedPackage> {
        let entry = self.client.find_version(name, req)?;
        let path = self.client.installed_path(name, &entry.version);

        let deps: Vec<String> = entry.dependencies.keys().cloned().collect();

        Ok(ResolvedPackage {
            name: name.to_string(),
            version: entry.version.clone(),
            checksum: entry.checksum.clone(),
            source: "registry".to_string(),
            dependencies: deps,
            path,
        })
    }

    /// Topological sort of resolved packages
    fn topological_sort(&self) -> RegistryResult<Vec<ResolvedPackage>> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp = HashSet::new();

        for name in self.resolved.keys() {
            self.visit_topo(name, &mut visited, &mut temp, &mut result)?;
        }

        Ok(result)
    }

    fn visit_topo(
        &self,
        name: &str,
        visited: &mut HashSet<String>,
        temp: &mut HashSet<String>,
        result: &mut Vec<ResolvedPackage>,
    ) -> RegistryResult<()> {
        if visited.contains(name) {
            return Ok(());
        }
        if temp.contains(name) {
            return Err(RegistryError::CyclicDependency {
                cycle: name.to_string(),
            });
        }

        temp.insert(name.to_string());

        if let Some(pkg) = self.resolved.get(name) {
            for dep in &pkg.dependencies {
                self.visit_topo(dep, visited, temp, result)?;
            }
            visited.insert(name.to_string());
            result.push(pkg.clone());
        }

        temp.remove(name);
        Ok(())
    }

    /// Generate a lock file from resolved packages
    pub fn generate_lock(&self) -> LockFile {
        let mut lock = LockFile::new();

        for (name, pkg) in &self.resolved {
            lock.insert(
                name.clone(),
                LockedPackage {
                    version: pkg.version.clone(),
                    checksum: pkg.checksum.clone(),
                    source: pkg.source.clone(),
                    dependencies: pkg.dependencies.clone(),
                },
            );
        }

        lock
    }
}

/// Simple resolution without full resolver (for testing)
pub fn resolve_simple(
    client: &RegistryClient,
    deps: &[(String, String)],
) -> RegistryResult<Vec<ResolvedPackage>> {
    let mut resolver = DependencyResolver::new(client);
    for (name, req) in deps {
        resolver.add(name, req)?;
    }
    resolver.resolve()
}

/// Install all resolved packages
pub fn install_resolved(
    client: &RegistryClient,
    packages: &[ResolvedPackage],
) -> RegistryResult<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for pkg in packages {
        if let Some(ref path) = pkg.path {
            paths.push(path.clone());
        } else {
            let installed = client.download(&pkg.name, &pkg.version)?;
            paths.push(installed);
        }
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_req_parsing() {
        let req = VersionReq::parse("^1.0.0").unwrap();
        assert!(req.matches(&Version::new(1, 0, 0)));
        assert!(req.matches(&Version::new(1, 5, 0)));
        assert!(!req.matches(&Version::new(2, 0, 0)));
    }

    #[test]
    fn test_topological_sort_simple() {
        // This test verifies the concept - actual testing requires a mock client
        let v = Version::new(1, 0, 0);
        let pkg_a = ResolvedPackage {
            name: "a".to_string(),
            version: v.clone(),
            checksum: "".to_string(),
            source: "test".to_string(),
            dependencies: vec!["b".to_string()],
            path: None,
        };
        let pkg_b = ResolvedPackage {
            name: "b".to_string(),
            version: v,
            checksum: "".to_string(),
            source: "test".to_string(),
            dependencies: vec![],
            path: None,
        };

        // b should come before a in sorted order
        let mut packages = vec![pkg_a.clone(), pkg_b.clone()];
        packages.sort_by(|a, b| {
            if a.dependencies.contains(&b.name) {
                std::cmp::Ordering::Greater
            } else if b.dependencies.contains(&a.name) {
                std::cmp::Ordering::Less
            } else {
                a.name.cmp(&b.name)
            }
        });

        assert_eq!(packages[0].name, "b");
        assert_eq!(packages[1].name, "a");
    }
}

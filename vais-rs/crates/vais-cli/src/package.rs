//! Vais Package Manager (APM)
//!
//! Manages Vais projects and dependencies via vais.toml

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Package manifest (vais.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub package: PackageInfo,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    pub dev_dependencies: HashMap<String, Dependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default = "default_entry")]
    pub entry: String,
}

fn default_entry() -> String {
    "src/main.vais".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    /// Simple version string: "1.0.0"
    Version(String),
    /// Detailed dependency with path or git
    Detailed(DetailedDependency),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedDependency {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub git: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
}

impl Manifest {
    /// Load manifest from vais.toml
    pub fn load(path: &Path) -> Result<Self, String> {
        let manifest_path = if path.is_dir() {
            path.join("vais.toml")
        } else {
            path.to_path_buf()
        };

        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read {}: {}", manifest_path.display(), e))?;

        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", manifest_path.display(), e))
    }

    /// Save manifest to vais.toml
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let manifest_path = if path.is_dir() {
            path.join("vais.toml")
        } else {
            path.to_path_buf()
        };

        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

        fs::write(&manifest_path, content)
            .map_err(|e| format!("Failed to write {}: {}", manifest_path.display(), e))
    }

    /// Create a new default manifest
    pub fn new(name: &str) -> Self {
        Self {
            package: PackageInfo {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: String::new(),
                authors: Vec::new(),
                license: Some("MIT".to_string()),
                repository: None,
                keywords: Vec::new(),
                entry: "src/main.vais".to_string(),
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
        }
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, name: &str, version: &str, dev: bool) {
        let dep = Dependency::Version(version.to_string());
        if dev {
            self.dev_dependencies.insert(name.to_string(), dep);
        } else {
            self.dependencies.insert(name.to_string(), dep);
        }
    }

    /// Remove a dependency
    pub fn remove_dependency(&mut self, name: &str) -> bool {
        self.dependencies.remove(name).is_some() || self.dev_dependencies.remove(name).is_some()
    }
}

/// Initialize a new Vais project
pub fn init_project(path: &Path, name: Option<&str>) -> Result<(), String> {
    let project_name = name
        .map(|s| s.to_string())
        .or_else(|| path.file_name().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| "my-project".to_string());

    // Create directories
    let src_dir = path.join("src");
    fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Failed to create src directory: {}", e))?;

    // Create vais.toml
    let manifest = Manifest::new(&project_name);
    manifest.save(path)?;

    // Create src/main.vais
    let main_content = format!(r#"// Vais Project: {}
// Entry point

greet(name) = print("Hello from " + name + "!")

greet("{}")
"#, &project_name, &project_name);

    fs::write(src_dir.join("main.vais"), main_content)
        .map_err(|e| format!("Failed to create main.vais: {}", e))?;

    // Create .gitignore
    let gitignore = r#"# Build outputs
/target/
*.o
*.c
*.ll
*.wat
*.wasm

# Editor files
.vscode/
.idea/
*.swp
*~

# OS files
.DS_Store
Thumbs.db
"#;

    fs::write(path.join(".gitignore"), gitignore)
        .map_err(|e| format!("Failed to create .gitignore: {}", e))?;

    Ok(())
}

/// Find the project root (directory containing vais.toml)
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join("vais.toml").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Package registry (local for now)
pub struct Registry {
    pub path: PathBuf,
}

/// Package metadata stored in registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMeta {
    pub name: String,
    pub versions: Vec<VersionInfo>,
    pub description: String,
    pub keywords: Vec<String>,
    pub downloads: u64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub checksum: Option<String>,
    pub dependencies: HashMap<String, String>,
    pub published_at: String,
}

#[allow(dead_code)]
impl Registry {
    /// Create or open local registry
    pub fn local() -> Result<Self, String> {
        let home = dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?;
        let registry_path = home.join(".vais").join("registry");

        fs::create_dir_all(&registry_path)
            .map_err(|e| format!("Failed to create registry: {}", e))?;

        Ok(Self {
            path: registry_path,
        })
    }

    /// List available packages
    pub fn list_packages(&self) -> Result<Vec<String>, String> {
        let mut packages = Vec::new();

        if self.path.exists() {
            for entry in fs::read_dir(&self.path)
                .map_err(|e| format!("Failed to read registry: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        packages.push(name.to_string());
                    }
                }
            }
        }

        Ok(packages)
    }

    /// List packages with metadata
    pub fn list_packages_detailed(&self) -> Result<Vec<PackageMeta>, String> {
        let mut packages = Vec::new();

        if self.path.exists() {
            for entry in fs::read_dir(&self.path)
                .map_err(|e| format!("Failed to read registry: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let pkg_path = entry.path();

                if pkg_path.is_dir() {
                    if let Some(meta) = self.get_package_meta(&pkg_path) {
                        packages.push(meta);
                    }
                }
            }
        }

        Ok(packages)
    }

    /// Get package metadata
    fn get_package_meta(&self, pkg_path: &Path) -> Option<PackageMeta> {
        let name = pkg_path.file_name()?.to_str()?.to_string();
        let meta_path = pkg_path.join("meta.json");

        if meta_path.exists() {
            let content = fs::read_to_string(&meta_path).ok()?;
            serde_json::from_str(&content).ok()
        } else {
            // Build metadata from versions
            let versions = self.list_versions(&name).ok()?;
            Some(PackageMeta {
                name: name.clone(),
                versions: versions.iter().map(|v| VersionInfo {
                    version: v.clone(),
                    checksum: None,
                    dependencies: HashMap::new(),
                    published_at: String::new(),
                }).collect(),
                description: String::new(),
                keywords: Vec::new(),
                downloads: 0,
                created_at: String::new(),
                updated_at: String::new(),
            })
        }
    }

    /// List all versions of a package
    pub fn list_versions(&self, name: &str) -> Result<Vec<String>, String> {
        let pkg_path = self.path.join(name);
        let mut versions = Vec::new();

        if pkg_path.exists() {
            for entry in fs::read_dir(&pkg_path)
                .map_err(|e| format!("Failed to read package: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                if entry.path().is_dir() {
                    if let Some(ver) = entry.file_name().to_str() {
                        if ver != "meta.json" {
                            versions.push(ver.to_string());
                        }
                    }
                }
            }
        }

        // Sort versions (semver)
        versions.sort_by(|a, b| compare_versions(b, a));
        Ok(versions)
    }

    /// Get latest version of a package
    pub fn get_latest_version(&self, name: &str) -> Result<Option<String>, String> {
        let versions = self.list_versions(name)?;
        Ok(versions.into_iter().next())
    }

    /// Search packages by keyword
    pub fn search(&self, query: &str) -> Result<Vec<PackageMeta>, String> {
        let all_packages = self.list_packages_detailed()?;
        let query_lower = query.to_lowercase();

        Ok(all_packages
            .into_iter()
            .filter(|pkg| {
                pkg.name.to_lowercase().contains(&query_lower)
                    || pkg.description.to_lowercase().contains(&query_lower)
                    || pkg.keywords.iter().any(|k| k.to_lowercase().contains(&query_lower))
            })
            .collect())
    }

    /// Get package path
    pub fn get_package_path(&self, name: &str, version: &str) -> PathBuf {
        self.path.join(name).join(version)
    }

    /// Check if package exists
    pub fn has_package(&self, name: &str, version: &str) -> bool {
        self.get_package_path(name, version).exists()
    }

    /// Resolve dependency version (supports *, ^, ~, ranges)
    pub fn resolve_version(&self, name: &str, version_req: &str) -> Result<Option<String>, String> {
        let versions = self.list_versions(name)?;

        if version_req == "*" || version_req == "latest" {
            return Ok(versions.into_iter().next());
        }

        // Handle caret (^) - compatible versions
        if let Some(base) = version_req.strip_prefix('^') {
            let base_parts = parse_version(base);
            for ver in &versions {
                let ver_parts = parse_version(ver);
                if is_caret_compatible(&base_parts, &ver_parts) {
                    return Ok(Some(ver.clone()));
                }
            }
            return Ok(None);
        }

        // Handle tilde (~) - patch updates only
        if let Some(base) = version_req.strip_prefix('~') {
            let base_parts = parse_version(base);
            for ver in &versions {
                let ver_parts = parse_version(ver);
                if is_tilde_compatible(&base_parts, &ver_parts) {
                    return Ok(Some(ver.clone()));
                }
            }
            return Ok(None);
        }

        // Handle >= and <=
        if let Some(min) = version_req.strip_prefix(">=") {
            for ver in &versions {
                if compare_versions(ver, min.trim()) >= std::cmp::Ordering::Equal {
                    return Ok(Some(ver.clone()));
                }
            }
            return Ok(None);
        }

        // Exact version
        if versions.contains(&version_req.to_string()) {
            return Ok(Some(version_req.to_string()));
        }

        Ok(None)
    }

    /// Install package from path (for local development)
    pub fn install_from_path(&self, name: &str, version: &str, src: &Path) -> Result<(), String> {
        let dest = self.get_package_path(name, version);
        fs::create_dir_all(&dest)
            .map_err(|e| format!("Failed to create package directory: {}", e))?;

        // Copy all .vais files
        copy_vais_files(src, &dest)?;

        // Copy manifest if exists
        let manifest_src = src.join("vais.toml");
        if manifest_src.exists() {
            fs::copy(&manifest_src, dest.join("vais.toml"))
                .map_err(|e| format!("Failed to copy manifest: {}", e))?;
        }

        // Update package metadata
        self.update_package_meta(name, version, src)?;

        Ok(())
    }

    /// Update package metadata
    fn update_package_meta(&self, name: &str, version: &str, src: &Path) -> Result<(), String> {
        let pkg_path = self.path.join(name);
        let meta_path = pkg_path.join("meta.json");

        let mut meta = if meta_path.exists() {
            let content = fs::read_to_string(&meta_path)
                .map_err(|e| format!("Failed to read meta: {}", e))?;
            serde_json::from_str(&content).unwrap_or_else(|_| PackageMeta {
                name: name.to_string(),
                versions: Vec::new(),
                description: String::new(),
                keywords: Vec::new(),
                downloads: 0,
                created_at: chrono_now(),
                updated_at: chrono_now(),
            })
        } else {
            PackageMeta {
                name: name.to_string(),
                versions: Vec::new(),
                description: String::new(),
                keywords: Vec::new(),
                downloads: 0,
                created_at: chrono_now(),
                updated_at: chrono_now(),
            }
        };

        // Load manifest if available
        let manifest_path = src.join("vais.toml");
        if manifest_path.exists() {
            if let Ok(manifest) = Manifest::load(src) {
                meta.description = manifest.package.description;
                meta.keywords = manifest.package.keywords;
            }
        }

        // Add version if not exists
        if !meta.versions.iter().any(|v| v.version == version) {
            meta.versions.push(VersionInfo {
                version: version.to_string(),
                checksum: None,
                dependencies: HashMap::new(),
                published_at: chrono_now(),
            });
        }

        meta.updated_at = chrono_now();

        let content = serde_json::to_string_pretty(&meta)
            .map_err(|e| format!("Failed to serialize meta: {}", e))?;
        fs::write(&meta_path, content)
            .map_err(|e| format!("Failed to write meta: {}", e))?;

        Ok(())
    }

    /// Resolve all dependencies for a package
    pub fn resolve_dependencies(&self, manifest: &Manifest) -> Result<Vec<LockedPackage>, String> {
        let mut resolved = Vec::new();
        let mut to_resolve: Vec<(String, String)> = manifest
            .dependencies
            .iter()
            .map(|(name, dep)| {
                let version = match dep {
                    Dependency::Version(v) => v.clone(),
                    Dependency::Detailed(d) => d.version.clone().unwrap_or_else(|| "*".to_string()),
                };
                (name.clone(), version)
            })
            .collect();

        while let Some((name, version_req)) = to_resolve.pop() {
            // Skip if already resolved
            if resolved.iter().any(|p: &LockedPackage| p.name == name) {
                continue;
            }

            // Resolve version
            let resolved_version = self
                .resolve_version(&name, &version_req)?
                .ok_or_else(|| format!("Could not resolve {} {}", name, version_req))?;

            // Load package manifest for transitive dependencies
            let pkg_path = self.get_package_path(&name, &resolved_version);
            if let Ok(pkg_manifest) = Manifest::load(&pkg_path) {
                for (dep_name, dep) in &pkg_manifest.dependencies {
                    let dep_version = match dep {
                        Dependency::Version(v) => v.clone(),
                        Dependency::Detailed(d) => d.version.clone().unwrap_or_else(|| "*".to_string()),
                    };
                    to_resolve.push((dep_name.clone(), dep_version));
                }
            }

            resolved.push(LockedPackage {
                name,
                version: resolved_version,
                checksum: None,
                source: Some("registry".to_string()),
            });
        }

        Ok(resolved)
    }
}

/// Parse version string into (major, minor, patch)
fn parse_version(v: &str) -> (u32, u32, u32) {
    let parts: Vec<u32> = v
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    )
}

/// Compare two version strings
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts = parse_version(a);
    let b_parts = parse_version(b);
    a_parts.cmp(&b_parts)
}

/// Check caret compatibility (^1.2.3)
#[allow(dead_code)]
fn is_caret_compatible(base: &(u32, u32, u32), ver: &(u32, u32, u32)) -> bool {
    if base.0 == 0 {
        // 0.x.y - minor must match
        ver.0 == base.0 && ver.1 == base.1 && ver.2 >= base.2
    } else {
        // x.y.z - major must match, version >= base
        ver.0 == base.0 && (ver.1, ver.2) >= (base.1, base.2)
    }
}

/// Check tilde compatibility (~1.2.3)
#[allow(dead_code)]
fn is_tilde_compatible(base: &(u32, u32, u32), ver: &(u32, u32, u32)) -> bool {
    ver.0 == base.0 && ver.1 == base.1 && ver.2 >= base.2
}

/// Get current timestamp
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

/// Copy .vais files recursively
fn copy_vais_files(src: &Path, dest: &Path) -> Result<(), String> {
    if src.is_dir() {
        for entry in fs::read_dir(src)
            .map_err(|e| format!("Failed to read directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                let name = path.file_name().unwrap();
                let new_dest = dest.join(name);
                fs::create_dir_all(&new_dest)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
                copy_vais_files(&path, &new_dest)?;
            } else if path.extension().is_some_and(|e| e == "vais") {
                let name = path.file_name().unwrap();
                fs::copy(&path, dest.join(name))
                    .map_err(|e| format!("Failed to copy file: {}", e))?;
            }
        }
    }
    Ok(())
}

/// Lock file for reproducible builds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    pub version: u32,
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub checksum: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

impl LockFile {
    pub fn new() -> Self {
        Self {
            version: 1,
            packages: Vec::new(),
        }
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let lock_path = if path.is_dir() {
            path.join("vais.lock")
        } else {
            path.to_path_buf()
        };

        if !lock_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&lock_path)
            .map_err(|e| format!("Failed to read lock file: {}", e))?;

        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse lock file: {}", e))
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let lock_path = if path.is_dir() {
            path.join("vais.lock")
        } else {
            path.to_path_buf()
        };

        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize lock file: {}", e))?;

        fs::write(&lock_path, content)
            .map_err(|e| format!("Failed to write lock file: {}", e))
    }
}

impl Default for LockFile {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_manifest_roundtrip() {
        let mut manifest = Manifest::new("test-project");
        manifest.add_dependency("utils", "1.0.0", false);
        manifest.add_dependency("test-helpers", "0.1.0", true);

        let toml_str = toml::to_string_pretty(&manifest).unwrap();
        let parsed: Manifest = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.package.name, "test-project");
        assert_eq!(parsed.package.version, "0.1.0");
        assert!(parsed.dependencies.contains_key("utils"));
        assert!(parsed.dev_dependencies.contains_key("test-helpers"));
    }

    #[test]
    fn test_init_project() {
        let temp_dir = env::temp_dir().join("vais-test-init");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        init_project(&temp_dir, Some("my-test-project")).unwrap();

        assert!(temp_dir.join("vais.toml").exists());
        assert!(temp_dir.join("src/main.vais").exists());
        assert!(temp_dir.join(".gitignore").exists());

        let manifest = Manifest::load(&temp_dir).unwrap();
        assert_eq!(manifest.package.name, "my-test-project");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_dependency_operations() {
        let mut manifest = Manifest::new("test");

        manifest.add_dependency("pkg-a", "1.0.0", false);
        assert!(manifest.dependencies.contains_key("pkg-a"));

        manifest.add_dependency("pkg-b", "2.0.0", true);
        assert!(manifest.dev_dependencies.contains_key("pkg-b"));

        assert!(manifest.remove_dependency("pkg-a"));
        assert!(!manifest.dependencies.contains_key("pkg-a"));

        assert!(!manifest.remove_dependency("nonexistent"));
    }
}

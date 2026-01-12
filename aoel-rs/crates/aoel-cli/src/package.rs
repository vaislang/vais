//! AOEL Package Manager (APM)
//!
//! Manages AOEL projects and dependencies via aoel.toml

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Package manifest (aoel.toml)
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
    "src/main.aoel".to_string()
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
    /// Load manifest from aoel.toml
    pub fn load(path: &Path) -> Result<Self, String> {
        let manifest_path = if path.is_dir() {
            path.join("aoel.toml")
        } else {
            path.to_path_buf()
        };

        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read {}: {}", manifest_path.display(), e))?;

        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", manifest_path.display(), e))
    }

    /// Save manifest to aoel.toml
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let manifest_path = if path.is_dir() {
            path.join("aoel.toml")
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
                entry: "src/main.aoel".to_string(),
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

/// Initialize a new AOEL project
pub fn init_project(path: &Path, name: Option<&str>) -> Result<(), String> {
    let project_name = name
        .map(|s| s.to_string())
        .or_else(|| path.file_name().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| "my-project".to_string());

    // Create directories
    let src_dir = path.join("src");
    fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Failed to create src directory: {}", e))?;

    // Create aoel.toml
    let manifest = Manifest::new(&project_name);
    manifest.save(path)?;

    // Create src/main.aoel
    let main_content = format!(r#"// AOEL Project: {}
// Entry point

greet(name) = print("Hello from " + name + "!")

greet("{}")
"#, &project_name, &project_name);

    fs::write(src_dir.join("main.aoel"), main_content)
        .map_err(|e| format!("Failed to create main.aoel: {}", e))?;

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

/// Find the project root (directory containing aoel.toml)
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join("aoel.toml").exists() {
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

impl Registry {
    /// Create or open local registry
    pub fn local() -> Result<Self, String> {
        let home = dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?;
        let registry_path = home.join(".aoel").join("registry");

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

    /// Get package path
    pub fn get_package_path(&self, name: &str, version: &str) -> PathBuf {
        self.path.join(name).join(version)
    }

    /// Check if package exists
    pub fn has_package(&self, name: &str, version: &str) -> bool {
        self.get_package_path(name, version).exists()
    }

    /// Install package from path (for local development)
    pub fn install_from_path(&self, name: &str, version: &str, src: &Path) -> Result<(), String> {
        let dest = self.get_package_path(name, version);
        fs::create_dir_all(&dest)
            .map_err(|e| format!("Failed to create package directory: {}", e))?;

        // Copy all .aoel files
        copy_aoel_files(src, &dest)?;

        // Copy manifest if exists
        let manifest_src = src.join("aoel.toml");
        if manifest_src.exists() {
            fs::copy(&manifest_src, dest.join("aoel.toml"))
                .map_err(|e| format!("Failed to copy manifest: {}", e))?;
        }

        Ok(())
    }
}

/// Copy .aoel files recursively
fn copy_aoel_files(src: &Path, dest: &Path) -> Result<(), String> {
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
                copy_aoel_files(&path, &new_dest)?;
            } else if path.extension().map_or(false, |e| e == "aoel") {
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
            path.join("aoel.lock")
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
            path.join("aoel.lock")
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
        let temp_dir = env::temp_dir().join("aoel-test-init");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        init_project(&temp_dir, Some("my-test-project")).unwrap();

        assert!(temp_dir.join("aoel.toml").exists());
        assert!(temp_dir.join("src/main.aoel").exists());
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

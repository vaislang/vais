//! Manifest loading, initialization, and dependency manipulation

use super::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
    // safe: fallback to "my-package" if no name provided and dir has no file name
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

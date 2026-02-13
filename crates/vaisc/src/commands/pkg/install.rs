//! Dependency management commands.

use crate::package;
use crate::registry;
use colored::Colorize;
use std::path::Path;

/// Add a dependency to vais.toml
pub(super) fn cmd_pkg_add(
    cwd: &Path,
    name: &str,
    path: Option<String>,
    version: Option<String>,
) -> Result<(), String> {
    let pkg_dir =
        package::find_manifest(cwd).ok_or_else(|| "could not find vais.toml".to_string())?;

    let manifest_path = pkg_dir.join("vais.toml");
    let is_registry_dep = path.is_none();

    package::add_dependency(&manifest_path, name, path.as_deref(), version.as_deref())
        .map_err(|e| e.to_string())?;

    println!("{} Added dependency '{}'", "✓".green(), name);

    if is_registry_dep {
        // Check if the package is already installed in the cache
        let cache_root = package::default_registry_cache_root();
        let version_str = version.as_deref().unwrap_or("*");
        let is_cached = cache_root
            .as_ref()
            .and_then(|root| package::find_cached_registry_dep(root, name, version_str))
            .is_some();

        if !is_cached {
            println!(
                "{} Run `vais pkg install` to download '{}' from the registry",
                "Note".yellow(),
                name
            );
        }
    }

    Ok(())
}

/// Remove a dependency from vais.toml
pub(super) fn cmd_pkg_remove(cwd: &Path, name: &str) -> Result<(), String> {
    let pkg_dir =
        package::find_manifest(cwd).ok_or_else(|| "could not find vais.toml".to_string())?;

    let manifest_path = pkg_dir.join("vais.toml");
    package::remove_dependency(&manifest_path, name).map_err(|e| e.to_string())?;

    println!("{} Removed dependency '{}'", "✓".green(), name);
    Ok(())
}

/// Install packages from registry
pub(super) fn cmd_pkg_install(
    cwd: &Path,
    packages: Vec<String>,
    update: bool,
    offline: bool,
    verbose: bool,
) -> Result<(), String> {
    use registry::{DependencyResolver, LockFile, RegistryClient, RegistrySource};

    // Initialize registry client
    let source = RegistrySource::default();
    let mut client = RegistryClient::new(source)
        .map_err(|e| format!("failed to initialize registry client: {}", e))?;

    // Try to load cached index, or update if needed
    if offline {
        // In offline mode, only use cached index
        if !client.load_cached_index().map_err(|e| e.to_string())? {
            return Err("No cached index available. Run without --offline first.".to_string());
        }
        println!("{} Using cached index (offline mode)", "Info".cyan());
    } else if !client.load_cached_index().map_err(|e| e.to_string())? {
        println!("{} Updating package index...", "Info".cyan());
        client
            .update_index()
            .map_err(|e| format!("failed to update index: {}", e))?;
    }

    // Load lock file if exists
    let lock_path = cwd.join("vais.lock");
    let lock = if lock_path.exists() && !update {
        Some(LockFile::load(&lock_path).map_err(|e| e.to_string())?)
    } else {
        None
    };

    // Parse package specs
    let mut resolver = DependencyResolver::new(&client);
    if let Some(ref l) = lock {
        resolver = resolver.with_lock(l);
    }

    for spec in &packages {
        let (name, version_req) = parse_package_spec(spec);
        resolver
            .add(&name, &version_req)
            .map_err(|e| format!("invalid version requirement '{}': {}", version_req, e))?;
    }

    // Resolve dependencies
    if verbose {
        println!("{} Resolving dependencies...", "Info".cyan());
    }

    let resolved = resolver
        .resolve()
        .map_err(|e| format!("dependency resolution failed: {}", e))?;

    if resolved.is_empty() {
        println!("{} No packages to install", "Info".cyan());
        return Ok(());
    }

    // Install packages
    for pkg in &resolved {
        if client.is_installed(&pkg.name, &pkg.version) {
            if verbose {
                println!(
                    "{} {} {} (cached)",
                    "Skipping".yellow(),
                    pkg.name,
                    pkg.version
                );
            }
        } else if offline {
            return Err(format!(
                "Package {} {} not in cache. Run without --offline to download.",
                pkg.name, pkg.version
            ));
        } else {
            println!("{} {} {}...", "Installing".green(), pkg.name, pkg.version);
            client
                .download(&pkg.name, &pkg.version)
                .map_err(|e| format!("failed to install {} {}: {}", pkg.name, pkg.version, e))?;
        }
    }

    // Save lock file
    let new_lock = resolver.generate_lock();
    new_lock
        .save(&lock_path)
        .map_err(|e| format!("failed to save lock file: {}", e))?;

    println!("{} Installed {} package(s)", "✓".green(), resolved.len());
    Ok(())
}

/// Update dependencies
pub(super) fn cmd_pkg_update(
    cwd: &Path,
    packages: Vec<String>,
    offline: bool,
    verbose: bool,
) -> Result<(), String> {
    use package::{find_manifest, load_manifest};
    use registry::{DependencyResolver, RegistryClient, RegistrySource};

    // Find and load manifest
    let pkg_dir = find_manifest(cwd).ok_or_else(|| "could not find vais.toml".to_string())?;
    let manifest = load_manifest(&pkg_dir).map_err(|e| e.to_string())?;

    // Initialize registry client
    let source = RegistrySource::default();
    let mut client = RegistryClient::new(source)
        .map_err(|e| format!("failed to initialize registry client: {}", e))?;

    if offline {
        if !client.load_cached_index().map_err(|e| e.to_string())? {
            return Err("No cached index available. Run without --offline first.".to_string());
        }
        println!("{} Using cached index (offline mode)", "Info".cyan());
    } else {
        println!("{} Updating package index...", "Info".cyan());
        client
            .update_index()
            .map_err(|e| format!("failed to update index: {}", e))?;
    }

    // Determine which packages to update
    let deps_to_update: Vec<(String, String)> = if packages.is_empty() {
        // Update all dependencies from manifest
        manifest
            .dependencies
            .iter()
            .filter_map(|(name, dep)| match dep {
                package::Dependency::Version(v) => Some((name.clone(), v.clone())),
                package::Dependency::Detailed(d) if d.version.is_some() => {
                    d.version.as_ref().map(|v| (name.clone(), v.clone()))
                }
                _ => None,
            })
            .collect()
    } else {
        // Update only specified packages
        packages
            .iter()
            .filter_map(|name| {
                manifest.dependencies.get(name).and_then(|dep| match dep {
                    package::Dependency::Version(v) => Some((name.clone(), v.clone())),
                    package::Dependency::Detailed(d) if d.version.is_some() => {
                        d.version.as_ref().map(|v| (name.clone(), v.clone()))
                    }
                    _ => None,
                })
            })
            .collect()
    };

    if deps_to_update.is_empty() {
        println!("{} No registry dependencies to update", "Info".cyan());
        return Ok(());
    }

    // Resolve with fresh versions (no lock file)
    let mut resolver = DependencyResolver::new(&client);
    for (name, req) in &deps_to_update {
        resolver
            .add(name, req)
            .map_err(|e| format!("invalid version requirement '{}': {}", req, e))?;
    }

    let resolved = resolver
        .resolve()
        .map_err(|e| format!("dependency resolution failed: {}", e))?;

    // Install/update packages
    for pkg in &resolved {
        if client.is_installed(&pkg.name, &pkg.version) {
            if verbose {
                println!(
                    "{} {} {} (up to date)",
                    "Skipping".yellow(),
                    pkg.name,
                    pkg.version
                );
            }
        } else {
            println!("{} {} {}...", "Updating".green(), pkg.name, pkg.version);
            client
                .download(&pkg.name, &pkg.version)
                .map_err(|e| format!("failed to update {} {}: {}", pkg.name, pkg.version, e))?;
        }
    }

    // Save new lock file
    let lock_path = pkg_dir.join("vais.lock");
    let new_lock = resolver.generate_lock();
    new_lock
        .save(&lock_path)
        .map_err(|e| format!("failed to save lock file: {}", e))?;

    println!("{} Updated {} package(s)", "✓".green(), resolved.len());
    Ok(())
}

/// Parse package spec (name or name@version)
pub(super) fn parse_package_spec(spec: &str) -> (String, String) {
    if let Some(idx) = spec.find('@') {
        let name = &spec[..idx];
        let version = &spec[idx + 1..];
        (name.to_string(), version.to_string())
    } else {
        (spec.to_string(), "*".to_string())
    }
}

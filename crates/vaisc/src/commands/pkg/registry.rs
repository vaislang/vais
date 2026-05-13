//! Registry queries and cache management.

use crate::registry;
use colored::Colorize;

use super::CacheAction;

/// Search for packages
pub(super) fn cmd_pkg_search(
    query: &str,
    limit: usize,
    offline: bool,
    verbose: bool,
    _sort: &str,
    _category: Option<&str>,
    _keyword: Option<&str>,
) -> Result<(), String> {
    use registry::{RegistryClient, RegistrySource};

    let source = RegistrySource::default();
    let mut client = RegistryClient::new(source)
        .map_err(|e| format!("failed to initialize registry client: {}", e))?;

    // Try cached index first, update if not available
    if offline {
        if !client.load_cached_index().map_err(|e| e.to_string())? {
            return Err("No cached index available. Run without --offline first.".to_string());
        }
        if verbose {
            println!("{} Using cached index (offline mode)", "Info".cyan());
        }
    } else if !client.load_cached_index().map_err(|e| e.to_string())? {
        println!("{} Updating package index...", "Info".cyan());
        client
            .update_index()
            .map_err(|e| format!("failed to update index: {}", e))?;
    }

    let results = client
        .search(query)
        .map_err(|e| format!("search failed: {}", e))?;

    if results.is_empty() {
        println!("{} No packages found matching '{}'", "Info".cyan(), query);
        return Ok(());
    }

    println!("{} packages found:\n", results.len().min(limit));

    for pkg in results.iter().take(limit) {
        let latest = pkg
            .latest_version()
            .map(|v| v.version.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        println!("  {} {}", pkg.name.bold(), format!("v{}", latest).cyan());
        if let Some(ref desc) = pkg.description {
            println!("    {}", desc);
        }
        if verbose && !pkg.keywords.is_empty() {
            println!("    {}: {}", "keywords".dimmed(), pkg.keywords.join(", "));
        }
        println!();
    }

    if results.len() > limit {
        println!("  ... and {} more", results.len() - limit);
    }

    Ok(())
}

/// Show package info
pub(super) fn cmd_pkg_info(name: &str, verbose: bool) -> Result<(), String> {
    use registry::{RegistryClient, RegistrySource};

    let source = RegistrySource::default();
    let mut client = RegistryClient::new(source)
        .map_err(|e| format!("failed to initialize registry client: {}", e))?;

    if !client.load_cached_index().map_err(|e| e.to_string())? {
        client
            .update_index()
            .map_err(|e| format!("failed to update index: {}", e))?;
    }

    let pkg = client
        .get_package(name)
        .map_err(|e| format!("package not found: {}", e))?;

    println!("{} {}", pkg.name.bold(), "package info".dimmed());
    println!();

    if let Some(ref desc) = pkg.description {
        println!("  {}: {}", "description".cyan(), desc);
    }

    if let Some(ref license) = pkg.license {
        println!("  {}: {}", "license".cyan(), license);
    }

    if !pkg.authors.is_empty() {
        println!("  {}: {}", "authors".cyan(), pkg.authors.join(", "));
    }

    if let Some(ref homepage) = pkg.homepage {
        println!("  {}: {}", "homepage".cyan(), homepage);
    }

    if let Some(ref repo) = pkg.repository {
        println!("  {}: {}", "repository".cyan(), repo);
    }

    if !pkg.keywords.is_empty() {
        println!("  {}: {}", "keywords".cyan(), pkg.keywords.join(", "));
    }

    println!();
    println!("  {}:", "versions".cyan());
    let versions = pkg.available_versions();
    for (i, v) in versions.iter().take(10).enumerate() {
        let marker = if i == 0 { " (latest)" } else { "" };
        println!("    {}{}", v.version, marker.green());
        if verbose && !v.dependencies.is_empty() {
            println!(
                "      deps: {}",
                v.dependencies
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
    if versions.len() > 10 {
        println!("    ... and {} more", versions.len() - 10);
    }

    Ok(())
}

/// Cache management
pub(super) fn cmd_pkg_cache(action: CacheAction, _verbose: bool) -> Result<(), String> {
    use registry::PackageCache;

    let cache = PackageCache::new().map_err(|e| format!("failed to access cache: {}", e))?;

    match action {
        CacheAction::Stats => {
            let stats = cache
                .stats()
                .map_err(|e| format!("failed to get cache stats: {}", e))?;

            println!("{}", "Cache Statistics".bold());
            println!("  {}: {}", "location".cyan(), cache.root().display());
            println!("  {}: {}", "packages".cyan(), stats.packages);
            println!("  {}: {}", "versions".cyan(), stats.versions);
            println!("  {}: {}", "size".cyan(), stats.size_display());
        }
        CacheAction::Clear => {
            cache
                .clear()
                .map_err(|e| format!("failed to clear cache: {}", e))?;
            println!("{} Cache cleared", "✓".green());
        }
        CacheAction::List => {
            let packages = cache
                .list_packages()
                .map_err(|e| format!("failed to list packages: {}", e))?;

            if packages.is_empty() {
                println!("{} Cache is empty", "Info".cyan());
                return Ok(());
            }

            println!("{}", "Cached packages:".bold());
            for name in packages {
                let versions = cache
                    .list_versions(&name)
                    .map_err(|e| format!("failed to list versions: {}", e))?;
                let version_strs: Vec<String> = versions.iter().map(|v| v.to_string()).collect();
                println!("  {} [{}]", name.bold(), version_strs.join(", "));
            }
        }
    }

    Ok(())
}

//! Utility package commands (tree, doc, vendor, package, metadata, owner, verify, audit).

use crate::doc_gen;
use crate::package;
use crate::registry;
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

/// Show dependency tree
pub(super) fn cmd_pkg_tree(
    manifest: &package::PackageManifest,
    pkg_dir: &Path,
    _all: bool,
    max_depth: Option<usize>,
    _verbose: bool,
) -> Result<(), String> {
    println!(
        "{} v{}",
        manifest.package.name.bold(),
        manifest.package.version
    );

    let cache_root = package::default_registry_cache_root();
    let deps = package::resolve_all_dependencies(manifest, pkg_dir, cache_root.as_deref())
        .map_err(|e| e.to_string())?;

    if deps.is_empty() {
        println!("  (no dependencies)");
        return Ok(());
    }

    let dep_count = deps.len();
    for (i, dep) in deps.iter().enumerate() {
        let is_last = i == dep_count - 1;
        let prefix = if is_last { "└── " } else { "├── " };

        // Try to get version from dependency's manifest
        let version = dep._manifest.package.version.clone();
        let path_info = if dep.path.starts_with(pkg_dir) {
            format!(
                " ({})",
                dep.path
                    .strip_prefix(pkg_dir)
                    .unwrap_or(&dep.path)
                    .display()
            )
        } else {
            String::new()
        };

        println!("{}{} v{}{}", prefix, dep.name.cyan(), version, path_info);

        // Show transitive dependencies (1 level deep unless max_depth allows more)
        if max_depth.is_none_or(|d| d > 0) {
            let child_cache = package::default_registry_cache_root();
            if let Ok(child_deps) =
                package::resolve_all_dependencies(&dep._manifest, &dep.path, child_cache.as_deref())
            {
                let child_prefix = if is_last { "    " } else { "│   " };
                let child_count = child_deps.len();
                let child_max = max_depth.map(|d| d.saturating_sub(1));
                for (j, child) in child_deps.iter().enumerate() {
                    if child_max == Some(0) {
                        break;
                    }
                    let child_last = j == child_count - 1;
                    let child_sym = if child_last {
                        "└── "
                    } else {
                        "├── "
                    };
                    println!(
                        "{}{}{}  v{}",
                        child_prefix,
                        child_sym,
                        child.name.cyan(),
                        child._manifest.package.version
                    );
                }
            }
        }
    }

    println!("\n{} {} direct dependencies", "✓".green(), dep_count);
    Ok(())
}

/// Generate documentation for a package
pub(super) fn cmd_pkg_doc(
    manifest: &package::PackageManifest,
    pkg_dir: &Path,
    output: &Path,
    format: &str,
    _open: bool,
    _verbose: bool,
) -> Result<(), String> {
    let src_dir = pkg_dir.join("src");
    if !src_dir.exists() {
        return Err("no src/ directory found in package".to_string());
    }

    // Collect all .vais files in src/
    let mut vais_files: Vec<PathBuf> = Vec::new();
    collect_vais_files(&src_dir, &mut vais_files)?;

    if vais_files.is_empty() {
        return Err("no .vais source files found in src/".to_string());
    }

    // Use the existing doc_gen module for each source file
    let output_dir = pkg_dir.join(output);
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("failed to create output directory: {}", e))?;

    println!(
        "{} Generating docs for {} (v{})",
        "Documenting".cyan().bold(),
        manifest.package.name,
        manifest.package.version
    );

    // Generate index file
    let ext = if format == "html" { "html" } else { "md" };
    let mut index_content = if format == "html" {
        format!(
            "<!DOCTYPE html>\n<html><head><title>{} Documentation</title></head>\n<body>\n<h1>{} v{}</h1>\n",
            manifest.package.name, manifest.package.name, manifest.package.version
        )
    } else {
        format!(
            "# {} v{}\n\n",
            manifest.package.name, manifest.package.version
        )
    };

    if let Some(desc) = &manifest.package.description {
        if format == "html" {
            index_content.push_str(&format!("<p>{}</p>\n", desc));
        } else {
            index_content.push_str(&format!("{}\n\n", desc));
        }
    }

    if format == "html" {
        index_content.push_str("<h2>Modules</h2>\n<ul>\n");
    } else {
        index_content.push_str("## Modules\n\n");
    }

    for file in &vais_files {
        let rel = file.strip_prefix(&src_dir).unwrap_or(file);
        let module_name = rel
            .with_extension("")
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "::");

        // Generate doc for this file using doc_gen
        if let Err(e) = doc_gen::run(file, &output_dir, format) {
            eprintln!(
                "  {} failed to generate docs for {}: {}",
                "warning:".yellow(),
                rel.display(),
                e
            );
            continue;
        }

        let doc_file = format!(
            "{}.{}",
            file.file_stem().unwrap_or_default().to_string_lossy(),
            ext
        );

        if format == "html" {
            index_content.push_str(&format!(
                "  <li><a href=\"{}\">{}</a></li>\n",
                doc_file, module_name
            ));
        } else {
            index_content.push_str(&format!("- [{}]({})\n", module_name, doc_file));
        }

        println!("  {} {}", "Generated".green(), rel.display());
    }

    if format == "html" {
        index_content.push_str("</ul>\n</body></html>");
    }

    let index_path = output_dir.join(format!("index.{}", ext));
    fs::write(&index_path, index_content).map_err(|e| format!("failed to write index: {}", e))?;

    println!(
        "\n{} Documentation generated in {}",
        "✓".green(),
        output_dir.display()
    );
    Ok(())
}

pub(super) fn collect_vais_files(dir: &Path, results: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries =
        fs::read_dir(dir).map_err(|e| format!("cannot read '{}': {}", dir.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("failed reading directory entry: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            collect_vais_files(&path, results)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("vais") {
            results.push(path);
        }
    }
    results.sort();
    Ok(())
}

/// Vendor dependencies into vendor/ directory
pub(super) fn cmd_pkg_vendor(
    pkg_dir: &Path,
    manifest: &package::PackageManifest,
    verbose: bool,
) -> Result<(), String> {
    use package::*;

    let vendor_dir = pkg_dir.join("vendor");
    fs::create_dir_all(&vendor_dir).map_err(|e| format!("failed to create vendor/: {}", e))?;

    let cache_root = default_registry_cache_root();
    let deps = resolve_all_dependencies(manifest, pkg_dir, cache_root.as_deref())
        .map_err(|e| e.to_string())?;

    if deps.is_empty() {
        println!("{} No dependencies to vendor", "✓".green());
        return Ok(());
    }

    let mut vendored = 0;
    let mut config_entries: Vec<String> = Vec::new();

    for dep in &deps {
        let dest = vendor_dir.join(&dep.name);
        if dest.exists() {
            let _ = fs::remove_dir_all(&dest);
        }

        // Copy dependency directory
        copy_dir_recursive(&dep.path, &dest)?;
        vendored += 1;

        config_entries.push(format!(
            "[source.\"{}\"]\npath = \"vendor/{}\"",
            dep.name, dep.name
        ));

        if verbose {
            println!(
                "  {} {} -> vendor/{}",
                "Vendored".cyan(),
                dep.name,
                dep.name
            );
        }
    }

    // Write vendor/config.toml
    let config_content = format!(
        "# Vendored dependencies\n# Generated by vaisc pkg vendor\n\n{}\n",
        config_entries.join("\n\n")
    );
    fs::write(vendor_dir.join("config.toml"), &config_content)
        .map_err(|e| format!("failed to write vendor/config.toml: {}", e))?;

    println!(
        "{} Vendored {} dependenc{} to vendor/",
        "✓".green(),
        vendored,
        if vendored == 1 { "y" } else { "ies" }
    );
    Ok(())
}

/// Recursively copy a directory
pub(super) fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("failed to create {}: {}", dst.display(), e))?;

    let entries =
        fs::read_dir(src).map_err(|e| format!("failed to read {}: {}", src.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read entry: {}", e))?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            // Skip target/, .git/, vendor/ directories
            let dir_name = entry.file_name();
            let name = dir_name.to_str().unwrap_or("");
            if name == "target" || name == ".git" || name == "vendor" {
                continue;
            }
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)
                .map_err(|e| format!("failed to copy {}: {}", path.display(), e))?;
        }
    }
    Ok(())
}

/// Create .vpkg archive for publishing preview
pub(super) fn cmd_pkg_package(
    pkg_dir: &Path,
    manifest: &package::PackageManifest,
    list_only: bool,
    verbose: bool,
) -> Result<(), String> {
    let pkg_name = &manifest.package.name;
    let pkg_version = &manifest.package.version;

    // Collect files to include
    let mut files: Vec<PathBuf> = Vec::new();
    let mut total_size: u64 = 0;

    // Always include vais.toml
    let manifest_file = pkg_dir.join("vais.toml");
    if manifest_file.exists() {
        files.push(PathBuf::from("vais.toml"));
        total_size += fs::metadata(&manifest_file).map(|m| m.len()).unwrap_or(0);
    }

    // Include src/ directory
    let src_dir = pkg_dir.join("src");
    if src_dir.exists() {
        collect_files_recursive(
            &src_dir,
            &PathBuf::from("src"),
            &mut files,
            &mut total_size,
            pkg_dir,
        )?;
    }

    // Include tests/ directory
    let tests_dir = pkg_dir.join("tests");
    if tests_dir.exists() {
        collect_files_recursive(
            &tests_dir,
            &PathBuf::from("tests"),
            &mut files,
            &mut total_size,
            pkg_dir,
        )?;
    }

    // Include README.md if it exists
    let readme = pkg_dir.join("README.md");
    if readme.exists() {
        files.push(PathBuf::from("README.md"));
        total_size += fs::metadata(&readme).map(|m| m.len()).unwrap_or(0);
    }

    // Include build.vais if it exists
    let build_script = pkg_dir.join("build.vais");
    if build_script.exists() {
        files.push(PathBuf::from("build.vais"));
        total_size += fs::metadata(&build_script).map(|m| m.len()).unwrap_or(0);
    }

    if list_only {
        println!(
            "{} {} v{} contents:",
            "Package".cyan(),
            pkg_name,
            pkg_version
        );
        for file in &files {
            let full_path = pkg_dir.join(file);
            let size = fs::metadata(&full_path).map(|m| m.len()).unwrap_or(0);
            println!("  {} ({} bytes)", file.display(), size);
        }
        println!("\n{} files, {} bytes total", files.len(), total_size);
        return Ok(());
    }

    // Create archive
    let archive_name = format!("{}-{}.vpkg", pkg_name, pkg_version);
    let archive_path = pkg_dir.join("target").join(&archive_name);
    let target_dir = pkg_dir.join("target");
    fs::create_dir_all(&target_dir).map_err(|e| format!("failed to create target/: {}", e))?;

    // Write as tar.gz
    let file =
        fs::File::create(&archive_path).map_err(|e| format!("failed to create archive: {}", e))?;
    let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);

    for rel_path in &files {
        let full_path = pkg_dir.join(rel_path);
        let mut f = fs::File::open(&full_path)
            .map_err(|e| format!("failed to open {}: {}", full_path.display(), e))?;
        tar.append_file(rel_path, &mut f)
            .map_err(|e| format!("failed to add {} to archive: {}", rel_path.display(), e))?;
    }

    tar.into_inner()
        .map_err(|e| format!("failed to finalize archive: {}", e))?
        .finish()
        .map_err(|e| format!("failed to compress archive: {}", e))?;

    let archive_size = fs::metadata(&archive_path).map(|m| m.len()).unwrap_or(0);
    println!(
        "{} Created {} ({} files, {} bytes compressed)",
        "✓".green(),
        archive_name,
        files.len(),
        archive_size
    );

    if verbose {
        for file in &files {
            println!("  {}", file.display());
        }
    }

    Ok(())
}

/// Collect files recursively for packaging
pub(super) fn collect_files_recursive(
    dir: &Path,
    rel_prefix: &Path,
    files: &mut Vec<PathBuf>,
    total_size: &mut u64,
    _pkg_dir: &Path,
) -> Result<(), String> {
    let entries =
        fs::read_dir(dir).map_err(|e| format!("failed to read {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("dir entry error: {}", e))?;
        let path = entry.path();
        let rel_path = rel_prefix.join(entry.file_name());

        if path.is_dir() {
            let name = entry.file_name();
            let name_str = name.to_str().unwrap_or("");
            if name_str == "target" || name_str == ".git" || name_str == "vendor" {
                continue;
            }
            collect_files_recursive(&path, &rel_path, files, total_size, _pkg_dir)?;
        } else {
            *total_size += fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            files.push(rel_path);
        }
    }
    Ok(())
}

/// Show package metadata in machine-readable format
pub(super) fn cmd_pkg_metadata(
    manifest: &package::PackageManifest,
    format: &str,
) -> Result<(), String> {
    match format {
        "json" => {
            let json = serde_json::to_string_pretty(manifest)
                .map_err(|e| format!("failed to serialize metadata: {}", e))?;
            println!("{}", json);
        }
        "toml" => {
            let toml_str = toml::to_string_pretty(manifest)
                .map_err(|e| format!("failed to serialize metadata: {}", e))?;
            println!("{}", toml_str);
        }
        _ => {
            return Err(format!(
                "unsupported format '{}'. Use 'json' or 'toml'",
                format
            ));
        }
    }
    Ok(())
}

/// Manage package owners
pub(super) fn cmd_pkg_owner(
    pkg_dir: &Path,
    manifest: &package::PackageManifest,
    add: Option<String>,
    remove: Option<String>,
    list: bool,
) -> Result<(), String> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct OwnersFile {
        owners: Vec<String>,
    }

    let owners_file = pkg_dir.join(".vais").join("owners.toml");

    // Load existing owners
    let mut owners: Vec<String> = if owners_file.exists() {
        let content = fs::read_to_string(&owners_file)
            .map_err(|e| format!("failed to read owners: {}", e))?;
        let parsed: OwnersFile =
            toml::from_str(&content).map_err(|e| format!("failed to parse owners.toml: {}", e))?;
        parsed.owners
    } else {
        Vec::new()
    };

    if let Some(user) = add {
        if owners.contains(&user) {
            println!("{} '{}' is already an owner", "Note".yellow(), user);
        } else {
            owners.push(user.clone());
            save_owners(pkg_dir, &owners)?;
            println!(
                "{} Added '{}' as owner of '{}'",
                "✓".green(),
                user,
                manifest.package.name
            );
        }
        return Ok(());
    }

    if let Some(user) = remove {
        if let Some(pos) = owners.iter().position(|o| o == &user) {
            owners.remove(pos);
            save_owners(pkg_dir, &owners)?;
            println!(
                "{} Removed '{}' from owners of '{}'",
                "✓".green(),
                user,
                manifest.package.name
            );
        } else {
            return Err(format!(
                "'{}' is not an owner of '{}'",
                user, manifest.package.name
            ));
        }
        return Ok(());
    }

    if list || (add.is_none() && remove.is_none()) {
        println!(
            "{} Owners of '{}':",
            "Package".cyan(),
            manifest.package.name
        );
        if owners.is_empty() {
            println!("  (no owners configured)");
        } else {
            for owner in &owners {
                println!("  {}", owner);
            }
        }
    }

    Ok(())
}

/// Save owners to .vais/owners.toml
pub(super) fn save_owners(pkg_dir: &Path, owners: &[String]) -> Result<(), String> {
    let vais_dir = pkg_dir.join(".vais");
    fs::create_dir_all(&vais_dir).map_err(|e| format!("failed to create .vais/: {}", e))?;

    let owners_str: Vec<String> = owners.iter().map(|o| format!("\"{}\"", o)).collect();
    let content = format!("owners = [{}]\n", owners_str.join(", "));
    fs::write(vais_dir.join("owners.toml"), &content)
        .map_err(|e| format!("failed to write owners.toml: {}", e))?;
    Ok(())
}

/// Verify package manifest is valid
pub(super) fn cmd_pkg_verify(pkg_dir: &Path, verbose: bool) -> Result<(), String> {
    let manifest_path = pkg_dir.join("vais.toml");
    let mut issues: Vec<String> = Vec::new();

    // 1. Check vais.toml exists and is valid TOML
    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("failed to read vais.toml: {}", e))?;

    let parsed: Result<toml::Value, _> = toml::from_str(&content);
    if let Err(e) = parsed {
        issues.push(format!("invalid TOML: {}", e));
    }

    // 2. Try to load as manifest
    match package::load_manifest(pkg_dir) {
        Ok(manifest) => {
            // Check required fields
            if manifest.package.name.is_empty() {
                issues.push("package name is empty".to_string());
            }
            if manifest.package.version.is_empty() {
                issues.push("package version is empty".to_string());
            }

            // Check version is valid semver-ish
            let version = &manifest.package.version;
            let parts: Vec<&str> = version.split('.').collect();
            if parts.len() < 2 {
                issues.push(format!(
                    "version '{}' should be semver (e.g., 1.0.0)",
                    version
                ));
            } else {
                for part in &parts {
                    if part.parse::<u64>().is_err() {
                        issues.push(format!("version component '{}' is not a number", part));
                        break;
                    }
                }
            }

            if verbose {
                println!("  {} name: {}", "✓".green(), manifest.package.name);
                println!("  {} version: {}", "✓".green(), manifest.package.version);
            }
        }
        Err(e) => {
            issues.push(format!("failed to parse manifest: {}", e));
        }
    }

    // 3. Check entry point exists
    let src_dir = pkg_dir.join("src");
    let has_main = src_dir.join("main.vais").exists();
    let has_lib = src_dir.join("lib.vais").exists();
    if !has_main && !has_lib {
        issues.push("no src/main.vais or src/lib.vais found".to_string());
    } else if verbose {
        if has_main {
            println!("  {} src/main.vais exists (binary package)", "✓".green());
        }
        if has_lib {
            println!("  {} src/lib.vais exists (library package)", "✓".green());
        }
    }

    if issues.is_empty() {
        println!("{} Package verification passed", "✓".green());
        Ok(())
    } else {
        println!(
            "{} Package verification found {} issue(s):",
            "✗".red(),
            issues.len()
        );
        for issue in &issues {
            println!("  {} {}", "•".red(), issue);
        }
        Err(format!("{} verification issue(s) found", issues.len()))
    }
}

/// Audit dependencies for known vulnerabilities
pub(super) fn cmd_pkg_audit(cwd: &Path, format: &str, verbose: bool) -> Result<(), String> {
    use package::{find_manifest, load_manifest};

    // Find and load manifest
    let pkg_dir = find_manifest(cwd).ok_or_else(|| "could not find vais.toml".to_string())?;
    let manifest = load_manifest(&pkg_dir).map_err(|e| e.to_string())?;

    // Load lock file if exists
    let lock_path = pkg_dir.join("vais.lock");
    let locked_packages: Vec<(String, String)> = if lock_path.exists() {
        use registry::LockFile;
        let lock = LockFile::load(&lock_path).map_err(|e| e.to_string())?;
        lock.packages
            .iter()
            .map(|(name, pkg)| (name.clone(), pkg.version.to_string()))
            .collect()
    } else {
        // Use manifest dependencies if no lock file
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
    };

    if locked_packages.is_empty() {
        println!("{} No dependencies to audit", "Info".cyan());
        return Ok(());
    }

    if verbose {
        println!(
            "{} Auditing {} package(s)...",
            "Info".cyan(),
            locked_packages.len()
        );
    }

    // Check for known vulnerabilities using OSV API
    use registry::VulnerabilityScanner;

    let scanner = VulnerabilityScanner::new();
    let vulns_by_package = scanner.query_batch(&locked_packages).unwrap_or_else(|e| {
        if verbose {
            eprintln!(
                "{} Failed to query vulnerability database: {}",
                "Warning".yellow(),
                e
            );
        }
        std::collections::HashMap::new()
    });

    // Flatten vulnerabilities into (package, version, advisory) tuples
    let mut vulnerabilities: Vec<(String, String, String)> = Vec::new();
    for (name, version) in &locked_packages {
        if let Some(vulns) = vulns_by_package.get(name) {
            for vuln in vulns {
                let severity = VulnerabilityScanner::severity_label(vuln);
                let advisory = format!("[{}] {} - {}", severity, vuln.id, vuln.summary);
                vulnerabilities.push((name.clone(), version.clone(), advisory));
            }
        }
    }

    match format {
        "json" => {
            println!("{{");
            println!("  \"packages\": {},", locked_packages.len());
            println!("  \"vulnerabilities\": [");
            for (i, (pkg, ver, advisory)) in vulnerabilities.iter().enumerate() {
                let comma = if i < vulnerabilities.len() - 1 {
                    ","
                } else {
                    ""
                };
                println!(
                    "    {{ \"package\": \"{}\", \"version\": \"{}\", \"advisory\": \"{}\" }}{}",
                    pkg, ver, advisory, comma
                );
            }
            println!("  ]");
            println!("}}");
        }
        _ => {
            println!("{}", "Dependency Audit".bold());
            println!("  {}: {}", "packages scanned".cyan(), locked_packages.len());

            if vulnerabilities.is_empty() {
                println!("\n{} No known vulnerabilities found", "✓".green());
            } else {
                println!(
                    "\n{} {} vulnerabilities found:",
                    "⚠".yellow().bold(),
                    vulnerabilities.len()
                );
                for (pkg, ver, advisory) in &vulnerabilities {
                    println!("  {} {} - {}", pkg.red(), ver, advisory);
                }
            }

            println!(
                "\n{} For more information, visit: https://osv.dev",
                "ℹ".blue()
            );
        }
    }

    Ok(())
}

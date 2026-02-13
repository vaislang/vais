//! Build-related package commands.

use crate::commands::build::cmd_build;
use crate::package;
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use vais_codegen::TargetTriple;
use vais_plugin::PluginRegistry;

/// Resolve feature flags from manifest and CLI options
pub(super) fn resolve_feature_flags(
    manifest: &package::PackageManifest,
    features_str: Option<&str>,
    all_features: bool,
    no_default_features: bool,
) -> Result<Vec<String>, String> {
    let feature_config = match &manifest.features {
        Some(fc) => fc,
        None => {
            // No [features] section — if user specified features, warn
            if features_str.is_some() || all_features {
                return Err("package has no [features] section in vais.toml".to_string());
            }
            return Ok(Vec::new());
        }
    };

    if all_features {
        return Ok(feature_config.all_features());
    }

    let selected: Vec<String> = features_str
        .map(|s| {
            s.split(',')
                .map(|f| f.trim().to_string())
                .filter(|f| !f.is_empty())
                .collect()
        })
        .unwrap_or_default();

    // Validate that all selected features exist
    for feat in &selected {
        if !feature_config.features.contains_key(feat) && !feature_config.default.contains(feat) {
            return Err(format!(
                "feature '{}' not found in [features] section. Available: {}",
                feat,
                feature_config.all_features().join(", ")
            ));
        }
    }

    let use_defaults = !no_default_features;
    Ok(feature_config.resolve_features(&selected, use_defaults))
}

/// Build the package and its dependencies
#[allow(clippy::too_many_arguments)]
pub(super) fn cmd_pkg_build(
    cwd: &Path,
    release: bool,
    debug: bool,
    hot: bool,
    workspace: bool,
    features: Option<String>,
    all_features: bool,
    no_default_features: bool,
    verbose: bool,
) -> Result<(), String> {
    use package::*;

    if workspace {
        return cmd_pkg_build_workspace(cwd, release, debug, hot, verbose);
    }

    // Find manifest
    let pkg_dir = find_manifest(cwd).ok_or_else(|| {
        "could not find vais.toml in current directory or parents".to_string()
    })?;

    let manifest = load_manifest(&pkg_dir).map_err(|e| e.to_string())?;

    // Auto-detect workspace: if manifest has [workspace], build all members
    if manifest.workspace.is_some() {
        return cmd_pkg_build_workspace(&pkg_dir, release, debug, hot, verbose);
    }

    // Resolve feature flags and inject into cfg_values
    let enabled_features = resolve_feature_flags(
        &manifest,
        features.as_deref(),
        all_features,
        no_default_features,
    )?;
    if !enabled_features.is_empty() {
        // Set feature cfg values as env var for the build pipeline
        std::env::set_var("VAIS_FEATURES", enabled_features.join(","));
        if verbose {
            println!(
                "{} features: {}",
                "Enabled".cyan(),
                enabled_features.join(", ")
            );
        }
    }

    cmd_pkg_build_single(&pkg_dir, &manifest, release, debug, hot, verbose)
}

/// Build a single package (non-workspace)
pub(super) fn cmd_pkg_build_single(
    pkg_dir: &Path,
    manifest: &package::PackageManifest,
    release: bool,
    debug: bool,
    hot: bool,
    verbose: bool,
) -> Result<(), String> {
    use package::*;

    if verbose {
        println!("{} {}", "Building".cyan(), manifest.package.name);
    }

    // Resolve dependencies (path + registry)
    let cache_root = package::default_registry_cache_root();
    let deps = resolve_all_dependencies(manifest, pkg_dir, cache_root.as_deref())
        .map_err(|e| e.to_string())?;

    if verbose && !deps.is_empty() {
        println!("{} dependencies:", "Resolved".cyan());
        for dep in &deps {
            println!("  {} -> {}", dep.name, dep.path.display());
        }
    }

    // Collect dependency source search paths for import resolution
    let dep_search_paths: Vec<PathBuf> = deps
        .iter()
        .filter_map(|dep| {
            let src_dir = dep.path.join("src");
            if src_dir.exists() {
                Some(src_dir)
            } else if dep.path.exists() {
                Some(dep.path.clone())
            } else {
                None
            }
        })
        .collect();

    // Determine entry point
    let src_dir = pkg_dir.join("src");
    let is_lib = !src_dir.join("main.vais").exists() && src_dir.join("lib.vais").exists();
    let entry = if src_dir.join("main.vais").exists() {
        src_dir.join("main.vais")
    } else if src_dir.join("lib.vais").exists() {
        src_dir.join("lib.vais")
    } else {
        return Err("no main.vais or lib.vais found in src/".to_string());
    };

    // Set dependency search paths as environment variable for import resolution
    if !dep_search_paths.is_empty() {
        let paths_str: Vec<String> = dep_search_paths
            .iter()
            .map(|p| p.display().to_string())
            .collect();
        std::env::set_var("VAIS_DEP_PATHS", paths_str.join(":"));
    }

    // Build options
    let opt_level = if release { 2 } else { 0 };
    let output = pkg_dir.join("target").join(&manifest.package.name);
    let profile = if release { "release" } else { "debug" };

    let lto_mode = if release {
        vais_codegen::optimize::LtoMode::Thin
    } else {
        vais_codegen::optimize::LtoMode::None
    };

    // Create target directory
    let target_dir = pkg_dir.join("target");
    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("failed to create target directory: {}", e))?;

    // Execute build script (build.vais) if present
    let build_script = pkg_dir.join("build.vais");
    if build_script.exists() {
        run_build_script(pkg_dir, &target_dir, profile, verbose)?;
    }

    let plugins = PluginRegistry::new();

    // Library packages: emit IR only (no linking), binary packages: full build
    cmd_build(
        &entry,
        Some(output.clone()),
        is_lib, // emit_ir = true for library packages
        opt_level,
        debug,
        verbose,
        &plugins,
        TargetTriple::Native,
        false,
        false,
        None,
        hot,
        lto_mode,
        vais_codegen::optimize::PgoMode::None,
        vais_codegen::optimize::CoverageMode::None,
        false,
        None,
        false,
        false,
        536870912,
    )?;

    if is_lib {
        println!("{} Built library {}", "✓".green(), output.display());
    } else if hot {
        println!(
            "{} Built hot-reload dylib {}",
            "✓".green(),
            output.display()
        );
    } else {
        println!("{} Built {}", "✓".green(), output.display());
    }
    Ok(())
}

/// Run build.vais build script before compilation
pub(super) fn run_build_script(
    pkg_dir: &Path,
    target_dir: &Path,
    profile: &str,
    verbose: bool,
) -> Result<(), String> {
    let build_script = pkg_dir.join("build.vais");
    if !build_script.exists() {
        return Ok(());
    }

    if verbose {
        println!("{} Running build script: build.vais", "Build".cyan());
    }

    let out_dir = target_dir.join("build");
    fs::create_dir_all(&out_dir).map_err(|e| format!("failed to create OUT_DIR: {}", e))?;

    // Determine target triple
    let target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);

    // Compile the build script
    let build_output = target_dir.join("build_script");
    let plugins = PluginRegistry::new();

    cmd_build(
        &build_script,
        Some(build_output.clone()),
        false,
        0,
        false,
        verbose,
        &plugins,
        TargetTriple::Native,
        false,
        false,
        None,
        false,
        vais_codegen::optimize::LtoMode::None,
        vais_codegen::optimize::PgoMode::None,
        vais_codegen::optimize::CoverageMode::None,
        false,
        None,
        false,
        false,
        536870912,
    )?;

    // Run the build script with environment variables
    let status = Command::new(&build_output)
        .current_dir(pkg_dir)
        .env("OUT_DIR", &out_dir)
        .env("TARGET", &target)
        .env("PROFILE", profile)
        .env("CARGO_MANIFEST_DIR", pkg_dir)
        .status()
        .map_err(|e| format!("failed to run build script: {}", e))?;

    if !status.success() {
        return Err(format!(
            "build script failed with exit code: {}",
            status.code().unwrap_or(-1)
        ));
    }

    if verbose {
        println!("{} Build script completed", "✓".green());
    }

    Ok(())
}

/// Get the global bin directory (~/.vais/bin/)
pub(crate) fn global_bin_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "could not determine home directory".to_string())?;
    Ok(home.join(".vais").join("bin"))
}

/// Install a package binary globally
pub(crate) fn cmd_install(
    package: &str,
    release: bool,
    verbose: bool,
    _plugins: &PluginRegistry,
) -> Result<(), String> {
    use package::*;

    let pkg_path = PathBuf::from(package);

    // Determine package directory
    let pkg_dir = if pkg_path.is_dir() {
        // Local path provided
        let manifest_dir = find_manifest(&pkg_path).ok_or_else(|| {
            format!(
                "could not find vais.toml in '{}' or its parents",
                pkg_path.display()
            )
        })?;
        manifest_dir
    } else {
        // Try current directory
        let cwd =
            std::env::current_dir().map_err(|e| format!("failed to get current dir: {}", e))?;
        find_manifest(&cwd).ok_or_else(|| {
            format!(
                "could not find vais.toml for package '{}'. Provide a path to a local package.",
                package
            )
        })?
    };

    let manifest = load_manifest(&pkg_dir).map_err(|e| e.to_string())?;
    let pkg_name = &manifest.package.name;

    // Verify it has a main.vais (binary package)
    let src_main = pkg_dir.join("src").join("main.vais");
    if !src_main.exists() {
        return Err(format!(
            "package '{}' does not have src/main.vais — only binary packages can be installed",
            pkg_name
        ));
    }

    println!(
        "{} {} v{}",
        "Installing".cyan(),
        pkg_name,
        manifest.package.version
    );

    // Build the package
    cmd_pkg_build_single(&pkg_dir, &manifest, release, false, false, verbose)?;

    // Copy binary to ~/.vais/bin/
    let bin_dir = global_bin_dir()?;
    fs::create_dir_all(&bin_dir)
        .map_err(|e| format!("failed to create {}: {}", bin_dir.display(), e))?;

    let built_binary = pkg_dir.join("target").join(pkg_name);
    let dest = bin_dir.join(pkg_name);

    fs::copy(&built_binary, &dest)
        .map_err(|e| format!("failed to copy binary to {}: {}", dest.display(), e))?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&dest, perms)
            .map_err(|e| format!("failed to set permissions: {}", e))?;
    }

    println!(
        "{} Installed {} to {}",
        "✓".green(),
        pkg_name,
        dest.display()
    );

    // Check if ~/.vais/bin is in PATH
    let path_env = std::env::var("PATH").unwrap_or_default();
    let bin_dir_str = bin_dir.to_str().unwrap_or("");
    if !path_env.split(':').any(|p| p == bin_dir_str) {
        println!();
        println!(
            "{} {} is not in your PATH. Add it with:",
            "Note".yellow(),
            bin_dir.display()
        );
        println!("  export PATH=\"{}:$PATH\"", bin_dir.display());
        println!("  Or add the line above to your ~/.bashrc, ~/.zshrc, or shell profile.");
    }

    Ok(())
}

/// Uninstall a globally installed package binary
pub(crate) fn cmd_uninstall(package: &str) -> Result<(), String> {
    let bin_dir = global_bin_dir()?;
    let binary = bin_dir.join(package);

    if !binary.exists() {
        return Err(format!(
            "package '{}' is not installed (not found in {})",
            package,
            bin_dir.display()
        ));
    }

    fs::remove_file(&binary)
        .map_err(|e| format!("failed to remove {}: {}", binary.display(), e))?;

    println!(
        "{} Uninstalled {} from {}",
        "✓".green(),
        package,
        bin_dir.display()
    );

    Ok(())
}

/// Build all workspace members
pub(super) fn cmd_pkg_build_workspace(
    start_dir: &Path,
    release: bool,
    debug: bool,
    hot: bool,
    verbose: bool,
) -> Result<(), String> {
    use package::*;

    let ws_root = find_workspace_root(start_dir).ok_or_else(|| {
        "could not find workspace root (vais.toml with [workspace] section)".to_string()
    })?;

    let mut workspace = resolve_workspace_members(&ws_root).map_err(|e| e.to_string())?;

    // Resolve inter-workspace path dependencies
    resolve_inter_workspace_deps(&mut workspace);

    let member_count = workspace.members.len();
    println!(
        "{} Building workspace with {} member{}",
        "Workspace".cyan(),
        member_count,
        if member_count == 1 { "" } else { "s" }
    );

    let mut built = 0;
    let mut failed = 0;

    for member in &workspace.members {
        let name = &member.manifest.package.name;
        if name.is_empty() {
            continue;
        }

        if verbose {
            println!(
                "\n{} Building member: {} ({})",
                ">>".cyan(),
                name,
                member.path.display()
            );
        }

        match cmd_pkg_build_single(&member.path, &member.manifest, release, debug, hot, verbose) {
            Ok(()) => {
                built += 1;
            }
            Err(e) => {
                eprintln!("{} Failed to build '{}': {}", "✗".red(), name, e);
                failed += 1;
            }
        }
    }

    if failed > 0 {
        Err(format!(
            "workspace build: {} succeeded, {} failed",
            built, failed
        ))
    } else {
        println!(
            "\n{} Built {} workspace member{}",
            "✓".green(),
            built,
            if built == 1 { "" } else { "s" }
        );
        Ok(())
    }
}

//! Package management commands.

use crate::commands::build::cmd_build;
use crate::commands::simple::cmd_check;
use crate::doc_gen;
use crate::package;
use crate::registry;
use clap::Subcommand;
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use vais_codegen::TargetTriple;
use vais_plugin::PluginRegistry;

#[derive(Subcommand)]
pub(crate) enum PkgCommands {
    /// Initialize a new package in the current directory
    Init {
        /// Package name (defaults to directory name)
        #[arg(long)]
        name: Option<String>,
    },

    /// Build the package and its dependencies
    Build {
        /// Build with optimizations
        #[arg(long)]
        release: bool,

        /// Include debug information
        #[arg(short = 'g', long)]
        debug: bool,

        /// Enable hot reload mode (generate dylib)
        #[arg(long)]
        hot: bool,

        /// Build all workspace members
        #[arg(long)]
        workspace: bool,

        /// Comma-separated list of features to enable
        #[arg(long)]
        features: Option<String>,

        /// Enable all available features
        #[arg(long)]
        all_features: bool,

        /// Do not enable default features
        #[arg(long)]
        no_default_features: bool,
    },

    /// Type-check the package without compiling
    Check {
        /// Check all workspace members
        #[arg(long)]
        workspace: bool,

        /// Comma-separated list of features to enable
        #[arg(long)]
        features: Option<String>,

        /// Enable all available features
        #[arg(long)]
        all_features: bool,

        /// Do not enable default features
        #[arg(long)]
        no_default_features: bool,
    },

    /// Add a dependency
    Add {
        /// Dependency name
        name: String,

        /// Path to local dependency
        #[arg(long)]
        path: Option<String>,

        /// Version specification (for future registry support)
        #[arg(long)]
        version: Option<String>,
    },

    /// Remove a dependency
    Remove {
        /// Dependency name
        name: String,
    },

    /// Remove build artifacts
    Clean,

    /// Install packages from registry
    Install {
        /// Package name with optional version (e.g., json-parser@1.0)
        packages: Vec<String>,

        /// Update the lock file
        #[arg(long)]
        update: bool,

        /// Use only cached packages (no network requests)
        #[arg(long)]
        offline: bool,
    },

    /// Update dependencies to latest compatible versions
    Update {
        /// Specific packages to update (or all if not specified)
        packages: Vec<String>,

        /// Use only cached packages (no network requests)
        #[arg(long)]
        offline: bool,
    },

    /// Search for packages in the registry
    Search {
        /// Search query
        query: String,

        /// Maximum results to show
        #[arg(long, default_value = "20")]
        limit: usize,

        /// Search only in cached index (no network requests)
        #[arg(long)]
        offline: bool,

        /// Sort order: downloads, newest, name, relevance
        #[arg(long, default_value = "downloads")]
        sort: String,

        /// Filter by category
        #[arg(long)]
        category: Option<String>,

        /// Filter by keyword
        #[arg(long)]
        keyword: Option<String>,
    },

    /// Show information about a package
    Info {
        /// Package name
        name: String,
    },

    /// Show cache statistics and manage cache
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },

    /// Audit dependencies for known vulnerabilities
    Audit {
        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Publish package to registry
    Publish {
        /// Registry URL (default: https://registry.vais.dev)
        #[arg(long)]
        registry: Option<String>,

        /// Authentication token
        #[arg(long)]
        token: Option<String>,

        /// Dry run - validate but don't actually publish
        #[arg(long)]
        dry_run: bool,
    },

    /// Yank a published version
    Yank {
        /// Package name
        name: String,

        /// Version to yank
        #[arg(long)]
        version: String,

        /// Authentication token
        #[arg(long)]
        token: Option<String>,

        /// Registry URL
        #[arg(long)]
        registry: Option<String>,
    },

    /// Login to registry and store token
    Login {
        /// Registry URL
        #[arg(long)]
        registry: Option<String>,
    },

    /// Show dependency tree
    Tree {
        /// Show all transitive dependencies
        #[arg(long)]
        all: bool,

        /// Maximum depth of the tree
        #[arg(long)]
        depth: Option<usize>,
    },

    /// Generate documentation for the package
    Doc {
        /// Output directory for documentation
        #[arg(short, long, default_value = "docs")]
        output: PathBuf,

        /// Output format (markdown or html)
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Open documentation in browser after generation
        #[arg(long)]
        open: bool,
    },

    /// Copy dependencies to vendor/ for offline builds
    Vendor,

    /// Create .vpkg archive for publishing preview
    Package {
        /// Just list contents without creating archive
        #[arg(long)]
        list: bool,
    },

    /// Show package metadata in machine-readable format
    Metadata {
        /// Output format (json or toml)
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Manage package owners
    Owner {
        /// Add an owner
        #[arg(long)]
        add: Option<String>,

        /// Remove an owner
        #[arg(long)]
        remove: Option<String>,

        /// List current owners
        #[arg(long)]
        list: bool,
    },

    /// Verify package manifest is valid
    Verify,
}

#[derive(Subcommand)]
pub(crate) enum CacheAction {
    /// Show cache statistics
    Stats,
    /// Clear the package cache
    Clear,
    /// List cached packages
    List,
}

pub(crate) fn cmd_pkg(cmd: PkgCommands, verbose: bool) -> Result<(), String> {
    use package::*;
    use std::env;

    let cwd = env::current_dir().map_err(|e| format!("failed to get current directory: {}", e))?;

    match cmd {
        PkgCommands::Init { name } => {
            init_package(&cwd, name.as_deref()).map_err(|e| e.to_string())?;
            println!("{} Created package in {}", "✓".green(), cwd.display());
            Ok(())
        }

        PkgCommands::Build {
            release,
            debug,
            hot,
            workspace,
            features,
            all_features,
            no_default_features,
        } => {
            if workspace {
                return cmd_pkg_build_workspace(&cwd, release, debug, hot, verbose);
            }

            // Find manifest
            let pkg_dir = find_manifest(&cwd).ok_or_else(|| {
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

        PkgCommands::Check {
            workspace,
            features,
            all_features,
            no_default_features,
        } => {
            if workspace {
                return cmd_pkg_check_workspace(&cwd, verbose);
            }

            let pkg_dir =
                find_manifest(&cwd).ok_or_else(|| "could not find vais.toml".to_string())?;

            let manifest = load_manifest(&pkg_dir).map_err(|e| e.to_string())?;

            // Auto-detect workspace
            if manifest.workspace.is_some() {
                return cmd_pkg_check_workspace(&pkg_dir, verbose);
            }

            // Resolve feature flags
            let enabled_features = resolve_feature_flags(
                &manifest,
                features.as_deref(),
                all_features,
                no_default_features,
            )?;
            if !enabled_features.is_empty() {
                std::env::set_var("VAIS_FEATURES", enabled_features.join(","));
                if verbose {
                    println!(
                        "{} features: {}",
                        "Enabled".cyan(),
                        enabled_features.join(", ")
                    );
                }
            }

            let src_dir = pkg_dir.join("src");
            let entry = if src_dir.join("main.vais").exists() {
                src_dir.join("main.vais")
            } else if src_dir.join("lib.vais").exists() {
                src_dir.join("lib.vais")
            } else {
                return Err("no main.vais or lib.vais found in src/".to_string());
            };

            let plugins = PluginRegistry::new();
            cmd_check(&entry, verbose, &plugins)?;

            println!(
                "{} {} type-checks correctly",
                "✓".green(),
                manifest.package.name
            );
            Ok(())
        }

        PkgCommands::Add {
            name,
            path,
            version,
        } => {
            let pkg_dir =
                find_manifest(&cwd).ok_or_else(|| "could not find vais.toml".to_string())?;

            let manifest_path = pkg_dir.join("vais.toml");
            let is_registry_dep = path.is_none();

            add_dependency(&manifest_path, &name, path.as_deref(), version.as_deref())
                .map_err(|e| e.to_string())?;

            println!("{} Added dependency '{}'", "✓".green(), name);

            if is_registry_dep {
                // Check if the package is already installed in the cache
                let cache_root = package::default_registry_cache_root();
                let version_str = version.as_deref().unwrap_or("*");
                let is_cached = cache_root
                    .as_ref()
                    .and_then(|root| package::find_cached_registry_dep(root, &name, version_str))
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

        PkgCommands::Remove { name } => {
            let pkg_dir =
                find_manifest(&cwd).ok_or_else(|| "could not find vais.toml".to_string())?;

            let manifest_path = pkg_dir.join("vais.toml");
            remove_dependency(&manifest_path, &name).map_err(|e| e.to_string())?;

            println!("{} Removed dependency '{}'", "✓".green(), name);
            Ok(())
        }

        PkgCommands::Clean => {
            let pkg_dir =
                find_manifest(&cwd).ok_or_else(|| "could not find vais.toml".to_string())?;

            let target_dir = pkg_dir.join("target");
            let cache_dir = pkg_dir.join(".vais-cache");

            if target_dir.exists() {
                fs::remove_dir_all(&target_dir)
                    .map_err(|e| format!("failed to remove target/: {}", e))?;
            }

            if cache_dir.exists() {
                fs::remove_dir_all(&cache_dir)
                    .map_err(|e| format!("failed to remove .vais-cache/: {}", e))?;
            }

            println!("{} Cleaned build artifacts", "✓".green());
            Ok(())
        }

        PkgCommands::Install {
            packages,
            update,
            offline,
        } => cmd_pkg_install(&cwd, packages, update, offline, verbose),

        PkgCommands::Update { packages, offline } => {
            cmd_pkg_update(&cwd, packages, offline, verbose)
        }

        PkgCommands::Search {
            query,
            limit,
            offline,
            sort,
            category,
            keyword,
        } => cmd_pkg_search(
            &query,
            limit,
            offline,
            verbose,
            &sort,
            category.as_deref(),
            keyword.as_deref(),
        ),

        PkgCommands::Info { name } => cmd_pkg_info(&name, verbose),

        PkgCommands::Cache { action } => cmd_pkg_cache(action, verbose),

        PkgCommands::Audit { format } => cmd_pkg_audit(&cwd, &format, verbose),

        PkgCommands::Publish {
            registry,
            token,
            dry_run,
        } => cmd_pkg_publish(&cwd, registry, token, dry_run, verbose),

        PkgCommands::Yank {
            name,
            version,
            token,
            registry,
        } => cmd_pkg_yank(&name, &version, token, registry, verbose),

        PkgCommands::Login { registry } => cmd_pkg_login(registry, verbose),

        PkgCommands::Tree { all, depth } => {
            let pkg_dir = find_manifest(&cwd).ok_or_else(|| {
                "could not find vais.toml in current directory or parents".to_string()
            })?;
            let manifest = load_manifest(&pkg_dir).map_err(|e| e.to_string())?;
            cmd_pkg_tree(&manifest, &pkg_dir, all, depth, verbose)
        }

        PkgCommands::Doc {
            output,
            format,
            open,
        } => {
            let pkg_dir = find_manifest(&cwd).ok_or_else(|| {
                "could not find vais.toml in current directory or parents".to_string()
            })?;
            let manifest = load_manifest(&pkg_dir).map_err(|e| e.to_string())?;
            cmd_pkg_doc(&manifest, &pkg_dir, &output, &format, open, verbose)
        }

        PkgCommands::Vendor => {
            let pkg_dir =
                find_manifest(&cwd).ok_or_else(|| "could not find vais.toml".to_string())?;
            let manifest = load_manifest(&pkg_dir).map_err(|e| e.to_string())?;
            cmd_pkg_vendor(&pkg_dir, &manifest, verbose)
        }

        PkgCommands::Package { list } => {
            let pkg_dir =
                find_manifest(&cwd).ok_or_else(|| "could not find vais.toml".to_string())?;
            let manifest = load_manifest(&pkg_dir).map_err(|e| e.to_string())?;
            cmd_pkg_package(&pkg_dir, &manifest, list, verbose)
        }

        PkgCommands::Metadata { format } => {
            let pkg_dir =
                find_manifest(&cwd).ok_or_else(|| "could not find vais.toml".to_string())?;
            let manifest = load_manifest(&pkg_dir).map_err(|e| e.to_string())?;
            cmd_pkg_metadata(&manifest, &format)
        }

        PkgCommands::Owner { add, remove, list } => {
            let pkg_dir =
                find_manifest(&cwd).ok_or_else(|| "could not find vais.toml".to_string())?;
            let manifest = load_manifest(&pkg_dir).map_err(|e| e.to_string())?;
            cmd_pkg_owner(&pkg_dir, &manifest, add, remove, list)
        }

        PkgCommands::Verify => {
            let pkg_dir =
                find_manifest(&cwd).ok_or_else(|| "could not find vais.toml".to_string())?;
            cmd_pkg_verify(&pkg_dir, verbose)
        }
    }
}

/// Resolve feature flags from manifest and CLI options
pub(crate) fn resolve_feature_flags(
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

/// Build a single package (non-workspace)
pub(crate) fn cmd_pkg_build_single(
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
pub(crate) fn run_build_script(
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
pub(crate) fn cmd_pkg_build_workspace(
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

/// Type-check all workspace members
pub(crate) fn cmd_pkg_check_workspace(start_dir: &Path, verbose: bool) -> Result<(), String> {
    use package::*;

    let ws_root = find_workspace_root(start_dir).ok_or_else(|| {
        "could not find workspace root (vais.toml with [workspace] section)".to_string()
    })?;

    let mut workspace = resolve_workspace_members(&ws_root).map_err(|e| e.to_string())?;

    resolve_inter_workspace_deps(&mut workspace);

    let member_count = workspace.members.len();
    println!(
        "{} Checking workspace with {} member{}",
        "Workspace".cyan(),
        member_count,
        if member_count == 1 { "" } else { "s" }
    );

    let mut checked = 0;
    let mut failed = 0;

    for member in &workspace.members {
        let name = &member.manifest.package.name;
        if name.is_empty() {
            continue;
        }

        let src_dir = member.path.join("src");
        let entry = if src_dir.join("main.vais").exists() {
            src_dir.join("main.vais")
        } else if src_dir.join("lib.vais").exists() {
            src_dir.join("lib.vais")
        } else {
            if verbose {
                println!(
                    "{} Skipping '{}' (no src/main.vais or src/lib.vais)",
                    ">>".cyan(),
                    name
                );
            }
            continue;
        };

        if verbose {
            println!("{} Checking member: {}", ">>".cyan(), name);
        }

        let plugins = PluginRegistry::new();
        match cmd_check(&entry, verbose, &plugins) {
            Ok(()) => {
                checked += 1;
            }
            Err(e) => {
                eprintln!("{} Failed to check '{}': {}", "✗".red(), name, e);
                failed += 1;
            }
        }
    }

    if failed > 0 {
        Err(format!(
            "workspace check: {} passed, {} failed",
            checked, failed
        ))
    } else {
        println!(
            "{} {} workspace member{} type-check correctly",
            "✓".green(),
            checked,
            if checked == 1 { "" } else { "s" }
        );
        Ok(())
    }
}

/// Show dependency tree
pub(crate) fn cmd_pkg_tree(
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
pub(crate) fn cmd_pkg_doc(
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

        let doc_file = format!("{}.{}", file.file_stem().unwrap_or_default().to_string_lossy(), ext);

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

pub(crate) fn collect_vais_files(dir: &Path, results: &mut Vec<PathBuf>) -> Result<(), String> {
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
pub(crate) fn cmd_pkg_vendor(
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
pub(crate) fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
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
pub(crate) fn cmd_pkg_package(
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
pub(crate) fn collect_files_recursive(
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
pub(crate) fn cmd_pkg_metadata(
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
pub(crate) fn cmd_pkg_owner(
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
pub(crate) fn save_owners(pkg_dir: &Path, owners: &[String]) -> Result<(), String> {
    let vais_dir = pkg_dir.join(".vais");
    fs::create_dir_all(&vais_dir).map_err(|e| format!("failed to create .vais/: {}", e))?;

    let owners_str: Vec<String> = owners.iter().map(|o| format!("\"{}\"", o)).collect();
    let content = format!("owners = [{}]\n", owners_str.join(", "));
    fs::write(vais_dir.join("owners.toml"), &content)
        .map_err(|e| format!("failed to write owners.toml: {}", e))?;
    Ok(())
}

/// Verify package manifest is valid
pub(crate) fn cmd_pkg_verify(pkg_dir: &Path, verbose: bool) -> Result<(), String> {
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

/// Install packages from registry
pub(crate) fn cmd_pkg_install(
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
pub(crate) fn cmd_pkg_update(
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

/// Search for packages
pub(crate) fn cmd_pkg_search(
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
pub(crate) fn cmd_pkg_info(name: &str, verbose: bool) -> Result<(), String> {
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
pub(crate) fn cmd_pkg_cache(action: CacheAction, _verbose: bool) -> Result<(), String> {
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

/// Audit dependencies for known vulnerabilities
pub(crate) fn cmd_pkg_audit(cwd: &Path, format: &str, verbose: bool) -> Result<(), String> {
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

/// Publish a package to the registry
pub(crate) fn cmd_pkg_publish(
    cwd: &Path,
    registry: Option<String>,
    token: Option<String>,
    dry_run: bool,
    verbose: bool,
) -> Result<(), String> {
    use package::{find_manifest, load_manifest};

    let registry_url = registry.unwrap_or_else(|| "https://registry.vais.dev".to_string());

    // Find and load manifest
    let pkg_dir = find_manifest(cwd).ok_or_else(|| "could not find vais.toml".to_string())?;
    let manifest = load_manifest(&pkg_dir).map_err(|e| e.to_string())?;

    let pkg_name = &manifest.package.name;
    let pkg_version = &manifest.package.version;

    println!(
        "{} Packaging {} v{}...",
        "Info".cyan(),
        pkg_name,
        pkg_version
    );

    // Pack the package into a temporary archive
    let tmp_dir = std::env::temp_dir().join(format!("vais-publish-{}", std::process::id()));
    fs::create_dir_all(&tmp_dir).map_err(|e| format!("failed to create temp directory: {}", e))?;
    let archive_path = tmp_dir.join(format!("{}-{}.tar.gz", pkg_name, pkg_version));

    registry::pack_package(&pkg_dir, &archive_path)
        .map_err(|e| format!("failed to pack package: {}", e))?;

    // Read archive and compute checksum
    let archive_bytes =
        fs::read(&archive_path).map_err(|e| format!("failed to read archive: {}", e))?;
    let checksum = registry::sha256_hex(&archive_bytes);

    if verbose {
        println!(
            "  Archive size: {} bytes, checksum: {}",
            archive_bytes.len(),
            &checksum[..16]
        );
    }

    // Build metadata JSON
    let deps: serde_json::Map<String, serde_json::Value> = manifest
        .dependencies
        .iter()
        .map(|(name, dep)| {
            let version_str = match dep {
                package::Dependency::Version(v) => v.clone(),
                package::Dependency::Detailed(d) => {
                    d.version.clone().unwrap_or_else(|| "*".to_string())
                }
            };
            (name.clone(), serde_json::Value::String(version_str))
        })
        .collect();

    let metadata = serde_json::json!({
        "name": pkg_name,
        "version": pkg_version,
        "description": manifest.package.description.as_deref().unwrap_or(""),
        "authors": manifest.package.authors,
        "license": manifest.package.license.as_deref().unwrap_or(""),
        "checksum": checksum,
        "dependencies": deps,
    });

    if dry_run {
        println!("{} Dry run - would publish:", "Info".cyan());
        println!("  Name: {}", pkg_name);
        println!("  Version: {}", pkg_version);
        println!("  Checksum: {}", checksum);
        println!("  Archive size: {} bytes", archive_bytes.len());
        // Clean up
        let _ = fs::remove_dir_all(&tmp_dir);
        println!("{} Dry run complete, package is valid", "✓".green());
        return Ok(());
    }

    // Resolve token: argument > credentials file > error
    let auth_token = token
        .or_else(|| load_credentials_token(&registry_url))
        .ok_or_else(|| {
            "authentication token required. Use --token or run `vaisc pkg login` first".to_string()
        })?;

    // Build multipart body
    let metadata_str = serde_json::to_string(&metadata)
        .map_err(|e| format!("failed to serialize metadata: {}", e))?;

    let boundary = format!("----vais-publish-{}", std::process::id());
    let mut body = Vec::new();

    // metadata part
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"metadata\"\r\nContent-Type: application/json\r\n\r\n",
    );
    body.extend_from_slice(metadata_str.as_bytes());
    body.extend_from_slice(b"\r\n");

    // archive part
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"archive\"; filename=\"{}-{}.tar.gz\"\r\nContent-Type: application/gzip\r\n\r\n",
            pkg_name, pkg_version
        )
        .as_bytes(),
    );
    body.extend_from_slice(&archive_bytes);
    body.extend_from_slice(b"\r\n");

    // closing boundary
    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    println!(
        "{} Publishing {} v{} to {}...",
        "Info".cyan(),
        pkg_name,
        pkg_version,
        registry_url
    );

    let publish_url = format!("{}/packages/publish", registry_url.trim_end_matches('/'));
    let response = ureq::post(&publish_url)
        .set("Authorization", &format!("Bearer {}", auth_token))
        .set(
            "Content-Type",
            &format!("multipart/form-data; boundary={}", boundary),
        )
        .send_bytes(&body)
        .map_err(|e| match e {
            ureq::Error::Status(code, resp) => {
                let msg = resp.into_string().unwrap_or_default();
                format!("publish failed (HTTP {}): {}", code, msg)
            }
            _ => format!("publish failed: {}", e),
        })?;

    let status = response.status();
    if verbose {
        println!("  Server responded with status {}", status);
    }

    // Verify checksum by fetching package metadata from registry
    if verbose {
        println!("{} Verifying checksum...", "Info".cyan());
    }

    let verify_url = format!(
        "{}/packages/{}/{}",
        registry_url.trim_end_matches('/'),
        pkg_name,
        pkg_version
    );

    let verify_response = ureq::get(&verify_url)
        .set("Authorization", &format!("Bearer {}", auth_token))
        .call();

    match verify_response {
        Ok(resp) => {
            if let Ok(body) = resp.into_string() {
                if let Ok(pkg_info) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(server_checksum) = pkg_info.get("checksum").and_then(|c| c.as_str())
                    {
                        if server_checksum == checksum {
                            if verbose {
                                println!("  Checksum verified: {}", &checksum[..16]);
                            }
                        } else {
                            eprintln!(
                                "{} Warning: checksum mismatch (local: {}, server: {})",
                                "⚠".yellow(),
                                &checksum[..16],
                                &server_checksum[..16]
                            );
                        }
                    }
                }
            }
        }
        Err(e) => {
            if verbose {
                eprintln!("{} Could not verify checksum: {}", "⚠".yellow(), e);
            }
        }
    }

    // Clean up temp files
    let _ = fs::remove_dir_all(&tmp_dir);

    println!(
        "{} Published {} v{} to {}",
        "✓".green(),
        pkg_name,
        pkg_version,
        registry_url
    );
    Ok(())
}

/// Yank a published package version from the registry
pub(crate) fn cmd_pkg_yank(
    name: &str,
    version: &str,
    token: Option<String>,
    registry: Option<String>,
    verbose: bool,
) -> Result<(), String> {
    let registry_url = registry.unwrap_or_else(|| "https://registry.vais.dev".to_string());

    let auth_token = token
        .or_else(|| load_credentials_token(&registry_url))
        .ok_or_else(|| {
            "authentication token required. Use --token or run `vaisc pkg login` first".to_string()
        })?;

    let yank_url = format!(
        "{}/packages/{}/{}/yank",
        registry_url.trim_end_matches('/'),
        name,
        version
    );

    if verbose {
        println!(
            "{} Yanking {}@{} from {}",
            "Info".cyan(),
            name,
            version,
            registry_url
        );
    }

    ureq::post(&yank_url)
        .set("Authorization", &format!("Bearer {}", auth_token))
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(code, resp) => {
                let msg = resp.into_string().unwrap_or_default();
                format!("yank failed (HTTP {}): {}", code, msg)
            }
            _ => format!("yank failed: {}", e),
        })?;

    println!(
        "{} Yanked {}@{} from {}",
        "✓".green(),
        name,
        version,
        registry_url
    );
    Ok(())
}

/// Login to a package registry and store credentials
pub(crate) fn cmd_pkg_login(registry: Option<String>, verbose: bool) -> Result<(), String> {
    let registry_url = registry.unwrap_or_else(|| "https://registry.vais.dev".to_string());

    println!("{} Logging in to {}", "Info".cyan(), registry_url);

    // Prompt for username
    eprint!("Username: ");
    let mut username = String::new();
    std::io::stdin()
        .read_line(&mut username)
        .map_err(|e| format!("failed to read username: {}", e))?;
    let username = username.trim().to_string();

    if username.is_empty() {
        return Err("username cannot be empty".to_string());
    }

    // Prompt for password
    eprint!("Password: ");
    let mut password = String::new();
    std::io::stdin()
        .read_line(&mut password)
        .map_err(|e| format!("failed to read password: {}", e))?;
    let password = password.trim().to_string();

    if password.is_empty() {
        return Err("password cannot be empty".to_string());
    }

    let login_url = format!("{}/auth/login", registry_url.trim_end_matches('/'));

    if verbose {
        println!("  Authenticating as {}...", username);
    }

    let response = ureq::post(&login_url)
        .send_json(serde_json::json!({
            "username": username,
            "password": password,
        }))
        .map_err(|e| match e {
            ureq::Error::Status(code, resp) => {
                let msg = resp.into_string().unwrap_or_default();
                format!("login failed (HTTP {}): {}", code, msg)
            }
            _ => format!("login failed: {}", e),
        })?;

    let body: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("failed to parse login response: {}", e))?;

    let token = body
        .get("token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "login response did not contain a token".to_string())?
        .to_string();

    // Save token to ~/.vais/credentials.toml
    let home = dirs::home_dir().ok_or_else(|| "could not determine home directory".to_string())?;
    let vais_dir = home.join(".vais");
    fs::create_dir_all(&vais_dir).map_err(|e| format!("failed to create ~/.vais: {}", e))?;

    let creds_path = vais_dir.join("credentials.toml");

    // Load existing credentials or start fresh
    let mut creds: toml::Value = if creds_path.exists() {
        let content = fs::read_to_string(&creds_path)
            .map_err(|e| format!("failed to read credentials: {}", e))?;
        content
            .parse()
            .unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()))
    } else {
        toml::Value::Table(toml::map::Map::new())
    };

    // Store token under the registry URL key
    if let Some(table) = creds.as_table_mut() {
        let mut registry_table = toml::map::Map::new();
        registry_table.insert("token".to_string(), toml::Value::String(token));
        table.insert(registry_url.clone(), toml::Value::Table(registry_table));
    }

    let creds_content = toml::to_string_pretty(&creds)
        .map_err(|e| format!("failed to serialize credentials: {}", e))?;
    fs::write(&creds_path, creds_content)
        .map_err(|e| format!("failed to write credentials: {}", e))?;

    println!(
        "{} Logged in to {} as {}",
        "✓".green(),
        registry_url,
        username
    );
    println!("  Token saved to {}", creds_path.display());
    Ok(())
}

/// Load authentication token from ~/.vais/credentials.toml for a given registry
pub(crate) fn load_credentials_token(registry_url: &str) -> Option<String> {
    let home = dirs::home_dir()?;
    let creds_path = home.join(".vais").join("credentials.toml");
    let content = fs::read_to_string(&creds_path).ok()?;
    let creds: toml::Value = content.parse().ok()?;
    creds
        .get(registry_url)?
        .get("token")?
        .as_str()
        .map(|s| s.to_string())
}

/// Parse package spec (name or name@version)
pub(crate) fn parse_package_spec(spec: &str) -> (String, String) {
    if let Some(idx) = spec.find('@') {
        let name = &spec[..idx];
        let version = &spec[idx + 1..];
        (name.to_string(), version.to_string())
    } else {
        (spec.to_string(), "*".to_string())
    }
}

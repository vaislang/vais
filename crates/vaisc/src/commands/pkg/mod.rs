//! Package management commands.

mod build;
mod check;
mod install;
mod publish;
mod registry;
mod util;

use clap::Subcommand;

// Re-export helper functions used by main.rs
pub(crate) use self::build::{cmd_install, cmd_uninstall};

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
        output: std::path::PathBuf,

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
    use crate::package;
    use colored::Colorize;
    use std::env;
    use std::fs;

    let cwd = env::current_dir().map_err(|e| format!("failed to get current directory: {}", e))?;

    match cmd {
        PkgCommands::Init { name } => {
            package::init_package(&cwd, name.as_deref()).map_err(|e| e.to_string())?;
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
        } => build::cmd_pkg_build(
            &cwd,
            release,
            debug,
            hot,
            workspace,
            features,
            all_features,
            no_default_features,
            verbose,
        ),

        PkgCommands::Check {
            workspace,
            features,
            all_features,
            no_default_features,
        } => check::cmd_pkg_check(
            &cwd,
            workspace,
            features,
            all_features,
            no_default_features,
            verbose,
        ),

        PkgCommands::Add {
            name,
            path,
            version,
        } => install::cmd_pkg_add(&cwd, &name, path, version),

        PkgCommands::Remove { name } => install::cmd_pkg_remove(&cwd, &name),

        PkgCommands::Clean => {
            let pkg_dir = package::find_manifest(&cwd)
                .ok_or_else(|| "could not find vais.toml".to_string())?;

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
        } => install::cmd_pkg_install(&cwd, packages, update, offline, verbose),

        PkgCommands::Update { packages, offline } => {
            install::cmd_pkg_update(&cwd, packages, offline, verbose)
        }

        PkgCommands::Search {
            query,
            limit,
            offline,
            sort,
            category,
            keyword,
        } => registry::cmd_pkg_search(
            &query,
            limit,
            offline,
            verbose,
            &sort,
            category.as_deref(),
            keyword.as_deref(),
        ),

        PkgCommands::Info { name } => registry::cmd_pkg_info(&name, verbose),

        PkgCommands::Cache { action } => registry::cmd_pkg_cache(action, verbose),

        PkgCommands::Audit { format } => util::cmd_pkg_audit(&cwd, &format, verbose),

        PkgCommands::Publish {
            registry,
            token,
            dry_run,
        } => publish::cmd_pkg_publish(&cwd, registry, token, dry_run, verbose),

        PkgCommands::Yank {
            name,
            version,
            token,
            registry,
        } => publish::cmd_pkg_yank(&name, &version, token, registry, verbose),

        PkgCommands::Login { registry } => publish::cmd_pkg_login(registry, verbose),

        PkgCommands::Tree { all, depth } => {
            let pkg_dir = package::find_manifest(&cwd).ok_or_else(|| {
                "could not find vais.toml in current directory or parents".to_string()
            })?;
            let manifest = package::load_manifest(&pkg_dir).map_err(|e| e.to_string())?;
            util::cmd_pkg_tree(&manifest, &pkg_dir, all, depth, verbose)
        }

        PkgCommands::Doc {
            output,
            format,
            open,
        } => {
            let pkg_dir = package::find_manifest(&cwd).ok_or_else(|| {
                "could not find vais.toml in current directory or parents".to_string()
            })?;
            let manifest = package::load_manifest(&pkg_dir).map_err(|e| e.to_string())?;
            util::cmd_pkg_doc(&manifest, &pkg_dir, &output, &format, open, verbose)
        }

        PkgCommands::Vendor => {
            let pkg_dir = package::find_manifest(&cwd)
                .ok_or_else(|| "could not find vais.toml".to_string())?;
            let manifest = package::load_manifest(&pkg_dir).map_err(|e| e.to_string())?;
            util::cmd_pkg_vendor(&pkg_dir, &manifest, verbose)
        }

        PkgCommands::Package { list } => {
            let pkg_dir = package::find_manifest(&cwd)
                .ok_or_else(|| "could not find vais.toml".to_string())?;
            let manifest = package::load_manifest(&pkg_dir).map_err(|e| e.to_string())?;
            util::cmd_pkg_package(&pkg_dir, &manifest, list, verbose)
        }

        PkgCommands::Metadata { format } => {
            let pkg_dir = package::find_manifest(&cwd)
                .ok_or_else(|| "could not find vais.toml".to_string())?;
            let manifest = package::load_manifest(&pkg_dir).map_err(|e| e.to_string())?;
            util::cmd_pkg_metadata(&manifest, &format)
        }

        PkgCommands::Owner { add, remove, list } => {
            let pkg_dir = package::find_manifest(&cwd)
                .ok_or_else(|| "could not find vais.toml".to_string())?;
            let manifest = package::load_manifest(&pkg_dir).map_err(|e| e.to_string())?;
            util::cmd_pkg_owner(&pkg_dir, &manifest, add, remove, list)
        }

        PkgCommands::Verify => {
            let pkg_dir = package::find_manifest(&cwd)
                .ok_or_else(|| "could not find vais.toml".to_string())?;
            util::cmd_pkg_verify(&pkg_dir, verbose)
        }
    }
}

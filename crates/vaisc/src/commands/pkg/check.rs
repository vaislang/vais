//! Type-checking package commands.

use crate::commands::simple::cmd_check;
use crate::package;
use colored::Colorize;
use std::path::Path;
use vais_plugin::PluginRegistry;

/// Type-check the package without compiling
pub(super) fn cmd_pkg_check(
    cwd: &Path,
    workspace: bool,
    features: Option<String>,
    all_features: bool,
    no_default_features: bool,
    verbose: bool,
) -> Result<(), String> {
    use package::*;

    if workspace {
        return cmd_pkg_check_workspace(cwd, verbose);
    }

    let pkg_dir =
        find_manifest(cwd).ok_or_else(|| "could not find vais.toml".to_string())?;

    let manifest = load_manifest(&pkg_dir).map_err(|e| e.to_string())?;

    // Auto-detect workspace
    if manifest.workspace.is_some() {
        return cmd_pkg_check_workspace(&pkg_dir, verbose);
    }

    // Resolve feature flags
    let enabled_features = super::build::resolve_feature_flags(
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

/// Type-check all workspace members
pub(super) fn cmd_pkg_check_workspace(start_dir: &Path, verbose: bool) -> Result<(), String> {
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

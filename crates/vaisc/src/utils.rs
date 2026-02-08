//! Utility functions (walkdir, plugin loading, diagnostics).

use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use vais_plugin::{find_config, Diagnostic, DiagnosticLevel, PluginRegistry, PluginsConfig};

pub(crate) fn walkdir(dir: &PathBuf, ext: &str) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(walkdir(&path, ext));
            } else if path.extension().is_some_and(|e| e == ext) {
                result.push(path);
            }
        }
    }
    result
}

/// Load plugins from configuration and CLI arguments
pub(crate) fn load_plugins(
    extra_plugins: &[PathBuf],
    verbose: bool,
    allow_plugins: bool,
) -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    registry.set_allow_plugins(allow_plugins);

    // Load configuration file if present
    let config = if let Some(config_path) = find_config() {
        if verbose {
            println!(
                "{} {}",
                "Loading plugin config".cyan(),
                config_path.display()
            );
        }
        match PluginsConfig::load(&config_path) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("{}: {}", "Warning".yellow(), e);
                PluginsConfig::empty()
            }
        }
    } else {
        PluginsConfig::empty()
    };

    // Load plugins from config paths
    for plugin_path in &config.plugins.path {
        match registry.load_from_path(plugin_path) {
            Ok(info) => {
                if verbose {
                    println!(
                        "  {} {} v{}",
                        "Loaded plugin".green(),
                        info.name,
                        info.version
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "{}: Failed to load '{}': {}",
                    "Warning".yellow(),
                    plugin_path.display(),
                    e
                );
            }
        }
    }

    // Load extra plugins from CLI
    for plugin_path in extra_plugins {
        match registry.load_from_path(plugin_path) {
            Ok(info) => {
                if verbose {
                    println!(
                        "  {} {} v{}",
                        "Loaded plugin".green(),
                        info.name,
                        info.version
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "{}: Failed to load '{}': {}",
                    "Warning".yellow(),
                    plugin_path.display(),
                    e
                );
            }
        }
    }

    // Apply configuration to loaded plugins
    for (name, plugin_config) in &config.plugins.config {
        if let Err(e) = registry.configure(name, plugin_config) {
            eprintln!(
                "{}: Failed to configure '{}': {}",
                "Warning".yellow(),
                name,
                e
            );
        }
    }

    registry
}

/// Profile-Guided Optimization workflow
///
/// Automates the 3-step PGO process:
/// 1. Build with instrumentation (--profile-generate)
/// 2. Run to collect profile data
/// 3. Merge profiles and rebuild with optimization (--profile-use)
pub(crate) fn print_suggested_fixes(error: &vais_types::TypeError, _source: &str) {
    use vais_types::TypeError;

    eprintln!("\n{} Suggested fixes:", "💡".cyan().bold());

    match error {
        TypeError::UndefinedVar {
            name, suggestion, ..
        } => {
            if let Some(similar) = suggestion {
                eprintln!("  {} Did you mean '{}'?", "•".green(), similar);
            } else {
                eprintln!("  {} Define variable: L {}: i64 = 0", "•".green(), name);
            }
        }
        TypeError::UndefinedFunction {
            name, suggestion, ..
        } => {
            if let Some(similar) = suggestion {
                eprintln!("  {} Did you mean '{}'?", "•".green(), similar);
            } else {
                // Check if it's a common standard library function
                let common_funcs = [
                    ("sqrt", "std/math"),
                    ("sin", "std/math"),
                    ("cos", "std/math"),
                    ("abs", "std/math"),
                    ("read_i64", "std/io"),
                ];

                for (func, module) in &common_funcs {
                    if name == *func {
                        eprintln!("  {} Add import: U {}", "•".green(), module);
                        break;
                    }
                }
            }
        }
        TypeError::Mismatch {
            expected, found, ..
        } => {
            if (expected == "i64" && found == "f64") || (expected == "f64" && found == "i64") {
                eprintln!("  {} Add type cast: value as {}", "•".green(), expected);
            }
        }
        TypeError::ImmutableAssign(name, _) => {
            eprintln!("  {} Declare as mutable: {}: mut Type", "•".green(), name);
        }
        _ => {
            // For other errors, show the help message if available
            if let Some(help) = error.help() {
                eprintln!("  {} {}", "•".green(), help);
            }
        }
    }
    eprintln!();
}

pub(crate) fn print_plugin_diagnostics(diagnostics: &[Diagnostic], source: &str, path: &Path) {
    let filename = path.to_str().unwrap_or("unknown");
    let lines: Vec<&str> = source.lines().collect();

    for diag in diagnostics {
        let (_level_str, level_colored) = match diag.level {
            DiagnosticLevel::Error => ("error", "error".red().bold()),
            DiagnosticLevel::Warning => ("warning", "warning".yellow().bold()),
            DiagnosticLevel::Info => ("info", "info".cyan().bold()),
            DiagnosticLevel::Hint => ("hint", "hint".blue().bold()),
        };

        if let Some(span) = diag.span {
            // Find line and column from span
            let mut line_num = 1;
            let mut col = 1;
            let mut char_count = 0;
            for (i, line) in lines.iter().enumerate() {
                let line_len = line.len() + 1; // +1 for newline
                if char_count + line_len > span.start {
                    line_num = i + 1;
                    col = span.start - char_count + 1;
                    break;
                }
                char_count += line_len;
            }

            let underline_len = (span.end - span.start).max(1);
            let source_line = lines.get(line_num - 1).unwrap_or(&"");

            eprintln!("{}{} {}", level_colored, ":".bold(), diag.message);
            eprintln!(
                "  {} {}:{}:{}",
                "-->".blue().bold(),
                filename,
                line_num,
                col
            );
            eprintln!("   {}", "|".blue().bold());
            eprintln!("{:4}{} {}", line_num, "|".blue().bold(), source_line);
            eprintln!(
                "   {} {}{}",
                "|".blue().bold(),
                " ".repeat(col.saturating_sub(1)),
                "^".repeat(underline_len).yellow()
            );
            if let Some(help) = &diag.help {
                eprintln!("   {} {} {}", "=".blue().bold(), "help:".cyan(), help);
            }
            eprintln!();
        } else {
            eprintln!("{}: {}", level_colored, diag.message);
            if let Some(help) = &diag.help {
                eprintln!("  {} {}", "help:".cyan(), help);
            }
        }
    }
}

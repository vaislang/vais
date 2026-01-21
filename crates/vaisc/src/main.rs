//! Vais Compiler CLI
//!
//! The `vaisc` command compiles Vais source files to LLVM IR or native binaries.

mod doc_gen;
mod repl;
mod error_formatter;
mod incremental;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

use vais_ast::{Item, Module};
use vais_codegen::{CodeGenerator, TargetTriple};
use vais_codegen::optimize::{optimize_ir, OptLevel};
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::{TypeChecker};
use vais_i18n::Locale;
use vais_plugin::{PluginRegistry, PluginsConfig, find_config, Diagnostic, DiagnosticLevel};

#[derive(Parser)]
#[command(name = "vaisc")]
#[command(author = "Vais Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Vais compiler - AI-optimized systems programming language")]
#[command(long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input source file (.vais)
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,

    /// Output file (default: input with .ll extension)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Emit LLVM IR only (don't compile to binary)
    #[arg(long)]
    emit_ir: bool,

    /// Show tokens (lexer output)
    #[arg(long)]
    show_tokens: bool,

    /// Show AST (parser output)
    #[arg(long)]
    show_ast: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Set the locale for error messages (en, ko, ja)
    #[arg(long, value_name = "LOCALE", global = true)]
    locale: Option<String>,

    /// Disable all plugins
    #[arg(long, global = true)]
    no_plugins: bool,

    /// Load additional plugin from file
    #[arg(long, value_name = "PATH", global = true)]
    plugin: Vec<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a Vais source file
    Build {
        /// Input source file
        input: PathBuf,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Emit LLVM IR only
        #[arg(long)]
        emit_ir: bool,

        /// Optimization level (0-3)
        #[arg(short = 'O', long, default_value = "0")]
        opt_level: u8,

        /// Include debug information (DWARF) for source-level debugging
        #[arg(short = 'g', long)]
        debug: bool,

        /// Target triple for compilation (default: native)
        /// Examples: wasm32-unknown-unknown, wasm32-wasi, x86_64-unknown-linux-gnu
        #[arg(long, value_name = "TRIPLE")]
        target: Option<String>,

        /// Force full rebuild, ignoring incremental cache
        #[arg(long)]
        force_rebuild: bool,
    },

    /// Run a Vais source file
    Run {
        /// Input source file
        input: PathBuf,

        /// Arguments to pass to the program
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Check a Vais source file for errors
    Check {
        /// Input source file
        input: PathBuf,
    },

    /// Start interactive REPL
    Repl,

    /// Generate documentation
    Doc {
        /// Input source file or directory
        input: PathBuf,

        /// Output directory for documentation
        #[arg(short, long, default_value = "docs")]
        output: PathBuf,

        /// Output format (markdown or html)
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },

    /// Show version information
    Version,

    /// Format Vais source files
    Fmt {
        /// Input source file(s) or directory
        input: PathBuf,

        /// Write formatted output to stdout instead of modifying files
        #[arg(long)]
        check: bool,

        /// Indentation size (default: 4)
        #[arg(long, default_value = "4")]
        indent: usize,
    },
}

fn main() {
    let cli = Cli::parse();

    // Initialize i18n system
    let locale = cli.locale
        .as_ref()
        .and_then(|s| Locale::from_str(s));
    vais_i18n::init(locale);

    // Load plugins
    let plugins = if cli.no_plugins {
        PluginRegistry::new()
    } else {
        load_plugins(&cli.plugin, cli.verbose)
    };

    let result = match cli.command {
        Some(Commands::Build { input, output, emit_ir, opt_level, debug, target, force_rebuild }) => {
            let target_triple = target.as_ref()
                .and_then(|s| TargetTriple::from_str(s))
                .unwrap_or(TargetTriple::Native);
            cmd_build(&input, output, emit_ir, opt_level, debug, cli.verbose, &plugins, target_triple, force_rebuild)
        }
        Some(Commands::Run { input, args }) => {
            cmd_run(&input, &args, cli.verbose, &plugins)
        }
        Some(Commands::Check { input }) => {
            cmd_check(&input, cli.verbose, &plugins)
        }
        Some(Commands::Repl) => {
            repl::run()
        }
        Some(Commands::Doc { input, output, format }) => {
            doc_gen::run(&input, &output, &format)
        }
        Some(Commands::Version) => {
            println!("{} {}", "vaisc".bold(), env!("CARGO_PKG_VERSION"));
            println!("Vais 0.0.1 - AI-optimized systems programming language");
            Ok(())
        }
        Some(Commands::Fmt { input, check, indent }) => {
            cmd_fmt(&input, check, indent)
        }
        None => {
            // Direct file compilation
            if let Some(input) = cli.input {
                cmd_build(&input, cli.output, cli.emit_ir, 0, false, cli.verbose, &plugins, TargetTriple::Native, false)
            } else {
                println!("{}", "Usage: vaisc <FILE.vais> or vaisc build <FILE.vais>".yellow());
                println!("Run 'vaisc --help' for more information.");
                Ok(())
            }
        }
    };

    if let Err(e) = result {
        eprintln!("{}: {}", "error".red().bold(), e);
        exit(1);
    }
}

fn cmd_build(
    input: &PathBuf,
    output: Option<PathBuf>,
    emit_ir: bool,
    opt_level: u8,
    debug: bool,
    verbose: bool,
    plugins: &PluginRegistry,
    target: TargetTriple,
    force_rebuild: bool,
) -> Result<(), String> {
    use incremental::{IncrementalCache, CompilationOptions, get_cache_dir};

    // Initialize incremental compilation cache
    let cache_dir = get_cache_dir(input);
    let mut cache = IncrementalCache::new(cache_dir).ok();

    // Set compilation options for cache validity checking
    if let Some(ref mut c) = cache {
        c.set_compilation_options(CompilationOptions {
            opt_level,
            debug,
            target_triple: target.triple_str().to_string(),
        });
    }

    // Check if we can skip compilation (only when not forcing rebuild)
    if !force_rebuild {
        if let Some(ref mut c) = cache {
            match c.detect_changes(input) {
                Ok(dirty_set) => {
                    if dirty_set.is_empty() {
                        if verbose {
                            println!("{} {} (no changes detected)", "Skipping".cyan().bold(), input.display());
                            let stats = c.stats();
                            println!("  {} files cached, {} dependencies tracked",
                                stats.total_files, stats.total_dependencies);
                        }
                        // Still need to output the binary path if not emit_ir
                        if !emit_ir {
                            let default_ext = match target {
                                TargetTriple::Wasm32Unknown | TargetTriple::Wasi => "wasm",
                                _ => "",
                            };
                            let bin_path = output.clone().unwrap_or_else(|| {
                                input.with_extension(default_ext)
                            });
                            if bin_path.exists() {
                                if !verbose {
                                    println!("{}", bin_path.display());
                                }
                                return Ok(());
                            }
                        } else {
                            let ir_path = output.clone().unwrap_or_else(|| input.with_extension("ll"));
                            if ir_path.exists() {
                                if !verbose {
                                    println!("{}", ir_path.display());
                                }
                                return Ok(());
                            }
                        }
                    } else if verbose {
                        println!("{} {} file(s) changed", "Rebuilding".yellow().bold(), dirty_set.count());
                    }
                }
                Err(e) => {
                    if verbose {
                        println!("{} Cache check failed: {}", "Warning".yellow(), e);
                    }
                }
            }
        }
    } else if verbose {
        println!("{} (--force-rebuild)", "Full rebuild".yellow().bold());
    }

    // Read source for error reporting
    let main_source = fs::read_to_string(input)
        .map_err(|e| format!("Cannot read '{}': {}", input.display(), e))?;

    // Parse main file and resolve imports
    let mut loaded_modules: HashSet<PathBuf> = HashSet::new();
    let merged_ast = load_module_with_imports_internal(input, &mut loaded_modules, verbose, &main_source)?;

    if verbose {
        println!("  {} total items (including imports)", merged_ast.items.len());
    }

    // Run lint plugins
    if !plugins.is_empty() {
        let diagnostics = plugins.run_lint(&merged_ast);
        if !diagnostics.is_empty() {
            print_plugin_diagnostics(&diagnostics, &main_source, input);

            // Check if any errors (not just warnings)
            let has_errors = diagnostics.iter().any(|d| d.level == DiagnosticLevel::Error);
            if has_errors {
                return Err("Plugin lint check failed".to_string());
            }
        }
    }

    // Run transform plugins
    let transformed_ast = if !plugins.is_empty() {
        plugins.run_transform(merged_ast)
            .map_err(|e| format!("Plugin transform error: {}", e))?
    } else {
        merged_ast
    };

    // Type check
    let mut checker = TypeChecker::new();
    if let Err(e) = checker.check_module(&transformed_ast) {
        // Format error with source context
        return Err(error_formatter::format_type_error(&e, &main_source, input));
    }

    if verbose {
        println!("  {}", "Type check passed".green());
    }

    // Generate LLVM IR
    let module_name = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");

    let mut codegen = CodeGenerator::new_with_target(module_name, target.clone());

    if verbose && !matches!(target, TargetTriple::Native) {
        println!("  {} {}", "Target:".cyan(), target.triple_str());
    }

    // Enable debug info if requested
    if debug {
        let source_file = input.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown.vais");
        let source_dir = input.parent()
            .and_then(|p| p.to_str())
            .unwrap_or(".");
        codegen.enable_debug(source_file, source_dir, &main_source);

        if verbose {
            println!("  {}", "Debug info enabled".cyan());
        }
    }

    let raw_ir = codegen.generate_module(&transformed_ast)
        .map_err(|e| format!("Codegen error: {}", e))?;

    // Apply optimization passes before emitting IR
    // When debug is enabled, disable optimizations to preserve debuggability
    let effective_opt_level = if debug { 0 } else { opt_level };
    let opt = match effective_opt_level {
        0 => OptLevel::O0,
        1 => OptLevel::O1,
        2 => OptLevel::O2,
        _ => OptLevel::O3,
    };
    let ir = optimize_ir(&raw_ir, opt);

    // Run plugin optimizations
    let plugin_opt = match effective_opt_level {
        0 => vais_plugin::OptLevel::O0,
        1 => vais_plugin::OptLevel::O1,
        2 => vais_plugin::OptLevel::O2,
        _ => vais_plugin::OptLevel::O3,
    };
    let ir = if !plugins.is_empty() {
        plugins.run_optimize(&ir, plugin_opt)
            .map_err(|e| format!("Plugin optimize error: {}", e))?
    } else {
        ir
    };

    if verbose && opt_level > 0 && !debug {
        println!("{} Applied Vais IR optimizations (O{})", "Optimizing".cyan().bold(), opt_level);
    } else if verbose && debug && opt_level > 0 {
        println!("{} Optimizations disabled for debug build", "Note".yellow().bold());
    }

    // Determine output paths
    let ir_path = if emit_ir {
        // If emitting IR, use the specified output or default to .ll
        output.clone().unwrap_or_else(|| input.with_extension("ll"))
    } else {
        // For binary compilation, always use .ll extension for intermediate IR
        input.with_extension("ll")
    };

    // Write IR
    fs::write(&ir_path, &ir)
        .map_err(|e| format!("Cannot write '{}': {}", ir_path.display(), e))?;

    if verbose || emit_ir {
        println!("{} {}", "Wrote".green().bold(), ir_path.display());
    }

    // Run codegen plugins (generate additional files)
    if !plugins.is_empty() {
        let output_dir = ir_path.parent().unwrap_or(Path::new("."));
        match plugins.run_codegen(&transformed_ast, output_dir) {
            Ok(generated_files) => {
                for file in generated_files {
                    if verbose {
                        println!("{} {} (plugin)", "Generated".green().bold(), file.display());
                    }
                }
            }
            Err(e) => {
                eprintln!("{}: Plugin codegen: {}", "Warning".yellow(), e);
            }
        }
    }

    // If not emit_ir only, compile to binary
    if !emit_ir {
        // Determine output extension based on target
        let default_ext = match target {
            TargetTriple::Wasm32Unknown | TargetTriple::Wasi => "wasm",
            _ => "",
        };
        let bin_path = output.unwrap_or_else(|| {
            input.with_extension(default_ext)
        });

        compile_ir_to_binary(&ir_path, &bin_path, effective_opt_level, debug, verbose, &target)?;
    }

    // Update incremental compilation cache after successful build
    if let Some(ref mut c) = cache {
        // Update file metadata for all loaded modules
        for loaded_path in &loaded_modules {
            if let Err(e) = c.update_file(loaded_path) {
                if verbose {
                    eprintln!("{}: Cache update for '{}': {}", "Warning".yellow(), loaded_path.display(), e);
                }
            }
        }

        // Persist cache to disk
        if let Err(e) = c.persist() {
            if verbose {
                eprintln!("{}: Cannot save cache: {}", "Warning".yellow(), e);
            }
        } else if verbose {
            let stats = c.stats();
            println!("{} {} files, {} dependencies",
                "Cache updated:".cyan(),
                stats.total_files,
                stats.total_dependencies);
        }
    }

    Ok(())
}

// Note: Error formatting functions have been moved to the error_formatter module
// They are now re-exported through error_formatter::format_type_error and error_formatter::format_parse_error
// This provides a centralized location for error handling logic

/// Load a module and recursively resolve its imports
fn load_module_with_imports(
    path: &PathBuf,
    loaded: &mut HashSet<PathBuf>,
    verbose: bool,
) -> Result<Module, String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read '{}': {}", path.display(), e))?;
    load_module_with_imports_internal(path, loaded, verbose, &source)
}

/// Internal function to load a module with source already read
fn load_module_with_imports_internal(
    path: &PathBuf,
    loaded: &mut HashSet<PathBuf>,
    verbose: bool,
    source: &str,
) -> Result<Module, String> {
    // Canonicalize path to avoid duplicate loading
    let canonical = path.canonicalize()
        .map_err(|e| format!("Cannot resolve path '{}': {}", path.display(), e))?;

    // Skip if already loaded
    if loaded.contains(&canonical) {
        return Ok(Module { items: vec![] });
    }
    loaded.insert(canonical.clone());

    if verbose {
        println!("{} {}", "Compiling".green().bold(), path.display());
    }

    let _tokens = tokenize(&source)
        .map_err(|e| format!("Lexer error in '{}': {}", path.display(), e))?;

    let ast = parse(&source)
        .map_err(|e| error_formatter::format_parse_error(&e, source, path))?;

    if verbose {
        println!("  {} items", ast.items.len());
    }

    // Collect items, processing imports
    let mut all_items = Vec::new();
    let base_dir = path.parent().unwrap_or(Path::new("."));

    for item in ast.items {
        match &item.node {
            Item::Use(use_stmt) => {
                // Resolve import path
                let module_path = resolve_import_path(base_dir, &use_stmt.path)?;

                if verbose {
                    println!("  {} {}", "Importing".cyan(), module_path.display());
                }

                // Recursively load the imported module
                let imported = load_module_with_imports(&module_path, loaded, verbose)?;
                all_items.extend(imported.items);
            }
            _ => {
                all_items.push(item);
            }
        }
    }

    Ok(Module { items: all_items })
}

/// Get the standard library path
fn get_std_path() -> Option<PathBuf> {
    // Try multiple locations for std library:
    // 1. Relative to current executable (for installed vaisc)
    // 2. Current working directory (for development)
    // 3. VAIS_STD_PATH environment variable

    if let Ok(std_path) = std::env::var("VAIS_STD_PATH") {
        let path = PathBuf::from(std_path);
        if path.exists() {
            return Some(path);
        }
    }

    // Try relative to executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let std_path = exe_dir.join("std");
            if std_path.exists() {
                return Some(std_path);
            }
            // Also try ../std (for cargo run)
            let std_path = exe_dir.parent().and_then(|p| Some(p.join("std")));
            if let Some(path) = std_path {
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }

    // Try current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let std_path = cwd.join("std");
        if std_path.exists() {
            return Some(std_path);
        }
    }

    None
}

/// Resolve import path to file path with security validation
fn resolve_import_path(base_dir: &Path, path: &[vais_ast::Spanned<String>]) -> Result<PathBuf, String> {
    if path.is_empty() {
        return Err("Empty import path".to_string());
    }

    // Check if this is a std library import (starts with "std")
    let is_std_import = path.first().map(|s| s.node.as_str()) == Some("std");

    let search_base = if is_std_import {
        // For std imports, use the standard library path
        match get_std_path() {
            Some(std_path) => std_path.parent().unwrap_or(Path::new(".")).to_path_buf(),
            None => return Err("Cannot find Vais standard library. Set VAIS_STD_PATH or run from project root.".to_string()),
        }
    } else {
        base_dir.to_path_buf()
    };

    // Canonicalize the search base to get the absolute path
    // This resolves symlinks and normalizes the path
    let canonical_base = search_base.canonicalize()
        .map_err(|_| format!("Cannot resolve base directory: {}", search_base.display()))?;

    // Convert module path to file path
    // e.g., `U utils` -> `utils.vais` or `utils/mod.vais`
    // e.g., `U std/option` -> `std/option.vais`
    let mut file_path = search_base;
    for (i, segment) in path.iter().enumerate() {
        if i == path.len() - 1 {
            // Last segment - try as file first, then as directory with mod.vais
            let as_file = file_path.join(format!("{}.vais", segment.node));
            let as_dir = file_path.join(&segment.node).join("mod.vais");

            // Try file path first
            if as_file.exists() {
                return validate_and_canonicalize_import(&as_file, &canonical_base);
            } else if as_dir.exists() {
                return validate_and_canonicalize_import(&as_dir, &canonical_base);
            } else {
                return Err(format!(
                    "Cannot find module '{}': tried '{}' and '{}'",
                    segment.node,
                    as_file.display(),
                    as_dir.display()
                ));
            }
        } else {
            file_path = file_path.join(&segment.node);
        }
    }

    Err(format!("Invalid import path"))
}

/// Validate and canonicalize an import path for security
///
/// This function performs critical security checks:
/// 1. Resolves the real path (following symlinks)
/// 2. Ensures the resolved path is within allowed directories
/// 3. Prevents path traversal attacks (../)
/// 4. Prevents symlink attacks
fn validate_and_canonicalize_import(path: &Path, allowed_base: &Path) -> Result<PathBuf, String> {
    // Canonicalize the path to resolve symlinks and get absolute path
    let canonical_path = path.canonicalize()
        .map_err(|e| format!("Cannot access file '{}': {}", path.display(), e))?;

    // Get the project root for additional validation
    let project_root = std::env::current_dir()
        .and_then(|p| p.canonicalize())
        .ok();

    // Get std library path for validation
    let std_root = get_std_path()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .and_then(|p| p.canonicalize().ok());

    // Check if the canonical path is within allowed directories
    let is_within_allowed = canonical_path.starts_with(allowed_base);

    // Also check if it's within project root or std library
    let is_within_project = project_root.as_ref()
        .map(|root| canonical_path.starts_with(root))
        .unwrap_or(false);

    let is_within_std = std_root.as_ref()
        .map(|root| canonical_path.starts_with(root))
        .unwrap_or(false);

    if !is_within_allowed && !is_within_project && !is_within_std {
        // Security: Path traversal or symlink attack detected
        return Err(format!(
            "Import path '{}' is outside allowed directories",
            path.display()
        ));
    }

    // Verify the file has .vais extension for additional safety
    if canonical_path.extension().map(|e| e != "vais").unwrap_or(true) {
        return Err(format!(
            "Invalid import file type: '{}' (only .vais files allowed)",
            canonical_path.display()
        ));
    }

    Ok(canonical_path)
}

fn compile_ir_to_binary(
    ir_path: &Path,
    bin_path: &Path,
    opt_level: u8,
    debug: bool,
    verbose: bool,
    target: &TargetTriple,
) -> Result<(), String> {
    match target {
        TargetTriple::Wasm32Unknown => compile_to_wasm32(ir_path, bin_path, opt_level, verbose),
        TargetTriple::Wasi => compile_to_wasi(ir_path, bin_path, opt_level, verbose),
        _ => compile_to_native(ir_path, bin_path, opt_level, debug, verbose),
    }
}

fn compile_to_native(
    ir_path: &Path,
    bin_path: &Path,
    opt_level: u8,
    debug: bool,
    verbose: bool,
) -> Result<(), String> {
    let opt_flag = format!("-O{}", opt_level.min(3));

    let mut args = vec![
        opt_flag,
        "-Wno-override-module".to_string(), // Suppress warning when clang sets target triple
    ];

    // Add debug flag if requested
    if debug {
        args.push("-g".to_string());  // Generate debug symbols
    }

    args.push("-o".to_string());
    args.push(bin_path.to_str()
        .ok_or_else(|| "Invalid UTF-8 in output path".to_string())?
        .to_string());
    args.push(ir_path.to_str()
        .ok_or_else(|| "Invalid UTF-8 in IR path".to_string())?
        .to_string());

    let status = Command::new("clang")
        .args(&args)
        .status();

    match status {
        Ok(s) if s.success() => {
            if verbose {
                if debug {
                    println!("{} {} (with debug symbols)", "Compiled".green().bold(), bin_path.display());
                } else {
                    println!("{} {}", "Compiled".green().bold(), bin_path.display());
                }
            } else {
                println!("{}", bin_path.display());
            }
            Ok(())
        }
        Ok(s) => {
            Err(format!("clang exited with code {}", s.code().unwrap_or(-1)))
        }
        Err(_) => {
            Err("clang not found. Install LLVM/clang or use --emit-ir to output LLVM IR only.".to_string())
        }
    }
}

fn compile_to_wasm32(
    ir_path: &Path,
    bin_path: &Path,
    opt_level: u8,
    verbose: bool,
) -> Result<(), String> {
    let opt_flag = format!("-O{}", opt_level.min(3));

    let ir_str = ir_path.to_str()
        .ok_or_else(|| "Invalid UTF-8 in IR path".to_string())?;
    let bin_str = bin_path.to_str()
        .ok_or_else(|| "Invalid UTF-8 in output path".to_string())?;

    // WebAssembly 32-bit compilation
    let args = vec![
        "--target=wasm32-unknown-unknown",
        "-nostdlib",
        "-Wl,--no-entry",
        "-Wl,--allow-undefined",
        "-Wl,--export-all",
        &opt_flag,
        "-o", bin_str,
        ir_str,
    ];

    let status = Command::new("clang")
        .args(&args)
        .status();

    match status {
        Ok(s) if s.success() => {
            if verbose {
                println!("{} {} (wasm32-unknown-unknown)", "Compiled".green().bold(), bin_path.display());
            } else {
                println!("{}", bin_path.display());
            }
            Ok(())
        }
        Ok(s) => {
            Err(format!("clang wasm32 compilation failed with code {}", s.code().unwrap_or(-1)))
        }
        Err(_) => {
            Err("clang not found. Install LLVM/clang with wasm32 support or use --emit-ir to output LLVM IR only.".to_string())
        }
    }
}

fn compile_to_wasi(
    ir_path: &Path,
    bin_path: &Path,
    opt_level: u8,
    verbose: bool,
) -> Result<(), String> {
    let opt_flag = format!("-O{}", opt_level.min(3));

    let ir_str = ir_path.to_str()
        .ok_or_else(|| "Invalid UTF-8 in IR path".to_string())?;
    let bin_str = bin_path.to_str()
        .ok_or_else(|| "Invalid UTF-8 in output path".to_string())?;

    // WASI compilation
    let args = vec![
        "--target=wasm32-wasi",
        &opt_flag,
        "-o", bin_str,
        ir_str,
    ];

    let status = Command::new("clang")
        .args(&args)
        .status();

    match status {
        Ok(s) if s.success() => {
            if verbose {
                println!("{} {} (wasm32-wasi)", "Compiled".green().bold(), bin_path.display());
            } else {
                println!("{}", bin_path.display());
            }
            Ok(())
        }
        Ok(s) => {
            Err(format!("clang wasi compilation failed with code {}", s.code().unwrap_or(-1)))
        }
        Err(_) => {
            Err("clang not found. Install LLVM/clang with wasi-sdk or use --emit-ir to output LLVM IR only.".to_string())
        }
    }
}

fn cmd_run(input: &PathBuf, args: &[String], verbose: bool, plugins: &PluginRegistry) -> Result<(), String> {
    // Build first (no debug for run command by default, native target only, use incremental cache)
    let bin_path = input.with_extension("");
    cmd_build(input, Some(bin_path.clone()), false, 0, false, verbose, plugins, TargetTriple::Native, false)?;

    // Run the binary
    if verbose {
        println!("{} {}", "Running".green().bold(), bin_path.display());
    }

    let status = Command::new(&bin_path)
        .args(args)
        .status()
        .map_err(|e| format!("Cannot run '{}': {}", bin_path.display(), e))?;

    if !status.success() {
        return Err(format!("Program exited with code {}", status.code().unwrap_or(-1)));
    }

    Ok(())
}

fn cmd_check(input: &PathBuf, verbose: bool, plugins: &PluginRegistry) -> Result<(), String> {
    let source = fs::read_to_string(input)
        .map_err(|e| format!("Cannot read '{}': {}", input.display(), e))?;

    if verbose {
        println!("{} {}", "Checking".green().bold(), input.display());
    }

    // Tokenize
    let _tokens = tokenize(&source)
        .map_err(|e| format!("Lexer error: {}", e))?;

    // Parse
    let ast = parse(&source)
        .map_err(|e| error_formatter::format_parse_error(&e, &source, input))?;

    // Run lint plugins
    if !plugins.is_empty() {
        let diagnostics = plugins.run_lint(&ast);
        if !diagnostics.is_empty() {
            print_plugin_diagnostics(&diagnostics, &source, input);

            // Check if any errors (not just warnings)
            let has_errors = diagnostics.iter().any(|d| d.level == DiagnosticLevel::Error);
            if has_errors {
                return Err("Plugin lint check failed".to_string());
            }
        }
    }

    // Type check
    let mut checker = TypeChecker::new();
    if let Err(e) = checker.check_module(&ast) {
        return Err(error_formatter::format_type_error(&e, &source, input));
    }

    println!("{} No errors found", "OK".green().bold());
    Ok(())
}

fn cmd_fmt(input: &PathBuf, check: bool, indent: usize) -> Result<(), String> {
    use vais_codegen::formatter::{Formatter, FormatConfig};

    // Handle directory or single file
    let files: Vec<PathBuf> = if input.is_dir() {
        walkdir(input, "vais")
    } else {
        vec![input.clone()]
    };

    if files.is_empty() {
        return Err("No .vais files found".to_string());
    }

    let config = FormatConfig {
        indent_size: indent,
        max_line_length: 100,
        use_tabs: false,
    };

    let mut needs_formatting = false;

    for file in &files {
        let source = fs::read_to_string(file)
            .map_err(|e| format!("Cannot read '{}': {}", file.display(), e))?;

        let module = vais_parser::parse(&source)
            .map_err(|e| format!("Parse error in '{}': {}", file.display(), e))?;

        let mut formatter = Formatter::new(config.clone());
        let formatted = formatter.format_module(&module);

        if check {
            // Check mode: just report if file needs formatting
            if source != formatted {
                println!("{} needs formatting: {}", "Would reformat".yellow(), file.display());
                needs_formatting = true;
            }
        } else {
            // Format mode: write back to file
            if source != formatted {
                fs::write(file, &formatted)
                    .map_err(|e| format!("Cannot write '{}': {}", file.display(), e))?;
                println!("{} {}", "Formatted".green().bold(), file.display());
            } else {
                println!("{} {} (no changes)", "OK".green(), file.display());
            }
        }
    }

    if check && needs_formatting {
        return Err("Some files need formatting. Run 'vaisc fmt' to fix.".to_string());
    }

    Ok(())
}

/// Walk directory recursively to find files with given extension
fn walkdir(dir: &PathBuf, ext: &str) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(walkdir(&path, ext));
            } else if path.extension().map_or(false, |e| e == ext) {
                result.push(path);
            }
        }
    }
    result
}

/// Load plugins from configuration and CLI arguments
fn load_plugins(extra_plugins: &[PathBuf], verbose: bool) -> PluginRegistry {
    let mut registry = PluginRegistry::new();

    // Load configuration file if present
    let config = if let Some(config_path) = find_config() {
        if verbose {
            println!("{} {}", "Loading plugin config".cyan(), config_path.display());
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
                    println!("  {} {} v{}", "Loaded plugin".green(), info.name, info.version);
                }
            }
            Err(e) => {
                eprintln!("{}: Failed to load '{}': {}", "Warning".yellow(), plugin_path.display(), e);
            }
        }
    }

    // Load extra plugins from CLI
    for plugin_path in extra_plugins {
        match registry.load_from_path(plugin_path) {
            Ok(info) => {
                if verbose {
                    println!("  {} {} v{}", "Loaded plugin".green(), info.name, info.version);
                }
            }
            Err(e) => {
                eprintln!("{}: Failed to load '{}': {}", "Warning".yellow(), plugin_path.display(), e);
            }
        }
    }

    // Apply configuration to loaded plugins
    for (name, plugin_config) in &config.plugins.config {
        if let Err(e) = registry.configure(name, plugin_config) {
            eprintln!("{}: Failed to configure '{}': {}", "Warning".yellow(), name, e);
        }
    }

    registry
}

/// Print plugin diagnostics
fn print_plugin_diagnostics(diagnostics: &[Diagnostic], source: &str, path: &Path) {
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
            eprintln!("  {} {}:{}:{}", "-->".blue().bold(), filename, line_num, col);
            eprintln!("   {}", "|".blue().bold());
            eprintln!("{:4}{} {}", line_num, "|".blue().bold(), source_line);
            eprintln!("   {} {}{}", "|".blue().bold(), " ".repeat(col.saturating_sub(1)), "^".repeat(underline_len).yellow());
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

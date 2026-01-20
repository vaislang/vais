//! Vais Compiler CLI
//!
//! The `vaisc` command compiles Vais source files to LLVM IR or native binaries.

mod doc_gen;
mod repl;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

use vais_ast::{Item, Module, Span};
use vais_codegen::CodeGenerator;
use vais_codegen::optimize::{optimize_ir, OptLevel};
use vais_lexer::tokenize;
use vais_parser::{parse, ParseError};
use vais_types::{TypeChecker, TypeError, error_report::ErrorReporter};
use vais_i18n::Locale;

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

    let result = match cli.command {
        Some(Commands::Build { input, output, emit_ir, opt_level, debug }) => {
            cmd_build(&input, output, emit_ir, opt_level, debug, cli.verbose)
        }
        Some(Commands::Run { input, args }) => {
            cmd_run(&input, &args, cli.verbose)
        }
        Some(Commands::Check { input }) => {
            cmd_check(&input, cli.verbose)
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
                cmd_build(&input, cli.output, cli.emit_ir, 0, false, cli.verbose)
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
) -> Result<(), String> {
    // Read source for error reporting
    let main_source = fs::read_to_string(input)
        .map_err(|e| format!("Cannot read '{}': {}", input.display(), e))?;

    // Parse main file and resolve imports
    let mut loaded_modules: HashSet<PathBuf> = HashSet::new();
    let merged_ast = load_module_with_imports_internal(input, &mut loaded_modules, verbose, &main_source)?;

    if verbose {
        println!("  {} total items (including imports)", merged_ast.items.len());
    }

    // Type check
    let mut checker = TypeChecker::new();
    if let Err(e) = checker.check_module(&merged_ast) {
        // Format error with source context
        return Err(format_type_error(&e, &main_source, input));
    }

    if verbose {
        println!("  {}", "Type check passed".green());
    }

    // Generate LLVM IR
    let module_name = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");

    let mut codegen = CodeGenerator::new(module_name);

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

    let raw_ir = codegen.generate_module(&merged_ast)
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

    // If not emit_ir only, compile to binary
    if !emit_ir {
        let bin_path = output.unwrap_or_else(|| {
            input.with_extension("")
        });

        compile_ir_to_binary(&ir_path, &bin_path, effective_opt_level, debug, verbose)?;
    }

    Ok(())
}

/// Format a type error with source context (localized)
fn format_type_error(error: &TypeError, source: &str, path: &PathBuf) -> String {
    let reporter = ErrorReporter::new(source)
        .with_filename(path.to_str().unwrap_or("unknown"));

    let span = error.span();
    let title = error.localized_title();
    let message = error.localized_message();
    let help = error.localized_help();

    reporter.format_error(
        error.error_code(),
        &title,
        span,
        &message,
        help.as_deref(),
    )
}

/// Format a parse error with source context (localized)
fn format_parse_error(error: &ParseError, source: &str, path: &PathBuf) -> String {
    let reporter = ErrorReporter::new(source)
        .with_filename(path.to_str().unwrap_or("unknown"));

    let span = error.span().map(|s| Span::new(s.start, s.end));
    let title = error.localized_title();
    let message = error.localized_message();

    reporter.format_error(
        error.error_code(),
        &title,
        span,
        &message,
        None,
    )
}

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
        .map_err(|e| format_parse_error(&e, source, path))?;

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

/// Resolve import path to file path
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

    // Convert module path to file path
    // e.g., `U utils` -> `utils.vais` or `utils/mod.vais`
    // e.g., `U std/option` -> `std/option.vais`
    let mut file_path = search_base;
    for (i, segment) in path.iter().enumerate() {
        if i == path.len() - 1 {
            // Last segment - try as file first, then as directory with mod.vais
            let as_file = file_path.join(format!("{}.vais", segment.node));
            let as_dir = file_path.join(&segment.node).join("mod.vais");

            if as_file.exists() {
                return Ok(as_file);
            } else if as_dir.exists() {
                return Ok(as_dir);
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

fn compile_ir_to_binary(
    ir_path: &Path,
    bin_path: &Path,
    opt_level: u8,
    debug: bool,
    verbose: bool,
) -> Result<(), String> {
    // Try clang first, then llc + ld
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

fn cmd_run(input: &PathBuf, args: &[String], verbose: bool) -> Result<(), String> {
    // Build first (no debug for run command by default)
    let bin_path = input.with_extension("");
    cmd_build(input, Some(bin_path.clone()), false, 0, false, verbose)?;

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

fn cmd_check(input: &PathBuf, verbose: bool) -> Result<(), String> {
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
        .map_err(|e| format_parse_error(&e, &source, input))?;

    // Type check
    let mut checker = TypeChecker::new();
    if let Err(e) = checker.check_module(&ast) {
        return Err(format_type_error(&e, &source, input));
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

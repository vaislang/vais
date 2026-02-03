//! Vais Compiler CLI
//!
//! The `vaisc` command compiles Vais source files to LLVM IR or native binaries.

#[allow(dead_code)]
mod doc_gen;
mod repl;
mod error_formatter;
#[allow(dead_code)]
mod incremental;
mod package;
#[allow(dead_code)]
mod registry;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

use std::sync::OnceLock;
use vais_ast::{Item, Module, Spanned};

/// Global ownership checking configuration
/// None = warn-only (default), Some(true) = strict errors, Some(false) = disabled
static OWNERSHIP_MODE: OnceLock<Option<bool>> = OnceLock::new();

fn get_ownership_mode() -> Option<bool> {
    OWNERSHIP_MODE.get().copied().unwrap_or(Some(false))
}

fn configure_type_checker(checker: &mut TypeChecker) {
    match get_ownership_mode() {
        Some(true) => checker.set_strict_ownership(true),
        Some(false) => {} // default: warn-only
        None => checker.disable_ownership_check(),
    }
}
use vais_codegen::{CodeGenerator, TargetTriple};
use vais_codegen::optimize::{optimize_ir, OptLevel};
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::{TypeChecker};
use vais_i18n::Locale;
use vais_plugin::{PluginRegistry, PluginsConfig, find_config, Diagnostic, DiagnosticLevel};
use vais_macro::{MacroRegistry, expand_macros, collect_macros, process_derives};
use vais_query::QueryDatabase;

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

    /// Show compilation phase timings
    #[arg(long)]
    time: bool,

    /// Set the locale for error messages (en, ko, ja)
    #[arg(long, value_name = "LOCALE", global = true)]
    locale: Option<String>,

    /// Disable all plugins
    #[arg(long, global = true)]
    no_plugins: bool,

    /// Load additional plugin from file
    #[arg(long, value_name = "PATH", global = true)]
    plugin: Vec<PathBuf>,

    /// Enable garbage collection mode
    #[arg(long, global = true)]
    gc: bool,

    /// Set GC threshold in bytes (default: 1048576 = 1MB)
    #[arg(long, value_name = "BYTES", global = true)]
    gc_threshold: Option<usize>,

    /// Compilation timeout in seconds (0 = no timeout, default: 300)
    #[arg(long, value_name = "SECS", default_value = "300", global = true)]
    timeout: u64,

    /// Allow loading plugins (plugins execute arbitrary native code)
    #[arg(long, global = true)]
    allow_plugins: bool,

    /// Enable strict ownership/borrow checking (errors instead of warnings)
    #[arg(long, global = true)]
    strict_ownership: bool,

    /// Disable ownership/borrow checking entirely
    #[arg(long, global = true)]
    no_ownership_check: bool,

    /// Use inkwell (LLVM API) backend instead of text-based IR generation
    /// Requires compilation with --features inkwell
    #[arg(long, global = true)]
    inkwell: bool,
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

        /// Enable hot reload mode (generate dylib)
        #[arg(long)]
        hot: bool,

        /// Generate GPU code instead of LLVM IR
        /// Targets: cuda, opencl, webgpu, metal
        #[arg(long, value_name = "GPU_TARGET")]
        gpu: Option<String>,

        /// Also generate host code template for GPU kernel dispatch
        #[arg(long)]
        gpu_host: bool,

        /// Compile generated GPU code with nvcc (CUDA) and link with gpu_runtime
        /// Produces a ready-to-run executable instead of just .cu/.cl source
        #[arg(long)]
        gpu_compile: bool,

        /// Enable Link-Time Optimization
        /// Values: thin, full, none
        /// Default: thin for O2/O3, none for O0/O1
        #[arg(long, value_name = "MODE")]
        lto: Option<String>,

        /// Disable automatic Link-Time Optimization (ThinLTO is enabled by default for O2/O3)
        #[arg(long)]
        no_lto: bool,

        /// Generate profile instrumentation for PGO
        /// Creates instrumented binary that writes profiling data to the specified directory
        /// After running the binary, use llvm-profdata to merge the .profraw files
        #[arg(long, value_name = "DIR")]
        profile_generate: Option<String>,

        /// Use profile data for Profile-Guided Optimization
        /// Provide path to merged .profdata file created by llvm-profdata
        #[arg(long, value_name = "FILE")]
        profile_use: Option<String>,

        /// Show suggested fixes for errors
        #[arg(long)]
        suggest_fixes: bool,

        /// Enable parallel compilation with N threads (0 = auto-detect)
        #[arg(short = 'j', long, value_name = "THREADS")]
        parallel: Option<usize>,

        /// Use inkwell (LLVM API) backend instead of text-based IR generation
        #[arg(long)]
        inkwell: bool,
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

    /// Package management commands
    #[command(subcommand)]
    Pkg(PkgCommands),

    /// Profile-Guided Optimization workflow
    Pgo {
        /// Input source file
        input: PathBuf,

        /// Output binary path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Command to run for profiling (default: runs the binary with no args)
        #[arg(long)]
        run_cmd: Option<String>,

        /// Profile data directory (default: ./profdata)
        #[arg(long, default_value = "./profdata")]
        profile_dir: String,

        /// Only merge existing profile data (skip build steps)
        #[arg(long)]
        merge_only: bool,
    },

    /// Watch source file and recompile on changes
    Watch {
        /// Input source file
        input: PathBuf,

        /// Command to run after successful compilation
        #[arg(long)]
        exec: Option<String>,

        /// Arguments to pass to the executed command
        #[arg(last = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
enum PkgCommands {
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
    },

    /// Type-check the package without compiling
    Check,

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
}

#[derive(Subcommand)]
enum CacheAction {
    /// Show cache statistics
    Stats,
    /// Clear the package cache
    Clear,
    /// List cached packages
    List,
}

fn main() {
    let cli = Cli::parse();

    // Configure ownership checking mode
    let ownership_mode = if cli.no_ownership_check {
        None // disabled
    } else if cli.strict_ownership {
        Some(true) // strict (errors)
    } else {
        Some(false) // warn-only (default)
    };
    let _ = OWNERSHIP_MODE.set(ownership_mode);

    // Initialize i18n system
    let locale = cli.locale
        .as_ref()
        .and_then(|s| Locale::parse(s));
    vais_i18n::init(locale);

    // Load plugins
    let plugins = if cli.no_plugins {
        PluginRegistry::new()
    } else {
        load_plugins(&cli.plugin, cli.verbose, cli.allow_plugins)
    };

    // Set up compilation timeout
    let timeout_secs = cli.timeout;
    if timeout_secs > 0 {
        let timeout = std::time::Duration::from_secs(timeout_secs);
        std::thread::spawn(move || {
            std::thread::sleep(timeout);
            eprintln!("error: compilation timed out after {} seconds", timeout_secs);
            exit(124);
        });
    }

    let result = match cli.command {
        Some(Commands::Build { input, output, emit_ir, opt_level, debug, target, force_rebuild, hot, gpu, gpu_host, gpu_compile, lto, no_lto, profile_generate, profile_use, suggest_fixes, parallel, inkwell: _build_inkwell }) => {
            // Check if GPU target is specified
            if let Some(gpu_target_str) = &gpu {
                cmd_build_gpu(&input, output, gpu_target_str, gpu_host, gpu_compile, cli.verbose)
            } else {

            let target_triple = target.as_ref()
                .and_then(|s| TargetTriple::parse(s))
                .unwrap_or(TargetTriple::Native);

            // Parse LTO mode with automatic ThinLTO for O2/O3
            let lto_mode = if no_lto {
                // Explicitly disable LTO
                vais_codegen::optimize::LtoMode::None
            } else if let Some(mode_str) = lto.as_deref() {
                // Explicitly specified LTO mode
                vais_codegen::optimize::LtoMode::parse(mode_str)
            } else {
                // Auto-enable ThinLTO for O2/O3 (release builds)
                if opt_level >= 2 {
                    vais_codegen::optimize::LtoMode::Thin
                } else {
                    vais_codegen::optimize::LtoMode::None
                }
            };

            // Parse PGO mode (mutually exclusive: generate vs use)
            let pgo_mode = if let Some(dir) = profile_generate.as_deref() {
                vais_codegen::optimize::PgoMode::Generate(dir.to_string())
            } else if let Some(path) = profile_use.as_deref() {
                vais_codegen::optimize::PgoMode::Use(path.to_string())
            } else {
                vais_codegen::optimize::PgoMode::None
            };

            // Configure parallel compilation
            let parallel_config = parallel.map(|threads| {
                vais_codegen::parallel::ParallelConfig::new(threads)
            });

            // Default to inkwell when feature is available
            #[cfg(feature = "inkwell")]
            let use_inkwell = true;
            #[cfg(not(feature = "inkwell"))]
            let use_inkwell = build_inkwell || cli.inkwell;
            cmd_build_with_timing(&input, output, emit_ir, opt_level, debug, cli.verbose, cli.time, &plugins, target_triple, force_rebuild, cli.gc, cli.gc_threshold, hot, lto_mode, pgo_mode, suggest_fixes, parallel_config, use_inkwell)
            }
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
        Some(Commands::Pkg(pkg_cmd)) => {
            cmd_pkg(pkg_cmd, cli.verbose)
        }
        Some(Commands::Pgo { input, output, run_cmd, profile_dir, merge_only }) => {
            cmd_pgo(&input, output, run_cmd, &profile_dir, merge_only, cli.verbose, &plugins)
        }
        Some(Commands::Watch { input, exec, args }) => {
            cmd_watch(&input, exec.as_deref(), &args, cli.verbose, &plugins)
        }
        None => {
            // Direct file compilation
            if let Some(input) = cli.input {
                cmd_build_with_timing(
                    &input,
                    cli.output,
                    cli.emit_ir,
                    0,
                    false,
                    cli.verbose,
                    cli.time,
                    &plugins,
                    TargetTriple::Native,
                    false,
                    cli.gc,
                    cli.gc_threshold,
                    false,
                    vais_codegen::optimize::LtoMode::None,
                    vais_codegen::optimize::PgoMode::None,
                    false,
                    None, // parallel_config
                    {
                        #[cfg(feature = "inkwell")]
                        { true }
                        #[cfg(not(feature = "inkwell"))]
                        { cli.inkwell }
                    },
                )
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

/// Build GPU code (CUDA, OpenCL, WebGPU, or Metal)
fn cmd_build_gpu(
    input: &PathBuf,
    output: Option<PathBuf>,
    gpu_target: &str,
    emit_host: bool,
    compile: bool,
    verbose: bool,
) -> Result<(), String> {
    use vais_gpu::{GpuCodeGenerator, GpuTarget};

    // Parse GPU target
    let target = GpuTarget::parse(gpu_target)
        .ok_or_else(|| format!("Unknown GPU target: '{}'. Valid targets: cuda, opencl, webgpu, metal", gpu_target))?;

    if verbose {
        println!("{} Compiling for GPU target: {}", "info:".blue().bold(), target.name());
    }

    // Read source
    let source = fs::read_to_string(input)
        .map_err(|e| format!("Failed to read {}: {}", input.display(), e))?;

    // Parse
    let module = parse(&source)
        .map_err(|e| format!("Parse error: {:?}", e))?;

    // Generate GPU code
    let mut generator = GpuCodeGenerator::new(target);
    let gpu_code = generator.generate(&module)
        .map_err(|e| format!("GPU codegen error: {}", e))?;

    // Determine output file
    let out_path = output.unwrap_or_else(|| {
        let stem = input.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        PathBuf::from(format!("{}.{}", stem, target.extension()))
    });

    // Write output
    fs::write(&out_path, &gpu_code)
        .map_err(|e| format!("Failed to write {}: {}", out_path.display(), e))?;

    println!("{} Generated {} ({})", "✓".green().bold(), out_path.display(), target.name());

    // Print kernel information
    let kernels = generator.kernels();
    if !kernels.is_empty() {
        println!("\n{} {} kernel(s) generated:", "info:".blue().bold(), kernels.len());
        for kernel in kernels {
            println!("  - {} ({} params, block size: {:?})",
                kernel.name,
                kernel.params.len(),
                kernel.block_size
            );
        }
    }

    // Generate host code template if requested
    if emit_host {
        let host_code = generator.generate_host_code();
        let host_ext = match target {
            GpuTarget::Cuda => "host.cu",
            GpuTarget::OpenCL => "host.c",
            GpuTarget::WebGPU => "host.ts",
            GpuTarget::Metal => "host.swift",
        };
        let host_path = input.file_stem()
            .and_then(|s| s.to_str())
            .map(|stem| PathBuf::from(format!("{}.{}", stem, host_ext)))
            .unwrap_or_else(|| PathBuf::from(format!("output.{}", host_ext)));

        fs::write(&host_path, &host_code)
            .map_err(|e| format!("Failed to write host code {}: {}", host_path.display(), e))?;

        println!("{} Generated host code: {} ({})", "✓".green().bold(), host_path.display(), target.name());
    }

    // Compile generated GPU code if --gpu-compile is specified
    if compile {
        match target {
            GpuTarget::Cuda => {
                compile_cuda(&out_path, emit_host, verbose)?;
            }
            GpuTarget::Metal => {
                compile_metal(&out_path, verbose)?;
            }
            GpuTarget::OpenCL => {
                compile_opencl(&out_path, emit_host, verbose)?;
            }
            _ => {
                eprintln!("{} --gpu-compile is currently supported for CUDA, Metal, and OpenCL targets", "warning:".yellow().bold());
            }
        }
    }

    Ok(())
}

/// Find the Vais standard library directory (for gpu_runtime.c etc.)
fn find_std_dir() -> Option<PathBuf> {
    // Check relative to executable
    if let Ok(exe) = std::env::current_exe() {
        let exe_dir = exe.parent()?;
        // Check ../std/ (installed layout)
        let std_dir = exe_dir.join("../std");
        if std_dir.exists() {
            return Some(std_dir.canonicalize().unwrap_or(std_dir));
        }
        // Check ../../std/ (cargo build layout)
        let std_dir = exe_dir.join("../../std");
        if std_dir.exists() {
            return Some(std_dir.canonicalize().unwrap_or(std_dir));
        }
    }
    // Check VAIS_STD_DIR environment variable
    if let Ok(dir) = std::env::var("VAIS_STD_DIR") {
        let path = PathBuf::from(&dir);
        if path.exists() {
            return Some(path);
        }
    }
    // Check current directory's std/
    let cwd_std = PathBuf::from("std");
    if cwd_std.exists() {
        return Some(cwd_std.canonicalize().unwrap_or(cwd_std));
    }
    None
}

/// Compile CUDA .cu file with nvcc and link with gpu_runtime
fn compile_cuda(cu_path: &PathBuf, has_host: bool, verbose: bool) -> Result<(), String> {
    use std::process::Command;

    // Check if nvcc is available
    let nvcc_check = Command::new("nvcc")
        .arg("--version")
        .output();

    match nvcc_check {
        Err(_) => {
            return Err(format!(
                "nvcc not found. Please install the CUDA Toolkit:\n\
                 - Linux: https://developer.nvidia.com/cuda-downloads\n\
                 - macOS: CUDA is no longer supported on macOS (use Metal instead)\n\
                 - Set CUDA_PATH or add nvcc to PATH"
            ));
        }
        Ok(output) if !output.status.success() => {
            return Err("nvcc found but failed to run. Check CUDA Toolkit installation.".to_string());
        }
        Ok(output) => {
            if verbose {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("{} {}", "nvcc:".blue().bold(), version.lines().last().unwrap_or("unknown"));
            }
        }
    }

    // Determine output binary name
    let binary_name = cu_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("gpu_output");
    let binary_path = cu_path.parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join(binary_name);

    // Find gpu_runtime.c
    let std_dir = find_std_dir();
    let runtime_path = std_dir.as_ref().map(|d| d.join("gpu_runtime.c"));

    // Build nvcc command
    let mut cmd = Command::new("nvcc");

    // Add the .cu source file
    cmd.arg(cu_path);

    // Add host code if generated
    if has_host {
        let host_path = cu_path.with_extension("host.cu");
        if host_path.exists() {
            cmd.arg(&host_path);
            if verbose {
                println!("{} Including host code: {}", "info:".blue().bold(), host_path.display());
            }
        }
    }

    // Add gpu_runtime.c if found
    if let Some(ref rt_path) = runtime_path {
        if rt_path.exists() {
            cmd.arg(rt_path);
            if verbose {
                println!("{} Linking gpu_runtime: {}", "info:".blue().bold(), rt_path.display());
            }
        } else if verbose {
            println!("{} gpu_runtime.c not found at {}", "warning:".yellow().bold(), rt_path.display());
        }
    }

    // Output binary
    cmd.arg("-o").arg(&binary_path);

    // Standard flags
    cmd.arg("-lcudart");

    if verbose {
        println!("{} Running: nvcc {} -o {}", "info:".blue().bold(), cu_path.display(), binary_path.display());
    }

    // Execute nvcc
    let result = cmd.output()
        .map_err(|e| format!("Failed to execute nvcc: {}", e))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!(
            "nvcc compilation failed:\n{}{}",
            stderr,
            if stderr.contains("No CUDA capable device") || stderr.contains("no CUDA-capable device") {
                "\n\nHint: No CUDA GPU detected. Ensure NVIDIA drivers are installed."
            } else if stderr.contains("unsupported gpu architecture") {
                "\n\nHint: Try specifying a GPU architecture, e.g., --gpu-arch sm_70"
            } else {
                ""
            }
        ));
    }

    let stdout = String::from_utf8_lossy(&result.stdout);
    if !stdout.is_empty() && verbose {
        println!("{}", stdout);
    }

    println!("{} Compiled GPU binary: {}", "✓".green().bold(), binary_path.display());
    Ok(())
}

/// Compile Metal .metal file to .metallib using xcrun
fn compile_metal(metal_path: &PathBuf, verbose: bool) -> Result<(), String> {
    use std::process::Command;

    // Check if xcrun metal compiler is available
    let xcrun_check = Command::new("xcrun")
        .args(["--find", "metal"])
        .output();

    match xcrun_check {
        Err(_) => {
            return Err(
                "xcrun not found. Please install Xcode Command Line Tools:\n\
                 xcode-select --install".to_string()
            );
        }
        Ok(output) if !output.status.success() => {
            return Err(
                "Metal compiler not found via xcrun. Ensure Xcode is installed with Metal support.".to_string()
            );
        }
        Ok(_) => {
            if verbose {
                println!("{} Metal compiler found via xcrun", "info:".blue().bold());
            }
        }
    }

    // Step 1: Compile .metal → .air (Apple Intermediate Representation)
    let air_path = metal_path.with_extension("air");
    if verbose {
        println!("{} Compiling {} → {}", "info:".blue().bold(), metal_path.display(), air_path.display());
    }

    let air_result = Command::new("xcrun")
        .args(["metal", "-c"])
        .arg(metal_path)
        .arg("-o")
        .arg(&air_path)
        .output()
        .map_err(|e| format!("Failed to execute xcrun metal: {}", e))?;

    if !air_result.status.success() {
        let stderr = String::from_utf8_lossy(&air_result.stderr);
        return Err(format!("Metal compilation failed:\n{}", stderr));
    }

    // Step 2: Link .air → .metallib
    let metallib_path = metal_path.with_extension("metallib");
    if verbose {
        println!("{} Linking {} → {}", "info:".blue().bold(), air_path.display(), metallib_path.display());
    }

    let lib_result = Command::new("xcrun")
        .args(["metallib"])
        .arg(&air_path)
        .arg("-o")
        .arg(&metallib_path)
        .output()
        .map_err(|e| format!("Failed to execute xcrun metallib: {}", e))?;

    if !lib_result.status.success() {
        let stderr = String::from_utf8_lossy(&lib_result.stderr);
        return Err(format!("Metal library linking failed:\n{}", stderr));
    }

    // Clean up intermediate .air file
    let _ = std::fs::remove_file(&air_path);

    println!("{} Compiled Metal library: {}", "✓".green().bold(), metallib_path.display());

    // Step 3: Compile host code with metal_runtime if available
    let std_dir = find_std_dir();
    let runtime_path = std_dir.as_ref().map(|d| d.join("metal_runtime.m"));

    if let Some(ref rt_path) = runtime_path {
        if rt_path.exists() {
            let binary_name = metal_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("metal_output");
            let binary_path = metal_path.parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join(binary_name);

            // Check for host code
            let host_path = metal_path.with_extension("host.swift");
            if host_path.exists() {
                if verbose {
                    println!("{} Host Swift code found: {}", "info:".blue().bold(), host_path.display());
                    println!("{} Note: Compile host code manually with:", "info:".blue().bold());
                    println!("  swiftc {} -framework Metal -framework Foundation -o {}",
                        host_path.display(), binary_path.display());
                }
            } else if verbose {
                println!("{} No host code found. Use --gpu-host to generate host code template.",
                    "info:".blue().bold());
            }
        }
    }

    Ok(())
}

/// Compile OpenCL .cl file and link with opencl_runtime
fn compile_opencl(cl_path: &PathBuf, has_host: bool, verbose: bool) -> Result<(), String> {
    use std::process::Command;

    // Determine output binary name
    let binary_name = cl_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("opencl_output");
    let binary_path = cl_path.parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join(binary_name);

    // Find opencl_runtime.c
    let std_dir = find_std_dir();
    let runtime_path = std_dir.as_ref().map(|d| d.join("opencl_runtime.c"));

    let rt_path = match runtime_path {
        Some(ref p) if p.exists() => p.clone(),
        _ => {
            return Err(
                "opencl_runtime.c not found. Ensure the std/ directory is accessible.".to_string()
            );
        }
    };

    if verbose {
        println!("{} Linking opencl_runtime: {}", "info:".blue().bold(), rt_path.display());
    }

    // Build with cc (clang/gcc)
    let compiler = if cfg!(target_os = "macos") { "clang" } else { "cc" };

    // Check compiler availability
    let cc_check = Command::new(compiler)
        .arg("--version")
        .output();

    if cc_check.is_err() {
        return Err(format!(
            "{} not found. Please install a C compiler (clang or gcc).", compiler
        ));
    }

    let mut cmd = Command::new(compiler);

    // Add opencl_runtime.c
    cmd.arg(&rt_path);

    // Add host code if generated
    if has_host {
        let host_path = cl_path.with_extension("host.c");
        if host_path.exists() {
            cmd.arg(&host_path);
            if verbose {
                println!("{} Including host code: {}", "info:".blue().bold(), host_path.display());
            }
        }
    }

    // Output binary
    cmd.arg("-o").arg(&binary_path);

    // OpenCL framework/library linking
    if cfg!(target_os = "macos") {
        cmd.arg("-framework").arg("OpenCL");
    } else {
        cmd.arg("-lOpenCL");
    }

    // Embed the .cl kernel source path as a define
    let cl_abs = std::fs::canonicalize(cl_path)
        .unwrap_or_else(|_| cl_path.clone());
    cmd.arg(format!("-DVAIS_OPENCL_KERNEL_PATH=\"{}\"", cl_abs.display()));

    if verbose {
        println!("{} Running: {} {} -o {}", "info:".blue().bold(),
            compiler, rt_path.display(), binary_path.display());
    }

    // Execute compiler
    let result = cmd.output()
        .map_err(|e| format!("Failed to execute {}: {}", compiler, e))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!(
            "OpenCL compilation failed:\n{}{}",
            stderr,
            if stderr.contains("opencl") || stderr.contains("OpenCL") || stderr.contains("CL/cl.h") {
                "\n\nHint: Ensure OpenCL SDK is installed.\n\
                 - macOS: OpenCL is built-in (no extra install needed)\n\
                 - Linux: Install ocl-icd-opencl-dev or vendor SDK\n\
                 - Windows: Install GPU vendor OpenCL SDK"
            } else {
                ""
            }
        ));
    }

    let stdout = String::from_utf8_lossy(&result.stdout);
    if !stdout.is_empty() && verbose {
        println!("{}", stdout);
    }

    println!("{} Compiled OpenCL binary: {}", "✓".green().bold(), binary_path.display());
    Ok(())
}

/// Wrapper around cmd_build that optionally prints timing information
#[allow(clippy::too_many_arguments)]
fn cmd_build_with_timing(
    input: &PathBuf,
    output: Option<PathBuf>,
    emit_ir: bool,
    opt_level: u8,
    debug: bool,
    verbose: bool,
    time: bool,
    plugins: &PluginRegistry,
    target: TargetTriple,
    force_rebuild: bool,
    gc: bool,
    gc_threshold: Option<usize>,
    hot: bool,
    lto_mode: vais_codegen::optimize::LtoMode,
    pgo_mode: vais_codegen::optimize::PgoMode,
    suggest_fixes: bool,
    parallel_config: Option<vais_codegen::parallel::ParallelConfig>,
    use_inkwell: bool,
) -> Result<(), String> {
    use std::time::Instant;

    let start = Instant::now();
    let result = cmd_build(
        input, output, emit_ir, opt_level, debug, verbose, plugins,
        target, force_rebuild, gc, gc_threshold, hot, lto_mode, pgo_mode, suggest_fixes, parallel_config, use_inkwell
    );
    let elapsed = start.elapsed();

    if time {
        println!("\n{} Total compilation time: {:.3}s",
            "⏱".cyan().bold(),
            elapsed.as_secs_f64()
        );
    }

    result
}

/// Text-based IR code generation (default backend).
#[allow(clippy::too_many_arguments)]
fn generate_with_text_backend(
    module_name: &str,
    target: &TargetTriple,
    gc: bool,
    gc_threshold: Option<usize>,
    debug: bool,
    input: &PathBuf,
    main_source: &str,
    checker: &TypeChecker,
    final_ast: &vais_ast::Module,
    verbose: bool,
) -> Result<String, String> {
    let mut codegen = CodeGenerator::new_with_target(module_name, target.clone());

    // Enable GC if requested
    if gc {
        codegen.enable_gc();
        if let Some(threshold) = gc_threshold {
            codegen.set_gc_threshold(threshold);
        }
        if verbose {
            println!("  {} (threshold: {} bytes)",
                "GC enabled".cyan(),
                gc_threshold.unwrap_or(1048576));
        }
    }

    // Enable debug info if requested
    if debug {
        let source_file = input.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown.vais");
        let source_dir = input.parent()
            .and_then(|p| p.to_str())
            .unwrap_or(".");
        codegen.enable_debug(source_file, source_dir, main_source);

        if verbose {
            println!("  {}", "Debug info enabled".cyan());
        }
    }

    // Pass resolved function signatures to codegen (for inferred parameter types)
    codegen.set_resolved_functions(checker.get_all_functions().clone());

    if verbose {
        println!("  {} text (IR generation)", "Backend:".cyan());
    }

    let codegen_start = std::time::Instant::now();
    let instantiations = checker.get_generic_instantiations();
    let raw_ir = if instantiations.is_empty() {
        codegen.generate_module(final_ast)
    } else {
        codegen.generate_module_with_instantiations(final_ast, instantiations)
    }.map_err(|e| format!("Codegen error: {}", e))?;
    let codegen_time = codegen_start.elapsed();

    if verbose {
        println!("  {} Codegen time: {:.3}s", "⏱".cyan(), codegen_time.as_secs_f64());
    }

    Ok(raw_ir)
}

#[allow(clippy::too_many_arguments)]
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
    gc: bool,
    gc_threshold: Option<usize>,
    hot: bool,
    lto_mode: vais_codegen::optimize::LtoMode,
    pgo_mode: vais_codegen::optimize::PgoMode,
    suggest_fixes: bool,
    parallel_config: Option<vais_codegen::parallel::ParallelConfig>,
    use_inkwell: bool,
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
                                TargetTriple::Wasm32Unknown | TargetTriple::WasiPreview1 | TargetTriple::WasiPreview2 => "wasm",
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

    // Initialize parallel compilation if requested
    let use_parallel = parallel_config.is_some();
    if let Some(ref config) = parallel_config {
        config.init_thread_pool()?;
        if verbose {
            println!("{} Parallel compilation enabled ({} threads)",
                "⚡".cyan().bold(),
                config.effective_threads());
        }
    }

    // Read source for error reporting
    let main_source = fs::read_to_string(input)
        .map_err(|e| format!("Cannot read '{}': {}", input.display(), e))?;

    // Initialize query database for memoized parsing
    let query_db = QueryDatabase::new();

    // Parse main file and resolve imports
    let parse_start = std::time::Instant::now();
    let mut loaded_modules: HashSet<PathBuf> = HashSet::new();
    let merged_ast = if use_parallel {
        load_module_with_imports_parallel(input, &mut loaded_modules, verbose, &main_source, &query_db)?
    } else {
        load_module_with_imports_internal(input, &mut loaded_modules, verbose, &main_source, &query_db)?
    };
    let parse_time = parse_start.elapsed();

    if verbose {
        println!("  {} total items (including imports)", merged_ast.items.len());
        println!("  {} Parse time: {:.3}s", "⏱".cyan(), parse_time.as_secs_f64());
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

    // Macro expansion phase
    // 1. Collect macro definitions from the AST
    let mut macro_registry = MacroRegistry::new();
    collect_macros(&transformed_ast, &mut macro_registry);

    // 2. Expand all macro invocations
    let macro_expanded_ast = expand_macros(transformed_ast, &macro_registry)
        .map_err(|e| format!("Macro expansion error: {}", e))?;

    // 3. Process #[derive(...)] attributes
    let mut final_ast = macro_expanded_ast;
    process_derives(&mut final_ast)
        .map_err(|e| format!("Derive macro error: {}", e))?;

    if verbose {
        let macro_count = macro_registry.macros_count();
        if macro_count > 0 {
            println!("  {} {} macro(s) expanded", "Macros:".cyan(), macro_count);
        }
    }

    // Type check
    let typecheck_start = std::time::Instant::now();
    let mut checker = TypeChecker::new();
    configure_type_checker(&mut checker);
    if let Err(e) = checker.check_module(&final_ast) {
        // If suggest_fixes is enabled, print suggested fixes
        if suggest_fixes {
            print_suggested_fixes(&e, &main_source);
        }
        // Format error with source context
        return Err(error_formatter::format_type_error(&e, &main_source, input));
    }
    let typecheck_time = typecheck_start.elapsed();

    // Print ownership warnings if any
    let ownership_warnings: Vec<_> = checker.get_warnings().iter()
        .filter(|w| w.starts_with("[ownership]"))
        .collect();
    if !ownership_warnings.is_empty() {
        for w in &ownership_warnings {
            eprintln!("{} {}", "warning:".yellow().bold(), w);
        }
    }

    if verbose {
        println!("  {}", "Type check passed".green());
        println!("  {} Type check time: {:.3}s", "⏱".cyan(), typecheck_time.as_secs_f64());
    }

    // Generate LLVM IR
    let module_name = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");

    if verbose && !matches!(target, TargetTriple::Native) {
        println!("  {} {}", "Target:".cyan(), target.triple_str());
    }

    // Inkwell backend path (opt-in via --inkwell flag)
    #[cfg(feature = "inkwell")]
    let raw_ir = if use_inkwell {
        // Warn about unsupported features in inkwell backend
        if gc {
            eprintln!("{}: --gc is not yet supported with the inkwell backend, ignoring", "warning".yellow().bold());
        }
        if debug {
            eprintln!("{}: -g/--debug is not yet supported with the inkwell backend, ignoring", "warning".yellow().bold());
        }

        if verbose {
            println!("  {} inkwell (LLVM API)", "Backend:".cyan());
        }

        let codegen_start = std::time::Instant::now();
        let context = ::inkwell::context::Context::create();
        let mut gen = vais_codegen::InkwellCodeGenerator::new_with_target(&context, module_name, target.clone());
        gen.generate_module(&final_ast)
            .map_err(|e| format!("Inkwell codegen error: {}", e))?;
        let ir = gen.get_ir_string();
        let codegen_time = codegen_start.elapsed();

        if verbose {
            println!("  {} Codegen time: {:.3}s", "⏱".cyan(), codegen_time.as_secs_f64());
        }

        ir
    } else {
        generate_with_text_backend(
            module_name, &target, gc, gc_threshold, debug, input, &main_source,
            &checker, &final_ast, verbose,
        )?
    };

    #[cfg(not(feature = "inkwell"))]
    let raw_ir = {
        if use_inkwell {
            return Err("Inkwell backend not available. Recompile with: cargo build --features inkwell".to_string());
        }
        generate_with_text_backend(
            module_name, &target, gc, gc_threshold, debug, input, &main_source,
            &checker, &final_ast, verbose,
        )?
    };

    // Apply optimization passes before emitting IR
    // When debug is enabled, disable optimizations to preserve debuggability
    let effective_opt_level = if debug { 0 } else { opt_level };
    let opt = match effective_opt_level {
        0 => OptLevel::O0,
        1 => OptLevel::O1,
        2 => OptLevel::O2,
        _ => OptLevel::O3,
    };
    let ir = if use_parallel && opt != OptLevel::O0 {
        if verbose {
            println!("  {} Parallel optimization enabled", "⚡".cyan());
        }
        vais_codegen::parallel::parallel_optimize_ir(&raw_ir, opt)
    } else {
        optimize_ir(&raw_ir, opt)
    };

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
        let mut opt_info = format!("Applied Vais IR optimizations (O{})", opt_level);
        if lto_mode.is_enabled() {
            opt_info.push_str(&format!(" + {:?}", lto_mode));
        }
        println!("{} {}", "Optimizing".cyan().bold(), opt_info);
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
        match plugins.run_codegen(&final_ast, output_dir) {
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
        // Determine output extension based on target and hot mode
        let default_ext = if hot {
            // Generate dylib for hot reload
            #[cfg(target_os = "macos")]
            let ext = "dylib";
            #[cfg(target_os = "linux")]
            let ext = "so";
            #[cfg(target_os = "windows")]
            let ext = "dll";
            ext
        } else {
            match target {
                TargetTriple::Wasm32Unknown | TargetTriple::WasiPreview1 | TargetTriple::WasiPreview2 => "wasm",
                _ => "",
            }
        };

        let bin_path = output.unwrap_or_else(|| {
            if hot {
                // For hot reload, prefix with 'lib' and use dylib extension
                let parent = input.parent().unwrap_or(Path::new("."));
                let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
                parent.join(format!("lib{}.{}", stem, default_ext))
            } else {
                input.with_extension(default_ext)
            }
        });

        compile_ir_to_binary(&ir_path, &bin_path, effective_opt_level, debug, verbose, &target, hot, &lto_mode, &pgo_mode)?;
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

/// Filter out main functions from imported module items
/// Main functions should only exist in the top-level module being compiled,
/// not in imported modules (which may have their own test main functions)
fn filter_imported_items(items: Vec<Spanned<Item>>) -> Vec<Spanned<Item>> {
    items.into_iter().filter(|item| {
        !matches!(&item.node, Item::Function(f) if f.name.node == "main")
    }).collect()
}

/// Load a module and recursively resolve its imports
fn load_module_with_imports(
    path: &PathBuf,
    loaded: &mut HashSet<PathBuf>,
    verbose: bool,
    query_db: &QueryDatabase,
) -> Result<Module, String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read '{}': {}", path.display(), e))?;
    load_module_with_imports_internal(path, loaded, verbose, &source, query_db)
}

/// Internal function to load a module with source already read
fn load_module_with_imports_internal(
    path: &Path,
    loaded: &mut HashSet<PathBuf>,
    verbose: bool,
    source: &str,
    query_db: &QueryDatabase,
) -> Result<Module, String> {
    // Canonicalize path to avoid duplicate loading
    let canonical = path.canonicalize()
        .map_err(|e| format!("Cannot resolve path '{}': {}", path.display(), e))?;

    // Skip if already loaded
    if loaded.contains(&canonical) {
        return Ok(Module { items: vec![] });
    }
    loaded.insert(canonical.clone());

    // Use QueryDatabase for memoized parsing
    let cached = query_db.has_current_source(&canonical, source);
    query_db.set_source_text(&canonical, source);

    if verbose {
        let cache_tag = if cached { " (cached)" } else { "" };
        println!("{} {}{}", "Compiling".green().bold(), path.display(), cache_tag);
    }

    let ast = query_db.parse(&canonical)
        .map_err(|e| match e {
            vais_query::QueryError::Parse(msg) => {
                // Try to provide formatted error using the original parser
                match vais_parser::parse(source) {
                    Err(parse_err) => error_formatter::format_parse_error(&parse_err, source, path),
                    Ok(_) => msg,
                }
            }
            other => format!("Error in '{}': {}", path.display(), other),
        })?;

    if verbose {
        println!("  {} items", ast.items.len());
    }

    // Collect items, processing imports
    let mut all_items = Vec::new();
    let base_dir = path.parent().unwrap_or(Path::new("."));

    for item in ast.items.iter() {
        match &item.node {
            Item::Use(use_stmt) => {
                // Resolve import path
                let module_path = resolve_import_path(base_dir, &use_stmt.path)?;

                if verbose {
                    println!("  {} {}", "Importing".cyan(), module_path.display());
                }

                // Recursively load the imported module
                let imported = load_module_with_imports(&module_path, loaded, verbose, query_db)?;
                // Filter out main functions from imported modules to avoid conflicts
                all_items.extend(filter_imported_items(imported.items));
            }
            _ => {
                all_items.push(item.clone());
            }
        }
    }

    Ok(Module { items: all_items })
}

/// Load a module with parallel parsing of imports
///
/// First pass: parse the main module to discover import paths.
/// Second pass: parse all imported modules in parallel using rayon.
/// Third pass: merge all items in correct order.
fn load_module_with_imports_parallel(
    path: &Path,
    loaded: &mut HashSet<PathBuf>,
    verbose: bool,
    source: &str,
    query_db: &QueryDatabase,
) -> Result<Module, String> {
    use rayon::prelude::*;

    // Canonicalize path
    let canonical = path.canonicalize()
        .map_err(|e| format!("Cannot resolve path '{}': {}", path.display(), e))?;

    if loaded.contains(&canonical) {
        return Ok(Module { items: vec![] });
    }
    loaded.insert(canonical.clone());

    // Use QueryDatabase for memoized parsing
    let cached = query_db.has_current_source(&canonical, source);
    query_db.set_source_text(&canonical, source);

    if verbose {
        let cache_tag = if cached { " (cached)" } else { "" };
        println!("{} {} (parallel){}", "Compiling".green().bold(), path.display(), cache_tag);
    }

    let ast = query_db.parse(&canonical)
        .map_err(|e| format!("Error in '{}': {}", path.display(), e))?;

    if verbose {
        println!("  {} items", ast.items.len());
    }

    let base_dir = path.parent().unwrap_or(Path::new("."));

    // Phase 1: Collect all import paths first
    let mut import_paths: Vec<PathBuf> = Vec::new();
    let mut import_indices: Vec<usize> = Vec::new();

    for (idx, item) in ast.items.iter().enumerate() {
        if let Item::Use(use_stmt) = &item.node {
            let module_path = resolve_import_path(base_dir, &use_stmt.path)?;
            let module_canonical = module_path.canonicalize()
                .map_err(|e| format!("Cannot resolve path '{}': {}", module_path.display(), e))?;
            if !loaded.contains(&module_canonical) {
                import_paths.push(module_path);
                import_indices.push(idx);
                loaded.insert(module_canonical);
            }
        }
    }

    // Phase 2: Parse all imports in parallel using QueryDatabase
    #[allow(clippy::type_complexity)]
    let parsed_results: Vec<(PathBuf, Result<Module, String>)> = if import_paths.len() > 1 {
        if verbose {
            println!("  {} Parsing {} imports in parallel", "⚡".cyan(), import_paths.len());
        }
        import_paths
            .par_iter()
            .map(|p| {
                let result = (|| -> Result<Module, String> {
                    let src = fs::read_to_string(p)
                        .map_err(|e| format!("Cannot read '{}': {}", p.display(), e))?;
                    let p_canonical = p.canonicalize()
                        .map_err(|e| format!("Cannot resolve path '{}': {}", p.display(), e))?;
                    query_db.set_source_text(&p_canonical, &src);
                    let module = query_db.parse(&p_canonical)
                        .map_err(|e| format!("Error in '{}': {}", p.display(), e))?;
                    Ok(Module { items: module.items.to_vec() })
                })();
                (p.clone(), result)
            })
            .collect()
    } else {
        import_paths.iter().map(|p| {
            let result = (|| -> Result<Module, String> {
                let src = fs::read_to_string(p)
                    .map_err(|e| format!("Cannot read '{}': {}", p.display(), e))?;
                let p_canonical = p.canonicalize()
                    .map_err(|e| format!("Cannot resolve path '{}': {}", p.display(), e))?;
                query_db.set_source_text(&p_canonical, &src);
                let module = query_db.parse(&p_canonical)
                    .map_err(|e| format!("Error in '{}': {}", p.display(), e))?;
                Ok(Module { items: module.items.to_vec() })
            })();
            (p.clone(), result)
        }).collect()
    };

    // Build a map from path -> parsed module
    let mut parsed_map: std::collections::HashMap<PathBuf, vais_ast::Module> =
        std::collections::HashMap::new();
    for (import_path, result) in parsed_results {
        let parsed_module = result?;
        // Recursively resolve imports within each parsed module
        let sub_base = import_path.parent().unwrap_or(Path::new("."));
        let mut sub_items = Vec::new();
        for item in parsed_module.items {
            match &item.node {
                Item::Use(use_stmt) => {
                    let sub_path = resolve_import_path(sub_base, &use_stmt.path)?;
                    let sub_imported = load_module_with_imports(&sub_path, loaded, verbose, query_db)?;
                    // Filter out main functions from imported modules to avoid conflicts
                    sub_items.extend(filter_imported_items(sub_imported.items));
                }
                _ => {
                    sub_items.push(item);
                }
            }
        }
        parsed_map.insert(import_path, Module { items: sub_items });
    }

    // Phase 3: Merge items in correct order
    let mut all_items = Vec::new();
    let mut import_idx = 0;
    for (idx, item) in ast.items.iter().enumerate() {
        match &item.node {
            Item::Use(_) => {
                if import_idx < import_indices.len() && import_indices[import_idx] == idx {
                    if let Some(imported_module) = parsed_map.remove(&import_paths[import_idx]) {
                        // Filter out main functions from imported modules to avoid conflicts
                        all_items.extend(filter_imported_items(imported_module.items));
                    }
                    import_idx += 1;
                }
            }
            _ => {
                all_items.push(item.clone());
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
            let std_path = exe_dir.parent().map(|p| p.join("std"));
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
            }

            // Fall back to dependency search paths (set by pkg build)
            if let Ok(dep_paths) = std::env::var("VAIS_DEP_PATHS") {
                for dep_dir in dep_paths.split(':') {
                    let dep_base = Path::new(dep_dir);
                    if !dep_base.exists() {
                        continue;
                    }
                    // Rebuild file path from the full import path segments
                    let mut dep_file_path = dep_base.to_path_buf();
                    for (j, seg) in path.iter().enumerate() {
                        if j == path.len() - 1 {
                            let dep_as_file = dep_file_path.join(format!("{}.vais", seg.node));
                            let dep_as_dir = dep_file_path.join(&seg.node).join("mod.vais");
                            let dep_as_lib = dep_file_path.join(&seg.node).join("lib.vais");
                            if dep_as_file.exists() {
                                let dep_canonical = dep_base.canonicalize()
                                    .map_err(|_| format!("Cannot resolve dep directory: {}", dep_base.display()))?;
                                return validate_and_canonicalize_import(&dep_as_file, &dep_canonical);
                            } else if dep_as_dir.exists() {
                                let dep_canonical = dep_base.canonicalize()
                                    .map_err(|_| format!("Cannot resolve dep directory: {}", dep_base.display()))?;
                                return validate_and_canonicalize_import(&dep_as_dir, &dep_canonical);
                            } else if dep_as_lib.exists() {
                                let dep_canonical = dep_base.canonicalize()
                                    .map_err(|_| format!("Cannot resolve dep directory: {}", dep_base.display()))?;
                                return validate_and_canonicalize_import(&dep_as_lib, &dep_canonical);
                            }
                        } else {
                            dep_file_path = dep_file_path.join(&seg.node);
                        }
                    }
                }
            }

            return Err(format!(
                "Cannot find module '{}': tried '{}' and '{}'",
                segment.node,
                as_file.display(),
                as_dir.display()
            ));
        } else {
            file_path = file_path.join(&segment.node);
        }
    }

    Err("Invalid import path".to_string())
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

    // Check if within dependency cache paths (set by pkg build)
    let is_within_dep_cache = std::env::var("VAIS_DEP_PATHS").ok()
        .map(|dep_paths| {
            dep_paths.split(':').any(|dp| {
                Path::new(dp).canonicalize().ok()
                    .map(|cp| canonical_path.starts_with(&cp))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    if !is_within_allowed && !is_within_project && !is_within_std && !is_within_dep_cache {
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

/// Find the HTTP runtime C source file for linking.
/// Searches: std/ relative to cwd, then next to compiler executable.
fn find_http_runtime() -> Option<PathBuf> {
    // Try std/ relative to current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let http_rt = cwd.join("std").join("http_runtime.c");
        if http_rt.exists() {
            return Some(http_rt);
        }
    }

    // Try next to the compiler executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Check ../std/ relative to the binary
            if let Some(parent) = exe_dir.parent() {
                let http_rt = parent.join("std").join("http_runtime.c");
                if http_rt.exists() {
                    return Some(http_rt);
                }
            }
        }
    }

    // Try VAIS_HTTP_RUNTIME environment variable
    if let Ok(rt_path) = std::env::var("VAIS_HTTP_RUNTIME") {
        let path = PathBuf::from(&rt_path);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Find the thread runtime C source file for linking.
/// Searches: std/ relative to cwd, then next to compiler executable.
fn find_thread_runtime() -> Option<PathBuf> {
    if let Ok(cwd) = std::env::current_dir() {
        let thread_rt = cwd.join("std").join("thread_runtime.c");
        if thread_rt.exists() {
            return Some(thread_rt);
        }
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            if let Some(parent) = exe_dir.parent() {
                let thread_rt = parent.join("std").join("thread_runtime.c");
                if thread_rt.exists() {
                    return Some(thread_rt);
                }
            }
        }
    }

    if let Ok(rt_path) = std::env::var("VAIS_THREAD_RUNTIME") {
        let path = PathBuf::from(&rt_path);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Find the sync runtime C source file for linking.
/// Searches: std/ relative to cwd, then next to compiler executable.
fn find_sync_runtime() -> Option<PathBuf> {
    if let Ok(cwd) = std::env::current_dir() {
        let sync_rt = cwd.join("std").join("sync_runtime.c");
        if sync_rt.exists() {
            return Some(sync_rt);
        }
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            if let Some(parent) = exe_dir.parent() {
                let sync_rt = parent.join("std").join("sync_runtime.c");
                if sync_rt.exists() {
                    return Some(sync_rt);
                }
            }
        }
    }

    if let Ok(rt_path) = std::env::var("VAIS_SYNC_RUNTIME") {
        let path = PathBuf::from(&rt_path);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Find the directory containing libvais_gc.a for GC runtime linking.
/// Searches: next to the compiler executable, then target/release/ in cwd.
fn find_gc_library() -> Option<PathBuf> {
    // Try next to the compiler executable (e.g. target/release/)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let gc_lib = exe_dir.join("libvais_gc.a");
            if gc_lib.exists() {
                return Some(exe_dir.to_path_buf());
            }
        }
    }

    // Try target/release/ relative to current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let release_dir = cwd.join("target").join("release");
        if release_dir.join("libvais_gc.a").exists() {
            return Some(release_dir);
        }
        // Also try target/debug/
        let debug_dir = cwd.join("target").join("debug");
        if debug_dir.join("libvais_gc.a").exists() {
            return Some(debug_dir);
        }
    }

    // Try VAIS_GC_LIB_DIR environment variable
    if let Ok(gc_dir) = std::env::var("VAIS_GC_LIB_DIR") {
        let path = PathBuf::from(&gc_dir);
        if path.join("libvais_gc.a").exists() {
            return Some(path);
        }
    }

    None
}

#[allow(clippy::too_many_arguments)]
fn compile_ir_to_binary(
    ir_path: &Path,
    bin_path: &Path,
    opt_level: u8,
    debug: bool,
    verbose: bool,
    target: &TargetTriple,
    hot: bool,
    lto_mode: &vais_codegen::optimize::LtoMode,
    pgo_mode: &vais_codegen::optimize::PgoMode,
) -> Result<(), String> {
    match target {
        TargetTriple::Wasm32Unknown => compile_to_wasm32(ir_path, bin_path, opt_level, verbose),
        TargetTriple::WasiPreview1 | TargetTriple::WasiPreview2 => compile_to_wasi(ir_path, bin_path, opt_level, verbose),
        _ => compile_to_native(ir_path, bin_path, opt_level, debug, verbose, hot, lto_mode, pgo_mode),
    }
}

#[allow(clippy::too_many_arguments)]
fn compile_to_native(
    ir_path: &Path,
    bin_path: &Path,
    opt_level: u8,
    debug: bool,
    verbose: bool,
    hot: bool,
    lto_mode: &vais_codegen::optimize::LtoMode,
    pgo_mode: &vais_codegen::optimize::PgoMode,
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

    // Add dylib flags if hot reload mode
    if hot {
        args.push("-shared".to_string());  // Generate shared library
        args.push("-fPIC".to_string());    // Position-independent code
    }

    // Add LTO flags
    for flag in lto_mode.clang_flags() {
        args.push(flag.to_string());
    }

    // Add PGO flags
    for flag in pgo_mode.clang_flags() {
        args.push(flag);
    }

    // Create profile directory if using profile-generate
    if let Some(dir) = pgo_mode.profile_dir() {
        let profile_path = Path::new(dir);
        if !profile_path.exists() {
            std::fs::create_dir_all(profile_path)
                .map_err(|e| format!("Failed to create profile directory '{}': {}", dir, e))?;
        }
        if verbose {
            println!("{} Profile data will be written to: {}/", "info:".blue().bold(), dir);
        }
    }

    // Show PGO info
    if let Some(path) = pgo_mode.profile_file() {
        if !Path::new(path).exists() {
            return Err(format!("Profile data file not found: '{}'. Run the instrumented binary first.", path));
        }
        if verbose {
            println!("{} Using profile data from: {}", "info:".blue().bold(), path);
        }
    }

    args.push("-o".to_string());
    args.push(bin_path.to_str()
        .ok_or_else(|| "Invalid UTF-8 in output path".to_string())?
        .to_string());
    args.push(ir_path.to_str()
        .ok_or_else(|| "Invalid UTF-8 in IR path".to_string())?
        .to_string());

    // Link math library (required on Linux for sqrt, sin, cos, etc.)
    #[cfg(not(target_os = "macos"))]
    args.push("-lm".to_string());

    // Link against libvais_gc if available (for GC runtime support)
    // Use the static library directly to avoid dylib path dependencies
    if let Some(gc_lib_path) = find_gc_library() {
        let static_lib = gc_lib_path.join("libvais_gc.a");
        args.push(static_lib.to_str().unwrap_or("libvais_gc.a").to_string());
        if verbose {
            println!("{} Linking GC runtime from: {}", "info:".blue().bold(), static_lib.display());
        }
    }

    // Link HTTP runtime if available (for std/http.vais support)
    if let Some(http_rt_path) = find_http_runtime() {
        args.push(http_rt_path.to_str().unwrap_or("http_runtime.c").to_string());
        if verbose {
            println!("{} Linking HTTP runtime from: {}", "info:".blue().bold(), http_rt_path.display());
        }
    }

    // Link thread runtime if available (for std/thread.vais support)
    if let Some(thread_rt_path) = find_thread_runtime() {
        args.push(thread_rt_path.to_str().unwrap_or("thread_runtime.c").to_string());
        args.push("-lpthread".to_string());
        if verbose {
            println!("{} Linking thread runtime from: {}", "info:".blue().bold(), thread_rt_path.display());
        }
    }

    // Link sync runtime if available (for std/sync.vais support)
    if let Some(sync_rt_path) = find_sync_runtime() {
        args.push(sync_rt_path.to_str().unwrap_or("sync_runtime.c").to_string());
        args.push("-lpthread".to_string());
        if verbose {
            println!("{} Linking sync runtime from: {}", "info:".blue().bold(), sync_rt_path.display());
        }
    }

    if verbose && (lto_mode.is_enabled() || pgo_mode.is_enabled()) {
        let mut features = vec![];
        if lto_mode.is_enabled() {
            features.push(format!("LTO={:?}", lto_mode));
        }
        if pgo_mode.is_generate() {
            features.push("PGO=generate".to_string());
        } else if pgo_mode.is_use() {
            features.push("PGO=use".to_string());
        }
        println!("{} Compiling with: {}", "info:".blue().bold(), features.join(", "));
    }

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
    // Build first (no debug for run command by default, native target only, use incremental cache, no hot reload, no LTO/PGO)
    let bin_path = input.with_extension("");
    cmd_build(
        input,
        Some(bin_path.clone()),
        false,
        0,
        false,
        verbose,
        plugins,
        TargetTriple::Native,
        false,
        false,
        None,
        false,
        vais_codegen::optimize::LtoMode::None,
        vais_codegen::optimize::PgoMode::None,
        false,
        None, // parallel_config
        false, // use_inkwell
    )?;

    // Run the binary
    if verbose {
        println!("{} {}", "Running".green().bold(), bin_path.display());
    }

    let status = Command::new(&bin_path)
        .args(args)
        .status()
        .map_err(|e| format!("Cannot run '{}': {}", bin_path.display(), e))?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
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
    configure_type_checker(&mut checker);
    if let Err(e) = checker.check_module(&ast) {
        return Err(error_formatter::format_type_error(&e, &source, input));
    }

    // Print ownership warnings if any
    let ownership_warnings: Vec<_> = checker.get_warnings().iter()
        .filter(|w| w.starts_with("[ownership]"))
        .collect();
    if !ownership_warnings.is_empty() {
        for w in &ownership_warnings {
            println!("{} {}", "warning:".yellow().bold(), w);
        }
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

/// Package management commands
fn cmd_pkg(cmd: PkgCommands, verbose: bool) -> Result<(), String> {
    use package::*;
    use std::env;

    let cwd = env::current_dir()
        .map_err(|e| format!("failed to get current directory: {}", e))?;

    match cmd {
        PkgCommands::Init { name } => {
            init_package(&cwd, name.as_deref())
                .map_err(|e| e.to_string())?;
            println!("{} Created package in {}", "✓".green(), cwd.display());
            Ok(())
        }

        PkgCommands::Build { release, debug, hot } => {
            // Find manifest
            let pkg_dir = find_manifest(&cwd)
                .ok_or_else(|| "could not find vais.toml in current directory or parents".to_string())?;

            let manifest = load_manifest(&pkg_dir)
                .map_err(|e| e.to_string())?;

            if verbose {
                println!("{} {}", "Building".cyan(), manifest.package.name);
            }

            // Resolve dependencies (path + registry)
            let cache_root = package::default_registry_cache_root();
            let deps = resolve_all_dependencies(
                &manifest,
                &pkg_dir,
                cache_root.as_deref(),
            ).map_err(|e| e.to_string())?;

            if verbose && !deps.is_empty() {
                println!("{} dependencies:", "Resolved".cyan());
                for dep in &deps {
                    println!("  {} -> {}", dep.name, dep.path.display());
                }
            }

            // Collect dependency source search paths for import resolution
            let dep_search_paths: Vec<PathBuf> = deps.iter().filter_map(|dep| {
                // Check for src/ directory in the dependency
                let src_dir = dep.path.join("src");
                if src_dir.exists() {
                    Some(src_dir)
                } else if dep.path.exists() {
                    // Use the dependency root directly if no src/
                    Some(dep.path.clone())
                } else {
                    None
                }
            }).collect();

            // Determine entry point
            let src_dir = pkg_dir.join("src");
            let entry = if src_dir.join("main.vais").exists() {
                src_dir.join("main.vais")
            } else if src_dir.join("lib.vais").exists() {
                src_dir.join("lib.vais")
            } else {
                return Err("no main.vais or lib.vais found in src/".to_string());
            };

            // Set dependency search paths as environment variable for import resolution
            if !dep_search_paths.is_empty() {
                let paths_str: Vec<String> = dep_search_paths.iter()
                    .map(|p| p.display().to_string())
                    .collect();
                std::env::set_var("VAIS_DEP_PATHS", paths_str.join(":"));
            }

            // Build options
            let opt_level = if release { 2 } else { 0 };
            let output = pkg_dir.join("target").join(&manifest.package.name);

            // Auto-enable ThinLTO for release builds
            let lto_mode = if release {
                vais_codegen::optimize::LtoMode::Thin
            } else {
                vais_codegen::optimize::LtoMode::None
            };

            // Create target directory
            let target_dir = pkg_dir.join("target");
            fs::create_dir_all(&target_dir)
                .map_err(|e| format!("failed to create target directory: {}", e))?;

            // Load empty plugin registry for build
            let plugins = PluginRegistry::new();

            // Build using existing infrastructure with automatic ThinLTO for release builds
            cmd_build(
                &entry,
                Some(output.clone()),
                false,
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
                false,
                None, // parallel_config
                false, // use_inkwell
            )?;

            if hot {
                println!("{} Built hot-reload dylib {}", "✓".green(), output.display());
            } else {
                println!("{} Built {}", "✓".green(), output.display());
            }
            Ok(())
        }

        PkgCommands::Check => {
            let pkg_dir = find_manifest(&cwd)
                .ok_or_else(|| "could not find vais.toml".to_string())?;

            let manifest = load_manifest(&pkg_dir)
                .map_err(|e| e.to_string())?;

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

            println!("{} {} type-checks correctly", "✓".green(), manifest.package.name);
            Ok(())
        }

        PkgCommands::Add { name, path, version } => {
            let pkg_dir = find_manifest(&cwd)
                .ok_or_else(|| "could not find vais.toml".to_string())?;

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
            let pkg_dir = find_manifest(&cwd)
                .ok_or_else(|| "could not find vais.toml".to_string())?;

            let manifest_path = pkg_dir.join("vais.toml");
            remove_dependency(&manifest_path, &name)
                .map_err(|e| e.to_string())?;

            println!("{} Removed dependency '{}'", "✓".green(), name);
            Ok(())
        }

        PkgCommands::Clean => {
            let pkg_dir = find_manifest(&cwd)
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

        PkgCommands::Install { packages, update, offline } => {
            cmd_pkg_install(&cwd, packages, update, offline, verbose)
        }

        PkgCommands::Update { packages, offline } => {
            cmd_pkg_update(&cwd, packages, offline, verbose)
        }

        PkgCommands::Search { query, limit, offline, sort, category, keyword } => {
            cmd_pkg_search(&query, limit, offline, verbose, &sort, category.as_deref(), keyword.as_deref())
        }

        PkgCommands::Info { name } => {
            cmd_pkg_info(&name, verbose)
        }

        PkgCommands::Cache { action } => {
            cmd_pkg_cache(action, verbose)
        }

        PkgCommands::Audit { format } => {
            cmd_pkg_audit(&cwd, &format, verbose)
        }

        PkgCommands::Publish { registry, token, dry_run } => {
            cmd_pkg_publish(&cwd, registry, token, dry_run, verbose)
        }

        PkgCommands::Yank { name, version, token, registry } => {
            cmd_pkg_yank(&name, &version, token, registry, verbose)
        }

        PkgCommands::Login { registry } => {
            cmd_pkg_login(registry, verbose)
        }
    }
}

/// Install packages from registry
fn cmd_pkg_install(cwd: &Path, packages: Vec<String>, update: bool, offline: bool, verbose: bool) -> Result<(), String> {
    use registry::{RegistryClient, RegistrySource, LockFile, DependencyResolver};

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
        client.update_index().map_err(|e| format!("failed to update index: {}", e))?;
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
        resolver.add(&name, &version_req)
            .map_err(|e| format!("invalid version requirement '{}': {}", version_req, e))?;
    }

    // Resolve dependencies
    if verbose {
        println!("{} Resolving dependencies...", "Info".cyan());
    }

    let resolved = resolver.resolve()
        .map_err(|e| format!("dependency resolution failed: {}", e))?;

    if resolved.is_empty() {
        println!("{} No packages to install", "Info".cyan());
        return Ok(());
    }

    // Install packages
    for pkg in &resolved {
        if client.is_installed(&pkg.name, &pkg.version) {
            if verbose {
                println!("{} {} {} (cached)", "Skipping".yellow(), pkg.name, pkg.version);
            }
        } else if offline {
            return Err(format!(
                "Package {} {} not in cache. Run without --offline to download.",
                pkg.name, pkg.version
            ));
        } else {
            println!("{} {} {}...", "Installing".green(), pkg.name, pkg.version);
            client.download(&pkg.name, &pkg.version)
                .map_err(|e| format!("failed to install {} {}: {}", pkg.name, pkg.version, e))?;
        }
    }

    // Save lock file
    let new_lock = resolver.generate_lock();
    new_lock.save(&lock_path)
        .map_err(|e| format!("failed to save lock file: {}", e))?;

    println!("{} Installed {} package(s)", "✓".green(), resolved.len());
    Ok(())
}

/// Update dependencies
fn cmd_pkg_update(cwd: &Path, packages: Vec<String>, offline: bool, verbose: bool) -> Result<(), String> {
    use registry::{RegistryClient, RegistrySource, DependencyResolver};
    use package::{find_manifest, load_manifest};

    // Find and load manifest
    let pkg_dir = find_manifest(cwd)
        .ok_or_else(|| "could not find vais.toml".to_string())?;
    let manifest = load_manifest(&pkg_dir)
        .map_err(|e| e.to_string())?;

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
        client.update_index().map_err(|e| format!("failed to update index: {}", e))?;
    }

    // Determine which packages to update
    let deps_to_update: Vec<(String, String)> = if packages.is_empty() {
        // Update all dependencies from manifest
        manifest.dependencies.iter()
            .filter_map(|(name, dep)| {
                match dep {
                    package::Dependency::Version(v) => Some((name.clone(), v.clone())),
                    package::Dependency::Detailed(d) if d.version.is_some() => {
                        Some((name.clone(), d.version.clone().unwrap()))
                    }
                    _ => None
                }
            })
            .collect()
    } else {
        // Update only specified packages
        packages.iter()
            .filter_map(|name| {
                manifest.dependencies.get(name).and_then(|dep| {
                    match dep {
                        package::Dependency::Version(v) => Some((name.clone(), v.clone())),
                        package::Dependency::Detailed(d) if d.version.is_some() => {
                            Some((name.clone(), d.version.clone().unwrap()))
                        }
                        _ => None
                    }
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
        resolver.add(name, req)
            .map_err(|e| format!("invalid version requirement '{}': {}", req, e))?;
    }

    let resolved = resolver.resolve()
        .map_err(|e| format!("dependency resolution failed: {}", e))?;

    // Install/update packages
    for pkg in &resolved {
        if client.is_installed(&pkg.name, &pkg.version) {
            if verbose {
                println!("{} {} {} (up to date)", "Skipping".yellow(), pkg.name, pkg.version);
            }
        } else {
            println!("{} {} {}...", "Updating".green(), pkg.name, pkg.version);
            client.download(&pkg.name, &pkg.version)
                .map_err(|e| format!("failed to update {} {}: {}", pkg.name, pkg.version, e))?;
        }
    }

    // Save new lock file
    let lock_path = pkg_dir.join("vais.lock");
    let new_lock = resolver.generate_lock();
    new_lock.save(&lock_path)
        .map_err(|e| format!("failed to save lock file: {}", e))?;

    println!("{} Updated {} package(s)", "✓".green(), resolved.len());
    Ok(())
}

/// Search for packages
fn cmd_pkg_search(query: &str, limit: usize, offline: bool, verbose: bool, _sort: &str, _category: Option<&str>, _keyword: Option<&str>) -> Result<(), String> {
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
        client.update_index().map_err(|e| format!("failed to update index: {}", e))?;
    }

    let results = client.search(query)
        .map_err(|e| format!("search failed: {}", e))?;

    if results.is_empty() {
        println!("{} No packages found matching '{}'", "Info".cyan(), query);
        return Ok(());
    }

    println!("{} packages found:\n", results.len().min(limit));

    for pkg in results.iter().take(limit) {
        let latest = pkg.latest_version()
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
fn cmd_pkg_info(name: &str, verbose: bool) -> Result<(), String> {
    use registry::{RegistryClient, RegistrySource};

    let source = RegistrySource::default();
    let mut client = RegistryClient::new(source)
        .map_err(|e| format!("failed to initialize registry client: {}", e))?;

    if !client.load_cached_index().map_err(|e| e.to_string())? {
        client.update_index().map_err(|e| format!("failed to update index: {}", e))?;
    }

    let pkg = client.get_package(name)
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
            println!("      deps: {}", v.dependencies.keys().cloned().collect::<Vec<_>>().join(", "));
        }
    }
    if versions.len() > 10 {
        println!("    ... and {} more", versions.len() - 10);
    }

    Ok(())
}

/// Cache management
fn cmd_pkg_cache(action: CacheAction, _verbose: bool) -> Result<(), String> {
    use registry::PackageCache;

    let cache = PackageCache::new()
        .map_err(|e| format!("failed to access cache: {}", e))?;

    match action {
        CacheAction::Stats => {
            let stats = cache.stats()
                .map_err(|e| format!("failed to get cache stats: {}", e))?;

            println!("{}", "Cache Statistics".bold());
            println!("  {}: {}", "location".cyan(), cache.root().display());
            println!("  {}: {}", "packages".cyan(), stats.packages);
            println!("  {}: {}", "versions".cyan(), stats.versions);
            println!("  {}: {}", "size".cyan(), stats.size_display());
        }
        CacheAction::Clear => {
            cache.clear()
                .map_err(|e| format!("failed to clear cache: {}", e))?;
            println!("{} Cache cleared", "✓".green());
        }
        CacheAction::List => {
            let packages = cache.list_packages()
                .map_err(|e| format!("failed to list packages: {}", e))?;

            if packages.is_empty() {
                println!("{} Cache is empty", "Info".cyan());
                return Ok(());
            }

            println!("{}", "Cached packages:".bold());
            for name in packages {
                let versions = cache.list_versions(&name)
                    .map_err(|e| format!("failed to list versions: {}", e))?;
                let version_strs: Vec<String> = versions.iter().map(|v| v.to_string()).collect();
                println!("  {} [{}]", name.bold(), version_strs.join(", "));
            }
        }
    }

    Ok(())
}

/// Audit dependencies for known vulnerabilities
fn cmd_pkg_audit(cwd: &Path, format: &str, verbose: bool) -> Result<(), String> {
    use package::{find_manifest, load_manifest};

    // Find and load manifest
    let pkg_dir = find_manifest(cwd)
        .ok_or_else(|| "could not find vais.toml".to_string())?;
    let manifest = load_manifest(&pkg_dir)
        .map_err(|e| e.to_string())?;

    // Load lock file if exists
    let lock_path = pkg_dir.join("vais.lock");
    let locked_packages: Vec<(String, String)> = if lock_path.exists() {
        use registry::LockFile;
        let lock = LockFile::load(&lock_path).map_err(|e| e.to_string())?;
        lock.packages.iter()
            .map(|(name, pkg)| (name.clone(), pkg.version.to_string()))
            .collect()
    } else {
        // Use manifest dependencies if no lock file
        manifest.dependencies.iter()
            .filter_map(|(name, dep)| {
                match dep {
                    package::Dependency::Version(v) => Some((name.clone(), v.clone())),
                    package::Dependency::Detailed(d) if d.version.is_some() => {
                        Some((name.clone(), d.version.clone().unwrap()))
                    }
                    _ => None
                }
            })
            .collect()
    };

    if locked_packages.is_empty() {
        println!("{} No dependencies to audit", "Info".cyan());
        return Ok(());
    }

    if verbose {
        println!("{} Auditing {} package(s)...", "Info".cyan(), locked_packages.len());
    }

    // Check for known vulnerabilities using OSV API
    use registry::VulnerabilityScanner;

    let scanner = VulnerabilityScanner::new();
    let vulns_by_package = scanner.query_batch(&locked_packages)
        .unwrap_or_else(|e| {
            if verbose {
                eprintln!("{} Failed to query vulnerability database: {}", "Warning".yellow(), e);
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
                let comma = if i < vulnerabilities.len() - 1 { "," } else { "" };
                println!("    {{ \"package\": \"{}\", \"version\": \"{}\", \"advisory\": \"{}\" }}{}", pkg, ver, advisory, comma);
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
                println!("\n{} {} vulnerabilities found:", "⚠".yellow().bold(), vulnerabilities.len());
                for (pkg, ver, advisory) in &vulnerabilities {
                    println!("  {} {} - {}", pkg.red(), ver, advisory);
                }
            }

            println!("\n{} For more information, visit: https://osv.dev", "ℹ".blue());
        }
    }

    Ok(())
}

/// Publish a package to the registry
fn cmd_pkg_publish(
    cwd: &Path,
    registry: Option<String>,
    token: Option<String>,
    dry_run: bool,
    verbose: bool,
) -> Result<(), String> {
    use package::{find_manifest, load_manifest};

    let registry_url = registry
        .unwrap_or_else(|| "https://registry.vais.dev".to_string());

    // Find and load manifest
    let pkg_dir = find_manifest(cwd)
        .ok_or_else(|| "could not find vais.toml".to_string())?;
    let manifest = load_manifest(&pkg_dir)
        .map_err(|e| e.to_string())?;

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
    fs::create_dir_all(&tmp_dir)
        .map_err(|e| format!("failed to create temp directory: {}", e))?;
    let archive_path = tmp_dir.join(format!("{}-{}.tar.gz", pkg_name, pkg_version));

    registry::pack_package(&pkg_dir, &archive_path)
        .map_err(|e| format!("failed to pack package: {}", e))?;

    // Read archive and compute checksum
    let archive_bytes = fs::read(&archive_path)
        .map_err(|e| format!("failed to read archive: {}", e))?;
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
fn cmd_pkg_yank(
    name: &str,
    version: &str,
    token: Option<String>,
    registry: Option<String>,
    verbose: bool,
) -> Result<(), String> {
    let registry_url = registry
        .unwrap_or_else(|| "https://registry.vais.dev".to_string());

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
        println!("{} Yanking {}@{} from {}", "Info".cyan(), name, version, registry_url);
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
fn cmd_pkg_login(registry: Option<String>, verbose: bool) -> Result<(), String> {
    let registry_url = registry
        .unwrap_or_else(|| "https://registry.vais.dev".to_string());

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
    let home = dirs::home_dir()
        .ok_or_else(|| "could not determine home directory".to_string())?;
    let vais_dir = home.join(".vais");
    fs::create_dir_all(&vais_dir)
        .map_err(|e| format!("failed to create ~/.vais: {}", e))?;

    let creds_path = vais_dir.join("credentials.toml");

    // Load existing credentials or start fresh
    let mut creds: toml::Value = if creds_path.exists() {
        let content = fs::read_to_string(&creds_path)
            .map_err(|e| format!("failed to read credentials: {}", e))?;
        content.parse().unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()))
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
fn load_credentials_token(registry_url: &str) -> Option<String> {
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
fn parse_package_spec(spec: &str) -> (String, String) {
    if let Some(idx) = spec.find('@') {
        let name = &spec[..idx];
        let version = &spec[idx + 1..];
        (name.to_string(), version.to_string())
    } else {
        (spec.to_string(), "*".to_string())
    }
}

/// Walk directory recursively to find files with given extension
fn walkdir(dir: &PathBuf, ext: &str) -> Vec<PathBuf> {
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
fn load_plugins(extra_plugins: &[PathBuf], verbose: bool, allow_plugins: bool) -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    registry.set_allow_plugins(allow_plugins);

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

/// Profile-Guided Optimization workflow
///
/// Automates the 3-step PGO process:
/// 1. Build with instrumentation (--profile-generate)
/// 2. Run to collect profile data
/// 3. Merge profiles and rebuild with optimization (--profile-use)
fn cmd_pgo(
    input: &PathBuf,
    output: Option<PathBuf>,
    run_cmd: Option<String>,
    profile_dir: &str,
    merge_only: bool,
    verbose: bool,
    plugins: &PluginRegistry,
) -> Result<(), String> {
    let bin_name = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("a.out");
    let output_path = output.unwrap_or_else(|| PathBuf::from(bin_name));
    let profdata_path = format!("{}/default.profdata", profile_dir);

    if !merge_only {
        // Step 1: Build with instrumentation
        println!("{} Step 1/3: Building instrumented binary...", "[PGO]".cyan().bold());
        let instrumented_bin = PathBuf::from(format!("{}-instrumented", output_path.display()));

        cmd_build(
            input,
            Some(instrumented_bin.clone()),
            false, // emit_ir
            2,     // opt_level
            false, // debug
            verbose,
            plugins,
            TargetTriple::Native,
            false, // force_rebuild
            false, // gc
            None,  // gc_threshold
            false, // hot
            vais_codegen::optimize::LtoMode::None,
            vais_codegen::optimize::PgoMode::Generate(profile_dir.to_string()),
            false, // suggest_fixes
            None, // parallel_config
            false, // use_inkwell
        )?;

        println!("{} Instrumented binary: {}", "  ✓".green(), instrumented_bin.display());

        // Step 2: Run to collect profile data
        println!("{} Step 2/3: Running to collect profile data...", "[PGO]".cyan().bold());
        let run_command = run_cmd.unwrap_or_else(|| instrumented_bin.display().to_string());

        let status = Command::new("sh")
            .arg("-c")
            .arg(&run_command)
            .env("LLVM_PROFILE_FILE", format!("{}/default-%p.profraw", profile_dir))
            .status()
            .map_err(|e| format!("failed to run instrumented binary: {}", e))?;

        if !status.success() {
            println!("{} Instrumented binary exited with non-zero status (profile data may still be usable)", "  ⚠".yellow());
        } else {
            println!("{} Profile data collected in {}/", "  ✓".green(), profile_dir);
        }

        // Clean up instrumented binary
        let _ = fs::remove_file(&instrumented_bin);
    }

    // Step 3: Merge profile data and rebuild
    println!("{} Step 3/3: Merging profiles and rebuilding with optimization...", "[PGO]".cyan().bold());

    // Merge profraw files using llvm-profdata
    let merge_status = Command::new("llvm-profdata")
        .args(["merge", "-sparse"])
        .arg(format!("{}/", profile_dir))
        .arg("-o")
        .arg(&profdata_path)
        .status();

    match merge_status {
        Ok(s) if s.success() => {
            println!("{} Merged profile data: {}", "  ✓".green(), profdata_path);
        }
        _ => {
            // Try xcrun llvm-profdata on macOS
            let merge_status2 = Command::new("xcrun")
                .args(["llvm-profdata", "merge", "-sparse"])
                .arg(format!("{}/", profile_dir))
                .arg("-o")
                .arg(&profdata_path)
                .status();

            match merge_status2 {
                Ok(s) if s.success() => {
                    println!("{} Merged profile data: {}", "  ✓".green(), profdata_path);
                }
                _ => {
                    return Err("Failed to merge profile data. Ensure llvm-profdata is installed.".to_string());
                }
            }
        }
    }

    // Rebuild with profile data
    cmd_build(
        input,
        Some(output_path.clone()),
        false, // emit_ir
        2,     // opt_level
        false, // debug
        verbose,
        plugins,
        TargetTriple::Native,
        false, // force_rebuild
        false, // gc
        None,  // gc_threshold
        false, // hot
        vais_codegen::optimize::LtoMode::Thin,
        vais_codegen::optimize::PgoMode::Use(profdata_path),
        false, // suggest_fixes
        None, // parallel_config
        false, // use_inkwell
    )?;

    println!("{} PGO-optimized binary: {}", "  ✓".green(), output_path.display());
    println!("\n{} PGO workflow complete!", "Done".green().bold());

    Ok(())
}

/// Watch for file changes and recompile
fn cmd_watch(
    input: &PathBuf,
    exec: Option<&str>,
    args: &[String],
    verbose: bool,
    plugins: &PluginRegistry,
) -> Result<(), String> {
    use std::time::Duration;
    use std::collections::HashSet;

    // Determine watch directory (parent of input file or current directory)
    let watch_dir = input.parent().unwrap_or(Path::new(".")).to_path_buf();

    println!("{} {} (directory: {})",
        "Watching".cyan().bold(),
        input.display(),
        watch_dir.display());

    // Collect all .vais files to watch (for import tracking)
    let mut watched_files: HashSet<PathBuf> = HashSet::new();
    watched_files.insert(input.canonicalize().unwrap_or_else(|_| input.clone()));

    // Scan for import statements and add imported files
    if let Ok(content) = std::fs::read_to_string(input) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("I ") || trimmed.starts_with("import ") {
                // Extract import path: I "path/to/file" or import "path/to/file"
                if let Some(start) = trimmed.find('"') {
                    if let Some(end) = trimmed[start+1..].find('"') {
                        let import_path = &trimmed[start+1..start+1+end];
                        let full_path = watch_dir.join(import_path);
                        if full_path.exists() {
                            if let Ok(canonical) = full_path.canonicalize() {
                                watched_files.insert(canonical);
                            }
                        }
                    }
                }
            }
        }
    }

    if verbose {
        println!("{} Watching {} file(s)", "Info".blue().bold(), watched_files.len());
        for file in &watched_files {
            println!("  - {}", file.display());
        }
    }

    // Perform initial build
    let bin_path = input.with_extension("");
    cmd_build(
        input,
        Some(bin_path.clone()),
        false,
        0,
        false,
        verbose,
        plugins,
        TargetTriple::Native,
        false,
        false,
        None,
        false,
        vais_codegen::optimize::LtoMode::None,
        vais_codegen::optimize::PgoMode::None,
        false,
        None, // parallel_config
        false, // use_inkwell
    )?;

    // Execute initial run if requested
    if let Some(cmd) = exec {
        if verbose {
            println!("{} {}", "Running".green().bold(), cmd);
        }
        let _ = Command::new(cmd)
            .args(args)
            .status();
    }

    // Create file watcher using notify crate
    use notify::{Watcher, RecursiveMode, RecommendedWatcher};
    use std::sync::mpsc::channel;

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        notify::Config::default(),
    ).map_err(|e| format!("Failed to create watcher: {}", e))?;

    // Watch the directory recursively for .vais files
    watcher.watch(&watch_dir, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch directory: {}", e))?;

    println!("{} Press Ctrl+C to stop", "Ready".green().bold());

    // Watch for changes
    let mut last_compile = std::time::SystemTime::now();
    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                // Debounce: ignore events within 100ms of last compile
                if let Ok(elapsed) = last_compile.elapsed() {
                    if elapsed < Duration::from_millis(100) {
                        continue;
                    }
                }

                // Only recompile on modify events for .vais files
                if matches!(event.kind, notify::EventKind::Modify(_)) {
                    // Check if the modified file is a .vais file
                    let is_vais_file = event.paths.iter().any(|p| {
                        p.extension().is_some_and(|ext| ext == "vais")
                    });

                    if !is_vais_file {
                        continue;
                    }

                    let changed_files: Vec<_> = event.paths.iter()
                        .filter(|p| p.extension().is_some_and(|ext| ext == "vais"))
                        .collect();

                    if verbose {
                        for path in &changed_files {
                            println!("{} Changed: {}", "⟳".cyan().bold(), path.display());
                        }
                    } else {
                        println!("\n{} Change detected, recompiling...", "⟳".cyan().bold());
                    }

                    last_compile = std::time::SystemTime::now();

                    // Rebuild
                    match cmd_build(
                        input,
                        Some(bin_path.clone()),
                        false,
                        0,
                        false,
                        verbose,
                        plugins,
                        TargetTriple::Native,
                        false,
                        false,
                        None,
                        false,
                        vais_codegen::optimize::LtoMode::None,
                        vais_codegen::optimize::PgoMode::None,
                        false,
                        None, // parallel_config
                        false, // use_inkwell
                    ) {
                        Ok(_) => {
                            println!("{} Compilation successful", "✓".green().bold());

                            // Execute if requested
                            if let Some(cmd) = exec {
                                println!("{} {}", "Running".green().bold(), cmd);
                                let _ = Command::new(cmd)
                                    .args(args)
                                    .status();
                            }
                        }
                        Err(e) => {
                            eprintln!("{} {}", "✗".red().bold(), e);
                        }
                    }
                }
            }
            Ok(Err(e)) => {
                eprintln!("{}: {}", "Watch error".yellow(), e);
            }
            Err(_) => {
                return Err("Watcher channel closed".to_string());
            }
        }
    }
}

/// Print suggested fixes for type errors
fn print_suggested_fixes(error: &vais_types::TypeError, _source: &str) {
    use vais_types::TypeError;

    eprintln!("\n{} Suggested fixes:", "💡".cyan().bold());

    match error {
        TypeError::UndefinedVar { name, suggestion, .. } => {
            if let Some(similar) = suggestion {
                eprintln!("  {} Did you mean '{}'?", "•".green(), similar);
            } else {
                eprintln!("  {} Define variable: L {}: i64 = 0", "•".green(), name);
            }
        }
        TypeError::UndefinedFunction { name, suggestion, .. } => {
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
        TypeError::Mismatch { expected, found, .. } => {
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

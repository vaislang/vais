//! Vais Compiler CLI
//!
//! The `vaisc` command compiles Vais source files to LLVM IR or native binaries.

mod commands;
mod doc_gen;
mod error_formatter;
mod imports;
mod incremental;
mod package;
mod registry;
mod repl;
mod runtime;
mod utils;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use std::process::exit;
use std::sync::OnceLock;

use commands::pkg::PkgCommands;
use vais_codegen::TargetTriple;
use vais_i18n::Locale;
use vais_plugin::PluginRegistry;
use vais_types::TypeChecker;

/// Global ownership checking configuration
/// None = disabled, Some(true) = strict errors (default), Some(false) = warn-only
static OWNERSHIP_MODE: OnceLock<Option<bool>> = OnceLock::new();

/// Global MIR borrow checking flag
static STRICT_BORROW: OnceLock<bool> = OnceLock::new();

pub(crate) fn get_ownership_mode() -> Option<bool> {
    OWNERSHIP_MODE.get().copied().unwrap_or(Some(true))
}

pub(crate) fn get_strict_borrow() -> bool {
    STRICT_BORROW.get().copied().unwrap_or(false)
}

/// Generate a crash report with system info, backtrace, and panic details
fn generate_crash_report(info: &std::panic::PanicHookInfo<'_>) -> String {
    let mut report = String::new();
    report.push_str("=== Vais Compiler Crash Report ===\n\n");

    // Timestamp
    report.push_str(&format!("Timestamp: {:?}\n", std::time::SystemTime::now()));

    // Version info
    report.push_str(&format!("Compiler: vaisc {}\n", env!("CARGO_PKG_VERSION")));
    report.push_str(&format!(
        "OS: {} {}\n",
        std::env::consts::OS,
        std::env::consts::ARCH
    ));

    // Panic info
    report.push_str(&format!("\nPanic: {}\n", info));
    if let Some(location) = info.location() {
        report.push_str(&format!(
            "Location: {}:{}:{}\n",
            location.file(),
            location.line(),
            location.column()
        ));
    }

    // Backtrace
    report.push_str("\nBacktrace:\n");
    report.push_str(&format!("{}", std::backtrace::Backtrace::force_capture()));

    // Command line args
    report.push_str("\nCommand line args:\n");
    for arg in std::env::args() {
        report.push_str(&format!("  {}\n", arg));
    }

    report.push_str("\n=== End of Crash Report ===\n");
    report
}

pub(crate) fn configure_type_checker(checker: &mut TypeChecker) {
    match get_ownership_mode() {
        Some(true) => checker.set_strict_ownership(true),
        Some(false) => {} // default: warn-only
        None => checker.disable_ownership_check(),
    }
}

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

    /// Enable strict ownership/borrow checking (now the default, kept for compatibility)
    #[arg(long, global = true)]
    strict_ownership: bool,

    /// Disable ownership/borrow checking entirely
    #[arg(long, global = true)]
    no_ownership_check: bool,

    /// Generate a crash report file when the compiler panics
    #[arg(long, global = true)]
    report_crash: bool,

    /// Use warn-only borrow checking (legacy default, not recommended for new projects)
    #[arg(long, global = true)]
    warn_only_ownership: bool,

    /// Enable MIR-based borrow checking (use-after-move, double-free, use-after-free detection)
    #[arg(long, global = true)]
    strict_borrow: bool,

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

        /// Enable per-module codegen for multi-file projects
        /// Each module gets its own .ll/.o file, enabling future incremental compilation
        #[arg(long)]
        per_module: bool,

        /// Cache size limit in bytes (default: 536870912 = 512MB)
        /// Old .o files will be deleted automatically to stay under this limit
        #[arg(long, value_name = "BYTES", default_value = "536870912")]
        cache_limit: u64,

        /// Disable incremental compilation cache entirely
        #[arg(long)]
        no_cache: bool,

        /// Pre-populate the cache by scanning all .vais files in the project
        #[arg(long)]
        warm_cache: bool,

        /// Clear the incremental compilation cache
        #[arg(long)]
        clear_cache: bool,

        /// Show incremental cache statistics and exit
        #[arg(long)]
        cache_stats: bool,

        /// Enable source-based code coverage instrumentation
        /// Generates an instrumented binary; run it to produce .profraw files
        /// Optional value: output directory for coverage data (default: ./coverage)
        #[arg(long, value_name = "DIR", default_missing_value = "./coverage", num_args = 0..=1)]
        coverage: Option<String>,
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

    /// Create a new Vais project
    New {
        /// Project name (creates directory with this name)
        name: String,

        /// Create a library project instead of a binary
        #[arg(long)]
        lib: bool,
    },

    /// Run tests in the project
    Test {
        /// Test file or directory (default: tests/)
        #[arg(default_value = "tests")]
        path: PathBuf,

        /// Filter tests by name pattern
        #[arg(long)]
        filter: Option<String>,

        /// Show verbose test output
        #[arg(short, long)]
        verbose: bool,

        /// Enable coverage instrumentation for tests
        /// Optional value: output directory for coverage data (default: ./coverage)
        #[arg(long, value_name = "DIR", default_missing_value = "./coverage", num_args = 0..=1)]
        coverage: Option<String>,
    },

    /// Run benchmarks
    Bench {
        /// Benchmark directory (default: benches/)
        #[arg(default_value = "benches")]
        path: PathBuf,

        /// Filter benchmarks by name pattern
        #[arg(long)]
        filter: Option<String>,
    },

    /// Auto-apply compiler suggested fixes
    Fix {
        /// Input source file or directory
        input: PathBuf,

        /// Show fixes without applying them
        #[arg(long)]
        dry_run: bool,
    },

    /// Run lint checks on source files
    Lint {
        /// Input source file or directory
        input: PathBuf,

        /// Warning level control: allow, warn, deny
        #[arg(short = 'W', long, value_name = "LEVEL")]
        warning_level: Option<String>,

        /// Output format (text or json)
        #[arg(long, default_value = "text")]
        format: String,
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

    /// Install a package binary globally (~/.vais/bin/)
    Install {
        /// Package path or name (local path to a vais project)
        package: String,

        /// Build with optimizations
        #[arg(long)]
        release: bool,
    },

    /// Uninstall a globally installed package binary
    Uninstall {
        /// Package name to uninstall
        package: String,
    },
}

fn main() {
    let cli = Cli::parse();

    // Install panic handler for crash reporting
    let report_crash = cli.report_crash;
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Always print the panic info
        default_hook(info);

        // Generate crash report if requested or always for ICEs
        if report_crash {
            let report = generate_crash_report(info);
            let report_path = format!("vaisc-crash-{}.txt", std::process::id());
            if let Ok(()) = std::fs::write(&report_path, &report) {
                eprintln!("\n{}: internal compiler error (ICE)", "error".red().bold());
                eprintln!("  Crash report saved to: {}", report_path);
                eprintln!("  Please report this bug at: https://github.com/vaislang/vais/issues");
            }
        } else {
            eprintln!("\n{}: internal compiler error (ICE)", "error".red().bold());
            eprintln!("  Use --report-crash to generate a detailed crash report");
            eprintln!("  Please report this bug at: https://github.com/vaislang/vais/issues");
        }
    }));

    // Configure ownership checking mode
    let ownership_mode = if cli.no_ownership_check {
        None // disabled
    } else if cli.warn_only_ownership {
        Some(false) // warn-only (legacy)
    } else {
        Some(true) // strict (default since Phase 34)
    };
    let _ = OWNERSHIP_MODE.set(ownership_mode);

    // Configure MIR borrow checking
    let _ = STRICT_BORROW.set(cli.strict_borrow);

    // Initialize i18n system
    let locale = cli.locale.as_ref().and_then(|s| Locale::parse(s));
    vais_i18n::init(locale);

    // Load plugins
    let plugins = if cli.no_plugins {
        PluginRegistry::new()
    } else {
        utils::load_plugins(&cli.plugin, cli.verbose, cli.allow_plugins)
    };

    // Set up compilation timeout
    let timeout_secs = cli.timeout;
    if timeout_secs > 0 {
        let timeout = std::time::Duration::from_secs(timeout_secs);
        std::thread::spawn(move || {
            std::thread::sleep(timeout);
            eprintln!(
                "error: compilation timed out after {} seconds",
                timeout_secs
            );
            exit(124);
        });
    }

    let result = match cli.command {
        Some(Commands::Build {
            input,
            output,
            emit_ir,
            opt_level,
            debug,
            target,
            force_rebuild,
            hot,
            gpu,
            gpu_host,
            gpu_compile,
            lto,
            no_lto,
            profile_generate,
            profile_use,
            suggest_fixes,
            parallel,
            inkwell: _build_inkwell,
            per_module,
            cache_limit,
            no_cache,
            warm_cache,
            clear_cache,
            cache_stats,
            coverage,
        }) => {
            // Resolve directory input to entry point file
            let (resolved_input, dir_dep_paths) = if input.is_dir() {
                let dir = &input;
                // Look for entry point
                let entry = if dir.join("main.vais").exists() {
                    dir.join("main.vais")
                } else if dir.join("src").join("main.vais").exists() {
                    dir.join("src").join("main.vais")
                } else if dir.join("lib.vais").exists() {
                    dir.join("lib.vais")
                } else if dir.join("src").join("lib.vais").exists() {
                    dir.join("src").join("lib.vais")
                } else {
                    let err_msg = format!(
                        "no entry point found in '{}': expected main.vais or lib.vais in directory or src/ subdirectory",
                        dir.display()
                    );
                    eprintln!("error: {}", err_msg);
                    exit(1);
                };

                // Collect search paths: the directory itself + src/ subdirectory
                let mut dep_paths = Vec::new();
                if dir.join("src").exists() {
                    dep_paths.push(dir.join("src"));
                }
                dep_paths.push(dir.to_path_buf());

                // If vais.toml exists, resolve package dependencies
                if let Some(pkg_dir) = package::find_manifest(dir) {
                    if let Ok(manifest) = package::load_manifest(&pkg_dir) {
                        let cache_root = package::default_registry_cache_root();
                        if let Ok(deps) = package::resolve_all_dependencies(
                            &manifest,
                            &pkg_dir,
                            cache_root.as_deref(),
                        ) {
                            for dep in &deps {
                                let src_dir = dep.path.join("src");
                                if src_dir.exists() {
                                    dep_paths.push(src_dir);
                                } else if dep.path.exists() {
                                    dep_paths.push(dep.path.clone());
                                }
                            }
                        }
                    }
                }

                if cli.verbose {
                    println!(
                        "{} Directory build: entry={}, search_paths=[{}]",
                        "info:".blue().bold(),
                        entry.display(),
                        dep_paths
                            .iter()
                            .map(|p| p.display().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }

                (entry, dep_paths)
            } else {
                (input.clone(), Vec::new())
            };

            // Set up module search paths for directory builds
            if !dir_dep_paths.is_empty() {
                let existing = std::env::var("VAIS_DEP_PATHS").unwrap_or_default();
                let new_paths: Vec<String> = dir_dep_paths
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect();
                let combined = if existing.is_empty() {
                    new_paths.join(":")
                } else {
                    format!("{}:{}", new_paths.join(":"), existing)
                };
                std::env::set_var("VAIS_DEP_PATHS", combined);
            }

            // Handle cache management CLI commands (exit early without building)
            if clear_cache {
                use incremental::get_cache_dir;
                let cache_dir = get_cache_dir(&resolved_input);
                // Resolve symlinks to prevent path traversal attacks
                let cache_dir = cache_dir.canonicalize().unwrap_or(cache_dir);
                if cache_dir.exists() {
                    if let Err(e) = std::fs::remove_dir_all(&cache_dir) {
                        Err(format!("cannot clear cache: {}", e))
                    } else {
                        println!(
                            "{} {}",
                            "Cache cleared:".green().bold(),
                            cache_dir.display()
                        );
                        Ok(())
                    }
                } else {
                    println!("No cache directory found at {}", cache_dir.display());
                    Ok(())
                }
            } else if cache_stats {
                use incremental::{get_cache_dir, IncrementalCache};
                let cache_dir = get_cache_dir(&resolved_input);
                match IncrementalCache::new(cache_dir.clone()) {
                    Ok(cache) => {
                        let stats = cache.stats();
                        println!("{}", "Incremental Cache Statistics".cyan().bold());
                        println!("  Cache dir:     {}", cache_dir.display());
                        println!("  Files cached:  {}", stats.total_files);
                        println!("  Dependencies:  {}", stats.total_dependencies);
                        if stats.last_build > 0 {
                            println!("  Last build:    {} (unix timestamp)", stats.last_build);
                        } else {
                            println!("  Last build:    (never)");
                        }
                        if let Ok(entries) = std::fs::read_dir(&cache_dir) {
                            let total_size: u64 = entries
                                .filter_map(|e| e.ok())
                                .filter_map(|e| e.metadata().ok())
                                .map(|m| m.len())
                                .sum();
                            println!(
                                "  Cache size:    {} bytes ({:.1} MB)",
                                total_size,
                                total_size as f64 / 1_048_576.0
                            );
                        }
                    }
                    Err(e) => {
                        println!("No cache available: {}", e);
                    }
                }
                Ok(())
            } else if warm_cache {
                use incremental::{get_cache_dir, IncrementalCache};
                let cache_dir = get_cache_dir(&resolved_input);
                match IncrementalCache::new(cache_dir) {
                    Ok(mut cache) => {
                        let project_root =
                            resolved_input.parent().unwrap_or(std::path::Path::new("."));
                        match cache.warm_cache(project_root) {
                            Ok(count) => {
                                let _ = cache.persist();
                                println!(
                                    "{} {} file(s) pre-cached",
                                    "Cache warmed:".green().bold(),
                                    count
                                );
                                Ok(())
                            }
                            Err(e) => Err(format!("warm cache failed: {}", e)),
                        }
                    }
                    Err(e) => Err(format!("cannot initialize cache: {}", e)),
                }
            } else
            // Check if JS target is specified
            if target
                .as_deref()
                .is_some_and(commands::build_js::is_js_target)
            {
                let js_config = commands::build_js::JsBuildConfig::default();
                commands::build_js::cmd_build_js(
                    &resolved_input,
                    output,
                    cli.verbose,
                    &plugins,
                    &js_config,
                )
            } else
            // Check if GPU target is specified
            if let Some(gpu_target_str) = &gpu {
                commands::build::cmd_build_gpu(
                    &resolved_input,
                    output,
                    gpu_target_str,
                    gpu_host,
                    gpu_compile,
                    cli.verbose,
                )
            } else {
                let target_triple = target
                    .as_ref()
                    .and_then(|s| TargetTriple::parse(s))
                    .unwrap_or(TargetTriple::Native);

                // Parse LTO mode with automatic ThinLTO for O2/O3
                let lto_mode = if no_lto {
                    vais_codegen::optimize::LtoMode::None
                } else if let Some(mode_str) = lto.as_deref() {
                    vais_codegen::optimize::LtoMode::parse(mode_str)
                } else if opt_level >= 2 {
                    vais_codegen::optimize::LtoMode::Thin
                } else {
                    vais_codegen::optimize::LtoMode::None
                };

                // Parse PGO mode (mutually exclusive: generate vs use)
                let pgo_mode = if let Some(dir) = profile_generate.as_deref() {
                    vais_codegen::optimize::PgoMode::Generate(dir.to_string())
                } else if let Some(path) = profile_use.as_deref() {
                    vais_codegen::optimize::PgoMode::Use(path.to_string())
                } else {
                    vais_codegen::optimize::PgoMode::None
                };

                // Parse coverage mode
                let coverage_mode = if let Some(dir) = coverage.as_deref() {
                    vais_codegen::optimize::CoverageMode::Enabled(dir.to_string())
                } else {
                    vais_codegen::optimize::CoverageMode::None
                };

                // Configure parallel compilation
                let parallel_config = parallel.map(vais_codegen::parallel::ParallelConfig::new);

                // Default to inkwell when feature is available
                #[cfg(feature = "inkwell")]
                let use_inkwell = true;
                #[cfg(not(feature = "inkwell"))]
                let use_inkwell = _build_inkwell || cli.inkwell;
                commands::build::cmd_build_with_timing(
                    &resolved_input,
                    output,
                    emit_ir,
                    opt_level,
                    debug,
                    cli.verbose,
                    cli.time,
                    &plugins,
                    target_triple,
                    force_rebuild || no_cache,
                    cli.gc,
                    cli.gc_threshold,
                    hot,
                    lto_mode,
                    pgo_mode,
                    coverage_mode,
                    suggest_fixes,
                    parallel_config,
                    use_inkwell,
                    per_module,
                    cache_limit,
                )
            }
        }
        Some(Commands::Run { input, args }) => {
            commands::simple::cmd_run(&input, &args, cli.verbose, &plugins)
        }
        Some(Commands::Check { input }) => {
            commands::simple::cmd_check(&input, cli.verbose, &plugins)
        }
        Some(Commands::Repl) => repl::run(),
        Some(Commands::Doc {
            input,
            output,
            format,
        }) => doc_gen::run(&input, &output, &format),
        Some(Commands::Version) => {
            println!("{} {}", "vaisc".bold(), env!("CARGO_PKG_VERSION"));
            println!("Vais 0.0.1 - AI-optimized systems programming language");
            Ok(())
        }
        Some(Commands::Fmt {
            input,
            check,
            indent,
        }) => commands::simple::cmd_fmt(&input, check, indent),
        Some(Commands::New { name, lib }) => commands::simple::cmd_new(&name, lib),
        Some(Commands::Test {
            path,
            filter,
            verbose,
            coverage,
        }) => {
            let coverage_mode = if let Some(dir) = coverage.as_deref() {
                vais_codegen::optimize::CoverageMode::Enabled(dir.to_string())
            } else {
                vais_codegen::optimize::CoverageMode::None
            };
            commands::test::cmd_test(
                &path,
                filter.as_deref(),
                verbose || cli.verbose,
                &coverage_mode,
            )
        }
        Some(Commands::Bench { path, filter }) => {
            commands::test::cmd_bench(&path, filter.as_deref(), cli.verbose)
        }
        Some(Commands::Fix { input, dry_run }) => {
            commands::test::cmd_fix(&input, dry_run, cli.verbose, &plugins)
        }
        Some(Commands::Lint {
            input,
            warning_level,
            format,
        }) => commands::test::cmd_lint(
            &input,
            warning_level.as_deref(),
            &format,
            cli.verbose,
            &plugins,
        ),
        Some(Commands::Pkg(pkg_cmd)) => commands::pkg::cmd_pkg(pkg_cmd, cli.verbose),
        Some(Commands::Pgo {
            input,
            output,
            run_cmd,
            profile_dir,
            merge_only,
        }) => commands::advanced::cmd_pgo(
            &input,
            output,
            run_cmd,
            &profile_dir,
            merge_only,
            cli.verbose,
            &plugins,
        ),
        Some(Commands::Watch { input, exec, args }) => {
            commands::advanced::cmd_watch(&input, exec.as_deref(), &args, cli.verbose, &plugins)
        }
        Some(Commands::Install { package, release }) => {
            commands::pkg::cmd_install(&package, release, cli.verbose, &plugins)
        }
        Some(Commands::Uninstall { package }) => commands::pkg::cmd_uninstall(&package),
        None => {
            // Direct file compilation
            if let Some(input) = cli.input {
                commands::build::cmd_build_with_timing(
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
                    vais_codegen::optimize::CoverageMode::None,
                    false,
                    None, // parallel_config
                    {
                        #[cfg(feature = "inkwell")]
                        {
                            true
                        }
                        #[cfg(not(feature = "inkwell"))]
                        {
                            cli.inkwell
                        }
                    },
                    false,     // per_module
                    536870912, // cache_limit (512MB default)
                )
            } else {
                println!(
                    "{}",
                    "Usage: vaisc <FILE.vais> or vaisc build <FILE.vais>".yellow()
                );
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

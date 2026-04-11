//! Simple commands (run, check, fmt, new).

use crate::commands::build::cmd_build;
use crate::configure_type_checker;
use crate::error_formatter;
use crate::imports::load_module_with_imports_internal;
use crate::utils::{print_plugin_diagnostics, walkdir};
use colored::Colorize;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use vais_codegen::TargetTriple;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_plugin::{DiagnosticLevel, PluginRegistry};
use vais_query::QueryDatabase;
use vais_types::TypeChecker;

pub(crate) fn cmd_run(
    input: &PathBuf,
    args: &[String],
    verbose: bool,
    plugins: &PluginRegistry,
    use_jit: bool,
) -> Result<(), String> {
    // When --jit is requested, try Cranelift JIT first and fall back to LLVM+clang on error.
    // The JIT path skips the entire clang link step (which measured at 96% of hello.vais
    // build time at Phase 4 iter 2), yielding a large wall-clock speedup for programs that
    // fit JIT's current feature coverage (single-file, i64 main, no extern IO).
    if use_jit {
        match cmd_run_jit(input, verbose) {
            Ok(()) => return Ok(()),
            Err(jit_err) => {
                if verbose {
                    println!(
                        "{} JIT path failed, falling back to LLVM+clang: {}",
                        "⚠".yellow().bold(),
                        jit_err
                    );
                }
                // Fall through to LLVM+clang path below
            }
        }
    }

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
        vais_codegen::optimize::CoverageMode::None,
        false,
        None,      // parallel_config
        false,     // use_inkwell
        false,     // per_module
        536870912, // cache_limit (512MB default)
        None,      // profile_out
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

/// Execute a Vais source file via Cranelift JIT without touching LLVM or clang.
///
/// Requires the `jit` cargo feature (returns an error when the feature is disabled,
/// so that cmd_run's fallback path takes over).
///
/// Current limitations:
/// - Single-file only (no imports)
/// - Program must define `F main() -> i64` or `__repl_main`
/// - JIT feature coverage is a subset of LLVM codegen (see `crates/vais-jit/src/tiered/tests.rs`)
#[cfg(feature = "jit")]
fn cmd_run_jit(input: &PathBuf, verbose: bool) -> Result<(), String> {
    use vais_jit::JitCompiler;

    let source = fs::read_to_string(input)
        .map_err(|e| format!("Cannot read '{}': {}", input.display(), e))?;

    if verbose {
        println!("{} {} (JIT)", "Running".green().bold(), input.display());
    }

    // Parse
    let ast = parse(&source).map_err(|e| format!("Parse error: {}", e))?;

    // Type check
    let mut checker = TypeChecker::new();
    configure_type_checker(&mut checker);
    checker
        .check_module(&ast)
        .map_err(|e| format!("Type error: {}", e))?;

    // JIT compile and run main
    let mut jit =
        JitCompiler::new().map_err(|e| format!("JIT init failed: {}", e))?;
    let exit_code = jit
        .compile_and_run_main(&ast)
        .map_err(|e| format!("JIT execution failed: {}", e))?;

    if exit_code != 0 {
        std::process::exit(exit_code as i32);
    }
    Ok(())
}

#[cfg(not(feature = "jit"))]
fn cmd_run_jit(_input: &PathBuf, _verbose: bool) -> Result<(), String> {
    Err("vaisc was built without the `jit` feature; rebuild with `--features jit`".to_string())
}

pub(crate) fn cmd_check(
    input: &PathBuf,
    verbose: bool,
    plugins: &PluginRegistry,
) -> Result<(), String> {
    // Canonicalize input path to ensure parent directory is resolvable
    let canonical_input = input
        .canonicalize()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default().join(input));

    let source = fs::read_to_string(&canonical_input)
        .map_err(|e| format!("Cannot read '{}': {}", input.display(), e))?;

    if verbose {
        println!("{} {}", "Checking".green().bold(), input.display());
    }

    // Tokenize (quick syntax check)
    let _tokens = tokenize(&source).map_err(|e| format!("Lexer error: {}", e))?;

    // Parse with import resolution — load all imported modules into a merged AST
    let mut query_db = QueryDatabase::new();
    query_db.set_cfg_values(std::collections::HashMap::new());
    let mut loaded_modules: HashSet<PathBuf> = HashSet::new();
    let mut loading_stack: Vec<PathBuf> = Vec::new();
    let source_root = canonical_input.parent().map(|p| p.to_path_buf());
    let merged = load_module_with_imports_internal(
        &canonical_input,
        &mut loaded_modules,
        &mut loading_stack,
        verbose,
        &source,
        &query_db,
        source_root.as_deref(),
    );

    let ast = match merged {
        Ok(module) => module,
        Err(import_err) => {
            // Fall back to single-file parse if import resolution fails
            if verbose {
                println!("{} import resolution: {}", "warning:".yellow().bold(), import_err);
            }
            let parsed = parse(&source)
                .map_err(|e| error_formatter::format_parse_error(&e, &source, input))?;
            vais_ast::Module {
                items: parsed.items,
                modules_map: None,
            }
        }
    };

    // Run lint plugins on merged AST
    if !plugins.is_empty() {
        // Convert Module to ast::Module for plugins
        let plugin_ast = parse(&source).unwrap_or_else(|_| vais_ast::Module {
            items: vec![],
            modules_map: None,
        });
        let diagnostics = plugins.run_lint(&plugin_ast);
        if !diagnostics.is_empty() {
            print_plugin_diagnostics(&diagnostics, &source, input);

            // Check if any errors (not just warnings)
            let has_errors = diagnostics
                .iter()
                .any(|d| d.level == DiagnosticLevel::Error);
            if has_errors {
                return Err("Plugin lint check failed".to_string());
            }
        }
    }

    // Type check merged AST (includes all imported struct/function definitions)
    let mut checker = TypeChecker::new();
    configure_type_checker(&mut checker);
    if let Err(e) = checker.check_module(&ast) {
        return Err(error_formatter::format_type_error(&e, &source, input));
    }

    // Print ownership warnings if any
    let ownership_warnings: Vec<_> = checker
        .get_warnings()
        .iter()
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

pub(crate) fn cmd_fmt(input: &PathBuf, check: bool, indent: usize) -> Result<(), String> {
    use vais_ast::formatter::{FormatConfig, Formatter};

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
                println!(
                    "{} needs formatting: {}",
                    "Would reformat".yellow(),
                    file.display()
                );
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

/// Create a new Vais project
pub(crate) fn cmd_new(name: &str, lib: bool, template: &str) -> Result<(), String> {
    // Determine effective template
    let effective_template = if lib {
        "lib"
    } else {
        match template {
            "binary" | "bin" => "binary",
            "lib" | "library" => "lib",
            "workspace" | "ws" => "workspace",
            other => {
                return Err(format!(
                    "unknown template '{}'. Available: binary, lib, workspace",
                    other
                ))
            }
        }
    };

    match effective_template {
        "workspace" => create_workspace_project(name),
        "lib" => create_standard_project(name, true),
        _ => create_standard_project(name, false),
    }
}

/// Create a standard (binary or library) project
fn create_standard_project(name: &str, lib: bool) -> Result<(), String> {
    use crate::package::init_package;

    let cwd =
        std::env::current_dir().map_err(|e| format!("failed to get current directory: {}", e))?;
    let project_dir = cwd.join(name);

    if project_dir.exists() {
        return Err(format!(
            "directory '{}' already exists",
            project_dir.display()
        ));
    }

    fs::create_dir_all(&project_dir)
        .map_err(|e| format!("failed to create directory '{}': {}", name, e))?;

    // Use existing init_package to create vais.toml + src/main.vais
    init_package(&project_dir, Some(name)).map_err(|e| e.to_string())?;

    // If library project, replace main.vais with lib.vais
    if lib {
        let main_path = project_dir.join("src").join("main.vais");
        let lib_path = project_dir.join("src").join("lib.vais");
        if main_path.exists() {
            fs::remove_file(&main_path)
                .map_err(|e| format!("failed to remove main.vais: {}", e))?;
        }
        let lib_content = format!(
            "# {} library\n\nF add(a: i64, b: i64) -> i64 {{\n    a + b\n}}\n",
            name
        );
        fs::write(&lib_path, lib_content)
            .map_err(|e| format!("failed to create lib.vais: {}", e))?;
    }

    // Create tests/ directory with a sample test
    let tests_dir = project_dir.join("tests");
    fs::create_dir_all(&tests_dir)
        .map_err(|e| format!("failed to create tests/ directory: {}", e))?;

    let test_content = if lib {
        format!(
            "# Tests for {}\n\nF test_add() -> i64 {{\n    result := add(2, 3)\n    I result == 5 {{\n        0\n    }} E {{\n        1\n    }}\n}}\n",
            name
        )
    } else {
        format!(
            "# Tests for {}\n\nF test_basic() -> i64 {{\n    # Basic test - return 0 for pass\n    0\n}}\n",
            name
        )
    };
    fs::write(tests_dir.join("test_main.vais"), test_content)
        .map_err(|e| format!("failed to create test file: {}", e))?;

    // Create .gitignore
    let gitignore_content = "target/\n*.ll\n*.o\n*.out\n.vais-cache/\n";
    fs::write(project_dir.join(".gitignore"), gitignore_content)
        .map_err(|e| format!("failed to create .gitignore: {}", e))?;

    println!(
        "{} Created {} project '{}'",
        "✓".green(),
        if lib { "library" } else { "binary" },
        name
    );
    println!("  {}", project_dir.display());
    println!();
    println!("  cd {}", name);
    if lib {
        println!("  vaisc build src/lib.vais");
    } else {
        println!("  vaisc build src/main.vais");
    }
    println!("  vaisc test");

    Ok(())
}

/// Create a workspace project with multiple member packages
fn create_workspace_project(name: &str) -> Result<(), String> {
    let cwd =
        std::env::current_dir().map_err(|e| format!("failed to get current directory: {}", e))?;
    let project_dir = cwd.join(name);

    if project_dir.exists() {
        return Err(format!(
            "directory '{}' already exists",
            project_dir.display()
        ));
    }

    // Create workspace root
    fs::create_dir_all(&project_dir)
        .map_err(|e| format!("failed to create directory '{}': {}", name, e))?;

    // Create workspace vais.toml
    let workspace_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
authors = []

[workspace]
members = [
    "crates/*",
]

[workspace.dependencies]
# Shared dependency versions for all workspace members
"#
    );
    fs::write(project_dir.join("vais.toml"), workspace_toml)
        .map_err(|e| format!("failed to create workspace vais.toml: {}", e))?;

    // Create crates/ directory
    let crates_dir = project_dir.join("crates");
    fs::create_dir_all(&crates_dir)
        .map_err(|e| format!("failed to create crates/ directory: {}", e))?;

    // Create a library crate
    let lib_name = format!("{}-core", name);
    let lib_dir = crates_dir.join(&lib_name);
    fs::create_dir_all(lib_dir.join("src"))
        .map_err(|e| format!("failed to create {}/src/ directory: {}", lib_name, e))?;

    let lib_toml = format!(
        r#"[package]
name = "{lib_name}"
version = "0.1.0"
authors = []
description = "Core library for {name}"
"#
    );
    fs::write(lib_dir.join("vais.toml"), lib_toml)
        .map_err(|e| format!("failed to create {}/vais.toml: {}", lib_name, e))?;

    let lib_content = format!(
        "# {lib_name} - core library\n\nP F add(a: i64, b: i64) -> i64 {{\n    a + b\n}}\n\nP F greet() -> i64 {{\n    print_str(\"{name} core library loaded\")\n    0\n}}\n"
    );
    fs::write(lib_dir.join("src").join("lib.vais"), lib_content)
        .map_err(|e| format!("failed to create {}/src/lib.vais: {}", lib_name, e))?;

    // Create a binary crate
    let bin_name = format!("{}-cli", name);
    let bin_dir = crates_dir.join(&bin_name);
    fs::create_dir_all(bin_dir.join("src"))
        .map_err(|e| format!("failed to create {}/src/ directory: {}", bin_name, e))?;

    let bin_toml = format!(
        r#"[package]
name = "{bin_name}"
version = "0.1.0"
authors = []
description = "CLI for {name}"

[dependencies]
{lib_name} = {{ path = "../{lib_name}" }}
"#
    );
    fs::write(bin_dir.join("vais.toml"), bin_toml)
        .map_err(|e| format!("failed to create {}/vais.toml: {}", bin_name, e))?;

    let bin_content = format!(
        "# {bin_name} - main binary\n\nF main() -> i64 {{\n    print_str(\"Hello from {name}!\")\n    0\n}}\n"
    );
    fs::write(bin_dir.join("src").join("main.vais"), bin_content)
        .map_err(|e| format!("failed to create {}/src/main.vais: {}", bin_name, e))?;

    // Create shared tests directory
    let tests_dir = project_dir.join("tests");
    fs::create_dir_all(&tests_dir)
        .map_err(|e| format!("failed to create tests/ directory: {}", e))?;

    let test_content = format!(
        "# Integration tests for {}\n\nF test_integration() -> i64 {{\n    # Integration test - return 0 for pass\n    0\n}}\n",
        name
    );
    fs::write(tests_dir.join("test_integration.vais"), test_content)
        .map_err(|e| format!("failed to create integration test: {}", e))?;

    // Create .gitignore
    let gitignore_content = "target/\n*.ll\n*.o\n*.out\n.vais-cache/\n";
    fs::write(project_dir.join(".gitignore"), gitignore_content)
        .map_err(|e| format!("failed to create .gitignore: {}", e))?;

    println!("{} Created workspace project '{}'", "✓".green(), name);
    println!("  {}", project_dir.display());
    println!("  Members:");
    println!("    crates/{} (library)", lib_name);
    println!("    crates/{} (binary)", bin_name);
    println!();
    println!("  cd {}", name);
    println!("  vaisc build crates/{}/src/main.vais", bin_name);
    println!("  vaisc test");

    Ok(())
}

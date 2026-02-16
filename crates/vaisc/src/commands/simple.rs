//! Simple commands (run, check, fmt, new).

use crate::commands::build::cmd_build;
use crate::configure_type_checker;
use crate::error_formatter;
use crate::utils::{print_plugin_diagnostics, walkdir};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use vais_codegen::TargetTriple;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_plugin::{DiagnosticLevel, PluginRegistry};
use vais_types::TypeChecker;

pub(crate) fn cmd_run(
    input: &PathBuf,
    args: &[String],
    verbose: bool,
    plugins: &PluginRegistry,
) -> Result<(), String> {
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

pub(crate) fn cmd_check(
    input: &PathBuf,
    verbose: bool,
    plugins: &PluginRegistry,
) -> Result<(), String> {
    let source = fs::read_to_string(input)
        .map_err(|e| format!("Cannot read '{}': {}", input.display(), e))?;

    if verbose {
        println!("{} {}", "Checking".green().bold(), input.display());
    }

    // Tokenize
    let _tokens = tokenize(&source).map_err(|e| format!("Lexer error: {}", e))?;

    // Parse
    let ast =
        parse(&source).map_err(|e| error_formatter::format_parse_error(&e, &source, input))?;

    // Run lint plugins
    if !plugins.is_empty() {
        let diagnostics = plugins.run_lint(&ast);
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

    // Type check
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
pub(crate) fn cmd_new(name: &str, lib: bool) -> Result<(), String> {
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

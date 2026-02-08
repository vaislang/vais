//! Test, benchmark, fix, and lint commands.

use crate::commands::build::cmd_build;
use crate::configure_type_checker;
use crate::utils::walkdir;
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use vais_codegen::{CodeGenerator, TargetTriple};
use vais_parser::parse;
use vais_plugin::{DiagnosticLevel, PluginRegistry};
use vais_types::TypeChecker;

pub(crate) fn cmd_test(
    path: &Path,
    filter: Option<&str>,
    verbose: bool,
    coverage_mode: &vais_codegen::optimize::CoverageMode,
) -> Result<(), String> {
    let test_dir = if path.is_dir() {
        path.to_path_buf()
    } else if path.is_file() {
        // Single test file
        return run_single_test(path, verbose, coverage_mode);
    } else {
        // Try relative to current directory
        let cwd = std::env::current_dir()
            .map_err(|e| format!("failed to get current directory: {}", e))?;
        let test_path = cwd.join(path);
        if test_path.is_dir() {
            test_path
        } else if test_path.is_file() {
            return run_single_test(&test_path, verbose, coverage_mode);
        } else {
            return Err(format!(
                "test path '{}' not found. Create a tests/ directory with .vais test files.",
                path.display()
            ));
        }
    };

    // Discover test files
    let mut test_files: Vec<PathBuf> = Vec::new();
    discover_test_files(&test_dir, &mut test_files)?;

    if test_files.is_empty() {
        println!(
            "{} No test files found in '{}'",
            "warning:".yellow().bold(),
            test_dir.display()
        );
        return Ok(());
    }

    // Apply filter
    let test_files: Vec<PathBuf> = if let Some(pattern) = filter {
        test_files
            .into_iter()
            .filter(|f| {
                f.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.contains(pattern))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        test_files
    };

    println!(
        "{} Running {} test file(s)...\n",
        "Testing".cyan().bold(),
        test_files.len()
    );

    let mut passed = 0;
    let mut failed = 0;
    let mut errors: Vec<(PathBuf, String)> = Vec::new();

    for test_file in &test_files {
        let result = run_single_test_inner(test_file, verbose, coverage_mode);
        match result {
            Ok(true) => {
                passed += 1;
                println!("  {} {}", "PASS".green().bold(), test_file.display());
            }
            Ok(false) => {
                failed += 1;
                println!("  {} {}", "FAIL".red().bold(), test_file.display());
            }
            Err(e) => {
                failed += 1;
                let msg = e.clone();
                errors.push((test_file.clone(), e));
                println!(
                    "  {} {} - {}",
                    "ERROR".red().bold(),
                    test_file.display(),
                    msg
                );
            }
        }
    }

    println!();
    if failed == 0 {
        println!("{} {} test(s) passed", "✓".green().bold(), passed);

        // Print coverage instructions if enabled
        if let Some(dir) = coverage_mode.coverage_dir() {
            println!();
            println!(
                "{} Coverage data collected in: {}/",
                "Coverage:".cyan().bold(),
                dir
            );
            println!("  Generate report:");
            println!(
                "    llvm-profdata merge -output={}/coverage.profdata {}/*.profraw",
                dir, dir
            );
            println!(
                "    llvm-cov report --instr-profile={}/coverage.profdata",
                dir
            );
        }

        Ok(())
    } else {
        if !errors.is_empty() && verbose {
            println!("\n{}", "Errors:".red().bold());
            for (path, err) in &errors {
                println!("  {}: {}", path.display(), err);
            }
        }
        Err(format!("{} passed, {} failed", passed, failed))
    }
}

/// Run benchmarks from benches/ directory
pub(crate) fn cmd_bench(path: &Path, filter: Option<&str>, verbose: bool) -> Result<(), String> {
    let bench_dir = if path.is_dir() {
        path.to_path_buf()
    } else {
        let cwd = std::env::current_dir()
            .map_err(|e| format!("failed to get current directory: {}", e))?;
        let bench_path = cwd.join(path);
        if bench_path.is_dir() {
            bench_path
        } else {
            return Err(format!(
                "benchmark directory '{}' not found. Create a benches/ directory with .vais benchmark files.",
                path.display()
            ));
        }
    };

    // Discover benchmark files
    let bench_files = walkdir(&bench_dir, "vais");
    if bench_files.is_empty() {
        println!(
            "{} No benchmark files found in '{}'",
            "warning:".yellow().bold(),
            bench_dir.display()
        );
        return Ok(());
    }

    // Apply filter
    let bench_files: Vec<PathBuf> = if let Some(pattern) = filter {
        bench_files
            .into_iter()
            .filter(|f| {
                f.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.contains(pattern))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        bench_files
    };

    println!(
        "{} Running {} benchmark(s)...\n",
        "Benchmarking".cyan().bold(),
        bench_files.len()
    );

    let plugins = PluginRegistry::new();
    let mut results: Vec<(PathBuf, std::time::Duration, bool)> = Vec::new();

    for bench_file in &bench_files {
        if verbose {
            println!("  {} {}", "Running".cyan(), bench_file.display());
        }

        let target_dir = std::env::temp_dir().join("vais-bench");
        let _ = fs::create_dir_all(&target_dir);
        let output_path = target_dir.join(
            bench_file
                .file_stem()
                .unwrap_or_default()
                .to_str()
                .unwrap_or("bench"),
        );

        // Compile
        let compile_result = cmd_build(
            bench_file,
            Some(output_path.clone()),
            false,
            0,
            false,
            false,
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
        );

        match compile_result {
            Ok(()) => {
                // Run and time it
                let start = std::time::Instant::now();
                let status = Command::new(&output_path)
                    .status()
                    .map_err(|e| format!("failed to run benchmark: {}", e))?;
                let elapsed = start.elapsed();
                results.push((bench_file.clone(), elapsed, status.success()));
            }
            Err(e) => {
                eprintln!(
                    "  {} {} - {}",
                    "ERROR".red().bold(),
                    bench_file.display(),
                    e
                );
                results.push((bench_file.clone(), std::time::Duration::ZERO, false));
            }
        }

        // Cleanup
        let _ = fs::remove_file(&output_path);
    }

    // Display results
    println!("\n{}", "Results:".bold());
    for (path, duration, success) in &results {
        let name = path.file_stem().unwrap_or_default().to_str().unwrap_or("?");
        if *success {
            println!(
                "  {} {} ... {:.3}ms",
                "✓".green(),
                name,
                duration.as_secs_f64() * 1000.0
            );
        } else {
            println!("  {} {} ... FAILED", "✗".red(), name);
        }
    }

    let passed = results.iter().filter(|(_, _, s)| *s).count();
    let failed = results.len() - passed;
    println!(
        "\n{} benchmarks: {} passed, {} failed",
        results.len(),
        passed,
        failed
    );

    if failed > 0 {
        Err(format!("{} benchmark(s) failed", failed))
    } else {
        Ok(())
    }
}

/// Auto-apply compiler suggested fixes
pub(crate) fn cmd_fix(
    input: &Path,
    dry_run: bool,
    verbose: bool,
    _plugins: &PluginRegistry,
) -> Result<(), String> {
    let files = if input.is_dir() {
        walkdir(&input.to_path_buf(), "vais")
    } else {
        vec![input.to_path_buf()]
    };

    if files.is_empty() {
        return Err(format!("no .vais files found in '{}'", input.display()));
    }

    let total_fixes = 0;
    let fixed_files = 0;

    for file in &files {
        if verbose {
            println!("{} Checking {}", "Fix".cyan(), file.display());
        }

        let source = fs::read_to_string(file)
            .map_err(|e| format!("failed to read {}: {}", file.display(), e))?;

        // Parse to check for syntax errors
        let tokens = vais_lexer::tokenize(&source);
        let _tokens = match tokens {
            Ok(t) => t,
            Err(_) => {
                if verbose {
                    println!(
                        "  {} {} — lexer error, skipping",
                        "⚠".yellow(),
                        file.display()
                    );
                }
                continue;
            }
        };

        let module = match vais_parser::parse(&source) {
            Ok(m) => m,
            Err(_) => {
                if verbose {
                    println!(
                        "  {} {} — parse error, skipping",
                        "⚠".yellow(),
                        file.display()
                    );
                }
                continue;
            }
        };

        // Type check
        let mut checker = TypeChecker::new();
        configure_type_checker(&mut checker);

        // For now, just report that fix functionality is limited
        // (TypeChecker returns a single error, not a list with suggestions)
        if let Err(_e) = checker.check_module(&module) {
            // TypeErrors don't consistently have suggestions
            // This is a simplified implementation
            if verbose {
                println!("  {} {} — has type errors", "→".cyan(), file.display());
            }
        }
    }

    if total_fixes == 0 {
        println!("{} No automatic fixes available", "✓".green());
        println!(
            "{}",
            "Note: The fix command currently has limited functionality.".dimmed()
        );
    } else if dry_run {
        println!(
            "\n{} {} fix(es) available in {} file(s) (dry run — no changes made)",
            "→".cyan(),
            total_fixes,
            fixed_files
        );
    } else {
        println!(
            "\n{} Applied {} fix(es) in {} file(s)",
            "✓".green(),
            total_fixes,
            fixed_files
        );
    }

    Ok(())
}

/// Run lint checks on source files
pub(crate) fn cmd_lint(
    input: &Path,
    warning_level: Option<&str>,
    format: &str,
    verbose: bool,
    plugins: &PluginRegistry,
) -> Result<(), String> {
    let files = if input.is_dir() {
        walkdir(&input.to_path_buf(), "vais")
    } else {
        vec![input.to_path_buf()]
    };

    if files.is_empty() {
        return Err(format!("no .vais files found in '{}'", input.display()));
    }

    let level = match warning_level {
        Some("allow") => 0, // suppress warnings
        Some("deny") => 2,  // treat warnings as errors
        _ => 1,             // default: warn
    };

    let mut total_warnings = 0;
    let mut total_errors = 0;
    let mut all_diagnostics: Vec<serde_json::Value> = Vec::new();

    for file in &files {
        if verbose {
            println!("{} Linting {}", "Lint".cyan(), file.display());
        }

        let source = fs::read_to_string(file)
            .map_err(|e| format!("failed to read {}: {}", file.display(), e))?;

        let _tokens = match vais_lexer::tokenize(&source) {
            Ok(t) => t,
            Err(e) => {
                total_errors += 1;
                if format == "json" {
                    let diag = serde_json::json!({
                        "file": file.display().to_string(),
                        "code": "L001",
                        "message": format!("Lexer error: {}", e),
                        "severity": "error",
                    });
                    all_diagnostics.push(diag);
                } else {
                    eprintln!("{}: [L001] Lexer error: {}", "error".red().bold(), e);
                    eprintln!("  {} {}", "-->".blue().bold(), file.display());
                }
                continue;
            }
        };

        let module = match vais_parser::parse(&source) {
            Ok(m) => m,
            Err(e) => {
                total_errors += 1;
                if format == "json" {
                    let diag = serde_json::json!({
                        "file": file.display().to_string(),
                        "code": e.error_code(),
                        "message": e.localized_message(),
                        "severity": "error",
                    });
                    all_diagnostics.push(diag);
                } else {
                    eprintln!(
                        "{}: [{}] {}",
                        "error".red().bold(),
                        e.error_code(),
                        e.localized_message()
                    );
                    eprintln!("  {} {}", "-->".blue().bold(), file.display());
                }
                continue;
            }
        };

        // Run plugin lints
        if !plugins.is_empty() {
            let diagnostics = plugins.run_lint(&module);
            for diag in &diagnostics {
                let is_warning = matches!(diag.level, DiagnosticLevel::Warning);

                if is_warning {
                    if level == 0 {
                        continue;
                    } // allow — skip warnings
                    total_warnings += 1;
                    if level == 2 {
                        total_errors += 1;
                    } // deny — count as error
                } else {
                    total_errors += 1;
                }

                if format == "json" {
                    let json_diag = serde_json::json!({
                        "file": file.display().to_string(),
                        "message": &diag.message,
                        "severity": if is_warning { "warning" } else { "error" },
                        "line": diag.span.map(|s| source[..s.start].matches('\n').count() + 1),
                    });
                    all_diagnostics.push(json_diag);
                } else {
                    let severity = if is_warning && level < 2 {
                        "warning".yellow().bold().to_string()
                    } else {
                        "error".red().bold().to_string()
                    };

                    eprintln!("{}: {}", severity, diag.message);

                    let line_info = diag
                        .span
                        .map(|s| {
                            let line = source[..s.start].matches('\n').count() + 1;
                            format!("{}:{}", file.display(), line)
                        })
                        .unwrap_or_else(|| file.display().to_string());
                    eprintln!("  {} {}", "-->".blue().bold(), line_info);
                }
            }
        }

        // Type check
        let mut checker = TypeChecker::new();
        configure_type_checker(&mut checker);

        if let Err(e) = checker.check_module(&module) {
            total_errors += 1;
            if format == "json" {
                let diag = serde_json::json!({
                    "file": file.display().to_string(),
                    "code": e.error_code(),
                    "message": e.localized_message(),
                    "severity": "error",
                });
                all_diagnostics.push(diag);
            } else {
                eprintln!(
                    "{}: [{}] {}",
                    "error".red().bold(),
                    e.error_code(),
                    e.localized_message()
                );
                eprintln!("  {} {}", "-->".blue().bold(), file.display());
            }
        }

        // Check warnings
        let warnings = checker.get_warnings();
        for w in warnings {
            if level == 0 {
                continue;
            } // allow — skip warnings
            total_warnings += 1;
            if level == 2 {
                total_errors += 1;
            } // deny — count as error

            if format == "json" {
                let diag = serde_json::json!({
                    "file": file.display().to_string(),
                    "code": "W000",
                    "message": w,
                    "severity": "warning",
                });
                all_diagnostics.push(diag);
            } else {
                let severity = if level < 2 {
                    "warning".yellow().bold().to_string()
                } else {
                    "error".red().bold().to_string()
                };
                eprintln!("{}: {}", severity, w);
                eprintln!("  {} {}", "-->".blue().bold(), file.display());
            }
        }
    }

    if format == "json" {
        let output = serde_json::json!({
            "diagnostics": all_diagnostics,
            "summary": {
                "errors": total_errors,
                "warnings": total_warnings,
                "files_checked": files.len(),
            }
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
    } else {
        println!(
            "\n{} {} file(s) checked: {} error(s), {} warning(s)",
            if total_errors == 0 {
                "✓".green().to_string()
            } else {
                "✗".red().to_string()
            },
            files.len(),
            total_errors,
            total_warnings
        );
    }

    if total_errors > 0 {
        Err(format!("{} error(s) found", total_errors))
    } else {
        Ok(())
    }
}

pub(crate) fn discover_test_files(dir: &Path, results: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries =
        fs::read_dir(dir).map_err(|e| format!("cannot read '{}': {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("failed reading directory entry: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            discover_test_files(&path, results)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("vais") {
            results.push(path);
        }
    }

    results.sort();
    Ok(())
}

pub(crate) fn run_single_test(
    path: &Path,
    verbose: bool,
    coverage_mode: &vais_codegen::optimize::CoverageMode,
) -> Result<(), String> {
    println!(
        "{} Running test: {}\n",
        "Testing".cyan().bold(),
        path.display()
    );
    match run_single_test_inner(path, verbose, coverage_mode) {
        Ok(true) => {
            println!("  {} {}", "PASS".green().bold(), path.display());
            Ok(())
        }
        Ok(false) => {
            println!("  {} {}", "FAIL".red().bold(), path.display());
            Err("test failed (non-zero exit code)".to_string())
        }
        Err(e) => {
            println!("  {} {} - {}", "ERROR".red().bold(), path.display(), e);
            Err(e)
        }
    }
}

pub(crate) fn run_single_test_inner(
    path: &Path,
    verbose: bool,
    coverage_mode: &vais_codegen::optimize::CoverageMode,
) -> Result<bool, String> {
    use std::process::Command;

    // Step 1: Compile to LLVM IR
    let ir = compile_to_ir_for_test(path)?;

    // Step 2: Write IR to temp file
    let tmp_dir = std::env::temp_dir();
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("test");
    let ir_path = tmp_dir.join(format!("vais_test_{}.ll", stem));
    let bin_path = tmp_dir.join(format!("vais_test_{}", stem));

    fs::write(&ir_path, &ir).map_err(|e| format!("failed to write IR: {}", e))?;

    // Step 3: Compile IR to binary with clang
    let ir_path_str = ir_path.to_str().ok_or_else(|| "IR path contains invalid UTF-8".to_string())?;
    let bin_path_str = bin_path.to_str().ok_or_else(|| "binary path contains invalid UTF-8".to_string())?;

    let mut clang_args = vec![
        ir_path_str.to_string(),
        "-o".to_string(),
        bin_path_str.to_string(),
        "-lm".to_string(),
    ];

    // Add coverage flags if enabled
    for flag in coverage_mode.clang_flags() {
        clang_args.push(flag.to_string());
    }

    let clang_output = Command::new("clang")
        .args(&clang_args)
        .output()
        .map_err(|e| format!("failed to run clang: {}", e))?;

    if !clang_output.status.success() {
        let stderr = String::from_utf8_lossy(&clang_output.stderr);
        return Err(format!("clang compilation failed: {}", stderr));
    }

    // Step 4: Run the binary (with LLVM_PROFILE_FILE if coverage is enabled)
    let mut cmd = Command::new(&bin_path);
    if let Some(dir) = coverage_mode.coverage_dir() {
        let cov_dir = std::path::Path::new(dir);
        if !cov_dir.exists() {
            let _ = std::fs::create_dir_all(cov_dir);
        }
        cmd.env(
            "LLVM_PROFILE_FILE",
            format!("{}/{}_test_%m.profraw", dir, stem),
        );
    }
    let run_output = cmd
        .output()
        .map_err(|e| format!("failed to run test binary: {}", e))?;

    if verbose {
        let stdout = String::from_utf8_lossy(&run_output.stdout);
        if !stdout.is_empty() {
            println!("    stdout: {}", stdout.trim());
        }
        let stderr = String::from_utf8_lossy(&run_output.stderr);
        if !stderr.is_empty() {
            println!("    stderr: {}", stderr.trim());
        }
    }

    // Clean up
    let _ = fs::remove_file(&ir_path);
    let _ = fs::remove_file(&bin_path);

    // Exit code 0 = pass
    Ok(run_output.status.code() == Some(0))
}

pub(crate) fn compile_to_ir_for_test(path: &Path) -> Result<String, String> {
    let source =
        fs::read_to_string(path).map_err(|e| format!("cannot read '{}': {}", path.display(), e))?;

    // Parse
    let ast = parse(&source).map_err(|e| format!("parse error: {:?}", e))?;

    // Type check
    let mut checker = TypeChecker::new();
    configure_type_checker(&mut checker);
    if let Err(e) = checker.check_module(&ast) {
        return Err(format!("type error: {:?}", e));
    }

    // Codegen
    let module_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("test");
    let mut codegen = CodeGenerator::new_with_target(module_name, TargetTriple::Native);
    codegen.set_resolved_functions(checker.get_all_functions().clone());

    let instantiations = checker.get_generic_instantiations();
    let ir = if instantiations.is_empty() {
        codegen.generate_module(&ast)
    } else {
        codegen.generate_module_with_instantiations(&ast, instantiations)
    }
    .map_err(|e| format!("codegen error: {}", e))?;

    Ok(ir)
}

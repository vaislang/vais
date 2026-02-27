//! Advanced commands (PGO, watch).

use crate::commands::build::cmd_build;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use vais_codegen::TargetTriple;
use vais_plugin::PluginRegistry;

/// Parse a command string respecting quoted arguments.
/// Rejects shell metacharacters for safety.
fn parse_command(cmd: &str) -> Result<Vec<String>, String> {
    // Reject shell metacharacters
    for ch in cmd.chars() {
        if matches!(ch, ';' | '|' | '&' | '$' | '`' | '>' | '<') {
            return Err(format!("command contains unsafe character '{}'", ch));
        }
    }

    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    for ch in cmd.chars() {
        match ch {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if in_single_quote || in_double_quote {
        return Err("unterminated quote in command".to_string());
    }

    if !current.is_empty() {
        args.push(current);
    }

    if args.is_empty() {
        return Err("empty run command".to_string());
    }

    Ok(args)
}

pub(crate) fn cmd_pgo(
    input: &PathBuf,
    output: Option<PathBuf>,
    run_cmd: Option<String>,
    profile_dir: &str,
    merge_only: bool,
    verbose: bool,
    plugins: &PluginRegistry,
) -> Result<(), String> {
    let bin_name = input
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("invalid input file stem: {}", input.display()))?;
    let output_path = output.unwrap_or_else(|| PathBuf::from(bin_name));
    let profdata_path = format!("{}/default.profdata", profile_dir);

    if !merge_only {
        // Step 1: Build with instrumentation
        println!(
            "{} Step 1/3: Building instrumented binary...",
            "[PGO]".cyan().bold()
        );
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
            vais_codegen::optimize::CoverageMode::None,
            false,     // suggest_fixes
            None,      // parallel_config
            false,     // use_inkwell
            false,     // per_module
            536870912, // cache_limit (512MB default)
        )?;

        println!(
            "{} Instrumented binary: {}",
            "  ✓".green(),
            instrumented_bin.display()
        );

        // Step 2: Run to collect profile data
        println!(
            "{} Step 2/3: Running to collect profile data...",
            "[PGO]".cyan().bold()
        );
        let run_command = run_cmd.unwrap_or_else(|| instrumented_bin.display().to_string());

        // Parse command safely respecting quotes and rejecting shell metacharacters
        let parts = parse_command(&run_command)?;

        let status = Command::new(&parts[0])
            .args(&parts[1..])
            .env(
                "LLVM_PROFILE_FILE",
                format!("{}/default-%p.profraw", profile_dir),
            )
            .status()
            .map_err(|e| format!("failed to run instrumented binary: {}", e))?;

        if !status.success() {
            println!("{} Instrumented binary exited with non-zero status (profile data may still be usable)", "  ⚠".yellow());
        } else {
            println!(
                "{} Profile data collected in {}/",
                "  ✓".green(),
                profile_dir
            );
        }

        // Clean up instrumented binary
        let _ = fs::remove_file(&instrumented_bin);
    }

    // Step 3: Merge profile data and rebuild
    println!(
        "{} Step 3/3: Merging profiles and rebuilding with optimization...",
        "[PGO]".cyan().bold()
    );

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
                    return Err(
                        "Failed to merge profile data. Ensure llvm-profdata is installed."
                            .to_string(),
                    );
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
        vais_codegen::optimize::CoverageMode::None,
        false,     // suggest_fixes
        None,      // parallel_config
        false,     // use_inkwell
        false,     // per_module
        536870912, // cache_limit (512MB default)
    )?;

    println!(
        "{} PGO-optimized binary: {}",
        "  ✓".green(),
        output_path.display()
    );
    println!("\n{} PGO workflow complete!", "Done".green().bold());

    Ok(())
}

/// Watch for file changes and recompile
pub(crate) fn cmd_watch(
    input: &PathBuf,
    exec: Option<&str>,
    args: &[String],
    verbose: bool,
    plugins: &PluginRegistry,
) -> Result<(), String> {
    use std::collections::HashSet;
    use std::time::Duration;

    // Determine watch directory (parent of input file or current directory)
    let watch_dir = input
        .parent()
        .ok_or_else(|| format!("cannot determine parent directory of {}", input.display()))?
        .to_path_buf();

    println!(
        "{} {} (directory: {})",
        "Watching".cyan().bold(),
        input.display(),
        watch_dir.display()
    );

    // Collect all .vais files to watch (for import tracking)
    let mut watched_files: HashSet<PathBuf> = HashSet::new();
    let canonical_input = input.canonicalize().map_err(|e| {
        format!(
            "failed to canonicalize input path {}: {}",
            input.display(),
            e
        )
    })?;
    watched_files.insert(canonical_input);

    // Scan for import statements and add imported files
    if let Ok(content) = std::fs::read_to_string(input) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("I ") || trimmed.starts_with("import ") {
                // Extract import path: I "path/to/file" or import "path/to/file"
                if let Some(start) = trimmed.find('"') {
                    if let Some(end) = trimmed[start + 1..].find('"') {
                        let import_path = &trimmed[start + 1..start + 1 + end];
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
        println!(
            "{} Watching {} file(s)",
            "Info".blue().bold(),
            watched_files.len()
        );
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
        vais_codegen::optimize::CoverageMode::None,
        false,
        None,      // parallel_config
        false,     // use_inkwell
        false,     // per_module
        536870912, // cache_limit (512MB default)
    )?;

    // Execute initial run if requested
    if let Some(cmd) = exec {
        if verbose {
            println!("{} {}", "Running".green().bold(), cmd);
        }
        let _ = Command::new(cmd).args(args).status();
    }

    // Create file watcher using notify crate
    use notify::{RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        notify::Config::default(),
    )
    .map_err(|e| format!("Failed to create watcher: {}", e))?;

    // Watch the directory recursively for .vais files
    watcher
        .watch(&watch_dir, RecursiveMode::Recursive)
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
                    let is_vais_file = event
                        .paths
                        .iter()
                        .any(|p| p.extension().is_some_and(|ext| ext == "vais"));

                    if !is_vais_file {
                        continue;
                    }

                    let changed_files: Vec<_> = event
                        .paths
                        .iter()
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
                        vais_codegen::optimize::CoverageMode::None,
                        false,
                        None,      // parallel_config
                        false,     // use_inkwell
                        false,     // per_module
                        536870912, // cache_limit (512MB default)
                    ) {
                        Ok(_) => {
                            println!("{} Compilation successful", "✓".green().bold());

                            // Execute if requested
                            if let Some(cmd) = exec {
                                println!("{} {}", "Running".green().bold(), cmd);
                                let _ = Command::new(cmd).args(args).status();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_simple() {
        let result = parse_command("./program arg1 arg2").unwrap();
        assert_eq!(result, vec!["./program", "arg1", "arg2"]);
    }

    #[test]
    fn test_parse_command_double_quotes() {
        let result = parse_command(r#"./program "arg with spaces" arg2"#).unwrap();
        assert_eq!(result, vec!["./program", "arg with spaces", "arg2"]);
    }

    #[test]
    fn test_parse_command_single_quotes() {
        let result = parse_command("./program 'arg with spaces' arg2").unwrap();
        assert_eq!(result, vec!["./program", "arg with spaces", "arg2"]);
    }

    #[test]
    fn test_parse_command_mixed_quotes() {
        let result = parse_command(r#"./program "double quoted" 'single quoted' normal"#).unwrap();
        assert_eq!(
            result,
            vec!["./program", "double quoted", "single quoted", "normal"]
        );
    }

    #[test]
    fn test_parse_command_nested_quotes() {
        let result = parse_command(r#"./program "it's ok" 'he said "hi"'"#).unwrap();
        assert_eq!(result, vec!["./program", "it's ok", r#"he said "hi""#]);
    }

    #[test]
    fn test_parse_command_empty() {
        let result = parse_command("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "empty run command");
    }

    #[test]
    fn test_parse_command_whitespace_only() {
        let result = parse_command("   ");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "empty run command");
    }

    #[test]
    fn test_parse_command_unterminated_double_quote() {
        let result = parse_command(r#"./program "unterminated"#);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "unterminated quote in command");
    }

    #[test]
    fn test_parse_command_unterminated_single_quote() {
        let result = parse_command("./program 'unterminated");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "unterminated quote in command");
    }

    #[test]
    fn test_parse_command_reject_semicolon() {
        let result = parse_command("./program; rm -rf /");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsafe character ';'"));
    }

    #[test]
    fn test_parse_command_reject_pipe() {
        let result = parse_command("./program | cat");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsafe character '|'"));
    }

    #[test]
    fn test_parse_command_reject_ampersand() {
        let result = parse_command("./program && echo hi");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsafe character '&'"));
    }

    #[test]
    fn test_parse_command_reject_dollar() {
        let result = parse_command("./program $(whoami)");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsafe character '$'"));
    }

    #[test]
    fn test_parse_command_reject_backtick() {
        let result = parse_command("./program `ls`");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsafe character '`'"));
    }

    #[test]
    fn test_parse_command_reject_redirect_out() {
        let result = parse_command("./program > output.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsafe character '>'"));
    }

    #[test]
    fn test_parse_command_reject_redirect_in() {
        let result = parse_command("./program < input.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsafe character '<'"));
    }

    #[test]
    fn test_parse_command_tabs() {
        let result = parse_command("./program\targ1\t\targ2").unwrap();
        assert_eq!(result, vec!["./program", "arg1", "arg2"]);
    }

    #[test]
    fn test_parse_command_multiple_spaces() {
        let result = parse_command("./program    arg1     arg2").unwrap();
        assert_eq!(result, vec!["./program", "arg1", "arg2"]);
    }
}

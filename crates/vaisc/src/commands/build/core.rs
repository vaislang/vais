//! Core build command implementation.
//!
//! Orchestrates the compilation pipeline: parse, macro expand, type check,
//! codegen, optimize, and link. Delegates to `parallel` and `serial` submodules
//! for the actual codegen paths.

use crate::configure_type_checker;
use crate::error_formatter;
use crate::imports::{load_module_with_imports_internal, load_module_with_imports_parallel};
use crate::incremental;
use crate::utils::{print_plugin_diagnostics, print_suggested_fixes};
use colored::Colorize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use vais_ast::Item;
use vais_codegen::TargetTriple;
use vais_macro::{collect_macros, expand_macros, process_derives, MacroRegistry};
use vais_plugin::{DiagnosticLevel, PluginRegistry};
use vais_query::QueryDatabase;
use vais_types::TypeChecker;

/// Per-phase compilation timing profile.
/// Used by `--profile` flag to display detailed pipeline breakdown.
#[derive(Debug, Clone, Default)]
pub struct CompileProfile {
    pub parse_ms: f64,
    pub macro_ms: f64,
    pub typecheck_ms: f64,
    pub codegen_ms: f64,
    pub optimize_ms: f64,
    pub clang_ms: f64,
    pub total_ms: f64,
}

impl CompileProfile {
    /// Print a detailed profile table with percentages.
    pub fn print(&self) {
        let total = self.total_ms.max(0.001); // avoid division by zero
        println!("\n{}", "=== Compilation Profile ===".cyan().bold());
        println!(
            "  {:<16} {:>8.1}ms  ({:>5.1}%)",
            "Parse + Import",
            self.parse_ms,
            self.parse_ms / total * 100.0
        );
        println!(
            "  {:<16} {:>8.1}ms  ({:>5.1}%)",
            "Macro Expand",
            self.macro_ms,
            self.macro_ms / total * 100.0
        );
        println!(
            "  {:<16} {:>8.1}ms  ({:>5.1}%)",
            "Type Check",
            self.typecheck_ms,
            self.typecheck_ms / total * 100.0
        );
        println!(
            "  {:<16} {:>8.1}ms  ({:>5.1}%)",
            "Codegen",
            self.codegen_ms,
            self.codegen_ms / total * 100.0
        );
        println!(
            "  {:<16} {:>8.1}ms  ({:>5.1}%)",
            "IR Optimize",
            self.optimize_ms,
            self.optimize_ms / total * 100.0
        );
        println!(
            "  {:<16} {:>8.1}ms  ({:>5.1}%)",
            "Clang Link",
            self.clang_ms,
            self.clang_ms / total * 100.0
        );
        println!("  {}", "─".repeat(40));
        println!("  {:<16} {:>8.1}ms", "Total", self.total_ms);
    }
}

/// Wrapper around cmd_build that optionally prints timing information
#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_build_with_timing(
    input: &PathBuf,
    output: Option<PathBuf>,
    emit_ir: bool,
    opt_level: u8,
    debug: bool,
    verbose: bool,
    time: bool,
    profile: bool,
    plugins: &PluginRegistry,
    target: TargetTriple,
    force_rebuild: bool,
    gc: bool,
    gc_threshold: Option<usize>,
    hot: bool,
    lto_mode: vais_codegen::optimize::LtoMode,
    pgo_mode: vais_codegen::optimize::PgoMode,
    coverage_mode: vais_codegen::optimize::CoverageMode,
    suggest_fixes: bool,
    parallel_config: Option<vais_codegen::parallel::ParallelConfig>,
    use_inkwell: bool,
    per_module: bool,
    cache_limit: u64,
) -> Result<(), String> {
    use std::time::Instant;

    let start = Instant::now();
    let mut compile_profile = CompileProfile::default();
    let result = cmd_build(
        input,
        output,
        emit_ir,
        opt_level,
        debug,
        verbose,
        plugins,
        target,
        force_rebuild,
        gc,
        gc_threshold,
        hot,
        lto_mode,
        pgo_mode,
        coverage_mode,
        suggest_fixes,
        parallel_config,
        use_inkwell,
        per_module,
        cache_limit,
        if profile {
            Some(&mut compile_profile)
        } else {
            None
        },
    );
    let elapsed = start.elapsed();

    if time {
        println!(
            "\n{} Total compilation time: {:.3}s",
            "⏱".cyan().bold(),
            elapsed.as_secs_f64()
        );
    }

    if profile {
        compile_profile.total_ms = elapsed.as_secs_f64() * 1000.0;
        compile_profile.print();
    }

    result
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::result_large_err)]
pub(crate) fn cmd_build(
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
    coverage_mode: vais_codegen::optimize::CoverageMode,
    suggest_fixes: bool,
    parallel_config: Option<vais_codegen::parallel::ParallelConfig>,
    use_inkwell: bool,
    per_module: bool,
    cache_limit: u64,
    mut profile_out: Option<&mut CompileProfile>,
) -> Result<(), String> {
    use incremental::{get_cache_dir, CompilationOptions, IncrementalCache};

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
            if let Some(result) = check_cache_skip(c, input, &output, emit_ir, verbose, &target)? {
                return result;
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
            println!(
                "{} Parallel compilation enabled ({} threads)",
                "⚡".cyan().bold(),
                config.effective_threads()
            );
        }
    }

    // Read source for error reporting
    let main_source = fs::read_to_string(input)
        .map_err(|e| format!("Cannot read '{}': {}", input.display(), e))?;

    // Initialize query database for memoized parsing
    let mut query_db = QueryDatabase::new();

    // Set cfg values from target triple for conditional compilation
    let mut cfg = target.cfg_values();

    // Inject feature flags into cfg values (set by `vaisc pkg build --features`)
    if let Ok(features_str) = std::env::var("VAIS_FEATURES") {
        for feat in features_str.split(',') {
            let feat = feat.trim();
            if !feat.is_empty() {
                cfg.insert(format!("feature:{}", feat), feat.to_string());
            }
        }
    }

    query_db.set_cfg_values(cfg);

    // Parse main file and resolve imports
    let parse_start = std::time::Instant::now();
    let mut loaded_modules: HashSet<PathBuf> = HashSet::new();
    let mut loading_stack: Vec<PathBuf> = Vec::new();
    let merged_ast = if use_parallel {
        load_module_with_imports_parallel(
            input,
            &mut loaded_modules,
            verbose,
            &main_source,
            &query_db,
        )?
    } else {
        load_module_with_imports_internal(
            input,
            &mut loaded_modules,
            &mut loading_stack,
            verbose,
            &main_source,
            &query_db,
        )?
    };
    let parse_time = parse_start.elapsed();
    if let Some(ref mut p) = profile_out {
        p.parse_ms = parse_time.as_secs_f64() * 1000.0;
    }

    if verbose {
        println!(
            "  {} total items (including imports)",
            merged_ast.items.len()
        );
        println!(
            "  {} Parse time: {:.3}s",
            "⏱".cyan(),
            parse_time.as_secs_f64()
        );
    }

    // Run lint plugins
    if !plugins.is_empty() {
        let diagnostics = plugins.run_lint(&merged_ast);
        if !diagnostics.is_empty() {
            print_plugin_diagnostics(&diagnostics, &main_source, input);

            // Check if any errors (not just warnings)
            let has_errors = diagnostics
                .iter()
                .any(|d| d.level == DiagnosticLevel::Error);
            if has_errors {
                return Err("Plugin lint check failed".to_string());
            }
        }
    }

    // Run transform plugins
    let transformed_ast = if !plugins.is_empty() {
        plugins
            .run_transform(merged_ast)
            .map_err(|e| format!("Plugin transform error: {}", e))?
    } else {
        merged_ast
    };

    // Macro expansion phase
    let macro_start = std::time::Instant::now();
    let mut macro_registry = MacroRegistry::new();

    // Register builtin panic! macro: panic!("msg") => __panic("msg")
    {
        use vais_ast::{
            MacroDef, MacroPattern, MacroPatternElement, MacroRule, MacroTemplate,
            MacroTemplateElement, MacroToken, MetaVarKind, Span, Spanned, Delimiter,
        };
        macro_registry.register(MacroDef {
            name: Spanned::new("panic".to_string(), Span::new(0, 5)),
            rules: vec![MacroRule {
                pattern: MacroPattern::Sequence(vec![MacroPatternElement::MetaVar {
                    name: "msg".to_string(),
                    kind: MetaVarKind::Expr,
                }]),
                template: MacroTemplate::Sequence(vec![
                    MacroTemplateElement::Token(MacroToken::Ident("__panic".to_string())),
                    MacroTemplateElement::Group {
                        delimiter: Delimiter::Paren,
                        content: vec![MacroTemplateElement::MetaVar("msg".to_string())],
                    },
                ]),
            }],
            is_pub: false,
        });
    }

    collect_macros(&transformed_ast, &mut macro_registry);

    let macro_expanded_ast = expand_macros(transformed_ast, &macro_registry)
        .map_err(|e| format!("Macro expansion error: {}", e))?;

    let mut final_ast = macro_expanded_ast;
    process_derives(&mut final_ast).map_err(|e| format!("Derive macro error: {}", e))?;
    let macro_time = macro_start.elapsed();
    if let Some(ref mut p) = profile_out {
        p.macro_ms = macro_time.as_secs_f64() * 1000.0;
    }

    if verbose {
        let macro_count = macro_registry.macros_count();
        if macro_count > 0 {
            println!("  {} {} macro(s) expanded", "Macros:".cyan(), macro_count);
        }
    }

    // Type check
    let typecheck_start = std::time::Instant::now();
    let checker = run_type_check(
        &final_ast,
        &query_db,
        input,
        &main_source,
        suggest_fixes,
        force_rebuild,
        use_parallel,
        verbose,
        &mut cache,
    )?;
    let typecheck_time = typecheck_start.elapsed();
    if let Some(ref mut p) = profile_out {
        p.typecheck_ms = typecheck_time.as_secs_f64() * 1000.0;
    }

    // Print ownership warnings if any
    let ownership_warnings: Vec<_> = checker
        .get_warnings()
        .iter()
        .filter(|w| w.starts_with("[ownership]"))
        .collect();
    if !ownership_warnings.is_empty() {
        for w in &ownership_warnings {
            eprintln!("{} {}", "warning:".yellow().bold(), w);
        }
    }

    if verbose {
        println!("  {}", "Type check passed".green());
        println!(
            "  {} Type check time: {:.3}s",
            "⏱".cyan(),
            typecheck_time.as_secs_f64()
        );
    }

    // MIR borrow checking (when --strict-borrow is enabled)
    if crate::get_strict_borrow() {
        run_mir_borrow_check(&final_ast, verbose)?;
    }

    // Per-module codegen path
    // VAIS_SINGLE_MODULE=1 forces single-module codegen (deprecated — per-module now supports generics)
    let force_single = std::env::var("VAIS_SINGLE_MODULE").map_or(false, |v| v == "1");
    if force_single && verbose {
        eprintln!("warning: VAIS_SINGLE_MODULE=1 is deprecated — per-module codegen now supports cross-module generics");
    }
    let use_per_module = !force_single && (per_module || final_ast.modules_map.as_ref().is_some_and(|m| m.len() > 1));
    if use_per_module {
        if let Some(ref mmap) = final_ast.modules_map {
            if mmap.len() > 1 {
                let input_canonical = input.canonicalize().unwrap_or_else(|_| input.to_path_buf());

                if emit_ir {
                    return super::parallel::run_per_module_emit_ir(
                        &final_ast,
                        &checker,
                        &target,
                        input,
                        &input_canonical,
                        &output,
                        debug,
                        opt_level,
                        verbose,
                        gc,
                        gc_threshold,
                        &main_source,
                        mmap,
                        &loaded_modules,
                        &mut cache,
                    );
                }

                return super::parallel::run_per_module_compile(
                    &final_ast,
                    &checker,
                    &target,
                    input,
                    &input_canonical,
                    output,
                    opt_level,
                    debug,
                    verbose,
                    gc,
                    gc_threshold,
                    &main_source,
                    &loaded_modules,
                    &mut cache,
                );
            }
        }
    }

    // Single-module codegen path
    let raw_ir = super::serial::generate_ir_single_module(
        &final_ast,
        &checker,
        &target,
        input,
        &main_source,
        debug,
        verbose,
        gc,
        gc_threshold,
        use_inkwell,
        use_parallel,
        &mut profile_out,
    )?;

    super::serial::optimize_and_output(
        &raw_ir,
        &final_ast,
        input,
        output,
        emit_ir,
        opt_level,
        debug,
        verbose,
        &target,
        hot,
        &lto_mode,
        &pgo_mode,
        &coverage_mode,
        plugins,
        use_parallel,
        &loaded_modules,
        &mut cache,
        cache_limit,
        &mut profile_out,
    )
}

// ============================================================================
// Helper functions extracted from cmd_build for readability
// ============================================================================

/// Check if we can skip compilation due to cache hit.
/// Returns Some(Ok(())) if skip is valid, None if compilation is needed.
fn check_cache_skip(
    c: &mut incremental::IncrementalCache,
    input: &Path,
    output: &Option<PathBuf>,
    emit_ir: bool,
    verbose: bool,
    target: &TargetTriple,
) -> Result<Option<Result<(), String>>, String> {
    match c.detect_changes_with_stats(input) {
        Ok((dirty_set, incr_stats)) => {
            if dirty_set.is_empty() {
                if verbose {
                    println!(
                        "{} {} (no changes detected)",
                        "Skipping".cyan().bold(),
                        input.display()
                    );
                    println!(
                        "  Cache hit rate: {:.1}% ({}/{} files)",
                        incr_stats.hit_rate(),
                        incr_stats.cache_hits,
                        incr_stats.files_checked
                    );
                    if incr_stats.signature_hits > 0 {
                        println!(
                            "  Signature hits: {} (dependents skipped)",
                            incr_stats.signature_hits
                        );
                    }
                    println!("  Cache check: {}ms", incr_stats.total_check_time_ms);
                }
                // Still need to output the binary path if not emit_ir
                if !emit_ir {
                    let default_ext = match target {
                        TargetTriple::Wasm32Unknown
                        | TargetTriple::WasiPreview1
                        | TargetTriple::WasiPreview2 => "wasm",
                        _ => "",
                    };
                    let bin_path = output
                        .clone()
                        .unwrap_or_else(|| input.with_extension(default_ext));
                    if bin_path.exists() {
                        if !verbose {
                            println!("{}", bin_path.display());
                        }
                        return Ok(Some(Ok(())));
                    }
                } else {
                    let ir_path = output.clone().unwrap_or_else(|| input.with_extension("ll"));
                    if ir_path.exists() {
                        if !verbose {
                            println!("{}", ir_path.display());
                        }
                        return Ok(Some(Ok(())));
                    }
                }
            } else if verbose {
                println!(
                    "{} {} file(s) changed",
                    "Rebuilding".yellow().bold(),
                    dirty_set.count()
                );
                println!(
                    "  Cache: {}/{} hits ({:.1}%), {} misses, check {}ms",
                    incr_stats.cache_hits,
                    incr_stats.files_checked,
                    incr_stats.hit_rate(),
                    incr_stats.cache_misses,
                    incr_stats.total_check_time_ms
                );
                // Show miss reasons for verbose debugging
                for (file, reasons) in &incr_stats.miss_reasons {
                    let reason_strs: Vec<&str> = reasons
                        .iter()
                        .map(|r| match r {
                            incremental::CacheMissReason::NewFile => "new",
                            incremental::CacheMissReason::ContentHashChanged => "content changed",
                            incremental::CacheMissReason::SignatureChanged => "signature changed",
                            incremental::CacheMissReason::DependencyChanged(_) => {
                                "dependency changed"
                            }
                            incremental::CacheMissReason::OptionsChanged => "options changed",
                            incremental::CacheMissReason::FileDeleted => "deleted",
                            incremental::CacheMissReason::CacheCorrupted => "cache corrupted",
                        })
                        .collect();
                    let file_name = file.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                    println!("    {} — {}", file_name, reason_strs.join(", "));
                }
            }
        }
        Err(e) => {
            if verbose {
                println!("{} Cache check failed: {}", "Warning".yellow(), e);
            }
        }
    }
    Ok(None)
}

/// Run type checking (parallel or sequential).
#[allow(clippy::too_many_arguments)]
fn run_type_check(
    final_ast: &vais_ast::Module,
    query_db: &QueryDatabase,
    input: &Path,
    main_source: &str,
    suggest_fixes: bool,
    force_rebuild: bool,
    use_parallel: bool,
    verbose: bool,
    cache: &mut Option<incremental::IncrementalCache>,
) -> Result<TypeChecker, String> {
    let mut tc_skipped = false;

    // Check if we can skip type checking based on cached signatures
    if !force_rebuild {
        if let Some(ref c) = cache {
            let tc_files: Vec<PathBuf> = final_ast
                .modules_map
                .as_ref()
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_else(|| vec![input.to_path_buf()]);
            if incremental::can_skip_type_checking(c, &tc_files) {
                tc_skipped = true;
                if verbose {
                    println!(
                        "  {} Type check skipped (signatures unchanged)",
                        "⚡".cyan()
                    );
                }
            }
        }
    }

    let mut checker = TypeChecker::new();
    configure_type_checker(&mut checker);

    if !tc_skipped {
        // Calculate imported item count so ownership checker can skip imported items
        let input_canonical = input.canonicalize().unwrap_or_else(|_| input.to_path_buf());
        if let Ok(original_ast) = query_db.parse(&input_canonical) {
            let original_non_use_count = original_ast
                .items
                .iter()
                .filter(|item| !matches!(item.node, Item::Use(_)))
                .count();
            let imported_count = final_ast.items.len().saturating_sub(original_non_use_count);
            if imported_count > 0 {
                checker.set_imported_item_count(imported_count);
            }
        }

        checker.multi_error_mode = true;

        const MIN_MODULES_FOR_PARALLEL_TC: usize = 4;

        let tc_result = if use_parallel
            && final_ast
                .modules_map
                .as_ref()
                .is_some_and(|m| m.len() >= MIN_MODULES_FOR_PARALLEL_TC)
        {
            let (final_checker, all_errors) =
                super::parallel::run_parallel_type_check(final_ast, verbose);
            checker = final_checker;

            if !all_errors.is_empty() {
                // In multi_error_mode (VAIS_TC_NONFATAL), add parallel TC errors
                // to collected_errors as non-fatal instead of failing immediately.
                // This allows codegen to proceed with partially type-checked code.
                let tc_nonfatal = std::env::var("VAIS_TC_NONFATAL")
                    .map_or(false, |v| v == "1");
                if tc_nonfatal {
                    for err_msg in &all_errors {
                        checker.collected_errors.push(
                            vais_types::TypeError::InferFailed {
                                kind: "module".to_string(),
                                name: "parallel_tc".to_string(),
                                context: err_msg.clone(),
                                span: None,
                                suggestion: None,
                            },
                        );
                    }
                    Ok(())
                } else {
                    Err(vais_types::TypeError::InferFailed {
                        kind: "module".to_string(),
                        name: "parallel_tc".to_string(),
                        context: all_errors.join("\n---\n"),
                        span: None,
                        suggestion: None,
                    })
                }
            } else {
                Ok(())
            }
        } else {
            checker.check_module(final_ast)
        };

        // Handle type checking result
        if let Err(e) = tc_result {
            let tc_nonfatal = std::env::var("VAIS_TC_NONFATAL")
                .map_or(false, |v| v == "1");

            if suggest_fixes {
                print_suggested_fixes(&e, main_source);
            }
            let total_errors = 1 + checker.get_collected_errors().len();
            for collected_err in checker.get_collected_errors() {
                eprintln!(
                    "{}",
                    error_formatter::format_type_error(collected_err, main_source, input)
                );
            }

            if tc_nonfatal {
                // Non-fatal: print error summary as warning and continue to codegen
                eprintln!(
                    "{}: {} type error(s) found (non-fatal, continuing to codegen)",
                    "warning".yellow().bold(),
                    total_errors
                );
            } else {
                if let Some(ref mut c) = cache {
                    incremental::update_tc_cache(c, final_ast, false);
                }
                if total_errors > 1 {
                    eprintln!("{}: {} errors found", "error".red().bold(), total_errors);
                }
                return Err(error_formatter::format_type_error(&e, main_source, input));
            }
        }

        // Even if check_module succeeded, there may be collected errors
        if !checker.get_collected_errors().is_empty() {
            let total_errors = checker.get_collected_errors().len();
            let tc_nonfatal = std::env::var("VAIS_TC_NONFATAL")
                .map_or(false, |v| v == "1");

            for collected_err in checker.get_collected_errors() {
                eprintln!(
                    "{}",
                    error_formatter::format_type_error(collected_err, main_source, input)
                );
            }

            if tc_nonfatal {
                // Non-fatal mode: print errors as warnings and continue to codegen
                eprintln!(
                    "{}: {} type error(s) found (non-fatal, continuing to codegen)",
                    "warning".yellow().bold(),
                    total_errors
                );
            } else {
                if let Some(ref mut c) = cache {
                    incremental::update_tc_cache(c, final_ast, false);
                }
                return Err(format!("{} type error(s) found", total_errors));
            }
        }

        // Update cache: TC passed
        if let Some(ref mut c) = cache {
            incremental::update_tc_cache(c, final_ast, true);
        }
    }

    Ok(checker)
}

/// Run MIR borrow checking if --strict-borrow is enabled.
fn run_mir_borrow_check(final_ast: &vais_ast::Module, verbose: bool) -> Result<(), String> {
    let borrow_start = std::time::Instant::now();
    if verbose {
        println!("  {} Running MIR borrow checker...", "🔍".cyan());
    }

    let mir_module = vais_mir::lower::lower_module(final_ast);
    let borrow_errors = vais_mir::borrow_check::check_module(&mir_module);

    if !borrow_errors.is_empty() {
        for error in &borrow_errors {
            eprintln!("{}: {}", "error".red().bold(), error);
        }
        return Err(format!(
            "Borrow checking failed: {} error(s) detected",
            borrow_errors.len()
        ));
    }

    if verbose {
        let borrow_time = borrow_start.elapsed();
        println!("  {}", "Borrow check passed".green());
        println!(
            "  {} Borrow check time: {:.3}s",
            "⏱".cyan(),
            borrow_time.as_secs_f64()
        );
    }
    Ok(())
}

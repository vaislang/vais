//! Core build command implementation.

use super::*;
use crate::commands::compile::{compile_ir_to_binary, compile_per_module};
use crate::configure_type_checker;
use crate::error_formatter;
use crate::imports::{load_module_with_imports_internal, load_module_with_imports_parallel};
use crate::incremental;
use crate::package;
use crate::runtime::extract_used_modules;
use crate::utils::{print_plugin_diagnostics, print_suggested_fixes};
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use vais_ast::Item;
use vais_codegen::optimize::{optimize_ir_with_pgo, OptLevel};
use vais_codegen::TargetTriple;
use vais_macro::{collect_macros, expand_macros, process_derives, MacroRegistry};
use vais_plugin::{DiagnosticLevel, PluginRegistry};
use vais_query::QueryDatabase;
use vais_types::TypeChecker;

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
    );
    let elapsed = start.elapsed();

    if time {
        println!(
            "\n{} Total compilation time: {:.3}s",
            "⏱".cyan().bold(),
            elapsed.as_secs_f64()
        );
    }

    result
}

#[allow(clippy::too_many_arguments)]
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
                                return Ok(());
                            }
                        } else {
                            let ir_path =
                                output.clone().unwrap_or_else(|| input.with_extension("ll"));
                            if ir_path.exists() {
                                if !verbose {
                                    println!("{}", ir_path.display());
                                }
                                return Ok(());
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
                                    incremental::CacheMissReason::ContentHashChanged => {
                                        "content changed"
                                    }
                                    incremental::CacheMissReason::SignatureChanged => {
                                        "signature changed"
                                    }
                                    incremental::CacheMissReason::DependencyChanged(_) => {
                                        "dependency changed"
                                    }
                                    incremental::CacheMissReason::OptionsChanged => {
                                        "options changed"
                                    }
                                    incremental::CacheMissReason::FileDeleted => "deleted",
                                    incremental::CacheMissReason::CacheCorrupted => {
                                        "cache corrupted"
                                    }
                                })
                                .collect();
                            let file_name =
                                file.file_name().and_then(|n| n.to_str()).unwrap_or("?");
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
    // 1. Collect macro definitions from the AST
    let mut macro_registry = MacroRegistry::new();
    collect_macros(&transformed_ast, &mut macro_registry);

    // 2. Expand all macro invocations
    let macro_expanded_ast = expand_macros(transformed_ast, &macro_registry)
        .map_err(|e| format!("Macro expansion error: {}", e))?;

    // 3. Process #[derive(...)] attributes
    let mut final_ast = macro_expanded_ast;
    process_derives(&mut final_ast).map_err(|e| format!("Derive macro error: {}", e))?;

    if verbose {
        let macro_count = macro_registry.macros_count();
        if macro_count > 0 {
            println!("  {} {} macro(s) expanded", "Macros:".cyan(), macro_count);
        }
    }

    // Type check (with incremental skip if signatures unchanged)
    let typecheck_start = std::time::Instant::now();
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

        // Enable multi-error collection to report all type errors at once
        checker.multi_error_mode = true;

        // Parallel type checking if enabled and multi-module
        // Note: TypeChecker contains Cell/RefCell (not Sync), so we can't share it via Arc.
        // Instead, each thread creates its own TypeChecker and checks independent modules.
        // Type definitions are merged after parallel execution.
        // Parallel TC threshold: only worthwhile with enough modules to offset rayon overhead
        const MIN_MODULES_FOR_PARALLEL_TC: usize = 4;

        let tc_result = if use_parallel
            && final_ast
                .modules_map
                .as_ref()
                .is_some_and(|m| m.len() >= MIN_MODULES_FOR_PARALLEL_TC)
        {
            use rayon::prelude::*;

            let modules_map = final_ast
                .modules_map
                .as_ref()
                .expect("BUG: modules_map should be Some (checked above)");
            let module_paths: Vec<PathBuf> = modules_map.keys().cloned().collect();

            if verbose {
                println!(
                    "  {} Parallel type check ({} modules)",
                    "⚡".cyan(),
                    module_paths.len()
                );
            }

            // Parallel type check each module independently
            let results: Vec<_> = module_paths
                .par_iter()
                .map(|module_path| {
                    let mut module_checker = TypeChecker::new();
                    configure_type_checker(&mut module_checker);
                    module_checker.multi_error_mode = true;

                    // Get items for this module
                    if let Some(item_indices) = modules_map.get(module_path) {
                        // Construct module with only this module's items (avoid full AST clone)
                        let module_items: Vec<_> = item_indices
                            .iter()
                            .filter_map(|&idx| final_ast.items.get(idx).cloned())
                            .collect();
                        let module_ast = vais_ast::Module {
                            items: module_items,
                            modules_map: None,
                        };

                        // Type check this module
                        match module_checker.check_module(&module_ast) {
                            Ok(_) => Ok(module_checker),
                            Err(e) => Err((format!("{}", e), module_checker)),
                        }
                    } else {
                        Ok(module_checker)
                    }
                })
                .collect();

            // Merge results from all modules
            let mut final_checker = TypeChecker::new();
            configure_type_checker(&mut final_checker);
            final_checker.multi_error_mode = true;

            let mut all_errors: Vec<String> = Vec::new();
            for result in results {
                match result {
                    Ok(module_checker) => {
                        // Merge type definitions from this module
                        final_checker.merge_type_defs_from(module_checker);
                    }
                    Err((err_msg, module_checker)) => {
                        all_errors.push(err_msg);
                        // Still merge type definitions even if error occurred
                        final_checker.merge_type_defs_from(module_checker);
                    }
                }
            }

            // Update checker with merged results
            checker = final_checker;

            // Return combined errors if any occurred
            if !all_errors.is_empty() {
                Err(vais_types::TypeError::InferFailed {
                    kind: "module".to_string(),
                    name: "parallel_tc".to_string(),
                    context: all_errors.join("\n---\n"),
                    span: None,
                    suggestion: None,
                })
            } else {
                Ok(())
            }
        } else {
            // Sequential type checking
            checker.check_module(&final_ast)
        };

        // Handle type checking result
        if let Err(e) = tc_result {
            // If suggest_fixes is enabled, print suggested fixes
            if suggest_fixes {
                print_suggested_fixes(&e, &main_source);
            }
            // Also print any additional collected errors
            for collected_err in checker.get_collected_errors() {
                eprintln!(
                    "{}",
                    error_formatter::format_type_error(collected_err, &main_source, input)
                );
            }
            // Update cache: TC failed
            if let Some(ref mut c) = cache {
                incremental::update_tc_cache(c, &final_ast, false);
            }
            let total_errors = 1 + checker.get_collected_errors().len();
            if total_errors > 1 {
                eprintln!("{}: {} errors found", "error".red().bold(), total_errors);
            }
            // Format error with source context
            return Err(error_formatter::format_type_error(&e, &main_source, input));
        }

        // Even if check_module succeeded, there may be collected errors
        if !checker.get_collected_errors().is_empty() {
            for collected_err in checker.get_collected_errors() {
                eprintln!(
                    "{}",
                    error_formatter::format_type_error(collected_err, &main_source, input)
                );
            }
            // Update cache: TC failed
            if let Some(ref mut c) = cache {
                incremental::update_tc_cache(c, &final_ast, false);
            }
            let total_errors = checker.get_collected_errors().len();
            return Err(format!("{} type error(s) found", total_errors));
        }

        // Update cache: TC passed
        if let Some(ref mut c) = cache {
            incremental::update_tc_cache(c, &final_ast, true);
        }
    }
    let typecheck_time = typecheck_start.elapsed();

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
        let borrow_start = std::time::Instant::now();
        if verbose {
            println!("  {} Running MIR borrow checker...", "🔍".cyan());
        }

        let mir_module = vais_mir::lower::lower_module(&final_ast);
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
    }

    // Per-module codegen path: split AST by source module, generate per-module .ll → .o → link
    // Auto-enable per-module for multi-file projects (opt-in flag OR auto-detect)
    let use_per_module = per_module || final_ast.modules_map.as_ref().is_some_and(|m| m.len() > 1);
    if use_per_module {
        if let Some(ref mmap) = final_ast.modules_map {
            if mmap.len() > 1 {
                let input_canonical = input.canonicalize().unwrap_or_else(|_| input.to_path_buf());

                // Special path for --emit-ir with per-module: just generate .ll files, no .o compilation
                if emit_ir {
                    use rayon::prelude::*;

                    let output_dir = output
                        .as_ref()
                        .and_then(|p| p.parent())
                        .unwrap_or_else(|| input.parent().unwrap_or(Path::new(".")));
                    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("main");

                    let effective_opt_level = if debug { 0 } else { opt_level };
                    let resolved_functions = checker.get_all_functions().clone();

                    let codegen_start = std::time::Instant::now();

                    // Generate IR for each module (parallel with rayon)
                    let module_entries: Vec<_> = mmap.iter().collect();
                    let ir_results: Vec<Result<(String, String), String>> = module_entries
                        .par_iter()
                        .map(|(module_path, item_indices)| {
                            let module_stem = module_path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let is_main = **module_path == input_canonical;

                            // Create a fresh CodeGenerator for this module
                            let mut codegen =
                                vais_codegen::CodeGenerator::new_with_target(&module_stem, target.clone());
                            codegen.set_resolved_functions(resolved_functions.clone());
                            codegen.set_string_prefix(&module_stem);

                            if gc {
                                codegen.enable_gc();
                                if let Some(threshold) = gc_threshold {
                                    codegen.set_gc_threshold(threshold);
                                }
                            }

                            if debug && is_main {
                                let source_file = module_path
                                    .file_name()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("unknown.vais");
                                let source_dir =
                                    module_path.parent().and_then(|p| p.to_str()).unwrap_or(".");
                                codegen.enable_debug(source_file, source_dir, &main_source);
                            }

                            // Generate IR for this module's subset
                            let raw_ir = codegen
                                .generate_module_subset(&final_ast, item_indices, is_main)
                                .map_err(|e| format!("Codegen error for {}: {}", module_stem, e))?;

                            // Apply optimizations
                            let opt = match effective_opt_level {
                                0 => vais_codegen::optimize::OptLevel::O0,
                                1 => vais_codegen::optimize::OptLevel::O1,
                                2 => vais_codegen::optimize::OptLevel::O2,
                                _ => vais_codegen::optimize::OptLevel::O3,
                            };
                            let ir = vais_codegen::optimize::optimize_ir(&raw_ir, opt);

                            Ok((module_stem, ir))
                        })
                        .collect();

                    // Write each module's IR to a separate .ll file
                    for result in ir_results {
                        let (module_stem, ir) = result?;
                        let ll_path = output_dir.join(format!("{}_{}.ll", stem, module_stem));
                        fs::write(&ll_path, &ir)
                            .map_err(|e| format!("Cannot write '{}': {}", ll_path.display(), e))?;
                        println!("{} {}", "Wrote".green().bold(), ll_path.display());
                    }

                    if verbose {
                        println!(
                            "  {} IR generation: {:.3}s",
                            "⏱".cyan(),
                            codegen_start.elapsed().as_secs_f64()
                        );
                    }

                    // Update incremental cache
                    if let Some(ref mut c) = cache {
                        for loaded_path in &loaded_modules {
                            let _ = c.update_file(loaded_path);
                        }
                        let _ = c.persist();
                    }

                    return Ok(());
                }

                // Normal per-module path: compile to .o and link
                let default_ext = match target {
                    TargetTriple::Wasm32Unknown
                    | TargetTriple::WasiPreview1
                    | TargetTriple::WasiPreview2 => "wasm",
                    _ => "",
                };
                let bin_path = output.unwrap_or_else(|| input.with_extension(default_ext));

                compile_per_module(
                    &final_ast,
                    &checker,
                    &target,
                    &input_canonical,
                    &bin_path,
                    opt_level,
                    debug,
                    verbose,
                    gc,
                    gc_threshold,
                    input,
                    &main_source,
                    cache.as_ref().map(|c| c.cache_dir()),
                )?;

                // Update incremental cache
                if let Some(ref mut c) = cache {
                    for loaded_path in &loaded_modules {
                        let _ = c.update_file(loaded_path);
                    }
                    let _ = c.persist();
                }

                return Ok(());
            }
        }
    }

    // Generate LLVM IR
    let module_name = input.file_stem().and_then(|s| s.to_str()).unwrap_or("main");

    if verbose && !matches!(target, TargetTriple::Native) {
        println!("  {} {}", "Target:".cyan(), target.triple_str());
    }

    // Inkwell backend path (opt-in via --inkwell flag)
    #[cfg(feature = "inkwell")]
    let raw_ir = if use_inkwell {
        // Warn about unsupported features in inkwell backend
        if gc {
            eprintln!(
                "{}: --gc is not yet supported with the inkwell backend, ignoring",
                "warning".yellow().bold()
            );
        }
        if debug {
            eprintln!(
                "{}: -g/--debug is not yet supported with the inkwell backend, ignoring",
                "warning".yellow().bold()
            );
        }

        if verbose {
            println!("  {} inkwell (LLVM API)", "Backend:".cyan());
        }

        let codegen_start = std::time::Instant::now();
        let context = ::inkwell::context::Context::create();
        let mut gen = vais_codegen::InkwellCodeGenerator::new_with_target(
            &context,
            module_name,
            target.clone(),
        );
        gen.set_resolved_functions(checker.get_all_functions().clone());
        let instantiations = checker.get_generic_instantiations();
        if instantiations.is_empty() {
            gen.generate_module(&final_ast)
                .map_err(|e| format!("Inkwell codegen error: {}", e))?;
        } else {
            gen.generate_module_with_instantiations(&final_ast, instantiations)
                .map_err(|e| format!("Inkwell codegen error: {}", e))?;
        }
        let ir = gen.get_ir_string();
        let codegen_time = codegen_start.elapsed();

        if verbose {
            println!(
                "  {} Codegen time: {:.3}s",
                "⏱".cyan(),
                codegen_time.as_secs_f64()
            );
        }

        ir
    } else {
        generate_with_text_backend(
            module_name,
            &target,
            gc,
            gc_threshold,
            debug,
            input,
            &main_source,
            &checker,
            &final_ast,
            verbose,
        )?
    };

    #[cfg(not(feature = "inkwell"))]
    let raw_ir = {
        if use_inkwell {
            return Err(
                "Inkwell backend not available. Recompile with: cargo build --features inkwell"
                    .to_string(),
            );
        }
        generate_with_text_backend(
            module_name,
            &target,
            gc,
            gc_threshold,
            debug,
            input,
            &main_source,
            &checker,
            &final_ast,
            verbose,
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
        optimize_ir_with_pgo(&raw_ir, opt, &pgo_mode)
    };

    // Run plugin optimizations
    let plugin_opt = match effective_opt_level {
        0 => vais_plugin::OptLevel::O0,
        1 => vais_plugin::OptLevel::O1,
        2 => vais_plugin::OptLevel::O2,
        _ => vais_plugin::OptLevel::O3,
    };
    let ir = if !plugins.is_empty() {
        plugins
            .run_optimize(&ir, plugin_opt)
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
        println!(
            "{} Optimizations disabled for debug build",
            "Note".yellow().bold()
        );
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
    fs::write(&ir_path, &ir).map_err(|e| format!("Cannot write '{}': {}", ir_path.display(), e))?;

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
        // Load native dependencies from vais.toml if present
        let native_deps = {
            let input_dir = input.parent().unwrap_or(Path::new("."));
            if let Some(pkg_dir) = package::find_manifest(input_dir) {
                match package::load_manifest(&pkg_dir) {
                    Ok(m) => m.native_dependencies,
                    Err(_) => HashMap::new(),
                }
            } else {
                HashMap::new()
            }
        };

        // Extract used modules from AST for smart C runtime linking
        let used_modules = extract_used_modules(&final_ast);
        if verbose && !used_modules.is_empty() {
            let std_modules: Vec<_> = used_modules
                .iter()
                .filter(|m| m.starts_with("std::"))
                .map(|m| m.strip_prefix("std::").unwrap_or(m))
                .collect();
            if !std_modules.is_empty() {
                println!(
                    "{} Detected std modules: {}",
                    "info:".blue().bold(),
                    std_modules.join(", ")
                );
            }
        }

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
                TargetTriple::Wasm32Unknown
                | TargetTriple::WasiPreview1
                | TargetTriple::WasiPreview2 => "wasm",
                _ => "",
            }
        };

        let bin_path = output.unwrap_or_else(|| {
            if hot {
                // For hot reload, prefix with 'lib' and use dylib extension
                let parent = input.parent().unwrap_or(Path::new("."));
                let stem = input
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
                parent.join(format!("lib{}.{}", stem, default_ext))
            } else {
                input.with_extension(default_ext)
            }
        });

        compile_ir_to_binary(
            &ir_path,
            &bin_path,
            effective_opt_level,
            debug,
            verbose,
            &target,
            hot,
            &lto_mode,
            &pgo_mode,
            &coverage_mode,
            &used_modules,
            &native_deps,
            cache.as_ref().map(|c| c.cache_dir()),
        )?;
    }

    // Update incremental compilation cache after successful build
    if let Some(ref mut c) = cache {
        // Update file metadata for all loaded modules
        for loaded_path in &loaded_modules {
            if let Err(e) = c.update_file(loaded_path) {
                if verbose {
                    eprintln!(
                        "{}: Cache update for '{}': {}",
                        "Warning".yellow(),
                        loaded_path.display(),
                        e
                    );
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
            println!(
                "{} {} files, {} dependencies",
                "Cache updated:".cyan(),
                stats.total_files,
                stats.total_dependencies
            );
        }

        // Clean up cache to stay under size limit
        match c.cleanup_cache(cache_limit) {
            Ok(deleted_count) => {
                if verbose && deleted_count > 0 {
                    println!(
                        "{} {} old cache file(s) to stay under {} bytes",
                        "Cache cleanup:".cyan(),
                        deleted_count,
                        cache_limit
                    );
                }
            }
            Err(e) => {
                if verbose {
                    eprintln!("{}: Cache cleanup failed: {}", "Warning".yellow(), e);
                }
            }
        }
    }

    Ok(())
}

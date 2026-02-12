//! Compilation functions for different targets.

use crate::incremental;
use crate::package;
use crate::runtime::{
    find_gc_library, find_http_runtime, find_runtime_file, find_sync_runtime, find_thread_runtime,
    get_runtime_for_module,
};
use colored::Colorize;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use vais_codegen::TargetTriple;
use vais_types::TypeChecker;

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_per_module(
    final_ast: &vais_ast::Module,
    checker: &TypeChecker,
    target: &TargetTriple,
    input_canonical: &Path,
    bin_path: &Path,
    opt_level: u8,
    debug: bool,
    verbose: bool,
    gc: bool,
    gc_threshold: Option<usize>,
    input: &Path,
    main_source: &str,
    obj_cache_dir: Option<&Path>,
) -> Result<(), String> {
    use rayon::prelude::*;
    use vais_codegen::CodeGenerator;

    let modules_map = final_ast
        .modules_map
        .as_ref()
        .ok_or_else(|| "Per-module codegen requires modules_map".to_string())?;

    if verbose {
        println!(
            "{} Per-module codegen: {} modules",
            "⚡".cyan().bold(),
            modules_map.len()
        );
    }

    let codegen_start = std::time::Instant::now();

    // Determine cache directory for intermediate .ll and .o files
    let cache_dir = if let Some(dir) = obj_cache_dir {
        dir.to_path_buf()
    } else {
        incremental::get_cache_dir(input).join("modules")
    };
    fs::create_dir_all(&cache_dir).map_err(|e| format!("Cannot create module cache dir: {}", e))?;

    let effective_opt_level = if debug { 0 } else { opt_level };
    let resolved_functions = checker.get_all_functions().clone();
    let _instantiations = checker.get_generic_instantiations();

    // Phase 1: Generate IR for all modules (parallelized with rayon)
    // Collect (module_stem, is_main, ir_string) tuples
    let module_entries: Vec<_> = modules_map.iter().collect();
    let ir_results: Vec<Result<(String, bool, String), String>> = module_entries
        .par_iter()
        .map(|(module_path, item_indices)| {
            let module_stem = module_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            let is_main = *module_path == input_canonical;

            // Create a fresh CodeGenerator for this module
            let mut codegen = CodeGenerator::new_with_target(&module_stem, target.clone());
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
                let source_dir = module_path.parent().and_then(|p| p.to_str()).unwrap_or(".");
                codegen.enable_debug(source_file, source_dir, main_source);
            }

            // Generate IR for this module's subset
            let raw_ir = codegen
                .generate_module_subset(final_ast, item_indices, is_main)
                .map_err(|e| format!("Codegen error for {}: {}", module_stem, e))?;

            // Apply optimizations
            let opt = match effective_opt_level {
                0 => vais_codegen::optimize::OptLevel::O0,
                1 => vais_codegen::optimize::OptLevel::O1,
                2 => vais_codegen::optimize::OptLevel::O2,
                _ => vais_codegen::optimize::OptLevel::O3,
            };
            let ir = vais_codegen::optimize::optimize_ir(&raw_ir, opt);

            Ok((module_stem, is_main, ir))
        })
        .collect();

    // Collect results, bail on first error
    let mut module_irs: Vec<(String, bool, String)> = Vec::with_capacity(ir_results.len());
    for result in ir_results {
        module_irs.push(result?);
    }

    if verbose {
        println!(
            "  {} IR generation: {:.3}s",
            "⏱".cyan(),
            codegen_start.elapsed().as_secs_f64()
        );
    }

    // Phase 2: Compile .ll → .o with content-hash caching (parallelized)
    let compile_start = std::time::Instant::now();

    let obj_results: Vec<Result<(PathBuf, bool), String>> = module_irs
        .par_iter()
        .map(|(module_stem, _is_main, ir)| {
            // Compute content hash of the IR
            let ir_hash = incremental::compute_content_hash(ir);
            let cached_obj_path =
                incremental::get_ir_cached_object_path(&cache_dir, &ir_hash, effective_opt_level);

            // Check cache: if .o exists for this IR hash, skip clang
            if cached_obj_path.exists() {
                return Ok((cached_obj_path, true)); // true = cache hit
            }

            // Cache miss: write .ll, compile to .o
            let ll_path = cache_dir.join(format!("{}.ll", module_stem));
            fs::write(&ll_path, ir)
                .map_err(|e| format!("Cannot write '{}': {}", ll_path.display(), e))?;

            let opt_flag = format!("-O{}", effective_opt_level.min(3));
            let mut compile_args = vec![
                "-c".to_string(),
                opt_flag,
                ll_path.display().to_string(),
                "-o".to_string(),
                cached_obj_path.display().to_string(),
            ];
            if debug {
                compile_args.push("-g".to_string());
            }

            let compile_output = std::process::Command::new("clang")
                .args(&compile_args)
                .output()
                .map_err(|e| format!("Cannot run clang: {}", e))?;

            if !compile_output.status.success() {
                let stderr = String::from_utf8_lossy(&compile_output.stderr);
                return Err(format!(
                    "clang compilation failed for module '{}': {}",
                    module_stem, stderr
                ));
            }

            Ok((cached_obj_path, false)) // false = cache miss
        })
        .collect();

    // Collect .o paths
    let mut obj_files: Vec<PathBuf> = Vec::with_capacity(obj_results.len());
    let mut cache_hits = 0usize;
    for result in obj_results {
        let (path, hit) = result?;
        if hit {
            cache_hits += 1;
        }
        obj_files.push(path);
    }

    let compile_time = compile_start.elapsed();
    if verbose {
        println!(
            "  {} Compile time: {:.3}s ({} cached, {} compiled)",
            "⏱".cyan(),
            compile_time.as_secs_f64(),
            cache_hits,
            obj_files.len() - cache_hits
        );
    }

    let codegen_time = codegen_start.elapsed();
    if verbose {
        println!(
            "  {} Codegen + compile time: {:.3}s",
            "⏱".cyan(),
            codegen_time.as_secs_f64()
        );
    }

    // Link all .o files → binary
    let link_start = std::time::Instant::now();
    let opt_flag = format!("-O{}", if debug { 0 } else { opt_level }.min(3));
    let mut link_args = vec![opt_flag];
    if debug {
        link_args.push("-g".to_string());
    }
    for obj in &obj_files {
        link_args.push(obj.display().to_string());
    }
    link_args.push("-o".to_string());
    link_args.push(bin_path.display().to_string());

    // Add system libraries
    #[cfg(target_os = "macos")]
    {
        link_args.push("-lSystem".to_string());
    }
    #[cfg(target_os = "linux")]
    {
        link_args.push("-lm".to_string());
    }

    let link_status = std::process::Command::new("clang")
        .args(&link_args)
        .status()
        .map_err(|e| format!("Cannot run clang: {}", e))?;

    if !link_status.success() {
        return Err("Linking failed".to_string());
    }

    let link_time = link_start.elapsed();
    if verbose {
        println!(
            "  {} Link time: {:.3}s",
            "⏱".cyan(),
            link_time.as_secs_f64()
        );
        println!(
            "{} {} ({} modules, {} cached)",
            "Compiled".green().bold(),
            bin_path.display(),
            obj_files.len(),
            cache_hits
        );
    } else {
        println!("{}", bin_path.display());
    }

    Ok(())
}

/// Parallel type-checking of independent modules using rayon.
///
/// Uses dependency graph to determine which modules can be type-checked in parallel.
/// Each level of modules is processed concurrently, with levels executed sequentially
/// to respect dependencies.
///
/// # Arguments
///
/// * `final_ast` - The complete merged AST containing all modules
/// * `dep_graph` - Dependency graph for topological ordering
/// * `verbose` - Print detailed timing and progress information
///
/// # Returns
///
/// A fully type-checked TypeChecker instance with all module information,
/// or an error if type checking fails.
///
/// # Implementation Note
///
/// This function creates independent TypeChecker instances for each module in a level,
/// then type-checks them in parallel using rayon. Results are merged sequentially
/// after each level completes. This approach avoids lock contention while maintaining
/// correctness.
///
/// Reserved for parallel compilation mode (Phase 2 implementation).
#[allow(dead_code)]
pub fn parallel_type_check(
    final_ast: &vais_ast::Module,
    dep_graph: &incremental::DependencyGraph,
    verbose: bool,
) -> Result<TypeChecker, String> {
    use std::time::Instant;

    let start = Instant::now();

    // Get parallel levels from dependency graph
    let levels = dep_graph.parallel_levels();

    if verbose {
        println!(
            "{} Parallel type-checking: {} level(s)",
            "⚡".cyan().bold(),
            levels.len()
        );
    }

    // Build a map from file path to item indices
    let modules_map = final_ast.modules_map.as_ref().ok_or_else(|| {
        "modules_map required for parallel type-checking".to_string()
    })?;

    // Create a global checker for collecting all type information
    let global_checker = Arc::new(Mutex::new(TypeChecker::new()));

    // Process each level sequentially (levels depend on each other)
    for (level_idx, level) in levels.iter().enumerate() {
        let level_start = Instant::now();

        if verbose && !level.is_empty() {
            println!(
                "  {} Level {}: {} module(s)",
                "→".cyan(),
                level_idx + 1,
                level.len()
            );
        }

        // Type-check all modules in this level in parallel
        let results: Vec<Result<TypeChecker, String>> = level
            .par_iter()
            .map(|module_path| {
                // Get item indices for this module
                let item_indices = modules_map.get(module_path).ok_or_else(|| {
                    format!("No items found for module: {}", module_path.display())
                })?;

                // Create independent TypeChecker for this module
                let mut checker = TypeChecker::new();

                // Copy type definitions from global checker (from previous levels)
                {
                    let global = global_checker
                        .lock()
                        .map_err(|_| "Mutex poisoned".to_string())?;

                    checker.clone_type_defs_from(&global);
                }

                // Extract items for this module
                let module_items: Vec<vais_ast::Spanned<vais_ast::Item>> = item_indices
                    .iter()
                    .filter_map(|&idx| final_ast.items.get(idx).cloned())
                    .collect();

                let module = vais_ast::Module {
                    items: module_items,
                    modules_map: None,
                };

                // Type-check this module
                checker.check_module(&module).map_err(|e| format!("{}", e))?;

                Ok(checker)
            })
            .collect();

        // Merge results into global checker
        for result in results {
            let module_checker = result?;

            let mut global = global_checker
                .lock()
                .map_err(|_| "Mutex poisoned during merge".to_string())?;

            // Merge type definitions
            global.merge_type_defs_from(module_checker);
        }

        if verbose {
            println!(
                "    {} Level {} completed in {:.3}s",
                "✓".green(),
                level_idx + 1,
                level_start.elapsed().as_secs_f64()
            );
        }
    }

    if verbose {
        println!(
            "  {} Total parallel type-check time: {:.3}s",
            "⏱".cyan(),
            start.elapsed().as_secs_f64()
        );
    }

    // Extract final checker from Arc<Mutex<>>
    Arc::try_unwrap(global_checker)
        .map_err(|_| "Failed to unwrap global checker".to_string())?
        .into_inner()
        .map_err(|_| "Mutex poisoned on final unwrap".to_string())
}

/// Parallel codegen of independent modules using rayon.
///
/// Uses dependency graph to determine which modules can be compiled in parallel.
/// Each level of modules is processed concurrently, with levels executed sequentially
/// to respect dependencies.
///
/// # Arguments
///
/// * `final_ast` - The complete merged AST containing all modules
/// * `checker` - TypeChecker with all type information
/// * `dep_graph` - Dependency graph for topological ordering
/// * `target` - Target triple for code generation
/// * `opt_level` - Optimization level (0-3)
/// * `debug` - Whether to generate debug information
/// * `verbose` - Print detailed timing and progress information
/// * `gc` - Enable garbage collection
/// * `gc_threshold` - GC threshold if enabled
/// * `input_canonical` - Canonical path to main input file
/// * `main_source` - Source code of main file (for debug info)
///
/// # Returns
///
/// A vector of (module_name, IR_string) pairs for all modules.
///
/// # Implementation Note
///
/// This function creates independent CodeGenerator instances for each module in a level,
/// then generates IR in parallel using rayon. Results are collected sequentially
/// after each level completes. This approach maximizes parallelism while respecting
/// module dependencies.
///
/// Reserved for parallel compilation mode (Phase 2 implementation).
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn parallel_codegen(
    final_ast: &vais_ast::Module,
    checker: &TypeChecker,
    dep_graph: &incremental::DependencyGraph,
    target: &TargetTriple,
    opt_level: u8,
    debug: bool,
    verbose: bool,
    gc: bool,
    gc_threshold: Option<usize>,
    input_canonical: &Path,
    main_source: &str,
) -> Result<Vec<(String, String)>, String> {
    use std::time::Instant;
    use vais_codegen::CodeGenerator;

    let start = Instant::now();

    // Get parallel levels from dependency graph
    let levels = dep_graph.parallel_levels();

    if verbose {
        println!(
            "{} Parallel codegen: {} level(s)",
            "⚡".cyan().bold(),
            levels.len()
        );
    }

    // Build a map from file path to item indices
    let modules_map = final_ast.modules_map.as_ref().ok_or_else(|| {
        "modules_map required for parallel codegen".to_string()
    })?;

    let effective_opt_level = if debug { 0 } else { opt_level };
    let resolved_functions = checker.get_all_functions().clone();

    // Collect all IR results across levels
    let all_irs = Arc::new(Mutex::new(Vec::new()));

    // Process each level sequentially (levels depend on each other)
    for (level_idx, level) in levels.iter().enumerate() {
        let level_start = Instant::now();

        if verbose && !level.is_empty() {
            println!(
                "  {} Level {}: {} module(s)",
                "→".cyan(),
                level_idx + 1,
                level.len()
            );
        }

        // Generate IR for all modules in this level in parallel
        let results: Vec<Result<(String, bool, String), String>> = level
            .par_iter()
            .map(|module_path| {
                let module_stem = module_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let is_main = *module_path == input_canonical;

                // Get item indices for this module
                let item_indices = modules_map.get(module_path).ok_or_else(|| {
                    format!("No items found for module: {}", module_path.display())
                })?;

                // Create a fresh CodeGenerator for this module
                let mut codegen = CodeGenerator::new_with_target(&module_stem, target.clone());
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
                    let source_dir = module_path.parent().and_then(|p| p.to_str()).unwrap_or(".");
                    codegen.enable_debug(source_file, source_dir, main_source);
                }

                // Generate IR for this module's subset
                let raw_ir = codegen
                    .generate_module_subset(final_ast, item_indices, is_main)
                    .map_err(|e| format!("Codegen error for {}: {}", module_stem, e))?;

                // Apply optimizations
                let opt = match effective_opt_level {
                    0 => vais_codegen::optimize::OptLevel::O0,
                    1 => vais_codegen::optimize::OptLevel::O1,
                    2 => vais_codegen::optimize::OptLevel::O2,
                    _ => vais_codegen::optimize::OptLevel::O3,
                };
                let ir = vais_codegen::optimize::optimize_ir(&raw_ir, opt);

                Ok((module_stem, is_main, ir))
            })
            .collect();

        // Collect results for this level
        for result in results {
            let (module_stem, _is_main, ir) = result?;
            all_irs
                .lock()
                .map_err(|_| "Mutex poisoned".to_string())?
                .push((module_stem, ir));
        }

        if verbose {
            println!(
                "    {} Level {} completed in {:.3}s",
                "✓".green(),
                level_idx + 1,
                level_start.elapsed().as_secs_f64()
            );
        }
    }

    if verbose {
        println!(
            "  {} Total parallel codegen time: {:.3}s",
            "⏱".cyan(),
            start.elapsed().as_secs_f64()
        );
    }

    // Extract final results from Arc<Mutex<>>
    Arc::try_unwrap(all_irs)
        .map_err(|_| "Failed to unwrap IR results".to_string())?
        .into_inner()
        .map_err(|_| "Mutex poisoned on final unwrap".to_string())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_ir_to_binary(
    ir_path: &Path,
    bin_path: &Path,
    opt_level: u8,
    debug: bool,
    verbose: bool,
    target: &TargetTriple,
    hot: bool,
    lto_mode: &vais_codegen::optimize::LtoMode,
    pgo_mode: &vais_codegen::optimize::PgoMode,
    coverage_mode: &vais_codegen::optimize::CoverageMode,
    used_modules: &HashSet<String>,
    native_deps: &HashMap<String, package::NativeDependency>,
    obj_cache_dir: Option<&Path>,
) -> Result<(), String> {
    match target {
        TargetTriple::Wasm32Unknown => compile_to_wasm32(ir_path, bin_path, opt_level, verbose),
        TargetTriple::WasiPreview1 | TargetTriple::WasiPreview2 => {
            compile_to_wasi(ir_path, bin_path, opt_level, verbose)
        }
        _ => compile_to_native(
            ir_path,
            bin_path,
            opt_level,
            debug,
            verbose,
            hot,
            lto_mode,
            pgo_mode,
            coverage_mode,
            used_modules,
            native_deps,
            obj_cache_dir,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_to_native(
    ir_path: &Path,
    bin_path: &Path,
    opt_level: u8,
    debug: bool,
    verbose: bool,
    hot: bool,
    lto_mode: &vais_codegen::optimize::LtoMode,
    pgo_mode: &vais_codegen::optimize::PgoMode,
    coverage_mode: &vais_codegen::optimize::CoverageMode,
    used_modules: &HashSet<String>,
    native_deps: &HashMap<String, package::NativeDependency>,
    obj_cache_dir: Option<&Path>,
) -> Result<(), String> {
    let opt_flag = format!("-O{}", opt_level.min(3));

    // --- Incremental .o caching: skip clang -c if IR unchanged ---
    let cached_obj = if let Some(cache_dir) = obj_cache_dir {
        // Compute hash of the IR file
        let ir_hash = incremental::compute_file_hash(ir_path)?;
        let obj_path = incremental::get_ir_cached_object_path(cache_dir, &ir_hash, opt_level);
        if obj_path.exists() {
            if verbose {
                println!(
                    "{} Using cached object: {}",
                    "⚡ Cache hit".green().bold(),
                    obj_path.display()
                );
            }
            Some((obj_path, ir_hash))
        } else {
            Some((obj_path, ir_hash))
        }
    } else {
        None
    };

    // If we have a cache directory, use 2-step: compile .ll → .o (cached), then link .o → binary
    if let Some((ref obj_path, _)) = cached_obj {
        if !obj_path.exists() {
            // Compile IR → .o
            let compile_args = vec![
                "-c".to_string(),
                format!("-O{}", opt_level.min(3)),
                "-Wno-override-module".to_string(),
                "-o".to_string(),
                obj_path.to_str().unwrap_or("cached.o").to_string(),
                ir_path.to_str().unwrap_or("input.ll").to_string(),
            ];
            if debug {
                // compile_args already set, add -g before -o
            }

            let compile_status = std::process::Command::new("clang")
                .args(&compile_args)
                .status()
                .map_err(|e| format!("Failed to run clang: {}", e))?;

            if !compile_status.success() {
                return Err("clang compilation failed (IR → .o)".to_string());
            }

            if verbose {
                println!(
                    "{} Compiled object: {}",
                    "⚡ Cache miss".yellow().bold(),
                    obj_path.display()
                );
            }
        }

        // Link .o → binary (this is the fast path: just linking)
        let mut link_args: Vec<String> = vec![format!("-O{}", opt_level.min(3))];

        if debug {
            link_args.push("-g".to_string());
        }

        if hot {
            link_args.push("-shared".to_string());
            link_args.push("-fPIC".to_string());
        }

        for flag in lto_mode.clang_flags() {
            link_args.push(flag.to_string());
        }

        for flag in pgo_mode.clang_flags() {
            link_args.push(flag);
        }

        // Add coverage flags
        for flag in coverage_mode.clang_flags() {
            link_args.push(flag.to_string());
        }

        link_args.push("-o".to_string());
        link_args.push(
            bin_path
                .to_str()
                .ok_or_else(|| "Invalid UTF-8 in output path".to_string())?
                .to_string(),
        );
        link_args.push(
            obj_path
                .to_str()
                .ok_or_else(|| "Invalid UTF-8 in object path".to_string())?
                .to_string(),
        );

        // Add runtime libraries (same as non-cached path)
        add_runtime_libs(&mut link_args, verbose, used_modules, native_deps, hot)?;

        let link_status = std::process::Command::new("clang")
            .args(&link_args)
            .status()
            .map_err(|e| format!("Failed to run clang (link): {}", e))?;

        if !link_status.success() {
            return Err("clang linking failed".to_string());
        }

        println!("{}", bin_path.display());

        return Ok(());
    }

    // --- Fallback: original single-step compilation (no cache) ---
    let mut args = vec![
        opt_flag,
        "-Wno-override-module".to_string(), // Suppress warning when clang sets target triple
    ];

    // Add debug flag if requested
    if debug {
        args.push("-g".to_string()); // Generate debug symbols
    }

    // Add dylib flags if hot reload mode
    if hot {
        args.push("-shared".to_string()); // Generate shared library
        args.push("-fPIC".to_string()); // Position-independent code
    }

    // Add LTO flags
    for flag in lto_mode.clang_flags() {
        args.push(flag.to_string());
    }

    // Add PGO flags
    for flag in pgo_mode.clang_flags() {
        args.push(flag);
    }

    // Add coverage flags
    for flag in coverage_mode.clang_flags() {
        args.push(flag.to_string());
    }

    // Setup directories and validate PGO/Coverage
    setup_profiling_dirs(pgo_mode, coverage_mode, verbose)?;

    args.push("-o".to_string());
    args.push(
        bin_path
            .to_str()
            .ok_or_else(|| "Invalid UTF-8 in output path".to_string())?
            .to_string(),
    );
    args.push(
        ir_path
            .to_str()
            .ok_or_else(|| "Invalid UTF-8 in IR path".to_string())?
            .to_string(),
    );

    // Add runtime libraries and native dependencies
    add_runtime_and_native_libs(
        &mut args,
        verbose,
        used_modules,
        native_deps,
        ir_path,
    )?;

    if verbose && (lto_mode.is_enabled() || pgo_mode.is_enabled() || coverage_mode.is_enabled()) {
        let mut features = vec![];
        if lto_mode.is_enabled() {
            features.push(format!("LTO={:?}", lto_mode));
        }
        if pgo_mode.is_generate() {
            features.push("PGO=generate".to_string());
        } else if pgo_mode.is_use() {
            features.push("PGO=use".to_string());
        }
        if coverage_mode.is_enabled() {
            features.push("Coverage=enabled".to_string());
        }
        println!(
            "{} Compiling with: {}",
            "info:".blue().bold(),
            features.join(", ")
        );
    }

    let status = Command::new("clang").args(&args).status();

    match status {
        Ok(s) if s.success() => {
            print_compilation_success(bin_path, debug, verbose, coverage_mode);
            Ok(())
        }
        Ok(s) => Err(format!("clang exited with code {}", s.code().unwrap_or(-1))),
        Err(_) => Err(
            "clang not found. Install LLVM/clang or use --emit-ir to output LLVM IR only."
                .to_string(),
        ),
    }
}

/// Print compilation success message and coverage instructions if applicable.
fn print_compilation_success(
    bin_path: &Path,
    debug: bool,
    verbose: bool,
    coverage_mode: &vais_codegen::optimize::CoverageMode,
) {
    if verbose {
        if debug {
            println!(
                "{} {} (with debug symbols)",
                "Compiled".green().bold(),
                bin_path.display()
            );
        } else {
            println!("{} {}", "Compiled".green().bold(), bin_path.display());
        }
    } else {
        println!("{}", bin_path.display());
    }

    // Print coverage usage instructions
    if let Some(dir) = coverage_mode.coverage_dir() {
        println!();
        println!(
            "{} Coverage instrumentation enabled.",
            "Coverage:".cyan().bold()
        );
        println!("  Run the binary to generate profile data:");
        println!(
            "    LLVM_PROFILE_FILE=\"{}/default_%m.profraw\" {}",
            dir,
            bin_path.display()
        );
        println!("  Then generate a report:");
        println!(
            "    llvm-profdata merge -output={}/coverage.profdata {}/*.profraw",
            dir, dir
        );
        println!(
            "    llvm-cov show {} -instr-profile={}/coverage.profdata",
            bin_path.display(),
            dir
        );
        println!(
            "    llvm-cov export {} -instr-profile={}/coverage.profdata -format=lcov > {}/lcov.info",
            bin_path.display(),
            dir,
            dir
        );
    }
}

/// Setup profiling directories and validate PGO profile files.
fn setup_profiling_dirs(
    pgo_mode: &vais_codegen::optimize::PgoMode,
    coverage_mode: &vais_codegen::optimize::CoverageMode,
    verbose: bool,
) -> Result<(), String> {
    // Create coverage directory if coverage is enabled
    if let Some(dir) = coverage_mode.coverage_dir() {
        let coverage_path = Path::new(dir);
        if !coverage_path.exists() {
            std::fs::create_dir_all(coverage_path)
                .map_err(|e| format!("Failed to create coverage directory '{}': {}", dir, e))?;
        }
        if verbose {
            println!(
                "{} Coverage enabled: run binary with LLVM_PROFILE_FILE=\"{}/default_%m.profraw\"",
                "info:".blue().bold(),
                dir,
            );
        }
    }

    // Create profile directory if using profile-generate
    if let Some(dir) = pgo_mode.profile_dir() {
        let profile_path = Path::new(dir);
        if !profile_path.exists() {
            std::fs::create_dir_all(profile_path)
                .map_err(|e| format!("Failed to create profile directory '{}': {}", dir, e))?;
        }
        if verbose {
            println!(
                "{} Profile data will be written to: {}/",
                "info:".blue().bold(),
                dir
            );
        }
    }

    // Show PGO info and validate profile file exists
    if let Some(path) = pgo_mode.profile_file() {
        if !Path::new(path).exists() {
            return Err(format!(
                "Profile data file not found: '{}'. Run the instrumented binary first.",
                path
            ));
        }
        if verbose {
            println!(
                "{} Using profile data from: {}",
                "info:".blue().bold(),
                path
            );
        }
    }

    Ok(())
}

/// Helper to add runtime libraries and native dependencies to clang link arguments (non-cached path).
/// This is the full version used by the direct compilation path.
fn add_runtime_and_native_libs(
    args: &mut Vec<String>,
    verbose: bool,
    used_modules: &HashSet<String>,
    native_deps: &HashMap<String, package::NativeDependency>,
    ir_path: &Path,
) -> Result<(), String> {
    // Link math library (required on Linux for sqrt, sin, cos, etc.)
    #[cfg(target_os = "linux")]
    args.push("-lm".to_string());

    // Link against libvais_gc if available
    if let Some(gc_lib_path) = find_gc_library() {
        let static_lib = gc_lib_path.join("libvais_gc.a");
        args.push(static_lib.to_str().unwrap_or("libvais_gc.a").to_string());
        if verbose {
            println!(
                "{} Linking GC runtime from: {}",
                "info:".blue().bold(),
                static_lib.display()
            );
        }
    }

    // Link C runtimes based on used modules
    let mut needs_pthread = false;
    let mut linked_libs: HashSet<&str> = HashSet::new();
    let mut linked_runtimes: Vec<String> = Vec::new();

    for module in used_modules {
        if let Some(runtime_info) = get_runtime_for_module(module) {
            if let Some(rt_path) = find_runtime_file(runtime_info.file) {
                let rt_str = rt_path.to_str().unwrap_or(runtime_info.file).to_string();
                if !linked_runtimes.contains(&rt_str) {
                    linked_runtimes.push(rt_str.clone());
                    args.push(rt_str);
                    if verbose {
                        println!(
                            "{} Linking {} runtime from: {}",
                            "info:".blue().bold(),
                            module.strip_prefix("std::").unwrap_or(module),
                            rt_path.display()
                        );
                    }
                }
            }
            if runtime_info.needs_pthread {
                needs_pthread = true;
            }
            for lib in runtime_info.libs {
                if !linked_libs.contains(lib) {
                    linked_libs.insert(lib);
                    args.push(lib.to_string());
                }
            }
        }
    }

    // Legacy fallbacks
    if linked_runtimes.is_empty() {
        if let Some(http_rt_path) = find_http_runtime() {
            args.push(
                http_rt_path
                    .to_str()
                    .unwrap_or("http_runtime.c")
                    .to_string(),
            );
            if verbose {
                println!(
                    "{} Linking HTTP runtime from: {} (legacy fallback)",
                    "info:".blue().bold(),
                    http_rt_path.display()
                );
            }
        }
        if let Some(thread_rt_path) = find_thread_runtime() {
            args.push(
                thread_rt_path
                    .to_str()
                    .unwrap_or("thread_runtime.c")
                    .to_string(),
            );
            needs_pthread = true;
            if verbose {
                println!(
                    "{} Linking thread runtime from: {} (legacy fallback)",
                    "info:".blue().bold(),
                    thread_rt_path.display()
                );
            }
        }
        if let Some(sync_rt_path) = find_sync_runtime() {
            args.push(
                sync_rt_path
                    .to_str()
                    .unwrap_or("sync_runtime.c")
                    .to_string(),
            );
            needs_pthread = true;
            if verbose {
                println!(
                    "{} Linking sync runtime from: {} (legacy fallback)",
                    "info:".blue().bold(),
                    sync_rt_path.display()
                );
            }
        }
    }

    if needs_pthread {
        args.push("-lpthread".to_string());
    }

    // Native dependencies from vais.toml
    if !native_deps.is_empty() {
        for (name, dep) in native_deps {
            if let Some(lib_path_flag) = dep.lib_path_flag() {
                args.push(lib_path_flag);
            }
            if let Some(include_flag) = dep.include_flag() {
                args.push(include_flag);
            }
            for src in dep.source_files() {
                let src_path = if Path::new(src).is_absolute() {
                    PathBuf::from(src)
                } else if let Some(parent) = ir_path.parent() {
                    parent.join(src)
                } else {
                    PathBuf::from(src)
                };
                args.push(src_path.to_string_lossy().to_string());
            }
            for flag in dep.lib_flags() {
                if !args.contains(&flag) {
                    args.push(flag);
                }
            }
            if verbose {
                println!(
                    "{} Linking native dependency: {}",
                    "info:".blue().bold(),
                    name
                );
            }
        }
    }

    Ok(())
}

/// Helper to add runtime libraries to clang link arguments.
/// Extracted from compile_to_native to share with cached .o link path.
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_runtime_libs(
    args: &mut Vec<String>,
    verbose: bool,
    used_modules: &HashSet<String>,
    native_deps: &HashMap<String, package::NativeDependency>,
    _hot: bool,
) -> Result<(), String> {
    // Link math library (required on Linux for sqrt, sin, cos, etc.)
    #[cfg(target_os = "linux")]
    args.push("-lm".to_string());

    // Link against libvais_gc if available
    if let Some(gc_lib_path) = find_gc_library() {
        let static_lib = gc_lib_path.join("libvais_gc.a");
        args.push(static_lib.to_str().unwrap_or("libvais_gc.a").to_string());
    }

    // Link C runtimes based on used modules
    let mut needs_pthread = false;
    let mut linked_libs: HashSet<&str> = HashSet::new();
    let mut linked_runtimes: Vec<String> = Vec::new();

    for module in used_modules {
        if let Some(runtime_info) = get_runtime_for_module(module) {
            if let Some(rt_path) = find_runtime_file(runtime_info.file) {
                let rt_str = rt_path.to_str().unwrap_or(runtime_info.file).to_string();
                if !linked_runtimes.contains(&rt_str) {
                    linked_runtimes.push(rt_str.clone());
                    args.push(rt_str);
                }
            }
            if runtime_info.needs_pthread {
                needs_pthread = true;
            }
            for lib in runtime_info.libs {
                if !linked_libs.contains(lib) {
                    linked_libs.insert(lib);
                    args.push(lib.to_string());
                }
            }
        }
    }

    // Legacy fallbacks
    if linked_runtimes.is_empty() {
        if let Some(http_rt_path) = find_http_runtime() {
            args.push(
                http_rt_path
                    .to_str()
                    .unwrap_or("http_runtime.c")
                    .to_string(),
            );
        }
        if let Some(thread_rt_path) = find_thread_runtime() {
            args.push(
                thread_rt_path
                    .to_str()
                    .unwrap_or("thread_runtime.c")
                    .to_string(),
            );
            needs_pthread = true;
        }
        if let Some(sync_rt_path) = find_sync_runtime() {
            args.push(
                sync_rt_path
                    .to_str()
                    .unwrap_or("sync_runtime.c")
                    .to_string(),
            );
            needs_pthread = true;
        }
    }

    if needs_pthread {
        args.push("-lpthread".to_string());
    }

    // Native dependencies from vais.toml
    for (name, dep) in native_deps {
        if let Some(lib_path_flag) = dep.lib_path_flag() {
            args.push(lib_path_flag);
        }
        if let Some(include_flag) = dep.include_flag() {
            args.push(include_flag);
        }
        for src in dep.source_files() {
            args.push(src.to_string());
        }
        for flag in dep.lib_flags() {
            if !args.contains(&flag) {
                args.push(flag);
            }
        }
        if verbose {
            println!(
                "{} Linking native dependency: {}",
                "info:".blue().bold(),
                name
            );
        }
    }

    Ok(())
}

/// Pipeline-based compilation where parsing, type-checking, and codegen overlap.
///
/// Uses a producer-consumer pattern with channels to enable immediate type-checking
/// as soon as each module is parsed, rather than waiting for all modules to be parsed.
///
/// # Architecture
///
/// ```text
/// Producer Thread:  parse(mod1) → send → parse(mod2) → send → ...
///                                  ↓                      ↓
/// Consumer Thread:           typecheck(mod1)      typecheck(mod2) → ...
/// ```
///
/// # Arguments
///
/// * `modules` - Map from module path to source code
/// * `dep_graph` - Dependency graph for determining compilation order
/// * `target` - Target triple for code generation
/// * `opt_level` - Optimization level (0-3)
/// * `debug` - Whether to generate debug information
/// * `verbose` - Print detailed timing and progress information
/// * `gc` - Enable garbage collection
/// * `gc_threshold` - GC threshold if enabled
/// * `input_canonical` - Canonical path to main input file
/// * `main_source` - Source code of main file (for debug info)
///
/// # Returns
///
/// A vector of (module_name, IR_string) pairs for all modules.
///
/// # Implementation Details
///
/// 1. Creates a bounded channel (capacity = num_cpus) for parsed modules
/// 2. Spawns producer thread that parses modules in dependency order
/// 3. Main thread consumes parsed modules and type-checks them immediately
/// 4. After type-checking completes, generates IR for all modules in parallel
/// 5. Independent modules are parsed concurrently within each dependency level
///
/// Reserved for pipelined compilation mode (Phase 2 implementation).
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn pipeline_compile(
    modules: HashMap<PathBuf, String>,
    dep_graph: &incremental::DependencyGraph,
    target: &TargetTriple,
    opt_level: u8,
    debug: bool,
    verbose: bool,
    gc: bool,
    gc_threshold: Option<usize>,
    input_canonical: &Path,
    main_source: &str,
) -> Result<Vec<(String, String)>, String> {
    use std::sync::mpsc;
    use std::time::Instant;
    use vais_codegen::CodeGenerator;

    let start = Instant::now();

    // Get dependency levels for processing order
    let levels = dep_graph.parallel_levels();

    if verbose {
        println!(
            "{} Pipeline compilation: {} level(s), {} module(s)",
            "⚡".cyan().bold(),
            levels.len(),
            modules.len()
        );
    }

    // Create bounded channel for parsed modules
    let num_cpus = rayon::current_num_threads();
    let (tx, rx) = mpsc::sync_channel::<Result<(PathBuf, vais_ast::Module), String>>(num_cpus);

    // Shared data structures
    let modules_arc = Arc::new(modules);
    let levels_arc = Arc::new(levels);
    let total_modules = modules_arc.len();

    // Clone Arc handles for the producer thread
    let modules_clone = Arc::clone(&modules_arc);
    let levels_clone = Arc::clone(&levels_arc);
    let tx_clone = tx.clone();

    // Spawn producer thread for parsing
    let parse_handle = std::thread::spawn(move || -> Result<(), String> {
        for (level_idx, level) in levels_clone.iter().enumerate() {
            if verbose {
                println!(
                    "  {} Parsing level {}: {} module(s)",
                    "→".cyan(),
                    level_idx + 1,
                    level.len()
                );
            }

            // Parse modules in this level in parallel using rayon
            let parse_results: Vec<Result<(PathBuf, vais_ast::Module), String>> = level
                .par_iter()
                .map(|module_path| {
                    let source = modules_clone.get(module_path).ok_or_else(|| {
                        format!("Source not found for module: {}", module_path.display())
                    })?;

                    // Parse the module
                    let ast = vais_parser::parse(source).map_err(|e| {
                        format!("Parse error in '{}': {}", module_path.display(), e)
                    })?;

                    Ok((module_path.clone(), ast))
                })
                .collect();

            // Send parsed results through channel in order
            for result in parse_results {
                tx_clone.send(result).map_err(|_| "Channel send failed")?;
            }
        }

        drop(tx_clone);
        Ok(())
    });

    // Consumer: receive parsed modules and type-check immediately
    let mut checker = TypeChecker::new();
    let mut all_modules: Vec<(PathBuf, vais_ast::Module)> = Vec::new();
    let mut modules_map: HashMap<PathBuf, Vec<usize>> = HashMap::new();
    let mut current_item_offset = 0;

    let mut parsed_count = 0;
    while let Ok(result) = rx.recv() {
        let (module_path, module) = result?;
        parsed_count += 1;

        if verbose {
            println!(
                "  {} Type-checking: {} ({}/{})",
                "✓".green(),
                module_path.display(),
                parsed_count,
                total_modules
            );
        }

        // Type-check this module immediately
        checker.check_module(&module).map_err(|e| {
            format!("Type error in '{}': {}", module_path.display(), e)
        })?;

        // Track module items for later codegen
        let num_items = module.items.len();
        let item_indices: Vec<usize> = (current_item_offset..current_item_offset + num_items).collect();
        modules_map.insert(module_path.clone(), item_indices);
        current_item_offset += num_items;

        all_modules.push((module_path, module));
    }

    // Wait for parsing thread to complete
    parse_handle
        .join()
        .map_err(|_| "Parse thread panicked".to_string())??;

    if verbose {
        println!(
            "  {} Parse + type-check time: {:.3}s",
            "⏱".cyan(),
            start.elapsed().as_secs_f64()
        );
    }

    // Phase 2: Merge all modules into a single AST for codegen
    let mut all_items = Vec::new();
    for (_, module) in &all_modules {
        all_items.extend(module.items.clone());
    }

    let final_ast = vais_ast::Module {
        items: all_items,
        modules_map: Some(modules_map.clone()),
    };

    // Phase 3: Generate IR for all modules in parallel using dependency levels
    let codegen_start = Instant::now();

    if verbose {
        println!(
            "{} Parallel codegen: {} level(s)",
            "⚡".cyan().bold(),
            levels_arc.len()
        );
    }

    let effective_opt_level = if debug { 0 } else { opt_level };
    let resolved_functions = checker.get_all_functions().clone();
    let all_irs = Arc::new(Mutex::new(Vec::new()));

    // Process each codegen level sequentially
    for (level_idx, level) in levels_arc.iter().enumerate() {
        let level_start = Instant::now();

        if verbose && !level.is_empty() {
            println!(
                "  {} Codegen level {}: {} module(s)",
                "→".cyan(),
                level_idx + 1,
                level.len()
            );
        }

        // Generate IR for modules in this level in parallel
        let results: Vec<Result<(String, String), String>> = level
            .par_iter()
            .map(|module_path| {
                let module_stem = module_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let is_main = *module_path == input_canonical;

                // Get item indices for this module
                let item_indices = modules_map.get(module_path).ok_or_else(|| {
                    format!("No items found for module: {}", module_path.display())
                })?;

                // Create a fresh CodeGenerator for this module
                let mut codegen = CodeGenerator::new_with_target(&module_stem, target.clone());
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
                    let source_dir = module_path.parent().and_then(|p| p.to_str()).unwrap_or(".");
                    codegen.enable_debug(source_file, source_dir, main_source);
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

        // Collect results for this level
        for result in results {
            let (module_stem, ir) = result?;
            all_irs
                .lock()
                .map_err(|_| "Mutex poisoned".to_string())?
                .push((module_stem, ir));
        }

        if verbose {
            println!(
                "    {} Level {} completed in {:.3}s",
                "✓".green(),
                level_idx + 1,
                level_start.elapsed().as_secs_f64()
            );
        }
    }

    if verbose {
        println!(
            "  {} Codegen time: {:.3}s",
            "⏱".cyan(),
            codegen_start.elapsed().as_secs_f64()
        );
        println!(
            "  {} Total pipeline time: {:.3}s",
            "⏱".cyan(),
            start.elapsed().as_secs_f64()
        );
    }

    // Extract final results from Arc<Mutex<>>
    Arc::try_unwrap(all_irs)
        .map_err(|_| "Failed to unwrap IR results".to_string())?
        .into_inner()
        .map_err(|_| "Mutex poisoned on final unwrap".to_string())
}

pub(crate) fn compile_to_wasm32(
    ir_path: &Path,
    bin_path: &Path,
    opt_level: u8,
    verbose: bool,
) -> Result<(), String> {
    let opt_flag = format!("-O{}", opt_level.min(3));

    let ir_str = ir_path
        .to_str()
        .ok_or_else(|| "Invalid UTF-8 in IR path".to_string())?;
    let bin_str = bin_path
        .to_str()
        .ok_or_else(|| "Invalid UTF-8 in output path".to_string())?;

    // WebAssembly 32-bit compilation
    let args = vec![
        "--target=wasm32-unknown-unknown",
        "-nostdlib",
        "-Wl,--no-entry",
        "-Wl,--allow-undefined",
        "-Wl,--export-all",
        "-Wl,--export=_start",
        "-Wl,--export=malloc",
        "-Wl,--export=free",
        "-Wl,--import-memory",
        &opt_flag,
        "-o",
        bin_str,
        ir_str,
    ];

    // Check for wasm-ld directly if clang fails
    let status = Command::new("clang").args(&args).status();

    match status {
        Ok(s) if s.success() => {
            // Run wasm-opt if available and optimization requested
            if opt_level > 0 {
                run_wasm_opt(bin_path, opt_level, verbose);
            }
            if verbose {
                println!(
                    "{} {} (wasm32-unknown-unknown)",
                    "Compiled".green().bold(),
                    bin_path.display()
                );
            } else {
                println!("{}", bin_path.display());
            }
            Ok(())
        }
        Ok(s) => {
            // Try wasm-ld as fallback
            let wasm_ld_result = compile_wasm32_with_wasm_ld(ir_path, bin_path, opt_level, verbose);
            if wasm_ld_result.is_ok() {
                return wasm_ld_result;
            }
            Err(format!(
                "clang wasm32 compilation failed with code {}",
                s.code().unwrap_or(-1)
            ))
        }
        Err(_) => {
            // Try wasm-ld as fallback
            let wasm_ld_result = compile_wasm32_with_wasm_ld(ir_path, bin_path, opt_level, verbose);
            if wasm_ld_result.is_ok() {
                return wasm_ld_result;
            }
            Err("clang not found. Install LLVM/clang with wasm32 support or use --emit-ir to output LLVM IR only.".to_string())
        }
    }
}

/// Compile WASM using wasm-ld directly (fallback when clang wasm32 fails)
fn compile_wasm32_with_wasm_ld(
    ir_path: &Path,
    bin_path: &Path,
    opt_level: u8,
    verbose: bool,
) -> Result<(), String> {
    // First compile IR to .o with llc
    let obj_path = ir_path.with_extension("o");
    let obj_str = obj_path
        .to_str()
        .ok_or_else(|| "Invalid UTF-8 in obj path".to_string())?;
    let ir_str = ir_path
        .to_str()
        .ok_or_else(|| "Invalid UTF-8 in IR path".to_string())?;

    let llc_status = Command::new("llc")
        .args([
            "-mtriple=wasm32-unknown-unknown",
            "-filetype=obj",
            &format!("-O{}", opt_level.min(3)),
            "-o",
            obj_str,
            ir_str,
        ])
        .status()
        .map_err(|_| "llc not found".to_string())?;

    if !llc_status.success() {
        return Err("llc compilation to wasm object failed".to_string());
    }

    let bin_str = bin_path
        .to_str()
        .ok_or_else(|| "Invalid UTF-8 in output path".to_string())?;

    // Link with wasm-ld
    let ld_status = Command::new("wasm-ld")
        .args([
            "--no-entry",
            "--allow-undefined",
            "--export-all",
            "--import-memory",
            "-o",
            bin_str,
            obj_str,
        ])
        .status()
        .map_err(|_| "wasm-ld not found".to_string())?;

    // Clean up .o file
    let _ = std::fs::remove_file(&obj_path);

    if ld_status.success() {
        if opt_level > 0 {
            run_wasm_opt(bin_path, opt_level, verbose);
        }
        if verbose {
            println!(
                "{} {} (wasm32, via wasm-ld)",
                "Compiled".green().bold(),
                bin_path.display()
            );
        } else {
            println!("{}", bin_path.display());
        }
        Ok(())
    } else {
        Err("wasm-ld linking failed".to_string())
    }
}

/// Run wasm-opt post-processing if available
fn run_wasm_opt(bin_path: &Path, opt_level: u8, verbose: bool) {
    let bin_str = match bin_path.to_str() {
        Some(s) => s,
        None => return,
    };
    let opt_flag = format!("-O{}", opt_level.min(4));

    if let Ok(status) = Command::new("wasm-opt")
        .args([&opt_flag, "-o", bin_str, bin_str])
        .status()
    {
        if status.success() && verbose {
            println!("  {} wasm-opt {}", "optimized".blue().bold(), opt_flag);
        }
    }
    // Silently skip if wasm-opt is not available
}

/// Detect WASI SDK sysroot path
fn detect_wasi_sysroot() -> Option<String> {
    // Check WASI_SDK_PATH environment variable
    if let Ok(sdk) = std::env::var("WASI_SDK_PATH") {
        let sysroot = PathBuf::from(&sdk).join("share").join("wasi-sysroot");
        if sysroot.exists() {
            return sysroot.to_str().map(|s| s.to_string());
        }
        // Also check direct sysroot path
        let sysroot = PathBuf::from(&sdk);
        if sysroot.join("lib").join("wasm32-wasi").exists() {
            return Some(sdk);
        }
    }

    // Check common installation paths
    let common_paths = [
        "/opt/wasi-sdk/share/wasi-sysroot",
        "/usr/local/share/wasi-sysroot",
        "/usr/share/wasi-sysroot",
    ];
    for path in &common_paths {
        if Path::new(path).exists() {
            return Some(path.to_string());
        }
    }

    None
}

pub(crate) fn compile_to_wasi(
    ir_path: &Path,
    bin_path: &Path,
    opt_level: u8,
    verbose: bool,
) -> Result<(), String> {
    let opt_flag = format!("-O{}", opt_level.min(3));

    let ir_str = ir_path
        .to_str()
        .ok_or_else(|| "Invalid UTF-8 in IR path".to_string())?;
    let bin_str = bin_path
        .to_str()
        .ok_or_else(|| "Invalid UTF-8 in output path".to_string())?;

    // Build WASI compilation args
    let mut args: Vec<String> = vec![
        "--target=wasm32-wasi".to_string(),
        opt_flag,
        "-o".to_string(),
        bin_str.to_string(),
        ir_str.to_string(),
    ];

    // Auto-detect WASI sysroot
    if let Some(sysroot) = detect_wasi_sysroot() {
        args.insert(1, format!("--sysroot={}", sysroot));
        if verbose {
            println!("  {} WASI sysroot: {}", "info:".blue().bold(), sysroot);
        }
    }

    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let status = Command::new("clang").args(&args_refs).status();

    match status {
        Ok(s) if s.success() => {
            // Run wasm-opt if available
            if opt_level > 0 {
                run_wasm_opt(bin_path, opt_level, verbose);
            }
            if verbose {
                println!(
                    "{} {} (wasm32-wasi)",
                    "Compiled".green().bold(),
                    bin_path.display()
                );
            } else {
                println!("{}", bin_path.display());
            }
            Ok(())
        }
        Ok(s) => Err(format!(
            "clang wasi compilation failed with code {}. \
             Ensure wasi-sdk is installed (set WASI_SDK_PATH env var).",
            s.code().unwrap_or(-1)
        )),
        Err(_) => Err(
            "clang not found. Install LLVM/clang with wasi-sdk or use --emit-ir to output LLVM IR only."
                .to_string(),
        ),
    }
}

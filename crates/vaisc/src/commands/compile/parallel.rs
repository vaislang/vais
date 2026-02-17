//! Parallel type-checking and codegen using dependency graph levels.

use super::*;

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
    let modules_map = final_ast
        .modules_map
        .as_ref()
        .ok_or_else(|| "modules_map required for parallel type-checking".to_string())?;

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
                checker
                    .check_module(&module)
                    .map_err(|e| format!("{}", e))?;

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
    let modules_map = final_ast
        .modules_map
        .as_ref()
        .ok_or_else(|| "modules_map required for parallel codegen".to_string())?;

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

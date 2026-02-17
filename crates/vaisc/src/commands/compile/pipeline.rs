//! Pipeline-based compilation where parsing, type-checking, and codegen overlap.

use super::*;

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
        checker
            .check_module(&module)
            .map_err(|e| format!("Type error in '{}': {}", module_path.display(), e))?;

        // Track module items for later codegen
        let num_items = module.items.len();
        let item_indices: Vec<usize> =
            (current_item_offset..current_item_offset + num_items).collect();
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

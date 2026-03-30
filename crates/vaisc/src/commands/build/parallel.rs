//! Parallel build paths: parallel type checking and per-module codegen.

use crate::commands::compile::compile_per_module;
use crate::configure_type_checker;
use crate::error_formatter;
use colored::Colorize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use vais_codegen::TargetTriple;
use vais_types::TypeChecker;

/// Run parallel type checking across multiple modules.
///
/// Returns (final_checker, error_messages) where error_messages is empty on success.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_parallel_type_check(
    final_ast: &vais_ast::Module,
    verbose: bool,
) -> (TypeChecker, Vec<String>) {
    use rayon::prelude::*;

    let modules_map = final_ast
        .modules_map
        .as_ref()
        .unwrap_or_else(|| unreachable!("BUG: modules_map should be Some"));
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
                    Err(e) => Err(Box::new((format!("{}", e), module_checker))),
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
                final_checker.merge_type_defs_from(module_checker);
            }
            Err(boxed) => {
                let (err_msg, module_checker) = *boxed;
                all_errors.push(err_msg);
                final_checker.merge_type_defs_from(module_checker);
            }
        }
    }

    (final_checker, all_errors)
}

/// Run per-module codegen with emit-IR-only path.
///
/// Generates per-module .ll files in parallel using rayon.
/// Returns Ok(()) on success, writing each module's IR to `{stem}_{module}.ll`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_per_module_emit_ir(
    final_ast: &vais_ast::Module,
    checker: &TypeChecker,
    target: &TargetTriple,
    input: &Path,
    input_canonical: &Path,
    output: &Option<PathBuf>,
    debug: bool,
    opt_level: u8,
    verbose: bool,
    gc: bool,
    gc_threshold: Option<usize>,
    main_source: &str,
    mmap: &std::collections::HashMap<PathBuf, Vec<usize>>,
    loaded_modules: &HashSet<PathBuf>,
    cache: &mut Option<crate::incremental::IncrementalCache>,
) -> Result<(), String> {
    use rayon::prelude::*;

    let output_dir = output
        .as_ref()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| input.parent().unwrap_or(Path::new(".")));
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("main");

    let effective_opt_level = if debug { 0 } else { opt_level };
    let resolved_functions = checker.get_all_functions_with_methods();
    let resolved_type_aliases = checker.get_type_aliases().clone();
    let resolved_expr_types = checker.get_expr_types().clone();
    let instantiations = checker.get_generic_instantiations();
    for inst in &instantiations {
        eprintln!(
            "  [INST] base={}, mangled={}, kind={:?}, args={:?}",
            inst.base_name, inst.mangled_name, inst.kind, inst.type_args
        );
    }
    let instantiations = &instantiations;

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
            let is_main = **module_path == *input_canonical;

            // Create a fresh CodeGenerator for this module
            let mut codegen =
                vais_codegen::CodeGenerator::new_with_target(&module_stem, target.clone());
            codegen.set_resolved_functions(resolved_functions.clone());
            codegen.set_type_aliases(resolved_type_aliases.clone());
            codegen.set_expr_types(resolved_expr_types.clone());
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
            let result =
                codegen.generate_module_subset(final_ast, item_indices, instantiations, is_main);
            let raw_ir = result.map_err(|e| {
                let spanned = vais_codegen::SpannedCodegenError {
                    span: codegen.last_error_span(),
                    error: e,
                };
                format!(
                    "Codegen error for {}:\n{}",
                    module_stem,
                    error_formatter::format_spanned_codegen_error(&spanned, main_source, input,)
                )
            })?;

            // Verify IR structural integrity before optimization.
            crate::utils::verify_ir_and_log(&raw_ir, &format!("module '{}'", module_stem));

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
        for loaded_path in loaded_modules {
            let _ = c.update_file(loaded_path);
        }
        let _ = c.persist();
    }

    Ok(())
}

/// Run per-module compilation: compile to .o and link.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_per_module_compile(
    final_ast: &vais_ast::Module,
    checker: &TypeChecker,
    target: &TargetTriple,
    input: &Path,
    input_canonical: &Path,
    output: Option<PathBuf>,
    opt_level: u8,
    debug: bool,
    verbose: bool,
    gc: bool,
    gc_threshold: Option<usize>,
    main_source: &str,
    loaded_modules: &HashSet<PathBuf>,
    cache: &mut Option<crate::incremental::IncrementalCache>,
) -> Result<(), String> {
    let default_ext = match target {
        TargetTriple::Wasm32Unknown | TargetTriple::WasiPreview1 | TargetTriple::WasiPreview2 => {
            "wasm"
        }
        _ => "",
    };
    let bin_path = output.unwrap_or_else(|| input.with_extension(default_ext));

    compile_per_module(
        final_ast,
        checker,
        target,
        input_canonical,
        &bin_path,
        opt_level,
        debug,
        verbose,
        gc,
        gc_threshold,
        input,
        main_source,
        cache.as_ref().map(|c| c.cache_dir()),
    )?;

    // Update incremental cache
    if let Some(ref mut c) = cache {
        for loaded_path in loaded_modules {
            let _ = c.update_file(loaded_path);
        }
        let _ = c.persist();
    }

    Ok(())
}

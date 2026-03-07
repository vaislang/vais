//! Backend code generation functions.

use crate::error_formatter;
use colored::Colorize;
use std::path::Path;
use vais_codegen::{CodeGenerator, TargetTriple};
use vais_types::TypeChecker;

/// Text-based IR code generation (default backend).
#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_with_text_backend(
    module_name: &str,
    target: &TargetTriple,
    gc: bool,
    gc_threshold: Option<usize>,
    debug: bool,
    input: &Path,
    main_source: &str,
    checker: &TypeChecker,
    final_ast: &vais_ast::Module,
    verbose: bool,
) -> Result<String, String> {
    let mut codegen = CodeGenerator::new_with_target(module_name, target.clone());

    // Enable GC if requested
    if gc {
        codegen.enable_gc();
        if let Some(threshold) = gc_threshold {
            codegen.set_gc_threshold(threshold);
        }
        if verbose {
            println!(
                "  {} (threshold: {} bytes)",
                "GC enabled".cyan(),
                gc_threshold.unwrap_or(1048576)
            );
        }
    }

    // Enable debug info if requested
    if debug {
        let source_file = input
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown.vais");
        let source_dir = input.parent().and_then(|p| p.to_str()).unwrap_or(".");
        codegen.enable_debug(source_file, source_dir, main_source);

        if verbose {
            println!("  {}", "Debug info enabled".cyan());
        }
    }

    // Pass resolved function signatures to codegen (for inferred parameter types)
    codegen.set_resolved_functions(checker.get_all_functions().clone());
    codegen.set_type_aliases(checker.get_type_aliases().clone());

    // Enable multi-error mode for graceful degradation:
    // collect codegen errors instead of stopping at the first one.
    codegen.multi_error_mode = true;

    if verbose {
        println!("  {} text (IR generation)", "Backend:".cyan());
    }

    let codegen_start = std::time::Instant::now();
    let instantiations = checker.get_generic_instantiations();
    let result = if instantiations.is_empty() {
        codegen.generate_module(final_ast)
    } else {
        codegen.generate_module_with_instantiations(final_ast, &instantiations)
    };

    // Report all collected codegen errors before returning the first fatal one
    for collected_err in codegen.get_collected_errors() {
        eprintln!(
            "{}",
            error_formatter::format_spanned_codegen_error(collected_err, main_source, input)
        );
    }

    let raw_ir = result.map_err(|e| {
        let spanned = vais_codegen::SpannedCodegenError {
            span: codegen.last_error_span(),
            error: e,
        };
        error_formatter::format_spanned_codegen_error(&spanned, main_source, input)
    })?;

    // Report codegen warnings summary
    let codegen_warnings = codegen.get_warnings();
    if !codegen_warnings.is_empty() {
        use std::collections::HashMap;
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for w in &codegen_warnings {
            let key = match w {
                vais_codegen::CodegenWarning::GenericFallback { .. } => "generic fallback",
                vais_codegen::CodegenWarning::AssociatedTypeFallback { .. } => {
                    "associated type fallback"
                }
                vais_codegen::CodegenWarning::UninstantiatedGeneric { .. } => {
                    "uninstantiated generic"
                }
                vais_codegen::CodegenWarning::UnresolvedTypeFallback { .. } => {
                    "unresolved type fallback"
                }
            };
            *counts.entry(key).or_insert(0) += 1;
        }
        if verbose {
            eprintln!(
                "{}: {} codegen warning(s):",
                "warning".yellow().bold(),
                codegen_warnings.len()
            );
            for (kind, count) in &counts {
                eprintln!("  {} {} ({}x)", "·".yellow(), kind, count);
            }
        }
    }

    // Verify IR structural integrity before returning.
    crate::utils::verify_ir_and_log(&raw_ir, "text backend");

    // If we got here, IR was generated but some functions may have failed.
    // Report collected errors as warnings and return the partial IR.
    if !codegen.get_collected_errors().is_empty() {
        let err_count = codegen.get_collected_errors().len();
        eprintln!(
            "{}: {} codegen error(s) occurred during compilation (partial IR generated)",
            "warning".yellow().bold(),
            err_count
        );
    }

    let codegen_time = codegen_start.elapsed();

    if verbose {
        println!(
            "  {} Codegen time: {:.3}s",
            "⏱".cyan(),
            codegen_time.as_secs_f64()
        );
    }

    Ok(raw_ir)
}

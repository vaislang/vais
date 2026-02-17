//! Backend code generation functions.

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

    if verbose {
        println!("  {} text (IR generation)", "Backend:".cyan());
    }

    let codegen_start = std::time::Instant::now();
    let instantiations = checker.get_generic_instantiations();
    let raw_ir = if instantiations.is_empty() {
        codegen.generate_module(final_ast)
    } else {
        codegen.generate_module_with_instantiations(final_ast, instantiations)
    }
    .map_err(|e| format!("Codegen error: {}", e))?;
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

//! Single-module (serial) build path: IR generation, optimization, and binary compilation.

use crate::commands::compile::compile_ir_to_binary;
use crate::error_formatter;
use crate::package;
use crate::runtime::extract_used_modules;
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use vais_codegen::optimize::{optimize_ir_with_pgo, OptLevel};
use vais_codegen::TargetTriple;
use vais_plugin::PluginRegistry;
use vais_types::TypeChecker;

use super::core::CompileProfile;

/// Generate LLVM IR using the text backend for a single module.
#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_ir_single_module(
    final_ast: &vais_ast::Module,
    checker: &TypeChecker,
    target: &TargetTriple,
    input: &Path,
    main_source: &str,
    debug: bool,
    verbose: bool,
    gc: bool,
    gc_threshold: Option<usize>,
    use_inkwell: bool,
    use_parallel: bool,
    profile_out: &mut Option<&mut CompileProfile>,
) -> Result<String, String> {
    let codegen_start = std::time::Instant::now();
    let module_name = input.file_stem().and_then(|s| s.to_str()).unwrap_or("main");

    if verbose && !matches!(target, TargetTriple::Native) {
        println!("  {} {}", "Target:".cyan(), target.triple_str());
    }

    // Inkwell backend path (opt-in via --inkwell flag)
    #[cfg(feature = "inkwell")]
    let raw_ir = if use_inkwell {
        generate_with_inkwell(
            final_ast,
            checker,
            target,
            input,
            main_source,
            module_name,
            verbose,
            gc,
            debug,
        )?
    } else {
        super::generate_with_text_backend(
            module_name,
            target,
            gc,
            gc_threshold,
            debug,
            input,
            main_source,
            checker,
            final_ast,
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
        super::generate_with_text_backend(
            module_name,
            target,
            gc,
            gc_threshold,
            debug,
            input,
            main_source,
            checker,
            final_ast,
            verbose,
        )?
    };

    let codegen_time = codegen_start.elapsed();
    if let Some(ref mut p) = profile_out {
        p.codegen_ms = codegen_time.as_secs_f64() * 1000.0;
    }

    let _ = use_parallel; // used below in optimize step
    Ok(raw_ir)
}

/// Optimize IR and write to file, optionally compiling to binary.
#[allow(clippy::too_many_arguments)]
pub(crate) fn optimize_and_output(
    raw_ir: &str,
    final_ast: &vais_ast::Module,
    input: &Path,
    output: Option<PathBuf>,
    emit_ir: bool,
    opt_level: u8,
    debug: bool,
    verbose: bool,
    target: &TargetTriple,
    hot: bool,
    lto_mode: &vais_codegen::optimize::LtoMode,
    pgo_mode: &vais_codegen::optimize::PgoMode,
    coverage_mode: &vais_codegen::optimize::CoverageMode,
    plugins: &PluginRegistry,
    use_parallel: bool,
    loaded_modules: &HashSet<PathBuf>,
    cache: &mut Option<crate::incremental::IncrementalCache>,
    cache_limit: u64,
    profile_out: &mut Option<&mut CompileProfile>,
) -> Result<(), String> {
    // Apply optimization passes before emitting IR
    let opt_start = std::time::Instant::now();
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
        vais_codegen::parallel::parallel_optimize_ir(raw_ir, opt)
    } else {
        optimize_ir_with_pgo(raw_ir, opt, pgo_mode)
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

    let opt_time = opt_start.elapsed();
    if let Some(ref mut p) = profile_out {
        p.optimize_ms = opt_time.as_secs_f64() * 1000.0;
    }

    // Determine output paths
    let ir_path = if emit_ir {
        output.clone().unwrap_or_else(|| input.with_extension("ll"))
    } else {
        input.with_extension("ll")
    };

    // Verify IR structural integrity before writing to file.
    crate::utils::verify_ir_and_log(&ir, "single module");

    // Write IR
    fs::write(&ir_path, &ir).map_err(|e| format!("Cannot write '{}': {}", ir_path.display(), e))?;

    if verbose || emit_ir {
        println!("{} {}", "Wrote".green().bold(), ir_path.display());
    }

    // Run codegen plugins (generate additional files)
    if !plugins.is_empty() {
        let output_dir = ir_path.parent().unwrap_or(Path::new("."));
        match plugins.run_codegen(final_ast, output_dir) {
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
        let used_modules = extract_used_modules(final_ast);
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

        let clang_start = std::time::Instant::now();
        compile_ir_to_binary(
            &ir_path,
            &bin_path,
            effective_opt_level,
            debug,
            verbose,
            target,
            hot,
            lto_mode,
            pgo_mode,
            coverage_mode,
            &used_modules,
            &native_deps,
            cache.as_ref().map(|c| c.cache_dir()),
        )?;
        let clang_time = clang_start.elapsed();
        if let Some(ref mut p) = profile_out {
            p.clang_ms = clang_time.as_secs_f64() * 1000.0;
        }
    }

    // Update incremental compilation cache after successful build
    update_cache(cache, loaded_modules, verbose, cache_limit);

    Ok(())
}

/// Update the incremental compilation cache after a successful build.
pub(crate) fn update_cache(
    cache: &mut Option<crate::incremental::IncrementalCache>,
    loaded_modules: &HashSet<PathBuf>,
    verbose: bool,
    cache_limit: u64,
) {
    if let Some(ref mut c) = cache {
        // Update file metadata for all loaded modules
        for loaded_path in loaded_modules {
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
}

/// Generate IR using the Inkwell (LLVM API) backend.
#[cfg(feature = "inkwell")]
#[allow(clippy::too_many_arguments)]
fn generate_with_inkwell(
    final_ast: &vais_ast::Module,
    checker: &TypeChecker,
    target: &TargetTriple,
    input: &Path,
    main_source: &str,
    module_name: &str,
    verbose: bool,
    gc: bool,
    debug: bool,
) -> Result<String, String> {
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
    let mut gen =
        vais_codegen::InkwellCodeGenerator::new_with_target(&context, module_name, target.clone());
    gen.set_resolved_functions(checker.get_all_functions().clone());
    gen.set_type_aliases(checker.get_type_aliases().clone());
    let instantiations = checker.get_generic_instantiations();
    if instantiations.is_empty() {
        gen.generate_module(final_ast)
            .map_err(|e| error_formatter::format_codegen_error(&e, main_source, input))?;
    } else {
        gen.generate_module_with_instantiations(final_ast, &instantiations)
            .map_err(|e| error_formatter::format_codegen_error(&e, main_source, input))?;
    }

    // Verify the generated LLVM module using LLVMVerifyModule
    if let Err(verify_err) = gen.verify_module() {
        if verbose {
            eprintln!(
                "{} {}",
                "LLVM IR verification warning:".yellow().bold(),
                verify_err
            );
        }
    }

    // Report codegen warnings summary
    let codegen_warnings = gen.get_warnings();
    if !codegen_warnings.is_empty() {
        use std::collections::HashMap as WarnMap;
        let mut counts: WarnMap<&str, usize> = WarnMap::new();
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

    let ir = gen.get_ir_string();
    let codegen_time = codegen_start.elapsed();

    if verbose {
        println!(
            "  {} Codegen time: {:.3}s",
            "⏱".cyan(),
            codegen_time.as_secs_f64()
        );
    }

    Ok(ir)
}

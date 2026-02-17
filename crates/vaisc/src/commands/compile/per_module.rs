//! Per-module compilation with parallel IR generation and incremental caching.

use super::*;

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

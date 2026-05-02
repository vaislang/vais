//! Per-module compilation with parallel IR generation and incremental caching.

use super::*;

/// Phase 17.H1: FNV-1a 32 hash of a module path for use as codegen
/// `current_file_id`. Must match the corresponding function in
/// `build::parallel` and `build::core` so the TC-stamped
/// `expr_types` entries line up with codegen's lookup.
fn phase17_fnv1a_file_id_compile(path: &Path) -> u32 {
    const FNV_OFFSET: u32 = 0x811c_9dc5;
    const FNV_PRIME: u32 = 0x0100_0193;
    let mut hash = FNV_OFFSET;
    for byte in path.to_string_lossy().as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    if hash == 0 {
        1
    } else {
        hash
    }
}

/// Stable, path-sensitive stem for per-module artifacts.
///
/// File stems alone are not unique in real packages (`types.vais` appears in
/// multiple VaisDB submodules). Using only the basename lets parallel codegen
/// race on the same `.ll` path and can compile the wrong IR into more than one
/// object file. Keep the readable basename, but suffix it with a deterministic
/// hash of the canonical module path.
pub(crate) fn module_artifact_stem(path: &Path) -> String {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    let raw_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("module");
    let mut safe_stem = String::with_capacity(raw_stem.len());
    for ch in raw_stem.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            safe_stem.push(ch);
        } else {
            safe_stem.push('_');
        }
    }
    if safe_stem.is_empty() {
        safe_stem.push_str("module");
    }

    let mut hash = FNV_OFFSET;
    for byte in path.to_string_lossy().as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("{}_{:016x}", safe_stem, hash)
}

/// Phase 17.H4.14: load generic struct templates (Vec, HashMap, Option,
/// Result) from stdlib. Parsed once per build; the resulting Rc<Struct>s
/// are injected into each per-module CodeGenerator's generics.struct_defs
/// so method specialization (Vec_new$T, etc.) works even when the user
/// didn't explicitly `U std/vec`.
pub(crate) fn phase17_load_stdlib_generic_templates_pub() -> Vec<vais_ast::Struct> {
    phase17_load_stdlib_generic_templates()
}

fn phase17_load_stdlib_generic_templates() -> Vec<vais_ast::Struct> {
    let Some(std_path) = crate::imports::get_std_path() else {
        return Vec::new();
    };
    let files = ["vec.vais", "option.vais", "hashmap.vais", "result.vais"];
    let mut structs_by_name: std::collections::HashMap<String, vais_ast::Struct> =
        std::collections::HashMap::new();
    let mut impls_by_type: std::collections::HashMap<
        String,
        Vec<vais_ast::Spanned<vais_ast::Function>>,
    > = std::collections::HashMap::new();
    for file in &files {
        let path = std_path.join(file);
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(module) = vais_parser::parse(&source) else {
            continue;
        };
        for item in module.items {
            match item.node {
                vais_ast::Item::Struct(s) if !s.generics.is_empty() => {
                    structs_by_name.insert(s.name.node.clone(), s);
                }
                vais_ast::Item::Impl(imp) => {
                    if let vais_ast::Type::Named { name, .. } = &imp.target_type.node {
                        impls_by_type
                            .entry(name.clone())
                            .or_default()
                            .extend(imp.methods.into_iter());
                    }
                }
                _ => {}
            }
        }
    }
    // Attach impl methods to corresponding struct templates
    for (name, methods) in impls_by_type {
        if let Some(s) = structs_by_name.get_mut(&name) {
            for m in methods {
                if !s
                    .methods
                    .iter()
                    .any(|em| em.node.name.node == m.node.name.node)
                {
                    s.methods.push(m);
                }
            }
        }
    }
    structs_by_name.into_values().collect()
}

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
    let resolved_functions = checker.get_all_functions_with_methods();
    let resolved_type_aliases = checker.get_type_aliases().clone();
    let resolved_expr_types = checker.get_resolved_expr_types();
    let resolved_implicit_try_sites = checker.get_implicit_try_sites().clone();
    let instantiations = checker.get_generic_instantiations();
    let instantiations = &instantiations;

    // Phase 17.H4.14: pre-parse stdlib generic struct templates (Vec,
    // HashMap, Option, Result) so each per-module CodeGenerator has
    // their method templates available for on-demand monomorphization.
    // Without this, modules that use `Vec.new()` but don't import
    // `std/vec` emit unmangled `@Vec_new` calls that clang rejects.
    let stdlib_templates = phase17_load_stdlib_generic_templates();

    // Phase 1: Generate IR for all modules (parallelized with rayon by default).
    //
    // Set VAIS_PARALLEL_CODEGEN=0 to force sequential codegen. This is useful for
    // debugging multi-error scenarios where parallel work-stealing causes the
    // "first reported error" to be non-deterministic across runs, making RCA
    // of multiple simultaneous codegen bugs very difficult. Sequential mode
    // processes modules in a stable iteration order.
    let parallel_codegen = std::env::var("VAIS_PARALLEL_CODEGEN")
        .map(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
        .unwrap_or(true);

    // Collect (module_stem, is_main, ir_string) tuples.
    // In sequential mode, sort by path to guarantee deterministic error reporting order.
    // (Parallel mode via rayon also has non-deterministic work-stealing, so sorting here
    //  doesn't fully stabilize parallel errors but doesn't hurt.)
    let mut module_entries: Vec<_> = modules_map.iter().collect();
    if !parallel_codegen {
        module_entries.sort_by(|a, b| a.0.cmp(b.0));
    }
    let ir_results: Vec<Result<(String, bool, String), String>> = {
        let mapper = |(module_path, item_indices): &(&std::path::PathBuf, &Vec<usize>)| {
            let module_stem = module_artifact_stem(module_path);
            let is_main = *module_path == input_canonical;

            // Create a fresh CodeGenerator for this module
            let mut codegen = CodeGenerator::new_with_target(&module_stem, target.clone());
            codegen.set_resolved_functions(resolved_functions.clone());
            codegen.set_type_aliases(resolved_type_aliases.clone());
            codegen.set_expr_types(resolved_expr_types.clone());
            codegen.set_implicit_try_sites(resolved_implicit_try_sites.clone());
            // Phase 17.H4.14: seed generic struct templates before subset
            // iterates final_ast items, so Vec/HashMap/Option/Result
            // specializations can fire when the user didn't explicitly
            // import std/vec etc.
            codegen.inject_generic_struct_templates(
                stdlib_templates
                    .iter()
                    .cloned()
                    .map(std::rc::Rc::new)
                    .collect(),
            );
            // Phase 17.H1: set per-module file_id identical to the one TC
            // used, so expr_types lookups find the entries stored under
            // this module's namespaced key.
            codegen.set_current_file_id(phase17_fnv1a_file_id_compile(module_path));
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
                // Error spans reference the module where the error originated,
                // not the main entry file. Load the module's own source so the
                // formatter renders the correct file + line + snippet.
                let (err_source, err_path): (std::borrow::Cow<'_, str>, &Path) = if is_main {
                    (std::borrow::Cow::Borrowed(main_source), input)
                } else {
                    match fs::read_to_string(module_path.as_path()) {
                        Ok(s) => (std::borrow::Cow::Owned(s), module_path.as_path()),
                        Err(_) => (std::borrow::Cow::Borrowed(main_source), input),
                    }
                };
                format!(
                    "Codegen error for {}:\n{}",
                    module_stem,
                    error_formatter::format_spanned_codegen_error(&spanned, &err_source, err_path)
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

            Ok((module_stem, is_main, ir))
        };
        if parallel_codegen {
            module_entries.par_iter().map(mapper).collect()
        } else {
            module_entries.iter().map(mapper).collect()
        }
    };

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

    // Phase 2: Compile .ll → .o with content-hash caching (parallelized unless
    // VAIS_PARALLEL_CODEGEN=0, in which case sequential to match Phase 1 ordering).
    let compile_start = std::time::Instant::now();

    let compile_one = |(module_stem, _is_main, ir): &(String, bool, String)| {
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
    };

    let obj_results: Vec<Result<(PathBuf, bool), String>> = if parallel_codegen {
        module_irs.par_iter().map(compile_one).collect()
    } else {
        module_irs.iter().map(compile_one).collect()
    };

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

    let mut needs_pthread = false;
    let mut linked_libs: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut linked_runtimes: Vec<String> = Vec::new();
    let mut used_modules = crate::runtime::extract_used_modules(final_ast);
    for module_path in modules_map.keys() {
        let is_std_file = module_path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            == Some("std");
        if !is_std_file {
            continue;
        }
        let Some(stem) = module_path.file_stem().and_then(|name| name.to_str()) else {
            continue;
        };
        let module_name = format!("std::{stem}");
        if get_runtime_for_module(&module_name).is_some() {
            used_modules.insert(module_name);
        }
    }
    for module in &used_modules {
        if let Some(runtime_info) = get_runtime_for_module(module) {
            for runtime_file in runtime_files_for_module(module, runtime_info.file) {
                if let Some(rt_path) = find_runtime_file(runtime_file) {
                    let rt_str = rt_path.to_str().unwrap_or(runtime_file).to_string();
                    if !linked_runtimes.contains(&rt_str) {
                        linked_runtimes.push(rt_str.clone());
                        link_args.push(rt_str);
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
            }
            if runtime_info.needs_pthread {
                needs_pthread = true;
            }
            for lib in runtime_info.libs {
                if !linked_libs.contains(lib) {
                    linked_libs.insert(lib);
                    link_args.push(lib.to_string());
                }
            }
        }
    }
    if needs_pthread {
        link_args.push("-lpthread".to_string());
    }

    // Phase 4c.4 / Task #55 — reproducible linker metadata.
    // Shared with `compile_to_native` so both the per-module and
    // whole-program link paths produce bit-identical binaries.
    append_reproducible_link_flags(&mut link_args);

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

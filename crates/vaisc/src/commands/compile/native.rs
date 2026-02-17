//! Native binary compilation with LLVM/clang backend.

use super::*;

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
    add_runtime_and_native_libs(&mut args, verbose, used_modules, native_deps, ir_path)?;

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
pub(super) fn print_compilation_success(
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
pub(super) fn setup_profiling_dirs(
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
pub(super) fn add_runtime_and_native_libs(
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

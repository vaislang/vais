//! WebAssembly compilation targets (wasm32-unknown-unknown and WASI).

use super::*;

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

/// Compile to WASI Preview 2 (wasm32-wasip2) with Component Model support
///
/// Unlike Preview 1, this compilation path:
/// 1. Uses `wasm32-wasip2` as the target triple
/// 2. Generates a core WASM module first, then optionally converts to a Component
/// 3. Supports Component Model adapter modules (wasi_snapshot_preview1.reactor.wasm)
/// 4. Can invoke `wasm-tools component new` for core-to-component conversion
pub(crate) fn compile_to_wasi_p2(
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

    // Build WASI Preview 2 compilation args with wasm32-wasip2 target
    let mut args: Vec<String> = vec![
        "--target=wasm32-wasip2".to_string(),
        opt_flag,
        "-o".to_string(),
        bin_str.to_string(),
        ir_str.to_string(),
    ];

    // Auto-detect WASI sysroot (shared with Preview 1)
    if let Some(sysroot) = detect_wasi_sysroot() {
        args.insert(1, format!("--sysroot={}", sysroot));
        if verbose {
            println!("  {} WASI P2 sysroot: {}", "info:".blue().bold(), sysroot);
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

            // Attempt component conversion with wasm-tools if available
            if let Some(wasm_tools) = detect_wasm_tools() {
                let component_path = bin_path.with_extension("component.wasm");
                match run_wasm_tools_component(&wasm_tools, bin_path, &component_path, verbose) {
                    Ok(()) => {
                        // Validate the generated component
                        run_wasm_tools_validate(&wasm_tools, &component_path, verbose);
                        if verbose {
                            println!(
                                "  {} component: {}",
                                "generated".blue().bold(),
                                component_path.display()
                            );
                        }
                    }
                    Err(e) => {
                        if verbose {
                            println!(
                                "  {} component conversion skipped: {}",
                                "note:".yellow().bold(),
                                e
                            );
                        }
                    }
                }
            }

            if verbose {
                println!(
                    "{} {} (wasm32-wasip2)",
                    "Compiled".green().bold(),
                    bin_path.display()
                );
            } else {
                println!("{}", bin_path.display());
            }
            Ok(())
        }
        Ok(s) => Err(format!(
            "clang wasi-preview2 compilation failed with code {}. \
             Ensure wasi-sdk >= 21 is installed with wasm32-wasip2 support \
             (set WASI_SDK_PATH env var).",
            s.code().unwrap_or(-1)
        )),
        Err(_) => Err(
            "clang not found. Install LLVM/clang with wasi-sdk or use --emit-ir to output LLVM IR only."
                .to_string(),
        ),
    }
}

/// Detect wasm-tools binary path
///
/// Checks in order:
/// 1. WASM_TOOLS_PATH environment variable
/// 2. PATH lookup via `which wasm-tools`
fn detect_wasm_tools() -> Option<PathBuf> {
    // Check WASM_TOOLS_PATH environment variable first
    if let Ok(path) = std::env::var("WASM_TOOLS_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Some(p);
        }
    }

    // Try to find wasm-tools in PATH
    if let Ok(output) = Command::new("which").arg("wasm-tools").output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return Some(PathBuf::from(path_str));
            }
        }
    }

    None
}

/// Convert a core WASM module to a Component Model component using wasm-tools
///
/// Runs: `wasm-tools component new <core.wasm> -o <component.wasm> [--adapt <adapter>]`
fn run_wasm_tools_component(
    wasm_tools: &Path,
    core_wasm: &Path,
    output: &Path,
    verbose: bool,
) -> Result<(), String> {
    let core_str = core_wasm
        .to_str()
        .ok_or_else(|| "Invalid UTF-8 in core WASM path".to_string())?;
    let out_str = output
        .to_str()
        .ok_or_else(|| "Invalid UTF-8 in output path".to_string())?;

    let mut cmd = Command::new(wasm_tools);
    cmd.args(["component", "new", core_str, "-o", out_str]);

    // Look for WASI adapter module
    if let Some(adapter) = detect_wasi_adapter() {
        cmd.arg("--adapt").arg(&adapter);
        if verbose {
            println!("  {} adapter: {}", "info:".blue().bold(), adapter);
        }
    }

    let status = cmd
        .status()
        .map_err(|e| format!("Failed to run wasm-tools: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "wasm-tools component new failed with code {}",
            status.code().unwrap_or(-1)
        ))
    }
}

/// Validate a WASM component using wasm-tools validate
fn run_wasm_tools_validate(wasm_tools: &Path, wasm_path: &Path, verbose: bool) {
    let wasm_str = match wasm_path.to_str() {
        Some(s) => s,
        None => return,
    };

    if let Ok(status) = Command::new(wasm_tools)
        .args(["validate", "--features", "component-model", wasm_str])
        .status()
    {
        if verbose {
            if status.success() {
                println!("  {} component validated", "ok:".green().bold());
            } else {
                println!("  {} component validation failed", "warn:".yellow().bold());
            }
        }
    }
}

/// Detect WASI Preview 1 adapter module for component conversion
///
/// The adapter (wasi_snapshot_preview1.reactor.wasm or command.wasm)
/// bridges Preview 1 imports to Preview 2 interfaces.
fn detect_wasi_adapter() -> Option<String> {
    // Check WASI_ADAPTER_PATH environment variable
    if let Ok(path) = std::env::var("WASI_ADAPTER_PATH") {
        if Path::new(&path).exists() {
            return Some(path);
        }
    }

    // Check common locations relative to WASI SDK
    if let Ok(sdk) = std::env::var("WASI_SDK_PATH") {
        let candidates = [
            PathBuf::from(&sdk)
                .join("share")
                .join("wasi-sysroot")
                .join("lib")
                .join("wasm32-wasip2")
                .join("wasi_snapshot_preview1.reactor.wasm"),
            PathBuf::from(&sdk)
                .join("share")
                .join("wasi-sysroot")
                .join("lib")
                .join("wasm32-wasip2")
                .join("wasi_snapshot_preview1.command.wasm"),
            PathBuf::from(&sdk)
                .join("lib")
                .join("wasi_snapshot_preview1.reactor.wasm"),
            PathBuf::from(&sdk)
                .join("lib")
                .join("wasi_snapshot_preview1.command.wasm"),
        ];
        for candidate in &candidates {
            if candidate.exists() {
                return candidate.to_str().map(|s| s.to_string());
            }
        }
    }

    // Check common system paths
    let system_paths = [
        "/usr/local/lib/wasi/wasi_snapshot_preview1.reactor.wasm",
        "/usr/local/share/wasi-adapters/wasi_snapshot_preview1.reactor.wasm",
    ];
    for path in &system_paths {
        if Path::new(path).exists() {
            return Some(path.to_string());
        }
    }

    None
}

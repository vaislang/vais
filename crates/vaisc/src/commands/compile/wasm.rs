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

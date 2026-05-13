//! GPU compilation and codegen functions.

use super::find_std_dir;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

pub(crate) fn cmd_build_gpu(
    input: &PathBuf,
    output: Option<PathBuf>,
    gpu_target: &str,
    emit_host: bool,
    compile: bool,
    verbose: bool,
) -> Result<(), String> {
    use vais_gpu::{GpuCodeGenerator, GpuTarget};

    // Parse GPU target
    let target = GpuTarget::parse(gpu_target).ok_or_else(|| {
        format!(
            "Unknown GPU target: '{}'. Valid targets: cuda, opencl, webgpu, metal",
            gpu_target
        )
    })?;

    if verbose {
        println!(
            "{} Compiling for GPU target: {}",
            "info:".blue().bold(),
            target.name()
        );
    }

    // Read source
    let source = fs::read_to_string(input)
        .map_err(|e| format!("Failed to read {}: {}", input.display(), e))?;

    // Parse
    let module = vais_parser::parse(&source).map_err(|e| format!("Parse error: {:?}", e))?;

    // Generate GPU code
    let mut generator = GpuCodeGenerator::new(target);
    let gpu_code = generator
        .generate(&module)
        .map_err(|e| format!("GPU codegen error: {}", e))?;

    // Determine output file
    let out_path = output.unwrap_or_else(|| {
        let stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        PathBuf::from(format!("{}.{}", stem, target.extension()))
    });

    // Write output
    fs::write(&out_path, &gpu_code)
        .map_err(|e| format!("Failed to write {}: {}", out_path.display(), e))?;

    println!(
        "{} Generated {} ({})",
        "✓".green().bold(),
        out_path.display(),
        target.name()
    );

    // Print kernel information
    let kernels = generator.kernels();
    if !kernels.is_empty() {
        println!(
            "\n{} {} kernel(s) generated:",
            "info:".blue().bold(),
            kernels.len()
        );
        for kernel in kernels {
            println!(
                "  - {} ({} params, block size: {:?})",
                kernel.name,
                kernel.params.len(),
                kernel.block_size
            );
        }
    }

    // Generate host code template if requested
    if emit_host {
        match generator.generate_host_code() {
            Ok(host_code) => {
                let host_ext = match target {
                    GpuTarget::Cuda => "host.cu",
                    GpuTarget::OpenCL => "host.c",
                    GpuTarget::WebGPU => "host.ts",
                    GpuTarget::Metal => "host.swift",
                };
                let host_path = input
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|stem| PathBuf::from(format!("{}.{}", stem, host_ext)))
                    .unwrap_or_else(|| PathBuf::from(format!("output.{}", host_ext)));

                fs::write(&host_path, &host_code).map_err(|e| {
                    format!("Failed to write host code {}: {}", host_path.display(), e)
                })?;

                println!(
                    "{} Generated host code: {} ({})",
                    "✓".green().bold(),
                    host_path.display(),
                    target.name()
                );
            }
            Err(e) => {
                eprintln!(
                    "{} Warning: Could not generate host code: {}",
                    "⚠".yellow().bold(),
                    e
                );
            }
        }
    }

    // Compile generated GPU code if --gpu-compile is specified
    if compile {
        match target {
            GpuTarget::Cuda => {
                compile_cuda(&out_path, emit_host, verbose)?;
            }
            GpuTarget::Metal => {
                compile_metal(&out_path, verbose)?;
            }
            GpuTarget::OpenCL => {
                compile_opencl(&out_path, emit_host, verbose)?;
            }
            _ => {
                eprintln!(
                    "{} --gpu-compile is currently supported for CUDA, Metal, and OpenCL targets",
                    "warning:".yellow().bold()
                );
            }
        }
    }

    Ok(())
}

/// Compile CUDA .cu file with nvcc and link with gpu_runtime
pub(crate) fn compile_cuda(cu_path: &PathBuf, has_host: bool, verbose: bool) -> Result<(), String> {
    use std::process::Command;

    // Check if nvcc is available
    let nvcc_check = Command::new("nvcc").arg("--version").output();

    match nvcc_check {
        Err(_) => {
            return Err("nvcc not found. Please install the CUDA Toolkit:\n\
                 - Linux: https://developer.nvidia.com/cuda-downloads\n\
                 - macOS: CUDA is no longer supported on macOS (use Metal instead)\n\
                 - Set CUDA_PATH or add nvcc to PATH"
                .to_string());
        }
        Ok(output) if !output.status.success() => {
            return Err(
                "nvcc found but failed to run. Check CUDA Toolkit installation.".to_string(),
            );
        }
        Ok(output) => {
            if verbose {
                let version = String::from_utf8_lossy(&output.stdout);
                println!(
                    "{} {}",
                    "nvcc:".blue().bold(),
                    version.lines().last().unwrap_or("unknown")
                );
            }
        }
    }

    // Determine output binary name
    let binary_name = cu_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("gpu_output");
    let binary_path = cu_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join(binary_name);

    // Find gpu_runtime.c
    let std_dir = find_std_dir();
    let runtime_path = std_dir.as_ref().map(|d| d.join("gpu_runtime.c"));

    // Build nvcc command
    let mut cmd = Command::new("nvcc");

    // Add the .cu source file
    cmd.arg(cu_path);

    // Add host code if generated
    if has_host {
        let host_path = cu_path.with_extension("host.cu");
        if host_path.exists() {
            cmd.arg(&host_path);
            if verbose {
                println!(
                    "{} Including host code: {}",
                    "info:".blue().bold(),
                    host_path.display()
                );
            }
        }
    }

    // Add gpu_runtime.c if found
    if let Some(ref rt_path) = runtime_path {
        if rt_path.exists() {
            cmd.arg(rt_path);
            if verbose {
                println!(
                    "{} Linking gpu_runtime: {}",
                    "info:".blue().bold(),
                    rt_path.display()
                );
            }
        } else if verbose {
            println!(
                "{} gpu_runtime.c not found at {}",
                "warning:".yellow().bold(),
                rt_path.display()
            );
        }
    }

    // Output binary
    cmd.arg("-o").arg(&binary_path);

    // Standard flags
    cmd.arg("-lcudart");

    if verbose {
        println!(
            "{} Running: nvcc {} -o {}",
            "info:".blue().bold(),
            cu_path.display(),
            binary_path.display()
        );
    }

    // Execute nvcc
    let result = cmd
        .output()
        .map_err(|e| format!("Failed to execute nvcc: {}", e))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!(
            "nvcc compilation failed:\n{}{}",
            stderr,
            if stderr.contains("No CUDA capable device")
                || stderr.contains("no CUDA-capable device")
            {
                "\n\nHint: No CUDA GPU detected. Ensure NVIDIA drivers are installed."
            } else if stderr.contains("unsupported gpu architecture") {
                "\n\nHint: Try specifying a GPU architecture, e.g., --gpu-arch sm_70"
            } else {
                ""
            }
        ));
    }

    let stdout = String::from_utf8_lossy(&result.stdout);
    if !stdout.is_empty() && verbose {
        println!("{}", stdout);
    }

    println!(
        "{} Compiled GPU binary: {}",
        "✓".green().bold(),
        binary_path.display()
    );
    Ok(())
}

/// Compile Metal .metal file to .metallib using xcrun
pub(crate) fn compile_metal(metal_path: &PathBuf, verbose: bool) -> Result<(), String> {
    use std::process::Command;

    // Check if xcrun metal compiler is available
    let xcrun_check = Command::new("xcrun").args(["--find", "metal"]).output();

    match xcrun_check {
        Err(_) => {
            return Err(
                "xcrun not found. Please install Xcode Command Line Tools:\n\
                 xcode-select --install"
                    .to_string(),
            );
        }
        Ok(output) if !output.status.success() => {
            return Err(
                "Metal compiler not found via xcrun. Ensure Xcode is installed with Metal support."
                    .to_string(),
            );
        }
        Ok(_) => {
            if verbose {
                println!("{} Metal compiler found via xcrun", "info:".blue().bold());
            }
        }
    }

    // Step 1: Compile .metal → .air (Apple Intermediate Representation)
    let air_path = metal_path.with_extension("air");
    if verbose {
        println!(
            "{} Compiling {} → {}",
            "info:".blue().bold(),
            metal_path.display(),
            air_path.display()
        );
    }

    let air_result = Command::new("xcrun")
        .args(["metal", "-c"])
        .arg(metal_path)
        .arg("-o")
        .arg(&air_path)
        .output()
        .map_err(|e| format!("Failed to execute xcrun metal: {}", e))?;

    if !air_result.status.success() {
        let stderr = String::from_utf8_lossy(&air_result.stderr);
        return Err(format!("Metal compilation failed:\n{}", stderr));
    }

    // Step 2: Link .air → .metallib
    let metallib_path = metal_path.with_extension("metallib");
    if verbose {
        println!(
            "{} Linking {} → {}",
            "info:".blue().bold(),
            air_path.display(),
            metallib_path.display()
        );
    }

    let lib_result = Command::new("xcrun")
        .args(["metallib"])
        .arg(&air_path)
        .arg("-o")
        .arg(&metallib_path)
        .output()
        .map_err(|e| format!("Failed to execute xcrun metallib: {}", e))?;

    if !lib_result.status.success() {
        let stderr = String::from_utf8_lossy(&lib_result.stderr);
        return Err(format!("Metal library linking failed:\n{}", stderr));
    }

    // Clean up intermediate .air file
    let _ = std::fs::remove_file(&air_path);

    println!(
        "{} Compiled Metal library: {}",
        "✓".green().bold(),
        metallib_path.display()
    );

    // Step 3: Compile host code with metal_runtime if available
    let std_dir = find_std_dir();
    let runtime_path = std_dir.as_ref().map(|d| d.join("metal_runtime.m"));

    if let Some(ref rt_path) = runtime_path {
        if rt_path.exists() {
            let binary_name = metal_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("metal_output");
            let binary_path = metal_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join(binary_name);

            // Check for host code
            let host_path = metal_path.with_extension("host.swift");
            if host_path.exists() {
                if verbose {
                    println!(
                        "{} Host Swift code found: {}",
                        "info:".blue().bold(),
                        host_path.display()
                    );
                    println!(
                        "{} Note: Compile host code manually with:",
                        "info:".blue().bold()
                    );
                    println!(
                        "  swiftc {} -framework Metal -framework Foundation -o {}",
                        host_path.display(),
                        binary_path.display()
                    );
                }
            } else if verbose {
                println!(
                    "{} No host code found. Use --gpu-host to generate host code template.",
                    "info:".blue().bold()
                );
            }
        }
    }

    Ok(())
}

/// Compile OpenCL .cl file and link with opencl_runtime
pub(crate) fn compile_opencl(
    cl_path: &PathBuf,
    has_host: bool,
    verbose: bool,
) -> Result<(), String> {
    use std::process::Command;

    // Determine output binary name
    let binary_name = cl_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("opencl_output");
    let binary_path = cl_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join(binary_name);

    // Find opencl_runtime.c
    let std_dir = find_std_dir();
    let runtime_path = std_dir.as_ref().map(|d| d.join("opencl_runtime.c"));

    let rt_path = match runtime_path {
        Some(ref p) if p.exists() => p.clone(),
        _ => {
            return Err(
                "opencl_runtime.c not found. Ensure the std/ directory is accessible.".to_string(),
            );
        }
    };

    if verbose {
        println!(
            "{} Linking opencl_runtime: {}",
            "info:".blue().bold(),
            rt_path.display()
        );
    }

    // Build with cc (clang/gcc); on macOS and Windows, use clang directly
    let compiler = if cfg!(target_os = "linux") {
        "cc"
    } else {
        "clang"
    };

    // Check compiler availability
    let cc_check = Command::new(compiler).arg("--version").output();

    if cc_check.is_err() {
        return Err(format!(
            "{} not found. Please install a C compiler (clang or gcc).",
            compiler
        ));
    }

    let mut cmd = Command::new(compiler);

    // Add opencl_runtime.c
    cmd.arg(&rt_path);

    // Add host code if generated
    if has_host {
        let host_path = cl_path.with_extension("host.c");
        if host_path.exists() {
            cmd.arg(&host_path);
            if verbose {
                println!(
                    "{} Including host code: {}",
                    "info:".blue().bold(),
                    host_path.display()
                );
            }
        }
    }

    // Output binary
    cmd.arg("-o").arg(&binary_path);

    // OpenCL framework/library linking
    if cfg!(target_os = "macos") {
        cmd.arg("-framework").arg("OpenCL");
    } else {
        cmd.arg("-lOpenCL");
    }

    // Embed the .cl kernel source path as a define
    let cl_abs = std::fs::canonicalize(cl_path).unwrap_or_else(|_| cl_path.clone());
    cmd.arg(format!(
        "-DVAIS_OPENCL_KERNEL_PATH=\"{}\"",
        cl_abs.display()
    ));

    if verbose {
        println!(
            "{} Running: {} {} -o {}",
            "info:".blue().bold(),
            compiler,
            rt_path.display(),
            binary_path.display()
        );
    }

    // Execute compiler
    let result = cmd
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", compiler, e))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!(
            "OpenCL compilation failed:\n{}{}",
            stderr,
            if stderr.contains("opencl") || stderr.contains("OpenCL") || stderr.contains("CL/cl.h")
            {
                "\n\nHint: Ensure OpenCL SDK is installed.\n\
                 - macOS: OpenCL is built-in (no extra install needed)\n\
                 - Linux: Install ocl-icd-opencl-dev or vendor SDK\n\
                 - Windows: Install GPU vendor OpenCL SDK"
            } else {
                ""
            }
        ));
    }

    let stdout = String::from_utf8_lossy(&result.stdout);
    if !stdout.is_empty() && verbose {
        println!("{}", stdout);
    }

    println!(
        "{} Compiled OpenCL binary: {}",
        "✓".green().bold(),
        binary_path.display()
    );
    Ok(())
}

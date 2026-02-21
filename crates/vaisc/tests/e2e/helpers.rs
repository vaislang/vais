use std::fs;
use std::process::Command;
use tempfile::TempDir;
use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Compile Vais source to LLVM IR string
pub fn compile_to_ir(source: &str) -> Result<String, String> {
    let _tokens = tokenize(source).map_err(|e| format!("Lexer error: {:?}", e))?;
    let module = parse(source).map_err(|e| format!("Parser error: {:?}", e))?;
    let mut checker = TypeChecker::new();
    checker
        .check_module(&module)
        .map_err(|e| format!("Type error: {:?}", e))?;
    let mut gen = CodeGenerator::new("e2e_test");
    // Pass resolved function signatures for inferred parameter type support
    gen.set_resolved_functions(checker.get_all_functions().clone());
    gen.set_type_aliases(checker.get_type_aliases().clone());
    let instantiations = checker.get_generic_instantiations();
    let ir = if instantiations.is_empty() {
        gen.generate_module(&module)
    } else {
        gen.generate_module_with_instantiations(&module, &instantiations)
    }
    .map_err(|e| format!("Codegen error: {:?}", e))?;
    Ok(ir)
}

/// Result of running a compiled program
pub struct RunResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Compile source, build executable with clang, run it, return exit code + output
pub fn compile_and_run(source: &str) -> Result<RunResult, String> {
    compile_and_run_with_extra_sources(source, &[])
}

/// Compile source with additional C source files linked in
pub fn compile_and_run_with_extra_sources(
    source: &str,
    extra_c_sources: &[&str],
) -> Result<RunResult, String> {
    let ir = compile_to_ir(source)?;

    let tmp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let ll_path = tmp_dir.path().join("test.ll");
    let exe_name = if cfg!(target_os = "windows") {
        "test_exe.exe"
    } else {
        "test_exe"
    };
    let exe_path = tmp_dir.path().join(exe_name);

    fs::write(&ll_path, &ir).map_err(|e| format!("Failed to write IR: {}", e))?;

    // Compile LLVM IR to executable with clang
    let mut cmd = Command::new("clang");
    cmd.arg(&ll_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("-Wno-override-module");

    for c_source in extra_c_sources {
        cmd.arg(c_source);
    }

    let clang_output = cmd
        .output()
        .map_err(|e| format!("Failed to run clang: {}", e))?;

    if !clang_output.status.success() {
        let stderr = String::from_utf8_lossy(&clang_output.stderr);
        return Err(format!("clang compilation failed:\n{}", stderr));
    }

    // Run the executable
    let run_output = Command::new(&exe_path)
        .output()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    let exit_code = run_output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&run_output.stderr).to_string();

    Ok(RunResult {
        exit_code,
        stdout,
        stderr,
    })
}

/// Compile source with coverage instrumentation flags, run it, return result
pub fn compile_and_run_with_coverage(source: &str) -> Result<RunResult, String> {
    let ir = compile_to_ir(source)?;

    let tmp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let ll_path = tmp_dir.path().join("test.ll");
    let exe_name = if cfg!(target_os = "windows") {
        "test_exe.exe"
    } else {
        "test_exe"
    };
    let exe_path = tmp_dir.path().join(exe_name);
    let profraw_path = tmp_dir.path().join("default_%m.profraw");

    fs::write(&ll_path, &ir).map_err(|e| format!("Failed to write IR: {}", e))?;

    // Compile with coverage instrumentation flags
    let clang_output = Command::new("clang")
        .arg(&ll_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("-Wno-override-module")
        .arg("-fprofile-instr-generate")
        .arg("-fcoverage-mapping")
        .output()
        .map_err(|e| format!("Failed to run clang: {}", e))?;

    if !clang_output.status.success() {
        let stderr = String::from_utf8_lossy(&clang_output.stderr);
        return Err(format!("clang compilation failed:\n{}", stderr));
    }

    // Run with LLVM_PROFILE_FILE set
    let run_output = Command::new(&exe_path)
        .env("LLVM_PROFILE_FILE", &profraw_path)
        .output()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    let exit_code = run_output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&run_output.stderr).to_string();

    Ok(RunResult {
        exit_code,
        stdout,
        stderr,
    })
}

/// Assert that source compiles, runs, and returns the expected exit code
pub fn assert_exit_code(source: &str, expected: i32) {
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(
                result.exit_code, expected,
                "Expected exit code {}, got {}.\nstdout: {}\nstderr: {}",
                expected, result.exit_code, result.stdout, result.stderr
            );
        }
        Err(e) => panic!("Compilation/execution failed: {}", e),
    }
}

/// Assert that source compiles, runs, and stdout contains expected string
pub fn assert_stdout_contains(source: &str, expected: &str) {
    match compile_and_run(source) {
        Ok(result) => {
            assert!(
                result.stdout.contains(expected),
                "Expected stdout to contain {:?}, got {:?}.\nstderr: {}",
                expected,
                result.stdout,
                result.stderr
            );
        }
        Err(e) => panic!("Compilation/execution failed: {}", e),
    }
}

/// Assert that source compiles to IR successfully (doesn't require clang/execution)
pub fn assert_compiles(source: &str) {
    match compile_to_ir(source) {
        Ok(_) => {}
        Err(e) => panic!("Expected compilation to succeed, but got error: {}", e),
    }
}

/// Assert that source fails to compile (expected compilation error)
pub fn assert_compile_error(source: &str) {
    assert!(
        compile_to_ir(source).is_err(),
        "Expected compilation to fail, but it succeeded"
    );
}

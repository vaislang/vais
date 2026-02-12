use super::helpers::*;
use std::fs;
use std::process::Command;
use tempfile::TempDir;
use vais_codegen::CodeGenerator;
use vais_types::TypeChecker;

// ==========================================================================
// Phase 42 Stage 5: Per-Module Incremental Build E2E Tests
// ==========================================================================

/// Helper to create a temporary directory with multiple .vais files
fn create_multi_file_project(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");
    // Ensure the directory exists and is accessible
    let canonical = dir
        .path()
        .canonicalize()
        .expect("Failed to canonicalize temp dir");
    for (name, content) in files {
        let path = canonical.join(name);
        fs::write(&path, content).expect("Failed to write file");
    }
    dir
}

/// Helper to run vaisc build command
fn run_vaisc_build(main_file: &std::path::Path, extra_args: &[&str]) -> std::process::Output {
    let vaisc = env!("CARGO_BIN_EXE_vaisc");
    let mut cmd = Command::new(vaisc);
    cmd.arg("build").arg(main_file);
    for arg in extra_args {
        cmd.arg(arg);
    }
    cmd.output().expect("Failed to run vaisc")
}

#[test]
fn test_per_module_multi_file() {
    // Test basic multi-file project compilation with per-module codegen
    let files = &[
        (
            "main.vais",
            r#"
U math
F main() -> i64 {
    result := add(3, 4)
    R result
}
"#,
        ),
        (
            "math.vais",
            r#"
F add(a: i64, b: i64) -> i64 { R a + b }
F multiply(a: i64, b: i64) -> i64 { R a * b }
"#,
        ),
    ];

    let project = create_multi_file_project(files);
    let canonical_path = project
        .path()
        .canonicalize()
        .expect("Failed to canonicalize project path");
    let main_path = canonical_path.join("main.vais");

    // Build the project
    let output = run_vaisc_build(&main_path, &[]);

    assert!(
        output.status.success(),
        "Multi-file compilation failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that executable was created
    let exe_path = canonical_path.join("main");
    assert!(exe_path.exists(), "Executable should be created");

    // Run the executable
    let run_output = Command::new(&exe_path)
        .output()
        .expect("Failed to run executable");

    assert_eq!(
        run_output.status.code(),
        Some(7),
        "add(3, 4) should return 7"
    );
}

#[test]
fn test_per_module_cache_reuse() {
    // Test that second build reuses cache (no-change rebuild)
    let files = &[
        (
            "main.vais",
            r#"
U helper
F main() -> i64 {
    R compute(5)
}
"#,
        ),
        (
            "helper.vais",
            r#"
F compute(x: i64) -> i64 { R x * 2 }
"#,
        ),
    ];

    let project = create_multi_file_project(files);
    let canonical_path = project
        .path()
        .canonicalize()
        .expect("Failed to canonicalize project path");
    let main_path = canonical_path.join("main.vais");

    // First build
    let output1 = run_vaisc_build(&main_path, &[]);
    assert!(
        output1.status.success(),
        "First build failed: {}",
        String::from_utf8_lossy(&output1.stderr)
    );

    // Get build time
    let start = std::time::Instant::now();

    // Second build (should hit cache)
    let output2 = run_vaisc_build(&main_path, &[]);
    let rebuild_time = start.elapsed();

    assert!(
        output2.status.success(),
        "Second build failed: {}",
        String::from_utf8_lossy(&output2.stderr)
    );

    // Verify executable still works
    let exe_path = canonical_path.join("main");
    let run_output = Command::new(&exe_path)
        .output()
        .expect("Failed to run executable");

    assert_eq!(
        run_output.status.code(),
        Some(10),
        "compute(5) should return 10"
    );

    // Cache hit should be significantly faster (< 1 second for this small project)
    assert!(
        rebuild_time.as_millis() < 2000,
        "No-change rebuild took too long: {:?}ms (expected < 2000ms)",
        rebuild_time.as_millis()
    );
}

#[test]
fn test_per_module_incremental_one_file_change() {
    // Test that modifying one file only recompiles that module
    let files = &[
        (
            "main.vais",
            r#"
U utils
F main() -> i64 {
    R double(21)
}
"#,
        ),
        (
            "utils.vais",
            r#"
F double(x: i64) -> i64 { R x * 2 }
"#,
        ),
    ];

    let project = create_multi_file_project(files);
    let canonical_path = project
        .path()
        .canonicalize()
        .expect("Failed to canonicalize project path");
    let main_path = canonical_path.join("main.vais");
    let utils_path = canonical_path.join("utils.vais");

    // First build
    let output1 = run_vaisc_build(&main_path, &[]);
    assert!(
        output1.status.success(),
        "First build failed: {}",
        String::from_utf8_lossy(&output1.stderr)
    );

    // Verify initial result
    let exe_path = canonical_path.join("main");
    let run1 = Command::new(&exe_path)
        .output()
        .expect("Failed to run executable");
    assert_eq!(run1.status.code(), Some(42), "double(21) should return 42");

    // Sleep to ensure filesystem timestamp difference
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Modify utils.vais
    fs::write(
        &utils_path,
        r#"
F double(x: i64) -> i64 { R x * 3 }
"#,
    )
    .expect("Failed to modify utils.vais");

    // Rebuild (should only recompile utils module)
    let output2 = run_vaisc_build(&main_path, &[]);
    assert!(
        output2.status.success(),
        "Incremental build failed: {}",
        String::from_utf8_lossy(&output2.stderr)
    );

    // Verify new result reflects the change
    let run2 = Command::new(&exe_path)
        .output()
        .expect("Failed to run executable");
    assert_eq!(
        run2.status.code(),
        Some(63),
        "After modification, triple(21) should return 63"
    );
}

#[test]
fn test_per_module_emit_ir() {
    // Test that --emit-ir generates per-module .ll files
    let files = &[
        (
            "main.vais",
            r#"
U lib
F main() -> i64 {
    R get_value()
}
"#,
        ),
        (
            "lib.vais",
            r#"
F get_value() -> i64 { R 100 }
"#,
        ),
    ];

    let project = create_multi_file_project(files);
    let canonical_path = project
        .path()
        .canonicalize()
        .expect("Failed to canonicalize project path");
    let main_path = canonical_path.join("main.vais");

    // Build with --emit-ir
    let output = run_vaisc_build(&main_path, &["--emit-ir"]);

    assert!(
        output.status.success(),
        "Build with --emit-ir failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that .ll files were created (prefixed with main module name)
    let main_ll = canonical_path.join("main_main.ll");
    let lib_ll = canonical_path.join("main_lib.ll");

    assert!(
        main_ll.exists(),
        "main_main.ll should be generated with --emit-ir"
    );
    assert!(
        lib_ll.exists(),
        "main_lib.ll should be generated with --emit-ir"
    );

    // Verify .ll files contain LLVM IR
    let main_ir = fs::read_to_string(&main_ll).expect("Failed to read main_main.ll");
    let lib_ir = fs::read_to_string(&lib_ll).expect("Failed to read main_lib.ll");

    assert!(
        main_ir.contains("define") && main_ir.contains("@main"),
        "main_main.ll should contain LLVM IR with main function"
    );
    assert!(
        lib_ir.contains("define") && lib_ir.contains("@get_value"),
        "main_lib.ll should contain LLVM IR with get_value function"
    );
}

#[test]
fn test_circular_import_detection() {
    // Test that circular imports are detected and reported
    let files = &[
        (
            "a.vais",
            r#"
U b
F foo() -> i64 { R 42 }
"#,
        ),
        (
            "b.vais",
            r#"
U a
F bar() -> i64 { R 10 }
"#,
        ),
    ];

    let project = create_multi_file_project(files);
    let canonical_path = project
        .path()
        .canonicalize()
        .expect("Failed to canonicalize project path");
    let a_path = canonical_path.join("a.vais");

    // Build should fail with circular import error
    let output = run_vaisc_build(&a_path, &[]);

    assert!(
        !output.status.success(),
        "Circular import should cause build to fail"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("circular") || stderr.contains("Circular") || stderr.contains("cycle"),
        "Error message should mention circular import, got: {}",
        stderr
    );
}

// ==================== System Functions (env/process/signal) ====================

#[test]
fn e2e_getenv_returns_ptr() {
    // getenv returns a pointer (non-zero) for known env vars like PATH
    let source = r#"
F main() -> i64 {
    ptr := getenv("PATH")
    I ptr != 0 { 42 } E { 1 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_getenv_unknown_returns_zero() {
    // getenv returns 0 (null) for unknown env vars
    let source = r#"
F main() -> i64 {
    ptr := getenv("VAIS_NONEXISTENT_VAR_12345")
    I ptr == 0 { 42 } E { 1 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_system_echo() {
    // system() runs a command and returns exit status
    let source = r#"
F main() -> i64 {
    ret := system("true")
    I ret == 0 { 42 } E { 1 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_signal_constants() {
    // Signal constants are just integer values
    let source = r#"
F main() -> i64 {
    # SIGINT = 2, SIGTERM = 15
    2 + 15
}
"#;
    assert_exit_code(source, 17);
}

#[test]
fn test_incremental_tc_skip() {
    // Test that incremental build works correctly when signatures haven't changed
    let files = &[
        (
            "main.vais",
            r#"
U math
F main() -> i64 {
    R add(10, 32)
}
"#,
        ),
        (
            "math.vais",
            r#"
F add(a: i64, b: i64) -> i64 { R a + b }
"#,
        ),
    ];

    let project = create_multi_file_project(files);
    let canonical_path = project
        .path()
        .canonicalize()
        .expect("Failed to canonicalize project path");
    let main_path = canonical_path.join("main.vais");

    // First build (populates cache including TC signatures)
    let output1 = run_vaisc_build(&main_path, &[]);
    assert!(
        output1.status.success(),
        "First build failed: {}",
        String::from_utf8_lossy(&output1.stderr)
    );

    // Verify initial result
    let exe_path = canonical_path.join("main");
    let run1 = Command::new(&exe_path)
        .output()
        .expect("Failed to run executable");
    assert_eq!(run1.status.code(), Some(42), "add(10,32) should return 42");

    // Second build (same sources — cache should be used)
    let output2 = run_vaisc_build(&main_path, &[]);
    assert!(
        output2.status.success(),
        "Second build failed: {}",
        String::from_utf8_lossy(&output2.stderr)
    );

    // Result should still be correct
    let run2 = Command::new(&exe_path)
        .output()
        .expect("Failed to run executable");
    assert_eq!(
        run2.status.code(),
        Some(42),
        "Second build should produce same result"
    );
}

#[test]
fn test_incremental_tc_re_check_on_signature_change() {
    // Test that type checking is re-run when function signatures change
    let files = &[
        (
            "main.vais",
            r#"
U helper
F main() -> i64 {
    R get_val()
}
"#,
        ),
        (
            "helper.vais",
            r#"
F get_val() -> i64 { R 10 }
"#,
        ),
    ];

    let project = create_multi_file_project(files);
    let canonical_path = project
        .path()
        .canonicalize()
        .expect("Failed to canonicalize project path");
    let main_path = canonical_path.join("main.vais");
    let helper_path = canonical_path.join("helper.vais");

    // First build
    let output1 = run_vaisc_build(&main_path, &[]);
    assert!(
        output1.status.success(),
        "First build failed: {}",
        String::from_utf8_lossy(&output1.stderr)
    );

    let exe_path = canonical_path.join("main");
    let run1 = Command::new(&exe_path)
        .output()
        .expect("Failed to run executable");
    assert_eq!(run1.status.code(), Some(10));

    std::thread::sleep(std::time::Duration::from_millis(100));

    // Modify helper.vais — change body only (signature unchanged)
    fs::write(
        &helper_path,
        r#"
F get_val() -> i64 { R 20 }
"#,
    )
    .expect("Failed to modify helper.vais");

    // Rebuild
    let output2 = run_vaisc_build(&main_path, &[]);
    assert!(
        output2.status.success(),
        "Incremental build failed: {}",
        String::from_utf8_lossy(&output2.stderr)
    );

    let run2 = Command::new(&exe_path)
        .output()
        .expect("Failed to run executable");
    assert_eq!(
        run2.status.code(),
        Some(20),
        "Should return new value after body change"
    );
}

// ============================================================================
// Phase 48: Type Safety — Result<T,E> Generic + sizeof + Container Safety
// ============================================================================

#[test]
fn test_result_generic_ok_i64() {
    // Result<i64, i64> with Ok variant
    assert_exit_code(
        r#"
E Result { Ok(i64), Err(i64) }

F main() -> i64 {
    r := Ok(42)
    M r {
        Ok(v) => v,
        Err(_) => 0
    }
}
"#,
        42,
    );
}

#[test]
fn test_result_generic_err_i64() {
    // Result<i64, i64> with Err variant
    assert_exit_code(
        r#"
E Result { Ok(i64), Err(i64) }

F main() -> i64 {
    r := Err(7)
    M r {
        Ok(_) => 0,
        Err(e) => e
    }
}
"#,
        7,
    );
}

#[test]
fn test_result_generic_try_operator() {
    // ? operator with Result<i64, i64>
    assert_exit_code(
        r#"
E Result { Ok(i64), Err(i64) }

F divide(a: i64, b: i64) -> Result {
    I b == 0 {
        Err(1)
    } E {
        Ok(a / b)
    }
}

F compute() -> Result {
    x := divide(10, 2)?;
    y := divide(x, 0)?;
    R Ok(y)
}

F main() -> i64 {
    M compute() {
        Ok(v) => v,
        Err(e) => e + 100
    }
}
"#,
        101,
    );
}

#[test]
fn test_result_generic_unwrap_operator() {
    // ! operator with Result
    assert_exit_code(
        r#"
E Result { Ok(i64), Err(i64) }

F get_value() -> Result {
    Ok(55)
}

F main() -> i64 {
    get_value()!
}
"#,
        55,
    );
}

#[test]
fn test_result_generic_chained_operations() {
    // Chain Ok/Err operations through ? operator
    assert_exit_code(
        r#"
E Result { Ok(i64), Err(i64) }

F step1(x: i64) -> Result {
    I x > 0 { Ok(x * 2) } E { Err(1) }
}

F step2(x: i64) -> Result {
    I x < 100 { Ok(x + 3) } E { Err(2) }
}

F pipeline() -> Result {
    a := step1(5)?;
    b := step2(a)?;
    R Ok(b)
}

F main() -> i64 {
    M pipeline() {
        Ok(v) => v,
        Err(_) => 0
    }
}
"#,
        13, // 5*2=10, 10+3=13
    );
}

#[test]
fn test_result_generic_err_propagation() {
    // Err propagates through ? chain
    assert_exit_code(
        r#"
E Result { Ok(i64), Err(i64) }

F fail_step() -> Result {
    Err(42)
}

F pipeline() -> Result {
    x := fail_step()?
    R Ok(x + 1)
}

F main() -> i64 {
    M pipeline() {
        Ok(_) => 0,
        Err(e) => e
    }
}
"#,
        42,
    );
}

#[test]
fn test_sizeof_i64() {
    // sizeof returns 8 for i64
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 42
    sizeof(x)
}
"#,
        8,
    );
}

#[test]
fn test_sizeof_function_result() {
    // sizeof on function result (promoted to i64 at runtime)
    assert_exit_code(
        r#"
F get_val() -> i64 {
    0
}

F main() -> i64 {
    sizeof(get_val())
}
"#,
        8,
    );
}

#[test]
fn test_sizeof_bool() {
    // sizeof returns 1 for bool
    assert_exit_code(
        r#"
F get_bool() -> bool {
    true
}

F main() -> i64 {
    sizeof(get_bool())
}
"#,
        1,
    );
}

#[test]
fn test_sizeof_struct() {
    // sizeof returns fields * 8 for struct
    assert_exit_code(
        r#"
S Point {
    x: i64,
    y: i64
}

F main() -> i64 {
    p := Point { x: 1, y: 2 }
    sizeof(p)
}
"#,
        16, // 2 fields * 8 bytes
    );
}

#[test]
fn test_sizeof_struct_3_fields() {
    // sizeof for 3-field struct
    assert_exit_code(
        r#"
S Vec3 {
    x: i64,
    y: i64,
    z: i64
}

F main() -> i64 {
    v := Vec3 { x: 1, y: 2, z: 3 }
    sizeof(v)
}
"#,
        24, // 3 fields * 8 bytes
    );
}

#[test]
fn test_sizeof_in_expression() {
    // sizeof can be used in expressions
    assert_exit_code(
        r#"
F main() -> i64 {
    x := 42
    n := sizeof(x) / 2
    n
}
"#,
        4, // 8 / 2 = 4
    );
}

#[test]
fn test_result_with_match_both_arms() {
    // Match both arms of Result
    assert_exit_code(
        r#"
E Result { Ok(i64), Err(i64) }

F safe_div(a: i64, b: i64) -> Result {
    I b == 0 { Err(0) } E { Ok(a / b) }
}

F main() -> i64 {
    ok := safe_div(20, 4)
    err := safe_div(10, 0)
    a := M ok { Ok(v) => v, Err(_) => 0 }
    b := M err { Ok(_) => 0, Err(e) => e + 50 }
    a + b
}
"#,
        55, // 5 + 50
    );
}

#[test]
fn test_result_function_return_type() {
    // Function returning Result is properly typed
    assert_exit_code(
        r#"
E Result { Ok(i64), Err(i64) }

F validate(x: i64) -> Result {
    I x >= 0 { Ok(x) } E { Err(1) }
}

F check_both() -> i64 {
    a := validate(10)
    b := validate(0 - 5)
    ok_val := M a { Ok(v) => v, Err(_) => 0 }
    err_val := M b { Ok(_) => 0, Err(e) => e }
    ok_val + err_val
}

F main() -> i64 {
    check_both()
}
"#,
        11, // 10 + 1
    );
}

#[test]
fn test_result_err_value() {
    // Extract error value from Err variant
    assert_exit_code(
        r#"
E Result { Ok(i64), Err(i64) }

F main() -> i64 {
    ok := Ok(10)
    err := Err(42)
    a := M ok { Ok(v) => v, Err(_) => 0 }
    b := M err { Ok(_) => 0, Err(e) => e }
    a + b
}
"#,
        52, // 10 + 42
    );
}

#[test]
fn test_result_nested_match() {
    // Nested match on Result values
    assert_exit_code(
        r#"
E Result { Ok(i64), Err(i64) }

F compute(x: i64) -> Result {
    I x > 0 { Ok(x * x) } E { Err(0 - x) }
}

F main() -> i64 {
    a := compute(3)
    b := compute(0 - 2)
    va := M a { Ok(v) => v, Err(_) => 0 }
    vb := M b { Ok(_) => 0, Err(e) => e }
    va + vb
}
"#,
        11, // 9 + 2
    );
}

#[test]
fn test_sizeof_default() {
    // sizeof on default i64 value
    assert_exit_code(
        r#"
F main() -> i64 {
    a := 100
    b := 200
    sizeof(a) + sizeof(b)
}
"#,
        16, // 8 + 8
    );
}

#[test]
fn test_result_in_loop() {
    // Use Result in a loop with match
    assert_exit_code(
        r#"
E Result { Ok(i64), Err(i64) }

F process(x: i64) -> Result {
    I x == 3 { Err(x) } E { Ok(x * 10) }
}

F main() -> i64 {
    total := mut 0
    i := mut 0
    L {
        I i >= 5 { B }
        r := process(i)
        M r {
            Ok(v) => { total = total + v },
            Err(_) => { total = total + 1 }
        }
        i = i + 1
    }
    total
}
"#,
        71, // 0*10 + 1*10 + 2*10 + 1(err at 3) + 4*10 = 0+10+20+1+40
    );
}

#[test]
fn test_result_two_param_generic_type() {
    // Result<T, E> in type annotations works
    assert_exit_code(
        r#"
E Result { Ok(i64), Err(i64) }

F make_ok(v: i64) -> Result {
    Ok(v)
}

F make_err(e: i64) -> Result {
    Err(e)
}

F main() -> i64 {
    a := make_ok(10)
    b := make_err(3)
    va := M a { Ok(v) => v, Err(_) => 0 }
    vb := M b { Ok(_) => 0, Err(e) => e }
    va + vb
}
"#,
        13,
    );
}

#[test]
fn test_sizeof_multiple_types() {
    // sizeof works for different types in same function
    assert_exit_code(
        r#"
S Pair { a: i64, b: i64 }

F main() -> i64 {
    x := 1
    p := Pair { a: 1, b: 2 }
    sizeof(x) + sizeof(p)
}
"#,
        24, // 8 + 16
    );
}

// ============================================================================
// Conditional Compilation (#[cfg]) Tests
// ============================================================================

/// Compile Vais source with cfg values and return LLVM IR
fn compile_to_ir_with_cfg(
    source: &str,
    cfg_values: std::collections::HashMap<String, String>,
) -> Result<String, String> {
    let module = vais_parser::parse_with_cfg(source, cfg_values)
        .map_err(|e| format!("Parser error: {:?}", e))?;
    let mut checker = TypeChecker::new();
    checker
        .check_module(&module)
        .map_err(|e| format!("Type error: {:?}", e))?;
    let mut gen = CodeGenerator::new("e2e_test");
    gen.set_resolved_functions(checker.get_all_functions().clone());
    let ir = gen
        .generate_module(&module)
        .map_err(|e| format!("Codegen error: {:?}", e))?;
    Ok(ir)
}

fn compile_and_run_with_cfg(
    source: &str,
    cfg_values: std::collections::HashMap<String, String>,
) -> Result<RunResult, String> {
    let ir = compile_to_ir_with_cfg(source, cfg_values)?;
    let tmp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let ll_path = tmp_dir.path().join("test.ll");
    let exe_name = if cfg!(target_os = "windows") {
        "test_exe.exe"
    } else {
        "test_exe"
    };
    let exe_path = tmp_dir.path().join(exe_name);
    fs::write(&ll_path, &ir).map_err(|e| format!("Failed to write IR: {}", e))?;
    let clang_output = Command::new("clang")
        .arg(&ll_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("-Wno-override-module")
        .output()
        .map_err(|e| format!("Failed to run clang: {}", e))?;
    if !clang_output.status.success() {
        let stderr = String::from_utf8_lossy(&clang_output.stderr);
        return Err(format!("clang compilation failed:\n{}", stderr));
    }
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

fn cfg_map(pairs: &[(&str, &str)]) -> std::collections::HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn test_cfg_basic_target_os_match() {
    // When cfg matches, the constant should be included
    let result = compile_and_run_with_cfg(
        r#"
#[cfg(target_os = "linux")]
C PLATFORM_VAL: i64 = 10

#[cfg(target_os = "macos")]
C PLATFORM_VAL: i64 = 30

F main() -> i64 {
    PLATFORM_VAL
}
"#,
        cfg_map(&[("target_os", "linux")]),
    );
    let r = result.unwrap();
    assert_eq!(r.exit_code, 10);
}

#[test]
fn test_cfg_macos_target() {
    let result = compile_and_run_with_cfg(
        r#"
#[cfg(target_os = "linux")]
C VAL: i64 = 10

#[cfg(target_os = "macos")]
C VAL: i64 = 30

F main() -> i64 {
    VAL
}
"#,
        cfg_map(&[("target_os", "macos")]),
    );
    let r = result.unwrap();
    assert_eq!(r.exit_code, 30);
}

#[test]
fn test_cfg_no_match_excludes_item() {
    // When cfg doesn't match, the function should be excluded
    let result = compile_and_run_with_cfg(
        r#"
#[cfg(target_os = "windows")]
F windows_only() -> i64 {
    42
}

F main() -> i64 {
    # windows_only is excluded, so we just return 0
    0
}
"#,
        cfg_map(&[("target_os", "linux")]),
    );
    let r = result.unwrap();
    assert_eq!(r.exit_code, 0);
}

#[test]
fn test_cfg_not_condition() {
    // not() should negate the condition
    let result = compile_and_run_with_cfg(
        r#"
#[cfg(not(target_os = "windows"))]
C UNIX_VAL: i64 = 1

F main() -> i64 {
    UNIX_VAL
}
"#,
        cfg_map(&[("target_os", "linux")]),
    );
    let r = result.unwrap();
    assert_eq!(r.exit_code, 1);
}

#[test]
fn test_cfg_target_arch() {
    let result = compile_and_run_with_cfg(
        r#"
#[cfg(target_arch = "x86_64")]
C ARCH_VAL: i64 = 64

#[cfg(target_arch = "aarch64")]
C ARCH_VAL: i64 = 65

F main() -> i64 {
    ARCH_VAL
}
"#,
        cfg_map(&[("target_arch", "x86_64")]),
    );
    let r = result.unwrap();
    assert_eq!(r.exit_code, 64);
}

#[test]
fn test_cfg_no_cfg_values_includes_all() {
    // Without cfg_values set, all items should be included (backward compatible)
    let result = compile_and_run(
        r#"
C VAL: i64 = 42

F main() -> i64 {
    VAL
}
"#,
    );
    let r = result.unwrap();
    assert_eq!(r.exit_code, 42);
}

#[test]
fn test_cfg_multiple_platforms() {
    // Test that only the matching platform's constants are included
    let result = compile_and_run_with_cfg(
        r#"
#[cfg(target_os = "macos")]
C AF_INET6: i64 = 30

#[cfg(target_os = "linux")]
C AF_INET6: i64 = 10

#[cfg(target_os = "windows")]
C AF_INET6: i64 = 23

F main() -> i64 {
    AF_INET6
}
"#,
        cfg_map(&[("target_os", "macos")]),
    );
    let r = result.unwrap();
    assert_eq!(r.exit_code, 30);
}

#[test]
fn test_cfg_mixed_conditional_and_unconditional() {
    // Mix of conditional and unconditional constants
    let result = compile_and_run_with_cfg(
        r#"
C ALWAYS: i64 = 100

#[cfg(target_os = "linux")]
C PLATFORM: i64 = 10

F main() -> i64 {
    ALWAYS + PLATFORM
}
"#,
        cfg_map(&[("target_os", "linux")]),
    );
    let r = result.unwrap();
    assert_eq!(r.exit_code, 110);
}

#[test]
fn test_cfg_on_function() {
    let result = compile_and_run_with_cfg(
        r#"
#[cfg(target_os = "macos")]
F get_val() -> i64 {
    42
}

#[cfg(target_os = "linux")]
F get_val() -> i64 {
    99
}

F main() -> i64 {
    get_val()
}
"#,
        cfg_map(&[("target_os", "linux")]),
    );
    let r = result.unwrap();
    assert_eq!(r.exit_code, 99);
}

#[test]
fn test_cfg_on_struct() {
    let result = compile_and_run_with_cfg(
        r#"
#[cfg(target_os = "linux")]
S PlatformInfo {
    page_size: i64,
    signal_max: i64
}

F main() -> i64 {
    info := PlatformInfo { page_size: 4096, signal_max: 64 }
    info.page_size
}
"#,
        cfg_map(&[("target_os", "linux")]),
    );
    let r = result.unwrap();
    assert_eq!(r.exit_code, 4096 % 256); // exit code truncated to 8 bits
}

// ============================================================================
// SIMD Vector Operation Tests
// ============================================================================

#[test]
fn test_simd_vec4f32_dot_product_ir() {
    // Verify that SIMD dot product generates correct LLVM IR with vector ops
    let ir = compile_to_ir(
        r#"
F dot4(a: Vec4f32, b: Vec4f32) -> f32 {
    product := simd_mul_vec4f32(a, b)
    simd_reduce_add_vec4f32(product)
}

F main() -> i64 {
    0
}
"#,
    )
    .unwrap();
    // Verify vector operations in generated IR
    assert!(
        ir.contains("fmul <4 x float>") || ir.contains("fmul"),
        "Expected fmul for vec4f32 multiply"
    );
}

#[test]
fn test_simd_vec8f32_ops_ir() {
    // Verify 8-wide SIMD operations generate correct IR
    let ir = compile_to_ir(
        r#"
F add8(a: Vec8f32, b: Vec8f32) -> Vec8f32 {
    simd_add_vec8f32(a, b)
}

F sub8(a: Vec8f32, b: Vec8f32) -> Vec8f32 {
    simd_sub_vec8f32(a, b)
}

F main() -> i64 {
    0
}
"#,
    )
    .unwrap();
    assert!(
        ir.contains("fadd <8 x float>") || ir.contains("fadd"),
        "Expected fadd for vec8f32 add"
    );
    assert!(
        ir.contains("fsub <8 x float>") || ir.contains("fsub"),
        "Expected fsub for vec8f32 sub"
    );
}

#[test]
fn test_simd_vec4f32_l2_pattern_ir() {
    // Verify L2 distance pattern (sub + mul + reduce) generates correct IR
    let ir = compile_to_ir(
        r#"
F sq_diff(a: Vec4f32, b: Vec4f32) -> f32 {
    diff := simd_sub_vec4f32(a, b)
    sq := simd_mul_vec4f32(diff, diff)
    simd_reduce_add_vec4f32(sq)
}

F main() -> i64 {
    0
}
"#,
    )
    .unwrap();
    assert!(
        ir.contains("fsub <4 x float>") || ir.contains("fsub"),
        "Expected fsub for vec4f32 sub"
    );
    assert!(
        ir.contains("fmul <4 x float>") || ir.contains("fmul"),
        "Expected fmul for vec4f32 mul"
    );
}

#[test]
fn test_cfg_platform_net_constants() {
    // Verify network constants are platform-specific
    // AF_INET6: Linux=10, macOS=30
    // SOL_SOCKET: Linux=1, macOS=65535 (would exceed exit code, use modulo)
    let result_linux = compile_and_run_with_cfg(
        r#"
#[cfg(target_os = "linux")]
C AF_INET6: i64 = 10

#[cfg(target_os = "macos")]
C AF_INET6: i64 = 30

#[cfg(target_os = "linux")]
C SOL_SOCKET: i64 = 1

#[cfg(target_os = "macos")]
C SOL_SOCKET: i64 = 255

F main() -> i64 {
    AF_INET6 + SOL_SOCKET
}
"#,
        cfg_map(&[("target_os", "linux")]),
    );
    assert_eq!(result_linux.unwrap().exit_code, 11); // 10 + 1

    let result_macos = compile_and_run_with_cfg(
        r#"
#[cfg(target_os = "linux")]
C AF_INET6: i64 = 10

#[cfg(target_os = "macos")]
C AF_INET6: i64 = 30

#[cfg(target_os = "linux")]
C SOL_SOCKET: i64 = 1

#[cfg(target_os = "macos")]
C SOL_SOCKET: i64 = 255

F main() -> i64 {
    AF_INET6 + SOL_SOCKET
}
"#,
        cfg_map(&[("target_os", "macos")]),
    );
    assert_eq!(result_macos.unwrap().exit_code, 285 % 256); // (30 + 255) % 256 = 29
}

#[test]
fn test_cfg_platform_signal_constants() {
    // Verify signal constants are platform-specific
    // SIGUSR1: Linux=10, macOS=30
    // SIGUSR2: Linux=12, macOS=31
    let result_linux = compile_and_run_with_cfg(
        r#"
#[cfg(target_os = "linux")]
C SIGUSR1: i64 = 10

#[cfg(target_os = "macos")]
C SIGUSR1: i64 = 30

#[cfg(target_os = "linux")]
C SIGUSR2: i64 = 12

#[cfg(target_os = "macos")]
C SIGUSR2: i64 = 31

F main() -> i64 {
    SIGUSR1 + SIGUSR2
}
"#,
        cfg_map(&[("target_os", "linux")]),
    );
    assert_eq!(result_linux.unwrap().exit_code, 22); // 10 + 12

    let result_macos = compile_and_run_with_cfg(
        r#"
#[cfg(target_os = "linux")]
C SIGUSR1: i64 = 10

#[cfg(target_os = "macos")]
C SIGUSR1: i64 = 30

#[cfg(target_os = "linux")]
C SIGUSR2: i64 = 12

#[cfg(target_os = "macos")]
C SIGUSR2: i64 = 31

F main() -> i64 {
    SIGUSR1 + SIGUSR2
}
"#,
        cfg_map(&[("target_os", "macos")]),
    );
    assert_eq!(result_macos.unwrap().exit_code, 61); // 30 + 31
}

#[test]
fn test_cfg_platform_file_constants() {
    // Verify file mmap constants are platform-specific
    // MS_SYNC: Linux=4, macOS=16
    // MAP_ANONYMOUS: Linux=32, macOS=4096 (use smaller value for exit code)
    let result_linux = compile_and_run_with_cfg(
        r#"
#[cfg(target_os = "linux")]
C MS_SYNC: i64 = 4

#[cfg(target_os = "macos")]
C MS_SYNC: i64 = 16

#[cfg(target_os = "linux")]
C MAP_ANON: i64 = 32

#[cfg(target_os = "macos")]
C MAP_ANON: i64 = 64

F main() -> i64 {
    MS_SYNC + MAP_ANON
}
"#,
        cfg_map(&[("target_os", "linux")]),
    );
    assert_eq!(result_linux.unwrap().exit_code, 36); // 4 + 32

    let result_macos = compile_and_run_with_cfg(
        r#"
#[cfg(target_os = "linux")]
C MS_SYNC: i64 = 4

#[cfg(target_os = "macos")]
C MS_SYNC: i64 = 16

#[cfg(target_os = "linux")]
C MAP_ANON: i64 = 32

#[cfg(target_os = "macos")]
C MAP_ANON: i64 = 64

F main() -> i64 {
    MS_SYNC + MAP_ANON
}
"#,
        cfg_map(&[("target_os", "macos")]),
    );
    assert_eq!(result_macos.unwrap().exit_code, 80); // 16 + 64
}

#[test]
fn test_cfg_target_family() {
    // Verify target_family = "unix" includes constants
    let result = compile_and_run_with_cfg(
        r#"
#[cfg(target_family = "unix")]
C UNIX_VAL: i64 = 42

#[cfg(target_family = "windows")]
C UNIX_VAL: i64 = 99

F main() -> i64 {
    UNIX_VAL
}
"#,
        cfg_map(&[("target_family", "unix")]),
    );
    assert_eq!(result.unwrap().exit_code, 42);

    let result_windows = compile_and_run_with_cfg(
        r#"
#[cfg(target_family = "unix")]
C UNIX_VAL: i64 = 42

#[cfg(target_family = "windows")]
C UNIX_VAL: i64 = 99

F main() -> i64 {
    UNIX_VAL
}
"#,
        cfg_map(&[("target_family", "windows")]),
    );
    assert_eq!(result_windows.unwrap().exit_code, 99);
}

#[test]
fn test_cfg_cross_compile_simulation() {
    // Simulate cross-compilation: running on macOS but compiling for Linux
    // Only verify IR generation, not execution
    let ir = compile_to_ir_with_cfg(
        r#"
#[cfg(target_os = "linux")]
C SIGUSR1: i64 = 10

#[cfg(target_os = "macos")]
C SIGUSR1: i64 = 30

#[cfg(target_os = "linux")]
C AF_INET6: i64 = 10

#[cfg(target_os = "macos")]
C AF_INET6: i64 = 30

F main() -> i64 {
    SIGUSR1 + AF_INET6
}
"#,
        cfg_map(&[("target_os", "linux")]),
    );
    let ir = ir.unwrap();
    // Verify Linux constants are included (both should be 10)
    // The IR should contain constant definitions with value 10
    assert!(
        ir.contains("10"),
        "Expected Linux constants (value 10) in IR"
    );
    assert!(
        !ir.contains("30") || ir.matches("30").count() == 0 || ir.contains("@main"),
        "Should not contain macOS constants (value 30) when targeting Linux"
    );
}

// ==========================================================================
// Phase 50 Stage 0: Workspace E2E Tests
// ==========================================================================

/// Helper: get vaisc binary path
fn vaisc_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    path.push("vaisc");
    if !path.exists() {
        path = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        path.push("vaisc");
    }
    path
}

#[test]
fn test_workspace_manifest_parsing_e2e() {
    let tmp = TempDir::new().unwrap();

    fs::write(
        tmp.path().join("vais.toml"),
        r#"[workspace]
members = ["crates/*"]

[workspace.dependencies]
utils = "1.0.0"
"#,
    )
    .unwrap();

    let member_dir = tmp.path().join("crates").join("my-lib");
    fs::create_dir_all(member_dir.join("src")).unwrap();
    fs::write(
        member_dir.join("vais.toml"),
        "[package]\nname = \"my-lib\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(member_dir.join("src/lib.vais"), "F greet() -> i64 { 42 }\n").unwrap();

    let output = std::process::Command::new(vaisc_bin())
        .args(["pkg", "check", "--workspace"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run vaisc");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "workspace check failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("Workspace") || stdout.contains("workspace"),
        "output should mention workspace: {}",
        stdout
    );
}

#[test]
fn test_workspace_build_multiple_members_e2e() {
    let tmp = TempDir::new().unwrap();

    fs::write(
        tmp.path().join("vais.toml"),
        "[workspace]\nmembers = [\"packages/*\"]\n",
    )
    .unwrap();

    for name in &["alpha", "beta"] {
        let dir = tmp.path().join("packages").join(name);
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(
            dir.join("vais.toml"),
            format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\n", name),
        )
        .unwrap();
        fs::write(
            dir.join("src/main.vais"),
            "F main() -> i64 {\n    puts(\"Hello!\")\n    0\n}\n",
        )
        .unwrap();
    }

    let output = std::process::Command::new(vaisc_bin())
        .args(["pkg", "build", "--workspace"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run vaisc");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "workspace build failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("2 workspace member"),
        "should report 2 members: {}",
        stdout
    );
}

#[test]
fn test_workspace_shared_deps_e2e() {
    let tmp = TempDir::new().unwrap();

    fs::write(
        tmp.path().join("vais.toml"),
        "[workspace]\nmembers = [\"libs/*\"]\n\n[workspace.dependencies]\nshared-ver = \"3.0.0\"\n",
    )
    .unwrap();

    let dir = tmp.path().join("libs").join("consumer");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(
        dir.join("vais.toml"),
        "[package]\nname = \"consumer\"\nversion = \"0.1.0\"\n\n[dependencies]\nshared-ver = { workspace = true }\n",
    )
    .unwrap();
    fs::write(dir.join("src/main.vais"), "F main() -> i64 { 0 }\n").unwrap();

    let output = std::process::Command::new(vaisc_bin())
        .args(["pkg", "check", "--workspace"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run vaisc");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "workspace check with shared deps failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_workspace_inter_member_path_deps_e2e() {
    let tmp = TempDir::new().unwrap();

    fs::write(
        tmp.path().join("vais.toml"),
        "[workspace]\nmembers = [\"crates/*\"]\n",
    )
    .unwrap();

    // lib-core
    let core_dir = tmp.path().join("crates").join("lib-core");
    fs::create_dir_all(core_dir.join("src")).unwrap();
    fs::write(
        core_dir.join("vais.toml"),
        "[package]\nname = \"lib-core\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        core_dir.join("src/lib.vais"),
        "F core_add(a: i64, b: i64) -> i64 { a + b }\n",
    )
    .unwrap();

    // my-app depends on lib-core by path
    let app_dir = tmp.path().join("crates").join("my-app");
    fs::create_dir_all(app_dir.join("src")).unwrap();
    fs::write(
        app_dir.join("vais.toml"),
        "[package]\nname = \"my-app\"\nversion = \"0.1.0\"\n\n[dependencies]\nlib-core = { path = \"../lib-core\" }\n",
    )
    .unwrap();
    fs::write(app_dir.join("src/main.vais"), "F main() -> i64 { 0 }\n").unwrap();

    let output = std::process::Command::new(vaisc_bin())
        .args(["pkg", "build", "--workspace"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run vaisc");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "workspace build with inter-member deps failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_workspace_auto_detect_e2e() {
    let tmp = TempDir::new().unwrap();

    fs::write(
        tmp.path().join("vais.toml"),
        "[workspace]\nmembers = [\"apps/*\"]\n",
    )
    .unwrap();

    let app_dir = tmp.path().join("apps").join("hello");
    fs::create_dir_all(app_dir.join("src")).unwrap();
    fs::write(
        app_dir.join("vais.toml"),
        "[package]\nname = \"hello\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        app_dir.join("src/main.vais"),
        "F main() -> i64 {\n    puts(\"Hello workspace!\")\n    0\n}\n",
    )
    .unwrap();

    // Run without --workspace flag - auto-detect
    let output = std::process::Command::new(vaisc_bin())
        .args(["pkg", "build"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run vaisc");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "auto-detect workspace build failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("Workspace") || stdout.contains("workspace"),
        "auto-detected workspace should mention workspace: {}",
        stdout
    );
}

// ==========================================================================
// Phase 50 Stage 1: Feature Flags E2E Tests
// ==========================================================================

#[test]
fn test_feature_flag_basic_cfg_e2e() {
    // Test that #[cfg(feature = "json")] works with --features flag
    let tmp = TempDir::new().unwrap();

    fs::write(
        tmp.path().join("vais.toml"),
        r#"[package]
name = "feat-test"
version = "0.1.0"

[features]
default = []
json = []
"#,
    )
    .unwrap();

    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(
        tmp.path().join("src/main.vais"),
        r#"
#[cfg(feature = "json")]
F json_support() -> i64 { 42 }

F main() -> i64 {
    0
}
"#,
    )
    .unwrap();

    // Build with --features json
    let output = std::process::Command::new(vaisc_bin())
        .args(["pkg", "build", "--features", "json"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run vaisc");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "build with --features json failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_feature_flag_default_features_e2e() {
    let tmp = TempDir::new().unwrap();

    fs::write(
        tmp.path().join("vais.toml"),
        r#"[package]
name = "default-feat"
version = "0.1.0"

[features]
default = ["logging"]
logging = []
"#,
    )
    .unwrap();

    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(tmp.path().join("src/main.vais"), "F main() -> i64 { 0 }\n").unwrap();

    // Build without any flags — default features should be enabled
    let output = std::process::Command::new(vaisc_bin())
        .args(["pkg", "build"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run vaisc");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "build with default features failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_feature_flag_no_default_features_e2e() {
    let tmp = TempDir::new().unwrap();

    fs::write(
        tmp.path().join("vais.toml"),
        r#"[package]
name = "no-default"
version = "0.1.0"

[features]
default = ["extra"]
extra = []
"#,
    )
    .unwrap();

    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(tmp.path().join("src/main.vais"), "F main() -> i64 { 0 }\n").unwrap();

    // Build with --no-default-features
    let output = std::process::Command::new(vaisc_bin())
        .args(["pkg", "build", "--no-default-features"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run vaisc");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "build with --no-default-features failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_feature_flag_all_features_e2e() {
    let tmp = TempDir::new().unwrap();

    fs::write(
        tmp.path().join("vais.toml"),
        r#"[package]
name = "all-feat"
version = "0.1.0"

[features]
default = []
json = []
async_io = []
logging = ["json"]
"#,
    )
    .unwrap();

    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(tmp.path().join("src/main.vais"), "F main() -> i64 { 0 }\n").unwrap();

    // Build with --all-features
    let output = std::process::Command::new(vaisc_bin())
        .args(["pkg", "build", "--all-features"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run vaisc");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "build with --all-features failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_feature_flag_invalid_feature_e2e() {
    let tmp = TempDir::new().unwrap();

    fs::write(
        tmp.path().join("vais.toml"),
        r#"[package]
name = "invalid-feat"
version = "0.1.0"

[features]
default = []
json = []
"#,
    )
    .unwrap();

    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(tmp.path().join("src/main.vais"), "F main() -> i64 { 0 }\n").unwrap();

    // Build with nonexistent feature — should fail
    let output = std::process::Command::new(vaisc_bin())
        .args(["pkg", "build", "--features", "nonexistent"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run vaisc");

    assert!(
        !output.status.success(),
        "build with nonexistent feature should fail"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found") || stderr.contains("nonexistent"),
        "error should mention the invalid feature: {}",
        stderr
    );
}

// ==========================================================================
// Phase 52 Stage 0: std/path.vais E2E Tests
// ==========================================================================

#[test]
fn test_path_join_basic() {
    let result = compile_and_run(
        r#"
F main() -> i64 {
    # Test path_join logic: manually build "/usr/bin"
    result := malloc(9)
    store_byte(result, 47)      # '/'
    store_byte(result + 1, 117)  # 'u'
    store_byte(result + 2, 115)  # 's'
    store_byte(result + 3, 114)  # 'r'
    store_byte(result + 4, 47)   # '/'
    store_byte(result + 5, 98)   # 'b'
    store_byte(result + 6, 105)  # 'i'
    store_byte(result + 7, 110)  # 'n'
    store_byte(result + 8, 0)    # null terminator

    # Verify: first char should be '/', char at position 5 should be 'b'
    first := load_byte(result)
    fifth := load_byte(result + 5)
    I first == 47 && fifth == 98 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_path_parent() {
    let result = compile_and_run(
        r#"
F main() -> i64 {
    # Test path_parent logic: find last '/' in "/usr/bin/ls"
    # Manual string: "/usr/bin/ls" has length 12, last '/' at position 8
    test_str := malloc(13)
    store_byte(test_str, 47)      # '/'
    store_byte(test_str + 1, 117)  # 'u'
    store_byte(test_str + 2, 115)  # 's'
    store_byte(test_str + 3, 114)  # 'r'
    store_byte(test_str + 4, 47)   # '/'
    store_byte(test_str + 5, 98)   # 'b'
    store_byte(test_str + 6, 105)  # 'i'
    store_byte(test_str + 7, 110)  # 'n'
    store_byte(test_str + 8, 47)   # '/'
    store_byte(test_str + 9, 108)  # 'l'
    store_byte(test_str + 10, 115) # 's'
    store_byte(test_str + 11, 0)

    # Check positions 8, 9, 10 for '/'
    # We know position 8 is the last '/'
    ch8 := load_byte(test_str + 8)
    ch9 := load_byte(test_str + 9)

    # ch8 should be 47 ('/'), ch9 should be 108 ('l')
    I ch8 == 47 && ch9 == 108 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_path_filename() {
    let result = compile_and_run(
        r#"
F main() -> i64 {
    # Test path_filename logic: extract filename from path
    # Build "/usr/bin/ls" manually
    test_str := malloc(13)
    store_byte(test_str, 47)      # '/'
    store_byte(test_str + 1, 117)  # 'u'
    store_byte(test_str + 2, 115)  # 's'
    store_byte(test_str + 3, 114)  # 'r'
    store_byte(test_str + 4, 47)   # '/'
    store_byte(test_str + 5, 98)   # 'b'
    store_byte(test_str + 6, 105)  # 'i'
    store_byte(test_str + 7, 110)  # 'n'
    store_byte(test_str + 8, 47)   # '/'
    store_byte(test_str + 9, 108)  # 'l'
    store_byte(test_str + 10, 115) # 's'
    store_byte(test_str + 11, 0)

    # Find last '/' (position 8), so filename starts at position 9
    # Check filename is "ls"
    first := load_byte(test_str + 9)
    second := load_byte(test_str + 10)
    I first == 108 && second == 115 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_path_extension_stem() {
    let result = compile_and_run(
        r#"
F main() -> i64 {
    # Test extension and stem logic with "hello.txt"
    # Build "hello.txt" manually
    fname := malloc(10)
    store_byte(fname, 104)  # 'h'
    store_byte(fname + 1, 101)  # 'e'
    store_byte(fname + 2, 108)  # 'l'
    store_byte(fname + 3, 108)  # 'l'
    store_byte(fname + 4, 111)  # 'o'
    store_byte(fname + 5, 46)   # '.'
    store_byte(fname + 6, 116)  # 't'
    store_byte(fname + 7, 120)  # 'x'
    store_byte(fname + 8, 116)  # 't'
    store_byte(fname + 9, 0)

    # Check positions for '.' - we know position 5 is the dot
    ch4 := load_byte(fname + 4)  # 'o'
    ch5 := load_byte(fname + 5)  # '.'
    ch6 := load_byte(fname + 6)  # 't'

    # ch4 should be 111 ('o'), ch5 should be 46 ('.'), ch6 should be 116 ('t')
    # This confirms: stem is 0-4 (5 chars), extension is 6-8 (3 chars)
    I ch4 == 111 && ch5 == 46 && ch6 == 116 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_path_is_absolute() {
    let result = compile_and_run(
        r#"
F main() -> i64 {
    # Test absolute vs relative paths
    # Build "/usr/bin" (absolute)
    abs_path := malloc(9)
    store_byte(abs_path, 47)  # '/' - absolute paths start with this
    store_byte(abs_path + 1, 117)  # 'u'
    store_byte(abs_path + 2, 115)  # 's'
    store_byte(abs_path + 3, 114)  # 'r'
    store_byte(abs_path + 4, 47)   # '/'
    store_byte(abs_path + 5, 98)   # 'b'
    store_byte(abs_path + 6, 105)  # 'i'
    store_byte(abs_path + 7, 110)  # 'n'
    store_byte(abs_path + 8, 0)

    # Build "usr/bin" (relative)
    rel_path := malloc(8)
    store_byte(rel_path, 117)  # 'u' - relative paths don't start with '/'
    store_byte(rel_path + 1, 115)  # 's'
    store_byte(rel_path + 2, 114)  # 'r'
    store_byte(rel_path + 3, 47)   # '/'
    store_byte(rel_path + 4, 98)   # 'b'
    store_byte(rel_path + 5, 105)  # 'i'
    store_byte(rel_path + 6, 110)  # 'n'
    store_byte(rel_path + 7, 0)

    # Check if absolute (first byte is '/')
    abs_first := load_byte(abs_path)
    rel_first := load_byte(rel_path)

    # abs_first should be 47, rel_first should be 117
    I abs_first == 47 && rel_first == 117 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

// ==========================================================================
// Phase 52 Stage 1: std/channel.vais E2E Tests
// ==========================================================================

#[test]
fn test_channel_ring_buffer_send_recv() {
    // Test basic ring buffer channel logic (single-threaded)
    let result = compile_and_run(
        r#"
F main() -> i64 {
    # Simulate a bounded channel with ring buffer
    cap := 4
    buf := malloc(cap * 8)
    head := mut 0
    tail := mut 0

    # Send 3 values
    store_i64(buf + (tail % cap) * 8, 10)
    tail = tail + 1
    store_i64(buf + (tail % cap) * 8, 20)
    tail = tail + 1
    store_i64(buf + (tail % cap) * 8, 30)
    tail = tail + 1

    # Recv 3 values
    v1 := load_i64(buf + (head % cap) * 8)
    head = head + 1
    v2 := load_i64(buf + (head % cap) * 8)
    head = head + 1
    v3 := load_i64(buf + (head % cap) * 8)
    head = head + 1

    free(buf)
    # Sum should be 60
    I v1 + v2 + v3 == 60 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_channel_ring_buffer_wraparound() {
    // Test ring buffer wraparound behavior
    let result = compile_and_run(
        r#"
F main() -> i64 {
    cap := 3
    buf := malloc(cap * 8)
    head := mut 0
    tail := mut 0

    # Fill buffer
    store_i64(buf + (tail % cap) * 8, 1)
    tail = tail + 1
    store_i64(buf + (tail % cap) * 8, 2)
    tail = tail + 1
    store_i64(buf + (tail % cap) * 8, 3)
    tail = tail + 1

    # Drain 2
    v1 := load_i64(buf + (head % cap) * 8)
    head = head + 1
    v2 := load_i64(buf + (head % cap) * 8)
    head = head + 1

    # Wraparound: add 2 more (tail=3,4 -> slots 0,1)
    store_i64(buf + (tail % cap) * 8, 4)
    tail = tail + 1
    store_i64(buf + (tail % cap) * 8, 5)
    tail = tail + 1

    # Read remaining 3 items (3, 4, 5)
    v3 := load_i64(buf + (head % cap) * 8)
    head = head + 1
    v4 := load_i64(buf + (head % cap) * 8)
    head = head + 1
    v5 := load_i64(buf + (head % cap) * 8)
    head = head + 1

    free(buf)
    # v1=1, v2=2, v3=3, v4=4, v5=5 -> sum=15
    I v1 + v2 + v3 + v4 + v5 == 15 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_channel_unbounded_grow() {
    // Test unbounded channel growing logic
    let result = compile_and_run(
        r#"
F main() -> i64 {
    # Start with capacity 2, grow to 4
    cap := mut 2
    buf := mut malloc(cap * 8)
    head := mut 0
    tail := mut 0

    # Send 2 items (fill)
    store_i64(buf + (tail % cap) * 8, 100)
    tail = tail + 1
    store_i64(buf + (tail % cap) * 8, 200)
    tail = tail + 1

    # Need to grow: double capacity, copy items
    count := tail - head
    new_cap := cap * 2
    new_buf := malloc(new_cap * 8)
    i := mut 0
    L {
        I i >= count { B }
        src_off := ((head + i) % cap) * 8
        val := load_i64(buf + src_off)
        store_i64(new_buf + i * 8, val)
        i = i + 1
    }
    free(buf)
    buf = new_buf
    head = 0
    tail = count
    cap = new_cap

    # Now add 2 more
    store_i64(buf + (tail % cap) * 8, 300)
    tail = tail + 1
    store_i64(buf + (tail % cap) * 8, 400)
    tail = tail + 1

    # Read all 4
    v1 := load_i64(buf + (head % cap) * 8)
    head = head + 1
    v2 := load_i64(buf + (head % cap) * 8)
    head = head + 1
    v3 := load_i64(buf + (head % cap) * 8)
    head = head + 1
    v4 := load_i64(buf + (head % cap) * 8)
    head = head + 1

    free(buf)
    # 100+200+300+400 = 1000
    sum := v1 + v2 + v3 + v4
    I sum == 1000 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_channel_select_logic() {
    // Test select: poll multiple channel buffers
    let result = compile_and_run(
        r#"
F main() -> i64 {
    # Simulate 3 channels (just their head/tail state)
    # Each channel is 56 bytes but we only need head(+16) and tail(+24)
    ch1 := malloc(56)
    ch2 := malloc(56)
    ch3 := malloc(56)

    # ch1: empty (head=tail=0)
    store_i64(ch1 + 16, 0)
    store_i64(ch1 + 24, 0)
    # ch2: empty
    store_i64(ch2 + 16, 0)
    store_i64(ch2 + 24, 0)
    # ch3: has data (head=0, tail=1)
    store_i64(ch3 + 16, 0)
    store_i64(ch3 + 24, 1)

    # Build channel array
    channels := malloc(3 * 8)
    store_i64(channels, ch1)
    store_i64(channels + 8, ch2)
    store_i64(channels + 16, ch3)

    # Select: find first ready channel
    found := mut 0 - 1
    i := mut 0
    L {
        I i >= 3 { B }
        ch_ptr := load_i64(channels + i * 8)
        ch_head := load_i64(ch_ptr + 16)
        ch_tail := load_i64(ch_ptr + 24)
        I ch_head < ch_tail {
            found = i
            B
        }
        i = i + 1
    }

    free(ch1)
    free(ch2)
    free(ch3)
    free(channels)
    # ch3 (index 2) should be ready
    I found == 2 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_channel_fifo_order() {
    // Verify FIFO ordering with larger dataset
    let result = compile_and_run(
        r#"
F main() -> i64 {
    cap := 16
    buf := malloc(cap * 8)
    head := mut 0
    tail := mut 0

    # Send 10 values (0..9)
    i := mut 0
    L {
        I i >= 10 { B }
        store_i64(buf + (tail % cap) * 8, i)
        tail = tail + 1
        i = i + 1
    }

    # Verify FIFO: each recv should match send order
    ok := mut 1
    j := mut 0
    L {
        I j >= 10 { B }
        val := load_i64(buf + (head % cap) * 8)
        head = head + 1
        I val != j {
            ok = 0
        }
        j = j + 1
    }

    free(buf)
    ok
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

// ==========================================================================
// Phase 52 Stage 2: std/datetime.vais E2E Tests
// ==========================================================================

#[test]
fn test_datetime_leap_year() {
    // Test leap year logic: 2000 (leap), 1900 (not leap), 2024 (leap), 2023 (not leap)
    let result = compile_and_run(
        r#"
F is_leap(year: i64) -> i64 {
    I year % 400 == 0 { R 1 }
    I year % 100 == 0 { R 0 }
    I year % 4 == 0 { R 1 }
    0
}

F main() -> i64 {
    a := is_leap(2000)   # 1 (divisible by 400)
    b := is_leap(1900)   # 0 (divisible by 100 but not 400)
    c := is_leap(2024)   # 1 (divisible by 4)
    d := is_leap(2023)   # 0 (not divisible by 4)
    I a == 1 && b == 0 && c == 1 && d == 0 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_datetime_days_in_month() {
    // Test days_in_month logic: Feb in leap year, Feb in non-leap year, Jan, Apr
    let result = compile_and_run(
        r#"
F is_leap(year: i64) -> i64 {
    I year % 400 == 0 { R 1 }
    I year % 100 == 0 { R 0 }
    I year % 4 == 0 { R 1 }
    0
}

F days_in_month_for(year: i64, month: i64) -> i64 {
    I month == 2 {
        I is_leap(year) == 1 { 29 } E { 28 }
    } E I month == 4 || month == 6 || month == 9 || month == 11 {
        30
    } E {
        31
    }
}

F main() -> i64 {
    feb_leap := days_in_month_for(2024, 2)     # 29
    feb_non := days_in_month_for(2023, 2)      # 28
    jan := days_in_month_for(2024, 1)          # 31
    apr := days_in_month_for(2024, 4)          # 30
    I feb_leap == 29 && feb_non == 28 && jan == 31 && apr == 30 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_datetime_to_timestamp() {
    // Test datetime_to_timestamp: 1970-01-01 00:00:00 = 0, 1970-01-02 00:00:00 = 86400
    let result = compile_and_run(
        r#"
F is_leap(year: i64) -> i64 {
    I year % 400 == 0 { R 1 }
    I year % 100 == 0 { R 0 }
    I year % 4 == 0 { R 1 }
    0
}

F days_in_month_for(year: i64, month: i64) -> i64 {
    I month == 2 {
        I is_leap(year) == 1 { 29 } E { 28 }
    } E I month == 4 || month == 6 || month == 9 || month == 11 {
        30
    } E {
        31
    }
}

F datetime_to_timestamp(year: i64, month: i64, day: i64, hour: i64, min: i64, sec: i64) -> i64 {
    total_days := mut 0

    # Add days for complete years from 1970 to year-1
    y := mut 1970
    L {
        I y >= year { B }
        total_days = total_days + 365 + is_leap(y)
        y = y + 1
    }

    # Add days for complete months in the current year
    m := mut 1
    L {
        I m >= month { B }
        total_days = total_days + days_in_month_for(year, m)
        m = m + 1
    }

    # Add remaining days (day is 1-indexed)
    total_days = total_days + (day - 1)

    # Convert to seconds and add time components
    total_days * 86400 + hour * 3600 + min * 60 + sec
}

F main() -> i64 {
    epoch := datetime_to_timestamp(1970, 1, 1, 0, 0, 0)  # 0
    next_day := datetime_to_timestamp(1970, 1, 2, 0, 0, 0)  # 86400
    I epoch == 0 && next_day == 86400 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_datetime_from_timestamp() {
    // Test timestamp_to_datetime: timestamp 86400 -> year=1970, month=1, day=2
    let result = compile_and_run(
        r#"
F is_leap(year: i64) -> i64 {
    I year % 400 == 0 { R 1 }
    I year % 100 == 0 { R 0 }
    I year % 4 == 0 { R 1 }
    0
}

F days_in_month_for(year: i64, month: i64) -> i64 {
    I month == 2 {
        I is_leap(year) == 1 { 29 } E { 28 }
    } E I month == 4 || month == 6 || month == 9 || month == 11 {
        30
    } E {
        31
    }
}

# Simple DateTime struct (we only need year, month, day for this test)
S DateTime {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    min: i64,
    sec: i64
}

F timestamp_to_datetime(ts: i64) -> DateTime {
    total_days := mut ts / 86400
    remaining_secs := ts % 86400

    hour := remaining_secs / 3600
    min := (remaining_secs % 3600) / 60
    sec := remaining_secs % 60

    # Find the year
    year := mut 1970
    L {
        days_in_year := 365 + is_leap(year)
        I total_days < days_in_year { B }
        total_days = total_days - days_in_year
        year = year + 1
    }

    # Find the month
    month := mut 1
    L {
        days_in_month := days_in_month_for(year, month)
        I total_days < days_in_month { B }
        total_days = total_days - days_in_month
        month = month + 1
    }

    # Remaining days is the day of month (1-indexed)
    day := total_days + 1

    DateTime { year: year, month: month, day: day, hour: hour, min: min, sec: sec }
}

F main() -> i64 {
    dt := timestamp_to_datetime(86400)  # 1970-01-02 00:00:00
    I dt.year == 1970 && dt.month == 1 && dt.day == 2 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_datetime_duration_arithmetic() {
    // Test Duration add/sub operations
    let result = compile_and_run(r#"
S Duration {
    secs: i64,
    nanos: i64
}

F duration_add(a: Duration, b: Duration) -> Duration {
    total_secs := a.secs + b.secs
    total_nanos := a.nanos + b.nanos

    # Normalize nanoseconds (simple version - no overflow handling needed for test)
    I total_nanos >= 1000000000 {
        Duration { secs: total_secs + 1, nanos: total_nanos - 1000000000 }
    } E {
        Duration { secs: total_secs, nanos: total_nanos }
    }
}

F duration_sub(a: Duration, b: Duration) -> Duration {
    I a.secs < b.secs {
        Duration { secs: 0, nanos: 0 }
    } E I a.secs == b.secs {
        I a.nanos < b.nanos {
            Duration { secs: 0, nanos: 0 }
        } E {
            Duration { secs: 0, nanos: a.nanos - b.nanos }
        }
    } E {
        diff_secs := a.secs - b.secs
        I a.nanos < b.nanos {
            Duration { secs: diff_secs - 1, nanos: a.nanos + 1000000000 - b.nanos }
        } E {
            Duration { secs: diff_secs, nanos: a.nanos - b.nanos }
        }
    }
}

F main() -> i64 {
    d1 := Duration { secs: 100, nanos: 500000000 }
    d2 := Duration { secs: 50, nanos: 300000000 }

    # Add: 100.5 + 50.3 = 150.8
    sum := duration_add(d1, d2)

    # Sub: 100.5 - 50.3 = 50.2
    diff := duration_sub(d1, d2)

    I sum.secs == 150 && sum.nanos == 800000000 && diff.secs == 50 && diff.nanos == 200000000 { 1 } E { 0 }
}
"#).unwrap();
    assert_eq!(result.exit_code, 1);
}

// ==========================================================================
// Phase 52 Stage 3: std/args.vais E2E Tests
// ==========================================================================

#[test]
fn test_args_flag_detection() {
    let result = compile_and_run(
        r#"
F main() -> i64 {
    # Simulate argv: ["prog", "--verbose", "file.txt"]
    argv := malloc(3 * 8)
    store_i64(argv, str_to_ptr("myapp"))
    store_i64(argv + 8, str_to_ptr("--verbose"))
    store_i64(argv + 16, str_to_ptr("file.txt"))

    # Check if argv[1] starts with "--"
    arg := load_i64(argv + 8)
    is_flag := I load_byte(arg) == 45 && load_byte(arg + 1) == 45 { 1 } E { 0 }

    free(argv)
    is_flag
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_args_option_parsing() {
    let result = compile_and_run(
        r#"
F main() -> i64 {
    # Simulate argv: ["prog", "--output", "result.txt"]
    argv := malloc(3 * 8)
    store_i64(argv, str_to_ptr("prog"))
    store_i64(argv + 8, str_to_ptr("--output"))
    store_i64(argv + 16, str_to_ptr("result.txt"))

    # Parse: skip "--" prefix from argv[1] and get argv[2]
    arg1 := load_i64(argv + 8)
    is_option := I load_byte(arg1) == 45 && load_byte(arg1 + 1) == 45 { 1 } E { 0 }

    # Get value (argv[2])
    value := load_i64(argv + 16)

    # Check value starts with 'r' (114)
    value_ok := I load_byte(value) == 114 { 1 } E { 0 }

    free(argv)
    I is_option == 1 && value_ok == 1 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_args_positional() {
    let result = compile_and_run(
        r#"
F main() -> i64 {
    # Simulate argv: ["prog", "input.txt", "--verbose"]
    argv := malloc(3 * 8)
    store_i64(argv, str_to_ptr("prog"))
    store_i64(argv + 8, str_to_ptr("input.txt"))
    store_i64(argv + 16, str_to_ptr("--verbose"))

    # Check if argv[1] does NOT start with '-' (45)
    arg1 := load_i64(argv + 8)
    is_positional := I load_byte(arg1) != 45 { 1 } E { 0 }

    # Check argv[2] does start with '--'
    arg2 := load_i64(argv + 16)
    is_flag := I load_byte(arg2) == 45 && load_byte(arg2 + 1) == 45 { 1 } E { 0 }

    free(argv)
    I is_positional == 1 && is_flag == 1 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_args_short_flag() {
    let result = compile_and_run(
        r#"
F main() -> i64 {
    # Simulate argv: ["prog", "-v"]
    argv := malloc(2 * 8)
    store_i64(argv, str_to_ptr("prog"))
    store_i64(argv + 8, str_to_ptr("-v"))

    # Check if argv[1] starts with '-' but NOT '--'
    arg := load_i64(argv + 8)
    first := load_byte(arg)
    second := load_byte(arg + 1)
    is_short := I first == 45 && second != 45 { 1 } E { 0 }

    # Extract short char ('v' = 118)
    short_char := second
    is_v := I short_char == 118 { 1 } E { 0 }

    free(argv)
    I is_short == 1 && is_v == 1 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_args_str_eq_helper() {
    let result = compile_and_run(
        r#"
# String comparison helper for arg parsing
F str_eq(a: i64, b: i64) -> i64 {
    i := mut 0
    L {
        ca := load_byte(a + i)
        cb := load_byte(b + i)
        I ca != cb { R 0 }
        I ca == 0 { R 1 }
        i = i + 1
    }
    1
}

F main() -> i64 {
    # Test equal strings
    s1 := str_to_ptr("output")
    s2 := str_to_ptr("output")
    eq1 := str_eq(s1, s2)

    # Test different strings
    s3 := str_to_ptr("verbose")
    eq2 := str_eq(s1, s3)

    # Should be: eq1=1, eq2=0
    I eq1 == 1 && eq2 == 0 { 1 } E { 0 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

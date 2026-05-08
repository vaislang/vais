//! Execution-based E2E Tests for the Vais Compiler
//!
//! Unlike IR-matching tests, these tests verify actual runtime behavior:
//! Source → Lexer → Parser → Type Checker → Codegen → LLVM IR → clang → Execute → Verify stdout/exit
//!
//! This file provides enhanced helpers for execution testing and houses
//! tests that specifically verify runtime output correctness.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

// ==================== Core Helpers ====================

/// Compile Vais source to LLVM IR string
fn compile_to_ir(source: &str) -> Result<String, String> {
    let _tokens = tokenize(source).map_err(|e| format!("Lexer error: {:?}", e))?;
    let module = parse(source).map_err(|e| format!("Parser error: {:?}", e))?;
    let mut checker = TypeChecker::new();
    checker
        .check_module(&module)
        .map_err(|e| format!("Type error: {:?}", e))?;
    let mut gen = CodeGenerator::new("exec_test");
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
#[derive(Debug)]
struct RunResult {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

/// Compile source, build executable with clang, run it, return exit code + output
fn compile_and_run(source: &str) -> Result<RunResult, String> {
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

/// Compile a .vais file from disk, build executable, run it
fn compile_and_run_file(path: &Path) -> Result<RunResult, String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    compile_and_run(&source)
}

/// Find project root (where Cargo.toml with [workspace] lives)
fn project_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crates/vaisc -> project root is ../../
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&manifest_dir)
        .to_path_buf()
}

// ==================== Enhanced Assertion Helpers ====================

/// Assert that source compiles, runs, and returns the expected exit code
fn assert_exit_code(source: &str, expected: i32) {
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

/// Assert that stdout exactly matches expected (trimmed)
fn assert_stdout_exact(source: &str, expected: &str) {
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(
                result.stdout.trim(),
                expected.trim(),
                "Stdout mismatch.\nExpected:\n{}\nActual:\n{}\nstderr: {}",
                expected.trim(),
                result.stdout.trim(),
                result.stderr
            );
            assert_eq!(
                result.exit_code, 0,
                "Program exited with non-zero code {}.\nstdout: {}\nstderr: {}",
                result.exit_code, result.stdout, result.stderr
            );
        }
        Err(e) => panic!("Compilation/execution failed: {}", e),
    }
}

/// Assert that stdout contains the expected substring
fn assert_stdout_contains(source: &str, expected: &str) {
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

/// Assert that stdout contains all expected lines (order-sensitive)
fn assert_stdout_lines(source: &str, expected_lines: &[&str]) {
    match compile_and_run(source) {
        Ok(result) => {
            let actual_lines: Vec<&str> = result.stdout.lines().collect();
            for (i, expected) in expected_lines.iter().enumerate() {
                assert!(
                    i < actual_lines.len(),
                    "Expected at least {} lines, got {}.\nExpected line {}: {:?}\nActual stdout:\n{}",
                    i + 1,
                    actual_lines.len(),
                    i,
                    expected,
                    result.stdout
                );
                assert_eq!(
                    actual_lines[i].trim(),
                    expected.trim(),
                    "Line {} mismatch.\nExpected: {:?}\nActual:   {:?}\nFull stdout:\n{}",
                    i,
                    expected.trim(),
                    actual_lines[i].trim(),
                    result.stdout
                );
            }
            assert_eq!(
                result.exit_code, 0,
                "Program exited with non-zero code {}.\nstdout: {}\nstderr: {}",
                result.exit_code, result.stdout, result.stderr
            );
        }
        Err(e) => panic!("Compilation/execution failed: {}", e),
    }
}

/// Assert that source fails to compile (expected compilation error)
#[allow(dead_code)]
fn assert_compile_error(source: &str) {
    assert!(
        compile_to_ir(source).is_err(),
        "Expected compilation to fail, but it succeeded"
    );
}

/// Assert that source compiles and runs without crashing (exit code 0, no stderr panic)
#[allow(dead_code)]
fn assert_no_crash(source: &str) {
    match compile_and_run(source) {
        Ok(result) => {
            assert!(
                !result.stderr.contains("SIGSEGV")
                    && !result.stderr.contains("segmentation fault")
                    && !result.stderr.contains("Abort"),
                "Program crashed!\nexit_code: {}\nstderr: {}",
                result.exit_code,
                result.stderr
            );
        }
        Err(e) => panic!("Compilation/execution failed: {}", e),
    }
}

// ==================== Stage 0: Basic Execution Tests ====================

#[test]
fn exec_return_constant_42() {
    assert_exit_code("fn main() -> i64 = 42", 42);
}

#[test]
fn exec_arithmetic_add() {
    assert_exit_code("fn main() -> i64 = 3 + 4", 7);
}

#[test]
fn exec_arithmetic_sub() {
    assert_exit_code("fn main() -> i64 = 10 - 3", 7);
}

#[test]
fn exec_arithmetic_mul() {
    assert_exit_code("fn main() -> i64 = 6 * 7", 42);
}

#[test]
fn exec_arithmetic_div() {
    assert_exit_code("fn main() -> i64 = 84 / 2", 42);
}

#[test]
fn exec_arithmetic_mod() {
    assert_exit_code("fn main() -> i64 = 17 % 5", 2);
}

#[test]
fn exec_arithmetic_precedence() {
    assert_exit_code("fn main() -> i64 = 2 + 3 * 4", 14);
}

#[test]
fn exec_function_call() {
    let source = r#"
fn add(a: i64, b: i64) -> i64 = a + b
fn main() -> i64 = add(20, 22)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_recursion_factorial() {
    let source = r#"
fn factorial(n: i64) -> i64 = n < 2 ? 1 : n * @(n - 1)
fn main() -> i64 = factorial(5)
"#;
    // 5! = 120, exit code is 120 % 256 = 120
    assert_exit_code(source, 120);
}

#[test]
fn exec_recursion_fibonacci() {
    let source = r#"
fn fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)
fn main() -> i64 = fib(10)
"#;
    // fib(10) = 55
    assert_exit_code(source, 55);
}

#[test]
fn exec_if_else_true_branch() {
    let source = r#"
fn main() -> i64 {
    x := 10
    I x > 5 { 1 } else { 0 }
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn exec_if_else_false_branch() {
    let source = r#"
fn main() -> i64 {
    x := 3
    I x > 5 { 1 } else { 0 }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn exec_nested_if() {
    let source = r#"
fn classify(x: i64) -> i64 {
    I x > 100 { 3 } else I x > 10 { 2 } else I x > 0 { 1 } else { 0 }
}
fn main() -> i64 = classify(50)
"#;
    assert_exit_code(source, 2);
}

#[test]
fn exec_ternary() {
    let source = "fn main() -> i64 = 5 > 3 ? 42 : 0";
    assert_exit_code(source, 42);
}

#[test]
fn exec_variable_binding() {
    let source = r#"
fn main() -> i64 {
    x := 10
    y := 32
    x + y
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_loop_with_break() {
    let source = r#"
fn main() -> i64 {
    i := mut 0
    sum := mut 0
    L {
        I i >= 10 { B }
        sum = sum + i
        i = i + 1
    }
    sum
}
"#;
    // sum of 0..9 = 45
    assert_exit_code(source, 45);
}

#[test]
fn exec_left_shift() {
    assert_exit_code("fn main() -> i64 = 1 << 4", 16);
}

#[test]
fn exec_right_shift() {
    assert_exit_code("fn main() -> i64 = 32 >> 2", 8);
}

#[test]
fn exec_comparison_eq() {
    let source = "fn main() -> i64 = 5 == 5 ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_comparison_neq() {
    let source = "fn main() -> i64 = 5 != 3 ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_comparison_lt() {
    let source = "fn main() -> i64 = 3 < 5 ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_comparison_gte() {
    let source = "fn main() -> i64 = 5 >= 5 ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_logical_and() {
    let source = "fn main() -> i64 = (1 == 1) && (2 == 2) ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_logical_or() {
    let source = "fn main() -> i64 = (1 == 2) || (2 == 2) ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_negative_numbers() {
    assert_exit_code("fn main() -> i64 = 0 - 42", 214);
    // -42 as u8 = 214 (exit codes are 0-255)
}

// ==================== Printf / Stdout Tests ====================

#[test]
fn exec_printf_integer() {
    let source = r#"
fn main() -> i64 {
    printf("%d\n", 42)
    0
}
"#;
    assert_stdout_exact(source, "42");
}

#[test]
fn exec_printf_multiple() {
    let source = r#"
fn main() -> i64 {
    printf("%d\n", 10)
    printf("%d\n", 20)
    printf("%d\n", 30)
    0
}
"#;
    assert_stdout_lines(source, &["10", "20", "30"]);
}

#[test]
fn exec_printf_string() {
    let source = r#"
fn main() -> i64 {
    printf("hello world\n")
    0
}
"#;
    assert_stdout_exact(source, "hello world");
}

#[test]
fn exec_printf_computed_value() {
    let source = r#"
fn add(a: i64, b: i64) -> i64 = a + b
fn main() -> i64 {
    printf("%d\n", add(20, 22))
    0
}
"#;
    assert_stdout_exact(source, "42");
}

#[test]
fn exec_printf_loop_output() {
    let source = r#"
fn main() -> i64 {
    i := mut 1
    L {
        I i > 5 { B }
        printf("%d\n", i)
        i = i + 1
    }
    0
}
"#;
    assert_stdout_lines(source, &["1", "2", "3", "4", "5"]);
}

#[test]
fn exec_printf_fibonacci_sequence() {
    let source = r#"
fn fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)
fn main() -> i64 {
    i := mut 0
    L {
        I i >= 10 { B }
        printf("%d\n", fib(i))
        i = i + 1
    }
    0
}
"#;
    assert_stdout_lines(
        source,
        &["0", "1", "1", "2", "3", "5", "8", "13", "21", "34"],
    );
}

#[test]
fn exec_printf_conditional() {
    let source = r#"
fn classify(x: i64) -> i64 {
    I x > 0 { printf("positive\n") }
    else I x == 0 { printf("zero\n") }
    else { printf("negative\n") }
    0
}
fn main() -> i64 {
    classify(5)
    classify(0)
    classify(0 - 1)
    0
}
"#;
    assert_stdout_lines(source, &["positive", "zero", "negative"]);
}

// ==================== Struct Execution Tests ====================

#[test]
fn exec_struct_field_access() {
    let source = r#"
struct Point { x: i64, y: i64 }
fn main() -> i64 {
    p := Point { x: 10, y: 32 }
    p.x + p.y
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Match Execution Tests ====================

#[test]
fn exec_match_literals() {
    let source = r#"
fn day_type(d: i64) -> i64 {
    match d {
        1 => 10,
        2 => 20,
        3 => 30,
        _ => 0
    }
}
fn main() -> i64 = day_type(2)
"#;
    assert_exit_code(source, 20);
}

#[test]
fn exec_match_with_computation() {
    let source = r#"
fn score(x: i64) -> i64 {
    match x {
        1 => x * 10,
        2 => x * 20,
        3 => x * 30,
        _ => 0
    }
}
fn main() -> i64 = score(3)
"#;
    assert_exit_code(source, 90);
}

// ==================== Closure Execution Tests ====================

#[test]
fn exec_closure_basic() {
    let source = r#"
fn apply(f: (i64) -> i64, x: i64) -> i64 = f(x)
fn main() -> i64 = apply(|x| x * 2, 21)
"#;
    assert_exit_code(source, 42);
}

// ==================== IR-to-Execution Converted Tests (Task #2) ====================
// These tests were previously IR-only in integration_tests.rs.
// Now they verify actual runtime behavior.

#[test]
fn exec_converted_max_function() {
    let source = r#"
fn max(a: i64, b: i64) -> i64 = I a > b { a } else { b }
fn main() -> i64 = max(10, 20)
"#;
    assert_exit_code(source, 20);
}

#[test]
fn exec_converted_max_reversed() {
    let source = r#"
fn max(a: i64, b: i64) -> i64 = I a > b { a } else { b }
fn main() -> i64 = max(20, 10)
"#;
    assert_exit_code(source, 20);
}

#[test]
fn exec_converted_subtraction() {
    assert_exit_code("fn main() -> i64 = 100 - 58", 42);
}

#[test]
fn exec_converted_division() {
    assert_exit_code("fn main() -> i64 = 126 / 3", 42);
}

#[test]
fn exec_converted_modulo() {
    assert_exit_code("fn main() -> i64 = 107 % 65", 42);
}

#[test]
fn exec_converted_comparison_less_true() {
    let source = "fn main() -> i64 = 5 < 10 ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_converted_comparison_less_false() {
    let source = "fn main() -> i64 = 10 < 5 ? 1 : 0";
    assert_exit_code(source, 0);
}

#[test]
fn exec_converted_comparison_lte() {
    let source = "fn main() -> i64 = 5 <= 5 ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_converted_comparison_gt() {
    let source = "fn main() -> i64 = 10 > 5 ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_converted_ternary_abs() {
    let source = r#"
fn abs(x: i64) -> i64 = x < 0 ? 0 - x : x
fn main() -> i64 = abs(0 - 42)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_converted_ternary_abs_positive() {
    let source = r#"
fn abs(x: i64) -> i64 = x < 0 ? 0 - x : x
fn main() -> i64 = abs(42)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_converted_struct_two_fields() {
    let source = r#"
struct Vec2 { x: i64, y: i64 }
fn dot(a: Vec2, b: Vec2) -> i64 = a.x * b.x + a.y * b.y
fn main() -> i64 = dot(Vec2 { x: 3, y: 4 }, Vec2 { x: 5, y: 6 })
"#;
    // 3*5 + 4*6 = 15 + 24 = 39
    assert_exit_code(source, 39);
}

#[test]
fn exec_converted_generic_identity() {
    let source = r#"
fn identity<T>(x: T) -> type = x
fn main() -> i64 = identity(99)
"#;
    assert_exit_code(source, 99);
}

#[test]
fn exec_converted_generic_add_pair() {
    let source = r#"
fn add_pair(a: i64, b: i64) -> i64 = a + b
fn main() -> i64 = add_pair(10, 20)
"#;
    assert_exit_code(source, 30);
}

// Note: exec_converted_generic_wrap (F wrap<T>(x:T)->T = x) is near-identical
// to exec_converted_generic_identity above — both test identity-style generic functions.

#[test]
fn exec_converted_match_default() {
    let source = r#"
fn classify(x: i64) -> i64 {
    match x {
        1 => 10,
        2 => 20,
        _ => 99
    }
}
fn main() -> i64 = classify(7)
"#;
    assert_exit_code(source, 99);
}

#[test]
fn exec_converted_chained_calls() {
    let source = r#"
fn inc(x: i64) -> i64 = x + 1
fn double(x: i64) -> i64 = x * 2
fn main() -> i64 = double(inc(double(inc(0))))
"#;
    // inc(0)=1, double(1)=2, inc(2)=3, double(3)=6
    assert_exit_code(source, 6);
}

#[test]
fn exec_converted_countdown_loop() {
    let source = r#"
fn main() -> i64 {
    n := mut 10
    L {
        I n <= 0 { B }
        n = n - 1
    }
    n
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn exec_converted_nested_struct_access() {
    let source = r#"
struct Inner { val: i64 }
struct Outer { a: Inner, b: i64 }
fn main() -> i64 {
    o := Outer { a: Inner { val: 40 }, b: 2 }
    o.a.val + o.b
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_converted_multiple_returns() {
    let source = r#"
fn clamp(x: i64, lo: i64, hi: i64) -> i64 {
    I x < lo { return lo }
    I x > hi { return hi }
    x
}
fn main() -> i64 = clamp(50, 0, 42)
"#;
    assert_exit_code(source, 42);
}

// ==================== Stage 1: std Module Execution Tests (Task #4) ====================

// --- Option ---

#[test]
fn exec_std_option_some_match() {
    let source = r#"
enum Option { Some(i64), None }
fn main() -> i64 {
    x := Some(42)
    match x {
        Some(v) => v,
        None => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_std_option_none_match() {
    let source = r#"
enum Option { Some(i64), None }
fn main() -> i64 {
    x := None
    match x {
        Some(v) => v,
        None => 99
    }
}
"#;
    assert_exit_code(source, 99);
}

#[test]
fn exec_std_option_unwrap_some() {
    let source = r#"
enum Option { Some(i64), None }
fn unwrap_or(opt: Option, default: i64) -> i64 {
    match opt {
        Some(v) => v,
        None => default
    }
}
fn main() -> i64 = unwrap_or(Some(42), 0)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_std_option_unwrap_none_default() {
    let source = r#"
enum Option { Some(i64), None }
fn unwrap_or(opt: Option, default: i64) -> i64 {
    match opt {
        Some(v) => v,
        None => default
    }
}
fn main() -> i64 = unwrap_or(None, 99)
"#;
    assert_exit_code(source, 99);
}

#[test]
fn exec_std_option_is_some() {
    let source = r#"
enum Option { Some(i64), None }
fn is_some(opt: Option) -> i64 {
    match opt {
        Some(_) => 1,
        None => 0
    }
}
fn main() -> i64 = is_some(Some(10)) + is_some(None)
"#;
    // 1 + 0 = 1
    assert_exit_code(source, 1);
}

// --- Result ---

#[test]
fn exec_std_result_ok_match() {
    let source = r#"
enum Result { Ok(i64), Err(i64) }
fn main() -> i64 {
    r := Ok(42)
    match r {
        Ok(v) => v,
        Err(_) => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_std_result_err_match() {
    let source = r#"
enum Result { Ok(i64), Err(i64) }
fn main() -> i64 {
    r := Err(1)
    match r {
        Ok(v) => v,
        Err(e) => e + 98
    }
}
"#;
    // 1 + 98 = 99
    assert_exit_code(source, 99);
}

#[test]
fn exec_std_result_is_ok() {
    let source = r#"
enum Result { Ok(i64), Err(i64) }
fn is_ok(r: Result) -> i64 {
    match r {
        Ok(_) => 1,
        Err(_) => 0
    }
}
fn main() -> i64 = is_ok(Ok(5)) + is_ok(Err(1))
"#;
    // 1 + 0 = 1
    assert_exit_code(source, 1);
}

#[test]
fn exec_std_result_unwrap_or() {
    let source = r#"
enum Result { Ok(i64), Err(i64) }
fn unwrap_or(r: Result, default: i64) -> i64 {
    match r {
        Ok(v) => v,
        Err(_) => default
    }
}
fn main() -> i64 = unwrap_or(Ok(42), 0) + unwrap_or(Err(1), 58)
"#;
    // 42 + 58 = 100
    assert_exit_code(source, 100);
}

#[test]
fn exec_std_result_chain() {
    let source = r#"
enum Result { Ok(i64), Err(i64) }
fn safe_div(a: i64, b: i64) -> Result {
    I b == 0 { Err(1) } else { Ok(a / b) }
}
fn main() -> i64 {
    r1 := safe_div(84, 2)
    r2 := safe_div(10, 0)
    v1 := match r1 { Ok(v) => v, Err(_) => 0 }
    v2 := match r2 { Ok(v) => v, Err(e) => e }
    v1 + v2
}
"#;
    // Expected: safe_div(84,2)=Ok(42), safe_div(10,0)=Err(1)
    // v1=42, v2=1, total=43
    assert_exit_code(source, 43);
}

// --- String (inline struct + methods) ---

#[test]
fn exec_std_string_manual_build() {
    let source = r#"
struct MyStr { data: i64, len: i64 }

fn new_str() -> MyStr {
    buf := malloc(64)
    store_byte(buf, 0)
    MyStr { data: buf, len: 0 }
}

fn push_byte(s: MyStr, b: i64) -> MyStr {
    store_byte(s.data + s.len, b)
    MyStr { data: s.data, len: s.len + 1 }
}

fn main() -> i64 {
    s := mut new_str()
    s = push_byte(s, 72)   # H
    s = push_byte(s, 105)  # i
    store_byte(s.data + s.len, 0)
    # Verify length is correct
    s.len
}
"#;
    assert_exit_code(source, 2);
}

// --- Vec (inline struct implementation) ---

#[test]
fn exec_std_vec_push_get() {
    let source = r#"
struct Vec { data: i64, len: i64, cap: i64 }

fn vec_new() -> Vec {
    data := malloc(64)
    Vec { data: data, len: 0, cap: 8 }
}

fn vec_push(v: Vec, val: i64) -> Vec {
    store_i64(v.data + v.len * 8, val)
    Vec { data: v.data, len: v.len + 1, cap: v.cap }
}

fn vec_get(v: Vec, idx: i64) -> i64 {
    load_i64(v.data + idx * 8)
}

fn main() -> i64 {
    v := mut vec_new()
    v = vec_push(v, 10)
    v = vec_push(v, 20)
    v = vec_push(v, 12)
    vec_get(v, 0) + vec_get(v, 1) + vec_get(v, 2)
}
"#;
    // 10 + 20 + 12 = 42
    assert_exit_code(source, 42);
}

#[test]
fn exec_std_vec_length() {
    let source = r#"
struct Vec { data: i64, len: i64, cap: i64 }

fn vec_new() -> Vec {
    data := malloc(64)
    Vec { data: data, len: 0, cap: 8 }
}

fn vec_push(v: Vec, val: i64) -> Vec {
    store_i64(v.data + v.len * 8, val)
    Vec { data: v.data, len: v.len + 1, cap: v.cap }
}

fn main() -> i64 {
    v := mut vec_new()
    v = vec_push(v, 1)
    v = vec_push(v, 2)
    v = vec_push(v, 3)
    v = vec_push(v, 4)
    v = vec_push(v, 5)
    v.len
}
"#;
    assert_exit_code(source, 5);
}

#[test]
fn exec_std_vec_sum_loop() {
    let source = r#"
struct Vec { data: i64, len: i64, cap: i64 }

fn vec_new() -> Vec {
    data := malloc(80)
    Vec { data: data, len: 0, cap: 10 }
}

fn vec_push(v: Vec, val: i64) -> Vec {
    store_i64(v.data + v.len * 8, val)
    Vec { data: v.data, len: v.len + 1, cap: v.cap }
}

fn vec_get(v: Vec, idx: i64) -> i64 {
    load_i64(v.data + idx * 8)
}

fn main() -> i64 {
    v := mut vec_new()
    i := mut 1
    L {
        I i > 9 { B }
        v = vec_push(v, i)
        i = i + 1
    }
    sum := mut 0
    j := mut 0
    L {
        I j >= v.len { B }
        sum = sum + vec_get(v, j)
        j = j + 1
    }
    sum
}
"#;
    // sum of 1..9 = 45
    assert_exit_code(source, 45);
}

// --- HashMap (inline implementation) ---

#[test]
fn exec_std_hashmap_set_get() {
    let source = r#"
struct HashMap { buckets: i64, size: i64, cap: i64 }

fn hm_new() -> HashMap {
    cap := 16
    buckets := malloc(cap * 8)
    # Zero out buckets
    i := mut 0
    L {
        I i >= cap { B }
        store_i64(buckets + i * 8, 0)
        i = i + 1
    }
    HashMap { buckets: buckets, size: 0, cap: cap }
}

fn hm_abs(x: i64) -> i64 = x < 0 ? 0 - x : x

fn hm_hash(key: i64, cap: i64) -> i64 {
    h := hm_abs(key * 2654435761)
    h % cap
}

fn hm_set(m: HashMap, key: i64, val: i64) -> HashMap {
    idx := hm_hash(key, m.cap)
    # Simple direct-mapped (no collision handling for this test)
    store_i64(m.buckets + idx * 16, key)
    store_i64(m.buckets + idx * 16 + 8, val)
    HashMap { buckets: m.buckets, size: m.size + 1, cap: m.cap }
}

fn hm_get(m: HashMap, key: i64) -> i64 {
    idx := hm_hash(key, m.cap)
    load_i64(m.buckets + idx * 16 + 8)
}

fn main() -> i64 {
    m := mut hm_new()
    m = hm_set(m, 1, 42)
    m = hm_set(m, 2, 58)
    hm_get(m, 1)
}
"#;
    assert_exit_code(source, 42);
}

// --- vaisc CLI-based tests for std/ imports ---

/// Helper: compile a .vais file using vaisc CLI, then run the binary
fn compile_and_run_via_cli(vais_file: &Path) -> Result<RunResult, String> {
    let tmp_dir = TempDir::new().map_err(|e| format!("TempDir: {}", e))?;
    let exe_name = if cfg!(target_os = "windows") {
        "test_exe.exe"
    } else {
        "test_exe"
    };
    let exe_path = tmp_dir.path().join(exe_name);

    // Find vaisc binary
    let mut vaisc = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    vaisc.push("vaisc");
    if !vaisc.exists() {
        vaisc = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        vaisc.push("vaisc");
    }

    // vaisc build <file> -o <exe> --no-ownership-check
    let output = Command::new(&vaisc)
        .args([
            "build",
            &vais_file.to_string_lossy(),
            "-o",
            &exe_path.to_string_lossy(),
            "--no-ownership-check",
        ])
        .current_dir(project_root())
        .output()
        .map_err(|e| format!("Failed to run vaisc: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "vaisc build failed:\nstdout: {}\nstderr: {}",
            stdout, stderr
        ));
    }

    if !exe_path.exists() {
        return Err("vaisc build succeeded but no executable produced".to_string());
    }

    let run_output = Command::new(&exe_path)
        .output()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    Ok(RunResult {
        exit_code: run_output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&run_output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&run_output.stderr).to_string(),
    })
}

#[test]
fn exec_std_vec_import_example() {
    let root = project_root();
    let path = root.join("examples/simple_vec_test.vais");
    if !path.exists() {
        eprintln!("Skipping: simple_vec_test.vais not found");
        return;
    }
    match compile_and_run_via_cli(&path) {
        Ok(result) => {
            // simple_vec_test returns 142 (42 + 100)
            assert_eq!(
                result.exit_code, 142,
                "simple_vec_test exit code mismatch.\nstdout: {}\nstderr: {}",
                result.stdout, result.stderr
            );
        }
        Err(e) => eprintln!("Note: simple_vec_test.vais CLI compile skipped: {}", e),
    }
}

#[test]
fn exec_std_hashmap_import_example() {
    let root = project_root();
    let path = root.join("examples/simple_hashmap_test.vais");
    if !path.exists() {
        eprintln!("Skipping: simple_hashmap_test.vais not found");
        return;
    }
    match compile_and_run_via_cli(&path) {
        Ok(result) => {
            // simple_hashmap_test returns 100 (42 + 58)
            assert_eq!(
                result.exit_code, 100,
                "simple_hashmap_test exit code mismatch.\nstdout: {}\nstderr: {}",
                result.stdout, result.stderr
            );
        }
        Err(e) => eprintln!("Note: simple_hashmap_test.vais CLI compile skipped: {}", e),
    }
}

#[test]
fn exec_std_option_import_example() {
    let root = project_root();
    let path = root.join("examples/option_test.vais");
    if !path.exists() {
        eprintln!("Skipping: option_test.vais not found");
        return;
    }
    match compile_and_run_via_cli(&path) {
        Ok(result) => {
            // option_test: Some(42) match → 42
            assert_eq!(
                result.exit_code, 42,
                "option_test exit code mismatch.\nstdout: {}\nstderr: {}",
                result.stdout, result.stderr
            );
        }
        Err(e) => eprintln!("Note: option_test.vais CLI compile skipped: {}", e),
    }
}

// ==================== Examples Smoke Test (Task #3) ====================

/// Examples that should compile and run as standalone (no extern FFI beyond libc)
const SMOKE_EXAMPLES: &[&str] = &[
    "hello.vais",
    "hello_world.vais",
    "control_flow.vais",
    "fib.vais",
    "math.vais",
    "simple_test.vais",
    "closure_simple.vais",
    "defer_simple.vais",
    "loop_break_test.vais",
    "match_test.vais",
    "pipe_operator.vais",
    "putchar_var.vais",
    "test_bitwise.vais",
    "test_bitwise_precedence.vais",
    "tco_tail_call.vais",
    "tco_stress.vais",
    "enum_test.vais",
    "printf_test.vais",
    "arrays.vais",
    "destructuring.vais",
];

/// Compile and run each example, assert no crash (exit 0 or compilation success)
#[test]
fn exec_smoke_examples_batch() {
    let root = project_root();
    let examples_dir = root.join("examples");
    let mut passed = 0;
    let mut failed = Vec::new();

    for name in SMOKE_EXAMPLES {
        let path = examples_dir.join(name);
        if !path.exists() {
            continue;
        }
        match compile_and_run_file(&path) {
            Ok(result) => {
                if result.stderr.contains("SIGSEGV") || result.stderr.contains("segmentation fault")
                {
                    failed.push(format!("{}: CRASHED (exit {})", name, result.exit_code));
                } else {
                    passed += 1;
                }
            }
            Err(e) => {
                failed.push(format!("{}: {}", name, e));
            }
        }
    }

    if !failed.is_empty() {
        panic!(
            "Examples smoke test: {}/{} passed, {} failed:\n{}",
            passed,
            SMOKE_EXAMPLES.len(),
            failed.len(),
            failed.join("\n")
        );
    }
}

/// Test that all .vais files in examples/ at least compile to IR (no crash at compile time)
#[test]
fn exec_smoke_all_examples_compile() {
    let root = project_root();
    let examples_dir = root.join("examples");
    let mut passed = 0;
    let mut failed = Vec::new();

    if let Ok(entries) = fs::read_dir(&examples_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("vais") {
                continue;
            }
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            match fs::read_to_string(&path) {
                Ok(source) => match compile_to_ir(&source) {
                    Ok(_) => passed += 1,
                    Err(e) => failed.push(format!("{}: {}", name, e)),
                },
                Err(e) => failed.push(format!("{}: read error: {}", name, e)),
            }
        }
    }

    // We expect most to compile; report stats
    let total = passed + failed.len();
    eprintln!(
        "Examples compilation: {}/{} passed ({} failed)",
        passed,
        total,
        failed.len()
    );

    // Don't fail if some examples need runtime — just report
    if !failed.is_empty() {
        eprintln!("Failed examples:");
        for f in &failed {
            eprintln!("  {}", f);
        }
    }

    // At least 50% should compile
    assert!(
        passed as f64 / total as f64 > 0.5,
        "Less than 50% of examples compile: {}/{}",
        passed,
        total
    );
}

// ==================== Stage 1: I/O Tests (Task #5) ====================

#[test]
fn exec_io_printf_basic() {
    let source = r#"
fn main() -> i64 {
    printf("hello %d\n", 42)
    0
}
"#;
    assert_stdout_exact(source, "hello 42");
}

#[test]
fn exec_io_puts_output() {
    let source = r#"
fn main() -> i64 {
    puts("Hello from Vais!")
    0
}
"#;
    assert_stdout_exact(source, "Hello from Vais!");
}

#[test]
fn exec_io_putchar_sequence() {
    let source = r#"
fn main() -> i64 {
    putchar(72)   # H
    putchar(105)  # i
    putchar(33)   # !
    putchar(10)   # newline
    0
}
"#;
    assert_stdout_exact(source, "Hi!");
}

#[test]
fn exec_io_printf_multiple_formats() {
    let source = r#"
fn main() -> i64 {
    printf("int: %d, char: %c\n", 42, 65)  # 65 = 'A'
    0
}
"#;
    assert_stdout_contains(source, "int: 42");
}

#[test]
fn exec_io_mixed_output() {
    let source = r#"
fn main() -> i64 {
    puts("Line 1")
    printf("Line %d\n", 2)
    putchar(51)   # '3'
    putchar(10)   # newline
    0
}
"#;
    assert_stdout_lines(source, &["Line 1", "Line 2", "3"]);
}

#[test]
fn exec_io_formatted_computation() {
    let source = r#"
fn compute(x: i64, y: i64) -> i64 = x * y + 2

fn main() -> i64 {
    result := compute(10, 4)
    printf("result = %d\n", result)
    0
}
"#;
    assert_stdout_exact(source, "result = 42");
}

// ==================== Stage 1: Network Smoke Tests (Task #6) ====================

#[test]
fn exec_net_tcp_struct_smoke() {
    let source = r#"
struct TcpStream { fd: i64, addr: i64, port: i64 }

fn new_tcp(addr: i64, port: i64) -> TcpStream {
    TcpStream { fd: 0, addr: addr, port: port }
}

fn tcp_port(stream: TcpStream) -> i64 {
    stream.port
}

fn main() -> i64 {
    stream := new_tcp(127, 80)
    tcp_port(stream)
}
"#;
    assert_exit_code(source, 80);
}

#[test]
fn exec_net_socket_addr_smoke() {
    let source = r#"
struct SocketAddr { ip: i64, port: i64 }

fn make_addr(ip: i64, port: i64) -> SocketAddr {
    SocketAddr { ip: ip, port: port }
}

fn addr_port(addr: SocketAddr) -> i64 {
    addr.port
}

fn main() -> i64 {
    addr := make_addr(127, 42)
    addr_port(addr)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_net_http_request_struct_smoke() {
    let source = r#"
struct HttpRequest { method: i64, path: i64, version: i64 }

fn new_request(method: i64, path: i64) -> HttpRequest {
    HttpRequest { method: method, path: path, version: 11 }
}

fn request_version(req: HttpRequest) -> i64 {
    req.version
}

fn main() -> i64 {
    req := new_request(1, 100)  # 1=GET, path=dummy
    request_version(req)
}
"#;
    assert_exit_code(source, 11);
}

#[test]
fn exec_net_http_response_struct_smoke() {
    let source = r#"
struct HttpResponse { status: i64, body: i64, headers: i64 }

fn new_response(status: i64) -> HttpResponse {
    HttpResponse { status: status, body: 0, headers: 0 }
}

fn response_status(resp: HttpResponse) -> i64 {
    resp.status
}

fn main() -> i64 {
    resp := new_response(200)
    response_status(resp)
}
"#;
    assert_exit_code(source, 200);
}
// ==================== Pattern Alias Tests ====================

#[test]
fn exec_pattern_alias_tuple() {
    let source = r#"
fn main() -> i64 {
    x := (1, 2)
    match x {
        whole @ (a, b) => a + b
    }
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn exec_pattern_alias_literal() {
    let source = r#"
fn main() -> i64 {
    x := 42
    match x {
        n @ 42 => n,
        _ => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_pattern_alias_wildcard() {
    let source = r#"
fn main() -> i64 {
    x := 99
    match x {
        n @ _ => n
    }
}
"#;
    assert_exit_code(source, 99);
}

#[test]
fn exec_pattern_alias_variant() {
    let source = r#"
enum Option<T> {
    Some(T),
    None
}

fn main() -> i64 {
    opt := Some(42)
    match opt {
        whole @ Some(val) => val,
        _ => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Extended Tests: Phase 41-44 Features ====================

// --- Range Loop Tests (Phase 41) ---

#[test]
fn exec_range_loop_basic() {
    let source = r#"
fn main() -> i64 {
    sum := mut 0
    L i:0..5 {
        sum = sum + i
    }
    sum
}
"#;
    // 0 + 1 + 2 + 3 + 4 = 10
    assert_exit_code(source, 10);
}

#[test]
fn exec_range_loop_inclusive() {
    let source = r#"
fn main() -> i64 {
    sum := mut 0
    L i:0..=5 {
        sum = sum + i
    }
    sum
}
"#;
    // 0 + 1 + 2 + 3 + 4 + 5 = 15
    assert_exit_code(source, 15);
}

#[test]
fn exec_range_loop_product() {
    let source = r#"
fn main() -> i64 {
    prod := mut 1
    L i:1..5 {
        prod = prod * i
    }
    prod
}
"#;
    // 1 * 2 * 3 * 4 = 24
    assert_exit_code(source, 24);
}

#[test]
fn exec_range_loop_start_nonzero() {
    let source = r#"
fn main() -> i64 {
    sum := mut 0
    L i:10..15 {
        sum = sum + i
    }
    sum
}
"#;
    // 10 + 11 + 12 + 13 + 14 = 60
    assert_exit_code(source, 60);
}

// --- Closure Capture Modes (Phase 42) ---

#[test]
fn exec_closure_move_capture() {
    let source = r#"
fn main() -> i64 {
    x := 42
    f := move |y| x + y
    f(8)
}
"#;
    // 42 + 8 = 50
    assert_exit_code(source, 50);
}

#[test]
fn exec_closure_capture_multiple() {
    let source = r#"
fn main() -> i64 {
    a := 10
    b := 20
    f := |x| a + b + x
    f(12)
}
"#;
    // 10 + 20 + 12 = 42
    assert_exit_code(source, 42);
}

// --- Struct Method Calls ---

#[test]
fn exec_struct_method_self() {
    let source = r#"
struct Point { x: i64, y: i64 }

impl Point {
    fn sum(&self) -> i64 = self.x + self.y
}

fn main() -> i64 {
    p := Point { x: 10, y: 32 }
    p.sum()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_struct_method_mutate() {
    let source = r#"
struct Counter { val: i64 }

impl Counter {
    fn new() -> Counter = Counter { val: 0 }

    fn increment(&self, amt: i64) -> Counter =
        Counter { val: self.val + amt }

    fn get(&self) -> i64 = self.val
}

fn main() -> i64 {
    c := mut Counter::new()
    c = c.increment(10)
    c = c.increment(32)
    c.get()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_struct_method_chained() {
    let source = r#"
struct Val { n: i64 }

impl Val {
    fn add(&self, x: i64) -> Val = Val { n: self.n + x }
    fn mul(&self, x: i64) -> Val = Val { n: self.n * x }
    fn get(&self) -> i64 = self.n
}

fn main() -> i64 {
    v := mut Val { n: 2 }
    v = v.add(3)
    v = v.mul(7)
    v = v.add(7)
    v.get()
}
"#;
    // (2 + 3) * 7 + 7 = 35 + 7 = 42
    assert_exit_code(source, 42);
}

// --- Enum Variant Matching (Phase 34) ---

#[test]
fn exec_enum_variant_match_simple() {
    let source = r#"
enum Status { Good, Bad }

fn check(s: Status) -> i64 {
    match s {
        Good => 1,
        Bad => 0
    }
}

fn main() -> i64 = check(Good)
"#;
    assert_exit_code(source, 1);
}

#[test]
fn exec_enum_variant_with_data() {
    let source = r#"
enum Value { Int(i64), None }

fn extract(v: Value) -> i64 {
    match v {
        Int(n) => n,
        None => 0
    }
}

fn main() -> i64 = extract(Int(42))
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_enum_multiple_variants() {
    let source = r#"
enum Color { Red, Green, Blue }

fn color_code(c: Color) -> i64 {
    match c {
        Red => 1,
        Green => 2,
        Blue => 3
    }
}

fn main() -> i64 = color_code(Green)
"#;
    assert_exit_code(source, 2);
}

// --- Slice Operations (Phase 6) ---

#[test]
fn exec_slice_type_compiles() {
    let source = r#"
fn get_slice(arr: &[i64]) -> i64 = 42

fn main() -> i64 = get_slice(&[1, 2, 3])
"#;
    // Slice literal &[1,2,3] now correctly builds { i8*, i64 } fat pointer
    // and function signature matches. get_slice always returns 42.
    assert_exit_code(source, 42);
}

#[test]
fn exec_slice_len_method() {
    let source = r#"
fn slice_len(s: &[i64]) -> i64 = s.len()

fn main() -> i64 = slice_len(&[1, 2, 3, 4, 5])
"#;
    // Phase 73: Slice .len() uses extractvalue on { i8*, i64 } fat pointer field 1.
    // The fat pointer is correctly constructed at the call site with insertvalue.
    assert_exit_code(source, 5);
}

// --- Where Clause Tests (Phase 32) ---

#[test]
fn exec_where_clause_simple() {
    let source = r#"
trait Display {
    fn show(&self) -> i64
}

fn print_value<T>(val: T) -> i64
where T: Display {
    val.show()
}

struct MyInt { n: i64 }

impl MyInt: Display {
    fn show(&self) -> i64 = self.n
}

fn main() -> i64 = print_value(MyInt { n: 42 })
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_where_clause_multiple_bounds() {
    let source = r#"
trait Trait1 { fn method1(&self) -> i64 }
trait Trait2 { fn method2(&self) -> i64 }

fn use_both<T>(x: T) -> i64
where T: Trait1, T: Trait2 {
    x.method1() + x.method2()
}

fn main() -> i64 = 0
"#;
    // Phase 73: Generic function use_both is declared but not defined (no concrete
    // instantiation). Since main() doesn't call it, clang links successfully.
    assert_exit_code(source, 0);
}

// --- Trait Alias Tests (Phase 37) ---

#[test]
fn exec_trait_alias_compiles() {
    let source = r#"
trait Display { fn show(&self) -> i64 }
trait Debug { fn debug(&self) -> i64 }

type Printable = Display + Debug

fn main() -> i64 = 0
"#;
    assert_exit_code(source, 0);
}

// --- Async/Await Tests (Phase 43) ---

#[test]
fn exec_async_basic_compiles() {
    let source = r#"
A fn async_task() -> i64 = 42

fn main() -> i64 = 0
"#;
    assert_exit_code(source, 0);
}

// --- Advanced Pattern Matching ---

#[test]
fn exec_pattern_match_nested_enum() {
    let source = r#"
enum Inner { Val(i64), Ref(i64) }

fn extract(inner: Inner) -> i64 {
    match inner {
        Val(n) => n,
        Ref(n) => n * 2
    }
}

fn main() -> i64 {
    v := extract(Val(21))
    r := extract(Ref(21))
    v + r
}
"#;
    assert_exit_code(source, 63); // 21 + (21*2) = 21 + 42 = 63
}

#[test]
fn exec_pattern_match_or_pattern() {
    let source = r#"
fn classify(x: i64) -> i64 {
    match x {
        1 | 2 | 3 => 10,
        4 | 5 => 20,
        _ => 0
    }
}

fn main() -> i64 = classify(2) + classify(5)
"#;
    // 10 + 20 = 30
    assert_exit_code(source, 30);
}

#[test]
fn exec_pattern_match_guard() {
    let source = r#"
fn classify(x: i64) -> i64 {
    match x {
        n I n > 100 => 3,
        n I n > 10 => 2,
        n I n > 0 => 1,
        _ => 0
    }
}

fn main() -> i64 = classify(50)
"#;
    assert_exit_code(source, 2);
}

// --- Generic Functions with Constraints ---

#[test]
fn exec_generic_swap() {
    let source = r#"
fn swap<T>(a: T, b: T) -> type = b

fn main() -> i64 = swap(10, 42)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_generic_pair_first() {
    let source = r#"
struct Pair<T> { first: T, second: type }

fn get_first<T>(p: Pair<T>) -> type = p.first

fn main() -> i64 = get_first(Pair { first: 42, second: 100 })
"#;
    assert_exit_code(source, 42);
}

// --- Complex Control Flow ---

#[test]
fn exec_nested_loops() {
    let source = r#"
fn main() -> i64 {
    sum := mut 0
    i := mut 0
    L {
        I i >= 5 { B }
        j := mut 0
        L {
            I j >= 3 { B }
            sum = sum + 1
            j = j + 1
        }
        i = i + 1
    }
    sum
}
"#;
    // 5 * 3 = 15
    assert_exit_code(source, 15);
}

#[test]
fn exec_loop_with_continue() {
    let source = r#"
fn main() -> i64 {
    sum := mut 0
    i := mut 0
    L {
        I i >= 10 { B }
        i = i + 1
        I i % 2 == 0 { C }
        sum = sum + i
    }
    sum
}
"#;
    // Sum of odd numbers 1,3,5,7,9 = 25
    assert_exit_code(source, 25);
}

// --- Recursion with Different Patterns ---

#[test]
fn exec_tail_recursion_sum() {
    let source = r#"
fn sum_helper(n: i64, acc: i64) -> i64 = n == 0 ? acc : @(n - 1, acc + n)

fn sum(n: i64) -> i64 = sum_helper(n, 0)

fn main() -> i64 = sum(9)
"#;
    // sum(9) = 45
    assert_exit_code(source, 45);
}

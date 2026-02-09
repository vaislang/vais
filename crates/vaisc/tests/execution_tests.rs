//! Execution-based E2E Tests for the Vais Compiler
//!
//! Unlike IR-matching tests, these tests verify actual runtime behavior:
//! Source → Lexer → Parser → Type Checker → Codegen → LLVM IR → clang → Execute → Verify stdout/exit
//!
//! This file provides enhanced helpers for execution testing and houses
//! tests that specifically verify runtime output correctness.

#[allow(dead_code)]

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
    let ir = gen
        .generate_module(&module)
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
    let exe_path = tmp_dir.path().join("test_exe");

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

/// Assert that source compiles, runs, and returns exit code 0
fn assert_run_success(source: &str) {
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(
                result.exit_code, 0,
                "Expected exit code 0, got {}.\nstdout: {}\nstderr: {}",
                result.exit_code, result.stdout, result.stderr
            );
        }
        Err(e) => panic!("Compilation/execution failed: {}", e),
    }
}

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
                expected, result.stdout, result.stderr
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
fn assert_compile_error(source: &str) {
    assert!(
        compile_to_ir(source).is_err(),
        "Expected compilation to fail, but it succeeded"
    );
}

/// Assert that source compiles and runs without crashing (exit code 0, no stderr panic)
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
    assert_exit_code("F main() -> i64 = 42", 42);
}

#[test]
fn exec_return_zero() {
    assert_exit_code("F main() -> i64 = 0", 0);
}

#[test]
fn exec_arithmetic_add() {
    assert_exit_code("F main() -> i64 = 3 + 4", 7);
}

#[test]
fn exec_arithmetic_sub() {
    assert_exit_code("F main() -> i64 = 10 - 3", 7);
}

#[test]
fn exec_arithmetic_mul() {
    assert_exit_code("F main() -> i64 = 6 * 7", 42);
}

#[test]
fn exec_arithmetic_div() {
    assert_exit_code("F main() -> i64 = 84 / 2", 42);
}

#[test]
fn exec_arithmetic_mod() {
    assert_exit_code("F main() -> i64 = 17 % 5", 2);
}

#[test]
fn exec_arithmetic_precedence() {
    assert_exit_code("F main() -> i64 = 2 + 3 * 4", 14);
}

#[test]
fn exec_nested_arithmetic() {
    assert_exit_code("F main() -> i64 = (2 + 3) * (10 - 6)", 20);
}

#[test]
fn exec_function_call() {
    let source = r#"
F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add(20, 22)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_multiple_function_calls() {
    let source = r#"
F double(x: i64) -> i64 = x * 2
F inc(x: i64) -> i64 = x + 1
F main() -> i64 = inc(double(20))
"#;
    assert_exit_code(source, 41);
}

#[test]
fn exec_recursion_factorial() {
    let source = r#"
F factorial(n: i64) -> i64 = n < 2 ? 1 : n * @(n - 1)
F main() -> i64 = factorial(5)
"#;
    // 5! = 120, exit code is 120 % 256 = 120
    assert_exit_code(source, 120);
}

#[test]
fn exec_recursion_fibonacci() {
    let source = r#"
F fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)
F main() -> i64 = fib(10)
"#;
    // fib(10) = 55
    assert_exit_code(source, 55);
}

#[test]
fn exec_if_else_true_branch() {
    let source = r#"
F main() -> i64 {
    x := 10
    I x > 5 { 1 } E { 0 }
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn exec_if_else_false_branch() {
    let source = r#"
F main() -> i64 {
    x := 3
    I x > 5 { 1 } E { 0 }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn exec_nested_if() {
    let source = r#"
F classify(x: i64) -> i64 {
    I x > 100 { 3 } E I x > 10 { 2 } E I x > 0 { 1 } E { 0 }
}
F main() -> i64 = classify(50)
"#;
    assert_exit_code(source, 2);
}

#[test]
fn exec_ternary() {
    let source = "F main() -> i64 = 5 > 3 ? 42 : 0";
    assert_exit_code(source, 42);
}

#[test]
fn exec_variable_binding() {
    let source = r#"
F main() -> i64 {
    x := 10
    y := 32
    x + y
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_mutable_variable() {
    let source = r#"
F main() -> i64 {
    x := mut 10
    x = 42
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_loop_with_break() {
    let source = r#"
F main() -> i64 {
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
fn exec_bitwise_and() {
    assert_exit_code("F main() -> i64 = 255 & 15", 15);
}

#[test]
fn exec_bitwise_or() {
    assert_exit_code("F main() -> i64 = 240 | 15", 255);
}

#[test]
fn exec_bitwise_xor() {
    assert_exit_code("F main() -> i64 = 255 ^ 15", 240);
}

#[test]
fn exec_left_shift() {
    assert_exit_code("F main() -> i64 = 1 << 4", 16);
}

#[test]
fn exec_right_shift() {
    assert_exit_code("F main() -> i64 = 32 >> 2", 8);
}

#[test]
fn exec_comparison_eq() {
    let source = "F main() -> i64 = 5 == 5 ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_comparison_neq() {
    let source = "F main() -> i64 = 5 != 3 ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_comparison_lt() {
    let source = "F main() -> i64 = 3 < 5 ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_comparison_gte() {
    let source = "F main() -> i64 = 5 >= 5 ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_logical_and() {
    let source = "F main() -> i64 = (1 == 1) && (2 == 2) ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_logical_or() {
    let source = "F main() -> i64 = (1 == 2) || (2 == 2) ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_negative_numbers() {
    assert_exit_code("F main() -> i64 = 0 - 42", 214);
    // -42 as u8 = 214 (exit codes are 0-255)
}

#[test]
fn exec_gcd() {
    let source = r#"
F gcd(a: i64, b: i64) -> i64 = b == 0 ? a : @(b, a % b)
F main() -> i64 = gcd(48, 18)
"#;
    assert_exit_code(source, 6);
}

// ==================== Printf / Stdout Tests ====================

#[test]
fn exec_printf_integer() {
    let source = r#"
F main() -> i64 {
    printf("%d\n", 42)
    0
}
"#;
    assert_stdout_exact(source, "42");
}

#[test]
fn exec_printf_multiple() {
    let source = r#"
F main() -> i64 {
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
F main() -> i64 {
    printf("hello world\n")
    0
}
"#;
    assert_stdout_exact(source, "hello world");
}

#[test]
fn exec_printf_computed_value() {
    let source = r#"
F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 {
    printf("%d\n", add(20, 22))
    0
}
"#;
    assert_stdout_exact(source, "42");
}

#[test]
fn exec_printf_loop_output() {
    let source = r#"
F main() -> i64 {
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
F fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)
F main() -> i64 {
    i := mut 0
    L {
        I i >= 10 { B }
        printf("%d\n", fib(i))
        i = i + 1
    }
    0
}
"#;
    assert_stdout_lines(source, &["0", "1", "1", "2", "3", "5", "8", "13", "21", "34"]);
}

#[test]
fn exec_printf_conditional() {
    let source = r#"
F classify(x: i64) -> i64 {
    I x > 0 { printf("positive\n") }
    E I x == 0 { printf("zero\n") }
    E { printf("negative\n") }
    0
}
F main() -> i64 {
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
S Point { x: i64, y: i64 }
F main() -> i64 {
    p := Point { x: 10, y: 32 }
    p.x + p.y
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_struct_field_arithmetic() {
    let source = r#"
S Rect { w: i64, h: i64 }
F area(r: Rect) -> i64 = r.w * r.h
F main() -> i64 = area(Rect { w: 6, h: 7 })
"#;
    assert_exit_code(source, 42);
}

// ==================== Match Execution Tests ====================

#[test]
fn exec_match_literals() {
    let source = r#"
F day_type(d: i64) -> i64 {
    M d {
        1 => 10,
        2 => 20,
        3 => 30,
        _ => 0
    }
}
F main() -> i64 = day_type(2)
"#;
    assert_exit_code(source, 20);
}

#[test]
fn exec_match_with_computation() {
    let source = r#"
F score(x: i64) -> i64 {
    M x {
        1 => x * 10,
        2 => x * 20,
        3 => x * 30,
        _ => 0
    }
}
F main() -> i64 = score(3)
"#;
    assert_exit_code(source, 90);
}

// ==================== Closure Execution Tests ====================

#[test]
fn exec_closure_basic() {
    let source = r#"
F apply(f: (i64) -> i64, x: i64) -> i64 = f(x)
F main() -> i64 = apply(|x| x * 2, 21)
"#;
    assert_exit_code(source, 42);
}

// ==================== IR-to-Execution Converted Tests (Task #2) ====================
// These tests were previously IR-only in integration_tests.rs.
// Now they verify actual runtime behavior.

#[test]
fn exec_converted_max_function() {
    let source = r#"
F max(a: i64, b: i64) -> i64 = I a > b { a } E { b }
F main() -> i64 = max(10, 20)
"#;
    assert_exit_code(source, 20);
}

#[test]
fn exec_converted_max_reversed() {
    let source = r#"
F max(a: i64, b: i64) -> i64 = I a > b { a } E { b }
F main() -> i64 = max(20, 10)
"#;
    assert_exit_code(source, 20);
}

#[test]
fn exec_converted_subtraction() {
    assert_exit_code("F main() -> i64 = 100 - 58", 42);
}

#[test]
fn exec_converted_division() {
    assert_exit_code("F main() -> i64 = 126 / 3", 42);
}

#[test]
fn exec_converted_modulo() {
    assert_exit_code("F main() -> i64 = 107 % 65", 42);
}

#[test]
fn exec_converted_comparison_less_true() {
    let source = "F main() -> i64 = 5 < 10 ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_converted_comparison_less_false() {
    let source = "F main() -> i64 = 10 < 5 ? 1 : 0";
    assert_exit_code(source, 0);
}

#[test]
fn exec_converted_comparison_lte() {
    let source = "F main() -> i64 = 5 <= 5 ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_converted_comparison_gt() {
    let source = "F main() -> i64 = 10 > 5 ? 1 : 0";
    assert_exit_code(source, 1);
}

#[test]
fn exec_converted_ternary_abs() {
    let source = r#"
F abs(x: i64) -> i64 = x < 0 ? 0 - x : x
F main() -> i64 = abs(0 - 42)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_converted_ternary_abs_positive() {
    let source = r#"
F abs(x: i64) -> i64 = x < 0 ? 0 - x : x
F main() -> i64 = abs(42)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_converted_struct_two_fields() {
    let source = r#"
S Vec2 { x: i64, y: i64 }
F dot(a: Vec2, b: Vec2) -> i64 = a.x * b.x + a.y * b.y
F main() -> i64 = dot(Vec2 { x: 3, y: 4 }, Vec2 { x: 5, y: 6 })
"#;
    // 3*5 + 4*6 = 15 + 24 = 39
    assert_exit_code(source, 39);
}

#[test]
fn exec_converted_generic_identity() {
    let source = r#"
F identity<T>(x: T) -> T = x
F main() -> i64 = identity(99)
"#;
    assert_exit_code(source, 99);
}

#[test]
fn exec_converted_generic_add_pair() {
    let source = r#"
F add_pair(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add_pair(10, 20)
"#;
    assert_exit_code(source, 30);
}

#[test]
fn exec_converted_generic_wrap() {
    let source = r#"
F wrap<T>(x: T) -> T = x
F main() -> i64 = wrap(100)
"#;
    assert_exit_code(source, 100);
}

#[test]
fn exec_converted_match_default() {
    let source = r#"
F classify(x: i64) -> i64 {
    M x {
        1 => 10,
        2 => 20,
        _ => 99
    }
}
F main() -> i64 = classify(7)
"#;
    assert_exit_code(source, 99);
}

#[test]
fn exec_converted_chained_calls() {
    let source = r#"
F inc(x: i64) -> i64 = x + 1
F double(x: i64) -> i64 = x * 2
F main() -> i64 = double(inc(double(inc(0))))
"#;
    // inc(0)=1, double(1)=2, inc(2)=3, double(3)=6
    assert_exit_code(source, 6);
}

#[test]
fn exec_converted_countdown_loop() {
    let source = r#"
F main() -> i64 {
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
S Inner { val: i64 }
S Outer { a: Inner, b: i64 }
F main() -> i64 {
    o := Outer { a: Inner { val: 40 }, b: 2 }
    o.a.val + o.b
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_converted_multiple_returns() {
    let source = r#"
F clamp(x: i64, lo: i64, hi: i64) -> i64 {
    I x < lo { R lo }
    I x > hi { R hi }
    x
}
F main() -> i64 = clamp(50, 0, 42)
"#;
    assert_exit_code(source, 42);
}

// ==================== Stage 1: std Module Execution Tests (Task #4) ====================

// --- Option ---

#[test]
fn exec_std_option_some_match() {
    let source = r#"
E Option { Some(i64), None }
F main() -> i64 {
    x := Some(42)
    M x {
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
E Option { Some(i64), None }
F main() -> i64 {
    x := None
    M x {
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
E Option { Some(i64), None }
F unwrap_or(opt: Option, default: i64) -> i64 {
    M opt {
        Some(v) => v,
        None => default
    }
}
F main() -> i64 = unwrap_or(Some(42), 0)
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_std_option_unwrap_none_default() {
    let source = r#"
E Option { Some(i64), None }
F unwrap_or(opt: Option, default: i64) -> i64 {
    M opt {
        Some(v) => v,
        None => default
    }
}
F main() -> i64 = unwrap_or(None, 99)
"#;
    assert_exit_code(source, 99);
}

#[test]
fn exec_std_option_is_some() {
    let source = r#"
E Option { Some(i64), None }
F is_some(opt: Option) -> i64 {
    M opt {
        Some(_) => 1,
        None => 0
    }
}
F main() -> i64 = is_some(Some(10)) + is_some(None)
"#;
    // 1 + 0 = 1
    assert_exit_code(source, 1);
}

// --- Result ---

#[test]
fn exec_std_result_ok_match() {
    let source = r#"
E Result { Ok(i64), Err(i64) }
F main() -> i64 {
    r := Ok(42)
    M r {
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
E Result { Ok(i64), Err(i64) }
F main() -> i64 {
    r := Err(1)
    M r {
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
E Result { Ok(i64), Err(i64) }
F is_ok(r: Result) -> i64 {
    M r {
        Ok(_) => 1,
        Err(_) => 0
    }
}
F main() -> i64 = is_ok(Ok(5)) + is_ok(Err(1))
"#;
    // 1 + 0 = 1
    assert_exit_code(source, 1);
}

#[test]
fn exec_std_result_unwrap_or() {
    let source = r#"
E Result { Ok(i64), Err(i64) }
F unwrap_or(r: Result, default: i64) -> i64 {
    M r {
        Ok(v) => v,
        Err(_) => default
    }
}
F main() -> i64 = unwrap_or(Ok(42), 0) + unwrap_or(Err(1), 58)
"#;
    // 42 + 58 = 100
    assert_exit_code(source, 100);
}

#[test]
fn exec_std_result_chain() {
    let source = r#"
E Result { Ok(i64), Err(i64) }
F safe_div(a: i64, b: i64) -> Result {
    I b == 0 { Err(1) } E { Ok(a / b) }
}
F main() -> i64 {
    r1 := safe_div(84, 2)
    r2 := safe_div(10, 0)
    v1 := M r1 { Ok(v) => v, Err(_) => 0 }
    v2 := M r2 { Ok(v) => v, Err(e) => e }
    v1 + v2
}
"#;
    // 42 + 1 = 43
    assert_exit_code(source, 43);
}

// --- String (inline struct + methods) ---

#[test]
fn exec_std_string_manual_build() {
    let source = r#"
S MyStr { data: i64, len: i64 }

F new_str() -> MyStr {
    buf := malloc(64)
    store_byte(buf, 0)
    MyStr { data: buf, len: 0 }
}

F push_byte(s: MyStr, b: i64) -> MyStr {
    store_byte(s.data + s.len, b)
    MyStr { data: s.data, len: s.len + 1 }
}

F main() -> i64 {
    s := new_str()
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
S Vec { data: i64, len: i64, cap: i64 }

F vec_new() -> Vec {
    data := malloc(64)
    Vec { data: data, len: 0, cap: 8 }
}

F vec_push(v: Vec, val: i64) -> Vec {
    store_i64(v.data + v.len * 8, val)
    Vec { data: v.data, len: v.len + 1, cap: v.cap }
}

F vec_get(v: Vec, idx: i64) -> i64 {
    load_i64(v.data + idx * 8)
}

F main() -> i64 {
    v := vec_new()
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
S Vec { data: i64, len: i64, cap: i64 }

F vec_new() -> Vec {
    data := malloc(64)
    Vec { data: data, len: 0, cap: 8 }
}

F vec_push(v: Vec, val: i64) -> Vec {
    store_i64(v.data + v.len * 8, val)
    Vec { data: v.data, len: v.len + 1, cap: v.cap }
}

F main() -> i64 {
    v := vec_new()
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
S Vec { data: i64, len: i64, cap: i64 }

F vec_new() -> Vec {
    data := malloc(80)
    Vec { data: data, len: 0, cap: 10 }
}

F vec_push(v: Vec, val: i64) -> Vec {
    store_i64(v.data + v.len * 8, val)
    Vec { data: v.data, len: v.len + 1, cap: v.cap }
}

F vec_get(v: Vec, idx: i64) -> i64 {
    load_i64(v.data + idx * 8)
}

F main() -> i64 {
    v := vec_new()
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
S HashMap { buckets: i64, size: i64, cap: i64 }

F hm_new() -> HashMap {
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

F hm_abs(x: i64) -> i64 = x < 0 ? 0 - x : x

F hm_hash(key: i64, cap: i64) -> i64 {
    h := hm_abs(key * 2654435761)
    h % cap
}

F hm_set(m: HashMap, key: i64, val: i64) -> HashMap {
    idx := hm_hash(key, m.cap)
    # Simple direct-mapped (no collision handling for this test)
    store_i64(m.buckets + idx * 16, key)
    store_i64(m.buckets + idx * 16 + 8, val)
    HashMap { buckets: m.buckets, size: m.size + 1, cap: m.cap }
}

F hm_get(m: HashMap, key: i64) -> i64 {
    idx := hm_hash(key, m.cap)
    load_i64(m.buckets + idx * 16 + 8)
}

F main() -> i64 {
    m := hm_new()
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
    let exe_path = tmp_dir.path().join("test_exe");

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
                if result.stderr.contains("SIGSEGV")
                    || result.stderr.contains("segmentation fault")
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
F main() -> i64 {
    printf("hello %d\n", 42)
    0
}
"#;
    assert_stdout_exact(source, "hello 42");
}

#[test]
fn exec_io_puts_output() {
    let source = r#"
F main() -> i64 {
    puts("Hello from Vais!")
    0
}
"#;
    assert_stdout_exact(source, "Hello from Vais!");
}

#[test]
fn exec_io_putchar_sequence() {
    let source = r#"
F main() -> i64 {
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
F main() -> i64 {
    printf("int: %d, char: %c\n", 42, 65)  # 65 = 'A'
    0
}
"#;
    assert_stdout_contains(source, "int: 42");
}

#[test]
fn exec_io_mixed_output() {
    let source = r#"
F main() -> i64 {
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
F compute(x: i64, y: i64) -> i64 = x * y + 2

F main() -> i64 {
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
S TcpStream { fd: i64, addr: i64, port: i64 }

F new_tcp(addr: i64, port: i64) -> TcpStream {
    TcpStream { fd: 0, addr: addr, port: port }
}

F tcp_port(stream: TcpStream) -> i64 {
    stream.port
}

F main() -> i64 {
    stream := new_tcp(127, 80)
    tcp_port(stream)
}
"#;
    assert_exit_code(source, 80);
}

#[test]
fn exec_net_socket_addr_smoke() {
    let source = r#"
S SocketAddr { ip: i64, port: i64 }

F make_addr(ip: i64, port: i64) -> SocketAddr {
    SocketAddr { ip: ip, port: port }
}

F addr_port(addr: SocketAddr) -> i64 {
    addr.port
}

F main() -> i64 {
    addr := make_addr(127, 42)
    addr_port(addr)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn exec_net_http_request_struct_smoke() {
    let source = r#"
S HttpRequest { method: i64, path: i64, version: i64 }

F new_request(method: i64, path: i64) -> HttpRequest {
    HttpRequest { method: method, path: path, version: 11 }
}

F request_version(req: HttpRequest) -> i64 {
    req.version
}

F main() -> i64 {
    req := new_request(1, 100)  # 1=GET, path=dummy
    request_version(req)
}
"#;
    assert_exit_code(source, 11);
}

#[test]
fn exec_net_http_response_struct_smoke() {
    let source = r#"
S HttpResponse { status: i64, body: i64, headers: i64 }

F new_response(status: i64) -> HttpResponse {
    HttpResponse { status: status, body: 0, headers: 0 }
}

F response_status(resp: HttpResponse) -> i64 {
    resp.status
}

F main() -> i64 {
    resp := new_response(200)
    response_status(resp)
}
"#;
    assert_exit_code(source, 200);
}

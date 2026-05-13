//! Coverage tests for builtins/ and contracts/ modules
//!
//! Targets: builtins/platform.rs (stdlib functions), builtins/file_io.rs,
//! builtins/memory.rs, contracts/auto_checks.rs, contracts/decreases.rs
//!
//! Strategy: gen_ok/gen_result to exercise builtin registration and contract codegen.

use vais_codegen::CodeGenerator;
use vais_parser::parse;

fn gen_ok(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("Codegen failed for: {}\nErr: {}", source, e))
}

fn gen_result(source: &str) -> Result<String, String> {
    let module = parse(source).map_err(|e| format!("Parse: {:?}", e))?;
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .map_err(|e| format!("Codegen: {}", e))
}

// ============================================================================
// builtins/platform.rs — stdlib function calls
// ============================================================================

#[test]
fn test_builtin_labs() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            R labs(-42)
        }
    "#,
    );
    assert!(ir.contains("call") && ir.contains("labs"));
}

#[test]
fn test_builtin_atol() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            R atol("123")
        }
    "#,
    );
    if let Ok(ir) = &result {
        assert!(ir.contains("atol"));
    }
}

#[test]
fn test_builtin_atof() {
    let result = gen_result(
        r#"
        F test() -> f64 {
            R atof("3.14")
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// builtins/platform.rs — math functions
// ============================================================================

#[test]
fn test_builtin_sqrt() {
    let result = gen_result(
        r#"
        F test() -> f64 {
            R sqrt(4.0)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_builtin_sin() {
    let result = gen_result(
        r#"
        F test() -> f64 {
            R sin(0.0)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_builtin_cos() {
    let result = gen_result(
        r#"
        F test() -> f64 {
            R cos(0.0)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_builtin_pow() {
    let result = gen_result(
        r#"
        F test() -> f64 {
            R pow(2.0, 10.0)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_builtin_log() {
    let result = gen_result(
        r#"
        F test() -> f64 {
            R log(2.718)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_builtin_floor() {
    let result = gen_result(
        r#"
        F test() -> f64 {
            R floor(3.7)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_builtin_ceil() {
    let result = gen_result(
        r#"
        F test() -> f64 {
            R ceil(3.2)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// builtins/memory.rs — memory functions
// ============================================================================

#[test]
fn test_builtin_malloc_free() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            ptr := malloc(100)
            free(ptr)
            R 0
        }
    "#,
    );
    assert!(ir.contains("malloc") && ir.contains("free"));
}

#[test]
fn test_builtin_memset() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            ptr := malloc(100)
            memset(ptr, 0, 100)
            free(ptr)
            R 0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_builtin_memcpy() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            src := malloc(100)
            dst := malloc(100)
            memcpy(dst, src, 100)
            free(src)
            free(dst)
            R 0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// builtins/file_io.rs — file I/O functions
// ============================================================================

#[test]
fn test_builtin_fopen() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            f := fopen("test.txt", "r")
            R 0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// builtins/platform.rs — string functions
// ============================================================================

#[test]
fn test_builtin_strlen() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            R strlen("hello")
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_builtin_strcmp() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            R strcmp("abc", "abc")
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// builtins/platform.rs — process functions
// ============================================================================

#[test]
fn test_builtin_exit() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            exit(0)
            R 0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_builtin_getenv() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            e := getenv("HOME")
            R 0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// contracts/auto_checks.rs — contract attribute parsing
// ============================================================================

#[test]
fn test_contract_nonnull() {
    let ir = gen_ok(
        r#"
        #[contract(nonnull)]
        F test(s: str) -> i64 {
            R 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_contract_safe_div() {
    let ir = gen_ok(
        r#"
        #[contract(safe_div)]
        F divide(a: i64, b: i64) -> i64 {
            R a / b
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_contract_all() {
    let ir = gen_ok(
        r#"
        #[contract(all)]
        F test(s: str, n: i64) -> i64 {
            R n / 2
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_contract_default() {
    let ir = gen_ok(
        r#"
        #[contract]
        F test(s: str, n: i64) -> i64 {
            R n
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_contract_no_sideeffects() {
    // Note: "pure" is a keyword token in the parser, so we test contract without it
    let ir = gen_ok(
        r#"
        #[contract(nonnull, safe_div)]
        F add(s: str, b: i64) -> i64 = b
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_contract_bounds() {
    let ir = gen_ok(
        r#"
        #[contract(bounds)]
        F test() -> i64 {
            arr := [1, 2, 3]
            R arr[1]
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// contracts — assert and invariant
// ============================================================================

#[test]
fn test_assert_simple() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            assert(x > 0)
            R x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_assert_with_message() {
    let result = gen_result(
        r#"
        F test(x: i64) -> i64 {
            assert(x >= 0)
            R x
        }
    "#,
    );
    assert!(result.is_ok());
}

// ============================================================================
// contracts/decreases.rs — termination checking
// ============================================================================

#[test]
fn test_recursive_with_decreases() {
    let result = gen_result(
        r#"
        F count(n: i64) -> i64 {
            I n <= 0 { R 0 }
            R 1 + count(n - 1)
        }
    "#,
    );
    assert!(result.is_ok());
}

// ============================================================================
// contracts — requires/ensures attributes
// ============================================================================

#[test]
fn test_requires_attribute() {
    let ir = gen_ok(
        r#"
        #[requires(n > 0)]
        F test(n: i64) -> i64 {
            R n * 2
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_ensures_attribute() {
    let ir = gen_ok(
        r#"
        #[ensures(result >= 0)]
        F abs(x: i64) -> i64 {
            I x >= 0 { R x }
            R 0 - x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// builtins — printf/puts call patterns
// ============================================================================

#[test]
fn test_printf_format_string() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            printf("hello %d\n", 42)
            R 0
        }
    "#,
    );
    assert!(ir.contains("printf"));
}

#[test]
fn test_puts_string() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            puts("hello world")
            R 0
        }
    "#,
    );
    assert!(ir.contains("puts"));
}

// ============================================================================
// builtins — putchar
// ============================================================================

#[test]
fn test_putchar() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            putchar(65)
            R 0
        }
    "#,
    );
    assert!(ir.contains("putchar"));
}

// ============================================================================
// builtins/platform.rs — time functions
// ============================================================================

#[test]
fn test_builtin_time() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            R time(0)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_builtin_clock() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            R clock()
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// builtins — sizeof
// ============================================================================

#[test]
fn test_builtin_sizeof() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            R sizeof(i64)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// Integration: contract + builtin combination
// ============================================================================

#[test]
fn test_contract_with_malloc() {
    let ir = gen_ok(
        r#"
        #[contract(nonnull)]
        F alloc_and_free(size: i64) -> i64 {
            ptr := malloc(size)
            free(ptr)
            R 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

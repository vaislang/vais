//! Coverage tests for vais-types/src/builtins/
//!
//! Targets: core.rs, print.rs, memory.rs, stdlib.rs, file_io.rs, simd.rs,
//! gc.rs, system.rs, io.rs, math.rs, enum_builtins.rs
//!
//! Strategy: Create TypeChecker, verify builtins are registered correctly,
//! and type check programs that use builtin functions.

use vais_parser::parse;
use vais_types::TypeChecker;

fn check_ok(source: &str) {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut tc = TypeChecker::new();
    tc.check_module(&module)
        .unwrap_or_else(|e| panic!("Type check failed for: {}\nErr: {:?}", source, e));
}

#[allow(dead_code)]
fn check_err(source: &str) {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut tc = TypeChecker::new();
    assert!(
        tc.check_module(&module).is_err(),
        "Expected type error for: {}",
        source
    );
}

// ============================================================================
// core.rs — printf, puts, putchar, malloc, free
// ============================================================================

#[test]
fn test_builtin_printf() {
    check_ok(
        r#"
        F test() -> i64 {
            printf("hello %d\n", 42)
            R 0
        }
    "#,
    );
}

#[test]
fn test_builtin_puts() {
    check_ok(
        r#"
        F test() -> i64 {
            puts("hello")
            R 0
        }
    "#,
    );
}

#[test]
fn test_builtin_putchar() {
    check_ok(
        r#"
        F test() -> i64 {
            putchar(65)
            R 0
        }
    "#,
    );
}

#[test]
fn test_builtin_malloc() {
    check_ok(
        r#"
        F test() -> i64 {
            ptr := malloc(100)
            R ptr
        }
    "#,
    );
}

#[test]
fn test_builtin_free() {
    check_ok(
        r#"
        F test() -> i64 {
            ptr := malloc(100)
            free(ptr)
            R 0
        }
    "#,
    );
}

// ============================================================================
// print.rs — println, print
// ============================================================================

#[test]
fn test_builtin_println_string() {
    check_ok(
        r#"
        F test() -> i64 {
            println("hello world")
            R 0
        }
    "#,
    );
}

#[test]
fn test_builtin_println_format() {
    check_ok(
        r#"
        F test() -> i64 {
            println("value: %d", 42)
            R 0
        }
    "#,
    );
}

#[test]
fn test_builtin_print_string() {
    check_ok(
        r#"
        F test() -> i64 {
            print("hello")
            R 0
        }
    "#,
    );
}

// ============================================================================
// memory.rs — memset, memcpy, memmove, realloc, calloc
// ============================================================================

#[test]
fn test_builtin_memcpy() {
    check_ok(
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
}

#[test]
fn test_builtin_memcmp() {
    check_ok(
        r#"
        F test() -> i64 {
            a := malloc(100)
            b := malloc(100)
            result := memcmp(a, b, 100)
            free(a)
            free(b)
            R result
        }
    "#,
    );
}

// ============================================================================
// stdlib.rs — abs, min, max, clamp, swap
// ============================================================================

#[test]
fn test_builtin_labs() {
    check_ok(
        r#"
        F test() -> i64 {
            R labs(-42)
        }
    "#,
    );
}

// ============================================================================
// file_io.rs — fopen, fclose, fread, fwrite, fprintf
// ============================================================================

#[test]
fn test_builtin_fopen_fclose() {
    check_ok(
        r#"
        F test() -> i64 {
            f := fopen("test.txt", "r")
            fclose(f)
            R 0
        }
    "#,
    );
}

// ============================================================================
// math.rs — sin, cos, sqrt, pow, log, floor, ceil, exp, tan, atan2
// ============================================================================

#[test]
fn test_builtin_sin() {
    check_ok(
        r#"
        F test() -> f64 {
            R sin(0.0)
        }
    "#,
    );
}

#[test]
fn test_builtin_cos() {
    check_ok(
        r#"
        F test() -> f64 {
            R cos(0.0)
        }
    "#,
    );
}

#[test]
fn test_builtin_sqrt() {
    check_ok(
        r#"
        F test() -> f64 {
            R sqrt(4.0)
        }
    "#,
    );
}

#[test]
fn test_builtin_pow() {
    check_ok(
        r#"
        F test() -> f64 {
            R pow(2.0, 10.0)
        }
    "#,
    );
}

#[test]
fn test_builtin_log() {
    check_ok(
        r#"
        F test() -> f64 {
            R log(1.0)
        }
    "#,
    );
}

#[test]
fn test_builtin_floor() {
    check_ok(
        r#"
        F test() -> f64 {
            R floor(3.7)
        }
    "#,
    );
}

#[test]
fn test_builtin_ceil() {
    check_ok(
        r#"
        F test() -> f64 {
            R ceil(3.2)
        }
    "#,
    );
}

#[test]
fn test_builtin_exp() {
    check_ok(
        r#"
        F test() -> f64 {
            R exp(1.0)
        }
    "#,
    );
}

#[test]
fn test_builtin_tan() {
    check_ok(
        r#"
        F test() -> f64 {
            R tan(0.0)
        }
    "#,
    );
}

#[test]
fn test_builtin_atan2() {
    check_ok(
        r#"
        F test() -> f64 {
            R atan2(1.0, 1.0)
        }
    "#,
    );
}

// ============================================================================
// system.rs — exit, getenv, system, time, clock
// ============================================================================

#[test]
fn test_builtin_exit() {
    // `exit(code)` is intentional termination, not a panic. Callers can
    // stay `total`. Phase 196 P196-D removed exit from PANIC_BUILTINS.
    check_ok(
        r#"
        F test() -> i64 {
            exit(0)
            R 0
        }
    "#,
    );
}

#[test]
fn test_builtin_getenv() {
    check_ok(
        r#"
        F test() -> i64 {
            e := getenv("HOME")
            R 0
        }
    "#,
    );
}

#[test]
fn test_builtin_system() {
    check_ok(
        r#"
        F test() -> i64 {
            result := system("echo hi")
            R 0
        }
    "#,
    );
}

#[test]
fn test_builtin_raise() {
    check_ok(
        r#"
        F test() -> i64 {
            raise(0)
            R 0
        }
    "#,
    );
}

// ============================================================================
// io.rs — read, write file operations
// ============================================================================

#[test]
fn test_builtin_fread() {
    check_ok(
        r#"
        F test() -> i64 {
            f := fopen("test.txt", "r")
            buf := malloc(100)
            n := fread(buf, 1, 100, f)
            fclose(f)
            free(buf)
            R n
        }
    "#,
    );
}

#[test]
fn test_builtin_fwrite() {
    check_ok(
        r#"
        F test() -> i64 {
            f := fopen("test.txt", "w")
            buf := malloc(100)
            n := fwrite(buf, 1, 100, f)
            fclose(f)
            free(buf)
            R n
        }
    "#,
    );
}

// ============================================================================
// gc.rs — GC functions
// ============================================================================

#[test]
fn test_builtin_gc_alloc() {
    check_ok(
        r#"
        F test() -> i64 {
            ptr := gc_alloc(100, 1)
            R ptr
        }
    "#,
    );
}

#[test]
fn test_builtin_gc_collect() {
    check_ok(
        r#"
        F test() -> i64 {
            gc_init()
            gc_collect()
            R 0
        }
    "#,
    );
}

// ============================================================================
// enum_builtins.rs — Option/Result type checking
// ============================================================================

#[test]
fn test_option_some_none() {
    check_ok(
        r#"
        E MyOption { Some(i64), None }
        F test() -> i64 {
            x := Some(42)
            M x {
                Some(v) => v,
                None => 0,
                _ => -1
            }
        }
    "#,
    );
}

#[test]
fn test_result_ok_err() {
    check_ok(
        r#"
        E MyResult { Ok(i64), Err(i64) }
        F test() -> i64 {
            x := Ok(42)
            M x {
                Ok(v) => v,
                Err(e) => e,
                _ => -1
            }
        }
    "#,
    );
}

// ============================================================================
// simd.rs — SIMD type checking
// ============================================================================

#[test]
fn test_builtin_load_store() {
    check_ok(
        r#"
        F test() -> i64 {
            ptr := malloc(8)
            store_i64(ptr, 42)
            v := load_i64(ptr)
            free(ptr)
            R v
        }
    "#,
    );
}

// ============================================================================
// Integration: multiple builtins in one function
// ============================================================================

#[test]
fn test_multiple_builtins() {
    check_ok(
        r#"
        F test() -> i64 {
            ptr := malloc(256)
            println("allocated buffer")
            free(ptr)
            R 0
        }
    "#,
    );
}

// ============================================================================
// core.rs — sizeof
// ============================================================================

#[test]
fn test_builtin_sizeof() {
    check_ok(
        r#"
        F test() -> i64 {
            x := 42
            R sizeof(x)
        }
    "#,
    );
}

// ============================================================================
// stdlib.rs — atol, atof string conversion
// ============================================================================

#[test]
fn test_builtin_atol() {
    check_ok(
        r#"
        F test() -> i64 {
            R atol("123")
        }
    "#,
    );
}

#[test]
fn test_builtin_atof() {
    check_ok(
        r#"
        F test() -> f64 {
            R atof("3.14")
        }
    "#,
    );
}

// ============================================================================
// stdlib.rs — strlen, strcmp
// ============================================================================

#[test]
fn test_builtin_strlen() {
    check_ok(
        r#"
        F test() -> i64 {
            R strlen("hello")
        }
    "#,
    );
}

#[test]
fn test_builtin_str_to_ptr() {
    check_ok(
        r#"
        F test() -> i64 {
            R str_to_ptr("hello")
        }
    "#,
    );
}

// ============================================================================
// Phase 24 Task 5 — .iter() / .enumerate() on iterable receivers + tuple
// destructuring in for-each loops. Verifies the type checker alone (not codegen).
// Uses array literals instead of Vec to avoid depending on std/vec.vais, which
// e2e/unit tests do not load. Array<T> is recognized by get_iterator_item_type.
// ============================================================================

#[test]
fn test_phase24_array_iter_is_noop_at_type_level() {
    // .iter() on [T] (Array<T>) returns the receiver, item type is still T.
    check_ok(
        r#"
        F test() -> i64 {
            arr := [10, 20, 30]
            sum := mut 0
            LF x: arr.iter() {
                sum = sum + x
            }
            sum
        }
    "#,
    );
}

#[test]
fn test_phase24_array_enumerate_returns_tuple_iterator() {
    // .enumerate() yields (i64, T) — tuple destructuring in LF must bind both.
    check_ok(
        r#"
        F test() -> i64 {
            arr := [10, 20, 30]
            sum := mut 0
            LF (i, x): arr.enumerate() {
                sum = sum + i * x
            }
            sum
        }
    "#,
    );
}

#[test]
fn test_phase24_array_iter_enumerate_chain() {
    // .iter().enumerate() is the canonical Rust-style form — both yield (i64, T).
    check_ok(
        r#"
        F test() -> i64 {
            arr := [100, 200]
            sum := mut 0
            LF (i, x): arr.iter().enumerate() {
                sum = sum + i + x
            }
            sum
        }
    "#,
    );
}

#[test]
fn test_phase24_enumerate_bindings_are_usable_as_i64() {
    // Index binding must unify with i64 in arithmetic.
    check_ok(
        r#"
        F test() -> i64 {
            arr := [5, 15, 25]
            acc := mut 0
            LF (idx, val): arr.enumerate() {
                doubled := idx * 2
                acc = acc + doubled + val
            }
            acc
        }
    "#,
    );
}

#[test]
fn test_phase24_tuple_pattern_in_foreach_generally() {
    // Regression: Pattern::Tuple binding in for-each should also work for
    // any tuple-yielding iterator, not just .enumerate(). This exercises the
    // register_pattern_bindings path in control_flow.rs independently.
    check_ok(
        r#"
        F pairs() -> i64 {
            arr := [1, 2, 3]
            total := mut 0
            LF (i, x): arr.enumerate() {
                total = total + x - i
            }
            total
        }
    "#,
    );
}

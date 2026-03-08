// REGRESSION(phase-110): heap allocations must be auto-freed before function return
//! Phase 110: Memory management — scope-based auto free
//!
//! Tests that heap allocations (string concat, slices, format buffers, trait objects)
//! are automatically freed at function scope exit via the alloc_tracker mechanism.

use crate::helpers::{compile_and_run, compile_to_ir};

// ============================================================
// IR verification — auto-free comments and free calls present
// ============================================================

#[test]
fn test_auto_free_ir_for_string_concat() {
    let ir = compile_to_ir(
        r#"
F main() -> i64 {
    x := "a" + "b"
    R 0
}
"#,
    )
    .unwrap();
    // The auto-free cleanup should emit free calls for tracked allocations
    let main_start = ir.find("define i64 @main").expect("main function not found");
    let main_end = ir[main_start..].find("\n}\n").unwrap() + main_start;
    let main_ir = &ir[main_start..main_end];
    assert!(
        main_ir.contains("auto-free tracked allocations"),
        "Expected auto-free comment in main function IR:\n{}",
        main_ir
    );
    assert!(
        main_ir.contains("call void @free"),
        "Expected free call in main function IR:\n{}",
        main_ir
    );
}

#[test]
fn test_simple_format_no_malloc() {
    // Simple format like println("{x}") with integers compiles to printf("%ld\n", x)
    // which does NOT use malloc. This verifies no spurious auto-free for simple formats.
    let ir = compile_to_ir(
        r#"
F main() -> i64 {
    x := 10
    println("{x}")
    R 0
}
"#,
    )
    .unwrap();
    let main_start = ir.find("define i64 @main").expect("main function not found");
    let main_end = ir[main_start..].find("\n}\n").unwrap() + main_start;
    let main_ir = &ir[main_start..main_end];
    // Simple integer format goes through printf directly, no malloc needed
    assert!(
        !main_ir.contains("auto-free"),
        "Simple format should NOT need auto-free:\n{}",
        main_ir
    );
}

#[test]
fn test_auto_free_ir_for_slice() {
    let ir = compile_to_ir(
        r#"
F main() -> i64 {
    arr := [10, 20, 30, 40, 50]
    s := arr[1..3]
    R 0
}
"#,
    )
    .unwrap();
    let main_start = ir.find("define i64 @main").expect("main function not found");
    let main_end = ir[main_start..].find("\n}\n").unwrap() + main_start;
    let main_ir = &ir[main_start..main_end];
    assert!(
        main_ir.contains("auto-free tracked allocations"),
        "Expected auto-free comment for slice allocation:\n{}",
        main_ir
    );
}

#[test]
fn test_no_alloc_no_free() {
    let ir = compile_to_ir(
        r#"
F add(a: i64, b: i64) -> i64 {
    R a + b
}

F main() -> i64 {
    R add(1, 2)
}
"#,
    )
    .unwrap();
    // Pure arithmetic function should not have auto-free cleanup
    let add_fn_start = ir.find("define i64 @add(").unwrap();
    let add_fn_end = ir[add_fn_start..].find("\n}\n").unwrap() + add_fn_start;
    let add_fn_ir = &ir[add_fn_start..add_fn_end];
    assert!(
        !add_fn_ir.contains("auto-free"),
        "Pure function should not have auto-free cleanup"
    );
}

#[test]
fn test_multiple_allocs_freed() {
    let ir = compile_to_ir(
        r#"
F main() -> i64 {
    a := "x" + "y"
    b := "p" + "q"
    R 0
}
"#,
    )
    .unwrap();
    let main_start = ir.find("define i64 @main(").unwrap();
    let main_end = ir[main_start..].find("\n}\n").unwrap() + main_start;
    let main_ir = &ir[main_start..main_end];
    let free_count = main_ir.matches("call void @free").count();
    assert!(
        free_count >= 2,
        "Expected at least 2 free calls for 2 string concats, got {}. IR:\n{}",
        free_count,
        main_ir
    );
}

// ============================================================
// Runtime tests — programs with auto-free run correctly
// ============================================================

#[test]
fn test_string_concat_runs() {
    let result = compile_and_run(
        r#"
F main() -> i64 {
    x := "hello" + " " + "world"
    R 0
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_string_concat_output() {
    let result = compile_and_run(
        r#"
F main() -> i64 {
    x := "abc" + "def"
    println(x)
    R 0
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout.trim(), "abcdef");
}

#[test]
fn test_format_string_runs() {
    let result = compile_and_run(
        r#"
F main() -> i64 {
    x := 42
    println("value: {x}")
    R 0
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout.trim(), "value: 42");
}

#[test]
fn test_early_return_with_alloc() {
    let result = compile_and_run(
        r#"
F helper(x: i64) -> i64 {
    s := "hello" + " world"
    I x > 0 {
        R 1
    }
    R 0
}

F main() -> i64 {
    R helper(5)
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 1);
}

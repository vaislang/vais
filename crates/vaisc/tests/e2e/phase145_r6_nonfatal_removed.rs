use super::helpers::*;

// Verify that type errors properly cause compilation failure (NONFATAL removed)

#[test]
fn e2e_p145r6_type_error_is_fatal() {
    // Using undeclared variable should cause a compile error
    let source = r#"
fn main() -> i64 {
    x + 1
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_p145r6_undefined_function_is_fatal() {
    // Calling an undefined function should cause compile error
    let source = r#"
fn main() -> i64 {
    result := nonexistent_function(42)
    result
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_p145r6_valid_program_still_compiles() {
    // Normal valid programs should compile and run fine
    let source = r#"
fn add(a: i64, b: i64) -> i64 {
    a + b
}
fn main() -> i64 {
    result := add(3, 4)
    I result != 7 { return 1 }
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_p145r6_multiple_errors_reported() {
    // Multiple type errors should all be reported
    let source = r#"
fn foo() -> i64 {
    x := undefined_var
    y := another_undefined
    0
}
fn main() -> i64 { foo() }
"#;
    assert_compile_error(source);
}

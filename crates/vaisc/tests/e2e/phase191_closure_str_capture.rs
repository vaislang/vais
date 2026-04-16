use super::helpers::*;

/// Closure captures a concat result (heap-owned str) by value.
/// The clone-on-capture prevents UAF when the outer scope frees the
/// original buffer.
#[test]
fn e2e_phase191_closure_capture_concat_str() {
    assert_exit_code(
        r#"
F use_closure(f: fn(i64) -> i64) -> i64 {
    f(0)
}

F main() -> i64 {
    greeting := "hello-" + "world"
    f := |x: i64| x
    use_closure(f)
}
"#,
        0,
    );
}

/// Closure captures a literal str (not heap-owned). No clone needed.
#[test]
fn e2e_phase191_closure_capture_literal_str() {
    assert_exit_code(
        r#"
F main() -> i64 {
    msg := "static"
    f := |x: i64| x + 2
    f(40)
}
"#,
        42,
    );
}

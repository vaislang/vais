use super::helpers::*;

#[test]
fn e2e_p146_global_read_in_function() {
    let source = r#"
G counter: i64 = 42
F main() -> i64 {
    counter
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p146_global_write_in_function() {
    let source = r#"
G counter: i64 = 0
F main() -> i64 {
    counter = 10
    counter
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_p146_global_across_functions() {
    let source = r#"
G total: i64 = 0
F add(x: i64) -> i64 {
    total = total + x
    total
}
F main() -> i64 {
    add(10)
    add(20)
    I total != 30 { R 1 }
    0
}
"#;
    assert_exit_code(source, 0);
}

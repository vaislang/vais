use vais_codegen::CodeGenerator;
use vais_parser::parse;

fn text_ir(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = CodeGenerator::new("text_enum_layout");
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("Codegen failed for: {}\nErr: {}", source, e))
}

fn marker_window(ir: &str, marker: &str, lines: usize) -> String {
    ir.split(marker)
        .nth(1)
        .unwrap_or_else(|| panic!("missing marker `{}` in IR:\n{}", marker, ir))
        .lines()
        .take(lines)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn text_option_unwrap_uses_declared_none_tag() {
    let ir = text_ir(
        r#"
E Option<T> {
    None,
    Some(T)
}

F main() -> i64 {
    x: Option<i64> = Some(42)
    R x!
}
"#,
    );

    assert!(
        ir.contains("%Option = type { i32, { i64 } }"),
        "Option must use the canonical text backend ABI:\n{}",
        ir
    );
    assert!(
        ir.contains("store i32 1, i32*") && ir.contains("store i64 42, i64*"),
        "Some must use declaration-order tag 1 when None is declared first:\n{}",
        ir
    );
    let unwrap_ir = marker_window(&ir, "; Unwrap expression", 5);
    assert!(
        unwrap_ir.contains("icmp eq i32") && unwrap_ir.contains(", 0"),
        "unwrap must compare against the declared None tag, not tag != 0:\n{}",
        unwrap_ir
    );
    assert!(
        !unwrap_ir.contains("icmp ne i32"),
        "unwrap must not assume nonzero tags are errors:\n{}",
        unwrap_ir
    );
}

#[test]
fn text_option_try_uses_declared_none_tag() {
    let ir = text_ir(
        r#"
E Option<T> {
    Some(T),
    None
}

F maybe() -> Option<i64> {
    R Some(7)
}

F main() -> Option<i64> {
    x := maybe()?
    R Some(x)
}
"#,
    );

    assert!(
        ir.contains("store i32 0, i32*") && ir.contains("store i64 7, i64*"),
        "Some must use declaration-order tag 0 when declared first:\n{}",
        ir
    );
    let try_ir = marker_window(&ir, "; Try expression (?)", 5);
    assert!(
        try_ir.contains("icmp eq i32") && try_ir.contains(", 1"),
        "try must propagate the declared None tag, not any nonzero tag:\n{}",
        try_ir
    );
}

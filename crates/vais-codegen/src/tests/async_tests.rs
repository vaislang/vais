use crate::CodeGenerator;

#[test]
fn test_async_basic_poll_ir() {
    use vais_types::TypeChecker;
    let source = r#"
A fn compute(x: i64) -> i64 {
    x + 10
}

fn main() -> i64 {
    result := compute(32).await
    result - 42
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut checker = TypeChecker::new();
    checker.check_module(&module).unwrap();
    let mut gen = CodeGenerator::new("test");
    gen.set_resolved_functions(checker.get_all_functions().clone());
    gen.set_type_aliases(checker.get_type_aliases().clone());
    let ir = gen.generate_module(&module).unwrap();
    assert!(
        ir.contains("@compute__poll"),
        "Expected compute__poll call:\n{}",
        ir
    );
}

#[test]
fn test_async_with_if_ir() {
    use vais_types::TypeChecker;
    let source = r#"
A fn conditional(x: i64) -> i64 {
    I x > 0 {
        return x * 2
    } else {
        return 0
    }
}

fn main() -> i64 {
    result := conditional(21).await
    result - 42
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut checker = TypeChecker::new();
    checker.check_module(&module).unwrap();
    let mut gen = CodeGenerator::new("test");
    gen.set_resolved_functions(checker.get_all_functions().clone());
    gen.set_type_aliases(checker.get_type_aliases().clone());
    let ir = gen.generate_module(&module).unwrap();
    assert!(
        ir.contains("@conditional__poll"),
        "Expected conditional__poll:\n{}",
        ir
    );
}

#[test]
fn test_async_bool_return_ir() {
    use vais_types::TypeChecker;
    let source = r#"
A fn check() -> bool {
    yield true
}

fn main() -> i64 {
    result := check().await
    I result { return 0 } else { return 1 }
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut checker = TypeChecker::new();
    checker.check_module(&module).unwrap();
    let mut gen = CodeGenerator::new("test");
    gen.set_resolved_functions(checker.get_all_functions().clone());
    gen.set_type_aliases(checker.get_type_aliases().clone());
    let ir = gen.generate_module(&module).unwrap();
    assert!(ir.contains("@check__poll"), "Expected check__poll:\n{}", ir);
}

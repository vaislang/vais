use crate::CodeGenerator;

#[test]
fn test_async_spawn_sync_await_ir() {
    use vais_types::TypeChecker;
    let source = r#"
F main() -> i64 {
    result := (spawn 42).await
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
        ir.contains("@__sync_spawn__poll"),
        "Expected __sync_spawn__poll call:\n{}",
        ir
    );
}

#[test]
fn test_async_basic_poll_ir() {
    use vais_types::TypeChecker;
    let source = r#"
A F compute(x: i64) -> i64 {
    x + 10
}

F main() -> i64 {
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
fn test_async_variable_await_ir() {
    use vais_types::TypeChecker;
    let source = r#"
A F compute(x: i64) -> i64 {
    x * 2
}

F main() -> i64 {
    future := spawn compute(21)
    result := future.await
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
        "Expected compute__poll in variable-based await:\n{}",
        ir
    );
}

#[test]
fn test_spawn_await_chain_ir() {
    use vais_types::TypeChecker;
    let source = r#"
A F compute(x: i64) -> i64 {
    x * 2
}

F main() -> i64 {
    result := (spawn compute(21)).await
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
        "Expected compute__poll for spawn+await chain:\n{}",
        ir
    );
}

#[test]
fn test_async_with_if_ir() {
    use vais_types::TypeChecker;
    let source = r#"
A F conditional(x: i64) -> i64 {
    I x > 0 {
        R x * 2
    } E {
        R 0
    }
}

F main() -> i64 {
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
A F check() -> bool {
    yield true
}

F main() -> i64 {
    result := check().await
    I result { R 0 } E { R 1 }
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

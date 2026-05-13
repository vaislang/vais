use crate::CodeGenerator;

#[test]
fn test_slice_len_codegen() {
    use vais_types::TypeChecker;
    let source = r#"
F baz(s: &[i64]) -> i64 {
s.len()
}
F main() -> i64 {
0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut checker = TypeChecker::new();
    checker.check_module(&module).unwrap();
    let mut gen = CodeGenerator::new("test");
    gen.set_resolved_functions(checker.get_all_functions().clone());
    gen.set_type_aliases(checker.get_type_aliases().clone());
    let ir = gen.generate_module(&module).unwrap();
    // The slice .len() should use extractvalue, NOT a call to @len
    assert!(
        ir.contains("extractvalue"),
        "Expected extractvalue in IR:\n{}",
        ir
    );
    assert!(
        !ir.contains("call i64 @len"),
        "Should not call @len:\n{}",
        ir
    );
    assert!(
        !ir.contains("call i64 @baz_len"),
        "Should not call @baz_len:\n{}",
        ir
    );
}

#[test]
fn test_slice_literal_fat_pointer_codegen() {
    use vais_types::TypeChecker;
    let source = r#"
F get_slice(arr: &[i64]) -> i64 = 42
F main() -> i64 = get_slice(&[1, 2, 3])
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut checker = TypeChecker::new();
    checker.check_module(&module).unwrap();
    let mut gen = CodeGenerator::new("test");
    gen.set_resolved_functions(checker.get_all_functions().clone());
    gen.set_type_aliases(checker.get_type_aliases().clone());
    let ir = gen.generate_module(&module).unwrap();
    // The slice literal &[1,2,3] should build a fat pointer via insertvalue
    assert!(
        ir.contains("insertvalue"),
        "Expected insertvalue in IR:\n{}",
        ir
    );
    assert!(ir.contains("bitcast"), "Expected bitcast in IR:\n{}", ir);
    assert!(
        ir.contains("{ i8*, i64 }"),
        "Expected fat pointer type:\n{}",
        ir
    );
}

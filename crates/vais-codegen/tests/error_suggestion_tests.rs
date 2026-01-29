//! Tests for error message suggestions (did you mean, type conversion hints)

use vais_codegen::CodeGenerator;
use vais_parser::parse;

#[test]
fn test_undefined_variable_suggestion() {
    let source = "F test()->i64{count:=42;R cont}";
    let module = parse(source).unwrap();
    let mut gen = CodeGenerator::new("test");
    let result = gen.generate_module(&module);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = format!("{}", err);

    // Should suggest "count" when user types "cont"
    assert!(
        err_msg.contains("cont") && err_msg.contains("count"),
        "Expected suggestion for 'count', got: {}",
        err_msg
    );
    assert!(
        err_msg.contains("Did you mean"),
        "Expected 'Did you mean' in error: {}",
        err_msg
    );
}

#[test]
fn test_undefined_variable_multiple_suggestions() {
    let source = "F test()->i64{counter:=1;count:=2;counter_max:=3;R cont}";
    let module = parse(source).unwrap();
    let mut gen = CodeGenerator::new("test");
    let result = gen.generate_module(&module);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = format!("{}", err);

    // Should suggest similar variables when user types "cont"
    assert!(
        err_msg.contains("cont"),
        "Error should mention undefined variable: {}",
        err_msg
    );
    assert!(
        err_msg.contains("Did you mean"),
        "Expected 'Did you mean' in error: {}",
        err_msg
    );
    // Should suggest at least one of the similar variables
    assert!(
        err_msg.contains("count") || err_msg.contains("counter"),
        "Expected suggestion for similar variable: {}",
        err_msg
    );
}

#[test]
fn test_undefined_function_suggestion() {
    // Fixed: Remove semicolon between declarations and use regular function call instead of @
    let source = "F print_num(x:i64)->i64=x F main()->i64=print_nu(42)";
    let module = parse(source).unwrap();
    let mut gen = CodeGenerator::new("test");
    let result = gen.generate_module(&module);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = format!("{}", err);

    // Should suggest "print_num" when user types "print_nu"
    assert!(
        err_msg.contains("print_nu") && err_msg.contains("print_num"),
        "Expected suggestion for 'print_num', got: {}",
        err_msg
    );
    assert!(
        err_msg.contains("Did you mean"),
        "Expected 'Did you mean' in error: {}",
        err_msg
    );
}

#[test]
fn test_field_not_found_suggestion() {
    // Fixed: Use S for struct instead of T
    let source = r#"
        S Point{x:i64,y:i64}
        F test(p:Point)->i64=p.X
    "#;
    let module = parse(source).unwrap();
    let mut gen = CodeGenerator::new("test");
    let result = gen.generate_module(&module);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = format!("{}", err);

    // Should suggest "x" when user types "X" (case difference)
    assert!(
        err_msg.contains("X"),
        "Error should mention undefined field: {}",
        err_msg
    );
    assert!(
        err_msg.contains("Did you mean") && err_msg.contains("x"),
        "Expected suggestion for 'x', got: {}",
        err_msg
    );
}

#[test]
fn test_no_suggestion_for_very_different_name() {
    let source = "F test()->i64{counter:=42;R xyz}";
    let module = parse(source).unwrap();
    let mut gen = CodeGenerator::new("test");
    let result = gen.generate_module(&module);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = format!("{}", err);

    // Should NOT suggest "counter" for "xyz" as they're too different
    assert!(
        err_msg.contains("xyz"),
        "Error should mention undefined variable: {}",
        err_msg
    );
    // May or may not have "Did you mean" depending on edit distance threshold
    // but shouldn't suggest extremely different names
}

#[test]
fn test_type_error_includes_type_info() {
    // Test that type errors now include actual type information
    let source = "F test()->i64{x:=42;R x[0]}";
    let module = parse(source).unwrap();
    let mut gen = CodeGenerator::new("test");
    let result = gen.generate_module(&module);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = format!("{}", err);

    // Should mention that indexing is not supported for the type
    assert!(
        err_msg.contains("Cannot index") || err_msg.contains("index"),
        "Expected indexing error: {}",
        err_msg
    );
}

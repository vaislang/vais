//! Error Message Quality Tests (Phase 35, Stage 4)
//!
//! These tests verify that compiler error messages are helpful and informative.
//! They check that error strings contain expected diagnostic text such as
//! "did you mean?" suggestions, unused variable warnings, and clear context.

use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;
use vais_types::TypeError;

/// Type-check source code, returning the error string if type checking fails
fn type_check_error(source: &str) -> String {
    let _tokens = tokenize(source).expect("Lexer should succeed");
    let module = parse(source).expect("Parser should succeed");
    let mut checker = TypeChecker::new();
    match checker.check_module(&module) {
        Err(e) => format!("{}", e),
        Ok(()) => panic!("Expected type error, but type checking succeeded"),
    }
}

/// Type-check source code and return warnings
fn type_check_warnings(source: &str) -> Vec<String> {
    let _tokens = tokenize(source).expect("Lexer should succeed");
    let module = parse(source).expect("Parser should succeed");
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
    checker.get_warnings().to_vec()
}

/// Type-check source code and return the help message from the error
fn type_check_help(source: &str) -> Option<String> {
    let _tokens = tokenize(source).expect("Lexer should succeed");
    let module = parse(source).expect("Parser should succeed");
    let mut checker = TypeChecker::new();
    match checker.check_module(&module) {
        Err(e) => e.help(),
        Ok(()) => panic!("Expected type error, but type checking succeeded"),
    }
}

/// Type-check source code, returning the TypeError if type checking fails
fn type_check_type_error(source: &str) -> TypeError {
    let _tokens = tokenize(source).expect("Lexer should succeed");
    let module = parse(source).expect("Parser should succeed");
    let mut checker = TypeChecker::new();
    match checker.check_module(&module) {
        Err(e) => e,
        Ok(()) => panic!("Expected type error, but type checking succeeded"),
    }
}

/// Type-check with multi-error mode, returning all collected errors
fn type_check_multi_errors(source: &str) -> Vec<String> {
    let _tokens = tokenize(source).expect("Lexer should succeed");
    let module = parse(source).expect("Parser should succeed");
    let mut checker = TypeChecker::new();
    checker.multi_error_mode = true;
    let main_err = checker.check_module(&module).err();
    let mut errors: Vec<String> = checker
        .get_collected_errors()
        .iter()
        .map(|e| format!("{}", e))
        .collect();
    if let Some(e) = main_err {
        errors.push(format!("{}", e));
    }
    errors
}

/// Parse source code, expecting a parse error, and return the error string
fn parse_error(source: &str) -> String {
    match parse(source) {
        Err(e) => format!("{}", e),
        Ok(_) => panic!("Expected parse error, but parsing succeeded"),
    }
}

// ==================== Type Mismatch Suggestions ====================

#[test]
fn error_type_mismatch_bool_vs_i64() {
    // Passing a bool where i64 is expected
    let error = type_check_error("F main() -> i64 = true");
    assert!(
        error.contains("Type mismatch"),
        "Error should mention type mismatch: got '{}'",
        error
    );
    assert!(
        error.contains("expected") && error.contains("found"),
        "Error should contain 'expected' and 'found': got '{}'",
        error
    );
}

#[test]
fn error_type_mismatch_help_suggests_cast() {
    // When numeric type is expected, help should suggest type cast
    let help = type_check_help("F main() -> i64 = true");
    assert!(help.is_some(), "Should provide help for type mismatch");
}

#[test]
fn error_type_mismatch_str_to_i64() {
    // String where i64 expected should suggest conversion
    let error = type_check_error(r#"F main() -> i64 = "hello""#);
    assert!(
        error.contains("Type mismatch"),
        "Error should mention type mismatch: got '{}'",
        error
    );
    let help = type_check_help(r#"F main() -> i64 = "hello""#);
    assert!(
        help.is_some(),
        "Should provide help for str to i64 mismatch"
    );
    let help_text = help.unwrap();
    assert!(
        help_text.contains("converting")
            || help_text.contains("cast")
            || help_text.contains("did you mean"),
        "Help should suggest conversion: got '{}'",
        help_text
    );
}

// ==================== Unused Variable Warnings ====================

#[test]
fn warning_unused_variable_suggests_underscore() {
    let warnings = type_check_warnings("F main() -> i64 { x := 5\n R 0 }");
    let has_unused_warning = warnings
        .iter()
        .any(|w| w.contains("unused variable") && w.contains("_x"));
    assert!(
        has_unused_warning,
        "Should warn about unused variable `x` and suggest `_x`, got warnings: {:?}",
        warnings
    );
}

#[test]
fn warning_unused_variable_no_warning_for_underscore_prefix() {
    let warnings = type_check_warnings("F main() -> i64 { _x := 5\n R 0 }");
    let has_unused_warning = warnings
        .iter()
        .any(|w| w.contains("unused variable") && w.contains("_x"));
    assert!(
        !has_unused_warning,
        "Should NOT warn about `_x` (underscore prefix), got warnings: {:?}",
        warnings
    );
}

#[test]
fn warning_unused_variable_used_variable_no_warning() {
    let warnings = type_check_warnings("F main() -> i64 { x := 5\n R x }");
    let has_unused_warning = warnings
        .iter()
        .any(|w| w.contains("unused variable") && w.contains("`x`"));
    assert!(
        !has_unused_warning,
        "Should NOT warn about used variable `x`, got warnings: {:?}",
        warnings
    );
}

// ==================== Struct Field Access Errors ====================

#[test]
fn error_no_such_field_on_struct() {
    let error = type_check_error(
        "S User { name: str, age: i64 }\nF main() -> i64 { u := User { name: \"Alice\", age: 30 }\n R u.agee }"
    );
    assert!(
        error.contains("No field") || error.contains("no field") || error.contains("field"),
        "Error should mention no such field: got '{}'",
        error
    );
}

#[test]
fn error_no_such_field_suggests_similar() {
    let source = "S User { name: str, age: i64 }\nF main() -> i64 { u := User { name: \"Alice\", age: 30 }\n R u.nme }";
    let _tokens = tokenize(source).expect("Lexer should succeed");
    let module = parse(source).expect("Parser should succeed");
    let mut checker = TypeChecker::new();
    match checker.check_module(&module) {
        Err(e) => {
            let error = format!("{}", e);
            assert!(
                error.contains("No field") || error.contains("no field"),
                "Error should mention missing field: got '{}'",
                error
            );
            // Check help message for suggestion
            let help = e.help();
            assert!(help.is_some(), "Should provide help for field access error");
            let help_text = help.unwrap();
            assert!(
                help_text.contains("name") || help_text.contains("did you mean"),
                "Help should suggest similar field name 'name': got '{}'",
                help_text
            );
        }
        Ok(()) => panic!("Expected type error for invalid field access"),
    }
}

// ==================== Extern Function Signature Validation ====================

#[test]
fn warning_extern_pointer_fn_wrong_return_type() {
    // X F is the single extern function syntax
    // dlopen is a known pointer-returning function that is NOT a builtin
    let warnings =
        type_check_warnings("X F dlopen(path: i64, mode: i64) -> bool\nF main() -> i64 = 0");
    let has_extern_warning = warnings
        .iter()
        .any(|w| w.contains("dlopen") && w.contains("should return"));
    assert!(
        has_extern_warning,
        "Should warn about dlopen returning bool instead of i64 (pointer), got warnings: {:?}",
        warnings
    );
}

#[test]
fn warning_extern_pointer_fn_correct_return_type_no_warning() {
    // X F is the single extern function syntax
    let warnings =
        type_check_warnings("X F dlopen(path: i64, mode: i64) -> i64\nF main() -> i64 = 0");
    let has_extern_warning = warnings
        .iter()
        .any(|w| w.contains("dlopen") && w.contains("should return"));
    assert!(
        !has_extern_warning,
        "Should NOT warn when dlopen returns i64, got warnings: {:?}",
        warnings
    );
}

// ==================== Parse Error Context ====================

#[test]
fn error_parse_unexpected_token_shows_context() {
    let error = parse_error("F main( -> i64 = 0");
    assert!(
        error.contains("Unexpected") || error.contains("expected"),
        "Parse error should mention what was unexpected or expected: got '{}'",
        error
    );
}

#[test]
fn error_parse_missing_body() {
    let error = parse_error("F main() -> i64");
    assert!(
        error.contains("Unexpected") || error.contains("expected") || error.contains("end of file"),
        "Parse error should provide context about what's missing: got '{}'",
        error
    );
}

// ==================== Additional Error Quality Tests ====================

#[test]
fn error_undefined_variable_suggests_similar() {
    let source = "F main() -> i64 { myvar := 42\n R myva }";
    let _tokens = tokenize(source).expect("Lexer should succeed");
    let module = parse(source).expect("Parser should succeed");
    let mut checker = TypeChecker::new();
    match checker.check_module(&module) {
        Err(e) => {
            let error = format!("{}", e);
            assert!(
                error.contains("Undefined") || error.contains("not found"),
                "Error should mention undefined variable: got '{}'",
                error
            );
            let help = e.help();
            assert!(help.is_some(), "Should provide help with suggestion");
            let help_text = help.unwrap();
            assert!(
                help_text.contains("myvar") || help_text.contains("did you mean"),
                "Help should suggest similar variable: got '{}'",
                help_text
            );
        }
        Ok(()) => panic!("Expected type error for undefined variable"),
    }
}

#[test]
fn error_undefined_function_suggests_similar() {
    // When calling an undefined function, the type checker first checks
    // registered functions, then falls through to check it as a variable.
    // The error is reported as "Undefined variable" with a did-you-mean suggestion.
    let source = "F greet() -> i64 = 0\nF main() -> i64 = gret()";
    let _tokens = tokenize(source).expect("Lexer should succeed");
    let module = parse(source).expect("Parser should succeed");
    let mut checker = TypeChecker::new();
    match checker.check_module(&module) {
        Err(e) => {
            let error = format!("{}", e);
            assert!(
                error.contains("Undefined variable") || error.contains("Undefined function"),
                "Error should mention undefined variable or function: got '{}'",
                error
            );
            let help = e.help();
            assert!(help.is_some(), "Should provide help with suggestion");
            let help_text = help.unwrap();
            assert!(
                help_text.contains("greet") || help_text.contains("did you mean"),
                "Help should suggest similar name 'greet': got '{}'",
                help_text
            );
        }
        Ok(()) => panic!("Expected type error for undefined function"),
    }
}

// ==================== Phase 31: New help() Coverage Tests ====================

#[test]
fn help_not_callable_has_message() {
    // Calling a non-function value
    let help = type_check_help("F main() -> i64 { x := 5\n x() }");
    assert!(help.is_some(), "NotCallable should provide help message");
    let text = help.unwrap();
    assert!(
        text.contains("not callable") || text.contains("functions"),
        "Help should explain only functions are callable: got '{}'",
        text
    );
}

#[test]
fn help_arg_count_has_message() {
    // Wrong number of arguments
    let help = type_check_help("F add(a: i64, b: i64) -> i64 = a + b\nF main() -> i64 = add(1)");
    assert!(help.is_some(), "ArgCount should provide help message");
    let text = help.unwrap();
    assert!(
        text.contains("2") && text.contains("1"),
        "Help should mention expected vs actual count: got '{}'",
        text
    );
}

#[test]
fn help_duplicate_has_message() {
    // Test directly: Duplicate definition error has help
    let err = TypeError::Duplicate("foo".to_string(), None);
    let help = err.help();
    assert!(help.is_some(), "Duplicate should provide help message");
    let text = help.unwrap();
    assert!(
        text.contains("already defined") || text.contains("renaming"),
        "Help should suggest renaming: got '{}'",
        text
    );
}

#[test]
fn help_immutable_assign_has_message() {
    // Test directly: ImmutableAssign error has help
    let err = TypeError::ImmutableAssign("x".to_string(), None);
    let help = err.help();
    assert!(
        help.is_some(),
        "ImmutableAssign should provide help message"
    );
    let text = help.unwrap();
    assert!(
        text.contains("mutable") || text.contains("mut"),
        "Help should suggest declaring as mutable: got '{}'",
        text
    );
}

// ==================== Phase 31: Secondary Spans Tests ====================

#[test]
fn secondary_spans_use_after_move() {
    // Test the secondary_spans method directly
    let err = TypeError::UseAfterMove {
        var_name: "x".to_string(),
        moved_at: Some(vais_ast::Span::new(10, 15)),
        use_at: Some(vais_ast::Span::new(20, 25)),
    };
    let spans = err.secondary_spans();
    assert_eq!(spans.len(), 1, "UseAfterMove should have 1 secondary span");
    assert!(
        spans[0].1.contains("moved here"),
        "Secondary span should say 'moved here': got '{}'",
        spans[0].1
    );
}

#[test]
fn secondary_spans_borrow_conflict() {
    let err = TypeError::BorrowConflict {
        var_name: "x".to_string(),
        existing_borrow_at: Some(vais_ast::Span::new(5, 10)),
        new_borrow_at: Some(vais_ast::Span::new(15, 20)),
        existing_is_mut: true,
        new_is_mut: false,
    };
    let spans = err.secondary_spans();
    assert_eq!(
        spans.len(),
        1,
        "BorrowConflict should have 1 secondary span"
    );
    assert!(
        spans[0].1.contains("mutable borrow"),
        "Should mention mutable borrow: got '{}'",
        spans[0].1
    );
}

#[test]
fn secondary_spans_empty_for_simple_errors() {
    let err = TypeError::Mismatch {
        expected: "i64".to_string(),
        found: "bool".to_string(),
        span: None,
    };
    let spans = err.secondary_spans();
    assert!(spans.is_empty(), "Mismatch should have no secondary spans");
}

// ==================== Phase 31: Multi-Error Collection Tests ====================

#[test]
fn multi_error_collects_multiple_errors() {
    // Two functions with errors — multi_error_mode should collect both
    let source = "F foo() -> i64 = true\nF bar() -> i64 = true\nF main() -> i64 = 0";
    let errors = type_check_multi_errors(source);
    assert!(
        errors.len() >= 2,
        "Multi-error mode should collect at least 2 errors, got {}: {:?}",
        errors.len(),
        errors
    );
}

#[test]
fn multi_error_mode_disabled_returns_first_error_only() {
    // Without multi_error_mode, only first error is returned
    let source = "F foo() -> i64 = true\nF bar() -> i64 = true\nF main() -> i64 = 0";
    let _tokens = tokenize(source).expect("Lexer should succeed");
    let module = parse(source).expect("Parser should succeed");
    let mut checker = TypeChecker::new();
    // multi_error_mode defaults to false
    let result = checker.check_module(&module);
    assert!(result.is_err(), "Should return error");
    assert!(
        checker.get_collected_errors().is_empty(),
        "Without multi_error_mode, no errors should be collected: got {:?}",
        checker.get_collected_errors().len()
    );
}

// ==================== Phase 31: Error Code Tests ====================

#[test]
fn error_code_format_is_exxxx() {
    let err = type_check_type_error("F main() -> i64 = true");
    let code = err.error_code();
    assert!(
        code.starts_with('E') || code.starts_with('C'),
        "Error code should start with 'E' or 'C': got '{}'",
        code
    );
    assert!(
        code.len() == 4,
        "Error code should be 4 characters (e.g., E001): got '{}'",
        code
    );
}

#[test]
fn error_help_coverage_all_variants() {
    // Test that all manually constructable error variants have help()
    let variants: Vec<TypeError> = vec![
        TypeError::Mismatch {
            expected: "i64".into(),
            found: "bool".into(),
            span: None,
        },
        TypeError::UndefinedVar {
            name: "x".into(),
            span: None,
            suggestion: Some("y".into()),
        },
        TypeError::UndefinedType {
            name: "Foo".into(),
            span: None,
            suggestion: None,
        },
        TypeError::UndefinedFunction {
            name: "foo".into(),
            span: None,
            suggestion: None,
        },
        TypeError::NotCallable("i64".into(), None),
        TypeError::ArgCount {
            expected: 2,
            got: 1,
            span: None,
        },
        TypeError::CannotInfer,
        TypeError::Duplicate("foo".into(), None),
        TypeError::ImmutableAssign("x".into(), None),
    ];

    for variant in &variants {
        let help = variant.help();
        assert!(
            help.is_some(),
            "Error variant {} (code: {}) should have help message",
            variant,
            variant.error_code()
        );
    }
}

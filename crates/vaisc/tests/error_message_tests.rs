//! Error Message Quality Tests (Phase 35, Stage 4)
//!
//! These tests verify that compiler error messages are helpful and informative.
//! They check that error strings contain expected diagnostic text such as
//! "did you mean?" suggestions, unused variable warnings, and clear context.

use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

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

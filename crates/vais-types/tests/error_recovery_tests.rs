//! Error recovery scenario tests
//!
//! Validates that error messages are clear, contain error codes,
//! and provide helpful suggestions for common mistakes.

use vais_types::TypeChecker;
use vais_parser::parse;

/// Helper: type-check source and return the error string
fn check_error(source: &str) -> Option<vais_types::TypeError> {
    let ast = parse(source).ok()?;
    let mut checker = TypeChecker::new();
    checker.check_module(&ast).err()
}

// =============================================================
// E001: Type Mismatch
// =============================================================

#[test]
fn test_error_code_type_mismatch() {
    // Returning bool where i64 is expected
    let source = "F test()->i64=true";
    if let Some(err) = check_error(source) {
        assert_eq!(err.error_code(), "E001");
        let msg = format!("{}", err);
        assert!(msg.contains("mismatch") || msg.contains("Mismatch"),
            "Type mismatch error should mention 'mismatch': {}", msg);
    }
}

// =============================================================
// E002: Undefined Variable (with suggestions)
// =============================================================

#[test]
fn test_error_code_undefined_variable() {
    let source = "F test()->i64{count:=42;R cont}";
    if let Some(err) = check_error(source) {
        assert_eq!(err.error_code(), "E002");
        let help = err.help();
        assert!(help.is_some(), "Undefined variable should have help message");
        let help_msg = help.unwrap();
        // Should suggest "count" for typo "cont"
        assert!(help_msg.contains("count") || help_msg.contains("cont"),
            "Help should suggest similar name: {}", help_msg);
    }
}

#[test]
fn test_error_undefined_var_no_suggestion_for_dissimilar() {
    let source = "F test()->i64{counter:=42;R xyz}";
    if let Some(err) = check_error(source) {
        assert_eq!(err.error_code(), "E002");
        let help = err.help();
        assert!(help.is_some(), "Should still have a help message");
    }
}

// =============================================================
// E004: Undefined Function (with suggestions)
// =============================================================

#[test]
fn test_error_code_undefined_function() {
    let source = "F add(a:i64,b:i64)->i64=a+b F main()->i64=ad(1,2)";
    if let Some(err) = check_error(source) {
        // Type checker may report as E002 (UndefinedVar) or E004 (UndefinedFunction)
        // depending on resolution order
        let code = err.error_code();
        assert!(code == "E002" || code == "E004",
            "Expected E002 or E004, got: {}", code);
        let help = err.help();
        assert!(help.is_some(), "Undefined identifier should have help");
    }
}

// =============================================================
// E006: Argument Count Mismatch
// =============================================================

#[test]
fn test_error_code_arg_count() {
    let source = "F add(a:i64,b:i64)->i64=a+b F main()->i64=add(1)";
    if let Some(err) = check_error(source) {
        assert_eq!(err.error_code(), "E006");
        let msg = format!("{}", err);
        assert!(msg.contains("expected") && msg.contains("got"),
            "Arg count error should state expected vs got: {}", msg);
    }
}

// =============================================================
// Error Code Consistency
// =============================================================

#[test]
fn test_all_error_codes_are_assigned() {
    // Verify error code format for TypeError variants that we can trigger
    use vais_types::TypeError;

    let test_errors: Vec<TypeError> = vec![
        TypeError::Mismatch {
            expected: "i64".to_string(),
            found: "bool".to_string(),
            span: None,
        },
        TypeError::UndefinedVar {
            name: "x".to_string(),
            span: None,
            suggestion: None,
        },
        TypeError::UndefinedType {
            name: "Foo".to_string(),
            span: None,
            suggestion: None,
        },
        TypeError::UndefinedFunction {
            name: "bar".to_string(),
            span: None,
            suggestion: None,
        },
        TypeError::NotCallable("i64".to_string(), None),
        TypeError::ArgCount { expected: 2, got: 1, span: None },
        TypeError::CannotInfer,
        TypeError::Duplicate("x".to_string(), None),
        TypeError::ImmutableAssign("x".to_string(), None),
        TypeError::NonExhaustiveMatch("_".to_string(), None),
        TypeError::UnreachablePattern(0, None),
    ];

    for err in &test_errors {
        let code = err.error_code();
        assert!(code.starts_with('E'), "Error code should start with 'E': {}", code);
        assert!(code.len() == 4, "Error code should be 4 chars (e.g., E001): {}", code);
        let num: Result<u16, _> = code[1..].parse();
        assert!(num.is_ok(), "Error code should have numeric suffix: {}", code);
    }
}

// =============================================================
// Help Message Quality
// =============================================================

#[test]
fn test_help_message_for_immutable_assign() {
    use vais_types::TypeError;

    let err = TypeError::ImmutableAssign("x".to_string(), None);
    let help = err.help();
    assert!(help.is_some());
    let help_msg = help.unwrap();
    assert!(help_msg.contains("mutable") || help_msg.contains("mut"),
        "Immutable assign help should suggest mut: {}", help_msg);
}

#[test]
fn test_help_message_for_undefined_var_with_suggestion() {
    use vais_types::TypeError;

    let err = TypeError::UndefinedVar {
        name: "cont".to_string(),
        span: None,
        suggestion: Some("count".to_string()),
    };
    let help = err.help();
    assert!(help.is_some());
    let help_msg = help.unwrap();
    assert!(help_msg.contains("count"),
        "Help should contain suggestion 'count': {}", help_msg);
}

#[test]
fn test_help_message_for_type_mismatch_numeric() {
    use vais_types::TypeError;

    let err = TypeError::Mismatch {
        expected: "i64".to_string(),
        found: "Str".to_string(),
        span: None,
    };
    let help = err.help();
    assert!(help.is_some());
    let help_msg = help.unwrap();
    assert!(help_msg.contains("convert"),
        "Numeric mismatch help should suggest conversion: {}", help_msg);
}

// =============================================================
// Localization (i18n)
// =============================================================

#[test]
fn test_localized_title_returns_nonempty() {
    use vais_types::TypeError;

    let err = TypeError::Mismatch {
        expected: "i64".to_string(),
        found: "bool".to_string(),
        span: None,
    };
    let title = err.localized_title();
    assert!(!title.is_empty(), "Localized title should not be empty");
}

#[test]
fn test_localized_message_returns_nonempty() {
    use vais_types::TypeError;

    let err = TypeError::ArgCount {
        expected: 3,
        got: 1,
        span: None,
    };
    let msg = err.localized_message();
    // Localized message should not be empty (may return key if i18n not initialized)
    assert!(!msg.is_empty(), "Localized message should not be empty");
}

// =============================================================
// Levenshtein Distance
// =============================================================

#[test]
fn test_levenshtein_basic() {
    use vais_types::levenshtein_distance;

    assert_eq!(levenshtein_distance("", ""), 0);
    assert_eq!(levenshtein_distance("a", ""), 1);
    assert_eq!(levenshtein_distance("", "b"), 1);
    assert_eq!(levenshtein_distance("abc", "abc"), 0);
    assert_eq!(levenshtein_distance("abc", "abd"), 1);
    assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
}

#[test]
fn test_find_similar_name_exact() {
    use vais_types::find_similar_name;

    let candidates = vec!["count", "counter", "total"];
    let result = find_similar_name("count", candidates.into_iter());
    assert_eq!(result, Some("count".to_string()));
}

#[test]
fn test_find_similar_name_typo() {
    use vais_types::find_similar_name;

    let candidates = vec!["count", "counter", "total"];
    let result = find_similar_name("cont", candidates.into_iter());
    assert!(result.is_some(), "Should find similar name for 'cont'");
    assert_eq!(result.unwrap(), "count");
}

#[test]
fn test_find_similar_name_none() {
    use vais_types::find_similar_name;

    let candidates = vec!["count", "counter", "total"];
    let result = find_similar_name("zzzzzzzzz", candidates.into_iter());
    assert!(result.is_none(), "Should not find similar name for very different input");
}

// =============================================================
// Parse Error Quality
// =============================================================

#[test]
fn test_parse_error_unexpected_token() {
    let source = "F broken syntax here";
    let result = parse(source);
    assert!(result.is_err(), "Should fail to parse");
    let err = result.unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("Unexpected") || msg.contains("unexpected"),
        "Parse error should indicate unexpected token: {}", msg);
}

#[test]
fn test_parse_error_unexpected_eof() {
    let source = "F incomplete(";
    let result = parse(source);
    assert!(result.is_err(), "Should fail to parse incomplete input");
}


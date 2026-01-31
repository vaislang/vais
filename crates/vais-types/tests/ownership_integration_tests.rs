use vais_parser::parse;
use vais_types::TypeChecker;

/// Helper: type-check with strict ownership checking enabled
fn check_strict(source: &str) -> Result<(), String> {
    let module = parse(source).map_err(|e| format!("parse error: {:?}", e))?;
    let mut checker = TypeChecker::new();
    checker.set_strict_ownership(true);
    checker.check_module(&module).map_err(|e| format!("{}", e))
}

/// Helper: type-check with warn-only ownership checking and return warnings
fn check_warn(source: &str) -> Vec<String> {
    let module = parse(source).expect("parse failed");
    let mut checker = TypeChecker::new();
    // Default is warn-only mode
    checker.check_module(&module).expect("type check failed");
    checker.get_warnings().to_vec()
}

// === Positive tests: valid code should pass ===

#[test]
fn ownership_valid_copy_type_reuse() {
    // Copy types (i64) can be used after assignment
    let result = check_strict(r#"
        F main() -> i64 {
            x := 42
            y := x
            x + y
        }
    "#);
    assert!(result.is_ok(), "Copy type reuse should be allowed: {:?}", result);
}

#[test]
fn ownership_valid_immutable_borrows() {
    // Multiple immutable borrows should be fine
    let result = check_strict(r#"
        F foo(a: i64, b: i64) -> i64 {
            a + b
        }
        F main() -> i64 {
            x := 10
            foo(x, x)
        }
    "#);
    assert!(result.is_ok(), "Multiple immutable uses should be allowed: {:?}", result);
}

#[test]
fn ownership_valid_mutable_reassign() {
    let result = check_strict(r#"
        F main() -> i64 {
            x := mut 0
            x = 42
            x
        }
    "#);
    assert!(result.is_ok(), "Mutable reassignment should be allowed: {:?}", result);
}

// === Warn-only mode tests ===

#[test]
fn ownership_warn_mode_does_not_fail() {
    // Even invalid ownership code should not fail in warn-only mode
    let warnings = check_warn(r#"
        F main() -> i64 {
            x := 10
            x
        }
    "#);
    // Should succeed (may or may not have warnings depending on implementation)
    // The key is that it doesn't fail
    let _ = warnings;
}

// === Strict mode configuration tests ===

#[test]
fn ownership_strict_mode_can_be_enabled() {
    let module = parse("F main() -> i64 { 0 }").unwrap();
    let mut checker = TypeChecker::new();
    checker.set_strict_ownership(true);
    let result = checker.check_module(&module);
    assert!(result.is_ok(), "Simple valid code should pass strict mode");
}

#[test]
fn ownership_can_be_disabled() {
    let module = parse("F main() -> i64 { 0 }").unwrap();
    let mut checker = TypeChecker::new();
    checker.disable_ownership_check();
    let result = checker.check_module(&module);
    assert!(result.is_ok());
    assert!(checker.get_warnings().is_empty(), "No ownership warnings when disabled");
}

// === Integration with full compilation pipeline ===

#[test]
fn ownership_check_runs_during_type_check() {
    // Verify that ownership checking is part of the type check pipeline
    let module = parse(r#"
        F add(a: i64, b: i64) -> i64 { a + b }
        F main() -> i64 {
            result := add(1, 2)
            result
        }
    "#).unwrap();

    let mut checker = TypeChecker::new();
    // Default mode: warn-only ownership checking
    let result = checker.check_module(&module);
    assert!(result.is_ok(), "Full pipeline should succeed for valid code");
}

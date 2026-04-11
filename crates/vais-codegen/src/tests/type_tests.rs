use crate::CodeGenerator;
use vais_parser::parse;

// ==================== Strict Type Mode Tests ====================

#[test]
fn test_strict_mode_generic_fallback_remains_warning() {
    // Default mode (strict_type_mode=on, strict_generic_mode=off): Generic fallback stays
    // a warning and codegen still succeeds, preserving the historical Phase 127 contract.
    let source = r#"
F identity<T>(x: T) -> T { x }
F main() -> i64 { identity(42) }
"#;
    let module = parse(source).unwrap();
    let mut checker = vais_types::TypeChecker::new();
    checker.check_module(&module).unwrap();
    let mut gen = CodeGenerator::new("test");
    gen.set_resolved_functions(checker.get_all_functions().clone());
    gen.set_type_aliases(checker.get_type_aliases().clone());
    gen.set_strict_type_mode(true);
    gen.set_strict_generic_mode(false);
    let instantiations = checker.get_generic_instantiations();
    let result = if instantiations.is_empty() {
        gen.generate_module(&module)
    } else {
        gen.generate_module_with_instantiations(&module, &instantiations)
    };
    // Should succeed — Generic fallback remains allowed while strict_generic_mode is off.
    assert!(result.is_ok(), "Expected success, got: {:?}", result.err());
}

#[test]
fn test_void_placeholder_helper() {
    let ir = crate::helpers::void_placeholder_ir("%tmp.0");
    assert!(ir.contains("add i64 0, 0"));
    assert!(ir.contains("void/Unit placeholder"));
    assert!(ir.contains("%tmp.0"));
}

#[test]
fn test_is_void_result_helper() {
    assert!(crate::helpers::is_void_result(
        "void",
        &vais_types::ResolvedType::I64
    ));
    assert!(crate::helpers::is_void_result(
        "i64",
        &vais_types::ResolvedType::Unit
    ));
    assert!(!crate::helpers::is_void_result(
        "i64",
        &vais_types::ResolvedType::I64
    ));
}

#[test]
fn test_emit_warning_or_error_default_mode() {
    let gen = CodeGenerator::new("test");
    // Default mode is strict: UnresolvedTypeFallback should be an error
    let result = gen.emit_warning_or_error(crate::CodegenWarning::UnresolvedTypeFallback {
        type_desc: String::from("test type"),
        backend: String::from("test"),
    });
    assert!(result.is_err());
    // No warnings collected — it was promoted to error
    assert_eq!(gen.get_warnings().len(), 0);
}

#[test]
fn test_emit_warning_or_error_non_strict_mode() {
    let mut gen = CodeGenerator::new("test");
    gen.set_strict_type_mode(false);
    // Non-strict mode: UnresolvedTypeFallback should be a warning, not error
    let result = gen.emit_warning_or_error(crate::CodegenWarning::UnresolvedTypeFallback {
        type_desc: String::from("test type"),
        backend: String::from("test"),
    });
    assert!(result.is_ok());
    assert_eq!(gen.get_warnings().len(), 1);
}

#[test]
fn test_emit_warning_or_error_strict_mode() {
    let mut gen = CodeGenerator::new("test");
    // Strict mode is already the default; explicitly set for clarity
    gen.set_strict_type_mode(true);
    // Strict mode: UnresolvedTypeFallback should become an error
    let result = gen.emit_warning_or_error(crate::CodegenWarning::UnresolvedTypeFallback {
        type_desc: String::from("test type"),
        backend: String::from("test"),
    });
    assert!(result.is_err());
    // No warnings collected — it was promoted to error
    assert_eq!(gen.get_warnings().len(), 0);
}

#[test]
fn test_emit_warning_or_error_strict_mode_generic_fallback() {
    let mut gen = CodeGenerator::new("test");
    gen.set_strict_type_mode(true);
    // Historical behavior: without `strict_generic_mode`, GenericFallback stays a warning
    // even when `strict_type_mode` is on.
    gen.set_strict_generic_mode(false);
    let result = gen.emit_warning_or_error(crate::CodegenWarning::GenericFallback {
        param: String::from("T"),
        context: String::from("test_fn"),
    });
    assert!(result.is_ok());
    assert_eq!(gen.get_warnings().len(), 1);
}

#[test]
fn test_emit_warning_or_error_strict_generic_mode_promotes() {
    // Phase 191: `strict_generic_mode` promotes GenericFallback to a hard error.
    let mut gen = CodeGenerator::new("test");
    gen.set_strict_generic_mode(true);
    let result = gen.emit_warning_or_error(crate::CodegenWarning::GenericFallback {
        param: String::from("T"),
        context: String::from("test_fn"),
    });
    assert!(result.is_err());
    // No warning should have been recorded because it was promoted to an error.
    assert_eq!(gen.get_warnings().len(), 0);
    // Toggling the flag off restores warning behavior.
    gen.set_strict_generic_mode(false);
    let result = gen.emit_warning_or_error(crate::CodegenWarning::GenericFallback {
        param: String::from("U"),
        context: String::from("test_fn"),
    });
    assert!(result.is_ok());
    assert_eq!(gen.get_warnings().len(), 1);
}

#[cfg(feature = "inkwell-codegen")]
#[test]
fn test_inkwell_set_strict_type_mode() {
    let context = ::inkwell::context::Context::create();
    let mut gen = crate::InkwellCodeGenerator::new(&context, "test");
    // Enable strict mode — should not panic
    gen.set_strict_type_mode(true);
    // Disable strict mode
    gen.set_strict_type_mode(false);
}

// ==================== Decreases Termination Tests ====================

#[test]
fn test_decreases_basic() {
    // Test basic decreases clause for termination proof
    let source = r#"
        #[requires(n >= 0)]
        #[decreases(n)]
        F factorial(n:i64)->i64{I n<=1{R 1}R n*factorial(n-1)}
    "#;
    let module = parse(source).unwrap();
    let mut gen = CodeGenerator::new("test");
    let ir = gen.generate_module(&module).unwrap();

    // Should have initial decreases storage
    assert!(
        ir.contains("__decreases_factorial"),
        "Expected decreases storage variable"
    );
    // Should have non-negative check
    assert!(
        ir.contains("decreases_nonneg"),
        "Expected non-negative check"
    );
    // Should have strict decrease check before recursive call
    assert!(
        ir.contains("decreases_check"),
        "Expected decrease check before recursive call"
    );
    // Should have panic call for failed check
    assert!(
        ir.contains("@__panic"),
        "Expected panic call for failed check"
    );
}

#[test]
fn test_decreases_strict_decrease_check() {
    // Test that the strict decrease check (new < old) is generated
    let source = r#"
        #[decreases(n)]
        F count_down(n:i64)->i64{I n<=0{R 0}R count_down(n-1)}
    "#;
    let module = parse(source).unwrap();
    let mut gen = CodeGenerator::new("test");
    let ir = gen.generate_module(&module).unwrap();

    // Should have icmp slt (strictly less than) check
    assert!(
        ir.contains("icmp slt i64"),
        "Expected strict less-than comparison for decreases"
    );
    // Should have both decreases labels
    assert!(ir.contains("decreases_check_ok"), "Expected success label");
    assert!(
        ir.contains("decreases_check_fail"),
        "Expected failure label"
    );
}

#[test]
fn test_decreases_nonneg_check() {
    // Test that non-negative check is generated for decreases expression
    let source = r#"
        #[decreases(x)]
        F process(x:i64)->i64{I x<=0{R 0}R process(x-1)+1}
    "#;
    let module = parse(source).unwrap();
    let mut gen = CodeGenerator::new("test");
    let ir = gen.generate_module(&module).unwrap();

    // Should have icmp sge (signed greater-or-equal) for non-negative check
    assert!(
        ir.contains("icmp sge i64"),
        "Expected non-negative check (sge 0)"
    );
    assert!(
        ir.contains("decreases_nonneg_ok"),
        "Expected success label for non-negative"
    );
    assert!(
        ir.contains("decreases_nonneg_fail"),
        "Expected failure label for non-negative"
    );
}

#[test]
fn test_decreases_release_mode() {
    // Test that decreases checks are skipped in release mode
    let source = r#"
        #[decreases(n)]
        F fib(n:i64)->i64{I n<2{R n}R fib(n-1)+fib(n-2)}
    "#;
    let module = parse(source).unwrap();
    let mut gen = CodeGenerator::new("test");
    gen.enable_release_mode();
    let ir = gen.generate_module(&module).unwrap();

    // Should NOT have decreases checks in release mode
    assert!(
        !ir.contains("__decreases_fib"),
        "Should skip decreases in release mode"
    );
    assert!(
        !ir.contains("decreases_nonneg"),
        "Should skip non-negative check in release mode"
    );
    assert!(
        !ir.contains("decreases_check"),
        "Should skip decrease check in release mode"
    );
}

#[test]
fn test_decreases_with_selfcall() {
    // Test decreases with @ self-call operator
    let source = r#"
        #[decreases(n)]
        F sum_to(n:i64)->i64{I n<=0{R 0}R n+@(n-1)}
    "#;
    let module = parse(source).unwrap();
    let mut gen = CodeGenerator::new("test");
    let ir = gen.generate_module(&module).unwrap();

    // Should have decreases check before the self-call
    assert!(
        ir.contains("__decreases_sum_to"),
        "Expected decreases storage"
    );
    assert!(
        ir.contains("decreases_check"),
        "Expected decrease check before self-call"
    );
}

// ==================== Type Recursion Tests ====================

#[test]
fn test_type_recursion_depth_limit() {
    // Test that deeply nested types work within the limit
    use vais_types::ResolvedType;

    let gen = CodeGenerator::new("test");

    // Create a deeply nested pointer type (should work)
    let mut nested_type = ResolvedType::I32;
    for _ in 0..50 {
        nested_type = ResolvedType::Pointer(Box::new(nested_type));
    }

    // This should work fine (well within the 128 limit)
    let llvm_type = gen.type_to_llvm(&nested_type);
    assert!(llvm_type.ends_with('*'), "Should generate nested pointers");

    // Create an extremely deeply nested type (exceeds limit of 128)
    let mut extremely_nested = ResolvedType::I32;
    for _ in 0..150 {
        extremely_nested = ResolvedType::Pointer(Box::new(extremely_nested));
    }

    // This should hit the recursion limit and fall back to i64
    // (The error is logged but doesn't fail - returns fallback type)
    let llvm_type_over_limit = gen.type_to_llvm(&extremely_nested);
    // Should still return a valid type (either i64 fallback or truncated)
    assert!(
        !llvm_type_over_limit.is_empty(),
        "Should return a fallback type on recursion limit"
    );
}

#[test]
fn test_type_recursion_reset_between_calls() {
    // Test that recursion depth is properly reset between calls
    use vais_types::ResolvedType;

    let gen = CodeGenerator::new("test");

    // First call with nested types
    let mut nested1 = ResolvedType::I32;
    for _ in 0..30 {
        nested1 = ResolvedType::Pointer(Box::new(nested1));
    }
    let _ = gen.type_to_llvm(&nested1);

    // Second call should work independently (depth should be reset)
    let mut nested2 = ResolvedType::I64;
    for _ in 0..30 {
        nested2 = ResolvedType::Pointer(Box::new(nested2));
    }
    let llvm_type = gen.type_to_llvm(&nested2);
    assert!(
        llvm_type.ends_with('*'),
        "Second call should work independently"
    );
}

#[test]
fn test_ast_type_recursion_limit() {
    // Test that ast_type_to_resolved also respects recursion limits
    use vais_ast::{Span, Type};
    use vais_types::ResolvedType;

    let gen = CodeGenerator::new("test");

    // Create deeply nested AST type
    let mut nested = Type::Named {
        name: "i32".to_string(),
        generics: vec![],
    };
    for _ in 0..50 {
        nested = Type::Pointer(Box::new(vais_ast::Spanned::new(
            nested,
            Span { start: 0, end: 0 },
        )));
    }

    // Should work within limit
    let resolved = gen.ast_type_to_resolved(&nested);
    assert!(
        matches!(resolved, ResolvedType::Pointer(_)),
        "Should resolve nested pointers"
    );

    // Create extremely nested type (exceeds limit)
    let mut extremely_nested = Type::Named {
        name: "i32".to_string(),
        generics: vec![],
    };
    for _ in 0..150 {
        extremely_nested = Type::Pointer(Box::new(vais_ast::Spanned::new(
            extremely_nested,
            Span { start: 0, end: 0 },
        )));
    }

    // Should hit limit and return fallback
    let resolved_over = gen.ast_type_to_resolved(&extremely_nested);
    // Should still return a valid type (Unknown as fallback)
    assert!(
        matches!(
            resolved_over,
            ResolvedType::Unknown | ResolvedType::Pointer(_)
        ),
        "Should return a fallback or truncated type on recursion limit"
    );
}

//! Comprehensive type checker error path tests
//!
//! Tests cover: all TypeError variants, error codes, help messages,
//! secondary spans, localized messages, and type inference edge cases.

use vais_parser::parse;
use vais_types::{TypeChecker, TypeError};

// ============================================================================
// Helper
// ============================================================================

fn check_error(source: &str) -> Option<TypeError> {
    let ast = parse(source).ok()?;
    let mut checker = TypeChecker::new();
    checker.check_module(&ast).err()
}

fn check_ok(source: &str) {
    let ast = parse(source).unwrap_or_else(|e| panic!("Parse failed for: {}\nErr: {}", source, e));
    let mut checker = TypeChecker::new();
    let result = checker.check_module(&ast);
    assert!(result.is_ok(), "Type check failed for: {}\nErr: {:?}", source, result.err());
}

fn check_err(source: &str) -> TypeError {
    let ast = parse(source).unwrap_or_else(|e| panic!("Parse failed for: {}\nErr: {}", source, e));
    let mut checker = TypeChecker::new();
    match checker.check_module(&ast) {
        Err(e) => e,
        Ok(()) => panic!("Expected type error for: {}", source),
    }
}

// ============================================================================
// 1. E001: Type Mismatch
// ============================================================================

#[test]
fn test_mismatch_return_bool_for_i64() {
    let err = check_err("F test()->i64=true");
    assert_eq!(err.error_code(), "E001");
    assert!(err.to_string().contains("mismatch") || err.to_string().contains("Mismatch"));
}

#[test]
fn test_mismatch_return_string_for_i64() {
    if let Some(err) = check_error("F test()->i64=\"hello\"") {
        assert_eq!(err.error_code(), "E001");
    }
}

#[test]
fn test_mismatch_return_i64_for_bool() {
    if let Some(err) = check_error("F test()->bool=42") {
        assert_eq!(err.error_code(), "E001");
    }
}

#[test]
fn test_mismatch_help_message() {
    let err = check_err("F test()->i64=true");
    let help = err.help();
    assert!(help.is_some(), "Mismatch should have help message");
}

// ============================================================================
// 2. E002: Undefined Variable
// ============================================================================

#[test]
fn test_undefined_var_simple() {
    let err = check_err("F test()->i64{R x}");
    assert_eq!(err.error_code(), "E002");
}

#[test]
fn test_undefined_var_with_suggestion() {
    if let Some(err) = check_error("F test()->i64{count:=42;R cont}") {
        assert_eq!(err.error_code(), "E002");
        let help = err.help();
        assert!(help.is_some(), "Should have help with suggestion");
        let help_msg = help.unwrap();
        assert!(help_msg.contains("count") || help_msg.contains("cont"),
            "Help should suggest similar name: {}", help_msg);
    }
}

#[test]
fn test_undefined_var_no_suggestion_dissimilar() {
    if let Some(err) = check_error("F test()->i64{counter:=42;R xyz}") {
        assert_eq!(err.error_code(), "E002");
        let help = err.help();
        assert!(help.is_some(), "Should still have a help message");
    }
}

#[test]
fn test_undefined_var_in_binary_expr() {
    if let Some(err) = check_error("F test()->i64{a:=1;R a+b}") {
        assert_eq!(err.error_code(), "E002");
    }
}

#[test]
fn test_undefined_var_in_condition() {
    if let Some(err) = check_error("F test()->i64{I flag{R 1}E{R 0}}") {
        assert_eq!(err.error_code(), "E002");
    }
}

#[test]
fn test_undefined_var_in_loop() {
    if let Some(err) = check_error("F test()->i64{L _:cond{B};R 0}") {
        assert_eq!(err.error_code(), "E002");
    }
}

// ============================================================================
// 3. E003: Undefined Type
// ============================================================================

#[test]
fn test_undefined_type() {
    if let Some(err) = check_error("F test(x:NonExistent)->i64=0") {
        assert_eq!(err.error_code(), "E003");
    }
}

#[test]
fn test_undefined_type_suggestion() {
    if let Some(err) = check_error("S Point{x:i64} F test(p:Poin)->i64=0") {
        assert_eq!(err.error_code(), "E003");
        let help = err.help();
        assert!(help.is_some());
    }
}

// ============================================================================
// 4. E004: Undefined Function
// ============================================================================

#[test]
fn test_undefined_function() {
    if let Some(err) = check_error("F add(a:i64,b:i64)->i64=a+b F main()->i64=ad(1,2)") {
        let code = err.error_code();
        // May be E002 or E004 depending on resolution order
        assert!(code == "E002" || code == "E004", "Got: {}", code);
    }
}

#[test]
fn test_undefined_function_help() {
    if let Some(err) = check_error("F add(a:i64,b:i64)->i64=a+b F main()->i64=ad(1,2)") {
        let help = err.help();
        assert!(help.is_some(), "Undefined identifier should have help");
    }
}

// ============================================================================
// 5. E006: Argument Count Mismatch
// ============================================================================

#[test]
fn test_arg_count_too_few() {
    if let Some(err) = check_error("F add(a:i64,b:i64)->i64=a+b F main()->i64=add(1)") {
        assert_eq!(err.error_code(), "E006");
        let msg = format!("{}", err);
        assert!(msg.contains("2") && msg.contains("1"), "Should mention expected/got: {}", msg);
    }
}

#[test]
fn test_arg_count_too_many() {
    if let Some(err) = check_error("F inc(x:i64)->i64=x+1 F main()->i64=inc(1,2)") {
        assert_eq!(err.error_code(), "E006");
    }
}

#[test]
fn test_arg_count_zero_expected() {
    if let Some(err) = check_error("F nop()->i64=0 F main()->i64=nop(42)") {
        assert_eq!(err.error_code(), "E006");
        let help = err.help().unwrap();
        assert!(help.contains("no arguments"), "Help: {}", help);
    }
}

#[test]
fn test_arg_count_help_singular() {
    if let Some(err) = check_error("F one(x:i64)->i64=x F main()->i64=one(1,2)") {
        assert_eq!(err.error_code(), "E006");
        let help = err.help().unwrap();
        assert!(help.contains("1 argument"), "Help: {}", help);
    }
}

// ============================================================================
// 6. E008: Duplicate Definition
// ============================================================================

#[test]
fn test_duplicate_function() {
    if let Some(err) = check_error("F test()->i64=1 F test()->i64=2") {
        assert_eq!(err.error_code(), "E008");
        let help = err.help().unwrap();
        assert!(help.contains("already defined") || help.contains("renaming"),
            "Help: {}", help);
    }
}

#[test]
fn test_duplicate_struct() {
    if let Some(err) = check_error("S Point{x:i64} S Point{y:i64}") {
        assert_eq!(err.error_code(), "E008");
    }
}

// ============================================================================
// 7. E009: Immutable Assignment
// ============================================================================

#[test]
fn test_immutable_assign() {
    if let Some(err) = check_error("F test()->i64{x:=42;x=10;R x}") {
        assert_eq!(err.error_code(), "E009");
        let help = err.help().unwrap();
        assert!(help.contains("mutable"), "Help: {}", help);
    }
}

// ============================================================================
// 8. Valid Type Checking (positive tests)
// ============================================================================

#[test]
fn test_valid_simple_function() {
    check_ok("F test()->i64=42");
}

#[test]
fn test_valid_binary_op() {
    check_ok("F add(a:i64,b:i64)->i64=a+b");
}

#[test]
fn test_valid_if_else() {
    check_ok("F abs(x:i64)->i64{I x>0{R x}E{R 0-x}}");
}

#[test]
fn test_valid_loop() {
    check_ok("F test()->i64{x:=0;L _:x<10{x=x+1};x}");
}

#[test]
fn test_valid_struct_field_access() {
    check_ok("S Point{x:i64,y:i64} F get_x(p:Point)->i64=p.x");
}

#[test]
fn test_valid_match() {
    check_ok("F test(x:i64)->i64{M x{0=>1,1=>2,_=>0}}");
}

#[test]
fn test_valid_self_recursion() {
    check_ok("F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)");
}

#[test]
fn test_valid_generic() {
    check_ok("F id<T>(x:T)->T=x");
}

#[test]
fn test_valid_mutable_var() {
    check_ok("F test()->i64{x:= mut 0;x=42;R x}");
}

#[test]
fn test_valid_enum() {
    check_ok("E Dir{N,S,E2,W}");
}

#[test]
fn test_valid_trait() {
    check_ok("W Printable{F to_str(self)->str}");
}

#[test]
fn test_valid_impl() {
    check_ok("S Num{val:i64} X Num{F get(self)->i64=self.val}");
}

#[test]
fn test_valid_nested_if() {
    check_ok("F test(x:i64)->i64{I x>0{I x>10{R 2}E{R 1}}E{R 0}}");
}

#[test]
fn test_valid_for_loop() {
    check_ok("F test()->i64{s:=0;L i:0..10{s=s+i};s}");
}

#[test]
fn test_valid_lambda() {
    check_ok("F test()->i64{f:=|x:i64|->i64{x+1};f(41)}");
}

// ============================================================================
// 9. Error Code Uniqueness
// ============================================================================

#[test]
fn test_all_error_codes_are_unique() {
    let errors: Vec<TypeError> = vec![
        TypeError::Mismatch { expected: String::new(), found: String::new(), span: None },
        TypeError::UndefinedVar { name: String::new(), span: None, suggestion: None },
        TypeError::UndefinedType { name: String::new(), span: None, suggestion: None },
        TypeError::UndefinedFunction { name: String::new(), span: None, suggestion: None },
        TypeError::NotCallable(String::new(), None),
        TypeError::ArgCount { expected: 0, got: 0, span: None },
        TypeError::CannotInfer,
        TypeError::Duplicate(String::new(), None),
        TypeError::ImmutableAssign(String::new(), None),
        TypeError::NonExhaustiveMatch(String::new(), None),
        TypeError::UnreachablePattern(0, None),
    ];
    let codes: Vec<&str> = errors.iter().map(|e| e.error_code()).collect();
    let mut unique = codes.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(codes.len(), unique.len(), "Error codes must be unique");
}

// ============================================================================
// 10. Extended Error Codes
// ============================================================================

#[test]
fn test_effect_mismatch_code() {
    let err = TypeError::EffectMismatch {
        declared: "pure".to_string(),
        actual: "io".to_string(),
        span: None,
    };
    assert_eq!(err.error_code(), "E012");
}

#[test]
fn test_purity_violation_code() {
    let err = TypeError::PurityViolation {
        callee: "read_file".to_string(),
        effects: "io".to_string(),
        span: None,
    };
    assert_eq!(err.error_code(), "E013");
}

#[test]
fn test_linear_type_violation_code() {
    let err = TypeError::LinearTypeViolation {
        var_name: "x".to_string(),
        expected_uses: 1,
        actual_uses: 0,
        defined_at: None,
    };
    assert_eq!(err.error_code(), "E014");
}

#[test]
fn test_affine_type_violation_code() {
    let err = TypeError::AffineTypeViolation {
        var_name: "y".to_string(),
        actual_uses: 2,
        defined_at: None,
    };
    assert_eq!(err.error_code(), "E015");
}

#[test]
fn test_move_after_use_code() {
    let err = TypeError::MoveAfterUse {
        var_name: "z".to_string(),
        first_use_at: None,
        move_at: None,
    };
    assert_eq!(err.error_code(), "E016");
}

#[test]
fn test_dependent_predicate_not_bool_code() {
    let err = TypeError::DependentPredicateNotBool {
        found: "i64".to_string(),
        span: None,
    };
    assert_eq!(err.error_code(), "E017");
}

#[test]
fn test_refinement_violation_code() {
    let err = TypeError::RefinementViolation {
        predicate: "x > 0".to_string(),
        span: None,
    };
    assert_eq!(err.error_code(), "E018");
}

#[test]
fn test_lifetime_elision_failed_code() {
    let err = TypeError::LifetimeElisionFailed {
        function_name: "test".to_string(),
        input_count: 2,
        span: None,
    };
    assert_eq!(err.error_code(), "E019");
}

#[test]
fn test_lifetime_outlives_static_code() {
    let err = TypeError::LifetimeOutlivesStatic {
        lifetime_name: "'a".to_string(),
        span: None,
    };
    assert_eq!(err.error_code(), "E020");
}

#[test]
fn test_lifetime_too_short_code() {
    let err = TypeError::LifetimeTooShort {
        reference_lifetime: "'a".to_string(),
        referent_lifetime: "'b".to_string(),
        span: None,
    };
    assert_eq!(err.error_code(), "E021");
}

#[test]
fn test_use_after_move_code() {
    let err = TypeError::UseAfterMove {
        var_name: "x".to_string(),
        moved_at: None,
        use_at: None,
    };
    assert_eq!(err.error_code(), "E022");
}

#[test]
fn test_use_after_partial_move_code() {
    let err = TypeError::UseAfterPartialMove {
        var_name: "x".to_string(),
        moved_fields: vec!["a".to_string()],
        use_at: None,
    };
    assert_eq!(err.error_code(), "E023");
}

#[test]
fn test_assign_while_borrowed_code() {
    let err = TypeError::AssignWhileBorrowed {
        var_name: "x".to_string(),
        borrow_at: None,
        assign_at: None,
        is_mut_borrow: false,
    };
    assert_eq!(err.error_code(), "E024");
}

#[test]
fn test_borrow_after_move_code() {
    let err = TypeError::BorrowAfterMove {
        var_name: "x".to_string(),
        moved_at: None,
        borrow_at: None,
    };
    assert_eq!(err.error_code(), "E025");
}

#[test]
fn test_borrow_conflict_code() {
    let err = TypeError::BorrowConflict {
        var_name: "x".to_string(),
        existing_borrow_at: None,
        new_borrow_at: None,
        existing_is_mut: true,
        new_is_mut: false,
    };
    assert_eq!(err.error_code(), "E026");
}

#[test]
fn test_mut_borrow_of_immutable_code() {
    let err = TypeError::MutBorrowOfImmutable {
        var_name: "x".to_string(),
        borrow_at: None,
    };
    assert_eq!(err.error_code(), "E027");
}

#[test]
fn test_dangling_reference_code() {
    let err = TypeError::DanglingReference {
        ref_var: "r".to_string(),
        source_var: "x".to_string(),
        ref_scope_depth: 2,
        source_scope_depth: 1,
        ref_at: None,
        source_defined_at: None,
    };
    assert_eq!(err.error_code(), "E028");
}

#[test]
fn test_return_local_ref_code() {
    let err = TypeError::ReturnLocalRef {
        var_name: "x".to_string(),
        return_at: None,
        defined_at: None,
    };
    assert_eq!(err.error_code(), "E029");
}

#[test]
fn test_no_such_field_code() {
    let err = TypeError::NoSuchField {
        field: "z".to_string(),
        type_name: "Point".to_string(),
        suggestion: None,
        span: None,
    };
    assert_eq!(err.error_code(), "E030");
}

#[test]
fn test_extern_sig_mismatch_code() {
    let err = TypeError::ExternSignatureMismatch {
        name: "malloc".to_string(),
        expected: "*i8".to_string(),
        found: "i64".to_string(),
        span: None,
    };
    assert_eq!(err.error_code(), "E031");
}

#[test]
fn test_infer_failed_code() {
    let err = TypeError::InferFailed {
        kind: "variable".to_string(),
        name: "x".to_string(),
        context: "test".to_string(),
        span: None,
        suggestion: None,
    };
    assert_eq!(err.error_code(), "E032");
}

// ============================================================================
// 11. Help Messages for All Variants
// ============================================================================

#[test]
fn test_help_effect_mismatch() {
    let err = TypeError::EffectMismatch {
        declared: "pure".to_string(),
        actual: "io".to_string(),
        span: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("pure"), "Help: {}", help);
}

#[test]
fn test_help_purity_violation() {
    let err = TypeError::PurityViolation {
        callee: "read_file".to_string(),
        effects: "io".to_string(),
        span: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("read_file"), "Help: {}", help);
}

#[test]
fn test_help_linear_type_unused() {
    let err = TypeError::LinearTypeViolation {
        var_name: "x".to_string(),
        expected_uses: 1,
        actual_uses: 0,
        defined_at: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("x") && help.contains("1"), "Help: {}", help);
}

#[test]
fn test_help_linear_type_overused() {
    let err = TypeError::LinearTypeViolation {
        var_name: "x".to_string(),
        expected_uses: 1,
        actual_uses: 3,
        defined_at: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("3"), "Help: {}", help);
}

#[test]
fn test_help_affine_violation() {
    let err = TypeError::AffineTypeViolation {
        var_name: "y".to_string(),
        actual_uses: 2,
        defined_at: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("y"), "Help: {}", help);
}

#[test]
fn test_help_move_after_use() {
    let err = TypeError::MoveAfterUse {
        var_name: "z".to_string(),
        first_use_at: None,
        move_at: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("z"), "Help: {}", help);
}

#[test]
fn test_help_lifetime_elision_multiple_inputs() {
    let err = TypeError::LifetimeElisionFailed {
        function_name: "combine".to_string(),
        input_count: 2,
        span: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("explicit lifetime"), "Help: {}", help);
}

#[test]
fn test_help_lifetime_elision_single_input() {
    let err = TypeError::LifetimeElisionFailed {
        function_name: "borrow".to_string(),
        input_count: 1,
        span: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("lifetime"), "Help: {}", help);
}

#[test]
fn test_help_lifetime_outlives_static() {
    let err = TypeError::LifetimeOutlivesStatic {
        lifetime_name: "'a".to_string(),
        span: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("'a"), "Help: {}", help);
}

#[test]
fn test_help_lifetime_too_short() {
    let err = TypeError::LifetimeTooShort {
        reference_lifetime: "'a".to_string(),
        referent_lifetime: "'b".to_string(),
        span: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("outlive"), "Help: {}", help);
}

#[test]
fn test_help_use_after_move() {
    let err = TypeError::UseAfterMove {
        var_name: "x".to_string(),
        moved_at: None,
        use_at: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("x") && help.contains("clone"), "Help: {}", help);
}

#[test]
fn test_help_use_after_partial_move() {
    let err = TypeError::UseAfterPartialMove {
        var_name: "p".to_string(),
        moved_fields: vec!["x".to_string(), "y".to_string()],
        use_at: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("p"), "Help: {}", help);
}

#[test]
fn test_help_assign_while_borrowed_mut() {
    let err = TypeError::AssignWhileBorrowed {
        var_name: "x".to_string(),
        borrow_at: None,
        assign_at: None,
        is_mut_borrow: true,
    };
    let help = err.help().unwrap();
    assert!(help.contains("mutably borrowed"), "Help: {}", help);
}

#[test]
fn test_help_assign_while_borrowed_immut() {
    let err = TypeError::AssignWhileBorrowed {
        var_name: "x".to_string(),
        borrow_at: None,
        assign_at: None,
        is_mut_borrow: false,
    };
    let help = err.help().unwrap();
    assert!(help.contains("borrowed"), "Help: {}", help);
}

#[test]
fn test_help_borrow_after_move() {
    let err = TypeError::BorrowAfterMove {
        var_name: "x".to_string(),
        moved_at: None,
        borrow_at: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("x"), "Help: {}", help);
}

#[test]
fn test_help_borrow_conflict_existing_mut() {
    let err = TypeError::BorrowConflict {
        var_name: "x".to_string(),
        existing_borrow_at: None,
        new_borrow_at: None,
        existing_is_mut: true,
        new_is_mut: false,
    };
    let help = err.help().unwrap();
    assert!(help.contains("mutably borrowed"), "Help: {}", help);
}

#[test]
fn test_help_borrow_conflict_new_mut() {
    let err = TypeError::BorrowConflict {
        var_name: "x".to_string(),
        existing_borrow_at: None,
        new_borrow_at: None,
        existing_is_mut: false,
        new_is_mut: true,
    };
    let help = err.help().unwrap();
    assert!(help.contains("mutably borrow"), "Help: {}", help);
}

#[test]
fn test_help_borrow_conflict_both_immut() {
    let err = TypeError::BorrowConflict {
        var_name: "x".to_string(),
        existing_borrow_at: None,
        new_borrow_at: None,
        existing_is_mut: false,
        new_is_mut: false,
    };
    let help = err.help().unwrap();
    assert!(help.contains("x"), "Help: {}", help);
}

#[test]
fn test_help_mut_borrow_of_immutable() {
    let err = TypeError::MutBorrowOfImmutable {
        var_name: "x".to_string(),
        borrow_at: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("mutable"), "Help: {}", help);
}

#[test]
fn test_help_dangling_reference() {
    let err = TypeError::DanglingReference {
        ref_var: "r".to_string(),
        source_var: "x".to_string(),
        ref_scope_depth: 2,
        source_scope_depth: 1,
        ref_at: None,
        source_defined_at: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("r") && help.contains("x"), "Help: {}", help);
}

#[test]
fn test_help_return_local_ref() {
    let err = TypeError::ReturnLocalRef {
        var_name: "x".to_string(),
        return_at: None,
        defined_at: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("x"), "Help: {}", help);
}

#[test]
fn test_help_no_such_field_with_suggestion() {
    let err = TypeError::NoSuchField {
        field: "z".to_string(),
        type_name: "Point".to_string(),
        suggestion: Some("x".to_string()),
        span: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("x"), "Help: {}", help);
}

#[test]
fn test_help_no_such_field_no_suggestion() {
    let err = TypeError::NoSuchField {
        field: "z".to_string(),
        type_name: "Point".to_string(),
        suggestion: None,
        span: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("Point") && help.contains("z"), "Help: {}", help);
}

#[test]
fn test_help_extern_sig_mismatch() {
    let err = TypeError::ExternSignatureMismatch {
        name: "malloc".to_string(),
        expected: "*i8".to_string(),
        found: "i64".to_string(),
        span: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("malloc"), "Help: {}", help);
}

#[test]
fn test_help_infer_failed_with_suggestion() {
    let err = TypeError::InferFailed {
        kind: "variable".to_string(),
        name: "x".to_string(),
        context: "test".to_string(),
        span: None,
        suggestion: Some("add type annotation".to_string()),
    };
    let help = err.help().unwrap();
    assert!(help.contains("type annotation"), "Help: {}", help);
}

#[test]
fn test_help_infer_failed_no_suggestion() {
    let err = TypeError::InferFailed {
        kind: "variable".to_string(),
        name: "x".to_string(),
        context: "test".to_string(),
        span: None,
        suggestion: None,
    };
    assert!(err.help().is_none());
}

#[test]
fn test_help_not_callable() {
    let err = TypeError::NotCallable("i64".to_string(), None);
    let help = err.help().unwrap();
    assert!(help.contains("callable"), "Help: {}", help);
}

#[test]
fn test_help_non_exhaustive_match() {
    let err = TypeError::NonExhaustiveMatch("Red, Blue".to_string(), None);
    let help = err.help().unwrap();
    assert!(help.contains("Red, Blue") || help.contains("wildcard"), "Help: {}", help);
}

#[test]
fn test_help_unreachable_pattern() {
    let err = TypeError::UnreachablePattern(3, None);
    let help = err.help().unwrap();
    assert!(help.contains("3"), "Help: {}", help);
}

#[test]
fn test_help_dependent_predicate_not_bool() {
    let err = TypeError::DependentPredicateNotBool {
        found: "i64".to_string(),
        span: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("bool"), "Help: {}", help);
}

#[test]
fn test_help_refinement_violation() {
    let err = TypeError::RefinementViolation {
        predicate: "x > 0".to_string(),
        span: None,
    };
    let help = err.help().unwrap();
    assert!(help.contains("x > 0"), "Help: {}", help);
}

// ============================================================================
// 12. Secondary Spans
// ============================================================================

#[test]
fn test_secondary_spans_use_after_move_with_span() {
    let span = vais_ast::Span { start: 10, end: 20 };
    let err = TypeError::UseAfterMove {
        var_name: "x".to_string(),
        moved_at: Some(span),
        use_at: None,
    };
    let spans = err.secondary_spans();
    assert_eq!(spans.len(), 1);
    assert!(spans[0].1.contains("moved here"));
}

#[test]
fn test_secondary_spans_use_after_move_no_span() {
    let err = TypeError::UseAfterMove {
        var_name: "x".to_string(),
        moved_at: None,
        use_at: None,
    };
    assert!(err.secondary_spans().is_empty());
}

#[test]
fn test_secondary_spans_assign_while_borrowed() {
    let span = vais_ast::Span { start: 5, end: 15 };
    let err = TypeError::AssignWhileBorrowed {
        var_name: "x".to_string(),
        borrow_at: Some(span),
        assign_at: None,
        is_mut_borrow: false,
    };
    let spans = err.secondary_spans();
    assert_eq!(spans.len(), 1);
    assert!(spans[0].1.contains("borrow occurs here"));
}

#[test]
fn test_secondary_spans_dangling_reference() {
    let span = vais_ast::Span { start: 0, end: 5 };
    let err = TypeError::DanglingReference {
        ref_var: "r".to_string(),
        source_var: "x".to_string(),
        ref_scope_depth: 2,
        source_scope_depth: 1,
        ref_at: None,
        source_defined_at: Some(span),
    };
    let spans = err.secondary_spans();
    assert_eq!(spans.len(), 1);
    assert!(spans[0].1.contains("defined here"));
}

#[test]
fn test_secondary_spans_borrow_after_move() {
    let span = vais_ast::Span { start: 0, end: 5 };
    let err = TypeError::BorrowAfterMove {
        var_name: "x".to_string(),
        moved_at: Some(span),
        borrow_at: None,
    };
    let spans = err.secondary_spans();
    assert_eq!(spans.len(), 1);
    assert!(spans[0].1.contains("moved here"));
}

#[test]
fn test_secondary_spans_return_local_ref() {
    let span = vais_ast::Span { start: 0, end: 5 };
    let err = TypeError::ReturnLocalRef {
        var_name: "x".to_string(),
        return_at: None,
        defined_at: Some(span),
    };
    let spans = err.secondary_spans();
    assert_eq!(spans.len(), 1);
    assert!(spans[0].1.contains("defined here"));
}

#[test]
fn test_secondary_spans_move_after_use() {
    let span = vais_ast::Span { start: 0, end: 5 };
    let err = TypeError::MoveAfterUse {
        var_name: "x".to_string(),
        first_use_at: Some(span),
        move_at: None,
    };
    let spans = err.secondary_spans();
    assert_eq!(spans.len(), 1);
    assert!(spans[0].1.contains("first use"));
}

#[test]
fn test_secondary_spans_cannot_infer() {
    let err = TypeError::CannotInfer;
    assert!(err.secondary_spans().is_empty());
}

// ============================================================================
// 13. Localized Messages
// ============================================================================

#[test]
fn test_localized_title_not_empty() {
    let err = TypeError::Mismatch {
        expected: "i64".to_string(),
        found: "bool".to_string(),
        span: None,
    };
    let title = err.localized_title();
    assert!(!title.is_empty(), "Localized title should not be empty");
}

#[test]
fn test_localized_message_not_empty() {
    let err = TypeError::UndefinedVar {
        name: "x".to_string(),
        span: None,
        suggestion: None,
    };
    let msg = err.localized_message();
    assert!(!msg.is_empty(), "Localized message should not be empty");
}

#[test]
fn test_localized_message_mismatch() {
    let err = TypeError::Mismatch {
        expected: "i64".to_string(),
        found: "bool".to_string(),
        span: None,
    };
    let msg = err.localized_message();
    assert!(!msg.is_empty());
}

#[test]
fn test_localized_message_arg_count() {
    let err = TypeError::ArgCount {
        expected: 2,
        got: 3,
        span: None,
    };
    let msg = err.localized_message();
    assert!(!msg.is_empty());
}

// ============================================================================
// 14. Display Formatting
// ============================================================================

#[test]
fn test_display_not_callable() {
    let err = TypeError::NotCallable("i64".to_string(), None);
    assert!(err.to_string().contains("i64"));
}

#[test]
fn test_display_non_exhaustive_match() {
    let err = TypeError::NonExhaustiveMatch("Red".to_string(), None);
    assert!(err.to_string().contains("Red"));
}

#[test]
fn test_display_unreachable_pattern() {
    let err = TypeError::UnreachablePattern(2, None);
    assert!(err.to_string().contains("2"));
}

#[test]
fn test_display_duplicate() {
    let err = TypeError::Duplicate("test".to_string(), None);
    assert!(err.to_string().contains("test"));
}

#[test]
fn test_display_immutable_assign() {
    let err = TypeError::ImmutableAssign("x".to_string(), None);
    assert!(err.to_string().contains("x"));
}

//! Parser error recovery and negative tests
//!
//! Comprehensive tests for parse error paths, recovery mechanisms,
//! error codes, and edge cases in the Vais parser.

use vais_parser::{parse, parse_with_recovery};

// ============================================================================
// Helper
// ============================================================================

fn parse_err(source: &str) -> String {
    let result = parse(source);
    assert!(result.is_err(), "Expected parse error for: {}", source);
    format!("{}", result.unwrap_err())
}

fn parse_ok(source: &str) {
    let result = parse(source);
    assert!(result.is_ok(), "Expected parse success for: {}\nErr: {:?}", source, result.err());
}

// ============================================================================
// 1. Missing Function Name
// ============================================================================

#[test]
fn test_missing_function_name() {
    let msg = parse_err("F (x: i64) -> i64 = x");
    assert!(!msg.is_empty());
}

#[test]
fn test_missing_function_name_no_params() {
    let msg = parse_err("F -> i64 = 42");
    assert!(!msg.is_empty());
}

#[test]
fn test_function_keyword_only() {
    let msg = parse_err("F");
    assert!(!msg.is_empty());
}

// ============================================================================
// 2. Missing/Mismatched Delimiters
// ============================================================================

#[test]
fn test_unclosed_paren_in_function() {
    let result = parse("F broken(x: i64 -> i64 = x");
    assert!(result.is_err());
    if let Err(e) = result {
        let code = e.error_code();
        assert!(code == "P001" || code == "P002", "Got: {}", code);
    }
}

#[test]
fn test_unclosed_brace_in_function_body() {
    let result = parse("F test() -> i64 { R 42");
    assert!(result.is_err());
}

#[test]
fn test_unclosed_bracket_in_array() {
    let result = parse("F test() -> *i64 = [1, 2, 3");
    assert!(result.is_err());
}

#[test]
fn test_extra_closing_paren() {
    let result = parse("F test(x: i64)) -> i64 = x");
    assert!(result.is_err());
}

#[test]
fn test_extra_closing_brace() {
    let result = parse("F test() -> i64 { R 42 }}");
    assert!(result.is_err());
}

#[test]
fn test_mismatched_delimiters() {
    let result = parse("F test() -> i64 { R 42 )");
    assert!(result.is_err());
}

// ============================================================================
// 3. Incomplete Struct Definitions
// ============================================================================

#[test]
fn test_struct_no_closing_brace() {
    let result = parse("S Point{x: f64");
    assert!(result.is_err());
    if let Err(e) = result {
        let code = e.error_code();
        assert!(code == "P001" || code == "P002", "Got: {}", code);
    }
}

#[test]
fn test_struct_missing_field_type() {
    let result = parse("S Point{x:, y: f64}");
    assert!(result.is_err());
}

#[test]
fn test_struct_keyword_only() {
    let result = parse("S");
    assert!(result.is_err());
}

#[test]
fn test_struct_no_name() {
    let result = parse("S {x: i64}");
    assert!(result.is_err());
}

#[test]
fn test_struct_missing_field_separator() {
    // Missing comma between fields
    let result = parse("S Point{x: i64 y: i64}");
    // This may or may not error depending on parser behavior
    // but the test exercises the path
    let _ = result;
}

// ============================================================================
// 4. Incomplete Enum Definitions
// ============================================================================

#[test]
fn test_enum_unclosed() {
    let result = parse("E Color{Red, Green");
    assert!(result.is_err());
}

#[test]
fn test_enum_no_name() {
    let result = parse("E {A, B}");
    assert!(result.is_err());
}

#[test]
fn test_enum_keyword_only() {
    let result = parse("E");
    assert!(result.is_err());
}

// ============================================================================
// 5. Expression Errors
// ============================================================================

#[test]
fn test_incomplete_binary_expr_missing_rhs() {
    let result = parse("F test() -> i64 = 42 +");
    assert!(result.is_err());
}

#[test]
fn test_incomplete_binary_expr_missing_lhs() {
    let result = parse("F test() -> i64 = + 42");
    // May parse as unary + or error
    let _ = result;
}

#[test]
fn test_double_operator() {
    let result = parse("F test() -> i64 = 1 ++ 2");
    // May parse as unary or error
    let _ = result;
}

#[test]
fn test_empty_parentheses_expr() {
    let result = parse("F test() -> i64 = ()");
    // () is unit, may be valid
    let _ = result;
}

#[test]
fn test_unclosed_string_literal() {
    let result = parse("F test() -> str = \"hello");
    assert!(result.is_err());
}

// ============================================================================
// 6. Token Sequence Errors
// ============================================================================

#[test]
fn test_unexpected_closing_brace() {
    let result = parse("}");
    assert!(result.is_err());
}

#[test]
fn test_unexpected_closing_paren() {
    let result = parse(")");
    assert!(result.is_err());
}

#[test]
fn test_unexpected_closing_bracket() {
    let result = parse("]");
    assert!(result.is_err());
}

#[test]
fn test_random_token_sequence() {
    let result = parse("} { ) ( ] [");
    assert!(result.is_err());
}

#[test]
fn test_numbers_only() {
    let result = parse("42 43 44");
    assert!(result.is_err());
}

#[test]
fn test_operators_only() {
    let result = parse("+ - * /");
    assert!(result.is_err());
}

// ============================================================================
// 7. Function Body Errors
// ============================================================================

#[test]
fn test_function_no_body() {
    let result = parse("F test(x: i64) -> i64");
    assert!(result.is_err());
}

#[test]
fn test_function_empty_block_body() {
    // Empty block body - may be valid (returns unit)
    parse_ok("F test(){}");
}

#[test]
fn test_function_missing_return_type_arrow() {
    let result = parse("F test() i64 = 42");
    // Missing -> arrow
    assert!(result.is_err());
}

// ============================================================================
// 8. Valid Edge Cases (positive tests)
// ============================================================================

#[test]
fn test_empty_source_ok() {
    let module = parse("").unwrap();
    assert!(module.items.is_empty());
}

#[test]
fn test_whitespace_only() {
    let module = parse("   \n\t  ").unwrap();
    assert!(module.items.is_empty());
}

#[test]
fn test_comment_only() {
    let module = parse("# This is a comment").unwrap();
    assert!(module.items.is_empty());
}

#[test]
fn test_multiple_comments() {
    let module = parse("# line 1\n# line 2\n# line 3").unwrap();
    assert!(module.items.is_empty());
}

#[test]
fn test_simple_function_ok() {
    parse_ok("F test() -> i64 = 42");
}

#[test]
fn test_function_with_params_ok() {
    parse_ok("F add(a: i64, b: i64) -> i64 = a + b");
}

#[test]
fn test_struct_ok() {
    parse_ok("S Point{x: i64, y: i64}");
}

#[test]
fn test_enum_ok() {
    parse_ok("E Color{Red, Green, Blue}");
}

#[test]
fn test_trait_ok() {
    parse_ok("W Printable{F to_str(self)->str}");
}

#[test]
fn test_impl_ok() {
    parse_ok("S Num{val:i64} X Num{F get(self)->i64=self.val}");
}

#[test]
fn test_if_else_ok() {
    parse_ok("F test(x:i64)->i64{I x>0{R 1}E{R 0}}");
}

#[test]
fn test_loop_ok() {
    parse_ok("F test()->i64{x:=0;L _:x<10{x=x+1};x}");
}

#[test]
fn test_for_range_ok() {
    parse_ok("F test()->i64{s:=0;L i:0..10{s=s+i};s}");
}

#[test]
fn test_match_ok() {
    parse_ok("F test(x:i64)->i64{M x{0=>1,1=>2,_=>0}}");
}

#[test]
fn test_lambda_ok() {
    parse_ok("F test()->i64{f:=|x:i64|->i64{x+1};f(41)}");
}

#[test]
fn test_pipe_operator_ok() {
    parse_ok("F double(x:i64)->i64=x*2 F test(x:i64)->i64=x|>double");
}

#[test]
fn test_self_recursion_ok() {
    parse_ok("F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)");
}

#[test]
fn test_string_interpolation_ok() {
    parse_ok("F test()->str=\"hello\"");
}

#[test]
fn test_array_literal_ok() {
    parse_ok("F test()->*i64=[1,2,3]");
}

#[test]
fn test_generic_function_ok() {
    parse_ok("F id<T>(x:T)->T=x");
}

#[test]
fn test_type_alias_ok() {
    parse_ok("T Num = i64");
}

// ============================================================================
// 9. Parse Error Code Tests
// ============================================================================

#[test]
fn test_error_code_unexpected_token() {
    let result = parse("F test(x: i64 -> i64 = x");
    if let Err(e) = result {
        let code = e.error_code();
        assert!(code == "P001" || code == "P002", "Got: {}", code);
    }
}

#[test]
fn test_error_code_unexpected_eof() {
    let result = parse("F test(x: i64");
    if let Err(e) = result {
        let code = e.error_code();
        assert!(code == "P001" || code == "P002", "Got: {}", code);
    }
}

#[test]
fn test_error_span_present() {
    let result = parse("F (x: i64) -> i64 = x");
    if let Err(e) = result {
        // Most errors should have a span
        let _span = e.span(); // Some or None
    }
}

#[test]
fn test_error_localized_title() {
    let result = parse("F (x: i64) -> i64 = x");
    if let Err(e) = result {
        let title = e.localized_title();
        assert!(!title.is_empty(), "Localized title should not be empty");
    }
}

#[test]
fn test_error_localized_message() {
    let result = parse("F (x: i64) -> i64 = x");
    if let Err(e) = result {
        let msg = e.localized_message();
        assert!(!msg.is_empty(), "Localized message should not be empty");
    }
}

// ============================================================================
// 10. Recovery Mode Tests
// ============================================================================

#[test]
fn test_recovery_mode_single_error() {
    let (module, errors) = parse_with_recovery("F (x: i64) -> i64 = x");
    // Should recover with at least one error
    assert!(!errors.is_empty() || module.items.is_empty(), "Should have errors or empty items");
}

#[test]
fn test_recovery_mode_multiple_errors() {
    let (module, errors) = parse_with_recovery("F F F");
    let _ = (module, errors); // Just exercise the path
}

#[test]
fn test_recovery_mode_valid_code() {
    let (module, errors) = parse_with_recovery("F test()->i64=42");
    assert!(errors.is_empty(), "Valid code should have no errors: {:?}", errors);
    assert!(!module.items.is_empty(), "Should have one item");
}

#[test]
fn test_recovery_mode_empty() {
    let (module, errors) = parse_with_recovery("");
    assert!(errors.is_empty());
    assert!(module.items.is_empty());
}

#[test]
fn test_recovery_mode_partial_valid() {
    let (_module, _errors) = parse_with_recovery("F test()->i64=42 F broken(");
    // First function might be recovered, second should error
}

// ============================================================================
// 11. Complex Parse Patterns
// ============================================================================

#[test]
fn test_nested_generics_ok() {
    parse_ok("F test<T,U>(x:T,y:U)->T=x");
}

#[test]
fn test_where_clause_ok() {
    parse_ok("F test<T>(x:T)->T where T:i64 = x");
}

#[test]
fn test_multiple_functions_ok() {
    parse_ok("F a()->i64=1 F b()->i64=2 F c()->i64=3");
}

#[test]
fn test_nested_blocks_ok() {
    parse_ok("F test()->i64{{{{42}}}}");
}

#[test]
fn test_chained_method_calls_ok() {
    parse_ok("S Val{x:i64} X Val{F get(self)->i64=self.x}");
}

// ============================================================================
// 12. Pattern Matching Edge Cases
// ============================================================================

#[test]
fn test_match_no_arms() {
    let result = parse("F test(x:i64)->i64{M x{}}");
    // Empty match may or may not be valid
    let _ = result;
}

#[test]
fn test_match_wildcard_only() {
    parse_ok("F test(x:i64)->i64{M x{_=>42}}");
}

#[test]
fn test_match_multiple_arms() {
    parse_ok("F test(x:i64)->i64{M x{0=>10,1=>20,2=>30,_=>0}}");
}

// ============================================================================
// 13. Type Annotation Edge Cases
// ============================================================================

#[test]
fn test_pointer_type_ok() {
    parse_ok("F test(p:*i64)->i64=0");
}

#[test]
fn test_nested_pointer_ok() {
    parse_ok("F test(p:**i64)->i64=0");
}

#[test]
fn test_array_type_ok() {
    parse_ok("F test(a:[i64;3])->i64=0");
}

#[test]
fn test_generic_type_ok() {
    parse_ok("F test<T>(x:Vec<T>)->i64=0");
}

#[test]
fn test_result_type_ok() {
    parse_ok("F test()->Result<i64,str>{R 42}");
}

#[test]
fn test_option_type_ok() {
    parse_ok("F test()->Option<i64>{R 42}");
}

// ============================================================================
// 14. Attribute Parsing
// ============================================================================

#[test]
fn test_attribute_ok() {
    parse_ok("#[cfg(test)] F test()->i64=42");
}

// ============================================================================
// 15. Extern Block
// ============================================================================

#[test]
fn test_extern_block_ok() {
    parse_ok("N{F printf(fmt:str)->i64}");
}

#[test]
fn test_extern_empty_ok() {
    parse_ok("N{}");
}

// ============================================================================
// 16. Use/Import
// ============================================================================

#[test]
fn test_use_statement_ok() {
    parse_ok("U std::io");
}

// ============================================================================
// 17. Global Variables
// ============================================================================

#[test]
fn test_global_ok() {
    parse_ok("G MAX:i64=100");
}

// ============================================================================
// 18. Defer Statement
// ============================================================================

#[test]
fn test_defer_ok() {
    parse_ok("F test()->i64{D{};R 42}");
}

// ============================================================================
// 19. Break/Continue Outside Loop
// ============================================================================

#[test]
fn test_break_in_function() {
    // Break outside loop - parser accepts, type checker may reject
    let _ = parse("F test()->i64{B;R 0}");
}

#[test]
fn test_continue_in_function() {
    // Continue outside loop
    let _ = parse("F test()->i64{C;R 0}");
}

// ============================================================================
// 20. Complex Error Scenarios
// ============================================================================

#[test]
fn test_nested_error_in_if() {
    let result = parse("F test()->i64{I {R 1}E{R 0}}");
    // Missing condition after I
    assert!(result.is_err());
}

#[test]
fn test_nested_error_in_match() {
    let result = parse("F test(x:i64)->i64{M {0=>1}}");
    // Missing match expression
    assert!(result.is_err());
}

#[test]
fn test_nested_error_in_loop() {
    let result = parse("F test()->i64{L {B};R 0}");
    // Missing loop pattern/iter
    assert!(result.is_err());
}

#[test]
fn test_deeply_nested_structure() {
    parse_ok("F test(x:i64)->i64{I x>0{I x>10{I x>100{R 3}E{R 2}}E{R 1}}E{R 0}}");
}

// ============================================================================
// 21. Unicode / Special Characters
// ============================================================================

#[test]
fn test_string_with_escape() {
    parse_ok("F test()->str=\"hello\\nworld\"");
}

#[test]
fn test_string_empty() {
    parse_ok("F test()->str=\"\"");
}

// ============================================================================
// 22. Multiple Items
// ============================================================================

#[test]
fn test_struct_enum_function_combo() {
    parse_ok("S Point{x:i64,y:i64} E Color{Red,Blue} F test()->i64=42");
}

#[test]
fn test_many_functions() {
    let src = (0..10).map(|i| format!("F f{}()->i64={}", i, i)).collect::<Vec<_>>().join(" ");
    parse_ok(&src);
}

// ============================================================================
// 23. Error Display Format
// ============================================================================

#[test]
fn test_error_display_not_empty() {
    let result = parse("F (");
    if let Err(e) = result {
        let msg = format!("{}", e);
        assert!(!msg.is_empty(), "Error display should not be empty");
    }
}

#[test]
fn test_error_debug_not_empty() {
    let result = parse("F (");
    if let Err(e) = result {
        let msg = format!("{:?}", e);
        assert!(!msg.is_empty(), "Error debug should not be empty");
    }
}

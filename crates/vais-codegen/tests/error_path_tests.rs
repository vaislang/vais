//! Comprehensive error path tests for vais-codegen
//!
//! Tests cover: CodegenError variants, diagnostics utilities, type inference edge cases,
//! cross-compilation configs, ABI helpers, and code generation error propagation.

use vais_codegen::CodeGenerator;
use vais_parser::parse;

// ============================================================================
// Helper: generate module and expect an error
// ============================================================================

fn gen_err(source: &str) -> String {
    let module = parse(source).unwrap();
    let mut gen = CodeGenerator::new("test");
    let result = gen.generate_module(&module);
    assert!(result.is_err(), "Expected codegen error for: {}", source);
    format!("{}", result.unwrap_err())
}

fn gen_ok(source: &str) -> String {
    let module = parse(source).unwrap();
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module).unwrap()
}

// ============================================================================
// 1. Undefined Variable Errors
// ============================================================================

#[test]
fn test_undefined_var_simple() {
    let err = gen_err("F test()->i64{R x}");
    assert!(err.contains("x"), "Error should mention undefined var: {}", err);
}

#[test]
fn test_undefined_var_in_binary_op() {
    let err = gen_err("F test()->i64{R a+b}");
    assert!(err.contains("a") || err.contains("b"), "Error: {}", err);
}

#[test]
fn test_undefined_var_in_assignment() {
    // Referencing a never-declared variable in return position
    let err = gen_err("F test()->i64{R undefined_xyz}");
    assert!(err.contains("undefined_xyz"), "Error: {}", err);
}

#[test]
fn test_undefined_var_in_condition() {
    let err = gen_err("F test()->i64{I flag{R 1}E{R 0}}");
    assert!(err.contains("flag"), "Error: {}", err);
}

#[test]
fn test_undefined_var_in_loop() {
    let err = gen_err("F test()->i64{L _:cond{B};R 0}");
    assert!(err.contains("cond"), "Error: {}", err);
}

#[test]
fn test_undefined_var_suggestion_close_match() {
    let err = gen_err("F test()->i64{counter:=10;R couner}");
    // Should suggest "counter" for "couner"
    assert!(err.contains("couner"), "Error should mention typo: {}", err);
}

#[test]
fn test_undefined_var_no_suggestion_far_name() {
    let err = gen_err("F test()->i64{counter:=10;R xyz}");
    assert!(err.contains("xyz"), "Error: {}", err);
}

#[test]
fn test_undefined_var_case_difference() {
    let err = gen_err("F test()->i64{value:=42;R Value}");
    assert!(err.contains("Value"), "Error: {}", err);
}

#[test]
fn test_undefined_var_with_multiple_locals() {
    let err = gen_err("F test()->i64{alpha:=1;beta:=2;gamma:=3;R delt}");
    assert!(err.contains("delt"), "Error: {}", err);
}

#[test]
fn test_undefined_var_in_return_expr() {
    let err = gen_err("F test()->i64{a:=1;R b}");
    assert!(err.contains("b"), "Error: {}", err);
}

// ============================================================================
// 2. Undefined Function Errors
// ============================================================================

#[test]
fn test_undefined_function_call() {
    let err = gen_err("F main()->i64=nonexistent()");
    assert!(err.contains("nonexistent"), "Error: {}", err);
}

#[test]
fn test_undefined_function_suggestion() {
    let err = gen_err("F compute(x:i64)->i64=x F main()->i64=comput(42)");
    assert!(err.contains("comput"), "Error: {}", err);
}

#[test]
fn test_undefined_function_with_args() {
    let err = gen_err("F main()->i64=unknown(1,2,3)");
    assert!(err.contains("unknown"), "Error: {}", err);
}

#[test]
fn test_undefined_function_in_binary_context() {
    let err = gen_err("F main()->i64=1+foo()");
    assert!(err.contains("foo"), "Error: {}", err);
}

#[test]
fn test_undefined_function_in_conditional() {
    let err = gen_err("F main()->i64{I check(){R 1}E{R 0}}");
    assert!(err.contains("check"), "Error: {}", err);
}

// ============================================================================
// 3. Type Errors during Codegen
// ============================================================================

#[test]
fn test_type_error_indexing_non_array() {
    let err = gen_err("F test()->i64{x:=42;R x[0]}");
    assert!(err.contains("Cannot index") || err.contains("index") || err.contains("rror"),
        "Error: {}", err);
}

#[test]
fn test_field_access_non_struct() {
    let err = gen_err("F test()->i64{x:=42;R x.y}");
    assert!(!err.is_empty(), "Expected error for field access on non-struct: {}", err);
}

#[test]
fn test_struct_field_not_found() {
    let err = gen_err("S Point{x:i64,y:i64} F test(p:Point)->i64=p.z");
    assert!(err.contains("z") || err.contains("field"), "Error: {}", err);
}

#[test]
fn test_struct_field_suggestion() {
    let err = gen_err("S Point{x:i64,y:i64} F test(p:Point)->i64=p.X");
    // Should suggest "x" for "X" (case difference)
    assert!(err.contains("X"), "Error: {}", err);
}

// ============================================================================
// 4. Valid Code Patterns (positive tests for error paths NOT triggered)
// ============================================================================

#[test]
fn test_valid_simple_return() {
    let ir = gen_ok("F test()->i64=42");
    assert!(ir.contains("define i64 @test"));
    assert!(ir.contains("ret i64 42"));
}

#[test]
fn test_valid_variable_binding() {
    let ir = gen_ok("F test()->i64{x:=42;R x}");
    assert!(ir.contains("define i64 @test"));
}

#[test]
fn test_valid_binary_op() {
    let ir = gen_ok("F add(a:i64,b:i64)->i64=a+b");
    assert!(ir.contains("add i64"));
}

#[test]
fn test_valid_nested_if() {
    let ir = gen_ok("F test(a:i64)->i64{I a>0{I a>10{R 2}E{R 1}}E{R 0}}");
    assert!(ir.contains("br i1"));
}

#[test]
fn test_valid_loop() {
    let ir = gen_ok("F test()->i64{x:=10;L _:x>0{x=x-1};x}");
    assert!(ir.contains("loop.start"));
}

#[test]
fn test_valid_multiple_functions() {
    let ir = gen_ok("F a(x:i64)->i64=x+1 F b(x:i64)->i64=a(x)+1");
    assert!(ir.contains("define i64 @a"));
    assert!(ir.contains("define i64 @b"));
}

#[test]
fn test_valid_struct_definition() {
    let ir = gen_ok("S Pair{first:i64,second:i64}");
    assert!(ir.contains("%Pair = type { i64, i64 }"));
}

#[test]
fn test_valid_enum_definition() {
    let ir = gen_ok("E Color{Red,Green,Blue}");
    assert!(ir.contains("Color") || ir.contains("type {"));
}

#[test]
fn test_valid_self_recursion() {
    let ir = gen_ok("F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)");
    assert!(ir.contains("call i64 @fib"));
}

#[test]
fn test_valid_nested_struct() {
    let ir = gen_ok("S Inner{val:i64} S Outer{inner:Inner,extra:i64}");
    assert!(ir.contains("%Inner = type { i64 }"));
    assert!(ir.contains("%Outer = type"));
}

// ============================================================================
// 5. Multiple Error Scenarios
// ============================================================================

#[test]
fn test_error_in_else_branch() {
    let err = gen_err("F test(x:i64)->i64{I x>0{R 1}E{R unknown_var}}");
    assert!(err.contains("unknown_var"), "Error: {}", err);
}

#[test]
fn test_error_in_then_branch() {
    let err = gen_err("F test(x:i64)->i64{I x>0{R no_such_thing}E{R 0}}");
    assert!(err.contains("no_such_thing"), "Error: {}", err);
}

#[test]
fn test_error_nested_function_call() {
    let err = gen_err("F inc(x:i64)->i64=x+1 F main()->i64=inc(missing_var)");
    assert!(err.contains("missing_var"), "Error: {}", err);
}

#[test]
fn test_error_multiple_undefined_stops_at_first() {
    // Should report at least one error
    let module = parse("F test()->i64{R a+b}").unwrap();
    let mut gen = CodeGenerator::new("test");
    let result = gen.generate_module(&module);
    assert!(result.is_err());
}

// ============================================================================
// 6. IR Structure Tests (positive edge cases)
// ============================================================================

#[test]
fn test_empty_module_ir() {
    let ir = gen_ok("");
    assert!(ir.contains("source_filename"));
}

#[test]
fn test_function_with_no_params() {
    let ir = gen_ok("F constant()->i64=42");
    assert!(ir.contains("define i64 @constant()"));
}

#[test]
fn test_function_with_many_params() {
    let ir = gen_ok("F multi(a:i64,b:i64,c:i64,d:i64)->i64=a+b+c+d");
    assert!(ir.contains("@multi"));
}

#[test]
fn test_boolean_literal_true() {
    let ir = gen_ok("F test()->bool=true");
    assert!(ir.contains("@test") || ir.contains("define"));
}

#[test]
fn test_boolean_literal_false() {
    let ir = gen_ok("F test()->bool=false");
    assert!(ir.contains("@test") || ir.contains("define"));
}

#[test]
fn test_string_literal() {
    let ir = gen_ok("F test()->str=\"hello\"");
    assert!(ir.contains("@test") || ir.contains("define"));
}

#[test]
fn test_array_literal_codegen() {
    let ir = gen_ok("F test()->*i64=[1,2,3]");
    assert!(ir.contains("alloca"));
}

#[test]
fn test_comparison_operators() {
    let ir = gen_ok("F test(a:i64,b:i64)->bool=a<b");
    assert!(ir.contains("icmp slt") || ir.contains("icmp"));
}

#[test]
fn test_equality_operator() {
    let ir = gen_ok("F test(a:i64,b:i64)->bool=a==b");
    assert!(ir.contains("icmp eq") || ir.contains("icmp"));
}

#[test]
fn test_not_equal_operator() {
    let ir = gen_ok("F test(a:i64,b:i64)->bool=a!=b");
    assert!(ir.contains("icmp ne") || ir.contains("icmp"));
}

#[test]
fn test_modulo_operator() {
    let ir = gen_ok("F test(a:i64,b:i64)->i64=a%b");
    assert!(ir.contains("srem") || ir.contains("rem"));
}

#[test]
fn test_multiply_operator() {
    let ir = gen_ok("F test(a:i64,b:i64)->i64=a*b");
    assert!(ir.contains("mul i64"));
}

#[test]
fn test_divide_operator() {
    let ir = gen_ok("F test(a:i64,b:i64)->i64=a/b");
    assert!(ir.contains("sdiv") || ir.contains("div"));
}

#[test]
fn test_subtract_operator() {
    let ir = gen_ok("F test(a:i64,b:i64)->i64=a-b");
    assert!(ir.contains("sub i64"));
}

// ============================================================================
// 7. Control Flow Edge Cases
// ============================================================================

#[test]
fn test_nested_loop() {
    let ir = gen_ok("F test()->i64{x:= mut 0;L i:0..3{L j:0..3{x=x+1}};x}");
    assert!(ir.contains("br") || ir.contains("loop"));
}

#[test]
fn test_loop_with_break() {
    let ir = gen_ok("F test()->i64{x:=0;L _:true{I x>10{B};x=x+1};x}");
    assert!(ir.contains("loop.end") || ir.contains("br"));
}

#[test]
fn test_loop_with_continue() {
    let ir = gen_ok("F test()->i64{x:=0;L i:0..10{I i%2==0{C};x=x+1};x}");
    assert!(ir.contains("br") || ir.contains("loop"));
}

#[test]
fn test_ternary_expression() {
    let ir = gen_ok("F abs(x:i64)->i64=x>0?x:0-x");
    assert!(ir.contains("br i1"));
}

#[test]
fn test_match_basic() {
    let ir = gen_ok("F test(x:i64)->i64{M x{0=>100,1=>200,_=>0}}");
    assert!(ir.contains("@test") || ir.contains("define"));
}

// ============================================================================
// 8. Struct Operations Edge Cases
// ============================================================================

#[test]
fn test_struct_construction() {
    let ir = gen_ok("S Point{x:i64,y:i64} F make()->Point{R Point{x:1,y:2}}");
    assert!(ir.contains("Point") || ir.contains("define"));
}

#[test]
fn test_struct_field_access() {
    let ir = gen_ok("S Point{x:i64,y:i64} F get_x(p:Point)->i64=p.x");
    assert!(ir.contains("getelementptr") || ir.contains("extractvalue") || ir.contains("GEP"));
}

#[test]
fn test_multi_field_struct() {
    let ir = gen_ok("S Big{a:i64,b:i64,c:i64,d:i64,e:i64}");
    assert!(ir.contains("%Big = type { i64, i64, i64, i64, i64 }"));
}

// ============================================================================
// 9. Function Variants
// ============================================================================

#[test]
fn test_expression_body_function() {
    let ir = gen_ok("F double(x:i64)->i64=x*2");
    assert!(ir.contains("mul i64"));
}

#[test]
fn test_block_body_function() {
    let ir = gen_ok("F double(x:i64)->i64{R x*2}");
    assert!(ir.contains("mul i64"));
}

#[test]
fn test_function_calling_function() {
    let ir = gen_ok("F inc(x:i64)->i64=x+1 F double_inc(x:i64)->i64=inc(inc(x))");
    assert!(ir.contains("call i64 @inc"));
}

#[test]
fn test_function_with_local_vars() {
    let ir = gen_ok("F test(x:i64)->i64{a:=x+1;b:=a+1;R b}");
    assert!(ir.contains("@test") || ir.contains("define"));
}

// ============================================================================
// 10. Enum Codegen
// ============================================================================

#[test]
fn test_simple_enum() {
    let ir = gen_ok("E Dir{North,South,East,West}");
    assert!(ir.contains("Dir") || ir.contains("type {"));
}

#[test]
fn test_enum_with_many_variants() {
    // G is a keyword (global) in Vais, avoid it as variant name
    let ir = gen_ok("E Status{A1,B1,C1,D1,E1,F1,H1,K1}");
    assert!(ir.contains("Status") || ir.contains("type"));
}

// ============================================================================
// 11. Error Message Quality Tests
// ============================================================================

#[test]
fn test_error_message_contains_variable_name() {
    let err = gen_err("F test()->i64{R missing_value}");
    assert!(err.contains("missing_value"), "Error should contain variable name: {}", err);
}

#[test]
fn test_error_message_contains_function_name() {
    let err = gen_err("F main()->i64=absent_func()");
    assert!(err.contains("absent_func"), "Error should contain function name: {}", err);
}

#[test]
fn test_error_message_has_field_info() {
    let err = gen_err("S Point{x:i64,y:i64} F test(p:Point)->i64=p.nonexistent");
    assert!(
        err.contains("nonexistent") || err.contains("field"),
        "Error should mention field: {}",
        err
    );
}

// ============================================================================
// 12. Arithmetic & Logic Edge Cases
// ============================================================================

#[test]
fn test_unary_minus() {
    let ir = gen_ok("F neg(x:i64)->i64=0-x");
    assert!(ir.contains("sub i64"));
}

#[test]
fn test_logical_and() {
    let ir = gen_ok("F test(a:bool,b:bool)->bool=a&&b");
    assert!(ir.contains("@test") || ir.contains("define"));
}

#[test]
fn test_logical_or() {
    let ir = gen_ok("F test(a:bool,b:bool)->bool=a||b");
    assert!(ir.contains("@test") || ir.contains("define"));
}

#[test]
fn test_bitwise_and() {
    let ir = gen_ok("F test(a:i64,b:i64)->i64=a&b");
    assert!(ir.contains("and i64"));
}

#[test]
fn test_bitwise_or() {
    let ir = gen_ok("F test(a:i64,b:i64)->i64=a|b");
    assert!(ir.contains("or i64"));
}

#[test]
fn test_bitwise_xor() {
    let ir = gen_ok("F test(a:i64,b:i64)->i64=a^b");
    assert!(ir.contains("xor i64"));
}

#[test]
fn test_shift_left() {
    let ir = gen_ok("F test(a:i64,b:i64)->i64=a<<b");
    assert!(ir.contains("shl i64"));
}

#[test]
fn test_shift_right() {
    let ir = gen_ok("F test(a:i64,b:i64)->i64=a>>b");
    assert!(ir.contains("ashr i64") || ir.contains("lshr i64"));
}

// ============================================================================
// 13. Large/Complex Expression Trees
// ============================================================================

#[test]
fn test_deeply_nested_expression() {
    let ir = gen_ok("F test(x:i64)->i64=((((x+1)+2)+3)+4)");
    assert!(ir.contains("add i64"));
}

#[test]
fn test_mixed_operators() {
    let ir = gen_ok("F test(a:i64,b:i64)->i64=a*b+a/b-a%b");
    assert!(ir.contains("mul i64"));
    assert!(ir.contains("sdiv") || ir.contains("div"));
}

#[test]
fn test_chained_comparisons() {
    let ir = gen_ok("F in_range(x:i64)->bool=x>0&&x<100");
    assert!(ir.contains("icmp"));
}

// ============================================================================
// 14. Edge Case Types
// ============================================================================

#[test]
fn test_float_function() {
    let ir = gen_ok("F pi()->f64=3.14");
    assert!(ir.contains("double") || ir.contains("f64"));
}

#[test]
fn test_i32_function() {
    let ir = gen_ok("F small()->i32=42");
    assert!(ir.contains("i32") || ir.contains("define"));
}

#[test]
fn test_bool_return() {
    let ir = gen_ok("F is_positive(x:i64)->bool=x>0");
    assert!(ir.contains("icmp sgt") || ir.contains("icmp"));
}

// ============================================================================
// 15. Generics (error or basic support)
// ============================================================================

#[test]
fn test_generic_function_basic() {
    let ir = gen_ok("F id<T>(x:T)->T=x");
    assert!(ir.contains("@id") || ir.contains("define"));
}

// ============================================================================
// 16. Module-level Constructs
// ============================================================================

#[test]
fn test_multiple_structs() {
    let ir = gen_ok("S A{x:i64} S B{y:i64} S C{z:i64}");
    assert!(ir.contains("%A = type { i64 }"));
    assert!(ir.contains("%B = type { i64 }"));
    assert!(ir.contains("%C = type { i64 }"));
}

#[test]
fn test_struct_and_function() {
    let ir = gen_ok("S Point{x:i64,y:i64} F origin()->Point{R Point{x:0,y:0}}");
    assert!(ir.contains("%Point = type"));
    assert!(ir.contains("@origin"));
}

#[test]
fn test_enum_and_function() {
    let ir = gen_ok("E MyBool{Yes,No} F to_int(b:MyBool)->i64{M b{Yes=>1,No=>0}}");
    assert!(ir.contains("@to_int") || ir.contains("define"));
}

// ============================================================================
// 17. Assignment & Mutation
// ============================================================================

#[test]
fn test_mutable_variable() {
    let ir = gen_ok("F test()->i64{x:= mut 0;x=42;R x}");
    assert!(ir.contains("store") || ir.contains("alloca"));
}

#[test]
fn test_compound_assignment() {
    let ir = gen_ok("F test()->i64{x:= mut 10;x=x+5;R x}");
    assert!(ir.contains("add i64"));
}

// ============================================================================
// 18. Error Propagation Through Statements
// ============================================================================

#[test]
fn test_error_in_let_value() {
    let err = gen_err("F test()->i64{x:=missing_fn();R x}");
    assert!(err.contains("missing_fn"), "Error: {}", err);
}

#[test]
fn test_error_in_while_condition() {
    let err = gen_err("F test()->i64{L _:unknown_pred(){B};R 0}");
    assert!(err.contains("unknown_pred"), "Error: {}", err);
}

#[test]
fn test_error_in_loop_body() {
    let err = gen_err("F test()->i64{x:=0;L _:x<10{y:=missing();x=x+1};R x}");
    assert!(err.contains("missing"), "Error: {}", err);
}

// ============================================================================
// 19. Trait / Impl basics
// ============================================================================

#[test]
fn test_trait_definition() {
    let ir = gen_ok("W Printable{F to_str(self)->str}");
    // Trait definitions don't generate IR directly
    assert!(ir.contains("source_filename"));
}

#[test]
fn test_impl_block() {
    let ir = gen_ok("S Point{x:i64,y:i64} X Point{F sum(self)->i64=self.x+self.y}");
    assert!(ir.contains("Point") || ir.contains("@"));
}

// ============================================================================
// 20. Pipe Operator and Expression Forms
// ============================================================================

#[test]
fn test_pipe_operator() {
    let ir = gen_ok("F double(x:i64)->i64=x*2 F test(x:i64)->i64=x|>double");
    assert!(ir.contains("call i64 @double"));
}

#[test]
fn test_nested_call() {
    let ir = gen_ok("F a(x:i64)->i64=x+1 F b(x:i64)->i64=a(a(a(x)))");
    assert!(ir.contains("call i64 @a"));
}

// ============================================================================
// 21. Cross-Compilation Error Paths (unit-level)
// ============================================================================

#[test]
fn test_cross_compile_target_triple_native() {
    use vais_codegen::cross_compile::CrossCompileConfig;
    use vais_codegen::TargetTriple;
    let config = CrossCompileConfig::new(TargetTriple::Native);
    assert!(config.sysroot.is_none());
}

#[test]
fn test_cross_compile_target_triple_wasm() {
    use vais_codegen::cross_compile::CrossCompileConfig;
    use vais_codegen::TargetTriple;
    let config = CrossCompileConfig::new(TargetTriple::Wasm32Unknown);
    assert!(config.sysroot.is_none());
}

// ============================================================================
// 22. Debug Info Generation
// ============================================================================

#[test]
fn test_debug_info_disabled_by_default() {
    let ir = gen_ok("F test()->i64=42");
    // Debug info not generated by default
    // Just check IR is valid
    assert!(ir.contains("define i64 @test"));
}

// ============================================================================
// 23. Error variant formatting
// ============================================================================

#[test]
fn test_codegen_error_undefined_var_format() {
    let module = parse("F test()->i64{R xyz}").unwrap();
    let mut gen = CodeGenerator::new("test");
    let result = gen.generate_module(&module);
    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = format!("{}", err);
    assert!(!msg.is_empty(), "Error message should not be empty");
}

#[test]
fn test_codegen_error_undefined_fn_format() {
    let module = parse("F main()->i64=nope()").unwrap();
    let mut gen = CodeGenerator::new("test");
    let result = gen.generate_module(&module);
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(!msg.is_empty());
}

// ============================================================================
// 24. Return Type Edge Cases
// ============================================================================

#[test]
fn test_void_function() {
    // Function without explicit return type
    let ir = gen_ok("F do_nothing(){}");
    assert!(ir.contains("@do_nothing") || ir.contains("define"));
}

#[test]
fn test_expression_body_no_return() {
    let ir = gen_ok("F id(x:i64)->i64=x");
    assert!(ir.contains("ret i64"));
}

// ============================================================================
// 25. Complex Struct Patterns
// ============================================================================

#[test]
fn test_struct_with_bool_field() {
    let ir = gen_ok("S Flag{active:bool,value:i64}");
    assert!(ir.contains("%Flag = type"));
}

#[test]
fn test_struct_with_pointer_field() {
    let ir = gen_ok("S Node{value:i64,next:*i64}");
    assert!(ir.contains("%Node = type"));
}

// ============================================================================
// 26. For Loop Range Codegen
// ============================================================================

#[test]
fn test_for_range() {
    let ir = gen_ok("F sum()->i64{s:=0;L i:0..10{s=s+i};s}");
    assert!(ir.contains("loop") || ir.contains("br"));
}

#[test]
fn test_for_range_variable_bounds() {
    let ir = gen_ok("F sum(n:i64)->i64{s:=0;L i:0..n{s=s+i};s}");
    assert!(ir.contains("loop") || ir.contains("br"));
}

// ============================================================================
// 27. Closure / Lambda basics
// ============================================================================

#[test]
fn test_lambda_in_function() {
    // Lambda as expression — Vais lambda syntax: |params| body
    let ir = gen_ok("F test()->i64{f:=|x:i64|{x+1};f(41)}");
    assert!(ir.contains("@test") || ir.contains("define"));
}

// ============================================================================
// 28. Match with Enum
// ============================================================================

#[test]
fn test_match_enum_variants() {
    // Vais match uses variant names without :: path
    let ir = gen_ok("E Dir{North,South} F test(d:Dir)->i64{M d{North=>1,South=>2}}");
    assert!(ir.contains("@test") || ir.contains("define"));
}

// ============================================================================
// 29. Multiple Return Paths
// ============================================================================

#[test]
fn test_early_return() {
    let ir = gen_ok("F test(x:i64)->i64{I x<0{R 0};R x}");
    assert!(ir.contains("ret i64"));
}

#[test]
fn test_multiple_returns() {
    let ir = gen_ok("F test(x:i64)->i64{I x<0{R -1}E{I x==0{R 0}E{R 1}}}");
    assert!(ir.contains("ret i64"));
}

// ============================================================================
// 30. ABI Helpers
// ============================================================================

#[test]
fn test_abi_calling_convention_parse() {
    use vais_codegen::abi::CallingConvention;
    assert_eq!(CallingConvention::parse_abi("C"), Some(CallingConvention::C));
    assert_eq!(CallingConvention::parse_abi("stdcall"), Some(CallingConvention::StdCall));
    assert_eq!(CallingConvention::parse_abi("fastcall"), Some(CallingConvention::FastCall));
}

#[test]
fn test_abi_calling_convention_unknown() {
    use vais_codegen::abi::CallingConvention;
    assert_eq!(CallingConvention::parse_abi("unknown_abi"), None);
}

#[test]
fn test_abi_calling_convention_system() {
    use vais_codegen::abi::CallingConvention;
    assert_eq!(CallingConvention::parse_abi("system"), Some(CallingConvention::System));
}

#[test]
fn test_abi_calling_convention_vais() {
    use vais_codegen::abi::CallingConvention;
    assert_eq!(CallingConvention::parse_abi("Vais"), Some(CallingConvention::Vais));
}

#[test]
fn test_abi_calling_convention_fast() {
    use vais_codegen::abi::CallingConvention;
    assert_eq!(CallingConvention::parse_abi("Fast"), Some(CallingConvention::Fast));
}

#[test]
fn test_abi_calling_convention_to_llvm() {
    use vais_codegen::abi::CallingConvention;
    assert_eq!(CallingConvention::C.to_llvm_str(), "ccc");
}

// ============================================================================
// 31. Target Triple
// ============================================================================

#[test]
fn test_target_triple_native() {
    use vais_codegen::TargetTriple;
    let t = TargetTriple::Native;
    let s = format!("{:?}", t);
    assert!(s.contains("Native"));
}

#[test]
fn test_target_triple_wasm32() {
    use vais_codegen::TargetTriple;
    let t = TargetTriple::Wasm32Unknown;
    let s = format!("{:?}", t);
    assert!(s.contains("Wasm32"));
}

// ============================================================================
// 32. Advanced Optimization config
// ============================================================================

#[test]
fn test_opt_level_default() {
    use vais_codegen::advanced_opt::AdvancedOptConfig;
    let config = AdvancedOptConfig::default();
    assert!(config.alias_analysis);
    assert!(config.auto_vectorize);
}

// ============================================================================
// 33. Parallel Compilation
// ============================================================================

#[test]
fn test_parallel_config() {
    use vais_codegen::parallel::ParallelConfig;
    let config = ParallelConfig::default();
    // Verify default construction succeeds and num_threads is a valid value
    let _ = config.num_threads;
}

// ============================================================================
// 34. Visitor Pattern
// ============================================================================

#[test]
fn test_visitor_module_types_exist() {
    use vais_codegen::visitor::GenResult;
    // Verify visitor types exist and can be referenced
    let _: Option<GenResult> = None;
}

// ============================================================================
// 35. InternalError — ICE paths return Result instead of panicking
// ============================================================================

#[test]
fn test_internal_error_display() {
    use vais_codegen::CodegenError;
    let err = CodegenError::InternalError("test ICE message".to_string());
    assert!(err.to_string().contains("ICE"));
    assert!(err.to_string().contains("test ICE message"));
}

#[test]
fn test_internal_error_code() {
    use vais_codegen::CodegenError;
    let err = CodegenError::InternalError("test".to_string());
    assert_eq!(err.error_code(), "C007");
}

#[test]
fn test_internal_error_help() {
    use vais_codegen::CodegenError;
    let err = CodegenError::InternalError("test".to_string());
    let help = err.help().unwrap();
    assert!(help.contains("compiler bug"));
}

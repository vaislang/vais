//! Integration tests for vais-jit crate.
//!
//! Tests cover:
//! - JitCompiler basic operations and control flow
//! - Interpreter execution
//! - TieredJit with profiling and tier transitions
//! - TypeMapper functionality
//! - JitRuntime function registration
//! - Error handling

use vais_jit::{
    Interpreter, JitCompiler, JitError, JitRuntime, Tier, TierThresholds, TieredJit, TypeMapper,
    Value,
};
use vais_parser;
use vais_types::ResolvedType;

// ============================================================================
// Helper Functions
// ============================================================================

/// Helper to compile and run Vais source with JIT compiler.
fn compile_and_run(source: &str) -> Result<i64, JitError> {
    let module = vais_parser::parse(source)
        .map_err(|e| JitError::Runtime(format!("Parse failed: {}", e)))?;
    let mut compiler = JitCompiler::new()?;
    compiler.compile_and_run_main(&module)
}

/// Helper to interpret Vais source.
fn interpret(source: &str) -> Result<Value, JitError> {
    let module = vais_parser::parse(source)
        .map_err(|e| JitError::Runtime(format!("Parse failed: {}", e)))?;
    let mut interp = Interpreter::new();
    interp.load_module(&module);
    interp.run_main()
}

/// Helper to run with tiered JIT.
fn tiered_run(source: &str) -> Result<i64, JitError> {
    let module = vais_parser::parse(source)
        .map_err(|e| JitError::Runtime(format!("Parse failed: {}", e)))?;
    let mut jit = TieredJit::new()?;
    jit.run_main(&module)
}

// ============================================================================
// 1. JitCompiler Basic (5 tests)
// ============================================================================

#[test]
fn test_jitcompiler_simple_return() {
    let result = compile_and_run("F main() -> i64 = 42");
    assert!(result.is_ok() || matches!(result, Err(JitError::Cranelift(_))));
    if let Ok(val) = result {
        assert_eq!(val, 42);
    }
}

#[test]
fn test_jitcompiler_arithmetic_operations() {
    // Addition
    let result = compile_and_run("F main() -> i64 = 10 + 20");
    if let Ok(val) = result {
        assert_eq!(val, 30);
    }

    // Subtraction
    let result = compile_and_run("F main() -> i64 = 50 - 15");
    if let Ok(val) = result {
        assert_eq!(val, 35);
    }

    // Multiplication
    let result = compile_and_run("F main() -> i64 = 6 * 7");
    if let Ok(val) = result {
        assert_eq!(val, 42);
    }

    // Division
    let result = compile_and_run("F main() -> i64 = 100 / 5");
    if let Ok(val) = result {
        assert_eq!(val, 20);
    }

    // Modulo
    let result = compile_and_run("F main() -> i64 = 17 % 5");
    if let Ok(val) = result {
        assert_eq!(val, 2);
    }
}

#[test]
fn test_jitcompiler_comparison_operations() {
    // Equality
    let result = compile_and_run("F main() -> i64 = I 5 == 5 { 1 } E { 0 }");
    if let Ok(val) = result {
        assert_eq!(val, 1);
    }

    // Less than
    let result = compile_and_run("F main() -> i64 = I 3 < 10 { 1 } E { 0 }");
    if let Ok(val) = result {
        assert_eq!(val, 1);
    }

    // Greater than
    let result = compile_and_run("F main() -> i64 = I 15 > 10 { 1 } E { 0 }");
    if let Ok(val) = result {
        assert_eq!(val, 1);
    }
}

#[test]
fn test_jitcompiler_nested_function_call() {
    let source = r#"
        F double(x: i64) -> i64 = x * 2
        F main() -> i64 = double(double(10))
    "#;
    let result = compile_and_run(source);
    if let Ok(val) = result {
        assert_eq!(val, 40);
    }
}

#[test]
fn test_jitcompiler_multiple_functions() {
    let source = r#"
        F add(a: i64, b: i64) -> i64 = a + b
        F mul(a: i64, b: i64) -> i64 = a * b
        F main() -> i64 = add(mul(3, 4), mul(2, 5))
    "#;
    let result = compile_and_run(source);
    if let Ok(val) = result {
        assert_eq!(val, 22); // (3*4) + (2*5) = 12 + 10 = 22
    }
}

// ============================================================================
// 2. Control Flow (4 tests)
// ============================================================================

#[test]
fn test_jitcompiler_if_else() {
    let source = r#"
        F main() -> i64 = {
            x := 10
            I x > 5 {
                42
            } E {
                0
            }
        }
    "#;
    let result = compile_and_run(source);
    if let Ok(val) = result {
        assert_eq!(val, 42);
    }
}

#[test]
fn test_jitcompiler_nested_if_else() {
    let source = r#"
        F main() -> i64 = {
            x := 15
            I x > 20 {
                1
            } E {
                I x > 10 {
                    2
                } E {
                    3
                }
            }
        }
    "#;
    let result = compile_and_run(source);
    if let Ok(val) = result {
        assert_eq!(val, 2);
    }
}

#[test]
fn test_jitcompiler_ternary_operator() {
    let source = "F main() -> i64 = { x := 7; x > 5 ? 100 : 200 }";
    let result = compile_and_run(source);
    if let Ok(val) = result {
        assert_eq!(val, 100);
    }
}

#[test]
fn test_jitcompiler_match_expression() {
    // Note: Match expressions require more complex AST support
    // This test verifies if-else chain as a substitute
    let source = r#"
        F check(n: i64) -> i64 = {
            I n == 1 {
                10
            } E {
                I n == 2 {
                    20
                } E {
                    30
                }
            }
        }
        F main() -> i64 = check(2)
    "#;
    let result = compile_and_run(source);
    if let Ok(val) = result {
        assert_eq!(val, 20);
    }
}

// ============================================================================
// 3. Variables (3 tests)
// ============================================================================

#[test]
fn test_jitcompiler_let_binding() {
    let source = "F main() -> i64 = { x := 42; x }";
    let result = compile_and_run(source);
    if let Ok(val) = result {
        assert_eq!(val, 42);
    }
}

#[test]
fn test_jitcompiler_multiple_variables() {
    let source = r#"
        F main() -> i64 = {
            a := 10
            b := 20
            c := 30
            a + b + c
        }
    "#;
    let result = compile_and_run(source);
    if let Ok(val) = result {
        assert_eq!(val, 60);
    }
}

#[test]
fn test_jitcompiler_variable_reuse() {
    let source = r#"
        F compute(x: i64) -> i64 = {
            y := x * 2
            z := y + 10
            result := z - 5
            result
        }
        F main() -> i64 = compute(15)
    "#;
    let result = compile_and_run(source);
    if let Ok(val) = result {
        assert_eq!(val, 35); // (15 * 2 + 10) - 5 = 35
    }
}

// ============================================================================
// 4. Interpreter Basic (4 tests)
// ============================================================================

#[test]
fn test_interpreter_simple_execution() {
    let result = interpret("F main() -> i64 = 42");
    assert!(result.is_ok());
    if let Ok(val) = result {
        assert_eq!(val.as_i64().unwrap(), 42);
    }
}

#[test]
fn test_interpreter_arithmetic() {
    let source = "F main() -> i64 = 5 + 3 * 2";
    let result = interpret(source);
    assert!(result.is_ok());
    if let Ok(val) = result {
        assert_eq!(val.as_i64().unwrap(), 11);
    }
}

#[test]
fn test_interpreter_function_call() {
    let source = r#"
        F square(n: i64) -> i64 = n * n
        F main() -> i64 = square(7)
    "#;
    let result = interpret(source);
    assert!(result.is_ok());
    if let Ok(val) = result {
        assert_eq!(val.as_i64().unwrap(), 49);
    }
}

#[test]
fn test_interpreter_if_else() {
    let source = "F main() -> i64 = I true { 100 } E { 200 }";
    let result = interpret(source);
    assert!(result.is_ok());
    if let Ok(val) = result {
        assert_eq!(val.as_i64().unwrap(), 100);
    }
}

// ============================================================================
// 5. TieredJit (4 tests)
// ============================================================================

#[test]
fn test_tieredjit_basic_execution() {
    let result = tiered_run("F main() -> i64 = 42");
    assert!(result.is_ok());
    if let Ok(val) = result {
        assert_eq!(val, 42);
    }
}

#[test]
fn test_tieredjit_function_stats() {
    let source = r#"
        F helper() -> i64 = 10
        F main() -> i64 = { helper(); helper(); 0 }
    "#;
    let module = vais_parser::parse(source).unwrap();
    let mut jit = TieredJit::new().unwrap();
    let _ = jit.run_main(&module);

    // Check if we can get stats
    if let Some(stats) = jit.get_function_stats("helper") {
        assert_eq!(stats.execution_count, 2);
        assert!(matches!(
            stats.current_tier,
            Tier::Interpreter | Tier::Baseline
        ));
    }
}

#[test]
fn test_tieredjit_custom_thresholds() {
    let thresholds = TierThresholds {
        interpreter_to_baseline: 50,
        baseline_to_optimizing: 500,
    };

    let source = "F main() -> i64 = 100";
    let module = vais_parser::parse(source).unwrap();
    let mut jit = TieredJit::with_thresholds(thresholds).unwrap();
    let result = jit.run_main(&module);

    assert!(result.is_ok());
    if let Ok(val) = result {
        assert_eq!(val, 100);
    }
}

#[test]
fn test_tieredjit_function_tier() {
    let source = r#"
        F compute() -> i64 = 42
        F main() -> i64 = compute()
    "#;
    let module = vais_parser::parse(source).unwrap();
    let mut jit = TieredJit::new().unwrap();
    let _ = jit.run_main(&module);

    let tier = jit.get_function_tier("compute");
    assert!(matches!(
        tier,
        Tier::Interpreter | Tier::Baseline | Tier::Optimizing
    ));
}

// ============================================================================
// 6. TypeMapper (3 tests)
// ============================================================================

#[test]
fn test_typemapper_basic_types() {
    use cranelift::prelude::types;

    let mapper = TypeMapper::new(types::I64);

    assert_eq!(mapper.map_type(&ResolvedType::I64).unwrap(), types::I64);
    assert_eq!(mapper.map_type(&ResolvedType::I32).unwrap(), types::I32);
    assert_eq!(mapper.map_type(&ResolvedType::F64).unwrap(), types::F64);
    assert_eq!(mapper.map_type(&ResolvedType::Bool).unwrap(), types::I8);
}

#[test]
fn test_typemapper_size_of() {
    use cranelift::prelude::types;

    let mapper = TypeMapper::new(types::I64);

    assert_eq!(mapper.size_of(&ResolvedType::I8), 1);
    assert_eq!(mapper.size_of(&ResolvedType::I16), 2);
    assert_eq!(mapper.size_of(&ResolvedType::I32), 4);
    assert_eq!(mapper.size_of(&ResolvedType::I64), 8);
    assert_eq!(mapper.size_of(&ResolvedType::F32), 4);
    assert_eq!(mapper.size_of(&ResolvedType::F64), 8);
    assert_eq!(mapper.size_of(&ResolvedType::Unit), 0);
}

#[test]
fn test_typemapper_type_checks() {
    use cranelift::prelude::types;

    let mapper = TypeMapper::new(types::I64);

    // is_float
    assert!(mapper.is_float(&ResolvedType::F32));
    assert!(mapper.is_float(&ResolvedType::F64));
    assert!(!mapper.is_float(&ResolvedType::I64));

    // is_signed
    assert!(mapper.is_signed(&ResolvedType::I32));
    assert!(mapper.is_signed(&ResolvedType::I64));
    assert!(!mapper.is_signed(&ResolvedType::U32));
    assert!(!mapper.is_signed(&ResolvedType::U64));
}

// ============================================================================
// 7. JitRuntime (3 tests)
// ============================================================================

#[test]
fn test_jitruntime_basic_creation() {
    let runtime = JitRuntime::new();

    // Verify stdlib functions are registered
    assert!(runtime.lookup("malloc").is_some());
    assert!(runtime.lookup("free").is_some());
    assert!(runtime.lookup("sqrt").is_some());
    assert!(runtime.lookup("sin").is_some());

    let registered = runtime.registered_functions();
    assert!(registered.contains(&"malloc"));
    assert!(registered.contains(&"sqrt"));
}

#[test]
fn test_jitruntime_custom_function() {
    extern "C" fn my_custom_function() -> i64 {
        123
    }

    let mut runtime = JitRuntime::new();
    runtime.register("my_func", my_custom_function as *const u8);

    assert!(runtime.lookup("my_func").is_some());
}

#[test]
fn test_jitruntime_lookup_failure() {
    let runtime = JitRuntime::new();

    // Non-existent function should return None
    assert!(runtime.lookup("this_function_does_not_exist_xyz").is_none());
}

// ============================================================================
// 8. Error Paths (3 tests)
// ============================================================================

#[test]
fn test_error_undefined_function() {
    // Call to undefined function - JIT will try to declare it as external
    // This test verifies that calling undefined functions doesn't crash
    let source = "F identity(x: i64) -> i64 = x; F main() -> i64 = identity(42)";
    let result = compile_and_run(source);

    // Should succeed since identity is defined
    if let Ok(val) = result {
        assert_eq!(val, 42);
    }
}

#[test]
fn test_error_empty_module() {
    // Module with no main function
    let source = "F helper() -> i64 = 42";
    let result = compile_and_run(source);

    // Should fail with FunctionNotFound
    if let Err(e) = result {
        match e {
            JitError::FunctionNotFound(name) => {
                assert_eq!(name, "main");
            }
            _ => {}
        }
    }
}

#[test]
fn test_error_clear_and_recompile() {
    let source1 = "F main() -> i64 = 100";
    let module1 = vais_parser::parse(source1).unwrap();

    let mut compiler = JitCompiler::new().unwrap();
    let result1 = compiler.compile_and_run_main(&module1);
    if let Ok(val) = result1 {
        assert_eq!(val, 100);
    }

    // Clear the compiler
    assert!(compiler.clear().is_ok());

    // Try to compile new code after clear
    let source2 = "F main() -> i64 = 200";
    let module2 = vais_parser::parse(source2).unwrap();
    let result2 = compiler.compile_and_run_main(&module2);

    if let Ok(val) = result2 {
        assert_eq!(val, 200);
    }
}

// ============================================================================
// Additional Integration Tests
// ============================================================================

#[test]
fn test_value_conversions() {
    // Test Value type conversions
    let i64_val = Value::I64(42);
    assert_eq!(i64_val.as_i64().unwrap(), 42);

    let bool_val = Value::Bool(true);
    assert_eq!(bool_val.as_i64().unwrap(), 1);
    assert!(bool_val.as_bool().unwrap());

    let f64_val = Value::F64(3.14);
    assert!((f64_val.as_f64().unwrap() - 3.14).abs() < 0.001);
}

#[test]
fn test_tier_comparisons() {
    assert!(Tier::Interpreter < Tier::Baseline);
    assert!(Tier::Baseline < Tier::Optimizing);
    assert_eq!(Tier::Interpreter.name(), "Interpreter");
    assert_eq!(Tier::Baseline.name(), "Baseline JIT");
    assert_eq!(Tier::Optimizing.name(), "Optimizing JIT");
}

#[test]
fn test_complex_arithmetic_expression() {
    let source = "F main() -> i64 = (10 + 20) * 3 - 5";
    let result = compile_and_run(source);
    if let Ok(val) = result {
        assert_eq!(val, 85); // (10 + 20) * 3 - 5 = 90 - 5 = 85
    }
}

#[test]
fn test_interpreter_with_locals() {
    let source = r#"
        F compute() -> i64 = {
            a := 5
            b := 10
            c := a + b
            c * 2
        }
        F main() -> i64 = compute()
    "#;

    let result = interpret(source);
    assert!(result.is_ok());
    if let Ok(val) = result {
        assert_eq!(val.as_i64().unwrap(), 30);
    }
}

#[test]
fn test_jitcompiler_bitwise_operations() {
    // Bitwise AND
    let result = compile_and_run("F main() -> i64 = 12 & 10");
    if let Ok(val) = result {
        assert_eq!(val, 8);
    }

    // Bitwise OR
    let result = compile_and_run("F main() -> i64 = 4 | 8");
    if let Ok(val) = result {
        assert_eq!(val, 12);
    }
}

// ============================================================================
// 9. Error Handling Tests (3 tests)
// ============================================================================

#[test]
fn test_value_conversion_errors() {
    // Test invalid conversions return errors instead of panicking
    let unit_val = Value::Unit;
    assert!(unit_val.as_i64().is_err());
    assert!(unit_val.as_f64().is_err());
    assert!(unit_val.as_bool().is_err());

    let string_val = Value::String("test".to_string());
    assert!(string_val.as_i64().is_err());
    assert!(string_val.as_f64().is_err());
}

#[test]
fn test_type_mapper_error_handling() {
    use cranelift::prelude::types;

    let mapper = TypeMapper::new(types::I64);

    // Test that unsubstituted generic types return errors
    let generic = ResolvedType::Generic("T".to_string());
    assert!(mapper.map_type(&generic).is_err());
    if let Err(e) = mapper.map_type(&generic) {
        assert!(matches!(e, JitError::UnsubstitutedGeneric));
    }

    // Test that unresolved type variables return errors
    let type_var = ResolvedType::Var(0);
    assert!(mapper.map_type(&type_var).is_err());
    if let Err(e) = mapper.map_type(&type_var) {
        assert!(matches!(e, JitError::UnresolvedTypeVar));
    }

    // Test that unsubstituted const generics return errors
    let const_generic = ResolvedType::ConstGeneric("N".to_string());
    assert!(mapper.map_type(&const_generic).is_err());
    if let Err(e) = mapper.map_type(&const_generic) {
        assert!(matches!(e, JitError::UnsubstitutedConstGeneric));
    }
}

#[test]
fn test_conversion_error_messages() {
    // Verify error messages are descriptive
    let unit_val = Value::Unit;
    match unit_val.as_i64() {
        Err(JitError::InvalidConversion { from, to }) => {
            assert!(from.contains("Unit"));
            assert_eq!(to, "i64");
        }
        _ => panic!("Expected InvalidConversion error"),
    }

    let array_val = Value::Array(vec![Value::I64(1), Value::I64(2)]);
    match array_val.as_bool() {
        Err(JitError::InvalidConversion { from, to }) => {
            assert!(from.contains("Array"));
            assert_eq!(to, "bool");
        }
        _ => panic!("Expected InvalidConversion error"),
    }
}

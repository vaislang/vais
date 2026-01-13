//! Vais Integration Tests
//!
//! End-to-end tests that verify the complete pipeline:
//! Source -> Lexer -> Parser -> Lowering -> VM Execution

use vais_ir::Value;
use vais_lowering::Lowerer;
use vais_vm::{execute, execute_function};

/// Helper function to run Vais source code and return the result
#[allow(dead_code)]
fn run_vais(source: &str) -> Result<Value, String> {
    let program = vais_parser::parse(source).map_err(|e| format!("Parse error: {:?}", e))?;
    let mut lowerer = Lowerer::new();
    let functions = lowerer
        .lower_program(&program)
        .map_err(|e| format!("Lowering error: {:?}", e))?;
    execute(functions).map_err(|e| format!("Runtime error: {:?}", e))
}

/// Helper to run a specific function with arguments
fn run_function(source: &str, func_name: &str, args: Vec<Value>) -> Result<Value, String> {
    let program = vais_parser::parse(source).map_err(|e| format!("Parse error: {:?}", e))?;
    let mut lowerer = Lowerer::new();
    let functions = lowerer
        .lower_program(&program)
        .map_err(|e| format!("Lowering error: {:?}", e))?;
    execute_function(functions, func_name, args).map_err(|e| format!("Runtime error: {:?}", e))
}

// ============================================================================
// Basic Function Tests
// ============================================================================

#[test]
fn test_simple_addition() {
    let source = "add(a, b) = a + b";
    let result = run_function(source, "add", vec![Value::Int(3), Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Int(8));
}

#[test]
fn test_simple_subtraction() {
    let source = "sub(a, b) = a - b";
    let result = run_function(source, "sub", vec![Value::Int(10), Value::Int(3)]);
    assert_eq!(result.unwrap(), Value::Int(7));
}

#[test]
fn test_simple_multiplication() {
    let source = "mul(a, b) = a * b";
    let result = run_function(source, "mul", vec![Value::Int(4), Value::Int(7)]);
    assert_eq!(result.unwrap(), Value::Int(28));
}

#[test]
fn test_simple_division() {
    let source = "div(a, b) = a / b";
    let result = run_function(source, "div", vec![Value::Int(20), Value::Int(4)]);
    assert_eq!(result.unwrap(), Value::Int(5));
}

#[test]
fn test_modulo() {
    let source = "mod_fn(a, b) = a % b";
    let result = run_function(source, "mod_fn", vec![Value::Int(17), Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Int(2));
}

// ============================================================================
// Ternary Operator Tests
// ============================================================================

#[test]
fn test_ternary_true_branch() {
    let source = "max(a, b) = a > b ? a : b";
    let result = run_function(source, "max", vec![Value::Int(10), Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Int(10));
}

#[test]
fn test_ternary_false_branch() {
    let source = "max(a, b) = a > b ? a : b";
    let result = run_function(source, "max", vec![Value::Int(3), Value::Int(8)]);
    assert_eq!(result.unwrap(), Value::Int(8));
}

#[test]
fn test_abs_function() {
    let source = "abs_fn(n) = n < 0 ? -n : n";

    let result = run_function(source, "abs_fn", vec![Value::Int(-5)]);
    assert_eq!(result.unwrap(), Value::Int(5));

    let result = run_function(source, "abs_fn", vec![Value::Int(7)]);
    assert_eq!(result.unwrap(), Value::Int(7));
}

// ============================================================================
// Recursion Tests (using $ operator)
// ============================================================================

#[test]
fn test_factorial() {
    let source = "fact(n) = n < 2 ? 1 : n * $(n - 1)";

    let result = run_function(source, "fact", vec![Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Int(120));

    let result = run_function(source, "fact", vec![Value::Int(0)]);
    assert_eq!(result.unwrap(), Value::Int(1));
}

#[test]
fn test_fibonacci() {
    let source = "fib(n) = n < 2 ? n : $(n - 1) + $(n - 2)";

    let result = run_function(source, "fib", vec![Value::Int(10)]);
    assert_eq!(result.unwrap(), Value::Int(55));

    let result = run_function(source, "fib", vec![Value::Int(0)]);
    assert_eq!(result.unwrap(), Value::Int(0));

    let result = run_function(source, "fib", vec![Value::Int(1)]);
    assert_eq!(result.unwrap(), Value::Int(1));
}

#[test]
fn test_sum_to_n() {
    let source = "sum_to(n) = n < 1 ? 0 : n + $(n - 1)";
    let result = run_function(source, "sum_to", vec![Value::Int(10)]);
    assert_eq!(result.unwrap(), Value::Int(55)); // 1+2+...+10 = 55
}

// ============================================================================
// Collection Operation Tests
// ============================================================================

#[test]
fn test_map_operation() {
    let source = "double(arr) = arr.@(_ * 2)";
    let input = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let result = run_function(source, "double", vec![input]);

    let expected = Value::Array(vec![Value::Int(2), Value::Int(4), Value::Int(6)]);
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn test_filter_operation() {
    let source = "evens(arr) = arr.?(_ % 2 == 0)";
    let input = Value::Array(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
        Value::Int(4),
        Value::Int(5),
    ]);
    let result = run_function(source, "evens", vec![input]);

    let expected = Value::Array(vec![Value::Int(2), Value::Int(4)]);
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn test_reduce_sum() {
    let source = "sum(arr) = arr./+";
    let input = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]);
    let result = run_function(source, "sum", vec![input]);
    assert_eq!(result.unwrap(), Value::Int(10));
}

#[test]
fn test_reduce_product() {
    let source = "product(arr) = arr./*";
    let input = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]);
    let result = run_function(source, "product", vec![input]);
    assert_eq!(result.unwrap(), Value::Int(24));
}

#[test]
fn test_chained_operations() {
    // Double all numbers then filter those > 5
    let source = "process(arr) = arr.@(_ * 2).?(_ > 5)";
    let input = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]);
    let result = run_function(source, "process", vec![input]);

    // [1,2,3,4] -> [2,4,6,8] -> [6,8]
    let expected = Value::Array(vec![Value::Int(6), Value::Int(8)]);
    assert_eq!(result.unwrap(), expected);
}

// ============================================================================
// Let Binding Tests
// ============================================================================

#[test]
fn test_let_binding_simple() {
    let source = "calc(x) = let y = x * 2 : y + 1";
    let result = run_function(source, "calc", vec![Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Int(11)); // 5*2 + 1 = 11
}

#[test]
fn test_let_binding_multiple() {
    let source = "calc(x) = let a = x + 1, b = x * 2 : a + b";
    let result = run_function(source, "calc", vec![Value::Int(3)]);
    assert_eq!(result.unwrap(), Value::Int(10)); // (3+1) + (3*2) = 4 + 6 = 10
}

// ============================================================================
// Array Literal and Indexing Tests
// ============================================================================

#[test]
fn test_array_literal() {
    let source = "get_arr() = [1, 2, 3, 4, 5]";
    let result = run_function(source, "get_arr", vec![]);

    let expected = Value::Array(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
        Value::Int(4),
        Value::Int(5),
    ]);
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn test_array_index() {
    let source = "get_element(arr, i) = arr[i]";
    let input = Value::Array(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);
    let result = run_function(source, "get_element", vec![input, Value::Int(1)]);
    assert_eq!(result.unwrap(), Value::Int(20));
}

#[test]
fn test_array_length() {
    let source = "len_fn(arr) = #arr";
    let input = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let result = run_function(source, "len_fn", vec![input]);
    assert_eq!(result.unwrap(), Value::Int(3));
}

// ============================================================================
// Comparison Tests
// ============================================================================

#[test]
fn test_equality() {
    let source = "eq(a, b) = a == b";

    let result = run_function(source, "eq", vec![Value::Int(5), Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Bool(true));

    let result = run_function(source, "eq", vec![Value::Int(5), Value::Int(3)]);
    assert_eq!(result.unwrap(), Value::Bool(false));
}

#[test]
fn test_less_than() {
    let source = "lt(a, b) = a < b";

    let result = run_function(source, "lt", vec![Value::Int(3), Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Bool(true));

    let result = run_function(source, "lt", vec![Value::Int(5), Value::Int(3)]);
    assert_eq!(result.unwrap(), Value::Bool(false));
}

#[test]
fn test_greater_than() {
    let source = "gt(a, b) = a > b";

    let result = run_function(source, "gt", vec![Value::Int(5), Value::Int(3)]);
    assert_eq!(result.unwrap(), Value::Bool(true));

    let result = run_function(source, "gt", vec![Value::Int(3), Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Bool(false));
}

// ============================================================================
// String Tests
// ============================================================================

#[test]
fn test_string_concatenation() {
    let source = r#"concat(a, b) = a + b"#;
    let result = run_function(
        source,
        "concat",
        vec![
            Value::String("Hello, ".to_string()),
            Value::String("World!".to_string()),
        ],
    );
    assert_eq!(
        result.unwrap(),
        Value::String("Hello, World!".to_string())
    );
}

// ============================================================================
// Float Tests
// ============================================================================

#[test]
fn test_float_arithmetic() {
    let source = "calc(a, b) = a + b";
    let result = run_function(source, "calc", vec![Value::Float(1.5), Value::Float(2.5)]);

    if let Value::Float(f) = result.unwrap() {
        assert!((f - 4.0).abs() < 0.0001);
    } else {
        panic!("Expected float result");
    }
}

// ============================================================================
// Multiple Function Tests
// ============================================================================

#[test]
fn test_multiple_functions() {
    let source = r#"
        double(x) = x * 2
        triple(x) = x * 3
        add_doubled_and_tripled(x) = double(x) + triple(x)
    "#;

    // double(5) + triple(5) = 10 + 15 = 25
    let result = run_function(source, "add_doubled_and_tripled", vec![Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Int(25));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_array_map() {
    let source = "double(arr) = arr.@(_ * 2)";
    let input = Value::Array(vec![]);
    let result = run_function(source, "double", vec![input]);
    assert_eq!(result.unwrap(), Value::Array(vec![]));
}

#[test]
fn test_empty_array_filter() {
    let source = "filter_positive(arr) = arr.?(_ > 0)";
    let input = Value::Array(vec![]);
    let result = run_function(source, "filter_positive", vec![input]);
    assert_eq!(result.unwrap(), Value::Array(vec![]));
}

#[test]
fn test_filter_none_match() {
    let source = "filter_negative(arr) = arr.?(_ < 0)";
    let input = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let result = run_function(source, "filter_negative", vec![input]);
    assert_eq!(result.unwrap(), Value::Array(vec![]));
}

#[test]
fn test_nested_ternary() {
    // sign(n) returns -1, 0, or 1
    let source = "sign(n) = n < 0 ? -1 : n > 0 ? 1 : 0";

    let result = run_function(source, "sign", vec![Value::Int(-5)]);
    assert_eq!(result.unwrap(), Value::Int(-1));

    let result = run_function(source, "sign", vec![Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Int(1));

    let result = run_function(source, "sign", vec![Value::Int(0)]);
    assert_eq!(result.unwrap(), Value::Int(0));
}

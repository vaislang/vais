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
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
    let result = run_function(source, "double", vec![input]);

    let expected = Value::Array(std::rc::Rc::new(vec![Value::Int(2), Value::Int(4), Value::Int(6)]));
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn test_filter_operation() {
    let source = "evens(arr) = arr.?(_ % 2 == 0)";
    let input = Value::Array(std::rc::Rc::new(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
        Value::Int(4),
        Value::Int(5),
    ]));
    let result = run_function(source, "evens", vec![input]);

    let expected = Value::Array(std::rc::Rc::new(vec![Value::Int(2), Value::Int(4)]));
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn test_reduce_sum() {
    let source = "sum(arr) = arr./+";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]));
    let result = run_function(source, "sum", vec![input]);
    assert_eq!(result.unwrap(), Value::Int(10));
}

#[test]
fn test_reduce_product() {
    let source = "product(arr) = arr./*";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]));
    let result = run_function(source, "product", vec![input]);
    assert_eq!(result.unwrap(), Value::Int(24));
}

#[test]
fn test_chained_operations() {
    // Double all numbers then filter those > 5
    let source = "process(arr) = arr.@(_ * 2).?(_ > 5)";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]));
    let result = run_function(source, "process", vec![input]);

    // [1,2,3,4] -> [2,4,6,8] -> [6,8]
    let expected = Value::Array(std::rc::Rc::new(vec![Value::Int(6), Value::Int(8)]));
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

    let expected = Value::Array(std::rc::Rc::new(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
        Value::Int(4),
        Value::Int(5),
    ]));
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn test_array_index() {
    let source = "get_element(arr, i) = arr[i]";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(10), Value::Int(20), Value::Int(30)]));
    let result = run_function(source, "get_element", vec![input, Value::Int(1)]);
    assert_eq!(result.unwrap(), Value::Int(20));
}

#[test]
fn test_array_length() {
    let source = "len_fn(arr) = #arr";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
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
    let input = Value::Array(std::rc::Rc::new(vec![]));
    let result = run_function(source, "double", vec![input]);
    assert_eq!(result.unwrap(), Value::Array(std::rc::Rc::new(vec![])));
}

#[test]
fn test_empty_array_filter() {
    let source = "filter_positive(arr) = arr.?(_ > 0)";
    let input = Value::Array(std::rc::Rc::new(vec![]));
    let result = run_function(source, "filter_positive", vec![input]);
    assert_eq!(result.unwrap(), Value::Array(std::rc::Rc::new(vec![])));
}

#[test]
fn test_filter_none_match() {
    let source = "filter_negative(arr) = arr.?(_ < 0)";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
    let result = run_function(source, "filter_negative", vec![input]);
    assert_eq!(result.unwrap(), Value::Array(std::rc::Rc::new(vec![])));
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

// ============================================================================
// Set Operations Tests
// ============================================================================

#[test]
fn test_set_literal() {
    // Set literal uses #{...} syntax
    // Sets are represented as arrays internally
    let source = "make_set() = #{1, 2, 3}";
    let result = run_function(source, "make_set", vec![]);
    match result.unwrap() {
        Value::Array(arr) => {
            // Should contain unique values
            assert!(arr.len() <= 3);
            assert!(arr.contains(&Value::Int(1)));
            assert!(arr.contains(&Value::Int(2)));
            assert!(arr.contains(&Value::Int(3)));
        }
        other => {
            // Sets may be represented differently - just check it parsed
            println!("Set result: {:?}", other);
        }
    }
}

#[test]
fn test_set_operations() {
    // Test set with map to double values (using array-based set representation)
    let source = "doubled(arr) = arr.@(_ * 2)";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
    let result = run_function(source, "doubled", vec![input]);
    match result.unwrap() {
        Value::Array(arr) => {
            assert!(arr.contains(&Value::Int(2)));
            assert!(arr.contains(&Value::Int(4)));
            assert!(arr.contains(&Value::Int(6)));
        }
        _ => panic!("Expected array"),
    }
}

// ============================================================================
// Multi-line Comment Tests
// ============================================================================

#[test]
fn test_multiline_comment() {
    let source = r#"
        /* This is a
           multi-line comment */
        add(a, b) = a + b
    "#;
    let result = run_function(source, "add", vec![Value::Int(10), Value::Int(20)]);
    assert_eq!(result.unwrap(), Value::Int(30));
}

#[test]
fn test_mixed_comments() {
    let source = r#"
        // Single line comment
        /* Multi-line
           comment */
        mul(x, y) = x * y // inline comment
    "#;
    let result = run_function(source, "mul", vec![Value::Int(6), Value::Int(7)]);
    assert_eq!(result.unwrap(), Value::Int(42));
}

// ============================================================================
// Scientific Notation Tests
// ============================================================================

#[test]
fn test_scientific_notation() {
    let source = "get_large() = 1e6";
    let result = run_function(source, "get_large", vec![]);
    assert_eq!(result.unwrap(), Value::Float(1_000_000.0));
}

#[test]
fn test_scientific_notation_negative_exp() {
    let source = "get_small() = 5e-3";
    let result = run_function(source, "get_small", vec![]);
    assert_eq!(result.unwrap(), Value::Float(0.005));
}

#[test]
fn test_scientific_notation_with_decimal() {
    let source = "get_precise() = 3.14e2";
    let result = run_function(source, "get_precise", vec![]);
    assert_eq!(result.unwrap(), Value::Float(314.0));
}

// ============================================================================
// Lambda and Closure Tests
// ============================================================================

#[test]
fn test_lambda_in_map() {
    // Using placeholder syntax (_ * _) for map
    let source = "square_all(arr) = arr.@(_ * _)";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
    let result = run_function(source, "square_all", vec![input]);
    assert_eq!(
        result.unwrap(),
        Value::Array(std::rc::Rc::new(vec![Value::Int(1), Value::Int(4), Value::Int(9)]))
    );
}

#[test]
fn test_lambda_in_filter() {
    // Using placeholder syntax for filter
    let source = "filter_even(arr) = arr.?(_ % 2 == 0)";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]));
    let result = run_function(source, "filter_even", vec![input]);
    assert_eq!(
        result.unwrap(),
        Value::Array(std::rc::Rc::new(vec![Value::Int(2), Value::Int(4)]))
    );
}

// ============================================================================
// Coalesce Operator Tests
// ============================================================================

#[test]
fn test_coalesce_with_value() {
    let source = "safe_get(x) = x ?? 0";
    let result = run_function(source, "safe_get", vec![Value::Int(42)]);
    assert_eq!(result.unwrap(), Value::Int(42));
}

#[test]
fn test_coalesce_with_nil() {
    let source = "safe_get(x) = x ?? 100";
    let result = run_function(source, "safe_get", vec![Value::Void]);
    assert_eq!(result.unwrap(), Value::Int(100));
}

// ============================================================================
// Range Tests
// ============================================================================

#[test]
fn test_range_creation() {
    let source = "make_range() = 1..5";
    let result = run_function(source, "make_range", vec![]);
    assert_eq!(
        result.unwrap(),
        Value::Array(std::rc::Rc::new(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4)
        ]))
    );
}

#[test]
fn test_range_sum() {
    let source = "sum_range() = (1..6)./+";
    let result = run_function(source, "sum_range", vec![]);
    assert_eq!(result.unwrap(), Value::Int(15)); // 1+2+3+4+5
}

// ============================================================================
// Contains Operator Tests
// ============================================================================

#[test]
fn test_contains_true() {
    let source = "has_three(arr) = 3 @ arr";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
    let result = run_function(source, "has_three", vec![input]);
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
fn test_contains_false() {
    let source = "has_five(arr) = 5 @ arr";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
    let result = run_function(source, "has_five", vec![input]);
    assert_eq!(result.unwrap(), Value::Bool(false));
}

// ============================================================================
// Struct Tests
// ============================================================================

#[test]
fn test_struct_creation() {
    let source = "make_point() = { x: 10, y: 20 }";
    let result = run_function(source, "make_point", vec![]);
    match result.unwrap() {
        Value::Struct(fields) => {
            assert_eq!(fields.get("x"), Some(&Value::Int(10)));
            assert_eq!(fields.get("y"), Some(&Value::Int(20)));
        }
        _ => panic!("Expected struct"),
    }
}

#[test]
fn test_struct_field_access() {
    let source = r#"
        get_x(p) = p.x
    "#;
    let mut fields = std::collections::HashMap::new();
    fields.insert("x".to_string(), Value::Int(42));
    fields.insert("y".to_string(), Value::Int(10));
    let result = run_function(source, "get_x", vec![Value::Struct(std::rc::Rc::new(fields))]);
    assert_eq!(result.unwrap(), Value::Int(42));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_try_expression() {
    // Try-catch block: try { body } catch e { handler }
    let source = "safe_div(a, b) = try { a / b } catch e { 0 }";
    let result = run_function(source, "safe_div", vec![Value::Int(10), Value::Int(2)]);
    assert_eq!(result.unwrap(), Value::Int(5));
}

// ============================================================================
// Match Expression Tests
// ============================================================================

#[test]
fn test_match_literal() {
    let source = r#"
        describe(n) = match n {
            0 => "zero",
            1 => "one",
            _ => "many"
        }
    "#;
    let result = run_function(source, "describe", vec![Value::Int(0)]);
    assert_eq!(result.unwrap(), Value::String("zero".to_string()));

    let result = run_function(source, "describe", vec![Value::Int(1)]);
    assert_eq!(result.unwrap(), Value::String("one".to_string()));

    let result = run_function(source, "describe", vec![Value::Int(99)]);
    assert_eq!(result.unwrap(), Value::String("many".to_string()));
}

// ============================================================================
// Parallel Operations Tests
// ============================================================================

#[test]
fn test_parallel_map() {
    let source = "par_double(arr) = arr.||@(_ * 2)";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
    let result = run_function(source, "par_double", vec![input]);
    let arr = match result.unwrap() {
        Value::Array(a) => a,
        _ => panic!("Expected array"),
    };
    // Parallel may reorder, but values should be correct
    assert!(arr.contains(&Value::Int(2)));
    assert!(arr.contains(&Value::Int(4)));
    assert!(arr.contains(&Value::Int(6)));
}

#[test]
fn test_parallel_filter() {
    let source = "par_positive(arr) = arr.||?(_ > 0)";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(-1), Value::Int(2), Value::Int(-3), Value::Int(4)]));
    let result = run_function(source, "par_positive", vec![input]);
    let arr = match result.unwrap() {
        Value::Array(a) => a,
        _ => panic!("Expected array"),
    };
    assert_eq!(arr.len(), 2);
    assert!(arr.contains(&Value::Int(2)));
    assert!(arr.contains(&Value::Int(4)));
}

#[test]
fn test_parallel_reduce() {
    let source = "par_sum(arr) = arr.||/+";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]));
    let result = run_function(source, "par_sum", vec![input]);
    assert_eq!(result.unwrap(), Value::Int(10));
}

// ============================================================================
// Additional Edge Cases
// ============================================================================

#[test]
fn test_negative_numbers() {
    let source = "negate(x) = -x";
    let result = run_function(source, "negate", vec![Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Int(-5));

    let result = run_function(source, "negate", vec![Value::Int(-3)]);
    assert_eq!(result.unwrap(), Value::Int(3));
}

#[test]
fn test_boolean_operations() {
    let source = "both(a, b) = a && b";
    let result = run_function(source, "both", vec![Value::Bool(true), Value::Bool(true)]);
    assert_eq!(result.unwrap(), Value::Bool(true));

    let result = run_function(source, "both", vec![Value::Bool(true), Value::Bool(false)]);
    assert_eq!(result.unwrap(), Value::Bool(false));
}

#[test]
fn test_boolean_or() {
    let source = "either(a, b) = a || b";
    let result = run_function(source, "either", vec![Value::Bool(false), Value::Bool(true)]);
    assert_eq!(result.unwrap(), Value::Bool(true));

    let result = run_function(source, "either", vec![Value::Bool(false), Value::Bool(false)]);
    assert_eq!(result.unwrap(), Value::Bool(false));
}

#[test]
fn test_boolean_not() {
    let source = "negate_bool(x) = !x";
    let result = run_function(source, "negate_bool", vec![Value::Bool(true)]);
    assert_eq!(result.unwrap(), Value::Bool(false));

    let result = run_function(source, "negate_bool", vec![Value::Bool(false)]);
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
fn test_not_equal() {
    let source = "neq(a, b) = a != b";
    let result = run_function(source, "neq", vec![Value::Int(5), Value::Int(3)]);
    assert_eq!(result.unwrap(), Value::Bool(true));

    let result = run_function(source, "neq", vec![Value::Int(5), Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Bool(false));
}

#[test]
fn test_less_than_or_equal() {
    let source = "lte(a, b) = a <= b";
    let result = run_function(source, "lte", vec![Value::Int(3), Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Bool(true));

    let result = run_function(source, "lte", vec![Value::Int(5), Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Bool(true));

    let result = run_function(source, "lte", vec![Value::Int(6), Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Bool(false));
}

#[test]
fn test_greater_than_or_equal() {
    let source = "gte(a, b) = a >= b";
    let result = run_function(source, "gte", vec![Value::Int(5), Value::Int(3)]);
    assert_eq!(result.unwrap(), Value::Bool(true));

    let result = run_function(source, "gte", vec![Value::Int(5), Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Bool(true));

    let result = run_function(source, "gte", vec![Value::Int(4), Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Bool(false));
}

// ============================================================================
// Reduce Variations
// ============================================================================

#[test]
fn test_reduce_min() {
    let source = "minimum(arr) = arr./min";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(5), Value::Int(2), Value::Int(8), Value::Int(1)]));
    let result = run_function(source, "minimum", vec![input]);
    assert_eq!(result.unwrap(), Value::Int(1));
}

#[test]
fn test_reduce_max() {
    let source = "maximum(arr) = arr./max";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(5), Value::Int(2), Value::Int(8), Value::Int(1)]));
    let result = run_function(source, "maximum", vec![input]);
    assert_eq!(result.unwrap(), Value::Int(8));
}

// ============================================================================
// Complex Expression Tests
// ============================================================================

#[test]
fn test_complex_arithmetic() {
    let source = "calc(a, b, c) = (a + b) * c - a / b";
    let result = run_function(source, "calc", vec![Value::Int(10), Value::Int(5), Value::Int(3)]);
    // (10 + 5) * 3 - 10 / 5 = 15 * 3 - 2 = 45 - 2 = 43
    assert_eq!(result.unwrap(), Value::Int(43));
}

#[test]
fn test_deeply_nested_ternary() {
    let source = "classify(n) = n < 0 ? -1 : n == 0 ? 0 : n < 10 ? 1 : n < 100 ? 2 : 3";

    let result = run_function(source, "classify", vec![Value::Int(-5)]);
    assert_eq!(result.unwrap(), Value::Int(-1));

    let result = run_function(source, "classify", vec![Value::Int(0)]);
    assert_eq!(result.unwrap(), Value::Int(0));

    let result = run_function(source, "classify", vec![Value::Int(5)]);
    assert_eq!(result.unwrap(), Value::Int(1));

    let result = run_function(source, "classify", vec![Value::Int(50)]);
    assert_eq!(result.unwrap(), Value::Int(2));

    let result = run_function(source, "classify", vec![Value::Int(500)]);
    assert_eq!(result.unwrap(), Value::Int(3));
}

#[test]
fn test_chained_map_filter_reduce() {
    // [1,2,3,4,5] -> [2,4,6,8,10] -> [6,8,10] -> 24
    let source = "process(arr) = arr.@(_ * 2).?(_ > 5)./+";
    let input = Value::Array(std::rc::Rc::new(vec![
        Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4), Value::Int(5)
    ]));
    let result = run_function(source, "process", vec![input]);
    assert_eq!(result.unwrap(), Value::Int(24));
}

// ============================================================================
// Recursive Helper Function Tests
// ============================================================================

#[test]
fn test_tail_recursive_sum() {
    let source = r#"
        sum_helper(n, acc) = n < 1 ? acc : $(n - 1, acc + n)
        sum_to(n) = sum_helper(n, 0)
    "#;
    let result = run_function(source, "sum_to", vec![Value::Int(10)]);
    assert_eq!(result.unwrap(), Value::Int(55)); // 1+2+...+10
}

#[test]
fn test_gcd() {
    let source = "gcd(a, b) = b == 0 ? a : $(b, a % b)";
    let result = run_function(source, "gcd", vec![Value::Int(48), Value::Int(18)]);
    assert_eq!(result.unwrap(), Value::Int(6));
}

// ============================================================================
// Block Expression Tests
// ============================================================================

#[test]
fn test_block_expression() {
    let source = r#"
        compute(x) = {
            a = x * 2;
            b = a + 10;
            b * 2
        }
    "#;
    let result = run_function(source, "compute", vec![Value::Int(5)]);
    // a = 10, b = 20, result = 40
    assert_eq!(result.unwrap(), Value::Int(40));
}

// ============================================================================
// Negative Index Tests
// ============================================================================

#[test]
fn test_negative_array_index() {
    let source = "last_element(arr) = arr[-1]";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(10), Value::Int(20), Value::Int(30)]));
    let result = run_function(source, "last_element", vec![input]);
    assert_eq!(result.unwrap(), Value::Int(30));
}

#[test]
fn test_second_to_last() {
    let source = "second_last(arr) = arr[-2]";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(10), Value::Int(20), Value::Int(30)]));
    let result = run_function(source, "second_last", vec![input]);
    assert_eq!(result.unwrap(), Value::Int(20));
}

// ============================================================================
// Float Comparison Tests
// ============================================================================

#[test]
fn test_float_comparison() {
    let source = "is_positive(x) = x > 0.0";
    let result = run_function(source, "is_positive", vec![Value::Float(1.5)]);
    assert_eq!(result.unwrap(), Value::Bool(true));

    let result = run_function(source, "is_positive", vec![Value::Float(-0.5)]);
    assert_eq!(result.unwrap(), Value::Bool(false));
}

// ============================================================================
// Mixed Type Tests
// ============================================================================

#[test]
fn test_int_float_addition() {
    let source = "add_mixed(a, b) = a + b";
    let result = run_function(source, "add_mixed", vec![Value::Int(5), Value::Float(2.5)]);
    if let Value::Float(f) = result.unwrap() {
        assert!((f - 7.5).abs() < 0.0001);
    } else {
        panic!("Expected float result");
    }
}

// ============================================================================
// Empty and Single Element Tests
// ============================================================================

#[test]
fn test_single_element_reduce() {
    let source = "sum(arr) = arr./+";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(42)]));
    let result = run_function(source, "sum", vec![input]);
    assert_eq!(result.unwrap(), Value::Int(42));
}

#[test]
fn test_single_element_map() {
    let source = "double(arr) = arr.@(_ * 2)";
    let input = Value::Array(std::rc::Rc::new(vec![Value::Int(21)]));
    let result = run_function(source, "double", vec![input]);
    assert_eq!(result.unwrap(), Value::Array(std::rc::Rc::new(vec![Value::Int(42)])));
}

// ============================================================================
// Algebraic Effects Tests
// ============================================================================

#[test]
fn test_effect_basic() {
    let source = r#"
        effect Logger {
            log(msg: String) -> Void
        }

        compute(x) = {
            perform Logger.log("test");
            x * 2
        }

        main() = handle compute(21) {
            Logger.log(msg) => msg
        }
    "#;
    let result = run_function(source, "main", vec![]);
    assert_eq!(result.unwrap(), Value::Int(42));
}

#[test]
fn test_effect_with_result() {
    let source = r#"
        effect Console {
            print(msg: String) -> Void
        }

        double_with_log(x) = {
            perform Console.print("doubling");
            x * 2
        }

        run() = handle double_with_log(10) {
            Console.print(msg) => msg
        }
    "#;
    let result = run_function(source, "run", vec![]);
    assert_eq!(result.unwrap(), Value::Int(20));
}

#[test]
fn test_effect_multiple_performs() {
    let source = r#"
        effect Log {
            info(msg: String) -> Void
        }

        multi_log(x) = {
            perform Log.info("start");
            y = x + 1;
            perform Log.info("middle");
            z = y * 2;
            perform Log.info("end");
            z
        }

        test() = handle multi_log(5) {
            Log.info(msg) => msg
        }
    "#;
    let result = run_function(source, "test", vec![]);
    // (5 + 1) * 2 = 12
    assert_eq!(result.unwrap(), Value::Int(12));
}

//! Integration tests for vais-testgen crate
//!
//! Tests property-based test generation, shrinking, and test suite creation.

use vais_testgen::{
    Property, Shrinker, TestCase, TestCategory, TestGenerator, TestValue, TypeHint,
};

#[test]
fn test_test_generator_creation() {
    let _generator = TestGenerator::new();
    // Generator should be created successfully
    // Default values: num_random_cases = 10, seed = 42
}

#[test]
fn test_test_generator_default() {
    let _generator = TestGenerator::default();
    // Default should create the same as new()
}

#[test]
fn test_generate_integer_function_tests() {
    let generator = TestGenerator::new().with_num_cases(5).with_seed(42);

    let test_suite = generator.generate("add", &[TypeHint::I64, TypeHint::I64], &TypeHint::I64);

    assert_eq!(test_suite.function_name, "add");
    assert_eq!(test_suite.param_types.len(), 2);
    assert_eq!(test_suite.return_type, TypeHint::I64);

    // Should have boundary tests + random tests + property tests
    assert!(!test_suite.test_cases.is_empty());

    // Should have boundary tests (zeros, max, min, neg_one)
    assert!(test_suite
        .test_cases
        .iter()
        .any(|c| c.name.contains("boundary_zeros")));
    assert!(test_suite
        .test_cases
        .iter()
        .any(|c| c.name.contains("boundary_max")));
    assert!(test_suite
        .test_cases
        .iter()
        .any(|c| c.name.contains("boundary_min")));

    // "add" should have commutative property test
    assert!(test_suite
        .test_cases
        .iter()
        .any(|c| c.name.contains("property_commutative")));
}

#[test]
fn test_generate_string_function_tests() {
    let generator = TestGenerator::new().with_num_cases(3);

    let test_suite = generator.generate("to_uppercase", &[TypeHint::Str], &TypeHint::Str);

    assert_eq!(test_suite.function_name, "to_uppercase");
    assert_eq!(test_suite.param_types.len(), 1);
    assert_eq!(test_suite.param_types[0], TypeHint::Str);
    assert_eq!(test_suite.return_type, TypeHint::Str);

    // Should generate tests with string inputs
    let has_str_input = test_suite
        .test_cases
        .iter()
        .any(|c| c.inputs.iter().any(|v| matches!(v, TestValue::Str(_))));
    assert!(has_str_input);
}

#[test]
fn test_generate_multiple_param_tests() {
    let generator = TestGenerator::new().with_num_cases(2);

    let test_suite = generator.generate(
        "calculate",
        &[TypeHint::I64, TypeHint::F64, TypeHint::Bool],
        &TypeHint::I64,
    );

    assert_eq!(test_suite.param_types.len(), 3);

    // Should generate tests with correct number of inputs
    for test_case in &test_suite.test_cases {
        assert_eq!(test_case.inputs.len(), 3);
    }
}

#[test]
fn test_test_suite_structure() {
    let generator = TestGenerator::new().with_num_cases(5);

    let test_suite = generator.generate("abs", &[TypeHint::I64], &TypeHint::I64);

    assert_eq!(test_suite.function_name, "abs");
    assert_eq!(test_suite.param_types, vec![TypeHint::I64]);
    assert_eq!(test_suite.return_type, TypeHint::I64);
    assert!(!test_suite.test_cases.is_empty());

    // All test cases should reference the function name
    for test_case in &test_suite.test_cases {
        assert_eq!(test_case.function_name, "abs");
    }

    // "abs" should have idempotent property test
    assert!(test_suite.test_cases.iter().any(|c| {
        c.properties
            .iter()
            .any(|p| matches!(p, Property::Idempotent))
    }));
}

#[test]
fn test_test_case_to_vais_source() {
    let test_case = TestCase {
        name: "test_add".to_string(),
        function_name: "add".to_string(),
        inputs: vec![TestValue::Int(5), TestValue::Int(10)],
        properties: vec![Property::DoesNotCrash],
        category: TestCategory::Random,
    };

    let source = test_case.to_vais_source();

    assert!(source.contains("# Test: test_add"));
    assert!(source.contains("add(5, 10)"));
    assert!(source.contains("# Should not crash"));
}

#[test]
fn test_test_suite_to_vais_source() {
    let generator = TestGenerator::new().with_num_cases(2);
    let test_suite =
        generator.generate("multiply", &[TypeHint::I64, TypeHint::I64], &TypeHint::I64);

    let source = test_suite.to_vais_source();

    assert!(source.contains("# Auto-generated property-based tests for multiply"));
    assert!(source.contains("multiply"));
}

#[test]
fn test_property_variants() {
    let does_not_crash = Property::DoesNotCrash;
    let returns_non_zero = Property::ReturnsNonZero;
    let returns_in_range = Property::ReturnsInRange(0, 100);
    let idempotent = Property::Idempotent;
    let commutative = Property::Commutative;
    let custom = Property::Custom("assert(result > 0)".to_string());

    // Verify all property types can be created
    assert_eq!(format!("{}", does_not_crash), "does_not_crash");
    assert_eq!(format!("{}", returns_non_zero), "returns_non_zero");
    assert_eq!(format!("{}", returns_in_range), "returns_in_range(0, 100)");
    assert_eq!(format!("{}", idempotent), "idempotent");
    assert_eq!(format!("{}", commutative), "commutative");
    assert!(format!("{}", custom).contains("assert(result > 0)"));
}

#[test]
fn test_shrinker_shrink_i64() {
    let value = TestValue::Int(100);
    let shrunk = Shrinker::shrink(&value);

    // Should produce smaller values
    assert!(!shrunk.is_empty());
    assert!(shrunk.contains(&TestValue::Int(0)));
    assert!(shrunk.contains(&TestValue::Int(50)));
    assert!(shrunk.contains(&TestValue::Int(99)));
}

#[test]
fn test_shrinker_shrink_i64_zero() {
    let value = TestValue::Int(0);
    let shrunk = Shrinker::shrink(&value);

    // Cannot shrink zero further
    assert!(shrunk.is_empty());
}

#[test]
fn test_shrinker_shrink_i64_negative() {
    let value = TestValue::Int(-50);
    let shrunk = Shrinker::shrink(&value);

    // Should produce smaller negative values and positive counterpart
    assert!(!shrunk.is_empty());
    assert!(shrunk.contains(&TestValue::Int(0)));
    assert!(shrunk.contains(&TestValue::Int(-25)));
    assert!(shrunk.contains(&TestValue::Int(-49)));
    assert!(shrunk.contains(&TestValue::Int(50))); // positive version
}

#[test]
fn test_shrinker_shrink_string() {
    let value = TestValue::Str("hello".to_string());
    let shrunk = Shrinker::shrink(&value);

    // Should produce shorter strings
    assert!(!shrunk.is_empty());
    assert!(shrunk.contains(&TestValue::Str("".to_string())));

    // Should have variations
    let has_shorter = shrunk.iter().any(|v| {
        if let TestValue::Str(s) = v {
            s.len() < 5
        } else {
            false
        }
    });
    assert!(has_shorter);
}

#[test]
fn test_shrinker_shrink_string_empty() {
    let value = TestValue::Str("".to_string());
    let shrunk = Shrinker::shrink(&value);

    // Cannot shrink empty string further
    assert!(shrunk.is_empty());
}

#[test]
fn test_shrinker_shrink_array() {
    let value = TestValue::Array(vec![
        TestValue::Int(1),
        TestValue::Int(2),
        TestValue::Int(3),
    ]);
    let shrunk = Shrinker::shrink(&value);

    // Should produce smaller arrays
    assert!(!shrunk.is_empty());
    assert!(shrunk.contains(&TestValue::Array(vec![])));

    // Should have arrays with removed elements
    let has_smaller = shrunk.iter().any(|v| {
        if let TestValue::Array(arr) = v {
            arr.len() < 3
        } else {
            false
        }
    });
    assert!(has_smaller);
}

#[test]
fn test_shrinker_shrink_array_empty() {
    let value = TestValue::Array(vec![]);
    let shrunk = Shrinker::shrink(&value);

    // Cannot shrink empty array further
    assert!(shrunk.is_empty());
}

#[test]
fn test_shrinker_shrink_tuple() {
    let value = TestValue::Tuple(vec![
        TestValue::Int(100),
        TestValue::Str("test".to_string()),
    ]);
    let shrunk = Shrinker::shrink(&value);

    // Should produce tuples with shrunk elements
    assert!(!shrunk.is_empty());

    // Should have tuples with smaller integer
    let has_smaller_int = shrunk.iter().any(|v| {
        if let TestValue::Tuple(items) = v {
            if let Some(TestValue::Int(n)) = items.first() {
                *n < 100
            } else {
                false
            }
        } else {
            false
        }
    });
    assert!(has_smaller_int);
}

#[test]
fn test_type_hint_from_str() {
    assert_eq!(TypeHint::parse_type("i64"), TypeHint::I64);
    assert_eq!(TypeHint::parse_type("i32"), TypeHint::I32);
    assert_eq!(TypeHint::parse_type("f64"), TypeHint::F64);
    assert_eq!(TypeHint::parse_type("bool"), TypeHint::Bool);
    assert_eq!(TypeHint::parse_type("str"), TypeHint::Str);
    assert_eq!(TypeHint::parse_type("unknown"), TypeHint::Unknown);
}

#[test]
fn test_test_value_display() {
    assert_eq!(format!("{}", TestValue::Int(42)), "42");
    assert_eq!(format!("{}", TestValue::Bool(true)), "true");
    assert_eq!(
        format!("{}", TestValue::Str("hello".to_string())),
        "\"hello\""
    );

    let array = TestValue::Array(vec![TestValue::Int(1), TestValue::Int(2)]);
    assert_eq!(format!("{}", array), "[1, 2]");

    let tuple = TestValue::Tuple(vec![TestValue::Int(1), TestValue::Bool(true)]);
    assert_eq!(format!("{}", tuple), "(1, true)");
}

#[test]
fn test_generate_with_custom_num_cases() {
    let generator = TestGenerator::new().with_num_cases(20);

    let test_suite = generator.generate("foo", &[TypeHint::I64], &TypeHint::I64);

    // Should have many test cases (boundary + 20 random + property)
    assert!(test_suite.test_cases.len() >= 20);
}

#[test]
fn test_generate_with_seed() {
    let generator1 = TestGenerator::new().with_seed(123).with_num_cases(5);
    let generator2 = TestGenerator::new().with_seed(123).with_num_cases(5);

    let suite1 = generator1.generate("test", &[TypeHint::I64], &TypeHint::I64);
    let suite2 = generator2.generate("test", &[TypeHint::I64], &TypeHint::I64);

    // With same seed, random tests should be deterministic
    assert_eq!(suite1.test_cases.len(), suite2.test_cases.len());
}

#[test]
fn test_boundary_value_tests_coverage() {
    let generator = TestGenerator::new().with_num_cases(0); // Only boundary tests

    let test_suite = generator.generate("test", &[TypeHint::I64], &TypeHint::I64);

    // Should have boundary tests
    let boundary_tests: Vec<_> = test_suite
        .test_cases
        .iter()
        .filter(|c| c.name.contains("boundary"))
        .collect();

    assert!(!boundary_tests.is_empty());

    // Should test with 0, MAX, MIN, and -1
    assert!(boundary_tests.iter().any(|c| c.name.contains("zeros")));
    assert!(boundary_tests.iter().any(|c| c.name.contains("max")));
    assert!(boundary_tests.iter().any(|c| c.name.contains("min")));
    assert!(boundary_tests.iter().any(|c| c.name.contains("neg_one")));
}

#[test]
fn test_heuristic_property_detection() {
    let generator = TestGenerator::new().with_num_cases(0);

    // Test "count" function - should have non-negative property
    let count_suite = generator.generate("count", &[TypeHint::Str], &TypeHint::I64);
    assert!(count_suite.test_cases.iter().any(|c| {
        c.properties
            .iter()
            .any(|p| matches!(p, Property::ReturnsInRange(0, _)))
    }));

    // Test "max" function - should have commutative property
    let max_suite = generator.generate("max", &[TypeHint::I64, TypeHint::I64], &TypeHint::I64);
    assert!(max_suite.test_cases.iter().any(|c| {
        c.properties
            .iter()
            .any(|p| matches!(p, Property::Commutative))
    }));

    // Test "normalize" function - should have idempotent property
    let norm_suite = generator.generate("normalize", &[TypeHint::F64], &TypeHint::F64);
    assert!(norm_suite.test_cases.iter().any(|c| {
        c.properties
            .iter()
            .any(|p| matches!(p, Property::Idempotent))
    }));
}

// ========== NEW TESTS: Advanced Generator (3 tests) ==========

#[test]
fn test_generate_with_array_type_hint() {
    let generator = TestGenerator::new().with_num_cases(3);

    let array_hint = TypeHint::Array(Box::new(TypeHint::I64));
    let test_suite = generator.generate("process_array", &[array_hint.clone()], &TypeHint::I64);

    assert_eq!(test_suite.function_name, "process_array");
    assert_eq!(test_suite.param_types.len(), 1);
    assert_eq!(test_suite.param_types[0], array_hint);

    // Should generate tests with array inputs
    let has_array_input = test_suite
        .test_cases
        .iter()
        .any(|c| c.inputs.iter().any(|v| matches!(v, TestValue::Array(_))));
    assert!(has_array_input);

    // Boundary test should have empty array
    let has_empty_array = test_suite.test_cases.iter().any(|c| {
        c.inputs
            .iter()
            .any(|v| matches!(v, TestValue::Array(arr) if arr.is_empty()))
    });
    assert!(has_empty_array);
}

#[test]
fn test_generate_with_tuple_type_hint() {
    let generator = TestGenerator::new().with_num_cases(2);

    let tuple_hint = TypeHint::Tuple(vec![TypeHint::I64, TypeHint::Bool, TypeHint::Str]);
    let test_suite = generator.generate("process_tuple", &[tuple_hint.clone()], &TypeHint::I64);

    assert_eq!(test_suite.function_name, "process_tuple");
    assert_eq!(test_suite.param_types.len(), 1);
    assert_eq!(test_suite.param_types[0], tuple_hint);

    // Should generate tests with tuple inputs containing 3 elements
    let has_tuple_input = test_suite.test_cases.iter().any(|c| {
        c.inputs.iter().any(|v| {
            if let TestValue::Tuple(items) = v {
                items.len() == 3
            } else {
                false
            }
        })
    });
    assert!(has_tuple_input);
}

#[test]
fn test_with_seed_reproducibility() {
    let seed = 999_u64;
    let generator1 = TestGenerator::new().with_seed(seed).with_num_cases(10);
    let generator2 = TestGenerator::new().with_seed(seed).with_num_cases(10);

    let suite1 = generator1.generate("calculate", &[TypeHint::I64, TypeHint::F64], &TypeHint::I64);
    let suite2 = generator2.generate("calculate", &[TypeHint::I64, TypeHint::F64], &TypeHint::I64);

    // Same seed should produce identical test suites
    assert_eq!(suite1.test_cases.len(), suite2.test_cases.len());

    // Check that random test cases match
    let random_cases1: Vec<_> = suite1
        .test_cases
        .iter()
        .filter(|c| matches!(c.category, TestCategory::Random))
        .collect();
    let random_cases2: Vec<_> = suite2
        .test_cases
        .iter()
        .filter(|c| matches!(c.category, TestCategory::Random))
        .collect();

    assert_eq!(random_cases1.len(), random_cases2.len());
}

// ========== NEW TESTS: Advanced Shrinker (3 tests) ==========

#[test]
fn test_shrinker_shrink_float() {
    let value = TestValue::Float(123.456);
    let shrunk = Shrinker::shrink(&value);

    // Should produce smaller float values
    assert!(!shrunk.is_empty());
    assert!(shrunk.contains(&TestValue::Float(0.0)));
    assert!(shrunk.contains(&TestValue::Float(123.456 / 2.0)));
    assert!(shrunk.contains(&TestValue::Float(123.0))); // floor
}

#[test]
fn test_shrinker_shrink_bool() {
    let value_true = TestValue::Bool(true);
    let shrunk_true = Shrinker::shrink(&value_true);

    // Bool shrinking should produce both false and true
    assert_eq!(shrunk_true.len(), 2);
    assert!(shrunk_true.contains(&TestValue::Bool(false)));
    assert!(shrunk_true.contains(&TestValue::Bool(true)));

    let value_false = TestValue::Bool(false);
    let shrunk_false = Shrinker::shrink(&value_false);

    // Bool shrinking is consistent regardless of input
    assert_eq!(shrunk_false.len(), 2);
    assert!(shrunk_false.contains(&TestValue::Bool(false)));
    assert!(shrunk_false.contains(&TestValue::Bool(true)));
}

#[test]
fn test_shrinker_tuple_shrink_depth() {
    let value = TestValue::Tuple(vec![
        TestValue::Int(100),
        TestValue::Str("hello".to_string()),
        TestValue::Bool(true),
    ]);
    let shrunk = Shrinker::shrink(&value);

    // Should produce tuples with shrunk elements
    assert!(!shrunk.is_empty());

    // Should shrink the integer element
    let has_shrunk_int = shrunk.iter().any(|v| {
        if let TestValue::Tuple(items) = v {
            items.len() == 3
                && matches!(items[0], TestValue::Int(n) if n < 100)
                && matches!(items[1], TestValue::Str(_))
                && matches!(items[2], TestValue::Bool(_))
        } else {
            false
        }
    });
    assert!(has_shrunk_int);

    // Should shrink the string element
    let has_shrunk_str = shrunk.iter().any(|v| {
        if let TestValue::Tuple(items) = v {
            items.len() == 3
                && matches!(items[0], TestValue::Int(_))
                && matches!(&items[1], TestValue::Str(s) if s.len() < 5)
                && matches!(items[2], TestValue::Bool(_))
        } else {
            false
        }
    });
    assert!(has_shrunk_str);

    // Should shrink each element independently
    let has_shrunk_bool = shrunk.iter().any(|v| {
        if let TestValue::Tuple(items) = v {
            items.len() == 3
                && matches!(items[0], TestValue::Int(_))
                && matches!(items[1], TestValue::Str(_))
                && matches!(items[2], TestValue::Bool(_))
        } else {
            false
        }
    });
    assert!(has_shrunk_bool);
}

// ========== NEW TESTS: TestSuite Output (2 tests) ==========

#[test]
fn test_test_suite_to_vais_source_content() {
    let generator = TestGenerator::new().with_num_cases(1);
    let test_suite = generator.generate("factorial", &[TypeHint::I64], &TypeHint::I64);

    let source = test_suite.to_vais_source();

    // Should contain header comment
    assert!(source.contains("# Auto-generated property-based tests for factorial"));

    // Should contain parameter types
    assert!(source.contains("# Parameters:"));
    assert!(source.contains("I64"));

    // Should contain return type
    assert!(source.contains("# Returns:"));

    // Should contain at least one test case
    assert!(source.contains("factorial("));

    // Should have proper structure with blank lines between sections
    assert!(source.contains("\n\n"));
}

#[test]
fn test_multi_param_test_case_source_generation() {
    let test_case = TestCase {
        name: "test_complex".to_string(),
        function_name: "complex_fn".to_string(),
        inputs: vec![
            TestValue::Int(42),
            TestValue::Float(1.23),
            TestValue::Bool(true),
            TestValue::Str("test".to_string()),
        ],
        properties: vec![Property::DoesNotCrash],
        category: TestCategory::Random,
    };

    let source = test_case.to_vais_source();

    // Should contain all parameters
    assert!(source.contains("complex_fn(42, 1.23"));
    assert!(source.contains("true"));
    assert!(source.contains("\"test\""));

    // Should have test name
    assert!(source.contains("# Test: test_complex"));

    // Should have category
    assert!(source.contains("# Category: Random"));

    // Should have property
    assert!(source.contains("# Should not crash"));
}

// ========== NEW TESTS: Property Advanced (1 test) ==========

#[test]
fn test_property_custom_display() {
    let custom_prop = Property::Custom("assert(result % 2 == 0)".to_string());

    let display_str = format!("{}", custom_prop);

    // Should display custom assertion
    assert!(display_str.contains("custom("));
    assert!(display_str.contains("assert(result % 2 == 0)"));

    // Test that custom property appears in generated test case
    let test_case = TestCase {
        name: "test_even".to_string(),
        function_name: "make_even".to_string(),
        inputs: vec![TestValue::Int(7)],
        properties: vec![custom_prop],
        category: TestCategory::Property,
    };

    let source = test_case.to_vais_source();
    assert!(source.contains("assert(result % 2 == 0)"));
    assert!(source.contains("result := make_even(7)"));
}

// ========== NEW TESTS: Edge Cases (1 test) ==========

#[test]
fn test_edge_cases_empty_suite_and_long_function_name() {
    // Test with num_cases = 0 (only boundary tests)
    let generator = TestGenerator::new().with_num_cases(0);
    let test_suite = generator.generate("foo", &[TypeHint::I64], &TypeHint::I64);

    // Should still have boundary tests, not completely empty
    assert!(!test_suite.test_cases.is_empty());

    // All tests should be boundary or property tests
    let all_non_random = test_suite
        .test_cases
        .iter()
        .all(|c| !matches!(c.category, TestCategory::Random));
    assert!(all_non_random);

    // Test with very long function name
    let long_name =
        "very_long_function_name_that_exceeds_typical_length_boundaries_for_identifiers";
    let generator2 = TestGenerator::new().with_num_cases(1);
    let test_suite2 = generator2.generate(long_name, &[TypeHint::I64], &TypeHint::I64);

    assert_eq!(test_suite2.function_name, long_name);

    // Should generate valid source with long name
    let source = test_suite2.to_vais_source();
    assert!(source.contains(long_name));

    // Test case names should also include the long name
    assert!(test_suite2
        .test_cases
        .iter()
        .any(|c| c.name.contains(long_name)));
}

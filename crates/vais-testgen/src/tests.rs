//! Tests for the test generation system.

use super::*;
use generator::*;
use properties::Property;
use shrink::Shrinker;

#[test]
fn test_generate_basic_suite() {
    let gen = TestGenerator::new().with_num_cases(5);
    let suite = gen.generate("add", &[TypeHint::I64, TypeHint::I64], &TypeHint::I64);

    assert_eq!(suite.function_name, "add");
    assert!(suite.test_cases.len() >= 5); // random + boundary + property
}

#[test]
fn test_boundary_tests_generated() {
    let gen = TestGenerator::new().with_num_cases(0);
    let suite = gen.generate("f", &[TypeHint::I64], &TypeHint::I64);

    let boundary_count = suite
        .test_cases
        .iter()
        .filter(|c| c.category == TestCategory::Boundary)
        .count();
    assert!(boundary_count >= 3); // zeros, max, min
}

#[test]
fn test_commutative_property_detected() {
    let gen = TestGenerator::new().with_num_cases(0);
    let suite = gen.generate("add", &[TypeHint::I64, TypeHint::I64], &TypeHint::I64);

    let has_commutative = suite.test_cases.iter().any(|c| {
        c.properties
            .iter()
            .any(|p| matches!(p, Property::Commutative))
    });
    assert!(has_commutative, "add should generate commutative test");
}

#[test]
fn test_idempotent_property_detected() {
    let gen = TestGenerator::new().with_num_cases(0);
    let suite = gen.generate("abs", &[TypeHint::I64], &TypeHint::I64);

    let has_idempotent = suite.test_cases.iter().any(|c| {
        c.properties
            .iter()
            .any(|p| matches!(p, Property::Idempotent))
    });
    assert!(has_idempotent, "abs should generate idempotent test");
}

#[test]
fn test_non_negative_property_for_len() {
    let gen = TestGenerator::new().with_num_cases(0);
    let suite = gen.generate("len", &[TypeHint::Str], &TypeHint::I64);

    let has_range = suite.test_cases.iter().any(|c| {
        c.properties
            .iter()
            .any(|p| matches!(p, Property::ReturnsInRange(0, _)))
    });
    assert!(has_range, "len should generate non-negative range test");
}

#[test]
fn test_random_values_within_range() {
    let gen = TestGenerator::new().with_num_cases(20);
    let suite = gen.generate("f", &[TypeHint::I8], &TypeHint::I64);

    for case in &suite.test_cases {
        if case.category == TestCategory::Random {
            for input in &case.inputs {
                if let TestValue::Int(v) = input {
                    assert!(*v >= -128 && *v <= 127, "i8 value {} out of range", v);
                }
            }
        }
    }
}

#[test]
fn test_to_vais_source() {
    let case = TestCase {
        name: "test_add".to_string(),
        function_name: "add".to_string(),
        inputs: vec![TestValue::Int(1), TestValue::Int(2)],
        properties: vec![Property::DoesNotCrash],
        category: TestCategory::Random,
    };

    let source = case.to_vais_source();
    assert!(source.contains("add(1, 2)"));
    assert!(source.contains("# Test: test_add"));
}

#[test]
fn test_suite_to_vais_source() {
    let gen = TestGenerator::new().with_num_cases(3);
    let suite = gen.generate("mul", &[TypeHint::I64, TypeHint::I64], &TypeHint::I64);
    let source = suite.to_vais_source();

    assert!(source.contains("Auto-generated"));
    assert!(source.contains("mul"));
}

#[test]
fn test_shrink_int() {
    let shrunk = Shrinker::shrink(&TestValue::Int(100));
    assert!(shrunk.contains(&TestValue::Int(0)));
    assert!(shrunk.contains(&TestValue::Int(50)));
    assert!(shrunk.contains(&TestValue::Int(99)));
}

#[test]
fn test_shrink_int_zero() {
    let shrunk = Shrinker::shrink(&TestValue::Int(0));
    assert!(shrunk.is_empty());
}

#[test]
fn test_shrink_negative_int() {
    let shrunk = Shrinker::shrink(&TestValue::Int(-10));
    assert!(shrunk.contains(&TestValue::Int(0)));
    assert!(shrunk.contains(&TestValue::Int(10))); // positive version
}

#[test]
fn test_shrink_string() {
    let shrunk = Shrinker::shrink(&TestValue::Str("hello".to_string()));
    assert!(shrunk.contains(&TestValue::Str(String::new())));
    assert!(shrunk.contains(&TestValue::Str("ello".to_string())));
    assert!(shrunk.contains(&TestValue::Str("hell".to_string())));
}

#[test]
fn test_shrink_array() {
    let arr = TestValue::Array(vec![
        TestValue::Int(1),
        TestValue::Int(2),
        TestValue::Int(3),
    ]);
    let shrunk = Shrinker::shrink(&arr);
    assert!(shrunk.contains(&TestValue::Array(vec![])));
    // Should contain arrays with one element removed
    assert!(shrunk.contains(&TestValue::Array(vec![
        TestValue::Int(2),
        TestValue::Int(3)
    ])));
}

#[test]
fn test_type_hint_from_str() {
    assert_eq!(TypeHint::parse_type("i64"), TypeHint::I64);
    assert_eq!(TypeHint::parse_type("bool"), TypeHint::Bool);
    assert_eq!(TypeHint::parse_type("str"), TypeHint::Str);
    assert_eq!(TypeHint::parse_type("unknown"), TypeHint::Unknown);
}

#[test]
fn test_test_value_display() {
    assert_eq!(format!("{}", TestValue::Int(42)), "42");
    assert_eq!(format!("{}", TestValue::Bool(true)), "true");
    assert_eq!(format!("{}", TestValue::Str("hi".into())), "\"hi\"");
    assert_eq!(
        format!(
            "{}",
            TestValue::Array(vec![TestValue::Int(1), TestValue::Int(2)])
        ),
        "[1, 2]"
    );
}

#[test]
fn test_property_display() {
    assert_eq!(format!("{}", Property::DoesNotCrash), "does_not_crash");
    assert_eq!(format!("{}", Property::Commutative), "commutative");
    assert_eq!(
        format!("{}", Property::ReturnsInRange(0, 100)),
        "returns_in_range(0, 100)"
    );
}

#[test]
fn test_generator_with_bool_params() {
    let gen = TestGenerator::new().with_num_cases(5);
    let suite = gen.generate("xor", &[TypeHint::Bool, TypeHint::Bool], &TypeHint::Bool);
    assert!(!suite.test_cases.is_empty());
}

#[test]
fn test_generator_with_string_params() {
    let gen = TestGenerator::new().with_num_cases(5);
    let suite = gen.generate("concat", &[TypeHint::Str, TypeHint::Str], &TypeHint::Str);
    assert!(!suite.test_cases.is_empty());
}

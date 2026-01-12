//! Type checker tests

use crate::check;
use crate::error::TypeCheckError;

fn parse_and_check(source: &str) -> Result<(), Vec<TypeCheckError>> {
    let unit = aoel_parser::parse(source).expect("Failed to parse");
    check(&unit)
}

#[test]
fn test_valid_hello_world() {
    let source = r#"
UNIT FUNCTION examples.hello_world V1.0.0

META
  DOMAIN examples.basic
  DETERMINISM true
ENDMETA

INPUT
ENDINPUT

OUTPUT
  message : STRING
ENDOUTPUT

INTENT
  GOAL TRANSFORM: () -> (message)
  PRIORITY CORRECTNESS
ENDINTENT

CONSTRAINT
  REQUIRE output.message != ""
ENDCONSTRAINT

FLOW
  NODE create_message : TRANSFORM
  EDGE create_message -> OUTPUT.message
ENDFLOW

EXECUTION
  PARALLEL false
  TARGET ANY
  MEMORY STACK_ONLY
ENDEXECUTION

VERIFY
  ASSERT output.message == "Hello, World!"
ENDVERIFY

END
"#;

    let result = parse_and_check(source);
    assert!(result.is_ok(), "Expected valid unit, got errors: {:?}", result.err());
}

#[test]
fn test_duplicate_input_field() {
    let source = r#"
UNIT FUNCTION test V1.0.0

META
  DOMAIN test
ENDMETA

INPUT
  name : STRING
  name : INT
ENDINPUT

OUTPUT
ENDOUTPUT

INTENT
  GOAL TRANSFORM: () -> ()
ENDINTENT

CONSTRAINT
ENDCONSTRAINT

FLOW
ENDFLOW

EXECUTION
ENDEXECUTION

VERIFY
ENDVERIFY

END
"#;

    let result = parse_and_check(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| matches!(e, TypeCheckError::DuplicateField { name, .. } if name == "name")),
        "Expected DuplicateField error for 'name'"
    );
}

#[test]
fn test_duplicate_node_id() {
    let source = r#"
UNIT FUNCTION test V1.0.0

META
  DOMAIN test
ENDMETA

INPUT
ENDINPUT

OUTPUT
ENDOUTPUT

INTENT
  GOAL TRANSFORM: () -> ()
ENDINTENT

CONSTRAINT
ENDCONSTRAINT

FLOW
  NODE process : TRANSFORM
  NODE process : VALIDATE
ENDFLOW

EXECUTION
ENDEXECUTION

VERIFY
ENDVERIFY

END
"#;

    let result = parse_and_check(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| matches!(e, TypeCheckError::DuplicateNodeId { id, .. } if id == "process")),
        "Expected DuplicateNodeId error for 'process'"
    );
}

#[test]
fn test_undefined_input_field_reference() {
    let source = r#"
UNIT FUNCTION test V1.0.0

META
  DOMAIN test
ENDMETA

INPUT
  name : STRING
ENDINPUT

OUTPUT
ENDOUTPUT

INTENT
  GOAL TRANSFORM: () -> ()
ENDINTENT

CONSTRAINT
  REQUIRE input.undefined_field != ""
ENDCONSTRAINT

FLOW
ENDFLOW

EXECUTION
ENDEXECUTION

VERIFY
ENDVERIFY

END
"#;

    let result = parse_and_check(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| matches!(e, TypeCheckError::InvalidFieldAccess { field, .. } if field == "undefined_field")),
        "Expected InvalidFieldAccess error for 'undefined_field'"
    );
}

#[test]
fn test_non_bool_constraint() {
    let source = r#"
UNIT FUNCTION test V1.0.0

META
  DOMAIN test
ENDMETA

INPUT
  value : INT
ENDINPUT

OUTPUT
ENDOUTPUT

INTENT
  GOAL TRANSFORM: () -> ()
ENDINTENT

CONSTRAINT
  REQUIRE input.value + 1
ENDCONSTRAINT

FLOW
ENDFLOW

EXECUTION
ENDEXECUTION

VERIFY
ENDVERIFY

END
"#;

    let result = parse_and_check(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| matches!(e, TypeCheckError::NonBoolConstraint { .. })),
        "Expected NonBoolConstraint error"
    );
}

#[test]
fn test_invalid_arithmetic_operand() {
    let source = r#"
UNIT FUNCTION test V1.0.0

META
  DOMAIN test
ENDMETA

INPUT
  name : STRING
ENDINPUT

OUTPUT
ENDOUTPUT

INTENT
  GOAL TRANSFORM: () -> ()
ENDINTENT

CONSTRAINT
  REQUIRE (input.name + 1) > 0
ENDCONSTRAINT

FLOW
ENDFLOW

EXECUTION
ENDEXECUTION

VERIFY
ENDVERIFY

END
"#;

    let result = parse_and_check(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| matches!(e, TypeCheckError::InvalidOperandType { operator, .. } if operator == "+")),
        "Expected InvalidOperandType error for '+'"
    );
}

#[test]
fn test_invalid_logical_operand() {
    let source = r#"
UNIT FUNCTION test V1.0.0

META
  DOMAIN test
ENDMETA

INPUT
  value : INT
ENDINPUT

OUTPUT
ENDOUTPUT

INTENT
  GOAL TRANSFORM: () -> ()
ENDINTENT

CONSTRAINT
  REQUIRE input.value AND true
ENDCONSTRAINT

FLOW
ENDFLOW

EXECUTION
ENDEXECUTION

VERIFY
ENDVERIFY

END
"#;

    let result = parse_and_check(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| matches!(e, TypeCheckError::InvalidOperandType { operator, .. } if operator == "AND")),
        "Expected InvalidOperandType error for 'AND'"
    );
}

#[test]
fn test_valid_with_builtin_function() {
    let source = r#"
UNIT FUNCTION test V1.0.0

META
  DOMAIN test
ENDMETA

INPUT
  name : STRING
ENDINPUT

OUTPUT
ENDOUTPUT

INTENT
  GOAL TRANSFORM: () -> ()
ENDINTENT

CONSTRAINT
  REQUIRE LEN(input.name) > 0
ENDCONSTRAINT

FLOW
ENDFLOW

EXECUTION
ENDEXECUTION

VERIFY
ENDVERIFY

END
"#;

    let result = parse_and_check(source);
    assert!(result.is_ok(), "Expected valid unit with builtin function, got errors: {:?}", result.err());
}

#[test]
fn test_invalid_edge_source() {
    let source = r#"
UNIT FUNCTION test V1.0.0

META
  DOMAIN test
ENDMETA

INPUT
  data : STRING
ENDINPUT

OUTPUT
  result : STRING
ENDOUTPUT

INTENT
  GOAL TRANSFORM: (data) -> (result)
ENDINTENT

CONSTRAINT
ENDCONSTRAINT

FLOW
  NODE process : TRANSFORM
  EDGE undefined_node -> process
ENDFLOW

EXECUTION
ENDEXECUTION

VERIFY
ENDVERIFY

END
"#;

    let result = parse_and_check(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| matches!(e, TypeCheckError::InvalidEdgeSource { .. })),
        "Expected InvalidEdgeSource error"
    );
}

#[test]
fn test_valid_flow_with_input_output() {
    let source = r#"
UNIT FUNCTION test V1.0.0

META
  DOMAIN test
ENDMETA

INPUT
  data : STRING
ENDINPUT

OUTPUT
  result : STRING
ENDOUTPUT

INTENT
  GOAL TRANSFORM: (data) -> (result)
ENDINTENT

CONSTRAINT
ENDCONSTRAINT

FLOW
  NODE process : TRANSFORM
  EDGE INPUT.data -> process
  EDGE process -> OUTPUT.result
ENDFLOW

EXECUTION
ENDEXECUTION

VERIFY
ENDVERIFY

END
"#;

    let result = parse_and_check(source);
    assert!(result.is_ok(), "Expected valid flow, got errors: {:?}", result.err());
}

#[test]
fn test_undefined_function_call() {
    let source = r#"
UNIT FUNCTION test V1.0.0

META
  DOMAIN test
ENDMETA

INPUT
  value : INT
ENDINPUT

OUTPUT
ENDOUTPUT

INTENT
  GOAL TRANSFORM: () -> ()
ENDINTENT

CONSTRAINT
  REQUIRE UNDEFINED_FUNC(input.value) > 0
ENDCONSTRAINT

FLOW
ENDFLOW

EXECUTION
ENDEXECUTION

VERIFY
ENDVERIFY

END
"#;

    let result = parse_and_check(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| matches!(e, TypeCheckError::UndefinedReference { name, .. } if name == "UNDEFINED_FUNC")),
        "Expected UndefinedReference error for 'UNDEFINED_FUNC'"
    );
}

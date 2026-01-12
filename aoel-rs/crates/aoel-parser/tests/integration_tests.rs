//! Integration tests for the AOEL parser

use aoel_parser::parse;
use aoel_ast::*;

/// Test parsing the simplest possible AOEL unit
#[test]
fn test_minimal_unit() {
    let source = r#"
UNIT FUNCTION minimal V1.0.0
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
ENDFLOW
EXECUTION
ENDEXECUTION
VERIFY
ENDVERIFY
END
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse minimal unit: {:?}", result.err());

    let unit = result.unwrap();
    assert_eq!(unit.header.kind, UnitKind::Function);
    assert_eq!(unit.full_name(), "minimal");
    assert!(unit.version().is_some());
    assert_eq!(unit.version().unwrap().major, 1);
    assert_eq!(unit.version().unwrap().minor, 0);
    assert_eq!(unit.version().unwrap().patch, 0);
}

/// Test parsing hello world example
#[test]
fn test_hello_world() {
    let source = r#"
UNIT FUNCTION hello_world V1.0.0
META
  DOMAIN greeting
  DETERMINISM true
  PURE true
ENDMETA

INPUT
  name: STRING
ENDINPUT

OUTPUT
  greeting: STRING
ENDOUTPUT

INTENT
  GOAL TRANSFORM: (name) -> (greeting)
  PRIORITY CORRECTNESS
ENDINTENT

CONSTRAINT
  REQUIRE LEN(name) > 0
ENDCONSTRAINT

FLOW
  NODE format: TRANSFORM(template="Hello, {name}!")
  EDGE name -> format
  EDGE format -> greeting
ENDFLOW

EXECUTION
  PARALLEL false
  TARGET ANY
  MEMORY STACK_ONLY
ENDEXECUTION

VERIFY
  ASSERT LEN(greeting) > 0
ENDVERIFY
END
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse hello_world: {:?}", result.err());

    let unit = result.unwrap();
    assert_eq!(unit.full_name(), "hello_world");
    assert_eq!(unit.meta.entries.len(), 3);
    assert_eq!(unit.input.fields.len(), 1);
    assert_eq!(unit.output.fields.len(), 1);
    assert_eq!(unit.flow.nodes.len(), 1);
    assert_eq!(unit.flow.edges.len(), 2);
    assert_eq!(unit.verify.entries.len(), 1);
}

/// Test parsing various type definitions
#[test]
fn test_type_parsing() {
    let source = r#"
UNIT FUNCTION types_test V1.0.0
META
  DOMAIN test
ENDMETA

INPUT
  string_field: STRING
  int_field: INT
  int64_field: INT64
  float_field: FLOAT64
  bool_field: BOOL
  bytes_field: BYTES
  array_field: ARRAY<STRING>
  optional_field: OPTIONAL<INT>
  struct_field: STRUCT{name: STRING, age: INT}
ENDINPUT

OUTPUT
  result: BOOL
ENDOUTPUT

INTENT
  GOAL VALIDATE: (string_field) -> (result)
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

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse types_test: {:?}", result.err());

    let unit = result.unwrap();
    assert_eq!(unit.input.fields.len(), 9);

    // Check STRING type
    if let Type::Primitive(prim) = &unit.input.fields[0].ty {
        assert_eq!(prim.kind, PrimitiveKind::String);
    } else {
        panic!("Expected STRING type");
    }

    // Check ARRAY<STRING> type
    if let Type::Array(array_type) = &unit.input.fields[6].ty {
        if let Type::Primitive(prim) = &array_type.element_type {
            assert_eq!(prim.kind, PrimitiveKind::String);
        } else {
            panic!("Expected ARRAY<STRING> element type");
        }
    } else {
        panic!("Expected ARRAY type");
    }

    // Check OPTIONAL<INT> type
    if let Type::Optional(opt_type) = &unit.input.fields[7].ty {
        if let Type::Primitive(prim) = &opt_type.inner_type {
            assert_eq!(prim.kind, PrimitiveKind::Int);
        } else {
            panic!("Expected OPTIONAL<INT> inner type");
        }
    } else {
        panic!("Expected OPTIONAL type");
    }
}

/// Test parsing expressions
#[test]
fn test_expression_parsing() {
    let source = r#"
UNIT FUNCTION expr_test V1.0.0
META
  DOMAIN test
ENDMETA

INPUT
  a: INT
  b: INT
  name: STRING
ENDINPUT

OUTPUT
  result: BOOL
ENDOUTPUT

INTENT
  GOAL VALIDATE: (a, b, name) -> (result)
ENDINTENT

CONSTRAINT
  REQUIRE a > 0 AND b < 100
  REQUIRE a + b == 50
  FORBID name == ""
  REQUIRE LEN(name) > 0
ENDCONSTRAINT

FLOW
ENDFLOW

EXECUTION
ENDEXECUTION

VERIFY
  ASSERT a >= 0
  ASSERT b <= 100
ENDVERIFY
END
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse expr_test: {:?}", result.err());

    let unit = result.unwrap();
    assert_eq!(unit.constraint.constraints.len(), 4);
    assert_eq!(unit.verify.entries.len(), 2);
}

/// Test parsing flow blocks with nodes and edges
#[test]
fn test_flow_parsing() {
    let source = r#"
UNIT PIPELINE data_pipeline V1.0.0
META
  DOMAIN data
ENDMETA

INPUT
  items: ARRAY<STRING>
ENDINPUT

OUTPUT
  processed: ARRAY<STRING>
ENDOUTPUT

INTENT
  GOAL TRANSFORM: (items) -> (processed)
ENDINTENT

CONSTRAINT
ENDCONSTRAINT

FLOW
  NODE validate_input: VALIDATE
  NODE filter_empty: FILTER(predicate=item != "")
  NODE transform_data: MAP(fn=@utils.uppercase)
  NODE collect: REDUCE

  EDGE items -> validate_input
  EDGE validate_input -> filter_empty
  EDGE filter_empty -> transform_data
  EDGE transform_data -> collect
  EDGE collect -> processed
ENDFLOW

EXECUTION
  PARALLEL true
  TARGET CPU
ENDEXECUTION

VERIFY
ENDVERIFY
END
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse flow: {:?}", result.err());

    let unit = result.unwrap();
    assert_eq!(unit.header.kind, UnitKind::Pipeline);
    assert_eq!(unit.flow.nodes.len(), 4);
    assert_eq!(unit.flow.edges.len(), 5);

    // Check node operations
    assert_eq!(unit.flow.nodes[0].op_type, OpType::Validate);
    assert_eq!(unit.flow.nodes[1].op_type, OpType::Filter);
    assert_eq!(unit.flow.nodes[2].op_type, OpType::Map);
    assert_eq!(unit.flow.nodes[3].op_type, OpType::Reduce);
}

/// Test parsing conditional edges
#[test]
fn test_conditional_edges() {
    let source = r#"
UNIT FUNCTION conditional V1.0.0
META
  DOMAIN test
ENDMETA

INPUT
  value: INT
ENDINPUT

OUTPUT
  result: STRING
ENDOUTPUT

INTENT
  GOAL ROUTE: (value) -> (result)
ENDINTENT

CONSTRAINT
ENDCONSTRAINT

FLOW
  NODE check: BRANCH
  NODE path_a: TRANSFORM
  NODE path_b: TRANSFORM
  NODE merge: MERGE

  EDGE value -> check
  EDGE check -> path_a WHEN value > 0
  EDGE check -> path_b WHEN value <= 0
  EDGE path_a -> merge
  EDGE path_b -> merge
  EDGE merge -> result
ENDFLOW

EXECUTION
ENDEXECUTION

VERIFY
ENDVERIFY
END
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse conditional: {:?}", result.err());

    let unit = result.unwrap();

    // Find edges with conditions
    let conditional_edges: Vec<_> = unit.flow.edges.iter()
        .filter(|e| e.condition.is_some())
        .collect();

    assert_eq!(conditional_edges.len(), 2);
}

/// Test parsing execution block options
#[test]
fn test_execution_block() {
    let source = r#"
UNIT SERVICE cached_service V1.0.0
META
  DOMAIN cache
  TIMEOUT 30s
ENDMETA

INPUT
  key: STRING
ENDINPUT

OUTPUT
  value: STRING
ENDOUTPUT

INTENT
  GOAL FETCH: (key) -> (value)
  ON_FAILURE ABORT
ENDINTENT

CONSTRAINT
ENDCONSTRAINT

FLOW
ENDFLOW

EXECUTION
  PARALLEL true
  TARGET NATIVE
  MEMORY BOUNDED 256MB
  ISOLATION PROCESS
  CACHE LRU 1000
ENDEXECUTION

VERIFY
ENDVERIFY
END
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse execution_block: {:?}", result.err());

    let unit = result.unwrap();
    assert_eq!(unit.header.kind, UnitKind::Service);
    assert!(unit.execution.parallel);
    assert_eq!(unit.execution.target, TargetKind::Native);
    assert_eq!(unit.execution.isolation, IsolationKind::Process);

    if let MemoryMode::Bounded(size) = &unit.execution.memory {
        assert_eq!(size, "256MB");
    } else {
        panic!("Expected BOUNDED memory mode");
    }

    if let CacheMode::Lru(Some(capacity)) = unit.execution.cache {
        assert_eq!(capacity, 1000);
    } else {
        panic!("Expected LRU cache mode");
    }
}

/// Test parsing verify block with different entry types
#[test]
fn test_verify_block() {
    let source = r#"
UNIT FUNCTION verified V1.0.0
META
  DOMAIN test
ENDMETA

INPUT
  items: ARRAY<INT>
ENDINPUT

OUTPUT
  total: INT
ENDOUTPUT

INTENT
  GOAL AGGREGATE: (items) -> (total)
ENDINTENT

CONSTRAINT
ENDCONSTRAINT

FLOW
ENDFLOW

EXECUTION
ENDEXECUTION

VERIFY
  ASSERT total >= 0
  PROPERTY LEN(items) > 0
  INVARIANT LEN(items) >= 0
  POSTCONDITION total == SUM(items)
  TEST @tests.sum_test
ENDVERIFY
END
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse verify_block: {:?}", result.err());

    let unit = result.unwrap();
    assert_eq!(unit.verify.entries.len(), 5);

    assert_eq!(unit.verify.entries[0].kind, VerifyKind::Assert);
    assert_eq!(unit.verify.entries[1].kind, VerifyKind::Property);
    assert_eq!(unit.verify.entries[2].kind, VerifyKind::Invariant);
    assert_eq!(unit.verify.entries[3].kind, VerifyKind::Postcondition);
    assert_eq!(unit.verify.entries[4].kind, VerifyKind::Test);

    // Check that TEST has external reference
    assert!(unit.verify.entries[4].test_ref.is_some());
}

/// Test parsing external references
#[test]
fn test_external_refs() {
    let source = r#"
UNIT FUNCTION with_refs V1.0.0
META
  DOMAIN test
ENDMETA

INPUT
  data: STRING
ENDINPUT

OUTPUT
  result: STRING
ENDOUTPUT

INTENT
  GOAL TRANSFORM: (data) -> (result)
  ON_FAILURE FALLBACK @handlers.fallback_handler
ENDINTENT

CONSTRAINT
ENDCONSTRAINT

FLOW
  NODE process: @external.processor
  EDGE data -> process
  EDGE process -> result
ENDFLOW

EXECUTION
ENDEXECUTION

VERIFY
  TEST @tests.integration_test
ENDVERIFY
END
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse external_refs: {:?}", result.err());

    let unit = result.unwrap();

    // Check ON_FAILURE fallback
    if let Some(FailureStrategy::Fallback(ref ext_ref)) = unit.intent.on_failure {
        assert_eq!(ext_ref.path, "handlers.fallback_handler");
    } else {
        panic!("Expected FALLBACK failure strategy");
    }

    // Check external node reference
    assert!(unit.flow.nodes[0].custom_op.is_some());
    assert_eq!(unit.flow.nodes[0].custom_op.as_ref().unwrap().path, "external.processor");
}

/// Test parsing qualified names
#[test]
fn test_qualified_names() {
    let source = r#"
UNIT FUNCTION my.namespace.function_name V2.1.3
META
  DOMAIN "my.domain.area"
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
ENDFLOW

EXECUTION
ENDEXECUTION

VERIFY
ENDVERIFY
END
"#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse qualified_names: {:?}", result.err());

    let unit = result.unwrap();
    assert_eq!(unit.full_name(), "my.namespace.function_name");
    assert_eq!(unit.header.name.parts.len(), 3);
    assert_eq!(unit.version().unwrap().major, 2);
    assert_eq!(unit.version().unwrap().minor, 1);
    assert_eq!(unit.version().unwrap().patch, 3);
}

/// Test error recovery - missing END keyword
#[test]
fn test_error_missing_end() {
    let source = r#"
UNIT FUNCTION broken V1.0.0
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
ENDFLOW
EXECUTION
ENDEXECUTION
VERIFY
ENDVERIFY
"#;

    let result = parse(source);
    assert!(result.is_err());
}

/// Test error recovery - invalid token
#[test]
fn test_error_invalid_goal_type() {
    let source = r#"
UNIT FUNCTION broken V1.0.0
META
  DOMAIN test
ENDMETA
INPUT
ENDINPUT
OUTPUT
ENDOUTPUT
INTENT
  GOAL INVALID_GOAL: () -> ()
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

    let result = parse(source);
    assert!(result.is_err());
}

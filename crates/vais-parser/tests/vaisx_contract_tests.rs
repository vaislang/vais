//! VaisX Contract Tests — Parser Compatibility
//!
//! Verifies that desugar'd VaisX code parses correctly through the core
//! vais-parser's `parse()` and `parse_with_recovery()` functions.
//!
//! These tests ensure the interface contract between vaisx-parser (desugar)
//! and the core parser remains stable.

use vais_ast::*;
use vais_parser::{parse, parse_with_recovery};

// ============================================================================
// 1. __vx_state() — $state(x) desugar
// ============================================================================

#[test]
fn test_vx_state_integer_literal() {
    let source = "F init() { count := __vx_state(0) }";
    let module = parse(source).expect("__vx_state(0) should parse");
    assert_eq!(module.items.len(), 1);
    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.name.node, "init");
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_vx_state_string_literal() {
    let source = r#"F init() { name := __vx_state("world") }"#;
    let module = parse(source).expect("__vx_state with string should parse");
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_vx_state_boolean_literal() {
    let source = "F init() { flag := __vx_state(true) }";
    let module = parse(source).expect("__vx_state with bool should parse");
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_vx_state_empty_array() {
    let source = "F init() { items := __vx_state([]) }";
    let module = parse(source).expect("__vx_state with empty array should parse");
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_vx_state_multiple_declarations() {
    let source = r#"
        F init() {
            count := __vx_state(0)
            name := __vx_state("world")
            flag := __vx_state(false)
        }
    "#;
    let module = parse(source).expect("Multiple __vx_state declarations should parse");
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// 2. __vx_derived() — $derived(expr) desugar
// ============================================================================

#[test]
fn test_vx_derived_simple_expr() {
    let source = "F init() { doubled := __vx_derived(|| { count * 2 }) }";
    let module = parse(source).expect("__vx_derived with closure should parse");
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_vx_derived_string_concat() {
    let source = r#"F init() { greeting := __vx_derived(|| { "Hello, " + name + "!" }) }"#;
    let module = parse(source).expect("__vx_derived with string concat should parse");
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_vx_derived_complex_expr() {
    let source = r#"
        F init() {
            count := __vx_state(0)
            doubled := __vx_derived(|| { count * 2 })
            tripled := __vx_derived(|| { count * 3 })
        }
    "#;
    let module = parse(source).expect("Multiple __vx_derived should parse");
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// 3. __vx_effect() — $effect { body } desugar
// ============================================================================

#[test]
fn test_vx_effect_simple() {
    let source = r#"F init() { __vx_effect(|| { console_log("changed") }) }"#;
    let module = parse(source).expect("__vx_effect with closure should parse");
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_vx_effect_multi_statement() {
    let source = r#"
        F init() {
            __vx_effect(|| {
                x := count + 1
                console_log("count: ", x)
            })
        }
    "#;
    let module = parse(source).expect("__vx_effect with multi-statement body should parse");
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// 4. S __VxProps__ — P { } desugar
// ============================================================================

#[test]
fn test_vx_props_basic_struct() {
    let source = "S __VxProps__ { user: User, showAvatar: bool }";
    let module = parse(source).expect("S __VxProps__ should parse as struct");
    assert_eq!(module.items.len(), 1);
    match &module.items[0].node {
        Item::Struct(s) => {
            assert_eq!(s.name.node, "__VxProps__");
            assert_eq!(s.fields.len(), 2);
            assert_eq!(s.fields[0].name.node, "user");
            assert_eq!(s.fields[1].name.node, "showAvatar");
        }
        _ => panic!("Expected Struct"),
    }
}

#[test]
fn test_vx_props_single_field() {
    let source = "S __VxProps__ { count: i64 }";
    let module = parse(source).expect("S __VxProps__ with single field should parse");
    assert_eq!(module.items.len(), 1);
    match &module.items[0].node {
        Item::Struct(s) => {
            assert_eq!(s.name.node, "__VxProps__");
            assert_eq!(s.fields.len(), 1);
        }
        _ => panic!("Expected Struct"),
    }
}

#[test]
fn test_vx_props_complex_types() {
    let source = "S __VxProps__ { items: Vec<Item>, callback: Fn<i64, bool> }";
    let module = parse(source).expect("S __VxProps__ with complex types should parse");
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// 5. __vx_emit() — emit x() desugar
// ============================================================================

#[test]
fn test_vx_emit_simple() {
    let source = r#"F handleClick() { __vx_emit("select") }"#;
    let module = parse(source).expect("__vx_emit should parse as function call");
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_vx_emit_with_args() {
    let source = r#"F handleClick() { __vx_emit("select", user) }"#;
    let module = parse(source).expect("__vx_emit with args should parse");
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// 6. #[server] / #[wasm] attributes
// ============================================================================

#[test]
fn test_server_attribute_async_function() {
    let source = r#"
        #[server]
        A F loadItems() -> Vec<Item> {
            items
        }
    "#;
    let module = parse(source).expect("#[server] async function should parse");
    assert_eq!(module.items.len(), 1);
    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.name.node, "loadItems");
            assert!(f.is_async);
            assert_eq!(f.attributes.len(), 1);
            assert_eq!(f.attributes[0].name, "server");
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_wasm_attribute_function() {
    let source = r#"
        #[wasm]
        F processData(raw: Vec<f64>) -> Vec<f64> {
            raw
        }
    "#;
    let module = parse(source).expect("#[wasm] function should parse");
    assert_eq!(module.items.len(), 1);
    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.name.node, "processData");
            assert!(!f.is_async);
            assert_eq!(f.attributes.len(), 1);
            assert_eq!(f.attributes[0].name, "wasm");
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_server_attribute_with_args() {
    let source = r#"
        #[server(rate_limit = "10/min")]
        A F sensitiveAction(data: str) -> str {
            data
        }
    "#;
    let module = parse(source).expect("#[server(rate_limit)] should parse");
    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.attributes[0].name, "server");
            assert!(!f.attributes[0].args.is_empty());
        }
        _ => panic!("Expected Function"),
    }
}

// ============================================================================
// 7. Combined desugar patterns
// ============================================================================

#[test]
fn test_combined_state_derived_effect() {
    let source = r#"
        F component() {
            count := __vx_state(0)
            doubled := __vx_derived(|| { count * 2 })
            __vx_effect(|| { console_log("count: ", count) })
        }
    "#;
    let module = parse(source).expect("Combined state+derived+effect should parse");
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_combined_props_and_functions() {
    let source = r#"
        S __VxProps__ {
            user: User,
            showAvatar: bool
        }

        F handleClick() {
            __vx_emit("select", user)
        }
    "#;
    let module = parse(source).expect("Props + functions should parse");
    assert_eq!(module.items.len(), 2);
}

#[test]
fn test_full_component_desugar() {
    let source = r#"
        S __VxProps__ {
            initial: i64
        }

        F setup() {
            count := __vx_state(0)
            doubled := __vx_derived(|| { count * 2 })
            __vx_effect(|| { console_log("updated") })
        }

        F increment() {
            count := count + 1
        }

        #[server]
        A F loadData() -> Vec<i64> {
            data
        }

        #[wasm]
        F heavyCompute(input: Vec<f64>) -> f64 {
            0.0
        }
    "#;
    let module = parse(source).expect("Full component desugar should parse");
    assert_eq!(module.items.len(), 5);
}

// ============================================================================
// 8. Error recovery mode
// ============================================================================

#[test]
fn test_recovery_mode_with_desugar() {
    let source = r#"
        F init() {
            count := __vx_state(0)
            doubled := __vx_derived(|| { count * 2 })
        }
    "#;
    let (module, errors) = parse_with_recovery(source);
    assert!(
        errors.is_empty(),
        "Valid desugar code should have no errors"
    );
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_recovery_mode_partial_error() {
    // First item is broken, second should still parse.
    // Use a more recoverable pattern: broken function with closing brace, then struct
    let source = r#"
        F broken( {}
        S __VxProps__ { x: i64 }
    "#;
    let (module, errors) = parse_with_recovery(source);
    assert!(!errors.is_empty(), "Should have errors for broken function");
    // The struct should still be recovered
    let has_struct = module
        .items
        .iter()
        .any(|item| matches!(&item.node, Item::Struct(s) if s.name.node == "__VxProps__"));
    assert!(
        has_struct,
        "__VxProps__ struct should be recovered after error"
    );
}

// ============================================================================
// 9. Double-underscore identifier validity
// ============================================================================

#[test]
fn test_double_underscore_identifiers() {
    let source = r#"
        F test() {
            __vx_state := 1
            __vx_derived := 2
            __vx_effect := 3
            __vx_emit := 4
        }
    "#;
    let module = parse(source).expect("__vx_* identifiers should be valid");
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// 10. Edge cases
// ============================================================================

#[test]
fn test_nested_closures_in_derived() {
    let source = r#"
        F init() {
            result := __vx_derived(|| {
                items := [1, 2, 3]
                items
            })
        }
    "#;
    let module = parse(source).expect("Nested expressions in derived closure should parse");
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_vx_state_with_struct_literal() {
    let source = r#"
        S Point { x: f64, y: f64 }
        F init() {
            pos := __vx_state(Point { x: 0.0, y: 0.0 })
        }
    "#;
    let module = parse(source).expect("__vx_state with struct literal should parse");
    assert_eq!(module.items.len(), 2);
}

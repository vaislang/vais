//! Tests for WASM VM (can run without wasm-pack)

use aoel_lowering::Lowerer;

// Note: These tests verify the non-WASM parts of the playground
// The actual WASM tests require wasm-bindgen-test

#[test]
fn test_parse_simple() {
    let source = "add(a, b) = a + b";
    let program = aoel_parser::parse(source);
    assert!(program.is_ok());
}

#[test]
fn test_parse_factorial() {
    let source = "factorial(n) = n < 2 ? 1 : n * $(n - 1)";
    let program = aoel_parser::parse(source);
    assert!(program.is_ok());
}

#[test]
fn test_parse_array_ops() {
    // Test array map operation
    let source = "double_all(arr) = arr.@(_ * 2)";
    let program = aoel_parser::parse(source);
    assert!(program.is_ok(), "Failed to parse map: {:?}", program.err());

    // Test array filter operation
    let source = "get_evens(arr) = arr.?(_ % 2 == 0)";
    let program = aoel_parser::parse(source);
    assert!(program.is_ok(), "Failed to parse filter: {:?}", program.err());

    // Test array reduce operation (./+ for sum, ./* for product)
    let source = "sum_all(arr) = arr./+";
    let program = aoel_parser::parse(source);
    assert!(program.is_ok(), "Failed to parse reduce: {:?}", program.err());

    // Test chained operations
    let source = "process(arr) = arr.@(_ * 2).?(_ > 5)";
    let program = aoel_parser::parse(source);
    assert!(program.is_ok(), "Failed to parse chained ops: {:?}", program.err());
}

#[test]
fn test_lowering() {
    let source = "add(a, b) = a + b";
    let program = aoel_parser::parse(source).unwrap();

    let mut lowerer = Lowerer::new();
    let functions = lowerer.lower_program(&program);
    assert!(functions.is_ok());

    let funcs = functions.unwrap();
    assert!(!funcs.is_empty());
}

#[test]
fn test_lowering_factorial() {
    let source = "factorial(n) = n < 2 ? 1 : n * $(n - 1)";
    let program = aoel_parser::parse(source).unwrap();

    let mut lowerer = Lowerer::new();
    let functions = lowerer.lower_program(&program);
    assert!(functions.is_ok());
}

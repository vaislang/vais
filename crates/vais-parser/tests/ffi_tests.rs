//! FFI parsing tests

use vais_ast::{Item, Type};
use vais_lexer::tokenize;
use vais_parser::Parser;

#[test]
fn test_extern_block_basic() {
    let source = r#"N "C" { F malloc(size: i64) -> *i8; }"#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    assert_eq!(module.items.len(), 1);
    match &module.items[0].node {
        Item::ExternBlock(block) => {
            assert_eq!(block.abi, "C");
            assert_eq!(block.functions.len(), 1);
            assert_eq!(block.functions[0].name.node, "malloc");
            assert!(!block.functions[0].is_vararg);
        }
        _ => panic!("Expected ExternBlock"),
    }
}

#[test]
fn test_extern_block_vararg() {
    let source = r#"N "C" { F printf(fmt: *i8, ...) -> i32; }"#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::ExternBlock(block) => {
            assert_eq!(block.functions[0].name.node, "printf");
            assert!(block.functions[0].is_vararg);
        }
        _ => panic!("Expected ExternBlock"),
    }
}

#[test]
fn test_extern_block_multiple_functions() {
    let source = r#"
        N "C" {
            F malloc(size: i64) -> *i8;
            F free(ptr: *i8) -> ();
            F printf(fmt: *i8, ...) -> i32;
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::ExternBlock(block) => {
            assert_eq!(block.functions.len(), 3);
            assert_eq!(block.functions[0].name.node, "malloc");
            assert_eq!(block.functions[1].name.node, "free");
            assert_eq!(block.functions[2].name.node, "printf");
            assert!(block.functions[2].is_vararg);
        }
        _ => panic!("Expected ExternBlock"),
    }
}

#[test]
fn test_function_pointer_type() {
    let source = "F test(callback: fn(i32, i32) -> i64) -> i64 = 0";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => {
            assert_eq!(f.params.len(), 1);
            match &f.params[0].ty.node {
                Type::FnPtr {
                    params, is_vararg, ..
                } => {
                    assert_eq!(params.len(), 2);
                    assert!(!is_vararg);
                }
                _ => panic!("Expected FnPtr type"),
            }
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_function_pointer_vararg() {
    let source = "F test(callback: fn(i32, ...) -> i32) -> i32 = 0";
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Function(f) => match &f.params[0].ty.node {
            Type::FnPtr { is_vararg, .. } => {
                assert!(is_vararg);
            }
            _ => panic!("Expected FnPtr type"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_repr_c_attribute() {
    let source = r#"
        #[repr(C)]
        S Point {
            x: i32,
            y: i32
        }
    "#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::Struct(s) => {
            assert_eq!(s.name.node, "Point");
            assert_eq!(s.attributes.len(), 1);
            assert_eq!(s.attributes[0].name, "repr");
            assert_eq!(s.attributes[0].args.len(), 1);
            assert_eq!(s.attributes[0].args[0], "C");
        }
        _ => panic!("Expected Struct"),
    }
}

#[test]
fn test_extern_default_abi() {
    let source = r#"N { F test() -> i32; }"#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::ExternBlock(block) => {
            assert_eq!(block.abi, "C"); // Default ABI should be C
        }
        _ => panic!("Expected ExternBlock"),
    }
}

#[test]
fn test_vararg_after_multiple_params() {
    let source = r#"N "C" { F fprintf(stream: *i8, fmt: *i8, ...) -> i32; }"#;
    let tokens = tokenize(source).unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    match &module.items[0].node {
        Item::ExternBlock(block) => {
            assert_eq!(block.functions[0].params.len(), 2);
            assert!(block.functions[0].is_vararg);
        }
        _ => panic!("Expected ExternBlock"),
    }
}

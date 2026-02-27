use crate::borrow_check::{check_module, BorrowError, Location};
use crate::*;
use std::collections::HashMap;

use super::helpers::make_test_body;

#[test]
fn test_check_module() {
    // Test checking an entire module
    let body1 = make_test_body(
        MirType::I64,
        vec![Statement::Assign(
            Place::local(Local(0)),
            Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
        )],
        Terminator::Return,
    );

    let body2 = Body {
        name: "test_error".to_string(),
        params: vec![MirType::Struct("TestNonCopy".into())],
        return_type: MirType::Unit,
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::Unit,
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("s".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: false,
                lifetime: None,
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![
                Statement::Drop(Place::local(Local(1))),
                Statement::Drop(Place::local(Local(1))), // Double drop
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };

    let module = MirModule {
        name: "test_module".to_string(),
        bodies: vec![body1, body2],
        structs: HashMap::new(),
        enums: HashMap::new(),
    };

    let errors = check_module(&module);
    assert_eq!(errors.len(), 1, "Should detect error in second body");
}

// New tests for advanced features

#[test]
fn test_error_display_format() {
    // Check that BorrowError::Display produces correct error code format
    let error = BorrowError::UseAfterMove {
        local: Local(1),
        moved_at: Location::new(BasicBlockId(0), 2),
        used_at: Location::new(BasicBlockId(0), 4),
    };
    let display = format!("{}", error);
    assert!(display.contains("error[E100]"));
    assert!(display.contains("use of moved value"));
    assert!(display.contains("moved at 0:2"));
    assert!(display.contains("used at 0:4"));

    let error2 = BorrowError::MutableBorrowConflict {
        local: Local(2),
        first_borrow: Location::new(BasicBlockId(1), 0),
        second_borrow: Location::new(BasicBlockId(1), 1),
    };
    let display2 = format!("{}", error2);
    assert!(display2.contains("error[E103]"));
    assert!(display2.contains("cannot borrow"));
    assert!(display2.contains("mutable more than once"));
}

#[test]
fn test_location_display() {
    // Check Location display format
    let loc = Location::new(BasicBlockId(3), 7);
    let display = format!("{}", loc);
    assert_eq!(display, "3:7");

    let loc2 = Location::new(BasicBlockId(0), 0);
    let display2 = format!("{}", loc2);
    assert_eq!(display2, "0:0");
}

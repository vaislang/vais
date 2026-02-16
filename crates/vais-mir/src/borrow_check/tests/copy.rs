use crate::*;
use crate::borrow_check::check_body;
use std::collections::HashMap;

use super::helpers::make_test_body;

#[test]
fn test_no_errors_copy_types() {
    // Copy types can be used multiple times without errors
    let body = make_test_body(
        MirType::I64,
        vec![
            Statement::Assign(
                Place::local(Local(0)),
                Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
            ),
            Statement::Assign(
                Place::local(Local(0)),
                Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
            ),
        ],
        Terminator::Return,
    );

    let errors = check_body(&body);
    assert_eq!(errors.len(), 0, "Copy types should not produce errors");
}


#[test]
fn test_copy_type_no_move_error() {
    // Copy types can be "moved" multiple times
    let body = make_test_body(
        MirType::I64,
        vec![
            Statement::Assign(
                Place::local(Local(0)),
                Rvalue::Use(Operand::Move(Place::local(Local(1)))),
            ),
            Statement::Assign(
                Place::local(Local(0)),
                Rvalue::Use(Operand::Move(Place::local(Local(1)))),
            ),
        ],
        Terminator::Return,
    );

    let errors = check_body(&body);
    assert_eq!(errors.len(), 0, "Copy types don't actually move");
}


#[test]
fn test_lifetime_ref_copy_semantics() {
    // RefLifetime and RefMutLifetime are Copy types. Can use after move.
    let body = Body {
        name: "test_ref_copy".to_string(),
        params: vec![MirType::RefLifetime {
            lifetime: "a".to_string(),
            inner: Box::new(MirType::Struct("TestNonCopy".into())),
        }],
        return_type: MirType::Unit,
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::Unit,
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("x".to_string()),
                ty: MirType::RefLifetime {
                    lifetime: "a".to_string(),
                    inner: Box::new(MirType::Struct("TestNonCopy".into())),
                },
                is_mutable: false,
                lifetime: Some("a".to_string()),
            },
            LocalDecl {
                name: Some("temp1".to_string()),
                ty: MirType::RefLifetime {
                    lifetime: "a".to_string(),
                    inner: Box::new(MirType::Struct("TestNonCopy".into())),
                },
                is_mutable: false,
                lifetime: Some("a".to_string()),
            },
            LocalDecl {
                name: Some("temp2".to_string()),
                ty: MirType::RefLifetime {
                    lifetime: "a".to_string(),
                    inner: Box::new(MirType::Struct("TestNonCopy".into())),
                },
                is_mutable: false,
                lifetime: Some("a".to_string()),
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![
                // "Move" x (but it's copy, so no actual move)
                Statement::Assign(
                    Place::local(Local(2)),
                    Rvalue::Use(Operand::Move(Place::local(Local(1)))),
                ),
                // Use x again (should be OK since refs are copy)
                Statement::Assign(
                    Place::local(Local(3)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec!["a".to_string()],
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    assert_eq!(errors.len(), 0, "RefLifetime is copy, can use after move");
}

// ========================================================================
// Negative Tests (Error Detection)
// ========================================================================



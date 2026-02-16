use crate::*;
use crate::borrow_check::{check_body, BorrowError};
use std::collections::HashMap;



#[test]
fn test_use_after_move() {
    // Non-copy type (Struct) moved then used again
    let body = Body {
        name: "test_move".to_string(),
        params: vec![MirType::Struct("TestNonCopy".into())],
        return_type: MirType::Struct("TestNonCopy".into()),
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
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
                // Move s to _ret
                Statement::Assign(
                    Place::local(Local(0)),
                    Rvalue::Use(Operand::Move(Place::local(Local(1)))),
                ),
                // Try to use s again (error!)
                Statement::Assign(
                    Place::local(Local(0)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    assert_eq!(errors.len(), 1, "Should detect use-after-move");
    match &errors[0] {
        BorrowError::UseAfterMove { local, .. } => {
            assert_eq!(*local, Local(1));
        }
        _ => panic!("Expected UseAfterMove error"),
    }
}


#[test]
fn test_double_drop() {
    // Drop the same non-copy value twice
    let body = Body {
        name: "test_double_drop".to_string(),
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
                Statement::Drop(Place::local(Local(1))), // Double drop!
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    assert_eq!(errors.len(), 1, "Should detect double-free");
    match &errors[0] {
        BorrowError::DoubleFree { local, .. } => {
            assert_eq!(*local, Local(1));
        }
        _ => panic!("Expected DoubleFree error"),
    }
}


#[test]
fn test_move_prevents_drop() {
    // Move a value, then drop it (should not error - drop is no-op after move)
    let body = Body {
        name: "test_move_no_drop".to_string(),
        params: vec![MirType::Struct("TestNonCopy".into())],
        return_type: MirType::Struct("TestNonCopy".into()),
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
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
                Statement::Assign(
                    Place::local(Local(0)),
                    Rvalue::Use(Operand::Move(Place::local(Local(1)))),
                ),
                Statement::Drop(Place::local(Local(1))), // Drop after move is OK (no-op)
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    assert_eq!(errors.len(), 0, "Drop after move should be allowed (no-op)");
}


#[test]
fn test_assign_after_move() {
    // Move a value, then reassign it, then use it (should be OK)
    let body = Body {
        name: "test_reassign".to_string(),
        params: vec![MirType::Struct("TestNonCopy".into())],
        return_type: MirType::Struct("TestNonCopy".into()),
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("s".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("temp".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: true,
                lifetime: None,
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![
                // Move s to temp
                Statement::Assign(
                    Place::local(Local(2)),
                    Rvalue::Use(Operand::Move(Place::local(Local(1)))),
                ),
                // Reassign s with the parameter (Local 0 holds new value conceptually)
                Statement::Assign(
                    Place::local(Local(1)),
                    Rvalue::Use(Operand::Constant(Constant::Str("new".to_string()))),
                ),
                // Use s again (should be OK now)
                Statement::Assign(
                    Place::local(Local(0)),
                    Rvalue::Use(Operand::Move(Place::local(Local(1)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    assert_eq!(
        errors.len(),
        0,
        "Reassignment should reinitialize the local"
    );
}


#[test]
fn test_use_after_drop() {
    // Drop a value, then try to use it
    let body = Body {
        name: "test_use_after_drop".to_string(),
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
            LocalDecl {
                name: Some("temp".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: true,
                lifetime: None,
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![
                Statement::Drop(Place::local(Local(1))),
                // Try to use s after drop (error!)
                Statement::Assign(
                    Place::local(Local(2)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    assert_eq!(errors.len(), 1, "Should detect use-after-free");
    match &errors[0] {
        BorrowError::UseAfterFree { local, .. } => {
            assert_eq!(*local, Local(1));
        }
        _ => panic!("Expected UseAfterFree error"),
    }
}


#[test]
fn test_move_while_borrowed() {
    // Borrow a value, then try to move it
    let body = Body {
        name: "test_move_borrowed".to_string(),
        params: vec![MirType::Struct("TestNonCopy".into())],
        return_type: MirType::Struct("TestNonCopy".into()),
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("s".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: false,
                lifetime: None,
            },
            LocalDecl {
                name: Some("r".to_string()),
                ty: MirType::Ref(Box::new(MirType::Struct("TestNonCopy".into()))),
                is_mutable: false,
                lifetime: None,
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![
                // Borrow s
                Statement::Assign(Place::local(Local(2)), Rvalue::Ref(Place::local(Local(1)))),
                // Try to move s (error!)
                Statement::Assign(
                    Place::local(Local(0)),
                    Rvalue::Use(Operand::Move(Place::local(Local(1)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    assert_eq!(errors.len(), 1, "Should detect move-while-borrowed");
    match &errors[0] {
        BorrowError::MoveWhileBorrowed { local, .. } => {
            assert_eq!(*local, Local(1));
        }
        _ => panic!("Expected MoveWhileBorrowed error"),
    }
}


#[test]
fn test_return_value_not_dropped() {
    // Return a non-copy value (should not cause errors)
    let body = Body {
        name: "test_return".to_string(),
        params: vec![MirType::Struct("TestNonCopy".into())],
        return_type: MirType::Struct("TestNonCopy".into()),
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
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
            statements: vec![Statement::Assign(
                Place::local(Local(0)),
                Rvalue::Use(Operand::Move(Place::local(Local(1)))),
            )],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    assert_eq!(errors.len(), 0, "Returning a value should not error");
}


#[test]
fn test_lifetime_use_after_move_str() {
    // RefLifetime is not used here. Use non-copy MirType::Struct to verify
    // existing UseAfterMove logic still works with lifetime-enabled bodies.
    let body = Body {
        name: "test_use_after_move".to_string(),
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
            LocalDecl {
                name: Some("temp".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: false,
                lifetime: None,
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![
                // Move s
                Statement::Assign(
                    Place::local(Local(2)),
                    Rvalue::Use(Operand::Move(Place::local(Local(1)))),
                ),
                // Use s again (error!)
                Statement::Assign(
                    Place::local(Local(2)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec!["a".to_string()], // lifetime_params present but not used
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    assert_eq!(
        errors.len(),
        1,
        "Should detect use-after-move for non-copy type"
    );
    match &errors[0] {
        BorrowError::UseAfterMove { local, .. } => {
            assert_eq!(*local, Local(1));
        }
        _ => panic!("Expected UseAfterMove, got {:?}", errors[0]),
    }
}



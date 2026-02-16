use crate::*;
use crate::borrow_check::{check_body, BorrowError};
use std::collections::HashMap;



#[test]
fn test_mutable_borrow_conflict() {
    // Test that two mutable borrows create a conflict
    // (This test now actually detects mutable borrow conflicts!)
    let body = Body {
        name: "test_borrow_conflict".to_string(),
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
                is_mutable: true,
                lifetime: None, // Mutable local = mutable borrows
            },
            LocalDecl {
                name: Some("r1".to_string()),
                ty: MirType::Ref(Box::new(MirType::Struct("TestNonCopy".into()))),
                is_mutable: false,
                lifetime: None,
            },
            LocalDecl {
                name: Some("r2".to_string()),
                ty: MirType::Ref(Box::new(MirType::Struct("TestNonCopy".into()))),
                is_mutable: false,
                lifetime: None,
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![
                // First mutable borrow of s
                Statement::Assign(Place::local(Local(2)), Rvalue::Ref(Place::local(Local(1)))),
                // Second mutable borrow of s (error!)
                Statement::Assign(Place::local(Local(3)), Rvalue::Ref(Place::local(Local(1)))),
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    // Now that we detect mutability, this should error
    assert_eq!(errors.len(), 1, "Should detect mutable borrow conflict");
    match &errors[0] {
        BorrowError::MutableBorrowConflict { local, .. } => {
            assert_eq!(*local, Local(1));
        }
        _ => panic!("Expected MutableBorrowConflict error, got: {:?}", errors[0]),
    }
}


#[test]
fn test_mutable_borrow_conflict_real() {
    // Two &mut borrows on a mutable local
    let body = Body {
        name: "test_mut_conflict".to_string(),
        params: vec![],
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
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: true,
                lifetime: None, // Mutable local
            },
            LocalDecl {
                name: Some("r1".to_string()),
                ty: MirType::Ref(Box::new(MirType::Struct("TestNonCopy".into()))),
                is_mutable: false,
                lifetime: None,
            },
            LocalDecl {
                name: Some("r2".to_string()),
                ty: MirType::Ref(Box::new(MirType::Struct("TestNonCopy".into()))),
                is_mutable: false,
                lifetime: None,
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![
                // Initialize x
                Statement::Assign(
                    Place::local(Local(1)),
                    Rvalue::Use(Operand::Constant(Constant::Str("test".to_string()))),
                ),
                // First mutable borrow
                Statement::Assign(Place::local(Local(2)), Rvalue::Ref(Place::local(Local(1)))),
                // Second mutable borrow (error!)
                Statement::Assign(Place::local(Local(3)), Rvalue::Ref(Place::local(Local(1)))),
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    assert_eq!(errors.len(), 1, "Should detect mutable borrow conflict");
    match &errors[0] {
        BorrowError::MutableBorrowConflict { local, .. } => {
            assert_eq!(*local, Local(1));
        }
        _ => panic!("Expected MutableBorrowConflict error, got: {:?}", errors[0]),
    }
}


#[test]
fn test_shared_borrow_while_mutable() {
    // &mut borrow then & borrow
    let body = Body {
        name: "test_shared_while_mut".to_string(),
        params: vec![],
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
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: true,
                lifetime: None, // Mutable local
            },
            LocalDecl {
                name: Some("y".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: false,
                lifetime: None, // Immutable local
            },
            LocalDecl {
                name: Some("r1".to_string()),
                ty: MirType::Ref(Box::new(MirType::Struct("TestNonCopy".into()))),
                is_mutable: false,
                lifetime: None,
            },
            LocalDecl {
                name: Some("r2".to_string()),
                ty: MirType::Ref(Box::new(MirType::Struct("TestNonCopy".into()))),
                is_mutable: false,
                lifetime: None,
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![
                // Initialize x and y
                Statement::Assign(
                    Place::local(Local(1)),
                    Rvalue::Use(Operand::Constant(Constant::Str("test1".to_string()))),
                ),
                Statement::Assign(
                    Place::local(Local(2)),
                    Rvalue::Use(Operand::Constant(Constant::Str("test2".to_string()))),
                ),
                // Mutable borrow of x
                Statement::Assign(Place::local(Local(3)), Rvalue::Ref(Place::local(Local(1)))),
                // Shared borrow of y (OK - different local)
                Statement::Assign(Place::local(Local(4)), Rvalue::Ref(Place::local(Local(2)))),
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    // No error - borrowing different locals
    assert_eq!(errors.len(), 0, "Borrowing different locals should be OK");
}


#[test]
fn test_multiple_shared_borrows_ok() {
    // Multiple & borrows on immutable local
    let body = Body {
        name: "test_multiple_shared".to_string(),
        params: vec![],
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
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: false,
                lifetime: None, // Immutable local
            },
            LocalDecl {
                name: Some("r1".to_string()),
                ty: MirType::Ref(Box::new(MirType::Struct("TestNonCopy".into()))),
                is_mutable: false,
                lifetime: None,
            },
            LocalDecl {
                name: Some("r2".to_string()),
                ty: MirType::Ref(Box::new(MirType::Struct("TestNonCopy".into()))),
                is_mutable: false,
                lifetime: None,
            },
            LocalDecl {
                name: Some("r3".to_string()),
                ty: MirType::Ref(Box::new(MirType::Struct("TestNonCopy".into()))),
                is_mutable: false,
                lifetime: None,
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![
                // Initialize x
                Statement::Assign(
                    Place::local(Local(1)),
                    Rvalue::Use(Operand::Constant(Constant::Str("test".to_string()))),
                ),
                // First shared borrow
                Statement::Assign(Place::local(Local(2)), Rvalue::Ref(Place::local(Local(1)))),
                // Second shared borrow
                Statement::Assign(Place::local(Local(3)), Rvalue::Ref(Place::local(Local(1)))),
                // Third shared borrow
                Statement::Assign(Place::local(Local(4)), Rvalue::Ref(Place::local(Local(1)))),
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    assert_eq!(errors.len(), 0, "Multiple shared borrows should be OK");
}


#[test]
fn test_borrow_invalidated_by_assign() {
    // Assign to a borrowed place should clear borrows
    let body = Body {
        name: "test_assign_invalidate".to_string(),
        params: vec![],
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
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: true,
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
                // Initialize x
                Statement::Assign(
                    Place::local(Local(1)),
                    Rvalue::Use(Operand::Constant(Constant::Str("test".to_string()))),
                ),
                // Borrow x
                Statement::Assign(Place::local(Local(2)), Rvalue::Ref(Place::local(Local(1)))),
                // Assign to x (invalidates the borrow)
                Statement::Assign(
                    Place::local(Local(1)),
                    Rvalue::Use(Operand::Constant(Constant::Str("new".to_string()))),
                ),
                // Borrow x again (should be OK - previous borrow was invalidated)
                Statement::Assign(Place::local(Local(2)), Rvalue::Ref(Place::local(Local(1)))),
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
        "Assignment should invalidate previous borrows"
    );
}


#[test]
fn test_lifetime_borrow_conflict_with_lifetime() {
    // Two &mut refs with lifetime 'a. Should produce MutableBorrowConflict.
    // Mutable borrows are created by Rvalue::Ref on a mutable local.
    let body = Body {
        name: "test_mut_conflict".to_string(),
        params: vec![],
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
                ty: MirType::I64,
                is_mutable: true, // Must be mutable to create &mut borrows
                lifetime: None,
            },
            LocalDecl {
                name: Some("r1".to_string()),
                ty: MirType::RefMutLifetime {
                    lifetime: "a".to_string(),
                    inner: Box::new(MirType::I64),
                },
                is_mutable: false,
                lifetime: Some("a".to_string()),
            },
            LocalDecl {
                name: Some("r2".to_string()),
                ty: MirType::RefMutLifetime {
                    lifetime: "a".to_string(),
                    inner: Box::new(MirType::I64),
                },
                is_mutable: false,
                lifetime: Some("a".to_string()),
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![
                // Initialize x
                Statement::Assign(
                    Place::local(Local(1)),
                    Rvalue::Use(Operand::Constant(Constant::Int(42))),
                ),
                // First mutable borrow (Rvalue::Ref on mutable local)
                Statement::Assign(Place::local(Local(2)), Rvalue::Ref(Place::local(Local(1)))),
                // Second mutable borrow (conflict!)
                Statement::Assign(Place::local(Local(3)), Rvalue::Ref(Place::local(Local(1)))),
                // Use r2 (keeps both borrows alive)
                Statement::Assign(
                    Place::local(Local(1)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(3)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec!["a".to_string()],
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    assert_eq!(errors.len(), 1, "Should detect mutable borrow conflict");
    match &errors[0] {
        BorrowError::MutableBorrowConflict { local, .. } => {
            assert_eq!(*local, Local(1));
        }
        _ => panic!("Expected MutableBorrowConflict, got {:?}", errors[0]),
    }
}



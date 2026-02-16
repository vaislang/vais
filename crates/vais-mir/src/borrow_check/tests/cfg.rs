use crate::*;
use crate::borrow_check::{check_body, BorrowError, cfg_successors, cfg_predecessors};
use std::collections::HashMap;



#[test]
fn test_cfg_successors() {
    // Create a body with if-else (SwitchInt)
    let body = Body {
        name: "test_cfg".to_string(),
        params: vec![MirType::I64],
        return_type: MirType::I64,
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::I64,
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("x".to_string()),
                ty: MirType::I64,
                is_mutable: false,
                lifetime: None,
            },
        ],
        basic_blocks: vec![
            // bb0: condition check
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::SwitchInt {
                    discriminant: Operand::Copy(Place::local(Local(1))),
                    targets: vec![(0, BasicBlockId(1))],
                    otherwise: BasicBlockId(2),
                }),
            },
            // bb1: then branch
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(0)),
                    Rvalue::Use(Operand::Constant(Constant::Int(10))),
                )],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb2: else branch
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(0)),
                    Rvalue::Use(Operand::Constant(Constant::Int(20))),
                )],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb3: merge block
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::Return),
            },
        ],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };

    let successors = cfg_successors(&body);

    // bb0 should have successors bb1, bb2
    assert_eq!(successors.get(&BasicBlockId(0)).unwrap().len(), 2);
    assert!(successors
        .get(&BasicBlockId(0))
        .unwrap()
        .contains(&BasicBlockId(1)));
    assert!(successors
        .get(&BasicBlockId(0))
        .unwrap()
        .contains(&BasicBlockId(2)));

    // bb1 should have successor bb3
    assert_eq!(
        successors.get(&BasicBlockId(1)).unwrap(),
        &vec![BasicBlockId(3)]
    );

    // bb2 should have successor bb3
    assert_eq!(
        successors.get(&BasicBlockId(2)).unwrap(),
        &vec![BasicBlockId(3)]
    );

    // bb3 should have no successors (Return)
    assert_eq!(successors.get(&BasicBlockId(3)).unwrap().len(), 0);
}


#[test]
fn test_cfg_predecessors() {
    // Use the same body as test_cfg_successors
    let body = Body {
        name: "test_cfg".to_string(),
        params: vec![MirType::I64],
        return_type: MirType::I64,
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::I64,
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("x".to_string()),
                ty: MirType::I64,
                is_mutable: false,
                lifetime: None,
            },
        ],
        basic_blocks: vec![
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::SwitchInt {
                    discriminant: Operand::Copy(Place::local(Local(1))),
                    targets: vec![(0, BasicBlockId(1))],
                    otherwise: BasicBlockId(2),
                }),
            },
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(0)),
                    Rvalue::Use(Operand::Constant(Constant::Int(10))),
                )],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(0)),
                    Rvalue::Use(Operand::Constant(Constant::Int(20))),
                )],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::Return),
            },
        ],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };

    let predecessors = cfg_predecessors(&body);

    // bb0 should have no predecessors (entry block)
    assert!(predecessors.get(&BasicBlockId(0)).is_none());

    // bb1 should have predecessor bb0
    assert_eq!(
        predecessors.get(&BasicBlockId(1)).unwrap(),
        &vec![BasicBlockId(0)]
    );

    // bb2 should have predecessor bb0
    assert_eq!(
        predecessors.get(&BasicBlockId(2)).unwrap(),
        &vec![BasicBlockId(0)]
    );

    // bb3 should have predecessors bb1 and bb2
    assert_eq!(predecessors.get(&BasicBlockId(3)).unwrap().len(), 2);
    assert!(predecessors
        .get(&BasicBlockId(3))
        .unwrap()
        .contains(&BasicBlockId(1)));
    assert!(predecessors
        .get(&BasicBlockId(3))
        .unwrap()
        .contains(&BasicBlockId(2)));
}


#[test]
fn test_cfg_move_on_one_branch() {
    // Test CFG analysis: move on one branch, both branches join
    // This tests that the join correctly marks the value as moved
    let body = Body {
        name: "test_cfg_move".to_string(),
        params: vec![MirType::I64],
        return_type: MirType::Unit,
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::Unit,
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("cond".to_string()),
                ty: MirType::I64,
                is_mutable: false,
                lifetime: None,
            },
            LocalDecl {
                name: Some("x".to_string()),
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
        basic_blocks: vec![
            // bb0: initialize x and check condition
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(2)),
                    Rvalue::Use(Operand::Constant(Constant::Int(0))),
                )],
                terminator: Some(Terminator::SwitchInt {
                    discriminant: Operand::Copy(Place::local(Local(1))),
                    targets: vec![(1, BasicBlockId(1))],
                    otherwise: BasicBlockId(2),
                }),
            },
            // bb1: move x (then branch)
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(3)),
                    Rvalue::Use(Operand::Move(Place::local(Local(2)))),
                )],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb2: don't touch x (else branch)
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb3: merge point - x should be considered moved (conservative)
            BasicBlock {
                statements: vec![
                    // Try to use x here - should error since it's moved on one path
                    Statement::Assign(
                        Place::local(Local(3)),
                        Rvalue::Use(Operand::Copy(Place::local(Local(2)))),
                    ),
                ],
                terminator: Some(Terminator::Return),
            },
        ],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    // Should detect use-after-move because x is moved in one branch
    assert_eq!(
        errors.len(),
        1,
        "Should detect use-after-move in merge block"
    );
    match &errors[0] {
        BorrowError::UseAfterMove { local, .. } => {
            assert_eq!(*local, Local(2), "Error should be for local 2 (x)");
        }
        _ => panic!("Expected UseAfterMove error, got: {:?}", errors[0]),
    }
}


#[test]
fn test_cfg_loop_fixpoint() {
    // Test that loop analysis reaches fixpoint
    // Create a simple loop that moves a value
    let body = Body {
        name: "test_loop".to_string(),
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
                name: Some("counter".to_string()),
                ty: MirType::I64,
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("x".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: true,
                lifetime: None,
            },
        ],
        basic_blocks: vec![
            // bb0: initialize
            BasicBlock {
                statements: vec![
                    Statement::Assign(
                        Place::local(Local(1)),
                        Rvalue::Use(Operand::Constant(Constant::Int(0))),
                    ),
                    Statement::Assign(
                        Place::local(Local(2)),
                        Rvalue::Use(Operand::Constant(Constant::Str("loop".to_string()))),
                    ),
                ],
                terminator: Some(Terminator::Goto(BasicBlockId(1))),
            },
            // bb1: loop header - check condition
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::SwitchInt {
                    discriminant: Operand::Copy(Place::local(Local(1))),
                    targets: vec![(0, BasicBlockId(2))],
                    otherwise: BasicBlockId(3),
                }),
            },
            // bb2: loop body - increment counter and loop back
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(1)),
                    Rvalue::BinaryOp(
                        BinOp::Add,
                        Operand::Copy(Place::local(Local(1))),
                        Operand::Constant(Constant::Int(1)),
                    ),
                )],
                terminator: Some(Terminator::Goto(BasicBlockId(1))), // Back edge
            },
            // bb3: exit loop
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::Return),
            },
        ],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    // No errors expected - just testing that it terminates
    assert_eq!(
        errors.len(),
        0,
        "Loop analysis should reach fixpoint without errors"
    );
}


#[test]
fn test_cfg_if_else_both_move() {
    // Move the same variable in both branches, then try to use it at merge point
    let body = Body {
        name: "test_both_move".to_string(),
        params: vec![MirType::I64],
        return_type: MirType::Unit,
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::Unit,
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("cond".to_string()),
                ty: MirType::I64,
                is_mutable: false,
                lifetime: None,
            },
            LocalDecl {
                name: Some("x".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("temp1".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("temp2".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: true,
                lifetime: None,
            },
        ],
        basic_blocks: vec![
            // bb0: initialize x
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(2)),
                    Rvalue::Use(Operand::Constant(Constant::Str("test".to_string()))),
                )],
                terminator: Some(Terminator::SwitchInt {
                    discriminant: Operand::Copy(Place::local(Local(1))),
                    targets: vec![(1, BasicBlockId(1))],
                    otherwise: BasicBlockId(2),
                }),
            },
            // bb1: move x to temp1
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(3)),
                    Rvalue::Use(Operand::Move(Place::local(Local(2)))),
                )],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb2: move x to temp2
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(4)),
                    Rvalue::Use(Operand::Move(Place::local(Local(2)))),
                )],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb3: merge - x is moved in both branches (no error if not used)
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::Return),
            },
        ],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };
    let errors = check_body(&body);
    assert_eq!(
        errors.len(),
        0,
        "Moving in both branches without merge use is OK"
    );
}


#[test]
fn test_cfg_if_else_use_after_partial_move() {
    // Move in then branch, not in else, then use at merge → error
    let body = Body {
        name: "test_partial_move".to_string(),
        params: vec![MirType::I64],
        return_type: MirType::Unit,
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::Unit,
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("cond".to_string()),
                ty: MirType::I64,
                is_mutable: false,
                lifetime: None,
            },
            LocalDecl {
                name: Some("x".to_string()),
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
        basic_blocks: vec![
            // bb0: initialize x
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(2)),
                    Rvalue::Use(Operand::Constant(Constant::Int(0))),
                )],
                terminator: Some(Terminator::SwitchInt {
                    discriminant: Operand::Copy(Place::local(Local(1))),
                    targets: vec![(1, BasicBlockId(1))],
                    otherwise: BasicBlockId(2),
                }),
            },
            // bb1: move x
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(3)),
                    Rvalue::Use(Operand::Move(Place::local(Local(2)))),
                )],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb2: don't move x
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb3: merge - use x (should error due to conservative join)
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(3)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(2)))),
                )],
                terminator: Some(Terminator::Return),
            },
        ],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };
    let errors = check_body(&body);
    assert_eq!(errors.len(), 1);
    match &errors[0] {
        BorrowError::UseAfterMove { local, .. } => {
            assert_eq!(*local, Local(2));
        }
        _ => panic!("Expected UseAfterMove"),
    }
}


#[test]
fn test_cfg_if_else_reassign_after_move() {
    // Move in then branch, reassign x → Owned, use at merge OK
    let body = Body {
        name: "test_reassign_merge".to_string(),
        params: vec![MirType::I64],
        return_type: MirType::Unit,
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::Unit,
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("cond".to_string()),
                ty: MirType::I64,
                is_mutable: false,
                lifetime: None,
            },
            LocalDecl {
                name: Some("x".to_string()),
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
        basic_blocks: vec![
            // bb0: initialize x
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(2)),
                    Rvalue::Use(Operand::Constant(Constant::Str("test".to_string()))),
                )],
                terminator: Some(Terminator::SwitchInt {
                    discriminant: Operand::Copy(Place::local(Local(1))),
                    targets: vec![(1, BasicBlockId(1))],
                    otherwise: BasicBlockId(2),
                }),
            },
            // bb1: move x, then reassign it
            BasicBlock {
                statements: vec![
                    Statement::Assign(
                        Place::local(Local(3)),
                        Rvalue::Use(Operand::Move(Place::local(Local(2)))),
                    ),
                    Statement::Assign(
                        Place::local(Local(2)),
                        Rvalue::Use(Operand::Constant(Constant::Str("new".to_string()))),
                    ),
                ],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb2: x remains Owned
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb3: merge - x is Owned in both paths
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(3)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(2)))),
                )],
                terminator: Some(Terminator::Return),
            },
        ],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };
    let errors = check_body(&body);
    assert_eq!(
        errors.len(),
        0,
        "Reassigning after move should make x Owned"
    );
}


#[test]
fn test_cfg_if_else_drop_one_branch() {
    // Drop in then branch, not in else → merge considers Dropped → use → error
    let body = Body {
        name: "test_drop_branch".to_string(),
        params: vec![MirType::I64],
        return_type: MirType::Unit,
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::Unit,
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("cond".to_string()),
                ty: MirType::I64,
                is_mutable: false,
                lifetime: None,
            },
            LocalDecl {
                name: Some("x".to_string()),
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
        basic_blocks: vec![
            // bb0: initialize x
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(2)),
                    Rvalue::Use(Operand::Constant(Constant::Int(0))),
                )],
                terminator: Some(Terminator::SwitchInt {
                    discriminant: Operand::Copy(Place::local(Local(1))),
                    targets: vec![(1, BasicBlockId(1))],
                    otherwise: BasicBlockId(2),
                }),
            },
            // bb1: drop x
            BasicBlock {
                statements: vec![Statement::Drop(Place::local(Local(2)))],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb2: don't drop x
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb3: merge - try to use x → UseAfterFree
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(3)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(2)))),
                )],
                terminator: Some(Terminator::Return),
            },
        ],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };
    let errors = check_body(&body);
    assert_eq!(errors.len(), 1);
    match &errors[0] {
        BorrowError::UseAfterFree { local, .. } => {
            assert_eq!(*local, Local(2));
        }
        _ => panic!("Expected UseAfterFree"),
    }
}


#[test]
fn test_cfg_if_else_borrow_conflict_merge() {
    // Both branches create &mut borrows → merge has both active → new &mut → conflicts with both
    let body = Body {
        name: "test_borrow_merge".to_string(),
        params: vec![MirType::I64],
        return_type: MirType::Unit,
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::Unit,
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("cond".to_string()),
                ty: MirType::I64,
                is_mutable: false,
                lifetime: None,
            },
            LocalDecl {
                name: Some("x".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: true,
                lifetime: None,
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
        basic_blocks: vec![
            // bb0: initialize x
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(2)),
                    Rvalue::Use(Operand::Constant(Constant::Str("test".to_string()))),
                )],
                terminator: Some(Terminator::SwitchInt {
                    discriminant: Operand::Copy(Place::local(Local(1))),
                    targets: vec![(1, BasicBlockId(1))],
                    otherwise: BasicBlockId(2),
                }),
            },
            // bb1: create &mut borrow r1
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(3)),
                    Rvalue::Ref(Place::local(Local(2))),
                )],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb2: create &mut borrow r2
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(4)),
                    Rvalue::Ref(Place::local(Local(2))),
                )],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb3: merge - both borrows active, create r3 → conflict with both previous borrows
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(5)),
                    Rvalue::Ref(Place::local(Local(2))),
                )],
                terminator: Some(Terminator::Return),
            },
        ],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };
    let errors = check_body(&body);
    // Merge combines both borrows from branches, new borrow conflicts with both → 2 errors
    assert_eq!(
        errors.len(),
        2,
        "Should detect conflicts with both merged borrows"
    );
    for err in &errors {
        match err {
            BorrowError::MutableBorrowConflict { local, .. } => {
                assert_eq!(*local, Local(2));
            }
            _ => panic!("Expected MutableBorrowConflict, got: {:?}", err),
        }
    }
}


#[test]
fn test_cfg_diamond_no_error() {
    // Diamond CFG: each branch moves a different variable, merge uses remaining ones OK
    let body = Body {
        name: "test_diamond".to_string(),
        params: vec![MirType::I64],
        return_type: MirType::Unit,
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::Unit,
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("cond".to_string()),
                ty: MirType::I64,
                is_mutable: false,
                lifetime: None,
            },
            LocalDecl {
                name: Some("x".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("y".to_string()),
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
        basic_blocks: vec![
            // bb0: initialize x and y
            BasicBlock {
                statements: vec![
                    Statement::Assign(
                        Place::local(Local(2)),
                        Rvalue::Use(Operand::Constant(Constant::Str("x".to_string()))),
                    ),
                    Statement::Assign(
                        Place::local(Local(3)),
                        Rvalue::Use(Operand::Constant(Constant::Str("y".to_string()))),
                    ),
                ],
                terminator: Some(Terminator::SwitchInt {
                    discriminant: Operand::Copy(Place::local(Local(1))),
                    targets: vec![(1, BasicBlockId(1))],
                    otherwise: BasicBlockId(2),
                }),
            },
            // bb1: move x
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(4)),
                    Rvalue::Use(Operand::Move(Place::local(Local(2)))),
                )],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb2: move y
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(4)),
                    Rvalue::Use(Operand::Move(Place::local(Local(3)))),
                )],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb3: merge - use y in then path (moved in else), x in else path (moved in then)
            // Conservative join will mark both as moved, but we don't use them here
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::Return),
            },
        ],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };
    let errors = check_body(&body);
    // Conservative join marks both x and y as moved/uninitialized, but no use at merge → OK
    assert_eq!(
        errors.len(),
        0,
        "Diamond CFG with different moves per branch OK"
    );
}


#[test]
fn test_cfg_sequential_blocks() {
    // Sequential Goto chain: bb0 → bb1 → bb2, move in bb0, use in bb2 → error
    let body = Body {
        name: "test_sequential".to_string(),
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
                name: Some("temp".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: true,
                lifetime: None,
            },
        ],
        basic_blocks: vec![
            // bb0: initialize and move x
            BasicBlock {
                statements: vec![
                    Statement::Assign(
                        Place::local(Local(1)),
                        Rvalue::Use(Operand::Constant(Constant::Int(0))),
                    ),
                    Statement::Assign(
                        Place::local(Local(2)),
                        Rvalue::Use(Operand::Move(Place::local(Local(1)))),
                    ),
                ],
                terminator: Some(Terminator::Goto(BasicBlockId(1))),
            },
            // bb1: intermediate block
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::Goto(BasicBlockId(2))),
            },
            // bb2: try to use x → UseAfterMove
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(2)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
                )],
                terminator: Some(Terminator::Return),
            },
        ],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };
    let errors = check_body(&body);
    assert_eq!(errors.len(), 1);
    match &errors[0] {
        BorrowError::UseAfterMove { local, .. } => {
            assert_eq!(*local, Local(1));
        }
        _ => panic!("Expected UseAfterMove"),
    }
}


#[test]
fn test_cfg_loop_borrow_conflict() {
    // Loop: each iteration creates &mut borrow → loop header joins active borrows → conflict
    let body = Body {
        name: "test_loop_borrow".to_string(),
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
                name: Some("i".to_string()),
                ty: MirType::I64,
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
        basic_blocks: vec![
            // bb0: init
            BasicBlock {
                statements: vec![
                    Statement::Assign(
                        Place::local(Local(1)),
                        Rvalue::Use(Operand::Constant(Constant::Int(0))),
                    ),
                    Statement::Assign(
                        Place::local(Local(2)),
                        Rvalue::Use(Operand::Constant(Constant::Str("test".to_string()))),
                    ),
                ],
                terminator: Some(Terminator::Goto(BasicBlockId(1))),
            },
            // bb1: loop header
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::SwitchInt {
                    discriminant: Operand::Copy(Place::local(Local(1))),
                    targets: vec![(0, BasicBlockId(2))],
                    otherwise: BasicBlockId(3),
                }),
            },
            // bb2: loop body - create &mut borrow, then loop back
            BasicBlock {
                statements: vec![
                    Statement::Assign(Place::local(Local(3)), Rvalue::Ref(Place::local(Local(2)))),
                    Statement::Assign(
                        Place::local(Local(1)),
                        Rvalue::BinaryOp(
                            BinOp::Add,
                            Operand::Copy(Place::local(Local(1))),
                            Operand::Constant(Constant::Int(1)),
                        ),
                    ),
                ],
                terminator: Some(Terminator::Goto(BasicBlockId(1))),
            },
            // bb3: exit
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::Return),
            },
        ],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };
    let errors = check_body(&body);
    // Loop back-edge brings borrow from previous iteration → conflict on second borrow
    assert_eq!(errors.len(), 1);
    match &errors[0] {
        BorrowError::MutableBorrowConflict { local, .. } => {
            assert_eq!(*local, Local(2));
        }
        _ => panic!("Expected MutableBorrowConflict"),
    }
}


#[test]
fn test_cfg_loop_reassign_ok() {
    // Loop: move x, reassign x → Owned at back-edge → next iteration move OK
    let body = Body {
        name: "test_loop_reassign".to_string(),
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
                name: Some("i".to_string()),
                ty: MirType::I64,
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
                name: Some("temp".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: true,
                lifetime: None,
            },
        ],
        basic_blocks: vec![
            // bb0: init
            BasicBlock {
                statements: vec![
                    Statement::Assign(
                        Place::local(Local(1)),
                        Rvalue::Use(Operand::Constant(Constant::Int(0))),
                    ),
                    Statement::Assign(
                        Place::local(Local(2)),
                        Rvalue::Use(Operand::Constant(Constant::Str("init".to_string()))),
                    ),
                ],
                terminator: Some(Terminator::Goto(BasicBlockId(1))),
            },
            // bb1: loop header
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::SwitchInt {
                    discriminant: Operand::Copy(Place::local(Local(1))),
                    targets: vec![(0, BasicBlockId(2)), (1, BasicBlockId(2))],
                    otherwise: BasicBlockId(3),
                }),
            },
            // bb2: loop body - move x, reassign x, increment i
            BasicBlock {
                statements: vec![
                    Statement::Assign(
                        Place::local(Local(3)),
                        Rvalue::Use(Operand::Move(Place::local(Local(2)))),
                    ),
                    Statement::Assign(
                        Place::local(Local(2)),
                        Rvalue::Use(Operand::Constant(Constant::Str("new".to_string()))),
                    ),
                    Statement::Assign(
                        Place::local(Local(1)),
                        Rvalue::BinaryOp(
                            BinOp::Add,
                            Operand::Copy(Place::local(Local(1))),
                            Operand::Constant(Constant::Int(1)),
                        ),
                    ),
                ],
                terminator: Some(Terminator::Goto(BasicBlockId(1))),
            },
            // bb3: exit
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::Return),
            },
        ],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };
    let errors = check_body(&body);
    assert_eq!(errors.len(), 0, "Loop reassign after move should be OK");
}


#[test]
fn test_cfg_unreachable_branch_no_error() {
    // SwitchInt → then (Goto merge), else (Unreachable) → merge use OK
    let body = Body {
        name: "test_unreachable".to_string(),
        params: vec![MirType::I64],
        return_type: MirType::Unit,
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::Unit,
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("cond".to_string()),
                ty: MirType::I64,
                is_mutable: false,
                lifetime: None,
            },
            LocalDecl {
                name: Some("x".to_string()),
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
        basic_blocks: vec![
            // bb0: initialize x
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(2)),
                    Rvalue::Use(Operand::Constant(Constant::Str("test".to_string()))),
                )],
                terminator: Some(Terminator::SwitchInt {
                    discriminant: Operand::Copy(Place::local(Local(1))),
                    targets: vec![(1, BasicBlockId(1))],
                    otherwise: BasicBlockId(2),
                }),
            },
            // bb1: goto merge
            BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::Goto(BasicBlockId(3))),
            },
            // bb2: unreachable (move x here, but path never taken)
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(3)),
                    Rvalue::Use(Operand::Move(Place::local(Local(2)))),
                )],
                terminator: Some(Terminator::Unreachable),
            },
            // bb3: merge - use x (OK because unreachable branch ignored)
            BasicBlock {
                statements: vec![Statement::Assign(
                    Place::local(Local(3)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(2)))),
                )],
                terminator: Some(Terminator::Return),
            },
        ],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    };
    let errors = check_body(&body);
    // Unreachable branch doesn't contribute to join → x is Owned at merge → OK
    assert_eq!(
        errors.len(),
        0,
        "Unreachable branch should not affect merge"
    );
}

// ======== NLL Tests ========



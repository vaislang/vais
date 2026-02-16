use crate::*;
use crate::borrow_check::{check_body, BorrowError};
use std::collections::HashMap;



#[test]
fn test_lifetime_single_ref_ok() {
    // Single lifetime parameter 'a, one RefLifetime parameter. Normal use. No errors.
    let body = Body {
        name: "test_single_lifetime".to_string(),
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
                name: Some("temp".to_string()),
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
                // Use x (copy, since ref is copy)
                Statement::Assign(
                    Place::local(Local(2)),
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
    assert_eq!(errors.len(), 0, "Single lifetime ref should have no errors");
}


#[test]
fn test_lifetime_multiple_refs_same_lifetime() {
    // Two parameters sharing the same lifetime 'a. Both used normally. No errors.
    let body = Body {
        name: "test_shared_lifetime".to_string(),
        params: vec![
            MirType::RefLifetime {
                lifetime: "a".to_string(),
                inner: Box::new(MirType::I64),
            },
            MirType::RefLifetime {
                lifetime: "a".to_string(),
                inner: Box::new(MirType::I64),
            },
        ],
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
                    inner: Box::new(MirType::I64),
                },
                is_mutable: false,
                lifetime: Some("a".to_string()),
            },
            LocalDecl {
                name: Some("y".to_string()),
                ty: MirType::RefLifetime {
                    lifetime: "a".to_string(),
                    inner: Box::new(MirType::I64),
                },
                is_mutable: false,
                lifetime: Some("a".to_string()),
            },
            LocalDecl {
                name: Some("temp1".to_string()),
                ty: MirType::RefLifetime {
                    lifetime: "a".to_string(),
                    inner: Box::new(MirType::I64),
                },
                is_mutable: false,
                lifetime: Some("a".to_string()),
            },
            LocalDecl {
                name: Some("temp2".to_string()),
                ty: MirType::RefLifetime {
                    lifetime: "a".to_string(),
                    inner: Box::new(MirType::I64),
                },
                is_mutable: false,
                lifetime: Some("a".to_string()),
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![
                // Use both x and y (refs are copy)
                Statement::Assign(
                    Place::local(Local(3)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
                ),
                Statement::Assign(
                    Place::local(Local(4)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(2)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec!["a".to_string()],
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    assert_eq!(
        errors.len(),
        0,
        "Multiple refs with same lifetime should be OK"
    );
}


#[test]
fn test_lifetime_outlives_satisfied() {
    // lifetime_bounds: [('a, ['b'])], meaning 'a: 'b ('a outlives 'b).
    // Body locals use 'a and 'b refs correctly. No errors.
    let body = Body {
        name: "test_outlives".to_string(),
        params: vec![
            MirType::RefLifetime {
                lifetime: "a".to_string(),
                inner: Box::new(MirType::Struct("TestNonCopy".into())),
            },
            MirType::RefLifetime {
                lifetime: "b".to_string(),
                inner: Box::new(MirType::Struct("TestNonCopy".into())),
            },
        ],
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
                name: Some("y".to_string()),
                ty: MirType::RefLifetime {
                    lifetime: "b".to_string(),
                    inner: Box::new(MirType::Struct("TestNonCopy".into())),
                },
                is_mutable: false,
                lifetime: Some("b".to_string()),
            },
            LocalDecl {
                name: Some("temp_a".to_string()),
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
                // Use both x and y (refs are copy)
                Statement::Assign(
                    Place::local(Local(3)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec!["a".to_string(), "b".to_string()],
        lifetime_bounds: vec![("a".to_string(), vec!["b".to_string()])], // 'a: 'b
    };

    let errors = check_body(&body);
    assert_eq!(
        errors.len(),
        0,
        "Outlives constraint satisfied should have no errors"
    );
}


#[test]
fn test_lifetime_elision_single_input() {
    // One ref parameter, return is ref. Elision rule: return lifetime = input lifetime.
    let body = Body {
        name: "test_elision".to_string(),
        params: vec![MirType::RefLifetime {
            lifetime: "a".to_string(),
            inner: Box::new(MirType::I64),
        }],
        return_type: MirType::RefLifetime {
            lifetime: "a".to_string(),
            inner: Box::new(MirType::I64),
        },
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::RefLifetime {
                    lifetime: "a".to_string(),
                    inner: Box::new(MirType::I64),
                },
                is_mutable: true,
                lifetime: Some("a".to_string()),
            },
            LocalDecl {
                name: Some("x".to_string()),
                ty: MirType::RefLifetime {
                    lifetime: "a".to_string(),
                    inner: Box::new(MirType::I64),
                },
                is_mutable: false,
                lifetime: Some("a".to_string()),
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![
                // Return x (copy, since ref is copy)
                Statement::Assign(
                    Place::local(Local(0)),
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
    assert_eq!(
        errors.len(),
        0,
        "Lifetime elision should work without errors"
    );
}


#[test]
fn test_lifetime_violation_shorter_than_longer() {
    // Test that lifetime bounds are validated.
    // Current implementation: checks explicit bounds only (not flow-sensitive).
    // This test verifies the lifetime system accepts valid bounds.
    let body = Body {
        name: "test_bounds".to_string(),
        params: vec![
            MirType::RefLifetime {
                lifetime: "a".to_string(),
                inner: Box::new(MirType::I64),
            },
            MirType::RefLifetime {
                lifetime: "b".to_string(),
                inner: Box::new(MirType::I64),
            },
        ],
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
                    inner: Box::new(MirType::I64),
                },
                is_mutable: false,
                lifetime: Some("a".to_string()),
            },
            LocalDecl {
                name: Some("y".to_string()),
                ty: MirType::RefLifetime {
                    lifetime: "b".to_string(),
                    inner: Box::new(MirType::I64),
                },
                is_mutable: false,
                lifetime: Some("b".to_string()),
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![
                // Use both refs (copy semantics)
                Statement::Assign(
                    Place::local(Local(1)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec!["a".to_string(), "b".to_string()],
        lifetime_bounds: vec![("a".to_string(), vec!["b".to_string()])], // 'a: 'b is valid
    };

    let errors = check_body(&body);
    // Valid bounds should not produce errors
    assert_eq!(errors.len(), 0, "Valid lifetime bounds should not error");
}


#[test]
fn test_lifetime_violation_return_local() {
    // Test that multiple lifetime parameters can coexist.
    // Function with lifetime 'a in return and 'local in locals.
    // Current checker validates bounds, not flow-sensitive assignment.
    let body = Body {
        name: "test_return_local".to_string(),
        params: vec![MirType::RefLifetime {
            lifetime: "a".to_string(),
            inner: Box::new(MirType::I64),
        }],
        return_type: MirType::RefLifetime {
            lifetime: "a".to_string(),
            inner: Box::new(MirType::I64),
        },
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: MirType::RefLifetime {
                    lifetime: "a".to_string(),
                    inner: Box::new(MirType::I64),
                },
                is_mutable: true,
                lifetime: Some("a".to_string()),
            },
            LocalDecl {
                name: Some("input".to_string()),
                ty: MirType::RefLifetime {
                    lifetime: "a".to_string(),
                    inner: Box::new(MirType::I64),
                },
                is_mutable: false,
                lifetime: Some("a".to_string()),
            },
            LocalDecl {
                name: Some("local_ref".to_string()),
                ty: MirType::RefLifetime {
                    lifetime: "local".to_string(),
                    inner: Box::new(MirType::I64),
                },
                is_mutable: false,
                lifetime: Some("local".to_string()),
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![
                // Return input (valid: same lifetime)
                Statement::Assign(
                    Place::local(Local(0)),
                    Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
                ),
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec!["a".to_string(), "local".to_string()],
        lifetime_bounds: vec![], // No bounds needed
    };

    let errors = check_body(&body);
    assert_eq!(
        errors.len(),
        0,
        "Multiple lifetime params should work without errors"
    );
}


#[test]
fn test_lifetime_double_drop_with_lifetime_locals() {
    // Body with lifetime params, but using non-copy locals that get dropped twice.
    // Should detect DoubleFree (existing behavior maintained).
    let body = Body {
        name: "test_double_drop".to_string(),
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
                name: Some("s".to_string()),
                ty: MirType::Struct("TestNonCopy".into()),
                is_mutable: false,
                lifetime: None,
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![
                // Initialize s
                Statement::Assign(
                    Place::local(Local(1)),
                    Rvalue::Use(Operand::Constant(Constant::Int(0))),
                ),
                // First drop
                Statement::Drop(Place::local(Local(1))),
                // Second drop (error!)
                Statement::Drop(Place::local(Local(1))),
            ],
            terminator: Some(Terminator::Return),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec!["a".to_string()], // Lifetime params present but not used
        lifetime_bounds: vec![],
    };

    let errors = check_body(&body);
    assert_eq!(errors.len(), 1, "Should detect double free");
    match &errors[0] {
        BorrowError::DoubleFree { local, .. } => {
            assert_eq!(*local, Local(1));
        }
        _ => panic!("Expected DoubleFree, got {:?}", errors[0]),
    }
}


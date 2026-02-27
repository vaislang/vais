use crate::*;
use std::collections::HashMap;

/// Helper to create a simple body for testing.
pub(super) fn make_test_body(
    ty: MirType,
    statements: Vec<Statement>,
    terminator: Terminator,
) -> Body {
    Body {
        name: "test".to_string(),
        params: vec![ty.clone()],
        return_type: ty.clone(),
        locals: vec![
            LocalDecl {
                name: Some("_ret".to_string()),
                ty: ty.clone(),
                is_mutable: true,
                lifetime: None,
            },
            LocalDecl {
                name: Some("x".to_string()),
                ty,
                is_mutable: false,
                lifetime: None,
            },
        ],
        basic_blocks: vec![BasicBlock {
            statements,
            terminator: Some(terminator),
        }],
        block_names: HashMap::new(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    }
}

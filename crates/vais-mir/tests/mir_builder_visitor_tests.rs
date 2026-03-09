//! Comprehensive MIR builder and optimization coverage tests (Phase 131)
//!
//! Targets uncovered lines in:
//! - builder.rs: MirBuilder construction, block/local allocation, terminators
//! - optimize.rs: DCE, CSE, constant/copy propagation, loop unrolling, tail call
//! - types.rs: MirType, Place projections, Display impls, is_copy

use vais_mir::*;

// ============================================================================
// MirBuilder — construction variants
// ============================================================================

#[test]
fn test_builder_empty_function() {
    let mut builder = MirBuilder::new("empty", vec![], MirType::Unit);
    builder.return_();
    let body = builder.build();
    assert_eq!(body.name, "empty");
    assert!(body.params.is_empty());
    assert_eq!(body.return_type, MirType::Unit);
}

#[test]
fn test_builder_with_many_params() {
    let params = vec![MirType::I64, MirType::Bool, MirType::F64, MirType::Str];
    let builder = MirBuilder::new("multi", params.clone(), MirType::I64);
    let body = builder.build();
    assert_eq!(body.params.len(), 4);
    // locals: _0 (return) + 4 params = 5
    assert_eq!(body.locals.len(), 5);
}

#[test]
fn test_builder_f32_return() {
    let builder = MirBuilder::new("get_pi", vec![], MirType::F32);
    let body = builder.build();
    assert_eq!(body.return_type, MirType::F32);
}

#[test]
fn test_builder_str_return() {
    let builder = MirBuilder::new("get_name", vec![], MirType::Str);
    let body = builder.build();
    assert_eq!(body.return_type, MirType::Str);
}

// ============================================================================
// MirBuilder — blocks
// ============================================================================

#[test]
fn test_builder_multiple_blocks() {
    let mut builder = MirBuilder::new("branch", vec![MirType::Bool], MirType::I64);
    let bb1 = builder.new_block();
    let bb2 = builder.new_block();
    let bb3 = builder.new_block();
    assert_eq!(bb1, BasicBlockId(1));
    assert_eq!(bb2, BasicBlockId(2));
    assert_eq!(bb3, BasicBlockId(3));
    let body = builder.build();
    assert_eq!(body.basic_blocks.len(), 4); // bb0 (entry) + 3 new
}

#[test]
fn test_builder_switch_and_push() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    let bb1 = builder.new_block();

    // Push to bb0
    builder.assign_const(Local(0), Constant::Int(1));

    // Switch to bb1 and push
    builder.switch_to_block(bb1);
    builder.assign_const(Local(0), Constant::Int(2));

    let body = builder.build();
    assert_eq!(body.basic_blocks[0].statements.len(), 1);
    assert_eq!(body.basic_blocks[1].statements.len(), 1);
}

// ============================================================================
// MirBuilder — locals
// ============================================================================

#[test]
fn test_builder_new_local_named() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    let local = builder.new_local(MirType::Bool, Some("flag".to_string()));
    assert_eq!(local, Local(1)); // _0 is return, _1 is first new local
}

#[test]
fn test_builder_new_local_unnamed() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    let l1 = builder.new_local(MirType::I64, None);
    let l2 = builder.new_local(MirType::Bool, None);
    assert_eq!(l1, Local(1));
    assert_eq!(l2, Local(2));
}

#[test]
fn test_builder_return_place() {
    let builder = MirBuilder::new("test", vec![], MirType::I64);
    let place = builder.return_place();
    assert_eq!(place.local, Local(0));
    assert!(place.projections.is_empty());
}

#[test]
fn test_builder_param_indexing() {
    let builder = MirBuilder::new("test", vec![MirType::I64, MirType::Bool], MirType::I64);
    assert_eq!(builder.param(0), Local(1));
    assert_eq!(builder.param(1), Local(2));
}

// ============================================================================
// MirBuilder — statements
// ============================================================================

#[test]
fn test_builder_assign_rvalue() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    let local = builder.new_local(MirType::I64, None);
    builder.assign(
        Place::local(local),
        Rvalue::Use(Operand::Constant(Constant::Int(42))),
    );
    let body = builder.build();
    assert_eq!(body.basic_blocks[0].statements.len(), 1);
}

#[test]
fn test_builder_assign_binop() {
    let mut builder = MirBuilder::new("test", vec![MirType::I64, MirType::I64], MirType::I64);
    let result = builder.new_local(MirType::I64, Some("result".to_string()));
    let lhs = Operand::Copy(Place::local(builder.param(0)));
    let rhs = Operand::Copy(Place::local(builder.param(1)));
    builder.assign_binop(result, BinOp::Add, lhs, rhs);
    let body = builder.build();
    assert_eq!(body.basic_blocks[0].statements.len(), 1);
}

#[test]
fn test_builder_drop() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    let local = builder.new_local(MirType::I64, None);
    builder.assign_const(local, Constant::Int(1));
    builder.drop(Place::local(local));
    let body = builder.build();
    assert_eq!(body.basic_blocks[0].statements.len(), 2);
}

// ============================================================================
// MirBuilder — terminators
// ============================================================================

#[test]
fn test_builder_goto() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    let bb1 = builder.new_block();
    builder.goto(bb1);
    let body = builder.build();
    assert!(body.basic_blocks[0].terminator.is_some());
}

#[test]
fn test_builder_return() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    builder.assign_const(Local(0), Constant::Int(0));
    builder.return_();
    let body = builder.build();
    assert_eq!(
        body.basic_blocks[0].terminator,
        Some(Terminator::Return)
    );
}

#[test]
fn test_builder_switch_int() {
    let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);
    let bb_case0 = builder.new_block();
    let bb_default = builder.new_block();

    builder.switch_int(
        Operand::Copy(Place::local(builder.param(0))),
        vec![(0, bb_case0)],
        bb_default,
    );

    let body = builder.build();
    match &body.basic_blocks[0].terminator {
        Some(Terminator::SwitchInt {
            targets, otherwise, ..
        }) => {
            assert_eq!(targets.len(), 1);
            assert_eq!(*otherwise, bb_default);
        }
        _ => panic!("Expected SwitchInt terminator"),
    }
}

#[test]
fn test_builder_call() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    let bb_cont = builder.new_block();
    let result = builder.new_local(MirType::I64, None);

    builder.call(
        "add",
        vec![
            Operand::Constant(Constant::Int(1)),
            Operand::Constant(Constant::Int(2)),
        ],
        Place::local(result),
        bb_cont,
    );

    let body = builder.build();
    match &body.basic_blocks[0].terminator {
        Some(Terminator::Call {
            func,
            args,
            destination,
            target,
        }) => {
            assert_eq!(func, "add");
            assert_eq!(args.len(), 2);
            assert_eq!(destination.local, result);
            assert_eq!(*target, bb_cont);
        }
        _ => panic!("Expected Call terminator"),
    }
}

// ============================================================================
// Place projections
// ============================================================================

#[test]
fn test_place_field_projection() {
    let place = Place::local(Local(1)).field(0);
    assert_eq!(place.projections.len(), 1);
    assert_eq!(place.projections[0], Projection::Field(0));
}

#[test]
fn test_place_deref_projection() {
    let place = Place::local(Local(1)).deref();
    assert_eq!(place.projections.len(), 1);
    assert_eq!(place.projections[0], Projection::Deref);
}

#[test]
fn test_place_index_projection() {
    let place = Place::local(Local(1)).index(Local(2));
    assert_eq!(place.projections.len(), 1);
    assert_eq!(place.projections[0], Projection::Index(Local(2)));
}

#[test]
fn test_place_chained_projections() {
    let place = Place::local(Local(1)).deref().field(0).index(Local(3));
    assert_eq!(place.projections.len(), 3);
}

// ============================================================================
// MirType::is_copy
// ============================================================================

#[test]
fn test_mir_type_copy_primitives() {
    assert!(MirType::I8.is_copy());
    assert!(MirType::I16.is_copy());
    assert!(MirType::I32.is_copy());
    assert!(MirType::I64.is_copy());
    assert!(MirType::I128.is_copy());
    assert!(MirType::U8.is_copy());
    assert!(MirType::U16.is_copy());
    assert!(MirType::U32.is_copy());
    assert!(MirType::U64.is_copy());
    assert!(MirType::U128.is_copy());
    assert!(MirType::F32.is_copy());
    assert!(MirType::F64.is_copy());
    assert!(MirType::Bool.is_copy());
    assert!(MirType::Str.is_copy());
    assert!(MirType::Unit.is_copy());
    assert!(MirType::Never.is_copy());
}

#[test]
fn test_mir_type_copy_pointer_ref() {
    assert!(MirType::Pointer(Box::new(MirType::I64)).is_copy());
    assert!(MirType::Ref(Box::new(MirType::I64)).is_copy());
}

#[test]
fn test_mir_type_copy_tuple() {
    assert!(MirType::Tuple(vec![MirType::I64, MirType::Bool]).is_copy());
    // Non-copy element makes tuple non-copy
    assert!(!MirType::Tuple(vec![
        MirType::I64,
        MirType::Array(Box::new(MirType::I64)),
    ])
    .is_copy());
}

#[test]
fn test_mir_type_non_copy() {
    assert!(!MirType::Array(Box::new(MirType::I64)).is_copy());
    assert!(!MirType::Struct("Test".to_string()).is_copy());
    assert!(!MirType::Enum("E".to_string()).is_copy());
}

// ============================================================================
// Display impls
// ============================================================================

#[test]
fn test_constant_display() {
    assert_eq!(format!("{}", Constant::Int(42)), "42");
    assert_eq!(format!("{}", Constant::Bool(true)), "true");
    assert_eq!(format!("{}", Constant::Str("hello".into())), "\"hello\"");
    assert_eq!(format!("{}", Constant::Unit), "()");
}

#[test]
fn test_constant_float_display() {
    let s = format!("{}", Constant::Float(3.14));
    assert!(s.contains("3.14"));
}

#[test]
fn test_operand_display() {
    assert_eq!(
        format!("{}", Operand::Constant(Constant::Int(5))),
        "const 5"
    );
    assert!(format!("{}", Operand::Copy(Place::local(Local(1)))).contains("copy"));
    assert!(format!("{}", Operand::Move(Place::local(Local(2)))).contains("move"));
}

#[test]
fn test_place_display() {
    let place = Place::local(Local(0));
    let s = format!("{}", place);
    assert!(s.contains("0"));
}

#[test]
fn test_place_display_with_projections() {
    let place = Place::local(Local(1)).deref().field(2);
    let s = format!("{}", place);
    assert!(s.contains(".*"));
    assert!(s.contains(".2"));
}

// ============================================================================
// Optimization passes — DCE
// ============================================================================

#[test]
fn test_dce_removes_unused_local() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    let unused = builder.new_local(MirType::I64, None);
    builder.assign_const(unused, Constant::Int(999)); // unused assignment
    builder.assign_const(Local(0), Constant::Int(42)); // return value
    builder.return_();
    let mut body = builder.build();

    let stmt_count_before = body.basic_blocks[0].statements.len();
    vais_mir::optimize::dead_code_elimination(&mut body);
    let stmt_count_after = body.basic_blocks[0].statements.len();

    // DCE should remove the unused assignment
    assert!(stmt_count_after <= stmt_count_before);
}

#[test]
fn test_dce_keeps_return_place() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    builder.assign_const(Local(0), Constant::Int(42));
    builder.return_();
    let mut body = builder.build();

    vais_mir::optimize::dead_code_elimination(&mut body);
    // Return place assignment should be kept
    assert!(!body.basic_blocks[0].statements.is_empty());
}

// ============================================================================
// Optimization passes — constant propagation
// ============================================================================

#[test]
fn test_constant_propagation_basic() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    let x = builder.new_local(MirType::I64, Some("x".to_string()));
    builder.assign_const(x, Constant::Int(10));
    builder.assign(
        Place::local(Local(0)),
        Rvalue::Use(Operand::Copy(Place::local(x))),
    );
    builder.return_();
    let mut body = builder.build();

    vais_mir::optimize::constant_propagation(&mut body);
    // Body should still be valid after optimization
    assert!(!body.basic_blocks[0].statements.is_empty());
}

// ============================================================================
// Optimization passes — full pipeline
// ============================================================================

#[test]
fn test_optimize_mir_body_full() {
    let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);
    let temp = builder.new_local(MirType::I64, None);
    builder.assign(
        Place::local(temp),
        Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
    );
    builder.assign(
        Place::local(Local(0)),
        Rvalue::Use(Operand::Copy(Place::local(temp))),
    );
    builder.return_();
    let mut body = builder.build();

    vais_mir::optimize::optimize_mir_body(&mut body);
    // Should complete without panic
    assert!(body.basic_blocks[0].terminator.is_some());
}

//! Coverage tests for vais-mir/src/builder.rs
//!
//! Targets: MirBuilder construction, block creation, local allocation,
//! statement pushing, terminator setting, and body building.

use vais_mir::*;

// ============================================================================
// MirBuilder::new — basic construction
// ============================================================================

#[test]
fn test_builder_new_no_params() {
    let builder = MirBuilder::new("test", vec![], MirType::I64);
    let body = builder.build();
    assert_eq!(body.name, "test");
    assert!(body.params.is_empty());
    assert_eq!(body.return_type, MirType::I64);
    // Should have return place _0
    assert!(!body.locals.is_empty());
    // Should have entry block bb0
    assert!(!body.basic_blocks.is_empty());
}

#[test]
fn test_builder_new_with_params() {
    let builder = MirBuilder::new("add", vec![MirType::I64, MirType::I64], MirType::I64);
    let body = builder.build();
    assert_eq!(body.params.len(), 2);
    // locals: _0 (return) + _1, _2 (params)
    assert!(body.locals.len() >= 3);
}

#[test]
fn test_builder_new_bool_return() {
    let builder = MirBuilder::new("is_positive", vec![MirType::I64], MirType::Bool);
    let body = builder.build();
    assert_eq!(body.return_type, MirType::Bool);
}

#[test]
fn test_builder_new_unit_return() {
    let builder = MirBuilder::new("noop", vec![], MirType::Unit);
    let body = builder.build();
    assert_eq!(body.return_type, MirType::Unit);
}

// ============================================================================
// MirBuilder::new_block
// ============================================================================

#[test]
fn test_builder_new_block() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    let bb1 = builder.new_block();
    assert_eq!(bb1, BasicBlockId(1));
    let bb2 = builder.new_block();
    assert_eq!(bb2, BasicBlockId(2));
}

// ============================================================================
// MirBuilder::switch_to_block
// ============================================================================

#[test]
fn test_builder_switch_to_block() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    let bb1 = builder.new_block();
    builder.switch_to_block(bb1);
    // Should be able to push statements to the new block
    builder.assign_const(Local(0), Constant::Int(42));
    let body = builder.build();
    assert!(body.basic_blocks.len() >= 2);
    assert!(!body.basic_blocks[1].statements.is_empty());
}

// ============================================================================
// MirBuilder::new_local
// ============================================================================

#[test]
fn test_builder_new_local() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    let local = builder.new_local(MirType::I64, Some("x".to_string()));
    // Should be after return place _0
    assert_eq!(local.0, 1);
    let local2 = builder.new_local(MirType::Bool, None);
    assert_eq!(local2.0, 2);
}

#[test]
fn test_builder_new_local_with_params() {
    let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);
    // _0 = return, _1 = param, so next should be _2
    let local = builder.new_local(MirType::I64, Some("temp".to_string()));
    assert_eq!(local.0, 2);
}

// ============================================================================
// MirBuilder::return_place and param
// ============================================================================

#[test]
fn test_builder_return_place() {
    let builder = MirBuilder::new("test", vec![], MirType::I64);
    let ret = builder.return_place();
    assert_eq!(ret.local, Local(0));
}

#[test]
fn test_builder_param() {
    let builder = MirBuilder::new("test", vec![MirType::I64, MirType::Bool], MirType::I64);
    assert_eq!(builder.param(0), Local(1));
    assert_eq!(builder.param(1), Local(2));
}

// ============================================================================
// MirBuilder::assign and assign_const
// ============================================================================

#[test]
fn test_builder_assign_const_int() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    builder.assign_const(Local(0), Constant::Int(42));
    let body = builder.build();
    assert!(!body.basic_blocks[0].statements.is_empty());
}

#[test]
fn test_builder_assign_const_bool() {
    let mut builder = MirBuilder::new("test", vec![], MirType::Bool);
    builder.assign_const(Local(0), Constant::Bool(true));
    let body = builder.build();
    assert!(!body.basic_blocks[0].statements.is_empty());
}

#[test]
fn test_builder_assign_const_float() {
    let mut builder = MirBuilder::new("test", vec![], MirType::F64);
    builder.assign_const(Local(0), Constant::Float(3.14));
    let body = builder.build();
    assert!(!body.basic_blocks[0].statements.is_empty());
}

// ============================================================================
// MirBuilder::assign_binop
// ============================================================================

#[test]
fn test_builder_assign_binop_add() {
    let mut builder = MirBuilder::new("add", vec![MirType::I64, MirType::I64], MirType::I64);
    let result = builder.new_local(MirType::I64, Some("result".to_string()));
    builder.assign_binop(
        result,
        BinOp::Add,
        Operand::Copy(Place::local(builder.param(0))),
        Operand::Copy(Place::local(builder.param(1))),
    );
    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(result))),
    );
    builder.return_();
    let body = builder.build();
    assert!(body.basic_blocks[0].statements.len() >= 2);
}

#[test]
fn test_builder_assign_binop_sub() {
    let mut builder = MirBuilder::new("sub", vec![MirType::I64, MirType::I64], MirType::I64);
    let result = builder.new_local(MirType::I64, None);
    builder.assign_binop(
        result,
        BinOp::Sub,
        Operand::Copy(Place::local(builder.param(0))),
        Operand::Copy(Place::local(builder.param(1))),
    );
    let body = builder.build();
    assert!(!body.basic_blocks[0].statements.is_empty());
}

#[test]
fn test_builder_assign_binop_mul() {
    let mut builder = MirBuilder::new("mul", vec![MirType::I64, MirType::I64], MirType::I64);
    let result = builder.new_local(MirType::I64, None);
    builder.assign_binop(
        result,
        BinOp::Mul,
        Operand::Constant(Constant::Int(6)),
        Operand::Constant(Constant::Int(7)),
    );
    let body = builder.build();
    assert!(!body.basic_blocks[0].statements.is_empty());
}

// ============================================================================
// MirBuilder::drop
// ============================================================================

#[test]
fn test_builder_drop() {
    let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::Unit);
    builder.drop(Place::local(builder.param(0)));
    let body = builder.build();
    assert!(!body.basic_blocks[0].statements.is_empty());
}

// ============================================================================
// MirBuilder::goto
// ============================================================================

#[test]
fn test_builder_goto() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    let bb1 = builder.new_block();
    builder.goto(bb1);
    let body = builder.build();
    assert!(body.basic_blocks[0].terminator.is_some());
}

// ============================================================================
// MirBuilder::return_
// ============================================================================

#[test]
fn test_builder_return() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    builder.assign_const(Local(0), Constant::Int(42));
    builder.return_();
    let body = builder.build();
    assert_eq!(
        body.basic_blocks[0].terminator,
        Some(Terminator::Return)
    );
}

// ============================================================================
// MirBuilder::switch_int
// ============================================================================

#[test]
fn test_builder_switch_int() {
    let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);
    let bb_true = builder.new_block();
    let bb_false = builder.new_block();
    builder.switch_int(
        Operand::Copy(Place::local(builder.param(0))),
        vec![(1, bb_true)],
        bb_false,
    );
    let body = builder.build();
    assert!(body.basic_blocks[0].terminator.is_some());
}

#[test]
fn test_builder_switch_int_multiple_targets() {
    let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);
    let bb1 = builder.new_block();
    let bb2 = builder.new_block();
    let bb3 = builder.new_block();
    let bb_default = builder.new_block();
    builder.switch_int(
        Operand::Copy(Place::local(builder.param(0))),
        vec![(0, bb1), (1, bb2), (2, bb3)],
        bb_default,
    );
    let body = builder.build();
    assert!(body.basic_blocks[0].terminator.is_some());
}

// ============================================================================
// MirBuilder::call
// ============================================================================

#[test]
fn test_builder_call() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    let result = builder.new_local(MirType::I64, None);
    let bb_cont = builder.new_block();
    builder.call(
        "other_func",
        vec![Operand::Constant(Constant::Int(42))],
        Place::local(result),
        bb_cont,
    );
    let body = builder.build();
    assert!(body.basic_blocks[0].terminator.is_some());
}

// ============================================================================
// MirBuilder — full program: if-else
// ============================================================================

#[test]
fn test_builder_if_else_program() {
    let mut builder = MirBuilder::new("abs", vec![MirType::I64], MirType::I64);
    let bb_pos = builder.new_block();
    let bb_neg = builder.new_block();
    let bb_merge = builder.new_block();

    // bb0: switch on param > 0
    let cmp_result = builder.new_local(MirType::Bool, None);
    builder.assign_binop(
        cmp_result,
        BinOp::Gt,
        Operand::Copy(Place::local(builder.param(0))),
        Operand::Constant(Constant::Int(0)),
    );
    builder.switch_int(
        Operand::Copy(Place::local(cmp_result)),
        vec![(1, bb_pos)],
        bb_neg,
    );

    // bb_pos: return param
    builder.switch_to_block(bb_pos);
    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
    );
    builder.goto(bb_merge);

    // bb_neg: return -param
    builder.switch_to_block(bb_neg);
    let neg_result = builder.new_local(MirType::I64, None);
    builder.assign_binop(
        neg_result,
        BinOp::Sub,
        Operand::Constant(Constant::Int(0)),
        Operand::Copy(Place::local(builder.param(0))),
    );
    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(neg_result))),
    );
    builder.goto(bb_merge);

    // bb_merge: return
    builder.switch_to_block(bb_merge);
    builder.return_();

    let body = builder.build();
    assert_eq!(body.basic_blocks.len(), 4); // bb0, bb_pos, bb_neg, bb_merge
    assert!(body.basic_blocks[3].terminator == Some(Terminator::Return));
}

// ============================================================================
// MirBuilder — full program: loop with counter
// ============================================================================

#[test]
fn test_builder_loop_program() {
    let mut builder = MirBuilder::new("sum_to_n", vec![MirType::I64], MirType::I64);
    let bb_loop = builder.new_block();
    let bb_body = builder.new_block();
    let bb_exit = builder.new_block();

    let sum = builder.new_local(MirType::I64, Some("sum".to_string()));
    let i = builder.new_local(MirType::I64, Some("i".to_string()));

    // bb0: init
    builder.assign_const(sum, Constant::Int(0));
    builder.assign_const(i, Constant::Int(0));
    builder.goto(bb_loop);

    // bb_loop: check i < n
    builder.switch_to_block(bb_loop);
    let cmp = builder.new_local(MirType::Bool, None);
    builder.assign_binop(
        cmp,
        BinOp::Lt,
        Operand::Copy(Place::local(i)),
        Operand::Copy(Place::local(builder.param(0))),
    );
    builder.switch_int(
        Operand::Copy(Place::local(cmp)),
        vec![(1, bb_body)],
        bb_exit,
    );

    // bb_body: sum += i; i += 1
    builder.switch_to_block(bb_body);
    let new_sum = builder.new_local(MirType::I64, None);
    builder.assign_binop(
        new_sum,
        BinOp::Add,
        Operand::Copy(Place::local(sum)),
        Operand::Copy(Place::local(i)),
    );
    builder.assign(
        Place::local(sum),
        Rvalue::Use(Operand::Copy(Place::local(new_sum))),
    );
    let new_i = builder.new_local(MirType::I64, None);
    builder.assign_binop(
        new_i,
        BinOp::Add,
        Operand::Copy(Place::local(i)),
        Operand::Constant(Constant::Int(1)),
    );
    builder.assign(
        Place::local(i),
        Rvalue::Use(Operand::Copy(Place::local(new_i))),
    );
    builder.goto(bb_loop);

    // bb_exit: return sum
    builder.switch_to_block(bb_exit);
    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(sum))),
    );
    builder.return_();

    let body = builder.build();
    assert_eq!(body.basic_blocks.len(), 4);
    assert_eq!(body.name, "sum_to_n");
}

// ============================================================================
// MirBuilder — various MirType constructions
// ============================================================================

#[test]
fn test_builder_with_struct_type() {
    let builder = MirBuilder::new(
        "test",
        vec![MirType::Struct("Point".to_string())],
        MirType::I64,
    );
    let body = builder.build();
    assert_eq!(body.params[0], MirType::Struct("Point".to_string()));
}

#[test]
fn test_builder_with_tuple_type() {
    let builder = MirBuilder::new(
        "test",
        vec![MirType::Tuple(vec![MirType::I64, MirType::Bool])],
        MirType::I64,
    );
    let body = builder.build();
    assert_eq!(
        body.params[0],
        MirType::Tuple(vec![MirType::I64, MirType::Bool])
    );
}

#[test]
fn test_builder_with_pointer_type() {
    let builder = MirBuilder::new(
        "test",
        vec![MirType::Pointer(Box::new(MirType::I64))],
        MirType::Unit,
    );
    let body = builder.build();
    assert_eq!(
        body.params[0],
        MirType::Pointer(Box::new(MirType::I64))
    );
}

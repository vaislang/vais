use super::*;
use crate::MirBuilder;

#[test]
fn test_dce_removes_unused_locals() {
    let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);

    let unused = builder.new_local(MirType::I64, Some("unused".into()));
    let used = builder.new_local(MirType::I64, Some("used".into()));

    // Assign to unused (should be removed)
    builder.assign_const(unused, Constant::Int(42));
    // Assign to used
    builder.assign_const(used, Constant::Int(10));
    // Return place uses 'used'
    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(used))),
    );
    builder.return_();

    let mut body = builder.build();
    dead_code_elimination(&mut body);

    // The unused assignment should be removed
    // Entry block should have 2 statements: assign to 'used' + assign to return place
    assert_eq!(body.basic_blocks[0].statements.len(), 2);
}

#[test]
fn test_cse_eliminates_duplicate_binop() {
    let mut builder = MirBuilder::new("test", vec![MirType::I64, MirType::I64], MirType::I64);

    let t1 = builder.new_local(MirType::I64, None);
    let t2 = builder.new_local(MirType::I64, None);

    let param_a = Operand::Copy(Place::local(builder.param(0)));
    let param_b = Operand::Copy(Place::local(builder.param(1)));

    // Same binary op twice
    builder.assign_binop(t1, BinOp::Add, param_a.clone(), param_b.clone());
    builder.assign_binop(t2, BinOp::Add, param_a, param_b);

    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(t2))),
    );
    builder.return_();

    let mut body = builder.build();
    common_subexpression_elimination(&mut body);

    // The second assignment should now be a copy of t1
    if let Statement::Assign(_, Rvalue::Use(Operand::Copy(place))) =
        &body.basic_blocks[0].statements[1]
    {
        assert_eq!(place.local, t1);
    } else {
        panic!("Expected CSE to replace with copy");
    }
}

#[test]
fn test_constant_propagation() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);

    let c = builder.new_local(MirType::I64, None);
    let result = builder.new_local(MirType::I64, None);

    builder.assign_const(c, Constant::Int(42));
    builder.assign(
        Place::local(result),
        Rvalue::BinaryOp(
            BinOp::Add,
            Operand::Copy(Place::local(c)),
            Operand::Constant(Constant::Int(1)),
        ),
    );
    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(result))),
    );
    builder.return_();

    let mut body = builder.build();
    constant_propagation(&mut body);

    // The use of 'c' in the binop should be replaced with const 42
    if let Statement::Assign(_, Rvalue::BinaryOp(_, lhs, _)) = &body.basic_blocks[0].statements[1] {
        assert_eq!(*lhs, Operand::Constant(Constant::Int(42)));
    } else {
        panic!("Expected constant propagation");
    }
}

#[test]
fn test_unreachable_block_removal() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);

    let bb1 = builder.new_block();
    let _bb2 = builder.new_block(); // unreachable

    builder.assign_const(Local(0), Constant::Int(0));
    builder.goto(bb1);

    builder.switch_to_block(bb1);
    builder.return_();

    // bb2 is unreachable (no goto/switch targets it)
    builder.switch_to_block(_bb2);
    builder.assign_const(Local(0), Constant::Int(99));
    builder.return_();

    let mut body = builder.build();
    remove_unreachable_blocks(&mut body);

    // bb2 should be replaced with unreachable
    assert_eq!(
        body.basic_blocks[2].terminator,
        Some(Terminator::Unreachable)
    );
    assert!(body.basic_blocks[2].statements.is_empty());
}

#[test]
fn test_constant_folding_with_binop() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);

    let c1 = builder.new_local(MirType::I64, None);
    let c2 = builder.new_local(MirType::I64, None);
    let result = builder.new_local(MirType::I64, None);

    // c1 = 10, c2 = 20
    builder.assign_const(c1, Constant::Int(10));
    builder.assign_const(c2, Constant::Int(20));

    // result = c1 + c2 (should propagate to const 10 + const 20)
    builder.assign(
        Place::local(result),
        Rvalue::BinaryOp(
            BinOp::Add,
            Operand::Copy(Place::local(c1)),
            Operand::Copy(Place::local(c2)),
        ),
    );
    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(result))),
    );
    builder.return_();

    let mut body = builder.build();
    constant_propagation(&mut body);

    // After propagation, the binop should use const operands
    if let Statement::Assign(_, Rvalue::BinaryOp(_, lhs, rhs)) = &body.basic_blocks[0].statements[2]
    {
        assert_eq!(*lhs, Operand::Constant(Constant::Int(10)));
        assert_eq!(*rhs, Operand::Constant(Constant::Int(20)));
    } else {
        panic!("Expected binop with constant operands");
    }
}

#[test]
fn test_dce_preserves_return_place() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);

    let temp = builder.new_local(MirType::I64, None);

    // temp = 100 (unused)
    builder.assign_const(temp, Constant::Int(100));
    // _0 = 42 (return place, must be kept)
    builder.assign_const(Local(0), Constant::Int(42));
    builder.return_();

    let mut body = builder.build();
    dead_code_elimination(&mut body);

    // Return place assignment must remain
    let has_return_assignment = body.basic_blocks[0]
        .statements
        .iter()
        .any(|stmt| matches!(stmt, Statement::Assign(place, _) if place.local.0 == 0));
    assert!(has_return_assignment);
}

#[test]
fn test_cse_does_not_eliminate_different_ops() {
    let mut builder = MirBuilder::new("test", vec![MirType::I64, MirType::I64], MirType::I64);

    let t1 = builder.new_local(MirType::I64, None);
    let t2 = builder.new_local(MirType::I64, None);

    let param_a = Operand::Copy(Place::local(builder.param(0)));
    let param_b = Operand::Copy(Place::local(builder.param(1)));

    // t1 = a + b
    builder.assign_binop(t1, BinOp::Add, param_a.clone(), param_b.clone());
    // t2 = a * b (different op, should NOT be eliminated)
    builder.assign_binop(t2, BinOp::Mul, param_a, param_b);

    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(t2))),
    );
    builder.return_();

    let mut body = builder.build();
    common_subexpression_elimination(&mut body);

    // t2 assignment should still be Mul, not replaced
    if let Statement::Assign(_, Rvalue::BinaryOp(op, _, _)) = &body.basic_blocks[0].statements[1] {
        assert_eq!(*op, BinOp::Mul);
    } else {
        panic!("Expected Mul to remain");
    }
}

#[test]
fn test_optimize_mir_body_integration() {
    let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);

    let unused = builder.new_local(MirType::I64, None);
    let const_val = builder.new_local(MirType::I64, None);
    let result = builder.new_local(MirType::I64, None);

    // unused = 999 (should be removed by DCE)
    builder.assign_const(unused, Constant::Int(999));
    // const_val = 5
    builder.assign_const(const_val, Constant::Int(5));
    // result = param + const_val (should propagate const 5)
    builder.assign(
        Place::local(result),
        Rvalue::BinaryOp(
            BinOp::Add,
            Operand::Copy(Place::local(builder.param(0))),
            Operand::Copy(Place::local(const_val)),
        ),
    );
    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(result))),
    );
    builder.return_();

    let mut body = builder.build();
    let before_count = body.basic_blocks[0].statements.len();

    optimize_mir_body(&mut body);

    let after_count = body.basic_blocks[0].statements.len();
    // Should have fewer statements after DCE removes unused
    assert!(after_count < before_count);
}

#[test]
fn test_switch_int_with_multiple_targets() {
    let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);

    let bb1 = builder.new_block();
    let bb2 = builder.new_block();
    let bb_default = builder.new_block();
    let bb_end = builder.new_block();

    let param = Operand::Copy(Place::local(builder.param(0)));

    // Switch on parameter with 3 cases
    builder.switch_int(param, vec![(1, bb1), (2, bb2)], bb_default);

    // bb1: return 100
    builder.switch_to_block(bb1);
    builder.assign_const(builder.return_place().local, Constant::Int(100));
    builder.goto(bb_end);

    // bb2: return 200
    builder.switch_to_block(bb2);
    builder.assign_const(builder.return_place().local, Constant::Int(200));
    builder.goto(bb_end);

    // bb_default: return 0
    builder.switch_to_block(bb_default);
    builder.assign_const(builder.return_place().local, Constant::Int(0));
    builder.goto(bb_end);

    // bb_end: return
    builder.switch_to_block(bb_end);
    builder.return_();

    let mut body = builder.build();
    remove_unreachable_blocks(&mut body);

    // All blocks should be reachable
    for bb in &body.basic_blocks {
        assert_ne!(bb.terminator, Some(Terminator::Unreachable));
    }
}

// ======================================================================
// Copy Propagation Tests
// ======================================================================

#[test]
fn test_copy_propagation_simple() {
    let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);

    let param0 = builder.param(0);
    let x = builder.new_local(MirType::I64, Some("x".into()));
    let y = builder.new_local(MirType::I64, Some("y".into()));

    // x = copy param0
    builder.assign(
        Place::local(x),
        Rvalue::Use(Operand::Copy(Place::local(param0))),
    );
    // y = copy x  (should be propagated to: y = copy param0)
    builder.assign(Place::local(y), Rvalue::Use(Operand::Copy(Place::local(x))));
    // _0 = copy y
    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(y))),
    );
    builder.return_();

    let mut body = builder.build();
    copy_propagation(&mut body);

    // After copy propagation, y's assignment should reference param0
    // and return should reference param0 (through y -> x -> param0 chain)
    if let Statement::Assign(_, Rvalue::Use(Operand::Copy(place))) =
        &body.basic_blocks[0].statements[2]
    {
        // _0 should now reference param0 (Local(1))
        assert_eq!(place.local, param0);
    } else {
        panic!("Expected copy propagation to resolve chain");
    }
}

#[test]
fn test_copy_propagation_no_multi_assign() {
    let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);

    let x = builder.new_local(MirType::I64, Some("x".into()));

    // x = copy param0
    builder.assign(
        Place::local(x),
        Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
    );
    // x = const 42 (reassigned -- should not be propagated)
    builder.assign_const(x, Constant::Int(42));
    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(x))),
    );
    builder.return_();

    let mut body = builder.build();
    copy_propagation(&mut body);

    // x should NOT be propagated since it's assigned more than once
    if let Statement::Assign(_, Rvalue::Use(Operand::Copy(place))) =
        &body.basic_blocks[0].statements[2]
    {
        assert_eq!(place.local, x); // Still x, not param0
    } else {
        panic!("Expected x to remain (multi-assign prevents propagation)");
    }
}

// ======================================================================
// Tail Call Detection Tests
// ======================================================================

#[test]
fn test_tail_call_detection() {
    let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);

    let bb_ret = builder.new_block();

    // bb0: call foo(param0), result -> _0, then goto bb_ret
    builder.call(
        "foo",
        vec![Operand::Copy(Place::local(builder.param(0)))],
        Place::local(Local(0)), // destination = _0
        bb_ret,
    );

    // bb_ret: just return
    builder.switch_to_block(bb_ret);
    builder.return_();

    let mut body = builder.build();
    tail_call_detection(&mut body);

    // The call should be converted to a TailCall
    assert!(matches!(
        body.basic_blocks[0].terminator,
        Some(Terminator::TailCall { .. })
    ));
}

#[test]
fn test_tail_call_not_applied_when_not_tail() {
    let mut builder = MirBuilder::new("test", vec![MirType::I64], MirType::I64);

    let result = builder.new_local(MirType::I64, None);
    let bb_after = builder.new_block();

    // bb0: call foo(param0), result -> _3 (NOT _0), then goto bb_after
    builder.call(
        "foo",
        vec![Operand::Copy(Place::local(builder.param(0)))],
        Place::local(result), // NOT return place
        bb_after,
    );

    // bb_after: use result, then return
    builder.switch_to_block(bb_after);
    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(result))),
    );
    builder.return_();

    let mut body = builder.build();
    tail_call_detection(&mut body);

    // The call should NOT be converted (destination is not _0)
    assert!(matches!(
        body.basic_blocks[0].terminator,
        Some(Terminator::Call { .. })
    ));
}

// ======================================================================
// Escape Analysis Tests
// ======================================================================

#[test]
fn test_escape_analysis_non_escaping() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);

    let alloc_result = builder.new_local(MirType::I64, None);
    let bb_after = builder.new_block();

    // Call malloc (heap allocation)
    builder.call(
        "malloc",
        vec![Operand::Constant(Constant::Int(64))],
        Place::local(alloc_result),
        bb_after,
    );

    // bb_after: just return a constant (alloc_result doesn't escape)
    builder.switch_to_block(bb_after);
    builder.assign_const(Local(0), Constant::Int(42));
    builder.return_();

    let mut body = builder.build();
    escape_analysis(&mut body);

    // alloc_result should be marked as non-escaping
    let key = format!("__escape_local_{}", alloc_result.0);
    assert!(
        body.block_names.contains_key(&key),
        "Expected non-escaping allocation to be marked"
    );
}

#[test]
fn test_escape_analysis_escaping_via_return() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);

    let alloc_result = builder.new_local(MirType::I64, None);
    let bb_after = builder.new_block();

    // Call malloc
    builder.call(
        "malloc",
        vec![Operand::Constant(Constant::Int(64))],
        Place::local(alloc_result),
        bb_after,
    );

    // bb_after: return the allocated pointer (escapes!)
    builder.switch_to_block(bb_after);
    builder.assign(
        Place::local(Local(0)),
        Rvalue::Use(Operand::Copy(Place::local(alloc_result))),
    );
    builder.return_();

    let mut body = builder.build();
    escape_analysis(&mut body);

    // alloc_result should NOT be marked (it escapes via return)
    let key = format!("__escape_local_{}", alloc_result.0);
    assert!(
        !body.block_names.contains_key(&key),
        "Expected escaping allocation to NOT be marked"
    );
}

// ======================================================================
// Loop Unrolling Tests
// ======================================================================

#[test]
fn test_loop_unrolling_simple() {
    // Build: loop header checks i < 4, body increments i
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);

    let i_var = builder.new_local(MirType::I64, Some("i".into()));
    let cond = builder.new_local(MirType::Bool, Some("cond".into()));
    let sum = builder.new_local(MirType::I64, Some("sum".into()));

    let bb_header = builder.new_block();
    let bb_body = builder.new_block();
    let bb_exit = builder.new_block();

    // bb0: init i=0, sum=0, goto header
    builder.assign_const(i_var, Constant::Int(0));
    builder.assign_const(sum, Constant::Int(0));
    builder.goto(bb_header);

    // bb_header: cond = i < 4; switchInt(cond) -> [1: bb_body], otherwise: bb_exit
    builder.switch_to_block(bb_header);
    builder.assign(
        Place::local(cond),
        Rvalue::BinaryOp(
            BinOp::Lt,
            Operand::Copy(Place::local(i_var)),
            Operand::Constant(Constant::Int(4)),
        ),
    );
    builder.switch_int(
        Operand::Copy(Place::local(cond)),
        vec![(1, bb_body)],
        bb_exit,
    );

    // bb_body: sum = sum + 1; i = i + 1; goto header
    builder.switch_to_block(bb_body);
    builder.assign(
        Place::local(sum),
        Rvalue::BinaryOp(
            BinOp::Add,
            Operand::Copy(Place::local(sum)),
            Operand::Constant(Constant::Int(1)),
        ),
    );
    builder.assign(
        Place::local(i_var),
        Rvalue::BinaryOp(
            BinOp::Add,
            Operand::Copy(Place::local(i_var)),
            Operand::Constant(Constant::Int(1)),
        ),
    );
    builder.goto(bb_header);

    // bb_exit: return sum
    builder.switch_to_block(bb_exit);
    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(sum))),
    );
    builder.return_();

    let mut body = builder.build();
    let before_body_stmts = body.basic_blocks[bb_body.0 as usize].statements.len();
    assert_eq!(before_body_stmts, 2); // sum += 1, i += 1

    loop_unrolling(&mut body);

    // After unrolling with trip count 4, the header block should contain
    // the original header stmts + 4 copies of the body stmts (2 each = 8)
    // and the body block should be replaced with unreachable
    let header_stmts = body.basic_blocks[bb_header.0 as usize].statements.len();
    // 1 (original cond) + 4*2 (unrolled body) = 9
    assert_eq!(header_stmts, 9);
    assert_eq!(
        body.basic_blocks[bb_body.0 as usize].terminator,
        Some(Terminator::Unreachable)
    );
}

#[test]
fn test_loop_unrolling_too_large_trip_count() {
    // Loop with trip count > 8 should NOT be unrolled
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);

    let i_var = builder.new_local(MirType::I64, None);
    let cond = builder.new_local(MirType::Bool, None);

    let bb_header = builder.new_block();
    let bb_body = builder.new_block();
    let bb_exit = builder.new_block();

    builder.assign_const(i_var, Constant::Int(0));
    builder.goto(bb_header);

    builder.switch_to_block(bb_header);
    builder.assign(
        Place::local(cond),
        Rvalue::BinaryOp(
            BinOp::Lt,
            Operand::Copy(Place::local(i_var)),
            Operand::Constant(Constant::Int(100)), // too large
        ),
    );
    builder.switch_int(
        Operand::Copy(Place::local(cond)),
        vec![(1, bb_body)],
        bb_exit,
    );

    builder.switch_to_block(bb_body);
    builder.assign(
        Place::local(i_var),
        Rvalue::BinaryOp(
            BinOp::Add,
            Operand::Copy(Place::local(i_var)),
            Operand::Constant(Constant::Int(1)),
        ),
    );
    builder.goto(bb_header);

    builder.switch_to_block(bb_exit);
    builder.assign_const(Local(0), Constant::Int(0));
    builder.return_();

    let mut body = builder.build();
    loop_unrolling(&mut body);

    // Body block should still have its original terminator (not unreachable)
    assert!(matches!(
        body.basic_blocks[bb_body.0 as usize].terminator,
        Some(Terminator::Goto(_))
    ));
}

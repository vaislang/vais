//! Tests for MIR types and builder.

use super::*;

#[test]
fn test_local_display() {
    assert_eq!(format!("{}", Local(0)), "_0");
    assert_eq!(format!("{}", Local(5)), "_5");
}

#[test]
fn test_basic_block_id_display() {
    assert_eq!(format!("{}", BasicBlockId(0)), "bb0");
    assert_eq!(format!("{}", BasicBlockId(3)), "bb3");
}

#[test]
fn test_place_display() {
    let place = Place::local(Local(1));
    assert_eq!(format!("{}", place), "_1");

    let place = Place::local(Local(2)).field(0).field(1);
    assert_eq!(format!("{}", place), "_2.0.1");

    let place = Place::local(Local(3)).deref();
    assert_eq!(format!("{}", place), "_3.*");
}

#[test]
fn test_constant_display() {
    assert_eq!(format!("{}", Constant::Int(42)), "42");
    assert_eq!(format!("{}", Constant::Bool(true)), "true");
    assert_eq!(format!("{}", Constant::Str("hello".into())), "\"hello\"");
    assert_eq!(format!("{}", Constant::Unit), "()");
}

#[test]
fn test_operand_display() {
    let op = Operand::Constant(Constant::Int(10));
    assert_eq!(format!("{}", op), "const 10");

    let op = Operand::Copy(Place::local(Local(1)));
    assert_eq!(format!("{}", op), "copy _1");

    let op = Operand::Move(Place::local(Local(2)));
    assert_eq!(format!("{}", op), "move _2");
}

#[test]
fn test_builder_simple_function() {
    // Build: fn answer() -> i64 { 42 }
    let mut b = MirBuilder::new("answer", vec![], MirType::I64);

    // _0 = const 42
    b.assign_const(Local(0), Constant::Int(42));
    b.return_();

    let body = b.build();
    assert_eq!(body.name, "answer");
    assert_eq!(body.basic_blocks.len(), 1);
    assert_eq!(body.locals.len(), 1); // just _0 (return place)

    let display = body.display();
    assert!(display.contains("fn answer()"));
    assert!(display.contains("return"));
}

#[test]
fn test_builder_with_params() {
    // Build: fn add(a: i64, b: i64) -> i64 { a + b }
    let mut b = MirBuilder::new("add", vec![MirType::I64, MirType::I64], MirType::I64);

    // _0 = Add(_1, _2)
    b.assign_binop(
        Local(0),
        BinOp::Add,
        Operand::Copy(Place::local(b.param(0))),
        Operand::Copy(Place::local(b.param(1))),
    );
    b.return_();

    let body = b.build();
    assert_eq!(body.params.len(), 2);
    assert_eq!(body.locals.len(), 3); // _0 (return) + _1, _2 (params)

    let bb0 = &body.basic_blocks[0];
    assert_eq!(bb0.statements.len(), 1);
    assert!(matches!(bb0.terminator, Some(Terminator::Return)));
}

#[test]
fn test_builder_control_flow() {
    // Build: fn abs(x: i64) -> i64 { if x < 0 { -x } else { x } }
    let mut b = MirBuilder::new("abs", vec![MirType::I64], MirType::I64);

    let bb_then = b.new_block();
    let bb_else = b.new_block();
    let bb_merge = b.new_block();

    // bb0: check condition
    let cond = b.new_local(MirType::Bool, Some("cond".into()));
    b.assign_binop(
        cond,
        BinOp::Lt,
        Operand::Copy(Place::local(b.param(0))),
        Operand::Constant(Constant::Int(0)),
    );
    b.switch_int(
        Operand::Copy(Place::local(cond)),
        vec![(1, bb_then)],
        bb_else,
    );

    // bb1 (then): _0 = -x
    b.switch_to_block(bb_then);
    b.assign(
        Place::local(Local(0)),
        Rvalue::UnaryOp(UnOp::Neg, Operand::Copy(Place::local(b.param(0)))),
    );
    b.goto(bb_merge);

    // bb2 (else): _0 = x
    b.switch_to_block(bb_else);
    b.assign(
        Place::local(Local(0)),
        Rvalue::Use(Operand::Copy(Place::local(b.param(0)))),
    );
    b.goto(bb_merge);

    // bb3 (merge): return
    b.switch_to_block(bb_merge);
    b.return_();

    let body = b.build();
    assert_eq!(body.basic_blocks.len(), 4);

    let display = body.display();
    assert!(display.contains("fn abs("));
    assert!(display.contains("switchInt("));
    assert!(display.contains("goto ->"));
    assert!(display.contains("return"));
}

#[test]
fn test_builder_function_call() {
    // Build: fn caller() -> i64 { add(10, 20) }
    let mut b = MirBuilder::new("caller", vec![], MirType::I64);

    let bb_after = b.new_block();

    b.call(
        "add",
        vec![
            Operand::Constant(Constant::Int(10)),
            Operand::Constant(Constant::Int(20)),
        ],
        Place::local(Local(0)),
        bb_after,
    );

    b.switch_to_block(bb_after);
    b.return_();

    let body = b.build();
    assert_eq!(body.basic_blocks.len(), 2);

    let display = body.display();
    assert!(display.contains("add("));
}

#[test]
fn test_mir_module() {
    let mut module = MirModule::new("test");
    module.structs.insert(
        "Point".into(),
        vec![("x".into(), MirType::I64), ("y".into(), MirType::I64)],
    );

    let mut b = MirBuilder::new("origin", vec![], MirType::Struct("Point".into()));
    let result = b.new_local(MirType::Struct("Point".into()), Some("point".into()));
    b.assign(
        Place::local(result),
        Rvalue::Aggregate(
            AggregateKind::Struct("Point".into()),
            vec![
                Operand::Constant(Constant::Int(0)),
                Operand::Constant(Constant::Int(0)),
            ],
        ),
    );
    b.assign(
        Place::local(Local(0)),
        Rvalue::Use(Operand::Move(Place::local(result))),
    );
    b.return_();

    module.bodies.push(b.build());

    let display = module.display();
    assert!(display.contains("MIR module: test"));
    assert!(display.contains("struct Point"));
    assert!(display.contains("fn origin()"));
}

#[test]
fn test_drop_statement() {
    let mut b = MirBuilder::new("test_drop", vec![MirType::Str], MirType::Unit);
    b.drop(Place::local(b.param(0)));
    b.assign_const(Local(0), Constant::Unit);
    b.return_();

    let body = b.build();
    let bb0 = &body.basic_blocks[0];
    assert_eq!(bb0.statements.len(), 2);
    assert!(matches!(&bb0.statements[0], Statement::Drop(_)));

    let display = body.display();
    assert!(display.contains("drop("));
}

#[test]
fn test_terminator_assert() {
    let mut b = MirBuilder::new("safe_div", vec![MirType::I64, MirType::I64], MirType::I64);
    let bb_ok = b.new_block();

    // Assert divisor != 0
    let cond = b.new_local(MirType::Bool, None);
    b.assign_binop(
        cond,
        BinOp::Ne,
        Operand::Copy(Place::local(b.param(1))),
        Operand::Constant(Constant::Int(0)),
    );
    b.terminate(Terminator::Assert {
        cond: Operand::Copy(Place::local(cond)),
        expected: true,
        msg: "division by zero".into(),
        target: bb_ok,
    });

    b.switch_to_block(bb_ok);
    b.assign_binop(
        Local(0),
        BinOp::Div,
        Operand::Copy(Place::local(b.param(0))),
        Operand::Copy(Place::local(b.param(1))),
    );
    b.return_();

    let body = b.build();
    let display = body.display();
    assert!(display.contains("assert("));
    assert!(display.contains("division by zero"));
}

// ==========================================================================
// Cross-verify tests: optimization before/after comparison
// ==========================================================================

#[test]
fn test_cross_verify_constprop_then_dce() {
    // Verify that constant propagation enables DCE to remove more statements.
    // After constprop, the local holding the constant becomes unused
    // (its value is inlined), so DCE can remove it.
    let mut b = MirBuilder::new("test", vec![], MirType::I64);

    let c = b.new_local(MirType::I64, None);
    let result = b.new_local(MirType::I64, None);

    // c = 42
    b.assign_const(c, Constant::Int(42));
    // result = c + 1
    b.assign(
        Place::local(result),
        Rvalue::BinaryOp(
            BinOp::Add,
            Operand::Copy(Place::local(c)),
            Operand::Constant(Constant::Int(1)),
        ),
    );
    // _0 = result
    b.assign(
        b.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(result))),
    );
    b.return_();

    let mut body = b.build();
    let before_count = body.basic_blocks[0].statements.len();
    assert_eq!(before_count, 3); // c=42, result=c+1, _0=result

    // Run constprop: c should be replaced with 42 in the binop
    optimize::constant_propagation(&mut body);

    // Now c is no longer read (42 is inlined), so DCE should remove it
    optimize::dead_code_elimination(&mut body);

    let after_count = body.basic_blocks[0].statements.len();
    assert!(
        after_count < before_count,
        "Expected DCE to remove the now-unused c assignment"
    );
    assert_eq!(after_count, 2); // result=42+1, _0=result
}

#[test]
fn test_cross_verify_copyprop_then_dce() {
    // Verify that copy propagation enables DCE.
    let mut b = MirBuilder::new("test", vec![MirType::I64], MirType::I64);

    let x = b.new_local(MirType::I64, None);
    let y = b.new_local(MirType::I64, None);

    // x = copy param0
    b.assign(
        Place::local(x),
        Rvalue::Use(Operand::Copy(Place::local(b.param(0)))),
    );
    // y = copy x (will become y = copy param0 after copyprop)
    b.assign(
        Place::local(y),
        Rvalue::Use(Operand::Copy(Place::local(x))),
    );
    // _0 = copy y
    b.assign(
        b.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(y))),
    );
    b.return_();

    let mut body = b.build();
    assert_eq!(body.basic_blocks[0].statements.len(), 3);

    // Copy propagation: y and _0 should reference param0 directly
    optimize::copy_propagation(&mut body);

    // After copyprop, x and y are no longer read -- DCE removes them
    optimize::dead_code_elimination(&mut body);

    // Only _0 = copy param0 should remain
    assert_eq!(body.basic_blocks[0].statements.len(), 1);
}

#[test]
fn test_cross_verify_full_pipeline_identity() {
    // Verify that the full optimization pipeline produces correct output
    // for an identity function.
    let mut b = MirBuilder::new("id", vec![MirType::I64], MirType::I64);

    b.assign(
        b.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(b.param(0)))),
    );
    b.return_();

    let mut body = b.build();
    let before_display = body.display();
    optimize::optimize_mir_body(&mut body);
    let after_display = body.display();

    // Identity function should remain essentially the same
    assert!(after_display.contains("return"));
    assert_eq!(
        body.basic_blocks[0].statements.len(),
        1,
        "Identity function should keep exactly one statement"
    );
    // Should still be a valid body
    assert!(before_display.contains("fn id("));
    assert!(after_display.contains("fn id("));
}

#[test]
fn test_cross_verify_cse_then_dce() {
    // CSE eliminates duplicate computation; DCE removes the now-redundant assignment.
    let mut b = MirBuilder::new("test", vec![MirType::I64, MirType::I64], MirType::I64);

    let t1 = b.new_local(MirType::I64, None);
    let t2 = b.new_local(MirType::I64, None);

    let pa = Operand::Copy(Place::local(b.param(0)));
    let pb = Operand::Copy(Place::local(b.param(1)));

    // t1 = a + b
    b.assign_binop(t1, BinOp::Add, pa.clone(), pb.clone());
    // t2 = a + b (duplicate -- CSE will replace with copy t1)
    b.assign_binop(t2, BinOp::Add, pa, pb);
    // _0 = t1 + t2
    b.assign(
        b.return_place(),
        Rvalue::BinaryOp(
            BinOp::Add,
            Operand::Copy(Place::local(t1)),
            Operand::Copy(Place::local(t2)),
        ),
    );
    b.return_();

    let mut body = b.build();

    optimize::common_subexpression_elimination(&mut body);

    // t2 should now be Use(Copy(t1))
    if let Statement::Assign(_, Rvalue::Use(Operand::Copy(place))) =
        &body.basic_blocks[0].statements[1]
    {
        assert_eq!(place.local, t1);
    } else {
        panic!("Expected CSE to replace t2 with copy of t1");
    }
}

#[test]
fn test_cross_verify_optimize_preserves_semantics() {
    // Build a function that computes (x + 1) * 2, optimize it,
    // and verify the output still contains the correct operations.
    let source = r#"
        F calc(x: i64) -> i64 = {
            temp := x + 1
            temp * 2
        }
    "#;
    let module = vais_parser::parse(source).expect("Parse failed");
    let mut mir = lower::lower_module(&module);

    // Capture pre-optimization state
    let pre_ir = emit_llvm::emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");
    assert!(pre_ir.contains("define i64 @calc("));

    // Optimize
    optimize::optimize_mir_module(&mut mir);

    // Post-optimization IR should still be valid and contain the operations
    let post_ir = emit_llvm::emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");
    assert!(post_ir.contains("define i64 @calc("));
    assert!(post_ir.contains("add i64"));
    assert!(post_ir.contains("mul i64"));
    assert!(post_ir.contains("ret i64"));
}

#[test]
fn test_cross_verify_tail_call_in_pipeline() {
    // Verify tail call detection works through the optimization pipeline.
    let mut b = MirBuilder::new("wrapper", vec![MirType::I64], MirType::I64);

    let bb_ret = b.new_block();

    // Call inner(param0), result -> _0, then goto bb_ret which just returns
    b.call(
        "inner",
        vec![Operand::Copy(Place::local(b.param(0)))],
        Place::local(Local(0)),
        bb_ret,
    );

    b.switch_to_block(bb_ret);
    b.return_();

    let mut body = b.build();
    optimize::optimize_mir_body(&mut body);

    // Should be converted to tail call
    assert!(matches!(
        body.basic_blocks[0].terminator,
        Some(Terminator::TailCall { .. })
    ));
}

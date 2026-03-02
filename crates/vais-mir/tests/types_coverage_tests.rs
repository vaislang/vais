//! Coverage tests for vais-mir/src/types.rs
//!
//! Targets: Display impls, MirType::is_copy, Place methods, Body::display,
//! MirModule::display, and all type/statement/terminator Display formatting.

use vais_mir::*;

// ============================================================================
// Local Display
// ============================================================================

#[test]
fn test_local_display_zero() {
    assert_eq!(format!("{}", Local(0)), "_0");
}

#[test]
fn test_local_display_large() {
    assert_eq!(format!("{}", Local(999)), "_999");
}

// ============================================================================
// BasicBlockId Display
// ============================================================================

#[test]
fn test_basic_block_id_display_zero() {
    assert_eq!(format!("{}", BasicBlockId(0)), "bb0");
}

#[test]
fn test_basic_block_id_display_large() {
    assert_eq!(format!("{}", BasicBlockId(42)), "bb42");
}

// ============================================================================
// MirType::is_copy comprehensive tests
// ============================================================================

#[test]
fn test_is_copy_all_integer_types() {
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
}

#[test]
fn test_is_copy_float_types() {
    assert!(MirType::F32.is_copy());
    assert!(MirType::F64.is_copy());
}

#[test]
fn test_is_copy_special_types() {
    assert!(MirType::Bool.is_copy());
    assert!(MirType::Str.is_copy());
    assert!(MirType::Unit.is_copy());
    assert!(MirType::Never.is_copy());
}

#[test]
fn test_is_copy_pointer_ref() {
    assert!(MirType::Pointer(Box::new(MirType::I64)).is_copy());
    assert!(MirType::Ref(Box::new(MirType::I64)).is_copy());
}

#[test]
fn test_is_copy_ref_lifetime() {
    assert!(
        MirType::RefLifetime {
            lifetime: "a".to_string(),
            inner: Box::new(MirType::I64),
        }
        .is_copy()
    );
    assert!(
        MirType::RefMutLifetime {
            lifetime: "b".to_string(),
            inner: Box::new(MirType::Str),
        }
        .is_copy()
    );
}

#[test]
fn test_is_copy_tuple_all_copy() {
    let tuple = MirType::Tuple(vec![MirType::I64, MirType::Bool, MirType::F32]);
    assert!(tuple.is_copy());
}

#[test]
fn test_is_copy_tuple_with_non_copy() {
    let tuple = MirType::Tuple(vec![
        MirType::I64,
        MirType::Struct("Foo".into()),
        MirType::Bool,
    ]);
    assert!(!tuple.is_copy());
}

#[test]
fn test_is_copy_empty_tuple() {
    let tuple = MirType::Tuple(vec![]);
    assert!(tuple.is_copy());
}

#[test]
fn test_is_not_copy_struct() {
    assert!(!MirType::Struct("Point".into()).is_copy());
}

#[test]
fn test_is_not_copy_enum() {
    assert!(!MirType::Enum("Option".into()).is_copy());
}

#[test]
fn test_is_not_copy_array() {
    assert!(!MirType::Array(Box::new(MirType::I64)).is_copy());
}

#[test]
fn test_is_not_copy_function() {
    assert!(
        !MirType::Function {
            params: vec![MirType::I64],
            ret: Box::new(MirType::Bool),
        }
        .is_copy()
    );
}

// ============================================================================
// Constant Display
// ============================================================================

#[test]
fn test_constant_int_display() {
    assert_eq!(format!("{}", Constant::Int(0)), "0");
    assert_eq!(format!("{}", Constant::Int(-42)), "-42");
    assert_eq!(format!("{}", Constant::Int(i64::MAX)), format!("{}", i64::MAX));
}

#[test]
fn test_constant_float_display() {
    assert_eq!(format!("{}", Constant::Float(3.14)), "3.14");
    assert_eq!(format!("{}", Constant::Float(0.0)), "0");
}

#[test]
fn test_constant_bool_display() {
    assert_eq!(format!("{}", Constant::Bool(true)), "true");
    assert_eq!(format!("{}", Constant::Bool(false)), "false");
}

#[test]
fn test_constant_str_display() {
    assert_eq!(
        format!("{}", Constant::Str("hello world".into())),
        "\"hello world\""
    );
}

#[test]
fn test_constant_unit_display() {
    assert_eq!(format!("{}", Constant::Unit), "()");
}

// ============================================================================
// Operand Display
// ============================================================================

#[test]
fn test_operand_copy_display() {
    let op = Operand::Copy(Place::local(Local(3)));
    assert_eq!(format!("{}", op), "copy _3");
}

#[test]
fn test_operand_move_display() {
    let op = Operand::Move(Place::local(Local(5)));
    assert_eq!(format!("{}", op), "move _5");
}

#[test]
fn test_operand_constant_display() {
    let op = Operand::Constant(Constant::Int(99));
    assert_eq!(format!("{}", op), "const 99");
}

// ============================================================================
// Place construction and Display
// ============================================================================

#[test]
fn test_place_local_only() {
    let p = Place::local(Local(2));
    assert_eq!(format!("{}", p), "_2");
    assert!(p.projections.is_empty());
}

#[test]
fn test_place_field_projection() {
    let p = Place::local(Local(1)).field(0);
    assert_eq!(format!("{}", p), "_1.0");
}

#[test]
fn test_place_nested_field_projection() {
    let p = Place::local(Local(1)).field(0).field(3);
    assert_eq!(format!("{}", p), "_1.0.3");
}

#[test]
fn test_place_deref_projection() {
    let p = Place::local(Local(4)).deref();
    assert_eq!(format!("{}", p), "_4.*");
}

#[test]
fn test_place_index_projection() {
    let p = Place::local(Local(1)).index(Local(2));
    assert_eq!(format!("{}", p), "_1[_2]");
}

#[test]
fn test_place_complex_projection() {
    let p = Place::local(Local(0)).field(1).deref().index(Local(3));
    assert_eq!(format!("{}", p), "_0.1.*[_3]");
}

// ============================================================================
// BinOp Display
// ============================================================================

#[test]
fn test_binop_display_all() {
    assert_eq!(format!("{}", BinOp::Add), "Add");
    assert_eq!(format!("{}", BinOp::Sub), "Sub");
    assert_eq!(format!("{}", BinOp::Mul), "Mul");
    assert_eq!(format!("{}", BinOp::Div), "Div");
    assert_eq!(format!("{}", BinOp::Rem), "Rem");
    assert_eq!(format!("{}", BinOp::BitAnd), "BitAnd");
    assert_eq!(format!("{}", BinOp::BitOr), "BitOr");
    assert_eq!(format!("{}", BinOp::BitXor), "BitXor");
    assert_eq!(format!("{}", BinOp::Shl), "Shl");
    assert_eq!(format!("{}", BinOp::Shr), "Shr");
    assert_eq!(format!("{}", BinOp::Eq), "Eq");
    assert_eq!(format!("{}", BinOp::Ne), "Ne");
    assert_eq!(format!("{}", BinOp::Lt), "Lt");
    assert_eq!(format!("{}", BinOp::Le), "Le");
    assert_eq!(format!("{}", BinOp::Gt), "Gt");
    assert_eq!(format!("{}", BinOp::Ge), "Ge");
}

// ============================================================================
// UnOp Display
// ============================================================================

#[test]
fn test_unop_display() {
    assert_eq!(format!("{}", UnOp::Neg), "Neg");
    assert_eq!(format!("{}", UnOp::Not), "Not");
}

// ============================================================================
// Statement Display
// ============================================================================

#[test]
fn test_statement_assign_display() {
    let stmt = Statement::Assign(
        Place::local(Local(1)),
        Rvalue::Use(Operand::Constant(Constant::Int(42))),
    );
    let display = format!("{}", stmt);
    assert!(display.contains("_1"));
    assert!(display.contains("Use"));
}

#[test]
fn test_statement_drop_display() {
    let stmt = Statement::Drop(Place::local(Local(2)));
    assert_eq!(format!("{}", stmt), "drop(_2)");
}

#[test]
fn test_statement_nop_display() {
    assert_eq!(format!("{}", Statement::Nop), "nop");
}

// ============================================================================
// Terminator Display
// ============================================================================

#[test]
fn test_terminator_goto_display() {
    let term = Terminator::Goto(BasicBlockId(2));
    assert_eq!(format!("{}", term), "goto -> bb2");
}

#[test]
fn test_terminator_return_display() {
    assert_eq!(format!("{}", Terminator::Return), "return");
}

#[test]
fn test_terminator_unreachable_display() {
    assert_eq!(format!("{}", Terminator::Unreachable), "unreachable");
}

#[test]
fn test_terminator_switch_int_display() {
    let term = Terminator::SwitchInt {
        discriminant: Operand::Copy(Place::local(Local(1))),
        targets: vec![(0, BasicBlockId(1)), (1, BasicBlockId(2))],
        otherwise: BasicBlockId(3),
    };
    let display = format!("{}", term);
    assert!(display.contains("switchInt(copy _1)"));
    assert!(display.contains("0: bb1"));
    assert!(display.contains("1: bb2"));
    assert!(display.contains("otherwise: bb3"));
}

#[test]
fn test_terminator_call_display() {
    let term = Terminator::Call {
        func: "add".to_string(),
        args: vec![
            Operand::Copy(Place::local(Local(1))),
            Operand::Copy(Place::local(Local(2))),
        ],
        destination: Place::local(Local(3)),
        target: BasicBlockId(1),
    };
    let display = format!("{}", term);
    assert!(display.contains("_3 = add(copy _1, copy _2) -> bb1"));
}

#[test]
fn test_terminator_call_no_args_display() {
    let term = Terminator::Call {
        func: "noop".to_string(),
        args: vec![],
        destination: Place::local(Local(0)),
        target: BasicBlockId(1),
    };
    let display = format!("{}", term);
    assert!(display.contains("noop()"));
}

#[test]
fn test_terminator_tailcall_display() {
    let term = Terminator::TailCall {
        func: "recurse".to_string(),
        args: vec![Operand::Constant(Constant::Int(5))],
    };
    let display = format!("{}", term);
    assert!(display.contains("tailcall recurse(const 5)"));
}

#[test]
fn test_terminator_tailcall_no_args() {
    let term = Terminator::TailCall {
        func: "f".to_string(),
        args: vec![],
    };
    let display = format!("{}", term);
    assert!(display.contains("tailcall f()"));
}

#[test]
fn test_terminator_assert_display() {
    let term = Terminator::Assert {
        cond: Operand::Copy(Place::local(Local(1))),
        expected: true,
        msg: "bounds check".to_string(),
        target: BasicBlockId(2),
    };
    let display = format!("{}", term);
    assert!(display.contains("assert(copy _1, true, \"bounds check\") -> bb2"));
}

// ============================================================================
// BasicBlock
// ============================================================================

#[test]
fn test_basic_block_new_default() {
    let bb = BasicBlock::new();
    assert!(bb.statements.is_empty());
    assert!(bb.terminator.is_none());

    let bb_default = BasicBlock::default();
    assert!(bb_default.statements.is_empty());
    assert!(bb_default.terminator.is_none());
}

// ============================================================================
// MirModule
// ============================================================================

#[test]
fn test_mir_module_new() {
    let m = MirModule::new("test");
    assert_eq!(m.name, "test");
    assert!(m.bodies.is_empty());
    assert!(m.structs.is_empty());
    assert!(m.enums.is_empty());
}

#[test]
fn test_mir_module_new_from_string() {
    let m = MirModule::new(String::from("my_module"));
    assert_eq!(m.name, "my_module");
}

#[test]
fn test_mir_module_display_with_structs() {
    let mut m = MirModule::new("test");
    m.structs.insert(
        "Point".to_string(),
        vec![
            ("x".to_string(), MirType::F64),
            ("y".to_string(), MirType::F64),
        ],
    );
    let display = m.display();
    assert!(display.contains("MIR module: test"));
    assert!(display.contains("struct Point"));
    assert!(display.contains("x: F64"));
    assert!(display.contains("y: F64"));
}

#[test]
fn test_mir_module_display_empty() {
    let m = MirModule::new("empty");
    let display = m.display();
    assert!(display.contains("MIR module: empty"));
}

// ============================================================================
// Body display
// ============================================================================

#[test]
fn test_body_display_with_lifetime_params() {
    let mut builder = MirBuilder::new("borrow", vec![MirType::I64], MirType::I64);
    builder.assign_const(Local(0), Constant::Int(0));
    builder.return_();
    let mut body = builder.build();
    body.lifetime_params = vec!["a".to_string(), "b".to_string()];

    let display = body.display();
    assert!(display.contains("fn borrow<'a, 'b>("));
}

#[test]
fn test_body_display_mutable_locals() {
    let mut builder = MirBuilder::new("test", vec![], MirType::I64);
    let local = builder.new_local(MirType::I64, Some("x".to_string()));
    builder.assign_const(local, Constant::Int(42));
    builder.assign_const(Local(0), Constant::Int(0));
    builder.return_();
    let body = builder.build();

    let display = body.display();
    assert!(display.contains("let mut"));
    assert!(display.contains("// x"));
}

// ============================================================================
// MirBuilder comprehensive tests
// ============================================================================

#[test]
fn test_builder_return_place() {
    let b = MirBuilder::new("f", vec![], MirType::I64);
    let rp = b.return_place();
    assert_eq!(rp.local, Local(0));
}

#[test]
fn test_builder_param() {
    let b = MirBuilder::new("f", vec![MirType::I64, MirType::Bool], MirType::I64);
    assert_eq!(b.param(0), Local(1));
    assert_eq!(b.param(1), Local(2));
}

#[test]
fn test_builder_new_local() {
    let mut b = MirBuilder::new("f", vec![MirType::I64], MirType::Bool);
    let l1 = b.new_local(MirType::F64, Some("temp".to_string()));
    let l2 = b.new_local(MirType::I32, None);
    assert_eq!(l1, Local(2)); // _0 return, _1 param, _2 first new local
    assert_eq!(l2, Local(3));
}

#[test]
fn test_builder_switch_int() {
    let mut b = MirBuilder::new("f", vec![MirType::I64], MirType::I64);
    let bb1 = b.new_block();
    let bb2 = b.new_block();
    let bb3 = b.new_block();
    b.switch_int(
        Operand::Copy(Place::local(Local(1))),
        vec![(0, bb1), (1, bb2)],
        bb3,
    );

    let body = b.build();
    let display = body.display();
    assert!(display.contains("switchInt"));
}

#[test]
fn test_builder_call() {
    let mut b = MirBuilder::new("main", vec![], MirType::I64);
    let result = b.new_local(MirType::I64, None);
    let bb1 = b.new_block();
    b.call(
        "add",
        vec![
            Operand::Constant(Constant::Int(1)),
            Operand::Constant(Constant::Int(2)),
        ],
        Place::local(result),
        bb1,
    );
    b.switch_to_block(bb1);
    b.assign_const(Local(0), Constant::Int(0));
    b.return_();

    let body = b.build();
    let display = body.display();
    assert!(display.contains("add(const 1, const 2)"));
}

#[test]
fn test_builder_drop() {
    let mut b = MirBuilder::new("f", vec![], MirType::I64);
    b.drop(Place::local(Local(0)));
    b.assign_const(Local(0), Constant::Int(0));
    b.return_();
    let body = b.build();
    let display = body.display();
    assert!(display.contains("drop(_0)"));
}

#[test]
fn test_builder_goto() {
    let mut b = MirBuilder::new("f", vec![], MirType::I64);
    let bb1 = b.new_block();
    b.goto(bb1);
    b.switch_to_block(bb1);
    b.assign_const(Local(0), Constant::Int(0));
    b.return_();
    let body = b.build();
    let display = body.display();
    assert!(display.contains("goto -> bb1"));
}

#[test]
fn test_builder_assign_binop() {
    let mut b = MirBuilder::new("f", vec![MirType::I64, MirType::I64], MirType::I64);
    b.assign_binop(
        Local(0),
        BinOp::Add,
        Operand::Copy(Place::local(Local(1))),
        Operand::Copy(Place::local(Local(2))),
    );
    b.return_();
    let body = b.build();
    let display = body.display();
    assert!(display.contains("Add"));
}

#[test]
fn test_builder_push_stmt() {
    let mut b = MirBuilder::new("f", vec![], MirType::I64);
    b.push_stmt(Statement::Nop);
    b.assign_const(Local(0), Constant::Int(0));
    b.return_();
    let body = b.build();
    let display = body.display();
    assert!(display.contains("nop"));
}

// ============================================================================
// Rvalue types (exercising constructors)
// ============================================================================

#[test]
fn test_rvalue_aggregate_struct() {
    let rvalue = Rvalue::Aggregate(
        AggregateKind::Struct("Point".to_string()),
        vec![
            Operand::Constant(Constant::Float(1.0)),
            Operand::Constant(Constant::Float(2.0)),
        ],
    );
    assert!(matches!(
        rvalue,
        Rvalue::Aggregate(AggregateKind::Struct(_), _)
    ));
}

#[test]
fn test_rvalue_aggregate_enum() {
    let rvalue = Rvalue::Aggregate(
        AggregateKind::Enum("Option".to_string(), 0),
        vec![Operand::Constant(Constant::Int(42))],
    );
    assert!(matches!(
        rvalue,
        Rvalue::Aggregate(AggregateKind::Enum(_, 0), _)
    ));
}

#[test]
fn test_rvalue_discriminant() {
    let rvalue = Rvalue::Discriminant(Place::local(Local(1)));
    assert!(matches!(rvalue, Rvalue::Discriminant(_)));
}

#[test]
fn test_rvalue_len() {
    let rvalue = Rvalue::Len(Place::local(Local(1)));
    assert!(matches!(rvalue, Rvalue::Len(_)));
}

#[test]
fn test_rvalue_cast() {
    let rvalue = Rvalue::Cast(Operand::Copy(Place::local(Local(1))), MirType::F64);
    assert!(matches!(rvalue, Rvalue::Cast(_, MirType::F64)));
}

#[test]
fn test_rvalue_ref() {
    let rvalue = Rvalue::Ref(Place::local(Local(1)));
    assert!(matches!(rvalue, Rvalue::Ref(_)));
}

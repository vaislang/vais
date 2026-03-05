//! Middle Intermediate Representation (MIR) for the Vais compiler.
//!
//! MIR sits between the typed AST and LLVM IR, providing a platform-independent
//! representation suitable for optimization passes:
//!
//! ```text
//! AST (vais-ast) → Type Check (vais-types) → MIR (vais-mir) → LLVM IR (vais-codegen)
//! ```
//!
//! MIR uses a control-flow graph (CFG) of basic blocks with explicit
//! temporaries, drops, and control flow edges. This enables:
//! - Borrow checking and move analysis
//! - Dead code elimination
//! - Constant propagation
//! - Common subexpression elimination
//! - Inlining decisions
//! - Drop elaboration

pub mod borrow_check;
mod builder;
pub mod emit_llvm;
pub mod lower;
pub mod optimize;
mod types;

pub use builder::MirBuilder;
pub use types::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_pipeline_simple_add() {
        let source = "F add(x: i64, y: i64) -> i64 = x + y";
        let module = vais_parser::parse(source).expect("Parse failed");
        let mut mir = lower::lower_module(&module);

        // Apply optimizations
        optimize::optimize_mir_module(&mut mir);

        // Emit LLVM IR
        let ir = emit_llvm::emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");

        assert!(ir.contains("define i64 @add("));
        assert!(ir.contains("add i64"));
        assert!(ir.contains("ret i64"));
    }

    #[test]
    fn test_full_pipeline_with_optimization() {
        let source = r#"
            F compute(x: i64) -> i64 = {
                unused := 999
                const_a := 10
                const_b := 20
                result := const_a + const_b
                x + result
            }
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mut mir = lower::lower_module(&module);

        let before_opt = mir.bodies[0].display();

        optimize::optimize_mir_module(&mut mir);

        let after_opt = mir.bodies[0].display();

        // After optimization, should have fewer operations
        assert!(after_opt.len() < before_opt.len() || after_opt.contains("const 10"));
    }

    #[test]
    fn test_full_pipeline_control_flow() {
        let source = "F abs(x: i64) -> i64 = I x < 0 { 0 - x } E { x }";
        let module = vais_parser::parse(source).expect("Parse failed");
        let mut mir = lower::lower_module(&module);

        optimize::optimize_mir_module(&mut mir);
        let ir = emit_llvm::emit_llvm_ir(&mir, "x86_64-apple-darwin");

        assert!(ir.contains("define i64 @abs("));
        assert!(ir.contains("icmp"));
        assert!(ir.contains("br"));
        assert!(ir.contains("sub i64 0,"));
    }

    #[test]
    fn test_full_pipeline_multiple_functions() {
        let source = r#"
            F double(x: i64) -> i64 = x * 2
            F triple(x: i64) -> i64 = x * 3
            F sum(a: i64, b: i64) -> i64 = a + b
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mut mir = lower::lower_module(&module);

        assert_eq!(mir.bodies.len(), 3);

        optimize::optimize_mir_module(&mut mir);
        let ir = emit_llvm::emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");

        assert!(ir.contains("define i64 @double("));
        assert!(ir.contains("define i64 @triple("));
        assert!(ir.contains("define i64 @sum("));
    }

    #[test]
    fn test_mir_module_display() {
        let source = "F test(x: i64, y: i64) -> i64 = x + y";
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower::lower_module(&module);

        let display = mir.display();
        assert!(display.contains("MIR module"));
        assert!(display.contains("fn test("));
        assert!(display.contains("_1: I64"));
        assert!(display.contains("_2: I64"));
    }

    #[test]
    fn test_body_display_with_blocks() {
        let source = "F branch(x: i64) -> i64 = I x > 0 { 1 } E { 0 }";
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower::lower_module(&module);

        let display = mir.bodies[0].display();
        assert!(display.contains("bb0:"));
        assert!(display.contains("bb1:"));
        assert!(display.contains("bb2:"));
        assert!(display.contains("switchInt"));
        assert!(display.contains("goto"));
    }

    // ==========================================================================
    // Emit LLVM IR edge cases
    // ==========================================================================

    #[test]
    fn test_emit_void_return() {
        let mut builder = MirBuilder::new("void_fn", vec![], MirType::Unit);
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("define void @void_fn()"));
        assert!(ir.contains("ret void"));
    }

    #[test]
    fn test_emit_assert_terminator() {
        let mut builder = MirBuilder::new("checked", vec![MirType::Bool], MirType::I64);
        let bb_ok = builder.new_block();
        builder.terminate(Terminator::Assert {
            cond: Operand::Copy(Place::local(builder.param(0))),
            expected: true,
            msg: "precondition violated".into(),
            target: bb_ok,
        });
        builder.switch_to_block(bb_ok);
        builder.assign_const(Local(0), Constant::Int(0));
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("icmp eq i1"));
        assert!(ir.contains("assert: precondition violated"));
        assert!(ir.contains("assert_fail"));
    }

    #[test]
    fn test_emit_tail_call() {
        let mut builder = MirBuilder::new("tail", vec![MirType::I64], MirType::I64);
        builder.terminate(Terminator::TailCall {
            func: "target_fn".into(),
            args: vec![Operand::Copy(Place::local(builder.param(0)))],
        });
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("musttail call"));
        assert!(ir.contains("@target_fn"));
    }

    #[test]
    fn test_emit_unreachable_terminator() {
        let mut builder = MirBuilder::new("unreachable_fn", vec![], MirType::I64);
        builder.terminate(Terminator::Unreachable);
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("unreachable"));
    }

    #[test]
    fn test_emit_cast_same_type() {
        let mut builder = MirBuilder::new("cast_same", vec![MirType::I64], MirType::I64);
        let result = builder.new_local(MirType::I64, None);
        builder.assign(
            Place::local(result),
            Rvalue::Cast(Operand::Copy(Place::local(builder.param(0))), MirType::I64),
        );
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(result))),
        );
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        // Same type cast should produce an add (identity)
        assert!(ir.contains("add i64 0,"));
    }

    #[test]
    fn test_emit_cast_different_type() {
        let mut builder = MirBuilder::new("cast_diff", vec![MirType::I32], MirType::I32);
        let result = builder.new_local(MirType::I32, None);
        builder.assign(
            Place::local(result),
            Rvalue::Cast(Operand::Copy(Place::local(builder.param(0))), MirType::I64),
        );
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(result))),
        );
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("bitcast"));
    }

    #[test]
    fn test_emit_discriminant() {
        let mut builder = MirBuilder::new("get_disc", vec![MirType::I64], MirType::I64);
        let disc = builder.new_local(MirType::I64, None);
        builder.assign(
            Place::local(disc),
            Rvalue::Discriminant(Place::local(builder.param(0))),
        );
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(disc))),
        );
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("discriminant"));
    }

    #[test]
    fn test_emit_len() {
        let mut builder = MirBuilder::new(
            "get_len",
            vec![MirType::Array(Box::new(MirType::I64))],
            MirType::I64,
        );
        let len = builder.new_local(MirType::I64, None);
        builder.assign(
            Place::local(len),
            Rvalue::Len(Place::local(builder.param(0))),
        );
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(len))),
        );
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("len"));
    }

    #[test]
    fn test_emit_ref_rvalue() {
        let mut builder = MirBuilder::new("take_ref", vec![MirType::I64], MirType::I64);
        let r = builder.new_local(MirType::Ref(Box::new(MirType::I64)), None);
        builder.assign(Place::local(r), Rvalue::Ref(Place::local(builder.param(0))));
        builder.assign_const(Local(0), Constant::Int(0));
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("getelementptr"));
    }

    #[test]
    fn test_emit_aggregate_rvalue() {
        let mut builder =
            MirBuilder::new("make_tuple", vec![MirType::I64, MirType::I64], MirType::I64);
        let tup = builder.new_local(MirType::Tuple(vec![MirType::I64, MirType::I64]), None);
        builder.assign(
            Place::local(tup),
            Rvalue::Aggregate(
                AggregateKind::Tuple,
                vec![
                    Operand::Copy(Place::local(builder.param(0))),
                    Operand::Copy(Place::local(builder.param(1))),
                ],
            ),
        );
        builder.assign_const(Local(0), Constant::Int(0));
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("aggregate construction"));
    }

    #[test]
    fn test_emit_drop_statement() {
        let mut builder = MirBuilder::new("drop_test", vec![MirType::I64], MirType::I64);
        builder.drop(Place::local(builder.param(0)));
        builder.assign_const(Local(0), Constant::Int(0));
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("; drop"));
    }

    #[test]
    fn test_emit_nop_statement() {
        let mut builder = MirBuilder::new("nop_test", vec![], MirType::I64);
        builder.push_stmt(Statement::Nop);
        builder.assign_const(Local(0), Constant::Int(0));
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("; nop"));
    }

    // ==========================================================================
    // MirType LLVM mapping edge cases
    // ==========================================================================

    #[test]
    fn test_emit_all_integer_types() {
        for (mir_ty, expected_llvm) in [
            (MirType::I8, "i8"),
            (MirType::I16, "i16"),
            (MirType::I32, "i32"),
            (MirType::I64, "i64"),
            (MirType::I128, "i128"),
            (MirType::U8, "i8"),
            (MirType::U16, "i16"),
            (MirType::U32, "i32"),
            (MirType::U64, "i64"),
            (MirType::U128, "i128"),
        ] {
            let mut builder = MirBuilder::new("test_ty", vec![mir_ty.clone()], mir_ty.clone());
            builder.assign(
                builder.return_place(),
                Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
            );
            builder.return_();
            let body = builder.build();
            let mut module = MirModule::new("test");
            module.bodies.push(body);
            let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
            assert!(
                ir.contains(&format!("define {} @test_ty", expected_llvm)),
                "Expected '{}' for {:?}, got: {}",
                expected_llvm,
                mir_ty,
                &ir[..ir.len().min(200)]
            );
        }
    }

    #[test]
    fn test_emit_float_types() {
        // F32
        let mut builder = MirBuilder::new("f32_fn", vec![MirType::F32], MirType::F32);
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
        );
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("define float @f32_fn"));
    }

    #[test]
    fn test_emit_bool_type() {
        let mut builder = MirBuilder::new("bool_fn", vec![MirType::Bool], MirType::Bool);
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
        );
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("define i1 @bool_fn"));
    }

    #[test]
    fn test_emit_str_type() {
        let mut builder = MirBuilder::new("str_fn", vec![MirType::Str], MirType::Str);
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
        );
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("i8*"));
    }

    #[test]
    fn test_emit_pointer_type() {
        let ptr_ty = MirType::Pointer(Box::new(MirType::I64));
        let mut builder = MirBuilder::new("ptr_fn", vec![ptr_ty.clone()], ptr_ty);
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
        );
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("i64*"));
    }

    #[test]
    fn test_emit_tuple_type() {
        let tup_ty = MirType::Tuple(vec![MirType::I64, MirType::Bool]);
        let mut builder = MirBuilder::new("tuple_fn", vec![tup_ty.clone()], tup_ty);
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
        );
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("{ i64, i1 }"));
    }

    #[test]
    fn test_emit_never_type() {
        let mut builder = MirBuilder::new("never_fn", vec![], MirType::Never);
        builder.terminate(Terminator::Unreachable);
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("define void @never_fn"));
    }

    #[test]
    fn test_emit_enum_type() {
        let mut module = MirModule::new("test");
        module.enums.insert(
            "Color".into(),
            vec![
                ("Red".into(), vec![]),
                ("Green".into(), vec![]),
                ("Blue".into(), vec![]),
            ],
        );
        let enum_ty = MirType::Enum("Color".into());
        let mut builder = MirBuilder::new("enum_fn", vec![enum_ty.clone()], enum_ty);
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
        );
        builder.return_();
        module.bodies.push(builder.build());
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("%Color"));
    }

    #[test]
    fn test_emit_function_type() {
        let fn_ty = MirType::Function {
            params: vec![MirType::I64],
            ret: Box::new(MirType::I64),
        };
        let mut builder = MirBuilder::new("fn_fn", vec![fn_ty.clone()], fn_ty);
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
        );
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("i64 (i64)"));
    }

    #[test]
    fn test_emit_ref_lifetime_type() {
        let ref_lt_ty = MirType::RefLifetime {
            lifetime: "a".into(),
            inner: Box::new(MirType::I64),
        };
        let mut builder = MirBuilder::new("ref_lt", vec![ref_lt_ty.clone()], ref_lt_ty);
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
        );
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("i64*"));
    }

    #[test]
    fn test_emit_ref_mut_lifetime_type() {
        let ref_mut_lt_ty = MirType::RefMutLifetime {
            lifetime: "a".into(),
            inner: Box::new(MirType::I32),
        };
        let mut builder = MirBuilder::new("ref_mut_lt", vec![ref_mut_lt_ty.clone()], ref_mut_lt_ty);
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
        );
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("i32*"));
    }

    #[test]
    fn test_emit_array_type() {
        let arr_ty = MirType::Array(Box::new(MirType::F64));
        let mut builder = MirBuilder::new("arr_fn", vec![arr_ty.clone()], arr_ty);
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
        );
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("[0 x double]"));
    }

    // ==========================================================================
    // MirType is_copy coverage
    // ==========================================================================

    #[test]
    fn test_mir_type_is_copy() {
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
        assert!(MirType::Pointer(Box::new(MirType::I64)).is_copy());
        assert!(MirType::Ref(Box::new(MirType::I64)).is_copy());
        assert!(MirType::RefLifetime {
            lifetime: "a".into(),
            inner: Box::new(MirType::I64)
        }
        .is_copy());
        assert!(MirType::RefMutLifetime {
            lifetime: "a".into(),
            inner: Box::new(MirType::I64)
        }
        .is_copy());
        // Non-copy types
        assert!(!MirType::Array(Box::new(MirType::I64)).is_copy());
        assert!(!MirType::Struct("Foo".into()).is_copy());
        assert!(!MirType::Enum("Bar".into()).is_copy());
        assert!(!(MirType::Function {
            params: vec![],
            ret: Box::new(MirType::I64)
        })
        .is_copy());
    }

    #[test]
    fn test_mir_type_tuple_copy() {
        // Tuple of copy types is copy
        assert!(MirType::Tuple(vec![MirType::I64, MirType::Bool]).is_copy());
        // Tuple containing non-copy is not copy
        assert!(!MirType::Tuple(vec![MirType::I64, MirType::Struct("X".into())]).is_copy());
        // Empty tuple is copy
        assert!(MirType::Tuple(vec![]).is_copy());
    }

    // ==========================================================================
    // Display formatting coverage
    // ==========================================================================

    #[test]
    fn test_binop_display() {
        let ops = vec![
            (BinOp::Add, "Add"),
            (BinOp::Sub, "Sub"),
            (BinOp::Mul, "Mul"),
            (BinOp::Div, "Div"),
            (BinOp::Rem, "Rem"),
            (BinOp::BitAnd, "BitAnd"),
            (BinOp::BitOr, "BitOr"),
            (BinOp::BitXor, "BitXor"),
            (BinOp::Shl, "Shl"),
            (BinOp::Shr, "Shr"),
            (BinOp::Eq, "Eq"),
            (BinOp::Ne, "Ne"),
            (BinOp::Lt, "Lt"),
            (BinOp::Le, "Le"),
            (BinOp::Gt, "Gt"),
            (BinOp::Ge, "Ge"),
        ];
        for (op, expected) in ops {
            assert_eq!(format!("{}", op), expected);
        }
    }

    #[test]
    fn test_unop_display() {
        assert_eq!(format!("{}", UnOp::Neg), "Neg");
        assert_eq!(format!("{}", UnOp::Not), "Not");
    }

    #[test]
    fn test_statement_display() {
        let nop = Statement::Nop;
        assert_eq!(format!("{}", nop), "nop");

        let drop = Statement::Drop(Place::local(Local(1)));
        assert_eq!(format!("{}", drop), "drop(_1)");

        let assign = Statement::Assign(
            Place::local(Local(0)),
            Rvalue::Use(Operand::Constant(Constant::Int(42))),
        );
        let assign_str = format!("{}", assign);
        assert!(assign_str.contains("_0"));
    }

    #[test]
    fn test_terminator_display_all_variants() {
        // Goto
        assert_eq!(
            format!("{}", Terminator::Goto(BasicBlockId(1))),
            "goto -> bb1"
        );
        // Return
        assert_eq!(format!("{}", Terminator::Return), "return");
        // Unreachable
        assert_eq!(format!("{}", Terminator::Unreachable), "unreachable");
        // TailCall
        let tc = Terminator::TailCall {
            func: "foo".into(),
            args: vec![Operand::Constant(Constant::Int(1))],
        };
        let tc_str = format!("{}", tc);
        assert!(tc_str.contains("tailcall foo("));
        // Assert
        let a = Terminator::Assert {
            cond: Operand::Constant(Constant::Bool(true)),
            expected: true,
            msg: "test".into(),
            target: BasicBlockId(2),
        };
        let a_str = format!("{}", a);
        assert!(a_str.contains("assert("));
        assert!(a_str.contains("test"));
        // SwitchInt
        let si = Terminator::SwitchInt {
            discriminant: Operand::Copy(Place::local(Local(1))),
            targets: vec![(0, BasicBlockId(1)), (1, BasicBlockId(2))],
            otherwise: BasicBlockId(3),
        };
        let si_str = format!("{}", si);
        assert!(si_str.contains("switchInt("));
        assert!(si_str.contains("otherwise: bb3"));
        // Call
        let call = Terminator::Call {
            func: "bar".into(),
            args: vec![
                Operand::Constant(Constant::Int(1)),
                Operand::Constant(Constant::Int(2)),
            ],
            destination: Place::local(Local(0)),
            target: BasicBlockId(1),
        };
        let call_str = format!("{}", call);
        assert!(call_str.contains("bar("));
        assert!(call_str.contains("-> bb1"));
    }

    #[test]
    fn test_constant_float_display() {
        let c = Constant::Float(3.14);
        let s = format!("{}", c);
        assert!(s.contains("3.14"));
    }

    #[test]
    fn test_place_index_display() {
        let place = Place::local(Local(1)).index(Local(2));
        assert_eq!(format!("{}", place), "_1[_2]");
    }

    // ==========================================================================
    // Lowering edge cases
    // ==========================================================================

    #[test]
    fn test_lower_struct_definition() {
        let source = r#"
            S Point {
                x: i64,
                y: i64
            }
            F origin() -> i64 = 0
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower::lower_module(&module);
        assert!(mir.structs.contains_key("Point"));
        let fields = &mir.structs["Point"];
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].0, "x");
        assert_eq!(fields[1].0, "y");
    }

    #[test]
    fn test_lower_assignment() {
        let source = r#"
            F mutate(x: i64) -> i64 = {
                y := mut x
                y = y + 1
                y
            }
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower::lower_module(&module);
        assert_eq!(mir.bodies.len(), 1);
    }

    #[test]
    fn test_lower_void_return() {
        let source = r#"
            F do_nothing() -> i64 = {
                R 0
            }
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower::lower_module(&module);
        assert_eq!(mir.bodies.len(), 1);
        let display = mir.bodies[0].display();
        assert!(display.contains("return"));
    }

    #[test]
    fn test_lower_function_call() {
        let source = r#"
            F callee(x: i64) -> i64 = x * 2
            F caller() -> i64 = callee(21)
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower::lower_module(&module);
        assert_eq!(mir.bodies.len(), 2);
        let caller_display = mir.bodies[1].display();
        assert!(caller_display.contains("callee"));
    }

    #[test]
    fn test_lower_float_literal() {
        let source = "F pi() -> f64 = 3.14";
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower::lower_module(&module);
        assert_eq!(mir.bodies.len(), 1);
    }

    #[test]
    fn test_lower_bool_literal() {
        let source = "F truth() -> bool = true";
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower::lower_module(&module);
        assert_eq!(mir.bodies.len(), 1);
    }

    #[test]
    fn test_lower_string_literal() {
        let source = r#"F greeting() -> str = "hello""#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower::lower_module(&module);
        assert_eq!(mir.bodies.len(), 1);
    }

    #[test]
    fn test_lower_match_with_bool_patterns() {
        let source = r#"
            F check(x: bool) -> i64 = M x {
                true => 1,
                false => 0
            }
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower::lower_module(&module);
        assert_eq!(mir.bodies.len(), 1);
        let display = mir.bodies[0].display();
        assert!(display.contains("switchInt"));
    }

    #[test]
    fn test_lower_nested_if() {
        let source = r#"
            F classify(x: i64) -> i64 = I x > 0 { I x > 100 { 2 } E { 1 } } E { 0 }
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mir = lower::lower_module(&module);
        assert_eq!(mir.bodies.len(), 1);
        assert!(mir.bodies[0].basic_blocks.len() >= 5);
    }

    // ==========================================================================
    // Body display with lifetime params
    // ==========================================================================

    #[test]
    fn test_body_display_with_lifetime_params() {
        let body = Body {
            name: "borrow_fn".into(),
            params: vec![MirType::RefLifetime {
                lifetime: "a".into(),
                inner: Box::new(MirType::I64),
            }],
            return_type: MirType::I64,
            locals: vec![
                LocalDecl {
                    name: Some("_return".into()),
                    ty: MirType::I64,
                    is_mutable: true,
                    lifetime: None,
                },
                LocalDecl {
                    name: Some("x".into()),
                    ty: MirType::RefLifetime {
                        lifetime: "a".into(),
                        inner: Box::new(MirType::I64),
                    },
                    is_mutable: false,
                    lifetime: None,
                },
            ],
            basic_blocks: vec![BasicBlock {
                statements: vec![],
                terminator: Some(Terminator::Return),
            }],
            block_names: Default::default(),
            lifetime_params: vec!["a".into()],
            lifetime_bounds: vec![],
        };
        let display = body.display();
        assert!(display.contains("'a"));
        assert!(display.contains("fn borrow_fn<"));
    }

    // ==========================================================================
    // Emit LLVM float comparison ops
    // ==========================================================================

    #[test]
    fn test_emit_float_comparison_ops() {
        for (op, expected) in [
            (BinOp::Lt, "fcmp olt"),
            (BinOp::Le, "fcmp ole"),
            (BinOp::Gt, "fcmp ogt"),
            (BinOp::Ge, "fcmp oge"),
            (BinOp::Add, "fadd"),
            (BinOp::Sub, "fsub"),
            (BinOp::Div, "fdiv"),
            (BinOp::Rem, "frem"),
        ] {
            let mut builder =
                MirBuilder::new("float_op", vec![MirType::F64, MirType::F64], MirType::F64);
            let result = builder.new_local(MirType::F64, None);
            builder.assign_binop(
                result,
                op,
                Operand::Copy(Place::local(builder.param(0))),
                Operand::Copy(Place::local(builder.param(1))),
            );
            builder.assign(
                builder.return_place(),
                Rvalue::Use(Operand::Copy(Place::local(result))),
            );
            builder.return_();
            let body = builder.build();
            let mut module = MirModule::new("test");
            module.bodies.push(body);
            let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
            assert!(
                ir.contains(expected),
                "Expected '{}' for {:?}, got:\n{}",
                expected,
                op,
                ir
            );
        }
    }

    #[test]
    fn test_emit_integer_bitwise_ops() {
        for (op, expected) in [
            (BinOp::BitAnd, "and i64"),
            (BinOp::BitOr, "or i64"),
            (BinOp::BitXor, "xor i64"),
            (BinOp::Shl, "shl i64"),
            (BinOp::Shr, "ashr i64"),
            (BinOp::Rem, "srem i64"),
        ] {
            let mut builder =
                MirBuilder::new("bit_op", vec![MirType::I64, MirType::I64], MirType::I64);
            let result = builder.new_local(MirType::I64, None);
            builder.assign_binop(
                result,
                op,
                Operand::Copy(Place::local(builder.param(0))),
                Operand::Copy(Place::local(builder.param(1))),
            );
            builder.assign(
                builder.return_place(),
                Rvalue::Use(Operand::Copy(Place::local(result))),
            );
            builder.return_();
            let body = builder.build();
            let mut module = MirModule::new("test");
            module.bodies.push(body);
            let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
            assert!(
                ir.contains(expected),
                "Expected '{}' for {:?}, got:\n{}",
                expected,
                op,
                ir
            );
        }
    }

    #[test]
    fn test_emit_comparison_ops_all() {
        for (op, expected) in [
            (BinOp::Eq, "icmp eq"),
            (BinOp::Ne, "icmp ne"),
            (BinOp::Lt, "icmp slt"),
            (BinOp::Le, "icmp sle"),
            (BinOp::Gt, "icmp sgt"),
            (BinOp::Ge, "icmp sge"),
        ] {
            let mut builder =
                MirBuilder::new("cmp_op", vec![MirType::I64, MirType::I64], MirType::I64);
            let result = builder.new_local(MirType::I64, None);
            builder.assign_binop(
                result,
                op,
                Operand::Copy(Place::local(builder.param(0))),
                Operand::Copy(Place::local(builder.param(1))),
            );
            builder.assign(
                builder.return_place(),
                Rvalue::Use(Operand::Copy(Place::local(result))),
            );
            builder.return_();
            let body = builder.build();
            let mut module = MirModule::new("test");
            module.bodies.push(body);
            let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
            assert!(
                ir.contains(expected),
                "Expected '{}' for {:?}, got:\n{}",
                expected,
                op,
                ir
            );
        }
    }

    // ==========================================================================
    // Operand type inference in emit_llvm
    // ==========================================================================

    #[test]
    fn test_emit_constant_bool_operand() {
        let mut builder = MirBuilder::new("const_bool", vec![], MirType::I64);
        let b = builder.new_local(MirType::Bool, None);
        builder.assign(
            Place::local(b),
            Rvalue::Use(Operand::Constant(Constant::Bool(true))),
        );
        builder.assign_const(Local(0), Constant::Int(0));
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("1")); // true -> 1
    }

    #[test]
    fn test_emit_constant_float_operand() {
        let mut builder = MirBuilder::new("const_float", vec![], MirType::F64);
        let f = builder.new_local(MirType::F64, None);
        builder.assign(
            Place::local(f),
            Rvalue::Use(Operand::Constant(Constant::Float(2.5))),
        );
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(f))),
        );
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("2.5"));
    }

    #[test]
    fn test_emit_constant_string_operand() {
        let mut builder = MirBuilder::new("const_str", vec![], MirType::Str);
        let s = builder.new_local(MirType::Str, None);
        builder.assign(
            Place::local(s),
            Rvalue::Use(Operand::Constant(Constant::Str("hello".into()))),
        );
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(s))),
        );
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("hello"));
    }

    #[test]
    fn test_emit_constant_unit_operand() {
        let mut builder = MirBuilder::new("const_unit", vec![], MirType::Unit);
        builder.push_stmt(Statement::Assign(
            Place::local(Local(0)),
            Rvalue::Use(Operand::Constant(Constant::Unit)),
        ));
        builder.return_();
        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);
        let ir = emit_llvm::emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("ret void"));
    }

    // ==========================================================================
    // Full pipeline with struct lowering + emit
    // ==========================================================================

    #[test]
    fn test_full_pipeline_with_struct() {
        let source = r#"
            S Point { x: i64, y: i64 }
            F make_point() -> i64 = 42
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mut mir = lower::lower_module(&module);
        optimize::optimize_mir_module(&mut mir);
        let ir = emit_llvm::emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("%Point = type { i64, i64 }"));
        assert!(ir.contains("define i64 @make_point"));
    }

    #[test]
    fn test_full_pipeline_with_match() {
        let source = r#"
            F dispatch(code: i64) -> i64 = M code {
                0 => 100,
                1 => 200,
                2 => 300,
                _ => 0
            }
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mut mir = lower::lower_module(&module);
        optimize::optimize_mir_module(&mut mir);
        let ir = emit_llvm::emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("define i64 @dispatch"));
        assert!(ir.contains("switch i64") || ir.contains("icmp eq"));
    }

    #[test]
    fn test_optimize_module_level() {
        let source = r#"
            F a(x: i64) -> i64 = x + 1
            F b(x: i64) -> i64 = x * 2
            F c(x: i64) -> i64 = x - 3
        "#;
        let module = vais_parser::parse(source).expect("Parse failed");
        let mut mir = lower::lower_module(&module);
        assert_eq!(mir.bodies.len(), 3);
        optimize::optimize_mir_module(&mut mir);
        // All bodies should still exist after optimization
        assert_eq!(mir.bodies.len(), 3);
    }
}

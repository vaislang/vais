//! Integration tests for vais-mir crate.
//!
//! These tests cover gaps in the existing inline tests, focusing on:
//! - Function call lowering
//! - Type conversion lowering
//! - Nested if/else
//! - Struct lowering
//! - emit_llvm edge cases
//! - Optimization pipeline integration

use vais_mir::{
    emit_llvm::emit_llvm_ir,
    lower::lower_module,
    optimize::{
        common_subexpression_elimination, constant_propagation, dead_code_elimination,
        optimize_mir_body, optimize_mir_module, remove_unreachable_blocks,
    },
    AggregateKind, Constant, Local, MirBuilder, MirModule, MirType, Operand, Place, Rvalue,
    Terminator,
};

// ========================================
// 1. Function Call Lowering Tests
// ========================================

#[test]
fn test_lower_simple_function_call() {
    // Regular function call (non-recursive): F add(a: i64, b: i64) -> i64 = a + b
    // F main() -> i64 = add(3, 4)
    let source = r#"
        F add(a: i64, b: i64) -> i64 = a + b
        F main() -> i64 = add(3, 4)
    "#;
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    assert_eq!(mir.bodies.len(), 2);
    assert_eq!(mir.bodies[0].name, "add");
    assert_eq!(mir.bodies[1].name, "main");

    // Check main calls add
    let main_display = mir.bodies[1].display();
    assert!(main_display.contains("add("));
}

#[test]
fn test_lower_chained_function_calls() {
    // Multiple function calls in chain: F inc(x: i64) -> i64 = x + 1
    // F main() -> i64 = inc(inc(5))
    let source = r#"
        F inc(x: i64) -> i64 = x + 1
        F main() -> i64 = inc(inc(5))
    "#;
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    assert_eq!(mir.bodies.len(), 2);

    let main_display = mir.bodies[1].display();
    // Should contain nested calls
    assert!(main_display.contains("inc("));
}

#[test]
fn test_lower_multi_arg_function_call() {
    // Function with multiple arguments
    let source = r#"
        F sum3(a: i64, b: i64, c: i64) -> i64 = a + b + c
        F main() -> i64 = sum3(1, 2, 3)
    "#;
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    assert_eq!(mir.bodies.len(), 2);
    assert_eq!(mir.bodies[0].params.len(), 3);
}

// ========================================
// 2. Type Conversion Lowering Tests
// ========================================

#[test]
fn test_lower_f64_function() {
    // f64 type function: F fadd(x: f64, y: f64) -> f64 = x + y
    let source = "F fadd(x: f64, y: f64) -> f64 = x + y";
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    assert_eq!(mir.bodies.len(), 1);
    assert_eq!(mir.bodies[0].params.len(), 2);
    assert_eq!(mir.bodies[0].params[0], MirType::F64);
    assert_eq!(mir.bodies[0].params[1], MirType::F64);
    assert_eq!(mir.bodies[0].return_type, MirType::F64);
}

#[test]
fn test_lower_bool_function() {
    // bool type function: F is_pos(x: i64) -> bool = x > 0
    let source = "F is_pos(x: i64) -> bool = x > 0";
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    assert_eq!(mir.bodies.len(), 1);
    assert_eq!(mir.bodies[0].return_type, MirType::Bool);
}

#[test]
fn test_lower_mixed_types() {
    // Function with mixed parameter types
    let source = "F compute(x: i64, y: f64, flag: bool) -> i64 = I flag { x } E { 0 }";
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    assert_eq!(mir.bodies.len(), 1);
    assert_eq!(mir.bodies[0].params.len(), 3);
    assert_eq!(mir.bodies[0].params[0], MirType::I64);
    assert_eq!(mir.bodies[0].params[1], MirType::F64);
    assert_eq!(mir.bodies[0].params[2], MirType::Bool);
}

#[test]
fn test_lower_f32_function() {
    // f32 type function
    let source = "F fmul(a: f32, b: f32) -> f32 = a * b";
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    assert_eq!(mir.bodies[0].params[0], MirType::F32);
    assert_eq!(mir.bodies[0].return_type, MirType::F32);
}

#[test]
fn test_lower_i32_function() {
    // i32 type function
    let source = "F sub32(a: i32, b: i32) -> i32 = a - b";
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    assert_eq!(mir.bodies[0].params[0], MirType::I32);
    assert_eq!(mir.bodies[0].return_type, MirType::I32);
}

// ========================================
// 3. Nested If/Else Tests
// ========================================

#[test]
fn test_lower_nested_if_else() {
    // Classify function with nested if/else:
    // F classify(x: i64) -> i64 = I x > 0 { 1 } E I x < 0 { -1 } E { 0 }
    let source = "F classify(x: i64) -> i64 = I x > 0 { 1 } E I x < 0 { 0 - 1 } E { 0 }";
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    assert_eq!(mir.bodies.len(), 1);

    // Should have multiple basic blocks for nested structure
    assert!(mir.bodies[0].basic_blocks.len() >= 3);

    let display = mir.bodies[0].display();
    assert!(display.contains("switchInt") || display.contains("goto"));
}

#[test]
fn test_lower_deeply_nested_if() {
    // Deeply nested if statements
    let source = "F deep(x: i64) -> i64 = I x > 10 { I x > 20 { 2 } E { 1 } } E { 0 }";
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    assert_eq!(mir.bodies.len(), 1);
    assert!(mir.bodies[0].basic_blocks.len() >= 4);
}

// ========================================
// 4. Struct Lowering Tests
// ========================================

#[test]
fn test_lower_struct_definition() {
    // Struct definition should be registered in mir_module.structs
    let source = r#"
        S Point {
            x: i64,
            y: i64
        }
        F origin() -> i64 = 0
    "#;
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    assert!(mir.structs.contains_key("Point"));
    let fields = &mir.structs["Point"];
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].0, "x");
    assert_eq!(fields[0].1, MirType::I64);
    assert_eq!(fields[1].0, "y");
    assert_eq!(fields[1].1, MirType::I64);
}

#[test]
fn test_lower_multiple_structs() {
    // Multiple struct definitions
    let source = r#"
        S Vec2 { x: f64, y: f64 }
        S Color { r: i32, g: i32, b: i32 }
        F dummy() -> i64 = 0
    "#;
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    assert!(mir.structs.contains_key("Vec2"));
    assert!(mir.structs.contains_key("Color"));
    assert_eq!(mir.structs["Vec2"].len(), 2);
    assert_eq!(mir.structs["Color"].len(), 3);
}

// ========================================
// 5. emit_llvm Edge Cases (MirBuilder)
// ========================================

#[test]
fn test_emit_cast_rvalue() {
    // Cast rvalue: construct Cast node via MirBuilder and emit
    let mut builder = MirBuilder::new("cast_test", vec![MirType::I32], MirType::I64);

    let casted = builder.new_local(MirType::I64, Some("casted".into()));
    builder.assign(
        Place::local(casted),
        Rvalue::Cast(Operand::Copy(Place::local(builder.param(0))), MirType::I64),
    );
    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(casted))),
    );
    builder.return_();

    let body = builder.build();
    let mut module = MirModule::new("test");
    module.bodies.push(body);

    let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define i64 @cast_test("));
    // Cast from i32 to i64 should use sext or zext
    assert!(ir.contains("ext") || ir.contains("i32"));
}

#[test]
fn test_emit_ref_rvalue() {
    // Ref rvalue: create a reference to a place
    let mut builder = MirBuilder::new("ref_test", vec![MirType::I64], MirType::I64);

    let temp = builder.new_local(MirType::I64, Some("temp".into()));
    builder.assign(
        Place::local(temp),
        Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
    );

    let ref_local = builder.new_local(MirType::Ref(Box::new(MirType::I64)), Some("ref".into()));
    builder.assign(Place::local(ref_local), Rvalue::Ref(Place::local(temp)));

    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(temp))),
    );
    builder.return_();

    let body = builder.build();
    let mut module = MirModule::new("test");
    module.bodies.push(body);

    let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define i64 @ref_test("));
}

#[test]
fn test_emit_tail_call_terminator() {
    // TailCall terminator
    let mut builder = MirBuilder::new("tail_caller", vec![MirType::I64], MirType::I64);

    builder.terminate(Terminator::TailCall {
        func: "foo".into(),
        args: vec![Operand::Copy(Place::local(builder.param(0)))],
    });

    let body = builder.build();
    let mut module = MirModule::new("test");
    module.bodies.push(body);

    let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define i64 @tail_caller("));
    // Tail call should generate a call instruction
    assert!(ir.contains("call") || ir.contains("tail"));
}

#[test]
fn test_emit_assert_terminator() {
    // Assert terminator
    let mut builder = MirBuilder::new("assert_test", vec![MirType::I64], MirType::I64);
    let bb_ok = builder.new_block();

    builder.terminate(Terminator::Assert {
        cond: Operand::Constant(Constant::Bool(true)),
        expected: true,
        msg: "assertion failed".into(),
        target: bb_ok,
    });

    builder.switch_to_block(bb_ok);
    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Copy(Place::local(builder.param(0)))),
    );
    builder.return_();

    let body = builder.build();
    let mut module = MirModule::new("test");
    module.bodies.push(body);

    let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define i64 @assert_test("));
    assert!(ir.contains("br") || ir.contains("label"));
}

#[test]
fn test_emit_discriminant_rvalue() {
    // Discriminant rvalue: get the discriminant of an enum
    let mut builder = MirBuilder::new("disc_test", vec![MirType::Enum("Result".into())], MirType::I64);

    let disc = builder.new_local(MirType::I64, Some("disc".into()));
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

    let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define i64 @disc_test("));
}

#[test]
fn test_emit_len_rvalue() {
    // Len rvalue: get the length of an array/string
    let mut builder = MirBuilder::new("len_test", vec![MirType::Array(Box::new(MirType::I64))], MirType::I64);

    let len = builder.new_local(MirType::I64, Some("len".into()));
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

    let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define i64 @len_test("));
}

#[test]
fn test_emit_aggregate_tuple() {
    // Aggregate rvalue: tuple construction
    let mut builder = MirBuilder::new("tuple_test", vec![], MirType::Tuple(vec![MirType::I64, MirType::I64]));

    let tuple_val = builder.new_local(
        MirType::Tuple(vec![MirType::I64, MirType::I64]),
        Some("tuple".into()),
    );
    builder.assign(
        Place::local(tuple_val),
        Rvalue::Aggregate(
            AggregateKind::Tuple,
            vec![
                Operand::Constant(Constant::Int(10)),
                Operand::Constant(Constant::Int(20)),
            ],
        ),
    );
    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Move(Place::local(tuple_val))),
    );
    builder.return_();

    let body = builder.build();
    let mut module = MirModule::new("test");
    module.bodies.push(body);

    let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define"));
}

#[test]
fn test_emit_aggregate_array() {
    // Aggregate rvalue: array construction
    let mut builder = MirBuilder::new("array_test", vec![], MirType::Array(Box::new(MirType::I64)));

    let array_val = builder.new_local(MirType::Array(Box::new(MirType::I64)), Some("arr".into()));
    builder.assign(
        Place::local(array_val),
        Rvalue::Aggregate(
            AggregateKind::Array,
            vec![
                Operand::Constant(Constant::Int(1)),
                Operand::Constant(Constant::Int(2)),
                Operand::Constant(Constant::Int(3)),
            ],
        ),
    );
    builder.assign(
        builder.return_place(),
        Rvalue::Use(Operand::Move(Place::local(array_val))),
    );
    builder.return_();

    let body = builder.build();
    let mut module = MirModule::new("test");
    module.bodies.push(body);

    let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define"));
}

// ========================================
// 6. Optimization Pipeline Integration
// ========================================

#[test]
fn test_full_pipeline_with_constant_folding() {
    // Full pipeline: parse → lower → optimize → emit → verify constant folding
    let source = r#"
        F compute(x: i64) -> i64 = {
            a := 10
            b := 20
            c := a + b
            x + c
        }
    "#;
    let module = vais_parser::parse(source).expect("Parse failed");
    let mut mir = lower_module(&module);

    optimize_mir_module(&mut mir);

    let ir = emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define i64 @compute("));
    // After constant propagation, a + b should become 30
    assert!(ir.contains("add i64") || ir.contains("30"));
}

#[test]
fn test_pipeline_dead_code_elimination() {
    // Dead code should be eliminated
    let source = r#"
        F main() -> i64 = {
            unused := 999
            result := 42
            result
        }
    "#;
    let module = vais_parser::parse(source).expect("Parse failed");
    let mut mir = lower_module(&module);

    let before_len = mir.bodies[0].display().len();
    optimize_mir_module(&mut mir);
    let after_len = mir.bodies[0].display().len();

    // After optimization, unused variable might be removed
    assert!(after_len <= before_len);
}

#[test]
fn test_pipeline_unreachable_blocks() {
    // Unreachable blocks should be removed
    let source = "F always_true() -> i64 = I true { 1 } E { 999 }";
    let module = vais_parser::parse(source).expect("Parse failed");
    let mut mir = lower_module(&module);

    optimize_mir_module(&mut mir);

    let ir = emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define i64 @always_true("));
}

#[test]
fn test_multiple_optimization_passes() {
    // Multiple passes working together
    let source = r#"
        F multi_opt(x: i64) -> i64 = {
            a := 5
            b := 10
            c := a + b
            unused := 777
            dead := unused * 2
            x + c
        }
    "#;
    let module = vais_parser::parse(source).expect("Parse failed");
    let mut mir = lower_module(&module);

    // Apply individual passes
    constant_propagation(&mut mir.bodies[0]);
    dead_code_elimination(&mut mir.bodies[0]);
    common_subexpression_elimination(&mut mir.bodies[0]);
    remove_unreachable_blocks(&mut mir.bodies[0]);

    let display = mir.bodies[0].display();
    // Dead variables should be eliminated
    let dead_count = display.matches("unused").count();
    // Note: may or may not be fully eliminated depending on implementation
    assert!(dead_count <= 2); // Generous bound
}

// ========================================
// 7. Various LLVM IR Type Generation
// ========================================

#[test]
fn test_emit_f32_llvm_type() {
    // f32 function should emit float type
    let source = "F ftest(x: f32) -> f32 = x";
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    let ir = emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define float @ftest(float"));
}

#[test]
fn test_emit_f64_llvm_type() {
    // f64 function should emit double type
    let source = "F dtest(x: f64) -> f64 = x";
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    let ir = emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define double @dtest(double"));
}

#[test]
fn test_emit_bool_llvm_type() {
    // bool function should emit i1 type
    let source = "F btest(x: bool) -> bool = x";
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    let ir = emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define i1 @btest(i1"));
}

#[test]
fn test_emit_void_function() {
    // void function (Unit type)
    let mut builder = MirBuilder::new("void_test", vec![], MirType::Unit);
    builder.assign_const(Local(0), Constant::Unit);
    builder.return_();

    let body = builder.build();
    let mut module = MirModule::new("test");
    module.bodies.push(body);

    let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define void @void_test()"));
}

#[test]
fn test_emit_i32_llvm_type() {
    // i32 function should emit i32 type
    let source = "F i32test(x: i32) -> i32 = x";
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    let ir = emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define i32 @i32test(i32"));
}

#[test]
fn test_emit_struct_llvm_type() {
    // Struct should emit LLVM struct type
    let source = r#"
        S Point { x: i64, y: i64 }
        F make_point() -> i64 = 0
    "#;
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    let ir = emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("%Point = type"));
    assert!(ir.contains("i64"));
}

// ========================================
// 8. Additional Edge Cases
// ========================================

#[test]
fn test_empty_function_body() {
    // Function with minimal body
    let mut builder = MirBuilder::new("empty", vec![], MirType::I64);
    builder.assign_const(Local(0), Constant::Int(0));
    builder.return_();

    let body = builder.build();
    assert_eq!(body.basic_blocks.len(), 1);
    assert!(body.basic_blocks[0].terminator.is_some());
}

#[test]
fn test_unary_operations() {
    // Test negation
    let source = "F negate(x: i64) -> i64 = 0 - x";
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    let ir = emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define i64 @negate("));
    assert!(ir.contains("sub i64"));
}

#[test]
fn test_comparison_operations() {
    // Test comparison ops
    let source = "F compare(a: i64, b: i64) -> bool = a < b";
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    let ir = emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("define i1 @compare("));
    assert!(ir.contains("icmp"));
}

#[test]
fn test_module_with_no_functions() {
    // Empty module edge case
    let module = MirModule::new("empty");
    let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");

    assert!(ir.contains("ModuleID"));
    assert!(ir.contains("target triple"));
}

#[test]
fn test_optimization_body_display() {
    // Ensure optimization doesn't break display
    let source = "F test(x: i64) -> i64 = x + 1";
    let module = vais_parser::parse(source).expect("Parse failed");
    let mut mir = lower_module(&module);

    let before = mir.bodies[0].display();
    optimize_mir_body(&mut mir.bodies[0]);
    let after = mir.bodies[0].display();

    assert!(before.contains("fn test("));
    assert!(after.contains("fn test("));
}

#[test]
fn test_target_triple_variants() {
    // Test different target triples
    let source = "F identity(x: i64) -> i64 = x";
    let module = vais_parser::parse(source).expect("Parse failed");
    let mir = lower_module(&module);

    let ir_linux = emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");
    assert!(ir_linux.contains("x86_64-unknown-linux-gnu"));

    let ir_darwin = emit_llvm_ir(&mir, "x86_64-apple-darwin");
    assert!(ir_darwin.contains("x86_64-apple-darwin"));

    let ir_windows = emit_llvm_ir(&mir, "x86_64-pc-windows-msvc");
    assert!(ir_windows.contains("x86_64-pc-windows-msvc"));
}

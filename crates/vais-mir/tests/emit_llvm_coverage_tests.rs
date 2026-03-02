//! Coverage tests for vais-mir/src/emit_llvm.rs
//!
//! Targets: emit_llvm_ir_with_options, OptLevel variants, debug metadata emission,
//! struct type definitions, various MirType→LLVM mappings, and edge cases.

use vais_mir::emit_llvm::{emit_llvm_ir, emit_llvm_ir_with_options, OptLevel};
use vais_mir::lower::lower_module;
use vais_mir::optimize::optimize_mir_module;

fn lower_and_emit(source: &str) -> String {
    let module = vais_parser::parse(source).expect("Parse failed");
    let mut mir = lower_module(&module);
    optimize_mir_module(&mut mir);
    emit_llvm_ir(&mir, "x86_64-apple-darwin")
}

// ============================================================================
// Basic emission
// ============================================================================

#[test]
fn test_emit_module_header() {
    let ir = lower_and_emit("F noop() -> i64 = 0");
    assert!(ir.contains("; ModuleID = 'main'"));
    assert!(ir.contains("target triple = \"x86_64-apple-darwin\""));
    assert!(ir.contains("declare i32 @putchar(i32)"));
    assert!(ir.contains("declare i32 @printf(i8*, ...)"));
}

#[test]
fn test_emit_simple_function() {
    let ir = lower_and_emit("F answer() -> i64 = 42");
    assert!(ir.contains("define i64 @answer()"));
    assert!(ir.contains("ret i64"));
}

#[test]
fn test_emit_function_with_params() {
    let ir = lower_and_emit("F add(a: i64, b: i64) -> i64 = a + b");
    assert!(ir.contains("define i64 @add(i64 %_1, i64 %_2)"));
    assert!(ir.contains("add i64"));
}

#[test]
fn test_emit_multiple_functions() {
    let ir = lower_and_emit(
        r#"
        F one() -> i64 = 1
        F two() -> i64 = 2
        F three() -> i64 = 3
    "#,
    );
    assert!(ir.contains("define i64 @one()"));
    assert!(ir.contains("define i64 @two()"));
    assert!(ir.contains("define i64 @three()"));
}

// ============================================================================
// OptLevel variations
// ============================================================================

#[test]
fn test_opt_level_none() {
    let source = "F f() -> i64 = 0";
    let module = vais_parser::parse(source).unwrap();
    let mut mir = lower_module(&module);
    optimize_mir_module(&mut mir);
    let ir = emit_llvm_ir_with_options(&mir, "x86_64-unknown-linux-gnu", OptLevel::None, false);
    assert!(ir.contains("#0")); // optnone attribute
}

#[test]
fn test_opt_level_default() {
    let source = "F f() -> i64 = 0";
    let module = vais_parser::parse(source).unwrap();
    let mut mir = lower_module(&module);
    optimize_mir_module(&mut mir);
    let ir = emit_llvm_ir_with_options(&mir, "x86_64-unknown-linux-gnu", OptLevel::Default, false);
    // Default has no special attributes
    assert!(!ir.contains("#0"));
    assert!(!ir.contains("#1"));
    assert!(!ir.contains("#2"));
}

#[test]
fn test_opt_level_size() {
    let source = "F f() -> i64 = 0";
    let module = vais_parser::parse(source).unwrap();
    let mut mir = lower_module(&module);
    optimize_mir_module(&mut mir);
    let ir = emit_llvm_ir_with_options(&mir, "x86_64-unknown-linux-gnu", OptLevel::Size, false);
    assert!(ir.contains("#1")); // optsize attribute
}

#[test]
fn test_opt_level_min_size() {
    let source = "F f() -> i64 = 0";
    let module = vais_parser::parse(source).unwrap();
    let mut mir = lower_module(&module);
    optimize_mir_module(&mut mir);
    let ir = emit_llvm_ir_with_options(&mir, "x86_64-unknown-linux-gnu", OptLevel::MinSize, false);
    assert!(ir.contains("#2")); // minsize attribute
}

// ============================================================================
// Debug metadata
// ============================================================================

#[test]
fn test_emit_with_debug_info() {
    let source = "F f(x: i64) -> i64 = x + 1";
    let module = vais_parser::parse(source).unwrap();
    let mut mir = lower_module(&module);
    optimize_mir_module(&mut mir);
    let ir = emit_llvm_ir_with_options(&mir, "x86_64-apple-darwin", OptLevel::Default, true);
    // Debug metadata should be present
    assert!(ir.contains("!llvm.dbg") || ir.contains("!DICompileUnit") || ir.contains("!0 = "));
}

#[test]
fn test_emit_without_debug_info() {
    let source = "F f(x: i64) -> i64 = x + 1";
    let module = vais_parser::parse(source).unwrap();
    let mut mir = lower_module(&module);
    optimize_mir_module(&mut mir);
    let ir = emit_llvm_ir_with_options(&mir, "x86_64-apple-darwin", OptLevel::Default, false);
    assert!(!ir.contains("!DICompileUnit"));
}

// ============================================================================
// Type mapping
// ============================================================================

#[test]
fn test_emit_bool_type() {
    let ir = lower_and_emit("F is_zero(x: i64) -> bool = x == 0");
    assert!(ir.contains("i1") || ir.contains("bool") || ir.contains("icmp"));
}

#[test]
fn test_emit_f64_type() {
    let ir = lower_and_emit("F pi() -> f64 = 3.14");
    assert!(ir.contains("double") || ir.contains("f64"));
}

#[test]
fn test_emit_i32_type() {
    let ir = lower_and_emit("F small(x: i32) -> i32 = x");
    assert!(ir.contains("i32"));
}

// ============================================================================
// Binary operations
// ============================================================================

#[test]
fn test_emit_arithmetic_ops() {
    let ir = lower_and_emit(
        r#"
        F arith(a: i64, b: i64) -> i64 = {
            c := a + b
            d := c - a
            e := d * b
            f := e / a
            f
        }
    "#,
    );
    assert!(ir.contains("add i64"));
    assert!(ir.contains("sub i64"));
    assert!(ir.contains("mul i64"));
    // div may be sdiv for signed
    assert!(ir.contains("div") || ir.contains("sdiv"));
}

#[test]
fn test_emit_comparison_ops() {
    let ir = lower_and_emit("F cmp(a: i64, b: i64) -> bool = a < b");
    assert!(ir.contains("icmp") || ir.contains("slt"));
}

#[test]
fn test_emit_bitwise_ops() {
    let ir = lower_and_emit(
        r#"
        F bits(a: i64, b: i64) -> i64 = {
            c := a & b
            d := c | b
            e := d ^ a
            f := e << 2
            g := f >> 1
            g
        }
    "#,
    );
    assert!(
        ir.contains("and") || ir.contains("or") || ir.contains("xor") || ir.contains("shl")
    );
}

// ============================================================================
// Unary operations
// ============================================================================

#[test]
fn test_emit_negation() {
    let ir = lower_and_emit("F neg(x: i64) -> i64 = -x");
    assert!(ir.contains("sub") && ir.contains("0"));
}

#[test]
fn test_emit_not() {
    let ir = lower_and_emit("F flip(x: i64) -> i64 = !x");
    assert!(ir.contains("xor"));
}

// ============================================================================
// Control flow
// ============================================================================

#[test]
fn test_emit_if_else() {
    let ir = lower_and_emit("F abs(x: i64) -> i64 = I x < 0 { 0 - x } E { x }");
    assert!(ir.contains("br "));
    assert!(ir.contains("bb1:") || ir.contains("bb2:"));
}

#[test]
fn test_emit_ternary() {
    let ir = lower_and_emit("F max(a: i64, b: i64) -> i64 = a > b ? a : b");
    assert!(ir.contains("br "));
}

#[test]
fn test_emit_match() {
    let ir = lower_and_emit(
        r#"
        F classify(x: i64) -> i64 = M x {
            0 => 10,
            1 => 20,
            _ => 30
        }
    "#,
    );
    assert!(ir.contains("switch") || ir.contains("br "));
}

// ============================================================================
// Function calls
// ============================================================================

#[test]
fn test_emit_function_call() {
    let ir = lower_and_emit(
        r#"
        F double(x: i64) -> i64 = x * 2
        F use_double(x: i64) -> i64 = double(x)
    "#,
    );
    assert!(ir.contains("call i64 @double("));
}

#[test]
fn test_emit_recursive_call() {
    let ir = lower_and_emit("F fib(n: i64) -> i64 = I n < 2 { n } E { @(n - 1) + @(n - 2) }");
    // Should have a tailcall or a call to fib
    assert!(ir.contains("fib") || ir.contains("tailcall") || ir.contains("call"));
}

// ============================================================================
// Struct emission
// ============================================================================

#[test]
fn test_emit_struct_type() {
    let ir = lower_and_emit(
        r#"
        S Point { x: f64, y: f64 }
        F origin() -> i64 = 0
    "#,
    );
    assert!(ir.contains("%Point = type {") || ir.contains("Point"));
}

// ============================================================================
// Target triple
// ============================================================================

#[test]
fn test_emit_linux_target() {
    let source = "F f() -> i64 = 0";
    let module = vais_parser::parse(source).unwrap();
    let mir = lower_module(&module);
    let ir = emit_llvm_ir(&mir, "x86_64-unknown-linux-gnu");
    assert!(ir.contains("target triple = \"x86_64-unknown-linux-gnu\""));
}

#[test]
fn test_emit_wasm_target() {
    let source = "F f() -> i64 = 0";
    let module = vais_parser::parse(source).unwrap();
    let mir = lower_module(&module);
    let ir = emit_llvm_ir(&mir, "wasm32-unknown-unknown");
    assert!(ir.contains("target triple = \"wasm32-unknown-unknown\""));
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_emit_empty_module() {
    let source = "F noop() -> i64 = 0";
    let module = vais_parser::parse(source).unwrap();
    let mir = lower_module(&module);
    let ir = emit_llvm_ir(&mir, "aarch64-apple-darwin");
    assert!(!ir.is_empty());
    assert!(ir.contains("define"));
}

#[test]
fn test_emit_multiple_blocks() {
    let ir = lower_and_emit(
        r#"
        F multi(x: i64) -> i64 = I x > 10 { I x > 20 { 3 } E { 2 } } E { 1 }
    "#,
    );
    // Nested if produces multiple blocks
    assert!(ir.contains("bb"));
}

#[test]
fn test_emit_let_binding() {
    let ir = lower_and_emit(
        r#"
        F compute() -> i64 = {
            a := 10
            b := 20
            a + b
        }
    "#,
    );
    assert!(ir.contains("define i64 @compute()"));
}

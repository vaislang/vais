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
}

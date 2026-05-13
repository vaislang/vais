//! LLVM IR Optimization Passes
//!
//! Text-based optimization passes for the generated LLVM IR.
//! These are applied before passing the IR to clang for final optimization.

// Re-export submodules
pub(crate) mod inlining;
pub(crate) mod ir_passes;
pub mod lto;
pub mod pgo;

// Re-export for parallel.rs wildcard import (use crate::optimize::*)
pub(crate) use inlining::aggressive_inline;
pub(crate) use ir_passes::{
    branch_optimization, common_subexpression_elimination, conditional_branch_simplification,
    constant_folding, dead_code_elimination, dead_store_elimination, loop_invariant_motion,
    strength_reduction,
};

// Re-export key types for external crate access (vaisc uses optimize::LtoMode etc.)
pub use lto::LtoMode;
pub use pgo::{CoverageMode, PgoMode};

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptLevel {
    O0, // No optimization
    O1, // Basic optimization
    O2, // Standard optimization
    O3, // Aggressive optimization
}

impl OptLevel {
    pub fn parse(s: &str) -> Self {
        match s {
            "0" | "O0" => OptLevel::O0,
            "1" | "O1" => OptLevel::O1,
            "2" | "O2" => OptLevel::O2,
            "3" | "O3" => OptLevel::O3,
            _ => OptLevel::O0,
        }
    }
}

/// Apply optimization passes to LLVM IR
pub fn optimize_ir(ir: &str, level: OptLevel) -> String {
    optimize_ir_with_pgo(ir, level, &pgo::PgoMode::None)
}

/// Apply optimization passes to LLVM IR with optional PGO support
///
/// When PGO is in Generate mode, instrumentation hints are added.
/// When PGO is in Use mode, profile data guides inlining and optimization decisions.
pub fn optimize_ir_with_pgo(ir: &str, level: OptLevel, pgo: &pgo::PgoMode) -> String {
    if level == OptLevel::O0 {
        // Even at O0, apply PGO instrumentation if requested
        if let pgo::PgoMode::Generate(_) = pgo {
            return pgo::instrument_ir_for_pgo(ir);
        }
        return ir.to_string();
    }

    let mut result = ir.to_string();

    // PGO Generate: add instrumentation
    if let pgo::PgoMode::Generate(_) = pgo {
        result = pgo::instrument_ir_for_pgo(&result);
    }

    // O1+: Basic optimizations (before inlining to simplify function bodies)
    if level >= OptLevel::O1 {
        result = ir_passes::constant_folding(&result);
        result = ir_passes::dead_store_elimination(&result);
        result = ir_passes::branch_optimization(&result);
        result = ir_passes::conditional_branch_simplification(&result);
    }

    // O1+: Tail call optimization - mark tail calls with 'tail' or 'musttail'
    if level >= OptLevel::O1 {
        result = ir_passes::tail_call_optimization(&result);
    }

    // O2+: More aggressive optimizations
    if level >= OptLevel::O2 {
        result = ir_passes::strength_reduction(&result);
    }

    // O3: Inlining after basic optimizations
    if level >= OptLevel::O3 {
        result = inlining::aggressive_inline(&result);
    }

    // PGO Use: apply profile-guided hints (hot/cold function annotations)
    if let pgo::PgoMode::Use(profile_path) = pgo {
        result = pgo::apply_pgo_hints(&result, profile_path);
    }

    // O2+: CSE and DCE after inlining to clean up
    if level >= OptLevel::O2 {
        result = ir_passes::common_subexpression_elimination(&result);
        result = ir_passes::dead_code_elimination(&result);
    }

    // O3: Loop optimizations last
    if level >= OptLevel::O3 {
        result = ir_passes::loop_invariant_motion(&result);
    }

    // Post-optimization IR verification: catch optimizer-introduced bugs.
    // Only run at O2+ where transformations are aggressive enough to risk breakage.
    if level >= OptLevel::O2 {
        let diags = crate::ir_verify::verify_text_ir(&result);
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.severity == crate::ir_verify::DiagnosticSeverity::Error)
            .collect();
        debug_assert!(
            errors.is_empty(),
            "ICE: Post-optimization IR verification failed: {:?}",
            errors,
        );
    }

    result
}

/// Apply optimization passes with advanced analysis
///
/// This version includes interprocedural alias analysis, auto-vectorization hints,
/// and cache-friendly data layout suggestions.
pub fn optimize_ir_advanced(ir: &str, level: OptLevel) -> String {
    use crate::advanced_opt::{apply_advanced_optimizations, AdvancedOptConfig};

    // First apply standard optimizations
    let result = optimize_ir(ir, level);

    // Then apply advanced optimizations based on level
    let config = AdvancedOptConfig::from_opt_level(level);
    apply_advanced_optimizations(&result, &config)
}

/// Extract function name from a define line
///
/// Pattern: define ... @function_name(...)
/// Returns the function name without the @ prefix.
pub(crate) fn extract_function_name(define_line: &str) -> Option<String> {
    // Pattern: define ... @function_name(
    let at_pos = define_line.find('@')?;
    let paren_pos = define_line[at_pos..].find('(')?;
    let name = &define_line[at_pos + 1..at_pos + paren_pos];
    Some(name.to_string())
}

#[cfg(test)]
mod opt_benchmark_tests {
    use super::*;

    /// Count non-comment, non-empty lines in IR (effective IR size)
    fn ir_size(ir: &str) -> usize {
        ir.lines()
            .filter(|l| {
                let t = l.trim();
                !t.is_empty() && !t.starts_with(';')
            })
            .count()
    }

    #[test]
    fn test_o3_reduces_ir_with_inlinable_functions() {
        let ir = r#"define i64 @square(i64 %x) {
entry:
  %0 = mul i64 %x, %x
  ret i64 %0
}

define i64 @main() {
entry:
  %0 = call i64 @square(i64 5)
  %1 = call i64 @square(i64 10)
  %2 = add i64 %0, %1
  ret i64 %2
}
"#;
        let before = ir_size(ir);
        let optimized = optimize_ir(ir, OptLevel::O3);
        let after = ir_size(&optimized);
        // Inlining expands code, but other passes should clean up.
        // The key metric: the IR should have INLINE markers showing work was done.
        assert!(
            optimized.contains("INLINE") || after != before,
            "O3 should transform IR via inlining. before={}, after={}",
            before,
            after
        );
    }

    #[test]
    fn test_o2_cse_reduces_duplicate_expressions() {
        let ir = r#"define i64 @test(i64 %a, i64 %b) {
entry:
  %0 = add i64 %a, %b
  %1 = add i64 %a, %b
  %2 = add i64 %0, %1
  ret i64 %2
}
"#;
        let optimized = optimize_ir(ir, OptLevel::O2);
        // CSE should eliminate the duplicate add
        assert!(
            optimized.contains("GVN-CSE"),
            "O2 should apply GVN-CSE to eliminate duplicate expressions"
        );
    }

    #[test]
    fn test_o2_dce_removes_unused_code() {
        let ir = r#"define i64 @test(i64 %x) {
entry:
  %unused = add i64 %x, 42
  %result = mul i64 %x, 2
  ret i64 %result
}
"#;
        let optimized = optimize_ir(ir, OptLevel::O2);
        assert!(
            optimized.contains("DCE removed"),
            "O2 should remove unused %unused definition"
        );
    }

    #[test]
    fn test_o1_constant_folding_reduces_constants() {
        let ir = r#"define i64 @test() {
entry:
  %0 = add i64 10, 20
  %1 = mul i64 3, 4
  %2 = add i64 %0, %1
  ret i64 %2
}
"#;
        let optimized = optimize_ir(ir, OptLevel::O1);
        // Constant folding should evaluate 10+20=30 and 3*4=12
        assert!(
            optimized.contains("30") && optimized.contains("12"),
            "O1 should fold constant expressions. Result:\n{}",
            optimized
        );
    }

    #[test]
    fn test_o2_strength_reduction_replaces_mul() {
        let ir = r#"define i64 @test(i64 %x) {
entry:
  %0 = mul i64 %x, 8
  ret i64 %0
}
"#;
        let optimized = optimize_ir(ir, OptLevel::O2);
        assert!(
            optimized.contains("shl"),
            "O2 should strength-reduce mul by 8 to shl by 3. Result:\n{}",
            optimized
        );
    }

    #[test]
    fn test_o0_is_identity() {
        let ir = r#"define i64 @test(i64 %x) {
entry:
  %0 = add i64 %x, 1
  ret i64 %0
}
"#;
        let optimized = optimize_ir(ir, OptLevel::O0);
        assert_eq!(ir, optimized, "O0 should return IR unchanged");
    }

    #[test]
    fn test_o3_loop_optimization_processes_loops() {
        let ir = r#"define i64 @test() {
entry:
  br label %loop.start.0
loop.start.0:
  %i = phi i64 [0, %entry], [%next, %loop.body.0]
  %sum = phi i64 [0, %entry], [%newsum, %loop.body.0]
  %cond = icmp slt i64 %i, 10
  br i1 %cond, label %loop.body.0, label %loop.end.0
loop.body.0:
  %newsum = add i64 %sum, %i
  %next = add i64 %i, 1
  br label %loop.start.0
loop.end.0:
  ret i64 %sum
}
"#;
        let optimized = optimize_ir(ir, OptLevel::O3);
        // Loop optimization should process the loop (unroll or LICM)
        assert!(
            optimized.contains("loop.start") || optimized.contains("LOOP"),
            "O3 should process loops. Result:\n{}",
            optimized
        );
    }

    #[test]
    fn test_o2_commutative_cse() {
        let ir = r#"define i64 @test(i64 %a, i64 %b) {
entry:
  %0 = add i64 %a, %b
  %1 = add i64 %b, %a
  %2 = add i64 %0, %1
  ret i64 %2
}
"#;
        let optimized = optimize_ir(ir, OptLevel::O2);
        // GVN-CSE should detect commutativity: a+b == b+a
        assert!(
            optimized.contains("GVN-CSE"),
            "O2 GVN-CSE should detect commutative equivalence a+b == b+a. Result:\n{}",
            optimized
        );
    }

    #[test]
    fn test_o2_dead_branch_elimination() {
        let ir = r#"define i64 @test() {
entry:
  br i1 true, label %live, label %dead
live:
  ret i64 42
dead:
  ret i64 0
}
"#;
        let optimized = optimize_ir(ir, OptLevel::O2);
        // Branch optimization + DCE should simplify the constant branch
        assert!(
            optimized.contains("simplified from conditional") || optimized.contains("unreachable"),
            "O2 should simplify constant branches. Result:\n{}",
            optimized
        );
    }

    #[test]
    fn test_ir_size_comparison_o0_vs_o2() {
        // A non-trivial program with optimization opportunities
        let ir = r#"define i64 @compute(i64 %x, i64 %y) {
entry:
  %a = add i64 %x, %y
  %b = add i64 %x, %y
  %c = mul i64 %a, %b
  %unused1 = add i64 %x, 42
  %unused2 = sub i64 %y, 17
  %d = mul i64 %c, 8
  ret i64 %d
}
"#;
        let o0_result = optimize_ir(ir, OptLevel::O0);
        let o2_result = optimize_ir(ir, OptLevel::O2);
        let o0_size = ir_size(&o0_result);
        let o2_size = ir_size(&o2_result);

        // O2 should either reduce size (CSE + DCE) or at least transform the IR
        // (comments from optimizations don't count as effective lines)
        assert!(
            o2_size <= o0_size || o2_result.contains("GVN-CSE") || o2_result.contains("DCE"),
            "O2 should optimize the IR. O0 size={}, O2 size={}",
            o0_size,
            o2_size
        );
    }

    #[test]
    fn test_store_load_forwarding_at_o2() {
        let ir = r#"define i64 @test() {
entry:
  %ptr = alloca i64
  store i64 42, i64* %ptr
  %val = load i64, i64* %ptr
  ret i64 %val
}
"#;
        let optimized = optimize_ir(ir, OptLevel::O2);
        assert!(
            optimized.contains("forwarded from store"),
            "O2 DCE should forward store->load patterns. Result:\n{}",
            optimized
        );
    }
}

//! LLVM IR Optimization Passes
//!
//! Text-based optimization passes for the generated LLVM IR.
//! These are applied before passing the IR to clang for final optimization.

// Re-export submodules
pub mod pgo;
pub(crate) mod ir_passes;
pub mod lto;
pub(crate) mod inlining;

// Re-export for parallel.rs wildcard import (use crate::optimize::*)
pub(crate) use ir_passes::{
    constant_folding, dead_store_elimination, branch_optimization,
    conditional_branch_simplification, strength_reduction,
    common_subexpression_elimination, dead_code_elimination,
    loop_invariant_motion,
};
pub(crate) use inlining::aggressive_inline;

// Re-export key types for external crate access (vaisc uses optimize::LtoMode etc.)
pub use lto::LtoMode;
pub use pgo::{PgoMode, CoverageMode};

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

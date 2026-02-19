#![allow(dead_code)] // Tiered JIT reserved for production use
//! Tiered JIT compilation system.
//!
//! Implements a multi-tier compilation strategy:
//! - Tier 0: Interpreter (for initial execution and profiling)
//! - Tier 1: Baseline JIT (fast compilation, minimal optimization)
//! - Tier 2: Optimizing JIT (slow compilation, full optimization)
//!
//! Hot functions are automatically promoted to higher tiers based on
//! execution count thresholds.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

pub mod interpreter;
pub mod jit;
pub mod value;

pub use interpreter::*;
pub use jit::*;
pub use value::*;

#[cfg(test)]
mod tests;

/// Compilation tier levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    /// Interpreter: No compilation, direct AST evaluation.
    Interpreter = 0,
    /// Baseline JIT: Fast compile, minimal optimization.
    Baseline = 1,
    /// Optimizing JIT: Slow compile, full optimization.
    Optimizing = 2,
}

impl Tier {
    /// Returns the name of this tier.
    pub fn name(&self) -> &'static str {
        match self {
            Tier::Interpreter => "Interpreter",
            Tier::Baseline => "Baseline JIT",
            Tier::Optimizing => "Optimizing JIT",
        }
    }
}

/// Threshold configuration for tier promotion.
#[derive(Debug, Clone)]
pub struct TierThresholds {
    /// Execution count to promote from Interpreter to Baseline.
    pub interpreter_to_baseline: u64,
    /// Execution count to promote from Baseline to Optimizing.
    pub baseline_to_optimizing: u64,
}

impl Default for TierThresholds {
    fn default() -> Self {
        Self {
            interpreter_to_baseline: 100,
            baseline_to_optimizing: 10_000,
        }
    }
}

/// Function profiling data.
#[derive(Debug)]
pub struct FunctionProfile {
    /// Total execution count.
    pub execution_count: AtomicU64,
    /// Current compilation tier.
    pub current_tier: RwLock<Tier>,
    /// Is compilation in progress for next tier?
    pub compiling: RwLock<bool>,
    /// Loop iteration counts for hot loop detection.
    pub loop_counts: RwLock<HashMap<usize, u64>>,
    /// Branch taken/not-taken counts for branch prediction.
    pub branch_counts: RwLock<HashMap<usize, (u64, u64)>>,
    /// Total accumulated loop iterations across all loops.
    pub total_loop_iterations: AtomicU64,
    /// Hot path score (weighted by loop iterations + branch bias).
    pub hot_path_score: RwLock<f64>,
    /// Last promoted execution count (to prevent rapid re-promotions).
    pub last_promoted_at: RwLock<u64>,
    /// Deoptimization count (tier downgrades due to mismatched assumptions).
    pub deopt_count: AtomicU64,
}

impl FunctionProfile {
    /// Creates a new function profile.
    pub fn new() -> Self {
        Self {
            execution_count: AtomicU64::new(0),
            current_tier: RwLock::new(Tier::Interpreter),
            compiling: RwLock::new(false),
            loop_counts: RwLock::new(HashMap::new()),
            branch_counts: RwLock::new(HashMap::new()),
            total_loop_iterations: AtomicU64::new(0),
            hot_path_score: RwLock::new(0.0),
            last_promoted_at: RwLock::new(0),
            deopt_count: AtomicU64::new(0),
        }
    }

    /// Increments execution count and returns the new value.
    pub fn increment_execution(&self) -> u64 {
        self.execution_count.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Records a loop iteration.
    pub fn record_loop(&self, loop_id: usize) {
        let mut counts = self.loop_counts.write().unwrap_or_else(|e| e.into_inner());
        *counts.entry(loop_id).or_insert(0) += 1;
        self.total_loop_iterations.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a branch outcome.
    pub fn record_branch(&self, branch_id: usize, taken: bool) {
        let mut counts = self
            .branch_counts
            .write()
            .unwrap_or_else(|e| e.into_inner());
        let entry = counts.entry(branch_id).or_insert((0, 0));
        if taken {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
    }

    /// Updates the hot path score based on execution profile.
    pub fn update_hot_path_score(&self) {
        let score = compute_hot_path_score(self);
        let mut hot_path_score = self
            .hot_path_score
            .write()
            .unwrap_or_else(|e| e.into_inner());
        *hot_path_score = score;
    }

    /// Records a deoptimization event.
    pub fn record_deopt(&self) -> u64 {
        self.deopt_count.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Checks if function is blacklisted from future promotions.
    pub fn is_blacklisted(&self) -> bool {
        self.deopt_count.load(Ordering::Relaxed) >= 3
    }

    /// Marks function as promoted at current execution count.
    pub fn mark_promoted(&self) {
        let count = self.execution_count.load(Ordering::Relaxed);
        let mut last_promoted = self
            .last_promoted_at
            .write()
            .unwrap_or_else(|e| e.into_inner());
        *last_promoted = count;
    }
}

impl Default for FunctionProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes hot path score based on profiling data.
///
/// Score = execution_count * 1.0 + total_loop_iterations * 0.5 + branch_bias_score * 0.3
pub(crate) fn compute_hot_path_score(profile: &FunctionProfile) -> f64 {
    let execution_count = profile.execution_count.load(Ordering::Relaxed) as f64;
    let total_loop_iterations = profile.total_loop_iterations.load(Ordering::Relaxed) as f64;

    // Calculate branch bias score (max bias across all branches)
    let branch_bias_score = {
        let branch_counts = profile
            .branch_counts
            .read()
            .unwrap_or_else(|e| e.into_inner());
        branch_counts
            .values()
            .map(|(taken, not_taken)| {
                let total = taken + not_taken;
                if total == 0 {
                    0.0
                } else {
                    let max_count = (*taken).max(*not_taken);
                    (max_count as f64) / (total as f64)
                }
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    };

    execution_count * 1.0 + total_loop_iterations * 0.5 + branch_bias_score * 0.3
}

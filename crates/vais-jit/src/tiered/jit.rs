use super::*;
use std::collections::HashMap;
use std::sync::RwLock;
use vais_ast::Module as AstModule;
use crate::JitError;

/// OSR (On-Stack Replacement) point for loop hot spot replacement.
#[derive(Debug, Clone)]
pub struct OsrPoint {
    /// Function name.
    pub function: String,
    /// Loop identifier.
    pub loop_id: usize,
    /// Target tier to promote to.
    pub target_tier: Tier,
    /// Iteration threshold to trigger OSR.
    pub iteration_threshold: u64,
}

/// Tiered JIT compiler orchestrator.
pub struct TieredJit {
    /// Interpreter for Tier 0.
    pub interpreter: Interpreter,
    /// Baseline JIT compiler for Tier 1.
    baseline: Option<crate::JitCompiler>,
    /// Compiled function pointers by tier.
    compiled: HashMap<String, HashMap<Tier, *const u8>>,
    /// Tier thresholds.
    thresholds: TierThresholds,
    /// OSR points registered for hot loop replacement.
    pub(crate) osr_points: RwLock<Vec<OsrPoint>>,
}

impl TieredJit {
    /// Creates a new tiered JIT compiler.
    pub fn new() -> Result<Self, JitError> {
        Ok(Self {
            interpreter: Interpreter::new(),
            baseline: Some(crate::JitCompiler::new()?),
            compiled: HashMap::new(),
            thresholds: TierThresholds::default(),
            osr_points: RwLock::new(Vec::new()),
        })
    }

    /// Creates a tiered JIT with custom thresholds.
    pub fn with_thresholds(thresholds: TierThresholds) -> Result<Self, JitError> {
        Ok(Self {
            interpreter: Interpreter::with_thresholds(thresholds.clone()),
            baseline: Some(crate::JitCompiler::new()?),
            compiled: HashMap::new(),
            thresholds,
            osr_points: RwLock::new(Vec::new()),
        })
    }

    /// Loads a module.
    pub fn load_module(&mut self, module: &AstModule) {
        self.interpreter.load_module(module);
    }

    /// Runs the main function with tiered compilation.
    pub fn run_main(&mut self, module: &AstModule) -> Result<i64, JitError> {
        self.load_module(module);

        // Start with interpreter
        let result = self.interpreter.run_main()?;

        // Check for tier promotion
        self.check_promotions()?;

        result.as_i64()
    }

    /// Checks and performs tier promotions.
    fn check_promotions(&mut self) -> Result<(), JitError> {
        let functions: Vec<String> = self.interpreter.functions.keys().cloned().collect();

        for name in functions {
            if let Some(new_tier) = self.interpreter.should_promote(&name) {
                self.promote_function(&name, new_tier)?;
            }
        }

        Ok(())
    }

    /// Promotes a function to a higher tier.
    fn promote_function(&mut self, name: &str, tier: Tier) -> Result<(), JitError> {
        let profile = match self.interpreter.get_profile(name) {
            Some(p) => p,
            None => return Ok(()),
        };

        // Check if already compiling
        {
            let compiling = profile.compiling.read().unwrap_or_else(|e| e.into_inner());
            if *compiling {
                return Ok(());
            }
        }

        // Mark as compiling
        {
            let mut compiling = profile.compiling.write().unwrap_or_else(|e| e.into_inner());
            *compiling = true;
        }

        // Perform compilation based on tier
        match tier {
            Tier::Baseline => {
                // Compile with baseline settings (fast, minimal optimization)
                if let Some(ref mut jit) = self.baseline {
                    if let Some(func) = self.interpreter.functions.get(name) {
                        // Use existing JIT compiler with default (speed) optimization
                        let ast_module = vais_ast::Module {
                            items: vec![vais_ast::Spanned {
                                node: vais_ast::Item::Function(func.clone()),
                                span: Default::default(),
                            }],
                            modules_map: None,
                        };
                        jit.compile_module(&ast_module)?;
                    }
                }
            }
            Tier::Optimizing => {
                // Compile with full optimization
                // Use profiling data to guide optimization
                if let Some(ref mut jit) = self.baseline {
                    if let Some(func) = self.interpreter.functions.get(name) {
                        let ast_module = vais_ast::Module {
                            items: vec![vais_ast::Spanned {
                                node: vais_ast::Item::Function(func.clone()),
                                span: Default::default(),
                            }],
                            modules_map: None,
                        };
                        jit.compile_module(&ast_module)?;
                    }
                }
            }
            Tier::Interpreter => {
                // No promotion needed
            }
        }

        // Update tier and mark as promoted
        {
            let mut current_tier = profile
                .current_tier
                .write()
                .unwrap_or_else(|e| e.into_inner());
            *current_tier = tier;
            profile.mark_promoted();
        }

        // Clear compiling flag
        {
            let mut compiling = profile.compiling.write().unwrap_or_else(|e| e.into_inner());
            *compiling = false;
        }

        Ok(())
    }

    /// Deoptimizes a function by downgrading it to a lower tier.
    pub fn deoptimize(&mut self, name: &str) -> Result<(), JitError> {
        let profile = match self.interpreter.get_profile(name) {
            Some(p) => p,
            None => return Ok(()),
        };

        let current_tier = *profile
            .current_tier
            .read()
            .unwrap_or_else(|e| e.into_inner());

        // Determine downgrade target
        let new_tier = match current_tier {
            Tier::Optimizing => Tier::Baseline,
            Tier::Baseline => Tier::Interpreter,
            Tier::Interpreter => return Ok(()), // Already at lowest tier
        };

        // Record deoptimization
        let deopt_count = profile.record_deopt();

        // Update tier
        {
            let mut tier = profile
                .current_tier
                .write()
                .unwrap_or_else(|e| e.into_inner());
            *tier = new_tier;
        }

        // Remove compiled code for the old tier
        if let Some(tiers) = self.compiled.get_mut(name) {
            tiers.remove(&current_tier);
        }

        // Check if blacklisted after 3 deopts
        if deopt_count >= 3 {
            // Function is now blacklisted from future promotions
        }

        Ok(())
    }

    /// Registers an OSR point for hot loop replacement.
    pub fn register_osr_point(&self, point: OsrPoint) {
        let mut points = self.osr_points.write().unwrap_or_else(|e| e.into_inner());
        points.push(point);
    }

    /// Checks if an OSR point should trigger tier promotion.
    pub fn check_osr(&self, func: &str, loop_id: usize, iteration: u64) -> Option<Tier> {
        let points = self.osr_points.read().unwrap_or_else(|e| e.into_inner());

        for point in points.iter() {
            if point.function == func
                && point.loop_id == loop_id
                && iteration >= point.iteration_threshold
            {
                return Some(point.target_tier);
            }
        }

        None
    }

    /// Gets the current tier for a function.
    pub fn get_function_tier(&self, name: &str) -> Tier {
        self.interpreter
            .get_profile(name)
            .map(|p| *p.current_tier.read().unwrap_or_else(|e| e.into_inner()))
            .unwrap_or(Tier::Interpreter)
    }

    /// Gets profiling statistics for a function.
    pub fn get_function_stats(&self, name: &str) -> Option<FunctionStats> {
        let profile = self.interpreter.get_profile(name)?;

        profile.update_hot_path_score();

        let execution_count = profile.execution_count.load(std::sync::atomic::Ordering::Relaxed);
        let current_tier = *profile
            .current_tier
            .read()
            .unwrap_or_else(|e| e.into_inner());
        let hot_loops = profile
            .loop_counts
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .filter(|(_, count)| **count > 1000)
            .count();
        let hot_path_score = *profile
            .hot_path_score
            .read()
            .unwrap_or_else(|e| e.into_inner());
        let deopt_count = profile.deopt_count.load(std::sync::atomic::Ordering::Relaxed);
        let is_blacklisted = profile.is_blacklisted();

        Some(FunctionStats {
            execution_count,
            current_tier,
            hot_loops,
            hot_path_score,
            deopt_count,
            is_blacklisted,
        })
    }

    /// Gets all function statistics.
    pub fn get_all_stats(&self) -> HashMap<String, FunctionStats> {
        let profiles = self
            .interpreter
            .profiles
            .read()
            .unwrap_or_else(|e| e.into_inner());

        profiles
            .iter()
            .map(|(name, profile)| {
                profile.update_hot_path_score();

                let execution_count = profile.execution_count.load(std::sync::atomic::Ordering::Relaxed);
                let current_tier = *profile
                    .current_tier
                    .read()
                    .unwrap_or_else(|e| e.into_inner());
                let hot_loops = profile
                    .loop_counts
                    .read()
                    .unwrap_or_else(|e| e.into_inner())
                    .iter()
                    .filter(|(_, count)| **count > 1000)
                    .count();
                let hot_path_score = *profile
                    .hot_path_score
                    .read()
                    .unwrap_or_else(|e| e.into_inner());
                let deopt_count = profile.deopt_count.load(std::sync::atomic::Ordering::Relaxed);
                let is_blacklisted = profile.is_blacklisted();

                (
                    name.clone(),
                    FunctionStats {
                        execution_count,
                        current_tier,
                        hot_loops,
                        hot_path_score,
                        deopt_count,
                        is_blacklisted,
                    },
                )
            })
            .collect()
    }
}

impl Default for TieredJit {
    fn default() -> Self {
        Self::new().expect("Failed to create TieredJit")
    }
}

/// Function statistics for debugging/profiling.
#[derive(Debug, Clone)]
pub struct FunctionStats {
    /// Total execution count.
    pub execution_count: u64,
    /// Current compilation tier.
    pub current_tier: Tier,
    /// Number of detected hot loops.
    pub hot_loops: usize,
    /// Hot path score (weighted profiling metric).
    pub hot_path_score: f64,
    /// Number of deoptimizations.
    pub deopt_count: u64,
    /// Whether function is blacklisted from promotion.
    pub is_blacklisted: bool,
}

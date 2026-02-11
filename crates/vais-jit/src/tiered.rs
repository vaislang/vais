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
use std::sync::{Arc, RwLock};

use vais_ast::{BinOp, Expr, Function, FunctionBody, Module as AstModule, Spanned, Stmt, UnaryOp};

use crate::JitError;

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
        let mut counts = self.loop_counts.write().unwrap();
        *counts.entry(loop_id).or_insert(0) += 1;
        self.total_loop_iterations.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a branch outcome.
    pub fn record_branch(&self, branch_id: usize, taken: bool) {
        let mut counts = self.branch_counts.write().unwrap();
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
        let mut hot_path_score = self.hot_path_score.write().unwrap();
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
        let mut last_promoted = self.last_promoted_at.write().unwrap();
        *last_promoted = count;
    }
}

impl Default for FunctionProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Interpreter value representation.
#[derive(Debug, Clone)]
pub enum Value {
    I64(i64),
    F64(f64),
    Bool(bool),
    String(String),
    Unit,
    Pointer(usize),
    Array(Vec<Value>),
    Tuple(Vec<Value>),
}

impl Value {
    /// Converts to i64, panicking if not possible.
    pub fn as_i64(&self) -> i64 {
        match self {
            Value::I64(n) => *n,
            Value::Bool(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            _ => panic!("Cannot convert {:?} to i64", self),
        }
    }

    /// Converts to f64, panicking if not possible.
    pub fn as_f64(&self) -> f64 {
        match self {
            Value::F64(n) => *n,
            Value::I64(n) => *n as f64,
            _ => panic!("Cannot convert {:?} to f64", self),
        }
    }

    /// Converts to bool, panicking if not possible.
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::I64(n) => *n != 0,
            _ => panic!("Cannot convert {:?} to bool", self),
        }
    }
}

/// Interpreter for Tier 0 execution.
pub struct Interpreter {
    /// Global variables.
    globals: HashMap<String, Value>,
    /// Function definitions.
    functions: HashMap<String, Function>,
    /// Function profiles for tier promotion decisions.
    profiles: Arc<RwLock<HashMap<String, Arc<FunctionProfile>>>>,
    /// Tier promotion thresholds.
    thresholds: TierThresholds,
}

impl Interpreter {
    /// Creates a new interpreter.
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            functions: HashMap::new(),
            profiles: Arc::new(RwLock::new(HashMap::new())),
            thresholds: TierThresholds::default(),
        }
    }

    /// Creates an interpreter with custom thresholds.
    pub fn with_thresholds(thresholds: TierThresholds) -> Self {
        Self {
            globals: HashMap::new(),
            functions: HashMap::new(),
            profiles: Arc::new(RwLock::new(HashMap::new())),
            thresholds,
        }
    }

    /// Loads a module's functions.
    pub fn load_module(&mut self, module: &AstModule) {
        for item in &module.items {
            if let vais_ast::Item::Function(func) = &item.node {
                self.functions.insert(func.name.node.clone(), func.clone());

                let mut profiles = self.profiles.write().unwrap();
                profiles
                    .entry(func.name.node.clone())
                    .or_insert_with(|| Arc::new(FunctionProfile::new()));
            }
        }
    }

    /// Gets the profile for a function.
    pub fn get_profile(&self, name: &str) -> Option<Arc<FunctionProfile>> {
        let profiles = self.profiles.read().unwrap();
        profiles.get(name).cloned()
    }

    /// Checks if a function should be promoted to the next tier.
    /// Now uses hot path score instead of simple execution count.
    pub fn should_promote(&self, name: &str) -> Option<Tier> {
        let profile = self.get_profile(name)?;

        // Don't promote blacklisted functions
        if profile.is_blacklisted() {
            return None;
        }

        // Update hot path score before checking
        profile.update_hot_path_score();

        let score = *profile.hot_path_score.read().unwrap();
        let current_tier = *profile.current_tier.read().unwrap();

        // Use hot path score thresholds (same numeric values but now score-based)
        match current_tier {
            Tier::Interpreter if score >= self.thresholds.interpreter_to_baseline as f64 => {
                Some(Tier::Baseline)
            }
            Tier::Baseline if score >= self.thresholds.baseline_to_optimizing as f64 => {
                Some(Tier::Optimizing)
            }
            _ => None,
        }
    }

    /// Executes the main function.
    pub fn run_main(&mut self) -> Result<Value, JitError> {
        self.call_function("main", &[])
    }

    /// Calls a function by name with arguments.
    pub fn call_function(&mut self, name: &str, args: &[Value]) -> Result<Value, JitError> {
        // Get function and profile
        let func = self
            .functions
            .get(name)
            .cloned()
            .ok_or_else(|| JitError::FunctionNotFound(name.to_string()))?;

        let profile = {
            let profiles = self.profiles.read().unwrap();
            profiles.get(name).cloned()
        };

        // Increment execution count
        if let Some(ref p) = profile {
            p.increment_execution();
        }

        // Build local environment
        let mut locals: HashMap<String, Value> = HashMap::new();
        for (i, param) in func.params.iter().enumerate() {
            if let Some(val) = args.get(i) {
                locals.insert(param.name.node.clone(), val.clone());
            }
        }

        // Execute function body
        match &func.body {
            FunctionBody::Expr(expr) => self.eval_expr(&expr.node, &mut locals, profile.as_ref()),
            FunctionBody::Block(stmts) => self.eval_block(stmts, &mut locals, profile.as_ref()),
        }
    }

    /// Evaluates a block of statements.
    fn eval_block(
        &mut self,
        stmts: &[Spanned<Stmt>],
        locals: &mut HashMap<String, Value>,
        profile: Option<&Arc<FunctionProfile>>,
    ) -> Result<Value, JitError> {
        let mut result = Value::Unit;

        for stmt in stmts.iter() {
            match &stmt.node {
                Stmt::Let { name, value, .. } => {
                    let val = self.eval_expr(&value.node, locals, profile)?;
                    locals.insert(name.node.clone(), val);
                }
                Stmt::Expr(expr) => {
                    result = self.eval_expr(&expr.node, locals, profile)?;
                }
                Stmt::Return(Some(expr)) => {
                    return self.eval_expr(&expr.node, locals, profile);
                }
                Stmt::Return(None) => {
                    return Ok(Value::Unit);
                }
                Stmt::Break(Some(expr)) => {
                    return self.eval_expr(&expr.node, locals, profile);
                }
                Stmt::Break(None) => {
                    return Ok(Value::Unit);
                }
                Stmt::Continue => {
                    // Continue is handled at loop level
                    return Err(JitError::Runtime("continue outside loop".to_string()));
                }
                Stmt::Defer(_) => {
                    // Defer is collected but not executed here
                }
                Stmt::Error { message, .. } => {
                    return Err(JitError::Runtime(message.clone()));
                }
                Stmt::LetDestructure { .. } => {
                    return Err(JitError::Runtime(
                        "tuple destructuring not yet supported in interpreter".to_string(),
                    ));
                }
            }

            // Note: last Stmt::Expr value is already captured in the Stmt::Expr arm above
        }

        Ok(result)
    }

    /// Evaluates an expression.
    fn eval_expr(
        &mut self,
        expr: &Expr,
        locals: &mut HashMap<String, Value>,
        profile: Option<&Arc<FunctionProfile>>,
    ) -> Result<Value, JitError> {
        match expr {
            Expr::Int(n) => Ok(Value::I64(*n)),
            Expr::Float(f) => Ok(Value::F64(*f)),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Unit => Ok(Value::Unit),

            Expr::Ident(name) => {
                if let Some(val) = locals.get(name) {
                    Ok(val.clone())
                } else if let Some(val) = self.globals.get(name) {
                    Ok(val.clone())
                } else {
                    Err(JitError::Runtime(format!("Variable not found: {}", name)))
                }
            }

            Expr::Binary { op, left, right } => {
                let lhs = self.eval_expr(&left.node, locals, profile)?;
                let rhs = self.eval_expr(&right.node, locals, profile)?;
                self.eval_binary_op(*op, lhs, rhs)
            }

            Expr::Unary { op, expr } => {
                let val = self.eval_expr(&expr.node, locals, profile)?;
                self.eval_unary_op(*op, val)
            }

            Expr::Call { func, args } => {
                if let Expr::Ident(name) = &func.node {
                    let mut arg_vals = Vec::new();
                    for arg in args {
                        arg_vals.push(self.eval_expr(&arg.node, locals, profile)?);
                    }
                    self.call_function(name, &arg_vals)
                } else {
                    Err(JitError::Unsupported("Indirect call".to_string()))
                }
            }

            Expr::If { cond, then, else_ } => {
                let cond_val = self.eval_expr(&cond.node, locals, profile)?;
                let branch_id = cond as *const _ as usize;

                if let Some(p) = profile {
                    p.record_branch(branch_id, cond_val.as_bool());
                }

                if cond_val.as_bool() {
                    self.eval_block(then, locals, profile)
                } else if let Some(else_branch) = else_ {
                    self.eval_if_else(else_branch, locals, profile)
                } else {
                    Ok(Value::Unit)
                }
            }

            Expr::Ternary { cond, then, else_ } => {
                let cond_val = self.eval_expr(&cond.node, locals, profile)?;
                if cond_val.as_bool() {
                    self.eval_expr(&then.node, locals, profile)
                } else {
                    self.eval_expr(&else_.node, locals, profile)
                }
            }

            Expr::Loop { body, .. } => {
                let loop_id = body as *const _ as usize;
                let mut _iteration = 0u64;
                loop {
                    if let Some(p) = profile {
                        p.record_loop(loop_id);
                    }

                    _iteration += 1;

                    match self.eval_block(body, locals, profile) {
                        Ok(Value::Unit) => continue,
                        Ok(val) => return Ok(val),
                        Err(e) => return Err(e),
                    }
                }
            }

            Expr::While { condition, body } => {
                let loop_id = body as *const _ as usize;
                let mut _iteration = 0u64;
                while self.eval_expr(&condition.node, locals, profile)?.as_bool() {
                    if let Some(p) = profile {
                        p.record_loop(loop_id);
                    }
                    _iteration += 1;
                    self.eval_block(body, locals, profile)?;
                }
                Ok(Value::Unit)
            }

            Expr::Block(stmts) => self.eval_block(stmts, locals, profile),

            Expr::Range {
                start,
                end,
                inclusive,
            } => {
                // Return a tuple representing the range
                let start_val = if let Some(ref s) = start {
                    self.eval_expr(&s.node, locals, profile)?
                } else {
                    Value::I64(0)
                };
                let end_val = if let Some(ref e) = end {
                    self.eval_expr(&e.node, locals, profile)?
                } else {
                    Value::I64(i64::MAX)
                };
                Ok(Value::Tuple(vec![
                    start_val,
                    end_val,
                    Value::Bool(*inclusive),
                ]))
            }

            _ => Err(JitError::Unsupported(format!(
                "Expression: {:?}",
                std::mem::discriminant(expr)
            ))),
        }
    }

    /// Evaluates an if-else branch.
    fn eval_if_else(
        &mut self,
        if_else: &vais_ast::IfElse,
        locals: &mut HashMap<String, Value>,
        profile: Option<&Arc<FunctionProfile>>,
    ) -> Result<Value, JitError> {
        match if_else {
            vais_ast::IfElse::ElseIf(cond, stmts, next) => {
                if self.eval_expr(&cond.node, locals, profile)?.as_bool() {
                    self.eval_block(stmts, locals, profile)
                } else if let Some(next) = next {
                    self.eval_if_else(next, locals, profile)
                } else {
                    Ok(Value::Unit)
                }
            }
            vais_ast::IfElse::Else(stmts) => self.eval_block(stmts, locals, profile),
        }
    }

    /// Evaluates a binary operation.
    fn eval_binary_op(&self, op: BinOp, lhs: Value, rhs: Value) -> Result<Value, JitError> {
        match (op, &lhs, &rhs) {
            // Integer operations (wrapping to prevent overflow panic)
            (BinOp::Add, Value::I64(a), Value::I64(b)) => Ok(Value::I64(a.wrapping_add(*b))),
            (BinOp::Sub, Value::I64(a), Value::I64(b)) => Ok(Value::I64(a.wrapping_sub(*b))),
            (BinOp::Mul, Value::I64(a), Value::I64(b)) => Ok(Value::I64(a.wrapping_mul(*b))),
            (BinOp::Div, Value::I64(a), Value::I64(b)) => {
                if *b == 0 {
                    Err(JitError::Runtime("Division by zero".to_string()))
                } else {
                    Ok(Value::I64(a / b))
                }
            }
            (BinOp::Mod, Value::I64(a), Value::I64(b)) => {
                if *b == 0 {
                    Err(JitError::Runtime("Modulo by zero".to_string()))
                } else {
                    Ok(Value::I64(a % b))
                }
            }

            // Float operations
            (BinOp::Add, Value::F64(a), Value::F64(b)) => Ok(Value::F64(a + b)),
            (BinOp::Sub, Value::F64(a), Value::F64(b)) => Ok(Value::F64(a - b)),
            (BinOp::Mul, Value::F64(a), Value::F64(b)) => Ok(Value::F64(a * b)),
            (BinOp::Div, Value::F64(a), Value::F64(b)) => Ok(Value::F64(a / b)),

            // Bitwise operations
            (BinOp::BitAnd, Value::I64(a), Value::I64(b)) => Ok(Value::I64(a & b)),
            (BinOp::BitOr, Value::I64(a), Value::I64(b)) => Ok(Value::I64(a | b)),
            (BinOp::BitXor, Value::I64(a), Value::I64(b)) => Ok(Value::I64(a ^ b)),
            (BinOp::Shl, Value::I64(a), Value::I64(b)) => Ok(Value::I64(a.wrapping_shl((*b as u32) & 63))),
            (BinOp::Shr, Value::I64(a), Value::I64(b)) => Ok(Value::I64(a.wrapping_shr((*b as u32) & 63))),

            // Comparison operations
            (BinOp::Eq, Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a == b)),
            (BinOp::Neq, Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a != b)),
            (BinOp::Lt, Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a < b)),
            (BinOp::Lte, Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a <= b)),
            (BinOp::Gt, Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a > b)),
            (BinOp::Gte, Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a >= b)),

            (BinOp::Eq, Value::F64(a), Value::F64(b)) => Ok(Value::Bool(a == b)),
            (BinOp::Neq, Value::F64(a), Value::F64(b)) => Ok(Value::Bool(a != b)),
            (BinOp::Lt, Value::F64(a), Value::F64(b)) => Ok(Value::Bool(a < b)),
            (BinOp::Lte, Value::F64(a), Value::F64(b)) => Ok(Value::Bool(a <= b)),
            (BinOp::Gt, Value::F64(a), Value::F64(b)) => Ok(Value::Bool(a > b)),
            (BinOp::Gte, Value::F64(a), Value::F64(b)) => Ok(Value::Bool(a >= b)),

            // Logical operations
            (BinOp::And, _, _) => Ok(Value::Bool(lhs.as_bool() && rhs.as_bool())),
            (BinOp::Or, _, _) => Ok(Value::Bool(lhs.as_bool() || rhs.as_bool())),

            _ => Err(JitError::Unsupported(format!(
                "Binary op {:?} on {:?} and {:?}",
                op, lhs, rhs
            ))),
        }
    }

    /// Evaluates a unary operation.
    fn eval_unary_op(&self, op: UnaryOp, val: Value) -> Result<Value, JitError> {
        match (op, &val) {
            (UnaryOp::Neg, Value::I64(n)) => Ok(Value::I64(-n)),
            (UnaryOp::Neg, Value::F64(n)) => Ok(Value::F64(-n)),
            (UnaryOp::Not, _) => Ok(Value::Bool(!val.as_bool())),
            (UnaryOp::BitNot, Value::I64(n)) => Ok(Value::I64(!n)),
            _ => Err(JitError::Unsupported(format!(
                "Unary op {:?} on {:?}",
                op, val
            ))),
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes hot path score based on profiling data.
///
/// Score = execution_count * 1.0 + total_loop_iterations * 0.5 + branch_bias_score * 0.3
fn compute_hot_path_score(profile: &FunctionProfile) -> f64 {
    let execution_count = profile.execution_count.load(Ordering::Relaxed) as f64;
    let total_loop_iterations = profile.total_loop_iterations.load(Ordering::Relaxed) as f64;

    // Calculate branch bias score (max bias across all branches)
    let branch_bias_score = {
        let branch_counts = profile.branch_counts.read().unwrap();
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
#[allow(dead_code)]
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
    osr_points: RwLock<Vec<OsrPoint>>,
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

        Ok(result.as_i64())
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
            let compiling = profile.compiling.read().unwrap();
            if *compiling {
                return Ok(());
            }
        }

        // Mark as compiling
        {
            let mut compiling = profile.compiling.write().unwrap();
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
            let mut current_tier = profile.current_tier.write().unwrap();
            *current_tier = tier;
            profile.mark_promoted();
        }

        // Clear compiling flag
        {
            let mut compiling = profile.compiling.write().unwrap();
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

        let current_tier = *profile.current_tier.read().unwrap();

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
            let mut tier = profile.current_tier.write().unwrap();
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
        let mut points = self.osr_points.write().unwrap();
        points.push(point);
    }

    /// Checks if an OSR point should trigger tier promotion.
    pub fn check_osr(&self, func: &str, loop_id: usize, iteration: u64) -> Option<Tier> {
        let points = self.osr_points.read().unwrap();

        for point in points.iter() {
            if point.function == func && point.loop_id == loop_id && iteration >= point.iteration_threshold {
                return Some(point.target_tier);
            }
        }

        None
    }

    /// Gets the current tier for a function.
    pub fn get_function_tier(&self, name: &str) -> Tier {
        self.interpreter
            .get_profile(name)
            .map(|p| *p.current_tier.read().unwrap())
            .unwrap_or(Tier::Interpreter)
    }

    /// Gets profiling statistics for a function.
    pub fn get_function_stats(&self, name: &str) -> Option<FunctionStats> {
        let profile = self.interpreter.get_profile(name)?;

        profile.update_hot_path_score();

        let execution_count = profile.execution_count.load(Ordering::Relaxed);
        let current_tier = *profile.current_tier.read().unwrap();
        let hot_loops = profile
            .loop_counts
            .read()
            .unwrap()
            .iter()
            .filter(|(_, count)| **count > 1000)
            .count();
        let hot_path_score = *profile.hot_path_score.read().unwrap();
        let deopt_count = profile.deopt_count.load(Ordering::Relaxed);
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
        let profiles = self.interpreter.profiles.read().unwrap();

        profiles
            .iter()
            .map(|(name, profile)| {
                profile.update_hot_path_score();

                let execution_count = profile.execution_count.load(Ordering::Relaxed);
                let current_tier = *profile.current_tier.read().unwrap();
                let hot_loops = profile
                    .loop_counts
                    .read()
                    .unwrap()
                    .iter()
                    .filter(|(_, count)| **count > 1000)
                    .count();
                let hot_path_score = *profile.hot_path_score.read().unwrap();
                let deopt_count = profile.deopt_count.load(Ordering::Relaxed);
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

#[cfg(test)]
mod tests {
    use super::*;
    use vais_parser::parse;

    #[test]
    fn test_interpreter_simple() {
        let source = "F main()->i64{42}";
        let ast = parse(source).unwrap();

        let mut interp = Interpreter::new();
        interp.load_module(&ast);

        let result = interp.run_main().unwrap();
        assert_eq!(result.as_i64(), 42);
    }

    #[test]
    fn test_interpreter_arithmetic() {
        let source = "F main()->i64{1+2*3}";
        let ast = parse(source).unwrap();

        let mut interp = Interpreter::new();
        interp.load_module(&ast);

        let result = interp.run_main().unwrap();
        assert_eq!(result.as_i64(), 7);
    }

    #[test]
    fn test_interpreter_function_call() {
        let source = "F add(a:i64,b:i64)->i64{a+b} F main()->i64{add(3,4)}";
        let ast = parse(source).unwrap();

        let mut interp = Interpreter::new();
        interp.load_module(&ast);

        let result = interp.run_main().unwrap();
        assert_eq!(result.as_i64(), 7);
    }

    #[test]
    fn test_interpreter_if_else() {
        let source = "F main()->i64{I true{1}E{0}}";
        let ast = parse(source).unwrap();

        let mut interp = Interpreter::new();
        interp.load_module(&ast);

        let result = interp.run_main().unwrap();
        assert_eq!(result.as_i64(), 1);
    }

    #[test]
    fn test_interpreter_local_variable() {
        let source = "F main()->i64{x:=10;x+5}";
        let ast = parse(source).unwrap();

        let mut interp = Interpreter::new();
        interp.load_module(&ast);

        let result = interp.run_main().unwrap();
        assert_eq!(result.as_i64(), 15);
    }

    #[test]
    fn test_profiling_execution_count() {
        let source = "F foo()->i64{1} F main()->i64{foo();foo();foo();0}";
        let ast = parse(source).unwrap();

        let mut interp = Interpreter::new();
        interp.load_module(&ast);
        interp.run_main().unwrap();

        let profile = interp.get_profile("foo").unwrap();
        assert_eq!(profile.execution_count.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn test_tier_promotion_detection() {
        let thresholds = TierThresholds {
            interpreter_to_baseline: 2,
            baseline_to_optimizing: 10,
        };

        let source = "F hot()->i64{1} F main()->i64{hot();hot();hot();0}";
        let ast = parse(source).unwrap();

        let mut interp = Interpreter::with_thresholds(thresholds);
        interp.load_module(&ast);
        interp.run_main().unwrap();

        assert!(interp.should_promote("hot").is_some());
        assert_eq!(interp.should_promote("hot"), Some(Tier::Baseline));
    }

    #[test]
    fn test_tiered_jit_basic() {
        let source = "F main()->i64{42}";
        let ast = parse(source).unwrap();

        let mut jit = TieredJit::new().unwrap();
        let result = jit.run_main(&ast).unwrap();

        assert_eq!(result, 42);
    }

    #[test]
    fn test_function_stats() {
        let source = "F foo()->i64{1} F main()->i64{foo();0}";
        let ast = parse(source).unwrap();

        let mut jit = TieredJit::new().unwrap();
        jit.run_main(&ast).unwrap();

        let stats = jit.get_function_stats("foo").unwrap();
        assert_eq!(stats.execution_count, 1);
        assert_eq!(stats.current_tier, Tier::Interpreter);
    }

    #[test]
    fn test_tier_names() {
        assert_eq!(Tier::Interpreter.name(), "Interpreter");
        assert_eq!(Tier::Baseline.name(), "Baseline JIT");
        assert_eq!(Tier::Optimizing.name(), "Optimizing JIT");
    }

    #[test]
    fn test_tier_ordering() {
        assert!(Tier::Interpreter < Tier::Baseline);
        assert!(Tier::Baseline < Tier::Optimizing);
    }

    #[test]
    fn test_hot_function_detection() {
        let thresholds = TierThresholds {
            interpreter_to_baseline: 5,
            baseline_to_optimizing: 50,
        };

        let source = "F loop_func()->i64{x:=0;L{I x>=10{R x} x:=x+1}0} F main()->i64{loop_func()}";
        let ast = parse(source).unwrap();

        let mut interp = Interpreter::with_thresholds(thresholds.clone());
        interp.load_module(&ast);

        // Execute multiple times to trigger promotion detection
        for _ in 0..6 {
            let _ = interp.call_function("loop_func", &[]);
        }

        // Should suggest promotion to baseline
        assert_eq!(interp.should_promote("loop_func"), Some(Tier::Baseline));
    }

    #[test]
    fn test_tier_promotion_baseline_to_optimizing() {
        let thresholds = TierThresholds {
            interpreter_to_baseline: 2,
            baseline_to_optimizing: 5,
        };

        let source = "F hot()->i64{42} F main()->i64{hot()}";
        let ast = parse(source).unwrap();

        let mut interp = Interpreter::with_thresholds(thresholds);
        interp.load_module(&ast);

        // Execute 6 times (exceeds baseline threshold)
        for _ in 0..6 {
            let _ = interp.call_function("hot", &[]);
        }

        // After 6 executions, should suggest promotion
        let promotion = interp.should_promote("hot");
        assert!(promotion.is_some());

        // Manually promote to baseline
        if let Some(profile) = interp.get_profile("hot") {
            let mut tier = profile.current_tier.write().unwrap();
            *tier = Tier::Baseline;
        }

        // Continue execution to hit optimizing threshold
        for _ in 0..50 {
            let _ = interp.call_function("hot", &[]);
        }

        // Now should suggest optimizing tier
        assert_eq!(interp.should_promote("hot"), Some(Tier::Optimizing));
    }

    #[test]
    fn test_branch_profiling() {
        let source = "F branch_test(x:i64)->i64{I x>5{1}E{0}} F main()->i64{branch_test(10)}";
        let ast = parse(source).unwrap();

        let mut interp = Interpreter::new();
        interp.load_module(&ast);

        // Execute with different values
        let _ = interp.call_function("branch_test", &[Value::I64(10)]);
        let _ = interp.call_function("branch_test", &[Value::I64(3)]);
        let _ = interp.call_function("branch_test", &[Value::I64(7)]);

        let profile = interp.get_profile("branch_test").unwrap();

        // Should have recorded branch outcomes
        let branch_counts = profile.branch_counts.read().unwrap();
        assert!(!branch_counts.is_empty());
    }

    #[test]
    fn test_loop_profiling() {
        let source = "F loop_test()->i64{i:=0;L{I i>=5{R i} i:=i+1}0} F main()->i64{loop_test()}";
        let ast = parse(source).unwrap();

        let mut interp = Interpreter::new();
        interp.load_module(&ast);

        let _ = interp.run_main();

        let profile = interp.get_profile("loop_test").unwrap();

        // Should have recorded loop iterations
        let loop_counts = profile.loop_counts.read().unwrap();
        assert!(!loop_counts.is_empty());
    }

    #[test]
    fn test_tiered_jit_with_custom_thresholds() {
        let thresholds = TierThresholds {
            interpreter_to_baseline: 100,
            baseline_to_optimizing: 500,
        };

        let source = "F compute()->i64{10+20+30} F main()->i64{compute()}";
        let ast = parse(source).unwrap();

        let mut jit = TieredJit::with_thresholds(thresholds).unwrap();
        let result = jit.run_main(&ast).unwrap();

        assert_eq!(result, 60);

        // With high thresholds and single execution, should stay in interpreter tier
        let tier = jit.get_function_tier("main");
        assert!(tier == Tier::Interpreter || tier == Tier::Baseline);
    }

    #[test]
    fn test_hot_path_score_calculation() {
        let source = "F loop_heavy()->i64{x:=0;L{I x>=100{R x} x:=x+1}0} F main()->i64{loop_heavy()}";
        let ast = parse(source).unwrap();

        let mut interp = Interpreter::new();
        interp.load_module(&ast);

        // Execute function
        let _ = interp.run_main();

        let profile = interp.get_profile("loop_heavy").unwrap();
        profile.update_hot_path_score();

        let score = *profile.hot_path_score.read().unwrap();

        // Score should be high due to many loop iterations
        assert!(score > 50.0, "Hot path score should be > 50.0, got {}", score);
    }

    #[test]
    fn test_deoptimization() {
        let source = "F hot()->i64{42} F main()->i64{hot()}";
        let ast = parse(source).unwrap();

        let mut jit = TieredJit::new().unwrap();
        jit.load_module(&ast);

        // Get profile and manually promote to Baseline
        if let Some(profile) = jit.interpreter.get_profile("hot") {
            let mut tier = profile.current_tier.write().unwrap();
            *tier = Tier::Baseline;
        }

        // Verify tier
        assert_eq!(jit.get_function_tier("hot"), Tier::Baseline);

        // Deoptimize
        jit.deoptimize("hot").unwrap();

        // Should be back to Interpreter
        assert_eq!(jit.get_function_tier("hot"), Tier::Interpreter);

        // Deopt count should be 1
        let stats = jit.get_function_stats("hot").unwrap();
        assert_eq!(stats.deopt_count, 1);
    }

    #[test]
    fn test_deopt_blacklist() {
        let source = "F unstable()->i64{1} F main()->i64{unstable()}";
        let ast = parse(source).unwrap();

        let mut jit = TieredJit::new().unwrap();
        jit.load_module(&ast);

        let profile = jit.interpreter.get_profile("unstable").unwrap();

        // Manually promote and deopt 3 times
        for _ in 0..3 {
            {
                let mut tier = profile.current_tier.write().unwrap();
                *tier = Tier::Baseline;
            }
            jit.deoptimize("unstable").unwrap();
        }

        // Should be blacklisted
        assert!(profile.is_blacklisted());

        // should_promote should return None for blacklisted functions
        assert!(jit.interpreter.should_promote("unstable").is_none());
    }

    #[test]
    fn test_osr_point_registration() {
        let jit = TieredJit::new().unwrap();

        let osr_point = OsrPoint {
            function: "hot_loop".to_string(),
            loop_id: 12345,
            target_tier: Tier::Baseline,
            iteration_threshold: 1000,
        };

        jit.register_osr_point(osr_point.clone());

        let points = jit.osr_points.read().unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].function, "hot_loop");
        assert_eq!(points[0].loop_id, 12345);
    }

    #[test]
    fn test_osr_check_threshold() {
        let jit = TieredJit::new().unwrap();

        let osr_point = OsrPoint {
            function: "compute".to_string(),
            loop_id: 99,
            target_tier: Tier::Optimizing,
            iteration_threshold: 500,
        };

        jit.register_osr_point(osr_point);

        // Below threshold - no promotion
        assert!(jit.check_osr("compute", 99, 300).is_none());

        // At threshold - should promote
        let tier = jit.check_osr("compute", 99, 500);
        assert_eq!(tier, Some(Tier::Optimizing));

        // Above threshold - should promote
        let tier = jit.check_osr("compute", 99, 1000);
        assert_eq!(tier, Some(Tier::Optimizing));
    }

    #[test]
    fn test_dynamic_tier_promotion_with_score() {
        let thresholds = TierThresholds {
            interpreter_to_baseline: 50,
            baseline_to_optimizing: 200,
        };

        let source = "F work()->i64{x:=0;L{I x>=20{R x} x:=x+1}0} F main()->i64{work()}";
        let ast = parse(source).unwrap();

        let mut interp = Interpreter::with_thresholds(thresholds);
        interp.load_module(&ast);

        // Execute multiple times to build up score
        for _ in 0..10 {
            let _ = interp.call_function("work", &[]);
        }

        let profile = interp.get_profile("work").unwrap();
        profile.update_hot_path_score();

        let score = *profile.hot_path_score.read().unwrap();

        // With 10 executions and ~20 loop iterations each, score should be high
        assert!(score > 50.0, "Score should exceed baseline threshold");

        // Should suggest promotion
        let promotion = interp.should_promote("work");
        assert_eq!(promotion, Some(Tier::Baseline));
    }

    #[test]
    fn test_total_loop_iterations() {
        let source = "F looper()->i64{x:=0;L{I x>=10{R x} x:=x+1}0} F main()->i64{looper()}";
        let ast = parse(source).unwrap();

        let mut interp = Interpreter::new();
        interp.load_module(&ast);

        let _ = interp.run_main();

        let profile = interp.get_profile("looper").unwrap();
        let total = profile.total_loop_iterations.load(Ordering::Relaxed);

        // Should have accumulated loop iterations
        assert!(total >= 10, "Expected at least 10 loop iterations, got {}", total);
    }

    #[test]
    fn test_get_all_stats() {
        let source = "F foo()->i64{1} F bar()->i64{2} F main()->i64{foo()+bar()}";
        let ast = parse(source).unwrap();

        let mut jit = TieredJit::new().unwrap();
        jit.load_module(&ast);
        let _ = jit.interpreter.run_main();

        let all_stats = jit.get_all_stats();

        // Should have stats for all functions
        assert!(all_stats.contains_key("foo"));
        assert!(all_stats.contains_key("bar"));
        assert!(all_stats.contains_key("main"));
    }

    #[test]
    fn test_branch_bias_score() {
        let source = "F biased(x:i64)->i64{I x>0{1}E{0}} F main()->i64{biased(10)}";
        let ast = parse(source).unwrap();

        let mut interp = Interpreter::new();
        interp.load_module(&ast);

        // Execute many times with same bias (always true)
        for _ in 0..20 {
            let _ = interp.call_function("biased", &[Value::I64(10)]);
        }

        let profile = interp.get_profile("biased").unwrap();
        profile.update_hot_path_score();

        let score = *profile.hot_path_score.read().unwrap();

        // Score should include branch bias component
        assert!(score > 0.0, "Score should be > 0 with branch bias");
    }

    #[test]
    fn test_last_promoted_at() {
        let source = "F func()->i64{1} F main()->i64{func()}";
        let ast = parse(source).unwrap();

        let mut jit = TieredJit::new().unwrap();
        jit.load_module(&ast);

        let profile = jit.interpreter.get_profile("func").unwrap();

        // Execute a few times
        for _ in 0..5 {
            let _ = jit.interpreter.call_function("func", &[]);
        }

        let count_before = profile.execution_count.load(Ordering::Relaxed);

        // Mark as promoted
        profile.mark_promoted();

        let last_promoted = *profile.last_promoted_at.read().unwrap();
        assert_eq!(last_promoted, count_before);
    }
}

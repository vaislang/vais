use super::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use vais_ast::{Module as AstModule, *};
use crate::JitError;

/// Interpreter for Tier 0 execution.
pub struct Interpreter {
    /// Global variables.
    pub(crate) globals: HashMap<String, Value>,
    /// Function definitions.
    pub(crate) functions: HashMap<String, Function>,
    /// Function profiles for tier promotion decisions.
    pub(crate) profiles: Arc<RwLock<HashMap<String, Arc<FunctionProfile>>>>,
    /// Tier promotion thresholds.
    pub(crate) thresholds: TierThresholds,
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

                let mut profiles = self.profiles.write().unwrap_or_else(|e| e.into_inner());
                profiles
                    .entry(func.name.node.clone())
                    .or_insert_with(|| Arc::new(FunctionProfile::new()));
            }
        }
    }

    /// Gets the profile for a function.
    pub fn get_profile(&self, name: &str) -> Option<Arc<FunctionProfile>> {
        let profiles = self.profiles.read().unwrap_or_else(|e| e.into_inner());
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

        let score = *profile
            .hot_path_score
            .read()
            .unwrap_or_else(|e| e.into_inner());
        let current_tier = *profile
            .current_tier
            .read()
            .unwrap_or_else(|e| e.into_inner());

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
            let profiles = self.profiles.read().unwrap_or_else(|e| e.into_inner());
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
                let cond_bool = cond_val.as_bool()?;

                if let Some(p) = profile {
                    p.record_branch(branch_id, cond_bool);
                }

                if cond_bool {
                    self.eval_block(then, locals, profile)
                } else if let Some(else_branch) = else_ {
                    self.eval_if_else(else_branch, locals, profile)
                } else {
                    Ok(Value::Unit)
                }
            }

            Expr::Ternary { cond, then, else_ } => {
                let cond_val = self.eval_expr(&cond.node, locals, profile)?;
                if cond_val.as_bool()? {
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
                while self
                    .eval_expr(&condition.node, locals, profile)?
                    .as_bool()?
                {
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
                if self.eval_expr(&cond.node, locals, profile)?.as_bool()? {
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
            (BinOp::Shl, Value::I64(a), Value::I64(b)) => {
                Ok(Value::I64(a.wrapping_shl((*b as u32) & 63)))
            }
            (BinOp::Shr, Value::I64(a), Value::I64(b)) => {
                Ok(Value::I64(a.wrapping_shr((*b as u32) & 63)))
            }

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
            (BinOp::And, _, _) => Ok(Value::Bool(lhs.as_bool()? && rhs.as_bool()?)),
            (BinOp::Or, _, _) => Ok(Value::Bool(lhs.as_bool()? || rhs.as_bool()?)),

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
            (UnaryOp::Not, _) => Ok(Value::Bool(!val.as_bool()?)),
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

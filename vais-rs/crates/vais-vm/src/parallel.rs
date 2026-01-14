//! Parallel Execution Context for Vais VM
//!
//! Lightweight execution context for parallel collection operations.
//! Uses Rayon for data-parallel operations on large collections.
//! Avoids full VM initialization overhead for simple parallel map/filter/reduce.

use std::collections::HashMap;
use std::sync::Arc;

use rayon::prelude::*;
use vais_ir::{Instruction, OpCode, Value};
use vais_lowering::CompiledFunction;

// Type alias for faster HashMap
type FastMap<K, V> = HashMap<K, V>;

/// Threshold for switching to parallel execution
/// Below this, sequential is faster due to Rayon overhead
const PARALLEL_THRESHOLD: usize = 1000;

/// Lightweight execution context for parallel operations
/// Avoids full VM initialization overhead
#[allow(dead_code)]
pub struct ParallelContext {
    stack: Vec<Value>,
    /// Named local variables (public for VM integration)
    pub named_locals: FastMap<String, Value>,
    /// Functions map (public for VM integration)
    pub functions: Arc<FastMap<String, Arc<CompiledFunction>>>,
}

#[allow(dead_code)]
impl ParallelContext {
    /// Create a new parallel context
    pub fn new(functions: Arc<FastMap<String, Arc<CompiledFunction>>>) -> Self {
        Self {
            stack: Vec::with_capacity(16),
            named_locals: FastMap::new(),
            functions,
        }
    }

    /// Set a local variable (used for lambda parameter binding)
    #[allow(dead_code)]
    pub fn set_local(&mut self, name: &str, value: Value) {
        self.named_locals.insert(name.to_string(), value);
    }

    /// Execute simple instructions and return the result
    /// Returns None if complex operations are encountered (fall back to full VM)
    pub fn execute_simple(&mut self, instructions: &[Instruction]) -> Option<Value> {
        for instr in instructions {
            match &instr.opcode {
                OpCode::Const(v) => self.stack.push(v.clone()),
                OpCode::Load(name) => {
                    if let Some(v) = self.named_locals.get(name) {
                        self.stack.push(v.clone());
                    } else {
                        self.stack.push(Value::Void);
                    }
                }
                OpCode::Store(name) => {
                    if let Some(v) = self.stack.pop() {
                        self.named_locals.insert(name.clone(), v);
                    }
                }
                OpCode::Add => {
                    if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                        let result = match (a, b) {
                            (Value::Int(x), Value::Int(y)) => Value::Int(x.wrapping_add(y)),
                            (Value::Float(x), Value::Float(y)) => Value::Float(x + y),
                            (Value::Int(x), Value::Float(y)) => Value::Float(x as f64 + y),
                            (Value::Float(x), Value::Int(y)) => Value::Float(x + y as f64),
                            _ => Value::Void,
                        };
                        self.stack.push(result);
                    }
                }
                OpCode::Sub => {
                    if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                        let result = match (a, b) {
                            (Value::Int(x), Value::Int(y)) => Value::Int(x.wrapping_sub(y)),
                            (Value::Float(x), Value::Float(y)) => Value::Float(x - y),
                            _ => Value::Void,
                        };
                        self.stack.push(result);
                    }
                }
                OpCode::Mul => {
                    if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                        let result = match (a, b) {
                            (Value::Int(x), Value::Int(y)) => Value::Int(x.wrapping_mul(y)),
                            (Value::Float(x), Value::Float(y)) => Value::Float(x * y),
                            (Value::Int(x), Value::Float(y)) => Value::Float(x as f64 * y),
                            (Value::Float(x), Value::Int(y)) => Value::Float(x * y as f64),
                            _ => Value::Void,
                        };
                        self.stack.push(result);
                    }
                }
                OpCode::Div => {
                    if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                        let result = match (a, b) {
                            (Value::Int(x), Value::Int(y)) if y != 0 => Value::Int(x / y),
                            (Value::Float(x), Value::Float(y)) if y != 0.0 => Value::Float(x / y),
                            _ => Value::Void,
                        };
                        self.stack.push(result);
                    }
                }
                OpCode::Mod => {
                    if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                        let result = match (a, b) {
                            (Value::Int(x), Value::Int(y)) if y != 0 => Value::Int(x % y),
                            _ => Value::Void,
                        };
                        self.stack.push(result);
                    }
                }
                OpCode::Lt => {
                    if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                        let result = match (a, b) {
                            (Value::Int(x), Value::Int(y)) => Value::Bool(x < y),
                            (Value::Float(x), Value::Float(y)) => Value::Bool(x < y),
                            _ => Value::Bool(false),
                        };
                        self.stack.push(result);
                    }
                }
                OpCode::Gt => {
                    if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                        let result = match (a, b) {
                            (Value::Int(x), Value::Int(y)) => Value::Bool(x > y),
                            (Value::Float(x), Value::Float(y)) => Value::Bool(x > y),
                            _ => Value::Bool(false),
                        };
                        self.stack.push(result);
                    }
                }
                OpCode::Eq => {
                    if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(Value::Bool(a == b));
                    }
                }
                OpCode::Not => {
                    if let Some(v) = self.stack.pop() {
                        self.stack.push(Value::Bool(!v.is_truthy()));
                    }
                }
                OpCode::Neg => {
                    if let Some(v) = self.stack.pop() {
                        let result = match v {
                            Value::Int(x) => Value::Int(-x),
                            Value::Float(x) => Value::Float(-x),
                            _ => Value::Void,
                        };
                        self.stack.push(result);
                    }
                }
                OpCode::And => {
                    if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(Value::Bool(a.is_truthy() && b.is_truthy()));
                    }
                }
                OpCode::Or => {
                    if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(Value::Bool(a.is_truthy() || b.is_truthy()));
                    }
                }
                OpCode::Return => break,
                _ => {
                    // For complex operations, return None to fall back to full VM
                    return None;
                }
            }
        }
        self.stack.pop()
    }
}

/// Parallel map for integer arrays (simple numeric transformation)
/// Note: Full Value parallel is not possible due to Rc (not Send).
/// This optimizes the common case of numeric array transformation.
#[allow(dead_code)]
pub fn parallel_map_int<F>(arr: &[i64], f: F) -> Vec<i64>
where
    F: Fn(i64) -> i64 + Send + Sync,
{
    if arr.len() < PARALLEL_THRESHOLD {
        return arr.iter().map(|&x| f(x)).collect();
    }
    arr.par_iter().map(|&x| f(x)).collect()
}

/// Parallel map for float arrays
#[allow(dead_code)]
pub fn parallel_map_float<F>(arr: &[f64], f: F) -> Vec<f64>
where
    F: Fn(f64) -> f64 + Send + Sync,
{
    if arr.len() < PARALLEL_THRESHOLD {
        return arr.iter().map(|&x| f(x)).collect();
    }
    arr.par_iter().map(|&x| f(x)).collect()
}

/// Parallel filter for integer arrays
#[allow(dead_code)]
pub fn parallel_filter_int<F>(arr: &[i64], predicate: F) -> Vec<i64>
where
    F: Fn(i64) -> bool + Send + Sync,
{
    if arr.len() < PARALLEL_THRESHOLD {
        return arr.iter().filter(|&&x| predicate(x)).copied().collect();
    }
    arr.par_iter().filter(|&&x| predicate(x)).copied().collect()
}

/// Parallel filter for float arrays
#[allow(dead_code)]
pub fn parallel_filter_float<F>(arr: &[f64], predicate: F) -> Vec<f64>
where
    F: Fn(f64) -> bool + Send + Sync,
{
    if arr.len() < PARALLEL_THRESHOLD {
        return arr.iter().filter(|&&x| predicate(x)).copied().collect();
    }
    arr.par_iter().filter(|&&x| predicate(x)).copied().collect()
}

/// Extract integers from Value array for parallel processing
#[allow(dead_code)]
pub fn extract_ints(arr: &[Value]) -> Option<Vec<i64>> {
    arr.iter()
        .map(|v| match v {
            Value::Int(n) => Some(*n),
            _ => None,
        })
        .collect()
}

/// Extract floats from Value array for parallel processing
#[allow(dead_code)]
pub fn extract_floats(arr: &[Value]) -> Option<Vec<f64>> {
    arr.iter()
        .map(|v| match v {
            Value::Int(n) => Some(*n as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        })
        .collect()
}

/// Convert integer array back to Value array
#[allow(dead_code)]
pub fn ints_to_values(arr: Vec<i64>) -> Vec<Value> {
    arr.into_iter().map(Value::Int).collect()
}

/// Convert float array back to Value array
#[allow(dead_code)]
pub fn floats_to_values(arr: Vec<f64>) -> Vec<Value> {
    arr.into_iter().map(Value::Float).collect()
}

/// Parallel sum for integer arrays
#[allow(dead_code)]
pub fn parallel_sum_int(arr: &[i64]) -> i64 {
    if arr.len() < PARALLEL_THRESHOLD {
        return arr.iter().sum();
    }
    arr.par_iter().sum()
}

/// Parallel sum for float arrays
#[allow(dead_code)]
pub fn parallel_sum_float(arr: &[f64]) -> f64 {
    if arr.len() < PARALLEL_THRESHOLD {
        return arr.iter().sum();
    }
    arr.par_iter().sum()
}

/// Parallel product for integer arrays
#[allow(dead_code)]
pub fn parallel_product_int(arr: &[i64]) -> i64 {
    if arr.len() < PARALLEL_THRESHOLD {
        return arr.iter().product();
    }
    arr.par_iter().product()
}

/// Parallel product for float arrays
#[allow(dead_code)]
pub fn parallel_product_float(arr: &[f64]) -> f64 {
    if arr.len() < PARALLEL_THRESHOLD {
        return arr.iter().product();
    }
    arr.par_iter().product()
}

/// Parallel min for integer arrays
#[allow(dead_code)]
pub fn parallel_min_int(arr: &[i64]) -> Option<i64> {
    if arr.is_empty() {
        return None;
    }
    if arr.len() < PARALLEL_THRESHOLD {
        return arr.iter().copied().min();
    }
    arr.par_iter().copied().min()
}

/// Parallel max for integer arrays
#[allow(dead_code)]
pub fn parallel_max_int(arr: &[i64]) -> Option<i64> {
    if arr.is_empty() {
        return None;
    }
    if arr.len() < PARALLEL_THRESHOLD {
        return arr.iter().copied().max();
    }
    arr.par_iter().copied().max()
}

/// Check if a lambda body is simple enough for parallel execution
#[allow(dead_code)]
fn is_simple_lambda(instructions: &[Instruction]) -> bool {
    // Limit instruction count to avoid complex lambdas
    if instructions.len() > 20 {
        return false;
    }

    // Check all instructions are supported
    instructions.iter().all(|instr| {
        matches!(
            &instr.opcode,
            OpCode::Const(_)
                | OpCode::Load(_)
                | OpCode::Store(_)
                | OpCode::Add
                | OpCode::Sub
                | OpCode::Mul
                | OpCode::Div
                | OpCode::Mod
                | OpCode::Lt
                | OpCode::Gt
                | OpCode::Lte
                | OpCode::Gte
                | OpCode::Eq
                | OpCode::Neq
                | OpCode::Not
                | OpCode::Neg
                | OpCode::And
                | OpCode::Or
                | OpCode::Return
        )
    })
}

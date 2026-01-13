//! Parallel Execution Context for Vais VM
//!
//! Lightweight execution context for parallel collection operations.
//! Avoids full VM initialization overhead for simple parallel map/filter/reduce.

use std::collections::HashMap;
use std::sync::Arc;

use vais_ir::{Instruction, OpCode, Value};
use vais_lowering::CompiledFunction;

// Type alias for faster HashMap
type FastMap<K, V> = HashMap<K, V>;

/// Lightweight execution context for parallel operations
/// Avoids full VM initialization overhead
pub struct ParallelContext {
    stack: Vec<Value>,
    /// Named local variables (public for VM integration)
    pub named_locals: FastMap<String, Value>,
    /// Functions map (public for VM integration)
    pub functions: Arc<FastMap<String, Arc<CompiledFunction>>>,
}

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

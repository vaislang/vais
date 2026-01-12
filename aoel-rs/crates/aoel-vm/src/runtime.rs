//! Runtime state for VM execution

use std::collections::HashMap;
use aoel_ir::Value;
use crate::error::{RuntimeError, RuntimeResult};

/// Runtime environment for VM execution
#[derive(Debug, Default)]
pub struct Runtime {
    /// Value stack
    stack: Vec<Value>,
    /// Local variables
    locals: HashMap<String, Value>,
    /// Input values
    inputs: HashMap<String, Value>,
    /// Output values
    outputs: HashMap<String, Value>,
}

impl Runtime {
    /// Create a new runtime
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a runtime with input values
    pub fn with_inputs(inputs: HashMap<String, Value>) -> Self {
        Self {
            inputs,
            ..Self::default()
        }
    }

    // ==================== Stack Operations ====================

    /// Push a value onto the stack
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Pop a value from the stack
    pub fn pop(&mut self) -> RuntimeResult<Value> {
        self.stack.pop().ok_or(RuntimeError::StackUnderflow)
    }

    /// Peek at the top value without removing it
    pub fn peek(&self) -> RuntimeResult<&Value> {
        self.stack.last().ok_or(RuntimeError::StackUnderflow)
    }

    /// Duplicate the top value
    pub fn dup(&mut self) -> RuntimeResult<()> {
        let top = self.peek()?.clone();
        self.push(top);
        Ok(())
    }

    /// Get the current stack depth
    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }

    // ==================== Variable Operations ====================

    /// Load a local variable
    pub fn load(&self, name: &str) -> RuntimeResult<Value> {
        self.locals
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::UndefinedVariable(name.to_string()))
    }

    /// Store a value to a local variable
    pub fn store(&mut self, name: &str, value: Value) {
        self.locals.insert(name.to_string(), value);
    }

    // ==================== Input/Output Operations ====================

    /// Load an input value
    pub fn load_input(&self, name: &str) -> RuntimeResult<Value> {
        self.inputs
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::UndefinedInput(name.to_string()))
    }

    /// Store a value to output
    pub fn store_output(&mut self, name: &str, value: Value) {
        self.outputs.insert(name.to_string(), value);
    }

    /// Get all outputs
    pub fn outputs(&self) -> &HashMap<String, Value> {
        &self.outputs
    }

    /// Take ownership of outputs (drains and returns them)
    pub fn take_outputs(&mut self) -> HashMap<String, Value> {
        std::mem::take(&mut self.outputs)
    }

    // ==================== Utility ====================

    /// Clear the runtime state
    pub fn clear(&mut self) {
        self.stack.clear();
        self.locals.clear();
        self.outputs.clear();
    }

    /// Reset with new inputs
    pub fn reset_with_inputs(&mut self, inputs: HashMap<String, Value>) {
        self.clear();
        self.inputs = inputs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_operations() {
        let mut runtime = Runtime::new();

        runtime.push(Value::Int(42));
        assert_eq!(runtime.stack_depth(), 1);

        let val = runtime.pop().unwrap();
        assert_eq!(val, Value::Int(42));
        assert_eq!(runtime.stack_depth(), 0);

        assert!(runtime.pop().is_err());
    }

    #[test]
    fn test_dup() {
        let mut runtime = Runtime::new();
        runtime.push(Value::Int(42));
        runtime.dup().unwrap();
        assert_eq!(runtime.stack_depth(), 2);

        let val1 = runtime.pop().unwrap();
        let val2 = runtime.pop().unwrap();
        assert_eq!(val1, val2);
    }

    #[test]
    fn test_variables() {
        let mut runtime = Runtime::new();

        runtime.store("x", Value::Int(10));
        let val = runtime.load("x").unwrap();
        assert_eq!(val, Value::Int(10));

        assert!(runtime.load("undefined").is_err());
    }

    #[test]
    fn test_inputs_outputs() {
        let mut inputs = HashMap::new();
        inputs.insert("name".to_string(), Value::String("test".to_string()));

        let mut runtime = Runtime::with_inputs(inputs);

        let input_val = runtime.load_input("name").unwrap();
        assert_eq!(input_val, Value::String("test".to_string()));

        runtime.store_output("result", Value::Int(42));
        assert_eq!(runtime.outputs().get("result"), Some(&Value::Int(42)));
    }
}

//! Main VM execution engine

use std::collections::HashMap;
use aoel_ir::{Module, OpCode, Value, Instruction, NodeIR, NodeOpType, ReduceOp};
use crate::builtins::call_builtin;
use crate::error::{RuntimeError, RuntimeResult};
use crate::runtime::Runtime;

/// AOEL Virtual Machine
pub struct Vm {
    runtime: Runtime,
}

impl Vm {
    /// Create a new VM
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
        }
    }

    /// Execute a module with the given inputs
    pub fn execute(
        &mut self,
        module: &Module,
        inputs: HashMap<String, Value>,
    ) -> RuntimeResult<HashMap<String, Value>> {
        // Reset runtime with inputs
        self.runtime.reset_with_inputs(inputs);

        // Build node lookup table
        let nodes: HashMap<String, &NodeIR> = module
            .main
            .nodes
            .iter()
            .map(|n| (n.id.clone(), n))
            .collect();

        // Execute nodes in topological order
        for node_id in &module.main.execution_order {
            if let Some(node) = nodes.get(node_id) {
                self.execute_node(node)?;
            }
        }

        Ok(self.runtime.take_outputs())
    }

    /// Execute a single node
    fn execute_node(&mut self, node: &NodeIR) -> RuntimeResult<()> {
        match &node.op_type {
            NodeOpType::Transform => {
                // Execute node instructions
                self.execute_instructions(&node.instructions)?;
            }
            NodeOpType::Map => {
                // Map: apply instructions to each element of input array
                self.execute_map(node)?;
            }
            NodeOpType::Filter => {
                // Filter: keep elements where predicate returns true
                self.execute_filter(node)?;
            }
            NodeOpType::Reduce(reduce_op) => {
                // Reduce: aggregate array to single value
                self.execute_reduce(node, reduce_op)?;
            }
            NodeOpType::Branch => {
                // Branch: conditional execution (handled via edges)
                self.execute_instructions(&node.instructions)?;
            }
            NodeOpType::Merge => {
                // Merge: combine multiple inputs (no-op for now)
            }
            NodeOpType::Fetch => {
                // Fetch: external data fetch (placeholder)
                self.execute_instructions(&node.instructions)?;
            }
            NodeOpType::Store => {
                // Store: external data store (placeholder)
                self.execute_instructions(&node.instructions)?;
            }
            NodeOpType::Validate => {
                // Validate: run validation logic
                self.execute_instructions(&node.instructions)?;
            }
        }
        Ok(())
    }

    /// Execute a sequence of instructions
    fn execute_instructions(&mut self, instructions: &[Instruction]) -> RuntimeResult<()> {
        for instr in instructions {
            self.execute_instruction(instr)?;
        }
        Ok(())
    }

    /// Execute a single instruction
    fn execute_instruction(&mut self, instr: &Instruction) -> RuntimeResult<()> {
        match &instr.opcode {
            // Stack operations
            OpCode::Const(value) => {
                self.runtime.push(value.clone());
            }
            OpCode::Pop => {
                self.runtime.pop()?;
            }
            OpCode::Dup => {
                self.runtime.dup()?;
            }

            // Variable operations
            OpCode::Load(name) => {
                let value = self.runtime.load(name)?;
                self.runtime.push(value);
            }
            OpCode::Store(name) => {
                let value = self.runtime.pop()?;
                self.runtime.store(name, value);
            }
            OpCode::LoadInput(name) => {
                let value = self.runtime.load_input(name)?;
                self.runtime.push(value);
            }
            OpCode::StoreOutput(name) => {
                let value = self.runtime.pop()?;
                self.runtime.store_output(name, value);
            }

            // Arithmetic operations
            OpCode::Add => {
                let b = self.runtime.pop()?;
                let a = self.runtime.pop()?;
                let result = self.binary_add(a, b)?;
                self.runtime.push(result);
            }
            OpCode::Sub => {
                let b = self.runtime.pop()?;
                let a = self.runtime.pop()?;
                let result = self.binary_sub(a, b)?;
                self.runtime.push(result);
            }
            OpCode::Mul => {
                let b = self.runtime.pop()?;
                let a = self.runtime.pop()?;
                let result = self.binary_mul(a, b)?;
                self.runtime.push(result);
            }
            OpCode::Div => {
                let b = self.runtime.pop()?;
                let a = self.runtime.pop()?;
                let result = self.binary_div(a, b)?;
                self.runtime.push(result);
            }
            OpCode::Neg => {
                let a = self.runtime.pop()?;
                let result = self.unary_neg(a)?;
                self.runtime.push(result);
            }

            // Comparison operations
            OpCode::Eq => {
                let b = self.runtime.pop()?;
                let a = self.runtime.pop()?;
                self.runtime.push(Value::Bool(a == b));
            }
            OpCode::Neq => {
                let b = self.runtime.pop()?;
                let a = self.runtime.pop()?;
                self.runtime.push(Value::Bool(a != b));
            }
            OpCode::Lt => {
                let b = self.runtime.pop()?;
                let a = self.runtime.pop()?;
                let result = self.compare_lt(a, b)?;
                self.runtime.push(Value::Bool(result));
            }
            OpCode::Gt => {
                let b = self.runtime.pop()?;
                let a = self.runtime.pop()?;
                let result = self.compare_lt(b, a)?; // gt = lt with swapped args
                self.runtime.push(Value::Bool(result));
            }
            OpCode::Lte => {
                let b = self.runtime.pop()?;
                let a = self.runtime.pop()?;
                let result = !self.compare_lt(b.clone(), a.clone())?; // lte = not gt
                self.runtime.push(Value::Bool(result));
            }
            OpCode::Gte => {
                let b = self.runtime.pop()?;
                let a = self.runtime.pop()?;
                let result = !self.compare_lt(a, b)?; // gte = not lt
                self.runtime.push(Value::Bool(result));
            }

            // Logical operations
            OpCode::And => {
                let b = self.runtime.pop()?;
                let a = self.runtime.pop()?;
                self.runtime.push(Value::Bool(a.is_truthy() && b.is_truthy()));
            }
            OpCode::Or => {
                let b = self.runtime.pop()?;
                let a = self.runtime.pop()?;
                self.runtime.push(Value::Bool(a.is_truthy() || b.is_truthy()));
            }
            OpCode::Not => {
                let a = self.runtime.pop()?;
                self.runtime.push(Value::Bool(!a.is_truthy()));
            }

            // Collection operations
            OpCode::Len => {
                let a = self.runtime.pop()?;
                let len = a.len().unwrap_or(0);
                self.runtime.push(Value::Int(len as i64));
            }
            OpCode::Index => {
                let index = self.runtime.pop()?;
                let base = self.runtime.pop()?;
                let result = self.index_access(base, index)?;
                self.runtime.push(result);
            }
            OpCode::GetField(name) => {
                let base = self.runtime.pop()?;
                let result = self.field_access(base, name)?;
                self.runtime.push(result);
            }
            OpCode::MakeArray(count) => {
                let mut items = Vec::new();
                for _ in 0..*count {
                    items.push(self.runtime.pop()?);
                }
                items.reverse();
                self.runtime.push(Value::Array(items));
            }
            OpCode::MakeStruct(fields) => {
                let mut struct_val = HashMap::new();
                for field in fields.iter().rev() {
                    let value = self.runtime.pop()?;
                    struct_val.insert(field.clone(), value);
                }
                self.runtime.push(Value::Struct(struct_val));
            }

            // Array operations (handled in node execution)
            OpCode::Map(_) | OpCode::Filter(_) | OpCode::Reduce(_, _) => {
                // These are handled at the node level
            }

            // Control flow
            OpCode::Jump(_) | OpCode::JumpIf(_) | OpCode::JumpIfNot(_) => {
                // Jump instructions are handled by the instruction executor
                // For now, we don't support jumps in this simple VM
            }
            OpCode::CallNode(node_id) => {
                // For now, just store the node_id reference
                self.runtime.push(Value::String(node_id.clone()));
            }
            OpCode::Return => {
                // Return is a no-op in this simple VM
            }

            // Built-in functions
            OpCode::CallBuiltin(name, arg_count) => {
                let mut args = Vec::new();
                for _ in 0..*arg_count {
                    args.push(self.runtime.pop()?);
                }
                args.reverse();
                let result = call_builtin(name, args)?;
                self.runtime.push(result);
            }

            // Special
            OpCode::Nop => {}
            OpCode::Halt => {
                return Err(RuntimeError::Halt("Execution halted".to_string()));
            }
            OpCode::Error(msg) => {
                return Err(RuntimeError::Internal(msg.clone()));
            }
        }
        Ok(())
    }

    // ==================== Map/Filter/Reduce ====================

    fn execute_map(&mut self, node: &NodeIR) -> RuntimeResult<()> {
        // Get input array from stack (or default)
        let input = if self.runtime.stack_depth() > 0 {
            self.runtime.pop()?
        } else {
            Value::Array(vec![])
        };

        if let Value::Array(items) = input {
            let mut results = Vec::new();
            for item in items {
                // Push item, execute instructions, pop result
                self.runtime.push(item);
                self.execute_instructions(&node.instructions)?;
                results.push(self.runtime.pop()?);
            }
            self.runtime.push(Value::Array(results));
        }
        Ok(())
    }

    fn execute_filter(&mut self, node: &NodeIR) -> RuntimeResult<()> {
        let input = if self.runtime.stack_depth() > 0 {
            self.runtime.pop()?
        } else {
            Value::Array(vec![])
        };

        if let Value::Array(items) = input {
            let mut results = Vec::new();
            for item in items {
                // Push item, execute predicate, check result
                self.runtime.push(item.clone());
                self.execute_instructions(&node.instructions)?;
                let keep = self.runtime.pop()?;
                if keep.is_truthy() {
                    results.push(item);
                }
            }
            self.runtime.push(Value::Array(results));
        }
        Ok(())
    }

    fn execute_reduce(&mut self, node: &NodeIR, reduce_op: &ReduceOp) -> RuntimeResult<()> {
        let input = if self.runtime.stack_depth() > 0 {
            self.runtime.pop()?
        } else {
            Value::Array(vec![])
        };

        if let Value::Array(items) = input {
            let result = match reduce_op {
                ReduceOp::Sum => {
                    let mut sum = 0i64;
                    for item in items {
                        if let Some(n) = item.as_int() {
                            sum += n;
                        }
                    }
                    Value::Int(sum)
                }
                ReduceOp::Count => Value::Int(items.len() as i64),
                ReduceOp::Min => {
                    call_builtin("MIN", vec![Value::Array(items)])?
                }
                ReduceOp::Max => {
                    call_builtin("MAX", vec![Value::Array(items)])?
                }
                ReduceOp::Avg => {
                    call_builtin("AVG", vec![Value::Array(items)])?
                }
                ReduceOp::First => items.first().cloned().unwrap_or(Value::Void),
                ReduceOp::Last => items.last().cloned().unwrap_or(Value::Void),
                ReduceOp::Custom(instructions) => {
                    // Custom reduce: execute instructions with accumulator
                    self.runtime.push(Value::Void);
                    for item in items {
                        self.runtime.push(item);
                        self.execute_instructions(instructions)?;
                    }
                    self.runtime.pop()?
                }
            };
            self.runtime.push(result);
        }
        Ok(())
    }

    // ==================== Binary Operations ====================

    fn binary_add(&self, a: Value, b: Value) -> RuntimeResult<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 + y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x + y as f64)),
            (Value::String(x), Value::String(y)) => Ok(Value::String(format!("{}{}", x, y))),
            _ => Ok(Value::Void),
        }
    }

    fn binary_sub(&self, a: Value, b: Value) -> RuntimeResult<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 - y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x - y as f64)),
            _ => Ok(Value::Void),
        }
    }

    fn binary_mul(&self, a: Value, b: Value) -> RuntimeResult<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 * y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x * y as f64)),
            _ => Ok(Value::Void),
        }
    }

    fn binary_div(&self, a: Value, b: Value) -> RuntimeResult<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => {
                if y == 0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                Ok(Value::Int(x / y))
            }
            (Value::Float(x), Value::Float(y)) => {
                if y == 0.0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                Ok(Value::Float(x / y))
            }
            (Value::Int(x), Value::Float(y)) => {
                if y == 0.0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                Ok(Value::Float(x as f64 / y))
            }
            (Value::Float(x), Value::Int(y)) => {
                if y == 0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                Ok(Value::Float(x / y as f64))
            }
            _ => Ok(Value::Void),
        }
    }

    fn unary_neg(&self, a: Value) -> RuntimeResult<Value> {
        match a {
            Value::Int(x) => Ok(Value::Int(-x)),
            Value::Float(x) => Ok(Value::Float(-x)),
            _ => Ok(Value::Void),
        }
    }

    fn compare_lt(&self, a: Value, b: Value) -> RuntimeResult<bool> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(x < y),
            (Value::Float(x), Value::Float(y)) => Ok(x < y),
            (Value::Int(x), Value::Float(y)) => Ok((x as f64) < y),
            (Value::Float(x), Value::Int(y)) => Ok(x < (y as f64)),
            (Value::String(x), Value::String(y)) => Ok(x < y),
            _ => Ok(false),
        }
    }

    // ==================== Collection Access ====================

    fn index_access(&self, base: Value, index: Value) -> RuntimeResult<Value> {
        match (base, index) {
            (Value::Array(arr), Value::Int(i)) => {
                let idx = if i < 0 {
                    arr.len() as i64 + i
                } else {
                    i
                };
                if idx < 0 || idx as usize >= arr.len() {
                    return Err(RuntimeError::IndexOutOfBounds {
                        index: i,
                        length: arr.len(),
                    });
                }
                Ok(arr[idx as usize].clone())
            }
            (Value::Map(map), Value::String(key)) => {
                Ok(map.get(&key).cloned().unwrap_or(Value::Void))
            }
            (Value::String(s), Value::Int(i)) => {
                let idx = if i < 0 {
                    s.len() as i64 + i
                } else {
                    i
                };
                if idx < 0 || idx as usize >= s.len() {
                    return Err(RuntimeError::IndexOutOfBounds {
                        index: i,
                        length: s.len(),
                    });
                }
                Ok(Value::String(
                    s.chars().nth(idx as usize).unwrap().to_string(),
                ))
            }
            _ => Ok(Value::Void),
        }
    }

    fn field_access(&self, base: Value, field: &str) -> RuntimeResult<Value> {
        match base {
            Value::Struct(s) => Ok(s.get(field).cloned().unwrap_or(Value::Void)),
            Value::Map(m) => Ok(m.get(field).cloned().unwrap_or(Value::Void)),
            other => Err(RuntimeError::InvalidFieldAccess {
                field: field.to_string(),
                value_type: other.value_type(),
            }),
        }
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoel_ir::{EdgeIR, Function, ModuleMetadata};

    fn create_simple_module() -> Module {
        let mut module = Module::new("test");
        module.add_input("x", "INT");
        module.add_output("result", "INT");

        // Create a simple node that doubles the input
        let mut node = NodeIR::new("double", NodeOpType::Transform);
        node.instructions = vec![
            Instruction::new(OpCode::LoadInput("x".to_string())),
            Instruction::new(OpCode::Const(Value::Int(2))),
            Instruction::new(OpCode::Mul),
            Instruction::new(OpCode::StoreOutput("result".to_string())),
        ];

        module.main.nodes.push(node);
        module.main.edges.push(EdgeIR::simple("INPUT", "double"));
        module.main.edges.push(EdgeIR::simple("double", "OUTPUT"));
        module.main.execution_order = vec!["double".to_string()];

        module
    }

    #[test]
    fn test_simple_execution() {
        let module = create_simple_module();
        let mut inputs = HashMap::new();
        inputs.insert("x".to_string(), Value::Int(21));

        let mut vm = Vm::new();
        let outputs = vm.execute(&module, inputs).unwrap();

        assert_eq!(outputs.get("result"), Some(&Value::Int(42)));
    }

    #[test]
    fn test_arithmetic() {
        let mut vm = Vm::new();

        // Test add
        vm.runtime.push(Value::Int(10));
        vm.runtime.push(Value::Int(5));
        vm.execute_instruction(&Instruction::new(OpCode::Add)).unwrap();
        assert_eq!(vm.runtime.pop().unwrap(), Value::Int(15));

        // Test sub
        vm.runtime.push(Value::Int(10));
        vm.runtime.push(Value::Int(3));
        vm.execute_instruction(&Instruction::new(OpCode::Sub)).unwrap();
        assert_eq!(vm.runtime.pop().unwrap(), Value::Int(7));

        // Test mul
        vm.runtime.push(Value::Int(6));
        vm.runtime.push(Value::Int(7));
        vm.execute_instruction(&Instruction::new(OpCode::Mul)).unwrap();
        assert_eq!(vm.runtime.pop().unwrap(), Value::Int(42));

        // Test div
        vm.runtime.push(Value::Int(20));
        vm.runtime.push(Value::Int(4));
        vm.execute_instruction(&Instruction::new(OpCode::Div)).unwrap();
        assert_eq!(vm.runtime.pop().unwrap(), Value::Int(5));
    }

    #[test]
    fn test_comparison() {
        let mut vm = Vm::new();

        // Test eq
        vm.runtime.push(Value::Int(5));
        vm.runtime.push(Value::Int(5));
        vm.execute_instruction(&Instruction::new(OpCode::Eq)).unwrap();
        assert_eq!(vm.runtime.pop().unwrap(), Value::Bool(true));

        // Test lt
        vm.runtime.push(Value::Int(3));
        vm.runtime.push(Value::Int(5));
        vm.execute_instruction(&Instruction::new(OpCode::Lt)).unwrap();
        assert_eq!(vm.runtime.pop().unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_logical() {
        let mut vm = Vm::new();

        // Test and
        vm.runtime.push(Value::Bool(true));
        vm.runtime.push(Value::Bool(false));
        vm.execute_instruction(&Instruction::new(OpCode::And)).unwrap();
        assert_eq!(vm.runtime.pop().unwrap(), Value::Bool(false));

        // Test or
        vm.runtime.push(Value::Bool(true));
        vm.runtime.push(Value::Bool(false));
        vm.execute_instruction(&Instruction::new(OpCode::Or)).unwrap();
        assert_eq!(vm.runtime.pop().unwrap(), Value::Bool(true));

        // Test not
        vm.runtime.push(Value::Bool(true));
        vm.execute_instruction(&Instruction::new(OpCode::Not)).unwrap();
        assert_eq!(vm.runtime.pop().unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_builtin_call() {
        let mut vm = Vm::new();

        // Test LEN builtin
        vm.runtime.push(Value::String("hello".to_string()));
        vm.execute_instruction(&Instruction::new(OpCode::CallBuiltin("LEN".to_string(), 1)))
            .unwrap();
        assert_eq!(vm.runtime.pop().unwrap(), Value::Int(5));
    }
}

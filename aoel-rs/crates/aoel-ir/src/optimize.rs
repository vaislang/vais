//! IR Optimization Passes
//!
//! This module provides optimization passes for AOEL IR.

use std::collections::HashMap;
use crate::instruction::{Instruction, NodeIR, OpCode};
use crate::module::{Function, Module};
use crate::value::Value;

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptLevel {
    /// No optimization
    None,
    /// Basic optimizations (constant folding, dead code elimination)
    Basic,
    /// Aggressive optimizations (includes constant propagation, CSE, instruction fusion)
    Aggressive,
}

/// Run all optimizations on a module
pub fn optimize(module: &mut Module, level: OptLevel) {
    if level == OptLevel::None {
        return;
    }

    // Basic optimizations
    // Run constant folding
    constant_folding(&mut module.main);

    // Run dead code elimination
    dead_code_elimination(&mut module.main);

    if level == OptLevel::Aggressive {
        // Aggressive optimizations
        // Run constant propagation
        constant_propagation(&mut module.main);

        // Run common subexpression elimination
        common_subexpression_elimination(&mut module.main);

        // Run instruction fusion
        instruction_fusion(&mut module.main);

        // Run tail call optimization
        tail_call_optimization(&mut module.main);

        // Run constant folding again after propagation
        constant_folding(&mut module.main);

        // Final dead code elimination
        dead_code_elimination(&mut module.main);
    }
}

/// Constant folding optimization
///
/// Evaluates constant expressions at compile time.
/// For example: `Const(2) + Const(3)` becomes `Const(5)`
pub fn constant_folding(function: &mut Function) {
    for node in &mut function.nodes {
        fold_node_constants(node);
    }
}

fn fold_node_constants(node: &mut NodeIR) {
    let mut new_instructions = Vec::new();
    let mut i = 0;
    let instructions = &node.instructions;

    while i < instructions.len() {
        // Look for patterns like: Const(a), Const(b), BinaryOp
        if i + 2 < instructions.len() {
            if let (Some(a), Some(b)) = (
                extract_const(&instructions[i]),
                extract_const(&instructions[i + 1]),
            ) {
                if let Some(folded) = try_fold_binary(&a, &b, &instructions[i + 2].opcode) {
                    new_instructions.push(Instruction::new(OpCode::Const(folded)));
                    i += 3;
                    continue;
                }
            }
        }

        // Look for patterns like: Const(a), UnaryOp
        if i + 1 < instructions.len() {
            if let Some(a) = extract_const(&instructions[i]) {
                if let Some(folded) = try_fold_unary(&a, &instructions[i + 1].opcode) {
                    new_instructions.push(Instruction::new(OpCode::Const(folded)));
                    i += 2;
                    continue;
                }
            }
        }

        new_instructions.push(instructions[i].clone());
        i += 1;
    }

    node.instructions = new_instructions;
}

fn extract_const(instr: &Instruction) -> Option<Value> {
    if let OpCode::Const(v) = &instr.opcode {
        Some(v.clone())
    } else {
        None
    }
}

fn try_fold_binary(a: &Value, b: &Value, op: &OpCode) -> Option<Value> {
    match op {
        OpCode::Add => match (a, b) {
            (Value::Int(x), Value::Int(y)) => Some(Value::Int(x + y)),
            (Value::Float(x), Value::Float(y)) => Some(Value::Float(x + y)),
            (Value::Int(x), Value::Float(y)) => Some(Value::Float(*x as f64 + y)),
            (Value::Float(x), Value::Int(y)) => Some(Value::Float(x + *y as f64)),
            (Value::String(x), Value::String(y)) => Some(Value::String(format!("{}{}", x, y))),
            _ => None,
        },
        OpCode::Sub => match (a, b) {
            (Value::Int(x), Value::Int(y)) => Some(Value::Int(x - y)),
            (Value::Float(x), Value::Float(y)) => Some(Value::Float(x - y)),
            (Value::Int(x), Value::Float(y)) => Some(Value::Float(*x as f64 - y)),
            (Value::Float(x), Value::Int(y)) => Some(Value::Float(x - *y as f64)),
            _ => None,
        },
        OpCode::Mul => match (a, b) {
            (Value::Int(x), Value::Int(y)) => Some(Value::Int(x * y)),
            (Value::Float(x), Value::Float(y)) => Some(Value::Float(x * y)),
            (Value::Int(x), Value::Float(y)) => Some(Value::Float(*x as f64 * y)),
            (Value::Float(x), Value::Int(y)) => Some(Value::Float(x * *y as f64)),
            _ => None,
        },
        OpCode::Div => match (a, b) {
            (Value::Int(x), Value::Int(y)) if *y != 0 => Some(Value::Int(x / y)),
            (Value::Float(x), Value::Float(y)) if *y != 0.0 => Some(Value::Float(x / y)),
            (Value::Int(x), Value::Float(y)) if *y != 0.0 => Some(Value::Float(*x as f64 / y)),
            (Value::Float(x), Value::Int(y)) if *y != 0 => Some(Value::Float(x / *y as f64)),
            _ => None,
        },
        OpCode::Eq => Some(Value::Bool(a == b)),
        OpCode::Neq => Some(Value::Bool(a != b)),
        OpCode::Lt => match (a, b) {
            (Value::Int(x), Value::Int(y)) => Some(Value::Bool(x < y)),
            (Value::Float(x), Value::Float(y)) => Some(Value::Bool(x < y)),
            (Value::String(x), Value::String(y)) => Some(Value::Bool(x < y)),
            _ => None,
        },
        OpCode::Gt => match (a, b) {
            (Value::Int(x), Value::Int(y)) => Some(Value::Bool(x > y)),
            (Value::Float(x), Value::Float(y)) => Some(Value::Bool(x > y)),
            (Value::String(x), Value::String(y)) => Some(Value::Bool(x > y)),
            _ => None,
        },
        OpCode::Lte => match (a, b) {
            (Value::Int(x), Value::Int(y)) => Some(Value::Bool(x <= y)),
            (Value::Float(x), Value::Float(y)) => Some(Value::Bool(x <= y)),
            (Value::String(x), Value::String(y)) => Some(Value::Bool(x <= y)),
            _ => None,
        },
        OpCode::Gte => match (a, b) {
            (Value::Int(x), Value::Int(y)) => Some(Value::Bool(x >= y)),
            (Value::Float(x), Value::Float(y)) => Some(Value::Bool(x >= y)),
            (Value::String(x), Value::String(y)) => Some(Value::Bool(x >= y)),
            _ => None,
        },
        OpCode::And => match (a, b) {
            (Value::Bool(x), Value::Bool(y)) => Some(Value::Bool(*x && *y)),
            _ => None,
        },
        OpCode::Or => match (a, b) {
            (Value::Bool(x), Value::Bool(y)) => Some(Value::Bool(*x || *y)),
            _ => None,
        },
        _ => None,
    }
}

fn try_fold_unary(a: &Value, op: &OpCode) -> Option<Value> {
    match op {
        OpCode::Neg => match a {
            Value::Int(x) => Some(Value::Int(-x)),
            Value::Float(x) => Some(Value::Float(-x)),
            _ => None,
        },
        OpCode::Not => match a {
            Value::Bool(x) => Some(Value::Bool(!x)),
            _ => None,
        },
        _ => None,
    }
}

/// Dead code elimination
///
/// Removes instructions that have no effect on the output.
pub fn dead_code_elimination(function: &mut Function) {
    for node in &mut function.nodes {
        eliminate_dead_code(node);
    }
}

fn eliminate_dead_code(node: &mut NodeIR) {
    let mut new_instructions = Vec::new();
    let mut i = 0;
    let instructions = &node.instructions;

    while i < instructions.len() {
        // Remove Nop instructions
        if matches!(instructions[i].opcode, OpCode::Nop) {
            i += 1;
            continue;
        }

        // Remove Const followed by Pop (useless push-pop)
        if i + 1 < instructions.len()
            && matches!(instructions[i].opcode, OpCode::Const(_))
            && matches!(instructions[i + 1].opcode, OpCode::Pop)
        {
            i += 2;
            continue;
        }

        // Remove duplicate Dup followed by Pop
        if i + 1 < instructions.len()
            && matches!(instructions[i].opcode, OpCode::Dup)
            && matches!(instructions[i + 1].opcode, OpCode::Pop)
        {
            i += 2;
            continue;
        }

        // Remove Load followed by Pop (useless load-pop)
        if i + 1 < instructions.len()
            && matches!(instructions[i].opcode, OpCode::Load(_))
            && matches!(instructions[i + 1].opcode, OpCode::Pop)
        {
            i += 2;
            continue;
        }

        new_instructions.push(instructions[i].clone());
        i += 1;
    }

    node.instructions = new_instructions;
}

/// Constant propagation optimization
///
/// Tracks constant values assigned to variables and replaces
/// Load instructions with the constant value when possible.
///
/// Example:
/// ```text
/// Store("x", 5)     // x = 5
/// ...
/// Load("x")         // becomes Const(5)
/// ```
pub fn constant_propagation(function: &mut Function) {
    for node in &mut function.nodes {
        propagate_constants(node);
    }
}

fn propagate_constants(node: &mut NodeIR) {
    // Track known constant values for variables
    let mut constants: HashMap<String, Value> = HashMap::new();
    let mut new_instructions: Vec<Instruction> = Vec::new();

    for instr in &node.instructions {
        match &instr.opcode {
            // Track Store of constants
            OpCode::Store(name) => {
                // Check if the previous instruction was a constant
                if let Some(prev) = new_instructions.last() {
                    if let OpCode::Const(v) = &prev.opcode {
                        constants.insert(name.clone(), v.clone());
                    } else {
                        // Variable is being assigned a non-constant value
                        constants.remove(name);
                    }
                }
                new_instructions.push(instr.clone());
            }

            // Replace Load with constant if known
            OpCode::Load(name) => {
                if let Some(value) = constants.get(name) {
                    new_instructions.push(Instruction::new(OpCode::Const(value.clone())));
                } else {
                    new_instructions.push(instr.clone());
                }
            }

            // Instructions that might modify variables invalidate our knowledge
            OpCode::Call(_, _) | OpCode::SelfCall(_) | OpCode::CallClosure(_) => {
                // Function calls might modify any variable, clear all knowledge
                constants.clear();
                new_instructions.push(instr.clone());
            }

            // Jump instructions create control flow complexity, be conservative
            OpCode::Jump(_) | OpCode::JumpIf(_) | OpCode::JumpIfNot(_) => {
                constants.clear();
                new_instructions.push(instr.clone());
            }

            _ => {
                new_instructions.push(instr.clone());
            }
        }
    }

    node.instructions = new_instructions;
}

/// Common subexpression elimination (CSE)
///
/// Identifies repeated computations and reuses their results.
///
/// Example:
/// ```text
/// Load("a"), Load("b"), Add   // first a + b
/// ...
/// Load("a"), Load("b"), Add   // second a + b -> reuse first result
/// ```
pub fn common_subexpression_elimination(function: &mut Function) {
    for node in &mut function.nodes {
        eliminate_common_subexpressions(node);
    }
}

/// Hash key for identifying common subexpressions
fn expression_key(instrs: &[&Instruction]) -> Option<String> {
    let mut key = String::new();
    for instr in instrs {
        match &instr.opcode {
            OpCode::Load(name) => key.push_str(&format!("L:{},", name)),
            OpCode::Const(v) => key.push_str(&format!("C:{:?},", v)),
            OpCode::Add => key.push_str("Add,"),
            OpCode::Sub => key.push_str("Sub,"),
            OpCode::Mul => key.push_str("Mul,"),
            OpCode::Div => key.push_str("Div,"),
            OpCode::Mod => key.push_str("Mod,"),
            OpCode::Eq => key.push_str("Eq,"),
            OpCode::Neq => key.push_str("Neq,"),
            OpCode::Lt => key.push_str("Lt,"),
            OpCode::Gt => key.push_str("Gt,"),
            OpCode::Lte => key.push_str("Lte,"),
            OpCode::Gte => key.push_str("Gte,"),
            OpCode::And => key.push_str("And,"),
            OpCode::Or => key.push_str("Or,"),
            _ => return None, // Not a simple expression
        }
    }
    Some(key)
}

fn eliminate_common_subexpressions(node: &mut NodeIR) {
    // Track seen expressions: key -> temp variable name
    let mut seen: HashMap<String, String> = HashMap::new();
    let mut temp_counter = 0;
    let mut new_instructions = Vec::new();
    let instructions = &node.instructions;

    let mut i = 0;
    while i < instructions.len() {
        // Look for binary expression patterns: operand1, operand2, operator
        if i + 2 < instructions.len() {
            let expr_instrs = [&instructions[i], &instructions[i + 1], &instructions[i + 2]];

            // Check if it's a binary operation
            if is_binary_op(&instructions[i + 2].opcode) {
                if let Some(key) = expression_key(&expr_instrs) {
                    if let Some(temp_name) = seen.get(&key) {
                        // Reuse the previously computed value
                        new_instructions.push(Instruction::new(OpCode::Load(temp_name.clone())));
                        i += 3;
                        continue;
                    } else {
                        // First occurrence: compute and store in temp
                        let temp_name = format!("__cse_temp_{}", temp_counter);
                        temp_counter += 1;

                        // Emit the computation
                        new_instructions.push(instructions[i].clone());
                        new_instructions.push(instructions[i + 1].clone());
                        new_instructions.push(instructions[i + 2].clone());

                        // Store result in temp (Dup + Store to keep value on stack)
                        new_instructions.push(Instruction::new(OpCode::Dup));
                        new_instructions.push(Instruction::new(OpCode::Store(temp_name.clone())));

                        seen.insert(key, temp_name);
                        i += 3;
                        continue;
                    }
                }
            }
        }

        // Check if store invalidates any cached expressions
        if let OpCode::Store(name) = &instructions[i].opcode {
            // Invalidate expressions that use this variable
            seen.retain(|key, _| !key.contains(&format!("L:{},", name)));
        }

        // Control flow invalidates all cached expressions
        if matches!(
            instructions[i].opcode,
            OpCode::Jump(_)
                | OpCode::JumpIf(_)
                | OpCode::JumpIfNot(_)
                | OpCode::Call(_, _)
                | OpCode::SelfCall(_)
                | OpCode::CallClosure(_)
        ) {
            seen.clear();
        }

        new_instructions.push(instructions[i].clone());
        i += 1;
    }

    node.instructions = new_instructions;
}

fn is_binary_op(op: &OpCode) -> bool {
    matches!(
        op,
        OpCode::Add
            | OpCode::Sub
            | OpCode::Mul
            | OpCode::Div
            | OpCode::Mod
            | OpCode::Eq
            | OpCode::Neq
            | OpCode::Lt
            | OpCode::Gt
            | OpCode::Lte
            | OpCode::Gte
            | OpCode::And
            | OpCode::Or
    )
}

/// Instruction fusion optimization
///
/// Combines multiple instructions into more efficient single operations.
///
/// Examples:
/// - Const(0), Add -> (remove both, no-op for addition)
/// - Const(1), Mul -> (remove both, no-op for multiplication)
/// - Const(0), Mul -> Const(0) (anything * 0 = 0)
/// - Load(x), Load(x), Add -> Load(x), Const(2), Mul (x + x = 2 * x)
pub fn instruction_fusion(function: &mut Function) {
    for node in &mut function.nodes {
        fuse_instructions(node);
    }
}

fn fuse_instructions(node: &mut NodeIR) {
    let mut new_instructions = Vec::new();
    let mut i = 0;
    let instructions = &node.instructions;

    while i < instructions.len() {
        // Pattern: x + 0 or 0 + x (identity for addition)
        if i + 2 < instructions.len() {
            if let OpCode::Const(Value::Int(0)) = &instructions[i + 1].opcode {
                if matches!(instructions[i + 2].opcode, OpCode::Add) {
                    // x + 0 = x, just keep the first operand
                    new_instructions.push(instructions[i].clone());
                    i += 3;
                    continue;
                }
            }
            if let OpCode::Const(Value::Int(0)) = &instructions[i].opcode {
                if matches!(instructions[i + 2].opcode, OpCode::Add) {
                    // 0 + x = x, just keep the second operand
                    new_instructions.push(instructions[i + 1].clone());
                    i += 3;
                    continue;
                }
            }

            // Pattern: x * 1 or 1 * x (identity for multiplication)
            if let OpCode::Const(Value::Int(1)) = &instructions[i + 1].opcode {
                if matches!(instructions[i + 2].opcode, OpCode::Mul) {
                    // x * 1 = x
                    new_instructions.push(instructions[i].clone());
                    i += 3;
                    continue;
                }
            }
            if let OpCode::Const(Value::Int(1)) = &instructions[i].opcode {
                if matches!(instructions[i + 2].opcode, OpCode::Mul) {
                    // 1 * x = x
                    new_instructions.push(instructions[i + 1].clone());
                    i += 3;
                    continue;
                }
            }

            // Pattern: x * 0 or 0 * x (zero for multiplication)
            if let OpCode::Const(Value::Int(0)) = &instructions[i + 1].opcode {
                if matches!(instructions[i + 2].opcode, OpCode::Mul) {
                    // x * 0 = 0 (but we need to consume x from stack)
                    new_instructions.push(instructions[i].clone());
                    new_instructions.push(Instruction::new(OpCode::Pop));
                    new_instructions.push(Instruction::new(OpCode::Const(Value::Int(0))));
                    i += 3;
                    continue;
                }
            }
            if let OpCode::Const(Value::Int(0)) = &instructions[i].opcode {
                if matches!(instructions[i + 2].opcode, OpCode::Mul) {
                    // 0 * x = 0
                    new_instructions.push(instructions[i + 1].clone());
                    new_instructions.push(Instruction::new(OpCode::Pop));
                    new_instructions.push(Instruction::new(OpCode::Const(Value::Int(0))));
                    i += 3;
                    continue;
                }
            }

            // Pattern: x - 0 (identity for subtraction)
            if let OpCode::Const(Value::Int(0)) = &instructions[i + 1].opcode {
                if matches!(instructions[i + 2].opcode, OpCode::Sub) {
                    // x - 0 = x
                    new_instructions.push(instructions[i].clone());
                    i += 3;
                    continue;
                }
            }

            // Pattern: x / 1 (identity for division)
            if let OpCode::Const(Value::Int(1)) = &instructions[i + 1].opcode {
                if matches!(instructions[i + 2].opcode, OpCode::Div) {
                    // x / 1 = x
                    new_instructions.push(instructions[i].clone());
                    i += 3;
                    continue;
                }
            }

            // Pattern: x + x -> 2 * x (strength reduction)
            if let (OpCode::Load(name1), OpCode::Load(name2)) =
                (&instructions[i].opcode, &instructions[i + 1].opcode)
            {
                if name1 == name2 && matches!(instructions[i + 2].opcode, OpCode::Add) {
                    // x + x = 2 * x
                    new_instructions.push(Instruction::new(OpCode::Load(name1.clone())));
                    new_instructions.push(Instruction::new(OpCode::Const(Value::Int(2))));
                    new_instructions.push(Instruction::new(OpCode::Mul));
                    i += 3;
                    continue;
                }
            }

            // Pattern: x * 2 -> x + x (strength reduction - if shift not available)
            // Skip this for now as multiplication is typically efficient

            // Pattern: double negation
            if i + 1 < instructions.len()
                && matches!(instructions[i].opcode, OpCode::Neg)
                && matches!(instructions[i + 1].opcode, OpCode::Neg)
            {
                // -(-x) = x, remove both negations
                i += 2;
                continue;
            }

            // Pattern: double not
            if i + 1 < instructions.len()
                && matches!(instructions[i].opcode, OpCode::Not)
                && matches!(instructions[i + 1].opcode, OpCode::Not)
            {
                // !!x = x, remove both nots
                i += 2;
                continue;
            }
        }

        new_instructions.push(instructions[i].clone());
        i += 1;
    }

    node.instructions = new_instructions;
}

/// Tail Call Optimization (TCO)
///
/// Detects tail-recursive calls (SelfCall immediately followed by Return)
/// and converts them to TailSelfCall which can be executed without growing
/// the call stack.
///
/// A tail call is when the result of a recursive call is immediately returned
/// without any additional computation.
///
/// Example:
/// ```text
/// // Before TCO:
/// SelfCall(1)
/// Return
///
/// // After TCO:
/// TailSelfCall(1)
/// Return
/// ```
pub fn tail_call_optimization(function: &mut Function) {
    for node in &mut function.nodes {
        optimize_tail_calls(node);
    }
}

fn optimize_tail_calls(node: &mut NodeIR) {
    let mut new_instructions = Vec::new();
    let mut i = 0;
    let instructions = &node.instructions;

    while i < instructions.len() {
        // Pattern: SelfCall followed by Return
        if i + 1 < instructions.len() {
            if let OpCode::SelfCall(arg_count) = &instructions[i].opcode {
                if matches!(instructions[i + 1].opcode, OpCode::Return) {
                    // Convert to TailSelfCall
                    new_instructions.push(Instruction::new(OpCode::TailSelfCall(*arg_count)));
                    new_instructions.push(instructions[i + 1].clone()); // Keep Return
                    i += 2;
                    continue;
                }
            }
        }

        new_instructions.push(instructions[i].clone());
        i += 1;
    }

    node.instructions = new_instructions;
}

/// Check if a function can benefit from TCO
/// Returns true if there are any tail-recursive SelfCall patterns
pub fn has_tail_calls(instructions: &[Instruction]) -> bool {
    for i in 0..instructions.len().saturating_sub(1) {
        if let OpCode::SelfCall(_) = &instructions[i].opcode {
            if matches!(instructions[i + 1].opcode, OpCode::Return) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::NodeOpType;

    #[test]
    fn test_constant_folding_add() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        node.instructions = vec![
            Instruction::new(OpCode::Const(Value::Int(2))),
            Instruction::new(OpCode::Const(Value::Int(3))),
            Instruction::new(OpCode::Add),
        ];

        fold_node_constants(&mut node);

        assert_eq!(node.instructions.len(), 1);
        assert_eq!(node.instructions[0].opcode, OpCode::Const(Value::Int(5)));
    }

    #[test]
    fn test_constant_folding_mul() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        node.instructions = vec![
            Instruction::new(OpCode::Const(Value::Int(4))),
            Instruction::new(OpCode::Const(Value::Int(7))),
            Instruction::new(OpCode::Mul),
        ];

        fold_node_constants(&mut node);

        assert_eq!(node.instructions.len(), 1);
        assert_eq!(node.instructions[0].opcode, OpCode::Const(Value::Int(28)));
    }

    #[test]
    fn test_constant_folding_string_concat() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        node.instructions = vec![
            Instruction::new(OpCode::Const(Value::String("Hello, ".to_string()))),
            Instruction::new(OpCode::Const(Value::String("World!".to_string()))),
            Instruction::new(OpCode::Add),
        ];

        fold_node_constants(&mut node);

        assert_eq!(node.instructions.len(), 1);
        assert_eq!(
            node.instructions[0].opcode,
            OpCode::Const(Value::String("Hello, World!".to_string()))
        );
    }

    #[test]
    fn test_constant_folding_comparison() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        node.instructions = vec![
            Instruction::new(OpCode::Const(Value::Int(5))),
            Instruction::new(OpCode::Const(Value::Int(3))),
            Instruction::new(OpCode::Gt),
        ];

        fold_node_constants(&mut node);

        assert_eq!(node.instructions.len(), 1);
        assert_eq!(node.instructions[0].opcode, OpCode::Const(Value::Bool(true)));
    }

    #[test]
    fn test_constant_folding_unary_neg() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        node.instructions = vec![
            Instruction::new(OpCode::Const(Value::Int(42))),
            Instruction::new(OpCode::Neg),
        ];

        fold_node_constants(&mut node);

        assert_eq!(node.instructions.len(), 1);
        assert_eq!(node.instructions[0].opcode, OpCode::Const(Value::Int(-42)));
    }

    #[test]
    fn test_dead_code_elimination_nop() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        node.instructions = vec![
            Instruction::new(OpCode::Const(Value::Int(1))),
            Instruction::new(OpCode::Nop),
            Instruction::new(OpCode::Const(Value::Int(2))),
            Instruction::new(OpCode::Nop),
        ];

        eliminate_dead_code(&mut node);

        assert_eq!(node.instructions.len(), 2);
        assert_eq!(node.instructions[0].opcode, OpCode::Const(Value::Int(1)));
        assert_eq!(node.instructions[1].opcode, OpCode::Const(Value::Int(2)));
    }

    #[test]
    fn test_dead_code_elimination_push_pop() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        node.instructions = vec![
            Instruction::new(OpCode::Const(Value::Int(42))),
            Instruction::new(OpCode::Pop),
            Instruction::new(OpCode::Const(Value::Int(1))),
        ];

        eliminate_dead_code(&mut node);

        assert_eq!(node.instructions.len(), 1);
        assert_eq!(node.instructions[0].opcode, OpCode::Const(Value::Int(1)));
    }

    #[test]
    fn test_dead_code_elimination_dup_pop() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        node.instructions = vec![
            Instruction::new(OpCode::Const(Value::Int(42))),
            Instruction::new(OpCode::Dup),
            Instruction::new(OpCode::Pop),
        ];

        eliminate_dead_code(&mut node);

        assert_eq!(node.instructions.len(), 1);
        assert_eq!(node.instructions[0].opcode, OpCode::Const(Value::Int(42)));
    }

    // === Constant Propagation Tests ===

    #[test]
    fn test_constant_propagation_simple() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        // x = 5; y = x + 3
        node.instructions = vec![
            Instruction::new(OpCode::Const(Value::Int(5))),
            Instruction::new(OpCode::Store("x".to_string())),
            Instruction::new(OpCode::Load("x".to_string())), // Should become Const(5)
            Instruction::new(OpCode::Const(Value::Int(3))),
            Instruction::new(OpCode::Add),
        ];

        propagate_constants(&mut node);

        // Load("x") should be replaced with Const(5)
        assert_eq!(node.instructions[2].opcode, OpCode::Const(Value::Int(5)));
    }

    #[test]
    fn test_constant_propagation_multiple() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        // x = 10; y = 20; z = x + y
        node.instructions = vec![
            Instruction::new(OpCode::Const(Value::Int(10))),
            Instruction::new(OpCode::Store("x".to_string())),
            Instruction::new(OpCode::Const(Value::Int(20))),
            Instruction::new(OpCode::Store("y".to_string())),
            Instruction::new(OpCode::Load("x".to_string())), // Should become Const(10)
            Instruction::new(OpCode::Load("y".to_string())), // Should become Const(20)
            Instruction::new(OpCode::Add),
        ];

        propagate_constants(&mut node);

        assert_eq!(node.instructions[4].opcode, OpCode::Const(Value::Int(10)));
        assert_eq!(node.instructions[5].opcode, OpCode::Const(Value::Int(20)));
    }

    // === CSE Tests ===

    #[test]
    fn test_cse_simple() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        // a + b; ... ; a + b (second should reuse first)
        node.instructions = vec![
            Instruction::new(OpCode::Load("a".to_string())),
            Instruction::new(OpCode::Load("b".to_string())),
            Instruction::new(OpCode::Add),
            Instruction::new(OpCode::Store("result1".to_string())),
            // Second occurrence of a + b
            Instruction::new(OpCode::Load("a".to_string())),
            Instruction::new(OpCode::Load("b".to_string())),
            Instruction::new(OpCode::Add),
        ];

        eliminate_common_subexpressions(&mut node);

        // The second a + b should be replaced with a Load from temp
        // First occurrence: Load(a), Load(b), Add, Dup, Store(__cse_temp_0)
        // Second occurrence: Load(__cse_temp_0)
        let len = node.instructions.len();
        assert!(len > 0);
        // Last instruction should be Load(__cse_temp_0)
        if let OpCode::Load(name) = &node.instructions[len - 1].opcode {
            assert!(name.starts_with("__cse_temp_"));
        } else {
            panic!("Expected Load for CSE temp variable");
        }
    }

    // === Instruction Fusion Tests ===

    #[test]
    fn test_fusion_add_zero() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        // x + 0 should become just x
        node.instructions = vec![
            Instruction::new(OpCode::Load("x".to_string())),
            Instruction::new(OpCode::Const(Value::Int(0))),
            Instruction::new(OpCode::Add),
        ];

        fuse_instructions(&mut node);

        assert_eq!(node.instructions.len(), 1);
        assert_eq!(node.instructions[0].opcode, OpCode::Load("x".to_string()));
    }

    #[test]
    fn test_fusion_mul_one() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        // x * 1 should become just x
        node.instructions = vec![
            Instruction::new(OpCode::Load("x".to_string())),
            Instruction::new(OpCode::Const(Value::Int(1))),
            Instruction::new(OpCode::Mul),
        ];

        fuse_instructions(&mut node);

        assert_eq!(node.instructions.len(), 1);
        assert_eq!(node.instructions[0].opcode, OpCode::Load("x".to_string()));
    }

    #[test]
    fn test_fusion_mul_zero() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        // x * 0 should result in 0
        node.instructions = vec![
            Instruction::new(OpCode::Load("x".to_string())),
            Instruction::new(OpCode::Const(Value::Int(0))),
            Instruction::new(OpCode::Mul),
        ];

        fuse_instructions(&mut node);

        // Should be: Load(x), Pop, Const(0)
        assert_eq!(node.instructions.len(), 3);
        assert_eq!(node.instructions[2].opcode, OpCode::Const(Value::Int(0)));
    }

    #[test]
    fn test_fusion_x_plus_x() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        // x + x should become 2 * x
        node.instructions = vec![
            Instruction::new(OpCode::Load("x".to_string())),
            Instruction::new(OpCode::Load("x".to_string())),
            Instruction::new(OpCode::Add),
        ];

        fuse_instructions(&mut node);

        assert_eq!(node.instructions.len(), 3);
        assert_eq!(node.instructions[0].opcode, OpCode::Load("x".to_string()));
        assert_eq!(node.instructions[1].opcode, OpCode::Const(Value::Int(2)));
        assert_eq!(node.instructions[2].opcode, OpCode::Mul);
    }

    #[test]
    fn test_fusion_sub_zero() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        // x - 0 should become just x
        node.instructions = vec![
            Instruction::new(OpCode::Load("x".to_string())),
            Instruction::new(OpCode::Const(Value::Int(0))),
            Instruction::new(OpCode::Sub),
        ];

        fuse_instructions(&mut node);

        assert_eq!(node.instructions.len(), 1);
        assert_eq!(node.instructions[0].opcode, OpCode::Load("x".to_string()));
    }

    #[test]
    fn test_fusion_div_one() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        // x / 1 should become just x
        node.instructions = vec![
            Instruction::new(OpCode::Load("x".to_string())),
            Instruction::new(OpCode::Const(Value::Int(1))),
            Instruction::new(OpCode::Div),
        ];

        fuse_instructions(&mut node);

        assert_eq!(node.instructions.len(), 1);
        assert_eq!(node.instructions[0].opcode, OpCode::Load("x".to_string()));
    }

    // === Tail Call Optimization Tests ===

    #[test]
    fn test_tco_simple() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        // Pattern: SelfCall followed by Return -> should become TailSelfCall
        node.instructions = vec![
            Instruction::new(OpCode::Load("n".to_string())),
            Instruction::new(OpCode::SelfCall(1)),
            Instruction::new(OpCode::Return),
        ];

        optimize_tail_calls(&mut node);

        assert_eq!(node.instructions.len(), 3);
        assert_eq!(node.instructions[0].opcode, OpCode::Load("n".to_string()));
        assert_eq!(node.instructions[1].opcode, OpCode::TailSelfCall(1));
        assert_eq!(node.instructions[2].opcode, OpCode::Return);
    }

    #[test]
    fn test_tco_non_tail_preserved() {
        let mut node = NodeIR::new("test", NodeOpType::Transform);
        // Pattern: SelfCall followed by computation -> NOT a tail call
        node.instructions = vec![
            Instruction::new(OpCode::Load("n".to_string())),
            Instruction::new(OpCode::SelfCall(1)),
            Instruction::new(OpCode::Const(Value::Int(1))),
            Instruction::new(OpCode::Add),
            Instruction::new(OpCode::Return),
        ];

        optimize_tail_calls(&mut node);

        // SelfCall should NOT be converted because it's not followed by Return
        assert_eq!(node.instructions.len(), 5);
        assert_eq!(node.instructions[1].opcode, OpCode::SelfCall(1));
    }

    #[test]
    fn test_has_tail_calls() {
        let tail_instructions = vec![
            Instruction::new(OpCode::Load("n".to_string())),
            Instruction::new(OpCode::SelfCall(1)),
            Instruction::new(OpCode::Return),
        ];

        let non_tail_instructions = vec![
            Instruction::new(OpCode::Load("n".to_string())),
            Instruction::new(OpCode::SelfCall(1)),
            Instruction::new(OpCode::Const(Value::Int(1))),
            Instruction::new(OpCode::Add),
            Instruction::new(OpCode::Return),
        ];

        assert!(has_tail_calls(&tail_instructions));
        assert!(!has_tail_calls(&non_tail_instructions));
    }
}

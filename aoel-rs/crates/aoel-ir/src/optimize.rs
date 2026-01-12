//! IR Optimization Passes
//!
//! This module provides optimization passes for AOEL IR.

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
    /// Aggressive optimizations
    Aggressive,
}

/// Run all optimizations on a module
pub fn optimize(module: &mut Module, level: OptLevel) {
    if level == OptLevel::None {
        return;
    }

    // Run constant folding
    constant_folding(&mut module.main);

    // Run dead code elimination
    dead_code_elimination(&mut module.main);
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
        if i + 1 < instructions.len() {
            if matches!(instructions[i].opcode, OpCode::Const(_))
                && matches!(instructions[i + 1].opcode, OpCode::Pop)
            {
                i += 2;
                continue;
            }
        }

        // Remove duplicate Dup followed by Pop
        if i + 1 < instructions.len() {
            if matches!(instructions[i].opcode, OpCode::Dup)
                && matches!(instructions[i + 1].opcode, OpCode::Pop)
            {
                i += 2;
                continue;
            }
        }

        new_instructions.push(instructions[i].clone());
        i += 1;
    }

    node.instructions = new_instructions;
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
}

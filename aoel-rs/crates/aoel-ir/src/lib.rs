//! AOEL Intermediate Representation
//!
//! This crate defines the IR for AOEL execution.
//! The IR is a stack-based representation optimized for execution.

mod value;
mod instruction;
mod module;
mod optimize;

pub use value::{Value, ValueType};
pub use instruction::{Instruction, OpCode, NodeIR, EdgeIR, NodeOpType, ReduceOp};
pub use module::{Module, Function, FieldDef, ModuleMetadata};
pub use optimize::{
    optimize, constant_folding, dead_code_elimination,
    constant_propagation, common_subexpression_elimination, instruction_fusion,
    tail_call_optimization, has_tail_calls,
    OptLevel
};

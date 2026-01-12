//! AOEL Intermediate Representation
//!
//! This crate defines the IR for AOEL execution.
//! The IR is a graph-based representation optimized for execution.

mod value;
mod instruction;
mod module;
mod lowering;
mod optimize;

pub use value::{Value, ValueType};
pub use instruction::{Instruction, OpCode, NodeIR, EdgeIR, NodeOpType, ReduceOp};
pub use module::{Module, Function, FieldDef, ModuleMetadata};
pub use lowering::lower;
pub use optimize::{optimize, constant_folding, dead_code_elimination, OptLevel};

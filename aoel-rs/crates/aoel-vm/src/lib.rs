//! AOEL Virtual Machine
//!
//! Stack-based VM for executing AOEL IR.

mod error;
mod runtime;
mod vm;
mod builtins;

pub use error::{RuntimeError, RuntimeResult};
pub use runtime::Runtime;
pub use vm::Vm;

/// Execute an IR module with the given inputs
pub fn execute(
    module: &aoel_ir::Module,
    inputs: std::collections::HashMap<String, aoel_ir::Value>,
) -> RuntimeResult<std::collections::HashMap<String, aoel_ir::Value>> {
    let mut vm = Vm::new();
    vm.execute(module, inputs)
}

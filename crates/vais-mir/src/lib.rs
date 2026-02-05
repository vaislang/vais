//! Middle Intermediate Representation (MIR) for the Vais compiler.
//!
//! MIR sits between the typed AST and LLVM IR, providing a platform-independent
//! representation suitable for optimization passes:
//!
//! ```text
//! AST (vais-ast) → Type Check (vais-types) → MIR (vais-mir) → LLVM IR (vais-codegen)
//! ```
//!
//! MIR uses a control-flow graph (CFG) of basic blocks with explicit
//! temporaries, drops, and control flow edges. This enables:
//! - Borrow checking and move analysis
//! - Dead code elimination
//! - Constant propagation
//! - Common subexpression elimination
//! - Inlining decisions
//! - Drop elaboration

mod builder;
pub mod emit_llvm;
pub mod lower;
pub mod optimize;
mod types;

pub use builder::MirBuilder;
pub use types::*;

#[cfg(test)]
mod tests;

//! JIT compiler for Vais using Cranelift.
//!
//! This crate provides JIT (Just-In-Time) compilation for the Vais REPL,
//! enabling immediate code execution without writing to disk or invoking
//! external compilers like clang.
//!
//! # Tiered JIT Compilation
//!
//! The JIT compiler supports three tiers:
//! - **Tier 0 (Interpreter)**: Direct AST evaluation for initial execution
//! - **Tier 1 (Baseline JIT)**: Fast compilation with minimal optimization
//! - **Tier 2 (Optimizing JIT)**: Slow compilation with full optimization
//!
//! Functions are automatically promoted to higher tiers based on execution counts.
//!
//! # Example
//!
//! ```ignore
//! use vais_jit::JitCompiler;
//! use vais_parser::parse;
//! use vais_types::TypeChecker;
//!
//! let mut jit = JitCompiler::new().unwrap();
//!
//! // Compile and run an expression
//! let ast = parse("F main()->i64 { 1 + 2 * 3 }").unwrap();
//! let mut checker = TypeChecker::new();
//! checker.check_module(&ast).unwrap();
//!
//! let result = jit.compile_and_run_main(&ast).unwrap();
//! assert_eq!(result, 7);
//! ```
//!
//! # Tiered JIT Example
//!
//! ```ignore
//! use vais_jit::{TieredJit, Tier, TierThresholds};
//! use vais_parser::parse;
//!
//! // Create tiered JIT with custom thresholds
//! let thresholds = TierThresholds {
//!     interpreter_to_baseline: 100,
//!     baseline_to_optimizing: 10_000,
//! };
//! let mut jit = TieredJit::with_thresholds(thresholds).unwrap();
//!
//! let ast = parse("F main()->i64 { 42 }").unwrap();
//! let result = jit.run_main(&ast).unwrap();
//! ```

mod compiler;
mod runtime;
mod tiered;
mod types;

pub use compiler::JitCompiler;
pub use runtime::JitRuntime;
pub use tiered::{
    FunctionProfile, FunctionStats, Interpreter, OsrPoint, Tier, TierThresholds, TieredJit, Value,
};
pub use types::TypeMapper;

/// JIT compilation error.
#[derive(Debug, thiserror::Error)]
pub enum JitError {
    #[error("Cranelift error: {0}")]
    Cranelift(String),

    #[error("Module error: {0}")]
    Module(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Type error: {0}")]
    Type(String),

    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    #[error("Runtime error: {0}")]
    Runtime(String),
}

impl From<cranelift_module::ModuleError> for JitError {
    fn from(e: cranelift_module::ModuleError) -> Self {
        JitError::Module(e.to_string())
    }
}

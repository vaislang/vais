//! JIT compiler for Vais using Cranelift.
//!
//! This crate provides JIT (Just-In-Time) compilation for the Vais REPL,
//! enabling immediate code execution without writing to disk or invoking
//! external compilers like clang.
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

mod compiler;
mod runtime;
mod types;

pub use compiler::JitCompiler;
pub use runtime::JitRuntime;
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

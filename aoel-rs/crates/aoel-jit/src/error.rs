//! JIT 컴파일러 에러 정의

use thiserror::Error;

/// JIT 컴파일 결과 타입
pub type JitResult<T> = Result<T, JitError>;

/// JIT 컴파일러 에러
#[derive(Error, Debug)]
pub enum JitError {
    #[error("Compilation error: {0}")]
    Compilation(String),

    #[error("Code generation error: {0}")]
    CodeGen(String),

    #[error("Unsupported opcode for JIT: {0}")]
    UnsupportedOpcode(String),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Module error: {0}")]
    Module(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Memory error: {0}")]
    Memory(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<cranelift_module::ModuleError> for JitError {
    fn from(err: cranelift_module::ModuleError) -> Self {
        JitError::Module(err.to_string())
    }
}

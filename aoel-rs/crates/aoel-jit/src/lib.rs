//! AOEL JIT Compiler
//!
//! Cranelift 기반 JIT 컴파일러로 AOEL IR을 네이티브 코드로 변환.
//! 핫 경로 감지 및 타입 특화를 통해 인터프리터 대비 5-20배 성능 향상.

pub mod compiler;
mod error;
mod profiler;
mod runtime;

pub use compiler::{JitCompiler, JittedFnInt};
pub use error::{JitError, JitResult};
pub use profiler::{ExecutionProfiler, FunctionProfile, JIT_THRESHOLD};
pub use runtime::{JitRuntime, CompiledCode};

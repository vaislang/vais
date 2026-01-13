//! Vais JIT Compiler
//!
//! Cranelift 기반 JIT 컴파일러로 Vais IR을 네이티브 코드로 변환.
//! 핫 경로 감지 및 타입 특화를 통해 인터프리터 대비 5-20배 성능 향상.

// JIT 컴파일에서 ip 인덱스 기반 점프가 필요하므로 needless_range_loop 허용
#![allow(clippy::needless_range_loop)]
// JIT 함수는 컴파일 상태와 컨텍스트가 복잡하여 많은 인수 허용
#![allow(clippy::too_many_arguments)]

pub mod compiler;
mod error;
mod profiler;
mod runtime;

pub use compiler::{JitCompiler, JittedFnInt};
pub use error::{JitError, JitResult};
pub use profiler::{ExecutionProfiler, FunctionProfile, JIT_THRESHOLD};
pub use runtime::{JitRuntime, CompiledCode};

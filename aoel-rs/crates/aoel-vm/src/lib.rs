//! AOEL AOEL Virtual Machine
//!
//! AOEL 문법용 스택 기반 VM.
//! 재귀 호출($), 컬렉션 연산(.@, .?, ./) 등을 지원.

mod vm;
mod error;
pub mod ffi;

pub use vm::{Vm, execute, execute_function};
pub use error::{RuntimeError, RuntimeResult};
pub use ffi::{FfiLoader, FfiType, FfiFunctionInfo};

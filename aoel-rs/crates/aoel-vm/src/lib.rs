//! AOEL v6b Virtual Machine
//!
//! v6b 문법용 스택 기반 VM.
//! 재귀 호출($), 컬렉션 연산(.@, .?, ./) 등을 지원.

mod vm;
mod error;

pub use vm::{Vm, execute, execute_function};
pub use error::{RuntimeError, RuntimeResult};

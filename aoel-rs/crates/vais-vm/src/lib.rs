//! Vais Virtual Machine
//!
//! Vais 문법용 스택 기반 VM.
//! 재귀 호출($), 컬렉션 연산(.@, .?, ./) 등을 지원.
//!
//! ## JIT 지원
//!
//! `jit` feature를 활성화하면 Cranelift 기반 JIT 컴파일을 사용할 수 있습니다:
//!
//! ```toml
//! vais-vm = { path = "...", features = ["jit"] }
//! ```

mod vm;
mod error;
pub mod ffi;

#[cfg(feature = "jit")]
mod jit_vm;

pub use vm::{Vm, execute, execute_function};
pub use error::{RuntimeError, RuntimeResult};
pub use ffi::{FfiLoader, FfiType, FfiFunctionInfo};

#[cfg(feature = "jit")]
pub use jit_vm::{JitVm, JitConfig, execute_with_jit, execute_function_with_jit};

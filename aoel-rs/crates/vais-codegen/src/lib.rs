//! Vais Native Code Generator
//!
//! Generates C, WebAssembly, LLVM IR, and Cranelift JIT code from Vais IR for native compilation

mod c_codegen;
mod wasm_codegen;
mod llvm_codegen;
mod cranelift_codegen;
mod error;

pub use c_codegen::{generate_c, CCodeGenerator};
pub use wasm_codegen::{generate_wat, WasmCodeGenerator};
pub use llvm_codegen::{generate_llvm_ir, LlvmCodeGenerator};
pub use cranelift_codegen::jit_execute;
#[cfg(feature = "cranelift")]
pub use cranelift_codegen::CraneliftCodeGenerator;
pub use error::{CodegenError, CodegenResult};

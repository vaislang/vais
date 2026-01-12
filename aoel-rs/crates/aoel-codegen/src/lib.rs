//! AOEL Native Code Generator
//!
//! Generates C and WebAssembly code from AOEL IR for native compilation

mod c_codegen;
mod wasm_codegen;
mod error;

pub use c_codegen::{generate_c, CCodeGenerator};
pub use wasm_codegen::{generate_wat, WasmCodeGenerator};
pub use error::{CodegenError, CodegenResult};

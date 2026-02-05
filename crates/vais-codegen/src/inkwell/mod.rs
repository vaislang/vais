//! Inkwell-based LLVM code generator for Vais.
//!
//! This module provides direct LLVM API access via inkwell bindings,
//! offering compile-time type safety and better performance compared
//! to text-based IR generation.
//!
//! # Features
//!
//! - Type-safe LLVM IR construction
//! - Direct memory-based JIT compilation support
//! - Native LLVM optimization passes
//!
//! # Requirements
//!
//! - LLVM 17+ must be installed
//! - Enable the `inkwell-codegen` feature

mod builtins;
mod generator;
mod types;

pub use generator::InkwellCodeGenerator;
pub use types::TypeMapper;

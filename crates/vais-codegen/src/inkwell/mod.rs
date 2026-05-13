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

// Inkwell 0.8 deprecates typed pointer constructors for LLVM opaque pointers.
// This backend still uses typed pointer casts consistently; migrate the pointer
// model in one dedicated pass instead of mixing pointer policies in this bump.
#![allow(deprecated)]

mod builtins;
mod generator;
mod types;

// Code generation modules (split from generator.rs)
mod gen_advanced;
mod gen_aggregate;
mod gen_declaration;
mod gen_expr;
mod gen_function;
mod gen_match;
mod gen_special;
mod gen_stmt;
mod gen_types;

pub use generator::InkwellCodeGenerator;

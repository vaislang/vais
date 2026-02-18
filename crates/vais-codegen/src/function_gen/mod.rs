//! Function and method code generation for Vais
//!
//! This module contains functions for generating LLVM IR for functions,
//! async functions, methods, and specialized generic functions.

pub(crate) mod async_gen;
pub(crate) mod codegen;
pub(crate) mod generics;
pub(crate) mod runtime;
pub(crate) mod signature;

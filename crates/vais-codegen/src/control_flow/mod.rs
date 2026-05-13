//! Control flow code generation for Vais
//!
//! This module contains functions for generating LLVM IR for control flow
//! constructs: if-else, match expressions, and pattern matching.

use super::*;

pub(crate) mod if_else;
pub(crate) mod match_gen;
pub(crate) mod pattern;

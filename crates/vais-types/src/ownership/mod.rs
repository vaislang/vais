//! Ownership and borrow checker for the Vais type system
//!
//! Implements Rust-style ownership semantics:
//! - Move semantics: values are moved by default for non-Copy types
//! - Borrow checking: at most one mutable reference OR any number of immutable references
//! - Scope-based invalidation: references cannot outlive their referents
//!
//! The checker runs as a second pass after type checking, operating on the typed AST.

mod types;
mod core;
mod var_tracking;
mod copy_check;
mod move_track;
mod borrow_track;
mod ast_check;
mod helpers;

pub use types::*;
pub use core::*;

#[cfg(test)]
mod tests;

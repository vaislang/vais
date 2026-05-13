//! Ownership and borrow checker for the Vais type system
//!
//! Implements Rust-style ownership semantics:
//! - Move semantics: values are moved by default for non-Copy types
//! - Borrow checking: at most one mutable reference OR any number of immutable references
//! - Scope-based invalidation: references cannot outlive their referents
//!
//! The checker runs as a second pass after type checking, operating on the typed AST.

mod ast_check;
mod borrow_track;
mod copy_check;
mod core;
mod helpers;
mod move_track;
mod types;
mod var_tracking;

pub use core::*;
pub use types::*;

#[cfg(test)]
mod tests;

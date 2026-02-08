//! Automatic property-based test generation for Vais programs.
//!
//! Analyzes function signatures and generates test cases with:
//! - Random input values for various types
//! - Property assertions (non-crash, idempotency, commutativity, etc.)
//! - Boundary value testing (0, -1, MAX, MIN)
//! - Shrinking of failing test cases

mod generator;
mod properties;
mod shrink;

pub use generator::{TestCase, TestCategory, TestGenerator, TestSuite, TestValue, TypeHint};
pub use properties::Property;
pub use shrink::Shrinker;

#[cfg(test)]
mod tests;

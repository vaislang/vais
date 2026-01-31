//! Salsa-style query-based compilation database for the Vais compiler.
//!
//! This crate provides a memoized, dependency-tracking compilation database
//! that caches intermediate results (tokens, AST, types, IR) and automatically
//! invalidates them when source files change.
//!
//! # Architecture
//!
//! The query system follows the Salsa pattern:
//! - **Inputs**: Source files (set by the user, trigger invalidation)
//! - **Derived queries**: Computed from inputs, automatically memoized
//! - **Dependency tracking**: Each query records which other queries it reads
//! - **Incremental recomputation**: Only re-runs queries whose inputs changed
//!
//! # Query Pipeline
//!
//! ```text
//! source_text(path)          [Input]
//!     ↓
//! tokenize(path)             [Derived]
//!     ↓
//! parse(path)                [Derived]
//!     ↓
//! type_check(path)           [Derived]
//!     ↓
//! generate_ir(path, target)  [Derived]
//! ```

mod database;
mod revision;

pub use database::{QueryDatabase, QueryError, QueryResult};
pub use revision::Revision;

#[cfg(test)]
mod tests;

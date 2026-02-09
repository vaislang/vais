//! Vais Language Server Protocol library
//!
//! This crate provides LSP implementation for the Vais programming language.
//! It can be used both as a binary (vais-lsp server) and as a library for testing.

pub mod ai_completion;
mod backend;
mod diagnostics;
mod semantic;

// Backend module extensions
mod analysis;
mod folding;
mod hints;
mod symbol_analysis;

// LSP request handlers
pub(crate) mod handlers;

// Re-export the backend for use in tests and as a library
pub use backend::VaisBackend;

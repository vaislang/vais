//! End-to-End Tests for the Vais Compiler (modularized)
//!
//! These tests verify the complete pipeline:
//! Source → Lexer → Parser → Type Checker → Codegen → LLVM IR → clang → Execute → Verify
//!
//! Each test compiles Vais source to LLVM IR, builds an executable with clang,
//! runs it, and checks the exit code (and optionally stdout output).

mod helpers;

mod basics;
mod builtins;
mod async_runtime;
mod concurrency;
mod adoption_types;
mod phase31;
mod closures_iter;
mod modules_system;
mod advanced;

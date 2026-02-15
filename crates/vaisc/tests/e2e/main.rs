//! End-to-End Tests for the Vais Compiler (modularized)
//!
//! These tests verify the complete pipeline:
//! Source → Lexer → Parser → Type Checker → Codegen → LLVM IR → clang → Execute → Verify
//!
//! Each test compiles Vais source to LLVM IR, builds an executable with clang,
//! runs it, and checks the exit code (and optionally stdout output).

mod helpers;

mod adoption_types;
mod advanced;
mod async_runtime;
mod basics;
mod builtins;
mod closures_iter;
mod concurrency;
mod modules_system;
mod phase31;
mod phase32;
mod phase37;
mod phase38;
mod phase40;

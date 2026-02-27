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
mod phase30;
mod phase31;
mod phase32;
mod phase32_async;
mod phase32_generics;
mod phase32_lang;
mod phase32_patterns;
mod phase37;
mod phase37_comptime_defer;
mod phase37_patterns;
mod phase37_pipe_string;
mod phase37_union_const;
mod phase38;
mod phase40;
mod phase41;
mod phase41_error_handling;
mod phase41_globals_ternary;
mod phase41_loop_control;
mod phase41_string_numeric;
mod phase42;
mod phase43;
mod phase44;
mod phase45;
mod phase45_advanced;
mod phase45_types;
mod phase47_closure_pipe;
mod phase47_struct_enum;
mod phase47_trait_impl;

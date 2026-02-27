//! Vais LLVM Code Generator
//!
//! Generates LLVM IR from typed AST for native code generation.
//!
//! # Backends
//!
//! This crate supports two code generation backends:
//!
//! - **text-codegen** (default): Generates LLVM IR as text, then compiles via clang.
//!   Does not require LLVM installation.
//!
//! - **inkwell-codegen**: Uses inkwell bindings for direct LLVM API access.
//!   Provides better type safety and performance. Requires LLVM 17+.
//!
//! # Feature Flags
//!
//! - `text-codegen` (default): Enable text-based IR generation
//! - `inkwell-codegen`: Enable inkwell-based generation (requires LLVM 17+)

pub mod abi;
#[cfg(test)]
mod abi_tests;
pub mod advanced_opt;
mod builtins;
#[cfg(test)]
mod cache_tests;
mod contracts;
mod control_flow;
pub mod cross_compile;
pub mod debug;
mod diagnostics;
mod emit;
mod error;
mod expr;
mod expr_helpers;
mod expr_helpers_call;
mod expr_helpers_control;
mod expr_helpers_data;
mod expr_helpers_misc;
mod expr_visitor;
mod ffi;
mod function_gen;
mod generate_expr;
mod generate_expr_call;
mod generate_expr_loop;
mod generate_expr_struct;
mod generics_helpers;
mod helpers;
mod init;
mod lambda_closure;
mod module_gen;
#[cfg(test)]
mod nested_field_tests;
pub mod optimize;
pub mod parallel;
mod registration;
mod state;
mod stmt;
mod stmt_visitor;
mod string_ops;
#[cfg(test)]
mod struct_param_tests;
mod target;
mod trait_dispatch;
mod type_inference;
mod types;
pub mod visitor;
pub mod vtable;
#[cfg(test)]
mod vtable_tests;
pub mod wasm_component;
mod wasm_helpers;

// Inkwell-based code generator (optional)
#[cfg(feature = "inkwell-codegen")]
pub mod inkwell;

#[cfg(feature = "inkwell-codegen")]
pub use inkwell::InkwellCodeGenerator;

pub use visitor::{ExprVisitor, ItemVisitor, StmtVisitor};

pub use debug::{DebugConfig, DebugInfoBuilder};

// Re-export error types
pub use error::{CodegenError, CodegenResult};

// Re-export state types
pub use state::DecreasesInfo;
pub(crate) use state::{
    AsyncPollContext, ContractState, FunctionContext, GenericState, LambdaState, StringPool,
    TypeRegistry,
};

use std::collections::HashMap;
use vais_ast::*;
use vais_types::ResolvedType;

/// Maximum recursion depth for type resolution to prevent stack overflow
/// This limit protects against infinite recursive types like: type A = B; type B = A;
const MAX_TYPE_RECURSION_DEPTH: usize = 128;

/// Escape a string for use in LLVM IR string constants.
///
/// Handles all control characters (0x00-0x1F, 0x7F) and special characters
/// that need escaping in LLVM IR constant strings.
pub(crate) fn escape_llvm_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'\\' => result.push_str("\\5C"),
            b'"' => result.push_str("\\22"),
            b'\n' => result.push_str("\\0A"),
            b'\r' => result.push_str("\\0D"),
            b'\t' => result.push_str("\\09"),
            b'\0' => result.push_str("\\00"),
            0x01..=0x08 | 0x0B..=0x0C | 0x0E..=0x1F | 0x7F => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                result.push('\\');
                result.push(HEX[(byte >> 4) as usize] as char);
                result.push(HEX[(byte & 0x0F) as usize] as char);
            }
            _ => result.push(byte as char),
        }
    }
    result
}

#[cfg(test)]
use diagnostics::edit_distance;
#[cfg(test)]
pub(crate) use diagnostics::suggest_type_conversion;
pub(crate) use diagnostics::{format_did_you_mean, suggest_similar};
pub use target::TargetTriple;
// Re-export type structs from types module
pub(crate) use types::*;

/// Result of generating a block of statements
/// (value, ir_code, is_terminated)
/// is_terminated is true if the block ends with break, continue, or return
type _BlockResult = (String, String, bool);

/// LLVM IR Code Generator for Vais 0.0.1
///
/// Generates LLVM IR text from typed AST for native code generation via clang.
pub struct CodeGenerator {
    // Type definitions registry
    pub(crate) types: TypeRegistry,

    // Generic type system state
    pub(crate) generics: GenericState,

    // Current function compilation context
    pub(crate) fn_ctx: FunctionContext,

    // String constant pool
    pub(crate) strings: StringPool,

    // Lambda/closure/async state
    pub(crate) lambdas: LambdaState,

    // Module name
    module_name: String,

    // Target architecture
    target: TargetTriple,

    // Flag to emit unwrap panic message and abort declaration
    needs_unwrap_panic: bool,

    // Flag to emit __sync_spawn__poll function for sync spawn→Future wrapping
    needs_sync_spawn_poll: bool,

    // Flag to emit string helper functions
    needs_string_helpers: bool,

    // Debug info builder for DWARF metadata generation
    debug_info: DebugInfoBuilder,

    // Cache for type_to_llvm conversions to avoid repeated computations
    // Uses interior mutability to allow caching through immutable references
    type_to_llvm_cache: std::cell::RefCell<HashMap<String, String>>,

    // GC mode configuration
    gc_enabled: bool,
    gc_threshold: usize,

    // VTable generator for trait objects (dyn Trait)
    // Uses trait definitions from `self.types.trait_defs` (TypeRegistry) for vtable layout
    vtable_generator: vtable::VtableGenerator,

    // Release mode flag (disables contract checks)
    release_mode: bool,

    // Contract verification state (old() snapshots, decreases, contract strings)
    contracts: ContractState,

    // Type recursion depth tracking (prevents infinite recursion)
    type_recursion_depth: std::cell::Cell<usize>,

    // WASM import metadata: function_name -> (module_name, import_name)
    pub(crate) wasm_imports: HashMap<String, (String, String)>,

    // WASM export metadata: function_name -> export_name
    pub(crate) wasm_exports: HashMap<String, String>,
}

#[cfg(test)]
mod tests;

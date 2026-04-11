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

/// Infallible `writeln!` to a `String` buffer used for IR emission.
///
/// Writing to a `String` via `std::fmt::Write` never fails (it only allocates),
/// so the `Result` can be safely discarded. This macro replaces the pervasive
/// `writeln!(ir, ...).unwrap()` pattern with an explicit, panic-free idiom.
macro_rules! write_ir {
    ($dst:expr, $($arg:tt)*) => {{
        use std::fmt::Write as _;
        let _ = writeln!($dst, $($arg)*);
    }};
}

/// Infallible `write!` (no trailing newline) to a `String` buffer.
///
/// Same rationale as [`write_ir!`]: `String` writes never fail.
macro_rules! write_ir_no_newline {
    ($dst:expr, $($arg:tt)*) => {{
        use std::fmt::Write as _;
        let _ = write!($dst, $($arg)*);
    }};
}

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
mod expr_helpers_assign;
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
pub mod ir_verify;
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
pub mod string_pool;
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
pub use error::{CodegenError, CodegenResult, CodegenWarning, SpannedCodegenError, WithSpan};

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
const MAX_TYPE_RECURSION_DEPTH: usize = 64;

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
            0x01..=0x08 | 0x0B..=0x0C | 0x0E..=0x1F | 0x7F..=0xFF => {
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

    // Flag to emit abort declaration for runtime bounds checking
    needs_bounds_check: bool,

    // Flag to emit __sync_spawn__poll function for sync spawn→Future wrapping
    needs_sync_spawn_poll: bool,

    // Flag to emit llvm.memcpy intrinsic declaration (used by typed memory ops on large structs)
    needs_llvm_memcpy: bool,

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

    // Sizeof visited set (prevents infinite recursion for circular struct references)
    sizeof_visited: std::cell::RefCell<std::collections::HashSet<String>>,

    // WASM import metadata: function_name -> (module_name, import_name)
    pub(crate) wasm_imports: HashMap<String, (String, String)>,

    // WASM export metadata: function_name -> export_name
    pub(crate) wasm_exports: HashMap<String, String>,

    // Last expression span for error reporting.
    // Updated at the top of generate_expr, used to decorate CodegenError
    // with source location when it propagates up to generate_module.
    pub(crate) last_error_span: Option<Span>,

    // Multi-error collection mode for graceful degradation.
    // When enabled, function body generation errors are collected instead of
    // immediately failing. A stub function is emitted for failed functions
    // so that the rest of the module can still be generated.
    pub multi_error_mode: bool,
    pub(crate) collected_errors: Vec<SpannedCodegenError>,

    // When true, ICE-level type fallbacks (Var, Unknown, Lifetime reaching
    // codegen) are promoted from warnings to hard errors.
    // Default: true. Can be disabled via `set_strict_type_mode(false)` or
    // `VAIS_STRICT_TYPE_MODE=0` env var.
    pub strict_type_mode: bool,

    // When true, un-monomorphized Generic/ConstGeneric reaching codegen is
    // promoted from warning to `InternalError` (Phase 191 — i64 fallback
    // removal). Default: false (preserves the historical i64 fallback path
    // for backward compatibility). Opt in via `set_strict_generic_mode(true)`
    // or `VAIS_STRICT_GENERIC=1` env var. Once enabled, any un-substituted
    // Generic(_)/ConstGeneric(_) reaching `type_to_llvm` will abort codegen
    // with an ICE-level error instead of silently erasing to `i64`.
    pub strict_generic_mode: bool,

    // String interning pool for identifier deduplication.
    // Reduces memory usage by storing each unique function/struct/variable name once.
    pub(crate) ident_pool: string_pool::IdentPool,

    // Structured warnings collected during code generation.
    // Unlike errors which halt compilation, warnings are accumulated and can be
    // queried after codegen completes (e.g., to report i64 fallback usage).
    // Uses RefCell for interior mutability so warnings can be emitted from &self methods
    // (same pattern as type_to_llvm_cache).
    pub(crate) warnings: std::cell::RefCell<Vec<CodegenWarning>>,

    // Global numeric constants for reference returns.
    // When a function with a reference return type (e.g., `-> &i64`) returns a literal value,
    // the literal must be stored in a global constant so the returned pointer is valid.
    // Each entry is (global_name, llvm_type, literal_value).
    pub(crate) ref_constants: Vec<(String, String, String)>,

    // Counter for unique global constant names
    pub(crate) ref_constant_counter: usize,

    // Expression types from type checker, keyed by (span.start, span.end).
    // Used by infer_expr_type to look up TC-resolved types before falling back
    // to the legacy inference heuristics.
    pub(crate) expr_types: HashMap<(usize, usize), ResolvedType>,
}

#[cfg(test)]
mod tests;

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
mod expr;
mod expr_helpers;
mod expr_helpers_call;
mod expr_helpers_control;
mod expr_helpers_data;
mod expr_helpers_misc;
mod expr_visitor;
mod ffi;
pub mod formatter;
mod function_gen;
mod generate_expr;
mod generate_expr_call;
mod generate_expr_loop;
mod generate_expr_struct;
mod lambda_closure;
#[cfg(test)]
mod nested_field_tests;
pub mod optimize;
pub mod parallel;
mod registration;
mod stmt;
mod stmt_visitor;
mod string_ops;
mod target;
#[cfg(test)]
mod struct_param_tests;
mod type_inference;
mod types;
pub mod visitor;
pub mod vtable;
#[cfg(test)]
mod vtable_tests;
pub mod wasm_component;

// Inkwell-based code generator (optional)
#[cfg(feature = "inkwell-codegen")]
pub mod inkwell;

#[cfg(feature = "inkwell-codegen")]
pub use inkwell::InkwellCodeGenerator;

pub use visitor::{ExprVisitor, ItemVisitor, StmtVisitor};

pub use debug::{DebugConfig, DebugInfoBuilder};

use std::collections::HashMap;
use thiserror::Error;
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

pub use target::TargetTriple;
pub(crate) use diagnostics::{format_did_you_mean, suggest_similar};
// Re-export type structs from types module
pub(crate) use types::*;

/// Error type for code generation failures.
///
/// Represents various kinds of errors that can occur during LLVM IR generation,
/// including undefined references, type mismatches, and unsupported features.
#[derive(Debug, Error)]
pub enum CodegenError {
    /// Reference to an undefined variable
    #[error("Undefined variable: {0}")]
    UndefinedVar(String),

    /// Call to an undefined function
    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    /// Type-related error during code generation
    #[error("Type error: {0}")]
    TypeError(String),

    /// LLVM-specific error
    #[error("LLVM error: {0}")]
    LlvmError(String),

    /// Feature not yet implemented in code generation
    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    /// Recursion depth limit exceeded (infinite type recursion)
    #[error("Recursion depth limit exceeded: {0}")]
    RecursionLimitExceeded(String),
}

impl CodegenError {
    /// Get the error code for this codegen error
    pub fn error_code(&self) -> &str {
        match self {
            CodegenError::UndefinedVar(_) => "C001",
            CodegenError::UndefinedFunction(_) => "C002",
            CodegenError::TypeError(_) => "C003",
            CodegenError::LlvmError(_) => "C004",
            CodegenError::Unsupported(_) => "C005",
            CodegenError::RecursionLimitExceeded(_) => "C006",
        }
    }

    /// Get a help message for this error
    pub fn help(&self) -> Option<String> {
        match self {
            CodegenError::UndefinedVar(msg) => {
                if msg.contains("Did you mean") {
                    None // suggestion already embedded
                } else {
                    Some("check that the variable is defined before use".to_string())
                }
            }
            CodegenError::UndefinedFunction(msg) => {
                if msg.contains("Did you mean") {
                    None
                } else {
                    Some("check that the function is defined before calling it".to_string())
                }
            }
            CodegenError::TypeError(_) => {
                Some("ensure all operands have compatible types".to_string())
            }
            CodegenError::Unsupported(feature) => Some(format!(
                "'{}' is not yet implemented in code generation",
                feature
            )),
            CodegenError::RecursionLimitExceeded(_) => {
                Some("consider reducing nesting depth or refactoring recursive types".to_string())
            }
            CodegenError::LlvmError(_) => None,
        }
    }
}

type CodegenResult<T> = Result<T, CodegenError>;

// ============================================================================

/// Result of generating a block of statements
/// (value, ir_code, is_terminated)
/// is_terminated is true if the block ends with break, continue, or return
type _BlockResult = (String, String, bool);

/// Type definitions registry — functions, structs, enums, unions, constants, globals, traits
pub(crate) struct TypeRegistry {
    /// All function names declared in the module (including generics, before instantiation)
    pub(crate) declared_functions: std::collections::HashSet<String>,
    /// Function signatures for lookup
    pub(crate) functions: HashMap<String, FunctionInfo>,
    /// Struct definitions
    pub(crate) structs: HashMap<String, StructInfo>,
    /// Enum definitions
    pub(crate) enums: HashMap<String, EnumInfo>,
    /// Union definitions (untagged, C-style)
    pub(crate) unions: HashMap<String, UnionInfo>,
    /// Constant definitions
    pub(crate) constants: HashMap<String, types::ConstInfo>,
    /// Global variable definitions
    pub(crate) globals: HashMap<String, types::GlobalInfo>,
    /// Trait definitions for vtable generation
    pub(crate) trait_defs: HashMap<String, vais_types::TraitDef>,
    /// Trait implementations: (impl_type, trait_name) -> method_impls
    pub(crate) trait_impl_methods: HashMap<(String, String), HashMap<String, String>>,
    /// Resolved function signatures from type checker (for inferred parameter types)
    pub(crate) resolved_function_sigs: HashMap<String, vais_types::FunctionSig>,
}

/// Generic type system state — templates, instantiations, substitutions
pub(crate) struct GenericState {
    /// Generic struct AST definitions (before monomorphization)
    pub(crate) struct_defs: HashMap<String, std::rc::Rc<vais_ast::Struct>>,
    /// Generic struct name aliases (base_name -> mangled_name, e.g., "Box" -> "Box$i64")
    pub(crate) struct_aliases: HashMap<String, String>,
    /// Generated struct instantiations (mangled_name -> already_generated)
    pub(crate) generated_structs: HashMap<String, bool>,
    /// Generic function templates stored for specialization (base_name -> Function)
    pub(crate) function_templates: HashMap<String, std::rc::Rc<Function>>,
    /// Generic function instantiation map: base_name -> Vec<(type_args, mangled_name)>
    pub(crate) fn_instantiations: HashMap<String, Vec<(Vec<ResolvedType>, String)>>,
    /// Generated function instantiations (mangled_name -> already_generated)
    pub(crate) generated_functions: HashMap<String, bool>,
    /// Generic substitutions for current function/method
    pub(crate) substitutions: HashMap<String, ResolvedType>,
}

/// Current function compilation context — locals, labels, control flow
pub(crate) struct FunctionContext {
    /// Current function being compiled
    pub(crate) current_function: Option<String>,
    /// Current function's return type (for generating ret instructions in nested contexts)
    pub(crate) current_return_type: Option<ResolvedType>,
    /// Local variables in current function
    pub(crate) locals: HashMap<String, LocalVar>,
    /// Label counter for unique basic block names
    pub(crate) label_counter: usize,
    /// Stack of loop labels for break/continue
    pub(crate) loop_stack: Vec<LoopLabels>,
    /// Stack of deferred expressions per function (LIFO order)
    pub(crate) defer_stack: Vec<vais_ast::Spanned<vais_ast::Expr>>,
    /// Current basic block name (for phi node predecessor tracking)
    pub(crate) current_block: String,
    /// Current source file being compiled (for contract error messages)
    pub(crate) current_file: Option<String>,
}

/// Lambda, closure, and async function state
pub(crate) struct LambdaState {
    /// Generated LLVM IR for lambda functions, emitted after the main body
    pub(crate) generated_ir: Vec<String>,
    /// Closure information for each lambda variable (maps var_name -> closure_info)
    pub(crate) closures: HashMap<String, ClosureInfo>,
    /// Last generated lambda info (for Let statement to pick up)
    pub(crate) last_lambda_info: Option<ClosureInfo>,
    /// Async function state machine counter
    pub(crate) async_state_counter: usize,
    /// Async await points
    pub(crate) async_await_points: Vec<AsyncAwaitPoint>,
    /// Current async function info
    pub(crate) current_async_function: Option<AsyncFunctionInfo>,
}

/// String constant pool — string literals, counters, module prefix
pub(crate) struct StringPool {
    /// String constants for global storage (name, value)
    pub(crate) constants: Vec<(String, String)>,
    /// Counter for string constant names
    pub(crate) counter: usize,
    /// Module-specific prefix for string constants (avoids collisions in multi-module builds)
    pub(crate) prefix: Option<String>,
}

/// Contract verification state — pre/post conditions, old() snapshots, decreases
pub(crate) struct ContractState {
    /// Contract string constants (separate from regular strings)
    pub(crate) contract_constants: HashMap<String, String>,
    /// Counter for contract string constant names
    pub(crate) contract_counter: usize,
    /// Pre-state snapshots for old() expressions in ensures clauses
    /// Maps snapshot variable name -> allocated storage name
    pub(crate) old_snapshots: HashMap<String, String>,
    /// Decreases expressions for current function (for termination proof)
    pub(crate) current_decreases_info: Option<DecreasesInfo>,
}

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

/// Information about a function's decreases clause for termination proof
#[derive(Clone)]
pub struct DecreasesInfo {
    /// Storage variable name for the initial decreases value
    pub storage_name: String,
    /// The decreases expression from the attribute (already boxed)
    pub expr: Box<vais_ast::Spanned<vais_ast::Expr>>,
    /// Function name with decreases clause
    pub function_name: String,
}

impl CodeGenerator {
    /// Creates a new code generator for the given module with native target.
    ///
    /// Initializes the code generator with built-in functions registered.
    ///
    /// # Arguments
    ///
    /// * `module_name` - Name of the module being compiled
    pub fn new(module_name: &str) -> Self {
        Self::new_with_target(module_name, TargetTriple::Native)
    }

    /// Creates a new code generator for the given module with specified target.
    ///
    /// Initializes the code generator with built-in functions registered.
    ///
    /// # Arguments
    ///
    /// * `module_name` - Name of the module being compiled
    /// * `target` - Target architecture for code generation
    pub fn new_with_target(module_name: &str, target: TargetTriple) -> Self {
        let mut gen = Self {
            types: TypeRegistry {
                declared_functions: std::collections::HashSet::new(),
                functions: HashMap::new(),
                structs: HashMap::new(),
                enums: HashMap::new(),
                unions: HashMap::new(),
                constants: HashMap::new(),
                globals: HashMap::new(),
                trait_defs: HashMap::new(),
                trait_impl_methods: HashMap::new(),
                resolved_function_sigs: HashMap::new(),
            },
            generics: GenericState {
                struct_defs: HashMap::new(),
                struct_aliases: HashMap::new(),
                generated_structs: HashMap::new(),
                function_templates: HashMap::new(),
                fn_instantiations: HashMap::new(),
                generated_functions: HashMap::new(),
                substitutions: HashMap::new(),
            },
            fn_ctx: FunctionContext {
                current_function: None,
                current_return_type: None,
                locals: HashMap::new(),
                label_counter: 0,
                loop_stack: Vec::new(),
                defer_stack: Vec::new(),
                current_block: "entry".to_string(),
                current_file: None,
            },
            strings: StringPool {
                constants: Vec::new(),
                counter: 0,
                prefix: None,
            },
            lambdas: LambdaState {
                generated_ir: Vec::new(),
                closures: HashMap::new(),
                last_lambda_info: None,
                async_state_counter: 0,
                async_await_points: Vec::new(),
                current_async_function: None,
            },
            module_name: module_name.to_string(),
            target,
            needs_unwrap_panic: false,
            needs_string_helpers: false,
            debug_info: DebugInfoBuilder::new(DebugConfig::default()),
            type_to_llvm_cache: std::cell::RefCell::new(HashMap::new()),
            gc_enabled: false,
            gc_threshold: 1048576, // 1 MB default
            vtable_generator: vtable::VtableGenerator::new(),
            release_mode: false,
            contracts: ContractState {
                contract_constants: HashMap::new(),
                contract_counter: 0,
                old_snapshots: HashMap::new(),
                current_decreases_info: None,
            },
            type_recursion_depth: std::cell::Cell::new(0),
            wasm_imports: HashMap::new(),
            wasm_exports: HashMap::new(),
        };

        // Register built-in extern functions
        gen.register_builtin_functions();
        gen
    }

    /// Get the target triple for this code generator
    pub fn target(&self) -> &TargetTriple {
        &self.target
    }

    /// Enable debug info generation with source file information
    ///
    /// This should be called before `generate_module` to enable DWARF debug
    /// metadata generation. The source code is used for line/column mapping.
    ///
    /// # Arguments
    /// * `source_file` - Name of the source file
    /// * `source_dir` - Directory containing the source file
    /// * `source_code` - The source code content for line number calculation
    pub fn enable_debug(&mut self, source_file: &str, source_dir: &str, source_code: &str) {
        let config = DebugConfig::new(source_file, source_dir);
        self.debug_info = DebugInfoBuilder::new(config);
        self.debug_info.set_source_code(source_code);
    }

    /// Check if debug info generation is enabled
    pub fn is_debug_enabled(&self) -> bool {
        self.debug_info.is_enabled()
    }

    /// Enable GC mode for automatic memory management
    pub fn enable_gc(&mut self) {
        self.gc_enabled = true;
    }

    /// Set GC threshold (bytes allocated before triggering collection)
    pub fn set_gc_threshold(&mut self, threshold: usize) {
        self.gc_threshold = threshold;
    }

    /// Check if GC mode is enabled
    pub fn is_gc_enabled(&self) -> bool {
        self.gc_enabled
    }

    /// Enable release mode (disables contract checks)
    pub fn enable_release_mode(&mut self) {
        self.release_mode = true;
    }

    /// Check if release mode is enabled
    pub fn is_release_mode(&self) -> bool {
        self.release_mode
    }

    /// Set resolved function signatures from the type checker.
    /// Used to provide inferred parameter types for functions with Type::Infer parameters.
    pub fn set_resolved_functions(&mut self, resolved: HashMap<String, vais_types::FunctionSig>) {
        self.types.resolved_function_sigs = resolved;
    }

    /// Set string prefix for per-module codegen (avoids .str.N collisions across modules)
    pub fn set_string_prefix(&mut self, prefix: &str) {
        self.strings.prefix = Some(prefix.to_string());
    }

    /// Generate WASM import/export attribute sections
    pub(crate) fn generate_wasm_metadata(&self) -> String {
        let mut ir = String::new();

        if self.wasm_imports.is_empty() && self.wasm_exports.is_empty() {
            return ir;
        }

        // Generate WASM import attributes using custom section metadata
        // These are recognized by LLVM's WASM backend
        let mut attr_idx = 1;
        for (module_name, import_name) in self.wasm_imports.values() {
            ir.push_str(&format!(
                "attributes #{} = {{ \"wasm-import-module\"=\"{}\" \"wasm-import-name\"=\"{}\" }}\n",
                attr_idx, module_name, import_name
            ));
            attr_idx += 1;
        }

        // Generate WASM export annotations
        for export_name in self.wasm_exports.values() {
            ir.push_str(&format!(
                "attributes #{} = {{ \"wasm-export-name\"=\"{}\" }}\n",
                attr_idx, export_name
            ));
            attr_idx += 1;
        }

        ir
    }

    /// Generate a unique string constant name, with optional module prefix
    fn make_string_name(&self) -> String {
        if let Some(ref prefix) = self.strings.prefix {
            format!("{}.str.{}", prefix, self.strings.counter)
        } else {
            format!(".str.{}", self.strings.counter)
        }
    }

    /// Emit LLVM IR module header (ModuleID, source_filename, target triple/datalayout).
    fn emit_module_header(&mut self, ir: &mut String) {
        ir.push_str(&format!("; ModuleID = '{}'\n", self.module_name));
        ir.push_str("source_filename = \"<vais>\"\n");
        if !matches!(self.target, TargetTriple::Native) {
            ir.push_str(&format!(
                "target datalayout = \"{}\"\n",
                self.target.data_layout()
            ));
            ir.push_str(&format!(
                "target triple = \"{}\"\n",
                self.target.triple_str()
            ));
        }
        ir.push('\n');
        if self.debug_info.is_enabled() {
            self.debug_info.initialize();
        }
    }

    /// Emit ABI version, string constants, and unwrap panic declaration.
    fn emit_string_constants(&self, ir: &mut String, is_main_module: bool) {
        if is_main_module {
            let abi_version = crate::abi::ABI_VERSION;
            let abi_version_len = abi_version.len() + 1;
            ir.push_str(&format!(
                "@__vais_abi_version = constant [{} x i8] c\"{}\\00\"\n\n",
                abi_version_len, abi_version
            ));
        }
        for (name, value) in &self.strings.constants {
            let escaped = escape_llvm_string(value);
            let len = value.len() + 1;
            ir.push_str(&format!(
                "@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n",
                name, len, escaped
            ));
        }
        if !self.strings.constants.is_empty() {
            ir.push('\n');
        }
        if self.needs_unwrap_panic {
            ir.push_str("@.unwrap_panic_msg = private unnamed_addr constant [22 x i8] c\"unwrap failed: panic!\\00\"\n");
            ir.push_str("declare void @abort()\n\n");
        }
    }

    /// Emit body IR, lambda functions, and vtable globals.
    fn emit_body_lambdas_vtables(&self, ir: &mut String, body_ir: &str) {
        ir.push_str(body_ir);
        for lambda_ir in &self.lambdas.generated_ir {
            ir.push('\n');
            ir.push_str(lambda_ir);
        }
        let vtable_ir = self.generate_vtable_globals();
        if !vtable_ir.is_empty() {
            ir.push_str("\n; VTable globals for trait objects\n");
            ir.push_str(&vtable_ir);
        }
        let drop_ir = self.vtable_generator.generate_drop_functions_ir();
        if !drop_ir.is_empty() {
            ir.push_str("\n; Drop functions for trait objects\n");
            ir.push_str(&drop_ir);
        }
    }

    /// Generate LLVM IR for a subset of module items (per-module codegen).
    ///
    /// This method generates IR for only the items at the specified indices,
    /// while declaring cross-module functions as extern. Type definitions
    /// (structs, enums, unions) are always included since they're needed everywhere.
    ///
    /// # Arguments
    /// * `full_module` - The complete merged AST (all modules)
    /// * `item_indices` - Indices into `full_module.items` for this module's items
    /// * `is_main_module` - Whether this module contains main() (emits ABI version, etc.)
    pub fn generate_module_subset(
        &mut self,
        full_module: &Module,
        item_indices: &[usize],
        is_main_module: bool,
    ) -> CodegenResult<String> {
        // Validate item_indices are within bounds
        let items_len = full_module.items.len();
        let mut out_of_bounds = Vec::new();
        for &idx in item_indices {
            if idx >= items_len {
                out_of_bounds.push(idx);
            }
        }
        if !out_of_bounds.is_empty() {
            eprintln!(
                "Warning: {} item indices out of bounds (>= {}): {:?}",
                out_of_bounds.len(),
                items_len,
                out_of_bounds
            );
        }

        // Filter to valid indices only
        let valid_indices: Vec<usize> = item_indices
            .iter()
            .copied()
            .filter(|&i| i < items_len)
            .collect();

        let mut ir = String::new();
        let index_set: std::collections::HashSet<usize> = valid_indices.iter().copied().collect();

        self.emit_module_header(&mut ir);

        // Snapshot builtin function keys (registered in constructor, before AST items)
        // These should NOT appear as cross-module extern declarations.
        let builtin_fn_keys: std::collections::HashSet<String> =
            self.types.functions.keys().cloned().collect();

        // First pass: register ALL type definitions (structs, enums, unions) from full module
        // and register functions — tracking which are "ours" vs external
        let mut module_functions: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for (idx, item) in full_module.items.iter().enumerate() {
            let is_ours = index_set.contains(&idx);
            match &item.node {
                Item::Function(f) => {
                    self.register_function(f)?;
                    if is_ours {
                        module_functions.insert(f.name.node.clone());
                    }
                }
                Item::Struct(s) => {
                    self.register_struct(s)?;
                    for method in &s.methods {
                        self.register_method(&s.name.node, &method.node)?;
                        if is_ours {
                            module_functions
                                .insert(format!("{}_{}", s.name.node, method.node.name.node));
                        }
                    }
                }
                Item::Enum(e) => self.register_enum(e)?,
                Item::Union(u) => self.register_union(u)?,
                Item::Impl(impl_block) => {
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        self.register_method(&type_name, &method.node)?;
                        if is_ours {
                            module_functions
                                .insert(format!("{}_{}", type_name, method.node.name.node));
                        }
                    }
                    if let Some(ref trait_name) = impl_block.trait_name {
                        let mut method_impls = HashMap::new();
                        for method in &impl_block.methods {
                            let fn_name = format!("{}_{}", type_name, method.node.name.node);
                            method_impls.insert(method.node.name.node.clone(), fn_name);
                        }
                        self.register_trait_impl(&type_name, &trait_name.node, method_impls);
                    }
                }
                Item::Trait(trait_def) => {
                    self.register_trait_from_ast(trait_def);
                }
                Item::ExternBlock(extern_block) => {
                    for func in &extern_block.functions {
                        self.register_extern_function(func, &extern_block.abi)?;
                    }
                }
                Item::Const(const_def) => {
                    self.register_const(const_def)?;
                }
                Item::Global(global_def) => {
                    self.register_global(global_def)?;
                }
                Item::Use(_) | Item::TypeAlias(_) | Item::Macro(_) | Item::Error { .. } => {}
            }
        }

        // Generate struct types (all modules need these)
        for (name, info) in &self.types.structs {
            ir.push_str(&self.generate_struct_type(name, info));
            ir.push('\n');
        }
        for (name, info) in &self.types.enums {
            ir.push_str(&self.generate_enum_type(name, info));
            ir.push('\n');
        }
        for (name, info) in &self.types.unions {
            ir.push_str(&self.generate_union_type(name, info));
            ir.push('\n');
        }

        // Generate extern declarations for ALL extern functions (is_extern = true)
        // Builtin helpers (is_extern = false) are handled separately below.
        let mut declared_fns = std::collections::HashSet::new();
        let mut sorted_fns: Vec<_> = self
            .types.functions
            .iter()
            .filter(|(_, info)| info.is_extern)
            .collect();
        sorted_fns.sort_by_key(|(key, info)| if **key == info.signature.name { 0 } else { 1 });
        for (key, info) in &sorted_fns {
            if !declared_fns.contains(&info.signature.name)
                && !module_functions.contains(&info.signature.name)
                && !module_functions.contains(*key)
            {
                if !is_main_module && info.signature.name == "fopen_ptr" {
                    // Non-main modules should declare fopen_ptr (not define it).
                    // The wrapper definition lives in the main module only.
                    let params: Vec<_> = info
                        .signature
                        .params
                        .iter()
                        .map(|(_, ty, _)| self.type_to_llvm(ty))
                        .collect();
                    let ret = self.type_to_llvm(&info.signature.ret);
                    ir.push_str(&format!(
                        "declare {} @fopen_ptr({})\n",
                        ret,
                        params.join(", ")
                    ));
                } else {
                    ir.push_str(&self.generate_extern_decl(info));
                    ir.push('\n');
                }
                declared_fns.insert(info.signature.name.clone());
            }
        }

        // Generate extern declarations for cross-module Vais functions
        // (functions registered from AST but not in this module's item set)
        // Skip builtins — they are handled by generate_helper_functions() or the non-main extern block.
        for (name, info) in &self.types.functions {
            if !info.is_extern
                && !module_functions.contains(name)
                && !declared_fns.contains(name)
                && !builtin_fn_keys.contains(name)
            {
                ir.push_str(&self.generate_extern_decl(info));
                ir.push('\n');
                declared_fns.insert(name.clone());
            }
        }

        // Generate function bodies only for this module's items
        let mut body_ir = String::new();
        for &idx in &valid_indices {
            let item = &full_module.items[idx];
            match &item.node {
                Item::Function(f) => {
                    body_ir.push_str(&self.generate_function_with_span(f, item.span)?);
                    body_ir.push('\n');
                }
                Item::Struct(s) => {
                    for method in &s.methods {
                        body_ir.push_str(&self.generate_method_with_span(
                            &s.name.node,
                            &method.node,
                            method.span,
                        )?);
                        body_ir.push('\n');
                    }
                }
                Item::Impl(impl_block) => {
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        body_ir.push_str(&self.generate_method_with_span(
                            &type_name,
                            &method.node,
                            method.span,
                        )?);
                        body_ir.push('\n');
                    }
                }
                _ => {} // Other items handled in registration pass
            }
        }

        self.emit_string_constants(&mut ir, is_main_module);
        self.emit_body_lambdas_vtables(&mut ir, &body_ir);

        // Add WASM runtime for main module
        if is_main_module && self.target.is_wasm() {
            ir.push_str(&self.generate_wasm_runtime());
        }

        if is_main_module {
            // Main module defines all helper functions
            if !matches!(self.target, TargetTriple::Wasm32Unknown) {
                ir.push_str(&self.generate_helper_functions());
            }
        } else {
            // Non-main modules declare builtin helpers as extern
            // (these are defined by generate_helper_functions() in the main module)
            ir.push_str("\n; Extern declarations for runtime helpers\n");
            let mut sorted_helpers: Vec<_> = builtin_fn_keys.iter().collect();
            sorted_helpers.sort();
            for key in sorted_helpers {
                if let Some(info) = self.types.functions.get(key) {
                    if !info.is_extern
                        && !declared_fns.contains(&info.signature.name)
                        && !module_functions.contains(key)
                        && !module_functions.contains(&info.signature.name)
                    {
                        ir.push_str(&self.generate_extern_decl(info));
                        ir.push('\n');
                        declared_fns.insert(info.signature.name.clone());
                    }
                }
            }
        }

        if self.needs_string_helpers {
            if is_main_module {
                ir.push_str(&self.generate_string_helper_functions());
            }
            if !self.target.is_wasm() {
                ir.push_str(&self.generate_string_extern_declarations());
            }
        }

        if !self.contracts.contract_constants.is_empty() {
            ir.push_str(&self.generate_contract_declarations());
            ir.push_str(&self.generate_contract_string_constants());
        }

        if self.debug_info.is_enabled() && !self.target.is_wasm() {
            ir.push_str("\n; Debug intrinsics\n");
            ir.push_str("declare void @llvm.dbg.declare(metadata, metadata, metadata)\n");
            ir.push_str("declare void @llvm.dbg.value(metadata, metadata, metadata)\n");
        }

        if !self.target.is_wasm() {
            ir.push_str(&self.debug_info.finalize());
        }

        // Add WASM import/export metadata attributes
        if self.target.is_wasm() && (!self.wasm_imports.is_empty() || !self.wasm_exports.is_empty())
        {
            ir.push_str("\n; WASM import/export metadata\n");
            ir.push_str(&self.generate_wasm_metadata());
        }

        Ok(ir)
    }

    /// Set current source file for error messages
    pub fn set_source_file(&mut self, file: &str) {
        self.fn_ctx.current_file = Some(file.to_string());
    }

    /// Check if a function call is recursive (calls the current function with decreases clause)
    fn is_recursive_call(&self, fn_name: &str) -> bool {
        // Check if we have a decreases clause for this function
        if let Some(ref decreases_info) = self.contracts.current_decreases_info {
            // A recursive call is when the called function matches the function with decreases
            decreases_info.function_name == fn_name
        } else {
            false
        }
    }

    /// Check if a function has the #[gc] attribute
    fn _has_gc_attribute(attributes: &[Attribute]) -> bool {
        attributes.iter().any(|attr| attr.name == "gc")
    }

    /// Get current generic substitution for a type parameter
    pub(crate) fn get_generic_substitution(&self, param: &str) -> Option<ResolvedType> {
        self.generics.substitutions.get(param).cloned()
    }

    /// Set generic substitutions for the current context
    pub(crate) fn _set_generic_substitutions(&mut self, subst: HashMap<String, ResolvedType>) {
        self.generics.substitutions = subst;
    }

    /// Clear generic substitutions
    pub(crate) fn _clear_generic_substitutions(&mut self) {
        self.generics.substitutions.clear();
    }

    /// Resolve a struct name, checking aliases for generic specializations.
    /// Returns the mangled name if the base name has a registered alias (e.g., "Box" -> "Box$i64").
    pub(crate) fn resolve_struct_name(&self, name: &str) -> String {
        if self.types.structs.contains_key(name) {
            return name.to_string();
        }
        if let Some(mangled) = self.generics.struct_aliases.get(name) {
            return mangled.clone();
        }
        name.to_string()
    }

    /// Generate mangled name for a generic struct
    pub(crate) fn mangle_struct_name(&self, name: &str, generics: &[ResolvedType]) -> String {
        vais_types::mangle_name(name, generics)
    }

    /// Generate mangled name for a generic function
    pub(crate) fn _mangle_function_name(&self, name: &str, generics: &[ResolvedType]) -> String {
        vais_types::mangle_name(name, generics)
    }

    /// Enter a type recursion level and check depth limit
    /// Returns an error if recursion limit is exceeded
    fn enter_type_recursion(&self, context: &str) -> CodegenResult<()> {
        let depth = self.type_recursion_depth.get();
        if depth >= MAX_TYPE_RECURSION_DEPTH {
            return Err(CodegenError::RecursionLimitExceeded(format!(
                "Type recursion depth limit ({}) exceeded in {}",
                MAX_TYPE_RECURSION_DEPTH, context
            )));
        }
        self.type_recursion_depth.set(depth + 1);
        Ok(())
    }

    /// Exit a type recursion level
    fn exit_type_recursion(&self) {
        let depth = self.type_recursion_depth.get();
        self.type_recursion_depth.set(depth.saturating_sub(1));
    }

    /// Get the size of a type in bytes (for generic operations)
    pub(crate) fn _type_size(&self, ty: &ResolvedType) -> usize {
        // Track recursion depth
        if self.enter_type_recursion("type_size").is_err() {
            // On recursion limit, return default size
            #[cfg(debug_assertions)]
            eprintln!("Warning: Type recursion limit exceeded in type_size");
            return 8;
        }

        let size = match ty {
            ResolvedType::I8 | ResolvedType::U8 | ResolvedType::Bool => 1,
            ResolvedType::I16 | ResolvedType::U16 => 2,
            ResolvedType::I32 | ResolvedType::U32 | ResolvedType::F32 => 4,
            ResolvedType::I64 | ResolvedType::U64 | ResolvedType::F64 => 8,
            ResolvedType::I128 | ResolvedType::U128 => 16,
            ResolvedType::Str => 8, // Pointer size
            ResolvedType::Pointer(_) | ResolvedType::Ref(_) | ResolvedType::RefMut(_) => 8,
            ResolvedType::Named { name, .. } => {
                // Calculate struct size
                if let Some(info) = self.types.structs.get(name) {
                    info.fields.iter().map(|(_, t)| self._type_size(t)).sum()
                } else {
                    8 // Default to pointer size
                }
            }
            ResolvedType::Generic(param) => {
                // Try to get concrete type from substitutions
                if let Some(concrete) = self.generics.substitutions.get(param) {
                    self._type_size(concrete)
                } else {
                    8 // Default to i64 size
                }
            }
            ResolvedType::DynTrait { .. } => 16, // Fat pointer: data + vtable
            _ => 8,                              // Default
        };

        // Always exit recursion
        self.exit_type_recursion();
        size
    }

    /// Register a trait definition for vtable generation
    pub fn register_trait(&mut self, trait_def: vais_types::TraitDef) {
        self.types.trait_defs.insert(trait_def.name.clone(), trait_def);
    }

    /// Register a trait from AST definition (converts AST Trait to TraitDef)
    fn register_trait_from_ast(&mut self, t: &vais_ast::Trait) {
        let mut methods = HashMap::new();
        for m in &t.methods {
            let params: Vec<(String, ResolvedType, bool)> = m
                .params
                .iter()
                .map(|p| {
                    let ty = if p.name.node == "self" {
                        // self parameter is a pointer to the implementing type
                        ResolvedType::I64 // placeholder, resolved at call site
                    } else {
                        self.ast_type_to_resolved(&p.ty.node)
                    };
                    (p.name.node.clone(), ty, p.is_mut)
                })
                .collect();

            let ret = m
                .ret_type
                .as_ref()
                .map(|t| self.ast_type_to_resolved(&t.node))
                .unwrap_or(ResolvedType::Unit);

            methods.insert(
                m.name.node.clone(),
                vais_types::TraitMethodSig {
                    name: m.name.node.clone(),
                    params,
                    ret,
                    has_default: m.default_body.is_some(),
                    is_async: m.is_async,
                    is_const: m.is_const,
                },
            );
        }

        let trait_def = vais_types::TraitDef {
            name: t.name.node.clone(),
            generics: t.generics.iter().map(|g| g.name.node.clone()).collect(),
            super_traits: t.super_traits.iter().map(|s| s.node.clone()).collect(),
            associated_types: HashMap::new(), // Simplified for now
            methods,
        };
        self.register_trait(trait_def);
    }

    /// Register a trait implementation for vtable generation
    pub fn register_trait_impl(
        &mut self,
        impl_type: &str,
        trait_name: &str,
        method_impls: HashMap<String, String>,
    ) {
        self.types.trait_impl_methods.insert(
            (impl_type.to_string(), trait_name.to_string()),
            method_impls,
        );
    }

    /// Get or generate a vtable for a specific type implementing a trait
    pub fn get_or_generate_vtable(
        &mut self,
        impl_type: &str,
        trait_name: &str,
    ) -> Option<vtable::VtableInfo> {
        let trait_def = self.types.trait_defs.get(trait_name)?.clone();
        let method_impls = self
            .types.trait_impl_methods
            .get(&(impl_type.to_string(), trait_name.to_string()))
            .cloned()
            .unwrap_or_default();

        Some(
            self.vtable_generator
                .generate_vtable(impl_type, &trait_def, &method_impls),
        )
    }

    /// Generate all vtable globals for the module
    pub fn generate_vtable_globals(&self) -> String {
        let mut ir = String::new();

        for vtable_info in self.vtable_generator.get_vtables() {
            if let Some(trait_def) = self.types.trait_defs.get(&vtable_info.trait_name) {
                let type_size = 8; // Default size, could be refined
                let type_align = 8; // Default alignment

                ir.push_str(&self.vtable_generator.generate_vtable_global(
                    vtable_info,
                    trait_def,
                    type_size,
                    type_align,
                ));
                ir.push('\n');
            }
        }

        ir
    }

    /// Generate code to create a trait object from a concrete value
    pub fn generate_trait_object_creation(
        &mut self,
        concrete_value: &str,
        concrete_type: &str,
        impl_type: &str,
        trait_name: &str,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let vtable_info = self
            .get_or_generate_vtable(impl_type, trait_name)
            .ok_or_else(|| {
                CodegenError::Unsupported(format!(
                    "No vtable for {} implementing {}",
                    impl_type, trait_name
                ))
            })?;

        Ok(self.vtable_generator.create_trait_object(
            concrete_value,
            concrete_type,
            &vtable_info,
            counter,
        ))
    }

    /// Generate code for a dynamic method call on a trait object
    pub fn generate_dyn_method_call(
        &self,
        trait_object: &str,
        trait_name: &str,
        method_name: &str,
        args: &[String],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let trait_def = self
            .types.trait_defs
            .get(trait_name)
            .ok_or_else(|| CodegenError::Unsupported(format!("Unknown trait: {}", trait_name)))?;

        // Find method index in trait
        let method_names: Vec<&String> = trait_def.methods.keys().collect();
        let method_index = method_names
            .iter()
            .position(|&n| n == method_name)
            .ok_or_else(|| {
                CodegenError::Unsupported(format!(
                    "Method {} not found in trait {}",
                    method_name, trait_name
                ))
            })?;

        // Get return type
        let method_sig = trait_def.methods.get(method_name).ok_or_else(|| {
            CodegenError::Unsupported(format!(
                "Method {} not found in trait {}",
                method_name, trait_name
            ))
        })?;

        let ret_type = if matches!(method_sig.ret, ResolvedType::Unit) {
            "void"
        } else {
            "i64" // Simplified
        };

        Ok(self.vtable_generator.generate_dynamic_call(
            trait_object,
            method_index,
            args,
            ret_type,
            trait_def,
            counter,
        ))
    }

    fn next_label(&mut self, prefix: &str) -> String {
        debug_assert!(
            !prefix.is_empty() && prefix.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_'),
            "Invalid label prefix: '{}'. Must be non-empty and contain only alphanumeric, '.', or '_' characters.",
            prefix
        );
        let label = format!("{}{}", prefix, self.fn_ctx.label_counter);
        self.fn_ctx.label_counter += 1;
        label
    }

    /// Generate allocation call (malloc or gc_alloc depending on GC mode)
    ///
    /// Returns: (result_register, IR code)
    fn _generate_alloc(
        &self,
        size_arg: &str,
        counter: &mut usize,
        type_id: u32,
    ) -> (String, String) {
        let mut ir = String::new();

        if self.gc_enabled {
            // Use GC allocation
            let ptr_tmp = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = call i8* @vais_gc_alloc(i64 {}, i32 {})\n",
                ptr_tmp, size_arg, type_id
            ));
            let result = self.next_temp(counter);
            ir.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", result, ptr_tmp));
            (result, ir)
        } else {
            // Use manual malloc
            let ptr_tmp = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = call i8* @malloc(i64 {})\n",
                ptr_tmp, size_arg
            ));
            let result = self.next_temp(counter);
            ir.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", result, ptr_tmp));
            (result, ir)
        }
    }

    /// Generates LLVM IR code for a complete module.
    ///
    /// Performs two-pass code generation:
    /// 1. First pass: Collect all type and function declarations
    /// 2. Second pass: Generate code for all function bodies
    ///
    /// # Arguments
    ///
    /// * `module` - The typed AST module to compile
    ///
    /// # Returns
    ///
    /// A string containing the complete LLVM IR code on success,
    /// or a CodegenError on failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use vais_codegen::CodeGenerator;
    /// use vais_parser::parse;
    ///
    /// let source = "F add(x:i64,y:i64)->i64=x+y";
    /// let module = parse(source).unwrap();
    ///
    /// let mut gen = CodeGenerator::new("test");
    /// let ir = gen.generate_module(&module).unwrap();
    /// assert!(ir.contains("define"));
    /// ```
    pub fn generate_module(&mut self, module: &Module) -> CodegenResult<String> {
        let mut ir = String::new();

        self.emit_module_header(&mut ir);

        // First pass: collect declarations
        for item in &module.items {
            match &item.node {
                Item::Function(f) => self.register_function(f)?,
                Item::Struct(s) => {
                    self.register_struct(s)?;
                    // Register struct methods
                    for method in &s.methods {
                        self.register_method(&s.name.node, &method.node)?;
                    }
                }
                Item::Enum(e) => self.register_enum(e)?,
                Item::Union(u) => self.register_union(u)?,
                Item::Impl(impl_block) => {
                    // Register impl methods
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        self.register_method(&type_name, &method.node)?;
                    }
                    // Register trait impl for vtable generation
                    if let Some(ref trait_name) = impl_block.trait_name {
                        let mut method_impls = HashMap::new();
                        for method in &impl_block.methods {
                            let fn_name = format!("{}_{}", type_name, method.node.name.node);
                            method_impls.insert(method.node.name.node.clone(), fn_name);
                        }
                        self.register_trait_impl(&type_name, &trait_name.node, method_impls);
                    }
                }
                Item::Use(_) => {
                    // Use statements are handled at the compiler level (AST merging)
                    // No code generation needed for imports
                }
                Item::Trait(trait_def) => {
                    // Register trait for vtable generation
                    self.register_trait_from_ast(trait_def);
                }
                Item::TypeAlias(_) => {
                    // Type aliases don't generate code
                }
                Item::Macro(_) => {
                    // Macro definitions are expanded at compile time
                    // No runtime code generation needed
                }
                Item::Error { .. } => {
                    // Error nodes indicate parsing failures
                    // Skip them during code generation - errors were reported during parsing
                }
                Item::ExternBlock(extern_block) => {
                    // Register extern functions
                    for func in &extern_block.functions {
                        self.register_extern_function(func, &extern_block.abi)?;
                    }
                }
                Item::Const(const_def) => {
                    // Register constant for code generation
                    self.register_const(const_def)?;
                }
                Item::Global(global_def) => {
                    // Register global variable
                    self.register_global(global_def)?;
                }
            }
        }

        // Generate struct types
        for (name, info) in &self.types.structs {
            ir.push_str(&self.generate_struct_type(name, info));
            ir.push('\n');
        }

        // Generate enum types
        for (name, info) in &self.types.enums {
            ir.push_str(&self.generate_enum_type(name, info));
            ir.push('\n');
        }

        // Generate union types
        for (name, info) in &self.types.unions {
            ir.push_str(&self.generate_union_type(name, info));
            ir.push('\n');
        }

        // Generate function declarations (deduplicate by actual function name)
        // Prioritize non-aliased functions (key == name) over aliased ones (key != name)
        // to ensure correct C type signatures in declare statements
        let mut declared_fns = std::collections::HashSet::new();
        let mut sorted_fns: Vec<_> = self
            .types.functions
            .iter()
            .filter(|(_, info)| info.is_extern)
            .collect();
        sorted_fns.sort_by_key(|(key, info)| if **key == info.signature.name { 0 } else { 1 });
        for (_, info) in &sorted_fns {
            if !declared_fns.contains(&info.signature.name) {
                ir.push_str(&self.generate_extern_decl(info));
                ir.push('\n');
                declared_fns.insert(info.signature.name.clone());
            }
        }

        // Generate string constants (after processing functions to collect all strings)
        let mut body_ir = String::new();

        // Second pass: generate function bodies
        for item in &module.items {
            match &item.node {
                Item::Function(f) => {
                    body_ir.push_str(&self.generate_function_with_span(f, item.span)?);
                    body_ir.push('\n');
                }
                Item::Struct(s) => {
                    // Generate methods for this struct
                    for method in &s.methods {
                        body_ir.push_str(&self.generate_method_with_span(
                            &s.name.node,
                            &method.node,
                            method.span,
                        )?);
                        body_ir.push('\n');
                    }
                }
                Item::Impl(impl_block) => {
                    // Generate methods from impl block
                    // Get the type name from target_type
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        body_ir.push_str(&self.generate_method_with_span(
                            &type_name,
                            &method.node,
                            method.span,
                        )?);
                        body_ir.push('\n');
                    }
                }
                Item::Enum(_)
                | Item::Union(_)
                | Item::Use(_)
                | Item::Trait(_)
                | Item::TypeAlias(_)
                | Item::Macro(_)
                | Item::ExternBlock(_) => {
                    // Already handled in first pass or no code generation needed
                }
                Item::Const(_) | Item::Global(_) => {
                    // Constants and globals are handled in first pass
                }
                Item::Error { .. } => {
                    // Error nodes are skipped - already logged in first pass
                }
            }
        }

        self.emit_string_constants(&mut ir, true);
        self.emit_body_lambdas_vtables(&mut ir, &body_ir);

        // Add WASM runtime functions if targeting WebAssembly
        if self.target.is_wasm() {
            ir.push_str(&self.generate_wasm_runtime());
        }

        // Add helper functions for memory operations (skip for wasm32-unknown-unknown,
        // which provides its own implementations)
        if !matches!(self.target, TargetTriple::Wasm32Unknown) {
            ir.push_str(&self.generate_helper_functions());
        } else {
            // For wasm32-unknown-unknown, only emit minimal helpers that don't conflict
            ir.push_str("\n; Minimal helpers for WASM\n");
        }

        // Add string helper functions if needed
        if self.needs_string_helpers {
            ir.push_str(&self.generate_string_helper_functions());
            if !self.target.is_wasm() {
                ir.push_str(&self.generate_string_extern_declarations());
            }
        }

        // Add contract runtime declarations if any contracts are present
        if !self.contracts.contract_constants.is_empty() {
            ir.push_str(&self.generate_contract_declarations());
            ir.push_str(&self.generate_contract_string_constants());
        }

        // Add debug intrinsic declaration if debug info is enabled
        if self.debug_info.is_enabled() && !self.target.is_wasm() {
            ir.push_str("\n; Debug intrinsics\n");
            ir.push_str("declare void @llvm.dbg.declare(metadata, metadata, metadata)\n");
            ir.push_str("declare void @llvm.dbg.value(metadata, metadata, metadata)\n");
        }

        // Add debug metadata at the end
        if !self.target.is_wasm() {
            ir.push_str(&self.debug_info.finalize());
        }

        // Add WASM import/export metadata attributes
        if self.target.is_wasm() && (!self.wasm_imports.is_empty() || !self.wasm_exports.is_empty())
        {
            ir.push_str("\n; WASM import/export metadata\n");
            ir.push_str(&self.generate_wasm_metadata());
        }

        // Add ABI version metadata
        // ABI version is stored in @__vais_abi_version global constant

        Ok(ir)
    }

    /// Generates LLVM IR code for a complete module with generic instantiations.
    ///
    /// This is the main entry point when monomorphization is enabled.
    /// It takes the generic instantiations collected by the type checker
    /// and generates specialized code for each unique type combination.
    ///
    /// # Arguments
    ///
    /// * `module` - The typed AST module to compile
    /// * `instantiations` - Generic instantiations collected by the type checker
    ///
    /// # Returns
    ///
    /// A string containing the complete LLVM IR code on success,
    /// or a CodegenError on failure.
    pub fn generate_module_with_instantiations(
        &mut self,
        module: &Module,
        instantiations: &[vais_types::GenericInstantiation],
    ) -> CodegenResult<String> {
        let mut ir = String::new();

        self.emit_module_header(&mut ir);

        // First pass: collect declarations (including generic templates)
        for item in &module.items {
            match &item.node {
                Item::Function(f) => {
                    // Track this function name (generic or not)
                    self.types.declared_functions.insert(f.name.node.clone());

                    if !f.generics.is_empty() {
                        // Store generic function for later specialization
                        self.generics.function_templates
                            .insert(f.name.node.clone(), std::rc::Rc::new(f.clone()));
                    } else {
                        self.register_function(f)?;
                    }
                }
                Item::Struct(s) => {
                    if !s.generics.is_empty() {
                        // Store generic struct for later specialization
                        self.generics.struct_defs
                            .insert(s.name.node.clone(), std::rc::Rc::new(s.clone()));
                    } else {
                        self.register_struct(s)?;
                        for method in &s.methods {
                            self.register_method(&s.name.node, &method.node)?;
                        }
                    }
                }
                Item::Enum(e) => self.register_enum(e)?,
                Item::Union(u) => self.register_union(u)?,
                Item::Impl(impl_block) => {
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        self.register_method(&type_name, &method.node)?;
                    }
                    // Register trait impl for vtable generation
                    if let Some(ref trait_name) = impl_block.trait_name {
                        let mut method_impls = HashMap::new();
                        for method in &impl_block.methods {
                            let fn_name = format!("{}_{}", type_name, method.node.name.node);
                            method_impls.insert(method.node.name.node.clone(), fn_name);
                        }
                        self.register_trait_impl(&type_name, &trait_name.node, method_impls);
                    }
                }
                Item::Trait(trait_def) => {
                    self.register_trait_from_ast(trait_def);
                }
                Item::ExternBlock(extern_block) => {
                    for func in &extern_block.functions {
                        self.register_extern_function(func, &extern_block.abi)?;
                    }
                }
                Item::Const(const_def) => {
                    self.register_const(const_def)?;
                }
                Item::Global(global_def) => {
                    self.register_global(global_def)?;
                }
                Item::Use(_) | Item::TypeAlias(_) | Item::Macro(_) | Item::Error { .. } => {}
            }
        }

        // Build generic function instantiation mapping and register specialized function signatures
        for inst in instantiations {
            if let vais_types::InstantiationKind::Function = inst.kind {
                if let Some(generic_fn) = self.generics.function_templates.get(&inst.base_name).cloned() {
                    // Build instantiation mapping: base_name -> [(type_args, mangled_name)]
                    self.generics.fn_instantiations
                        .entry(inst.base_name.clone())
                        .or_default()
                        .push((inst.type_args.clone(), inst.mangled_name.clone()));

                    // Register the specialized function signature so call codegen can find it
                    let substitutions: HashMap<String, ResolvedType> = generic_fn
                        .generics
                        .iter()
                        .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                        .zip(inst.type_args.iter())
                        .map(|(g, t)| (g.name.node.to_string(), t.clone()))
                        .collect();

                    let params: Vec<_> = generic_fn
                        .params
                        .iter()
                        .map(|p| {
                            let ty = self.ast_type_to_resolved(&p.ty.node);
                            let concrete_ty = vais_types::substitute_type(&ty, &substitutions);
                            (p.name.node.to_string(), concrete_ty, p.is_mut)
                        })
                        .collect();

                    let ret_type = generic_fn
                        .ret_type
                        .as_ref()
                        .map(|t| {
                            let ty = self.ast_type_to_resolved(&t.node);
                            vais_types::substitute_type(&ty, &substitutions)
                        })
                        .unwrap_or(ResolvedType::Unit);

                    self.types.functions.insert(
                        inst.mangled_name.clone(),
                        FunctionInfo {
                            signature: vais_types::FunctionSig {
                                name: inst.mangled_name.clone(),
                                params,
                                ret: ret_type,
                                is_async: generic_fn.is_async,
                                ..Default::default()
                            },
                            is_extern: false,
                            _extern_abi: None,
                        },
                    );
                }
            }
        }

        // Generate specialized struct types from instantiations
        for inst in instantiations {
            if let vais_types::InstantiationKind::Struct = inst.kind {
                if let Some(generic_struct) = self.generics.struct_defs.get(&inst.base_name).cloned() {
                    self.generate_specialized_struct_type(&generic_struct, inst, &mut ir)?;
                }
            }
        }

        // Generate non-generic struct types (skip already-emitted specialized generics)
        for (name, info) in &self.types.structs {
            if self.generics.generated_structs.contains_key(name) {
                continue;
            }
            ir.push_str(&self.generate_struct_type(name, info));
            ir.push('\n');
        }

        // Generate enum types
        for (name, info) in &self.types.enums {
            ir.push_str(&self.generate_enum_type(name, info));
            ir.push('\n');
        }

        // Generate union types
        for (name, info) in &self.types.unions {
            ir.push_str(&self.generate_union_type(name, info));
            ir.push('\n');
        }

        // Generate function declarations (extern functions)
        // Prioritize non-aliased functions (key == name) over aliased ones (key != name)
        let mut declared_fns = std::collections::HashSet::new();
        let mut sorted_fns: Vec<_> = self
            .types.functions
            .iter()
            .filter(|(_, info)| info.is_extern)
            .collect();
        sorted_fns.sort_by_key(|(key, info)| if **key == info.signature.name { 0 } else { 1 });
        for (_, info) in &sorted_fns {
            if !declared_fns.contains(&info.signature.name) {
                ir.push_str(&self.generate_extern_decl(info));
                ir.push('\n');
                declared_fns.insert(info.signature.name.clone());
            }
        }

        // Generate string constants (after processing functions to collect all strings)
        let mut body_ir = String::new();

        // Generate specialized functions from instantiations
        for inst in instantiations {
            if let vais_types::InstantiationKind::Function = inst.kind {
                if let Some(generic_fn) = self.generics.function_templates.get(&inst.base_name).cloned() {
                    body_ir.push_str(&self.generate_specialized_function(&generic_fn, inst)?);
                    body_ir.push('\n');
                }
            }
        }

        // Second pass: generate non-generic function bodies
        for item in &module.items {
            match &item.node {
                Item::Function(f) => {
                    if f.generics.is_empty() {
                        body_ir.push_str(&self.generate_function_with_span(f, item.span)?);
                        body_ir.push('\n');
                    }
                }
                Item::Struct(s) => {
                    if s.generics.is_empty() {
                        for method in &s.methods {
                            body_ir.push_str(&self.generate_method_with_span(
                                &s.name.node,
                                &method.node,
                                method.span,
                            )?);
                            body_ir.push('\n');
                        }
                    }
                }
                Item::Impl(impl_block) => {
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        body_ir.push_str(&self.generate_method_with_span(
                            &type_name,
                            &method.node,
                            method.span,
                        )?);
                        body_ir.push('\n');
                    }
                }
                Item::Enum(_)
                | Item::Union(_)
                | Item::Use(_)
                | Item::Trait(_)
                | Item::TypeAlias(_)
                | Item::Macro(_)
                | Item::ExternBlock(_)
                | Item::Const(_)
                | Item::Global(_)
                | Item::Error { .. } => {}
            }
        }

        self.emit_string_constants(&mut ir, true);
        self.emit_body_lambdas_vtables(&mut ir, &body_ir);

        // Add WASM runtime if targeting WebAssembly
        if self.target.is_wasm() {
            ir.push_str(&self.generate_wasm_runtime());
        }

        // Add helper functions
        if !matches!(self.target, TargetTriple::Wasm32Unknown) {
            ir.push_str(&self.generate_helper_functions());
        }

        // Add string helper functions if needed
        if self.needs_string_helpers {
            ir.push_str(&self.generate_string_helper_functions());
            if !self.target.is_wasm() {
                ir.push_str(&self.generate_string_extern_declarations());
            }
        }

        // Add contract runtime declarations if any contracts are present
        if !self.contracts.contract_constants.is_empty() {
            ir.push_str(&self.generate_contract_declarations());
            ir.push_str(&self.generate_contract_string_constants());
        }

        // Add debug intrinsics if debug info is enabled
        if self.debug_info.is_enabled() && !self.target.is_wasm() {
            ir.push_str("\n; Debug intrinsics\n");
            ir.push_str("declare void @llvm.dbg.declare(metadata, metadata, metadata)\n");
            ir.push_str("declare void @llvm.dbg.value(metadata, metadata, metadata)\n");
        }

        // Add debug metadata
        if !self.target.is_wasm() {
            ir.push_str(&self.debug_info.finalize());
        }

        // Add ABI version metadata
        // ABI version is stored in @__vais_abi_version global constant

        Ok(ir)
    }

    // Function generation functions are in function_gen.rs module

    fn next_temp(&self, counter: &mut usize) -> String {
        let tmp = format!("%t.{}", counter);
        *counter += 1;
        tmp
    }

    /// Resolve a generic function call to the appropriate mangled specialized name.
    /// Given a base function name and the inferred argument types, finds the
    /// matching instantiation from the pre-computed instantiation list.
    fn resolve_generic_call(
        &self,
        base_name: &str,
        arg_types: &[ResolvedType],
        instantiations_list: &[(Vec<ResolvedType>, String)],
    ) -> String {
        // If only one instantiation exists, use it directly
        if instantiations_list.len() == 1 {
            return instantiations_list[0].1.clone();
        }

        // Look up the generic function template to map argument types to type parameters
        if let Some(template) = self.generics.function_templates.get(base_name) {
            let type_params: Vec<&String> = template
                .generics
                .iter()
                .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                .map(|g| &g.name.node)
                .collect();

            // Infer type arguments from argument types by matching against parameter types
            let mut inferred: HashMap<String, ResolvedType> = HashMap::new();
            for (i, param) in template.params.iter().enumerate() {
                if i < arg_types.len() {
                    self.infer_type_args(
                        &self.ast_type_to_resolved(&param.ty.node),
                        &arg_types[i],
                        &type_params,
                        &mut inferred,
                    );
                }
            }

            // Build type_args vector in order of generic params
            let type_args: Vec<ResolvedType> = type_params
                .iter()
                .map(|name| inferred.get(*name).cloned().unwrap_or(ResolvedType::I64))
                .collect();

            // Find exact match in instantiations
            for (inst_types, mangled) in instantiations_list {
                if inst_types == &type_args {
                    return mangled.clone();
                }
            }
        }

        // Fallback: try to mangle based on argument types directly
        let mangled = vais_types::mangle_name(base_name, arg_types);
        if self.types.functions.contains_key(&mangled) {
            return mangled;
        }

        // Last resort: use first instantiation
        instantiations_list
            .first()
            .map(|(_, name)| name.clone())
            .unwrap_or_else(|| base_name.to_string())
    }

    /// Infer type arguments by matching a parameter type pattern against a concrete argument type.
    fn infer_type_args(
        &self,
        param_type: &ResolvedType,
        arg_type: &ResolvedType,
        type_params: &[&String],
        inferred: &mut HashMap<String, ResolvedType>,
    ) {
        match param_type {
            ResolvedType::Generic(name) => {
                // Direct generic type parameter (e.g., T)
                if type_params.contains(&name) {
                    inferred
                        .entry(name.clone())
                        .or_insert_with(|| arg_type.clone());
                }
            }
            ResolvedType::Named { name, generics } => {
                // Check if this is a type parameter name
                if type_params.contains(&name) {
                    inferred
                        .entry(name.clone())
                        .or_insert_with(|| arg_type.clone());
                } else if let ResolvedType::Named {
                    generics: arg_generics,
                    ..
                } = arg_type
                {
                    // Recurse into generic arguments
                    for (g, ag) in generics.iter().zip(arg_generics.iter()) {
                        self.infer_type_args(g, ag, type_params, inferred);
                    }
                }
            }
            ResolvedType::Array(inner) => {
                if let ResolvedType::Array(arg_inner) = arg_type {
                    self.infer_type_args(inner, arg_inner, type_params, inferred);
                }
            }
            ResolvedType::Pointer(inner) => {
                if let ResolvedType::Pointer(arg_inner) = arg_type {
                    self.infer_type_args(inner, arg_inner, type_params, inferred);
                }
            }
            ResolvedType::Optional(inner) => {
                if let ResolvedType::Optional(arg_inner) = arg_type {
                    self.infer_type_args(inner, arg_inner, type_params, inferred);
                }
            }
            _ => {}
        }
    }

    /// Generate code for a block expression (used in if/else branches)
    fn _generate_block_expr(
        &mut self,
        expr: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        match &expr.node {
            Expr::Block(stmts) => {
                let (val, ir, _terminated) = self.generate_block_stmts(stmts, counter)?;
                Ok((val, ir))
            }
            _ => self.generate_expr(expr, counter),
        }
    }

    /// Generate code for a block of statements
    /// Returns (value, ir_code, is_terminated)
    fn generate_block_stmts(
        &mut self,
        stmts: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String, bool)> {
        // Use StmtVisitor trait for statement code generation
        use crate::visitor::StmtVisitor;
        self.visit_block_stmts(stmts, counter)
    }

    // Control flow functions (if-else, match, pattern matching) are in control_flow.rs module

    // Type inference functions are in type_inference.rs module

    /// Generate code for array slicing: arr[start..end]
    /// Returns a new array (allocated on heap) containing the slice
    fn generate_slice(
        &mut self,
        array_expr: &Spanned<Expr>,
        start: Option<&Spanned<Expr>>,
        end: Option<&Spanned<Expr>>,
        inclusive: bool,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (arr_val, arr_ir) = self.generate_expr(array_expr, counter)?;
        let mut ir = arr_ir;

        // Get start index (default 0)
        let start_val = if let Some(start_expr) = start {
            let (val, start_ir) = self.generate_expr(start_expr, counter)?;
            ir.push_str(&start_ir);
            val
        } else {
            "0".to_string()
        };

        // Get end index
        // For simplicity, we require end to be specified for now
        // A proper implementation would need array length tracking
        let end_val = if let Some(end_expr) = end {
            let (val, end_ir) = self.generate_expr(end_expr, counter)?;
            ir.push_str(&end_ir);

            // If inclusive (..=), add 1 to end
            if inclusive {
                let adj_end = self.next_temp(counter);
                ir.push_str(&format!("  {} = add i64 {}, 1\n", adj_end, val));
                adj_end
            } else {
                val
            }
        } else {
            return Err(CodegenError::Unsupported(
                "Slice without end index requires array length".to_string(),
            ));
        };

        // Calculate slice length: end - start
        let length = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = sub i64 {}, {}\n",
            length, end_val, start_val
        ));

        // Allocate new array for slice
        let byte_size = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = mul i64 {}, 8\n", // 8 bytes per i64 element
            byte_size, length
        ));

        let raw_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i8* @malloc(i64 {})\n",
            raw_ptr, byte_size
        ));

        let slice_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = bitcast i8* {} to i64*\n",
            slice_ptr, raw_ptr
        ));

        // Copy elements using a loop
        let loop_idx_ptr = self.next_temp(counter);
        ir.push_str(&format!("  {} = alloca i64\n", loop_idx_ptr));
        ir.push_str(&format!("  store i64 0, i64* {}\n", loop_idx_ptr));

        let loop_start = self.next_label("slice_loop");
        let loop_body = self.next_label("slice_body");
        let loop_end = self.next_label("slice_end");

        ir.push_str(&format!("  br label %{}\n", loop_start));
        ir.push_str(&format!("{}:\n", loop_start));

        let loop_idx = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = load i64, i64* {}\n",
            loop_idx, loop_idx_ptr
        ));

        let cmp = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = icmp slt i64 {}, {}\n",
            cmp, loop_idx, length
        ));
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cmp, loop_body, loop_end
        ));

        ir.push_str(&format!("{}:\n", loop_body));

        // Calculate source index: start + loop_idx
        let src_idx = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = add i64 {}, {}\n",
            src_idx, start_val, loop_idx
        ));

        // Get source element pointer
        let src_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr i64, i64* {}, i64 {}\n",
            src_ptr, arr_val, src_idx
        ));

        // Load source element
        let elem = self.next_temp(counter);
        ir.push_str(&format!("  {} = load i64, i64* {}\n", elem, src_ptr));

        // Get destination element pointer
        let dst_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr i64, i64* {}, i64 {}\n",
            dst_ptr, slice_ptr, loop_idx
        ));

        // Store element
        ir.push_str(&format!("  store i64 {}, i64* {}\n", elem, dst_ptr));

        // Increment loop index
        let next_idx = self.next_temp(counter);
        ir.push_str(&format!("  {} = add i64 {}, 1\n", next_idx, loop_idx));
        ir.push_str(&format!(
            "  store i64 {}, i64* {}\n",
            next_idx, loop_idx_ptr
        ));
        ir.push_str(&format!("  br label %{}\n", loop_start));

        ir.push_str(&format!("{}:\n", loop_end));

        Ok((slice_ptr, ir))
    }

    // Lambda closure capture functions are in lambda_closure.rs module
}

#[cfg(test)]
mod tests {
    use super::*;
    use vais_parser::parse;

    #[test]
    fn test_simple_function() {
        let source = "F add(a:i64,b:i64)->i64=a+b";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @add"));
        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_fibonacci() {
        let source = "F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @fib"));
        assert!(ir.contains("call i64 @fib"));
    }

    #[test]
    fn test_if_else() {
        // I cond { then } E { else }
        let source = "F max(a:i64,b:i64)->i64{I a>b{R a}E{R b}}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @max"));
        assert!(ir.contains("br i1"));
        assert!(ir.contains("then"));
        assert!(ir.contains("else"));
    }

    #[test]
    fn test_loop_with_condition() {
        // L pattern:iter { body } - `L _:condition{body}` for while loop
        let source = "F countdown(n:i64)->i64{x:=n;L _:x>0{x=x-1};x}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @countdown"));
        assert!(ir.contains("loop.start"));
        assert!(ir.contains("loop.body"));
        assert!(ir.contains("loop.end"));
    }

    #[test]
    fn test_array_literal() {
        let source = "F get_arr()->*i64=[1,2,3]";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("alloca [3  x i64]"));
        assert!(ir.contains("getelementptr"));
        assert!(ir.contains("store i64"));
    }

    #[test]
    fn test_array_index() {
        let source = "F get_elem(arr:*i64, idx:i64)->i64=arr[idx]";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("getelementptr i64, i64*"));
        assert!(ir.contains("load i64, i64*"));
    }

    #[test]
    fn test_struct_codegen() {
        let source = "S Point{x:i64,y:i64}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("%Point = type { i64, i64 }"));
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_empty_module() {
        let source = "";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Should generate valid LLVM IR even with empty module
        assert!(ir.contains("source_filename"));
    }

    #[test]
    fn test_minimal_function() {
        let source = "F f()->()=()";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define void @f"));
        assert!(ir.contains("ret void"));
    }

    #[test]
    fn test_function_returning_unit() {
        let source = "F void_fn()->(){}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define void @void_fn"));
    }

    #[test]
    fn test_empty_struct() {
        let source = "S Empty{}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Empty struct should still generate a type
        assert!(ir.contains("%Empty = type"));
    }

    #[test]
    fn test_single_field_struct() {
        let source = "S Single{x:i64}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("%Single = type { i64 }"));
    }

    #[test]
    fn test_enum_with_variants() {
        let source = "E Color{Red,Green,Blue}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Enum should generate a type
        assert!(ir.contains("%Color = type"));
    }

    #[test]
    fn test_i64_max_value() {
        let source = "F max()->i64=9223372036854775807";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("9223372036854775807"));
    }

    #[test]
    fn test_negative_number() {
        let source = "F neg()->i64=-42";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Negative numbers involve subtraction from 0
        assert!(ir.contains("sub i64 0, 42"));
    }

    #[test]
    fn test_zero_value() {
        let source = "F zero()->i64=0";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("ret i64 0"));
    }

    #[test]
    fn test_float_values() {
        let source = "F pi()->f64=3.141592653589793";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("double"));
    }

    #[test]
    fn test_boolean_true() {
        let source = "F yes()->bool=true";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("ret i1 true") || ir.contains("ret i1 1"));
    }

    #[test]
    fn test_boolean_false() {
        let source = "F no()->bool=false";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("ret i1 false") || ir.contains("ret i1 0"));
    }

    #[test]
    fn test_empty_string() {
        let source = r#"F empty()->str="""#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Should handle empty string
        assert!(ir.contains("@str") || ir.contains("i8*"));
    }

    #[test]
    fn test_string_with_escape() {
        let source = r#"F escaped()->str="hello\nworld""#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Should handle escape sequences
        assert!(ir.contains("@str"));
    }

    #[test]
    fn test_empty_array() {
        let source = "F empty_arr()->*i64=[]";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Empty array should still work
        assert!(ir.contains("define"));
    }

    #[test]
    fn test_nested_if_else() {
        let source = r#"
            F classify(x:i64)->i64{
                I x>0{
                    I x>100{2}E{1}
                }E{
                    I x<-100{-2}E{-1}
                }
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @classify"));
        // Should have multiple branches
        assert!(ir.contains("br i1"));
    }

    #[test]
    fn test_simple_match() {
        let source = "F digit(n:i64)->str=M n{0=>\"zero\",1=>\"one\",_=>\"other\"}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define"));
    }

    #[test]
    fn test_for_loop() {
        let source = "F sum_to(n:i64)->i64{s:=0;L i:0..n{s+=i};s}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @sum_to"));
        assert!(ir.contains("for.cond"));
        assert!(ir.contains("for.body"));
        assert!(ir.contains("for.inc"));
    }

    #[test]
    fn test_while_loop() {
        let source = "F count_down(n:i64)->i64{x:=n;L _:x>0{x-=1};x}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @count_down"));
    }

    #[test]
    fn test_infinite_loop_with_break() {
        let source = "F find()->i64{x:=0;L{I x>10{B x};x+=1};0}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @find"));
    }

    #[test]
    fn test_arithmetic_operations() {
        let source = "F math(a:i64,b:i64)->i64=a+b-a*b/a%b";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("add i64"));
        assert!(ir.contains("sub i64"));
        assert!(ir.contains("mul i64"));
        assert!(ir.contains("sdiv i64"));
        assert!(ir.contains("srem i64"));
    }

    #[test]
    fn test_comparison_operations() {
        let source = r#"
            F compare(a:i64,b:i64)->bool{
                x:=a<b;
                y:=a<=b;
                z:=a>b;
                w:=a>=b;
                u:=a==b;
                v:=a!=b;
                x
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("icmp slt"));
        assert!(ir.contains("icmp sle"));
        assert!(ir.contains("icmp sgt"));
        assert!(ir.contains("icmp sge"));
        assert!(ir.contains("icmp eq"));
        assert!(ir.contains("icmp ne"));
    }

    #[test]
    fn test_bitwise_operations() {
        let source = "F bits(a:i64,b:i64)->i64{x:=a&b;y:=a|b;z:=a^b;w:=a<<2;v:=a>>1;x}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("and i64"));
        assert!(ir.contains("or i64"));
        assert!(ir.contains("xor i64"));
        assert!(ir.contains("shl i64"));
        assert!(ir.contains("ashr i64"));
    }

    #[test]
    fn test_logical_operations() {
        let source = "F logic(a:bool,b:bool)->bool=a&&b||!a";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i1 @logic"));
    }

    #[test]
    fn test_unary_minus() {
        let source = "F negate(x:i64)->i64=-x";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("sub i64 0"));
    }

    #[test]
    fn test_bitwise_not() {
        let source = "F complement(x:i64)->i64=~x";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("xor i64") && ir.contains("-1"));
    }

    #[test]
    fn test_ternary_expression() {
        let source = "F abs(x:i64)->i64=x<0?-x:x";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @abs"));
        assert!(ir.contains("br i1"));
    }

    #[test]
    fn test_compound_assignment() {
        // In Vais, mutable variables use := for declaration
        let source = r#"
            F compound(x:i64)->i64{
                y:=x;
                y+=1;
                y-=2;
                y*=3;
                y/=4;
                y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @compound"));
    }

    #[test]
    fn test_struct_literal() {
        let source = r#"
            S Point{x:i64,y:i64}
            F origin()->Point=Point{x:0,y:0}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("%Point = type { i64, i64 }"));
        assert!(ir.contains("define %Point"));
    }

    #[test]
    fn test_struct_field_access() {
        let source = r#"
            S Point{x:i64,y:i64}
            F get_x(p:Point)->i64=p.x
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("getelementptr"));
    }

    #[test]
    fn test_lambda_simple() {
        let source = "F f()->i64{add:=|a:i64,b:i64|a+b;add(1,2)}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @f"));
    }

    #[test]
    fn test_recursive_factorial() {
        let source = "F factorial(n:i64)->i64=n<=1?1:n*@(n-1)";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @factorial"));
        assert!(ir.contains("call i64 @factorial"));
    }

    #[test]
    fn test_multiple_functions() {
        let source = r#"
            F add(a:i64,b:i64)->i64=a+b
            F sub(a:i64,b:i64)->i64=a-b
            F mul(a:i64,b:i64)->i64=a*b
            F test()->i64=mul(add(1,2),sub(5,2))
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @add"));
        assert!(ir.contains("define i64 @sub"));
        assert!(ir.contains("define i64 @mul"));
        assert!(ir.contains("define i64 @test"));
    }

    #[test]
    fn test_function_with_many_params() {
        let source = "F many(a:i64,b:i64,c:i64,d:i64,e:i64,f:i64,g:i64,h:i64)->i64=a+b+c+d+e+f+g+h";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // LLVM IR uses %a, %b etc, and the define line may not have spaces
        assert!(ir.contains("define i64 @many"));
        assert!(ir.contains("i64 %a"));
        assert!(ir.contains("i64 %h"));
    }

    #[test]
    fn test_all_integer_types() {
        let source = r#"
            F test_i8(x:i8)->i8=x
            F test_i16(x:i16)->i16=x
            F test_i32(x:i32)->i32=x
            F test_i64(x:i64)->i64=x
            F test_u8(x:u8)->u8=x
            F test_u16(x:u16)->u16=x
            F test_u32(x:u32)->u32=x
            F test_u64(x:u64)->u64=x
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i8 @test_i8"));
        assert!(ir.contains("define i16 @test_i16"));
        assert!(ir.contains("define i32 @test_i32"));
        assert!(ir.contains("define i64 @test_i64"));
        assert!(ir.contains("define i8 @test_u8"));
        assert!(ir.contains("define i16 @test_u16"));
        assert!(ir.contains("define i32 @test_u32"));
        assert!(ir.contains("define i64 @test_u64"));
    }

    #[test]
    fn test_float_types() {
        let source = r#"
            F test_f32(x:f32)->f32=x
            F test_f64(x:f64)->f64=x
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define float @test_f32"));
        assert!(ir.contains("define double @test_f64"));
    }

    #[test]
    fn test_deeply_nested_expression() {
        let source = "F deep(a:i64)->i64=((((a+1)+2)+3)+4)+5";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @deep"));
    }

    #[test]
    fn test_mixed_arithmetic_precedence() {
        let source = "F prec(a:i64,b:i64,c:i64)->i64=a+b*c";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Should multiply first then add (precedence)
        assert!(ir.contains("mul i64"));
        assert!(ir.contains("add i64"));
    }

    // ==================== Generic Instantiation Tests ====================

    #[test]
    fn test_generate_specialized_function() {
        use vais_types::TypeChecker;

        let source = r#"
            F identity<T>(x:T)->T=x
            F main()->i64=identity(42)
        "#;
        let module = parse(source).unwrap();

        // First, type check to get instantiations
        let mut checker = TypeChecker::new();
        checker.check_module(&module).unwrap();
        let instantiations = checker.get_generic_instantiations();

        // Generate code with instantiations
        let mut gen = CodeGenerator::new("test");
        let ir = gen
            .generate_module_with_instantiations(&module, instantiations)
            .unwrap();

        // Should contain specialized function identity$i64
        assert!(
            ir.contains("define i64 @identity$i64"),
            "Expected identity$i64 in IR: {}",
            ir
        );
        assert!(ir.contains("ret i64 %x"), "Expected return in identity$i64");
    }

    #[test]
    fn test_generate_specialized_struct_type() {
        use vais_types::TypeChecker;

        // Test that generic struct type definition is specialized
        // Note: Full struct literal code generation with generics requires additional work
        // This test verifies the type definition is generated correctly
        let source = r#"
            S Pair<T>{first:T,second:T}
            F main()->i64{
                p:=Pair{first:1,second:2};
                p.first
            }
        "#;
        let module = parse(source).unwrap();

        // Type check to get instantiations
        let mut checker = TypeChecker::new();
        checker.check_module(&module).unwrap();
        let instantiations = checker.get_generic_instantiations();

        // Verify instantiation was recorded
        let pair_inst = instantiations.iter().find(|i| i.base_name == "Pair");
        assert!(
            pair_inst.is_some(),
            "Expected Pair instantiation to be recorded"
        );

        // Verify mangled name
        let inst = pair_inst.unwrap();
        assert_eq!(
            inst.mangled_name, "Pair$i64",
            "Expected mangled name Pair$i64, got {}",
            inst.mangled_name
        );
    }

    #[test]
    fn test_multiple_instantiations() {
        use vais_types::TypeChecker;

        let source = r#"
            F identity<T>(x:T)->T=x
            F main()->f64{
                a:=identity(42);
                b:=identity(3.14);
                b
            }
        "#;
        let module = parse(source).unwrap();

        // Type check to get instantiations
        let mut checker = TypeChecker::new();
        checker.check_module(&module).unwrap();
        let instantiations = checker.get_generic_instantiations();

        // Should have at least 2 instantiations
        assert!(
            instantiations.len() >= 2,
            "Expected at least 2 instantiations, got {}",
            instantiations.len()
        );

        // Generate code with instantiations
        let mut gen = CodeGenerator::new("test");
        let ir = gen
            .generate_module_with_instantiations(&module, instantiations)
            .unwrap();

        // Should contain both specialized functions
        assert!(ir.contains("@identity$i64"), "Expected identity$i64 in IR");
        assert!(ir.contains("@identity$f64"), "Expected identity$f64 in IR");
    }

    #[test]
    fn test_no_code_for_generic_template() {
        use vais_types::TypeChecker;

        let source = r#"
            F identity<T>(x:T)->T=x
        "#;
        let module = parse(source).unwrap();

        // Type check (no instantiations since function isn't called)
        let mut checker = TypeChecker::new();
        checker.check_module(&module).unwrap();
        let instantiations = checker.get_generic_instantiations();

        // No instantiations
        assert!(instantiations.is_empty());

        // Generate code with empty instantiations
        let mut gen = CodeGenerator::new("test");
        let ir = gen
            .generate_module_with_instantiations(&module, instantiations)
            .unwrap();

        // Should NOT contain any identity function definition
        assert!(
            !ir.contains("define i64 @identity"),
            "Generic template should not generate code"
        );
        assert!(
            !ir.contains("define double @identity"),
            "Generic template should not generate code"
        );
    }

    // ==================== Advanced Edge Case Tests ====================

    #[test]
    fn test_i8_boundary_values() {
        // Test i8 min (-128) and max (127)
        let source = r#"
            F i8_bounds()->(i8,i8){
                min:i8=-128;
                max:i8=127;
                (min,max)
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Check for i8 type usage
        assert!(ir.contains("i8"));
    }

    #[test]
    fn test_i8_overflow_value() {
        // Test arithmetic that could overflow (using i64 as i8 not fully supported)
        let source = r#"
            F add_large()->i64{
                x:=9000000000000000000;
                y:=1000000000000000000;
                x+y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should generate code (overflow behavior is runtime)
        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_i8_underflow_value() {
        // Test arithmetic that could underflow (using i64)
        let source = r#"
            F sub_large()->i64{
                x:=-9000000000000000000;
                y:=1000000000000000000;
                x-y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("sub i64"));
    }

    #[test]
    fn test_i64_max_value_codegen() {
        // Test i64 max: 9223372036854775807
        let source = r#"
            F i64_max()->i64=9223372036854775807
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("9223372036854775807"));
    }

    #[test]
    fn test_i64_min_value_codegen() {
        // Test i64 min (approximately): -9223372036854775808
        let source = r#"
            F i64_near_min()->i64=-9223372036854775807
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("sub i64 0, 9223372036854775807"));
    }

    #[test]
    fn test_integer_overflow_addition() {
        // Test potential overflow in addition
        let source = r#"
            F add_overflow(a:i64,b:i64)->i64=a+b
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should generate regular add (overflow is runtime behavior)
        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_integer_overflow_multiplication() {
        // Test potential overflow in multiplication
        let source = r#"
            F mul_large()->i64{
                a:i64=1000000000;
                b:i64=1000000000;
                a*b
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("mul i64"));
    }

    #[test]
    fn test_division_by_zero() {
        // Test division by zero (runtime error, should compile)
        let source = r#"
            F div_zero()->i64{
                x:=10;
                y:=0;
                x/y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("sdiv i64"));
    }

    #[test]
    fn test_modulo_by_zero() {
        // Test modulo by zero
        let source = r#"
            F mod_zero()->i64{
                x:=10;
                y:=0;
                x%y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("srem i64"));
    }

    #[test]
    fn test_all_integer_type_boundaries() {
        // Test boundary values for all integer types
        // Note: Variables must be used for type information to appear in IR
        // Vais primarily uses i64 for integer arithmetic, but stores typed values
        // Test that integer literals with annotations generate valid IR
        let source = r#"
            F get_i8()->i8{
                a:i8=127;
                a
            }
            F get_i32()->i32{
                e:i32=2147483647;
                e
            }
            F get_i64()->i64{
                f:i64=9223372036854775807;
                f
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Check that the IR contains function definitions with correct return types
        assert!(ir.contains("i8"), "IR should contain i8 type");
        assert!(ir.contains("i32"), "IR should contain i32 type");
        assert!(ir.contains("i64"), "IR should contain i64 type");
    }

    #[test]
    fn test_signed_integer_wraparound() {
        // Test signed integer wraparound behavior (using i64)
        let source = r#"
            F wraparound()->i64{
                max:=9223372036854775806;
                max+1
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_nested_generic_codegen() {
        // Simplified generic struct test
        let source = r#"
            S Container<T>{data:T}
            F empty()->Container<i64> =Container{data:0}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("%Container"));
    }

    #[test]
    fn test_pattern_match_with_guard_codegen() {
        // Test pattern match with guard generates correct branches (fix escaping)
        let source = r#"
            F classify(x:i64)->str=M x{
                n I n>0=>"pos",
                n I n<0=>"neg",
                _=>"zero"
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have branches for guards
        assert!(ir.contains("br i1"));
    }

    #[test]
    fn test_mutual_recursion_codegen() {
        // Test mutual recursion generates correct calls
        let source = r#"
            F is_even(n:i64)->bool=n==0?true:is_odd(n-1)
            F is_odd(n:i64)->bool=n==0?false:is_even(n-1)
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i1 @is_even"));
        assert!(ir.contains("define i1 @is_odd"));
        assert!(ir.contains("call i1 @is_odd"));
        assert!(ir.contains("call i1 @is_even"));
    }

    #[test]
    fn test_deeply_nested_if_codegen() {
        // Test deeply nested if-else generates correct basic blocks
        let source = r#"
            F deep(x:i64)->i64{
                I x>100{
                    I x>1000{1}E{2}
                }E{
                    I x>10{
                        I x>50{3}E{4}
                    }E{5}
                }
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have multiple branches
        let br_count = ir.matches("br i1").count();
        assert!(br_count >= 4, "Expected at least 4 branches");
    }

    #[test]
    fn test_large_number_of_parameters() {
        // Test function with many parameters
        let source = r#"
            F many_params(
                a:i64,b:i64,c:i64,d:i64,e:i64,
                f:i64,g:i64,h:i64,i:i64,j:i64,
                k:i64,l:i64,m:i64,n:i64,o:i64
            )->i64=a+b+c+d+e+f+g+h+i+j+k+l+m+n+o
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @many_params"));
        // Check for parameter usage
        assert!(ir.contains("%a"));
        assert!(ir.contains("%o"));
    }

    #[test]
    fn test_zero_return_optimization() {
        // Test that returning 0 is optimized
        let source = r#"
            F zero()->i64=0
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("ret i64 0"));
    }

    #[test]
    fn test_constant_folding_candidate() {
        // Test expressions that could be constant folded
        let source = r#"
            F const_expr()->i64=2+3*4-1
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should generate arithmetic operations
        assert!(ir.contains("add i64") || ir.contains("ret i64 13"));
        assert!(ir.contains("mul i64") || ir.contains("ret i64 13"));
    }

    #[test]
    fn test_boolean_short_circuit() {
        // Test boolean short-circuit evaluation
        let source = r#"
            F short_circuit(a:bool,b:bool)->bool=a&&b||!a
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i1 @short_circuit"));
    }

    #[test]
    fn test_comparison_chain_codegen() {
        // Test comparison chains: a < b < c
        let source = r#"
            F compare_chain(a:i64,b:i64,c:i64)->bool{
                x:=a<b;
                y:=b<c;
                x&&y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("icmp slt"));
    }

    #[test]
    fn test_bitwise_operations_all_types() {
        // Test bitwise operations (i8 not fully supported, use i64)
        let source = r#"
            F bitwise_i64(a:i64,b:i64)->i64=a&b|a^b
            F bitwise_test()->i64=bitwise_i64(5,3)
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("and i64"));
        assert!(ir.contains("or i64"));
        assert!(ir.contains("xor i64"));
    }

    #[test]
    fn test_shift_operations_boundaries() {
        // Test shift operations at boundaries
        let source = r#"
            F shift_max(x:i64)->i64{
                a:=x<<63;
                b:=x>>63;
                a+b
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("shl i64"));
        assert!(ir.contains("ashr i64"));
    }

    #[test]
    fn test_negative_shift_amount() {
        // Test negative shift (undefined behavior, should compile)
        let source = r#"
            F neg_shift(x:i64)->i64{
                shift:=-1;
                x<<shift
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("shl i64"));
    }

    #[test]
    fn test_all_unary_operators() {
        // Test all unary operators
        let source = r#"
            F unary_ops(x:i64,b:bool)->(i64,i64,bool){
                neg:=-x;
                bit_not:=(~x);
                log_not:=!b;
                (neg,bit_not,log_not)
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("sub i64 0")); // negation
        assert!(ir.contains("xor i64") && ir.contains("-1")); // bitwise not
    }

    #[test]
    fn test_float_division_by_zero() {
        // Test float division (check IR has float division instruction)
        let source = r#"
            F fdiv_test(x:f64,y:f64)->f64=x/y
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Check that float division is generated
        assert!(ir.contains("fdiv") || ir.contains("define double"));
    }

    #[test]
    fn test_recursive_depth() {
        // Test deep recursion (should compile, runtime stack depth)
        let source = r#"
            F deep_recursion(n:i64)->i64=n<1?0:@(n-1)+1
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("call i64 @deep_recursion"));
    }

    // ==================== Decreases Termination Tests ====================

    #[test]
    fn test_decreases_basic() {
        // Test basic decreases clause for termination proof
        let source = r#"
            #[requires(n >= 0)]
            #[decreases(n)]
            F factorial(n:i64)->i64{I n<=1{R 1}R n*factorial(n-1)}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have initial decreases storage
        assert!(
            ir.contains("__decreases_factorial"),
            "Expected decreases storage variable"
        );
        // Should have non-negative check
        assert!(
            ir.contains("decreases_nonneg"),
            "Expected non-negative check"
        );
        // Should have strict decrease check before recursive call
        assert!(
            ir.contains("decreases_check"),
            "Expected decrease check before recursive call"
        );
        // Should have panic call for failed check
        assert!(
            ir.contains("@__panic"),
            "Expected panic call for failed check"
        );
    }

    #[test]
    fn test_decreases_strict_decrease_check() {
        // Test that the strict decrease check (new < old) is generated
        let source = r#"
            #[decreases(n)]
            F count_down(n:i64)->i64{I n<=0{R 0}R count_down(n-1)}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have icmp slt (strictly less than) check
        assert!(
            ir.contains("icmp slt i64"),
            "Expected strict less-than comparison for decreases"
        );
        // Should have both decreases labels
        assert!(ir.contains("decreases_check_ok"), "Expected success label");
        assert!(
            ir.contains("decreases_check_fail"),
            "Expected failure label"
        );
    }

    #[test]
    fn test_decreases_nonneg_check() {
        // Test that non-negative check is generated for decreases expression
        let source = r#"
            #[decreases(x)]
            F process(x:i64)->i64{I x<=0{R 0}R process(x-1)+1}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have icmp sge (signed greater-or-equal) for non-negative check
        assert!(
            ir.contains("icmp sge i64"),
            "Expected non-negative check (sge 0)"
        );
        assert!(
            ir.contains("decreases_nonneg_ok"),
            "Expected success label for non-negative"
        );
        assert!(
            ir.contains("decreases_nonneg_fail"),
            "Expected failure label for non-negative"
        );
    }

    #[test]
    fn test_decreases_release_mode() {
        // Test that decreases checks are skipped in release mode
        let source = r#"
            #[decreases(n)]
            F fib(n:i64)->i64{I n<2{R n}R fib(n-1)+fib(n-2)}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        gen.enable_release_mode();
        let ir = gen.generate_module(&module).unwrap();

        // Should NOT have decreases checks in release mode
        assert!(
            !ir.contains("__decreases_fib"),
            "Should skip decreases in release mode"
        );
        assert!(
            !ir.contains("decreases_nonneg"),
            "Should skip non-negative check in release mode"
        );
        assert!(
            !ir.contains("decreases_check"),
            "Should skip decrease check in release mode"
        );
    }

    #[test]
    fn test_decreases_with_selfcall() {
        // Test decreases with @ self-call operator
        let source = r#"
            #[decreases(n)]
            F sum_to(n:i64)->i64{I n<=0{R 0}R n+@(n-1)}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have decreases check before the self-call
        assert!(
            ir.contains("__decreases_sum_to"),
            "Expected decreases storage"
        );
        assert!(
            ir.contains("decreases_check"),
            "Expected decrease check before self-call"
        );
    }

    #[test]
    fn test_type_recursion_depth_limit() {
        // Test that deeply nested types work within the limit
        use vais_types::ResolvedType;

        let gen = CodeGenerator::new("test");

        // Create a deeply nested pointer type (should work)
        let mut nested_type = ResolvedType::I32;
        for _ in 0..50 {
            nested_type = ResolvedType::Pointer(Box::new(nested_type));
        }

        // This should work fine (well within the 128 limit)
        let llvm_type = gen.type_to_llvm(&nested_type);
        assert!(llvm_type.ends_with('*'), "Should generate nested pointers");

        // Create an extremely deeply nested type (exceeds limit of 128)
        let mut extremely_nested = ResolvedType::I32;
        for _ in 0..150 {
            extremely_nested = ResolvedType::Pointer(Box::new(extremely_nested));
        }

        // This should hit the recursion limit and fall back to i64
        // (The error is logged but doesn't fail - returns fallback type)
        let llvm_type_over_limit = gen.type_to_llvm(&extremely_nested);
        // Should still return a valid type (either i64 fallback or truncated)
        assert!(
            !llvm_type_over_limit.is_empty(),
            "Should return a fallback type on recursion limit"
        );
    }

    #[test]
    fn test_type_recursion_reset_between_calls() {
        // Test that recursion depth is properly reset between calls
        use vais_types::ResolvedType;

        let gen = CodeGenerator::new("test");

        // First call with nested types
        let mut nested1 = ResolvedType::I32;
        for _ in 0..30 {
            nested1 = ResolvedType::Pointer(Box::new(nested1));
        }
        let _ = gen.type_to_llvm(&nested1);

        // Second call should work independently (depth should be reset)
        let mut nested2 = ResolvedType::I64;
        for _ in 0..30 {
            nested2 = ResolvedType::Pointer(Box::new(nested2));
        }
        let llvm_type = gen.type_to_llvm(&nested2);
        assert!(
            llvm_type.ends_with('*'),
            "Second call should work independently"
        );
    }

    #[test]
    fn test_ast_type_recursion_limit() {
        // Test that ast_type_to_resolved also respects recursion limits
        use vais_ast::{Span, Type};

        let gen = CodeGenerator::new("test");

        // Create deeply nested AST type
        let mut nested = Type::Named {
            name: "i32".to_string(),
            generics: vec![],
        };
        for _ in 0..50 {
            nested = Type::Pointer(Box::new(Spanned::new(nested, Span { start: 0, end: 0 })));
        }

        // Should work within limit
        let resolved = gen.ast_type_to_resolved(&nested);
        assert!(
            matches!(resolved, ResolvedType::Pointer(_)),
            "Should resolve nested pointers"
        );

        // Create extremely nested type (exceeds limit)
        let mut extremely_nested = Type::Named {
            name: "i32".to_string(),
            generics: vec![],
        };
        for _ in 0..150 {
            extremely_nested = Type::Pointer(Box::new(Spanned::new(
                extremely_nested,
                Span { start: 0, end: 0 },
            )));
        }

        // Should hit limit and return fallback
        let resolved_over = gen.ast_type_to_resolved(&extremely_nested);
        // Should still return a valid type (Unknown as fallback)
        assert!(
            matches!(
                resolved_over,
                ResolvedType::Unknown | ResolvedType::Pointer(_)
            ),
            "Should return a fallback or truncated type on recursion limit"
        );
    }

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("", ""), 0);
        assert_eq!(edit_distance("hello", "hello"), 0);
        assert_eq!(edit_distance("hello", "hallo"), 1);
        assert_eq!(edit_distance("hello", "hell"), 1);
        assert_eq!(edit_distance("hello", "helloo"), 1);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("saturday", "sunday"), 3);
    }

    #[test]
    fn test_suggest_similar() {
        let candidates = vec!["count", "counter", "account", "mount", "county"];

        // Exact case-insensitive match should be prioritized
        let suggestions = suggest_similar("COUNT", &candidates, 3);
        assert_eq!(suggestions[0], "count");

        // Close matches
        let suggestions = suggest_similar("countr", &candidates, 3);
        assert!(suggestions.contains(&"counter".to_string()));
        assert!(suggestions.contains(&"count".to_string()));

        // Should limit to max_suggestions
        let suggestions = suggest_similar("cont", &candidates, 2);
        assert!(suggestions.len() <= 2);

        // No matches if too far
        let suggestions = suggest_similar("xyz", &candidates, 3);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_format_did_you_mean() {
        assert_eq!(format_did_you_mean(&[]), "");
        assert_eq!(
            format_did_you_mean(&["foo".to_string()]),
            ". Did you mean `foo`?"
        );
        assert_eq!(
            format_did_you_mean(&["foo".to_string(), "bar".to_string()]),
            ". Did you mean `foo` or `bar`?"
        );
        assert_eq!(
            format_did_you_mean(&["foo".to_string(), "bar".to_string(), "baz".to_string()]),
            ". Did you mean `foo`, `bar`, or `baz`?"
        );
    }

    #[test]
    fn test__suggest_type_conversion() {
        // Numeric conversions
        assert!(_suggest_type_conversion("i64", "f64").contains("as i64"));
        assert!(_suggest_type_conversion("f64", "i64").contains("as f64"));
        assert!(_suggest_type_conversion("i32", "i64").contains("as i32"));

        // String conversions
        assert!(_suggest_type_conversion("String", "&str").contains(".to_string()"));
        assert!(_suggest_type_conversion("&str", "String").contains(".as_str()"));

        // Bool to int
        assert!(_suggest_type_conversion("i64", "bool").contains("as i64"));

        // No suggestion for unrelated types
        assert_eq!(_suggest_type_conversion("Vec", "HashMap"), "");
    }
}

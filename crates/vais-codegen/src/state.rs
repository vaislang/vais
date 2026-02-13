//! Internal state structures for code generation
//!
//! Defines various state management structures used during code generation.

use std::collections::HashMap;
use vais_ast::Spanned;
use vais_types::ResolvedType;

use crate::types::{ConstInfo, EnumInfo, FunctionInfo, GlobalInfo, StructInfo, UnionInfo};

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
    pub(crate) constants: HashMap<String, ConstInfo>,
    /// Global variable definitions
    pub(crate) globals: HashMap<String, GlobalInfo>,
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
    pub(crate) function_templates: HashMap<String, std::rc::Rc<vais_ast::Function>>,
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
    pub(crate) locals: HashMap<String, crate::types::LocalVar>,
    /// Label counter for unique basic block names
    pub(crate) label_counter: usize,
    /// Stack of loop labels for break/continue
    pub(crate) loop_stack: Vec<crate::types::LoopLabels>,
    /// Stack of deferred expressions per function (LIFO order)
    pub(crate) defer_stack: Vec<Spanned<vais_ast::Expr>>,
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
    pub(crate) closures: HashMap<String, crate::types::ClosureInfo>,
    /// Last generated lambda info (for Let statement to pick up)
    pub(crate) last_lambda_info: Option<crate::types::ClosureInfo>,
    /// Async function state machine counter
    pub(crate) async_state_counter: usize,
    /// Async await points
    pub(crate) async_await_points: Vec<crate::types::AsyncAwaitPoint>,
    /// Current async function info
    pub(crate) current_async_function: Option<crate::types::AsyncFunctionInfo>,
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

/// Information about a function's decreases clause for termination proof
#[derive(Clone)]
pub struct DecreasesInfo {
    /// Storage variable name for the initial decreases value
    pub storage_name: String,
    /// The decreases expression from the attribute (already boxed)
    pub expr: Box<Spanned<vais_ast::Expr>>,
    /// Function name with decreases clause
    pub function_name: String,
}

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
    /// Trait aliases: name -> expanded trait bounds
    pub(crate) trait_aliases: HashMap<String, Vec<String>>,
    /// Trait implementations: (impl_type, trait_name) -> method_impls
    pub(crate) trait_impl_methods: HashMap<(String, String), HashMap<String, String>>,
    /// Resolved function signatures from type checker (for inferred parameter types)
    pub(crate) resolved_function_sigs: HashMap<String, vais_types::FunctionSig>,
    /// Type aliases from type checker (for resolving type alias names in codegen)
    pub(crate) type_aliases: HashMap<String, vais_types::ResolvedType>,
    /// Default parameter expressions: `function_name -> Vec<Option<Box<Spanned<Expr>>>>`
    /// Each element corresponds to a parameter; Some(expr) means it has a default value.
    pub(crate) default_params: HashMap<String, Vec<Option<Box<vais_ast::Spanned<vais_ast::Expr>>>>>,
    /// Drop trait registry: type_name -> drop function IR name.
    /// Populated when `X Type: Drop { F drop(&self) { ... } }` is registered.
    /// Used at scope exit to emit automatic drop calls for local variables.
    pub(crate) drop_registry: HashMap<String, String>,
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
    /// Generic method bodies from impl blocks on generic structs.
    /// Populated during module processing for on-demand specialization.
    /// Key: (struct_name, method_name), Value: Function AST
    pub(crate) generic_method_bodies: HashMap<(String, String), std::rc::Rc<vais_ast::Function>>,
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
    /// Maps variable names bound to futures → the poll function name to call on await.
    /// Populated when `let x := spawn asyncFn(...)` is processed, so that
    /// `x.await` can resolve the correct poll function instead of falling back
    /// to `__sync_spawn__poll`.
    pub(crate) future_poll_fns: HashMap<String, String>,
    /// Set to Some when inside an async poll function body.
    /// Contains (state_struct_name, ret_llvm_type, state_field_ptr)
    /// so that Return statements can properly wrap results.
    pub(crate) async_poll_context: Option<AsyncPollContext>,

    /// Tracks heap allocations (malloc'd pointers) in the current function scope.
    /// At function exit, all tracked pointers are freed automatically.
    /// Each entry is (alloca_name, original_ptr_reg): the entry-block alloca stores the
    /// i8* pointer so it can be loaded from any basic block at cleanup time.
    pub(crate) alloc_tracker: Vec<(String, String)>,

    /// Maps temporary variable names (e.g., "%5", "%t.3") to their resolved types.
    /// Used by downstream passes to emit correct LLVM IR types instead of
    /// falling back to i64 for every temporary. Populated by generate_expr paths
    /// and consumed by store/binary/icmp/call emission to fix width mismatches
    /// and void-call naming issues (R2 IR Type Tracking).
    pub(crate) temp_var_types: HashMap<String, ResolvedType>,

    /// Scope stack for block-scoped drop cleanup.
    /// Each entry is a list of variable names declared in that scope (in declaration order).
    /// When a block exits, variables declared in that scope are dropped in LIFO order.
    /// The outer Vec is a stack of scopes (innermost scope last).
    pub(crate) scope_stack: Vec<Vec<String>>,

    /// Collected alloca instructions to be hoisted to the function entry block.
    ///
    /// LLVM can only optimize alloca instructions that appear in the entry basic block.
    /// Non-entry-block allocas (e.g., inside if/else branches) cause "Instruction does
    /// not dominate all uses" errors when the allocated pointer is referenced from
    /// another basic block.
    ///
    /// During expression/statement codegen, static-size allocas (struct, union, enum,
    /// array literals) are recorded here instead of being emitted inline. After the
    /// full function body is generated, these are spliced into the entry block.
    ///
    /// Each entry is a complete IR line, e.g., `"  %tmp.5 = alloca %MyStruct"`.
    pub(crate) entry_allocas: Vec<String>,

    /// IR code for on-demand generated specialized functions (e.g., Vec$str_push).
    /// Accumulated during method call processing and emitted after the current
    /// function's body in the final IR output.
    pub(crate) pending_specialized_ir: Vec<String>,

    /// Cross-module async poll function declarations needed by await expressions.
    /// Collected during await codegen and emitted at module level (outside functions).
    pub(crate) async_poll_declares: std::collections::HashSet<String>,
}

impl FunctionContext {
    /// Register the resolved type of a named temporary variable.
    ///
    /// Only call this for named temporaries (`%N` format). Constants and
    /// literals do not need registration.
    pub(crate) fn register_temp_type(&mut self, name: &str, ty: ResolvedType) {
        self.temp_var_types.insert(name.to_string(), ty);
    }

    /// Look up the resolved type of a temporary variable.
    pub(crate) fn get_temp_type(&self, name: &str) -> Option<&ResolvedType> {
        self.temp_var_types.get(name)
    }
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
    /// Deduplication cache: string_value -> constant_name
    /// When the same string literal appears multiple times, reuses the existing global constant
    /// instead of creating a new one. Reduces binary size and IR output.
    pub(crate) dedup_cache: HashMap<String, String>,
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

/// Context information when generating code inside an async poll function.
/// Allows Return statements to properly wrap values as poll results.
#[derive(Clone)]
pub struct AsyncPollContext {
    /// The LLVM return type string (e.g., "i64", "i1")
    pub ret_llvm: String,
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

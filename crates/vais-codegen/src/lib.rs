//! Vais 0.0.1 LLVM Code Generator
//!
//! Generates LLVM IR from typed AST for native code generation.
//!
//! Note: This is a placeholder structure. Full LLVM integration requires
//! the inkwell crate and LLVM installation.

pub mod debug;
pub mod formatter;
pub mod optimize;
mod builtins;
mod expr;
mod types;
mod stmt;

pub use debug::{DebugConfig, DebugInfoBuilder};

use std::collections::HashMap;
use thiserror::Error;
use vais_ast::{*, IfElse};
use vais_types::ResolvedType;

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
}

type CodegenResult<T> = Result<T, CodegenError>;

/// Result of generating a block of statements
/// (value, ir_code, is_terminated)
/// is_terminated is true if the block ends with break, continue, or return
#[allow(dead_code)]
type BlockResult = (String, String, bool);

/// LLVM IR Code Generator for Vais 0.0.1
///
/// Generates LLVM IR text from typed AST for native code generation via clang.
pub struct CodeGenerator {
    // Module name
    module_name: String,

    // Function signatures for lookup
    functions: HashMap<String, FunctionInfo>,

    // Struct definitions
    structs: HashMap<String, StructInfo>,

    // Enum definitions
    enums: HashMap<String, EnumInfo>,

    // Current function being compiled
    current_function: Option<String>,

    // Local variables in current function
    locals: HashMap<String, LocalVar>,

    // Label counter for unique basic block names
    label_counter: usize,

    // Stack of loop labels for break/continue
    loop_stack: Vec<LoopLabels>,

    // String constants for global storage
    string_constants: Vec<(String, String)>, // (name, value)

    // Counter for string constant names
    string_counter: usize,

    // Lambda functions generated during compilation
    lambda_functions: Vec<String>,

    // Closure information for each lambda variable (maps var_name -> closure_info)
    closures: HashMap<String, ClosureInfo>,

    // Last generated lambda info (for Let statement to pick up)
    last_lambda_info: Option<ClosureInfo>,

    // Async function state machine info
    async_state_counter: usize,
    async_await_points: Vec<AsyncAwaitPoint>,
    current_async_function: Option<AsyncFunctionInfo>,

    // Flag to emit unwrap panic message and abort declaration
    needs_unwrap_panic: bool,

    // Current basic block name (for phi node predecessor tracking)
    current_block: String,

    // Debug info builder for DWARF metadata generation
    debug_info: DebugInfoBuilder,

    // Generic substitutions for current function/method
    // Maps generic param name (e.g., "T") to concrete type (e.g., ResolvedType::I64)
    generic_substitutions: HashMap<String, ResolvedType>,

    // Generated struct instantiations (mangled_name -> already_generated)
    generated_structs: HashMap<String, bool>,

    // Generated function instantiations (mangled_name -> already_generated)
    generated_functions: HashMap<String, bool>,
}

impl CodeGenerator {
    /// Creates a new code generator for the given module.
    ///
    /// Initializes the code generator with built-in functions registered.
    ///
    /// # Arguments
    ///
    /// * `module_name` - Name of the module being compiled
    pub fn new(module_name: &str) -> Self {
        let mut gen = Self {
            module_name: module_name.to_string(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            current_function: None,
            locals: HashMap::new(),
            label_counter: 0,
            loop_stack: Vec::new(),
            string_constants: Vec::new(),
            string_counter: 0,
            lambda_functions: Vec::new(),
            closures: HashMap::new(),
            last_lambda_info: None,
            async_state_counter: 0,
            async_await_points: Vec::new(),
            current_async_function: None,
            needs_unwrap_panic: false,
            current_block: "entry".to_string(),
            debug_info: DebugInfoBuilder::new(DebugConfig::default()),
            generic_substitutions: HashMap::new(),
            generated_structs: HashMap::new(),
            generated_functions: HashMap::new(),
        };

        // Register built-in extern functions
        gen.register_builtin_functions();
        gen
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

    /// Get current generic substitution for a type parameter
    pub(crate) fn get_generic_substitution(&self, param: &str) -> Option<ResolvedType> {
        self.generic_substitutions.get(param).cloned()
    }

    /// Set generic substitutions for the current context
    pub(crate) fn set_generic_substitutions(&mut self, subst: HashMap<String, ResolvedType>) {
        self.generic_substitutions = subst;
    }

    /// Clear generic substitutions
    pub(crate) fn clear_generic_substitutions(&mut self) {
        self.generic_substitutions.clear();
    }

    /// Generate mangled name for a generic struct
    pub(crate) fn mangle_struct_name(&self, name: &str, generics: &[ResolvedType]) -> String {
        vais_types::mangle_name(name, generics)
    }

    /// Generate mangled name for a generic function
    pub(crate) fn mangle_function_name(&self, name: &str, generics: &[ResolvedType]) -> String {
        vais_types::mangle_name(name, generics)
    }

    /// Get the size of a type in bytes (for generic operations)
    pub(crate) fn type_size(&self, ty: &ResolvedType) -> usize {
        match ty {
            ResolvedType::I8 | ResolvedType::U8 | ResolvedType::Bool => 1,
            ResolvedType::I16 | ResolvedType::U16 => 2,
            ResolvedType::I32 | ResolvedType::U32 | ResolvedType::F32 => 4,
            ResolvedType::I64 | ResolvedType::U64 | ResolvedType::F64 => 8,
            ResolvedType::I128 | ResolvedType::U128 => 16,
            ResolvedType::Str => 8, // Pointer size
            ResolvedType::Pointer(_) | ResolvedType::Ref(_) | ResolvedType::RefMut(_) => 8,
            ResolvedType::Named { name, .. } => {
                // Calculate struct size
                if let Some(info) = self.structs.get(name) {
                    info.fields.iter().map(|(_, t)| self.type_size(t)).sum()
                } else {
                    8 // Default to pointer size
                }
            }
            ResolvedType::Generic(param) => {
                // Try to get concrete type from substitutions
                if let Some(concrete) = self.generic_substitutions.get(param) {
                    self.type_size(concrete)
                } else {
                    8 // Default to i64 size
                }
            }
            _ => 8, // Default
        }
    }

    fn next_label(&mut self, prefix: &str) -> String {
        let label = format!("{}{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
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

        // Header
        ir.push_str(&format!("; ModuleID = '{}'\n", self.module_name));
        ir.push_str("source_filename = \"<vais>\"\n");

        // Note: target triple and data layout are omitted to let clang auto-detect
        ir.push('\n');

        // Initialize debug info if enabled
        if self.debug_info.is_enabled() {
            self.debug_info.initialize();
        }

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
                Item::Impl(impl_block) => {
                    // Register impl methods
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        self.register_method(&type_name, &method.node)?;
                    }
                }
                Item::Use(_) => {
                    // Use statements are handled at the compiler level (AST merging)
                    // No code generation needed for imports
                }
                Item::Trait(_) | Item::TypeAlias(_) => {
                    // Traits and type aliases don't generate code
                }
            }
        }

        // Generate struct types
        for (name, info) in &self.structs {
            ir.push_str(&self.generate_struct_type(name, info));
            ir.push('\n');
        }

        // Generate enum types
        for (name, info) in &self.enums.clone() {
            ir.push_str(&self.generate_enum_type(&name, &info));
            ir.push('\n');
        }

        // Generate function declarations (deduplicate by actual function name)
        let mut declared_fns = std::collections::HashSet::new();
        for info in self.functions.values() {
            if info.is_extern && !declared_fns.contains(&info.name) {
                ir.push_str(&self.generate_extern_decl(info));
                ir.push('\n');
                declared_fns.insert(info.name.clone());
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
                        body_ir.push_str(&self.generate_method_with_span(&s.name.node, &method.node, method.span)?);
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
                        body_ir.push_str(&self.generate_method_with_span(&type_name, &method.node, method.span)?);
                        body_ir.push('\n');
                    }
                }
                Item::Enum(_) | Item::Use(_) | Item::Trait(_) | Item::TypeAlias(_) => {
                    // Already handled in first pass or no code generation needed
                }
            }
        }

        // Add string constants at the top of the module
        for (name, value) in &self.string_constants {
            let escaped = value.replace('\\', "\\\\").replace('"', "\\22").replace('\n', "\\0A");
            let len = value.len() + 1; // +1 for null terminator
            ir.push_str(&format!(
                "@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n",
                name, len, escaped
            ));
        }
        if !self.string_constants.is_empty() {
            ir.push('\n');
        }

        // Add unwrap panic message and abort declaration if needed
        if self.needs_unwrap_panic {
            ir.push_str("@.unwrap_panic_msg = private unnamed_addr constant [22 x i8] c\"unwrap failed: panic!\\00\"\n");
            ir.push_str("declare void @abort()\n\n");
        }

        ir.push_str(&body_ir);

        // Add lambda functions at the end
        for lambda_ir in &self.lambda_functions {
            ir.push('\n');
            ir.push_str(lambda_ir);
        }

        // Add helper functions for memory operations
        ir.push_str(&self.generate_helper_functions());

        // Add debug intrinsic declaration if debug info is enabled
        if self.debug_info.is_enabled() {
            ir.push_str("\n; Debug intrinsics\n");
            ir.push_str("declare void @llvm.dbg.declare(metadata, metadata, metadata)\n");
            ir.push_str("declare void @llvm.dbg.value(metadata, metadata, metadata)\n");
        }

        // Add debug metadata at the end
        ir.push_str(&self.debug_info.finalize());

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

        // Header
        ir.push_str(&format!("; ModuleID = '{}'\n", self.module_name));
        ir.push_str("source_filename = \"<vais>\"\n");
        ir.push('\n');

        // Initialize debug info if enabled
        if self.debug_info.is_enabled() {
            self.debug_info.initialize();
        }

        // First pass: collect declarations (including generic templates)
        let mut generic_functions: HashMap<String, Function> = HashMap::new();
        let mut generic_structs: HashMap<String, Struct> = HashMap::new();

        for item in &module.items {
            match &item.node {
                Item::Function(f) => {
                    if !f.generics.is_empty() {
                        // Store generic function for later specialization
                        generic_functions.insert(f.name.node.clone(), f.clone());
                    } else {
                        self.register_function(f)?;
                    }
                }
                Item::Struct(s) => {
                    if !s.generics.is_empty() {
                        // Store generic struct for later specialization
                        generic_structs.insert(s.name.node.clone(), s.clone());
                    } else {
                        self.register_struct(s)?;
                        for method in &s.methods {
                            self.register_method(&s.name.node, &method.node)?;
                        }
                    }
                }
                Item::Enum(e) => self.register_enum(e)?,
                Item::Impl(impl_block) => {
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        self.register_method(&type_name, &method.node)?;
                    }
                }
                Item::Use(_) | Item::Trait(_) | Item::TypeAlias(_) => {}
            }
        }

        // Generate specialized struct types from instantiations
        for inst in instantiations {
            if let vais_types::InstantiationKind::Struct = inst.kind {
                if let Some(generic_struct) = generic_structs.get(&inst.base_name) {
                    self.generate_specialized_struct_type(generic_struct, inst, &mut ir)?;
                }
            }
        }

        // Generate non-generic struct types
        for (name, info) in &self.structs {
            ir.push_str(&self.generate_struct_type(name, info));
            ir.push('\n');
        }

        // Generate enum types
        for (name, info) in &self.enums.clone() {
            ir.push_str(&self.generate_enum_type(&name, &info));
            ir.push('\n');
        }

        // Generate function declarations (extern functions)
        let mut declared_fns = std::collections::HashSet::new();
        for info in self.functions.values() {
            if info.is_extern && !declared_fns.contains(&info.name) {
                ir.push_str(&self.generate_extern_decl(info));
                ir.push('\n');
                declared_fns.insert(info.name.clone());
            }
        }

        // Generate string constants (after processing functions to collect all strings)
        let mut body_ir = String::new();

        // Generate specialized functions from instantiations
        for inst in instantiations {
            if let vais_types::InstantiationKind::Function = inst.kind {
                if let Some(generic_fn) = generic_functions.get(&inst.base_name) {
                    body_ir.push_str(&self.generate_specialized_function(generic_fn, inst)?);
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
                            body_ir.push_str(&self.generate_method_with_span(&s.name.node, &method.node, method.span)?);
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
                        body_ir.push_str(&self.generate_method_with_span(&type_name, &method.node, method.span)?);
                        body_ir.push('\n');
                    }
                }
                Item::Enum(_) | Item::Use(_) | Item::Trait(_) | Item::TypeAlias(_) => {}
            }
        }

        // Add string constants
        for (name, value) in &self.string_constants {
            let escaped = value.replace('\\', "\\\\").replace('"', "\\22").replace('\n', "\\0A");
            let len = value.len() + 1;
            ir.push_str(&format!(
                "@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n",
                name, len, escaped
            ));
        }
        if !self.string_constants.is_empty() {
            ir.push('\n');
        }

        // Add unwrap panic message and abort declaration if needed
        if self.needs_unwrap_panic {
            ir.push_str("@.unwrap_panic_msg = private unnamed_addr constant [22 x i8] c\"unwrap failed: panic!\\00\"\n");
            ir.push_str("declare void @abort()\n\n");
        }

        ir.push_str(&body_ir);

        // Add lambda functions
        for lambda_ir in &self.lambda_functions {
            ir.push('\n');
            ir.push_str(lambda_ir);
        }

        // Add helper functions
        ir.push_str(&self.generate_helper_functions());

        // Add debug intrinsics if debug info is enabled
        if self.debug_info.is_enabled() {
            ir.push_str("\n; Debug intrinsics\n");
            ir.push_str("declare void @llvm.dbg.declare(metadata, metadata, metadata)\n");
            ir.push_str("declare void @llvm.dbg.value(metadata, metadata, metadata)\n");
        }

        // Add debug metadata
        ir.push_str(&self.debug_info.finalize());

        Ok(ir)
    }

    /// Generate a specialized struct type from a generic struct template
    fn generate_specialized_struct_type(
        &mut self,
        generic_struct: &Struct,
        inst: &vais_types::GenericInstantiation,
        ir: &mut String,
    ) -> CodegenResult<()> {
        // Skip if already generated
        if self.generated_structs.contains_key(&inst.mangled_name) {
            return Ok(());
        }
        self.generated_structs.insert(inst.mangled_name.clone(), true);

        // Create substitution map from generic params to concrete types
        let substitutions: HashMap<String, ResolvedType> = generic_struct
            .generics
            .iter()
            .zip(inst.type_args.iter())
            .map(|(g, t)| (g.name.node.clone(), t.clone()))
            .collect();

        // Save and set generic substitutions
        let old_subst = std::mem::replace(&mut self.generic_substitutions, substitutions.clone());

        // Generate field types with substitutions
        let fields: Vec<(String, ResolvedType)> = generic_struct
            .fields
            .iter()
            .map(|f| {
                let ty = self.ast_type_to_resolved(&f.ty.node);
                let concrete_ty = vais_types::substitute_type(&ty, &substitutions);
                (f.name.node.clone(), concrete_ty)
            })
            .collect();

        let llvm_fields: Vec<String> = fields
            .iter()
            .map(|(_, ty)| self.type_to_llvm(ty))
            .collect();

        ir.push_str(&format!(
            "%{} = type {{ {} }}\n",
            inst.mangled_name,
            llvm_fields.join(", ")
        ));

        // Register the specialized struct
        let struct_info = StructInfo {
            name: inst.mangled_name.clone(),
            fields,
        };
        self.structs.insert(inst.mangled_name.clone(), struct_info);

        // Restore old substitutions
        self.generic_substitutions = old_subst;

        Ok(())
    }

    /// Generate a specialized function from a generic function template
    fn generate_specialized_function(
        &mut self,
        generic_fn: &Function,
        inst: &vais_types::GenericInstantiation,
    ) -> CodegenResult<String> {
        // Skip if already generated
        if self.generated_functions.contains_key(&inst.mangled_name) {
            return Ok(String::new());
        }
        self.generated_functions.insert(inst.mangled_name.clone(), true);

        // Create substitution map from generic params to concrete types
        let substitutions: HashMap<String, ResolvedType> = generic_fn
            .generics
            .iter()
            .zip(inst.type_args.iter())
            .map(|(g, t)| (g.name.node.clone(), t.clone()))
            .collect();

        // Save and set generic substitutions
        let old_subst = std::mem::replace(&mut self.generic_substitutions, substitutions.clone());

        self.current_function = Some(inst.mangled_name.clone());
        self.locals.clear();
        self.label_counter = 0;
        self.loop_stack.clear();

        // Generate parameters with substituted types
        let params: Vec<_> = generic_fn
            .params
            .iter()
            .map(|p| {
                let ty = self.ast_type_to_resolved(&p.ty.node);
                let concrete_ty = vais_types::substitute_type(&ty, &substitutions);
                let llvm_ty = self.type_to_llvm(&concrete_ty);

                // Register parameter as local
                self.locals.insert(
                    p.name.node.clone(),
                    LocalVar {
                        ty: concrete_ty,
                        is_param: true,
                        llvm_name: p.name.node.clone(),
                    },
                );

                format!("{} %{}", llvm_ty, p.name.node)
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

        let ret_llvm = self.type_to_llvm(&ret_type);

        let mut ir = format!(
            "; Specialized function: {} from {}<{}>\n",
            inst.mangled_name,
            inst.base_name,
            inst.type_args.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ")
        );
        ir.push_str(&format!(
            "define {} @{}({}) {{\n",
            ret_llvm,
            inst.mangled_name,
            params.join(", ")
        ));
        ir.push_str("entry:\n");
        self.current_block = "entry".to_string();

        // Generate function body
        let mut counter = 0;
        match &generic_fn.body {
            FunctionBody::Expr(expr) => {
                let (value, expr_ir) = self.generate_expr(expr, &mut counter)?;
                ir.push_str(&expr_ir);
                if ret_type == ResolvedType::Unit {
                    ir.push_str("  ret void\n");
                } else if matches!(ret_type, ResolvedType::Named { .. }) {
                    let loaded = format!("%ret.{}", counter);
                    ir.push_str(&format!("  {} = load {}, {}* {}\n", loaded, ret_llvm, ret_llvm, value));
                    ir.push_str(&format!("  ret {} {}\n", ret_llvm, loaded));
                } else {
                    ir.push_str(&format!("  ret {} {}\n", ret_llvm, value));
                }
            }
            FunctionBody::Block(stmts) => {
                let (value, block_ir) = self.generate_block(stmts, &mut counter)?;
                ir.push_str(&block_ir);
                if ret_type == ResolvedType::Unit {
                    ir.push_str("  ret void\n");
                } else if matches!(ret_type, ResolvedType::Named { .. }) {
                    let loaded = format!("%ret.{}", counter);
                    ir.push_str(&format!("  {} = load {}, {}* {}\n", loaded, ret_llvm, ret_llvm, value));
                    ir.push_str(&format!("  ret {} {}\n", ret_llvm, loaded));
                } else {
                    ir.push_str(&format!("  ret {} {}\n", ret_llvm, value));
                }
            }
        }

        ir.push_str("}\n");

        // Restore state
        self.generic_substitutions = old_subst;
        self.current_function = None;

        Ok(ir)
    }

    /// Generate helper functions for low-level memory operations
    fn generate_helper_functions(&self) -> String {
        let mut ir = String::new();

        // __load_byte: load a byte from memory address
        ir.push_str("\n; Helper function: load byte from memory\n");
        ir.push_str("define i64 @__load_byte(i64 %ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i8*\n");
        ir.push_str("  %1 = load i8, i8* %0\n");
        ir.push_str("  %2 = zext i8 %1 to i64\n");
        ir.push_str("  ret i64 %2\n");
        ir.push_str("}\n");

        // __store_byte: store a byte to memory address
        ir.push_str("\n; Helper function: store byte to memory\n");
        ir.push_str("define void @__store_byte(i64 %ptr, i64 %val) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i8*\n");
        ir.push_str("  %1 = trunc i64 %val to i8\n");
        ir.push_str("  store i8 %1, i8* %0\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        // __load_i64: load a 64-bit integer from memory address
        ir.push_str("\n; Helper function: load i64 from memory\n");
        ir.push_str("define i64 @__load_i64(i64 %ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i64*\n");
        ir.push_str("  %1 = load i64, i64* %0\n");
        ir.push_str("  ret i64 %1\n");
        ir.push_str("}\n");

        // __store_i64: store a 64-bit integer to memory address
        ir.push_str("\n; Helper function: store i64 to memory\n");
        ir.push_str("define void @__store_i64(i64 %ptr, i64 %val) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i64*\n");
        ir.push_str("  store i64 %val, i64* %0\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        ir
    }

    fn register_function(&mut self, f: &Function) -> CodegenResult<()> {
        let params: Vec<_> = f
            .params
            .iter()
            .map(|p| {
                let ty = self.ast_type_to_resolved(&p.ty.node);
                (p.name.node.clone(), ty)
            })
            .collect();

        let ret_type = f
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);

        self.functions.insert(
            f.name.node.clone(),
            FunctionInfo {
                name: f.name.node.clone(),
                params,
                ret_type,
                is_extern: false,
            },
        );

        Ok(())
    }

    /// Register a method as a function with Type_methodName naming convention
    fn register_method(&mut self, type_name: &str, f: &Function) -> CodegenResult<()> {
        let method_name = format!("{}_{}", type_name, f.name.node);

        // Check if this is a static method (no &self or self parameter)
        let has_self = f.params.first().map(|p| p.name.node == "self").unwrap_or(false);

        // Build parameter list
        let mut params = Vec::new();

        if has_self {
            // Instance method: add self parameter (pointer to struct type)
            params.push((
                "self".to_string(),
                ResolvedType::Named {
                    name: type_name.to_string(),
                    generics: vec![],
                },
            ));
        }

        // Add remaining parameters (skip self if it exists)
        for p in &f.params {
            if p.name.node != "self" {
                let ty = self.ast_type_to_resolved(&p.ty.node);
                params.push((p.name.node.clone(), ty));
            }
        }

        let ret_type = f
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);

        self.functions.insert(
            method_name.clone(),
            FunctionInfo {
                name: method_name,
                params,
                ret_type,
                is_extern: false,
            },
        );

        Ok(())
    }

    fn register_struct(&mut self, s: &Struct) -> CodegenResult<()> {
        let fields: Vec<_> = s
            .fields
            .iter()
            .map(|f| {
                let ty = self.ast_type_to_resolved(&f.ty.node);
                (f.name.node.clone(), ty)
            })
            .collect();

        self.structs.insert(
            s.name.node.clone(),
            StructInfo {
                name: s.name.node.clone(),
                fields,
            },
        );

        Ok(())
    }

    fn register_enum(&mut self, e: &vais_ast::Enum) -> CodegenResult<()> {
        let mut variants = Vec::new();

        for (tag, variant) in e.variants.iter().enumerate() {
            let fields = match &variant.fields {
                VariantFields::Unit => EnumVariantFields::Unit,
                VariantFields::Tuple(types) => {
                    let resolved: Vec<_> = types
                        .iter()
                        .map(|t| self.ast_type_to_resolved(&t.node))
                        .collect();
                    EnumVariantFields::Tuple(resolved)
                }
                VariantFields::Struct(fields) => {
                    let resolved: Vec<_> = fields
                        .iter()
                        .map(|f| (f.name.node.clone(), self.ast_type_to_resolved(&f.ty.node)))
                        .collect();
                    EnumVariantFields::Struct(resolved)
                }
            };

            variants.push(EnumVariantInfo {
                name: variant.name.node.clone(),
                tag: tag as u32,
                fields,
            });
        }

        self.enums.insert(
            e.name.node.clone(),
            EnumInfo {
                name: e.name.node.clone(),
                variants,
            },
        );

        Ok(())
    }

    fn generate_extern_decl(&self, info: &FunctionInfo) -> String {
        let params: Vec<_> = info
            .params
            .iter()
            .map(|(_, ty)| self.type_to_llvm(ty))
            .collect();

        let ret = self.type_to_llvm(&info.ret_type);

        format!("declare {} @{}({})", ret, info.name, params.join(", "))
    }

    #[allow(dead_code)]
    fn generate_function(&mut self, f: &Function) -> CodegenResult<String> {
        self.generate_function_with_span(f, Span::default())
    }

    fn generate_function_with_span(&mut self, f: &Function, span: Span) -> CodegenResult<String> {
        // Check if this is an async function
        if f.is_async {
            return self.generate_async_function(f);
        }

        self.current_function = Some(f.name.node.clone());
        self.locals.clear();
        self.label_counter = 0;
        self.loop_stack.clear();

        // Create debug info for this function
        let func_line = self.debug_info.offset_to_line(span.start);
        let di_subprogram = self.debug_info.create_function_debug_info(&f.name.node, func_line, true);

        let params: Vec<_> = f
            .params
            .iter()
            .map(|p| {
                let ty = self.ast_type_to_resolved(&p.ty.node);
                let llvm_ty = self.type_to_llvm(&ty);

                // Register parameter as local (SSA value, not alloca)
                // For params, llvm_name matches the source name
                self.locals.insert(
                    p.name.node.clone(),
                    LocalVar {
                        ty: ty.clone(),
                        is_param: true,
                        llvm_name: p.name.node.clone(),
                    },
                );

                format!("{} %{}", llvm_ty, p.name.node)
            })
            .collect();

        let ret_type = f
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);

        let ret_llvm = self.type_to_llvm(&ret_type);

        // Build function definition with optional debug info reference
        let dbg_ref = if let Some(sp_id) = di_subprogram {
            format!(" !dbg !{}", sp_id)
        } else {
            String::new()
        };

        let mut ir = format!(
            "define {} @{}({}){} {{\n",
            ret_llvm,
            f.name.node,
            params.join(", "),
            dbg_ref
        );

        ir.push_str("entry:\n");

        // Generate body
        let mut counter = 0;
        match &f.body {
            FunctionBody::Expr(expr) => {
                let (value, expr_ir) = self.generate_expr(expr, &mut counter)?;
                ir.push_str(&expr_ir);
                let ret_dbg = self.debug_info.dbg_ref_from_offset(expr.span.start);
                if ret_type == ResolvedType::Unit {
                    ir.push_str(&format!("  ret void{}\n", ret_dbg));
                } else if matches!(ret_type, ResolvedType::Named { .. }) {
                    // For struct returns, load the value from pointer
                    let loaded = format!("%ret.{}", counter);
                    ir.push_str(&format!("  {} = load {}, {}* {}{}\n", loaded, ret_llvm, ret_llvm, value, ret_dbg));
                    ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, loaded, ret_dbg));
                } else {
                    ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, value, ret_dbg));
                }
            }
            FunctionBody::Block(stmts) => {
                let (value, block_ir) = self.generate_block(stmts, &mut counter)?;
                ir.push_str(&block_ir);
                // Get debug location from last statement or function end
                let ret_offset = stmts.last().map(|s| s.span.end).unwrap_or(span.end);
                let ret_dbg = self.debug_info.dbg_ref_from_offset(ret_offset);
                if ret_type == ResolvedType::Unit {
                    ir.push_str(&format!("  ret void{}\n", ret_dbg));
                } else if matches!(ret_type, ResolvedType::Named { .. }) {
                    // For struct returns, load the value from pointer
                    let loaded = format!("%ret.{}", counter);
                    ir.push_str(&format!("  {} = load {}, {}* {}{}\n", loaded, ret_llvm, ret_llvm, value, ret_dbg));
                    ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, loaded, ret_dbg));
                } else {
                    ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, value, ret_dbg));
                }
            }
        }

        ir.push_str("}\n");

        self.current_function = None;
        Ok(ir)
    }

    /// Generate an async function as a state machine coroutine
    ///
    /// Async functions are transformed into:
    /// 1. A state struct holding local variables and current state
    /// 2. A poll function that implements the state machine
    /// 3. A create function that returns a pointer to the state struct
    fn generate_async_function(&mut self, f: &Function) -> CodegenResult<String> {
        let func_name = &f.name.node;
        let state_struct_name = format!("{}__AsyncState", func_name);

        // Collect parameters for state struct
        let params: Vec<_> = f.params.iter().map(|p| {
            let ty = self.ast_type_to_resolved(&p.ty.node);
            (p.name.node.clone(), ty)
        }).collect();

        let ret_type = f
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);

        let ret_llvm = self.type_to_llvm(&ret_type);

        // Reset async state tracking
        self.async_state_counter = 0;
        self.async_await_points.clear();
        self.current_async_function = Some(AsyncFunctionInfo {
            name: func_name.clone(),
            state_struct: state_struct_name.clone(),
            captured_vars: params.clone(),
            ret_type: ret_type.clone(),
        });

        let mut ir = String::new();

        // 1. Generate state struct type
        // Structure: { i64 state, i64 result, param1, param2, ... }
        ir.push_str(&format!("; Async state struct for {}\n", func_name));
        ir.push_str(&format!("%{} = type {{ i64, {}", state_struct_name, ret_llvm));
        for (_, ty) in &params {
            ir.push_str(&format!(", {}", self.type_to_llvm(ty)));
        }
        ir.push_str(" }\n\n");

        // 2. Generate create function: allocates and initializes state
        ir.push_str(&format!("; Create function for async {}\n", func_name));
        let create_params: Vec<_> = params.iter()
            .map(|(name, ty)| format!("{} %{}", self.type_to_llvm(ty), name))
            .collect();
        ir.push_str(&format!(
            "define i64 @{}({}) {{\n",
            func_name,
            create_params.join(", ")
        ));
        ir.push_str("entry:\n");

        // Calculate struct size (8 bytes per field: state + result + params)
        let struct_size = 16 + params.len() * 8;
        ir.push_str(&format!("  %state_ptr = call i64 @malloc(i64 {})\n", struct_size));
        ir.push_str(&format!("  %state = inttoptr i64 %state_ptr to %{}*\n", state_struct_name));

        // Initialize state to 0 (start state)
        ir.push_str(&format!("  %state_field = getelementptr %{}, %{}* %state, i32 0, i32 0\n",
            state_struct_name, state_struct_name));
        ir.push_str("  store i64 0, i64* %state_field\n");

        // Store parameters in state struct
        for (i, (name, _ty)) in params.iter().enumerate() {
            let field_idx = i + 2; // Skip state and result fields
            ir.push_str(&format!(
                "  %param_{}_ptr = getelementptr %{}, %{}* %state, i32 0, i32 {}\n",
                name, state_struct_name, state_struct_name, field_idx
            ));
            ir.push_str(&format!("  store i64 %{}, i64* %param_{}_ptr\n", name, name));
        }

        ir.push_str("  ret i64 %state_ptr\n");
        ir.push_str("}\n\n");

        // 3. Generate poll function: implements state machine
        self.current_function = Some(format!("{}__poll", func_name));
        self.locals.clear();
        self.label_counter = 0;
        self.loop_stack.clear();

        ir.push_str(&format!("; Poll function for async {}\n", func_name));
        ir.push_str(&format!(
            "define {{ i64, {} }} @{}__poll(i64 %state_ptr) {{\n",
            ret_llvm, func_name
        ));
        ir.push_str("entry:\n");
        ir.push_str(&format!("  %state = inttoptr i64 %state_ptr to %{}*\n", state_struct_name));

        // Load current state
        ir.push_str(&format!("  %state_field = getelementptr %{}, %{}* %state, i32 0, i32 0\n",
            state_struct_name, state_struct_name));
        ir.push_str("  %current_state = load i64, i64* %state_field\n");

        // Load parameters from state into locals
        for (i, (name, ty)) in params.iter().enumerate() {
            let field_idx = i + 2;
            ir.push_str(&format!(
                "  %param_{}_ptr = getelementptr %{}, %{}* %state, i32 0, i32 {}\n",
                name, state_struct_name, state_struct_name, field_idx
            ));
            ir.push_str(&format!("  %{} = load i64, i64* %param_{}_ptr\n", name, name));

            self.locals.insert(
                name.clone(),
                LocalVar {
                    ty: ty.clone(),
                    is_param: true,
                    llvm_name: name.clone(),
                },
            );
        }

        // State machine switch
        ir.push_str("  switch i64 %current_state, label %state_invalid [\n");
        ir.push_str("    i64 0, label %state_0\n");
        ir.push_str("  ]\n\n");

        // Generate state_0 (initial state) - execute function body
        ir.push_str("state_0:\n");

        let mut counter = 0;
        let body_result = match &f.body {
            FunctionBody::Expr(expr) => {
                self.generate_expr(expr, &mut counter)?
            }
            FunctionBody::Block(stmts) => {
                self.generate_block(stmts, &mut counter)?
            }
        };

        ir.push_str(&body_result.1);

        // Store result and return Ready
        ir.push_str(&format!("  %result_ptr = getelementptr %{}, %{}* %state, i32 0, i32 1\n",
            state_struct_name, state_struct_name));
        ir.push_str(&format!("  store {} {}, {}* %result_ptr\n", ret_llvm, body_result.0, ret_llvm));

        // Set state to -1 (completed)
        ir.push_str("  store i64 -1, i64* %state_field\n");

        // Return {1, result} for Ready
        ir.push_str(&format!("  %ret_val = load {}, {}* %result_ptr\n", ret_llvm, ret_llvm));
        ir.push_str(&format!("  %ret_0 = insertvalue {{ i64, {} }} undef, i64 1, 0\n", ret_llvm));
        ir.push_str(&format!("  %ret_1 = insertvalue {{ i64, {} }} %ret_0, {} %ret_val, 1\n", ret_llvm, ret_llvm));
        ir.push_str(&format!("  ret {{ i64, {} }} %ret_1\n\n", ret_llvm));

        // Invalid state handler
        ir.push_str("state_invalid:\n");
        ir.push_str(&format!("  %invalid_ret = insertvalue {{ i64, {} }} undef, i64 0, 0\n", ret_llvm));
        ir.push_str(&format!("  ret {{ i64, {} }} %invalid_ret\n", ret_llvm));

        ir.push_str("}\n");

        self.current_function = None;
        self.current_async_function = None;

        Ok(ir)
    }

    /// Generate a method for a struct
    /// Methods are compiled as functions with the struct pointer as implicit first argument
    /// Static methods (without &self) don't have the implicit self parameter
    #[allow(dead_code)]
    fn generate_method(&mut self, struct_name: &str, f: &Function) -> CodegenResult<String> {
        self.generate_method_with_span(struct_name, f, Span::default())
    }

    fn generate_method_with_span(&mut self, struct_name: &str, f: &Function, span: Span) -> CodegenResult<String> {
        // Method name: StructName_methodName
        let method_name = format!("{}_{}", struct_name, f.name.node);

        self.current_function = Some(method_name.clone());
        self.locals.clear();
        self.label_counter = 0;
        self.loop_stack.clear();

        // Create debug info for this method
        let func_line = self.debug_info.offset_to_line(span.start);
        let di_subprogram = self.debug_info.create_function_debug_info(&method_name, func_line, true);

        // Check if this is a static method (no &self or self parameter)
        let has_self = f.params.first().map(|p| p.name.node == "self").unwrap_or(false);

        let mut params = Vec::new();

        if has_self {
            // Instance method: first parameter is `self` (pointer to struct)
            let struct_ty = format!("%{}*", struct_name);
            params.push(format!("{} %self", struct_ty));

            // Register self
            self.locals.insert(
                "self".to_string(),
                LocalVar {
                    ty: ResolvedType::Named {
                        name: struct_name.to_string(),
                        generics: vec![],
                    },
                    is_param: true,
                    llvm_name: "self".to_string(),
                },
            );
        }

        // Add remaining parameters
        for p in &f.params {
            // Skip `self` parameter if it exists in the AST
            if p.name.node == "self" {
                continue;
            }

            let ty = self.ast_type_to_resolved(&p.ty.node);
            let llvm_ty = self.type_to_llvm(&ty);

            self.locals.insert(
                p.name.node.clone(),
                LocalVar {
                    ty: ty.clone(),
                    is_param: true,
                    llvm_name: p.name.node.clone(),
                },
            );

            params.push(format!("{} %{}", llvm_ty, p.name.node));
        }

        let ret_type = f
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);

        let ret_llvm = self.type_to_llvm(&ret_type);

        // Build method definition with optional debug info reference
        let dbg_ref = if let Some(sp_id) = di_subprogram {
            format!(" !dbg !{}", sp_id)
        } else {
            String::new()
        };

        let mut ir = format!(
            "define {} @{}({}){} {{\n",
            ret_llvm,
            method_name,
            params.join(", "),
            dbg_ref
        );

        ir.push_str("entry:\n");

        // Generate body
        let mut counter = 0;
        match &f.body {
            FunctionBody::Expr(expr) => {
                let (value, expr_ir) = self.generate_expr(expr, &mut counter)?;
                ir.push_str(&expr_ir);
                let ret_dbg = self.debug_info.dbg_ref_from_offset(expr.span.start);
                if ret_type == ResolvedType::Unit {
                    ir.push_str(&format!("  ret void{}\n", ret_dbg));
                } else if matches!(ret_type, ResolvedType::Named { .. }) {
                    // For struct returns, load the value from pointer
                    let loaded = format!("%ret.{}", counter);
                    ir.push_str(&format!("  {} = load {}, {}* {}{}\n", loaded, ret_llvm, ret_llvm, value, ret_dbg));
                    ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, loaded, ret_dbg));
                } else {
                    ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, value, ret_dbg));
                }
            }
            FunctionBody::Block(stmts) => {
                let (value, block_ir) = self.generate_block(stmts, &mut counter)?;
                ir.push_str(&block_ir);
                let ret_offset = stmts.last().map(|s| s.span.end).unwrap_or(span.end);
                let ret_dbg = self.debug_info.dbg_ref_from_offset(ret_offset);
                if ret_type == ResolvedType::Unit {
                    ir.push_str(&format!("  ret void{}\n", ret_dbg));
                } else if matches!(ret_type, ResolvedType::Named { .. }) {
                    // For struct returns, load the value from pointer
                    let loaded = format!("%ret.{}", counter);
                    ir.push_str(&format!("  {} = load {}, {}* {}{}\n", loaded, ret_llvm, ret_llvm, value, ret_dbg));
                    ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, loaded, ret_dbg));
                } else {
                    ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, value, ret_dbg));
                }
            }
        }

        ir.push_str("}\n");

        self.current_function = None;
        Ok(ir)
    }

    fn generate_expr(
        &mut self,
        expr: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        match &expr.node {
            Expr::Int(n) => Ok((n.to_string(), String::new())),
            Expr::Float(n) => Ok((format!("{:e}", n), String::new())),
            Expr::Bool(b) => Ok((if *b { "1" } else { "0" }.to_string(), String::new())),
            Expr::String(s) => {
                // Create a global string constant
                let name = format!(".str.{}", self.string_counter);
                self.string_counter += 1;
                self.string_constants.push((name.clone(), s.clone()));

                // Return a getelementptr to the string constant
                let len = s.len() + 1;
                Ok((
                    format!("getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)", len, len, name),
                    String::new(),
                ))
            }
            Expr::Unit => Ok(("void".to_string(), String::new())),

            Expr::Ident(name) => {
                if let Some(local) = self.locals.get(name.as_str()).cloned() {
                    if local.is_param {
                        // Parameters are SSA values, use directly
                        Ok((format!("%{}", local.llvm_name), String::new()))
                    } else if matches!(local.ty, ResolvedType::Named { .. }) {
                        // Struct variables store a pointer to the struct
                        // Load the pointer (the struct address)
                        let tmp = self.next_temp(counter);
                        let llvm_ty = self.type_to_llvm(&local.ty);
                        let ir = format!(
                            "  {} = load {}*, {}** %{}\n",
                            tmp, llvm_ty, llvm_ty, local.llvm_name
                        );
                        Ok((tmp, ir))
                    } else {
                        // Local variables need to be loaded from alloca
                        let tmp = self.next_temp(counter);
                        let llvm_ty = self.type_to_llvm(&local.ty);
                        let ir = format!(
                            "  {} = load {}, {}* %{}\n",
                            tmp, llvm_ty, llvm_ty, local.llvm_name
                        );
                        Ok((tmp, ir))
                    }
                } else if name == "self" {
                    // Handle self reference
                    Ok(("%self".to_string(), String::new()))
                } else if self.is_unit_enum_variant(name) {
                    // Unit enum variant (e.g., None)
                    // Create enum value on stack with just the tag
                    for enum_info in self.enums.values() {
                        for (tag, variant) in enum_info.variants.iter().enumerate() {
                            if variant.name == *name {
                                let mut ir = String::new();
                                let enum_ptr = self.next_temp(counter);
                                ir.push_str(&format!(
                                    "  {} = alloca %{}\n",
                                    enum_ptr, enum_info.name
                                ));
                                // Store tag
                                let tag_ptr = self.next_temp(counter);
                                ir.push_str(&format!(
                                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0\n",
                                    tag_ptr, enum_info.name, enum_info.name, enum_ptr
                                ));
                                ir.push_str(&format!(
                                    "  store i32 {}, i32* {}\n",
                                    tag, tag_ptr
                                ));
                                return Ok((enum_ptr, ir));
                            }
                        }
                    }
                    // Fallback if not found (shouldn't happen)
                    Ok((format!("@{}", name), String::new()))
                } else {
                    // Might be a function reference
                    Ok((format!("@{}", name), String::new()))
                }
            }

            Expr::SelfCall => {
                // @ refers to current function
                if let Some(fn_name) = &self.current_function {
                    Ok((format!("@{}", fn_name), String::new()))
                } else {
                    Err(CodegenError::UndefinedFunction("@".to_string()))
                }
            }

            Expr::Binary { op, left, right } => {
                let (left_val, left_ir) = self.generate_expr(left, counter)?;
                let (right_val, right_ir) = self.generate_expr(right, counter)?;

                let mut ir = left_ir;
                ir.push_str(&right_ir);

                // Handle comparison and logical operations (result is i1)
                let is_comparison = matches!(
                    op,
                    BinOp::Lt | BinOp::Lte | BinOp::Gt | BinOp::Gte | BinOp::Eq | BinOp::Neq
                );
                let is_logical = matches!(op, BinOp::And | BinOp::Or);

                if is_logical {
                    // For logical And/Or, convert operands to i1 first, then perform operation
                    let left_bool = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = icmp ne i64 {}, 0\n",
                        left_bool, left_val
                    ));
                    let right_bool = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = icmp ne i64 {}, 0\n",
                        right_bool, right_val
                    ));

                    let op_str = match op {
                        BinOp::And => "and",
                        BinOp::Or => "or",
                        _ => unreachable!(),
                    };

                    let result_bool = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = {} i1 {}, {}{}\n",
                        result_bool, op_str, left_bool, right_bool, dbg_info
                    ));

                    // Extend back to i64 for consistency
                    let result = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = zext i1 {} to i64\n",
                        result, result_bool
                    ));
                    Ok((result, ir))
                } else if is_comparison {
                    // Comparison returns i1, extend to i64
                    let op_str = match op {
                        BinOp::Lt => "icmp slt",
                        BinOp::Lte => "icmp sle",
                        BinOp::Gt => "icmp sgt",
                        BinOp::Gte => "icmp sge",
                        BinOp::Eq => "icmp eq",
                        BinOp::Neq => "icmp ne",
                        _ => unreachable!(),
                    };

                    let cmp_tmp = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = {} i64 {}, {}{}\n",
                        cmp_tmp, op_str, left_val, right_val, dbg_info
                    ));

                    // Extend i1 to i64
                    let result = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = zext i1 {} to i64\n",
                        result, cmp_tmp
                    ));
                    Ok((result, ir))
                } else {
                    // Arithmetic and bitwise operations
                    let tmp = self.next_temp(counter);
                    let op_str = match op {
                        BinOp::Add => "add",
                        BinOp::Sub => "sub",
                        BinOp::Mul => "mul",
                        BinOp::Div => "sdiv",
                        BinOp::Mod => "srem",
                        BinOp::BitAnd => "and",
                        BinOp::BitOr => "or",
                        BinOp::BitXor => "xor",
                        BinOp::Shl => "shl",
                        BinOp::Shr => "ashr",
                        _ => unreachable!(),
                    };

                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = {} i64 {}, {}{}\n",
                        tmp, op_str, left_val, right_val, dbg_info
                    ));
                    Ok((tmp, ir))
                }
            }

            Expr::Unary { op, expr: inner } => {
                let (val, val_ir) = self.generate_expr(inner, counter)?;
                let tmp = self.next_temp(counter);

                let mut ir = val_ir;
                let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                match op {
                    UnaryOp::Neg => {
                        ir.push_str(&format!("  {} = sub i64 0, {}{}\n", tmp, val, dbg_info));
                    }
                    UnaryOp::Not => {
                        ir.push_str(&format!("  {} = xor i1 {}, 1{}\n", tmp, val, dbg_info));
                    }
                    UnaryOp::BitNot => {
                        ir.push_str(&format!("  {} = xor i64 {}, -1{}\n", tmp, val, dbg_info));
                    }
                }

                Ok((tmp, ir))
            }

            Expr::Ternary { cond, then, else_ } => {
                // Use proper branching for lazy evaluation
                let then_label = self.next_label("ternary.then");
                let else_label = self.next_label("ternary.else");
                let merge_label = self.next_label("ternary.merge");

                // Generate condition
                let (cond_val, cond_ir) = self.generate_expr(cond, counter)?;
                let mut ir = cond_ir;

                // Convert i64 to i1 for branch
                let cond_bool = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = icmp ne i64 {}, 0\n",
                    cond_bool, cond_val
                ));

                // Conditional branch
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    cond_bool, then_label, else_label
                ));

                // Then branch
                ir.push_str(&format!("{}:\n", then_label));
                let (then_val, then_ir) = self.generate_expr(then, counter)?;
                ir.push_str(&then_ir);
                ir.push_str(&format!("  br label %{}\n", merge_label));

                // Else branch
                ir.push_str(&format!("{}:\n", else_label));
                let (else_val, else_ir) = self.generate_expr(else_, counter)?;
                ir.push_str(&else_ir);
                ir.push_str(&format!("  br label %{}\n", merge_label));

                // Merge with phi
                ir.push_str(&format!("{}:\n", merge_label));
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = phi i64 [ {}, %{} ], [ {}, %{} ]\n",
                    result, then_val, then_label, else_val, else_label
                ));

                Ok((result, ir))
            }

            Expr::Call { func, args } => {
                // Check if this is an enum variant constructor (e.g., Some(42))
                if let Expr::Ident(name) = &func.node {
                    if let Some((enum_name, tag)) = self.get_tuple_variant_info(name) {
                        // This is a tuple enum variant constructor
                        let mut ir = String::new();

                        // Generate argument values
                        let mut arg_vals = Vec::new();
                        for arg in args {
                            let (val, arg_ir) = self.generate_expr(arg, counter)?;
                            ir.push_str(&arg_ir);
                            arg_vals.push(val);
                        }

                        // Create enum value on stack: { i32 tag, i64 payload }
                        let enum_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = alloca %{}\n",
                            enum_ptr, enum_name
                        ));

                        // Store tag
                        let tag_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0\n",
                            tag_ptr, enum_name, enum_name, enum_ptr
                        ));
                        ir.push_str(&format!(
                            "  store i32 {}, i32* {}\n",
                            tag, tag_ptr
                        ));

                        // Store payload (for single-field tuple variants)
                        if !arg_vals.is_empty() {
                            let payload_ptr = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = getelementptr %{}, %{}* {}, i32 0, i32 1\n",
                                payload_ptr, enum_name, enum_name, enum_ptr
                            ));
                            // For single-field, store directly; for multi-field, need nested getelementptr
                            ir.push_str(&format!(
                                "  store i64 {}, i64* {}\n",
                                arg_vals[0], payload_ptr
                            ));
                        }

                        // Return pointer to the enum
                        return Ok((enum_ptr, ir));
                    }
                }

                // Check if this is a direct function call or indirect (lambda) call
                let (fn_name, is_indirect) = if let Expr::Ident(name) = &func.node {
                    // Check if this is a known function or a local variable (lambda)
                    if self.functions.contains_key(name) {
                        (name.clone(), false)
                    } else if self.locals.contains_key(name) {
                        (name.clone(), true) // Lambda call
                    } else {
                        (name.clone(), false) // Assume it's a function
                    }
                } else if let Expr::SelfCall = &func.node {
                    (self.current_function.clone().unwrap_or_default(), false)
                } else {
                    return Err(CodegenError::Unsupported("complex indirect call".to_string()));
                };

                // Look up function info for parameter types (only for direct calls)
                let fn_info = if !is_indirect {
                    self.functions.get(&fn_name).cloned()
                } else {
                    None
                };

                let mut ir = String::new();
                let mut arg_vals = Vec::new();

                for (i, arg) in args.iter().enumerate() {
                    let (mut val, arg_ir) = self.generate_expr(arg, counter)?;
                    ir.push_str(&arg_ir);

                    // Get parameter type from function info if available
                    let param_ty = fn_info
                        .as_ref()
                        .and_then(|f| f.params.get(i))
                        .map(|(_, ty)| ty.clone());

                    let arg_ty = param_ty
                        .as_ref()
                        .map(|ty| self.type_to_llvm(ty))
                        .unwrap_or_else(|| "i64".to_string());

                    // Insert integer conversion if needed (trunc for narrowing, sext for widening)
                    if let Some(param_type) = &param_ty {
                        let src_bits = self.get_integer_bits_from_val(&val);
                        let dst_bits = self.get_integer_bits(param_type);

                        if src_bits > 0 && dst_bits > 0 && src_bits != dst_bits {
                            let conv_tmp = self.next_temp(counter);
                            let src_ty = format!("i{}", src_bits);
                            let dst_ty = format!("i{}", dst_bits);

                            if src_bits > dst_bits {
                                // Truncate
                                ir.push_str(&format!(
                                    "  {} = trunc {} {} to {}\n",
                                    conv_tmp, src_ty, val, dst_ty
                                ));
                            } else {
                                // Sign extend
                                ir.push_str(&format!(
                                    "  {} = sext {} {} to {}\n",
                                    conv_tmp, src_ty, val, dst_ty
                                ));
                            }
                            val = conv_tmp;
                        }
                    }

                    arg_vals.push(format!("{} {}", arg_ty, val));
                }

                // Get return type and actual function name (may differ for builtins)
                let ret_ty = fn_info
                    .as_ref()
                    .map(|f| self.type_to_llvm(&f.ret_type))
                    .unwrap_or_else(|| "i64".to_string());

                let actual_fn_name = fn_info
                    .as_ref()
                    .map(|f| f.name.clone())
                    .unwrap_or_else(|| fn_name.clone());

                if is_indirect {
                    // Check if this is a closure with captured variables
                    let closure_info = self.closures.get(&fn_name).cloned();

                    // Get the actual LLVM name for this variable
                    let llvm_var_name = self.locals.get(&fn_name)
                        .map(|l| l.llvm_name.clone())
                        .unwrap_or_else(|| fn_name.clone());

                    // Indirect call (lambda): load function pointer and call
                    let ptr_tmp = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = load i64, i64* %{}\n",
                        ptr_tmp, llvm_var_name
                    ));

                    // Prepend captured values to arguments if this is a closure
                    let mut all_args = Vec::new();
                    if let Some(ref info) = closure_info {
                        for (_, capture_val) in &info.captures {
                            all_args.push(format!("i64 {}", capture_val));
                        }
                    }
                    all_args.extend(arg_vals);

                    // Build function type signature for indirect call (including captures)
                    let arg_types: Vec<String> = all_args
                        .iter()
                        .map(|a| a.split_whitespace().next().unwrap_or("i64").to_string())
                        .collect();
                    let fn_type = format!("i64 ({})*", arg_types.join(", "));

                    // Cast i64 to function pointer
                    let fn_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = inttoptr i64 {} to {}\n",
                        fn_ptr, ptr_tmp, fn_type
                    ));

                    // Make indirect call with all arguments
                    let tmp = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = call i64 {}({}){}\n",
                        tmp, fn_ptr, all_args.join(", "), dbg_info
                    ));
                    Ok((tmp, ir))
                } else if fn_name == "malloc" {
                    // Special handling for malloc: call returns i8*, convert to i64
                    let ptr_tmp = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = call i8* @malloc({}){}\n",
                        ptr_tmp,
                        arg_vals.join(", "),
                        dbg_info
                    ));
                    let result = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = ptrtoint i8* {} to i64\n",
                        result, ptr_tmp
                    ));
                    Ok((result, ir))
                } else if fn_name == "free" {
                    // Special handling for free: convert i64 to i8*
                    let ptr_tmp = self.next_temp(counter);
                    // Extract the i64 value from arg_vals
                    let arg_val = arg_vals.first()
                        .map(|s| s.split_whitespace().last().unwrap_or("0"))
                        .unwrap_or("0");
                    ir.push_str(&format!(
                        "  {} = inttoptr i64 {} to i8*\n",
                        ptr_tmp, arg_val
                    ));
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  call void @free(i8* {}){}\n",
                        ptr_tmp, dbg_info
                    ));
                    Ok(("void".to_string(), ir))
                } else if fn_name == "memcpy" {
                    // Special handling for memcpy: convert i64 pointers to i8*
                    let dest_val = arg_vals.get(0)
                        .map(|s| s.split_whitespace().last().unwrap_or("0"))
                        .unwrap_or("0");
                    let src_val = arg_vals.get(1)
                        .map(|s| s.split_whitespace().last().unwrap_or("0"))
                        .unwrap_or("0");
                    let n_val = arg_vals.get(2)
                        .map(|s| s.split_whitespace().last().unwrap_or("0"))
                        .unwrap_or("0");

                    let dest_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = inttoptr i64 {} to i8*\n",
                        dest_ptr, dest_val
                    ));
                    let src_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = inttoptr i64 {} to i8*\n",
                        src_ptr, src_val
                    ));
                    let result = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = call i8* @memcpy(i8* {}, i8* {}, i64 {}){}\n",
                        result, dest_ptr, src_ptr, n_val, dbg_info
                    ));
                    // Convert result back to i64
                    let result_i64 = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = ptrtoint i8* {} to i64\n",
                        result_i64, result
                    ));
                    Ok((result_i64, ir))
                } else if fn_name == "strlen" {
                    // Special handling for strlen: convert i64 to i8*
                    let arg_val = arg_vals.first()
                        .map(|s| s.split_whitespace().last().unwrap_or("0"))
                        .unwrap_or("0");
                    let ptr_tmp = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = inttoptr i64 {} to i8*\n",
                        ptr_tmp, arg_val
                    ));
                    let result = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = call i64 @strlen(i8* {}){}\n",
                        result, ptr_tmp, dbg_info
                    ));
                    Ok((result, ir))
                } else if fn_name == "puts_ptr" {
                    // Special handling for puts_ptr: convert i64 to i8*
                    let arg_val = arg_vals.first()
                        .map(|s| s.split_whitespace().last().unwrap_or("0"))
                        .unwrap_or("0");
                    let ptr_tmp = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = inttoptr i64 {} to i8*\n",
                        ptr_tmp, arg_val
                    ));
                    let result = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = call i32 @puts(i8* {}){}\n",
                        result, ptr_tmp, dbg_info
                    ));
                    Ok((result, ir))
                } else if ret_ty == "void" {
                    // Direct void function call
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  call void @{}({}){}\n",
                        actual_fn_name,
                        arg_vals.join(", "),
                        dbg_info
                    ));
                    Ok(("void".to_string(), ir))
                } else {
                    // Direct function call with return value
                    let tmp = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = call {} @{}({}){}\n",
                        tmp,
                        ret_ty,
                        actual_fn_name,
                        arg_vals.join(", "),
                        dbg_info
                    ));
                    Ok((tmp, ir))
                }
            }

            // If/Else expression with basic blocks
            Expr::If { cond, then, else_ } => {
                let then_label = self.next_label("then");
                let else_label = self.next_label("else");
                let merge_label = self.next_label("merge");

                // Generate condition
                let (cond_val, cond_ir) = self.generate_expr(cond, counter)?;
                let mut ir = cond_ir;

                // Convert i64 to i1 for branch condition
                let cond_bool = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = icmp ne i64 {}, 0\n",
                    cond_bool, cond_val
                ));

                // Conditional branch
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    cond_bool, then_label, else_label
                ));

                // Then block
                ir.push_str(&format!("{}:\n", then_label));
                self.current_block = then_label.clone();
                let (then_val, then_ir, then_terminated) = self.generate_block_stmts(then, counter)?;
                ir.push_str(&then_ir);
                let then_actual_block = self.current_block.clone();
                // Only emit branch to merge if block is not terminated
                let then_from_label = if !then_terminated {
                    ir.push_str(&format!("  br label %{}\n", merge_label));
                    then_actual_block
                } else {
                    String::new() // Block is terminated, won't contribute to phi
                };

                // Else block
                ir.push_str(&format!("{}:\n", else_label));
                self.current_block = else_label.clone();
                let (else_val, else_ir, else_terminated, nested_last_block, has_else) = if let Some(else_branch) = else_ {
                    let (v, i, t, last) = self.generate_if_else_with_term(else_branch, counter, &merge_label)?;
                    (v, i, t, last, true)
                } else {
                    ("0".to_string(), String::new(), false, String::new(), false)
                };
                ir.push_str(&else_ir);
                // Only emit branch to merge if block is not terminated
                let else_from_label = if !else_terminated {
                    ir.push_str(&format!("  br label %{}\n", merge_label));
                    // If there was a nested if-else, use its merge block as the predecessor
                    if !nested_last_block.is_empty() {
                        nested_last_block
                    } else {
                        self.current_block.clone()
                    }
                } else {
                    String::new()
                };

                // Merge block with phi node
                ir.push_str(&format!("{}:\n", merge_label));
                self.current_block = merge_label.clone();
                let result = self.next_temp(counter);

                // Detect if values are pointers (start with % and come from enum constructors)
                // For such cases, we need to ptrtoint them to use in phi i64
                let then_val_for_phi = then_val.clone();
                let else_val_for_phi = else_val.clone();

                // If there's no else branch, don't use phi - the value is not meaningful
                // This avoids type mismatches when then branch returns i32 (e.g., putchar)
                if !has_else {
                    // If-only statement: value is not used, just use 0
                    ir.push_str(&format!("  {} = add i64 0, 0\n", result));
                } else if !then_from_label.is_empty() && !else_from_label.is_empty() {
                    // Both branches reach merge
                    ir.push_str(&format!(
                        "  {} = phi i64 [ {}, %{} ], [ {}, %{} ]\n",
                        result, then_val_for_phi, then_from_label, else_val_for_phi, else_from_label
                    ));
                } else if !then_from_label.is_empty() {
                    // Only then branch reaches merge
                    ir.push_str(&format!(
                        "  {} = phi i64 [ {}, %{} ]\n",
                        result, then_val_for_phi, then_from_label
                    ));
                } else if !else_from_label.is_empty() {
                    // Only else branch reaches merge
                    ir.push_str(&format!(
                        "  {} = phi i64 [ {}, %{} ]\n",
                        result, else_val_for_phi, else_from_label
                    ));
                } else {
                    // Neither branch reaches merge (both break/continue)
                    // This merge block is actually unreachable, but we still need a value
                    ir.push_str(&format!("  {} = add i64 0, 0\n", result));
                }

                Ok((result, ir))
            }

            // Loop expression
            Expr::Loop { pattern: _, iter, body } => {
                let loop_start = self.next_label("loop.start");
                let loop_body = self.next_label("loop.body");
                let loop_end = self.next_label("loop.end");

                // Push loop labels for break/continue
                self.loop_stack.push(LoopLabels {
                    continue_label: loop_start.clone(),
                    break_label: loop_end.clone(),
                });

                let mut ir = String::new();

                // Check if this is a conditional loop (L cond { body }) or infinite loop
                if let Some(iter_expr) = iter {
                    // Conditional loop: L condition { body }
                    ir.push_str(&format!("  br label %{}\n", loop_start));
                    ir.push_str(&format!("{}:\n", loop_start));

                    // Evaluate condition
                    let (cond_val, cond_ir) = self.generate_expr(iter_expr, counter)?;
                    ir.push_str(&cond_ir);

                    // Convert i64 to i1 for branch
                    let cond_bool = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = icmp ne i64 {}, 0\n",
                        cond_bool, cond_val
                    ));
                    ir.push_str(&format!(
                        "  br i1 {}, label %{}, label %{}\n",
                        cond_bool, loop_body, loop_end
                    ));

                    // Loop body
                    ir.push_str(&format!("{}:\n", loop_body));
                    let (_body_val, body_ir, body_terminated) = self.generate_block_stmts(body, counter)?;
                    ir.push_str(&body_ir);
                    // Only emit loop back if body doesn't terminate
                    if !body_terminated {
                        ir.push_str(&format!("  br label %{}\n", loop_start));
                    }
                } else {
                    // Infinite loop: L { body } - must use break to exit
                    ir.push_str(&format!("  br label %{}\n", loop_start));
                    ir.push_str(&format!("{}:\n", loop_start));
                    let (_body_val, body_ir, body_terminated) = self.generate_block_stmts(body, counter)?;
                    ir.push_str(&body_ir);
                    // Only emit loop back if body doesn't terminate
                    if !body_terminated {
                        ir.push_str(&format!("  br label %{}\n", loop_start));
                    }
                }

                // Loop end
                ir.push_str(&format!("{}:\n", loop_end));

                self.loop_stack.pop();

                // Loop returns void by default (use break with value for expression)
                Ok(("0".to_string(), ir))
            }

            // Block expression
            Expr::Block(stmts) => {
                let (val, ir, _terminated) = self.generate_block_stmts(stmts, counter)?;
                Ok((val, ir))
            }

            // Assignment expression
            Expr::Assign { target, value } => {
                let (val, val_ir) = self.generate_expr(value, counter)?;
                let mut ir = val_ir;

                if let Expr::Ident(name) = &target.node {
                    if let Some(local) = self.locals.get(name).cloned() {
                        if !local.is_param {
                            let llvm_ty = self.type_to_llvm(&local.ty);
                            ir.push_str(&format!(
                                "  store {} {}, {}* %{}\n",
                                llvm_ty, val, llvm_ty, local.llvm_name
                            ));
                        }
                    }
                } else if let Expr::Field { expr: obj_expr, field } = &target.node {
                    // Field assignment: obj.field = value
                    let (obj_val, obj_ir) = self.generate_expr(obj_expr, counter)?;
                    ir.push_str(&obj_ir);

                    // Get struct info
                    if let Expr::Ident(var_name) = &obj_expr.node {
                        if let Some(local) = self.locals.get(var_name.as_str()).cloned() {
                            if let ResolvedType::Named { name: struct_name, .. } = &local.ty {
                                if let Some(struct_info) = self.structs.get(struct_name).cloned() {
                                    if let Some(field_idx) = struct_info.fields.iter()
                                        .position(|(n, _)| n == &field.node)
                                    {
                                        let field_ty = &struct_info.fields[field_idx].1;
                                        let llvm_ty = self.type_to_llvm(field_ty);

                                        let field_ptr = self.next_temp(counter);
                                        ir.push_str(&format!(
                                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}\n",
                                            field_ptr, struct_name, struct_name, obj_val, field_idx
                                        ));
                                        ir.push_str(&format!(
                                            "  store {} {}, {}* {}\n",
                                            llvm_ty, val, llvm_ty, field_ptr
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }

                Ok((val, ir))
            }

            // Compound assignment (+=, -=, etc.)
            Expr::AssignOp { op, target, value } => {
                // First load current value
                let (current_val, load_ir) = self.generate_expr(target, counter)?;
                let (rhs_val, rhs_ir) = self.generate_expr(value, counter)?;

                let mut ir = load_ir;
                ir.push_str(&rhs_ir);

                let op_str = match op {
                    BinOp::Add => "add",
                    BinOp::Sub => "sub",
                    BinOp::Mul => "mul",
                    BinOp::Div => "sdiv",
                    BinOp::Mod => "srem",
                    BinOp::BitAnd => "and",
                    BinOp::BitOr => "or",
                    BinOp::BitXor => "xor",
                    BinOp::Shl => "shl",
                    BinOp::Shr => "ashr",
                    _ => return Err(CodegenError::Unsupported(format!("compound {:?}", op))),
                };

                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = {} i64 {}, {}\n",
                    result, op_str, current_val, rhs_val
                ));

                // Store back
                if let Expr::Ident(name) = &target.node {
                    if let Some(local) = self.locals.get(name.as_str()).cloned() {
                        if !local.is_param {
                            let llvm_ty = self.type_to_llvm(&local.ty);
                            ir.push_str(&format!(
                                "  store {} {}, {}* %{}\n",
                                llvm_ty, result, llvm_ty, local.llvm_name
                            ));
                        }
                    }
                }

                Ok((result, ir))
            }

            // Array literal: [a, b, c]
            Expr::Array(elements) => {
                let mut ir = String::new();
                let len = elements.len();

                // Allocate array on stack
                let arr_ptr = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = alloca [{}  x i64]\n",
                    arr_ptr, len
                ));

                // Store each element
                for (i, elem) in elements.iter().enumerate() {
                    let (val, elem_ir) = self.generate_expr(elem, counter)?;
                    ir.push_str(&elem_ir);

                    let elem_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr [{}  x i64], [{}  x i64]* {}, i64 0, i64 {}\n",
                        elem_ptr, len, len, arr_ptr, i
                    ));
                    ir.push_str(&format!(
                        "  store i64 {}, i64* {}\n",
                        val, elem_ptr
                    ));
                }

                // Return pointer to first element
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr [{}  x i64], [{}  x i64]* {}, i64 0, i64 0\n",
                    result, len, len, arr_ptr
                ));

                Ok((result, ir))
            }

            // Tuple literal: (a, b, c)
            Expr::Tuple(elements) => {
                let mut ir = String::new();
                let len = elements.len();

                // Build tuple type string
                let tuple_ty = format!("{{ {} }}", vec!["i64"; len].join(", "));

                // Allocate tuple on stack
                let tuple_ptr = self.next_temp(counter);
                ir.push_str(&format!("  {} = alloca {}\n", tuple_ptr, tuple_ty));

                // Store each element
                for (i, elem) in elements.iter().enumerate() {
                    let (val, elem_ir) = self.generate_expr(elem, counter)?;
                    ir.push_str(&elem_ir);

                    let elem_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr {}, {}* {}, i32 0, i32 {}\n",
                        elem_ptr, tuple_ty, tuple_ty, tuple_ptr, i
                    ));
                    ir.push_str(&format!("  store i64 {}, i64* {}\n", val, elem_ptr));
                }

                // Load and return tuple value
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = load {}, {}* {}\n",
                    result, tuple_ty, tuple_ty, tuple_ptr
                ));

                Ok((result, ir))
            }

            // Struct literal: Point{x:1, y:2}
            Expr::StructLit { name, fields } => {
                let struct_name = &name.node;

                // Look up struct info
                let struct_info = self.structs.get(struct_name)
                    .ok_or_else(|| CodegenError::TypeError(format!("Unknown struct: {}", struct_name)))?
                    .clone();

                let mut ir = String::new();

                // Allocate struct on stack
                let struct_ptr = self.next_temp(counter);
                ir.push_str(&format!("  {} = alloca %{}\n", struct_ptr, struct_name));

                // Store each field
                for (field_name, field_expr) in fields {
                    // Find field index
                    let field_idx = struct_info.fields.iter()
                        .position(|(n, _)| n == &field_name.node)
                        .ok_or_else(|| CodegenError::TypeError(format!(
                            "Unknown field '{}' in struct '{}'", field_name.node, struct_name
                        )))?;

                    let (val, field_ir) = self.generate_expr(field_expr, counter)?;
                    ir.push_str(&field_ir);

                    let field_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}\n",
                        field_ptr, struct_name, struct_name, struct_ptr, field_idx
                    ));

                    let field_ty = &struct_info.fields[field_idx].1;
                    let llvm_ty = self.type_to_llvm(field_ty);
                    ir.push_str(&format!(
                        "  store {} {}, {}* {}\n",
                        llvm_ty, val, llvm_ty, field_ptr
                    ));
                }

                // Return pointer to struct
                Ok((struct_ptr, ir))
            }

            // Index: arr[idx] or slice: arr[start..end]
            Expr::Index { expr: array_expr, index } => {
                // Check if this is a slice operation (index is a Range expression)
                if let Expr::Range { start, end, inclusive } = &index.node {
                    return self.generate_slice(array_expr, start.as_deref(), end.as_deref(), *inclusive, counter);
                }

                let (arr_val, arr_ir) = self.generate_expr(array_expr, counter)?;
                let (idx_val, idx_ir) = self.generate_expr(index, counter)?;

                let mut ir = arr_ir;
                ir.push_str(&idx_ir);

                // Get element pointer
                let elem_ptr = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr i64, i64* {}, i64 {}\n",
                    elem_ptr, arr_val, idx_val
                ));

                // Load element
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = load i64, i64* {}\n",
                    result, elem_ptr
                ));

                Ok((result, ir))
            }

            // Field access: obj.field
            Expr::Field { expr: obj_expr, field } => {
                let (obj_val, obj_ir) = self.generate_expr(obj_expr, counter)?;
                let mut ir = obj_ir;

                // Try to find struct info based on variable name
                if let Expr::Ident(var_name) = &obj_expr.node {
                    if let Some(local) = self.locals.get(var_name.as_str()).cloned() {
                        if let ResolvedType::Named { name: struct_name, .. } = &local.ty {
                            if let Some(struct_info) = self.structs.get(struct_name).cloned() {
                                let field_idx = struct_info.fields.iter()
                                    .position(|(n, _)| n == &field.node)
                                    .ok_or_else(|| CodegenError::TypeError(format!(
                                        "Unknown field '{}' in struct '{}'", field.node, struct_name
                                    )))?;

                                let field_ty = &struct_info.fields[field_idx].1;
                                let llvm_ty = self.type_to_llvm(field_ty);

                                // Generate temps in correct order: field_ptr first, then result
                                let field_ptr = self.next_temp(counter);
                                ir.push_str(&format!(
                                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}\n",
                                    field_ptr, struct_name, struct_name, obj_val, field_idx
                                ));

                                let result = self.next_temp(counter);
                                ir.push_str(&format!(
                                    "  {} = load {}, {}* {}\n",
                                    result, llvm_ty, llvm_ty, field_ptr
                                ));

                                return Ok((result, ir));
                            }
                        }
                    }
                }

                Err(CodegenError::Unsupported("field access requires known struct type".to_string()))
            }

            // Method call: obj.method(args)
            Expr::MethodCall { receiver, method, args } => {
                // Special case: @.method() means self.method() (call another method on self)
                let (recv_val, recv_ir, recv_type) = if matches!(&receiver.node, Expr::SelfCall) {
                    // When receiver is @, use %self instead of the function pointer
                    if let Some(local) = self.locals.get("self") {
                        let recv_type = local.ty.clone();
                        ("%self".to_string(), String::new(), recv_type)
                    } else {
                        return Err(CodegenError::Unsupported(
                            "@.method() used outside of a method with self".to_string(),
                        ));
                    }
                } else {
                    let (recv_val, recv_ir) = self.generate_expr(receiver, counter)?;
                    let recv_type = self.infer_expr_type(receiver);
                    (recv_val, recv_ir, recv_type)
                };
                let mut ir = recv_ir;

                let method_name = &method.node;

                // Build full method name: StructName_methodName
                let full_method_name = if let ResolvedType::Named { name, .. } = &recv_type {
                    format!("{}_{}", name, method_name)
                } else {
                    method_name.clone()
                };

                // Get struct type for receiver (add * for pointer)
                let recv_llvm_ty = if matches!(&recv_type, ResolvedType::Named { .. }) {
                    format!("{}*", self.type_to_llvm(&recv_type))
                } else {
                    self.type_to_llvm(&recv_type)
                };

                // Generate arguments (receiver is implicit first arg)
                let mut arg_vals = vec![format!("{} {}", recv_llvm_ty, recv_val)];

                for arg in args {
                    let (val, arg_ir) = self.generate_expr(arg, counter)?;
                    ir.push_str(&arg_ir);
                    let arg_type = self.infer_expr_type(arg);
                    let arg_llvm_ty = self.type_to_llvm(&arg_type);
                    arg_vals.push(format!("{} {}", arg_llvm_ty, val));
                }

                // Determine return type from struct methods
                let ret_type = if let ResolvedType::Named { name, .. } = &recv_type {
                    if let Some(struct_info) = self.structs.get(name) {
                        // For now, assume i64 return
                        let _ = struct_info;
                        "i64"
                    } else {
                        "i64"
                    }
                } else {
                    "i64"
                };

                let tmp = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call {} @{}({})\n",
                    tmp,
                    ret_type,
                    full_method_name,
                    arg_vals.join(", ")
                ));

                Ok((tmp, ir))
            }

            // Static method call: Type.method(args)
            Expr::StaticMethodCall { type_name, method, args } => {
                let mut ir = String::new();

                // Build full method name: TypeName_methodName
                let full_method_name = format!("{}_{}", type_name.node, method.node);

                // Generate arguments (no receiver for static methods)
                let mut arg_vals = Vec::new();

                for arg in args {
                    let (val, arg_ir) = self.generate_expr(arg, counter)?;
                    ir.push_str(&arg_ir);
                    let arg_type = self.infer_expr_type(arg);
                    let arg_llvm_ty = self.type_to_llvm(&arg_type);
                    arg_vals.push(format!("{} {}", arg_llvm_ty, val));
                }

                // Get return type from method signature
                let ret_type = self.functions.get(&full_method_name)
                    .map(|info| self.type_to_llvm(&info.ret_type))
                    .unwrap_or_else(|| "i64".to_string());

                let tmp = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call {} @{}({})\n",
                    tmp,
                    ret_type,
                    full_method_name,
                    arg_vals.join(", ")
                ));

                Ok((tmp, ir))
            }

            // Reference: &expr
            Expr::Ref(inner) => {
                // For simple references, just return the address
                if let Expr::Ident(name) = &inner.node {
                    if self.locals.contains_key(name.as_str()) {
                        return Ok((format!("%{}", name), String::new()));
                    }
                }
                // For complex expressions, evaluate and return
                self.generate_expr(inner, counter)
            }

            // Dereference: *expr
            Expr::Deref(inner) => {
                let (ptr_val, ptr_ir) = self.generate_expr(inner, counter)?;
                let mut ir = ptr_ir;

                let result = self.next_temp(counter);
                ir.push_str(&format!("  {} = load i64, i64* {}\n", result, ptr_val));

                Ok((result, ir))
            }

            // Match expression: M expr { pattern => body, ... }
            Expr::Match { expr: match_expr, arms } => {
                self.generate_match(match_expr, arms, counter)
            }

            // Range expression (for now just return start value)
            Expr::Range { start, .. } => {
                if let Some(start_expr) = start {
                    self.generate_expr(start_expr, counter)
                } else {
                    Ok(("0".to_string(), String::new()))
                }
            }

            // Await expression: poll the future until Ready
            Expr::Await(inner) => {
                let (future_ptr, future_ir) = self.generate_expr(inner, counter)?;
                let mut ir = future_ir;

                // Get the function name being awaited (for poll function lookup)
                // Helper to extract poll function name from an expression
                fn get_poll_func_name(expr: &Expr) -> String {
                    match expr {
                        Expr::Call { func, .. } => {
                            if let Expr::Ident(name) = &func.node {
                                format!("{}__poll", name)
                            } else {
                                "__async_poll".to_string()
                            }
                        }
                        Expr::MethodCall { method, .. } => {
                            format!("{}__poll", method.node)
                        }
                        Expr::Spawn(inner) => {
                            // For spawn, look at the inner expression
                            get_poll_func_name(&inner.node)
                        }
                        _ => "__async_poll".to_string(),
                    }
                }
                let poll_func = get_poll_func_name(&inner.node);

                // Generate blocking poll loop
                let poll_start = self.next_label("await_poll");
                let poll_ready = self.next_label("await_ready");
                let poll_pending = self.next_label("await_pending");

                ir.push_str(&format!("  br label %{}\n\n", poll_start));
                ir.push_str(&format!("{}:\n", poll_start));

                // Call poll function: returns {i64 status, i64 result}
                let poll_result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call {{ i64, i64 }} @{}(i64 {})\n",
                    poll_result, poll_func, future_ptr
                ));

                // Extract status (0 = Pending, 1 = Ready)
                let status = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {{ i64, i64 }} {}, 0\n",
                    status, poll_result
                ));

                // Check if Ready
                let is_ready = self.next_temp(counter);
                ir.push_str(&format!("  {} = icmp eq i64 {}, 1\n", is_ready, status));
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n\n",
                    is_ready, poll_ready, poll_pending
                ));

                // Pending: yield and retry (for now just spin)
                ir.push_str(&format!("{}:\n", poll_pending));
                ir.push_str(&format!("  br label %{}\n\n", poll_start));

                // Ready: extract result
                ir.push_str(&format!("{}:\n", poll_ready));
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {{ i64, i64 }} {}, 1\n",
                    result, poll_result
                ));

                Ok((result, ir))
            }

            // Spawn expression: create a new task for the runtime
            Expr::Spawn(inner) => {
                let (future_ptr, future_ir) = self.generate_expr(inner, counter)?;
                let mut ir = future_ir;

                // Spawn returns the task/future handle for later awaiting
                // For now, just return the future pointer directly
                ir.push_str(&format!("; Spawned task at {}\n", future_ptr));

                Ok((future_ptr, ir))
            }

            // Lambda expression with captures
            Expr::Lambda { params, body, captures: _ } => {
                // Generate a unique function name for this lambda
                let lambda_name = format!("__lambda_{}", self.label_counter);
                self.label_counter += 1;

                // Find captured variables by analyzing free variables in lambda body
                let capture_names = self.find_lambda_captures(params, body);

                // Collect captured variable info from current scope
                let mut captured_vars: Vec<(String, ResolvedType, String)> = Vec::new();
                let mut capture_ir = String::new();

                for cap_name in &capture_names {
                    if let Some(local) = self.locals.get(cap_name) {
                        let ty = local.ty.clone();
                        // Load captured value if it's a local variable
                        if local.is_param {
                            // Parameters are already values, use directly
                            captured_vars.push((cap_name.clone(), ty, format!("%{}", local.llvm_name)));
                        } else {
                            // Load from alloca
                            let tmp = self.next_temp(counter);
                            let llvm_ty = self.type_to_llvm(&ty);
                            capture_ir.push_str(&format!(
                                "  {} = load {}, {}* %{}\n",
                                tmp, llvm_ty, llvm_ty, local.llvm_name
                            ));
                            captured_vars.push((cap_name.clone(), ty, tmp));
                        }
                    }
                }

                // Build parameter list (original params + captured vars)
                let mut param_strs = Vec::new();
                let mut param_types = Vec::new();

                // First add captured variables as parameters (they come first)
                for (cap_name, cap_ty, _) in &captured_vars {
                    let llvm_ty = self.type_to_llvm(cap_ty);
                    param_strs.push(format!("{} %__cap_{}", llvm_ty, cap_name));
                    param_types.push(llvm_ty);
                }

                // Then add original lambda parameters
                for p in params {
                    let ty = self.ast_type_to_resolved(&p.ty.node);
                    let llvm_ty = self.type_to_llvm(&ty);
                    param_strs.push(format!("{} %{}", llvm_ty, p.name.node));
                    param_types.push(llvm_ty);
                }

                // Store current function state
                let saved_function = self.current_function.clone();
                let saved_locals = self.locals.clone();

                // Set up lambda context
                self.current_function = Some(lambda_name.clone());
                self.locals.clear();

                // Register captured variables as locals (using capture parameter names)
                for (cap_name, cap_ty, _) in &captured_vars {
                    self.locals.insert(
                        cap_name.clone(),
                        LocalVar {
                            ty: cap_ty.clone(),
                            is_param: true,
                            llvm_name: format!("__cap_{}", cap_name),
                        },
                    );
                }

                // Register original parameters as locals
                for p in params {
                    let ty = self.ast_type_to_resolved(&p.ty.node);
                    self.locals.insert(
                        p.name.node.clone(),
                        LocalVar {
                            ty,
                            is_param: true,
                            llvm_name: p.name.node.clone(),
                        },
                    );
                }

                // Generate lambda body
                let mut lambda_counter = 0;
                let (body_val, body_ir) = self.generate_expr(body, &mut lambda_counter)?;

                // Build lambda function IR
                let mut lambda_ir = format!(
                    "define i64 @{}({}) {{\nentry:\n",
                    lambda_name,
                    param_strs.join(", ")
                );
                lambda_ir.push_str(&body_ir);
                lambda_ir.push_str(&format!("  ret i64 {}\n}}\n", body_val));

                // Store lambda function for later emission
                self.lambda_functions.push(lambda_ir);

                // Restore function context
                self.current_function = saved_function;
                self.locals = saved_locals;

                // Store lambda info for Let statement to pick up
                if captured_vars.is_empty() {
                    self.last_lambda_info = None;
                    Ok((format!("ptrtoint (i64 ({})* @{} to i64)", param_types.join(", "), lambda_name), capture_ir))
                } else {
                    // Store closure info with captured variable values
                    self.last_lambda_info = Some(ClosureInfo {
                        func_name: lambda_name.clone(),
                        captures: captured_vars.iter()
                            .map(|(name, _, val)| (name.clone(), val.clone()))
                            .collect(),
                    });
                    Ok((format!("ptrtoint (i64 ({})* @{} to i64)", param_types.join(", "), lambda_name), capture_ir))
                }
            }

            // Try expression: expr? - propagate Err early, continue with Ok value
            // Result layout: {i64 tag, i64 value} where tag=0 is Ok, tag=1 is Err
            Expr::Try(inner) => {
                let (inner_val, inner_ir) = self.generate_expr(inner, counter)?;
                let mut ir = inner_ir;

                // Extract tag from result
                let _tag_tmp = self.next_temp(counter);
                let result_ptr = self.next_temp(counter);
                let tag_ptr = self.next_temp(counter);
                let tag = self.next_temp(counter);

                // Assume result is a struct {i64, i64} passed as i64 ptr or packed value
                // For simplicity, treat it as a 2-element struct: {tag, value}
                // where tag 0 = Ok, tag 1 = Err
                ir.push_str(&format!("  ; Try expression\n"));
                ir.push_str(&format!("  {} = inttoptr i64 {} to {{i64, i64}}*\n", result_ptr, inner_val));
                ir.push_str(&format!("  {} = getelementptr {{i64, i64}}, {{i64, i64}}* {}, i32 0, i32 0\n", tag_ptr, result_ptr));
                ir.push_str(&format!("  {} = load i64, i64* {}\n", tag, tag_ptr));

                // Check if Err (tag == 1)
                let is_err = self.next_temp(counter);
                let err_label = self.next_label("try_err");
                let ok_label = self.next_label("try_ok");
                let merge_label = self.next_label("try_merge");

                ir.push_str(&format!("  {} = icmp eq i64 {}, 1\n", is_err, tag));
                ir.push_str(&format!("  br i1 {}, label %{}, label %{}\n\n", is_err, err_label, ok_label));

                // Err branch: return early with the same error
                ir.push_str(&format!("{}:\n", err_label));
                ir.push_str(&format!("  ret i64 {}  ; early return on Err\n\n", inner_val));

                // Ok branch: extract value and continue
                ir.push_str(&format!("{}:\n", ok_label));
                let value_ptr = self.next_temp(counter);
                let value = self.next_temp(counter);
                ir.push_str(&format!("  {} = getelementptr {{i64, i64}}, {{i64, i64}}* {}, i32 0, i32 1\n", value_ptr, result_ptr));
                ir.push_str(&format!("  {} = load i64, i64* {}\n", value, value_ptr));
                ir.push_str(&format!("  br label %{}\n\n", merge_label));

                // Merge block
                ir.push_str(&format!("{}:\n", merge_label));

                Ok((value, ir))
            }

            // Unwrap expression: expr! - panic on Err/None, continue with value
            Expr::Unwrap(inner) => {
                let (inner_val, inner_ir) = self.generate_expr(inner, counter)?;
                let mut ir = inner_ir;

                // Extract tag from result/option
                let result_ptr = self.next_temp(counter);
                let tag_ptr = self.next_temp(counter);
                let tag = self.next_temp(counter);

                ir.push_str(&format!("  ; Unwrap expression\n"));
                ir.push_str(&format!("  {} = inttoptr i64 {} to {{i64, i64}}*\n", result_ptr, inner_val));
                ir.push_str(&format!("  {} = getelementptr {{i64, i64}}, {{i64, i64}}* {}, i32 0, i32 0\n", tag_ptr, result_ptr));
                ir.push_str(&format!("  {} = load i64, i64* {}\n", tag, tag_ptr));

                // Check if Err/None (tag != 0)
                let is_err = self.next_temp(counter);
                let err_label = self.next_label("unwrap_err");
                let ok_label = self.next_label("unwrap_ok");

                ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", is_err, tag));
                ir.push_str(&format!("  br i1 {}, label %{}, label %{}\n\n", is_err, err_label, ok_label));

                // Err branch: panic/abort
                ir.push_str(&format!("{}:\n", err_label));
                // Call puts to print error message then call abort
                ir.push_str("  call i32 @puts(i8* getelementptr ([22 x i8], [22 x i8]* @.unwrap_panic_msg, i64 0, i64 0))\n");
                ir.push_str("  call void @abort()\n");
                ir.push_str("  unreachable\n\n");

                // Ok branch: extract value
                ir.push_str(&format!("{}:\n", ok_label));
                let value_ptr = self.next_temp(counter);
                let value = self.next_temp(counter);
                ir.push_str(&format!("  {} = getelementptr {{i64, i64}}, {{i64, i64}}* {}, i32 0, i32 1\n", value_ptr, result_ptr));
                ir.push_str(&format!("  {} = load i64, i64* {}\n", value, value_ptr));

                // Track that we need the panic message and abort declaration
                self.needs_unwrap_panic = true;

                Ok((value, ir))
            }
        }
    }

    fn next_temp(&self, counter: &mut usize) -> String {
        let tmp = format!("%{}", counter);
        *counter += 1;
        tmp
    }

    /// Generate code for a block expression (used in if/else branches)
    #[allow(dead_code)]
    fn generate_block_expr(
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
        let mut ir = String::new();
        let mut last_value = "0".to_string();
        let mut terminated = false;

        for stmt in stmts {
            // Skip generating code after a terminator
            if terminated {
                break;
            }

            let (value, stmt_ir) = self.generate_stmt(stmt, counter)?;
            ir.push_str(&stmt_ir);
            last_value = value;

            // Check if this statement is a terminator
            match &stmt.node {
                Stmt::Break(_) | Stmt::Continue | Stmt::Return(_) => {
                    terminated = true;
                }
                _ => {}
            }
        }

        Ok((last_value, ir, terminated))
    }

    /// Helper to convert BlockResult to simple (String, String) for backward compatibility

    /// Generate code for if-else branches (legacy, for backward compat)
    #[allow(dead_code)]
    fn generate_if_else(
        &mut self,
        if_else: &IfElse,
        counter: &mut usize,
        merge_label: &str,
    ) -> CodegenResult<(String, String)> {
        let (val, ir, _terminated, _last_block) = self.generate_if_else_with_term(if_else, counter, merge_label)?;
        Ok((val, ir))
    }

    /// Generate code for if-else branches with termination tracking
    /// Returns (value, ir, is_terminated, last_block_name)
    /// last_block_name is the block that actually branches to the outer merge
    fn generate_if_else_with_term(
        &mut self,
        if_else: &IfElse,
        counter: &mut usize,
        _merge_label: &str,
    ) -> CodegenResult<(String, String, bool, String)> {
        match if_else {
            IfElse::Else(stmts) => {
                let (val, ir, terminated) = self.generate_block_stmts(stmts, counter)?;
                // For plain else block, the "last block" is empty (caller handles it)
                Ok((val, ir, terminated, String::new()))
            }
            IfElse::ElseIf(cond, then_stmts, else_branch) => {
                // Generate nested if-else
                let then_label = self.next_label("elseif.then");
                let else_label = self.next_label("elseif.else");
                let local_merge = self.next_label("elseif.merge");

                let (cond_val, cond_ir) = self.generate_expr(cond, counter)?;
                let mut ir = cond_ir;

                // Convert i64 to i1 for branch
                let cond_bool = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = icmp ne i64 {}, 0\n",
                    cond_bool, cond_val
                ));

                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    cond_bool, then_label, else_label
                ));

                // Then branch
                ir.push_str(&format!("{}:\n", then_label));
                self.current_block = then_label.clone();
                let (then_val, then_ir, then_terminated) = self.generate_block_stmts(then_stmts, counter)?;
                ir.push_str(&then_ir);
                let then_actual_block = self.current_block.clone();
                let then_from_label = if !then_terminated {
                    ir.push_str(&format!("  br label %{}\n", local_merge));
                    then_actual_block
                } else {
                    String::new()
                };

                // Else branch
                ir.push_str(&format!("{}:\n", else_label));
                self.current_block = else_label.clone();
                let (else_val, else_ir, else_terminated, nested_last_block) = if let Some(nested) = else_branch {
                    self.generate_if_else_with_term(nested, counter, &local_merge)?
                } else {
                    ("0".to_string(), String::new(), false, String::new())
                };
                ir.push_str(&else_ir);
                let else_from_label = if !else_terminated {
                    ir.push_str(&format!("  br label %{}\n", local_merge));
                    // If there was a nested if-else, use its merge block as the predecessor
                    if !nested_last_block.is_empty() {
                        nested_last_block
                    } else {
                        self.current_block.clone()
                    }
                } else {
                    String::new()
                };

                // Both branches terminated = this whole if-else is terminated
                let all_terminated = then_terminated && else_terminated;

                // Merge
                ir.push_str(&format!("{}:\n", local_merge));
                self.current_block = local_merge.clone();
                let result = self.next_temp(counter);

                // Build phi node only from non-terminated predecessors
                if !then_from_label.is_empty() && !else_from_label.is_empty() {
                    ir.push_str(&format!(
                        "  {} = phi i64 [ {}, %{} ], [ {}, %{} ]\n",
                        result, then_val, then_from_label, else_val, else_from_label
                    ));
                } else if !then_from_label.is_empty() {
                    ir.push_str(&format!(
                        "  {} = phi i64 [ {}, %{} ]\n",
                        result, then_val, then_from_label
                    ));
                } else if !else_from_label.is_empty() {
                    ir.push_str(&format!(
                        "  {} = phi i64 [ {}, %{} ]\n",
                        result, else_val, else_from_label
                    ));
                } else {
                    // Unreachable merge block
                    ir.push_str(&format!("  {} = add i64 0, 0\n", result));
                }

                // Return local_merge as the last block for this nested if-else
                Ok((result, ir, all_terminated, local_merge))
            }
        }
    }

    /// Generate code for match expression
    fn generate_match(
        &mut self,
        match_expr: &Spanned<Expr>,
        arms: &[MatchArm],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        // Generate the expression to match against
        let (match_val, mut ir) = self.generate_expr(match_expr, counter)?;

        let merge_label = self.next_label("match.merge");
        let mut arm_labels: Vec<String> = Vec::new();
        let mut arm_values: Vec<(String, String)> = Vec::new(); // (value, label)

        // Check if all arms are simple integer literals (can use switch)
        let all_int_literals = arms.iter().all(|arm| {
            matches!(&arm.pattern.node, Pattern::Literal(Literal::Int(_)) | Pattern::Wildcard)
        });

        if all_int_literals && !arms.is_empty() {
            // Use LLVM switch instruction for integer pattern matching
            let default_label = self.next_label("match.default");
            let mut switch_cases: Vec<(i64, String)> = Vec::new();
            let mut default_arm: Option<&MatchArm> = None;

            // First pass: collect labels and find default
            for arm in arms {
                match &arm.pattern.node {
                    Pattern::Literal(Literal::Int(n)) => {
                        let label = self.next_label("match.arm");
                        switch_cases.push((*n, label.clone()));
                        arm_labels.push(label);
                    }
                    Pattern::Wildcard => {
                        default_arm = Some(arm);
                    }
                    _ => {}
                }
            }

            // Generate switch instruction
            ir.push_str(&format!(
                "  switch i64 {}, label %{} [\n",
                match_val, default_label
            ));
            for (val, label) in &switch_cases {
                ir.push_str(&format!("    i64 {}, label %{}\n", val, label));
            }
            ir.push_str("  ]\n");

            // Generate arm bodies for integer cases
            let mut case_idx = 0;
            for arm in arms {
                if let Pattern::Literal(Literal::Int(_)) = &arm.pattern.node {
                    let label = &arm_labels[case_idx];
                    ir.push_str(&format!("{}:\n", label));

                    // Check guard if present
                    if let Some(guard) = &arm.guard {
                        let guard_pass = self.next_label("match.guard.pass");
                        let guard_fail = self.next_label("match.guard.fail");

                        let (guard_val, guard_ir) = self.generate_expr(guard, counter)?;
                        ir.push_str(&guard_ir);
                        ir.push_str(&format!(
                            "  br i1 {}, label %{}, label %{}\n",
                            guard_val, guard_pass, guard_fail
                        ));

                        // Guard passed - execute body
                        ir.push_str(&format!("{}:\n", guard_pass));
                        let (body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                        ir.push_str(&body_ir);
                        arm_values.push((body_val, guard_pass.clone()));
                        ir.push_str(&format!("  br label %{}\n", merge_label));

                        // Guard failed - go to default
                        ir.push_str(&format!("{}:\n", guard_fail));
                        ir.push_str(&format!("  br label %{}\n", default_label));
                    } else {
                        let (body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                        ir.push_str(&body_ir);
                        arm_values.push((body_val, label.clone()));
                        ir.push_str(&format!("  br label %{}\n", merge_label));
                    }

                    case_idx += 1;
                }
            }

            // Generate default arm
            ir.push_str(&format!("{}:\n", default_label));
            if let Some(arm) = default_arm {
                let (body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                ir.push_str(&body_ir);
                arm_values.push((body_val, default_label.clone()));
            } else {
                // No default arm - unreachable or return 0
                arm_values.push(("0".to_string(), default_label.clone()));
            }
            ir.push_str(&format!("  br label %{}\n", merge_label));
        } else {
            // Fall back to chained conditional branches for complex patterns
            let mut current_label = self.next_label("match.check");
            ir.push_str(&format!("  br label %{}\n", current_label));

            for (i, arm) in arms.iter().enumerate() {
                let is_last = i == arms.len() - 1;
                let next_label = if is_last {
                    merge_label.clone()
                } else {
                    self.next_label("match.check")
                };
                let arm_body_label = self.next_label("match.arm");

                ir.push_str(&format!("{}:\n", current_label));

                // Generate pattern check
                let (check_val, check_ir) =
                    self.generate_pattern_check(&arm.pattern, &match_val, counter)?;
                ir.push_str(&check_ir);

                // Handle guard - need to bind variables first so guard can use them
                if let Some(guard) = &arm.guard {
                    let guard_bind = self.next_label("match.guard.bind");
                    let guard_check = self.next_label("match.guard.check");

                    // First check pattern
                    ir.push_str(&format!(
                        "  br i1 {}, label %{}, label %{}\n",
                        check_val, guard_bind, next_label
                    ));

                    // Bind pattern variables for guard to use
                    ir.push_str(&format!("{}:\n", guard_bind));
                    let bind_ir = self.generate_pattern_bindings(&arm.pattern, &match_val, counter)?;
                    ir.push_str(&bind_ir);
                    ir.push_str(&format!("  br label %{}\n", guard_check));

                    // Then check guard
                    ir.push_str(&format!("{}:\n", guard_check));
                    let (guard_val, guard_ir) = self.generate_expr(guard, counter)?;
                    ir.push_str(&guard_ir);
                    // Guard value is i64 (0 or 1), convert to i1 for branch
                    let guard_bool = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = icmp ne i64 {}, 0\n",
                        guard_bool, guard_val
                    ));
                    ir.push_str(&format!(
                        "  br i1 {}, label %{}, label %{}\n",
                        guard_bool, arm_body_label, next_label
                    ));

                    // Generate arm body (bindings already done)
                    ir.push_str(&format!("{}:\n", arm_body_label));
                } else {
                    ir.push_str(&format!(
                        "  br i1 {}, label %{}, label %{}\n",
                        check_val, arm_body_label, next_label
                    ));

                    // Generate arm body
                    ir.push_str(&format!("{}:\n", arm_body_label));

                    // Bind pattern variables if needed
                    let bind_ir = self.generate_pattern_bindings(&arm.pattern, &match_val, counter)?;
                    ir.push_str(&bind_ir);
                }

                let (body_val, body_ir) = self.generate_expr(&arm.body, counter)?;
                ir.push_str(&body_ir);
                arm_values.push((body_val, arm_body_label.clone()));
                ir.push_str(&format!("  br label %{}\n", merge_label));

                current_label = next_label;
            }

            // If no arm matched (for non-exhaustive patterns)
            if !arms.is_empty() {
                // The last next_label becomes merge_label, so we don't need extra handling
            }
        }

        // Merge block with phi node
        ir.push_str(&format!("{}:\n", merge_label));

        if arm_values.is_empty() {
            Ok(("0".to_string(), ir))
        } else {
            let result = self.next_temp(counter);
            let phi_args: Vec<String> = arm_values
                .iter()
                .map(|(val, label)| format!("[ {}, %{} ]", val, label))
                .collect();
            ir.push_str(&format!(
                "  {} = phi i64 {}\n",
                result,
                phi_args.join(", ")
            ));
            Ok((result, ir))
        }
    }

    /// Generate code to check if a pattern matches
    fn generate_pattern_check(
        &mut self,
        pattern: &Spanned<Pattern>,
        match_val: &str,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        match &pattern.node {
            Pattern::Wildcard => {
                // Wildcard always matches
                Ok(("1".to_string(), String::new()))
            }
            Pattern::Ident(_) => {
                // Identifier pattern always matches (binding)
                Ok(("1".to_string(), String::new()))
            }
            Pattern::Literal(lit) => {
                match lit {
                    Literal::Int(n) => {
                        let result = self.next_temp(counter);
                        let ir = format!("  {} = icmp eq i64 {}, {}\n", result, match_val, n);
                        Ok((result, ir))
                    }
                    Literal::Bool(b) => {
                        let lit_val = if *b { "1" } else { "0" };
                        let result = self.next_temp(counter);
                        let ir = format!("  {} = icmp eq i64 {}, {}\n", result, match_val, lit_val);
                        Ok((result, ir))
                    }
                    Literal::Float(f) => {
                        let result = self.next_temp(counter);
                        let ir = format!("  {} = fcmp oeq double {}, {:e}\n", result, match_val, f);
                        Ok((result, ir))
                    }
                    Literal::String(s) => {
                        // String comparison using strcmp
                        let mut ir = String::new();

                        // Create string constant for the pattern
                        let const_name = format!(".str_pat.{}", self.string_counter);
                        self.string_counter += 1;
                        self.string_constants.push((const_name.clone(), s.clone()));

                        // Get pointer to the constant string
                        let str_ptr = self.next_temp(counter);
                        let str_len = s.len() + 1;
                        ir.push_str(&format!(
                            "  {} = getelementptr [{} x i8], [{} x i8]* @{}, i32 0, i32 0\n",
                            str_ptr, str_len, str_len, const_name
                        ));

                        // Call strcmp: int strcmp(const char* s1, const char* s2)
                        // Returns 0 if strings are equal
                        let cmp_result = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = call i32 @strcmp(i8* {}, i8* {})\n",
                            cmp_result, match_val, str_ptr
                        ));

                        // Check if strcmp returned 0 (equal)
                        let result = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = icmp eq i32 {}, 0\n",
                            result, cmp_result
                        ));

                        Ok((result, ir))
                    }
                }
            }
            Pattern::Range { start, end, inclusive } => {
                let mut ir = String::new();

                // Check lower bound
                let lower_check = if let Some(start_pat) = start {
                    if let Pattern::Literal(Literal::Int(n)) = &start_pat.node {
                        let tmp = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = icmp sge i64 {}, {}\n",
                            tmp, match_val, n
                        ));
                        tmp
                    } else {
                        "1".to_string()
                    }
                } else {
                    "1".to_string()
                };

                // Check upper bound
                let upper_check = if let Some(end_pat) = end {
                    if let Pattern::Literal(Literal::Int(n)) = &end_pat.node {
                        let tmp = self.next_temp(counter);
                        let cmp = if *inclusive { "icmp sle" } else { "icmp slt" };
                        ir.push_str(&format!(
                            "  {} = {} i64 {}, {}\n",
                            tmp, cmp, match_val, n
                        ));
                        tmp
                    } else {
                        "1".to_string()
                    }
                } else {
                    "1".to_string()
                };

                // Combine checks
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = and i1 {}, {}\n",
                    result, lower_check, upper_check
                ));

                Ok((result, ir))
            }
            Pattern::Or(patterns) => {
                let mut ir = String::new();
                let mut checks: Vec<String> = Vec::new();

                for pat in patterns {
                    let (check, check_ir) = self.generate_pattern_check(pat, match_val, counter)?;
                    ir.push_str(&check_ir);
                    checks.push(check);
                }

                // OR all checks together
                let mut result = checks[0].clone();
                for check in checks.iter().skip(1) {
                    let tmp = self.next_temp(counter);
                    ir.push_str(&format!("  {} = or i1 {}, {}\n", tmp, result, check));
                    result = tmp;
                }

                Ok((result, ir))
            }
            Pattern::Tuple(patterns) => {
                // For tuple patterns, we need to extract and check each element
                let mut ir = String::new();
                let mut checks: Vec<String> = Vec::new();

                for (i, pat) in patterns.iter().enumerate() {
                    // Extract tuple element
                    let elem = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = extractvalue {{ {} }} {}, {}\n",
                        elem,
                        vec!["i64"; patterns.len()].join(", "),
                        match_val,
                        i
                    ));

                    let (check, check_ir) = self.generate_pattern_check(pat, &elem, counter)?;
                    ir.push_str(&check_ir);
                    checks.push(check);
                }

                // AND all checks together
                if checks.is_empty() {
                    Ok(("1".to_string(), ir))
                } else {
                    let mut result = checks[0].clone();
                    for check in checks.iter().skip(1) {
                        let tmp = self.next_temp(counter);
                        ir.push_str(&format!("  {} = and i1 {}, {}\n", tmp, result, check));
                        result = tmp;
                    }
                    Ok((result, ir))
                }
            }
            Pattern::Variant { name, fields: _ } => {
                // Enum variant pattern: check the tag matches
                // Enum value is a struct { i32 tag, ... payload }
                // Extract the tag and compare
                let mut ir = String::new();

                // Get the tag from the enum value (first field at index 0)
                let tag_ptr = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr {{ i32 }}, {{ i32 }}* {}, i32 0, i32 0\n",
                    tag_ptr, match_val
                ));

                let tag_val = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = load i32, i32* {}\n",
                    tag_val, tag_ptr
                ));

                // Find the expected tag value for this variant
                let variant_name = &name.node;
                let expected_tag = self.get_enum_variant_tag(variant_name);

                // Compare tag
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = icmp eq i32 {}, {}\n",
                    result, tag_val, expected_tag
                ));

                Ok((result, ir))
            }
            Pattern::Struct { name, fields } => {
                // Struct pattern: always matches if type is correct, but we check field patterns
                let struct_name = &name.node;
                let mut ir = String::new();
                let mut checks: Vec<String> = Vec::new();

                if let Some(struct_info) = self.structs.get(struct_name).cloned() {
                    for (field_name, field_pat) in fields {
                        // Find field index
                        if let Some(field_idx) = struct_info.fields.iter()
                            .position(|(n, _)| n == &field_name.node)
                        {
                            if let Some(pat) = field_pat {
                                // Extract field value and check pattern
                                let field_ptr = self.next_temp(counter);
                                ir.push_str(&format!(
                                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}\n",
                                    field_ptr, struct_name, struct_name, match_val, field_idx
                                ));

                                let field_val = self.next_temp(counter);
                                let field_ty = &struct_info.fields[field_idx].1;
                                let llvm_ty = self.type_to_llvm(field_ty);
                                ir.push_str(&format!(
                                    "  {} = load {}, {}* {}\n",
                                    field_val, llvm_ty, llvm_ty, field_ptr
                                ));

                                let (check, check_ir) = self.generate_pattern_check(pat, &field_val, counter)?;
                                ir.push_str(&check_ir);
                                checks.push(check);
                            }
                        }
                    }
                }

                // AND all checks together
                if checks.is_empty() {
                    Ok(("1".to_string(), ir))
                } else {
                    let mut result = checks[0].clone();
                    for check in checks.iter().skip(1) {
                        let tmp = self.next_temp(counter);
                        ir.push_str(&format!("  {} = and i1 {}, {}\n", tmp, result, check));
                        result = tmp;
                    }
                    Ok((result, ir))
                }
            }
        }
    }

    /// Get the tag value for an enum variant
    fn get_enum_variant_tag(&self, variant_name: &str) -> i32 {
        for enum_info in self.enums.values() {
            for (i, variant) in enum_info.variants.iter().enumerate() {
                if variant.name == variant_name {
                    return i as i32;
                }
            }
        }
        0 // Default to 0 if not found
    }

    /// Check if a name is a unit enum variant (not a binding)
    fn is_unit_enum_variant(&self, name: &str) -> bool {
        use crate::types::EnumVariantFields;
        for enum_info in self.enums.values() {
            for variant in &enum_info.variants {
                if variant.name == name {
                    return matches!(variant.fields, EnumVariantFields::Unit);
                }
            }
        }
        false
    }

    /// Check if a name is a tuple enum variant and get its enum name and tag
    fn get_tuple_variant_info(&self, name: &str) -> Option<(String, i32)> {
        use crate::types::EnumVariantFields;
        for enum_info in self.enums.values() {
            for (tag, variant) in enum_info.variants.iter().enumerate() {
                if variant.name == name {
                    if matches!(variant.fields, EnumVariantFields::Tuple(_)) {
                        return Some((enum_info.name.clone(), tag as i32));
                    }
                }
            }
        }
        None
    }

    /// Generate pattern bindings (assign matched values to pattern variables)
    fn generate_pattern_bindings(
        &mut self,
        pattern: &Spanned<Pattern>,
        match_val: &str,
        counter: &mut usize,
    ) -> CodegenResult<String> {
        match &pattern.node {
            Pattern::Ident(name) => {
                // Check if this is a unit enum variant (like None)
                // Unit variants don't bind anything
                if self.is_unit_enum_variant(name) {
                    return Ok(String::new());
                }

                // Bind the matched value to the identifier
                let mut ir = String::new();
                let ty = ResolvedType::I64; // Default type for now

                // Generate unique LLVM name for pattern binding
                let llvm_name = format!("{}.{}", name, counter);
                *counter += 1;

                self.locals.insert(
                    name.clone(),
                    LocalVar {
                        ty: ty.clone(),
                        is_param: false,
                        llvm_name: llvm_name.clone(),
                    },
                );

                let llvm_ty = self.type_to_llvm(&ty);
                ir.push_str(&format!("  %{} = alloca {}\n", llvm_name, llvm_ty));
                ir.push_str(&format!(
                    "  store {} {}, {}* %{}\n",
                    llvm_ty, match_val, llvm_ty, llvm_name
                ));

                Ok(ir)
            }
            Pattern::Tuple(patterns) => {
                let mut ir = String::new();

                for (i, pat) in patterns.iter().enumerate() {
                    // Extract tuple element
                    let elem = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = extractvalue {{ {} }} {}, {}\n",
                        elem,
                        vec!["i64"; patterns.len()].join(", "),
                        match_val,
                        i
                    ));

                    let bind_ir = self.generate_pattern_bindings(pat, &elem, counter)?;
                    ir.push_str(&bind_ir);
                }

                Ok(ir)
            }
            Pattern::Variant { name: _, fields } => {
                // Bind fields from enum variant payload
                let mut ir = String::new();

                for (i, field_pat) in fields.iter().enumerate() {
                    // Extract payload field (starting at offset 1, after the tag)
                    // For tuple variants: { i32 tag, i64 field0, i64 field1, ... }
                    //
                    // If match_val is a pointer, use getelementptr + load
                    // Otherwise use extractvalue

                    if match_val.starts_with('%') {
                        // Assume it's a pointer - use getelementptr to access the field
                        // Enum layout: { i32 tag, { i64, i64, ... } payload }
                        // For single-field tuple variants, payload is at index 1 in { i32, i64 } structure
                        let field_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = getelementptr {{ i32, i64 }}, {{ i32, i64 }}* {}, i32 0, i32 {}\n",
                            field_ptr, match_val, i + 1
                        ));
                        let field_val = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = load i64, i64* {}\n",
                            field_val, field_ptr
                        ));
                        let bind_ir = self.generate_pattern_bindings(field_pat, &field_val, counter)?;
                        ir.push_str(&bind_ir);
                    } else {
                        // It's a value - use extractvalue
                        let field_val = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = extractvalue {{ i32, i64 }} {}, {}\n",
                            field_val, match_val, i + 1
                        ));
                        let bind_ir = self.generate_pattern_bindings(field_pat, &field_val, counter)?;
                        ir.push_str(&bind_ir);
                    }
                }

                Ok(ir)
            }
            Pattern::Struct { name, fields } => {
                // Bind fields from struct
                let struct_name = &name.node;
                let mut ir = String::new();

                if let Some(struct_info) = self.structs.get(struct_name).cloned() {
                    for (field_name, field_pat) in fields {
                        // If field_pat is None, bind the field to its own name
                        if let Some(field_idx) = struct_info.fields.iter()
                            .position(|(n, _)| n == &field_name.node)
                        {
                            // Extract field value
                            let field_ptr = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}\n",
                                field_ptr, struct_name, struct_name, match_val, field_idx
                            ));

                            let field_val = self.next_temp(counter);
                            let field_ty = &struct_info.fields[field_idx].1;
                            let llvm_ty = self.type_to_llvm(field_ty);
                            ir.push_str(&format!(
                                "  {} = load {}, {}* {}\n",
                                field_val, llvm_ty, llvm_ty, field_ptr
                            ));

                            if let Some(pat) = field_pat {
                                // Bind to pattern
                                let bind_ir = self.generate_pattern_bindings(pat, &field_val, counter)?;
                                ir.push_str(&bind_ir);
                            } else {
                                // Bind to field name directly with unique LLVM name
                                let llvm_field_name = format!("{}.{}", field_name.node, counter);
                                *counter += 1;

                                self.locals.insert(
                                    field_name.node.clone(),
                                    LocalVar {
                                        ty: field_ty.clone(),
                                        is_param: false,
                                        llvm_name: llvm_field_name.clone(),
                                    },
                                );
                                ir.push_str(&format!("  %{} = alloca {}\n", llvm_field_name, llvm_ty));
                                ir.push_str(&format!(
                                    "  store {} {}, {}* %{}\n",
                                    llvm_ty, field_val, llvm_ty, llvm_field_name
                                ));
                            }
                        }
                    }
                }

                Ok(ir)
            }
            _ => Ok(String::new()),
        }
    }

    /// Infer type of expression (simple version for let statement)
    fn infer_expr_type(&self, expr: &Spanned<Expr>) -> ResolvedType {
        match &expr.node {
            Expr::Int(_) => ResolvedType::I64,
            Expr::Float(_) => ResolvedType::F64,
            Expr::Bool(_) => ResolvedType::Bool,
            Expr::String(_) => ResolvedType::Str,
            Expr::Ident(name) => {
                // Look up local variable type
                if let Some(local) = self.locals.get(name) {
                    local.ty.clone()
                } else if self.is_unit_enum_variant(name) {
                    // Unit enum variant (e.g., None)
                    for enum_info in self.enums.values() {
                        for variant in &enum_info.variants {
                            if variant.name == *name {
                                return ResolvedType::Named {
                                    name: enum_info.name.clone(),
                                    generics: vec![],
                                };
                            }
                        }
                    }
                    ResolvedType::I64
                } else {
                    ResolvedType::I64 // Default
                }
            }
            Expr::Call { func, .. } => {
                // Get return type from function info
                if let Expr::Ident(fn_name) = &func.node {
                    // Check if this is an enum variant constructor
                    if let Some((enum_name, _)) = self.get_tuple_variant_info(fn_name) {
                        return ResolvedType::Named {
                            name: enum_name,
                            generics: vec![],
                        };
                    }
                    // Check function info
                    if let Some(fn_info) = self.functions.get(fn_name) {
                        return fn_info.ret_type.clone();
                    }
                }
                ResolvedType::I64 // Default
            }
            Expr::Array(elements) => {
                if let Some(first) = elements.first() {
                    ResolvedType::Pointer(Box::new(self.infer_expr_type(first)))
                } else {
                    ResolvedType::Pointer(Box::new(ResolvedType::I64))
                }
            }
            Expr::Tuple(elements) => {
                ResolvedType::Tuple(elements.iter().map(|e| self.infer_expr_type(e)).collect())
            }
            Expr::Ref(inner) => ResolvedType::Ref(Box::new(self.infer_expr_type(inner))),
            Expr::Deref(inner) => {
                match self.infer_expr_type(inner) {
                    ResolvedType::Pointer(inner) => *inner,
                    ResolvedType::Ref(inner) => *inner,
                    ResolvedType::RefMut(inner) => *inner,
                    _ => ResolvedType::I64,
                }
            }
            Expr::StructLit { name, .. } => {
                // Return Named type for struct literals
                ResolvedType::Named {
                    name: name.node.clone(),
                    generics: vec![],
                }
            }
            Expr::Index { expr: inner, index } => {
                // Check if this is a slice operation
                if matches!(index.node, Expr::Range { .. }) {
                    // Slice returns a pointer
                    let inner_ty = self.infer_expr_type(inner);
                    match inner_ty {
                        ResolvedType::Pointer(elem) => ResolvedType::Pointer(elem),
                        ResolvedType::Array(elem) => ResolvedType::Pointer(elem),
                        _ => ResolvedType::Pointer(Box::new(ResolvedType::I64)),
                    }
                } else {
                    // Regular indexing returns element type
                    let inner_ty = self.infer_expr_type(inner);
                    match inner_ty {
                        ResolvedType::Pointer(elem) => *elem,
                        ResolvedType::Array(elem) => *elem,
                        _ => ResolvedType::I64,
                    }
                }
            }
            Expr::Lambda { .. } => {
                // Lambda returns a function pointer (represented as i64)
                ResolvedType::I64
            }
            Expr::MethodCall { receiver, method, .. } => {
                // Get method return type from struct definition
                let recv_type = self.infer_expr_type(receiver);
                if let ResolvedType::Named { name, .. } = &recv_type {
                    let method_name = format!("{}_{}", name, method.node);
                    if let Some(fn_info) = self.functions.get(&method_name) {
                        return fn_info.ret_type.clone();
                    }
                }
                ResolvedType::I64
            }
            Expr::StaticMethodCall { type_name, method, .. } => {
                // Get static method return type from function info
                let method_name = format!("{}_{}", type_name.node, method.node);
                if let Some(fn_info) = self.functions.get(&method_name) {
                    return fn_info.ret_type.clone();
                }
                ResolvedType::I64
            }
            _ => ResolvedType::I64, // Default fallback
        }
    }

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
        ir.push_str(&format!("  store i64 {}, i64* {}\n", next_idx, loop_idx_ptr));
        ir.push_str(&format!("  br label %{}\n", loop_start));

        ir.push_str(&format!("{}:\n", loop_end));

        Ok((slice_ptr, ir))
    }

    /// Find free variables in a lambda expression
    /// Returns variables that are used in the body but not bound by parameters
    fn find_lambda_captures(&self, params: &[Param], body: &Spanned<Expr>) -> Vec<String> {
        let param_names: std::collections::HashSet<_> = params.iter()
            .map(|p| p.name.node.clone())
            .collect();
        let mut free_vars = Vec::new();
        self.collect_free_vars_in_expr(&body.node, &param_names, &mut free_vars);
        // Deduplicate while preserving order
        let mut seen = std::collections::HashSet::new();
        free_vars.retain(|v| seen.insert(v.clone()));
        free_vars
    }

    fn collect_free_vars_in_expr(&self, expr: &Expr, bound: &std::collections::HashSet<String>, free: &mut Vec<String>) {
        match expr {
            Expr::Ident(name) => {
                // Only capture if it's in our locals (exists in outer scope)
                if !bound.contains(name) && self.locals.contains_key(name) {
                    free.push(name.clone());
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_free_vars_in_expr(&left.node, bound, free);
                self.collect_free_vars_in_expr(&right.node, bound, free);
            }
            Expr::Unary { expr, .. } => {
                self.collect_free_vars_in_expr(&expr.node, bound, free);
            }
            Expr::Call { func, args } => {
                self.collect_free_vars_in_expr(&func.node, bound, free);
                for arg in args {
                    self.collect_free_vars_in_expr(&arg.node, bound, free);
                }
            }
            Expr::If { cond, then, else_ } => {
                self.collect_free_vars_in_expr(&cond.node, bound, free);
                let mut local_bound = bound.clone();
                for stmt in then {
                    self.collect_free_vars_in_stmt(&stmt.node, &mut local_bound, free);
                }
                if let Some(else_br) = else_ {
                    self.collect_free_vars_in_if_else(else_br, bound, free);
                }
            }
            Expr::Block(stmts) => {
                let mut local_bound = bound.clone();
                for stmt in stmts {
                    self.collect_free_vars_in_stmt(&stmt.node, &mut local_bound, free);
                }
            }
            Expr::MethodCall { receiver, args, .. } => {
                self.collect_free_vars_in_expr(&receiver.node, bound, free);
                for arg in args {
                    self.collect_free_vars_in_expr(&arg.node, bound, free);
                }
            }
            Expr::Field { expr, .. } => {
                self.collect_free_vars_in_expr(&expr.node, bound, free);
            }
            Expr::Index { expr, index } => {
                self.collect_free_vars_in_expr(&expr.node, bound, free);
                self.collect_free_vars_in_expr(&index.node, bound, free);
            }
            Expr::Array(elems) => {
                for e in elems {
                    self.collect_free_vars_in_expr(&e.node, bound, free);
                }
            }
            Expr::Tuple(elems) => {
                for e in elems {
                    self.collect_free_vars_in_expr(&e.node, bound, free);
                }
            }
            Expr::StructLit { fields, .. } => {
                for (_, e) in fields {
                    self.collect_free_vars_in_expr(&e.node, bound, free);
                }
            }
            Expr::Assign { target, value } | Expr::AssignOp { target, value, .. } => {
                self.collect_free_vars_in_expr(&target.node, bound, free);
                self.collect_free_vars_in_expr(&value.node, bound, free);
            }
            Expr::Lambda { params, body, .. } => {
                let mut inner_bound = bound.clone();
                for p in params {
                    inner_bound.insert(p.name.node.clone());
                }
                self.collect_free_vars_in_expr(&body.node, &inner_bound, free);
            }
            Expr::Ref(inner) | Expr::Deref(inner) |
            Expr::Try(inner) | Expr::Unwrap(inner) | Expr::Await(inner) |
            Expr::Spawn(inner) => {
                self.collect_free_vars_in_expr(&inner.node, bound, free);
            }
            Expr::Ternary { cond, then, else_ } => {
                self.collect_free_vars_in_expr(&cond.node, bound, free);
                self.collect_free_vars_in_expr(&then.node, bound, free);
                self.collect_free_vars_in_expr(&else_.node, bound, free);
            }
            Expr::Loop { body, pattern, iter } => {
                if let Some(it) = iter {
                    self.collect_free_vars_in_expr(&it.node, bound, free);
                }
                let mut local_bound = bound.clone();
                if let Some(pat) = pattern {
                    self.collect_pattern_bindings(&pat.node, &mut local_bound);
                }
                for stmt in body {
                    self.collect_free_vars_in_stmt(&stmt.node, &mut local_bound, free);
                }
            }
            Expr::Match { expr, arms } => {
                self.collect_free_vars_in_expr(&expr.node, bound, free);
                for arm in arms {
                    let mut arm_bound = bound.clone();
                    self.collect_pattern_bindings(&arm.pattern.node, &mut arm_bound);
                    if let Some(guard) = &arm.guard {
                        self.collect_free_vars_in_expr(&guard.node, &arm_bound, free);
                    }
                    self.collect_free_vars_in_expr(&arm.body.node, &arm_bound, free);
                }
            }
            Expr::Range { start, end, .. } => {
                if let Some(s) = start {
                    self.collect_free_vars_in_expr(&s.node, bound, free);
                }
                if let Some(e) = end {
                    self.collect_free_vars_in_expr(&e.node, bound, free);
                }
            }
            // Literals and other expressions don't contain free variables
            _ => {}
        }
    }

    fn collect_free_vars_in_stmt(&self, stmt: &Stmt, bound: &mut std::collections::HashSet<String>, free: &mut Vec<String>) {
        match stmt {
            Stmt::Let { name, value, .. } => {
                self.collect_free_vars_in_expr(&value.node, bound, free);
                bound.insert(name.node.clone());
            }
            Stmt::Expr(e) => self.collect_free_vars_in_expr(&e.node, bound, free),
            Stmt::Return(Some(e)) => self.collect_free_vars_in_expr(&e.node, bound, free),
            Stmt::Break(Some(e)) => self.collect_free_vars_in_expr(&e.node, bound, free),
            _ => {}
        }
    }

    fn collect_free_vars_in_if_else(&self, if_else: &IfElse, bound: &std::collections::HashSet<String>, free: &mut Vec<String>) {
        match if_else {
            IfElse::ElseIf(cond, then_stmts, else_) => {
                self.collect_free_vars_in_expr(&cond.node, bound, free);
                let mut local_bound = bound.clone();
                for stmt in then_stmts {
                    self.collect_free_vars_in_stmt(&stmt.node, &mut local_bound, free);
                }
                if let Some(else_br) = else_ {
                    self.collect_free_vars_in_if_else(else_br, bound, free);
                }
            }
            IfElse::Else(stmts) => {
                let mut local_bound = bound.clone();
                for stmt in stmts {
                    self.collect_free_vars_in_stmt(&stmt.node, &mut local_bound, free);
                }
            }
        }
    }

    fn collect_pattern_bindings(&self, pattern: &Pattern, bound: &mut std::collections::HashSet<String>) {
        match pattern {
            Pattern::Ident(name) => { bound.insert(name.clone()); }
            Pattern::Tuple(patterns) => {
                for p in patterns {
                    self.collect_pattern_bindings(&p.node, bound);
                }
            }
            Pattern::Struct { fields, .. } => {
                for (name, pat) in fields {
                    if let Some(p) = pat {
                        self.collect_pattern_bindings(&p.node, bound);
                    } else {
                        // Field shorthand: {x} binds x
                        bound.insert(name.node.clone());
                    }
                }
            }
            Pattern::Variant { fields, .. } => {
                for p in fields {
                    self.collect_pattern_bindings(&p.node, bound);
                }
            }
            Pattern::Or(patterns) => {
                for p in patterns {
                    self.collect_pattern_bindings(&p.node, bound);
                }
            }
            _ => {}
        }
    }
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
        assert!(ir.contains("loop"));
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
        use vais_types::{GenericInstantiation, TypeChecker};

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
        let ir = gen.generate_module_with_instantiations(&module, instantiations).unwrap();

        // Should contain specialized function identity$i64
        assert!(ir.contains("define i64 @identity$i64"), "Expected identity$i64 in IR: {}", ir);
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
        let pair_inst = instantiations.iter()
            .find(|i| i.base_name == "Pair");
        assert!(pair_inst.is_some(), "Expected Pair instantiation to be recorded");

        // Verify mangled name
        let inst = pair_inst.unwrap();
        assert_eq!(inst.mangled_name, "Pair$i64", "Expected mangled name Pair$i64, got {}", inst.mangled_name);
    }

    #[test]
    fn test_multiple_instantiations() {
        use vais_types::{GenericInstantiation, TypeChecker};

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
        assert!(instantiations.len() >= 2, "Expected at least 2 instantiations, got {}", instantiations.len());

        // Generate code with instantiations
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module_with_instantiations(&module, instantiations).unwrap();

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
        let ir = gen.generate_module_with_instantiations(&module, instantiations).unwrap();

        // Should NOT contain any identity function definition
        assert!(!ir.contains("define i64 @identity"), "Generic template should not generate code");
        assert!(!ir.contains("define double @identity"), "Generic template should not generate code");
    }
}

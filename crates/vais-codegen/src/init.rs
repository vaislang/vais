//! CodeGenerator initialization and configuration methods

use super::*;

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
                declared_functions: std::collections::HashSet::with_capacity(64),
                functions: HashMap::with_capacity(64),
                structs: HashMap::with_capacity(16),
                enums: HashMap::with_capacity(8),
                unions: HashMap::new(),
                constants: HashMap::with_capacity(16),
                globals: HashMap::new(),
                trait_defs: HashMap::with_capacity(8),
                trait_aliases: HashMap::new(),
                trait_impl_methods: HashMap::with_capacity(16),
                resolved_function_sigs: HashMap::with_capacity(64),
                type_aliases: HashMap::with_capacity(16),
                default_params: HashMap::new(),
            },
            generics: GenericState {
                struct_defs: HashMap::new(),
                struct_aliases: HashMap::new(),
                generated_structs: HashMap::with_capacity(16),
                function_templates: HashMap::new(),
                fn_instantiations: HashMap::new(),
                generated_functions: HashMap::with_capacity(16),
                substitutions: HashMap::new(),
            },
            fn_ctx: FunctionContext {
                current_function: None,
                current_return_type: None,
                locals: HashMap::with_capacity(32),
                label_counter: 0,
                loop_stack: Vec::with_capacity(4),
                defer_stack: Vec::new(),
                current_block: String::from("entry"),
                current_file: None,
                future_poll_fns: HashMap::new(),
                async_poll_context: None,
                alloc_tracker: Vec::new(),
            },
            strings: StringPool {
                constants: Vec::with_capacity(16),
                counter: 0,
                prefix: None,
                dedup_cache: HashMap::with_capacity(32),
            },
            lambdas: LambdaState {
                generated_ir: Vec::new(),
                closures: HashMap::new(),
                last_lambda_info: None,
                async_state_counter: 0,
                async_await_points: Vec::new(),
                current_async_function: None,
                last_lazy_info: None,
                lazy_bindings: HashMap::new(),
            },
            module_name: module_name.to_string(),
            target,
            needs_unwrap_panic: false,
            needs_bounds_check: false,
            needs_sync_spawn_poll: false,
            needs_string_helpers: false,
            debug_info: DebugInfoBuilder::new(DebugConfig::default()),
            type_to_llvm_cache: std::cell::RefCell::new(HashMap::with_capacity(64)),
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
            last_error_span: None,
            multi_error_mode: false,
            collected_errors: Vec::new(),
            strict_type_mode: true,
            ident_pool: crate::string_pool::IdentPool::with_capacity(256),
            warnings: std::cell::RefCell::new(Vec::new()),
            ref_constants: Vec::new(),
            ref_constant_counter: 0,
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

    /// Set type aliases from the type checker (for resolving type alias names in codegen).
    pub fn set_type_aliases(&mut self, aliases: HashMap<String, vais_types::ResolvedType>) {
        self.types.type_aliases = aliases;
    }

    /// Set string prefix for per-module codegen (avoids .str.N collisions across modules)
    pub fn set_string_prefix(&mut self, prefix: &str) {
        self.strings.prefix = Some(prefix.to_string());
    }

    /// Set current source file for error messages
    pub fn set_source_file(&mut self, file: &str) {
        self.fn_ctx.current_file = Some(file.to_string());
    }

    /// Get the last expression span recorded during code generation.
    ///
    /// When a `CodegenError` occurs, this span points to the AST expression
    /// that was being processed at the time.  The compiler driver can use it
    /// to construct a [`SpannedCodegenError`] for rich diagnostics.
    pub fn last_error_span(&self) -> Option<Span> {
        self.last_error_span
    }

    /// Get collected codegen errors (when multi_error_mode is enabled).
    ///
    /// In multi-error mode, function body generation errors are collected
    /// instead of immediately halting compilation. This allows reporting
    /// multiple codegen errors at once.
    pub fn get_collected_errors(&self) -> &[SpannedCodegenError] {
        &self.collected_errors
    }

    /// Get structured warnings collected during code generation.
    ///
    /// Warnings indicate situations where the compiler made a best-effort
    /// decision (e.g., falling back to i64 for an unresolved generic parameter).
    /// They do not halt compilation but may signal suboptimal code generation.
    ///
    /// Returns a clone of the warnings vector since it is stored in a `RefCell`.
    pub fn get_warnings(&self) -> Vec<crate::CodegenWarning> {
        self.warnings.borrow().clone()
    }

    /// Enable strict type mode.
    ///
    /// In strict mode, ICE-level type fallbacks (`Var`, `Unknown`, `Lifetime`,
    /// `ImplTrait`, `HigherKinded` reaching codegen) become hard errors instead
    /// of warnings with i64 fallback. Generic/ConstGeneric fallbacks remain as
    /// warnings since they are legitimate during monomorphization.
    pub fn set_strict_type_mode(&mut self, strict: bool) {
        self.strict_type_mode = strict;
    }

    /// Record a structured codegen warning.
    ///
    /// Uses interior mutability (`RefCell`) so this can be called from `&self` methods
    /// such as `type_to_llvm` which cannot take `&mut self`.
    pub(crate) fn emit_warning(&self, warning: crate::CodegenWarning) {
        self.warnings.borrow_mut().push(warning);
    }

    /// Emit a warning, or return an error in strict type mode for ICE-level fallbacks.
    ///
    /// In strict mode, [`CodegenWarning::UnresolvedTypeFallback`] is promoted to
    /// [`CodegenError::InternalError`]. Other warning types (e.g., `GenericFallback`)
    /// remain warnings in all modes.
    pub(crate) fn emit_warning_or_error(
        &self,
        warning: crate::CodegenWarning,
    ) -> crate::CodegenResult<()> {
        if self.strict_type_mode {
            if let crate::CodegenWarning::UnresolvedTypeFallback {
                ref type_desc,
                ref backend,
            } = warning
            {
                return Err(crate::CodegenError::InternalError(format!(
                    "[strict] {} in {} codegen — i64 fallback disabled",
                    type_desc, backend
                )));
            }
        }
        self.emit_warning(warning);
        Ok(())
    }
}

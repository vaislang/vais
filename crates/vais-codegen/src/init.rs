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
                declared_functions: std::collections::HashSet::new(),
                functions: HashMap::new(),
                structs: HashMap::new(),
                enums: HashMap::new(),
                unions: HashMap::new(),
                constants: HashMap::new(),
                globals: HashMap::new(),
                trait_defs: HashMap::new(),
                trait_aliases: HashMap::new(),
                trait_impl_methods: HashMap::new(),
                resolved_function_sigs: HashMap::new(),
                type_aliases: HashMap::new(),
                default_params: HashMap::new(),
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
                last_lazy_info: None,
                lazy_bindings: HashMap::new(),
            },
            module_name: module_name.to_string(),
            target,
            needs_unwrap_panic: false,
            needs_sync_spawn_poll: false,
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
}

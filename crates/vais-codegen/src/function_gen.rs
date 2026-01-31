//! Function and method code generation for Vais
//!
//! This module contains functions for generating LLVM IR for functions,
//! async functions, methods, and specialized generic functions.

use crate::types::{FunctionInfo, LocalVar, StructInfo};
use crate::{AsyncFunctionInfo, CodeGenerator, CodegenResult};
use std::collections::HashMap;
use vais_ast::{Function, FunctionBody, GenericParamKind, Span, Struct};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Generate a specialized struct type from a generic struct template
    pub(crate) fn generate_specialized_struct_type(
        &mut self,
        generic_struct: &Struct,
        inst: &vais_types::GenericInstantiation,
        ir: &mut String,
    ) -> CodegenResult<()> {
        // Skip if already generated
        if self.generated_structs.contains_key(&inst.mangled_name) {
            return Ok(());
        }
        self.generated_structs
            .insert(inst.mangled_name.clone(), true);

        // Create substitution map from generic params to concrete types
        // Filter out lifetime params (they don't have runtime representation)
        let type_params: Vec<_> = generic_struct
            .generics
            .iter()
            .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
            .collect();
        let substitutions: HashMap<String, ResolvedType> = type_params
            .iter()
            .zip(inst.type_args.iter())
            .map(|(g, t)| (g.name.node.to_string(), t.clone()))
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
                (f.name.node.to_string(), concrete_ty)
            })
            .collect();

        let llvm_fields: Vec<String> = fields.iter().map(|(_, ty)| self.type_to_llvm(ty)).collect();

        ir.push_str(&format!(
            "%{} = type {{ {} }}\n",
            inst.mangled_name,
            llvm_fields.join(", ")
        ));

        // Register the specialized struct
        let struct_info = StructInfo {
            name: inst.mangled_name.to_string(),
            fields,
            repr_c: false,
            invariants: Vec::new(),
        };
        self.structs
            .insert(inst.mangled_name.to_string(), struct_info);

        // Register a name mapping from base name to mangled name
        // so struct literals and field accesses in generic impl methods can resolve it
        self.generic_struct_aliases
            .insert(inst.base_name.to_string(), inst.mangled_name.to_string());

        // Restore old substitutions
        self.generic_substitutions = old_subst;

        Ok(())
    }

    /// Generate a specialized function from a generic function template
    pub(crate) fn generate_specialized_function(
        &mut self,
        generic_fn: &Function,
        inst: &vais_types::GenericInstantiation,
    ) -> CodegenResult<String> {
        // Skip if already generated
        if self.generated_functions.contains_key(&inst.mangled_name) {
            return Ok(String::new());
        }
        self.generated_functions
            .insert(inst.mangled_name.clone(), true);

        // Create substitution map from generic params to concrete types
        // Filter out lifetime params (they don't have runtime representation)
        let type_params: Vec<_> = generic_fn
            .generics
            .iter()
            .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
            .collect();
        let substitutions: HashMap<String, ResolvedType> = type_params
            .iter()
            .zip(inst.type_args.iter())
            .map(|(g, t)| (g.name.node.to_string(), t.clone()))
            .collect();

        // Save and set generic substitutions
        let old_subst = std::mem::replace(&mut self.generic_substitutions, substitutions.clone());

        self.current_function = Some(inst.mangled_name.to_string());
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
                    p.name.node.to_string(),
                    LocalVar::param(concrete_ty, p.name.node.to_string()),
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
            inst.type_args
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join(", ")
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
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}\n",
                        loaded, ret_llvm, ret_llvm, value
                    ));
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
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}\n",
                        loaded, ret_llvm, ret_llvm, value
                    ));
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
    pub(crate) fn generate_helper_functions(&self) -> String {
        let mut ir = String::new();

        // Declare C library functions needed by runtime helpers
        // Note: exit and strlen are already declared by builtins
        ir.push_str("\n; C library function declarations\n");
        ir.push_str("declare i64 @write(i32, i8*, i64)\n");

        // Global constant for newline (used by panic functions)
        ir.push_str("\n; Global constants for runtime functions\n");
        ir.push_str("@.panic_newline = private unnamed_addr constant [2 x i8] c\"\\0A\\00\"\n");

        // __panic: runtime panic function (used by assert)
        // Prints message to stderr (fd=2) and exits with code 1
        ir.push_str("\n; Runtime panic function (used by assert)\n");
        ir.push_str("define i64 @__panic(i8* %msg) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  ; Calculate message length\n");
        ir.push_str("  %len = call i64 @strlen(i8* %msg)\n");
        ir.push_str("  ; Write message to stderr (fd=2)\n");
        ir.push_str("  %0 = call i64 @write(i32 2, i8* %msg, i64 %len)\n");
        ir.push_str("  ; Write newline\n");
        ir.push_str("  %1 = call i64 @write(i32 2, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @.panic_newline, i64 0, i64 0), i64 1)\n");
        ir.push_str("  call void @exit(i32 1)\n");
        ir.push_str("  unreachable\n");
        ir.push_str("}\n");

        // __contract_fail: runtime contract failure function
        // Prints contract failure message to stderr and exits with code 1
        ir.push_str("\n; Runtime contract failure function\n");
        ir.push_str("define i64 @__contract_fail(i64 %kind, i8* %condition, i8* %file, i64 %line, i8* %func) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  ; Calculate message length\n");
        ir.push_str("  %len = call i64 @strlen(i8* %condition)\n");
        ir.push_str("  ; Write contract failure message to stderr (fd=2)\n");
        ir.push_str("  %0 = call i64 @write(i32 2, i8* %condition, i64 %len)\n");
        ir.push_str("  ; Write newline\n");
        ir.push_str("  %1 = call i64 @write(i32 2, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @.panic_newline, i64 0, i64 0), i64 1)\n");
        ir.push_str("  call void @exit(i32 1)\n");
        ir.push_str("  unreachable\n");
        ir.push_str("}\n");

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

        // __load_f64: load a 64-bit float from memory address
        ir.push_str("\n; Helper function: load f64 from memory\n");
        ir.push_str("define double @__load_f64(i64 %ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to double*\n");
        ir.push_str("  %1 = load double, double* %0\n");
        ir.push_str("  ret double %1\n");
        ir.push_str("}\n");

        // __store_f64: store a 64-bit float to memory address
        ir.push_str("\n; Helper function: store f64 to memory\n");
        ir.push_str("define void @__store_f64(i64 %ptr, double %val) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to double*\n");
        ir.push_str("  store double %val, double* %0\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        ir
    }

    pub(crate) fn generate_extern_decl(&self, info: &FunctionInfo) -> String {
        let params: Vec<_> = info
            .signature
            .params
            .iter()
            .map(|(_, ty, _)| self.type_to_llvm(ty))
            .collect();

        let ret = self.type_to_llvm(&info.signature.ret);

        // Special handling for fopen_ptr: generate wrapper that calls fopen
        if info.signature.name == "fopen_ptr" {
            // Generate a wrapper function that forwards to fopen
            return format!(
                "define {} @fopen_ptr({} %path, {} %mode) {{\nentry:\n  %0 = call {} @fopen({} %path, {} %mode)\n  ret {} %0\n}}",
                ret,
                self.type_to_llvm(&ResolvedType::I64),
                self.type_to_llvm(&ResolvedType::Str),
                ret,
                self.type_to_llvm(&ResolvedType::I64),  // ptr type
                self.type_to_llvm(&ResolvedType::Str),
                ret
            );
        }

        if info.signature.is_vararg {
            let mut all_params = params.join(", ");
            if !all_params.is_empty() {
                all_params.push_str(", ...");
            } else {
                all_params.push_str("...");
            }
            format!("declare {} @{}({})", ret, info.signature.name, all_params)
        } else {
            format!("declare {} @{}({})", ret, info.signature.name, params.join(", "))
        }
    }

    #[allow(dead_code)]
    pub(crate) fn generate_function(&mut self, f: &Function) -> CodegenResult<String> {
        self.generate_function_with_span(f, Span::default())
    }

    pub(crate) fn generate_function_with_span(
        &mut self,
        f: &Function,
        span: Span,
    ) -> CodegenResult<String> {
        // Check if this is an async function
        if f.is_async {
            return self.generate_async_function(f);
        }

        self.current_function = Some(f.name.node.to_string());
        self.locals.clear();
        self.label_counter = 0;
        self.loop_stack.clear();
        self.clear_defer_stack();

        // Create debug info for this function
        let func_line = self.debug_info.offset_to_line(span.start);
        let di_subprogram =
            self.debug_info
                .create_function_debug_info(&f.name.node, func_line, true);

        let params: Vec<_> = f
            .params
            .iter()
            .map(|p| {
                let ty = self.ast_type_to_resolved(&p.ty.node);
                let llvm_ty = self.type_to_llvm(&ty);

                // Register parameter as local (SSA value, not alloca)
                // For params, llvm_name matches the source name
                self.locals.insert(
                    p.name.node.to_string(),
                    LocalVar::param(ty.clone(), p.name.node.to_string()),
                );

                format!("{} %{}", llvm_ty, p.name.node)
            })
            .collect();

        let ret_type = f
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);

        // Store current return type for nested return statements
        self.current_return_type = Some(ret_type.clone());

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

        // For struct parameters, allocate stack space and store the value
        // This allows field access to work via getelementptr
        for p in &f.params {
            let ty = self.ast_type_to_resolved(&p.ty.node);
            if matches!(ty, ResolvedType::Named { .. }) {
                let llvm_ty = self.type_to_llvm(&ty);
                let param_ptr_name = format!("__{}_ptr", p.name.node);
                let param_ptr = format!("%{}", param_ptr_name);
                ir.push_str(&format!("  {} = alloca {}\n", param_ptr, llvm_ty));
                ir.push_str(&format!("  store {} %{}, {}* {}\n",
                    llvm_ty, p.name.node, llvm_ty, param_ptr));
                // Update locals to use SSA with the pointer as the value (including %)
                // This makes the ident handler treat it as a direct pointer value, not a double pointer
                self.locals.insert(
                    p.name.node.to_string(),
                    LocalVar::ssa(ty.clone(), param_ptr),
                );
            }
        }

        // Generate body
        let mut counter = 0;

        // Generate requires (precondition) checks
        let requires_ir = self.generate_requires_checks(f, &mut counter)?;
        ir.push_str(&requires_ir);

        // Generate automatic contract checks from #[contract] attribute
        let auto_contract_ir = self.generate_auto_contract_checks(f, &mut counter)?;
        ir.push_str(&auto_contract_ir);

        // Generate decreases checks for termination proof
        let decreases_ir = self.generate_decreases_checks(f, &mut counter)?;
        ir.push_str(&decreases_ir);

        match &f.body {
            FunctionBody::Expr(expr) => {
                let (value, expr_ir) = self.generate_expr(expr, &mut counter)?;
                ir.push_str(&expr_ir);

                // Execute deferred expressions before return (LIFO order)
                let defer_ir = self.generate_defer_cleanup(&mut counter)?;
                ir.push_str(&defer_ir);

                // Generate ensures (postcondition) checks before return
                let ensures_ir = self.generate_ensures_checks(f, &value, &ret_type, &mut counter)?;
                ir.push_str(&ensures_ir);

                let ret_dbg = self.debug_info.dbg_ref_from_offset(expr.span.start);
                if ret_type == ResolvedType::Unit {
                    ir.push_str(&format!("  ret void{}\n", ret_dbg));
                } else if matches!(ret_type, ResolvedType::Named { .. }) {
                    // For struct returns, load the value from pointer
                    let loaded = format!("%ret.{}", counter);
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}{}\n",
                        loaded, ret_llvm, ret_llvm, value, ret_dbg
                    ));
                    ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, loaded, ret_dbg));
                } else {
                    ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, value, ret_dbg));
                }
            }
            FunctionBody::Block(stmts) => {
                let (value, block_ir, terminated) = self.generate_block_stmts(stmts, &mut counter)?;
                ir.push_str(&block_ir);

                // If block is already terminated (has return/break), don't emit ret
                if terminated {
                    // Block already has a terminator, no need for ret
                    // Note: defer cleanup for early returns is handled in Return statement
                    // Note: ensures checks for early returns need to be added to Return statement handling
                } else {
                    // Execute deferred expressions before return (LIFO order)
                    let defer_ir = self.generate_defer_cleanup(&mut counter)?;
                    ir.push_str(&defer_ir);

                    // Generate ensures (postcondition) checks before return
                    let ensures_ir = self.generate_ensures_checks(f, &value, &ret_type, &mut counter)?;
                    ir.push_str(&ensures_ir);

                    // Get debug location from last statement or function end
                    let ret_offset = stmts.last().map(|s| s.span.end).unwrap_or(span.end);
                    let ret_dbg = self.debug_info.dbg_ref_from_offset(ret_offset);
                    if ret_type == ResolvedType::Unit {
                        ir.push_str(&format!("  ret void{}\n", ret_dbg));
                    } else if matches!(ret_type, ResolvedType::Named { .. }) {
                        // Check if the result is already a value (from phi node) or a pointer (from struct lit)
                        if self.is_block_result_value(stmts) {
                            // Value (e.g., from if-else phi node) - return directly
                            ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, value, ret_dbg));
                        } else {
                            // Pointer (e.g., from struct literal) - load then return
                            let loaded = format!("%ret.{}", counter);
                            ir.push_str(&format!(
                                "  {} = load {}, {}* {}{}\n",
                                loaded, ret_llvm, ret_llvm, value, ret_dbg
                            ));
                            ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, loaded, ret_dbg));
                        }
                    } else {
                        ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, value, ret_dbg));
                    }
                }
            }
        }

        ir.push_str("}\n");

        self.current_function = None;
        self.current_return_type = None;
        self.clear_decreases_info();
        Ok(ir)
    }

    /// Generate an async function as a state machine coroutine
    ///
    /// Async functions are transformed into:
    /// 1. A state struct holding local variables and current state
    /// 2. A poll function that implements the state machine
    /// 3. A create function that returns a pointer to the state struct
    pub(crate) fn generate_async_function(&mut self, f: &Function) -> CodegenResult<String> {
        let func_name = &f.name.node;
        let state_struct_name = format!("{}__AsyncState", func_name);

        // Collect parameters for state struct
        let params: Vec<_> = f
            .params
            .iter()
            .map(|p| {
                let ty = self.ast_type_to_resolved(&p.ty.node);
                (p.name.node.to_string(), ty)
            })
            .collect();

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
            name: func_name.to_string(),
            state_struct: state_struct_name.to_string(),
            captured_vars: params.clone(),
            ret_type: ret_type.clone(),
        });

        let mut ir = String::new();

        // 1. Generate state struct type
        // Structure: { i64 state, i64 result, param1, param2, ... }
        ir.push_str(&format!("; Async state struct for {}\n", func_name));
        ir.push_str(&format!(
            "%{} = type {{ i64, {}",
            state_struct_name, ret_llvm
        ));
        for (_, ty) in &params {
            ir.push_str(&format!(", {}", self.type_to_llvm(ty)));
        }
        ir.push_str(" }\n\n");

        // 2. Generate create function: allocates and initializes state
        ir.push_str(&format!("; Create function for async {}\n", func_name));
        let create_params: Vec<_> = params
            .iter()
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
        ir.push_str(&format!(
            "  %state_ptr = call i64 @malloc(i64 {})\n",
            struct_size
        ));
        ir.push_str(&format!(
            "  %state = inttoptr i64 %state_ptr to %{}*\n",
            state_struct_name
        ));

        // Initialize state to 0 (start state)
        ir.push_str(&format!(
            "  %state_field = getelementptr %{}, %{}* %state, i32 0, i32 0\n",
            state_struct_name, state_struct_name
        ));
        ir.push_str("  store i64 0, i64* %state_field\n");

        // Store parameters in state struct
        for (i, (name, _ty)) in params.iter().enumerate() {
            let field_idx = i + 2; // Skip state and result fields
            ir.push_str(&format!(
                "  %param_{}_ptr = getelementptr %{}, %{}* %state, i32 0, i32 {}\n",
                name, state_struct_name, state_struct_name, field_idx
            ));
            ir.push_str(&format!(
                "  store i64 %{}, i64* %param_{}_ptr\n",
                name, name
            ));
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
        ir.push_str(&format!(
            "  %state = inttoptr i64 %state_ptr to %{}*\n",
            state_struct_name
        ));

        // Load current state
        ir.push_str(&format!(
            "  %state_field = getelementptr %{}, %{}* %state, i32 0, i32 0\n",
            state_struct_name, state_struct_name
        ));
        ir.push_str("  %current_state = load i64, i64* %state_field\n");

        // Load parameters from state into locals
        for (i, (name, ty)) in params.iter().enumerate() {
            let field_idx = i + 2;
            ir.push_str(&format!(
                "  %param_{}_ptr = getelementptr %{}, %{}* %state, i32 0, i32 {}\n",
                name, state_struct_name, state_struct_name, field_idx
            ));
            ir.push_str(&format!(
                "  %{} = load i64, i64* %param_{}_ptr\n",
                name, name
            ));

            self.locals
                .insert(name.clone(), LocalVar::param(ty.clone(), name.clone()));
        }

        // State machine switch
        ir.push_str("  switch i64 %current_state, label %state_invalid [\n");
        ir.push_str("    i64 0, label %state_0\n");
        ir.push_str("  ]\n\n");

        // Generate state_0 (initial state) - execute function body
        ir.push_str("state_0:\n");

        let mut counter = 0;
        let body_result = match &f.body {
            FunctionBody::Expr(expr) => self.generate_expr(expr, &mut counter)?,
            FunctionBody::Block(stmts) => self.generate_block(stmts, &mut counter)?,
        };

        ir.push_str(&body_result.1);

        // Store result and return Ready
        ir.push_str(&format!(
            "  %result_ptr = getelementptr %{}, %{}* %state, i32 0, i32 1\n",
            state_struct_name, state_struct_name
        ));
        ir.push_str(&format!(
            "  store {} {}, {}* %result_ptr\n",
            ret_llvm, body_result.0, ret_llvm
        ));

        // Set state to -1 (completed)
        ir.push_str("  store i64 -1, i64* %state_field\n");

        // Return {1, result} for Ready
        ir.push_str(&format!(
            "  %ret_val = load {}, {}* %result_ptr\n",
            ret_llvm, ret_llvm
        ));
        ir.push_str(&format!(
            "  %ret_0 = insertvalue {{ i64, {} }} undef, i64 1, 0\n",
            ret_llvm
        ));
        ir.push_str(&format!(
            "  %ret_1 = insertvalue {{ i64, {} }} %ret_0, {} %ret_val, 1\n",
            ret_llvm, ret_llvm
        ));
        ir.push_str(&format!("  ret {{ i64, {} }} %ret_1\n\n", ret_llvm));

        // Invalid state handler
        ir.push_str("state_invalid:\n");
        ir.push_str(&format!(
            "  %invalid_ret = insertvalue {{ i64, {} }} undef, i64 0, 0\n",
            ret_llvm
        ));
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
    pub(crate) fn generate_method(
        &mut self,
        struct_name: &str,
        f: &Function,
    ) -> CodegenResult<String> {
        self.generate_method_with_span(struct_name, f, Span::default())
    }

    pub(crate) fn generate_method_with_span(
        &mut self,
        struct_name: &str,
        f: &Function,
        span: Span,
    ) -> CodegenResult<String> {
        // Resolve generic struct aliases (e.g., "Pair" -> "Pair$i64")
        let resolved_struct_name = self.resolve_struct_name(struct_name);
        let struct_name = resolved_struct_name.as_str();

        // Method name: StructName_methodName
        let method_name = format!("{}_{}", struct_name, f.name.node);

        self.current_function = Some(method_name.to_string());
        self.locals.clear();
        self.label_counter = 0;
        self.loop_stack.clear();

        // Create debug info for this method
        let func_line = self.debug_info.offset_to_line(span.start);
        let di_subprogram = self
            .debug_info
            .create_function_debug_info(&method_name, func_line, true);

        // Check if this is a static method (no &self or self parameter)
        let has_self = f
            .params
            .first()
            .map(|p| p.name.node == "self")
            .unwrap_or(false);

        let mut params = Vec::new();

        if has_self {
            // Instance method: first parameter is `self` (pointer to struct)
            let struct_ty = format!("%{}*", struct_name);
            params.push(format!("{} %self", struct_ty));

            // Register self
            self.locals.insert(
                "self".to_string(),
                LocalVar::param(
                    ResolvedType::Named {
                        name: struct_name.to_string(),
                        generics: vec![],
                    },
                    "self".to_string(),
                ),
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
                p.name.node.to_string(),
                LocalVar::param(ty.clone(), p.name.node.to_string()),
            );

            params.push(format!("{} %{}", llvm_ty, p.name.node));
        }

        let ret_type = f
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);

        // Store current return type for nested return statements
        self.current_return_type = Some(ret_type.clone());

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

        // For struct parameters, allocate stack space and store the value
        // This allows field access to work via getelementptr
        for p in &f.params {
            if p.name.node == "self" {
                continue;
            }
            let ty = self.ast_type_to_resolved(&p.ty.node);
            if matches!(ty, ResolvedType::Named { .. }) {
                let llvm_ty = self.type_to_llvm(&ty);
                let param_ptr = format!("%__{}_ptr", p.name.node);
                ir.push_str(&format!("  {} = alloca {}\n", param_ptr, llvm_ty));
                ir.push_str(&format!("  store {} %{}, {}* {}\n",
                    llvm_ty, p.name.node, llvm_ty, param_ptr));
                // Update locals to use the pointer instead of the value
                self.locals.insert(
                    p.name.node.to_string(),
                    LocalVar::alloca(ty.clone(), param_ptr.trim_start_matches('%').to_string()),
                );
            }
        }

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
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}{}\n",
                        loaded, ret_llvm, ret_llvm, value, ret_dbg
                    ));
                    ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, loaded, ret_dbg));
                } else {
                    ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, value, ret_dbg));
                }
            }
            FunctionBody::Block(stmts) => {
                let (value, block_ir, terminated) = self.generate_block_stmts(stmts, &mut counter)?;
                ir.push_str(&block_ir);

                // If block is already terminated (has return/break), don't emit ret
                if terminated {
                    // Block already has a terminator, no need for ret
                } else {
                    let ret_offset = stmts.last().map(|s| s.span.end).unwrap_or(span.end);
                    let ret_dbg = self.debug_info.dbg_ref_from_offset(ret_offset);
                    if ret_type == ResolvedType::Unit {
                        ir.push_str(&format!("  ret void{}\n", ret_dbg));
                    } else if matches!(ret_type, ResolvedType::Named { .. }) {
                        // Check if the result is already a value (from phi node) or a pointer (from struct lit)
                        if self.is_block_result_value(stmts) {
                            // Value (e.g., from if-else phi node) - return directly
                            ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, value, ret_dbg));
                        } else {
                            // Pointer (e.g., from struct literal) - load then return
                            let loaded = format!("%ret.{}", counter);
                            ir.push_str(&format!(
                                "  {} = load {}, {}* {}{}\n",
                                loaded, ret_llvm, ret_llvm, value, ret_dbg
                            ));
                            ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, loaded, ret_dbg));
                        }
                    } else {
                        ir.push_str(&format!("  ret {} {}{}\n", ret_llvm, value, ret_dbg));
                    }
                }
            }
        }

        ir.push_str("}\n");

        self.current_function = None;
        self.current_return_type = None;
        Ok(ir)
    }
}

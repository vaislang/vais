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
            _name: inst.mangled_name.to_string(),
            fields,
            _repr_c: false,
            _invariants: Vec::new(),
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

        let ret_type = if let Some(t) = generic_fn.ret_type.as_ref() {
            let ty = self.ast_type_to_resolved(&t.node);
            vais_types::substitute_type(&ty, &substitutions)
        } else {
            self.functions
                .get(&generic_fn.name.node)
                .map(|info| info.signature.ret.clone())
                .unwrap_or(ResolvedType::Unit)
        };

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

        // __load_i8: load an 8-bit integer from memory address
        ir.push_str("\n; Helper function: load i8 from memory\n");
        ir.push_str("define i64 @__load_i8(i64 %ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i8*\n");
        ir.push_str("  %1 = load i8, i8* %0\n");
        ir.push_str("  %2 = zext i8 %1 to i64\n");
        ir.push_str("  ret i64 %2\n");
        ir.push_str("}\n");

        // __store_i8: store an 8-bit integer to memory address
        ir.push_str("\n; Helper function: store i8 to memory\n");
        ir.push_str("define void @__store_i8(i64 %ptr, i64 %val) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i8*\n");
        ir.push_str("  %1 = trunc i64 %val to i8\n");
        ir.push_str("  store i8 %1, i8* %0\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        // __load_i16: load a 16-bit integer from memory address
        ir.push_str("\n; Helper function: load i16 from memory\n");
        ir.push_str("define i64 @__load_i16(i64 %ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i16*\n");
        ir.push_str("  %1 = load i16, i16* %0\n");
        ir.push_str("  %2 = zext i16 %1 to i64\n");
        ir.push_str("  ret i64 %2\n");
        ir.push_str("}\n");

        // __store_i16: store a 16-bit integer to memory address
        ir.push_str("\n; Helper function: store i16 to memory\n");
        ir.push_str("define void @__store_i16(i64 %ptr, i64 %val) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i16*\n");
        ir.push_str("  %1 = trunc i64 %val to i16\n");
        ir.push_str("  store i16 %1, i16* %0\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        // __load_i32: load a 32-bit integer from memory address
        ir.push_str("\n; Helper function: load i32 from memory\n");
        ir.push_str("define i64 @__load_i32(i64 %ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i32*\n");
        ir.push_str("  %1 = load i32, i32* %0\n");
        ir.push_str("  %2 = zext i32 %1 to i64\n");
        ir.push_str("  ret i64 %2\n");
        ir.push_str("}\n");

        // __store_i32: store a 32-bit integer to memory address
        ir.push_str("\n; Helper function: store i32 to memory\n");
        ir.push_str("define void @__store_i32(i64 %ptr, i64 %val) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i32*\n");
        ir.push_str("  %1 = trunc i64 %val to i32\n");
        ir.push_str("  store i32 %1, i32* %0\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        // __load_f32: load a 32-bit float from memory address
        ir.push_str("\n; Helper function: load f32 from memory\n");
        ir.push_str("define double @__load_f32(i64 %ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to float*\n");
        ir.push_str("  %1 = load float, float* %0\n");
        ir.push_str("  %2 = fpext float %1 to double\n");
        ir.push_str("  ret double %2\n");
        ir.push_str("}\n");

        // __store_f32: store a 32-bit float to memory address
        ir.push_str("\n; Helper function: store f32 to memory\n");
        ir.push_str("define void @__store_f32(i64 %ptr, double %val) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to float*\n");
        ir.push_str("  %1 = fptrunc double %val to float\n");
        ir.push_str("  store float %1, float* %0\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        // === Async runtime helper functions ===

        // __call_poll: call an indirect function pointer (poll_fn) with future_ptr
        // poll_fn is a function pointer: i64 (i64) -> i64
        // Returns packed i64 with status in high 32 bits, value in low 32 bits
        ir.push_str("\n; Async helper: call indirect poll function\n");
        ir.push_str("define i64 @__call_poll(i64 %poll_fn, i64 %future_ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %poll_fn to i64 (i64)*\n");
        ir.push_str("  %1 = call i64 %0(i64 %future_ptr)\n");
        ir.push_str("  ret i64 %1\n");
        ir.push_str("}\n");

        // __extract_poll_status: extract status from packed poll result
        // status = result >> 32 (high 32 bits: 0=Pending, 1=Ready)
        ir.push_str("\n; Async helper: extract poll status from packed result\n");
        ir.push_str("define i64 @__extract_poll_status(i64 %poll_result) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = lshr i64 %poll_result, 32\n");
        ir.push_str("  %1 = and i64 %0, 4294967295\n");
        ir.push_str("  ret i64 %1\n");
        ir.push_str("}\n");

        // __extract_poll_value: extract value from packed poll result
        // value = result & 0xFFFFFFFF (low 32 bits)
        ir.push_str("\n; Async helper: extract poll value from packed result\n");
        ir.push_str("define i64 @__extract_poll_value(i64 %poll_result) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = and i64 %poll_result, 4294967295\n");
        ir.push_str("  ret i64 %0\n");
        ir.push_str("}\n");

        // __time_now_ms: get current time in milliseconds using gettimeofday
        ir.push_str("\n; Async helper: current time in milliseconds\n");
        ir.push_str("declare i32 @gettimeofday(i8*, i8*)\n");
        ir.push_str("define i64 @__time_now_ms() {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %tv = alloca [16 x i8], align 8\n");
        ir.push_str("  %tvptr = bitcast [16 x i8]* %tv to i8*\n");
        ir.push_str("  %0 = call i32 @gettimeofday(i8* %tvptr, i8* null)\n");
        ir.push_str("  %secptr = bitcast [16 x i8]* %tv to i64*\n");
        ir.push_str("  %sec = load i64, i64* %secptr\n");
        ir.push_str(
            "  %usecptr = getelementptr inbounds [16 x i8], [16 x i8]* %tv, i64 0, i64 8\n",
        );
        ir.push_str("  %usecptr64 = bitcast i8* %usecptr to i64*\n");
        ir.push_str("  %usec = load i64, i64* %usecptr64\n");
        ir.push_str("  %ms_sec = mul i64 %sec, 1000\n");
        ir.push_str("  %ms_usec = sdiv i64 %usec, 1000\n");
        ir.push_str("  %ms = add i64 %ms_sec, %ms_usec\n");
        ir.push_str("  ret i64 %ms\n");
        ir.push_str("}\n");

        // === macOS-only: kqueue helpers ===
        // Only include kqueue-related functions on macOS (they use the kevent syscall)
        #[cfg(target_os = "macos")]
        {
            // __kevent_register: wrapper around kevent syscall for registration
            ir.push_str("\n; Async helper: kqueue event registration\n");
            ir.push_str("declare i32 @kevent(i32, i8*, i32, i8*, i32, i8*)\n");
            ir.push_str(
                "define i64 @__kevent_register(i64 %kq, i64 %fd, i64 %filter, i64 %flags) {\n",
            );
            ir.push_str("entry:\n");
            // Allocate kevent struct (sizeof(struct kevent) = 64 bytes on macOS)
            ir.push_str("  %ev = alloca [64 x i8], align 8\n");
            ir.push_str("  %evptr = bitcast [64 x i8]* %ev to i8*\n");
            // Set ident (fd) at offset 0
            ir.push_str("  %identptr = bitcast [64 x i8]* %ev to i64*\n");
            ir.push_str("  store i64 %fd, i64* %identptr\n");
            // Set filter at offset 8 (i16)
            ir.push_str(
                "  %filterptr = getelementptr inbounds [64 x i8], [64 x i8]* %ev, i64 0, i64 8\n",
            );
            ir.push_str("  %filterptr16 = bitcast i8* %filterptr to i16*\n");
            ir.push_str("  %filter16 = trunc i64 %filter to i16\n");
            ir.push_str("  store i16 %filter16, i16* %filterptr16\n");
            // Set flags at offset 10 (u16)
            ir.push_str(
                "  %flagsptr = getelementptr inbounds [64 x i8], [64 x i8]* %ev, i64 0, i64 10\n",
            );
            ir.push_str("  %flagsptr16 = bitcast i8* %flagsptr to i16*\n");
            ir.push_str("  %flags16 = trunc i64 %flags to i16\n");
            ir.push_str("  store i16 %flags16, i16* %flagsptr16\n");
            // Call kevent
            ir.push_str("  %kq32 = trunc i64 %kq to i32\n");
            ir.push_str(
                "  %ret = call i32 @kevent(i32 %kq32, i8* %evptr, i32 1, i8* null, i32 0, i8* null)\n",
            );
            ir.push_str("  %retval = sext i32 %ret to i64\n");
            ir.push_str("  ret i64 %retval\n");
            ir.push_str("}\n");

            // __kevent_wait: wait for events with timeout
            ir.push_str("\n; Async helper: kqueue event wait\n");
            ir.push_str("define i64 @__kevent_wait(i64 %kq, i64 %events_buf, i64 %max_events, i64 %timeout_ms) {\n");
            ir.push_str("entry:\n");
            ir.push_str("  %bufptr = inttoptr i64 %events_buf to i8*\n");
            ir.push_str("  %kq32 = trunc i64 %kq to i32\n");
            ir.push_str("  %max32 = trunc i64 %max_events to i32\n");
            // Allocate timespec for timeout
            ir.push_str("  %ts = alloca [16 x i8], align 8\n");
            ir.push_str("  %tsptr = bitcast [16 x i8]* %ts to i8*\n");
            ir.push_str("  %secval = sdiv i64 %timeout_ms, 1000\n");
            ir.push_str("  %nsval = mul i64 %timeout_ms, 1000000\n");
            ir.push_str("  %nsrem = srem i64 %nsval, 1000000000\n");
            ir.push_str("  %secptr = bitcast [16 x i8]* %ts to i64*\n");
            ir.push_str("  store i64 %secval, i64* %secptr\n");
            ir.push_str(
                "  %nsptr = getelementptr inbounds [16 x i8], [16 x i8]* %ts, i64 0, i64 8\n",
            );
            ir.push_str("  %nsptr64 = bitcast i8* %nsptr to i64*\n");
            ir.push_str("  store i64 %nsrem, i64* %nsptr64\n");
            ir.push_str("  %ret = call i32 @kevent(i32 %kq32, i8* null, i32 0, i8* %bufptr, i32 %max32, i8* %tsptr)\n");
            ir.push_str("  %retval = sext i32 %ret to i64\n");
            ir.push_str("  ret i64 %retval\n");
            ir.push_str("}\n");

            // __kevent_get_fd: get fd from kevent result at index
            ir.push_str("\n; Async helper: get fd from kevent result\n");
            ir.push_str("define i64 @__kevent_get_fd(i64 %events_buf, i64 %index) {\n");
            ir.push_str("entry:\n");
            ir.push_str("  %base = inttoptr i64 %events_buf to i8*\n");
            // sizeof(struct kevent) = 64 on macOS
            ir.push_str("  %offset = mul i64 %index, 64\n");
            ir.push_str("  %evptr = getelementptr inbounds i8, i8* %base, i64 %offset\n");
            ir.push_str("  %identptr = bitcast i8* %evptr to i64*\n");
            ir.push_str("  %ident = load i64, i64* %identptr\n");
            ir.push_str("  ret i64 %ident\n");
            ir.push_str("}\n");

            // __kevent_get_filter: get filter from kevent result at index
            ir.push_str("\n; Async helper: get filter from kevent result\n");
            ir.push_str("define i64 @__kevent_get_filter(i64 %events_buf, i64 %index) {\n");
            ir.push_str("entry:\n");
            ir.push_str("  %base = inttoptr i64 %events_buf to i8*\n");
            ir.push_str("  %offset = mul i64 %index, 64\n");
            ir.push_str("  %evptr = getelementptr inbounds i8, i8* %base, i64 %offset\n");
            // filter is at offset 8 (i16)
            ir.push_str("  %filterptr = getelementptr inbounds i8, i8* %evptr, i64 8\n");
            ir.push_str("  %filterptr16 = bitcast i8* %filterptr to i16*\n");
            ir.push_str("  %filter16 = load i16, i16* %filterptr16\n");
            ir.push_str("  %filter = sext i16 %filter16 to i64\n");
            ir.push_str("  ret i64 %filter\n");
            ir.push_str("}\n");
        }

        // __write_byte: write a single byte to file descriptor
        ir.push_str("\n; Async helper: write byte to fd\n");
        ir.push_str("define i64 @__write_byte(i64 %fd, i64 %value) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %buf = alloca i8\n");
        ir.push_str("  %byte = trunc i64 %value to i8\n");
        ir.push_str("  store i8 %byte, i8* %buf\n");
        ir.push_str("  %fd32 = trunc i64 %fd to i32\n");
        ir.push_str("  %ret = call i64 @write(i32 %fd32, i8* %buf, i64 1)\n");
        ir.push_str("  ret i64 %ret\n");
        ir.push_str("}\n");

        // __read_byte: read a single byte from file descriptor
        ir.push_str("\n; Async helper: read byte from fd\n");
        ir.push_str("declare i64 @read(i32, i8*, i64)\n");
        ir.push_str("define i64 @__read_byte(i64 %fd) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %buf = alloca i8\n");
        ir.push_str("  %fd32 = trunc i64 %fd to i32\n");
        ir.push_str("  %ret = call i64 @read(i32 %fd32, i8* %buf, i64 1)\n");
        ir.push_str("  %byte = load i8, i8* %buf\n");
        ir.push_str("  %val = zext i8 %byte to i64\n");
        ir.push_str("  ret i64 %val\n");
        ir.push_str("}\n");

        // __readdir_wrapper: readdir wrapper that returns pointer to d_name
        ir.push_str("\n; Filesystem helper: readdir wrapper\n");
        ir.push_str("%struct.dirent = type opaque\n");
        ir.push_str("declare %struct.dirent* @readdir(i8*)\n");
        ir.push_str("define i64 @__readdir_wrapper(i64 %dirp) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %dirp to i8*\n");
        ir.push_str("  %1 = call %struct.dirent* @readdir(i8* %0)\n");
        ir.push_str("  %2 = icmp eq %struct.dirent* %1, null\n");
        ir.push_str("  br i1 %2, label %ret_null, label %ret_name\n");
        ir.push_str("ret_null:\n");
        ir.push_str("  ret i64 0\n");
        ir.push_str("ret_name:\n");
        ir.push_str("  %3 = bitcast %struct.dirent* %1 to i8*\n");
        // On macOS (Darwin), d_name is at offset 21
        // On Linux, d_name is at offset 19
        let d_name_offset = if cfg!(target_os = "linux") { 19 } else { 21 };
        ir.push_str(&format!(
            "  %4 = getelementptr inbounds i8, i8* %3, i64 {}\n",
            d_name_offset
        ));
        ir.push_str("  %5 = ptrtoint i8* %4 to i64\n");
        ir.push_str("  ret i64 %5\n");
        ir.push_str("}\n");

        // __getcwd_wrapper: getcwd wrapper that converts ptr result to i64
        ir.push_str("\n; Filesystem helper: getcwd wrapper\n");
        ir.push_str("declare i8* @getcwd(i8*, i64)\n");
        ir.push_str("define i64 @__getcwd_wrapper(i64 %buf, i64 %size) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %buf to i8*\n");
        ir.push_str("  %1 = call i8* @getcwd(i8* %0, i64 %size)\n");
        ir.push_str("  %2 = ptrtoint i8* %1 to i64\n");
        ir.push_str("  ret i64 %2\n");
        ir.push_str("}\n");

        // __stat_size: get file size using stat
        ir.push_str("\n; Filesystem helper: stat file size\n");
        ir.push_str("%struct.stat = type opaque\n");
        ir.push_str("declare i32 @stat(i8*, %struct.stat*)\n");
        ir.push_str("define i64 @__stat_size(i8* %path) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %statbuf = alloca [144 x i8], align 8\n");
        ir.push_str("  %0 = bitcast [144 x i8]* %statbuf to %struct.stat*\n");
        ir.push_str("  %1 = call i32 @stat(i8* %path, %struct.stat* %0)\n");
        ir.push_str("  %2 = icmp ne i32 %1, 0\n");
        ir.push_str("  br i1 %2, label %error, label %success\n");
        ir.push_str("error:\n");
        ir.push_str("  ret i64 -1\n");
        ir.push_str("success:\n");
        // st_size is at offset 96 on macOS (after dev:4, mode:2, nlink:2, ino:8, uid:4, gid:4, rdev:4, atim:16, mtim:16, ctim:16, birthtim:16, size:8)
        // Actually on macOS x86_64, st_size is at offset 96
        ir.push_str(
            "  %3 = getelementptr inbounds [144 x i8], [144 x i8]* %statbuf, i64 0, i64 96\n",
        );
        ir.push_str("  %4 = bitcast i8* %3 to i64*\n");
        ir.push_str("  %5 = load i64, i64* %4\n");
        ir.push_str("  ret i64 %5\n");
        ir.push_str("}\n");

        // __stat_mtime: get file modification time using stat
        ir.push_str("\n; Filesystem helper: stat modification time\n");
        ir.push_str("define i64 @__stat_mtime(i8* %path) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %statbuf = alloca [144 x i8], align 8\n");
        ir.push_str("  %0 = bitcast [144 x i8]* %statbuf to %struct.stat*\n");
        ir.push_str("  %1 = call i32 @stat(i8* %path, %struct.stat* %0)\n");
        ir.push_str("  %2 = icmp ne i32 %1, 0\n");
        ir.push_str("  br i1 %2, label %error, label %success\n");
        ir.push_str("error:\n");
        ir.push_str("  ret i64 -1\n");
        ir.push_str("success:\n");
        // st_mtimespec is at offset 48 on macOS (after dev:4, mode:2, nlink:2, ino:8, uid:4, gid:4, rdev:4, atim:16, mtim starts here)
        // The tv_sec field is the first 8 bytes of the timespec
        ir.push_str(
            "  %3 = getelementptr inbounds [144 x i8], [144 x i8]* %statbuf, i64 0, i64 48\n",
        );
        ir.push_str("  %4 = bitcast i8* %3 to i64*\n");
        ir.push_str("  %5 = load i64, i64* %4\n");
        ir.push_str("  ret i64 %5\n");
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
            // Both path and mode are i8* (strings) to match fopen's declaration
            let str_ty = self.type_to_llvm(&ResolvedType::Str);
            return format!(
                "define {} @fopen_ptr({} %path, {} %mode) {{\nentry:\n  %0 = call {} @fopen({} %path, {} %mode)\n  ret {} %0\n}}",
                ret,
                str_ty, str_ty,
                ret,
                str_ty, str_ty,
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
            format!(
                "declare {} @{}({})",
                ret,
                info.signature.name,
                params.join(", ")
            )
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

        // Get registered function signature for resolved param types (supports Type::Infer)
        let registered_param_types: Vec<_> = self
            .functions
            .get(&f.name.node)
            .map(|info| {
                info.signature
                    .params
                    .iter()
                    .map(|(_, ty, _)| ty.clone())
                    .collect()
            })
            .unwrap_or_default();

        let params: Vec<_> = f
            .params
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let ty = if i < registered_param_types.len() {
                    registered_param_types[i].clone()
                } else {
                    self.ast_type_to_resolved(&p.ty.node)
                };
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

        let ret_type = if let Some(t) = f.ret_type.as_ref() {
            self.ast_type_to_resolved(&t.node)
        } else {
            // Use registered return type from type checker (supports return type inference)
            self.functions
                .get(&f.name.node)
                .map(|info| info.signature.ret.clone())
                .unwrap_or(ResolvedType::Unit)
        };

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
        for (i, p) in f.params.iter().enumerate() {
            let ty = if i < registered_param_types.len() {
                registered_param_types[i].clone()
            } else {
                self.ast_type_to_resolved(&p.ty.node)
            };
            if matches!(ty, ResolvedType::Named { .. }) {
                let llvm_ty = self.type_to_llvm(&ty);
                let param_ptr_name = format!("__{}_ptr", p.name.node);
                let param_ptr = format!("%{}", param_ptr_name);
                ir.push_str(&format!("  {} = alloca {}\n", param_ptr, llvm_ty));
                ir.push_str(&format!(
                    "  store {} %{}, {}* {}\n",
                    llvm_ty, p.name.node, llvm_ty, param_ptr
                ));
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
                let ensures_ir =
                    self.generate_ensures_checks(f, &value, &ret_type, &mut counter)?;
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
                let (value, block_ir, terminated) =
                    self.generate_block_stmts(stmts, &mut counter)?;
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
                    let ensures_ir =
                        self.generate_ensures_checks(f, &value, &ret_type, &mut counter)?;
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

        let ret_type = if let Some(t) = f.ret_type.as_ref() {
            self.ast_type_to_resolved(&t.node)
        } else {
            self.functions
                .get(func_name)
                .map(|info| info.signature.ret.clone())
                .unwrap_or(ResolvedType::Unit)
        };

        let ret_llvm = self.type_to_llvm(&ret_type);

        // Reset async state tracking
        self.async_state_counter = 0;
        self.async_await_points.clear();
        self.current_async_function = Some(AsyncFunctionInfo {
            _name: func_name.to_string(),
            _state_struct: state_struct_name.to_string(),
            _captured_vars: params.clone(),
            _ret_type: ret_type.clone(),
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
        let di_subprogram =
            self.debug_info
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

        let ret_type = if let Some(t) = f.ret_type.as_ref() {
            self.ast_type_to_resolved(&t.node)
        } else {
            self.functions
                .get(&f.name.node)
                .map(|info| info.signature.ret.clone())
                .unwrap_or(ResolvedType::Unit)
        };

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
                ir.push_str(&format!(
                    "  store {} %{}, {}* {}\n",
                    llvm_ty, p.name.node, llvm_ty, param_ptr
                ));
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
                let (value, block_ir, terminated) =
                    self.generate_block_stmts(stmts, &mut counter)?;
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

    /// Generate WASM-specific runtime functions and declarations.
    ///
    /// For `wasm32-unknown-unknown` targets, this generates:
    /// - Linear memory export
    /// - `_start` entry point that calls main
    /// - `fd_write`-based `puts` implementation (no libc)
    /// - Simple bump allocator using `memory.grow`
    ///
    /// For WASI targets, this generates:
    /// - WASI-compatible `_start` entry point
    /// - Memory export for WASI runtime
    pub(crate) fn generate_wasm_runtime(&self) -> String {
        use crate::TargetTriple;

        if !self.target.is_wasm() {
            return String::new();
        }

        let mut ir = String::new();

        ir.push_str("\n; ========================================\n");
        ir.push_str("; WASM Runtime Support\n");
        ir.push_str("; ========================================\n\n");

        match &self.target {
            TargetTriple::Wasm32Unknown => {
                self.generate_wasm32_unknown_runtime(&mut ir);
            }
            TargetTriple::WasiPreview1 | TargetTriple::WasiPreview2 => {
                self.generate_wasi_runtime(&mut ir);
            }
            _ => {}
        }

        ir
    }

    /// Generate runtime for wasm32-unknown-unknown (no WASI, browser environment)
    fn generate_wasm32_unknown_runtime(&self, ir: &mut String) {
        // Memory export (1 page = 64KB initial)
        ir.push_str("; Linear memory (exported)\n");
        ir.push_str("@__wasm_memory = external global i8\n\n");

        // Bump allocator state: heap pointer starts at 1MB (leaves stack space)
        ir.push_str("; Bump allocator heap pointer (starts at 1MB offset)\n");
        ir.push_str("@__heap_ptr = global i32 1048576\n\n");

        // malloc replacement using bump allocator
        ir.push_str("; WASM malloc: bump allocator with memory.grow\n");
        ir.push_str("define i8* @malloc(i64 %size) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %size32 = trunc i64 %size to i32\n");
        ir.push_str("  ; Align to 8 bytes\n");
        ir.push_str("  %aligned = add i32 %size32, 7\n");
        ir.push_str("  %aligned_size = and i32 %aligned, -8\n");
        ir.push_str("  ; Load current heap pointer\n");
        ir.push_str("  %cur = load i32, i32* @__heap_ptr\n");
        ir.push_str("  %new = add i32 %cur, %aligned_size\n");
        ir.push_str("  ; Check if we need to grow memory\n");
        ir.push_str("  %cur_pages = call i32 @llvm.wasm.memory.size.i32(i32 0)\n");
        ir.push_str("  %cur_bytes = mul i32 %cur_pages, 65536\n");
        ir.push_str("  %needs_grow = icmp ugt i32 %new, %cur_bytes\n");
        ir.push_str("  br i1 %needs_grow, label %grow, label %done\n");
        ir.push_str("grow:\n");
        ir.push_str("  %needed = sub i32 %new, %cur_bytes\n");
        ir.push_str("  %pages_needed_raw = add i32 %needed, 65535\n");
        ir.push_str("  %pages_needed = udiv i32 %pages_needed_raw, 65536\n");
        ir.push_str(
            "  %grow_result = call i32 @llvm.wasm.memory.grow.i32(i32 0, i32 %pages_needed)\n",
        );
        ir.push_str("  %grow_failed = icmp eq i32 %grow_result, -1\n");
        ir.push_str("  br i1 %grow_failed, label %oom, label %done\n");
        ir.push_str("oom:\n");
        ir.push_str("  call void @__wasm_trap()\n");
        ir.push_str("  unreachable\n");
        ir.push_str("done:\n");
        ir.push_str("  ; Update heap pointer\n");
        ir.push_str("  store i32 %new, i32* @__heap_ptr\n");
        ir.push_str("  %ptr = inttoptr i32 %cur to i8*\n");
        ir.push_str("  ret i8* %ptr\n");
        ir.push_str("}\n\n");

        // free is a no-op for bump allocator
        ir.push_str("; WASM free: no-op for bump allocator\n");
        ir.push_str("define void @free(i8* %ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n\n");

        // realloc: allocate new block and copy
        ir.push_str("; WASM realloc: allocate new + copy (conservative)\n");
        ir.push_str("define i8* @realloc(i8* %old, i64 %new_size) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %new_ptr = call i8* @malloc(i64 %new_size)\n");
        ir.push_str("  %old_is_null = icmp eq i8* %old, null\n");
        ir.push_str("  br i1 %old_is_null, label %done, label %copy\n");
        ir.push_str("copy:\n");
        ir.push_str("  ; Copy old data (conservative: copy new_size bytes)\n");
        ir.push_str("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* %new_ptr, i8* %old, i64 %new_size, i1 false)\n");
        ir.push_str("  br label %done\n");
        ir.push_str("done:\n");
        ir.push_str("  ret i8* %new_ptr\n");
        ir.push_str("}\n\n");

        // puts replacement: write string to fd 1 (stdout) via imported function
        ir.push_str("; WASM puts: calls imported __wasm_write for output\n");
        ir.push_str("define i64 @puts(i8* %str) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %len = call i64 @strlen(i8* %str)\n");
        ir.push_str("  %len32 = trunc i64 %len to i32\n");
        ir.push_str("  %ptr32 = ptrtoint i8* %str to i32\n");
        ir.push_str("  call void @__wasm_write(i32 1, i32 %ptr32, i32 %len32)\n");
        ir.push_str("  ; Write newline\n");
        ir.push_str("  call void @__wasm_write_byte(i32 1, i32 10)\n");
        ir.push_str("  ret i64 0\n");
        ir.push_str("}\n\n");

        // printf replacement (simplified: just write the format string)
        ir.push_str("; WASM printf: simplified output\n");
        ir.push_str("define i64 @printf(i8* %fmt, ...) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %len = call i64 @strlen(i8* %fmt)\n");
        ir.push_str("  %len32 = trunc i64 %len to i32\n");
        ir.push_str("  %ptr32 = ptrtoint i8* %fmt to i32\n");
        ir.push_str("  call void @__wasm_write(i32 1, i32 %ptr32, i32 %len32)\n");
        ir.push_str("  ret i64 %len\n");
        ir.push_str("}\n\n");

        // strlen implementation (no libc)
        ir.push_str("; WASM strlen: pure LLVM implementation\n");
        ir.push_str("define i64 @__wasm_strlen(i8* %str) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  br label %loop\n");
        ir.push_str("loop:\n");
        ir.push_str("  %i = phi i64 [0, %entry], [%next, %loop]\n");
        ir.push_str("  %ptr = getelementptr i8, i8* %str, i64 %i\n");
        ir.push_str("  %ch = load i8, i8* %ptr\n");
        ir.push_str("  %is_zero = icmp eq i8 %ch, 0\n");
        ir.push_str("  %next = add i64 %i, 1\n");
        ir.push_str("  br i1 %is_zero, label %done, label %loop\n");
        ir.push_str("done:\n");
        ir.push_str("  ret i64 %i\n");
        ir.push_str("}\n\n");

        // exit implementation via trap
        ir.push_str("; WASM exit: unreachable trap\n");
        ir.push_str("define void @exit(i32 %code) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  call void @__wasm_trap()\n");
        ir.push_str("  unreachable\n");
        ir.push_str("}\n\n");

        // Imported functions from JS host
        ir.push_str("; Host-imported functions (provided by JS runtime)\n");
        ir.push_str("declare void @__wasm_write(i32 %fd, i32 %ptr, i32 %len)\n");
        ir.push_str("declare void @__wasm_write_byte(i32 %fd, i32 %byte)\n");
        ir.push_str("declare void @__wasm_trap()\n\n");

        // LLVM intrinsics for WASM
        ir.push_str("; LLVM WASM intrinsics\n");
        ir.push_str("declare i32 @llvm.wasm.memory.size.i32(i32)\n");
        ir.push_str("declare i32 @llvm.wasm.memory.grow.i32(i32, i32)\n");
        ir.push_str("declare void @llvm.memcpy.p0i8.p0i8.i64(i8*, i8*, i64, i1)\n\n");

        // _start entry point that calls main
        ir.push_str("; _start entry point (calls main)\n");
        ir.push_str("define void @_start() {\n");
        ir.push_str("entry:\n");
        if self.functions.contains_key("main") {
            ir.push_str("  %ret = call i64 @main()\n");
        }
        ir.push_str("  ret void\n");
        ir.push_str("}\n\n");
    }

    /// Generate runtime for WASI targets
    fn generate_wasi_runtime(&self, ir: &mut String) {
        // WASI _start entry point
        ir.push_str("; WASI _start entry point\n");
        ir.push_str("define void @_start() {\n");
        ir.push_str("entry:\n");
        if self.functions.contains_key("main") {
            ir.push_str("  %ret = call i64 @main()\n");
            ir.push_str("  ; Exit with main's return code\n");
            ir.push_str("  %code = trunc i64 %ret to i32\n");
            ir.push_str("  call void @__wasi_proc_exit(i32 %code)\n");
        } else {
            ir.push_str("  call void @__wasi_proc_exit(i32 0)\n");
        }
        ir.push_str("  unreachable\n");
        ir.push_str("}\n\n");

        // WASI fd_write-based puts
        ir.push_str("; WASI puts: fd_write based\n");
        ir.push_str("; Uses WASI fd_write(fd, iovs, iovs_len, nwritten) -> errno\n");
        ir.push_str("@__wasi_iov = global [2 x i32] zeroinitializer\n");
        ir.push_str("@__wasi_nwritten = global i32 0\n\n");
        ir.push_str("define i64 @__wasi_puts(i8* %str) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %len = call i64 @strlen(i8* %str)\n");
        ir.push_str("  %len32 = trunc i64 %len to i32\n");
        ir.push_str("  %ptr32 = ptrtoint i8* %str to i32\n");
        ir.push_str("  ; Set up iov: [ptr, len]\n");
        ir.push_str("  %iov_ptr = getelementptr [2 x i32], [2 x i32]* @__wasi_iov, i32 0, i32 0\n");
        ir.push_str("  store i32 %ptr32, i32* %iov_ptr\n");
        ir.push_str("  %iov_len = getelementptr [2 x i32], [2 x i32]* @__wasi_iov, i32 0, i32 1\n");
        ir.push_str("  store i32 %len32, i32* %iov_len\n");
        ir.push_str("  ; Call fd_write(stdout=1, iovs, iovs_len=1, nwritten)\n");
        ir.push_str("  %errno = call i32 @__wasi_fd_write(i32 1, i32* %iov_ptr, i32 1, i32* @__wasi_nwritten)\n");
        ir.push_str("  %result = sext i32 %errno to i64\n");
        ir.push_str("  ret i64 %result\n");
        ir.push_str("}\n\n");

        // WASI syscall declarations
        ir.push_str("; WASI syscall declarations\n");
        ir.push_str("declare void @__wasi_proc_exit(i32) noreturn\n");
        ir.push_str("declare i32 @__wasi_fd_write(i32, i32*, i32, i32*)\n");
        ir.push_str("declare i32 @__wasi_fd_read(i32, i32*, i32, i32*)\n\n");
    }
}

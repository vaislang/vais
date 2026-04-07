//! Function signature resolution and external declarations

use crate::types::FunctionInfo;
use crate::CodeGenerator;
use vais_ast::Function;
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Resolve the return type of a function, using either:
    /// 1. Explicit return type from AST
    /// 2. Registered return type from type checker (for inference)
    /// 3. Default to Unit if not found
    pub(crate) fn resolve_fn_return_type(&self, f: &Function, lookup_key: &str) -> ResolvedType {
        if let Some(t) = f.ret_type.as_ref() {
            self.ast_type_to_resolved(&t.node)
        } else {
            // Use registered return type from type checker (supports return type inference)
            self.types
                .functions
                .get(lookup_key)
                .map(|info| &info.signature.ret)
                .cloned() // explicit single clone at end
                .unwrap_or(ResolvedType::Unit)
        }
    }

    /// Initialize function state for code generation.
    /// Clears locals, resets label counter and loop stack.
    pub(crate) fn initialize_function_state(&mut self, func_name: &str) {
        self.fn_ctx.current_function = Some(func_name.to_string());
        self.fn_ctx.locals.clear();
        self.fn_ctx.label_counter = 0;
        self.fn_ctx.loop_stack.clear();
        self.fn_ctx.future_poll_fns.clear();
        self.fn_ctx.async_poll_context = None;
        self.fn_ctx.temp_var_types.clear();
        self.fn_ctx.scope_stack.clear();
        self.fn_ctx.entry_allocas.clear();
        // Don't clear pending_specialized_ir — accumulate across functions
    }

    /// Map a type to LLVM for extern C ABI declarations.
    /// Str maps to i8* (C-compatible) instead of { i8*, i64 } (Vais fat pointer).
    pub(crate) fn type_to_llvm_extern(&self, ty: &ResolvedType) -> String {
        match ty {
            ResolvedType::Str => String::from("i8*"),
            // &str is also passed as i8* in C ABI (not as a pointer to fat pointer)
            ResolvedType::Ref(inner) if matches!(inner.as_ref(), ResolvedType::Str) => {
                String::from("i8*")
            }
            _ => self.type_to_llvm(ty),
        }
    }

    pub(crate) fn generate_extern_decl(&self, info: &FunctionInfo) -> String {
        let params: Vec<_> = info
            .signature
            .params
            .iter()
            .map(|(_, ty, _)| self.type_to_llvm_extern(ty))
            .collect();

        let ret = self.type_to_llvm_extern(&info.signature.ret);

        // Special handling for fopen_ptr: generate wrapper that calls fopen
        if info.signature.name == "fopen_ptr" {
            // fopen uses i8* (C strings) for both path and mode
            return format!(
                "define {} @fopen_ptr(i8* %path, i8* %mode) {{\nentry:\n  %0 = call {} @fopen(i8* %path, i8* %mode)\n  ret {} %0\n}}",
                ret,
                ret,
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
}

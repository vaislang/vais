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
}

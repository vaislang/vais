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
        self.fn_ctx.actual_llvm_type.clear();
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
        // Phase E: filter out Unit (`void`) parameters. LLVM only permits
        // `void` as a function return type, not as a parameter type.
        // Generic specializations like `RwLock_new$unit` erase their `T`
        // parameter to Unit → previous emission produced
        // `declare %RwLock$unit @RwLock_new$unit(void)` which clang rejected
        // with "void type only allowed for function results".
        //
        // Also resolve `Self` in param / return positions: method extern
        // decls reach here with `Named("Self")` which otherwise prints as
        // `%Self` (undefined in cross-module IR). Parse the struct name
        // from the mangled function name (format: "<Struct>_<method>"
        // or "<Struct>_<method>$<args>") and rewrite Self → Struct.
        let self_struct: Option<String> = {
            let name = &info.signature.name;
            let base = name.split('$').next().unwrap_or(name);
            base.split_once('_')
                .map(|(s, _)| s.to_string())
                .filter(|s| self.types.structs.contains_key(s) || self.types.enums.contains_key(s))
        };
        let resolve_self = |ty: &ResolvedType| -> ResolvedType {
            if let (ResolvedType::Named { name, generics }, Some(struct_name)) =
                (ty, self_struct.as_ref())
            {
                if name == "Self" {
                    return ResolvedType::Named {
                        name: struct_name.clone(),
                        generics: generics.clone(),
                    };
                }
            }
            ty.clone()
        };
        // Phase 17.H4 iter 18: `type_to_llvm_extern` lowers `&str` → `i8*`
        // for C ABI compatibility. That is correct for true extern C
        // functions (malloc, free, fopen …), but wrong for **cross-module
        // Vais function declares** which must preserve the fat pointer
        // `{ i8*, i64 }` ABI that call sites emit via `type_to_llvm`.
        // Mismatch caused `declare i64 @fnv1a_hash(i8*)` vs
        // `call i64 @fnv1a_hash({ i8*, i64 } %t0)` — clang link error
        // `'%t0' defined with type 'ptr' but expected '{ ptr, i64 }'`.
        // Use native ABI for non-extern (Vais-owned) functions.
        let lower = |ty: &ResolvedType| -> String {
            if info.is_extern {
                self.type_to_llvm_extern(ty)
            } else {
                self.type_to_llvm(ty)
            }
        };
        let params: Vec<_> = info
            .signature
            .params
            .iter()
            .filter(|(_, ty, _)| !matches!(ty, ResolvedType::Unit))
            .map(|(_, ty, _)| lower(&resolve_self(ty)))
            .collect();

        let ret = lower(&resolve_self(&info.signature.ret));

        // Special handling for C memory functions: declare with C ABI types
        // to match call-site IR (which uses i8* for pointers).
        // Without this, declare uses i64 (from register_extern) causing
        // declare/call type mismatch → LLVM undefined behavior → SIGSEGV.
        match info.signature.name.as_str() {
            "malloc" => return "declare i8* @malloc(i64)".to_string(),
            "free" => return "declare void @free(i8*)".to_string(),
            "realloc" => return "declare i8* @realloc(i8*, i64)".to_string(),
            _ => {}
        }

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

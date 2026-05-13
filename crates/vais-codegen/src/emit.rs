//! IR emission helper methods for module headers, strings, and metadata

use super::*;

impl CodeGenerator {
    /// Emit LLVM IR module header (ModuleID, source_filename, target triple/datalayout).
    pub(crate) fn emit_module_header(&mut self, ir: &mut String) {
        write_ir!(ir, "; ModuleID = '{}'", self.module_name);
        ir.push_str("source_filename = \"<vais>\"\n");
        if !matches!(self.target, TargetTriple::Native) {
            write_ir!(ir, "target datalayout = \"{}\"", self.target.data_layout());
            write_ir!(ir, "target triple = \"{}\"", self.target.triple_str());
        }
        ir.push('\n');
        if self.debug_info.is_enabled() {
            self.debug_info.initialize();
        }
    }

    /// Emit ABI version, string constants, and unwrap panic declaration.
    pub(crate) fn emit_string_constants(&self, ir: &mut String, is_main_module: bool) {
        if is_main_module {
            let abi_version = crate::abi::ABI_VERSION;
            let abi_version_len = abi_version.len() + 1;
            write_ir!(
                ir,
                "@__vais_abi_version = constant [{} x i8] c\"{}\\00\"\n",
                abi_version_len,
                abi_version
            );
        }
        for (name, value) in &self.strings.constants {
            let escaped = escape_llvm_string(value);
            let len = value.len() + 1;
            write_ir!(
                ir,
                "@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"",
                name,
                len,
                escaped
            );
        }
        if !self.strings.constants.is_empty() {
            ir.push('\n');
        }
        // Emit global numeric constants for reference returns
        for (name, ty, value) in &self.ref_constants {
            write_ir!(ir, "@{} = internal constant {} {}", name, ty, value);
        }
        if !self.ref_constants.is_empty() {
            ir.push('\n');
        }
        if self.needs_unwrap_panic {
            ir.push_str("@.unwrap_panic_msg = private unnamed_addr constant [22 x i8] c\"unwrap failed: panic!\\00\"\n");
            ir.push_str("declare void @abort()\n\n");
        } else if self.needs_bounds_check {
            // Bounds check uses abort() for OOB access
            ir.push_str("declare void @abort()\n\n");
        }
        if self.needs_llvm_memcpy {
            ir.push_str("declare void @llvm.memcpy.p0i8.p0i8.i64(i8*, i8*, i64, i1)\n\n");
        }
    }

    /// Emit global variable declarations, with optional ownership filtering.
    ///
    /// If `owned` is `None`, every registered global is emitted as a full
    /// definition (legacy single-module path). If `owned` is `Some(set)`,
    /// globals in the set are emitted as definitions and globals **not** in
    /// the set are emitted as `external global` declarations instead. This
    /// is the multi-module path used by `subset.rs` — see ROADMAP Phase 2
    /// iter 15 (monitor D1 link pass) for the duplicate-symbol bug this
    /// prevents.
    pub(crate) fn emit_global_vars_with_ownership(
        &self,
        ir: &mut String,
        owned: Option<&std::collections::HashSet<String>>,
    ) {
        if self.types.globals.is_empty() {
            return;
        }
        for (name, info) in &self.types.globals {
            let llvm_ty = self.type_to_llvm(&info._ty);
            // Non-owner module in a multi-module build: emit as `external
            // global` (declaration only). clang will resolve the symbol to
            // the owner module's definition at link time.
            if let Some(set) = owned {
                if !set.contains(name) {
                    write_ir!(ir, "@{} = external global {}", name, llvm_ty);
                    continue;
                }
            }
            // Evaluate constant initializer to a literal value
            let init_val = match &info._value.node {
                vais_ast::Expr::Int(n) => n.to_string(),
                vais_ast::Expr::Bool(b) => {
                    if *b {
                        "1".to_string()
                    } else {
                        "0".to_string()
                    }
                }
                vais_ast::Expr::Float(f) => format!("{}", f),
                vais_ast::Expr::String(s) => {
                    // Str globals: emit as a reference to the string constant.
                    // The string constant @.strN is emitted by emit_string_constants.
                    // Look up the string in the dedup cache to find the constant name.
                    if let Some(const_name) = self.strings.dedup_cache.get(s.as_str()) {
                        let len = s.len();
                        let const_len = len + 1; // +1 for null terminator
                        format!(
                            "{{ i8* getelementptr inbounds ([{} x i8], [{} x i8]* @{}, i32 0, i32 0), i64 {} }}",
                            const_len, const_len, const_name, len
                        )
                    } else {
                        // String not in pool — use zeroinitializer as fallback
                        "zeroinitializer".to_string()
                    }
                }
                _ => {
                    // For compound types (structs, arrays, etc.), use zeroinitializer
                    // instead of 0, which is invalid for non-integer LLVM types
                    if llvm_ty.starts_with('{')
                        || llvm_ty.starts_with('[')
                        || llvm_ty.starts_with('<')
                    {
                        "zeroinitializer".to_string()
                    } else {
                        "0".to_string()
                    }
                }
            };
            let linkage = if info._is_mutable {
                "global"
            } else {
                "constant"
            };
            write_ir!(ir, "@{} = {} {} {}", name, linkage, llvm_ty, init_val);
        }
        ir.push('\n');
    }

    /// Thin wrapper matching the legacy signature — emits every global as a
    /// full definition. Used by the single-module path (`mod.rs`,
    /// `instantiations.rs`) which always owns every global in its module.
    pub(crate) fn emit_global_vars(&self, ir: &mut String) {
        self.emit_global_vars_with_ownership(ir, None);
    }

    /// Emit body IR, lambda functions, and vtable globals.
    pub(crate) fn emit_body_lambdas_vtables(&self, ir: &mut String, body_ir: &str) {
        ir.push_str(body_ir);
        for lambda_ir in &self.lambdas.generated_ir {
            ir.push('\n');
            ir.push_str(lambda_ir);
        }
        // Emit __sync_spawn__poll if any sync spawn was used
        if self.needs_sync_spawn_poll {
            ir.push_str("\n; Sync spawn poll function — always returns Ready with stored result\n");
            ir.push_str("define { i64, i64 } @__sync_spawn__poll(i64 %state_ptr) {\n");
            ir.push_str("entry:\n");
            ir.push_str("  %ptr = inttoptr i64 %state_ptr to {i64, i64}*\n");
            ir.push_str(
                "  %result_ptr = getelementptr {i64, i64}, {i64, i64}* %ptr, i32 0, i32 1\n",
            );
            ir.push_str("  %result = load i64, i64* %result_ptr\n");
            ir.push_str("  %ret_0 = insertvalue { i64, i64 } undef, i64 1, 0\n");
            ir.push_str("  %ret_1 = insertvalue { i64, i64 } %ret_0, i64 %result, 1\n");
            ir.push_str("  ret { i64, i64 } %ret_1\n");
            ir.push_str("}\n");
        }
        let vtable_ir = self.generate_vtable_globals();
        if !vtable_ir.is_empty() {
            ir.push_str("\n; VTable globals for trait objects\n");
            ir.push_str(&vtable_ir);
        }
        let drop_ir = self.vtable_generator.generate_drop_functions_ir();
        if !drop_ir.is_empty() {
            ir.push_str("\n; Drop functions for trait objects\n");
            ir.push_str(&drop_ir);
        }
    }
}

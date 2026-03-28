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

    /// Emit global variable declarations.
    pub(crate) fn emit_global_vars(&self, ir: &mut String) {
        if self.types.globals.is_empty() {
            return;
        }
        for (name, info) in &self.types.globals {
            let llvm_ty = self.type_to_llvm(&info._ty);
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
                _ => "0".to_string(), // Default zero-initialize for complex expressions
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

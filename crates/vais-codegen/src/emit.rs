//! IR emission helper methods for module headers, strings, and metadata

use super::*;

impl CodeGenerator {
    /// Emit LLVM IR module header (ModuleID, source_filename, target triple/datalayout).
    pub(crate) fn emit_module_header(&mut self, ir: &mut String) {
        ir.push_str(&format!("; ModuleID = '{}'\n", self.module_name));
        ir.push_str("source_filename = \"<vais>\"\n");
        if !matches!(self.target, TargetTriple::Native) {
            ir.push_str(&format!(
                "target datalayout = \"{}\"\n",
                self.target.data_layout()
            ));
            ir.push_str(&format!(
                "target triple = \"{}\"\n",
                self.target.triple_str()
            ));
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
            ir.push_str(&format!(
                "@__vais_abi_version = constant [{} x i8] c\"{}\\00\"\n\n",
                abi_version_len, abi_version
            ));
        }
        for (name, value) in &self.strings.constants {
            let escaped = escape_llvm_string(value);
            let len = value.len() + 1;
            ir.push_str(&format!(
                "@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n",
                name, len, escaped
            ));
        }
        if !self.strings.constants.is_empty() {
            ir.push('\n');
        }
        if self.needs_unwrap_panic {
            ir.push_str("@.unwrap_panic_msg = private unnamed_addr constant [22 x i8] c\"unwrap failed: panic!\\00\"\n");
            ir.push_str("declare void @abort()\n\n");
        }
    }

    /// Emit body IR, lambda functions, and vtable globals.
    pub(crate) fn emit_body_lambdas_vtables(&self, ir: &mut String, body_ir: &str) {
        ir.push_str(body_ir);
        for lambda_ir in &self.lambdas.generated_ir {
            ir.push('\n');
            ir.push_str(lambda_ir);
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

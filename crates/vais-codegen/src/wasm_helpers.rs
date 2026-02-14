//! WASM-specific code generation helpers

use super::*;

impl CodeGenerator {
    /// Generate WASM import/export attribute sections
    pub(crate) fn generate_wasm_metadata(&self) -> String {
        let mut ir = String::new();

        if self.wasm_imports.is_empty() && self.wasm_exports.is_empty() {
            return ir;
        }

        // Generate WASM import attributes using custom section metadata
        // These are recognized by LLVM's WASM backend
        let mut attr_idx = 1;
        for (module_name, import_name) in self.wasm_imports.values() {
            ir.push_str(&format!(
                "attributes #{} = {{ \"wasm-import-module\"=\"{}\" \"wasm-import-name\"=\"{}\" }}\n",
                attr_idx, module_name, import_name
            ));
            attr_idx += 1;
        }

        // Generate WASM export annotations
        for export_name in self.wasm_exports.values() {
            ir.push_str(&format!(
                "attributes #{} = {{ \"wasm-export-name\"=\"{}\" }}\n",
                attr_idx, export_name
            ));
            attr_idx += 1;
        }

        ir
    }
}

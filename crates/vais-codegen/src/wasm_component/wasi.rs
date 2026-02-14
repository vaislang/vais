//! WASI manifest for managing WASI interface imports/exports

use super::types::WitType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct WasiManifest {
    /// WASI interface imports (e.g., "wasi:filesystem/types")
    pub imports: Vec<String>,
    /// WASI exports (name -> WIT type)
    pub exports: HashMap<String, WitType>,
}

impl WasiManifest {
    /// Create a new WASI manifest
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
            exports: HashMap::new(),
        }
    }

    /// Add a WASI import interface
    pub fn add_import(&mut self, interface: &str) {
        if !self.imports.contains(&interface.to_string()) {
            self.imports.push(interface.to_string());
        }
    }

    /// Add a WASI export
    pub fn add_export(&mut self, name: &str, wit_type: &WitType) {
        self.exports.insert(name.to_string(), wit_type.clone());
    }

    /// Generate WIT file content from the manifest
    pub fn to_wit_string(&self) -> String {
        let mut output = String::new();

        // Imports
        if !self.imports.is_empty() {
            output.push_str("// WASI Imports\n");
            for import in &self.imports {
                output.push_str(&format!("import {};\n", import));
            }
            output.push('\n');
        }

        // Exports
        if !self.exports.is_empty() {
            output.push_str("// WASI Exports\n");
            for (name, wit_type) in &self.exports {
                output.push_str(&format!("export {}: {};\n", name, wit_type));
            }
            output.push('\n');
        }

        output
    }
}

impl Default for WasiManifest {
    fn default() -> Self {
        Self::new()
    }
}

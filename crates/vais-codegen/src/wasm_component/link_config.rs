//! Component linking configuration for wasi-preview2

use super::wasi::WasiManifest;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ComponentLinkConfig {
    /// Component model adapter module (for core -> component)
    pub adapter_module: Option<String>,
    /// Enable reactor mode (no start function)
    pub reactor_mode: bool,
    /// Enable command mode (with _start)
    pub command_mode: bool,
    /// Additional component-level imports
    pub component_imports: HashMap<String, String>,
    /// Additional component-level exports
    pub component_exports: HashMap<String, String>,
    /// WASI manifest for managing WASI interfaces
    pub wasi_manifest: Option<WasiManifest>,
}

impl Default for ComponentLinkConfig {
    fn default() -> Self {
        Self {
            adapter_module: None,
            reactor_mode: false,
            command_mode: true,
            component_imports: HashMap::new(),
            component_exports: HashMap::new(),
            wasi_manifest: None,
        }
    }
}

impl ComponentLinkConfig {
    /// Create a new component link configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable reactor mode (disable command mode)
    pub fn reactor(mut self) -> Self {
        self.reactor_mode = true;
        self.command_mode = false;
        self
    }

    /// Enable command mode (disable reactor mode)
    pub fn command(mut self) -> Self {
        self.reactor_mode = false;
        self.command_mode = true;
        self
    }

    /// Set adapter module path
    pub fn with_adapter(mut self, path: &str) -> Self {
        self.adapter_module = Some(path.to_string());
        self
    }

    /// Add component import
    pub fn add_import(&mut self, name: String, interface: String) {
        self.component_imports.insert(name, interface);
    }

    /// Add component export
    pub fn add_export(&mut self, name: String, interface: String) {
        self.component_exports.insert(name, interface);
    }

    /// Set WASI manifest
    pub fn with_wasi_manifest(mut self, manifest: WasiManifest) -> Self {
        self.wasi_manifest = Some(manifest);
        self
    }

    /// Get mutable reference to WASI manifest (creates one if not present)
    pub fn wasi_manifest_mut(&mut self) -> &mut WasiManifest {
        self.wasi_manifest.get_or_insert_with(WasiManifest::new)
    }

    /// Generate wasm-tools component-link flags
    pub fn to_link_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(adapter) = &self.adapter_module {
            args.push("--adapt".to_string());
            args.push(adapter.clone());
        }

        for (name, interface) in &self.component_imports {
            args.push("--import".to_string());
            args.push(format!("{}={}", name, interface));
        }

        for (name, interface) in &self.component_exports {
            args.push("--export".to_string());
            args.push(format!("{}={}", name, interface));
        }

        args
    }
}

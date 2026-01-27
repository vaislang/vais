//! Vais Dynamic Module Loading System
//!
//! Provides runtime module loading, WASM sandboxing, and plugin discovery.
//!
//! # Features
//!
//! - **Dynamic Module Loading**: Load and reload `.vais` modules at runtime
//! - **WASM Sandboxing**: Execute WASM plugins in a secure sandbox
//! - **Plugin Discovery**: Automatically discover plugins from standard paths
//! - **Hot Reload**: Support for hot-reloading modules during development
//!
//! # Example
//!
//! ```ignore
//! use vais_dynload::{ModuleLoader, WasmSandbox, PluginDiscovery};
//!
//! // Load a Vais module dynamically
//! let mut loader = ModuleLoader::new()?;
//! let module = loader.load("./my_module.vais")?;
//!
//! // Execute a WASM plugin in sandbox
//! let sandbox = WasmSandbox::new()?;
//! sandbox.load_plugin("./plugin.wasm")?;
//! let result = sandbox.call("process", &[42i32.into()])?;
//!
//! // Discover plugins from standard paths
//! let discovery = PluginDiscovery::new();
//! let plugins = discovery.scan_all()?;
//! ```

mod error;
mod module_loader;
mod wasm_sandbox;
mod plugin_discovery;
mod manifest;
mod host_functions;
mod resource_limits;

pub use error::{DynloadError, Result};
pub use module_loader::{ModuleLoader, ModuleLoaderConfig, LoadedModule, ModuleHandle, ReloadEvent};
pub use wasm_sandbox::{WasmSandbox, WasmInstance, SandboxConfig, SandboxState};
pub use plugin_discovery::{PluginDiscovery, DiscoveredPlugin, PluginSource, DiscoveryConfig};
pub use manifest::{PluginManifest, PluginCapability, PluginDependency, PluginFormat};
pub use host_functions::{HostFunctions, HostFunction, HostFunctionRegistry};
pub use resource_limits::{ResourceLimits, MemoryLimit, TimeLimit, StackLimit, ResourceUsage};

/// Re-export commonly used types from dependent crates
pub use vais_plugin::{PluginInfo, PluginType};
pub use vais_hotreload::{HotReloader, HotReloadConfig};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_compiles() {
        // Basic compilation test
        assert!(true);
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert!(limits.memory.max_bytes > 0);
        assert!(limits.time.max_duration_ms > 0);
    }
}

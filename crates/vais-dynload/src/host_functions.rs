//! Host functions for WASM plugins
//!
//! Defines the interface between host (Vais runtime) and guest (WASM plugin).
//! Host functions are functions provided by the host that can be called by plugins.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use wasmtime::{Caller, Linker};

use crate::error::{DynloadError, Result};
use crate::manifest::PluginCapability;

/// Type alias for host function implementation.
/// Reserved for dynamic host function registration.
#[allow(dead_code)]
pub type HostFunctionImpl =
    Box<dyn Fn(&[wasmtime::Val]) -> Result<Vec<wasmtime::Val>> + Send + Sync>;

/// A registered host function
pub struct HostFunction {
    /// Function name
    pub name: String,
    /// Module name (namespace)
    pub module: String,
    /// Required capability to use this function
    pub required_capability: Option<PluginCapability>,
    /// Function description
    pub description: String,
    /// Parameter types
    pub param_types: Vec<wasmtime::ValType>,
    /// Return types
    pub result_types: Vec<wasmtime::ValType>,
}

impl HostFunction {
    /// Create a new host function descriptor
    pub fn new(name: &str, module: &str) -> Self {
        Self {
            name: name.to_string(),
            module: module.to_string(),
            required_capability: None,
            description: String::new(),
            param_types: vec![],
            result_types: vec![],
        }
    }

    /// Set required capability
    pub fn with_capability(mut self, cap: PluginCapability) -> Self {
        self.required_capability = Some(cap);
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Set parameter types
    pub fn with_params(mut self, params: Vec<wasmtime::ValType>) -> Self {
        self.param_types = params;
        self
    }

    /// Set return types
    pub fn with_results(mut self, results: Vec<wasmtime::ValType>) -> Self {
        self.result_types = results;
        self
    }
}

/// Host function registry
pub struct HostFunctionRegistry {
    /// Registered functions by (module, name)
    functions: HashMap<(String, String), Arc<HostFunction>>,
    /// Granted capabilities for current plugin
    granted_capabilities: RwLock<Vec<PluginCapability>>,
}

impl HostFunctionRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            granted_capabilities: RwLock::new(vec![]),
        }
    }

    /// Create a registry with standard host functions
    pub fn with_standard_functions() -> Self {
        let mut registry = Self::new();
        registry.register_standard_functions();
        registry
    }

    /// Register a host function
    pub fn register(&mut self, func: HostFunction) {
        let key = (func.module.clone(), func.name.clone());
        self.functions.insert(key, Arc::new(func));
    }

    /// Get a function by module and name
    pub fn get(&self, module: &str, name: &str) -> Option<&Arc<HostFunction>> {
        self.functions.get(&(module.to_string(), name.to_string()))
    }

    /// List all registered functions
    pub fn list(&self) -> Vec<&Arc<HostFunction>> {
        self.functions.values().collect()
    }

    /// Grant a capability
    pub fn grant_capability(&self, cap: PluginCapability) {
        let mut caps = self.granted_capabilities.write();
        if !caps.contains(&cap) {
            caps.push(cap);
        }
    }

    /// Revoke a capability
    pub fn revoke_capability(&self, cap: &PluginCapability) {
        let mut caps = self.granted_capabilities.write();
        caps.retain(|c| c != cap);
    }

    /// Check if a capability is granted
    pub fn has_capability(&self, cap: &PluginCapability) -> bool {
        self.granted_capabilities.read().contains(cap)
    }

    /// Check if a function call is allowed
    pub fn is_call_allowed(&self, module: &str, name: &str) -> Result<()> {
        if let Some(func) = self.get(module, name) {
            if let Some(ref required) = func.required_capability {
                if !self.has_capability(required) {
                    return Err(DynloadError::SecurityViolation(format!(
                        "Function '{}.{}' requires capability {:?}",
                        module, name, required
                    )));
                }
            }
            Ok(())
        } else {
            Err(DynloadError::HostFunctionNotRegistered(format!(
                "{}.{}",
                module, name
            )))
        }
    }

    /// Register standard Vais host functions
    fn register_standard_functions(&mut self) {
        // Console functions
        self.register(
            HostFunction::new("print", "vais")
                .with_capability(PluginCapability::Console)
                .with_description("Print a string to console")
                .with_params(vec![wasmtime::ValType::I32, wasmtime::ValType::I32]),
        );

        self.register(
            HostFunction::new("println", "vais")
                .with_capability(PluginCapability::Console)
                .with_description("Print a string with newline")
                .with_params(vec![wasmtime::ValType::I32, wasmtime::ValType::I32]),
        );

        self.register(
            HostFunction::new("log", "vais")
                .with_capability(PluginCapability::Console)
                .with_description("Log a message with level")
                .with_params(vec![
                    wasmtime::ValType::I32, // level
                    wasmtime::ValType::I32, // ptr
                    wasmtime::ValType::I32, // len
                ]),
        );

        // Time functions
        self.register(
            HostFunction::new("now_ms", "vais")
                .with_capability(PluginCapability::Time)
                .with_description("Get current time in milliseconds")
                .with_results(vec![wasmtime::ValType::I64]),
        );

        self.register(
            HostFunction::new("sleep_ms", "vais")
                .with_capability(PluginCapability::Time)
                .with_description("Sleep for specified milliseconds")
                .with_params(vec![wasmtime::ValType::I64]),
        );

        // Random functions
        self.register(
            HostFunction::new("random", "vais")
                .with_capability(PluginCapability::Random)
                .with_description("Generate random u64")
                .with_results(vec![wasmtime::ValType::I64]),
        );

        self.register(
            HostFunction::new("random_range", "vais")
                .with_capability(PluginCapability::Random)
                .with_description("Generate random number in range")
                .with_params(vec![wasmtime::ValType::I64, wasmtime::ValType::I64])
                .with_results(vec![wasmtime::ValType::I64]),
        );

        // Environment functions
        self.register(
            HostFunction::new("get_env", "vais")
                .with_capability(PluginCapability::Env)
                .with_description("Get environment variable")
                .with_params(vec![wasmtime::ValType::I32, wasmtime::ValType::I32])
                .with_results(vec![wasmtime::ValType::I32]),
        );

        // File system functions (read)
        self.register(
            HostFunction::new("read_file", "vais_fs")
                .with_capability(PluginCapability::FsRead)
                .with_description("Read file contents")
                .with_params(vec![wasmtime::ValType::I32, wasmtime::ValType::I32])
                .with_results(vec![wasmtime::ValType::I32]),
        );

        self.register(
            HostFunction::new("file_exists", "vais_fs")
                .with_capability(PluginCapability::FsRead)
                .with_description("Check if file exists")
                .with_params(vec![wasmtime::ValType::I32, wasmtime::ValType::I32])
                .with_results(vec![wasmtime::ValType::I32]),
        );

        // File system functions (write)
        self.register(
            HostFunction::new("write_file", "vais_fs")
                .with_capability(PluginCapability::FsWrite)
                .with_description("Write file contents")
                .with_params(vec![
                    wasmtime::ValType::I32, // path ptr
                    wasmtime::ValType::I32, // path len
                    wasmtime::ValType::I32, // data ptr
                    wasmtime::ValType::I32, // data len
                ])
                .with_results(vec![wasmtime::ValType::I32]),
        );

        // Memory functions (always available)
        self.register(
            HostFunction::new("alloc", "vais")
                .with_description("Allocate memory")
                .with_params(vec![wasmtime::ValType::I32])
                .with_results(vec![wasmtime::ValType::I32]),
        );

        self.register(
            HostFunction::new("dealloc", "vais")
                .with_description("Deallocate memory")
                .with_params(vec![wasmtime::ValType::I32, wasmtime::ValType::I32]),
        );

        // Network functions
        self.register(
            HostFunction::new("http_get", "vais_net")
                .with_capability(PluginCapability::Network)
                .with_description("Make HTTP GET request")
                .with_params(vec![wasmtime::ValType::I32, wasmtime::ValType::I32])
                .with_results(vec![wasmtime::ValType::I32]),
        );

        self.register(
            HostFunction::new("http_post", "vais_net")
                .with_capability(PluginCapability::Network)
                .with_description("Make HTTP POST request")
                .with_params(vec![
                    wasmtime::ValType::I32, // url ptr
                    wasmtime::ValType::I32, // url len
                    wasmtime::ValType::I32, // body ptr
                    wasmtime::ValType::I32, // body len
                ])
                .with_results(vec![wasmtime::ValType::I32]),
        );
    }
}

impl Default for HostFunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Host functions implementation for linking with WASM
pub struct HostFunctions {
    registry: Arc<HostFunctionRegistry>,
}

impl HostFunctions {
    /// Create new host functions with a registry
    pub fn new(registry: Arc<HostFunctionRegistry>) -> Self {
        Self { registry }
    }

    /// Create with standard functions
    pub fn with_standard() -> Self {
        Self::new(Arc::new(HostFunctionRegistry::with_standard_functions()))
    }

    /// Get the registry
    pub fn registry(&self) -> &Arc<HostFunctionRegistry> {
        &self.registry
    }

    /// Link host functions to a WASM linker
    pub fn link_to<T: Send + 'static>(&self, linker: &mut Linker<T>) -> Result<()> {
        // Link console functions
        self.link_console_functions(linker)?;

        // Link time functions
        self.link_time_functions(linker)?;

        // Link random functions
        self.link_random_functions(linker)?;

        // Link memory functions
        self.link_memory_functions(linker)?;

        Ok(())
    }

    fn link_console_functions<T: Send + 'static>(&self, linker: &mut Linker<T>) -> Result<()> {
        // print function
        linker
            .func_wrap(
                "vais",
                "print",
                |mut caller: Caller<'_, T>, ptr: i32, len: i32| {
                    if let Some(memory) = caller.get_export("memory") {
                        if let Some(mem) = memory.into_memory() {
                            let data = mem.data(&caller);
                            if let Some(slice) = data.get(ptr as usize..(ptr + len) as usize) {
                                if let Ok(s) = std::str::from_utf8(slice) {
                                    print!("{}", s);
                                }
                            }
                        }
                    }
                },
            )
            .map_err(|e| DynloadError::WasmInstantiationError(e.to_string()))?;

        // println function
        linker
            .func_wrap(
                "vais",
                "println",
                |mut caller: Caller<'_, T>, ptr: i32, len: i32| {
                    if let Some(memory) = caller.get_export("memory") {
                        if let Some(mem) = memory.into_memory() {
                            let data = mem.data(&caller);
                            if let Some(slice) = data.get(ptr as usize..(ptr + len) as usize) {
                                if let Ok(s) = std::str::from_utf8(slice) {
                                    println!("{}", s);
                                }
                            }
                        }
                    }
                },
            )
            .map_err(|e| DynloadError::WasmInstantiationError(e.to_string()))?;

        // log function
        linker
            .func_wrap(
                "vais",
                "log",
                |mut caller: Caller<'_, T>, level: i32, ptr: i32, len: i32| {
                    let level_str = match level {
                        0 => "TRACE",
                        1 => "DEBUG",
                        2 => "INFO",
                        3 => "WARN",
                        4 => "ERROR",
                        _ => "UNKNOWN",
                    };

                    if let Some(memory) = caller.get_export("memory") {
                        if let Some(mem) = memory.into_memory() {
                            let data = mem.data(&caller);
                            if let Some(slice) = data.get(ptr as usize..(ptr + len) as usize) {
                                if let Ok(s) = std::str::from_utf8(slice) {
                                    println!("[{}] {}", level_str, s);
                                }
                            }
                        }
                    }
                },
            )
            .map_err(|e| DynloadError::WasmInstantiationError(e.to_string()))?;

        Ok(())
    }

    fn link_time_functions<T: Send + 'static>(&self, linker: &mut Linker<T>) -> Result<()> {
        // now_ms function
        linker
            .func_wrap("vais", "now_ms", || -> i64 {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0)
            })
            .map_err(|e| DynloadError::WasmInstantiationError(e.to_string()))?;

        // sleep_ms function
        linker
            .func_wrap("vais", "sleep_ms", |ms: i64| {
                std::thread::sleep(std::time::Duration::from_millis(ms as u64));
            })
            .map_err(|e| DynloadError::WasmInstantiationError(e.to_string()))?;

        Ok(())
    }

    fn link_random_functions<T: Send + 'static>(&self, linker: &mut Linker<T>) -> Result<()> {
        // random function
        linker
            .func_wrap("vais", "random", || -> i64 {
                use std::collections::hash_map::RandomState;
                use std::hash::{BuildHasher, Hasher};
                let state = RandomState::new();
                let mut hasher = state.build_hasher();
                hasher.write_u64(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64,
                );
                hasher.finish() as i64
            })
            .map_err(|e| DynloadError::WasmInstantiationError(e.to_string()))?;

        // random_range function
        linker
            .func_wrap("vais", "random_range", |min: i64, max: i64| -> i64 {
                if min >= max {
                    return min;
                }
                use std::collections::hash_map::RandomState;
                use std::hash::{BuildHasher, Hasher};
                let state = RandomState::new();
                let mut hasher = state.build_hasher();
                hasher.write_u64(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64,
                );
                let random = hasher.finish();
                min + (random as i64).abs() % (max - min)
            })
            .map_err(|e| DynloadError::WasmInstantiationError(e.to_string()))?;

        Ok(())
    }

    fn link_memory_functions<T: Send + 'static>(&self, linker: &mut Linker<T>) -> Result<()> {
        // Note: Memory allocation is typically handled by the WASM module itself
        // These are placeholder implementations

        // alloc function (stub - real implementation depends on WASM module's allocator)
        linker
            .func_wrap("vais", "alloc", |_size: i32| -> i32 {
                // In practice, this would call the WASM module's allocator
                0
            })
            .map_err(|e| DynloadError::WasmInstantiationError(e.to_string()))?;

        // dealloc function (stub)
        linker
            .func_wrap("vais", "dealloc", |_ptr: i32, _size: i32| {
                // In practice, this would call the WASM module's deallocator
            })
            .map_err(|e| DynloadError::WasmInstantiationError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_function_creation() {
        let func = HostFunction::new("test", "module")
            .with_capability(PluginCapability::Console)
            .with_description("Test function")
            .with_params(vec![wasmtime::ValType::I32])
            .with_results(vec![wasmtime::ValType::I64]);

        assert_eq!(func.name, "test");
        assert_eq!(func.module, "module");
        assert_eq!(func.required_capability, Some(PluginCapability::Console));
        assert_eq!(func.param_types.len(), 1);
        assert_eq!(func.result_types.len(), 1);
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = HostFunctionRegistry::new();

        registry.register(HostFunction::new("test", "module"));

        assert!(registry.get("module", "test").is_some());
        assert!(registry.get("module", "nonexistent").is_none());
    }

    #[test]
    fn test_registry_capabilities() {
        let registry = HostFunctionRegistry::new();

        assert!(!registry.has_capability(&PluginCapability::Console));

        registry.grant_capability(PluginCapability::Console);
        assert!(registry.has_capability(&PluginCapability::Console));

        registry.revoke_capability(&PluginCapability::Console);
        assert!(!registry.has_capability(&PluginCapability::Console));
    }

    #[test]
    fn test_registry_call_allowed() {
        let mut registry = HostFunctionRegistry::new();

        registry.register(
            HostFunction::new("secure_fn", "test").with_capability(PluginCapability::FsWrite),
        );

        // Should fail without capability
        assert!(registry.is_call_allowed("test", "secure_fn").is_err());

        // Should succeed with capability
        registry.grant_capability(PluginCapability::FsWrite);
        assert!(registry.is_call_allowed("test", "secure_fn").is_ok());
    }

    #[test]
    fn test_standard_functions() {
        let registry = HostFunctionRegistry::with_standard_functions();

        // Check some standard functions exist
        assert!(registry.get("vais", "print").is_some());
        assert!(registry.get("vais", "println").is_some());
        assert!(registry.get("vais", "now_ms").is_some());
        assert!(registry.get("vais", "random").is_some());
    }

    #[test]
    fn test_registry_default() {
        let registry = HostFunctionRegistry::default();
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_registry_list() {
        let mut registry = HostFunctionRegistry::new();
        registry.register(HostFunction::new("fn1", "mod1"));
        registry.register(HostFunction::new("fn2", "mod1"));
        registry.register(HostFunction::new("fn3", "mod2"));

        let funcs = registry.list();
        assert_eq!(funcs.len(), 3);
    }

    #[test]
    fn test_host_function_no_capability() {
        let func = HostFunction::new("open", "module");
        assert!(func.required_capability.is_none());
        assert!(func.description.is_empty());
        assert!(func.param_types.is_empty());
        assert!(func.result_types.is_empty());
    }

    #[test]
    fn test_grant_capability_idempotent() {
        let registry = HostFunctionRegistry::new();
        registry.grant_capability(PluginCapability::Console);
        registry.grant_capability(PluginCapability::Console);
        // Should still just have one Console capability
        assert!(registry.has_capability(&PluginCapability::Console));
    }

    #[test]
    fn test_revoke_nonexistent_capability() {
        let registry = HostFunctionRegistry::new();
        registry.revoke_capability(&PluginCapability::Network);
        // Should not panic
        assert!(!registry.has_capability(&PluginCapability::Network));
    }

    #[test]
    fn test_is_call_allowed_not_registered() {
        let registry = HostFunctionRegistry::new();
        let result = registry.is_call_allowed("unknown", "fn");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DynloadError::HostFunctionNotRegistered(_)
        ));
    }

    #[test]
    fn test_is_call_allowed_no_capability_needed() {
        let mut registry = HostFunctionRegistry::new();
        registry.register(HostFunction::new("free_fn", "mod"));
        // No capability required, should be allowed
        assert!(registry.is_call_allowed("mod", "free_fn").is_ok());
    }

    #[test]
    fn test_is_call_allowed_security_violation() {
        let mut registry = HostFunctionRegistry::new();
        registry
            .register(HostFunction::new("write", "fs").with_capability(PluginCapability::FsWrite));
        let result = registry.is_call_allowed("fs", "write");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DynloadError::SecurityViolation(_)
        ));
    }

    #[test]
    fn test_standard_functions_with_capabilities() {
        let registry = HostFunctionRegistry::with_standard_functions();

        // Verify print requires Console capability
        let print_fn = registry.get("vais", "print").unwrap();
        assert_eq!(
            print_fn.required_capability,
            Some(PluginCapability::Console)
        );

        // Verify alloc has no required capability
        let alloc_fn = registry.get("vais", "alloc").unwrap();
        assert!(alloc_fn.required_capability.is_none());

        // Verify network functions exist
        assert!(registry.get("vais_net", "http_get").is_some());
        assert!(registry.get("vais_net", "http_post").is_some());

        // Verify fs functions exist
        assert!(registry.get("vais_fs", "read_file").is_some());
        assert!(registry.get("vais_fs", "write_file").is_some());
        assert!(registry.get("vais_fs", "file_exists").is_some());
    }

    #[test]
    fn test_host_functions_with_standard() {
        let host_fns = HostFunctions::with_standard();
        let registry = host_fns.registry();
        assert!(registry.get("vais", "print").is_some());
    }

    #[test]
    fn test_host_functions_registry_access() {
        let registry = Arc::new(HostFunctionRegistry::new());
        let host_fns = HostFunctions::new(registry.clone());
        assert!(Arc::ptr_eq(host_fns.registry(), &registry));
    }

    #[test]
    fn test_multiple_capabilities() {
        let registry = HostFunctionRegistry::new();
        registry.grant_capability(PluginCapability::Console);
        registry.grant_capability(PluginCapability::Time);
        registry.grant_capability(PluginCapability::Random);

        assert!(registry.has_capability(&PluginCapability::Console));
        assert!(registry.has_capability(&PluginCapability::Time));
        assert!(registry.has_capability(&PluginCapability::Random));
        assert!(!registry.has_capability(&PluginCapability::Network));

        registry.revoke_capability(&PluginCapability::Time);
        assert!(!registry.has_capability(&PluginCapability::Time));
        assert!(registry.has_capability(&PluginCapability::Console));
    }
}

//! WASM plugin sandboxing
//!
//! Provides a secure sandbox environment for executing WASM plugins
//! with resource limits and capability-based security.

use parking_lot::Mutex;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use wasmtime::*;

use crate::error::{DynloadError, Result};
use crate::host_functions::{HostFunctionRegistry, HostFunctions};
use crate::manifest::PluginCapability;
use crate::resource_limits::{ResourceLimits, ResourceUsage};

/// Configuration for the WASM sandbox
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Resource limits
    pub limits: ResourceLimits,
    /// Granted capabilities
    pub capabilities: Vec<PluginCapability>,
    /// Enable debug mode
    pub debug: bool,
    /// Enable WASI support
    pub enable_wasi: bool,
    /// Maximum compilation cache size
    pub cache_size: usize,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            limits: ResourceLimits::default(),
            capabilities: vec![PluginCapability::Console],
            debug: false,
            enable_wasi: false,
            cache_size: 10,
        }
    }
}

impl SandboxConfig {
    /// Create a new default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a restrictive configuration for untrusted plugins
    pub fn restrictive() -> Self {
        Self {
            limits: ResourceLimits::restrictive(),
            capabilities: vec![],
            debug: false,
            enable_wasi: false,
            cache_size: 5,
        }
    }

    /// Create a permissive configuration for trusted plugins
    pub fn permissive() -> Self {
        Self {
            limits: ResourceLimits::permissive(),
            capabilities: vec![
                PluginCapability::Console,
                PluginCapability::Time,
                PluginCapability::Random,
                PluginCapability::FsRead,
            ],
            debug: true,
            enable_wasi: true,
            cache_size: 20,
        }
    }

    /// Set resource limits
    pub fn with_limits(mut self, limits: ResourceLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Grant a capability
    pub fn with_capability(mut self, cap: PluginCapability) -> Self {
        if !self.capabilities.contains(&cap) {
            self.capabilities.push(cap);
        }
        self
    }

    /// Grant multiple capabilities
    pub fn with_capabilities(mut self, caps: Vec<PluginCapability>) -> Self {
        for cap in caps {
            if !self.capabilities.contains(&cap) {
                self.capabilities.push(cap);
            }
        }
        self
    }

    /// Enable debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Enable WASI support
    pub fn with_wasi(mut self, enable: bool) -> Self {
        self.enable_wasi = enable;
        self
    }
}

/// Store data for WASM instances
pub struct SandboxState {
    /// Host function registry
    pub registry: Arc<HostFunctionRegistry>,
    /// Resource usage tracker
    pub usage: ResourceUsage,
    /// Resource limits
    pub limits: ResourceLimits,
    /// Start time for timeout tracking
    pub start_time: Instant,
}

/// A WASM sandbox for secure plugin execution
pub struct WasmSandbox {
    /// Wasmtime engine
    engine: Engine,
    /// Sandbox configuration
    config: SandboxConfig,
    /// Host function registry
    host_registry: Arc<HostFunctionRegistry>,
    /// Compiled modules cache
    module_cache: Mutex<Vec<(String, Module)>>,
}

impl WasmSandbox {
    /// Create a new WASM sandbox with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(SandboxConfig::default())
    }

    /// Create a new WASM sandbox with custom configuration
    pub fn with_config(config: SandboxConfig) -> Result<Self> {
        // Validate configuration
        config
            .limits
            .validate()
            .map_err(|e| DynloadError::ConfigError(format!("Invalid resource limits: {}", e)))?;

        // Create engine configuration
        let mut engine_config = Config::new();

        // Enable fuel consumption for execution limits
        if config.limits.time.fuel_limit.is_some() {
            engine_config.consume_fuel(true);
        }

        // Enable epoch interruption for timeouts
        if config.limits.time.use_epoch_interruption {
            engine_config.epoch_interruption(true);
        }

        // Set stack size limit
        engine_config.max_wasm_stack(config.limits.stack.max_bytes as usize);

        // Enable debug if requested
        if config.debug {
            engine_config.debug_info(true);
        }

        let engine = Engine::new(&engine_config)
            .map_err(|e| DynloadError::WasmInstantiationError(e.to_string()))?;

        // Create host function registry with capabilities
        let registry = HostFunctionRegistry::with_standard_functions();
        for cap in &config.capabilities {
            registry.grant_capability(cap.clone());
        }

        Ok(Self {
            engine,
            config,
            host_registry: Arc::new(registry),
            module_cache: Mutex::new(Vec::new()),
        })
    }

    /// Get the sandbox configuration
    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }

    /// Grant a capability at runtime
    pub fn grant_capability(&self, cap: PluginCapability) {
        self.host_registry.grant_capability(cap);
    }

    /// Revoke a capability at runtime
    pub fn revoke_capability(&self, cap: &PluginCapability) {
        self.host_registry.revoke_capability(cap);
    }

    /// Load a WASM plugin from file
    pub fn load_plugin<P: AsRef<Path>>(&self, path: P) -> Result<WasmInstance> {
        let path = path.as_ref();
        let wasm_bytes = std::fs::read(path)?;
        self.load_plugin_bytes(&wasm_bytes, path.to_string_lossy().as_ref())
    }

    /// Load a WASM plugin from bytes
    pub fn load_plugin_bytes(&self, wasm_bytes: &[u8], name: &str) -> Result<WasmInstance> {
        // Check cache first
        {
            let cache = self.module_cache.lock();
            if let Some((_, module)) = cache.iter().find(|(n, _)| n == name) {
                return self.instantiate_module(module.clone(), name);
            }
        }

        // Compile the module
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| DynloadError::WasmValidationError(e.to_string()))?;

        // Add to cache
        {
            let mut cache = self.module_cache.lock();
            if cache.len() >= self.config.cache_size {
                cache.remove(0);
            }
            cache.push((name.to_string(), module.clone()));
        }

        self.instantiate_module(module, name)
    }

    /// Load a WASM plugin from WAT (WebAssembly Text) format
    pub fn load_plugin_wat(&self, wat_source: &str, name: &str) -> Result<WasmInstance> {
        // Parse WAT to WASM bytes using wasmtime's built-in wat parser
        let module = Module::new(&self.engine, wat_source)
            .map_err(|e| DynloadError::WasmValidationError(e.to_string()))?;

        // Add to cache
        {
            let mut cache = self.module_cache.lock();
            if cache.len() >= self.config.cache_size {
                cache.remove(0);
            }
            cache.push((name.to_string(), module.clone()));
        }

        self.instantiate_module(module, name)
    }

    /// Instantiate a compiled module
    fn instantiate_module(&self, module: Module, name: &str) -> Result<WasmInstance> {
        // Create store with state
        let state = SandboxState {
            registry: self.host_registry.clone(),
            usage: ResourceUsage::new(),
            limits: self.config.limits.clone(),
            start_time: Instant::now(),
        };

        let mut store = Store::new(&self.engine, state);

        // Set fuel limit if configured
        if let Some(fuel) = self.config.limits.time.fuel_limit {
            store.set_fuel(fuel).map_err(|e| {
                DynloadError::WasmInstantiationError(format!("Failed to set fuel: {}", e))
            })?;
        }

        // Set epoch deadline if configured
        if self.config.limits.time.use_epoch_interruption {
            store.epoch_deadline_trap();
        }

        // Create linker and add host functions
        let mut linker = Linker::new(&self.engine);
        let host_functions = HostFunctions::new(self.host_registry.clone());
        host_functions.link_to(&mut linker)?;

        // Instantiate the module
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| DynloadError::WasmInstantiationError(e.to_string()))?;

        Ok(WasmInstance {
            instance,
            store,
            name: name.to_string(),
            module,
        })
    }

    /// Clear the module cache
    pub fn clear_cache(&self) {
        self.module_cache.lock().clear();
    }
}

impl Default for WasmSandbox {
    fn default() -> Self {
        Self::new().expect("Failed to create default WASM sandbox")
    }
}

/// An instantiated WASM plugin
pub struct WasmInstance {
    instance: Instance,
    store: Store<SandboxState>,
    name: String,
    #[allow(dead_code)]
    module: Module,
}

impl WasmInstance {
    /// Get the instance name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Call a function with no arguments and no return value
    pub fn call_void(&mut self, name: &str) -> Result<()> {
        let func = self.get_func(name)?;
        self.track_call();

        func.call(&mut self.store, &[], &mut [])
            .map_err(|e| self.handle_trap(e))?;

        Ok(())
    }

    /// Call a function with i32 arguments and i32 return
    pub fn call_i32(&mut self, name: &str, args: &[i32]) -> Result<i32> {
        let func = self.get_func(name)?;
        self.track_call();

        let args: Vec<Val> = args.iter().map(|&a| Val::I32(a)).collect();
        let mut results = [Val::I32(0)];

        func.call(&mut self.store, &args, &mut results)
            .map_err(|e| self.handle_trap(e))?;

        match results[0] {
            Val::I32(v) => Ok(v),
            _ => Err(DynloadError::WasmTypeMismatch {
                expected: "i32".to_string(),
                actual: format!("{:?}", results[0]),
            }),
        }
    }

    /// Call a function with i64 arguments and i64 return
    pub fn call_i64(&mut self, name: &str, args: &[i64]) -> Result<i64> {
        let func = self.get_func(name)?;
        self.track_call();

        let args: Vec<Val> = args.iter().map(|&a| Val::I64(a)).collect();
        let mut results = [Val::I64(0)];

        func.call(&mut self.store, &args, &mut results)
            .map_err(|e| self.handle_trap(e))?;

        match results[0] {
            Val::I64(v) => Ok(v),
            _ => Err(DynloadError::WasmTypeMismatch {
                expected: "i64".to_string(),
                actual: format!("{:?}", results[0]),
            }),
        }
    }

    /// Call a function with generic Val arguments
    pub fn call(&mut self, name: &str, args: &[Val], results: &mut [Val]) -> Result<()> {
        let func = self.get_func(name)?;
        self.track_call();

        func.call(&mut self.store, args, results)
            .map_err(|e| self.handle_trap(e))?;

        Ok(())
    }

    /// Get a typed function
    pub fn get_typed_func<Params, Results>(
        &mut self,
        name: &str,
    ) -> Result<TypedFunc<Params, Results>>
    where
        Params: WasmParams,
        Results: WasmResults,
    {
        self.instance
            .get_typed_func::<Params, Results>(&mut self.store, name)
            .map_err(|e| DynloadError::WasmFunctionNotFound(format!("{}: {}", name, e)))
    }

    /// Read memory from the instance
    pub fn read_memory(&mut self, offset: usize, len: usize) -> Result<Vec<u8>> {
        let memory = self.get_memory()?;
        let data = memory.data(&self.store);

        if offset + len > data.len() {
            return Err(DynloadError::WasmExecutionError(
                "Memory read out of bounds".to_string(),
            ));
        }

        Ok(data[offset..offset + len].to_vec())
    }

    /// Write memory to the instance
    pub fn write_memory(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        let memory = self.get_memory()?;
        let mem_data = memory.data_mut(&mut self.store);

        if offset + data.len() > mem_data.len() {
            return Err(DynloadError::WasmExecutionError(
                "Memory write out of bounds".to_string(),
            ));
        }

        mem_data[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }

    /// Read a string from memory
    pub fn read_string(&mut self, offset: usize, len: usize) -> Result<String> {
        let bytes = self.read_memory(offset, len)?;
        String::from_utf8(bytes)
            .map_err(|e| DynloadError::WasmExecutionError(format!("Invalid UTF-8 string: {}", e)))
    }

    /// Get resource usage
    pub fn resource_usage(&self) -> &ResourceUsage {
        &self.store.data().usage
    }

    /// Check if limits are exceeded
    pub fn check_limits(&self) -> Result<()> {
        let state = self.store.data();
        let limits = &state.limits;
        let usage = &state.usage;

        if usage.exceeds_memory(&limits.memory) {
            return Err(DynloadError::MemoryLimitExceeded(usage.memory_bytes));
        }

        if usage.exceeds_time(&limits.time) {
            return Err(DynloadError::ExecutionTimeout(usage.execution_time_ms));
        }

        Ok(())
    }

    /// Get remaining fuel
    pub fn remaining_fuel(&self) -> Option<u64> {
        self.store.get_fuel().ok()
    }

    fn get_func(&mut self, name: &str) -> Result<Func> {
        self.instance
            .get_func(&mut self.store, name)
            .ok_or_else(|| DynloadError::WasmFunctionNotFound(name.to_string()))
    }

    fn get_memory(&mut self) -> Result<Memory> {
        self.instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| DynloadError::WasmExecutionError("No memory export found".to_string()))
    }

    fn track_call(&mut self) {
        let state = self.store.data_mut();
        state.usage.increment_calls();

        // Update execution time
        let elapsed = state.start_time.elapsed().as_millis() as u64;
        state.usage.execution_time_ms = elapsed;
    }

    fn handle_trap(&self, err: wasmtime::Error) -> DynloadError {
        let err_str = err.to_string();

        if err_str.contains("fuel") || err_str.contains("out of fuel") {
            DynloadError::ExecutionTimeout(self.store.data().usage.execution_time_ms)
        } else if err_str.contains("memory") {
            DynloadError::MemoryLimitExceeded(self.store.data().usage.memory_bytes)
        } else {
            DynloadError::WasmExecutionError(err_str)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let sandbox = WasmSandbox::new();
        assert!(sandbox.is_ok());
    }

    #[test]
    fn test_sandbox_config() {
        let config = SandboxConfig::new()
            .with_capability(PluginCapability::Console)
            .with_capability(PluginCapability::Time)
            .with_debug(true);

        assert!(config.capabilities.contains(&PluginCapability::Console));
        assert!(config.capabilities.contains(&PluginCapability::Time));
        assert!(config.debug);
    }

    #[test]
    fn test_sandbox_restrictive_config() {
        let config = SandboxConfig::restrictive();
        assert!(config.capabilities.is_empty());
        assert!(!config.debug);
        assert!(!config.enable_wasi);
    }

    #[test]
    fn test_sandbox_permissive_config() {
        let config = SandboxConfig::permissive();
        assert!(!config.capabilities.is_empty());
        assert!(config.debug);
        assert!(config.enable_wasi);
    }

    #[test]
    fn test_load_simple_wasm() {
        // Use permissive config to avoid fuel issues
        let config = SandboxConfig::permissive();
        let sandbox = WasmSandbox::with_config(config).unwrap();

        // Simple WASM module that exports an add function
        let wat = r#"
            (module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                )
            )
        "#;

        let mut instance = sandbox.load_plugin_wat(wat, "test").unwrap();

        let result = instance.call_i32("add", &[2, 3]).unwrap();
        assert_eq!(result, 5);
    }

    #[test]
    fn test_function_not_found() {
        let config = SandboxConfig::permissive();
        let sandbox = WasmSandbox::with_config(config).unwrap();

        let wat = r#"
            (module
                (func (export "foo"))
            )
        "#;

        let mut instance = sandbox.load_plugin_wat(wat, "test").unwrap();
        let result = instance.call_void("bar");

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DynloadError::WasmFunctionNotFound(_)
        ));
    }

    #[test]
    fn test_memory_operations() {
        let config = SandboxConfig::permissive();
        let sandbox = WasmSandbox::with_config(config).unwrap();

        let wat = r#"
            (module
                (memory (export "memory") 1)
                (func (export "store") (param i32 i32)
                    local.get 0
                    local.get 1
                    i32.store
                )
                (func (export "load") (param i32) (result i32)
                    local.get 0
                    i32.load
                )
            )
        "#;

        let mut instance = sandbox.load_plugin_wat(wat, "test").unwrap();

        // Write memory directly
        instance.write_memory(0, &42i32.to_le_bytes()).unwrap();

        // Read back
        let bytes = instance.read_memory(0, 4).unwrap();
        let value = i32::from_le_bytes(bytes.try_into().unwrap());
        assert_eq!(value, 42);
    }

    #[test]
    fn test_resource_tracking() {
        let config = SandboxConfig::permissive();
        let sandbox = WasmSandbox::with_config(config).unwrap();

        let wat = r#"
            (module
                (func (export "noop"))
            )
        "#;

        let mut instance = sandbox.load_plugin_wat(wat, "test").unwrap();
        instance.call_void("noop").unwrap();

        let usage = instance.resource_usage();
        assert!(usage.call_count > 0);
    }

    #[test]
    fn test_module_cache() {
        let config = SandboxConfig::permissive();
        let sandbox = WasmSandbox::with_config(config).unwrap();

        let wat = r#"(module (func (export "f")))"#;

        // Load twice - both go through load_plugin_wat which adds to cache
        let _inst1 = sandbox.load_plugin_wat(wat, "test").unwrap();
        let _inst2 = sandbox.load_plugin_wat(wat, "test").unwrap();

        // Verify cache has the module (may have 1 or 2 depending on whether cache hit)
        let cache = sandbox.module_cache.lock();
        assert!(!cache.is_empty());
    }

    #[test]
    fn test_clear_cache() {
        let config = SandboxConfig::permissive();
        let sandbox = WasmSandbox::with_config(config).unwrap();

        let wat = r#"(module (func (export "f")))"#;
        let _inst = sandbox.load_plugin_wat(wat, "test").unwrap();

        assert!(!sandbox.module_cache.lock().is_empty());

        sandbox.clear_cache();
        assert_eq!(sandbox.module_cache.lock().len(), 0);
    }
}

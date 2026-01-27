//! Error types for dynamic module loading

use std::path::PathBuf;
use thiserror::Error;

/// Result type for dynamic loading operations
pub type Result<T> = std::result::Result<T, DynloadError>;

/// Errors that can occur during dynamic module loading
#[derive(Debug, Error)]
pub enum DynloadError {
    /// Module file not found
    #[error("Module not found: {0}")]
    ModuleNotFound(PathBuf),

    /// Failed to compile module
    #[error("Compilation failed: {0}")]
    CompilationError(String),

    /// Failed to load dynamic library
    #[error("Failed to load library: {0}")]
    LibraryLoadError(String),

    /// Symbol not found in loaded module
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    /// Module is already loaded
    #[error("Module already loaded: {0}")]
    ModuleAlreadyLoaded(String),

    /// Module not loaded
    #[error("Module not loaded: {0}")]
    ModuleNotLoaded(String),

    /// WASM validation error
    #[error("WASM validation failed: {0}")]
    WasmValidationError(String),

    /// WASM instantiation error
    #[error("WASM instantiation failed: {0}")]
    WasmInstantiationError(String),

    /// WASM execution error
    #[error("WASM execution failed: {0}")]
    WasmExecutionError(String),

    /// WASM function not found
    #[error("WASM function not found: {0}")]
    WasmFunctionNotFound(String),

    /// WASM type mismatch
    #[error("WASM type mismatch: expected {expected}, got {actual}")]
    WasmTypeMismatch { expected: String, actual: String },

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {resource} (limit: {limit}, used: {used})")]
    ResourceLimitExceeded {
        resource: String,
        limit: String,
        used: String,
    },

    /// Memory limit exceeded
    #[error("Memory limit exceeded: {0} bytes")]
    MemoryLimitExceeded(u64),

    /// Execution timeout
    #[error("Execution timeout after {0}ms")]
    ExecutionTimeout(u64),

    /// Plugin manifest error
    #[error("Invalid plugin manifest: {0}")]
    ManifestError(String),

    /// Plugin version incompatibility
    #[error("Plugin version incompatible: {plugin} requires {required}, found {actual}")]
    VersionIncompatible {
        plugin: String,
        required: String,
        actual: String,
    },

    /// Plugin dependency not found
    #[error("Plugin dependency not found: {0}")]
    DependencyNotFound(String),

    /// Host function error
    #[error("Host function error: {0}")]
    HostFunctionError(String),

    /// Host function not registered
    #[error("Host function not registered: {0}")]
    HostFunctionNotRegistered(String),

    /// Sandbox security violation
    #[error("Sandbox security violation: {0}")]
    SecurityViolation(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Hot reload error
    #[error("Hot reload error: {0}")]
    HotReloadError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Plugin discovery error
    #[error("Plugin discovery error: {0}")]
    DiscoveryError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<vais_hotreload::HotReloadError> for DynloadError {
    fn from(err: vais_hotreload::HotReloadError) -> Self {
        DynloadError::HotReloadError(err.to_string())
    }
}

impl From<wasmtime::Error> for DynloadError {
    fn from(err: wasmtime::Error) -> Self {
        DynloadError::WasmExecutionError(err.to_string())
    }
}

impl From<toml::de::Error> for DynloadError {
    fn from(err: toml::de::Error) -> Self {
        DynloadError::ManifestError(err.to_string())
    }
}

impl From<semver::Error> for DynloadError {
    fn from(err: semver::Error) -> Self {
        DynloadError::VersionIncompatible {
            plugin: "unknown".to_string(),
            required: "unknown".to_string(),
            actual: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = DynloadError::ModuleNotFound(PathBuf::from("/test/module.vais"));
        assert!(err.to_string().contains("Module not found"));

        let err = DynloadError::CompilationError("syntax error".to_string());
        assert!(err.to_string().contains("Compilation failed"));

        let err = DynloadError::MemoryLimitExceeded(1024);
        assert!(err.to_string().contains("1024"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: DynloadError = io_err.into();
        assert!(matches!(err, DynloadError::IoError(_)));
    }
}

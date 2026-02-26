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

    #[test]
    fn test_error_display_all_variants() {
        let errors: Vec<DynloadError> = vec![
            DynloadError::ModuleNotFound(PathBuf::from("/test")),
            DynloadError::CompilationError("error".into()),
            DynloadError::LibraryLoadError("error".into()),
            DynloadError::SymbolNotFound("sym".into()),
            DynloadError::ModuleAlreadyLoaded("mod".into()),
            DynloadError::ModuleNotLoaded("mod".into()),
            DynloadError::WasmValidationError("error".into()),
            DynloadError::WasmInstantiationError("error".into()),
            DynloadError::WasmExecutionError("error".into()),
            DynloadError::WasmFunctionNotFound("fn".into()),
            DynloadError::WasmTypeMismatch {
                expected: "i32".into(),
                actual: "i64".into(),
            },
            DynloadError::ResourceLimitExceeded {
                resource: "memory".into(),
                limit: "1MB".into(),
                used: "2MB".into(),
            },
            DynloadError::MemoryLimitExceeded(1024),
            DynloadError::ExecutionTimeout(5000),
            DynloadError::ManifestError("error".into()),
            DynloadError::VersionIncompatible {
                plugin: "test".into(),
                required: ">=1.0".into(),
                actual: "0.5".into(),
            },
            DynloadError::DependencyNotFound("dep".into()),
            DynloadError::HostFunctionError("error".into()),
            DynloadError::HostFunctionNotRegistered("fn".into()),
            DynloadError::SecurityViolation("violation".into()),
            DynloadError::HotReloadError("error".into()),
            DynloadError::ConfigError("error".into()),
            DynloadError::DiscoveryError("error".into()),
            DynloadError::InternalError("error".into()),
        ];

        for err in errors {
            let display = err.to_string();
            assert!(!display.is_empty(), "Error display should not be empty");
        }
    }

    #[test]
    fn test_error_wasm_type_mismatch_display() {
        let err = DynloadError::WasmTypeMismatch {
            expected: "i32".to_string(),
            actual: "f64".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("i32"));
        assert!(display.contains("f64"));
    }

    #[test]
    fn test_error_resource_limit_display() {
        let err = DynloadError::ResourceLimitExceeded {
            resource: "memory".to_string(),
            limit: "64MB".to_string(),
            used: "128MB".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("memory"));
        assert!(display.contains("64MB"));
        assert!(display.contains("128MB"));
    }

    #[test]
    fn test_error_version_incompatible_display() {
        let err = DynloadError::VersionIncompatible {
            plugin: "my-plugin".to_string(),
            required: ">=1.0.0".to_string(),
            actual: "0.5.0".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("my-plugin"));
    }

    #[test]
    fn test_error_from_toml() {
        let result: std::result::Result<toml::Value, toml::de::Error> =
            toml::from_str("not valid toml {{{");
        if let Err(toml_err) = result {
            let err: DynloadError = toml_err.into();
            assert!(matches!(err, DynloadError::ManifestError(_)));
        }
    }

    #[test]
    fn test_error_manifest_error() {
        let err = DynloadError::ManifestError("bad manifest".to_string());
        assert!(err.to_string().contains("bad manifest"));
    }

    #[test]
    fn test_error_wasm_validation() {
        let err = DynloadError::WasmValidationError("validation fail".to_string());
        assert!(err.to_string().contains("validation fail"));
    }

    #[test]
    fn test_error_wasm_instantiation() {
        let err = DynloadError::WasmInstantiationError("instantiation fail".to_string());
        assert!(err.to_string().contains("instantiation fail"));
    }

    #[test]
    fn test_error_wasm_execution() {
        let err = DynloadError::WasmExecutionError("execution fail".to_string());
        assert!(err.to_string().contains("execution fail"));
    }

    #[test]
    fn test_error_wasm_function_not_found() {
        let err = DynloadError::WasmFunctionNotFound("missing_fn".to_string());
        assert!(err.to_string().contains("missing_fn"));
    }

    #[test]
    fn test_error_library_load() {
        let err = DynloadError::LibraryLoadError("load fail".to_string());
        assert!(err.to_string().contains("load fail"));
    }

    #[test]
    fn test_error_symbol_not_found() {
        let err = DynloadError::SymbolNotFound("my_symbol".to_string());
        assert!(err.to_string().contains("my_symbol"));
    }

    #[test]
    fn test_error_module_already_loaded() {
        let err = DynloadError::ModuleAlreadyLoaded("mod1".to_string());
        assert!(err.to_string().contains("mod1"));
    }

    #[test]
    fn test_error_module_not_loaded() {
        let err = DynloadError::ModuleNotLoaded("mod2".to_string());
        assert!(err.to_string().contains("mod2"));
    }

    #[test]
    fn test_error_dependency_not_found() {
        let err = DynloadError::DependencyNotFound("missing-dep".to_string());
        assert!(err.to_string().contains("missing-dep"));
    }

    #[test]
    fn test_error_host_function_error() {
        let err = DynloadError::HostFunctionError("host fn crashed".to_string());
        assert!(err.to_string().contains("host fn crashed"));
    }

    #[test]
    fn test_error_host_function_not_registered() {
        let err = DynloadError::HostFunctionNotRegistered("unknown_fn".to_string());
        assert!(err.to_string().contains("unknown_fn"));
    }

    #[test]
    fn test_error_security_violation() {
        let err = DynloadError::SecurityViolation("unauthorized access".to_string());
        assert!(err.to_string().contains("unauthorized access"));
    }

    #[test]
    fn test_error_hot_reload() {
        let err = DynloadError::HotReloadError("reload fail".to_string());
        assert!(err.to_string().contains("reload fail"));
    }

    #[test]
    fn test_error_config() {
        let err = DynloadError::ConfigError("bad config".to_string());
        assert!(err.to_string().contains("bad config"));
    }

    #[test]
    fn test_error_discovery() {
        let err = DynloadError::DiscoveryError("scan fail".to_string());
        assert!(err.to_string().contains("scan fail"));
    }

    #[test]
    fn test_error_internal() {
        let err = DynloadError::InternalError("internal panic".to_string());
        assert!(err.to_string().contains("internal panic"));
    }

    #[test]
    fn test_error_execution_timeout() {
        let err = DynloadError::ExecutionTimeout(5000);
        assert!(err.to_string().contains("5000"));
    }

    #[test]
    fn test_error_memory_limit() {
        let err = DynloadError::MemoryLimitExceeded(65536);
        assert!(err.to_string().contains("65536"));
    }

    #[test]
    fn test_error_compilation() {
        let err = DynloadError::CompilationError("failed to compile".to_string());
        assert!(err.to_string().contains("failed to compile"));
    }

    #[test]
    fn test_result_type_alias() {
        let ok: Result<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);

        let err: Result<i32> = Err(DynloadError::InternalError("test".to_string()));
        assert!(err.is_err());
    }
}

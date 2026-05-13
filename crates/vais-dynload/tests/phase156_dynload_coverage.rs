//! Phase 156 coverage tests for vais-dynload
//!
//! Adds +30 tests covering WASM sandbox capabilities, module loader path
//! resolution, and dynamic loading error cases.

use std::path::PathBuf;
use vais_dynload::{
    DynloadError, MemoryLimit, ModuleLoaderConfig, PluginCapability, ResourceLimits, SandboxConfig,
    StackLimit, TimeLimit, WasmSandbox,
};

// ─── SandboxConfig: capability management ────────────────────────────────────

#[test]
fn test_sandbox_config_default_has_console_capability() {
    let config = SandboxConfig::default();
    assert!(config.capabilities.contains(&PluginCapability::Console));
}

#[test]
fn test_sandbox_config_restrictive_has_no_capabilities() {
    let config = SandboxConfig::restrictive();
    assert!(config.capabilities.is_empty());
    assert!(!config.debug);
    assert!(!config.enable_wasi);
}

#[test]
fn test_sandbox_config_permissive_has_many_capabilities() {
    let config = SandboxConfig::permissive();
    assert!(config.capabilities.contains(&PluginCapability::Console));
    assert!(config.capabilities.contains(&PluginCapability::Time));
    assert!(config.capabilities.contains(&PluginCapability::Random));
    assert!(config.capabilities.contains(&PluginCapability::FsRead));
    assert!(config.debug);
    assert!(config.enable_wasi);
}

#[test]
fn test_sandbox_config_duplicate_capability_not_added() {
    let config = SandboxConfig::new()
        .with_capability(PluginCapability::Console)
        .with_capability(PluginCapability::Console);
    let count = config
        .capabilities
        .iter()
        .filter(|&&ref c| *c == PluginCapability::Console)
        .count();
    // Console was already in default config; adding again should not duplicate
    assert_eq!(count, 1);
}

#[test]
fn test_sandbox_config_with_capabilities_dedup() {
    let config = SandboxConfig::restrictive()
        .with_capabilities(vec![PluginCapability::Time, PluginCapability::Time]);
    let time_count = config
        .capabilities
        .iter()
        .filter(|&&ref c| *c == PluginCapability::Time)
        .count();
    assert_eq!(time_count, 1);
}

#[test]
fn test_sandbox_config_debug_default_false() {
    let config = SandboxConfig::new();
    assert!(!config.debug);
}

#[test]
fn test_sandbox_config_wasi_default_false() {
    let config = SandboxConfig::new();
    assert!(!config.enable_wasi);
}

#[test]
fn test_sandbox_config_cache_size_default() {
    let config = SandboxConfig::default();
    assert_eq!(config.cache_size, 10);
}

#[test]
fn test_sandbox_config_restrictive_cache_size() {
    let config = SandboxConfig::restrictive();
    assert_eq!(config.cache_size, 5);
}

#[test]
fn test_sandbox_config_permissive_cache_size() {
    let config = SandboxConfig::permissive();
    assert_eq!(config.cache_size, 20);
}

// ─── WasmSandbox: instance operations ────────────────────────────────────────

fn permissive_sandbox() -> WasmSandbox {
    WasmSandbox::with_config(SandboxConfig::permissive()).unwrap()
}

#[test]
fn test_wasm_sandbox_creation_with_restrictive_config() {
    let result = WasmSandbox::with_config(SandboxConfig::restrictive());
    assert!(result.is_ok());
}

#[test]
fn test_wasm_sandbox_grant_capability_at_runtime() {
    // Grant a capability and verify by loading a WAT module that succeeds
    let sandbox = WasmSandbox::with_config(SandboxConfig::restrictive()).unwrap();
    sandbox.grant_capability(PluginCapability::Time);
    // config() returns the original config; grant/revoke mutate the registry only
    // Just verify grant_capability doesn't panic
}

#[test]
fn test_wasm_sandbox_revoke_capability_at_runtime() {
    let sandbox = permissive_sandbox();
    // Revoke a capability — should not panic
    sandbox.revoke_capability(&PluginCapability::Console);
    // Verify the sandbox still works after revoke
    let wat = r#"(module (func (export "f")))"#;
    let result = sandbox.load_plugin_wat(wat, "after_revoke");
    assert!(result.is_ok());
}

#[test]
fn test_wasm_invalid_bytes_returns_validation_error() {
    let sandbox = permissive_sandbox();
    let result = sandbox.load_plugin_bytes(b"\x00invalid", "bad");
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err, DynloadError::WasmValidationError(_)));
    }
}

#[test]
fn test_wasm_empty_bytes_returns_validation_error() {
    let sandbox = permissive_sandbox();
    let result = sandbox.load_plugin_bytes(b"", "empty");
    assert!(result.is_err());
}

#[test]
fn test_wasm_call_nonexistent_function_returns_error() {
    let sandbox = permissive_sandbox();
    let wat = r#"(module (func (export "exists")))"#;
    let mut instance = sandbox.load_plugin_wat(wat, "test").unwrap();
    let result = instance.call_void("does_not_exist");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DynloadError::WasmFunctionNotFound(_)
    ));
}

#[test]
fn test_wasm_i32_arithmetic_subtract() {
    let sandbox = permissive_sandbox();
    let wat = r#"
        (module
            (func (export "sub") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.sub
            )
        )
    "#;
    let mut instance = sandbox.load_plugin_wat(wat, "sub_test").unwrap();
    let result = instance.call_i32("sub", &[100, 37]).unwrap();
    assert_eq!(result, 63);
}

#[test]
fn test_wasm_i64_identity_function() {
    let sandbox = permissive_sandbox();
    let wat = r#"
        (module
            (func (export "id") (param i64) (result i64)
                local.get 0
            )
        )
    "#;
    let mut instance = sandbox.load_plugin_wat(wat, "id_test").unwrap();
    let result = instance.call_i64("id", &[123456789i64]).unwrap();
    assert_eq!(result, 123456789i64);
}

#[test]
fn test_wasm_instance_name_preserved() {
    let sandbox = permissive_sandbox();
    let wat = r#"(module)"#;
    let instance = sandbox.load_plugin_wat(wat, "my-unique-name").unwrap();
    assert_eq!(instance.name(), "my-unique-name");
}

#[test]
fn test_wasm_module_cache_cleared_after_clear_cache() {
    let sandbox = permissive_sandbox();
    let wat = r#"(module (func (export "noop")))"#;
    let _ = sandbox.load_plugin_wat(wat, "cached_mod").unwrap();
    sandbox.clear_cache();
    // After clear, loading the same name should recompile without error
    let result = sandbox.load_plugin_wat(wat, "cached_mod");
    assert!(result.is_ok());
}

#[test]
fn test_wasm_remaining_fuel_positive_after_load() {
    let sandbox = permissive_sandbox();
    let wat = r#"(module (func (export "f")))"#;
    let instance = sandbox.load_plugin_wat(wat, "fuel_test").unwrap();
    let fuel = instance.remaining_fuel();
    assert!(fuel.is_some());
    assert!(fuel.unwrap() > 0);
}

#[test]
fn test_wasm_check_limits_ok_immediately_after_load() {
    let sandbox = permissive_sandbox();
    let wat = r#"(module (func (export "f")))"#;
    let instance = sandbox.load_plugin_wat(wat, "limits_test").unwrap();
    assert!(instance.check_limits().is_ok());
}

// ─── ModuleLoaderConfig: path resolution ─────────────────────────────────────

#[test]
fn test_module_loader_config_default_values() {
    let config = ModuleLoaderConfig::default();
    assert_eq!(config.compiler_command, "vaisc");
    assert!(config.compiler_args.is_empty());
    assert!(config.output_dir.is_none());
    assert!(config.hot_reload);
    assert_eq!(config.debounce_ms, 100);
    assert_eq!(config.max_cache_size, 50);
}

#[test]
fn test_module_loader_config_with_compiler_sets_command() {
    let config = ModuleLoaderConfig::new().with_compiler("my-vaisc");
    assert_eq!(config.compiler_command, "my-vaisc");
}

#[test]
fn test_module_loader_config_with_args_sets_args() {
    let config = ModuleLoaderConfig::new().with_args(vec!["--release".to_string()]);
    assert_eq!(config.compiler_args, vec!["--release"]);
}

#[test]
fn test_module_loader_config_with_output_dir_sets_path() {
    let config = ModuleLoaderConfig::new().with_output_dir("/tmp/output");
    assert_eq!(config.output_dir, Some(PathBuf::from("/tmp/output")));
}

#[test]
fn test_module_loader_config_with_hot_reload_disabled() {
    let config = ModuleLoaderConfig::new().with_hot_reload(false);
    assert!(!config.hot_reload);
}

#[test]
fn test_module_loader_config_chained_builder() {
    let config = ModuleLoaderConfig::new()
        .with_compiler("vaisc-custom")
        .with_args(vec!["-O3".to_string()])
        .with_output_dir("/build")
        .with_hot_reload(false);
    assert_eq!(config.compiler_command, "vaisc-custom");
    assert_eq!(config.compiler_args, vec!["-O3"]);
    assert_eq!(config.output_dir, Some(PathBuf::from("/build")));
    assert!(!config.hot_reload);
}

// ─── DynloadError: error case coverage ───────────────────────────────────────

#[test]
fn test_dynload_error_module_not_found_display() {
    let err = DynloadError::ModuleNotFound(PathBuf::from("/missing/module.vais"));
    let display = err.to_string();
    assert!(display.contains("Module not found"));
    assert!(display.contains("missing"));
}

#[test]
fn test_dynload_error_compilation_failed_display() {
    let err = DynloadError::CompilationError("undefined symbol".to_string());
    let display = err.to_string();
    assert!(display.contains("undefined symbol"));
}

#[test]
fn test_dynload_error_version_incompatible_all_fields() {
    let err = DynloadError::VersionIncompatible {
        plugin: "my-plugin".to_string(),
        required: ">=2.0.0".to_string(),
        actual: "1.5.0".to_string(),
    };
    let display = err.to_string();
    assert!(display.contains("my-plugin"));
    assert!(display.contains("2.0.0"));
    assert!(display.contains("1.5.0"));
}

#[test]
fn test_dynload_error_security_violation_display() {
    let err = DynloadError::SecurityViolation(
        "attempted filesystem write without capability".to_string(),
    );
    let display = err.to_string();
    assert!(display.contains("attempted filesystem write"));
}

#[test]
fn test_dynload_error_wasm_type_mismatch_fields() {
    let err = DynloadError::WasmTypeMismatch {
        expected: "i64".to_string(),
        actual: "f32".to_string(),
    };
    let display = err.to_string();
    assert!(display.contains("i64"));
    assert!(display.contains("f32"));
}

// ─── ResourceLimits: validation edge cases ───────────────────────────────────

#[test]
fn test_resource_limits_validate_memory_max_less_than_initial_fails() {
    let mut limits = ResourceLimits::default();
    limits.memory.max_bytes = 100;
    limits.memory.initial_bytes = 200;
    assert!(limits.validate().is_err());
}

#[test]
fn test_resource_limits_validate_zero_call_depth_fails() {
    let mut limits = ResourceLimits::default();
    limits.stack.max_call_depth = 0;
    assert!(limits.validate().is_err());
}

#[test]
fn test_resource_limits_validate_zero_time_fails() {
    let mut limits = ResourceLimits::default();
    limits.time.max_duration_ms = 0;
    assert!(limits.validate().is_err());
}

#[test]
fn test_resource_limits_restrictive_fields() {
    let limits = ResourceLimits::restrictive();
    assert_eq!(limits.memory.max_bytes, 16 * 1024 * 1024);
    assert_eq!(limits.time.max_duration_ms, 1000);
    assert_eq!(limits.max_tables, 1);
    assert_eq!(limits.max_memories, 1);
    assert_eq!(limits.max_instances, 1);
}

#[test]
fn test_stack_limit_new_values() {
    let stack = StackLimit::new(512 * 1024, 200);
    assert_eq!(stack.max_bytes, 512 * 1024);
    assert_eq!(stack.max_call_depth, 200);
}

#[test]
fn test_time_limit_fuel_set_via_builder() {
    let limit = TimeLimit::seconds(5).with_fuel(999);
    assert_eq!(limit.fuel_limit, Some(999));
    assert_eq!(limit.max_duration_ms, 5000);
}

#[test]
fn test_memory_limit_megabytes_calculation() {
    let limit = MemoryLimit::megabytes(64);
    assert_eq!(limit.max_bytes, 64 * 1024 * 1024);
    // WASM pages: 64MB / 64KB = 1024 pages
    assert_eq!(limit.max_wasm_pages, 1024);
}

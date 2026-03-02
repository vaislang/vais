//! Phase 85: WASM Ecosystem — WASI Preview 2 & Component Model E2E Tests
//!
//! Tests verify:
//! 1. WASI Preview 2 compile path produces wasm32-wasip2 target triple
//! 2. Preview 2 interface bindings parse and generate correct IR
//! 3. Component Model infrastructure (WIT generation, link config)
//! 4. Preview 1 vs Preview 2 target triple differentiation
//! 5. Existing WASM tests regression-free

use super::helpers::compile_to_ir;

// ==============================================================================
// WASI Preview 2 Target Triple Tests
// ==============================================================================

#[test]
fn test_wasip2_target_triple_in_ir() {
    // WasiPreview2 target should produce wasm32-wasip2 triple
    let source = r#"
F main() -> i64 {
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::WasiPreview2,
    );
    let ir = gen.generate_module(&module).unwrap();
    assert!(
        ir.contains("target triple = \"wasm32-wasip2\""),
        "IR should contain wasm32-wasip2 target triple, got:\n{}",
        ir.lines().take(5).collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn test_wasip1_target_triple_in_ir() {
    // WasiPreview1 target should produce wasm32-wasi triple (not wasip2)
    let source = r#"
F main() -> i64 {
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::WasiPreview1,
    );
    let ir = gen.generate_module(&module).unwrap();
    assert!(
        ir.contains("target triple = \"wasm32-wasi\""),
        "IR should contain wasm32-wasi target triple"
    );
    assert!(
        !ir.contains("wasm32-wasip2"),
        "Preview 1 IR should NOT contain wasip2"
    );
}

#[test]
fn test_wasip2_vs_wasip1_different_triple() {
    // Same source code should produce different target triples
    let source = r#"
F add(a: i64, b: i64) -> i64 = a + b

F main() -> i64 {
    R add(1, 2)
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut checker = vais_types::TypeChecker::new();
    checker.check_module(&module).unwrap();

    // Preview 1
    let mut gen1 = vais_codegen::CodeGenerator::new_with_target(
        "test_p1",
        vais_codegen::TargetTriple::WasiPreview1,
    );
    gen1.set_resolved_functions(checker.get_all_functions().clone());
    let ir1 = gen1.generate_module(&module).unwrap();

    // Preview 2
    let module2 = vais_parser::parse(source).unwrap();
    let mut checker2 = vais_types::TypeChecker::new();
    checker2.check_module(&module2).unwrap();
    let mut gen2 = vais_codegen::CodeGenerator::new_with_target(
        "test_p2",
        vais_codegen::TargetTriple::WasiPreview2,
    );
    gen2.set_resolved_functions(checker2.get_all_functions().clone());
    let ir2 = gen2.generate_module(&module2).unwrap();

    assert!(ir1.contains("wasm32-wasi"));
    assert!(ir2.contains("wasm32-wasip2"));
    // Both should have the same data layout (wasm32)
    assert!(ir1.contains("e-m:e-p:32:32-i64:64-n32:64-S128"));
    assert!(ir2.contains("e-m:e-p:32:32-i64:64-n32:64-S128"));
}

#[test]
fn test_wasip2_data_layout() {
    // WASI Preview 2 should use 32-bit pointer data layout
    let source = r#"
F main() -> i64 {
    R 42
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::WasiPreview2,
    );
    let ir = gen.generate_module(&module).unwrap();
    assert!(
        ir.contains("e-m:e-p:32:32-i64:64-n32:64-S128"),
        "WASI P2 should use 32-bit pointer data layout"
    );
}

// ==============================================================================
// Target Triple Parse Tests
// ==============================================================================

#[test]
fn test_parse_wasip2_target_strings() {
    use vais_codegen::TargetTriple;
    assert_eq!(
        TargetTriple::parse("wasi-preview2"),
        Some(TargetTriple::WasiPreview2)
    );
    assert_eq!(
        TargetTriple::parse("wasm32-wasip2"),
        Some(TargetTriple::WasiPreview2)
    );
}

#[test]
fn test_wasip2_triple_str() {
    use vais_codegen::TargetTriple;
    assert_eq!(TargetTriple::WasiPreview2.triple_str(), "wasm32-wasip2");
    assert_eq!(TargetTriple::WasiPreview1.triple_str(), "wasm32-wasi");
}

#[test]
fn test_wasip2_is_wasm() {
    use vais_codegen::TargetTriple;
    assert!(TargetTriple::WasiPreview2.is_wasm());
    assert!(TargetTriple::WasiPreview1.is_wasm());
    assert!(TargetTriple::Wasm32Unknown.is_wasm());
}

#[test]
fn test_wasip2_pointer_bits() {
    use vais_codegen::TargetTriple;
    assert_eq!(TargetTriple::WasiPreview2.pointer_bits(), 32);
}

#[test]
fn test_wasip2_output_extension() {
    use vais_codegen::TargetTriple;
    assert_eq!(TargetTriple::WasiPreview2.output_extension(), "wasm");
}

#[test]
fn test_wasip2_target_os() {
    use vais_codegen::TargetTriple;
    assert_eq!(TargetTriple::WasiPreview2.target_os(), "wasm");
}

#[test]
fn test_wasip2_target_arch() {
    use vais_codegen::TargetTriple;
    assert_eq!(TargetTriple::WasiPreview2.target_arch(), "wasm32");
}

#[test]
fn test_wasip2_cfg_values() {
    use vais_codegen::TargetTriple;
    let cfg = TargetTriple::WasiPreview2.cfg_values();
    assert_eq!(cfg.get("target_os").unwrap(), "wasm");
    assert_eq!(cfg.get("target_arch").unwrap(), "wasm32");
    assert_eq!(cfg.get("target_family").unwrap(), "wasm");
}

// ==============================================================================
// WASI Preview 2 Import Bindings — IR Generation Tests
// ==============================================================================

#[test]
fn test_wasip2_io_stream_import() {
    // WASI Preview 2 stream imports with wasi: namespace
    let source = r#"
N "C" {
    #[wasm_import("wasi:io/streams@0.2.0", "read")]
    F stream_read(stream: i64, buf_ptr: i64, buf_len: i64) -> i64;

    #[wasm_import("wasi:io/streams@0.2.0", "write")]
    F stream_write(stream: i64, buf_ptr: i64, buf_len: i64) -> i64;
}

F main() -> i64 {
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::WasiPreview2,
    );
    let ir = gen.generate_module(&module).unwrap();
    assert!(ir.contains("wasm32-wasip2"));
    assert!(ir.contains("wasi:io/streams@0.2.0"));
    assert!(ir.contains("wasm-import-module"));
}

#[test]
fn test_wasip2_cli_stdio_import() {
    // WASI CLI stdin/stdout/stderr imports
    let source = r#"
N "C" {
    #[wasm_import("wasi:cli/stdout@0.2.0", "get-stdout")]
    F get_stdout() -> i64;

    #[wasm_import("wasi:cli/stderr@0.2.0", "get-stderr")]
    F get_stderr() -> i64;

    #[wasm_import("wasi:cli/stdin@0.2.0", "get-stdin")]
    F get_stdin() -> i64;
}

F main() -> i64 {
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::WasiPreview2,
    );
    let ir = gen.generate_module(&module).unwrap();
    assert!(ir.contains("wasi:cli/stdout@0.2.0"));
    assert!(ir.contains("wasi:cli/stderr@0.2.0"));
    assert!(ir.contains("wasi:cli/stdin@0.2.0"));
}

#[test]
fn test_wasip2_filesystem_import() {
    // WASI filesystem types import
    let source = r#"
N "C" {
    #[wasm_import("wasi:filesystem/types@0.2.0", "open-at")]
    F fs_open_at(dir_fd: i64, path_flags: i64, path_ptr: i64, path_len: i64, open_flags: i64, desc_flags: i64) -> i64;

    #[wasm_import("wasi:filesystem/types@0.2.0", "stat")]
    F fs_stat(fd: i64, stat_ptr: i64) -> i64;
}

F main() -> i64 {
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::WasiPreview2,
    );
    let ir = gen.generate_module(&module).unwrap();
    assert!(ir.contains("wasi:filesystem/types@0.2.0"));
}

#[test]
fn test_wasip2_clock_import() {
    // WASI monotonic clock import
    let source = r#"
N "C" {
    #[wasm_import("wasi:clocks/monotonic-clock@0.2.0", "now")]
    F clock_now() -> i64;

    #[wasm_import("wasi:clocks/monotonic-clock@0.2.0", "resolution")]
    F clock_resolution() -> i64;
}

F main() -> i64 {
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::WasiPreview2,
    );
    let ir = gen.generate_module(&module).unwrap();
    assert!(ir.contains("wasi:clocks/monotonic-clock@0.2.0"));
}

#[test]
fn test_wasip2_random_import() {
    // WASI random interface import
    let source = r#"
N "C" {
    #[wasm_import("wasi:random/random@0.2.0", "get-random-u64")]
    F random_u64() -> i64;

    #[wasm_import("wasi:random/random@0.2.0", "get-random-bytes")]
    F random_bytes(buf_ptr: i64, buf_len: i64) -> i64;
}

F main() -> i64 {
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::WasiPreview2,
    );
    let ir = gen.generate_module(&module).unwrap();
    assert!(ir.contains("wasi:random/random@0.2.0"));
}

#[test]
fn test_wasip2_http_import() {
    // WASI HTTP outgoing handler import
    let source = r#"
N "C" {
    #[wasm_import("wasi:http/types@0.2.0", "new-outgoing-request")]
    F http_new_request(method: i64, path_ptr: i64, path_len: i64) -> i64;

    #[wasm_import("wasi:http/outgoing-handler@0.2.0", "handle")]
    F http_handle(request: i64, options: i64) -> i64;
}

F main() -> i64 {
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::WasiPreview2,
    );
    let ir = gen.generate_module(&module).unwrap();
    assert!(ir.contains("wasi:http/types@0.2.0"));
    assert!(ir.contains("wasi:http/outgoing-handler@0.2.0"));
}

// ==============================================================================
// Component Model Infrastructure Tests
// ==============================================================================

#[test]
fn test_component_link_config_default() {
    use vais_codegen::wasm_component::ComponentLinkConfig;
    let config = ComponentLinkConfig::new();
    assert!(config.command_mode);
    assert!(!config.reactor_mode);
    assert!(config.adapter_module.is_none());
    assert!(config.component_imports.is_empty());
    assert!(config.component_exports.is_empty());
}

#[test]
fn test_component_link_config_reactor() {
    use vais_codegen::wasm_component::ComponentLinkConfig;
    let config = ComponentLinkConfig::new().reactor();
    assert!(config.reactor_mode);
    assert!(!config.command_mode);
}

#[test]
fn test_component_link_config_with_adapter() {
    use vais_codegen::wasm_component::ComponentLinkConfig;
    let config = ComponentLinkConfig::new().with_adapter("wasi_snapshot_preview1.reactor.wasm");
    assert_eq!(
        config.adapter_module.as_deref(),
        Some("wasi_snapshot_preview1.reactor.wasm")
    );
}

#[test]
fn test_component_link_config_to_link_args() {
    use vais_codegen::wasm_component::ComponentLinkConfig;
    let mut config = ComponentLinkConfig::new().with_adapter("adapter.wasm");
    config.add_import("fs".to_string(), "wasi:filesystem/types".to_string());
    config.add_export("run".to_string(), "wasi:cli/run".to_string());

    let args = config.to_link_args();
    assert!(args.contains(&"--adapt".to_string()));
    assert!(args.contains(&"adapter.wasm".to_string()));
    assert!(args.contains(&"--import".to_string()));
    assert!(args.contains(&"--export".to_string()));
}

#[test]
fn test_wasi_manifest_imports() {
    use vais_codegen::wasm_component::WasiManifest;
    let mut manifest = WasiManifest::new();
    manifest.add_import("wasi:filesystem/types@0.2.0");
    manifest.add_import("wasi:cli/stdio@0.2.0");
    manifest.add_import("wasi:io/streams@0.2.0");

    assert_eq!(manifest.imports.len(), 3);
    assert!(manifest
        .imports
        .contains(&"wasi:filesystem/types@0.2.0".to_string()));
    assert!(manifest
        .imports
        .contains(&"wasi:cli/stdio@0.2.0".to_string()));
    assert!(manifest
        .imports
        .contains(&"wasi:io/streams@0.2.0".to_string()));
}

#[test]
fn test_wasi_manifest_no_duplicate_imports() {
    use vais_codegen::wasm_component::WasiManifest;
    let mut manifest = WasiManifest::new();
    manifest.add_import("wasi:io/streams@0.2.0");
    manifest.add_import("wasi:io/streams@0.2.0"); // duplicate
    assert_eq!(manifest.imports.len(), 1);
}

#[test]
fn test_wasi_manifest_to_wit_string() {
    use vais_codegen::wasm_component::WasiManifest;
    let mut manifest = WasiManifest::new();
    manifest.add_import("wasi:io/streams@0.2.0");
    manifest.add_import("wasi:filesystem/types@0.2.0");

    let wit = manifest.to_wit_string();
    assert!(wit.contains("import wasi:io/streams@0.2.0;"));
    assert!(wit.contains("import wasi:filesystem/types@0.2.0;"));
}

#[test]
fn test_component_link_config_with_wasi_manifest() {
    use vais_codegen::wasm_component::{ComponentLinkConfig, WasiManifest};
    let mut manifest = WasiManifest::new();
    manifest.add_import("wasi:cli/stdout@0.2.0");

    let config = ComponentLinkConfig::new()
        .with_wasi_manifest(manifest)
        .with_adapter("adapter.wasm");

    assert!(config.wasi_manifest.is_some());
    let m = config.wasi_manifest.unwrap();
    assert_eq!(m.imports.len(), 1);
}

#[test]
fn test_component_link_config_wasi_manifest_mut() {
    use vais_codegen::wasm_component::ComponentLinkConfig;
    let mut config = ComponentLinkConfig::new();
    // wasi_manifest_mut should auto-create manifest if not present
    let manifest = config.wasi_manifest_mut();
    manifest.add_import("wasi:io/poll@0.2.0");
    assert_eq!(config.wasi_manifest.as_ref().unwrap().imports.len(), 1);
}

// ==============================================================================
// WIT Type & Function Tests
// ==============================================================================

#[test]
fn test_wit_type_display() {
    use vais_codegen::wasm_component::WitType;
    assert_eq!(format!("{}", WitType::S32), "s32");
    assert_eq!(format!("{}", WitType::S64), "s64");
    assert_eq!(format!("{}", WitType::U32), "u32");
    assert_eq!(format!("{}", WitType::Bool), "bool");
    assert_eq!(format!("{}", WitType::String), "string");
    assert_eq!(format!("{}", WitType::F32), "f32");
    assert_eq!(format!("{}", WitType::F64), "f64");
}

#[test]
fn test_wit_function_construction() {
    use vais_codegen::wasm_component::{WitFunction, WitParam, WitResult, WitType};
    let func = WitFunction {
        name: "add".to_string(),
        params: vec![
            WitParam {
                name: "a".to_string(),
                ty: WitType::S64,
            },
            WitParam {
                name: "b".to_string(),
                ty: WitType::S64,
            },
        ],
        results: Some(WitResult::Anon(WitType::S64)),
        docs: Some("Add two numbers".to_string()),
    };
    assert_eq!(func.name, "add");
    assert_eq!(func.params.len(), 2);
}

// ==============================================================================
// WASM Export with Preview 2 Target Tests
// ==============================================================================

#[test]
fn test_wasip2_export_function() {
    // wasm_export should work with wasip2 target
    let source = r#"
#[wasm_export("compute")]
F compute(x: i64) -> i64 = x * x + 1

F main() -> i64 {
    R compute(5)
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut checker = vais_types::TypeChecker::new();
    checker.check_module(&module).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::WasiPreview2,
    );
    gen.set_resolved_functions(checker.get_all_functions().clone());
    let ir = gen.generate_module(&module).unwrap();
    assert!(ir.contains("wasm32-wasip2"));
    assert!(ir.contains("wasm-export-name"));
    assert!(ir.contains("compute"));
}

#[test]
fn test_wasip2_mixed_import_export() {
    // Mixed imports and exports on wasip2 target
    let source = r#"
N "C" {
    #[wasm_import("wasi:cli/stdout@0.2.0", "get-stdout")]
    F get_stdout() -> i64;

    #[wasm_import("wasi:io/streams@0.2.0", "write")]
    F stream_write(stream: i64, buf_ptr: i64, buf_len: i64) -> i64;
}

#[wasm_export("greet")]
F greet() -> i64 = 42

F main() -> i64 {
    R greet()
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut checker = vais_types::TypeChecker::new();
    checker.check_module(&module).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::WasiPreview2,
    );
    gen.set_resolved_functions(checker.get_all_functions().clone());
    let ir = gen.generate_module(&module).unwrap();
    // Should have both import and export metadata
    assert!(ir.contains("wasm-import-module"));
    assert!(ir.contains("wasm-export-name"));
    assert!(ir.contains("wasi:cli/stdout@0.2.0"));
    assert!(ir.contains("wasi:io/streams@0.2.0"));
}

// ==============================================================================
// Regression: Existing WASM Tests Still Pass
// ==============================================================================

#[test]
fn test_wasm32_unknown_still_works() {
    // Ensure wasm32-unknown-unknown target still generates correct IR
    let source = r#"
N "C" {
    #[wasm_import("env", "log")]
    F host_log(ptr: *i8, len: i64);
}

F main() -> i64 {
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::Wasm32Unknown,
    );
    let ir = gen.generate_module(&module).unwrap();
    assert!(ir.contains("wasm32-unknown-unknown"));
    assert!(!ir.contains("wasip2"));
    assert!(ir.contains("wasm-import-module"));
}

#[test]
fn test_native_target_no_wasm_metadata() {
    // Native target should not produce any WASM metadata
    let source = r#"
F main() -> i64 {
    R 42
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(!ir.contains("wasm32"));
    assert!(!ir.contains("wasm-import-module"));
    assert!(!ir.contains("wasm-export-name"));
}

#[test]
fn test_wasip2_import_not_on_native() {
    // WASI P2 imports should NOT produce wasm metadata on native target
    let source = r#"
N "C" {
    #[wasm_import("wasi:io/streams@0.2.0", "read")]
    F stream_read(stream: i64, buf_ptr: i64, buf_len: i64) -> i64;
}

F main() -> i64 {
    R 0
}
"#;
    let ir = compile_to_ir(source).unwrap();
    assert!(!ir.contains("wasm-import-module"));
}

// ==============================================================================
// WasmBindgenGenerator Tests (Component Model JS bindings)
// ==============================================================================

#[test]
fn test_bindgen_js_for_wasip2_exports() {
    use vais_codegen::wasm_component::{
        WasmBindgenGenerator, WitFunction, WitParam, WitResult, WitType,
    };

    let gen = WasmBindgenGenerator::new("wasip2_module");
    let functions = vec![WitFunction {
        name: "process".to_string(),
        params: vec![WitParam {
            name: "input".to_string(),
            ty: WitType::S64,
        }],
        results: Some(WitResult::Anon(WitType::S64)),
        docs: Some("Process input".to_string()),
    }];
    let js = gen.generate_js_bindings(&functions);
    assert!(!js.is_empty());
    assert!(js.contains("process"));
}

#[test]
fn test_bindgen_ts_declarations() {
    use vais_codegen::wasm_component::{
        WasmBindgenGenerator, WitFunction, WitParam, WitResult, WitType,
    };

    let gen = WasmBindgenGenerator::new("wasip2_module");
    let functions = vec![WitFunction {
        name: "add".to_string(),
        params: vec![
            WitParam {
                name: "a".to_string(),
                ty: WitType::S64,
            },
            WitParam {
                name: "b".to_string(),
                ty: WitType::S64,
            },
        ],
        results: Some(WitResult::Anon(WitType::S64)),
        docs: None,
    }];
    let ts = gen.generate_ts_declarations(&functions);
    assert!(!ts.is_empty());
    assert!(ts.contains("add"));
}

// ==============================================================================
// WIT Package & Interface Tests
// ==============================================================================

#[test]
fn test_wit_package_construction() {
    use vais_codegen::wasm_component::WitPackage;
    let pkg = WitPackage::new("my-namespace", "my-package").with_version("0.1.0");
    assert_eq!(pkg.namespace, "my-namespace");
    assert_eq!(pkg.name, "my-package");
    assert_eq!(pkg.version.as_deref(), Some("0.1.0"));
}

#[test]
fn test_wit_package_to_wit_string() {
    use vais_codegen::wasm_component::WitPackage;
    let pkg = WitPackage::new("vais", "math").with_version("1.0.0");
    let wit = pkg.to_wit_string();
    assert!(wit.contains("package vais:math@1.0.0;"));
}

#[test]
fn test_wit_interface_construction() {
    use vais_codegen::wasm_component::{WitFunction, WitInterface, WitParam, WitResult, WitType};
    let iface = WitInterface {
        name: "math".to_string(),
        types: vec![],
        functions: vec![WitFunction {
            name: "add".to_string(),
            params: vec![
                WitParam {
                    name: "a".to_string(),
                    ty: WitType::S32,
                },
                WitParam {
                    name: "b".to_string(),
                    ty: WitType::S32,
                },
            ],
            results: Some(WitResult::Anon(WitType::S32)),
            docs: None,
        }],
        resources: vec![],
        docs: None,
    };
    assert_eq!(iface.name, "math");
    assert_eq!(iface.functions.len(), 1);
}

// ==============================================================================
// Vais Type to WIT Conversion Tests
// ==============================================================================

#[test]
fn test_vais_type_to_wit_i64() {
    use vais_codegen::wasm_component::{vais_type_to_wit, WitType};
    use vais_types::ResolvedType;
    let wit = vais_type_to_wit(&ResolvedType::I64);
    assert_eq!(wit, Some(WitType::S64));
}

#[test]
fn test_vais_type_to_wit_i32() {
    use vais_codegen::wasm_component::{vais_type_to_wit, WitType};
    use vais_types::ResolvedType;
    let wit = vais_type_to_wit(&ResolvedType::I32);
    assert_eq!(wit, Some(WitType::S32));
}

#[test]
fn test_vais_type_to_wit_bool() {
    use vais_codegen::wasm_component::{vais_type_to_wit, WitType};
    use vais_types::ResolvedType;
    let wit = vais_type_to_wit(&ResolvedType::Bool);
    assert_eq!(wit, Some(WitType::Bool));
}

#[test]
fn test_vais_type_to_wit_str() {
    use vais_codegen::wasm_component::{vais_type_to_wit, WitType};
    use vais_types::ResolvedType;
    let wit = vais_type_to_wit(&ResolvedType::Str);
    assert_eq!(wit, Some(WitType::String));
}

#[test]
fn test_vais_type_to_wit_f64() {
    use vais_codegen::wasm_component::{vais_type_to_wit, WitType};
    use vais_types::ResolvedType;
    let wit = vais_type_to_wit(&ResolvedType::F64);
    assert_eq!(wit, Some(WitType::F64));
}

#[test]
fn test_vais_type_to_wit_f32() {
    use vais_codegen::wasm_component::{vais_type_to_wit, WitType};
    use vais_types::ResolvedType;
    let wit = vais_type_to_wit(&ResolvedType::F32);
    assert_eq!(wit, Some(WitType::F32));
}

#[test]
fn test_vais_type_to_wit_u8() {
    use vais_codegen::wasm_component::{vais_type_to_wit, WitType};
    use vais_types::ResolvedType;
    let wit = vais_type_to_wit(&ResolvedType::U8);
    assert_eq!(wit, Some(WitType::U8));
}

#[test]
fn test_vais_type_to_wit_optional() {
    use vais_codegen::wasm_component::{vais_type_to_wit, WitType};
    use vais_types::ResolvedType;
    let wit = vais_type_to_wit(&ResolvedType::Optional(Box::new(ResolvedType::I32)));
    assert_eq!(wit, Some(WitType::Option_(Box::new(WitType::S32))));
}

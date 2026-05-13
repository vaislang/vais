//! Phase 115: WASM Component Model 실전 검증
//!
//! Tests verify:
//! 1. wasm32-unknown-unknown basic E2E IR generation
//! 2. WASI P2 build path verification (compile_to_wasi_p2 path logic)
//! 3. wasm_component/ WIT generation verification
//! 4. examples/wasm_*.vais compilation verification E2E
//! 5. WASM bindgen (JS/TS) generation coverage
//! 6. Cross-target regression tests

use super::helpers::compile_to_ir;

// Helper: compile Vais source to IR with a specific WASM target
fn compile_to_wasm_ir(source: &str, target: vais_codegen::TargetTriple) -> Result<String, String> {
    let _tokens = vais_lexer::tokenize(source).map_err(|e| format!("Lexer error: {:?}", e))?;
    let module = vais_parser::parse(source).map_err(|e| format!("Parser error: {:?}", e))?;
    let mut checker = vais_types::TypeChecker::new();
    checker
        .check_module(&module)
        .map_err(|e| format!("Type error: {:?}", e))?;
    let mut gen = vais_codegen::CodeGenerator::new_with_target("wasm_test", target);
    gen.set_resolved_functions(checker.get_all_functions().clone());
    gen.set_type_aliases(checker.get_type_aliases().clone());
    let instantiations = checker.get_generic_instantiations();
    let ir = if instantiations.is_empty() {
        gen.generate_module(&module)
    } else {
        gen.generate_module_with_instantiations(&module, &instantiations)
    }
    .map_err(|e| format!("Codegen error: {:?}", e))?;
    Ok(ir)
}

// ==============================================================================
// 1. wasm32-unknown-unknown Basic E2E IR Generation Tests
// ==============================================================================

#[test]
fn test_wasm32_basic_function_ir() {
    let source = r#"
F add(a: i64, b: i64) -> i64 = a + b

F main() -> i64 {
    R add(3, 4)
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::Wasm32Unknown).unwrap();
    assert!(
        ir.contains("wasm32-unknown-unknown"),
        "Should target wasm32-unknown-unknown"
    );
    assert!(ir.contains("define"), "Should have function definitions");
    assert!(
        ir.contains("e-m:e-p:32:32"),
        "Should have 32-bit pointer data layout"
    );
}

#[test]
fn test_wasm32_with_globals() {
    // Global variables should compile to WASM IR
    let source = r#"
G counter: i64 = 0
G max_val: i64 = 100

F main() -> i64 {
    R 0
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::Wasm32Unknown).unwrap();
    assert!(ir.contains("wasm32-unknown-unknown"));
    // Verify IR is valid (has main function)
    assert!(ir.contains("define i64 @main"));
}

#[test]
fn test_wasm32_with_conditionals() {
    let source = r#"
F abs_val(n: i64) -> i64 {
    I n < 0 {
        R -n
    }
    n
}

F main() -> i64 {
    R abs_val(-5)
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::Wasm32Unknown).unwrap();
    assert!(ir.contains("wasm32-unknown-unknown"));
    assert!(ir.contains("icmp"), "Should have comparison instruction");
    assert!(ir.contains("br "), "Should have branch instruction");
}

#[test]
fn test_wasm32_with_loop() {
    let source = r#"
F sum_to(n: i64) -> i64 {
    result := mut 0
    i := mut 0
    L i < n {
        result = result + i
        i = i + 1
    }
    result
}

F main() -> i64 {
    R sum_to(10)
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::Wasm32Unknown).unwrap();
    assert!(ir.contains("wasm32-unknown-unknown"));
}

#[test]
fn test_wasm32_with_recursion() {
    let source = r#"
F fib(n: i64) -> i64 {
    I n <= 1 { n }
    E { @(n - 1) + @(n - 2) }
}

F main() -> i64 {
    R fib(10)
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::Wasm32Unknown).unwrap();
    assert!(ir.contains("wasm32-unknown-unknown"));
    assert!(ir.contains("call"), "Should have recursive call");
}

#[test]
fn test_wasm32_export_attribute() {
    let source = r#"
#[wasm_export("compute")]
F compute(x: i64, y: i64) -> i64 = x * y + 1

F main() -> i64 {
    R compute(3, 4)
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::Wasm32Unknown).unwrap();
    assert!(ir.contains("wasm32-unknown-unknown"));
    assert!(
        ir.contains("wasm-export-name"),
        "Should have WASM export metadata"
    );
    assert!(ir.contains("compute"), "Should export 'compute'");
}

#[test]
fn test_wasm32_import_attribute() {
    let source = r#"
N "C" {
    #[wasm_import("env", "js_log")]
    F js_log(ptr: i64, len: i64) -> i64;
}

F main() -> i64 {
    R 0
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::Wasm32Unknown).unwrap();
    assert!(ir.contains("wasm32-unknown-unknown"));
    assert!(
        ir.contains("wasm-import-module"),
        "Should have WASM import module"
    );
    assert!(
        ir.contains("wasm-import-name"),
        "Should have WASM import name"
    );
    assert!(ir.contains("env"), "Import module should be 'env'");
}

#[test]
fn test_wasm32_multiple_exports() {
    let source = r#"
#[wasm_export("add")]
F add(a: i64, b: i64) -> i64 = a + b

#[wasm_export("mul")]
F mul(a: i64, b: i64) -> i64 = a * b

#[wasm_export("sub")]
F sub(a: i64, b: i64) -> i64 = a - b

F main() -> i64 {
    R add(1, 2) + mul(3, 4) + sub(10, 5)
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::Wasm32Unknown).unwrap();
    assert!(ir.contains("wasm32-unknown-unknown"));
    // All three exports should be present
    let export_count = ir.matches("wasm-export-name").count();
    assert!(
        export_count >= 3,
        "Should have at least 3 wasm-export-name attributes, got {}",
        export_count
    );
}

#[test]
fn test_wasm32_import_and_export_combined() {
    let source = r#"
N "C" {
    #[wasm_import("env", "get_input")]
    F get_input() -> i64;
}

#[wasm_export("process")]
F process() -> i64 {
    x := get_input()
    x * 2
}

F main() -> i64 {
    R process()
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::Wasm32Unknown).unwrap();
    assert!(ir.contains("wasm-import-module"));
    assert!(ir.contains("wasm-export-name"));
}

// ==============================================================================
// 2. WASI P2 Build Path Verification
// ==============================================================================

#[test]
fn test_wasip2_basic_ir_generation() {
    let source = r#"
F main() -> i64 {
    R 42
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::WasiPreview2).unwrap();
    assert!(ir.contains("wasm32-wasip2"), "Should target wasm32-wasip2");
    assert!(
        ir.contains("e-m:e-p:32:32"),
        "Should have 32-bit pointer data layout"
    );
}

#[test]
fn test_wasip2_function_with_multiple_params() {
    let source = r#"
F compute(a: i64, b: i64, c: i64) -> i64 = a * b + c

F main() -> i64 {
    R compute(2, 3, 4)
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::WasiPreview2).unwrap();
    assert!(ir.contains("wasm32-wasip2"));
    assert!(ir.contains("@compute"));
}

#[test]
fn test_wasip2_with_struct() {
    let source = r#"
S Point {
    x: i64,
    y: i64
}

F distance_sq(p: Point) -> i64 = p.x * p.x + p.y * p.y

F main() -> i64 {
    p := Point { x: 3, y: 4 }
    R distance_sq(p)
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::WasiPreview2).unwrap();
    assert!(ir.contains("wasm32-wasip2"));
}

#[test]
fn test_wasip2_with_enum() {
    let source = r#"
E Color {
    Red,
    Green,
    Blue
}

F color_value(c: Color) -> i64 {
    M c {
        Red => 1,
        Green => 2,
        Blue => 3,
        _ => 0
    }
}

F main() -> i64 {
    c := Green
    R color_value(c)
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::WasiPreview2).unwrap();
    assert!(ir.contains("wasm32-wasip2"));
}

#[test]
fn test_wasip2_wasi_io_full_pipeline() {
    // Full WASI P2 I/O pipeline with multiple interfaces
    let source = r#"
N "C" {
    #[wasm_import("wasi:io/streams@0.2.0", "read")]
    F stream_read(stream: i64, buf_ptr: i64, buf_len: i64) -> i64;

    #[wasm_import("wasi:io/streams@0.2.0", "write")]
    F stream_write(stream: i64, buf_ptr: i64, buf_len: i64) -> i64;

    #[wasm_import("wasi:io/poll@0.2.0", "poll-one")]
    F poll_one(pollable: i64) -> i64;

    #[wasm_import("wasi:cli/stdout@0.2.0", "get-stdout")]
    F get_stdout() -> i64;

    #[wasm_import("wasi:cli/stdin@0.2.0", "get-stdin")]
    F get_stdin() -> i64;
}

F main() -> i64 {
    R 0
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::WasiPreview2).unwrap();
    assert!(ir.contains("wasm32-wasip2"));
    assert!(ir.contains("wasi:io/streams@0.2.0"));
    assert!(ir.contains("wasi:io/poll@0.2.0"));
    assert!(ir.contains("wasi:cli/stdout@0.2.0"));
    assert!(ir.contains("wasi:cli/stdin@0.2.0"));
    // All should have wasm-import-module
    let import_count = ir.matches("wasm-import-module").count();
    assert!(
        import_count >= 4,
        "Should have at least 4 wasm-import-module attributes, got {}",
        import_count
    );
}

#[test]
fn test_wasip2_filesystem_full_pipeline() {
    let source = r#"
N "C" {
    #[wasm_import("wasi:filesystem/types@0.2.0", "open-at")]
    F fs_open_at(fd: i64, flags: i64, path_ptr: i64, path_len: i64) -> i64;

    #[wasm_import("wasi:filesystem/types@0.2.0", "read-via-stream")]
    F fs_read_stream(fd: i64) -> i64;

    #[wasm_import("wasi:filesystem/types@0.2.0", "write-via-stream")]
    F fs_write_stream(fd: i64) -> i64;

    #[wasm_import("wasi:filesystem/types@0.2.0", "stat")]
    F fs_stat(fd: i64) -> i64;

    #[wasm_import("wasi:filesystem/preopens@0.2.0", "get-directories")]
    F fs_get_dirs() -> i64;
}

F main() -> i64 {
    R 0
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::WasiPreview2).unwrap();
    assert!(ir.contains("wasi:filesystem/types@0.2.0"));
    assert!(ir.contains("wasi:filesystem/preopens@0.2.0"));
}

#[test]
fn test_wasip2_sockets_network_import() {
    let source = r#"
N "C" {
    #[wasm_import("wasi:sockets/tcp@0.2.0", "create-tcp-socket")]
    F tcp_create(af: i64) -> i64;

    #[wasm_import("wasi:sockets/tcp@0.2.0", "connect")]
    F tcp_connect(socket: i64, addr_ptr: i64, addr_len: i64) -> i64;

    #[wasm_import("wasi:sockets/ip-name-lookup@0.2.0", "resolve-addresses")]
    F dns_resolve(name_ptr: i64, name_len: i64) -> i64;
}

F main() -> i64 {
    R 0
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::WasiPreview2).unwrap();
    assert!(ir.contains("wasi:sockets/tcp@0.2.0"));
    assert!(ir.contains("wasi:sockets/ip-name-lookup@0.2.0"));
}

#[test]
fn test_wasip2_export_with_wasi_imports() {
    // Realistic: export functions that use WASI imports internally
    let source = r#"
N "C" {
    #[wasm_import("wasi:cli/stdout@0.2.0", "get-stdout")]
    F get_stdout() -> i64;

    #[wasm_import("wasi:io/streams@0.2.0", "write")]
    F stream_write(stream: i64, buf_ptr: i64, buf_len: i64) -> i64;
}

#[wasm_export("hello")]
F hello() -> i64 {
    out := get_stdout()
    msg := "Hello from WASI P2"
    _ := stream_write(out, msg as i64, 19)
    0
}

#[wasm_export("add")]
F add(a: i64, b: i64) -> i64 = a + b

F main() -> i64 {
    R hello()
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::WasiPreview2).unwrap();
    assert!(ir.contains("wasm32-wasip2"));
    assert!(ir.contains("wasm-import-module"));
    assert!(ir.contains("wasm-export-name"));
    // Verify both imports and exports coexist
    assert!(ir.contains("wasi:cli/stdout@0.2.0"));
    assert!(ir.contains("wasi:io/streams@0.2.0"));
}

// ==============================================================================
// 3. wasm_component/ WIT Generation Verification Tests
// ==============================================================================

#[test]
fn test_wit_package_with_multiple_interfaces() {
    use vais_codegen::wasm_component::{
        WitFunction, WitInterface, WitPackage, WitParam, WitResult, WitType,
    };

    let mut pkg = WitPackage::new("vais", "calculator").with_version("1.0.0");

    // Add math interface
    pkg.add_interface(WitInterface {
        name: "math".to_string(),
        types: vec![],
        functions: vec![
            WitFunction {
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
            },
            WitFunction {
                name: "multiply".to_string(),
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
            },
        ],
        resources: vec![],
        docs: Some("Basic math operations".to_string()),
    });

    // Add utility interface
    pkg.add_interface(WitInterface {
        name: "utils".to_string(),
        types: vec![],
        functions: vec![WitFunction {
            name: "abs".to_string(),
            params: vec![WitParam {
                name: "n".to_string(),
                ty: WitType::S64,
            }],
            results: Some(WitResult::Anon(WitType::S64)),
            docs: None,
        }],
        resources: vec![],
        docs: None,
    });

    let wit = pkg.to_wit_string();
    assert!(wit.contains("package vais:calculator@1.0.0;"));
    assert!(wit.contains("interface math"));
    assert!(wit.contains("interface utils"));
    assert!(wit.contains("add: func"));
    assert!(wit.contains("multiply: func"));
    assert!(wit.contains("abs: func"));
}

#[test]
fn test_wit_package_no_version() {
    use vais_codegen::wasm_component::WitPackage;

    let pkg = WitPackage::new("test", "simple");
    let wit = pkg.to_wit_string();
    assert!(wit.contains("package test:simple;"));
    assert!(
        !wit.contains("@"),
        "No version should mean no @ in package line"
    );
}

#[test]
fn test_wit_record_construction() {
    use vais_codegen::wasm_component::{WitField, WitRecord, WitType};

    let record = WitRecord {
        name: "config".to_string(),
        fields: vec![
            WitField {
                name: "name".to_string(),
                ty: WitType::String,
                docs: None,
            },
            WitField {
                name: "max-retries".to_string(),
                ty: WitType::U32,
                docs: Some("Maximum number of retries".to_string()),
            },
            WitField {
                name: "timeout-ms".to_string(),
                ty: WitType::U64,
                docs: None,
            },
            WitField {
                name: "enabled".to_string(),
                ty: WitType::Bool,
                docs: None,
            },
        ],
        docs: Some("Configuration record".to_string()),
    };

    assert_eq!(record.name, "config");
    assert_eq!(record.fields.len(), 4);
    assert_eq!(record.fields[0].name, "name");
    assert_eq!(format!("{}", record.fields[0].ty), "string");
    assert_eq!(record.fields[1].name, "max-retries");
    assert_eq!(format!("{}", record.fields[1].ty), "u32");
    assert_eq!(record.fields[2].name, "timeout-ms");
    assert_eq!(format!("{}", record.fields[2].ty), "u64");
    assert_eq!(record.fields[3].name, "enabled");
    assert_eq!(format!("{}", record.fields[3].ty), "bool");
}

#[test]
fn test_wit_enum_construction() {
    use vais_codegen::wasm_component::WitEnumCase;

    let cases = vec![
        WitEnumCase {
            name: "red".to_string(),
            docs: None,
        },
        WitEnumCase {
            name: "green".to_string(),
            docs: None,
        },
        WitEnumCase {
            name: "blue".to_string(),
            docs: Some("Primary blue".to_string()),
        },
    ];

    assert_eq!(cases.len(), 3);
    assert_eq!(cases[0].name, "red");
    assert_eq!(cases[1].name, "green");
    assert_eq!(cases[2].name, "blue");
    assert!(cases[2].docs.is_some());
}

#[test]
fn test_wit_flags_construction() {
    use vais_codegen::wasm_component::WitFlags;

    let flags = WitFlags {
        name: "permissions".to_string(),
        flags: vec![
            "read".to_string(),
            "write".to_string(),
            "execute".to_string(),
        ],
        docs: Some("File permissions".to_string()),
    };

    assert_eq!(flags.name, "permissions");
    assert_eq!(flags.flags.len(), 3);
    assert!(flags.flags.contains(&"read".to_string()));
    assert!(flags.flags.contains(&"write".to_string()));
    assert!(flags.flags.contains(&"execute".to_string()));
    assert!(flags.docs.is_some());
}

#[test]
fn test_wit_world_construction() {
    use vais_codegen::wasm_component::{
        WitExport, WitExportItem, WitFunction, WitImport, WitImportItem, WitParam, WitResult,
        WitType, WitWorld,
    };

    let world = WitWorld {
        name: "my-app".to_string(),
        imports: vec![
            WitImport {
                name: "stdio".to_string(),
                item: WitImportItem::Interface("wasi:cli/stdio@0.2.0".to_string()),
            },
            WitImport {
                name: "fs".to_string(),
                item: WitImportItem::Interface("wasi:filesystem/types@0.2.0".to_string()),
            },
        ],
        exports: vec![WitExport {
            name: "run".to_string(),
            item: WitExportItem::Function(WitFunction {
                name: "run".to_string(),
                params: vec![WitParam {
                    name: "args".to_string(),
                    ty: WitType::List(Box::new(WitType::String)),
                }],
                results: Some(WitResult::Anon(WitType::S32)),
                docs: Some("Entry point".to_string()),
            }),
        }],
        docs: Some("My application world".to_string()),
    };

    assert_eq!(world.name, "my-app");
    assert_eq!(world.imports.len(), 2);
    assert_eq!(world.exports.len(), 1);
    assert_eq!(world.imports[0].name, "stdio");
    assert_eq!(world.imports[1].name, "fs");
    assert_eq!(world.exports[0].name, "run");
    assert!(world.docs.is_some());
}

#[test]
fn test_wit_complex_type_display() {
    use vais_codegen::wasm_component::WitType;

    // Nested generics
    let ty = WitType::List(Box::new(WitType::Option_(Box::new(WitType::S32))));
    assert_eq!(format!("{}", ty), "list<option<s32>>");

    // Result with both ok and err
    let ty = WitType::Result_ {
        ok: Some(Box::new(WitType::List(Box::new(WitType::U8)))),
        err: Some(Box::new(WitType::String)),
    };
    assert_eq!(format!("{}", ty), "result<list<u8>, string>");

    // Tuple type
    let ty = WitType::Tuple(vec![WitType::S32, WitType::String, WitType::Bool]);
    assert_eq!(format!("{}", ty), "tuple<s32, string, bool>");
}

#[test]
fn test_wasi_manifest_exports() {
    use vais_codegen::wasm_component::{WasiManifest, WitType};

    let mut manifest = WasiManifest::new();
    manifest.add_import("wasi:cli/stdout@0.2.0");
    manifest.add_import("wasi:io/streams@0.2.0");
    manifest.add_export("main", &WitType::S32);
    manifest.add_export("process", &WitType::S64);

    assert_eq!(manifest.imports.len(), 2);
    assert_eq!(manifest.exports.len(), 2);

    let wit = manifest.to_wit_string();
    assert!(wit.contains("import wasi:cli/stdout@0.2.0;"));
    assert!(wit.contains("import wasi:io/streams@0.2.0;"));
    assert!(wit.contains("export main: s32;"));
    assert!(wit.contains("export process: s64;"));
}

#[test]
fn test_wasi_manifest_empty() {
    use vais_codegen::wasm_component::WasiManifest;

    let manifest = WasiManifest::new();
    assert!(manifest.imports.is_empty());
    assert!(manifest.exports.is_empty());

    let wit = manifest.to_wit_string();
    // Empty manifest should still produce valid (possibly empty) WIT
    assert!(!wit.contains("import"));
    assert!(!wit.contains("export"));
}

// ==============================================================================
// 4. examples/wasm_*.vais — Equivalent Inline Compilation E2E
//    (The example .vais files use `N F` shorthand syntax which requires the
//     full CLI parser. Here we test equivalent programs using `N "C" { ... }`
//     block syntax that the library parser supports.)
// ==============================================================================

#[test]
fn test_wasm_interop_pattern_wasm32() {
    // Equivalent to examples/wasm_interop.vais using N "C" block syntax
    let source = r#"
N "C" {
    #[wasm_import("env", "console_log")]
    F console_log(ptr: i64, len: i64) -> i64;

    #[wasm_import("env", "get_time")]
    F get_time() -> i64;
}

#[wasm_export("add")]
F add(a: i64, b: i64) -> i64 = a + b

#[wasm_export("fibonacci")]
F fib(n: i64) -> i64 {
    I n <= 1 { n }
    E { @(n - 1) + @(n - 2) }
}

#[wasm_export("factorial")]
F factorial(n: i64) -> i64 = I n <= 1 { 1 } E { n * @(n - 1) }

F main() -> i64 {
    result := add(10, 20)
    f := fib(10)
    0
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::Wasm32Unknown).unwrap();
    assert!(ir.contains("wasm32-unknown-unknown"));
    assert!(ir.contains("wasm-import-module"));
    assert!(ir.contains("wasm-export-name"));
    // Should have 3 exports (add, fibonacci, factorial)
    let export_count = ir.matches("wasm-export-name").count();
    assert!(
        export_count >= 3,
        "Should have at least 3 exports, got {}",
        export_count
    );
}

#[test]
fn test_wasm_calculator_pattern_wasm32() {
    // Equivalent to examples/wasm_calculator.vais core operations
    let source = r#"
#[wasm_export("add")]
F add(a: i64, b: i64) -> i64 = a + b

#[wasm_export("subtract")]
F subtract(a: i64, b: i64) -> i64 = a - b

#[wasm_export("multiply")]
F multiply(a: i64, b: i64) -> i64 = a * b

#[wasm_export("divide")]
F divide(a: i64, b: i64) -> i64 {
    I b == 0 { R -1 }
    a / b
}

#[wasm_export("power")]
F power(base: i64, exp: i64) -> i64 {
    I exp == 0 { R 1 }
    result := mut base
    i := mut 1
    L i < exp {
        result = result * base
        i = i + 1
    }
    result
}

#[wasm_export("factorial")]
F factorial(n: i64) -> i64 {
    I n <= 1 { R 1 }
    result := mut 1
    i := mut 2
    L i <= n {
        result = result * i
        i = i + 1
    }
    result
}

F main() -> i64 {
    _ := add(10, 5)
    _ := multiply(4, 7)
    _ := power(2, 8)
    _ := factorial(5)
    0
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::Wasm32Unknown).unwrap();
    assert!(ir.contains("wasm32-unknown-unknown"));
    let export_count = ir.matches("wasm-export-name").count();
    assert!(
        export_count >= 6,
        "Calculator should have at least 6 exports, got {}",
        export_count
    );
}

#[test]
fn test_wasm_todo_pattern_wasm32() {
    // Simplified todo app pattern with imports and exports
    let source = r#"
N "C" {
    #[wasm_import("env", "console_log")]
    F console_log(ptr: i64, len: i64) -> i64;

    #[wasm_import("env", "dom_create_element")]
    F dom_create_element(tag_ptr: i64, tag_len: i64) -> i64;
}

#[wasm_export("init")]
F init() -> i64 {
    0
}

#[wasm_export("add_todo")]
F add_todo(title_ptr: i64, title_len: i64) -> i64 {
    I title_len <= 0 { R -1 }
    title_ptr
}

#[wasm_export("get_count")]
F get_count() -> i64 = 0

F main() -> i64 {
    _ := init()
    0
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::Wasm32Unknown).unwrap();
    assert!(ir.contains("wasm32-unknown-unknown"));
    assert!(
        ir.contains("wasm-import-module"),
        "Todo app should have imports"
    );
    assert!(
        ir.contains("wasm-export-name"),
        "Todo app should have exports"
    );
}

#[test]
fn test_wasm_api_client_pattern_wasm32() {
    // Simplified API client pattern with fetch imports
    let source = r#"
N "C" {
    #[wasm_import("env", "fetch_get")]
    F fetch_get(url_ptr: i64, url_len: i64) -> i64;

    #[wasm_import("env", "console_log")]
    F console_log(ptr: i64, len: i64) -> i64;
}

#[wasm_export("get_users")]
F get_users() -> i64 {
    url := "https://api.example.com/users"
    response := fetch_get(url as i64, 30)
    I response < 0 { R -1 }
    response
}

#[wasm_export("get_status")]
F get_status() -> i64 = 200

F main() -> i64 {
    _ := get_users()
    0
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::Wasm32Unknown).unwrap();
    assert!(ir.contains("wasm32-unknown-unknown"));
    assert!(ir.contains("wasm-import-module"));
    assert!(ir.contains("wasm-export-name"));
}

#[test]
fn test_wasm_calculator_pattern_wasip2() {
    // Calculator pattern compiled to WASI P2
    let source = r#"
#[wasm_export("add")]
F add(a: i64, b: i64) -> i64 = a + b

#[wasm_export("multiply")]
F multiply(a: i64, b: i64) -> i64 = a * b

F main() -> i64 {
    R add(1, 2) + multiply(3, 4)
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::WasiPreview2).unwrap();
    assert!(ir.contains("wasm32-wasip2"));
    assert!(ir.contains("wasm-export-name"));
}

#[test]
fn test_wasm_interop_pattern_wasip2() {
    // Interop pattern compiled to WASI P2
    let source = r#"
N "C" {
    #[wasm_import("env", "console_log")]
    F console_log(ptr: i64, len: i64) -> i64;
}

#[wasm_export("add")]
F add(a: i64, b: i64) -> i64 = a + b

#[wasm_export("fib")]
F fib(n: i64) -> i64 {
    I n <= 1 { n }
    E { @(n - 1) + @(n - 2) }
}

F main() -> i64 {
    R fib(10)
}
"#;
    let ir = compile_to_wasm_ir(source, vais_codegen::TargetTriple::WasiPreview2).unwrap();
    assert!(ir.contains("wasm32-wasip2"));
    assert!(ir.contains("wasm-import-module"));
    assert!(ir.contains("wasm-export-name"));
}

#[test]
fn test_example_files_exist() {
    // Verify all wasm example files exist
    let project_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    // wasm_todo_app.vais was moved to examples/archive/ in Phase 196 P196-C1
    // pending inkwell array-GEP codegen for global `[T; N]` access. The
    // other three still live under examples/.
    let examples = [
        "examples/wasm_interop.vais",
        "examples/wasm_calculator.vais",
        "examples/archive/wasm_todo_app.vais",
        "examples/wasm_api_client.vais",
    ];
    for rel_path in &examples {
        let path = project_root.join(rel_path);
        assert!(
            path.exists(),
            "Example file should exist: {}",
            path.display()
        );
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("wasm_export") || content.contains("wasm_import"),
            "Example {} should contain WASM annotations",
            path.display()
        );
    }
}

// ==============================================================================
// 5. WASM Bindgen (JS/TS) Generation Coverage
// ==============================================================================

#[test]
fn test_bindgen_js_multiple_functions() {
    use vais_codegen::wasm_component::{
        WasmBindgenGenerator, WitFunction, WitParam, WitResult, WitType,
    };

    let gen = WasmBindgenGenerator::new("math_lib");
    let functions = vec![
        WitFunction {
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
        },
        WitFunction {
            name: "multiply".to_string(),
            params: vec![
                WitParam {
                    name: "x".to_string(),
                    ty: WitType::F64,
                },
                WitParam {
                    name: "y".to_string(),
                    ty: WitType::F64,
                },
            ],
            results: Some(WitResult::Anon(WitType::F64)),
            docs: Some("Multiply two doubles".to_string()),
        },
        WitFunction {
            name: "negate".to_string(),
            params: vec![WitParam {
                name: "n".to_string(),
                ty: WitType::S32,
            }],
            results: Some(WitResult::Anon(WitType::S32)),
            docs: None,
        },
    ];

    let js = gen.generate_js_bindings(&functions);
    assert!(js.contains("add"));
    assert!(js.contains("multiply"));
    assert!(js.contains("negate"));

    let ts = gen.generate_ts_declarations(&functions);
    assert!(ts.contains("add"));
    assert!(ts.contains("multiply"));
    assert!(ts.contains("negate"));
}

#[test]
fn test_bindgen_js_no_params_function() {
    use vais_codegen::wasm_component::{WasmBindgenGenerator, WitFunction, WitResult, WitType};

    let gen = WasmBindgenGenerator::new("status");
    let functions = vec![WitFunction {
        name: "get_status".to_string(),
        params: vec![],
        results: Some(WitResult::Anon(WitType::S32)),
        docs: None,
    }];

    let js = gen.generate_js_bindings(&functions);
    assert!(js.contains("get_status"));

    let ts = gen.generate_ts_declarations(&functions);
    assert!(ts.contains("get_status"));
}

#[test]
fn test_bindgen_js_void_function() {
    use vais_codegen::wasm_component::{WasmBindgenGenerator, WitFunction, WitParam, WitType};

    let gen = WasmBindgenGenerator::new("actions");
    let functions = vec![WitFunction {
        name: "reset".to_string(),
        params: vec![WitParam {
            name: "code".to_string(),
            ty: WitType::S32,
        }],
        results: None,
        docs: Some("Reset with code".to_string()),
    }];

    let js = gen.generate_js_bindings(&functions);
    assert!(js.contains("reset"));
}

#[test]
fn test_bindgen_wasm_js_module() {
    use vais_bindgen::wasm_js::{WasmExportInfo, WasmImportInfo, WasmJsBindgen};

    let mut gen = WasmJsBindgen::new("my_app");

    gen.add_import(WasmImportInfo {
        module: "env".to_string(),
        name: "console_log".to_string(),
        vais_name: "console_log".to_string(),
        params: vec!["i64".to_string(), "i64".to_string()],
        return_type: None,
    });

    gen.add_import(WasmImportInfo {
        module: "wasi_snapshot_preview1".to_string(),
        name: "fd_write".to_string(),
        vais_name: "fd_write".to_string(),
        params: vec![
            "i32".to_string(),
            "i32".to_string(),
            "i32".to_string(),
            "i32".to_string(),
        ],
        return_type: Some("i32".to_string()),
    });

    gen.add_export(WasmExportInfo {
        wasm_name: "process".to_string(),
        js_name: "process".to_string(),
        params: vec![("input".to_string(), "i64".to_string())],
        return_type: Some("i64".to_string()),
    });

    gen.add_export(WasmExportInfo {
        wasm_name: "init".to_string(),
        js_name: "init".to_string(),
        params: vec![],
        return_type: Some("i32".to_string()),
    });

    let js = gen.generate_js();
    assert!(js.contains("createImports"));
    assert!(js.contains("\"env\""));
    assert!(js.contains("\"wasi_snapshot_preview1\""));
    assert!(js.contains("console_log"));
    assert!(js.contains("fd_write"));
    assert!(js.contains("process: (input) => instance.exports.process(input)"));
    assert!(js.contains("init: () => instance.exports.init()"));
    assert!(js.contains("WebAssembly.instantiate"));

    let dts = gen.generate_dts();
    assert!(dts.contains("My_appModule"));
    assert!(dts.contains("process(input: number): number"));
    assert!(dts.contains("init(): number"));
    assert!(dts.contains("Promise<My_appModule>"));
}

// ==============================================================================
// 6. Cross-target Regression Tests
// ==============================================================================

// REGRESSION(phase-115): same source must compile to all three WASM targets
#[test]
fn test_same_source_all_wasm_targets() {
    let source = r#"
F fib(n: i64) -> i64 {
    I n <= 1 { n }
    E { @(n - 1) + @(n - 2) }
}

F main() -> i64 {
    R fib(10)
}
"#;
    // All three WASM targets should produce valid IR
    let ir_wasm32 = compile_to_wasm_ir(source, vais_codegen::TargetTriple::Wasm32Unknown)
        .expect("wasm32-unknown-unknown should compile");
    let ir_wasip1 = compile_to_wasm_ir(source, vais_codegen::TargetTriple::WasiPreview1)
        .expect("wasm32-wasi should compile");
    let ir_wasip2 = compile_to_wasm_ir(source, vais_codegen::TargetTriple::WasiPreview2)
        .expect("wasm32-wasip2 should compile");

    // Each should have distinct target triple
    assert!(ir_wasm32.contains("wasm32-unknown-unknown"));
    assert!(ir_wasip1.contains("wasm32-wasi"));
    assert!(ir_wasip2.contains("wasm32-wasip2"));

    // All should have the same data layout (32-bit pointers)
    assert!(ir_wasm32.contains("e-m:e-p:32:32"));
    assert!(ir_wasip1.contains("e-m:e-p:32:32"));
    assert!(ir_wasip2.contains("e-m:e-p:32:32"));

    // All should contain the fib function
    assert!(ir_wasm32.contains("@fib"));
    assert!(ir_wasip1.contains("@fib"));
    assert!(ir_wasip2.contains("@fib"));
}

#[test]
fn test_native_vs_wasm_same_ir_structure() {
    let source = r#"
F double(x: i64) -> i64 = x * 2

F main() -> i64 {
    R double(21)
}
"#;
    let ir_native = compile_to_ir(source).expect("native should compile");
    let ir_wasm = compile_to_wasm_ir(source, vais_codegen::TargetTriple::Wasm32Unknown)
        .expect("wasm32 should compile");

    // Both should contain the double function
    assert!(ir_native.contains("@double"));
    assert!(ir_wasm.contains("@double"));

    // Different target triples
    assert!(!ir_native.contains("wasm32"));
    assert!(ir_wasm.contains("wasm32-unknown-unknown"));
}

#[test]
fn test_wasm_target_parse_all_variants() {
    use vais_codegen::TargetTriple;

    // wasm32-unknown-unknown aliases
    assert_eq!(
        TargetTriple::parse("wasm32"),
        Some(TargetTriple::Wasm32Unknown)
    );
    assert_eq!(
        TargetTriple::parse("wasm32-unknown-unknown"),
        Some(TargetTriple::Wasm32Unknown)
    );

    // WASI Preview 1 aliases
    assert_eq!(
        TargetTriple::parse("wasi"),
        Some(TargetTriple::WasiPreview1)
    );
    assert_eq!(
        TargetTriple::parse("wasm32-wasi"),
        Some(TargetTriple::WasiPreview1)
    );
    assert_eq!(
        TargetTriple::parse("wasi-preview1"),
        Some(TargetTriple::WasiPreview1)
    );

    // WASI Preview 2 aliases
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
fn test_wasm_serializer_record_layout() {
    use vais_codegen::wasm_component::{WasmSerializer, WitType};

    let ser = WasmSerializer::new();

    // Verify size calculations for complex types
    assert_eq!(ser.wit_type_size(&WitType::U8), 1);
    assert_eq!(ser.wit_type_size(&WitType::U16), 2);
    assert_eq!(ser.wit_type_size(&WitType::U32), 4);
    assert_eq!(ser.wit_type_size(&WitType::U64), 8);
    assert_eq!(ser.wit_type_size(&WitType::S8), 1);
    assert_eq!(ser.wit_type_size(&WitType::S16), 2);
    assert_eq!(ser.wit_type_size(&WitType::S32), 4);
    assert_eq!(ser.wit_type_size(&WitType::S64), 8);
    assert_eq!(ser.wit_type_size(&WitType::F32), 4);
    assert_eq!(ser.wit_type_size(&WitType::F64), 8);
    assert_eq!(ser.wit_type_size(&WitType::Bool), 1);

    // Alignment should round up
    assert_eq!(ser.aligned_size(&WitType::U8), 4);
    assert_eq!(ser.aligned_size(&WitType::U16), 4);
    assert_eq!(ser.aligned_size(&WitType::U32), 4);
    assert_eq!(ser.aligned_size(&WitType::U64), 8);
}

#[test]
fn test_wasm_serializer_js_read_write_all_types() {
    use vais_codegen::wasm_component::{WasmSerializer, WitType};

    let ser = WasmSerializer::new();

    // Verify JS read/write for each integer type
    let types_and_methods = vec![
        (WitType::S8, "setInt8", "getInt8"),
        (WitType::S16, "setInt16", "getInt16"),
        (WitType::S32, "setInt32", "getInt32"),
        (WitType::U8, "setUint8", "getUint8"),
        (WitType::U16, "setUint16", "getUint16"),
        (WitType::U32, "setUint32", "getUint32"),
        (WitType::F32, "setFloat32", "getFloat32"),
        (WitType::F64, "setFloat64", "getFloat64"),
    ];

    for (ty, write_method, read_method) in &types_and_methods {
        let write = ser.generate_js_write(ty, "val", "0");
        assert!(
            write.contains(write_method),
            "Write for {:?} should use {}, got: {}",
            ty,
            write_method,
            write
        );

        let read = ser.generate_js_read(ty, "0");
        assert!(
            read.contains(read_method),
            "Read for {:?} should use {}, got: {}",
            ty,
            read_method,
            read
        );
    }
}

#[test]
fn test_vais_type_to_wit_comprehensive() {
    use vais_codegen::wasm_component::{vais_type_to_wit, WitType};
    use vais_types::ResolvedType;

    // Integer types
    assert_eq!(vais_type_to_wit(&ResolvedType::I8), Some(WitType::S8));
    assert_eq!(vais_type_to_wit(&ResolvedType::I16), Some(WitType::S16));
    assert_eq!(vais_type_to_wit(&ResolvedType::I32), Some(WitType::S32));
    assert_eq!(vais_type_to_wit(&ResolvedType::I64), Some(WitType::S64));
    assert_eq!(vais_type_to_wit(&ResolvedType::U8), Some(WitType::U8));
    assert_eq!(vais_type_to_wit(&ResolvedType::U16), Some(WitType::U16));
    assert_eq!(vais_type_to_wit(&ResolvedType::U32), Some(WitType::U32));
    assert_eq!(vais_type_to_wit(&ResolvedType::U64), Some(WitType::U64));

    // Float types
    assert_eq!(vais_type_to_wit(&ResolvedType::F32), Some(WitType::F32));
    assert_eq!(vais_type_to_wit(&ResolvedType::F64), Some(WitType::F64));

    // Bool and String
    assert_eq!(vais_type_to_wit(&ResolvedType::Bool), Some(WitType::Bool));
    assert_eq!(vais_type_to_wit(&ResolvedType::Str), Some(WitType::String));

    // Collection types
    assert_eq!(
        vais_type_to_wit(&ResolvedType::Array(Box::new(ResolvedType::I64))),
        Some(WitType::List(Box::new(WitType::S64)))
    );
    assert_eq!(
        vais_type_to_wit(&ResolvedType::Optional(Box::new(ResolvedType::Bool))),
        Some(WitType::Option_(Box::new(WitType::Bool)))
    );

    // Unit type
    assert_eq!(vais_type_to_wit(&ResolvedType::Unit), None);
}

#[test]
fn test_component_link_config_full_workflow() {
    use vais_codegen::wasm_component::{ComponentLinkConfig, WasiManifest, WitType};

    // Build a realistic component link config
    let mut manifest = WasiManifest::new();
    manifest.add_import("wasi:cli/stdout@0.2.0");
    manifest.add_import("wasi:cli/stdin@0.2.0");
    manifest.add_import("wasi:filesystem/types@0.2.0");
    manifest.add_import("wasi:io/streams@0.2.0");
    manifest.add_export("run", &WitType::S32);

    let mut config = ComponentLinkConfig::new()
        .with_wasi_manifest(manifest)
        .with_adapter("wasi_snapshot_preview1.reactor.wasm");

    config.add_import("io".to_string(), "wasi:io/streams@0.2.0".to_string());
    config.add_export("run".to_string(), "wasi:cli/run@0.2.0".to_string());

    let args = config.to_link_args();
    assert!(args.contains(&"--adapt".to_string()));
    assert!(args.contains(&"wasi_snapshot_preview1.reactor.wasm".to_string()));
    assert!(args.contains(&"--import".to_string()));
    assert!(args.contains(&"--export".to_string()));

    // Verify WASI manifest is accessible
    let wasi = config.wasi_manifest.as_ref().unwrap();
    assert_eq!(wasi.imports.len(), 4);
    assert_eq!(wasi.exports.len(), 1);
}

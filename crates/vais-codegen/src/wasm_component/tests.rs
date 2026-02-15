#[cfg(test)]
mod tests {
    use crate::wasm_component::*;
    use vais_types::ResolvedType;

    #[test]
    fn test_wit_type_display() {
        assert_eq!(WitType::Bool.to_string(), "bool");
        assert_eq!(WitType::String.to_string(), "string");
        assert_eq!(
            WitType::List(Box::new(WitType::U32)).to_string(),
            "list<u32>"
        );
        assert_eq!(
            WitType::Option_(Box::new(WitType::String)).to_string(),
            "option<string>"
        );
    }

    #[test]
    fn test_wit_result_display() {
        let result = WitType::Result_ {
            ok: Some(Box::new(WitType::U32)),
            err: Some(Box::new(WitType::String)),
        };
        assert_eq!(result.to_string(), "result<u32, string>");

        let result_no_err = WitType::Result_ {
            ok: Some(Box::new(WitType::U32)),
            err: None,
        };
        assert_eq!(result_no_err.to_string(), "result<u32>");
    }

    #[test]
    fn test_wit_package_creation() {
        let mut package = WitPackage::new("vais", "example");
        package.version = Some("0.1.0".to_string());

        let interface = WitInterface {
            name: "calculator".to_string(),
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
                docs: Some("Add two numbers".to_string()),
            }],
            resources: vec![],
            docs: Some("Calculator interface".to_string()),
        };

        package.add_interface(interface);

        let wit = package.to_wit_string();
        assert!(wit.contains("package vais:example@0.1.0;"));
        assert!(wit.contains("interface calculator"));
        assert!(wit.contains("add: func(a: s32, b: s32) -> s32"));
    }

    #[test]
    fn test_wit_record_generation() {
        let record = WitRecord {
            name: "person".to_string(),
            fields: vec![
                WitField {
                    name: "name".to_string(),
                    ty: WitType::String,
                    docs: Some("Person's name".to_string()),
                },
                WitField {
                    name: "age".to_string(),
                    ty: WitType::U32,
                    docs: None,
                },
            ],
            docs: Some("A person record".to_string()),
        };

        let package = WitPackage::new("test", "types");
        let output = package.format_type_definition(&WitTypeDefinition::Record(record), 0);

        assert!(output.contains("record person"));
        assert!(output.contains("name: string"));
        assert!(output.contains("age: u32"));
    }

    #[test]
    fn test_wit_variant_generation() {
        let variant = WitVariant {
            name: "result".to_string(),
            cases: vec![
                WitVariantCase {
                    name: "ok".to_string(),
                    ty: Some(WitType::S32),
                    docs: None,
                },
                WitVariantCase {
                    name: "err".to_string(),
                    ty: Some(WitType::String),
                    docs: None,
                },
            ],
            docs: None,
        };

        let package = WitPackage::new("test", "types");
        let output = package.format_type_definition(&WitTypeDefinition::Variant(variant), 0);

        assert!(output.contains("variant result"));
        assert!(output.contains("ok(s32)"));
        assert!(output.contains("err(string)"));
    }

    #[test]
    fn test_wit_world_generation() {
        let world = WitWorld {
            name: "my-world".to_string(),
            imports: vec![WitImport {
                name: "console".to_string(),
                item: WitImportItem::Interface("wasi:cli/stdio".to_string()),
            }],
            exports: vec![WitExport {
                name: "run".to_string(),
                item: WitExportItem::Function(WitFunction {
                    name: "run".to_string(),
                    params: vec![],
                    results: None,
                    docs: None,
                }),
            }],
            docs: Some("Main world".to_string()),
        };

        let package = WitPackage::new("test", "world");
        let output = package.format_world(&world);

        assert!(output.contains("world my-world"));
        assert!(output.contains("import wasi:cli/stdio"));
        assert!(output.contains("export"));
    }

    #[test]
    fn test_component_link_config() {
        let config = ComponentLinkConfig::new()
            .reactor()
            .with_adapter("wasi_snapshot_preview1.wasm");

        assert!(config.reactor_mode);
        assert!(!config.command_mode);
        assert_eq!(
            config.adapter_module,
            Some("wasi_snapshot_preview1.wasm".to_string())
        );

        let args = config.to_link_args();
        assert!(args.contains(&"--adapt".to_string()));
    }

    #[test]
    fn test_vais_type_conversion() {
        assert_eq!(vais_type_to_wit(&ResolvedType::Bool), Some(WitType::Bool));
        assert_eq!(vais_type_to_wit(&ResolvedType::I32), Some(WitType::S32));
        assert_eq!(vais_type_to_wit(&ResolvedType::U64), Some(WitType::U64));
        assert_eq!(vais_type_to_wit(&ResolvedType::Str), Some(WitType::String));

        let list_type = ResolvedType::Array(Box::new(ResolvedType::U32));
        assert_eq!(
            vais_type_to_wit(&list_type),
            Some(WitType::List(Box::new(WitType::U32)))
        );

        let option_type = ResolvedType::Optional(Box::new(ResolvedType::Str));
        assert_eq!(
            vais_type_to_wit(&option_type),
            Some(WitType::Option_(Box::new(WitType::String)))
        );
    }

    #[test]
    fn test_wasi_manifest_creation() {
        let mut manifest = WasiManifest::new();

        manifest.add_import("wasi:filesystem/types");
        manifest.add_import("wasi:cli/stdio");
        manifest.add_export("process", &WitType::S32);

        assert_eq!(manifest.imports.len(), 2);
        assert_eq!(manifest.exports.len(), 1);

        let wit = manifest.to_wit_string();
        assert!(wit.contains("import wasi:filesystem/types"));
        assert!(wit.contains("import wasi:cli/stdio"));
        assert!(wit.contains("export process: s32"));
    }

    #[test]
    fn test_wasi_manifest_duplicate_imports() {
        let mut manifest = WasiManifest::new();

        manifest.add_import("wasi:filesystem/types");
        manifest.add_import("wasi:filesystem/types");

        // Should only add once
        assert_eq!(manifest.imports.len(), 1);
    }

    #[test]
    fn test_wasm_bindgen_generator_js() {
        let generator = WasmBindgenGenerator::new("calculator");

        let functions = vec![WitFunction {
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
            docs: Some("Add two numbers".to_string()),
        }];

        let js_code = generator.generate_js_bindings(&functions);

        assert!(js_code.contains("class VaisModule"));
        assert!(js_code.contains("add(a, b)"));
        assert!(js_code.contains("this.exports.add(a, b)"));
        assert!(js_code.contains("loadcalculator"));
    }

    #[test]
    fn test_wasm_bindgen_generator_ts() {
        let generator = WasmBindgenGenerator::new("math");

        let functions = vec![WitFunction {
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
            docs: Some("Multiply two numbers".to_string()),
        }];

        let ts_code = generator.generate_ts_declarations(&functions);

        assert!(ts_code.contains("class VaisModule"));
        assert!(ts_code.contains("multiply(x: number, y: number): number"));
        assert!(ts_code.contains("loadmath(): Promise<VaisModule>"));
    }

    #[test]
    fn test_wasm_bindgen_string_conversion() {
        let generator = WasmBindgenGenerator::new("strings");

        let functions = vec![WitFunction {
            name: "greet".to_string(),
            params: vec![WitParam {
                name: "name".to_string(),
                ty: WitType::String,
            }],
            results: Some(WitResult::Anon(WitType::String)),
            docs: None,
        }];

        let js_code = generator.generate_js_bindings(&functions);

        assert!(js_code.contains("_convert_string"));
        assert!(js_code.contains("_convert_from_string"));
    }

    #[test]
    fn test_wasm_bindgen_complex_types() {
        let generator = WasmBindgenGenerator::new("complex");

        let functions = vec![WitFunction {
            name: "process".to_string(),
            params: vec![WitParam {
                name: "items".to_string(),
                ty: WitType::List(Box::new(WitType::U32)),
            }],
            results: Some(WitResult::Anon(WitType::Option_(Box::new(WitType::U32)))),
            docs: None,
        }];

        let ts_code = generator.generate_ts_declarations(&functions);

        assert!(ts_code.contains("items: Array<number>"));
        assert!(ts_code.contains("number | null"));
    }

    #[test]
    fn test_component_link_config_with_wasi_manifest() {
        let mut manifest = WasiManifest::new();
        manifest.add_import("wasi:cli/stdio");

        let config = ComponentLinkConfig::new()
            .with_wasi_manifest(manifest)
            .with_adapter("wasi_snapshot_preview1.wasm");

        assert!(config.wasi_manifest.is_some());
        let wasi = config.wasi_manifest.as_ref().unwrap();
        assert_eq!(wasi.imports.len(), 1);
    }

    #[test]
    fn test_component_link_config_wasi_manifest_mut() {
        let mut config = ComponentLinkConfig::new();

        let manifest = config.wasi_manifest_mut();
        manifest.add_import("wasi:filesystem/types");
        manifest.add_export("main", &WitType::S32);

        assert!(config.wasi_manifest.is_some());
        assert_eq!(config.wasi_manifest.as_ref().unwrap().imports.len(), 1);
        assert_eq!(config.wasi_manifest.as_ref().unwrap().exports.len(), 1);
    }

    #[test]
    fn test_wasm_serializer_type_sizes() {
        let ser = WasmSerializer::new();
        assert_eq!(ser.wit_type_size(&WitType::Bool), 1);
        assert_eq!(ser.wit_type_size(&WitType::S32), 4);
        assert_eq!(ser.wit_type_size(&WitType::S64), 8);
        assert_eq!(ser.wit_type_size(&WitType::F64), 8);
        assert_eq!(ser.wit_type_size(&WitType::String), 8);
        assert_eq!(ser.wit_type_size(&WitType::List(Box::new(WitType::S32))), 8);
    }

    #[test]
    fn test_wasm_serializer_alignment() {
        let ser = WasmSerializer::new();
        assert_eq!(ser.aligned_size(&WitType::Bool), 4); // 1 → 4 (aligned)
        assert_eq!(ser.aligned_size(&WitType::S32), 4); // 4 → 4 (exact)
        assert_eq!(ser.aligned_size(&WitType::S64), 8); // 8 → 8 (exact)
        assert_eq!(ser.aligned_size(&WitType::String), 8);
    }

    #[test]
    fn test_wasm_serializer_js_write() {
        let ser = WasmSerializer::new();
        let write = ser.generate_js_write(&WitType::S32, "x", "offset");
        assert!(write.contains("setInt32"));
        assert!(write.contains("true")); // little-endian

        let write_str = ser.generate_js_write(&WitType::String, "s", "offset");
        assert!(write_str.contains("encoder.encode"));
        assert!(write_str.contains("alloc"));
    }

    #[test]
    fn test_wasm_serializer_js_read() {
        let ser = WasmSerializer::new();
        let read = ser.generate_js_read(&WitType::S32, "offset");
        assert!(read.contains("getInt32"));

        let read_str = ser.generate_js_read(&WitType::String, "offset");
        assert!(read_str.contains("decoder.decode"));
    }

    #[test]
    fn test_wasm_serializer_serde_module() {
        let ser = WasmSerializer::new();
        let module = ser.generate_js_serde_module();
        assert!(module.contains("class WasmSerde"));
        assert!(module.contains("writeString"));
        assert!(module.contains("readString"));
        assert!(module.contains("writeArray"));
        assert!(module.contains("readArray"));
        assert!(module.contains("writeStruct"));
        assert!(module.contains("readStruct"));
        assert!(module.contains("writeOption"));
        assert!(module.contains("readOption"));
        assert!(module.contains("writeResult"));
        assert!(module.contains("readResult"));
        assert!(module.contains("TextEncoder"));
        assert!(module.contains("TextDecoder"));
    }

    #[test]
    fn test_wasm_serializer_ir_types() {
        let ser = WasmSerializer::new();
        let ir = ser.generate_wasm_serde_ir();
        assert!(ir.contains("%WasmString"));
        assert!(ir.contains("%WasmArray"));
        assert!(ir.contains("%WasmOption"));
        assert!(ir.contains("%WasmResult"));
    }

    #[test]
    fn test_wasm_serializer_option_write() {
        let ser = WasmSerializer::new();
        let write =
            ser.generate_js_write(&WitType::Option_(Box::new(WitType::S32)), "val", "offset");
        assert!(write.contains("null"));
        assert!(write.contains("undefined"));
        assert!(write.contains("setUint32"));
    }

    #[test]
    fn test_wasm_serializer_list_read() {
        let ser = WasmSerializer::new();
        let read = ser.generate_js_read(&WitType::List(Box::new(WitType::S32)), "offset");
        assert!(read.contains("getUint32"));
        assert!(read.contains("result"));
        assert!(read.contains("push"));
    }
}

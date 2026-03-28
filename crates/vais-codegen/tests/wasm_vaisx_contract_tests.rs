//! VaisX Contract Tests — WASM Bindgen
//!
//! Verifies that `#[wasm]` function scenarios produce valid JS glue code
//! via `WasmBindgenGenerator::generate_js_bindings()` and valid TS declarations
//! via `generate_ts_declarations()`.
//!
//! These tests ensure the interface contract between vaisx-compiler's
//! codegen_wasm.rs and the core wasm_component module remains stable.

use vais_codegen::wasm_component::*;

// ============================================================================
// 1. Basic #[wasm] function → JS bindings
// ============================================================================

#[test]
fn test_wasm_simple_numeric_function() {
    let gen = WasmBindgenGenerator::new("component");
    let funcs = vec![WitFunction {
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

    let js = gen.generate_js_bindings(&funcs);

    assert!(
        js.contains("class VaisModule"),
        "Should contain VaisModule class"
    );
    assert!(
        js.contains("add(a, b)"),
        "Should contain function signature"
    );
    assert!(
        js.contains("this.exports.add(a, b)"),
        "Should call WASM export"
    );
    assert!(js.contains("return result"), "Should return result");
    assert!(
        js.contains("loadcomponent"),
        "Should contain loader function"
    );
}

#[test]
fn test_wasm_simple_numeric_function_ts() {
    let gen = WasmBindgenGenerator::new("component");
    let funcs = vec![WitFunction {
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
    }];

    let ts = gen.generate_ts_declarations(&funcs);

    assert!(
        ts.contains("add(a: number, b: number): number"),
        "TS should have typed signature"
    );
    assert!(
        ts.contains("loadcomponent(): Promise<VaisModule>"),
        "TS should have loader"
    );
}

// ============================================================================
// 2. String parameter conversion
// ============================================================================

#[test]
fn test_wasm_string_param_conversion() {
    let gen = WasmBindgenGenerator::new("greet_module");
    let funcs = vec![WitFunction {
        name: "greet".to_string(),
        params: vec![WitParam {
            name: "name".to_string(),
            ty: WitType::String,
        }],
        results: Some(WitResult::Anon(WitType::String)),
        docs: None,
    }];

    let js = gen.generate_js_bindings(&funcs);

    assert!(
        js.contains("_convert_string(name)"),
        "Should convert string param"
    );
    assert!(
        js.contains("_convert_from_string(result)"),
        "Should convert string result"
    );
    assert!(js.contains("TextEncoder"), "Should include encoder helper");
    assert!(js.contains("TextDecoder"), "Should include decoder helper");
}

#[test]
fn test_wasm_string_param_ts_declaration() {
    let gen = WasmBindgenGenerator::new("greet_module");
    let funcs = vec![WitFunction {
        name: "greet".to_string(),
        params: vec![WitParam {
            name: "name".to_string(),
            ty: WitType::String,
        }],
        results: Some(WitResult::Anon(WitType::String)),
        docs: None,
    }];

    let ts = gen.generate_ts_declarations(&funcs);

    assert!(
        ts.contains("greet(name: string): string"),
        "TS should have string types"
    );
}

// ============================================================================
// 3. List/Array parameter conversion
// ============================================================================

#[test]
fn test_wasm_list_param_conversion() {
    let gen = WasmBindgenGenerator::new("data_processor");
    let funcs = vec![WitFunction {
        name: "processData".to_string(),
        params: vec![WitParam {
            name: "raw".to_string(),
            ty: WitType::List(Box::new(WitType::F64)),
        }],
        results: Some(WitResult::Anon(WitType::List(Box::new(WitType::F64)))),
        docs: None,
    }];

    let js = gen.generate_js_bindings(&funcs);

    assert!(
        js.contains("_convert_list(raw)"),
        "Should convert list param"
    );
    assert!(
        js.contains("_convert_from_list(result)"),
        "Should convert list result"
    );
}

#[test]
fn test_wasm_list_ts_declaration() {
    let gen = WasmBindgenGenerator::new("data_processor");
    let funcs = vec![WitFunction {
        name: "processData".to_string(),
        params: vec![WitParam {
            name: "raw".to_string(),
            ty: WitType::List(Box::new(WitType::F64)),
        }],
        results: Some(WitResult::Anon(WitType::List(Box::new(WitType::F64)))),
        docs: None,
    }];

    let ts = gen.generate_ts_declarations(&funcs);

    assert!(
        ts.contains("raw: Array<number>"),
        "TS should have Array<number> param"
    );
    assert!(
        ts.contains("Array<number>"),
        "TS should have Array<number> return"
    );
}

// ============================================================================
// 4. Multiple #[wasm] functions
// ============================================================================

#[test]
fn test_wasm_multiple_functions() {
    let gen = WasmBindgenGenerator::new("math");
    let funcs = vec![
        WitFunction {
            name: "add".to_string(),
            params: vec![
                WitParam {
                    name: "a".to_string(),
                    ty: WitType::F64,
                },
                WitParam {
                    name: "b".to_string(),
                    ty: WitType::F64,
                },
            ],
            results: Some(WitResult::Anon(WitType::F64)),
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
            docs: None,
        },
    ];

    let js = gen.generate_js_bindings(&funcs);

    assert!(js.contains("add(a, b)"), "Should contain first function");
    assert!(
        js.contains("multiply(x, y)"),
        "Should contain second function"
    );
}

// ============================================================================
// 5. vais_type_to_wit — VaisX type mapping contract
// ============================================================================

#[test]
fn test_vais_type_to_wit_primitives() {
    use vais_types::ResolvedType;

    assert_eq!(vais_type_to_wit(&ResolvedType::Bool), Some(WitType::Bool));
    assert_eq!(vais_type_to_wit(&ResolvedType::I32), Some(WitType::S32));
    assert_eq!(vais_type_to_wit(&ResolvedType::I64), Some(WitType::S64));
    assert_eq!(vais_type_to_wit(&ResolvedType::U32), Some(WitType::U32));
    assert_eq!(vais_type_to_wit(&ResolvedType::U64), Some(WitType::U64));
    assert_eq!(vais_type_to_wit(&ResolvedType::F32), Some(WitType::F32));
    assert_eq!(vais_type_to_wit(&ResolvedType::F64), Some(WitType::F64));
    assert_eq!(vais_type_to_wit(&ResolvedType::Str), Some(WitType::String));
}

#[test]
fn test_vais_type_to_wit_containers() {
    use vais_types::ResolvedType;

    // Vec<f64> → list<f64>
    let vec_f64 = ResolvedType::Array(Box::new(ResolvedType::F64));
    assert_eq!(
        vais_type_to_wit(&vec_f64),
        Some(WitType::List(Box::new(WitType::F64)))
    );

    // Optional<str> → option<string>
    let opt_str = ResolvedType::Optional(Box::new(ResolvedType::Str));
    assert_eq!(
        vais_type_to_wit(&opt_str),
        Some(WitType::Option_(Box::new(WitType::String)))
    );

    // Result<i32, str> → result<s32, string>
    let result_type =
        ResolvedType::Result(Box::new(ResolvedType::I32), Box::new(ResolvedType::Str));
    assert_eq!(
        vais_type_to_wit(&result_type),
        Some(WitType::Result_ {
            ok: Some(Box::new(WitType::S32)),
            err: Some(Box::new(WitType::String)),
        })
    );
}

#[test]
fn test_vais_type_to_wit_tuple() {
    use vais_types::ResolvedType;

    let tuple = ResolvedType::Tuple(vec![ResolvedType::I32, ResolvedType::Str]);
    assert_eq!(
        vais_type_to_wit(&tuple),
        Some(WitType::Tuple(vec![WitType::S32, WitType::String]))
    );
}

#[test]
fn test_vais_type_to_wit_unmappable() {
    use vais_types::ResolvedType;

    // Function types can't be mapped to WIT
    let fn_type = ResolvedType::Fn {
        params: vec![ResolvedType::I32],
        ret: Box::new(ResolvedType::Bool),
        effects: Some(Box::new(vais_types::EffectSet::pure())),
    };
    assert_eq!(
        vais_type_to_wit(&fn_type),
        None,
        "Fn type should not map to WIT"
    );
}

// ============================================================================
// 6. WasmSerializer contract — JS serde for complex types
// ============================================================================

#[test]
fn test_wasm_serializer_generates_complete_serde_module() {
    let ser = WasmSerializer::new();
    let module = ser.generate_js_serde_module();

    // Must include all helper methods for VaisX codegen_wasm to use
    assert!(
        module.contains("class WasmSerde"),
        "Missing WasmSerde class"
    );
    assert!(module.contains("writeString"), "Missing writeString");
    assert!(module.contains("readString"), "Missing readString");
    assert!(module.contains("writeArray"), "Missing writeArray");
    assert!(module.contains("readArray"), "Missing readArray");
    assert!(module.contains("writeStruct"), "Missing writeStruct");
    assert!(module.contains("readStruct"), "Missing readStruct");
    assert!(module.contains("writeOption"), "Missing writeOption");
    assert!(module.contains("readOption"), "Missing readOption");
    assert!(module.contains("writeResult"), "Missing writeResult");
    assert!(module.contains("readResult"), "Missing readResult");
}

// ============================================================================
// 7. Named result type (multi-return)
// ============================================================================

#[test]
fn test_wasm_named_result_js() {
    let gen = WasmBindgenGenerator::new("multi_return");
    let funcs = vec![WitFunction {
        name: "divmod".to_string(),
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
        results: Some(WitResult::Named(vec![
            WitParam {
                name: "quotient".to_string(),
                ty: WitType::S32,
            },
            WitParam {
                name: "remainder".to_string(),
                ty: WitType::S32,
            },
        ])),
        docs: None,
    }];

    let js = gen.generate_js_bindings(&funcs);
    assert!(js.contains("quotient"), "Should destructure named results");
    assert!(js.contains("remainder"), "Should destructure named results");
}

#[test]
fn test_wasm_named_result_ts() {
    let gen = WasmBindgenGenerator::new("multi_return");
    let funcs = vec![WitFunction {
        name: "divmod".to_string(),
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
        results: Some(WitResult::Named(vec![
            WitParam {
                name: "quotient".to_string(),
                ty: WitType::S32,
            },
            WitParam {
                name: "remainder".to_string(),
                ty: WitType::S32,
            },
        ])),
        docs: None,
    }];

    let ts = gen.generate_ts_declarations(&funcs);
    assert!(
        ts.contains("quotient: number"),
        "TS should have named result fields"
    );
    assert!(
        ts.contains("remainder: number"),
        "TS should have named result fields"
    );
}

// ============================================================================
// 8. Void return (no result)
// ============================================================================

#[test]
fn test_wasm_void_function() {
    let gen = WasmBindgenGenerator::new("side_effects");
    let funcs = vec![WitFunction {
        name: "logMessage".to_string(),
        params: vec![WitParam {
            name: "msg".to_string(),
            ty: WitType::String,
        }],
        results: None,
        docs: None,
    }];

    let js = gen.generate_js_bindings(&funcs);
    assert!(js.contains("logMessage(msg)"), "Should have function");
    assert!(js.contains("return result"), "Should still return");

    let ts = gen.generate_ts_declarations(&funcs);
    assert!(ts.contains("void"), "TS should have void return");
}

// ============================================================================
// 9. Option/Result types in TS declarations
// ============================================================================

#[test]
fn test_wasm_option_result_ts_types() {
    let gen = WasmBindgenGenerator::new("nullable");
    let funcs = vec![
        WitFunction {
            name: "findItem".to_string(),
            params: vec![WitParam {
                name: "id".to_string(),
                ty: WitType::U32,
            }],
            results: Some(WitResult::Anon(WitType::Option_(Box::new(WitType::String)))),
            docs: None,
        },
        WitFunction {
            name: "parseNumber".to_string(),
            params: vec![WitParam {
                name: "s".to_string(),
                ty: WitType::String,
            }],
            results: Some(WitResult::Anon(WitType::Result_ {
                ok: Some(Box::new(WitType::S32)),
                err: Some(Box::new(WitType::String)),
            })),
            docs: None,
        },
    ];

    let ts = gen.generate_ts_declarations(&funcs);
    assert!(
        ts.contains("string | null"),
        "Option should map to T | null"
    );
    assert!(ts.contains("{ ok: number }"), "Result ok should be typed");
    assert!(ts.contains("{ err: string }"), "Result err should be typed");
}

// ============================================================================
// 10. Documentation preservation
// ============================================================================

#[test]
fn test_wasm_docs_in_js_bindings() {
    let gen = WasmBindgenGenerator::new("documented");
    let funcs = vec![WitFunction {
        name: "calculate".to_string(),
        params: vec![WitParam {
            name: "x".to_string(),
            ty: WitType::F64,
        }],
        results: Some(WitResult::Anon(WitType::F64)),
        docs: Some("Calculate the square root".to_string()),
    }];

    let js = gen.generate_js_bindings(&funcs);
    assert!(
        js.contains("Calculate the square root"),
        "JS should preserve docs"
    );

    let ts = gen.generate_ts_declarations(&funcs);
    assert!(
        ts.contains("Calculate the square root"),
        "TS should preserve docs"
    );
}

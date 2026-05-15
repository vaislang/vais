//! Browser-safe Vais compiler facade.
//!
//! This crate intentionally excludes the native `vaisc` CLI, LLVM, registry,
//! filesystem, and process execution layers. It exposes the parser plus
//! `vais-codegen-js` as a small WASM-compatible API for playground use.

use serde::Serialize;
use vais_codegen_js::JsCodeGenerator;
use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize)]
struct Diagnostic {
    line: usize,
    column: usize,
    message: String,
    severity: &'static str,
}

#[derive(Debug, Serialize)]
struct CompileResponse {
    success: bool,
    errors: Vec<Diagnostic>,
    warnings: Vec<Diagnostic>,
    js_code: Option<String>,
    compiler: &'static str,
}

/// Compile Vais source to JavaScript and return a JSON response.
///
/// Returning JSON keeps the JS boundary stable without exposing Rust structs
/// through wasm-bindgen-generated classes.
#[wasm_bindgen]
pub fn compile_to_js_json(source: &str) -> String {
    let response = compile_to_js(source);
    serde_json::to_string(&response).unwrap_or_else(|error| {
        format!(
            r#"{{"success":false,"errors":[{{"line":0,"column":0,"message":"failed to serialize response: {}","severity":"error"}}],"warnings":[],"js_code":null,"compiler":"vais-browser-compiler"}}"#,
            escape_json_string(&error.to_string())
        )
    })
}

/// Return the package version compiled into this browser compiler.
#[wasm_bindgen]
pub fn browser_compiler_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn compile_to_js(source: &str) -> CompileResponse {
    let module = match vais_parser::parse(source) {
        Ok(module) => module,
        Err(error) => {
            return CompileResponse {
                success: false,
                errors: vec![diagnostic_from_parse_error(source, &error)],
                warnings: Vec::new(),
                js_code: None,
                compiler: "vais-browser-compiler",
            };
        }
    };

    let mut generator = JsCodeGenerator::new();
    match generator.generate_module(&module) {
        Ok(js_code) => CompileResponse {
            success: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            js_code: Some(js_code),
            compiler: "vais-browser-compiler",
        },
        Err(error) => CompileResponse {
            success: false,
            errors: vec![Diagnostic {
                line: 0,
                column: 0,
                message: error.to_string(),
                severity: "error",
            }],
            warnings: Vec::new(),
            js_code: None,
            compiler: "vais-browser-compiler",
        },
    }
}

fn diagnostic_from_parse_error(source: &str, error: &vais_parser::ParseError) -> Diagnostic {
    let (line, column) = error
        .span()
        .map(|span| offset_to_line_column(source, span.start))
        .unwrap_or((0, 0));

    Diagnostic {
        line,
        column,
        message: error.to_string(),
        severity: "error",
    }
}

fn offset_to_line_column(source: &str, offset: usize) -> (usize, usize) {
    let clamped = offset.min(source.len());
    let mut line = 1usize;
    let mut column = 1usize;

    for (idx, ch) in source.char_indices() {
        if idx >= clamped {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
}

fn escape_json_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn compiles_basic_main_to_js() {
        let raw = compile_to_js_json("fn main() -> i64 = 42");
        let value: Value = serde_json::from_str(&raw).expect("valid JSON response");

        assert_eq!(value["success"], true);
        assert!(value["js_code"]
            .as_str()
            .expect("js_code")
            .contains("function main()"));
    }

    #[test]
    fn reports_parse_error_as_json() {
        let raw = compile_to_js_json("fn main( -> i64 = 42");
        let value: Value = serde_json::from_str(&raw).expect("valid JSON response");

        assert_eq!(value["success"], false);
        assert!(value["errors"][0]["message"]
            .as_str()
            .expect("message")
            .contains("Unexpected"));
    }

    #[test]
    fn maps_offsets_to_line_and_column() {
        assert_eq!(offset_to_line_column("a\nbc", 0), (1, 1));
        assert_eq!(offset_to_line_column("a\nbc", 2), (2, 1));
        assert_eq!(offset_to_line_column("a\nbc", 3), (2, 2));
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn wasm_compile_smoke() {
        let raw = compile_to_js_json("fn main() -> i64 = 7");
        assert!(raw.contains(r#""success":true"#));
        assert!(raw.contains("function main()"));
    }
}

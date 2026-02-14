//! wasm-bindgen generator for creating JavaScript bindings

use super::types::{WitFunction, WitResult, WitType};

#[derive(Debug, Clone)]
pub struct WasmBindgenGenerator {
    /// Module name for generated bindings
    pub module_name: String,
}

impl WasmBindgenGenerator {
    /// Create a new wasm-bindgen generator
    pub fn new(module_name: &str) -> Self {
        Self {
            module_name: module_name.to_string(),
        }
    }

    /// Generate JavaScript glue code from WIT functions
    pub fn generate_js_bindings(&self, funcs: &[WitFunction]) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "// Generated JavaScript bindings for {}\n\n",
            self.module_name
        ));

        output.push_str("export class VaisModule {\n");
        output.push_str("  constructor(instance) {\n");
        output.push_str("    this.instance = instance;\n");
        output.push_str("    this.exports = instance.exports;\n");
        output.push_str("  }\n\n");

        for func in funcs {
            let params = func
                .params
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", ");

            output.push_str(&format!("  {}({}) {{\n", func.name, params));

            // Add documentation if available
            if let Some(docs) = &func.docs {
                output.push_str(&format!("    // {}\n", docs));
            }

            // Generate parameter conversion
            for param in &func.params {
                if self.needs_conversion(&param.ty) {
                    output.push_str(&format!(
                        "    const _{} = this._convert_{}({});\n",
                        param.name,
                        self.wit_type_to_js_type(&param.ty),
                        param.name
                    ));
                }
            }

            // Call the WASM function
            let call_params = func
                .params
                .iter()
                .map(|p| {
                    if self.needs_conversion(&p.ty) {
                        format!("_{}", p.name)
                    } else {
                        p.name.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");

            output.push_str(&format!(
                "    const result = this.exports.{}({});\n",
                func.name, call_params
            ));

            // Generate result conversion
            if let Some(results) = &func.results {
                match results {
                    WitResult::Anon(ty) => {
                        if self.needs_conversion(ty) {
                            output.push_str(&format!(
                                "    return this._convert_from_{}(result);\n",
                                self.wit_type_to_js_type(ty)
                            ));
                        } else {
                            output.push_str("    return result;\n");
                        }
                    }
                    WitResult::Named(params) => {
                        output.push_str("    return {\n");
                        for (i, param) in params.iter().enumerate() {
                            output.push_str(&format!("      {}: result[{}],\n", param.name, i));
                        }
                        output.push_str("    };\n");
                    }
                }
            } else {
                output.push_str("    return result;\n");
            }

            output.push_str("  }\n\n");
        }

        // Add helper conversion methods
        output.push_str("  // Type conversion helpers\n");
        output.push_str("  _convert_string(str) {\n");
        output.push_str("    const encoder = new TextEncoder();\n");
        output.push_str("    return encoder.encode(str);\n");
        output.push_str("  }\n\n");

        output.push_str("  _convert_from_string(ptr) {\n");
        output.push_str("    const decoder = new TextDecoder();\n");
        output.push_str("    return decoder.decode(ptr);\n");
        output.push_str("  }\n\n");

        output.push_str("  _convert_list(array) {\n");
        output.push_str("    return array;\n");
        output.push_str("  }\n\n");

        output.push_str("  _convert_from_list(ptr) {\n");
        output.push_str("    return ptr;\n");
        output.push_str("  }\n");

        output.push_str("}\n\n");

        // Add module loader
        output.push_str(&format!(
            "export async function load{}() {{\n",
            self.module_name
        ));
        output.push_str(&format!(
            "  const response = await fetch('{}.wasm');\n",
            self.module_name
        ));
        output.push_str("  const bytes = await response.arrayBuffer();\n");
        output.push_str("  const module = await WebAssembly.compile(bytes);\n");
        output.push_str("  const instance = await WebAssembly.instantiate(module, {});\n");
        output.push_str("  return new VaisModule(instance);\n");
        output.push_str("}\n");

        output
    }

    /// Generate TypeScript type declarations from WIT functions
    pub fn generate_ts_declarations(&self, funcs: &[WitFunction]) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "// Generated TypeScript declarations for {}\n\n",
            self.module_name
        ));

        output.push_str("export class VaisModule {\n");
        output.push_str("  constructor(instance: WebAssembly.Instance);\n\n");

        for func in funcs {
            // Add documentation if available
            if let Some(docs) = &func.docs {
                output.push_str(&format!("  /**\n   * {}\n   */\n", docs));
            }

            // Generate parameter types
            let params = func
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name, self.wit_type_to_ts_type(&p.ty)))
                .collect::<Vec<_>>()
                .join(", ");

            // Generate return type
            let return_type = if let Some(results) = &func.results {
                match results {
                    WitResult::Anon(ty) => self.wit_type_to_ts_type(ty),
                    WitResult::Named(params) => {
                        let fields = params
                            .iter()
                            .map(|p| format!("{}: {}", p.name, self.wit_type_to_ts_type(&p.ty)))
                            .collect::<Vec<_>>()
                            .join("; ");
                        format!("{{ {} }}", fields)
                    }
                }
            } else {
                "void".to_string()
            };

            output.push_str(&format!(
                "  {}({}): {};\n\n",
                func.name, params, return_type
            ));
        }

        output.push_str("}\n\n");

        // Add module loader declaration
        output.push_str(&format!(
            "export function load{}(): Promise<VaisModule>;\n",
            self.module_name
        ));

        output
    }

    /// Convert WIT type to JavaScript type name
    fn wit_type_to_js_type(&self, ty: &WitType) -> String {
        match ty {
            WitType::String => "string".to_string(),
            WitType::List(_) => "list".to_string(),
            _ => "value".to_string(),
        }
    }

    /// Convert WIT type to TypeScript type
    fn wit_type_to_ts_type(&self, ty: &WitType) -> String {
        match ty {
            WitType::Bool => "boolean".to_string(),
            WitType::U8 | WitType::U16 | WitType::U32 | WitType::U64 => "number".to_string(),
            WitType::S8 | WitType::S16 | WitType::S32 | WitType::S64 => "number".to_string(),
            WitType::F32 | WitType::F64 => "number".to_string(),
            WitType::Char => "string".to_string(),
            WitType::String => "string".to_string(),
            WitType::List(inner) => format!("Array<{}>", self.wit_type_to_ts_type(inner)),
            WitType::Option_(inner) => format!("{} | null", self.wit_type_to_ts_type(inner)),
            WitType::Result_ { ok, err } => {
                let ok_type = ok
                    .as_ref()
                    .map(|t| self.wit_type_to_ts_type(t))
                    .unwrap_or_else(|| "void".to_string());
                let err_type = err
                    .as_ref()
                    .map(|t| self.wit_type_to_ts_type(t))
                    .unwrap_or_else(|| "Error".to_string());
                format!("{{ ok: {} }} | {{ err: {} }}", ok_type, err_type)
            }
            WitType::Tuple(types) => {
                let inner = types
                    .iter()
                    .map(|t| self.wit_type_to_ts_type(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", inner)
            }
            WitType::Record(name)
            | WitType::Variant(name)
            | WitType::Enum(name)
            | WitType::Flags(name)
            | WitType::Resource(name)
            | WitType::Named(name) => name.clone(),
        }
    }

    /// Check if a WIT type needs conversion between JS and WASM
    fn needs_conversion(&self, ty: &WitType) -> bool {
        matches!(ty, WitType::String | WitType::List(_))
    }
}

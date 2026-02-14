//! Serialization layer for JS↔WASM type conversions across the linear memory boundary

use super::types::WitType;

///
/// Handles encoding Vais types into WASM linear memory format and generating
/// JavaScript code to read/write these types.
#[derive(Debug, Clone)]
pub struct WasmSerializer {
    /// Alignment requirements for WASM types (in bytes)
    pub alignment: usize,
}

impl WasmSerializer {
    /// Create a new serializer with default 4-byte alignment (wasm32)
    pub fn new() -> Self {
        Self { alignment: 4 }
    }

    /// Get the size of a WIT type in WASM linear memory (in bytes)
    pub fn wit_type_size(&self, ty: &WitType) -> usize {
        match ty {
            WitType::Bool => 1,
            WitType::U8 | WitType::S8 => 1,
            WitType::U16 | WitType::S16 => 2,
            WitType::U32 | WitType::S32 | WitType::F32 => 4,
            WitType::U64 | WitType::S64 | WitType::F64 => 8,
            WitType::Char => 4,           // Unicode code point
            WitType::String => 8,         // ptr(4) + len(4) in wasm32
            WitType::List(_) => 8,        // ptr(4) + len(4) in wasm32
            WitType::Option_(_) => 8,     // tag(4) + value(4+)
            WitType::Result_ { .. } => 8, // tag(4) + payload(4+)
            WitType::Tuple(types) => types.iter().map(|t| self.aligned_size(t)).sum(),
            WitType::Record(_)
            | WitType::Variant(_)
            | WitType::Enum(_)
            | WitType::Flags(_)
            | WitType::Resource(_)
            | WitType::Named(_) => 4, // pointer
        }
    }

    /// Get aligned size (round up to alignment boundary)
    pub fn aligned_size(&self, ty: &WitType) -> usize {
        let size = self.wit_type_size(ty);
        (size + self.alignment - 1) & !(self.alignment - 1)
    }

    /// Generate JavaScript code to write a value of the given type to WASM linear memory
    pub fn generate_js_write(&self, ty: &WitType, var_name: &str, offset_expr: &str) -> String {
        match ty {
            WitType::Bool => format!(
                "view.setUint8({}, {} ? 1 : 0);",
                offset_expr, var_name
            ),
            WitType::U8 => format!("view.setUint8({}, {});", offset_expr, var_name),
            WitType::S8 => format!("view.setInt8({}, {});", offset_expr, var_name),
            WitType::U16 => format!("view.setUint16({}, {}, true);", offset_expr, var_name),
            WitType::S16 => format!("view.setInt16({}, {}, true);", offset_expr, var_name),
            WitType::U32 | WitType::Char => format!("view.setUint32({}, {}, true);", offset_expr, var_name),
            WitType::S32 => format!("view.setInt32({}, {}, true);", offset_expr, var_name),
            WitType::U64 => format!("view.setBigUint64({}, BigInt({}), true);", offset_expr, var_name),
            WitType::S64 => format!("view.setBigInt64({}, BigInt({}), true);", offset_expr, var_name),
            WitType::F32 => format!("view.setFloat32({}, {}, true);", offset_expr, var_name),
            WitType::F64 => format!("view.setFloat64({}, {}, true);", offset_expr, var_name),
            WitType::String => format!(
                "{{ const encoded = encoder.encode({}); const ptr = alloc(encoded.length); new Uint8Array(memory.buffer, ptr, encoded.length).set(encoded); view.setUint32({}, ptr, true); view.setUint32({} + 4, encoded.length, true); }}",
                var_name, offset_expr, offset_expr
            ),
            WitType::List(inner) => {
                let elem_size = self.wit_type_size(inner);
                format!(
                    "{{ const arr = {}; const ptr = alloc(arr.length * {}); for (let i = 0; i < arr.length; i++) {{ {} }} view.setUint32({}, ptr, true); view.setUint32({} + 4, arr.length, true); }}",
                    var_name, elem_size,
                    self.generate_js_write(inner, "arr[i]", &format!("ptr + i * {}", elem_size)),
                    offset_expr, offset_expr
                )
            }
            WitType::Option_(inner) => format!(
                "if ({} === null || {} === undefined) {{ view.setUint32({}, 0, true); }} else {{ view.setUint32({}, 1, true); {} }}",
                var_name, var_name, offset_expr, offset_expr,
                self.generate_js_write(inner, var_name, &format!("{} + 4", offset_expr))
            ),
            _ => format!("view.setUint32({}, {}, true);", offset_expr, var_name),
        }
    }

    /// Generate JavaScript code to read a value of the given type from WASM linear memory
    pub fn generate_js_read(&self, ty: &WitType, offset_expr: &str) -> String {
        match ty {
            WitType::Bool => format!("view.getUint8({}) !== 0", offset_expr),
            WitType::U8 => format!("view.getUint8({})", offset_expr),
            WitType::S8 => format!("view.getInt8({})", offset_expr),
            WitType::U16 => format!("view.getUint16({}, true)", offset_expr),
            WitType::S16 => format!("view.getInt16({}, true)", offset_expr),
            WitType::U32 | WitType::Char => format!("view.getUint32({}, true)", offset_expr),
            WitType::S32 => format!("view.getInt32({}, true)", offset_expr),
            WitType::U64 => format!("view.getBigUint64({}, true)", offset_expr),
            WitType::S64 => format!("view.getBigInt64({}, true)", offset_expr),
            WitType::F32 => format!("view.getFloat32({}, true)", offset_expr),
            WitType::F64 => format!("view.getFloat64({}, true)", offset_expr),
            WitType::String => format!(
                "decoder.decode(new Uint8Array(memory.buffer, view.getUint32({}, true), view.getUint32({} + 4, true)))",
                offset_expr, offset_expr
            ),
            WitType::List(inner) => {
                let elem_size = self.wit_type_size(inner);
                format!(
                    "(() => {{ const ptr = view.getUint32({}, true); const len = view.getUint32({} + 4, true); const result = []; for (let i = 0; i < len; i++) {{ result.push({}); }} return result; }})()",
                    offset_expr, offset_expr,
                    self.generate_js_read(inner, &format!("ptr + i * {}", elem_size))
                )
            }
            WitType::Option_(inner) => format!(
                "view.getUint32({}, true) === 0 ? null : {}",
                offset_expr,
                self.generate_js_read(inner, &format!("{} + 4", offset_expr))
            ),
            _ => format!("view.getUint32({}, true)", offset_expr),
        }
    }

    /// Generate a complete JS serialization helper module
    pub fn generate_js_serde_module(&self) -> String {
        let mut js = String::new();

        js.push_str("// Vais WASM Serialization Helpers\n");
        js.push_str("// Auto-generated by vais-codegen\n\n");

        js.push_str("export class WasmSerde {\n");
        js.push_str("  constructor(memory, alloc, dealloc) {\n");
        js.push_str("    this.memory = memory;\n");
        js.push_str("    this.alloc = alloc;\n");
        js.push_str("    this.dealloc = dealloc || (() => {});\n");
        js.push_str("    this.encoder = new TextEncoder();\n");
        js.push_str("    this.decoder = new TextDecoder();\n");
        js.push_str("  }\n\n");

        js.push_str("  get view() {\n");
        js.push_str("    return new DataView(this.memory.buffer);\n");
        js.push_str("  }\n\n");

        // writeString
        js.push_str("  writeString(str) {\n");
        js.push_str("    const encoded = this.encoder.encode(str);\n");
        js.push_str("    const ptr = this.alloc(encoded.length + 1);\n");
        js.push_str("    new Uint8Array(this.memory.buffer, ptr, encoded.length).set(encoded);\n");
        js.push_str("    new Uint8Array(this.memory.buffer)[ptr + encoded.length] = 0;\n");
        js.push_str("    return { ptr, len: encoded.length };\n");
        js.push_str("  }\n\n");

        // readString
        js.push_str("  readString(ptr, len) {\n");
        js.push_str(
            "    return this.decoder.decode(new Uint8Array(this.memory.buffer, ptr, len));\n",
        );
        js.push_str("  }\n\n");

        // writeArray
        js.push_str("  writeArray(arr, elemSize, writeFn) {\n");
        js.push_str("    const ptr = this.alloc(arr.length * elemSize);\n");
        js.push_str("    for (let i = 0; i < arr.length; i++) {\n");
        js.push_str("      writeFn(this.view, ptr + i * elemSize, arr[i]);\n");
        js.push_str("    }\n");
        js.push_str("    return { ptr, len: arr.length };\n");
        js.push_str("  }\n\n");

        // readArray
        js.push_str("  readArray(ptr, len, elemSize, readFn) {\n");
        js.push_str("    const result = [];\n");
        js.push_str("    for (let i = 0; i < len; i++) {\n");
        js.push_str("      result.push(readFn(this.view, ptr + i * elemSize));\n");
        js.push_str("    }\n");
        js.push_str("    return result;\n");
        js.push_str("  }\n\n");

        // writeStruct
        js.push_str("  writeStruct(obj, layout) {\n");
        js.push_str("    const ptr = this.alloc(layout.size);\n");
        js.push_str("    for (const field of layout.fields) {\n");
        js.push_str("      field.write(this.view, ptr + field.offset, obj[field.name]);\n");
        js.push_str("    }\n");
        js.push_str("    return ptr;\n");
        js.push_str("  }\n\n");

        // readStruct
        js.push_str("  readStruct(ptr, layout) {\n");
        js.push_str("    const obj = {};\n");
        js.push_str("    for (const field of layout.fields) {\n");
        js.push_str("      obj[field.name] = field.read(this.view, ptr + field.offset);\n");
        js.push_str("    }\n");
        js.push_str("    return obj;\n");
        js.push_str("  }\n\n");

        // writeOption
        js.push_str("  writeOption(val, writeFn) {\n");
        js.push_str("    const ptr = this.alloc(8);\n");
        js.push_str("    if (val === null || val === undefined) {\n");
        js.push_str("      this.view.setUint32(ptr, 0, true);\n");
        js.push_str("    } else {\n");
        js.push_str("      this.view.setUint32(ptr, 1, true);\n");
        js.push_str("      writeFn(this.view, ptr + 4, val);\n");
        js.push_str("    }\n");
        js.push_str("    return ptr;\n");
        js.push_str("  }\n\n");

        // readOption
        js.push_str("  readOption(ptr, readFn) {\n");
        js.push_str("    const tag = this.view.getUint32(ptr, true);\n");
        js.push_str("    if (tag === 0) return null;\n");
        js.push_str("    return readFn(this.view, ptr + 4);\n");
        js.push_str("  }\n\n");

        // writeResult
        js.push_str("  writeResult(val, writeOkFn, writeErrFn) {\n");
        js.push_str("    const ptr = this.alloc(8);\n");
        js.push_str("    if (val.ok !== undefined) {\n");
        js.push_str("      this.view.setUint32(ptr, 0, true);\n");
        js.push_str("      writeOkFn(this.view, ptr + 4, val.ok);\n");
        js.push_str("    } else {\n");
        js.push_str("      this.view.setUint32(ptr, 1, true);\n");
        js.push_str("      writeErrFn(this.view, ptr + 4, val.err);\n");
        js.push_str("    }\n");
        js.push_str("    return ptr;\n");
        js.push_str("  }\n\n");

        // readResult
        js.push_str("  readResult(ptr, readOkFn, readErrFn) {\n");
        js.push_str("    const tag = this.view.getUint32(ptr, true);\n");
        js.push_str("    if (tag === 0) return { ok: readOkFn(this.view, ptr + 4) };\n");
        js.push_str("    return { err: readErrFn(this.view, ptr + 4) };\n");
        js.push_str("  }\n");

        js.push_str("}\n");

        js
    }

    /// Generate LLVM IR helper functions for WASM type serialization
    /// These are emitted when compiling for a WASM target
    pub fn generate_wasm_serde_ir(&self) -> String {
        let mut ir = String::new();

        ir.push_str("; WASM serialization helpers\n");

        // String layout: [ptr: i32, len: i32]
        ir.push_str("%WasmString = type { i32, i32 }\n");
        // Array layout: [ptr: i32, len: i32]
        ir.push_str("%WasmArray = type { i32, i32 }\n");
        // Option layout: [tag: i32, value: i32]
        ir.push_str("%WasmOption = type { i32, i32 }\n");
        // Result layout: [tag: i32, payload: i32]
        ir.push_str("%WasmResult = type { i32, i32 }\n");

        ir
    }
}

impl Default for WasmSerializer {
    fn default() -> Self {
        Self::new()
    }
}

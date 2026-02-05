//! WebAssembly Component Model support (wasi-preview2)
//!
//! This module provides support for the WebAssembly Component Model, which enables
//! composition and interoperability between WebAssembly modules using WIT (WebAssembly
//! Interface Types).
//!
//! Key features:
//! - WIT type definitions (string, list, record, variant, enum, flags, etc.)
//! - Component interface generation from Vais module signatures
//! - Component linking configuration
//! - wasi-preview2 integration
//!
//! References:
//! - Component Model spec: https://github.com/WebAssembly/component-model
//! - WIT IDL spec: https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md

use std::collections::HashMap;
use std::fmt;

use vais_types::ResolvedType;

/// WIT (WebAssembly Interface Types) representation
#[derive(Debug, Clone, PartialEq)]
pub enum WitType {
    /// Primitive types
    Bool,
    U8,
    U16,
    U32,
    U64,
    S8,
    S16,
    S32,
    S64,
    F32,
    F64,
    Char,
    String,

    /// Container types
    List(Box<WitType>),
    Option_(Box<WitType>),
    Result_ {
        ok: Option<Box<WitType>>,
        err: Option<Box<WitType>>,
    },
    Tuple(Vec<WitType>),

    /// Named types
    Record(String),
    Variant(String),
    Enum(String),
    Flags(String),
    Resource(String),

    /// Custom types
    Named(String),
}

impl fmt::Display for WitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WitType::Bool => write!(f, "bool"),
            WitType::U8 => write!(f, "u8"),
            WitType::U16 => write!(f, "u16"),
            WitType::U32 => write!(f, "u32"),
            WitType::U64 => write!(f, "u64"),
            WitType::S8 => write!(f, "s8"),
            WitType::S16 => write!(f, "s16"),
            WitType::S32 => write!(f, "s32"),
            WitType::S64 => write!(f, "s64"),
            WitType::F32 => write!(f, "f32"),
            WitType::F64 => write!(f, "f64"),
            WitType::Char => write!(f, "char"),
            WitType::String => write!(f, "string"),
            WitType::List(inner) => write!(f, "list<{}>", inner),
            WitType::Option_(inner) => write!(f, "option<{}>", inner),
            WitType::Result_ {
                ok: None,
                err: None,
            } => write!(f, "result"),
            WitType::Result_ {
                ok: Some(ok),
                err: None,
            } => write!(f, "result<{}>", ok),
            WitType::Result_ {
                ok: None,
                err: Some(err),
            } => write!(f, "result<_, {}>", err),
            WitType::Result_ {
                ok: Some(ok),
                err: Some(err),
            } => write!(f, "result<{}, {}>", ok, err),
            WitType::Tuple(types) => {
                write!(f, "tuple<")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ">")
            }
            WitType::Record(name) => write!(f, "{}", name),
            WitType::Variant(name) => write!(f, "{}", name),
            WitType::Enum(name) => write!(f, "{}", name),
            WitType::Flags(name) => write!(f, "{}", name),
            WitType::Resource(name) => write!(f, "{}", name),
            WitType::Named(name) => write!(f, "{}", name),
        }
    }
}

/// WIT record field
#[derive(Debug, Clone)]
pub struct WitField {
    pub name: String,
    pub ty: WitType,
    pub docs: Option<String>,
}

/// WIT record definition
#[derive(Debug, Clone)]
pub struct WitRecord {
    pub name: String,
    pub fields: Vec<WitField>,
    pub docs: Option<String>,
}

/// WIT variant case
#[derive(Debug, Clone)]
pub struct WitVariantCase {
    pub name: String,
    pub ty: Option<WitType>,
    pub docs: Option<String>,
}

/// WIT variant definition
#[derive(Debug, Clone)]
pub struct WitVariant {
    pub name: String,
    pub cases: Vec<WitVariantCase>,
    pub docs: Option<String>,
}

/// WIT enum case
#[derive(Debug, Clone)]
pub struct WitEnumCase {
    pub name: String,
    pub docs: Option<String>,
}

/// WIT enum definition
#[derive(Debug, Clone)]
pub struct WitEnum {
    pub name: String,
    pub cases: Vec<WitEnumCase>,
    pub docs: Option<String>,
}

/// WIT flags definition
#[derive(Debug, Clone)]
pub struct WitFlags {
    pub name: String,
    pub flags: Vec<String>,
    pub docs: Option<String>,
}

/// WIT function parameter
#[derive(Debug, Clone)]
pub struct WitParam {
    pub name: String,
    pub ty: WitType,
}

/// WIT function result
#[derive(Debug, Clone)]
pub enum WitResult {
    Named(Vec<WitParam>),
    Anon(WitType),
}

/// WIT function definition
#[derive(Debug, Clone)]
pub struct WitFunction {
    pub name: String,
    pub params: Vec<WitParam>,
    pub results: Option<WitResult>,
    pub docs: Option<String>,
}

/// WIT resource method
#[derive(Debug, Clone)]
pub struct WitResourceMethod {
    pub name: String,
    pub kind: WitMethodKind,
    pub params: Vec<WitParam>,
    pub results: Option<WitResult>,
    pub docs: Option<String>,
}

/// WIT resource method kind
#[derive(Debug, Clone, PartialEq)]
pub enum WitMethodKind {
    Constructor,
    Static,
    Method,
}

/// WIT resource definition
#[derive(Debug, Clone)]
pub struct WitResource {
    pub name: String,
    pub methods: Vec<WitResourceMethod>,
    pub docs: Option<String>,
}

/// WIT interface definition
#[derive(Debug, Clone)]
pub struct WitInterface {
    pub name: String,
    pub types: Vec<WitTypeDefinition>,
    pub functions: Vec<WitFunction>,
    pub resources: Vec<WitResource>,
    pub docs: Option<String>,
}

/// WIT type definition
#[derive(Debug, Clone)]
pub enum WitTypeDefinition {
    Record(WitRecord),
    Variant(WitVariant),
    Enum(WitEnum),
    Flags(WitFlags),
    Type { name: String, ty: WitType },
}

/// WIT world import
#[derive(Debug, Clone)]
pub struct WitImport {
    pub name: String,
    pub item: WitImportItem,
}

/// WIT world import item
#[derive(Debug, Clone)]
pub enum WitImportItem {
    Interface(String),
    Function(WitFunction),
}

/// WIT world export
#[derive(Debug, Clone)]
pub struct WitExport {
    pub name: String,
    pub item: WitExportItem,
}

/// WIT world export item
#[derive(Debug, Clone)]
pub enum WitExportItem {
    Interface(String),
    Function(WitFunction),
}

/// WIT world definition (top-level component interface)
#[derive(Debug, Clone)]
pub struct WitWorld {
    pub name: String,
    pub imports: Vec<WitImport>,
    pub exports: Vec<WitExport>,
    pub docs: Option<String>,
}

/// WIT package definition
#[derive(Debug, Clone)]
pub struct WitPackage {
    pub namespace: String,
    pub name: String,
    pub version: Option<String>,
    pub interfaces: Vec<WitInterface>,
    pub worlds: Vec<WitWorld>,
    pub docs: Option<String>,
}

impl WitPackage {
    /// Create a new WIT package
    pub fn new(namespace: &str, name: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            name: name.to_string(),
            version: None,
            interfaces: Vec::new(),
            worlds: Vec::new(),
            docs: None,
        }
    }

    /// Set version
    pub fn with_version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    /// Add interface
    pub fn add_interface(&mut self, interface: WitInterface) {
        self.interfaces.push(interface);
    }

    /// Add world
    pub fn add_world(&mut self, world: WitWorld) {
        self.worlds.push(world);
    }

    /// Generate WIT file content
    pub fn to_wit_string(&self) -> String {
        let mut output = String::new();

        // Package declaration
        if let Some(version) = &self.version {
            output.push_str(&format!(
                "package {}:{}@{};\n\n",
                self.namespace, self.name, version
            ));
        } else {
            output.push_str(&format!("package {}:{};\n\n", self.namespace, self.name));
        }

        // Package docs
        if let Some(docs) = &self.docs {
            for line in docs.lines() {
                output.push_str(&format!("/// {}\n", line));
            }
            output.push('\n');
        }

        // Interfaces
        for interface in &self.interfaces {
            output.push_str(&self.format_interface(interface));
            output.push('\n');
        }

        // Worlds
        for world in &self.worlds {
            output.push_str(&self.format_world(world));
            output.push('\n');
        }

        output
    }

    fn format_interface(&self, interface: &WitInterface) -> String {
        let mut output = String::new();

        if let Some(docs) = &interface.docs {
            for line in docs.lines() {
                output.push_str(&format!("/// {}\n", line));
            }
        }

        output.push_str(&format!("interface {} {{\n", interface.name));

        // Type definitions
        for typedef in &interface.types {
            output.push_str(&self.format_type_definition(typedef, 1));
            output.push('\n');
        }

        // Functions
        for function in &interface.functions {
            output.push_str(&self.format_function(function, 1));
            output.push('\n');
        }

        // Resources
        for resource in &interface.resources {
            output.push_str(&self.format_resource(resource, 1));
            output.push('\n');
        }

        output.push_str("}\n");
        output
    }

    fn format_type_definition(&self, typedef: &WitTypeDefinition, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        match typedef {
            WitTypeDefinition::Record(record) => {
                let mut output = String::new();
                if let Some(docs) = &record.docs {
                    for line in docs.lines() {
                        output.push_str(&format!("{}/// {}\n", indent_str, line));
                    }
                }
                output.push_str(&format!("{}record {} {{\n", indent_str, record.name));
                for field in &record.fields {
                    if let Some(docs) = &field.docs {
                        output.push_str(&format!("{}  /// {}\n", indent_str, docs));
                    }
                    output.push_str(&format!("{}  {}: {},\n", indent_str, field.name, field.ty));
                }
                output.push_str(&format!("{}}}\n", indent_str));
                output
            }
            WitTypeDefinition::Variant(variant) => {
                let mut output = String::new();
                if let Some(docs) = &variant.docs {
                    for line in docs.lines() {
                        output.push_str(&format!("{}/// {}\n", indent_str, line));
                    }
                }
                output.push_str(&format!("{}variant {} {{\n", indent_str, variant.name));
                for case in &variant.cases {
                    if let Some(docs) = &case.docs {
                        output.push_str(&format!("{}  /// {}\n", indent_str, docs));
                    }
                    if let Some(ty) = &case.ty {
                        output.push_str(&format!("{}  {}({}),\n", indent_str, case.name, ty));
                    } else {
                        output.push_str(&format!("{}  {},\n", indent_str, case.name));
                    }
                }
                output.push_str(&format!("{}}}\n", indent_str));
                output
            }
            WitTypeDefinition::Enum(enum_def) => {
                let mut output = String::new();
                if let Some(docs) = &enum_def.docs {
                    for line in docs.lines() {
                        output.push_str(&format!("{}/// {}\n", indent_str, line));
                    }
                }
                output.push_str(&format!("{}enum {} {{\n", indent_str, enum_def.name));
                for case in &enum_def.cases {
                    if let Some(docs) = &case.docs {
                        output.push_str(&format!("{}  /// {}\n", indent_str, docs));
                    }
                    output.push_str(&format!("{}  {},\n", indent_str, case.name));
                }
                output.push_str(&format!("{}}}\n", indent_str));
                output
            }
            WitTypeDefinition::Flags(flags) => {
                let mut output = String::new();
                if let Some(docs) = &flags.docs {
                    for line in docs.lines() {
                        output.push_str(&format!("{}/// {}\n", indent_str, line));
                    }
                }
                output.push_str(&format!("{}flags {} {{\n", indent_str, flags.name));
                for flag in &flags.flags {
                    output.push_str(&format!("{}  {},\n", indent_str, flag));
                }
                output.push_str(&format!("{}}}\n", indent_str));
                output
            }
            WitTypeDefinition::Type { name, ty } => {
                format!("{}type {} = {};\n", indent_str, name, ty)
            }
        }
    }

    fn format_function(&self, function: &WitFunction, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        let mut output = String::new();

        if let Some(docs) = &function.docs {
            for line in docs.lines() {
                output.push_str(&format!("{}/// {}\n", indent_str, line));
            }
        }

        output.push_str(&format!("{}{}: func(", indent_str, function.name));

        // Parameters
        for (i, param) in function.params.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            output.push_str(&format!("{}: {}", param.name, param.ty));
        }

        output.push(')');

        // Results
        if let Some(results) = &function.results {
            match results {
                WitResult::Anon(ty) => {
                    output.push_str(&format!(" -> {}", ty));
                }
                WitResult::Named(params) => {
                    output.push_str(" -> (");
                    for (i, param) in params.iter().enumerate() {
                        if i > 0 {
                            output.push_str(", ");
                        }
                        output.push_str(&format!("{}: {}", param.name, param.ty));
                    }
                    output.push(')');
                }
            }
        }

        output.push_str(";\n");
        output
    }

    fn format_resource(&self, resource: &WitResource, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        let mut output = String::new();

        if let Some(docs) = &resource.docs {
            for line in docs.lines() {
                output.push_str(&format!("{}/// {}\n", indent_str, line));
            }
        }

        output.push_str(&format!("{}resource {} {{\n", indent_str, resource.name));

        for method in &resource.methods {
            if let Some(docs) = &method.docs {
                output.push_str(&format!("{}  /// {}\n", indent_str, docs));
            }

            match method.kind {
                WitMethodKind::Constructor => {
                    output.push_str(&format!("{}  constructor(", indent_str));
                }
                WitMethodKind::Static => {
                    output.push_str(&format!("{}  {}: static func(", indent_str, method.name));
                }
                WitMethodKind::Method => {
                    output.push_str(&format!("{}  {}: func(", indent_str, method.name));
                }
            }

            // Parameters
            for (i, param) in method.params.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&format!("{}: {}", param.name, param.ty));
            }

            output.push(')');

            // Results
            if let Some(results) = &method.results {
                match results {
                    WitResult::Anon(ty) => {
                        output.push_str(&format!(" -> {}", ty));
                    }
                    WitResult::Named(params) => {
                        output.push_str(" -> (");
                        for (i, param) in params.iter().enumerate() {
                            if i > 0 {
                                output.push_str(", ");
                            }
                            output.push_str(&format!("{}: {}", param.name, param.ty));
                        }
                        output.push(')');
                    }
                }
            }

            output.push_str(";\n");
        }

        output.push_str(&format!("{}}}\n", indent_str));
        output
    }

    fn format_world(&self, world: &WitWorld) -> String {
        let mut output = String::new();

        if let Some(docs) = &world.docs {
            for line in docs.lines() {
                output.push_str(&format!("/// {}\n", line));
            }
        }

        output.push_str(&format!("world {} {{\n", world.name));

        // Imports
        for import in &world.imports {
            match &import.item {
                WitImportItem::Interface(name) => {
                    output.push_str(&format!("  import {};\n", name));
                }
                WitImportItem::Function(func) => {
                    output.push_str(&format!(
                        "  import {};\n",
                        self.format_function(func, 1).trim()
                    ));
                }
            }
        }

        // Exports
        for export in &world.exports {
            match &export.item {
                WitExportItem::Interface(name) => {
                    output.push_str(&format!("  export {};\n", name));
                }
                WitExportItem::Function(func) => {
                    output.push_str(&format!(
                        "  export {};\n",
                        self.format_function(func, 1).trim()
                    ));
                }
            }
        }

        output.push_str("}\n");
        output
    }
}

/// Convert Vais type to WIT type
pub fn vais_type_to_wit(ty: &ResolvedType) -> Option<WitType> {
    match ty {
        ResolvedType::Bool => Some(WitType::Bool),
        ResolvedType::U8 => Some(WitType::U8),
        ResolvedType::U16 => Some(WitType::U16),
        ResolvedType::U32 => Some(WitType::U32),
        ResolvedType::U64 => Some(WitType::U64),
        ResolvedType::I8 => Some(WitType::S8),
        ResolvedType::I16 => Some(WitType::S16),
        ResolvedType::I32 => Some(WitType::S32),
        ResolvedType::I64 => Some(WitType::S64),
        ResolvedType::F32 => Some(WitType::F32),
        ResolvedType::F64 => Some(WitType::F64),
        // Vais uses Str for string type
        ResolvedType::Str => Some(WitType::String),
        ResolvedType::Array(inner) => {
            let inner_wit = vais_type_to_wit(inner)?;
            Some(WitType::List(Box::new(inner_wit)))
        }
        ResolvedType::ConstArray { element, .. } => {
            // WIT doesn't have const-sized arrays, map to list
            let inner_wit = vais_type_to_wit(element)?;
            Some(WitType::List(Box::new(inner_wit)))
        }
        ResolvedType::Optional(inner) => {
            let inner_wit = vais_type_to_wit(inner)?;
            Some(WitType::Option_(Box::new(inner_wit)))
        }
        ResolvedType::Result(inner) => {
            // Vais Result is Result<T>, map to result<T, _>
            let ok_wit = vais_type_to_wit(inner)?;
            Some(WitType::Result_ {
                ok: Some(Box::new(ok_wit)),
                err: None,
            })
        }
        ResolvedType::Tuple(types) => {
            let wit_types: Option<Vec<_>> = types.iter().map(vais_type_to_wit).collect();
            wit_types.map(WitType::Tuple)
        }
        ResolvedType::Named { name, .. } => Some(WitType::Named(name.clone())),
        _ => None, // Other types (pointers, refs, functions, etc.) not directly mappable to WIT
    }
}

/// Component linking configuration for wasi-preview2
#[derive(Debug, Clone)]
pub struct ComponentLinkConfig {
    /// Component model adapter module (for core -> component)
    pub adapter_module: Option<String>,
    /// Enable reactor mode (no start function)
    pub reactor_mode: bool,
    /// Enable command mode (with _start)
    pub command_mode: bool,
    /// Additional component-level imports
    pub component_imports: HashMap<String, String>,
    /// Additional component-level exports
    pub component_exports: HashMap<String, String>,
}

impl Default for ComponentLinkConfig {
    fn default() -> Self {
        Self {
            adapter_module: None,
            reactor_mode: false,
            command_mode: true,
            component_imports: HashMap::new(),
            component_exports: HashMap::new(),
        }
    }
}

impl ComponentLinkConfig {
    /// Create a new component link configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable reactor mode (disable command mode)
    pub fn reactor(mut self) -> Self {
        self.reactor_mode = true;
        self.command_mode = false;
        self
    }

    /// Enable command mode (disable reactor mode)
    pub fn command(mut self) -> Self {
        self.reactor_mode = false;
        self.command_mode = true;
        self
    }

    /// Set adapter module path
    pub fn with_adapter(mut self, path: &str) -> Self {
        self.adapter_module = Some(path.to_string());
        self
    }

    /// Add component import
    pub fn add_import(&mut self, name: String, interface: String) {
        self.component_imports.insert(name, interface);
    }

    /// Add component export
    pub fn add_export(&mut self, name: String, interface: String) {
        self.component_exports.insert(name, interface);
    }

    /// Generate wasm-tools component-link flags
    pub fn to_link_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(adapter) = &self.adapter_module {
            args.push("--adapt".to_string());
            args.push(adapter.clone());
        }

        for (name, interface) in &self.component_imports {
            args.push("--import".to_string());
            args.push(format!("{}={}", name, interface));
        }

        for (name, interface) in &self.component_exports {
            args.push("--export".to_string());
            args.push(format!("{}={}", name, interface));
        }

        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

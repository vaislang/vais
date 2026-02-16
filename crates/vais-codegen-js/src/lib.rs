#![allow(dead_code)] // JS codegen metadata structs used internally
//! Vais JavaScript Code Generator
//!
//! Generates JavaScript (ESM) from typed Vais AST.
//!
//! # Pipeline
//!
//! ```text
//! .vais source → Lexer → Parser → AST → Type Checker → JsCodeGenerator → .js (ESM)
//! ```

mod expr;
mod items;
mod modules;
mod sourcemap;
mod stmt;
pub mod tree_shaking;
mod types;

pub use sourcemap::SourceMap;
pub use types::JsType;

use std::collections::HashMap;
use thiserror::Error;
use vais_ast::*;

#[derive(Debug, Error)]
pub enum JsCodegenError {
    #[error("Unsupported feature for JS target: {0}")]
    UnsupportedFeature(String),

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, JsCodegenError>;

/// Configuration for JS code generation
#[derive(Debug, Clone)]
pub struct JsConfig {
    /// Use `const`/`let` (true) or `var` (false)
    pub use_const_let: bool,
    /// Generate TypeScript-style JSDoc comments
    pub emit_jsdoc: bool,
    /// Indentation string (default: 2 spaces)
    pub indent: String,
    /// Target ES version (default: "es2020")
    pub target: String,
}

impl Default for JsConfig {
    fn default() -> Self {
        Self {
            use_const_let: true,
            emit_jsdoc: false,
            indent: "  ".to_string(),
            target: "es2020".to_string(),
        }
    }
}

/// JavaScript code generator for Vais AST
pub struct JsCodeGenerator {
    /// Configuration
    pub config: JsConfig,
    /// Current indentation level
    indent_level: usize,
    /// Registered struct definitions (name → fields)
    structs: HashMap<String, Vec<(String, String)>>,
    /// Registered enum definitions (name → variants)
    enums: HashMap<String, Vec<EnumVariantInfo>>,
    /// Trait definitions (name → methods)
    traits: HashMap<String, Vec<TraitMethodInfo>>,
    /// Impl blocks (type_name → trait_name → methods)
    impls: HashMap<String, Vec<ImplInfo>>,
    /// Generated helper functions
    helpers: Vec<String>,
    /// Current function name (for self-recursion @)
    current_function: Option<String>,
    /// Label counter for unique names
    label_counter: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct EnumVariantInfo {
    pub name: String,
    pub fields: VariantFieldsInfo,
}

#[derive(Debug, Clone)]
pub(crate) enum VariantFieldsInfo {
    Unit,
    Tuple(usize),
    Struct(Vec<String>),
}

#[derive(Debug, Clone)]
pub(crate) struct TraitMethodInfo {
    pub name: String,
    pub params: Vec<String>,
    pub has_default: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ImplInfo {
    pub trait_name: Option<String>,
    pub methods: Vec<(String, String)>,
}

impl JsCodeGenerator {
    /// Create a new JS code generator with default config
    pub fn new() -> Self {
        Self {
            config: JsConfig::default(),
            indent_level: 0,
            structs: HashMap::new(),
            enums: HashMap::new(),
            traits: HashMap::new(),
            impls: HashMap::new(),
            helpers: Vec::new(),
            current_function: None,
            label_counter: 0,
        }
    }

    /// Create a new JS code generator with custom config
    pub fn with_config(config: JsConfig) -> Self {
        Self {
            config,
            indent_level: 0,
            structs: HashMap::new(),
            enums: HashMap::new(),
            traits: HashMap::new(),
            impls: HashMap::new(),
            helpers: Vec::new(),
            current_function: None,
            label_counter: 0,
        }
    }

    /// Generate JavaScript from a Vais module
    pub fn generate_module(&mut self, module: &Module) -> Result<String> {
        let mut output = String::new();

        // Pass 1: Register all types (structs, enums, traits)
        for item in &module.items {
            self.register_item(&item.node)?;
        }

        // Pass 2: Generate code for each item
        for item in &module.items {
            let js = self.generate_item(&item.node)?;
            if !js.is_empty() {
                output.push_str(&js);
                output.push('\n');
            }
        }

        // Append helper functions if any
        if !self.helpers.is_empty() {
            output.push('\n');
            for helper in &self.helpers {
                output.push_str(helper);
                output.push('\n');
            }
        }

        Ok(output)
    }

    /// Register item for forward reference (pass 1)
    fn register_item(&mut self, item: &Item) -> Result<()> {
        match item {
            Item::Struct(s) => {
                let fields: Vec<(String, String)> = s
                    .fields
                    .iter()
                    .map(|f| (f.name.node.clone(), types::type_to_js(&f.ty.node)))
                    .collect();
                self.structs.insert(s.name.node.clone(), fields);
            }
            Item::Enum(e) => {
                let variants: Vec<EnumVariantInfo> = e
                    .variants
                    .iter()
                    .map(|v| EnumVariantInfo {
                        name: v.name.node.clone(),
                        fields: match &v.fields {
                            VariantFields::Unit => VariantFieldsInfo::Unit,
                            VariantFields::Tuple(types) => VariantFieldsInfo::Tuple(types.len()),
                            VariantFields::Struct(fields) => VariantFieldsInfo::Struct(
                                fields.iter().map(|f| f.name.node.clone()).collect(),
                            ),
                        },
                    })
                    .collect();
                self.enums.insert(e.name.node.clone(), variants);
            }
            Item::Trait(t) => {
                let methods: Vec<TraitMethodInfo> = t
                    .methods
                    .iter()
                    .map(|m| TraitMethodInfo {
                        name: m.name.node.clone(),
                        params: m.params.iter().map(|p| p.name.node.clone()).collect(),
                        has_default: m.default_body.is_some(),
                    })
                    .collect();
                self.traits.insert(t.name.node.clone(), methods);
            }
            _ => {}
        }
        Ok(())
    }

    // --- Indentation helpers ---

    fn indent(&self) -> String {
        self.config.indent.repeat(self.indent_level)
    }

    fn next_label(&mut self) -> String {
        self.label_counter += 1;
        format!("_L{}", self.label_counter)
    }
}

impl Default for JsCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_module() {
        let mut gen = JsCodeGenerator::new();
        let module = Module {
            items: vec![],
            modules_map: None,
        };
        let result = gen.generate_module(&module).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_simple_function() {
        let mut gen = JsCodeGenerator::new();
        let module = Module {
            items: vec![Spanned::new(
                Item::Function(Function {
                    name: Spanned::new("add".to_string(), Span::new(0, 3)),
                    generics: vec![],
                    params: vec![
                        Param {
                            name: Spanned::new("a".to_string(), Span::new(4, 5)),
                            ty: Spanned::new(
                                Type::Named {
                                    name: "i64".to_string(),
                                    generics: vec![],
                                },
                                Span::new(7, 10),
                            ),
                            is_mut: false,
                            is_vararg: false,
                            ownership: Ownership::Regular,
                            default_value: None,
                        },
                        Param {
                            name: Spanned::new("b".to_string(), Span::new(12, 13)),
                            ty: Spanned::new(
                                Type::Named {
                                    name: "i64".to_string(),
                                    generics: vec![],
                                },
                                Span::new(15, 18),
                            ),
                            is_mut: false,
                            is_vararg: false,
                            ownership: Ownership::Regular,
                            default_value: None,
                        },
                    ],
                    ret_type: Some(Spanned::new(
                        Type::Named {
                            name: "i64".to_string(),
                            generics: vec![],
                        },
                        Span::new(23, 26),
                    )),
                    body: FunctionBody::Expr(Box::new(Spanned::new(
                        Expr::Binary {
                            op: BinOp::Add,
                            left: Box::new(Spanned::new(
                                Expr::Ident("a".to_string()),
                                Span::new(27, 28),
                            )),
                            right: Box::new(Spanned::new(
                                Expr::Ident("b".to_string()),
                                Span::new(31, 32),
                            )),
                        },
                        Span::new(27, 32),
                    ))),
                    is_pub: false,
                    is_async: false,
                    attributes: vec![],
                    where_clause: vec![],
                }),
                Span::new(0, 32),
            )],
            modules_map: None,
        };
        let result = gen.generate_module(&module).unwrap();
        assert!(result.contains("function add(a, b)"));
        assert!(result.contains("return (a + b)"));
    }

    #[test]
    fn test_struct_as_class() {
        let mut gen = JsCodeGenerator::new();
        let module = Module {
            items: vec![Spanned::new(
                Item::Struct(Struct {
                    name: Spanned::new("Point".to_string(), Span::new(0, 5)),
                    generics: vec![],
                    fields: vec![
                        Field {
                            name: Spanned::new("x".to_string(), Span::new(6, 7)),
                            ty: Spanned::new(
                                Type::Named {
                                    name: "f64".to_string(),
                                    generics: vec![],
                                },
                                Span::new(9, 12),
                            ),
                            is_pub: true,
                        },
                        Field {
                            name: Spanned::new("y".to_string(), Span::new(14, 15)),
                            ty: Spanned::new(
                                Type::Named {
                                    name: "f64".to_string(),
                                    generics: vec![],
                                },
                                Span::new(17, 20),
                            ),
                            is_pub: true,
                        },
                    ],
                    methods: vec![],
                    is_pub: false,
                    attributes: vec![],
                    where_clause: vec![],
                }),
                Span::new(0, 20),
            )],
            modules_map: None,
        };
        let result = gen.generate_module(&module).unwrap();
        assert!(result.contains("class Point"));
        assert!(result.contains("constructor(x, y)"));
    }
}

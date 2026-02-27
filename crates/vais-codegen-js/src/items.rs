//! Top-level item code generation: Vais Item → JavaScript declarations

use crate::expr::sanitize_js_ident;
use crate::{ImplInfo, JsCodeGenerator, Result};
use vais_ast::*;

impl JsCodeGenerator {
    /// Generate JavaScript for a top-level item
    pub(crate) fn generate_item(&mut self, item: &Item) -> Result<String> {
        match item {
            Item::Function(func) => self.generate_function(func, false),
            Item::Struct(s) => self.generate_struct(s),
            Item::Enum(e) => self.generate_enum(e),
            Item::Impl(imp) => self.generate_impl(imp),
            Item::Trait(t) => self.generate_trait(t),
            Item::Const(c) => self.generate_const(c),
            Item::Global(g) => self.generate_global(g),
            Item::TypeAlias(_) | Item::TraitAlias(_) => Ok(String::new()), // No runtime representation
            Item::Use(u) => self.generate_use(u),                          // Generate ESM import
            Item::ExternBlock(ext) => self.generate_extern_block(ext),
            Item::Macro(_) => Ok(String::new()), // Macros are expanded during parsing
            Item::Union(u) => self.generate_union(u),
            Item::Error { message, .. } => Ok(format!("/* error: {message} */")),
        }
    }

    /// Generate a function declaration
    pub(crate) fn generate_function(&mut self, func: &Function, is_method: bool) -> Result<String> {
        let name = sanitize_js_ident(&func.name.node);
        let prev_func = self.current_function.take();
        self.current_function = Some(name.clone());

        let params: Vec<String> = func
            .params
            .iter()
            .map(|p| {
                let pname = sanitize_js_ident(&p.name.node);
                if let Some(default) = &p.default_value {
                    // Generate default value inline
                    let mut gen = JsCodeGenerator::new();
                    let val = gen.generate_expr(&default.node).unwrap_or_default();
                    format!("{pname} = {val}")
                } else {
                    pname
                }
            })
            .collect();

        let async_prefix = if func.is_async { "async " } else { "" };
        let export_prefix = if func.is_pub && !is_method {
            "export "
        } else {
            ""
        };

        // Generate generic annotation comment if present
        let generic_comment = if !func.generics.is_empty() {
            let generic_names: Vec<String> =
                func.generics.iter().map(|g| g.name.node.clone()).collect();
            format!(" /* <{}> */", generic_names.join(", "))
        } else {
            String::new()
        };

        let indent = self.indent();
        let mut output = format!(
            "{indent}{export_prefix}{async_prefix}function {name}({}){}  {{\n",
            params.join(", "),
            generic_comment
        );

        self.indent_level += 1;
        let body = self.generate_function_body(&func.body)?;
        output.push_str(&body);
        output.push('\n');
        self.indent_level -= 1;
        output.push_str(&format!("{indent}}}\n"));

        self.current_function = prev_func;
        Ok(output)
    }

    /// Generate a struct as a JavaScript class
    fn generate_struct(&mut self, s: &Struct) -> Result<String> {
        let name = sanitize_js_ident(&s.name.node);
        let export_prefix = if s.is_pub { "export " } else { "" };
        let indent = self.indent();

        // Generate generic annotation comment if present
        let generic_comment = if !s.generics.is_empty() {
            let generic_names: Vec<String> =
                s.generics.iter().map(|g| g.name.node.clone()).collect();
            format!(" /* <{}> */", generic_names.join(", "))
        } else {
            String::new()
        };

        let mut output = format!(
            "{indent}{export_prefix}class {name}{} {{\n",
            generic_comment
        );
        self.indent_level += 1;
        let inner = self.indent();

        // Constructor
        let field_names: Vec<String> = s
            .fields
            .iter()
            .map(|f| sanitize_js_ident(&f.name.node))
            .collect();

        // Support both positional args and object arg
        output.push_str(&format!(
            "{inner}constructor({}) {{\n",
            field_names
                .iter()
                .map(|n| n.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
        self.indent_level += 1;
        let body_indent = self.indent();
        for (i, f) in s.fields.iter().enumerate() {
            let fname = sanitize_js_ident(&f.name.node);
            // Check if first arg is an object (from StructLit)
            if i == 0 && s.fields.len() > 1 {
                output.push_str(&format!(
                    "{body_indent}if (typeof {fname} === 'object' && {fname} !== null && !Array.isArray({fname}) && arguments.length === 1) {{\n"
                ));
                self.indent_level += 1;
                let obj_indent = self.indent();
                output.push_str(&format!("{obj_indent}const __obj = {fname};\n"));
                for f2 in &s.fields {
                    let fn2 = sanitize_js_ident(&f2.name.node);
                    output.push_str(&format!("{obj_indent}this.{fn2} = __obj.{fn2};\n"));
                }
                output.push_str(&format!("{obj_indent}return;\n"));
                self.indent_level -= 1;
                output.push_str(&format!("{body_indent}}}\n"));
            }
            output.push_str(&format!("{body_indent}this.{fname} = {fname};\n"));
        }
        self.indent_level -= 1;
        output.push_str(&format!("{inner}}}\n"));

        // Methods
        for method in &s.methods {
            let m = self.generate_method(&method.node)?;
            output.push_str(&m);
        }

        self.indent_level -= 1;
        output.push_str(&format!("{indent}}}\n"));
        Ok(output)
    }

    /// Generate an enum as tagged union factory functions
    fn generate_enum(&mut self, e: &Enum) -> Result<String> {
        let name = sanitize_js_ident(&e.name.node);
        let export_prefix = if e.is_pub { "export " } else { "" };
        let indent = self.indent();

        let mut output = format!("{indent}{export_prefix}const {name} = Object.freeze({{\n");
        self.indent_level += 1;
        let inner = self.indent();

        for variant in &e.variants {
            let vname = &variant.name.node;
            match &variant.fields {
                VariantFields::Unit => {
                    output.push_str(&format!(
                        "{inner}{vname}: Object.freeze({{ __tag: \"{vname}\", __data: [] }}),\n"
                    ));
                }
                VariantFields::Tuple(types) => {
                    let params: Vec<String> = (0..types.len()).map(|i| format!("__{i}")).collect();
                    output.push_str(&format!(
                        "{inner}{vname}({}) {{ return {{ __tag: \"{vname}\", __data: [{}] }}; }},\n",
                        params.join(", "),
                        params.join(", ")
                    ));
                }
                VariantFields::Struct(fields) => {
                    let params: Vec<String> = fields
                        .iter()
                        .map(|f| sanitize_js_ident(&f.name.node))
                        .collect();
                    output.push_str(&format!(
                        "{inner}{vname}({}) {{ return {{ __tag: \"{vname}\", __data: [{}] }}; }},\n",
                        params.join(", "),
                        params.join(", ")
                    ));
                }
            }
        }

        self.indent_level -= 1;
        output.push_str(&format!("{indent}}});\n"));

        // Add helper methods for Result<T,E> and Option<T> enums
        if name == "Result" || name == "Option" {
            self.generate_enum_helpers(&name, &mut output)?;
        }

        Ok(output)
    }

    /// Generate helper methods for Result/Option enums
    fn generate_enum_helpers(&mut self, enum_name: &str, output: &mut String) -> Result<()> {
        let indent = self.indent();

        if enum_name == "Result" {
            // is_Ok, is_Err
            output.push_str(&format!(
                "{indent}{enum_name}.is_Ok = function(val) {{ return val.__tag === \"Ok\"; }};\n"
            ));
            output.push_str(&format!(
                "{indent}{enum_name}.is_Err = function(val) {{ return val.__tag === \"Err\"; }};\n"
            ));

            // unwrap
            output.push_str(&format!("{indent}{enum_name}.unwrap = function(val) {{\n"));
            self.indent_level += 1;
            let inner = self.indent();
            output.push_str(&format!("{inner}if (val.__tag === \"Err\") throw new Error(\"Called unwrap on Err: \" + JSON.stringify(val.__data[0]));\n"));
            output.push_str(&format!("{inner}return val.__data[0];\n"));
            self.indent_level -= 1;
            output.push_str(&format!("{indent}}};\n"));

            // unwrap_or
            output.push_str(&format!(
                "{indent}{enum_name}.unwrap_or = function(val, defaultValue) {{\n"
            ));
            self.indent_level += 1;
            let inner = self.indent();
            output.push_str(&format!(
                "{inner}return val.__tag === \"Ok\" ? val.__data[0] : defaultValue;\n"
            ));
            self.indent_level -= 1;
            output.push_str(&format!("{indent}}};\n"));

            // map
            output.push_str(&format!("{indent}{enum_name}.map = function(val, fn) {{\n"));
            self.indent_level += 1;
            let inner = self.indent();
            output.push_str(&format!("{inner}if (val.__tag === \"Ok\") return {{ __tag: \"Ok\", __data: [fn(val.__data[0])] }};\n"));
            output.push_str(&format!("{inner}return val;\n"));
            self.indent_level -= 1;
            output.push_str(&format!("{indent}}};\n"));
        } else if enum_name == "Option" {
            // is_Some, is_None
            output.push_str(&format!("{indent}{enum_name}.is_Some = function(val) {{ return val.__tag === \"Some\"; }};\n"));
            output.push_str(&format!("{indent}{enum_name}.is_None = function(val) {{ return val.__tag === \"None\"; }};\n"));

            // unwrap
            output.push_str(&format!("{indent}{enum_name}.unwrap = function(val) {{\n"));
            self.indent_level += 1;
            let inner = self.indent();
            output.push_str(&format!(
                "{inner}if (val.__tag === \"None\") throw new Error(\"Called unwrap on None\");\n"
            ));
            output.push_str(&format!("{inner}return val.__data[0];\n"));
            self.indent_level -= 1;
            output.push_str(&format!("{indent}}};\n"));

            // unwrap_or
            output.push_str(&format!(
                "{indent}{enum_name}.unwrap_or = function(val, defaultValue) {{\n"
            ));
            self.indent_level += 1;
            let inner = self.indent();
            output.push_str(&format!(
                "{inner}return val.__tag === \"Some\" ? val.__data[0] : defaultValue;\n"
            ));
            self.indent_level -= 1;
            output.push_str(&format!("{indent}}};\n"));

            // map
            output.push_str(&format!("{indent}{enum_name}.map = function(val, fn) {{\n"));
            self.indent_level += 1;
            let inner = self.indent();
            output.push_str(&format!("{inner}if (val.__tag === \"Some\") return {{ __tag: \"Some\", __data: [fn(val.__data[0])] }};\n"));
            output.push_str(&format!("{inner}return val;\n"));
            self.indent_level -= 1;
            output.push_str(&format!("{indent}}};\n"));
        }

        Ok(())
    }

    /// Generate impl block: attach methods to class prototype
    fn generate_impl(&mut self, imp: &Impl) -> Result<String> {
        let type_name = match &imp.target_type.node {
            Type::Named { name, .. } => sanitize_js_ident(name),
            _ => return Ok(String::new()),
        };

        let indent = self.indent();
        let mut output = String::new();
        let mut method_entries = Vec::new();

        // If implementing a trait, add trait tracking
        if let Some(ref trait_name) = imp.trait_name {
            let trait_name_sanitized = sanitize_js_ident(&trait_name.node);

            // Initialize __implements set if not exists
            output.push_str(&format!("{indent}if (!{type_name}.__implements) {{\n"));
            self.indent_level += 1;
            let inner = self.indent();
            output.push_str(&format!("{inner}{type_name}.__implements = new Set();\n"));
            output.push_str(&format!(
                "{inner}{type_name}.__implementsTrait = function(traitName) {{ return this.__implements.has(traitName); }};\n"
            ));
            self.indent_level -= 1;
            output.push_str(&format!("{indent}}}\n"));

            // Add trait name to the set
            output.push_str(&format!(
                "{indent}{type_name}.__implements.add(\"{trait_name_sanitized}\");\n"
            ));
        }

        for method in &imp.methods {
            let mname = sanitize_js_ident(&method.node.name.node);
            let params: Vec<String> = method
                .node
                .params
                .iter()
                .filter(|p| p.name.node != "self")
                .map(|p| sanitize_js_ident(&p.name.node))
                .collect();

            let has_self = method.node.params.iter().any(|p| p.name.node == "self");
            let async_prefix = if method.node.is_async { "async " } else { "" };

            if has_self {
                // Instance method → prototype
                output.push_str(&format!(
                    "{indent}{type_name}.prototype.{mname} = {async_prefix}function({}) {{\n",
                    params.join(", ")
                ));
            } else {
                // Static method
                output.push_str(&format!(
                    "{indent}{type_name}.{mname} = {async_prefix}function({}) {{\n",
                    params.join(", ")
                ));
            }

            self.indent_level += 1;
            // In instance methods, `self` → `this`
            let body = self.generate_function_body_with_self(&method.node.body, has_self)?;
            output.push_str(&body);
            output.push('\n');
            self.indent_level -= 1;
            output.push_str(&format!("{indent}}};\n"));

            method_entries.push((mname, String::new()));
        }

        // Track impl info
        self.impls.entry(type_name).or_default().push(ImplInfo {
            trait_name: imp.trait_name.as_ref().map(|t| t.node.clone()),
            methods: method_entries,
        });

        Ok(output)
    }

    /// Generate function body, replacing 'self' with 'this' for instance methods
    fn generate_function_body_with_self(
        &mut self,
        body: &FunctionBody,
        _has_self: bool,
    ) -> Result<String> {
        // For now, the expression codegen handles `self` → `this` at the ident level
        // (Vais uses `self` in method bodies, which we can translate)
        self.generate_function_body(body)
    }

    /// Generate a method inside a class body
    fn generate_method(&mut self, func: &Function) -> Result<String> {
        let name = sanitize_js_ident(&func.name.node);
        let params: Vec<String> = func
            .params
            .iter()
            .filter(|p| p.name.node != "self")
            .map(|p| sanitize_js_ident(&p.name.node))
            .collect();

        let indent = self.indent();
        let async_prefix = if func.is_async { "async " } else { "" };

        let mut output = format!("{indent}{async_prefix}{name}({}) {{\n", params.join(", "));
        self.indent_level += 1;
        let body = self.generate_function_body(&func.body)?;
        output.push_str(&body);
        output.push('\n');
        self.indent_level -= 1;
        output.push_str(&format!("{indent}}}\n"));
        Ok(output)
    }

    /// Generate a trait as a base class with default methods
    fn generate_trait(&mut self, t: &Trait) -> Result<String> {
        let name = sanitize_js_ident(&t.name.node);
        let export_prefix = if t.is_pub { "export " } else { "" };
        let indent = self.indent();

        let mut output = format!("{indent}{export_prefix}class {name} {{\n");
        self.indent_level += 1;
        let inner = self.indent();

        // Default methods
        for method in &t.methods {
            if let Some(ref body) = method.default_body {
                let mname = sanitize_js_ident(&method.name.node);
                let params: Vec<String> = method
                    .params
                    .iter()
                    .filter(|p| p.name.node != "self")
                    .map(|p| sanitize_js_ident(&p.name.node))
                    .collect();

                let async_prefix = if method.is_async { "async " } else { "" };
                output.push_str(&format!(
                    "{inner}{async_prefix}{mname}({}) {{\n",
                    params.join(", ")
                ));
                self.indent_level += 1;
                let body_js = self.generate_function_body(body)?;
                output.push_str(&body_js);
                output.push('\n');
                self.indent_level -= 1;
                output.push_str(&format!("{inner}}}\n"));
            } else {
                // Abstract method: throw
                let mname = sanitize_js_ident(&method.name.node);
                let params: Vec<String> = method
                    .params
                    .iter()
                    .filter(|p| p.name.node != "self")
                    .map(|p| sanitize_js_ident(&p.name.node))
                    .collect();
                output.push_str(&format!(
                    "{inner}{mname}({}) {{ throw new Error(\"{name}.{mname} not implemented\"); }}\n",
                    params.join(", ")
                ));
            }
        }

        self.indent_level -= 1;
        output.push_str(&format!("{indent}}}\n"));
        Ok(output)
    }

    /// Generate const declaration
    fn generate_const(&mut self, c: &ConstDef) -> Result<String> {
        let name = sanitize_js_ident(&c.name.node);
        let val = self.generate_expr(&c.value.node)?;
        let export_prefix = if c.is_pub { "export " } else { "" };
        Ok(format!("{export_prefix}const {name} = {val};\n"))
    }

    /// Generate global variable declaration
    fn generate_global(&mut self, g: &GlobalDef) -> Result<String> {
        let name = sanitize_js_ident(&g.name.node);
        let val = self.generate_expr(&g.value.node)?;
        let export_prefix = if g.is_pub { "export " } else { "" };
        Ok(format!("{export_prefix}let {name} = {val};\n"))
    }

    /// Generate extern block (stubs)
    fn generate_extern_block(&mut self, ext: &ExternBlock) -> Result<String> {
        let indent = self.indent();
        let mut output = String::new();
        for func in &ext.functions {
            let name = sanitize_js_ident(&func.name.node);
            output.push_str(&format!(
                "{indent}/* extern: {name} - must be provided at runtime */\n"
            ));
        }
        Ok(output)
    }

    /// Generate union as a plain object (C-style, no tag)
    fn generate_union(&mut self, u: &Union) -> Result<String> {
        let name = sanitize_js_ident(&u.name.node);
        let export_prefix = if u.is_pub { "export " } else { "" };
        let indent = self.indent();

        let field_names: Vec<String> = u
            .fields
            .iter()
            .map(|f| sanitize_js_ident(&f.name.node))
            .collect();

        let mut output = format!("{indent}{export_prefix}class {name} {{\n");
        self.indent_level += 1;
        let inner = self.indent();
        output.push_str(&format!(
            "{inner}constructor(value) {{ this._value = value; }}\n"
        ));
        for fname in &field_names {
            output.push_str(&format!("{inner}get {fname}() {{ return this._value; }}\n"));
            output.push_str(&format!("{inner}set {fname}(v) {{ this._value = v; }}\n"));
        }
        self.indent_level -= 1;
        output.push_str(&format!("{indent}}}\n"));
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_generation() {
        let mut gen = JsCodeGenerator::new();
        let c = ConstDef {
            name: Spanned::new("MAX_SIZE".to_string(), Span::new(0, 8)),
            ty: Spanned::new(
                Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                Span::new(10, 13),
            ),
            value: Spanned::new(Expr::Int(1024), Span::new(16, 20)),
            is_pub: true,
            attributes: vec![],
        };
        let result = gen.generate_const(&c).unwrap();
        assert_eq!(result, "export const MAX_SIZE = 1024;\n");
    }

    #[test]
    fn test_enum_generation() {
        let mut gen = JsCodeGenerator::new();
        let e = Enum {
            name: Spanned::new("Color".to_string(), Span::new(0, 5)),
            generics: vec![],
            variants: vec![
                Variant {
                    name: Spanned::new("Red".to_string(), Span::new(6, 9)),
                    fields: VariantFields::Unit,
                },
                Variant {
                    name: Spanned::new("Rgb".to_string(), Span::new(11, 14)),
                    fields: VariantFields::Tuple(vec![
                        Spanned::new(
                            Type::Named {
                                name: "i64".to_string(),
                                generics: vec![],
                            },
                            Span::new(15, 18),
                        ),
                        Spanned::new(
                            Type::Named {
                                name: "i64".to_string(),
                                generics: vec![],
                            },
                            Span::new(20, 23),
                        ),
                        Spanned::new(
                            Type::Named {
                                name: "i64".to_string(),
                                generics: vec![],
                            },
                            Span::new(25, 28),
                        ),
                    ]),
                },
            ],
            is_pub: false,
            attributes: vec![],
        };
        let result = gen.generate_enum(&e).unwrap();
        assert!(result.contains("const Color = Object.freeze"));
        assert!(result.contains("Red: Object.freeze"));
        assert!(result.contains("Rgb(__0, __1, __2)"));
    }

    #[test]
    fn test_result_enum_helpers() {
        let mut gen = JsCodeGenerator::new();
        let e = Enum {
            name: Spanned::new("Result".to_string(), Span::new(0, 6)),
            generics: vec![],
            variants: vec![
                Variant {
                    name: Spanned::new("Ok".to_string(), Span::new(7, 9)),
                    fields: VariantFields::Tuple(vec![Spanned::new(
                        Type::Infer,
                        Span::new(10, 11),
                    )]),
                },
                Variant {
                    name: Spanned::new("Err".to_string(), Span::new(12, 15)),
                    fields: VariantFields::Tuple(vec![Spanned::new(
                        Type::Infer,
                        Span::new(16, 17),
                    )]),
                },
            ],
            is_pub: true,
            attributes: vec![],
        };
        let result = gen.generate_enum(&e).unwrap();
        assert!(result.contains("Result.is_Ok = function"));
        assert!(result.contains("Result.is_Err = function"));
        assert!(result.contains("Result.unwrap = function"));
        assert!(result.contains("Result.unwrap_or = function"));
        assert!(result.contains("Result.map = function"));
    }

    #[test]
    fn test_option_enum_helpers() {
        let mut gen = JsCodeGenerator::new();
        let e = Enum {
            name: Spanned::new("Option".to_string(), Span::new(0, 6)),
            generics: vec![],
            variants: vec![
                Variant {
                    name: Spanned::new("Some".to_string(), Span::new(7, 11)),
                    fields: VariantFields::Tuple(vec![Spanned::new(
                        Type::Infer,
                        Span::new(12, 13),
                    )]),
                },
                Variant {
                    name: Spanned::new("None".to_string(), Span::new(14, 18)),
                    fields: VariantFields::Unit,
                },
            ],
            is_pub: true,
            attributes: vec![],
        };
        let result = gen.generate_enum(&e).unwrap();
        assert!(result.contains("Option.is_Some = function"));
        assert!(result.contains("Option.is_None = function"));
        assert!(result.contains("Option.unwrap = function"));
        assert!(result.contains("Option.unwrap_or = function"));
        assert!(result.contains("Option.map = function"));
    }

    #[test]
    fn test_generic_function_comment() {
        use vais_ast::{GenericParam, GenericParamKind, Variance};

        let mut gen = JsCodeGenerator::new();
        let func = Function {
            name: Spanned::new("identity".to_string(), Span::new(0, 8)),
            generics: vec![GenericParam {
                name: Spanned::new("T".to_string(), Span::new(9, 10)),
                bounds: vec![],
                kind: GenericParamKind::Type { bounds: vec![] },
                variance: Variance::Invariant,
            }],
            params: vec![Param {
                name: Spanned::new("x".to_string(), Span::new(11, 12)),
                ty: Spanned::new(Type::Infer, Span::new(14, 15)),
                is_mut: false,
                is_vararg: false,
                ownership: Ownership::Regular,
                default_value: None,
            }],
            ret_type: Some(Spanned::new(Type::Infer, Span::new(20, 21))),
            body: FunctionBody::Expr(Box::new(Spanned::new(
                Expr::Ident("x".to_string()),
                Span::new(22, 23),
            ))),
            is_pub: false,
            is_async: false,
            attributes: vec![],
            where_clause: vec![],
        };
        let result = gen.generate_function(&func, false).unwrap();
        assert!(result.contains("/* <T> */"));
        assert!(result.contains("function identity(x)"));
    }

    #[test]
    fn test_generic_struct_comment() {
        use vais_ast::{GenericParam, GenericParamKind, Variance};

        let mut gen = JsCodeGenerator::new();
        let s = Struct {
            name: Spanned::new("Box".to_string(), Span::new(0, 3)),
            generics: vec![GenericParam {
                name: Spanned::new("T".to_string(), Span::new(4, 5)),
                bounds: vec![],
                kind: GenericParamKind::Type { bounds: vec![] },
                variance: Variance::Invariant,
            }],
            fields: vec![Field {
                name: Spanned::new("value".to_string(), Span::new(6, 11)),
                ty: Spanned::new(Type::Infer, Span::new(13, 14)),
                is_pub: true,
            }],
            methods: vec![],
            is_pub: true,
            attributes: vec![],
            where_clause: vec![],
        };
        let result = gen.generate_struct(&s).unwrap();
        assert!(result.contains("/* <T> */"));
        assert!(result.contains("class Box"));
    }

    #[test]
    fn test_trait_impl_tracking() {
        let mut gen = JsCodeGenerator::new();

        // First create a simple struct to impl on
        let s = Struct {
            name: Spanned::new("MyType".to_string(), Span::new(0, 6)),
            generics: vec![],
            fields: vec![],
            methods: vec![],
            is_pub: false,
            attributes: vec![],
            where_clause: vec![],
        };
        gen.generate_struct(&s).unwrap();

        // Now create an impl block for a trait
        let imp = Impl {
            trait_name: Some(Spanned::new("Display".to_string(), Span::new(0, 7))),
            target_type: Spanned::new(
                Type::Named {
                    name: "MyType".to_string(),
                    generics: vec![],
                },
                Span::new(12, 18),
            ),
            generics: vec![],
            associated_types: vec![],
            methods: vec![Spanned::new(
                Function {
                    name: Spanned::new("to_string".to_string(), Span::new(19, 28)),
                    generics: vec![],
                    params: vec![Param {
                        name: Spanned::new("self".to_string(), Span::new(29, 33)),
                        ty: Spanned::new(Type::Infer, Span::new(35, 36)),
                        is_mut: false,
                        is_vararg: false,
                        ownership: Ownership::Regular,
                        default_value: None,
                    }],
                    ret_type: Some(Spanned::new(
                        Type::Named {
                            name: "str".to_string(),
                            generics: vec![],
                        },
                        Span::new(41, 44),
                    )),
                    body: FunctionBody::Expr(Box::new(Spanned::new(
                        Expr::String("MyType".to_string()),
                        Span::new(45, 53),
                    ))),
                    is_pub: false,
                    is_async: false,
                    attributes: vec![],
                    where_clause: vec![],
                },
                Span::new(19, 53),
            )],
        };
        let result = gen.generate_impl(&imp).unwrap();
        assert!(result.contains("MyType.__implements = new Set()"));
        assert!(result.contains("MyType.__implementsTrait = function"));
        assert!(result.contains("MyType.__implements.add(\"Display\")"));
        assert!(result.contains("MyType.prototype.to_string"));
    }

    #[test]
    fn test_const_private() {
        let mut gen = JsCodeGenerator::new();
        let c = ConstDef {
            name: Spanned::new("PI".to_string(), Span::new(0, 2)),
            ty: Spanned::new(
                Type::Named {
                    name: "f64".to_string(),
                    generics: vec![],
                },
                Span::new(4, 7),
            ),
            value: Spanned::new(Expr::Float(3.14), Span::new(10, 14)),
            is_pub: false,
            attributes: vec![],
        };
        let result = gen.generate_const(&c).unwrap();
        assert!(result.starts_with("const PI = "));
        assert!(!result.contains("export"));
    }

    #[test]
    fn test_global_generation() {
        let mut gen = JsCodeGenerator::new();
        let g = GlobalDef {
            name: Spanned::new("counter".to_string(), Span::new(0, 7)),
            ty: Spanned::new(
                Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                Span::new(9, 12),
            ),
            value: Spanned::new(Expr::Int(0), Span::new(15, 16)),
            is_pub: false,
            is_mutable: true,
        };
        let result = gen.generate_global(&g).unwrap();
        assert_eq!(result, "let counter = 0;\n");
    }

    #[test]
    fn test_global_public() {
        let mut gen = JsCodeGenerator::new();
        let g = GlobalDef {
            name: Spanned::new("VERSION".to_string(), Span::new(0, 7)),
            ty: Spanned::new(
                Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                Span::new(9, 12),
            ),
            value: Spanned::new(Expr::Int(1), Span::new(15, 16)),
            is_pub: true,
            is_mutable: false,
        };
        let result = gen.generate_global(&g).unwrap();
        assert_eq!(result, "export let VERSION = 1;\n");
    }

    #[test]
    fn test_extern_block_generation() {
        let mut gen = JsCodeGenerator::new();
        let ext = ExternBlock {
            abi: "C".to_string(),
            functions: vec![
                ExternFunction {
                    name: Spanned::new("malloc".to_string(), Span::new(0, 6)),
                    params: vec![],
                    ret_type: None,
                    is_vararg: false,
                    attributes: vec![],
                },
                ExternFunction {
                    name: Spanned::new("free".to_string(), Span::new(10, 14)),
                    params: vec![],
                    ret_type: None,
                    is_vararg: false,
                    attributes: vec![],
                },
            ],
        };
        let result = gen.generate_extern_block(&ext).unwrap();
        assert!(result.contains("/* extern: malloc"));
        assert!(result.contains("/* extern: free"));
        assert!(result.contains("must be provided at runtime"));
    }

    #[test]
    fn test_union_generation() {
        let mut gen = JsCodeGenerator::new();
        let u = Union {
            name: Spanned::new("Data".to_string(), Span::new(0, 4)),
            generics: vec![],
            fields: vec![
                Field {
                    name: Spanned::new("int_val".to_string(), Span::new(5, 12)),
                    ty: Spanned::new(
                        Type::Named {
                            name: "i64".to_string(),
                            generics: vec![],
                        },
                        Span::new(14, 17),
                    ),
                    is_pub: true,
                },
                Field {
                    name: Spanned::new("float_val".to_string(), Span::new(19, 28)),
                    ty: Spanned::new(
                        Type::Named {
                            name: "f64".to_string(),
                            generics: vec![],
                        },
                        Span::new(30, 33),
                    ),
                    is_pub: true,
                },
            ],
            is_pub: false,
        };
        let result = gen.generate_union(&u).unwrap();
        assert!(result.contains("class Data"));
        assert!(result.contains("constructor(value)"));
        assert!(result.contains("get int_val()"));
        assert!(result.contains("set int_val(v)"));
        assert!(result.contains("get float_val()"));
        assert!(result.contains("set float_val(v)"));
    }

    #[test]
    fn test_union_public() {
        let mut gen = JsCodeGenerator::new();
        let u = Union {
            name: Spanned::new("PubUnion".to_string(), Span::new(0, 8)),
            generics: vec![],
            fields: vec![Field {
                name: Spanned::new("val".to_string(), Span::new(9, 12)),
                ty: Spanned::new(
                    Type::Named {
                        name: "i64".to_string(),
                        generics: vec![],
                    },
                    Span::new(14, 17),
                ),
                is_pub: true,
            }],
            is_pub: true,
        };
        let result = gen.generate_union(&u).unwrap();
        assert!(result.contains("export class PubUnion"));
    }

    #[test]
    fn test_enum_struct_variant() {
        let mut gen = JsCodeGenerator::new();
        let e = Enum {
            name: Spanned::new("Shape".to_string(), Span::new(0, 5)),
            generics: vec![],
            variants: vec![Variant {
                name: Spanned::new("Circle".to_string(), Span::new(6, 12)),
                fields: VariantFields::Struct(vec![Field {
                    name: Spanned::new("radius".to_string(), Span::new(13, 19)),
                    ty: Spanned::new(
                        Type::Named {
                            name: "f64".to_string(),
                            generics: vec![],
                        },
                        Span::new(21, 24),
                    ),
                    is_pub: true,
                }]),
            }],
            is_pub: false,
            attributes: vec![],
        };
        let result = gen.generate_enum(&e).unwrap();
        assert!(result.contains("Circle(radius)"));
        assert!(result.contains("__tag: \"Circle\""));
    }

    #[test]
    fn test_enum_public() {
        let mut gen = JsCodeGenerator::new();
        let e = Enum {
            name: Spanned::new("Dir".to_string(), Span::new(0, 3)),
            generics: vec![],
            variants: vec![
                Variant {
                    name: Spanned::new("Up".to_string(), Span::new(4, 6)),
                    fields: VariantFields::Unit,
                },
                Variant {
                    name: Spanned::new("Down".to_string(), Span::new(8, 12)),
                    fields: VariantFields::Unit,
                },
            ],
            is_pub: true,
            attributes: vec![],
        };
        let result = gen.generate_enum(&e).unwrap();
        assert!(result.contains("export const Dir"));
    }

    #[test]
    fn test_type_alias_empty() {
        let mut gen = JsCodeGenerator::new();
        let item = Item::TypeAlias(TypeAlias {
            name: Spanned::new("Num".to_string(), Span::new(0, 3)),
            generics: vec![],
            ty: Spanned::new(
                Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                Span::new(6, 9),
            ),
            is_pub: false,
        });
        let result = gen.generate_item(&item).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_error_item() {
        let mut gen = JsCodeGenerator::new();
        let item = Item::Error {
            message: "parse failed".to_string(),
            skipped_tokens: vec![],
        };
        let result = gen.generate_item(&item).unwrap();
        assert!(result.contains("error"));
        assert!(result.contains("parse failed"));
    }

    #[test]
    fn test_async_function() {
        let mut gen = JsCodeGenerator::new();
        let func = Function {
            name: Spanned::new("fetch_data".to_string(), Span::new(0, 10)),
            generics: vec![],
            params: vec![],
            ret_type: None,
            body: FunctionBody::Expr(Box::new(Spanned::new(Expr::Int(42), Span::new(0, 2)))),
            is_pub: false,
            is_async: true,
            attributes: vec![],
            where_clause: vec![],
        };
        let result = gen.generate_function(&func, false).unwrap();
        assert!(result.contains("async function fetch_data()"));
    }

    #[test]
    fn test_function_with_default_param() {
        let mut gen = JsCodeGenerator::new();
        let func = Function {
            name: Spanned::new("greet".to_string(), Span::new(0, 5)),
            generics: vec![],
            params: vec![Param {
                name: Spanned::new("name".to_string(), Span::new(6, 10)),
                ty: Spanned::new(
                    Type::Named {
                        name: "str".to_string(),
                        generics: vec![],
                    },
                    Span::new(12, 15),
                ),
                is_mut: false,
                is_vararg: false,
                ownership: Ownership::Regular,
                default_value: Some(Box::new(Spanned::new(
                    Expr::String("world".to_string()),
                    Span::new(18, 25),
                ))),
            }],
            ret_type: None,
            body: FunctionBody::Expr(Box::new(Spanned::new(
                Expr::Ident("name".to_string()),
                Span::new(28, 32),
            ))),
            is_pub: false,
            is_async: false,
            attributes: vec![],
            where_clause: vec![],
        };
        let result = gen.generate_function(&func, false).unwrap();
        assert!(result.contains("name = \"world\""));
    }

    #[test]
    fn test_struct_with_method() {
        let mut gen = JsCodeGenerator::new();
        let s = Struct {
            name: Spanned::new("Counter".to_string(), Span::new(0, 7)),
            generics: vec![],
            fields: vec![Field {
                name: Spanned::new("count".to_string(), Span::new(8, 13)),
                ty: Spanned::new(
                    Type::Named {
                        name: "i64".to_string(),
                        generics: vec![],
                    },
                    Span::new(15, 18),
                ),
                is_pub: true,
            }],
            methods: vec![Spanned::new(
                Function {
                    name: Spanned::new("inc".to_string(), Span::new(20, 23)),
                    generics: vec![],
                    params: vec![],
                    ret_type: None,
                    body: FunctionBody::Expr(Box::new(Spanned::new(
                        Expr::Int(1),
                        Span::new(26, 27),
                    ))),
                    is_pub: false,
                    is_async: false,
                    attributes: vec![],
                    where_clause: vec![],
                },
                Span::new(20, 28),
            )],
            is_pub: false,
            attributes: vec![],
            where_clause: vec![],
        };
        let result = gen.generate_struct(&s).unwrap();
        assert!(result.contains("class Counter"));
        assert!(result.contains("constructor(count)"));
        assert!(result.contains("this.count = count;"));
        assert!(result.contains("inc()"));
    }

    #[test]
    fn test_trait_with_default_method() {
        let mut gen = JsCodeGenerator::new();
        let t = Trait {
            name: Spanned::new("Greet".to_string(), Span::new(0, 5)),
            generics: vec![],
            super_traits: vec![],
            methods: vec![TraitMethod {
                name: Spanned::new("greet".to_string(), Span::new(6, 11)),
                params: vec![],
                ret_type: None,
                is_async: false,
                is_const: false,
                default_body: Some(FunctionBody::Expr(Box::new(Spanned::new(
                    Expr::String("hello".to_string()),
                    Span::new(14, 21),
                )))),
            }],
            associated_types: vec![],
            is_pub: false,
            where_clause: vec![],
        };
        let result = gen.generate_trait(&t).unwrap();
        assert!(result.contains("class Greet"));
        assert!(result.contains("greet()"));
        assert!(result.contains("return \"hello\""));
    }

    #[test]
    fn test_trait_abstract_method() {
        let mut gen = JsCodeGenerator::new();
        let t = Trait {
            name: Spanned::new("Drawable".to_string(), Span::new(0, 8)),
            generics: vec![],
            super_traits: vec![],
            methods: vec![TraitMethod {
                name: Spanned::new("draw".to_string(), Span::new(9, 13)),
                params: vec![],
                ret_type: None,
                is_async: false,
                is_const: false,
                default_body: None,
            }],
            associated_types: vec![],
            is_pub: false,
            where_clause: vec![],
        };
        let result = gen.generate_trait(&t).unwrap();
        assert!(result.contains("draw()"));
        assert!(result.contains("throw new Error"));
        assert!(result.contains("Drawable.draw not implemented"));
    }

    #[test]
    fn test_trait_public() {
        let mut gen = JsCodeGenerator::new();
        let t = Trait {
            name: Spanned::new("PubTrait".to_string(), Span::new(0, 8)),
            generics: vec![],
            super_traits: vec![],
            methods: vec![],
            associated_types: vec![],
            is_pub: true,
            where_clause: vec![],
        };
        let result = gen.generate_trait(&t).unwrap();
        assert!(result.contains("export class PubTrait"));
    }

    #[test]
    fn test_impl_static_method() {
        let mut gen = JsCodeGenerator::new();
        let imp = Impl {
            trait_name: None,
            target_type: Spanned::new(
                Type::Named {
                    name: "Vec".to_string(),
                    generics: vec![],
                },
                Span::new(0, 3),
            ),
            generics: vec![],
            associated_types: vec![],
            methods: vec![Spanned::new(
                Function {
                    name: Spanned::new("new".to_string(), Span::new(4, 7)),
                    generics: vec![],
                    params: vec![], // No self param = static method
                    ret_type: None,
                    body: FunctionBody::Expr(Box::new(Spanned::new(
                        Expr::Array(vec![]),
                        Span::new(10, 12),
                    ))),
                    is_pub: false,
                    is_async: false,
                    attributes: vec![],
                    where_clause: vec![],
                },
                Span::new(4, 13),
            )],
        };
        let result = gen.generate_impl(&imp).unwrap();
        assert!(result.contains("Vec._new = function()"));
        // Should NOT be on prototype (no self param)
        assert!(!result.contains("prototype._new"));
    }

    #[test]
    fn test_impl_without_trait() {
        let mut gen = JsCodeGenerator::new();
        let imp = Impl {
            trait_name: None,
            target_type: Spanned::new(
                Type::Named {
                    name: "Point".to_string(),
                    generics: vec![],
                },
                Span::new(0, 5),
            ),
            generics: vec![],
            associated_types: vec![],
            methods: vec![],
        };
        let result = gen.generate_impl(&imp).unwrap();
        // No trait = no __implements tracking
        assert!(!result.contains("__implements"));
    }

    #[test]
    fn test_impl_non_named_type() {
        let mut gen = JsCodeGenerator::new();
        let imp = Impl {
            trait_name: None,
            target_type: Spanned::new(Type::Unit, Span::new(0, 2)),
            generics: vec![],
            associated_types: vec![],
            methods: vec![],
        };
        let result = gen.generate_impl(&imp).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_macro_item_empty() {
        let mut gen = JsCodeGenerator::new();
        let item = Item::Macro(MacroDef {
            name: Spanned::new("my_macro".to_string(), Span::new(0, 8)),
            rules: vec![],
            is_pub: false,
        });
        let result = gen.generate_item(&item).unwrap();
        assert_eq!(result, "");
    }
}

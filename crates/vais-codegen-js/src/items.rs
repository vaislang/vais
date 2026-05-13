//! Top-level item code generation: Vais Item → JavaScript declarations

use crate::expr_helpers::sanitize_js_ident;
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
#[path = "items_tests.rs"]
mod items_tests;

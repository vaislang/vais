//! Vais Documentation Generator
//!
//! Generate documentation from Vais source code.
//! Supports Markdown, HTML, and JSON output formats.

use vais_ast::{Program, Item, FunctionDef, TypeExpr};

/// Documentation entry for a function
#[derive(Debug, Clone)]
pub struct FunctionDoc {
    /// Function name
    pub name: String,
    /// Parameters with types
    pub params: Vec<(String, Option<String>)>,
    /// Return type
    pub return_type: Option<String>,
    /// Documentation comment
    pub doc_comment: Option<String>,
    /// Is async function
    pub is_async: bool,
    /// Source location (line number)
    pub line: usize,
}

/// Documentation for a module/file
#[derive(Debug, Clone)]
pub struct ModuleDoc {
    /// Module name (file name without extension)
    pub name: String,
    /// Module-level documentation
    pub doc_comment: Option<String>,
    /// Functions in this module
    pub functions: Vec<FunctionDoc>,
    /// Type definitions
    pub types: Vec<TypeDoc>,
    /// Constants
    pub constants: Vec<ConstDoc>,
}

/// Documentation for a type definition
#[derive(Debug, Clone)]
pub struct TypeDoc {
    /// Type name
    pub name: String,
    /// Type definition
    pub definition: String,
    /// Documentation comment
    pub doc_comment: Option<String>,
}

/// Documentation for a constant
#[derive(Debug, Clone)]
pub struct ConstDoc {
    /// Constant name
    pub name: String,
    /// Value representation
    pub value: String,
    /// Type
    pub type_name: Option<String>,
    /// Documentation comment
    pub doc_comment: Option<String>,
}

/// Documentation generator
pub struct DocGenerator {
    /// Output format
    format: DocFormat,
}

/// Output format for documentation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DocFormat {
    Markdown,
    Html,
    Json,
}

impl DocGenerator {
    pub fn new(format: DocFormat) -> Self {
        Self { format }
    }

    /// Generate documentation for a program
    pub fn generate(&self, program: &Program, module_name: &str) -> String {
        let module_doc = self.extract_module_doc(program, module_name);

        match self.format {
            DocFormat::Markdown => self.generate_markdown(&module_doc),
            DocFormat::Html => self.generate_html(&module_doc),
            DocFormat::Json => self.generate_json(&module_doc),
        }
    }

    /// Extract documentation from a program
    fn extract_module_doc(&self, program: &Program, module_name: &str) -> ModuleDoc {
        let mut functions = Vec::new();
        let mut types = Vec::new();
        let constants = Vec::new();

        for item in &program.items {
            match item {
                Item::Function(func) => {
                    // Skip internal functions
                    if !func.name.starts_with('_') {
                        functions.push(self.extract_function_doc(func));
                    }
                }
                Item::TypeDef(typedef) => {
                    types.push(TypeDoc {
                        name: typedef.name.clone(),
                        definition: self.format_type_expr(&typedef.ty),
                        doc_comment: None,
                    });
                }
                Item::Enum(enum_def) => {
                    let variants_str = enum_def.variants
                        .iter()
                        .map(|v| v.name.clone())
                        .collect::<Vec<_>>()
                        .join(" | ");
                    types.push(TypeDoc {
                        name: enum_def.name.clone(),
                        definition: variants_str,
                        doc_comment: None,
                    });
                }
                _ => {}
            }
        }

        ModuleDoc {
            name: module_name.to_string(),
            doc_comment: None,
            functions,
            types,
            constants,
        }
    }

    /// Extract documentation from a function
    fn extract_function_doc(&self, func: &FunctionDef) -> FunctionDoc {
        let params: Vec<(String, Option<String>)> = func
            .params
            .iter()
            .map(|p| {
                let name = p.name.clone();
                let type_str = p.ty.as_ref().map(|t| self.format_type_expr(t));
                (name, type_str)
            })
            .collect();

        let return_type = func.return_type.as_ref().map(|t| self.format_type_expr(t));

        FunctionDoc {
            name: func.name.clone(),
            params,
            return_type,
            doc_comment: None,
            is_async: func.is_async,
            line: func.span.start, // Approximate line number
        }
    }

    /// Format type expression to string
    fn format_type_expr(&self, type_expr: &TypeExpr) -> String {
        match type_expr {
            TypeExpr::Simple(name) => name.clone(),
            TypeExpr::TypeVar(name) => name.clone(),
            TypeExpr::Generic(name, args) => {
                let args_str: Vec<_> = args.iter().map(|t| self.format_type_expr(t)).collect();
                format!("{}<{}>", name, args_str.join(", "))
            }
            TypeExpr::Function(params, ret) => {
                let params_str: Vec<_> = params.iter().map(|t| self.format_type_expr(t)).collect();
                format!("({}) -> {}", params_str.join(", "), self.format_type_expr(ret))
            }
            TypeExpr::Tuple(elems) => {
                let elems_str: Vec<_> = elems.iter().map(|t| self.format_type_expr(t)).collect();
                format!("({})", elems_str.join(", "))
            }
            TypeExpr::Array(elem) => format!("[{}]", self.format_type_expr(elem)),
            TypeExpr::Map(key, value) => {
                format!("{{{}:{}}}", self.format_type_expr(key), self.format_type_expr(value))
            }
            TypeExpr::Set(elem) => format!("#{{{}}}", self.format_type_expr(elem)),
            TypeExpr::Optional(inner) => format!("?{}", self.format_type_expr(inner)),
            TypeExpr::Result(inner) => format!("!{}", self.format_type_expr(inner)),
            TypeExpr::Future(inner) => format!("Future<{}>", self.format_type_expr(inner)),
            TypeExpr::Channel(inner) => format!("Chan<{}>", self.format_type_expr(inner)),
            TypeExpr::Struct(fields) => {
                let fields_str: Vec<_> = fields
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, self.format_type_expr(t)))
                    .collect();
                format!("{{ {} }}", fields_str.join(", "))
            }
        }
    }

    /// Generate Markdown documentation
    fn generate_markdown(&self, module: &ModuleDoc) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("# {}\n\n", module.name));

        if let Some(doc) = &module.doc_comment {
            output.push_str(doc);
            output.push_str("\n\n");
        }

        // Types section
        if !module.types.is_empty() {
            output.push_str("## Types\n\n");
            for type_doc in &module.types {
                output.push_str(&format!("### `{}`\n\n", type_doc.name));
                output.push_str(&format!("```\n{}\n```\n\n", type_doc.definition));
                if let Some(doc) = &type_doc.doc_comment {
                    output.push_str(doc);
                    output.push_str("\n\n");
                }
            }
        }

        // Functions section
        if !module.functions.is_empty() {
            output.push_str("## Functions\n\n");
            for func in &module.functions {
                // Function signature
                let async_prefix = if func.is_async { "async " } else { "" };
                let params_str: Vec<String> = func
                    .params
                    .iter()
                    .map(|(name, ty)| {
                        if let Some(t) = ty {
                            format!("{}: {}", name, t)
                        } else {
                            name.clone()
                        }
                    })
                    .collect();
                let return_str = func
                    .return_type
                    .as_ref()
                    .map(|t| format!(" -> {}", t))
                    .unwrap_or_default();

                output.push_str(&format!(
                    "### `{}{}({}){}`\n\n",
                    async_prefix,
                    func.name,
                    params_str.join(", "),
                    return_str
                ));

                if let Some(doc) = &func.doc_comment {
                    output.push_str(doc);
                    output.push_str("\n\n");
                }

                // Parameters table
                if !func.params.is_empty() {
                    output.push_str("**Parameters:**\n\n");
                    output.push_str("| Name | Type |\n");
                    output.push_str("|------|------|\n");
                    for (name, ty) in &func.params {
                        let type_str = ty.as_deref().unwrap_or("-");
                        output.push_str(&format!("| `{}` | `{}` |\n", name, type_str));
                    }
                    output.push('\n');
                }

                if let Some(ret) = &func.return_type {
                    output.push_str(&format!("**Returns:** `{}`\n\n", ret));
                }

                output.push_str("---\n\n");
            }
        }

        // Constants section
        if !module.constants.is_empty() {
            output.push_str("## Constants\n\n");
            for const_doc in &module.constants {
                output.push_str(&format!("### `{}`\n\n", const_doc.name));
                output.push_str(&format!("Value: `{}`\n\n", const_doc.value));
            }
        }

        output
    }

    /// Generate HTML documentation
    fn generate_html(&self, module: &ModuleDoc) -> String {
        let mut output = String::new();

        output.push_str("<!DOCTYPE html>\n");
        output.push_str("<html lang=\"en\">\n");
        output.push_str("<head>\n");
        output.push_str("  <meta charset=\"UTF-8\">\n");
        output.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        output.push_str(&format!("  <title>{} - Vais Documentation</title>\n", module.name));
        output.push_str("  <style>\n");
        output.push_str("    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 900px; margin: 0 auto; padding: 2rem; line-height: 1.6; }\n");
        output.push_str("    h1 { border-bottom: 2px solid #eee; padding-bottom: 0.5rem; }\n");
        output.push_str("    h2 { margin-top: 2rem; color: #333; }\n");
        output.push_str("    h3 { margin-top: 1.5rem; }\n");
        output.push_str("    code { background: #f4f4f4; padding: 0.2rem 0.4rem; border-radius: 3px; font-family: 'SF Mono', Consolas, monospace; }\n");
        output.push_str("    pre { background: #f8f8f8; padding: 1rem; border-radius: 5px; overflow-x: auto; }\n");
        output.push_str("    pre code { background: none; padding: 0; }\n");
        output.push_str("    table { border-collapse: collapse; width: 100%; margin: 1rem 0; }\n");
        output.push_str("    th, td { border: 1px solid #ddd; padding: 0.5rem; text-align: left; }\n");
        output.push_str("    th { background: #f4f4f4; }\n");
        output.push_str("    .async-badge { background: #4CAF50; color: white; padding: 0.1rem 0.4rem; border-radius: 3px; font-size: 0.8em; margin-left: 0.5rem; }\n");
        output.push_str("    hr { border: none; border-top: 1px solid #eee; margin: 2rem 0; }\n");
        output.push_str("  </style>\n");
        output.push_str("</head>\n");
        output.push_str("<body>\n");

        // Header
        output.push_str(&format!("<h1>{}</h1>\n", module.name));

        if let Some(doc) = &module.doc_comment {
            output.push_str(&format!("<p>{}</p>\n", doc));
        }

        // Types section
        if !module.types.is_empty() {
            output.push_str("<h2>Types</h2>\n");
            for type_doc in &module.types {
                output.push_str(&format!("<h3><code>{}</code></h3>\n", type_doc.name));
                output.push_str(&format!("<pre><code>{}</code></pre>\n", type_doc.definition));
            }
        }

        // Functions section
        if !module.functions.is_empty() {
            output.push_str("<h2>Functions</h2>\n");
            for func in &module.functions {
                let async_badge = if func.is_async {
                    "<span class=\"async-badge\">async</span>"
                } else {
                    ""
                };

                let params_str: Vec<String> = func
                    .params
                    .iter()
                    .map(|(name, ty)| {
                        if let Some(t) = ty {
                            format!("{}: {}", name, t)
                        } else {
                            name.clone()
                        }
                    })
                    .collect();
                let return_str = func
                    .return_type
                    .as_ref()
                    .map(|t| format!(" -&gt; {}", t))
                    .unwrap_or_default();

                output.push_str(&format!(
                    "<h3><code>{}({}){}</code>{}</h3>\n",
                    func.name,
                    params_str.join(", "),
                    return_str,
                    async_badge
                ));

                if !func.params.is_empty() {
                    output.push_str("<p><strong>Parameters:</strong></p>\n");
                    output.push_str("<table>\n<thead><tr><th>Name</th><th>Type</th></tr></thead>\n<tbody>\n");
                    for (name, ty) in &func.params {
                        let type_str = ty.as_deref().unwrap_or("-");
                        output.push_str(&format!("<tr><td><code>{}</code></td><td><code>{}</code></td></tr>\n", name, type_str));
                    }
                    output.push_str("</tbody></table>\n");
                }

                if let Some(ret) = &func.return_type {
                    output.push_str(&format!("<p><strong>Returns:</strong> <code>{}</code></p>\n", ret));
                }

                output.push_str("<hr>\n");
            }
        }

        output.push_str("</body>\n");
        output.push_str("</html>\n");

        output
    }

    /// Generate JSON documentation
    fn generate_json(&self, module: &ModuleDoc) -> String {
        let functions_json: Vec<String> = module
            .functions
            .iter()
            .map(|f| {
                let params_json: Vec<String> = f
                    .params
                    .iter()
                    .map(|(n, t)| {
                        format!(
                            r#"{{"name": "{}", "type": {}}}"#,
                            n,
                            t.as_ref()
                                .map(|t| format!("\"{}\"", t))
                                .unwrap_or_else(|| "null".to_string())
                        )
                    })
                    .collect();

                format!(
                    r#"    {{
      "name": "{}",
      "params": [{}],
      "return_type": {},
      "is_async": {},
      "line": {}
    }}"#,
                    f.name,
                    params_json.join(", "),
                    f.return_type
                        .as_ref()
                        .map(|t| format!("\"{}\"", t))
                        .unwrap_or_else(|| "null".to_string()),
                    f.is_async,
                    f.line
                )
            })
            .collect();

        let types_json: Vec<String> = module
            .types
            .iter()
            .map(|t| {
                format!(
                    r#"    {{
      "name": "{}",
      "definition": "{}"
    }}"#,
                    t.name, t.definition
                )
            })
            .collect();

        format!(
            r#"{{
  "module": "{}",
  "functions": [
{}
  ],
  "types": [
{}
  ]
}}"#,
            module.name,
            functions_json.join(",\n"),
            types_json.join(",\n")
        )
    }
}

impl Default for DocGenerator {
    fn default() -> Self {
        Self::new(DocFormat::Markdown)
    }
}

impl DocFormat {
    /// Parse format from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Some(DocFormat::Markdown),
            "html" | "htm" => Some(DocFormat::Html),
            "json" => Some(DocFormat::Json),
            _ => None,
        }
    }

    /// Get file extension for format
    pub fn extension(&self) -> &'static str {
        match self {
            DocFormat::Markdown => "md",
            DocFormat::Html => "html",
            DocFormat::Json => "json",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vais_ast::{Expr, Param};
    use vais_lexer::Span;

    #[test]
    fn test_markdown_generation() {
        let program = Program {
            items: vec![Item::Function(FunctionDef {
                name: "add".to_string(),
                type_params: vec![],
                params: vec![
                    Param { name: "a".to_string(), ty: None, default: None, span: Span::new(0, 1) },
                    Param { name: "b".to_string(), ty: None, default: None, span: Span::new(0, 1) },
                ],
                body: Expr::Integer(0, Span::new(0, 1)),
                return_type: Some(TypeExpr::Simple("Int".to_string())),
                is_pub: false,
                is_async: false,
                span: Span::new(0, 1),
            })],
            span: Span::new(0, 1),
        };

        let gen = DocGenerator::new(DocFormat::Markdown);
        let doc = gen.generate(&program, "test");

        assert!(doc.contains("# test"));
        assert!(doc.contains("## Functions"));
        assert!(doc.contains("### `add(a, b) -> Int`"));
    }

    #[test]
    fn test_html_generation() {
        let program = Program {
            items: vec![Item::Function(FunctionDef {
                name: "fetch".to_string(),
                type_params: vec![],
                params: vec![
                    Param { name: "url".to_string(), ty: None, default: None, span: Span::new(0, 1) },
                ],
                body: Expr::Integer(0, Span::new(0, 1)),
                return_type: None,
                is_pub: false,
                is_async: true,
                span: Span::new(0, 1),
            })],
            span: Span::new(0, 1),
        };

        let gen = DocGenerator::new(DocFormat::Html);
        let doc = gen.generate(&program, "test");

        assert!(doc.contains("<html"));
        assert!(doc.contains("async-badge"));
        assert!(doc.contains("fetch"));
    }

    #[test]
    fn test_json_generation() {
        let program = Program {
            items: vec![Item::Function(FunctionDef {
                name: "multiply".to_string(),
                type_params: vec![],
                params: vec![],
                body: Expr::Integer(0, Span::new(0, 1)),
                return_type: Some(TypeExpr::Simple("Float".to_string())),
                is_pub: false,
                is_async: false,
                span: Span::new(0, 1),
            })],
            span: Span::new(0, 1),
        };

        let gen = DocGenerator::new(DocFormat::Json);
        let doc = gen.generate(&program, "test");

        assert!(doc.contains("\"module\": \"test\""));
        assert!(doc.contains("\"name\": \"multiply\""));
        assert!(doc.contains("\"return_type\": \"Float\""));
    }
}

//! Documentation generator for Vais - Rustdoc style
//!
//! Generates HTML documentation from Vais source files with doc comments (///).

use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

use vais_ast::{Item, Module, Function, Struct, Enum, Trait};
use vais_parser::parse;

/// Documentation item extracted from source
#[derive(Debug, Clone)]
struct DocItem {
    name: String,
    kind: DocKind,
    signature: String,
    docs: Vec<String>,
    params: Vec<ParamDoc>,
    returns: Option<String>,
    examples: Vec<String>,
    _generics: Vec<GenericDoc>,
    visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq)]
enum DocKind {
    Function,
    Struct,
    Enum,
    Trait,
    #[allow(dead_code)]
    Module,
}

#[derive(Debug, Clone)]
struct ParamDoc {
    name: String,
    ty: String,
    is_mut: bool,
}

#[derive(Debug, Clone)]
struct GenericDoc {
    name: String,
    bounds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum Visibility {
    Public,
    Private,
}

/// Module documentation
struct ModuleDoc {
    name: String,
    path: PathBuf,
    items: Vec<DocItem>,
}

/// Generate documentation from source files
pub fn run(input: &PathBuf, output: &PathBuf, format: &str) -> Result<(), String> {
    println!(
        "{} documentation from {}",
        "Generating".green().bold(),
        input.display()
    );

    // Create output directory
    fs::create_dir_all(output)
        .map_err(|e| format!("Cannot create output directory: {}", e))?;

    // Collect source files
    let files = if input.is_dir() {
        collect_vais_files(input)?
    } else {
        vec![input.clone()]
    };

    if files.is_empty() {
        return Err("No .vais files found".to_string());
    }

    let mut all_docs = Vec::new();

    for file in &files {
        let source = fs::read_to_string(file)
            .map_err(|e| format!("Cannot read '{}': {}", file.display(), e))?;

        let ast = parse(&source)
            .map_err(|e| format!("Parse error in '{}': {}", file.display(), e))?;

        let doc = extract_documentation(file, &ast, &source);
        all_docs.push(doc);
    }

    // Generate output based on format
    match format {
        "markdown" | "md" => {
            generate_markdown_docs(&all_docs, output)?;
        }
        "html" => {
            generate_html_docs(&all_docs, output)?;
        }
        _ => {
            return Err(format!(
                "Unknown format: {}. Use 'markdown' or 'html'.",
                format
            ));
        }
    }

    println!(
        "{} Documentation written to {}",
        "Done".green().bold(),
        output.display()
    );
    Ok(())
}

/// Collect all .vais files in a directory recursively
fn collect_vais_files(dir: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir).map_err(|e| format!("Cannot read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Cannot read entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            files.extend(collect_vais_files(&path)?);
        } else if path.extension().map(|e| e == "vais").unwrap_or(false) {
            files.push(path);
        }
    }

    Ok(files)
}

/// Extract documentation from AST and source code
fn extract_documentation(file: &Path, ast: &Module, source: &str) -> ModuleDoc {
    let module_name = file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut items = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for item in &ast.items {
        // Extract doc comments by looking backwards from item position
        let docs = extract_doc_comments(&lines, item.span.start);

        match &item.node {
            Item::Function(f) => {
                items.push(extract_function_doc(f, docs));
            }
            Item::Struct(s) => {
                items.push(extract_struct_doc(s, docs));
            }
            Item::Enum(e) => {
                items.push(extract_enum_doc(e, docs));
            }
            Item::Trait(t) => {
                items.push(extract_trait_doc(t, docs));
            }
            _ => {}
        }
    }

    ModuleDoc {
        name: module_name,
        path: file.to_path_buf(),
        items,
    }
}

/// Extract doc comments by looking backwards from a given position
fn extract_doc_comments(lines: &[&str], start_pos: usize) -> Vec<String> {
    let mut docs = Vec::new();
    let mut byte_count = 0;

    for (i, line) in lines.iter().enumerate() {
        let line_end = byte_count + line.len() + 1; // +1 for newline

        if line_end > start_pos {
            // We've reached the item, look backwards for doc comments
            for j in (0..i).rev() {
                let prev_line = lines[j].trim();
                if let Some(stripped) = prev_line.strip_prefix("///") {
                    docs.insert(0, stripped.trim().to_string());
                } else if !prev_line.is_empty() {
                    // Non-doc-comment line, stop
                    break;
                }
            }
            break;
        }

        byte_count = line_end;
    }

    docs
}

/// Parse examples from doc comments
fn parse_examples(docs: &[String]) -> Vec<String> {
    let mut examples = Vec::new();
    let mut in_example = false;
    let mut current_example = String::new();

    for line in docs {
        if line.starts_with("```vais") || line.starts_with("```") {
            if in_example {
                examples.push(current_example.clone());
                current_example.clear();
                in_example = false;
            } else {
                in_example = true;
            }
        } else if in_example {
            current_example.push_str(line);
            current_example.push('\n');
        }
    }

    examples
}

/// Extract function documentation
fn extract_function_doc(f: &Function, docs: Vec<String>) -> DocItem {
    let params: Vec<ParamDoc> = f
        .params
        .iter()
        .map(|p| ParamDoc {
            name: p.name.node.clone(),
            ty: format!("{}", p.ty.node),
            is_mut: p.is_mut,
        })
        .collect();

    let generics: Vec<GenericDoc> = f
        .generics
        .iter()
        .map(|g| GenericDoc {
            name: g.name.node.clone(),
            bounds: g.bounds.iter().map(|b| b.node.clone()).collect(),
        })
        .collect();

    let returns = f.ret_type.as_ref().map(|t| format!("{}", t.node));

    let mut signature = String::new();
    if f.is_pub {
        signature.push_str("P ");
    }
    if f.is_async {
        signature.push_str("A ");
    }
    signature.push_str(&format!("F {}", f.name.node));

    if !generics.is_empty() {
        signature.push('<');
        for (i, g) in generics.iter().enumerate() {
            if i > 0 {
                signature.push_str(", ");
            }
            signature.push_str(&g.name);
            if !g.bounds.is_empty() {
                signature.push_str(": ");
                signature.push_str(&g.bounds.join(" + "));
            }
        }
        signature.push('>');
    }

    signature.push('(');
    for (i, p) in params.iter().enumerate() {
        if i > 0 {
            signature.push_str(", ");
        }
        if p.is_mut {
            signature.push_str("mut ");
        }
        signature.push_str(&format!("{}: {}", p.name, p.ty));
    }
    signature.push(')');

    if let Some(ret) = &returns {
        signature.push_str(&format!(" -> {}", ret));
    }

    let examples = parse_examples(&docs);

    DocItem {
        name: f.name.node.clone(),
        kind: DocKind::Function,
        signature,
        docs: docs.clone(),
        params,
        returns,
        examples,
        _generics: generics,
        visibility: if f.is_pub { Visibility::Public } else { Visibility::Private },
    }
}

/// Extract struct documentation
fn extract_struct_doc(s: &Struct, docs: Vec<String>) -> DocItem {
    let generics: Vec<GenericDoc> = s
        .generics
        .iter()
        .map(|g| GenericDoc {
            name: g.name.node.clone(),
            bounds: g.bounds.iter().map(|b| b.node.clone()).collect(),
        })
        .collect();

    let fields: Vec<String> = s
        .fields
        .iter()
        .map(|f| {
            let vis = if f.is_pub { "P " } else { "" };
            format!("{}{}: {}", vis, f.name.node, f.ty.node)
        })
        .collect();

    let mut signature = String::new();
    if s.is_pub {
        signature.push_str("P ");
    }
    signature.push_str(&format!("S {}", s.name.node));

    if !generics.is_empty() {
        signature.push('<');
        for (i, g) in generics.iter().enumerate() {
            if i > 0 {
                signature.push_str(", ");
            }
            signature.push_str(&g.name);
            if !g.bounds.is_empty() {
                signature.push_str(": ");
                signature.push_str(&g.bounds.join(" + "));
            }
        }
        signature.push('>');
    }

    signature.push_str(" { ");
    signature.push_str(&fields.join(", "));
    signature.push_str(" }");

    let examples = parse_examples(&docs);

    DocItem {
        name: s.name.node.clone(),
        kind: DocKind::Struct,
        signature,
        docs,
        params: vec![],
        returns: None,
        examples,
        _generics: generics,
        visibility: if s.is_pub { Visibility::Public } else { Visibility::Private },
    }
}

/// Extract enum documentation
fn extract_enum_doc(e: &Enum, docs: Vec<String>) -> DocItem {
    let generics: Vec<GenericDoc> = e
        .generics
        .iter()
        .map(|g| GenericDoc {
            name: g.name.node.clone(),
            bounds: g.bounds.iter().map(|b| b.node.clone()).collect(),
        })
        .collect();

    let variants: Vec<String> = e
        .variants
        .iter()
        .map(|v| v.name.node.clone())
        .collect();

    let mut signature = String::new();
    if e.is_pub {
        signature.push_str("P ");
    }
    signature.push_str(&format!("E {}", e.name.node));

    if !generics.is_empty() {
        signature.push('<');
        for (i, g) in generics.iter().enumerate() {
            if i > 0 {
                signature.push_str(", ");
            }
            signature.push_str(&g.name);
            if !g.bounds.is_empty() {
                signature.push_str(": ");
                signature.push_str(&g.bounds.join(" + "));
            }
        }
        signature.push('>');
    }

    signature.push_str(" { ");
    signature.push_str(&variants.join(", "));
    signature.push_str(" }");

    let examples = parse_examples(&docs);

    DocItem {
        name: e.name.node.clone(),
        kind: DocKind::Enum,
        signature,
        docs,
        params: vec![],
        returns: None,
        examples,
        _generics: generics,
        visibility: if e.is_pub { Visibility::Public } else { Visibility::Private },
    }
}

/// Extract trait documentation
fn extract_trait_doc(t: &Trait, docs: Vec<String>) -> DocItem {
    let generics: Vec<GenericDoc> = t
        .generics
        .iter()
        .map(|g| GenericDoc {
            name: g.name.node.clone(),
            bounds: g.bounds.iter().map(|b| b.node.clone()).collect(),
        })
        .collect();

    let methods: Vec<String> = t
        .methods
        .iter()
        .map(|m| m.name.node.clone())
        .collect();

    let mut signature = String::new();
    if t.is_pub {
        signature.push_str("P ");
    }
    signature.push_str(&format!("W {}", t.name.node));

    if !generics.is_empty() {
        signature.push('<');
        for (i, g) in generics.iter().enumerate() {
            if i > 0 {
                signature.push_str(", ");
            }
            signature.push_str(&g.name);
            if !g.bounds.is_empty() {
                signature.push_str(": ");
                signature.push_str(&g.bounds.join(" + "));
            }
        }
        signature.push('>');
    }

    if !t.super_traits.is_empty() {
        signature.push_str(": ");
        signature.push_str(
            &t.super_traits
                .iter()
                .map(|s| s.node.as_str())
                .collect::<Vec<_>>()
                .join(" + "),
        );
    }

    signature.push_str(" { ");
    signature.push_str(&methods.join(", "));
    signature.push_str(" }");

    let examples = parse_examples(&docs);

    DocItem {
        name: t.name.node.clone(),
        kind: DocKind::Trait,
        signature,
        docs,
        params: vec![],
        returns: None,
        examples,
        _generics: generics,
        visibility: if t.is_pub { Visibility::Public } else { Visibility::Private },
    }
}

/// Generate markdown documentation
fn generate_markdown_docs(docs: &[ModuleDoc], output: &Path) -> Result<(), String> {
    let mut index = String::new();
    index.push_str("# Vais API Documentation\n\n");
    index.push_str("## Modules\n\n");

    for doc in docs {
        index.push_str(&format!("- [{}]({}.md)\n", doc.name, doc.name));

        // Generate module file
        let mut content = String::new();
        content.push_str(&format!("# Module: {}\n\n", doc.name));
        content.push_str(&format!("Source: `{}`\n\n", doc.path.display()));

        // Group by kind
        let structs: Vec<_> = doc
            .items
            .iter()
            .filter(|i| matches!(i.kind, DocKind::Struct))
            .collect();
        let enums: Vec<_> = doc
            .items
            .iter()
            .filter(|i| matches!(i.kind, DocKind::Enum))
            .collect();
        let traits: Vec<_> = doc
            .items
            .iter()
            .filter(|i| matches!(i.kind, DocKind::Trait))
            .collect();
        let functions: Vec<_> = doc
            .items
            .iter()
            .filter(|i| matches!(i.kind, DocKind::Function))
            .collect();

        // Structs
        if !structs.is_empty() {
            content.push_str("## Structs\n\n");
            for item in structs {
                content.push_str(&format!("### {}\n\n", item.name));
                content.push_str(&format!("```vais\n{}\n```\n\n", item.signature));

                if !item.docs.is_empty() {
                    for doc_line in &item.docs {
                        content.push_str(doc_line);
                        content.push('\n');
                    }
                    content.push('\n');
                }

                if !item.examples.is_empty() {
                    content.push_str("**Examples:**\n\n");
                    for example in &item.examples {
                        content.push_str("```vais\n");
                        content.push_str(example);
                        content.push_str("```\n\n");
                    }
                }
            }
        }

        // Enums
        if !enums.is_empty() {
            content.push_str("## Enums\n\n");
            for item in enums {
                content.push_str(&format!("### {}\n\n", item.name));
                content.push_str(&format!("```vais\n{}\n```\n\n", item.signature));

                if !item.docs.is_empty() {
                    for doc_line in &item.docs {
                        content.push_str(doc_line);
                        content.push('\n');
                    }
                    content.push('\n');
                }
            }
        }

        // Traits
        if !traits.is_empty() {
            content.push_str("## Traits\n\n");
            for item in traits {
                content.push_str(&format!("### {}\n\n", item.name));
                content.push_str(&format!("```vais\n{}\n```\n\n", item.signature));

                if !item.docs.is_empty() {
                    for doc_line in &item.docs {
                        content.push_str(doc_line);
                        content.push('\n');
                    }
                    content.push('\n');
                }
            }
        }

        // Functions
        if !functions.is_empty() {
            content.push_str("## Functions\n\n");
            for item in functions {
                content.push_str(&format!("### {}\n\n", item.name));
                content.push_str(&format!("```vais\n{}\n```\n\n", item.signature));

                if !item.docs.is_empty() {
                    for doc_line in &item.docs {
                        content.push_str(doc_line);
                        content.push('\n');
                    }
                    content.push('\n');
                }

                if !item.params.is_empty() {
                    content.push_str("**Parameters:**\n\n");
                    for param in &item.params {
                        let mutability = if param.is_mut { " (mutable)" } else { "" };
                        content.push_str(&format!(
                            "- `{}`: {}{}\n",
                            param.name, param.ty, mutability
                        ));
                    }
                    content.push('\n');
                }

                if let Some(ret) = &item.returns {
                    content.push_str(&format!("**Returns:** `{}`\n\n", ret));
                }

                if !item.examples.is_empty() {
                    content.push_str("**Examples:**\n\n");
                    for example in &item.examples {
                        content.push_str("```vais\n");
                        content.push_str(example);
                        content.push_str("```\n\n");
                    }
                }
            }
        }

        let module_path = output.join(format!("{}.md", doc.name));
        fs::write(&module_path, &content)
            .map_err(|e| format!("Cannot write '{}': {}", module_path.display(), e))?;
    }

    let index_path = output.join("README.md");
    fs::write(&index_path, &index).map_err(|e| format!("Cannot write index: {}", e))?;

    Ok(())
}

/// Generate HTML documentation (Rustdoc style)
fn generate_html_docs(docs: &[ModuleDoc], output: &Path) -> Result<(), String> {
    // Modern Rustdoc-inspired styles
    let style = r#"
* { box-sizing: border-box; }
body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    margin: 0;
    padding: 0;
    background: #fff;
    color: #333;
}
.container { display: flex; min-height: 100vh; }
.sidebar {
    width: 250px;
    background: #f5f5f5;
    padding: 20px;
    overflow-y: auto;
    border-right: 1px solid #ddd;
}
.content {
    flex: 1;
    padding: 40px;
    max-width: 900px;
}
h1 {
    color: #333;
    font-size: 2em;
    margin: 0 0 0.5em 0;
    font-weight: 500;
}
h2 {
    color: #333;
    font-size: 1.5em;
    margin: 1.5em 0 0.5em 0;
    border-bottom: 1px solid #ddd;
    padding-bottom: 0.3em;
}
h3 {
    color: #333;
    font-size: 1.2em;
    margin: 1em 0 0.5em 0;
}
pre {
    background: #f5f5f5;
    padding: 15px;
    border-radius: 4px;
    overflow-x: auto;
    border: 1px solid #ddd;
    font-family: "Fira Code", "Courier New", monospace;
    font-size: 0.9em;
}
code {
    background: #f0f0f0;
    padding: 2px 6px;
    border-radius: 3px;
    font-family: "Fira Code", "Courier New", monospace;
    font-size: 0.9em;
}
pre code {
    background: none;
    padding: 0;
}
a {
    color: #0066cc;
    text-decoration: none;
}
a:hover {
    text-decoration: underline;
}
.sidebar h2 {
    font-size: 1em;
    margin: 0 0 0.5em 0;
    border: none;
    padding: 0;
}
.sidebar ul {
    list-style: none;
    padding: 0;
    margin: 0 0 1.5em 0;
}
.sidebar li {
    margin: 0.3em 0;
}
.badge {
    display: inline-block;
    padding: 2px 6px;
    font-size: 0.75em;
    font-weight: 600;
    border-radius: 3px;
    margin-left: 0.5em;
}
.badge.public { background: #d4edda; color: #155724; }
.badge.function { background: #d1ecf1; color: #0c5460; }
.badge.struct { background: #fff3cd; color: #856404; }
.badge.enum { background: #f8d7da; color: #721c24; }
.badge.trait { background: #e2e3e5; color: #383d41; }
.doc-block {
    margin: 1em 0;
    line-height: 1.6;
}
.param-list {
    margin: 0.5em 0;
}
.param-item {
    margin: 0.3em 0;
    padding-left: 1em;
}
.signature {
    font-weight: 500;
    color: #0066cc;
}
"#;

    // Generate index page
    let mut index = String::new();
    index.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    index.push_str("<meta charset=\"UTF-8\">\n");
    index.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    index.push_str("<title>Vais API Documentation</title>\n");
    index.push_str(&format!("<style>{}</style>\n", style));
    index.push_str("</head>\n<body>\n");
    index.push_str("<div class=\"container\">\n");

    // Sidebar
    index.push_str("<div class=\"sidebar\">\n");
    index.push_str("<h2>Modules</h2>\n<ul>\n");
    for doc in docs {
        index.push_str(&format!(
            "<li><a href=\"{}.html\">{}</a></li>\n",
            doc.name, doc.name
        ));
    }
    index.push_str("</ul>\n");
    index.push_str("</div>\n");

    // Content
    index.push_str("<div class=\"content\">\n");
    index.push_str("<h1>Vais API Documentation</h1>\n");
    index.push_str("<p>Welcome to the Vais API documentation. Select a module from the sidebar to get started.</p>\n");
    index.push_str("</div>\n");
    index.push_str("</div>\n</body>\n</html>\n");

    let index_path = output.join("index.html");
    fs::write(&index_path, &index).map_err(|e| format!("Cannot write index: {}", e))?;

    // Generate module pages
    for doc in docs {
        let mut content = String::new();
        content.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        content.push_str("<meta charset=\"UTF-8\">\n");
        content.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        content.push_str(&format!("<title>{} - Vais API</title>\n", doc.name));
        content.push_str(&format!("<style>{}</style>\n", style));
        content.push_str("</head>\n<body>\n");
        content.push_str("<div class=\"container\">\n");

        // Sidebar with navigation
        content.push_str("<div class=\"sidebar\">\n");
        content.push_str("<h2><a href=\"index.html\">← Back</a></h2>\n");

        // Build sections
        let structs: Vec<_> = doc.items.iter().filter(|i| matches!(i.kind, DocKind::Struct)).collect();
        let enums: Vec<_> = doc.items.iter().filter(|i| matches!(i.kind, DocKind::Enum)).collect();
        let traits: Vec<_> = doc.items.iter().filter(|i| matches!(i.kind, DocKind::Trait)).collect();
        let functions: Vec<_> = doc.items.iter().filter(|i| matches!(i.kind, DocKind::Function)).collect();

        if !structs.is_empty() {
            content.push_str("<h2>Structs</h2>\n<ul>\n");
            for item in &structs {
                content.push_str(&format!("<li><a href=\"#{}\">{}</a></li>\n", item.name, item.name));
            }
            content.push_str("</ul>\n");
        }

        if !enums.is_empty() {
            content.push_str("<h2>Enums</h2>\n<ul>\n");
            for item in &enums {
                content.push_str(&format!("<li><a href=\"#{}\">{}</a></li>\n", item.name, item.name));
            }
            content.push_str("</ul>\n");
        }

        if !traits.is_empty() {
            content.push_str("<h2>Traits</h2>\n<ul>\n");
            for item in &traits {
                content.push_str(&format!("<li><a href=\"#{}\">{}</a></li>\n", item.name, item.name));
            }
            content.push_str("</ul>\n");
        }

        if !functions.is_empty() {
            content.push_str("<h2>Functions</h2>\n<ul>\n");
            for item in &functions {
                content.push_str(&format!("<li><a href=\"#{}\">{}</a></li>\n", item.name, item.name));
            }
            content.push_str("</ul>\n");
        }

        content.push_str("</div>\n");

        // Content area
        content.push_str("<div class=\"content\">\n");
        content.push_str(&format!("<h1>Module: {}</h1>\n", doc.name));
        content.push_str(&format!("<p><code>{}</code></p>\n", doc.path.display()));

        // Render items
        for item in &doc.items {
            let kind_str = match item.kind {
                DocKind::Function => "function",
                DocKind::Struct => "struct",
                DocKind::Enum => "enum",
                DocKind::Trait => "trait",
                DocKind::Module => "module",
            };

            content.push_str(&format!("<h3 id=\"{}\"><span class=\"signature\">{}</span>", item.name, item.name));
            content.push_str(&format!("<span class=\"badge {}\">{}</span>", kind_str, kind_str));
            if item.visibility == Visibility::Public {
                content.push_str("<span class=\"badge public\">public</span>");
            }
            content.push_str("</h3>\n");

            content.push_str(&format!("<pre><code>{}</code></pre>\n", html_escape(&item.signature)));

            if !item.docs.is_empty() {
                content.push_str("<div class=\"doc-block\">\n");
                for doc_line in &item.docs {
                    content.push_str("<p>");
                    content.push_str(&html_escape(doc_line));
                    content.push_str("</p>\n");
                }
                content.push_str("</div>\n");
            }

            if !item.params.is_empty() {
                content.push_str("<h4>Parameters</h4>\n");
                content.push_str("<div class=\"param-list\">\n");
                for param in &item.params {
                    let mutability = if param.is_mut { " <em>(mutable)</em>" } else { "" };
                    content.push_str(&format!(
                        "<div class=\"param-item\"><code>{}</code>: {}{}</div>\n",
                        html_escape(&param.name),
                        html_escape(&param.ty),
                        mutability
                    ));
                }
                content.push_str("</div>\n");
            }

            if let Some(ret) = &item.returns {
                content.push_str(&format!("<h4>Returns</h4>\n<p><code>{}</code></p>\n", html_escape(ret)));
            }

            if !item.examples.is_empty() {
                content.push_str("<h4>Examples</h4>\n");
                for example in &item.examples {
                    content.push_str(&format!("<pre><code>{}</code></pre>\n", html_escape(example)));
                }
            }
        }

        content.push_str("</div>\n");
        content.push_str("</div>\n</body>\n</html>\n");

        let module_path = output.join(format!("{}.html", doc.name));
        fs::write(&module_path, &content)
            .map_err(|e| format!("Cannot write '{}': {}", module_path.display(), e))?;
    }

    Ok(())
}

/// HTML escape helper
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use vais_parser::parse;

    #[test]
    fn test_extract_doc_comments() {
        let source = r#"
/// This is a function
/// that adds two numbers
F add(a:i64,b:i64)->i64=a+b
"#;
        let lines: Vec<&str> = source.lines().collect();
        let docs = extract_doc_comments(&lines, 50); // Position of F
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0], "This is a function");
        assert_eq!(docs[1], "that adds two numbers");
    }

    #[test]
    fn test_parse_examples() {
        let docs = vec![
            "This is a test".to_string(),
            "```vais".to_string(),
            "F test()->i64=42".to_string(),
            "```".to_string(),
        ];
        let examples = parse_examples(&docs);
        assert_eq!(examples.len(), 1);
        assert!(examples[0].contains("F test()->i64=42"));
    }

    #[test]
    fn test_extract_function_doc() {
        let source = "F add(a:i64,b:i64)->i64=a+b";
        let ast = parse(source).unwrap();

        if let vais_ast::Item::Function(f) = &ast.items[0].node {
            let doc = extract_function_doc(f, vec!["Adds two numbers".to_string()]);
            assert_eq!(doc.name, "add");
            assert_eq!(doc.params.len(), 2);
            assert_eq!(doc.docs.len(), 1);
        } else {
            panic!("Expected function item");
        }
    }

    #[test]
    fn test_extract_struct_doc() {
        let source = "S Point{x:i64,y:i64}";
        let ast = parse(source).unwrap();

        if let vais_ast::Item::Struct(s) = &ast.items[0].node {
            let doc = extract_struct_doc(s, vec!["A 2D point".to_string()]);
            assert_eq!(doc.name, "Point");
            assert_eq!(doc.docs.len(), 1);
        } else {
            panic!("Expected struct item");
        }
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<test>"), "&lt;test&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("\"quote\""), "&quot;quote&quot;");
    }
}

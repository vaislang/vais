//! Documentation generator for Vais
//!
//! Generates Markdown and HTML documentation from Vais source files.

use colored::Colorize;
use std::fs;
use std::path::PathBuf;

use vais_ast::{Item, Module};
use vais_parser::parse;

/// Documentation item
struct DocItem {
    name: String,
    kind: DocKind,
    signature: String,
    #[allow(dead_code)]
    description: String,
    params: Vec<(String, String)>, // (name, type)
    returns: Option<String>,
}

enum DocKind {
    Function,
    Struct,
    Enum,
    Trait,
}

/// Module documentation
struct ModuleDoc {
    file: PathBuf,
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

    let mut all_docs = Vec::new();

    for file in &files {
        let source = fs::read_to_string(file)
            .map_err(|e| format!("Cannot read '{}': {}", file.display(), e))?;

        let ast = parse(&source)
            .map_err(|e| format!("Parse error in '{}': {}", file.display(), e))?;

        let doc = extract_documentation(file, &ast);
        all_docs.push(doc);
    }

    // Generate output
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

/// Collect all .vais files in a directory
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

/// Extract documentation from AST
fn extract_documentation(file: &PathBuf, ast: &Module) -> ModuleDoc {
    let mut items = Vec::new();

    for item in &ast.items {
        match &item.node {
            Item::Function(f) => {
                let params: Vec<(String, String)> = f
                    .params
                    .iter()
                    .map(|p| (p.name.node.clone(), format!("{}", p.ty.node)))
                    .collect();

                let returns = f.ret_type.as_ref().map(|t| format!("{}", t.node));

                let signature = format!(
                    "F {}({}) -> {}",
                    f.name.node,
                    params
                        .iter()
                        .map(|(n, t)| format!("{}: {}", n, t))
                        .collect::<Vec<_>>()
                        .join(", "),
                    returns.clone().unwrap_or_else(|| "()".to_string())
                );

                items.push(DocItem {
                    name: f.name.node.clone(),
                    kind: DocKind::Function,
                    signature,
                    description: String::new(),
                    params,
                    returns,
                });
            }
            Item::Struct(s) => {
                let fields: Vec<String> = s
                    .fields
                    .iter()
                    .map(|f| format!("{}: {}", f.name.node, f.ty.node))
                    .collect();

                let signature = format!("S {} {{ {} }}", s.name.node, fields.join(", "));

                items.push(DocItem {
                    name: s.name.node.clone(),
                    kind: DocKind::Struct,
                    signature,
                    description: String::new(),
                    params: vec![],
                    returns: None,
                });
            }
            Item::Enum(e) => {
                let variants: Vec<String> = e.variants.iter().map(|v| v.name.node.clone()).collect();

                let signature = format!("E {} {{ {} }}", e.name.node, variants.join(", "));

                items.push(DocItem {
                    name: e.name.node.clone(),
                    kind: DocKind::Enum,
                    signature,
                    description: String::new(),
                    params: vec![],
                    returns: None,
                });
            }
            Item::Trait(t) => {
                let methods: Vec<String> = t.methods.iter().map(|m| m.name.node.clone()).collect();

                let signature = format!("W {} {{ {} }}", t.name.node, methods.join(", "));

                items.push(DocItem {
                    name: t.name.node.clone(),
                    kind: DocKind::Trait,
                    signature,
                    description: String::new(),
                    params: vec![],
                    returns: None,
                });
            }
            _ => {}
        }
    }

    ModuleDoc {
        file: file.clone(),
        items,
    }
}

/// Generate markdown documentation
fn generate_markdown_docs(docs: &[ModuleDoc], output: &PathBuf) -> Result<(), String> {
    let mut index = String::new();
    index.push_str("# Vais API Documentation\n\n");
    index.push_str("## Modules\n\n");

    for doc in docs {
        let module_name = doc
            .file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        index.push_str(&format!("- [{}]({}.md)\n", module_name, module_name));

        // Generate module file
        let mut content = String::new();
        content.push_str(&format!("# Module: {}\n\n", module_name));
        content.push_str(&format!("Source: `{}`\n\n", doc.file.display()));

        // Group by kind
        let functions: Vec<_> = doc
            .items
            .iter()
            .filter(|i| matches!(i.kind, DocKind::Function))
            .collect();
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

        if !structs.is_empty() {
            content.push_str("## Structs\n\n");
            for item in structs {
                content.push_str(&format!("### `{}`\n\n", item.name));
                content.push_str(&format!("```vais\n{}\n```\n\n", item.signature));
            }
        }

        if !enums.is_empty() {
            content.push_str("## Enums\n\n");
            for item in enums {
                content.push_str(&format!("### `{}`\n\n", item.name));
                content.push_str(&format!("```vais\n{}\n```\n\n", item.signature));
            }
        }

        if !traits.is_empty() {
            content.push_str("## Traits\n\n");
            for item in traits {
                content.push_str(&format!("### `{}`\n\n", item.name));
                content.push_str(&format!("```vais\n{}\n```\n\n", item.signature));
            }
        }

        if !functions.is_empty() {
            content.push_str("## Functions\n\n");
            for item in functions {
                content.push_str(&format!("### `{}`\n\n", item.name));
                content.push_str(&format!("```vais\n{}\n```\n\n", item.signature));

                if !item.params.is_empty() {
                    content.push_str("**Parameters:**\n\n");
                    for (name, ty) in &item.params {
                        content.push_str(&format!("- `{}`: {}\n", name, ty));
                    }
                    content.push('\n');
                }

                if let Some(ret) = &item.returns {
                    content.push_str(&format!("**Returns:** {}\n\n", ret));
                }
            }
        }

        let module_path = output.join(format!("{}.md", module_name));
        fs::write(&module_path, &content)
            .map_err(|e| format!("Cannot write '{}': {}", module_path.display(), e))?;
    }

    let index_path = output.join("README.md");
    fs::write(&index_path, &index).map_err(|e| format!("Cannot write index: {}", e))?;

    Ok(())
}

/// Generate HTML documentation
fn generate_html_docs(docs: &[ModuleDoc], output: &PathBuf) -> Result<(), String> {
    let style = r#"
body { font-family: sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }
pre { background: #f4f4f4; padding: 10px; border-radius: 5px; overflow-x: auto; }
code { background: #f4f4f4; padding: 2px 5px; border-radius: 3px; }
h1, h2, h3 { color: #333; }
a { color: #0066cc; }
"#;

    let mut index = String::new();
    index.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    index.push_str("<title>Vais API Documentation</title>\n");
    index.push_str(&format!("<style>{}</style>\n", style));
    index.push_str("</head>\n<body>\n");
    index.push_str("<h1>Vais API Documentation</h1>\n");
    index.push_str("<h2>Modules</h2>\n<ul>\n");

    for doc in docs {
        let module_name = doc
            .file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        index.push_str(&format!(
            "<li><a href=\"{}.html\">{}</a></li>\n",
            module_name, module_name
        ));

        // Generate module file
        let mut content = String::new();
        content.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        content.push_str(&format!("<title>{} - Vais API</title>\n", module_name));
        content.push_str(&format!("<style>{}</style>\n", style));
        content.push_str("</head>\n<body>\n");
        content.push_str(&format!("<h1>Module: {}</h1>\n", module_name));
        content.push_str(&format!(
            "<p>Source: <code>{}</code></p>\n",
            doc.file.display()
        ));
        content.push_str("<p><a href=\"index.html\">&larr; Back to index</a></p>\n");

        for item in &doc.items {
            let kind = match item.kind {
                DocKind::Function => "Function",
                DocKind::Struct => "Struct",
                DocKind::Enum => "Enum",
                DocKind::Trait => "Trait",
            };

            content.push_str(&format!(
                "<h3>{}: <code>{}</code></h3>\n",
                kind, item.name
            ));
            content.push_str(&format!("<pre>{}</pre>\n", item.signature));
        }

        content.push_str("</body>\n</html>\n");

        let module_path = output.join(format!("{}.html", module_name));
        fs::write(&module_path, &content)
            .map_err(|e| format!("Cannot write '{}': {}", module_path.display(), e))?;
    }

    index.push_str("</ul>\n</body>\n</html>\n");

    let index_path = output.join("index.html");
    fs::write(&index_path, &index).map_err(|e| format!("Cannot write index: {}", e))?;

    Ok(())
}

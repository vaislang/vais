use super::*;

use std::fs;
use std::path::Path;

/// Generate HTML documentation (Rustdoc style)
pub(super) fn generate_html_docs(docs: &[ModuleDoc], output: &Path) -> Result<(), String> {
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
        content.push_str(
            "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        content.push_str(&format!("<title>{} - Vais API</title>\n", doc.name));
        content.push_str(&format!("<style>{}</style>\n", style));
        content.push_str("</head>\n<body>\n");
        content.push_str("<div class=\"container\">\n");

        // Sidebar with navigation
        content.push_str("<div class=\"sidebar\">\n");
        content.push_str("<h2><a href=\"index.html\">← Back</a></h2>\n");

        // Build sections
        let constants: Vec<_> = doc
            .items
            .iter()
            .filter(|i| matches!(i.kind, DocKind::Constant))
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
        let functions: Vec<_> = doc
            .items
            .iter()
            .filter(|i| matches!(i.kind, DocKind::Function))
            .collect();
        let extern_functions: Vec<_> = doc
            .items
            .iter()
            .filter(|i| matches!(i.kind, DocKind::ExternFunction))
            .collect();

        if !constants.is_empty() {
            content.push_str("<h2>Constants</h2>\n<ul>\n");
            for item in &constants {
                content.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>\n",
                    item.name, item.name
                ));
            }
            content.push_str("</ul>\n");
        }

        if !structs.is_empty() {
            content.push_str("<h2>Structs</h2>\n<ul>\n");
            for item in &structs {
                content.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>\n",
                    item.name, item.name
                ));
            }
            content.push_str("</ul>\n");
        }

        if !enums.is_empty() {
            content.push_str("<h2>Enums</h2>\n<ul>\n");
            for item in &enums {
                content.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>\n",
                    item.name, item.name
                ));
            }
            content.push_str("</ul>\n");
        }

        if !traits.is_empty() {
            content.push_str("<h2>Traits</h2>\n<ul>\n");
            for item in &traits {
                content.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>\n",
                    item.name, item.name
                ));
            }
            content.push_str("</ul>\n");
        }

        if !functions.is_empty() {
            content.push_str("<h2>Functions</h2>\n<ul>\n");
            for item in &functions {
                content.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>\n",
                    item.name, item.name
                ));
            }
            content.push_str("</ul>\n");
        }

        if !extern_functions.is_empty() {
            content.push_str("<h2>External Functions</h2>\n<ul>\n");
            for item in &extern_functions {
                content.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>\n",
                    item.name, item.name
                ));
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
                DocKind::Constant => "constant",
                DocKind::ExternFunction => "extern",
            };

            content.push_str(&format!(
                "<h3 id=\"{}\"><span class=\"signature\">{}</span>",
                item.name, item.name
            ));
            content.push_str(&format!(
                "<span class=\"badge {}\">{}</span>",
                kind_str, kind_str
            ));
            if item.visibility == Visibility::Public {
                content.push_str("<span class=\"badge public\">public</span>");
            }
            content.push_str("</h3>\n");

            content.push_str(&format!(
                "<pre><code>{}</code></pre>\n",
                html_escape(&item.signature)
            ));

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
                    let mutability = if param.is_mut {
                        " <em>(mutable)</em>"
                    } else {
                        ""
                    };
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
                content.push_str(&format!(
                    "<h4>Returns</h4>\n<p><code>{}</code></p>\n",
                    html_escape(ret)
                ));
            }

            if !item.examples.is_empty() {
                content.push_str("<h4>Examples</h4>\n");
                for example in &item.examples {
                    content.push_str(&format!(
                        "<pre><code>{}</code></pre>\n",
                        html_escape(example)
                    ));
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
pub(super) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

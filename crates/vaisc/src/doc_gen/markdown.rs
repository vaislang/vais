use super::*;

use std::fs;
use std::path::Path;

/// Generate markdown documentation
pub(super) fn generate_markdown_docs(docs: &[ModuleDoc], output: &Path) -> Result<(), String> {
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

        // Constants
        if !constants.is_empty() {
            content.push_str("## Constants\n\n");
            for item in constants {
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
                        content
                            .push_str(&format!("- `{}`: {}{}\n", param.name, param.ty, mutability));
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

        // External Functions
        if !extern_functions.is_empty() {
            content.push_str("## External Functions\n\n");
            for item in extern_functions {
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
                        content
                            .push_str(&format!("- `{}`: {}{}\n", param.name, param.ty, mutability));
                    }
                    content.push('\n');
                }

                if let Some(ret) = &item.returns {
                    content.push_str(&format!("**Returns:** `{}`\n\n", ret));
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

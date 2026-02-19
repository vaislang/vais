use super::*;

use std::path::Path;

/// Extract documentation from AST and source code
pub(super) fn extract_documentation(file: &Path, ast: &Module, source: &str) -> ModuleDoc {
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
            Item::Const(c) => {
                items.push(extract_const_doc(c, docs));
            }
            Item::ExternBlock(eb) => {
                // Extract each extern function from the block
                for f in &eb.functions {
                    // For extern functions, we'll extract docs from above the entire block
                    items.push(extract_extern_function_doc(f, docs.clone()));
                }
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
/// Supports both /// (Rust-style) and # (Vais-style) comments
pub(super) fn extract_doc_comments(lines: &[&str], start_pos: usize) -> Vec<String> {
    let mut docs = Vec::new();
    let mut byte_count = 0;

    for (i, line) in lines.iter().enumerate() {
        let line_end = byte_count + line.len() + 1; // +1 for newline

        if line_end > start_pos {
            // We've reached the item, look backwards for doc comments
            for j in (0..i).rev() {
                let prev_line = lines[j].trim();
                // Support both /// and # for comments
                if let Some(stripped) = prev_line.strip_prefix("///") {
                    docs.insert(0, stripped.trim().to_string());
                } else if let Some(stripped) = prev_line.strip_prefix('#') {
                    let comment = stripped.trim();
                    // Skip separator lines like "============"
                    if !comment.is_empty() && !comment.chars().all(|c| c == '=' || c == '-') {
                        docs.insert(0, comment.to_string());
                    }
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
pub(super) fn parse_examples(docs: &[String]) -> Vec<String> {
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
pub(super) fn extract_function_doc(f: &Function, docs: Vec<String>) -> DocItem {
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
        visibility: if f.is_pub {
            Visibility::Public
        } else {
            Visibility::Private
        },
    }
}

/// Extract struct documentation
pub(super) fn extract_struct_doc(s: &Struct, docs: Vec<String>) -> DocItem {
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
        visibility: if s.is_pub {
            Visibility::Public
        } else {
            Visibility::Private
        },
    }
}

/// Extract enum documentation
pub(super) fn extract_enum_doc(e: &Enum, docs: Vec<String>) -> DocItem {
    let generics: Vec<GenericDoc> = e
        .generics
        .iter()
        .map(|g| GenericDoc {
            name: g.name.node.clone(),
            bounds: g.bounds.iter().map(|b| b.node.clone()).collect(),
        })
        .collect();

    let variants: Vec<String> = e.variants.iter().map(|v| v.name.node.clone()).collect();

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
        visibility: if e.is_pub {
            Visibility::Public
        } else {
            Visibility::Private
        },
    }
}

/// Extract constant documentation
pub(super) fn extract_const_doc(c: &ConstDef, docs: Vec<String>) -> DocItem {
    let mut signature = String::new();
    if c.is_pub {
        signature.push_str("P ");
    }
    signature.push_str(&format!(
        "C {}: {} = {:?}",
        c.name.node, c.ty.node, c.value.node
    ));

    DocItem {
        name: c.name.node.clone(),
        kind: DocKind::Constant,
        signature,
        docs,
        params: vec![],
        returns: None,
        examples: vec![],
        _generics: vec![],
        visibility: if c.is_pub {
            Visibility::Public
        } else {
            Visibility::Private
        },
    }
}

/// Extract external function documentation
pub(super) fn extract_extern_function_doc(f: &ExternFunction, docs: Vec<String>) -> DocItem {
    let params: Vec<ParamDoc> = f
        .params
        .iter()
        .map(|p| ParamDoc {
            name: p.name.node.clone(),
            ty: format!("{}", p.ty.node),
            is_mut: false, // Extern functions typically don't have mut params
        })
        .collect();

    let returns = f.ret_type.as_ref().map(|t| format!("{}", t.node));

    let mut signature = String::new();
    signature.push_str("X F ");
    signature.push_str(&f.name.node);

    signature.push('(');
    for (i, p) in params.iter().enumerate() {
        if i > 0 {
            signature.push_str(", ");
        }
        signature.push_str(&format!("{}: {}", p.name, p.ty));
    }
    signature.push(')');

    if let Some(ret) = &returns {
        signature.push_str(&format!(" -> {}", ret));
    }

    DocItem {
        name: f.name.node.clone(),
        kind: DocKind::ExternFunction,
        signature,
        docs,
        params,
        returns,
        examples: vec![],
        _generics: vec![],
        visibility: Visibility::Public,
    }
}

/// Extract trait documentation
pub(super) fn extract_trait_doc(t: &Trait, docs: Vec<String>) -> DocItem {
    let generics: Vec<GenericDoc> = t
        .generics
        .iter()
        .map(|g| GenericDoc {
            name: g.name.node.clone(),
            bounds: g.bounds.iter().map(|b| b.node.clone()).collect(),
        })
        .collect();

    let methods: Vec<String> = t.methods.iter().map(|m| m.name.node.clone()).collect();

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
        visibility: if t.is_pub {
            Visibility::Public
        } else {
            Visibility::Private
        },
    }
}

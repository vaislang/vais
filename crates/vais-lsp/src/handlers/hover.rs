//! Hover handler implementation

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::backend::{position_in_range, get_builtin_hover, VaisBackend};

pub(crate) async fn handle_hover(
    backend: &VaisBackend,
    params: HoverParams,
) -> Result<Option<Hover>> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    if let Some(doc) = backend.documents.get(uri) {
        // Convert position to offset
        let line = position.line as usize;
        let col = position.character as usize;
        let line_start = doc.content.line_to_char(line);
        let offset = line_start + col;

        // Get identifier at position
        if let Some(ast) = &doc.ast {
            let ident = backend.get_identifier_at(uri, offset);

            // Check for builtin functions first
            if let Some(ref name) = ident {
                if let Some(hover) = get_builtin_hover(name) {
                    return Ok(Some(hover));
                }
            }

            // Find the item at the cursor position
            for item in &ast.items {
                match &item.node {
                    vais_ast::Item::Function(f) => {
                        let range = backend.span_to_range(&doc.content, &f.name.span);
                        if position_in_range(&position, &range)
                            || ident.as_ref() == Some(&f.name.node)
                        {
                            let params_str: Vec<String> = f
                                .params
                                .iter()
                                .map(|p| format!("{}: {:?}", p.name.node, p.ty.node))
                                .collect();

                            let ret_str = f
                                .ret_type
                                .as_ref()
                                .map(|t| format!(" -> {:?}", t.node))
                                .unwrap_or_default();

                            let is_async = if f.is_async { "A " } else { "" };
                            let signature = format!(
                                "{}F {}({}){}",
                                is_async,
                                f.name.node,
                                params_str.join(", "),
                                ret_str
                            );

                            return Ok(Some(Hover {
                                contents: HoverContents::Markup(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value: format!(
                                        "```vais\n{}\n```\n\nFunction defined in current file",
                                        signature
                                    ),
                                }),
                                range: Some(range),
                            }));
                        }
                    }
                    vais_ast::Item::Struct(s) => {
                        let range = backend.span_to_range(&doc.content, &s.name.span);
                        if position_in_range(&position, &range)
                            || ident.as_ref() == Some(&s.name.node)
                        {
                            let fields_str: Vec<String> = s
                                .fields
                                .iter()
                                .map(|f| format!("    {}: {:?}", f.name.node, f.ty.node))
                                .collect();

                            let generics = if s.generics.is_empty() {
                                String::new()
                            } else {
                                format!(
                                    "<{}>",
                                    s.generics
                                        .iter()
                                        .map(|g| g.name.node.clone())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                )
                            };

                            let signature = format!(
                                "S {}{} {{\n{}\n}}",
                                s.name.node,
                                generics,
                                fields_str.join(",\n")
                            );

                            return Ok(Some(Hover {
                                contents: HoverContents::Markup(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value: format!(
                                        "```vais\n{}\n```\n\nStruct with {} field(s)",
                                        signature,
                                        s.fields.len()
                                    ),
                                }),
                                range: Some(range),
                            }));
                        }
                    }
                    vais_ast::Item::Enum(e) => {
                        let range = backend.span_to_range(&doc.content, &e.name.span);
                        if position_in_range(&position, &range)
                            || ident.as_ref() == Some(&e.name.node)
                        {
                            let variants_str: Vec<String> = e
                                .variants
                                .iter()
                                .map(|v| format!("    {}", v.name.node))
                                .collect();

                            let generics = if e.generics.is_empty() {
                                String::new()
                            } else {
                                format!(
                                    "<{}>",
                                    e.generics
                                        .iter()
                                        .map(|g| g.name.node.clone())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                )
                            };

                            let signature = format!(
                                "E {}{} {{\n{}\n}}",
                                e.name.node,
                                generics,
                                variants_str.join(",\n")
                            );

                            return Ok(Some(Hover {
                                contents: HoverContents::Markup(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value: format!(
                                        "```vais\n{}\n```\n\nEnum with {} variant(s)",
                                        signature,
                                        e.variants.len()
                                    ),
                                }),
                                range: Some(range),
                            }));
                        }

                        // Check if hovering over a variant
                        for variant in &e.variants {
                            if ident.as_ref() == Some(&variant.name.node) {
                                return Ok(Some(Hover {
                                    contents: HoverContents::Markup(MarkupContent {
                                        kind: MarkupKind::Markdown,
                                        value: format!(
                                            "```vais\n{}::{}\n```\n\nVariant of enum `{}`",
                                            e.name.node, variant.name.node, e.name.node
                                        ),
                                    }),
                                    range: None,
                                }));
                            }
                        }
                    }
                    vais_ast::Item::Trait(t) => {
                        let range = backend.span_to_range(&doc.content, &t.name.span);
                        if position_in_range(&position, &range)
                            || ident.as_ref() == Some(&t.name.node)
                        {
                            let methods_str: Vec<String> = t
                                .methods
                                .iter()
                                .map(|m| {
                                    let params: Vec<String> = m
                                        .params
                                        .iter()
                                        .map(|p| format!("{}: {:?}", p.name.node, p.ty.node))
                                        .collect();
                                    let ret = m
                                        .ret_type
                                        .as_ref()
                                        .map(|r| format!(" -> {:?}", r.node))
                                        .unwrap_or_default();
                                    format!(
                                        "    F {}({}){}",
                                        m.name.node,
                                        params.join(", "),
                                        ret
                                    )
                                })
                                .collect();

                            let signature =
                                format!("W {} {{\n{}\n}}", t.name.node, methods_str.join("\n"));

                            return Ok(Some(Hover {
                                contents: HoverContents::Markup(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value: format!(
                                        "```vais\n{}\n```\n\nTrait with {} method(s)",
                                        signature,
                                        t.methods.len()
                                    ),
                                }),
                                range: Some(range),
                            }));
                        }
                    }
                    vais_ast::Item::Impl(impl_block) => {
                        // Check if hovering over a method in impl block
                        for method in &impl_block.methods {
                            if ident.as_ref() == Some(&method.node.name.node) {
                                let params_str: Vec<String> = method
                                    .node
                                    .params
                                    .iter()
                                    .map(|p| format!("{}: {:?}", p.name.node, p.ty.node))
                                    .collect();

                                let ret_str = method
                                    .node
                                    .ret_type
                                    .as_ref()
                                    .map(|t| format!(" -> {:?}", t.node))
                                    .unwrap_or_default();

                                let trait_info = impl_block
                                    .trait_name
                                    .as_ref()
                                    .map(|t| format!(" (impl {})", t.node))
                                    .unwrap_or_default();

                                let signature = format!(
                                    "F {}({}){}",
                                    method.node.name.node,
                                    params_str.join(", "),
                                    ret_str
                                );

                                let target_type = format!("{:?}", impl_block.target_type.node);

                                return Ok(Some(Hover {
                                    contents: HoverContents::Markup(MarkupContent {
                                        kind: MarkupKind::Markdown,
                                        value: format!(
                                            "```vais\n{}\n```\n\nMethod of `{}`{}",
                                            signature, target_type, trait_info
                                        ),
                                    }),
                                    range: None,
                                }));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(None)
}

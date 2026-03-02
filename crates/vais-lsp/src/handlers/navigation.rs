//! Navigation handlers (goto definition, references, symbols, etc.)

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::backend::VaisBackend;

pub(crate) async fn handle_goto_definition(
    backend: &VaisBackend,
    params: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    if let Some(doc) = backend.documents.get(uri) {
        // Convert position to offset
        let line = position.line as usize;
        let col = position.character as usize;
        let line_start = doc.content.line_to_char(line);
        let offset = line_start + col;
        drop(doc); // Release read lock

        // Use the new find_definition_at method (uses cache)
        if let Some(def) = backend.find_definition_at(uri, offset) {
            if let Some(doc) = backend.documents.get(uri) {
                let range = backend.span_to_range(&doc.content, &def.span);
                return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range,
                })));
            }
        }

        // If not found in current file, try cross-module navigation
        if let Some(symbol_name) = backend.get_identifier_at(uri, offset) {
            // Search across all open documents
            if let Some(location) = find_in_workspace(backend, uri, &symbol_name) {
                return Ok(Some(GotoDefinitionResponse::Scalar(location)));
            }

            // Try to resolve via import (U) statements
            if let Some(doc) = backend.documents.get(uri) {
                if let Some(ast) = &doc.ast {
                    if let Some(location) = resolve_import_target(backend, ast, &symbol_name) {
                        return Ok(Some(GotoDefinitionResponse::Scalar(location)));
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Search for a symbol definition across all open workspace documents
fn find_in_workspace(
    backend: &VaisBackend,
    current_uri: &Url,
    symbol_name: &str,
) -> Option<Location> {
    for entry in backend.documents.iter() {
        let doc_uri = entry.key();
        // Skip current document (already searched)
        if doc_uri == current_uri {
            continue;
        }
        let doc = entry.value();
        if let Some(ast) = &doc.ast {
            for item in &ast.items {
                let (name, span) = match &item.node {
                    vais_ast::Item::Function(f) => (&f.name.node, f.name.span),
                    vais_ast::Item::Struct(s) => (&s.name.node, s.name.span),
                    vais_ast::Item::Enum(e) => (&e.name.node, e.name.span),
                    vais_ast::Item::Trait(t) => (&t.name.node, t.name.span),
                    vais_ast::Item::Macro(m) => (&m.name.node, m.name.span),
                    _ => continue,
                };
                if name == symbol_name {
                    let range = backend.span_to_range(&doc.content, &span);
                    return Some(Location {
                        uri: doc_uri.clone(),
                        range,
                    });
                }

                // Also check impl block methods
                if let vais_ast::Item::Impl(impl_block) = &item.node {
                    for method in &impl_block.methods {
                        if method.node.name.node == symbol_name {
                            let range = backend.span_to_range(&doc.content, &method.node.name.span);
                            return Some(Location {
                                uri: doc_uri.clone(),
                                range,
                            });
                        }
                    }
                }
            }
        }
    }
    None
}

/// Try to resolve a symbol through import (U) statements
fn resolve_import_target(
    backend: &VaisBackend,
    ast: &vais_ast::Module,
    symbol_name: &str,
) -> Option<Location> {
    for item in &ast.items {
        if let vais_ast::Item::Use(use_stmt) = &item.node {
            // Build module path string from path segments
            let path_str: String = use_stmt
                .path
                .iter()
                .map(|s| s.node.as_str())
                .collect::<Vec<_>>()
                .join("/");

            // Check if the import path ends with the symbol name
            let last_segment = use_stmt.path.last().map(|s| s.node.as_str()).unwrap_or("");
            if last_segment == symbol_name {
                // Try to find the module file and look up the definition
                if let Some(location) = find_in_module_file(backend, &path_str, symbol_name) {
                    return Some(location);
                }
            }

            // Check selective imports: U mod.{A, B}
            if let Some(ref items) = use_stmt.items {
                for imported_item in items {
                    if imported_item.node == symbol_name {
                        if let Some(location) = find_in_module_file(backend, &path_str, symbol_name)
                        {
                            return Some(location);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Try to find a symbol in a module file based on import path
fn find_in_module_file(
    backend: &VaisBackend,
    module_path: &str,
    symbol_name: &str,
) -> Option<Location> {
    // Convert module path (e.g., "std/math") to possible file URIs
    // Check all open documents for matching module paths
    for entry in backend.documents.iter() {
        let doc_uri = entry.key();
        let uri_str = doc_uri.to_string();

        // Check if the URI path contains the module path
        let module_file = format!("{}.vais", module_path);
        if uri_str.contains(&module_file) || uri_str.ends_with(&format!("/{}.vais", module_path)) {
            let doc = entry.value();
            if let Some(ast) = &doc.ast {
                for item in &ast.items {
                    let (name, span) = match &item.node {
                        vais_ast::Item::Function(f) => (&f.name.node, f.name.span),
                        vais_ast::Item::Struct(s) => (&s.name.node, s.name.span),
                        vais_ast::Item::Enum(e) => (&e.name.node, e.name.span),
                        vais_ast::Item::Trait(t) => (&t.name.node, t.name.span),
                        _ => continue,
                    };
                    if name == symbol_name {
                        let range = backend.span_to_range(&doc.content, &span);
                        return Some(Location {
                            uri: doc_uri.clone(),
                            range,
                        });
                    }
                }
            }
        }
    }
    None
}

pub(crate) async fn handle_references(
    backend: &VaisBackend,
    params: ReferenceParams,
) -> Result<Option<Vec<Location>>> {
    let uri = &params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;

    if let Some(doc) = backend.documents.get(uri) {
        // Convert position to offset
        let line = position.line as usize;
        let col = position.character as usize;
        let line_start = doc.content.line_to_char(line);
        let offset = line_start + col;
        drop(doc); // Release read lock

        // Get the symbol name at the position (uses cache)
        if let Some(symbol_name) = backend.get_identifier_at(uri, offset) {
            let spans = backend.find_all_references(uri, &symbol_name);

            if let Some(doc) = backend.documents.get(uri) {
                let locations: Vec<Location> = spans
                    .iter()
                    .map(|span| Location {
                        uri: uri.clone(),
                        range: backend.span_to_range(&doc.content, span),
                    })
                    .collect();

                if !locations.is_empty() {
                    return Ok(Some(locations));
                }
            }
        }
    }

    Ok(None)
}

#[allow(deprecated)]
pub(crate) async fn handle_document_symbol(
    backend: &VaisBackend,
    params: DocumentSymbolParams,
) -> Result<Option<DocumentSymbolResponse>> {
    let uri = &params.text_document.uri;

    if let Some(doc) = backend.documents.get(uri) {
        if let Some(ast) = &doc.ast {
            let mut symbols = vec![];

            for item in &ast.items {
                match &item.node {
                    vais_ast::Item::Function(f) => {
                        let range = backend.span_to_range(&doc.content, &item.span);
                        #[allow(deprecated)]
                        symbols.push(SymbolInformation {
                            name: f.name.node.clone(),
                            kind: SymbolKind::FUNCTION,
                            location: Location {
                                uri: uri.clone(),
                                range,
                            },
                            tags: None,
                            container_name: None,
                            deprecated: None,
                        });
                    }
                    vais_ast::Item::Struct(s) => {
                        let range = backend.span_to_range(&doc.content, &item.span);
                        #[allow(deprecated)]
                        symbols.push(SymbolInformation {
                            name: s.name.node.clone(),
                            kind: SymbolKind::STRUCT,
                            location: Location {
                                uri: uri.clone(),
                                range,
                            },
                            tags: None,
                            container_name: None,
                            deprecated: None,
                        });
                    }
                    vais_ast::Item::Enum(e) => {
                        let range = backend.span_to_range(&doc.content, &item.span);
                        #[allow(deprecated)]
                        symbols.push(SymbolInformation {
                            name: e.name.node.clone(),
                            kind: SymbolKind::ENUM,
                            location: Location {
                                uri: uri.clone(),
                                range,
                            },
                            tags: None,
                            container_name: None,
                            deprecated: None,
                        });
                    }
                    vais_ast::Item::Trait(t) => {
                        let range = backend.span_to_range(&doc.content, &item.span);
                        #[allow(deprecated)]
                        symbols.push(SymbolInformation {
                            name: t.name.node.clone(),
                            kind: SymbolKind::INTERFACE,
                            location: Location {
                                uri: uri.clone(),
                                range,
                            },
                            tags: None,
                            container_name: None,
                            deprecated: None,
                        });
                    }
                    _ => {}
                }
            }

            if !symbols.is_empty() {
                return Ok(Some(DocumentSymbolResponse::Flat(symbols)));
            }
        }
    }

    Ok(None)
}

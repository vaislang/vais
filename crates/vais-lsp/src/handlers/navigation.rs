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
    }

    Ok(None)
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

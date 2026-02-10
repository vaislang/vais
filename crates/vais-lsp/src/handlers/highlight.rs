//! Document highlight handler implementation

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::backend::VaisBackend;

pub(crate) async fn handle_document_highlight(
    backend: &VaisBackend,
    params: DocumentHighlightParams,
) -> Result<Option<Vec<DocumentHighlight>>> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    if let Some(doc) = backend.documents.get(uri) {
        // Convert position to offset
        let line = position.line as usize;
        let col = position.character as usize;
        let line_start = doc.content.line_to_char(line);
        let offset = line_start + col;
        drop(doc); // Release read lock

        // Get the identifier at the cursor position
        if let Some(symbol_name) = backend.get_identifier_at(uri, offset) {
            // Get the symbol cache
            if let Some(cache) = backend.get_symbol_cache(uri) {
                let mut highlights = Vec::new();

                if let Some(doc) = backend.documents.get(uri) {
                    // Add highlights for all definitions (Write kind)
                    for def in &cache.definitions {
                        if def.name == symbol_name {
                            let range = backend.span_to_range(&doc.content, &def.span);
                            highlights.push(DocumentHighlight {
                                range,
                                kind: Some(DocumentHighlightKind::WRITE),
                            });
                        }
                    }

                    // Add highlights for all references (Read kind)
                    for ref_item in &cache.references {
                        if ref_item.name == symbol_name {
                            let range = backend.span_to_range(&doc.content, &ref_item.span);
                            highlights.push(DocumentHighlight {
                                range,
                                kind: Some(DocumentHighlightKind::READ),
                            });
                        }
                    }

                    if !highlights.is_empty() {
                        return Ok(Some(highlights));
                    }
                }
            }
        }
    }

    Ok(None)
}

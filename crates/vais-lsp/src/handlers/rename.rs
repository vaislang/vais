//! Rename symbol handlers (textDocument/rename, textDocument/prepareRename)

use std::collections::HashMap;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::backend::VaisBackend;

/// Handle textDocument/prepareRename
///
/// Returns the range of the symbol under the cursor so the editor can
/// populate its rename input field, or `None` if the position is not
/// on a renameable symbol.
pub(crate) async fn handle_prepare_rename(
    backend: &VaisBackend,
    params: TextDocumentPositionParams,
) -> Result<Option<PrepareRenameResponse>> {
    let uri = &params.text_document.uri;
    let position = params.position;

    if let Some(doc) = backend.documents.get(uri) {
        let line = position.line as usize;
        let col = position.character as usize;
        let line_start = doc.content.line_to_char(line);
        let offset = line_start + col;
        drop(doc);

        if let Some(symbol_name) = backend.get_identifier_at(uri, offset) {
            if let Some(cache) = backend.get_symbol_cache(uri) {
                if let Some(doc) = backend.documents.get(uri) {
                    // Check definitions first
                    for d in &cache.definitions {
                        if d.span.start <= offset && offset <= d.span.end {
                            let range = backend.span_to_range(&doc.content, &d.span);
                            return Ok(Some(PrepareRenameResponse::RangeWithPlaceholder {
                                range,
                                placeholder: symbol_name,
                            }));
                        }
                    }

                    // Then check references
                    for r in &cache.references {
                        if r.span.start <= offset && offset <= r.span.end {
                            let range = backend.span_to_range(&doc.content, &r.span);
                            return Ok(Some(PrepareRenameResponse::RangeWithPlaceholder {
                                range,
                                placeholder: symbol_name,
                            }));
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Handle textDocument/rename
///
/// Finds all occurrences of the symbol across all open workspace documents
/// and returns a `WorkspaceEdit` that renames every occurrence atomically.
pub(crate) async fn handle_rename(
    backend: &VaisBackend,
    params: RenameParams,
) -> Result<Option<WorkspaceEdit>> {
    let uri = &params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;
    let new_name = &params.new_name;

    let symbol_name = {
        let doc = match backend.documents.get(uri) {
            Some(d) => d,
            None => return Ok(None),
        };
        let line = position.line as usize;
        let col = position.character as usize;
        let line_start = doc.content.line_to_char(line);
        let offset = line_start + col;
        drop(doc);
        match backend.get_identifier_at(uri, offset) {
            Some(n) => n,
            None => return Ok(None),
        }
    };

    let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();

    // Collect edits from every open document
    for entry in backend.documents.iter() {
        let doc_uri = entry.key().clone();
        let doc = entry.value();

        let spans = backend.find_all_references(&doc_uri, &symbol_name);
        if spans.is_empty() {
            continue;
        }

        let edits: Vec<TextEdit> = spans
            .iter()
            .map(|span| TextEdit {
                range: backend.span_to_range(&doc.content, span),
                new_text: new_name.clone(),
            })
            .collect();

        changes.insert(doc_uri, edits);
    }

    if changes.is_empty() {
        return Ok(None);
    }

    Ok(Some(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    }))
}

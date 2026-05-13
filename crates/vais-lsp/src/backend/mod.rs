//! LSP Backend implementation for Vais
//!
//! # Submodules
//!
//! - `builtins`: Builtin function hover data and utility functions
//! - `types`: Backend data structures (symbols, hints, folding ranges)
//! - `language_server`: LanguageServer trait implementation

mod builtins;
mod language_server;
mod types;

#[cfg(test)]
mod tests;

// Re-export for use by other LSP modules via `crate::backend::`
pub(crate) use builtins::{get_builtin_hover, position_in_range};
pub(crate) use types::*;

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use vais_ast::formatter::{FormatConfig, Formatter};
use vais_ast::{Item, Module, Span, Type};
use vais_parser::parse;

use crate::diagnostics::parse_error_to_diagnostic;
use crate::semantic::get_semantic_tokens;

/// Document state
pub struct Document {
    pub content: Rope,
    pub ast: Option<Module>,
    pub version: i32,
    /// Cached symbol information (invalidated on document change)
    pub(crate) symbol_cache: Option<SymbolCache>,
}

/// Vais Language Server Backend
pub struct VaisBackend {
    pub(crate) client: Client,
    pub(crate) documents: DashMap<Url, Document>,
}

impl VaisBackend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
        }
    }

    /// Parse a document and update its AST
    async fn parse_document(&self, uri: &Url, content: &str) {
        match parse(content) {
            Ok(module) => {
                if let Some(mut doc) = self.documents.get_mut(uri) {
                    doc.ast = Some(module);
                    // Invalidate symbol cache on parse
                    doc.symbol_cache = None;
                }
                // Clear diagnostics on successful parse
                self.client
                    .publish_diagnostics(uri.clone(), vec![], None)
                    .await;
            }
            Err(err) => {
                // Convert parse error to diagnostic using helper function
                let diagnostic = parse_error_to_diagnostic(&err, content);
                self.client
                    .publish_diagnostics(uri.clone(), vec![diagnostic], None)
                    .await;
            }
        }
    }

    /// Get or build symbol cache for a document
    pub(crate) fn get_symbol_cache(&self, uri: &Url) -> Option<SymbolCache> {
        if let Some(doc) = self.documents.get(uri) {
            // Check if cache is valid
            if let Some(cache) = &doc.symbol_cache {
                if cache.version == doc.version {
                    return Some(cache.clone());
                }
            }

            // Build new cache if AST is available
            if let Some(ast) = &doc.ast {
                let definitions = self.collect_definitions(ast);
                let references = self.collect_references(ast);
                let call_graph = self.build_call_graph(ast);
                let cache = SymbolCache {
                    version: doc.version,
                    definitions,
                    references,
                    call_graph,
                };

                // Store cache in document
                drop(doc); // Release read lock
                if let Some(mut doc_mut) = self.documents.get_mut(uri) {
                    doc_mut.symbol_cache = Some(cache.clone());
                }

                return Some(cache);
            }
        }

        None
    }

    pub(crate) fn offset_to_position(&self, rope: &Rope, offset: usize) -> Position {
        let line = rope.char_to_line(offset.min(rope.len_chars()));
        let line_start = rope.line_to_char(line);
        let col = offset.saturating_sub(line_start);
        Position::new(line as u32, col as u32)
    }

    pub(crate) fn span_to_range(&self, rope: &Rope, span: &Span) -> Range {
        Range {
            start: self.offset_to_position(rope, span.start),
            end: self.offset_to_position(rope, span.end),
        }
    }
}

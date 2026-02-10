//! LSP Backend implementation for Vais

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use vais_ast::{Item, Module, Span, Type};
use vais_codegen::formatter::{FormatConfig, Formatter};
use vais_parser::parse;

use crate::diagnostics::parse_error_to_diagnostic;
use crate::semantic::get_semantic_tokens;

/// Call graph entry representing function call relationships
#[derive(Debug, Clone)]
pub(crate) struct CallGraphEntry {
    /// Caller function name
    pub(crate) caller: String,
    /// Caller span
    pub(crate) caller_span: Span,
    /// Callee function name
    pub(crate) callee: String,
    /// Call site span
    pub(crate) call_span: Span,
}

/// Check if a position is within a range
pub(crate) fn position_in_range(position: &Position, range: &Range) -> bool {
    if position.line < range.start.line || position.line > range.end.line {
        return false;
    }
    if position.line == range.start.line && position.character < range.start.character {
        return false;
    }
    if position.line == range.end.line && position.character > range.end.character {
        return false;
    }
    true
}

/// Get hover information for builtin functions
pub(crate) fn get_builtin_hover(name: &str) -> Option<Hover> {
    let info = match name {
        "puts" => Some(("fn(str) -> i64", "Print a string to stdout with newline")),
        "putchar" => Some(("fn(i64) -> i64", "Print a single character (ASCII value)")),
        "print_i64" => Some(("fn(i64) -> i64", "Print a 64-bit signed integer")),
        "print_f64" => Some(("fn(f64) -> i64", "Print a 64-bit floating point number")),
        "malloc" => Some((
            "fn(i64) -> i64",
            "Allocate `size` bytes of heap memory, returns pointer",
        )),
        "free" => Some(("fn(i64) -> i64", "Free heap memory at pointer")),
        "memcpy" => Some((
            "fn(i64, i64, i64) -> i64",
            "Copy `n` bytes from `src` to `dst`",
        )),
        "strlen" => Some(("fn(i64) -> i64", "Get length of null-terminated string")),
        "load_i64" => Some((
            "fn(i64) -> i64",
            "Load a 64-bit integer from memory address",
        )),
        "store_i64" => Some((
            "fn(i64, i64) -> i64",
            "Store a 64-bit integer to memory address",
        )),
        "load_byte" => Some(("fn(i64) -> i64", "Load a single byte from memory address")),
        "store_byte" => Some((
            "fn(i64, i64) -> i64",
            "Store a single byte to memory address",
        )),
        "sqrt" => Some(("fn(f64) -> f64", "Square root (from std/math)")),
        "sin" => Some(("fn(f64) -> f64", "Sine function (from std/math)")),
        "cos" => Some(("fn(f64) -> f64", "Cosine function (from std/math)")),
        "tan" => Some(("fn(f64) -> f64", "Tangent function (from std/math)")),
        "pow" => Some(("fn(f64, f64) -> f64", "Power function x^y (from std/math)")),
        "log" => Some(("fn(f64) -> f64", "Natural logarithm (from std/math)")),
        "exp" => Some(("fn(f64) -> f64", "Exponential e^x (from std/math)")),
        "floor" => Some(("fn(f64) -> f64", "Round down to integer (from std/math)")),
        "ceil" => Some(("fn(f64) -> f64", "Round up to integer (from std/math)")),
        "round" => Some(("fn(f64) -> f64", "Round to nearest integer (from std/math)")),
        "abs" => Some(("fn(f64) -> f64", "Absolute value for f64 (from std/math)")),
        "abs_i64" => Some(("fn(i64) -> i64", "Absolute value for i64 (from std/math)")),
        "min" => Some((
            "fn(f64, f64) -> f64",
            "Minimum of two f64 values (from std/math)",
        )),
        "max" => Some((
            "fn(f64, f64) -> f64",
            "Maximum of two f64 values (from std/math)",
        )),
        "PI" => Some((
            "const f64 = 3.14159...",
            "Mathematical constant π (from std/math)",
        )),
        "TAU" => Some((
            "const f64 = 6.28318...",
            "Mathematical constant τ = 2π (from std/math)",
        )),
        "read_i64" => Some(("fn() -> i64", "Read integer from stdin (from std/io)")),
        "read_f64" => Some(("fn() -> f64", "Read float from stdin (from std/io)")),
        "read_line" => Some(("fn(i64, i64) -> i64", "Read line into buffer (from std/io)")),
        "read_char" => Some(("fn() -> i64", "Read single character (from std/io)")),
        _ => None,
    };

    info.map(|(sig, doc)| Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("```vais\n{}\n```\n\n{}\n\n*Built-in function*", sig, doc),
        }),
        range: None,
    })
}

/// Symbol definition information
#[derive(Debug, Clone)]
pub(crate) struct SymbolDef {
    pub(crate) name: String,
    pub(crate) kind: SymbolKind,
    pub(crate) span: Span,
}

/// Symbol reference information
#[derive(Debug, Clone)]
pub(crate) struct SymbolRef {
    pub(crate) name: String,
    pub(crate) span: Span,
}

/// Cached symbol information for a document
#[derive(Debug, Clone)]
pub(crate) struct SymbolCache {
    /// Document version this cache is valid for
    pub(crate) version: i32,
    /// All symbol definitions in the document
    pub(crate) definitions: Vec<SymbolDef>,
    /// All symbol references in the document
    pub(crate) references: Vec<SymbolRef>,
    /// Call graph entries for call hierarchy
    pub(crate) call_graph: Vec<CallGraphEntry>,
}

/// Inlay hint information
#[derive(Debug, Clone)]
pub(crate) struct InlayHintInfo {
    /// Position in the source
    pub(crate) position: usize,
    /// Hint label
    pub(crate) label: String,
    /// Hint kind (Type, Parameter)
    pub(crate) kind: InlayHintKind,
}

/// Folding range information
#[derive(Debug, Clone)]
pub(crate) struct FoldingRangeInfo {
    /// Start line (0-indexed)
    pub(crate) start_line: u32,
    /// End line (0-indexed)
    pub(crate) end_line: u32,
    /// Kind of folding range
    pub(crate) kind: Option<FoldingRangeKind>,
}

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

    /// Get position offset in text
    pub(crate) fn offset_to_position(&self, rope: &Rope, offset: usize) -> Position {
        let line = rope.char_to_line(offset.min(rope.len_chars()));
        let line_start = rope.line_to_char(line);
        let col = offset.saturating_sub(line_start);
        Position::new(line as u32, col as u32)
    }

    /// Convert span to LSP range
    pub(crate) fn span_to_range(&self, rope: &Rope, span: &Span) -> Range {
        Range {
            start: self.offset_to_position(rope, span.start),
            end: self.offset_to_position(rope, span.end),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for VaisBackend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "vais-lsp".to_string(),
                version: Some("0.0.1".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: Some(vec![",".to_string()]),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::FUNCTION,
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::STRUCT,
                                    SemanticTokenType::ENUM,
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::TYPE,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::COMMENT,
                                    SemanticTokenType::PARAMETER,
                                ],
                                token_modifiers: vec![],
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: None,
                            ..Default::default()
                        },
                    ),
                ),
                document_highlight_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                })),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                // New capabilities
                inlay_hint_provider: Some(OneOf::Left(true)),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                call_hierarchy_provider: Some(CallHierarchyServerCapability::Simple(true)),
                document_link_provider: Some(DocumentLinkOptions {
                    resolve_provider: Some(false),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_range_formatting_provider: Some(OneOf::Left(true)),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(false),
                }),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Vais LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;

        self.documents.insert(
            uri.clone(),
            Document {
                content: Rope::from_str(&content),
                ast: None,
                version,
                symbol_cache: None,
            },
        );

        self.parse_document(&uri, &content).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(change) = params.content_changes.into_iter().next() {
            let content = change.text;

            if let Some(mut doc) = self.documents.get_mut(&uri) {
                doc.content = Rope::from_str(&content);
                doc.version = params.text_document.version;
                // Invalidate symbol cache on content change
                doc.symbol_cache = None;
            }

            self.parse_document(&uri, &content).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        crate::handlers::hover::handle_hover(self, params).await
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        crate::handlers::completion::handle_completion(self, params).await
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        crate::handlers::signature::handle_signature_help(self, params).await
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        crate::handlers::navigation::handle_goto_definition(self, params).await
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        crate::handlers::navigation::handle_references(self, params).await
    }

    #[allow(deprecated)]
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        crate::handlers::navigation::handle_document_symbol(self, params).await
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = &params.text_document.uri;

        if let Some(doc) = self.documents.get(uri) {
            let tokens = get_semantic_tokens(&doc.content.to_string());
            return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: tokens,
            })));
        }

        Ok(None)
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        crate::handlers::highlight::handle_document_highlight(self, params).await
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let uri = &params.text_document.uri;
        let position = params.position;

        if let Some(doc) = self.documents.get(uri) {
            // Convert position to offset
            let line = position.line as usize;
            let col = position.character as usize;
            let line_start = doc.content.line_to_char(line);
            let offset = line_start + col;
            drop(doc); // Release read lock

            // Check if we're on a renameable symbol
            if let Some(symbol_name) = self.get_identifier_at(uri, offset) {
                // Find the exact span of the symbol at the cursor using cache
                if let Some(cache) = self.get_symbol_cache(uri) {
                    let defs = &cache.definitions;
                    let refs = &cache.references;

                    if let Some(doc) = self.documents.get(uri) {
                        // Check definitions
                        for d in defs {
                            if d.span.start <= offset && offset <= d.span.end {
                                let range = self.span_to_range(&doc.content, &d.span);
                                return Ok(Some(PrepareRenameResponse::RangeWithPlaceholder {
                                    range,
                                    placeholder: symbol_name.clone(),
                                }));
                            }
                        }

                        // Check references
                        for r in refs {
                            if r.span.start <= offset && offset <= r.span.end {
                                let range = self.span_to_range(&doc.content, &r.span);
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

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        if let Some(doc) = self.documents.get(uri) {
            // Convert position to offset
            let line = position.line as usize;
            let col = position.character as usize;
            let line_start = doc.content.line_to_char(line);
            let offset = line_start + col;
            drop(doc); // Release read lock

            // Get the symbol name at the position (uses cache)
            if let Some(symbol_name) = self.get_identifier_at(uri, offset) {
                // Find all references to this symbol
                let spans = self.find_all_references(uri, &symbol_name);

                if !spans.is_empty() {
                    if let Some(doc) = self.documents.get(uri) {
                        // Create text edits for all occurrences
                        let text_edits: Vec<TextEdit> = spans
                            .iter()
                            .map(|span| TextEdit {
                                range: self.span_to_range(&doc.content, span),
                                new_text: new_name.clone(),
                            })
                            .collect();

                        // Create workspace edit
                        let mut changes = std::collections::HashMap::new();
                        changes.insert(uri.clone(), text_edits);

                        return Ok(Some(WorkspaceEdit {
                            changes: Some(changes),
                            document_changes: None,
                            change_annotations: None,
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        crate::handlers::code_action::handle_code_action(self, params).await
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let uri = &params.text_document.uri;

        if let Some(doc) = self.documents.get(uri) {
            if let Some(ast) = &doc.ast {
                let hint_infos = self.collect_inlay_hints(ast, &doc.content);

                let hints: Vec<InlayHint> = hint_infos
                    .iter()
                    .map(|info| {
                        let position = self.offset_to_position(&doc.content, info.position);
                        InlayHint {
                            position,
                            label: InlayHintLabel::String(info.label.clone()),
                            kind: Some(info.kind),
                            text_edits: None,
                            tooltip: None,
                            padding_left: Some(true),
                            padding_right: None,
                            data: None,
                        }
                    })
                    .collect();

                if !hints.is_empty() {
                    return Ok(Some(hints));
                }
            }
        }

        Ok(None)
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let uri = &params.text_document.uri;

        if let Some(doc) = self.documents.get(uri) {
            if let Some(ast) = &doc.ast {
                let range_infos = self.collect_folding_ranges(ast, &doc.content);

                let ranges: Vec<FoldingRange> = range_infos
                    .iter()
                    .map(|info| FoldingRange {
                        start_line: info.start_line,
                        start_character: None,
                        end_line: info.end_line,
                        end_character: None,
                        kind: info.kind.clone(),
                        collapsed_text: None,
                    })
                    .collect();

                if !ranges.is_empty() {
                    return Ok(Some(ranges));
                }
            }
        }

        Ok(None)
    }

    async fn prepare_call_hierarchy(
        &self,
        params: CallHierarchyPrepareParams,
    ) -> Result<Option<Vec<CallHierarchyItem>>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(uri) {
            // Convert position to offset
            let line = position.line as usize;
            let col = position.character as usize;
            let line_start = doc.content.line_to_char(line);
            let offset = line_start + col;
            drop(doc); // Release read lock

            // Check if we're on a function definition or call
            if let Some(cache) = self.get_symbol_cache(uri) {
                // Check definitions for functions
                for def in &cache.definitions {
                    if def.kind == SymbolKind::FUNCTION
                        && def.span.start <= offset
                        && offset <= def.span.end
                    {
                        if let Some(doc) = self.documents.get(uri) {
                            let range = self.span_to_range(&doc.content, &def.span);
                            return Ok(Some(vec![CallHierarchyItem {
                                name: def.name.clone(),
                                kind: SymbolKind::FUNCTION,
                                tags: None,
                                detail: None,
                                uri: uri.clone(),
                                range,
                                selection_range: range,
                                data: None,
                            }]));
                        }
                    }
                }

                // Check references for function calls
                for r in &cache.references {
                    if r.span.start <= offset && offset <= r.span.end {
                        // Check if this reference is a function
                        for def in &cache.definitions {
                            if def.name == r.name && def.kind == SymbolKind::FUNCTION {
                                if let Some(doc) = self.documents.get(uri) {
                                    let range = self.span_to_range(&doc.content, &def.span);
                                    return Ok(Some(vec![CallHierarchyItem {
                                        name: def.name.clone(),
                                        kind: SymbolKind::FUNCTION,
                                        tags: None,
                                        detail: None,
                                        uri: uri.clone(),
                                        range,
                                        selection_range: range,
                                        data: None,
                                    }]));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn incoming_calls(
        &self,
        params: CallHierarchyIncomingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyIncomingCall>>> {
        let uri = &params.item.uri;
        let func_name = &params.item.name;

        let calls = self.find_incoming_calls(uri, func_name);

        if calls.is_empty() {
            return Ok(None);
        }

        if let Some(doc) = self.documents.get(uri) {
            let incoming: Vec<CallHierarchyIncomingCall> = calls
                .iter()
                .map(|(caller_name, caller_span, call_span)| {
                    let caller_range = self.span_to_range(&doc.content, caller_span);
                    let call_range = self.span_to_range(&doc.content, call_span);

                    CallHierarchyIncomingCall {
                        from: CallHierarchyItem {
                            name: caller_name.clone(),
                            kind: SymbolKind::FUNCTION,
                            tags: None,
                            detail: None,
                            uri: uri.clone(),
                            range: caller_range,
                            selection_range: caller_range,
                            data: None,
                        },
                        from_ranges: vec![call_range],
                    }
                })
                .collect();

            return Ok(Some(incoming));
        }

        Ok(None)
    }

    async fn outgoing_calls(
        &self,
        params: CallHierarchyOutgoingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyOutgoingCall>>> {
        let uri = &params.item.uri;
        let func_name = &params.item.name;

        let calls = self.find_outgoing_calls(uri, func_name);

        if calls.is_empty() {
            return Ok(None);
        }

        if let Some(doc) = self.documents.get(uri) {
            // Get cache to look up function definitions
            let cache = self.get_symbol_cache(uri);

            let outgoing: Vec<CallHierarchyOutgoingCall> = calls
                .iter()
                .map(|(callee_name, call_span)| {
                    let call_range = self.span_to_range(&doc.content, call_span);

                    // Find the callee definition
                    let callee_range = cache.as_ref().and_then(|c| {
                        c.definitions
                            .iter()
                            .find(|d| d.name == *callee_name && d.kind == SymbolKind::FUNCTION)
                            .map(|d| self.span_to_range(&doc.content, &d.span))
                    });

                    CallHierarchyOutgoingCall {
                        to: CallHierarchyItem {
                            name: callee_name.clone(),
                            kind: SymbolKind::FUNCTION,
                            tags: None,
                            detail: None,
                            uri: uri.clone(),
                            range: callee_range.unwrap_or(call_range),
                            selection_range: callee_range.unwrap_or(call_range),
                            data: None,
                        },
                        from_ranges: vec![call_range],
                    }
                })
                .collect();

            return Ok(Some(outgoing));
        }

        Ok(None)
    }

    async fn document_link(&self, params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
        let uri = &params.text_document.uri;

        if let Some(doc) = self.documents.get(uri) {
            if let Some(ast) = &doc.ast {
                let mut links = Vec::new();

                // Find U (use/import) statements and create document links
                for item in &ast.items {
                    if let Item::Use(use_item) = &item.node {
                        let path_str = use_item
                            .path
                            .iter()
                            .map(|s| s.node.as_str())
                            .collect::<Vec<_>>()
                            .join("/");

                        // Calculate the range for the path
                        if let Some(first) = use_item.path.first() {
                            if let Some(last) = use_item.path.last() {
                                let range = Range {
                                    start: self.offset_to_position(&doc.content, first.span.start),
                                    end: self.offset_to_position(&doc.content, last.span.end),
                                };

                                // Create a file URI for the import
                                // This assumes imports like "std/math" map to "std/math.vais"
                                let target_path = format!("{}.vais", path_str);

                                // Try to construct a proper file URI
                                if let Ok(base_path) = uri.to_file_path() {
                                    if let Some(parent) = base_path.parent() {
                                        let target = parent.join(&target_path);
                                        if let Ok(target_uri) = Url::from_file_path(&target) {
                                            links.push(DocumentLink {
                                                range,
                                                target: Some(target_uri),
                                                tooltip: Some(format!("Open {}", target_path)),
                                                data: None,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if !links.is_empty() {
                    return Ok(Some(links));
                }
            }
        }

        Ok(None)
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let query = params.query.to_lowercase();
        let mut symbols = Vec::new();

        // Search across all loaded documents
        for entry in self.documents.iter() {
            let uri = entry.key();
            let doc = entry.value();

            if let Some(ast) = &doc.ast {
                for item in &ast.items {
                    let (name, kind, span) = match &item.node {
                        Item::Function(f) => (&f.name.node, SymbolKind::FUNCTION, &f.name.span),
                        Item::Struct(s) => (&s.name.node, SymbolKind::STRUCT, &s.name.span),
                        Item::Enum(e) => (&e.name.node, SymbolKind::ENUM, &e.name.span),
                        Item::Trait(t) => (&t.name.node, SymbolKind::INTERFACE, &t.name.span),
                        Item::TypeAlias(ta) => {
                            (&ta.name.node, SymbolKind::TYPE_PARAMETER, &ta.name.span)
                        }
                        Item::Const(c) => (&c.name.node, SymbolKind::CONSTANT, &c.name.span),
                        Item::Global(g) => (&g.name.node, SymbolKind::VARIABLE, &g.name.span),
                        _ => continue,
                    };

                    // Filter by query (fuzzy match - contains)
                    if query.is_empty() || name.to_lowercase().contains(&query) {
                        let range = self.span_to_range(&doc.content, span);
                        #[allow(deprecated)]
                        symbols.push(SymbolInformation {
                            name: name.clone(),
                            kind,
                            location: Location {
                                uri: uri.clone(),
                                range,
                            },
                            tags: None,
                            deprecated: None,
                            container_name: None,
                        });
                    }
                }
            }
        }

        if symbols.is_empty() {
            Ok(None)
        } else {
            Ok(Some(symbols))
        }
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;
        if let Some(doc) = self.documents.get(uri) {
            let source = doc.content.to_string();
            // Parse the source to get an AST
            if let Ok(module) = parse(&source) {
                let config = FormatConfig {
                    indent_size: params.options.tab_size as usize,
                    use_tabs: !params.options.insert_spaces,
                    ..FormatConfig::default()
                };
                let mut formatter = Formatter::new(config);
                let formatted = formatter.format_module(&module);

                // Replace entire document content
                let line_count = doc.content.len_lines();
                let last_line = doc.content.line(line_count.saturating_sub(1));
                let last_char = last_line.len_chars();

                let edit = TextEdit {
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(line_count as u32, last_char as u32),
                    },
                    new_text: formatted,
                };
                return Ok(Some(vec![edit]));
            }
        }
        Ok(None)
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        crate::handlers::formatting::handle_range_formatting(self, params).await
    }

    async fn prepare_type_hierarchy(
        &self,
        params: TypeHierarchyPrepareParams,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Find the type at the cursor position
        if let Some((type_name, kind, span)) = self.find_type_at_position(uri, position) {
            if let Some(doc) = self.documents.get(uri) {
                let range = self.span_to_range(&doc.content, &span);
                let selection_range = range;

                let item = TypeHierarchyItem {
                    name: type_name.clone(),
                    kind,
                    tags: None,
                    detail: None,
                    uri: uri.clone(),
                    range,
                    selection_range,
                    data: None,
                };

                return Ok(Some(vec![item]));
            }
        }

        Ok(None)
    }

    async fn supertypes(
        &self,
        params: TypeHierarchySupertypesParams,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        let item = &params.item;
        let type_name = &item.name;
        let mut supertypes = Vec::new();

        match item.kind {
            SymbolKind::STRUCT | SymbolKind::ENUM => {
                // For structs/enums, supertypes are the traits they implement
                let traits = self.find_implemented_traits(type_name);
                for (uri, trait_name, span) in traits {
                    if let Some(doc) = self.documents.get(&uri) {
                        let range = self.span_to_range(&doc.content, &span);
                        supertypes.push(TypeHierarchyItem {
                            name: trait_name,
                            kind: SymbolKind::INTERFACE,
                            tags: None,
                            detail: Some("Trait".to_string()),
                            uri,
                            range,
                            selection_range: range,
                            data: None,
                        });
                    }
                }
            }
            SymbolKind::INTERFACE => {
                // For traits, supertypes are the super traits
                let super_traits = self.find_super_traits(type_name);
                for (uri, super_trait_name, span) in super_traits {
                    if let Some(doc) = self.documents.get(&uri) {
                        let range = self.span_to_range(&doc.content, &span);
                        supertypes.push(TypeHierarchyItem {
                            name: super_trait_name,
                            kind: SymbolKind::INTERFACE,
                            tags: None,
                            detail: Some("Super trait".to_string()),
                            uri,
                            range,
                            selection_range: range,
                            data: None,
                        });
                    }
                }
            }
            _ => {}
        }

        if supertypes.is_empty() {
            Ok(None)
        } else {
            Ok(Some(supertypes))
        }
    }

    async fn subtypes(
        &self,
        params: TypeHierarchySubtypesParams,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        let item = &params.item;
        let type_name = &item.name;
        let mut subtypes = Vec::new();

        match item.kind {
            SymbolKind::INTERFACE => {
                // For traits, subtypes include:
                // 1. Traits that extend this trait
                let sub_traits = self.find_sub_traits(type_name);
                for (uri, sub_trait_name, span) in sub_traits {
                    if let Some(doc) = self.documents.get(&uri) {
                        let range = self.span_to_range(&doc.content, &span);
                        subtypes.push(TypeHierarchyItem {
                            name: sub_trait_name,
                            kind: SymbolKind::INTERFACE,
                            tags: None,
                            detail: Some("Sub trait".to_string()),
                            uri,
                            range,
                            selection_range: range,
                            data: None,
                        });
                    }
                }

                // 2. Types that implement this trait
                let impls = self.find_trait_implementations(type_name);
                for (uri, impl_type_name, span) in impls {
                    if let Some(doc) = self.documents.get(&uri) {
                        let range = self.span_to_range(&doc.content, &span);
                        // Determine the kind of the implementing type
                        let kind = if let Some(ast) = &doc.ast {
                            let mut found_kind = SymbolKind::STRUCT; // default
                            for item in &ast.items {
                                match &item.node {
                                    Item::Struct(s) if s.name.node == impl_type_name => {
                                        found_kind = SymbolKind::STRUCT;
                                        break;
                                    }
                                    Item::Enum(e) if e.name.node == impl_type_name => {
                                        found_kind = SymbolKind::ENUM;
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                            found_kind
                        } else {
                            SymbolKind::STRUCT
                        };

                        subtypes.push(TypeHierarchyItem {
                            name: impl_type_name,
                            kind,
                            tags: None,
                            detail: Some("Implementor".to_string()),
                            uri,
                            range,
                            selection_range: range,
                            data: None,
                        });
                    }
                }
            }
            _ => {
                // For structs/enums, there are no subtypes in the traditional sense
                // (Vais doesn't have struct/enum inheritance)
            }
        }

        if subtypes.is_empty() {
            Ok(None)
        } else {
            Ok(Some(subtypes))
        }
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let uri = &params.text_document.uri;

        if let Some(doc) = self.documents.get(uri) {
            if let Some(ast) = &doc.ast {
                let mut lenses = Vec::new();

                for item in &ast.items {
                    match &item.node {
                        Item::Function(f) => {
                            let range = self.span_to_range(&doc.content, &f.name.span);

                            // Test lens: show "Run Test" for #[test] functions
                            let is_test = f.attributes.iter().any(|attr| attr.name == "test");
                            if is_test {
                                lenses.push(CodeLens {
                                    range,
                                    command: Some(Command {
                                        title: "$(testing-run-icon) Run Test".to_string(),
                                        command: "vais.runTest".to_string(),
                                        arguments: Some(vec![
                                            serde_json::Value::String(f.name.node.clone()),
                                            serde_json::Value::String(uri.to_string()),
                                        ]),
                                    }),
                                    data: None,
                                });
                                lenses.push(CodeLens {
                                    range,
                                    command: Some(Command {
                                        title: "$(debug-alt) Debug Test".to_string(),
                                        command: "vais.debugTest".to_string(),
                                        arguments: Some(vec![
                                            serde_json::Value::String(f.name.node.clone()),
                                            serde_json::Value::String(uri.to_string()),
                                        ]),
                                    }),
                                    data: None,
                                });
                            }

                            // References lens: count references to this function
                            let ref_count = self.count_references_in_ast(ast, &f.name.node);
                            if ref_count > 0 {
                                lenses.push(CodeLens {
                                    range,
                                    command: Some(Command {
                                        title: format!("{} reference{}", ref_count, if ref_count == 1 { "" } else { "s" }),
                                        command: "vais.showReferences".to_string(),
                                        arguments: Some(vec![
                                            serde_json::Value::String(uri.to_string()),
                                            serde_json::json!({ "line": range.start.line, "character": range.start.character }),
                                        ]),
                                    }),
                                    data: None,
                                });
                            }

                            // Benchmark lens for #[bench] functions
                            let is_bench = f
                                .attributes
                                .iter()
                                .any(|attr| attr.name == "bench" || attr.name == "benchmark");
                            if is_bench {
                                lenses.push(CodeLens {
                                    range,
                                    command: Some(Command {
                                        title: "$(graph) Run Benchmark".to_string(),
                                        command: "vais.runBenchmark".to_string(),
                                        arguments: Some(vec![
                                            serde_json::Value::String(f.name.node.clone()),
                                            serde_json::Value::String(uri.to_string()),
                                        ]),
                                    }),
                                    data: None,
                                });
                            }
                        }
                        Item::Struct(s) => {
                            let range = self.span_to_range(&doc.content, &s.name.span);

                            // Implementation count
                            let impl_count = ast
                                .items
                                .iter()
                                .filter(|i| {
                                    if let Item::Impl(imp) = &i.node {
                                        if let Type::Named { name, .. } = &imp.target_type.node {
                                            return name == &s.name.node;
                                        }
                                    }
                                    false
                                })
                                .count();

                            if impl_count > 0 {
                                lenses.push(CodeLens {
                                    range,
                                    command: Some(Command {
                                        title: format!("{} impl block{}", impl_count, if impl_count == 1 { "" } else { "s" }),
                                        command: "vais.showImplementations".to_string(),
                                        arguments: Some(vec![
                                            serde_json::Value::String(uri.to_string()),
                                            serde_json::json!({ "line": range.start.line, "character": range.start.character }),
                                        ]),
                                    }),
                                    data: None,
                                });
                            }

                            // Reference count for structs
                            let ref_count = self.count_references_in_ast(ast, &s.name.node);
                            if ref_count > 0 {
                                lenses.push(CodeLens {
                                    range,
                                    command: Some(Command {
                                        title: format!("{} reference{}", ref_count, if ref_count == 1 { "" } else { "s" }),
                                        command: "vais.showReferences".to_string(),
                                        arguments: Some(vec![
                                            serde_json::Value::String(uri.to_string()),
                                            serde_json::json!({ "line": range.start.line, "character": range.start.character }),
                                        ]),
                                    }),
                                    data: None,
                                });
                            }
                        }
                        Item::Enum(e) => {
                            let range = self.span_to_range(&doc.content, &e.name.span);
                            let ref_count = self.count_references_in_ast(ast, &e.name.node);
                            if ref_count > 0 {
                                lenses.push(CodeLens {
                                    range,
                                    command: Some(Command {
                                        title: format!("{} reference{}", ref_count, if ref_count == 1 { "" } else { "s" }),
                                        command: "vais.showReferences".to_string(),
                                        arguments: Some(vec![
                                            serde_json::Value::String(uri.to_string()),
                                            serde_json::json!({ "line": range.start.line, "character": range.start.character }),
                                        ]),
                                    }),
                                    data: None,
                                });
                            }
                        }
                        Item::Trait(t) => {
                            let range = self.span_to_range(&doc.content, &t.name.span);

                            // Count trait implementations
                            let impl_count = ast
                                .items
                                .iter()
                                .filter(|i| {
                                    if let Item::Impl(imp) = &i.node {
                                        if let Some(trait_name) = &imp.trait_name {
                                            return trait_name.node == t.name.node;
                                        }
                                    }
                                    false
                                })
                                .count();

                            if impl_count > 0 {
                                lenses.push(CodeLens {
                                    range,
                                    command: Some(Command {
                                        title: format!("{} implementation{}", impl_count, if impl_count == 1 { "" } else { "s" }),
                                        command: "vais.showImplementations".to_string(),
                                        arguments: Some(vec![
                                            serde_json::Value::String(uri.to_string()),
                                            serde_json::json!({ "line": range.start.line, "character": range.start.character }),
                                        ]),
                                    }),
                                    data: None,
                                });
                            }
                        }
                        _ => {}
                    }
                }

                if !lenses.is_empty() {
                    return Ok(Some(lenses));
                }
            }
        }

        Ok(None)
    }
}

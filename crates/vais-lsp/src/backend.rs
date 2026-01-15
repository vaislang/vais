//! LSP Backend implementation for Vais

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use vais_ast::{Module, Span};
use vais_parser::parse;

use crate::semantic::get_semantic_tokens;

/// Document state
pub struct Document {
    pub content: Rope,
    pub ast: Option<Module>,
    pub version: i32,
}

/// Vais Language Server Backend
pub struct VaisBackend {
    client: Client,
    documents: DashMap<Url, Document>,
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
                }
                // Clear diagnostics on successful parse
                self.client
                    .publish_diagnostics(uri.clone(), vec![], None)
                    .await;
            }
            Err(err) => {
                // Publish parse error as diagnostic
                let diagnostics = vec![Diagnostic {
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 1),
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("{}", err),
                    source: Some("vais".to_string()),
                    ..Default::default()
                }];
                self.client
                    .publish_diagnostics(uri.clone(), diagnostics, None)
                    .await;
            }
        }
    }

    /// Get position offset in text
    fn offset_to_position(&self, rope: &Rope, offset: usize) -> Position {
        let line = rope.char_to_line(offset.min(rope.len_chars()));
        let line_start = rope.line_to_char(line);
        let col = offset.saturating_sub(line_start);
        Position::new(line as u32, col as u32)
    }

    /// Convert span to LSP range
    fn span_to_range(&self, rope: &Rope, span: &Span) -> Range {
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
                definition_provider: Some(OneOf::Left(true)),
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
            }

            self.parse_document(&uri, &content).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(uri) {
            if let Some(ast) = &doc.ast {
                // Find the item at the cursor position
                for item in &ast.items {
                    match &item.node {
                        vais_ast::Item::Function(f) => {
                            let range = self.span_to_range(&doc.content, &f.name.span);
                            if position.line >= range.start.line
                                && position.line <= range.end.line
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

                                let signature = format!(
                                    "F {}({}){}",
                                    f.name.node,
                                    params_str.join(", "),
                                    ret_str
                                );

                                return Ok(Some(Hover {
                                    contents: HoverContents::Markup(MarkupContent {
                                        kind: MarkupKind::Markdown,
                                        value: format!("```vais\n{}\n```", signature),
                                    }),
                                    range: Some(range),
                                }));
                            }
                        }
                        vais_ast::Item::Struct(s) => {
                            let range = self.span_to_range(&doc.content, &s.name.span);
                            if position.line >= range.start.line
                                && position.line <= range.end.line
                            {
                                let fields_str: Vec<String> = s
                                    .fields
                                    .iter()
                                    .map(|f| format!("  {}: {:?}", f.name.node, f.ty.node))
                                    .collect();

                                let signature = format!(
                                    "S {} {{\n{}\n}}",
                                    s.name.node,
                                    fields_str.join(",\n")
                                );

                                return Ok(Some(Hover {
                                    contents: HoverContents::Markup(MarkupContent {
                                        kind: MarkupKind::Markdown,
                                        value: format!("```vais\n{}\n```", signature),
                                    }),
                                    range: Some(range),
                                }));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;

        let mut items = vec![];

        // Add keyword completions
        let keywords = [
            ("F", "Function definition", "F ${1:name}($2) -> ${3:type} {\n\t$0\n}"),
            ("S", "Struct definition", "S ${1:Name} {\n\t${2:field}: ${3:type}\n}"),
            ("E", "Enum definition", "E ${1:Name} {\n\t${2:Variant}\n}"),
            ("I", "If expression", "I ${1:condition} {\n\t$0\n}"),
            ("L", "Loop expression", "L ${1:item}: ${2:iter} {\n\t$0\n}"),
            ("M", "Match expression", "M ${1:expr} {\n\t${2:pattern} => $0\n}"),
            ("R", "Return", "R $0"),
            ("W", "Trait definition", "W ${1:Name} {\n\t$0\n}"),
            ("X", "Impl block", "X ${1:Type}: ${2:Trait} {\n\t$0\n}"),
            ("U", "Use/Import", "U ${1:module}"),
            ("A", "Async function", "A F ${1:name}($2) -> ${3:type} {\n\t$0\n}"),
        ];

        for (kw, detail, snippet) in keywords {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(detail.to_string()),
                insert_text: Some(snippet.to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }

        // Add type completions
        let types = ["i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "f32", "f64", "bool", "str"];
        for ty in types {
            items.push(CompletionItem {
                label: ty.to_string(),
                kind: Some(CompletionItemKind::TYPE_PARAMETER),
                ..Default::default()
            });
        }

        // Add functions/structs from current document
        if let Some(doc) = self.documents.get(uri) {
            if let Some(ast) = &doc.ast {
                for item in &ast.items {
                    match &item.node {
                        vais_ast::Item::Function(f) => {
                            items.push(CompletionItem {
                                label: f.name.node.clone(),
                                kind: Some(CompletionItemKind::FUNCTION),
                                detail: Some("Function".to_string()),
                                ..Default::default()
                            });
                        }
                        vais_ast::Item::Struct(s) => {
                            items.push(CompletionItem {
                                label: s.name.node.clone(),
                                kind: Some(CompletionItemKind::STRUCT),
                                detail: Some("Struct".to_string()),
                                ..Default::default()
                            });
                        }
                        vais_ast::Item::Enum(e) => {
                            items.push(CompletionItem {
                                label: e.name.node.clone(),
                                kind: Some(CompletionItemKind::ENUM),
                                detail: Some("Enum".to_string()),
                                ..Default::default()
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(uri) {
            if let Some(ast) = &doc.ast {
                // Convert position to offset
                let line = position.line as usize;
                let col = position.character as usize;
                let line_start = doc.content.line_to_char(line);
                let offset = line_start + col;

                // Find identifier at position and look for its definition
                for item in &ast.items {
                    match &item.node {
                        vais_ast::Item::Function(f) => {
                            if f.name.span.start <= offset && offset <= f.name.span.end {
                                let range = self.span_to_range(&doc.content, &f.name.span);
                                return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                                    uri: uri.clone(),
                                    range,
                                })));
                            }
                        }
                        vais_ast::Item::Struct(s) => {
                            if s.name.span.start <= offset && offset <= s.name.span.end {
                                let range = self.span_to_range(&doc.content, &s.name.span);
                                return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                                    uri: uri.clone(),
                                    range,
                                })));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        if let Some(doc) = self.documents.get(uri) {
            if let Some(ast) = &doc.ast {
                let mut symbols = vec![];

                for item in &ast.items {
                    match &item.node {
                        vais_ast::Item::Function(f) => {
                            let range = self.span_to_range(&doc.content, &item.span);
                            symbols.push(SymbolInformation {
                                name: f.name.node.clone(),
                                kind: SymbolKind::FUNCTION,
                                location: Location {
                                    uri: uri.clone(),
                                    range,
                                },
                                tags: None,
                                deprecated: None,
                                container_name: None,
                            });
                        }
                        vais_ast::Item::Struct(s) => {
                            let range = self.span_to_range(&doc.content, &item.span);
                            symbols.push(SymbolInformation {
                                name: s.name.node.clone(),
                                kind: SymbolKind::STRUCT,
                                location: Location {
                                    uri: uri.clone(),
                                    range,
                                },
                                tags: None,
                                deprecated: None,
                                container_name: None,
                            });
                        }
                        vais_ast::Item::Enum(e) => {
                            let range = self.span_to_range(&doc.content, &item.span);
                            symbols.push(SymbolInformation {
                                name: e.name.node.clone(),
                                kind: SymbolKind::ENUM,
                                location: Location {
                                    uri: uri.clone(),
                                    range,
                                },
                                tags: None,
                                deprecated: None,
                                container_name: None,
                            });
                        }
                        vais_ast::Item::Trait(t) => {
                            let range = self.span_to_range(&doc.content, &item.span);
                            symbols.push(SymbolInformation {
                                name: t.name.node.clone(),
                                kind: SymbolKind::INTERFACE,
                                location: Location {
                                    uri: uri.clone(),
                                    range,
                                },
                                tags: None,
                                deprecated: None,
                                container_name: None,
                            });
                        }
                        _ => {}
                    }
                }

                return Ok(Some(DocumentSymbolResponse::Flat(symbols)));
            }
        }

        Ok(None)
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
}

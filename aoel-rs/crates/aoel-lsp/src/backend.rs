//! LSP Backend implementation

use std::sync::Arc;
use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::builtins::{BUILTIN_FUNCTIONS, KEYWORDS, OPERATORS};
use crate::diagnostics::generate_diagnostics;
use crate::document::Document;

/// LSP Backend
pub struct Backend {
    client: Client,
    documents: Arc<DashMap<Url, Document>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DashMap::new()),
        }
    }

    async fn publish_diagnostics(&self, uri: Url) {
        if let Some(doc) = self.documents.get(&uri) {
            let diagnostics = generate_diagnostics(&doc);
            self.client
                .publish_diagnostics(uri, diagnostics, Some(doc.version))
                .await;
        }
    }

    /// Get word at position
    fn get_word_at_position(&self, uri: &Url, position: Position) -> Option<(String, usize, usize)> {
        let doc = self.documents.get(uri)?;
        let offset = doc.position_to_offset(position);
        let text = doc.text();

        if offset > text.len() {
            return None;
        }

        let before = &text[..offset];
        let after = &text[offset..];

        let word_start = before.rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let word_end = offset + after.find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(after.len());

        let word = text[word_start..word_end].to_string();
        if word.is_empty() {
            None
        } else {
            Some((word, word_start, word_end))
        }
    }

    /// Get completions at position
    fn get_completions_at(&self, uri: &Url, position: Position) -> Vec<CompletionItem> {
        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return vec![],
        };

        let offset = doc.position_to_offset(position);
        let text = doc.text();

        // Find prefix at cursor
        let before = &text[..offset];
        let word_start = before.rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let prefix = &before[word_start..];

        let mut completions = Vec::new();

        // Check if we're after a dot (method/operator completion)
        let after_dot = before.trim_end().ends_with('.');

        if after_dot || prefix.starts_with('.') {
            // Operator completions
            for (op, detail, snippet) in OPERATORS {
                if op.starts_with(prefix) || (after_dot && op.starts_with('.')) {
                    completions.push(CompletionItem {
                        label: op.to_string(),
                        kind: Some(CompletionItemKind::OPERATOR),
                        detail: Some(detail.to_string()),
                        insert_text: Some(snippet.to_string()),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        ..Default::default()
                    });
                }
            }
        }

        // Keyword completions
        for (kw, detail, snippet) in KEYWORDS {
            if kw.starts_with(prefix) {
                completions.push(CompletionItem {
                    label: kw.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some(detail.to_string()),
                    insert_text: Some(snippet.to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                });
            }
        }

        // Built-in function completions
        for func in BUILTIN_FUNCTIONS {
            if func.name.starts_with(prefix) {
                let documentation = format!(
                    "**{}**({})\n\n{}\n\nReturns: `{}`\n\nCategory: {}",
                    func.name, func.params, func.description, func.return_type, func.category
                );

                completions.push(CompletionItem {
                    label: func.name.to_string(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail: Some(format!("({}) -> {}", func.params, func.return_type)),
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: documentation,
                    })),
                    insert_text: Some(func.snippet.to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                });
            }
        }

        // User-defined functions from document
        if let Ok(program) = aoel_parser::parse(&text) {
            for item in &program.items {
                if let aoel_ast::Item::Function(func) = item {
                    if func.name.starts_with(prefix) {
                        let params = func.params.iter()
                            .map(|p| p.name.clone())
                            .collect::<Vec<_>>()
                            .join(", ");

                        completions.push(CompletionItem {
                            label: func.name.clone(),
                            kind: Some(CompletionItemKind::FUNCTION),
                            detail: Some(format!("fn {}({})", func.name, params)),
                            insert_text: Some(format!("{}($0)", func.name)),
                            insert_text_format: Some(InsertTextFormat::SNIPPET),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        completions
    }

    /// Get hover information
    fn get_hover_info(&self, uri: &Url, position: Position) -> Option<Hover> {
        let (word, _, _) = self.get_word_at_position(uri, position)?;
        let doc = self.documents.get(uri)?;
        let text = doc.text();

        // Check keywords
        let keyword_info = match word.as_str() {
            "let" => Some("**let** - Local variable binding\n\n```aoel\nlet x = 1 : x + 1\n```"),
            "if" => Some("**if** - Conditional expression\n\n```aoel\nif cond { then } else { else }\n```"),
            "true" | "false" => Some("**Bool** - Boolean literal"),
            "nil" => Some("**nil** - Nil/null value"),
            "mod" => Some("**mod** - Module declaration"),
            "use" => Some("**use** - Import from module"),
            "type" => Some("**type** - Type alias definition"),
            "pub" => Some("**pub** - Public visibility modifier"),
            _ => None,
        };

        if let Some(info) = keyword_info {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: info.to_string(),
                }),
                range: None,
            });
        }

        // Check built-in functions
        for func in BUILTIN_FUNCTIONS {
            if func.name == word {
                let info = format!(
                    "**{}**\n\n```aoel\n{}({}) -> {}\n```\n\n{}\n\n*Category: {}*",
                    func.name, func.name, func.params, func.return_type, func.description, func.category
                );
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: info,
                    }),
                    range: None,
                });
            }
        }

        // Check user-defined functions
        if let Ok(program) = aoel_parser::parse(&text) {
            for item in &program.items {
                if let aoel_ast::Item::Function(func) = item {
                    if func.name == word {
                        let params = func.params.iter()
                            .map(|p| {
                                if let Some(ty) = &p.ty {
                                    format!("{}: {:?}", p.name, ty)
                                } else {
                                    p.name.clone()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", ");

                        let ret_type = func.return_type.as_ref()
                            .map(|t| format!(" -> {:?}", t))
                            .unwrap_or_default();

                        let info = format!(
                            "**{}** (user-defined)\n\n```aoel\n{}({}){}= ...\n```",
                            func.name, func.name, params, ret_type
                        );

                        return Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: info,
                            }),
                            range: None,
                        });
                    }
                }
            }
        }

        None
    }

    /// Get signature help
    fn get_signature_help(&self, uri: &Url, position: Position) -> Option<SignatureHelp> {
        let doc = self.documents.get(uri)?;
        let offset = doc.position_to_offset(position);
        let text = doc.text();

        // Find the function name before the opening paren
        let before = &text[..offset];

        // Count open parens to find which function we're in
        let mut paren_depth = 0;
        let mut func_end = before.len();

        for (i, c) in before.char_indices().rev() {
            match c {
                ')' => paren_depth += 1,
                '(' => {
                    if paren_depth == 0 {
                        func_end = i;
                        break;
                    }
                    paren_depth -= 1;
                }
                _ => {}
            }
        }

        if func_end == before.len() {
            return None;
        }

        // Extract function name
        let func_text = &before[..func_end];
        let func_start = func_text.rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let func_name = &func_text[func_start..];

        if func_name.is_empty() {
            return None;
        }

        // Count commas to determine active parameter
        let in_call = &text[func_end..offset];
        let active_param = in_call.chars().filter(|&c| c == ',').count() as u32;

        // Check built-in functions
        for func in BUILTIN_FUNCTIONS {
            if func.name == func_name {
                let signature = SignatureInformation {
                    label: format!("{}({}) -> {}", func.name, func.params, func.return_type),
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: func.description.to_string(),
                    })),
                    parameters: Some(
                        func.params.split(", ")
                            .map(|p| ParameterInformation {
                                label: ParameterLabel::Simple(p.to_string()),
                                documentation: None,
                            })
                            .collect()
                    ),
                    active_parameter: Some(active_param),
                };

                return Some(SignatureHelp {
                    signatures: vec![signature],
                    active_signature: Some(0),
                    active_parameter: Some(active_param),
                });
            }
        }

        // Check user-defined functions
        if let Ok(program) = aoel_parser::parse(&text) {
            for item in &program.items {
                if let aoel_ast::Item::Function(func) = item {
                    if func.name == func_name {
                        let params = func.params.iter()
                            .map(|p| {
                                if let Some(ty) = &p.ty {
                                    format!("{}: {:?}", p.name, ty)
                                } else {
                                    p.name.clone()
                                }
                            })
                            .collect::<Vec<_>>();

                        let signature = SignatureInformation {
                            label: format!("{}({})", func.name, params.join(", ")),
                            documentation: None,
                            parameters: Some(
                                params.iter()
                                    .map(|p| ParameterInformation {
                                        label: ParameterLabel::Simple(p.to_string()),
                                        documentation: None,
                                    })
                                    .collect()
                            ),
                            active_parameter: Some(active_param),
                        };

                        return Some(SignatureHelp {
                            signatures: vec![signature],
                            active_signature: Some(0),
                            active_parameter: Some(active_param),
                        });
                    }
                }
            }
        }

        None
    }

    /// Find all references to a symbol
    fn find_references(&self, uri: &Url, position: Position, include_declaration: bool) -> Vec<Location> {
        let (word, _, _) = match self.get_word_at_position(uri, position) {
            Some(w) => w,
            None => return vec![],
        };

        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return vec![],
        };

        let text = doc.text();
        let mut locations = Vec::new();

        // Simple text search for the word
        let mut search_start = 0;
        while let Some(pos) = text[search_start..].find(&word) {
            let abs_pos = search_start + pos;

            // Check it's a whole word (not part of larger identifier)
            let before_ok = abs_pos == 0 ||
                text.chars().nth(abs_pos - 1).is_none_or(|c| !c.is_alphanumeric());
            let after_ok = abs_pos + word.len() >= text.len() ||
                text.chars().nth(abs_pos + word.len()).is_none_or(|c| !c.is_alphanumeric());

            if before_ok && after_ok {
                let start = doc.offset_to_position(abs_pos);
                let end = doc.offset_to_position(abs_pos + word.len());

                // Skip declaration if not requested
                if !include_declaration {
                    // Check if this is the definition
                    if let Ok(program) = aoel_parser::parse(&text) {
                        let mut is_definition = false;
                        for item in &program.items {
                            if let aoel_ast::Item::Function(func) = item {
                                if func.name == word && func.span.start == abs_pos {
                                    is_definition = true;
                                    break;
                                }
                            }
                        }
                        if is_definition {
                            search_start = abs_pos + 1;
                            continue;
                        }
                    }
                }

                locations.push(Location {
                    uri: uri.clone(),
                    range: Range { start, end },
                });
            }

            search_start = abs_pos + 1;
        }

        locations
    }

    /// Prepare rename (check if rename is valid)
    fn prepare_rename(&self, uri: &Url, position: Position) -> Option<PrepareRenameResponse> {
        let (word, start, end) = self.get_word_at_position(uri, position)?;
        let doc = self.documents.get(uri)?;

        // Check if it's a keyword (can't rename)
        for (kw, _, _) in KEYWORDS {
            if *kw == word {
                return None;
            }
        }

        // Check if it's a built-in (can't rename)
        for func in BUILTIN_FUNCTIONS {
            if func.name == word {
                return None;
            }
        }

        Some(PrepareRenameResponse::Range(Range {
            start: doc.offset_to_position(start),
            end: doc.offset_to_position(end),
        }))
    }

    /// Perform rename
    fn rename(&self, uri: &Url, position: Position, new_name: String) -> Option<WorkspaceEdit> {
        let locations = self.find_references(uri, position, true);

        if locations.is_empty() {
            return None;
        }

        let edits: Vec<TextEdit> = locations.iter()
            .map(|loc| TextEdit {
                range: loc.range,
                new_text: new_name.clone(),
            })
            .collect();

        let mut changes = std::collections::HashMap::new();
        changes.insert(uri.clone(), edits);

        Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        })
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), "(".to_string()]),
                    resolve_provider: Some(false),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: Some(vec![",".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: Default::default(),
                })),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "AOEL Language Server".to_string(),
                version: Some("0.2.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "AOEL Language Server v0.2.0 initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let doc = Document::new(params.text_document.text, params.text_document.version);
        self.documents.insert(uri.clone(), doc);
        self.publish_diagnostics(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(mut doc) = self.documents.get_mut(&uri) {
            doc.version = params.text_document.version;
            for change in params.content_changes {
                doc.apply_change(&change);
            }
        }

        self.publish_diagnostics(uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.publish_diagnostics(params.text_document.uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let completions = self.get_completions_at(&uri, position);

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        Ok(self.get_hover_info(&uri, position))
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        Ok(self.get_signature_help(&uri, position))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let (word, _, _) = match self.get_word_at_position(&uri, position) {
            Some(w) => w,
            None => return Ok(None),
        };

        let doc = match self.documents.get(&uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        let text = doc.text();

        // Find function definition
        if let Ok(program) = aoel_parser::parse(&text) {
            for item in &program.items {
                if let aoel_ast::Item::Function(func) = item {
                    if func.name == word {
                        let range = Range {
                            start: doc.offset_to_position(func.span.start),
                            end: doc.offset_to_position(func.span.end),
                        };
                        return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                            uri: uri.clone(),
                            range,
                        })));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let include_declaration = params.context.include_declaration;

        let locations = self.find_references(&uri, position, include_declaration);

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
    }

    async fn prepare_rename(&self, params: TextDocumentPositionParams) -> Result<Option<PrepareRenameResponse>> {
        let uri = params.text_document.uri;
        let position = params.position;

        Ok(self.prepare_rename(&uri, position))
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        Ok(self.rename(&uri, position, new_name))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        let doc = match self.documents.get(&uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        let text = doc.text();
        let mut symbols = Vec::new();

        if let Ok(program) = aoel_parser::parse(&text) {
            for item in &program.items {
                match item {
                    aoel_ast::Item::Function(func) => {
                        let range = Range {
                            start: doc.offset_to_position(func.span.start),
                            end: doc.offset_to_position(func.span.end),
                        };
                        let params = func.params.iter()
                            .map(|p| p.name.clone())
                            .collect::<Vec<_>>()
                            .join(", ");

                        #[allow(deprecated)]
                        symbols.push(DocumentSymbol {
                            name: func.name.clone(),
                            detail: Some(format!("({})", params)),
                            kind: SymbolKind::FUNCTION,
                            range,
                            selection_range: range,
                            children: None,
                            tags: None,
                            deprecated: None,
                        });
                    }
                    aoel_ast::Item::TypeDef(typedef) => {
                        let range = Range {
                            start: doc.offset_to_position(typedef.span.start),
                            end: doc.offset_to_position(typedef.span.end),
                        };

                        #[allow(deprecated)]
                        symbols.push(DocumentSymbol {
                            name: typedef.name.clone(),
                            detail: None,
                            kind: SymbolKind::TYPE_PARAMETER,
                            range,
                            selection_range: range,
                            children: None,
                            tags: None,
                            deprecated: None,
                        });
                    }
                    aoel_ast::Item::Module(module) => {
                        let range = Range {
                            start: doc.offset_to_position(module.span.start),
                            end: doc.offset_to_position(module.span.end),
                        };

                        #[allow(deprecated)]
                        symbols.push(DocumentSymbol {
                            name: module.name.clone(),
                            detail: None,
                            kind: SymbolKind::MODULE,
                            range,
                            selection_range: range,
                            children: None,
                            tags: None,
                            deprecated: None,
                        });
                    }
                    _ => {}
                }
            }
        }

        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }
}

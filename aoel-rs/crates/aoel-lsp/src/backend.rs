//! LSP Backend implementation

use std::sync::Arc;
use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

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

    fn get_completions_at(&self, uri: &Url, position: Position) -> Vec<CompletionItem> {
        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return vec![],
        };

        let offset = doc.position_to_offset(position);
        let text = doc.text();

        // 간단한 완성: 현재 위치 앞의 단어 찾기
        let before = &text[..offset];
        let word_start = before.rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let prefix = &before[word_start..];

        let mut completions = Vec::new();

        // 키워드 완성
        let keywords = [
            ("let", "let binding", "let $1 = $2 : $0"),
            ("if", "if expression", "if $1 { $2 } else { $0 }"),
            ("true", "boolean true", "true"),
            ("false", "boolean false", "false"),
            ("nil", "nil value", "nil"),
            ("mod", "module declaration", "mod $0"),
            ("use", "use import", "use $0"),
            ("type", "type alias", "type $1 = $0"),
            ("pub", "public modifier", "pub "),
        ];

        for (kw, detail, snippet) in keywords {
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

        // 빌트인 함수 완성
        let builtins = [
            ("len", "Get length", "len($0)"),
            ("abs", "Absolute value", "abs($0)"),
            ("sqrt", "Square root", "sqrt($0)"),
            ("print", "Print to stdout", "print($0)"),
        ];

        for (name, detail, snippet) in builtins {
            if name.starts_with(prefix) {
                completions.push(CompletionItem {
                    label: name.to_string(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail: Some(detail.to_string()),
                    insert_text: Some(snippet.to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                });
            }
        }

        // 연산자 완성
        if prefix.starts_with('.') || prefix.is_empty() {
            let operators = [
                (".@", "Map operation", ".@($0)"),
                (".?", "Filter operation", ".?($0)"),
                ("./+", "Sum reduce", "./+"),
                ("./*", "Product reduce", "./*"),
                ("./min", "Min reduce", "./min"),
                ("./max", "Max reduce", "./max"),
            ];

            for (op, detail, snippet) in operators {
                if op.starts_with(prefix) {
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

        // 문서에서 정의된 함수 이름 추출
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

    fn get_hover_info(&self, uri: &Url, position: Position) -> Option<Hover> {
        let doc = self.documents.get(uri)?;
        let offset = doc.position_to_offset(position);
        let text = doc.text();

        // 현재 위치의 단어 찾기
        let before = &text[..offset];
        let after = &text[offset..];

        let word_start = before.rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let word_end = after.find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(after.len());

        let word = &text[word_start..offset + word_end];
        if word.is_empty() {
            return None;
        }

        // 키워드 설명
        let keyword_info = match word {
            "let" => Some("**let** - 로컬 변수 바인딩\n\n```aoel\nlet x = 1 : x + 1\n```"),
            "if" => Some("**if** - 조건 표현식\n\n```aoel\nif cond { then } else { else }\n```"),
            "true" | "false" => Some("**Bool** - 불리언 리터럴"),
            "nil" => Some("**nil** - 빈 값"),
            "mod" => Some("**mod** - 모듈 선언"),
            "use" => Some("**use** - 모듈 임포트"),
            "type" => Some("**type** - 타입 별칭 정의"),
            "pub" => Some("**pub** - 공개 가시성 수정자"),
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

        // 문서에서 함수 정의 찾기
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
                            "**{}**\n\n```aoel\n{}({}){}= ...\n```",
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
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "AOEL Language Server".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "AOEL Language Server initialized")
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

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let doc = match self.documents.get(&uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        let offset = doc.position_to_offset(position);
        let text = doc.text();

        // 현재 위치의 단어 찾기
        let before = &text[..offset];
        let after = &text[offset..];

        let word_start = before.rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let word_end = after.find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(after.len());

        let word = &text[word_start..offset + word_end];

        // 함수 정의 찾기
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

//! LSP Backend implementation for Vais

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use vais_ast::{FunctionBody, Item, Module, Span, Stmt, Type};
use vais_codegen::formatter::{FormatConfig, Formatter};
use vais_parser::parse;

use crate::ai_completion::{generate_ai_completions, CompletionContext as AiContext};
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
fn position_in_range(position: &Position, range: &Range) -> bool {
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
fn get_builtin_hover(name: &str) -> Option<Hover> {
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
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(uri) {
            // Convert position to offset
            let line = position.line as usize;
            let col = position.character as usize;
            let line_start = doc.content.line_to_char(line);
            let offset = line_start + col;

            // Get identifier at position
            if let Some(ast) = &doc.ast {
                let ident = self.get_identifier_at(uri, offset);

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
                            let range = self.span_to_range(&doc.content, &f.name.span);
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
                            let range = self.span_to_range(&doc.content, &s.name.span);
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
                            let range = self.span_to_range(&doc.content, &e.name.span);
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
                            let range = self.span_to_range(&doc.content, &t.name.span);
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

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let mut items = vec![];

        // Check if we're completing after a dot (method completion)
        let is_method_completion = if let Some(doc) = self.documents.get(uri) {
            let line_idx = position.line as usize;
            if let Some(line) = doc.content.get_line(line_idx) {
                let line_str: String = line.chars().collect();
                let col = position.character as usize;
                col > 0 && line_str.chars().nth(col - 1) == Some('.')
            } else {
                false
            }
        } else {
            false
        };

        if is_method_completion {
            // Add common method completions
            let methods = [
                ("len", "Get length", "len()"),
                ("is_empty", "Check if empty", "is_empty()"),
                ("push", "Push element", "push(${1:value})"),
                ("pop", "Pop element", "pop()"),
                ("get", "Get element", "get(${1:index})"),
                ("clone", "Clone value", "clone()"),
                ("drop", "Drop/free", "drop()"),
                ("print", "Print value", "print()"),
                (
                    "unwrap_or",
                    "Unwrap with default",
                    "unwrap_or(${1:default})",
                ),
                ("is_some", "Check if Some", "is_some()"),
                ("is_none", "Check if None", "is_none()"),
                ("is_ok", "Check if Ok", "is_ok()"),
                ("is_err", "Check if Err", "is_err()"),
                ("await", "Await async result", "await"),
            ];

            for (name, detail, snippet) in methods {
                items.push(CompletionItem {
                    label: name.to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some(detail.to_string()),
                    insert_text: Some(snippet.to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                });
            }

            // Add methods from impl blocks in current document
            if let Some(doc) = self.documents.get(uri) {
                if let Some(ast) = &doc.ast {
                    for item in &ast.items {
                        if let vais_ast::Item::Impl(impl_block) = &item.node {
                            for method in &impl_block.methods {
                                let params_str: Vec<String> = method
                                    .node
                                    .params
                                    .iter()
                                    .filter(|p| p.name.node != "self")
                                    .enumerate()
                                    .map(|(i, p)| format!("${{{}:{}}}", i + 1, p.name.node))
                                    .collect();

                                items.push(CompletionItem {
                                    label: method.node.name.node.clone(),
                                    kind: Some(CompletionItemKind::METHOD),
                                    detail: Some(format!(
                                        "Method of {:?}",
                                        impl_block.target_type.node
                                    )),
                                    insert_text: Some(format!(
                                        "{}({})",
                                        method.node.name.node,
                                        params_str.join(", ")
                                    )),
                                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
            }

            return Ok(Some(CompletionResponse::Array(items)));
        }

        // Add keyword completions
        let keywords = [
            (
                "F",
                "Function definition",
                "F ${1:name}($2) -> ${3:type} {\n\t$0\n}",
            ),
            (
                "S",
                "Struct definition",
                "S ${1:Name} {\n\t${2:field}: ${3:type}\n}",
            ),
            ("E", "Enum definition", "E ${1:Name} {\n\t${2:Variant}\n}"),
            ("I", "If expression", "I ${1:condition} {\n\t$0\n}"),
            ("L", "Loop expression", "L ${1:item}: ${2:iter} {\n\t$0\n}"),
            (
                "M",
                "Match expression",
                "M ${1:expr} {\n\t${2:pattern} => $0\n}",
            ),
            ("R", "Return", "R $0"),
            ("B", "Break", "B"),
            ("C", "Continue", "C"),
            ("W", "Trait definition", "W ${1:Name} {\n\t$0\n}"),
            ("X", "Impl block", "X ${1:Type} {\n\t$0\n}"),
            ("U", "Use/Import", "U ${1:std/module}"),
            (
                "A",
                "Async function",
                "A F ${1:name}($2) -> ${3:type} {\n\t$0\n}",
            ),
            ("spawn", "Spawn async task", "spawn ${1:expr}"),
            ("await", "Await async result", "await"),
            ("true", "Boolean true", "true"),
            ("false", "Boolean false", "false"),
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
        let types = [
            ("i8", "8-bit signed integer"),
            ("i16", "16-bit signed integer"),
            ("i32", "32-bit signed integer"),
            ("i64", "64-bit signed integer"),
            ("i128", "128-bit signed integer"),
            ("u8", "8-bit unsigned integer"),
            ("u16", "16-bit unsigned integer"),
            ("u32", "32-bit unsigned integer"),
            ("u64", "64-bit unsigned integer"),
            ("u128", "128-bit unsigned integer"),
            ("f32", "32-bit floating point"),
            ("f64", "64-bit floating point"),
            ("bool", "Boolean type"),
            ("str", "String type"),
        ];
        for (ty, doc) in types {
            items.push(CompletionItem {
                label: ty.to_string(),
                kind: Some(CompletionItemKind::TYPE_PARAMETER),
                detail: Some(doc.to_string()),
                ..Default::default()
            });
        }

        // Add builtin function completions
        let builtins = [
            (
                "puts",
                "Print string with newline",
                "puts(${1:s})",
                "fn(str) -> i64",
            ),
            (
                "putchar",
                "Print single character",
                "putchar(${1:c})",
                "fn(i64) -> i64",
            ),
            (
                "print_i64",
                "Print 64-bit integer",
                "print_i64(${1:n})",
                "fn(i64) -> i64",
            ),
            (
                "print_f64",
                "Print 64-bit float",
                "print_f64(${1:n})",
                "fn(f64) -> i64",
            ),
            (
                "malloc",
                "Allocate heap memory",
                "malloc(${1:size})",
                "fn(i64) -> i64",
            ),
            (
                "free",
                "Free heap memory",
                "free(${1:ptr})",
                "fn(i64) -> i64",
            ),
            (
                "memcpy",
                "Copy memory",
                "memcpy(${1:dst}, ${2:src}, ${3:n})",
                "fn(i64, i64, i64) -> i64",
            ),
            (
                "strlen",
                "Get string length",
                "strlen(${1:s})",
                "fn(i64) -> i64",
            ),
            (
                "load_i64",
                "Load i64 from memory",
                "load_i64(${1:ptr})",
                "fn(i64) -> i64",
            ),
            (
                "store_i64",
                "Store i64 to memory",
                "store_i64(${1:ptr}, ${2:val})",
                "fn(i64, i64) -> i64",
            ),
            (
                "load_byte",
                "Load byte from memory",
                "load_byte(${1:ptr})",
                "fn(i64) -> i64",
            ),
            (
                "store_byte",
                "Store byte to memory",
                "store_byte(${1:ptr}, ${2:val})",
                "fn(i64, i64) -> i64",
            ),
        ];

        for (name, doc, snippet, sig) in builtins {
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(sig.to_string()),
                documentation: Some(Documentation::String(doc.to_string())),
                insert_text: Some(snippet.to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }

        // Add standard library module completions (for U statements)
        let std_modules = [
            ("std/math", "Math functions (sin, cos, sqrt, etc.)"),
            ("std/io", "Input/output functions"),
            ("std/option", "Option type (Some, None)"),
            ("std/result", "Result type (Ok, Err)"),
            ("std/vec", "Dynamic array (Vec)"),
            ("std/string", "String type and operations"),
            ("std/hashmap", "Hash map collection"),
            ("std/file", "File I/O operations"),
            ("std/iter", "Iterator trait"),
            ("std/future", "Async Future type"),
            ("std/rc", "Reference counting (Rc)"),
            ("std/box", "Heap allocation (Box)"),
            ("std/arena", "Arena allocator"),
            ("std/runtime", "Async runtime"),
        ];

        for (module, doc) in std_modules {
            items.push(CompletionItem {
                label: module.to_string(),
                kind: Some(CompletionItemKind::MODULE),
                detail: Some(doc.to_string()),
                ..Default::default()
            });
        }

        // Add math constants and functions if std/math might be imported
        let math_items = [
            ("PI", "π = 3.14159...", CompletionItemKind::CONSTANT),
            ("E", "e = 2.71828...", CompletionItemKind::CONSTANT),
            ("TAU", "τ = 2π", CompletionItemKind::CONSTANT),
            ("sqrt", "Square root", CompletionItemKind::FUNCTION),
            ("pow", "Power function", CompletionItemKind::FUNCTION),
            ("sin", "Sine", CompletionItemKind::FUNCTION),
            ("cos", "Cosine", CompletionItemKind::FUNCTION),
            ("tan", "Tangent", CompletionItemKind::FUNCTION),
            ("log", "Natural logarithm", CompletionItemKind::FUNCTION),
            ("exp", "Exponential", CompletionItemKind::FUNCTION),
            ("floor", "Floor function", CompletionItemKind::FUNCTION),
            ("ceil", "Ceiling function", CompletionItemKind::FUNCTION),
            ("abs", "Absolute value (f64)", CompletionItemKind::FUNCTION),
            (
                "abs_i64",
                "Absolute value (i64)",
                CompletionItemKind::FUNCTION,
            ),
            ("min", "Minimum (f64)", CompletionItemKind::FUNCTION),
            ("max", "Maximum (f64)", CompletionItemKind::FUNCTION),
        ];

        for (name, doc, kind) in math_items {
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(kind),
                detail: Some(doc.to_string()),
                ..Default::default()
            });
        }

        // Add IO functions
        let io_items = [
            ("read_i64", "Read integer from stdin", "read_i64()"),
            ("read_f64", "Read float from stdin", "read_f64()"),
            (
                "read_line",
                "Read line from stdin",
                "read_line(${1:buffer}, ${2:max_len})",
            ),
            ("read_char", "Read character from stdin", "read_char()"),
            (
                "prompt_i64",
                "Prompt and read integer",
                "prompt_i64(${1:prompt})",
            ),
            (
                "prompt_f64",
                "Prompt and read float",
                "prompt_f64(${1:prompt})",
            ),
        ];

        for (name, doc, snippet) in io_items {
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(doc.to_string()),
                insert_text: Some(snippet.to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }

        // Add common type constructors
        let constructors = [
            ("Some", "Option::Some variant", "Some(${1:value})"),
            ("None", "Option::None variant", "None"),
            ("Ok", "Result::Ok variant", "Ok(${1:value})"),
            ("Err", "Result::Err variant", "Err(${1:error})"),
        ];

        for (name, doc, snippet) in constructors {
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::CONSTRUCTOR),
                detail: Some(doc.to_string()),
                insert_text: Some(snippet.to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }

        // Add functions/structs/enums from current document
        if let Some(doc) = self.documents.get(uri) {
            if let Some(ast) = &doc.ast {
                for item in &ast.items {
                    match &item.node {
                        vais_ast::Item::Function(f) => {
                            let params_str: Vec<String> = f
                                .params
                                .iter()
                                .enumerate()
                                .map(|(i, p)| format!("${{{}:{}}}", i + 1, p.name.node))
                                .collect();

                            let ret_str = f
                                .ret_type
                                .as_ref()
                                .map(|t| format!(" -> {:?}", t.node))
                                .unwrap_or_default();

                            items.push(CompletionItem {
                                label: f.name.node.clone(),
                                kind: Some(CompletionItemKind::FUNCTION),
                                detail: Some(format!(
                                    "fn({}){}",
                                    f.params
                                        .iter()
                                        .map(|p| format!("{}: {:?}", p.name.node, p.ty.node))
                                        .collect::<Vec<_>>()
                                        .join(", "),
                                    ret_str
                                )),
                                insert_text: Some(format!(
                                    "{}({})",
                                    f.name.node,
                                    params_str.join(", ")
                                )),
                                insert_text_format: Some(InsertTextFormat::SNIPPET),
                                ..Default::default()
                            });
                        }
                        vais_ast::Item::Struct(s) => {
                            // Add struct name
                            items.push(CompletionItem {
                                label: s.name.node.clone(),
                                kind: Some(CompletionItemKind::STRUCT),
                                detail: Some("Struct".to_string()),
                                ..Default::default()
                            });

                            // Add struct literal completion
                            let fields_str: Vec<String> = s
                                .fields
                                .iter()
                                .enumerate()
                                .map(|(i, f)| {
                                    format!("{}: ${{{}:{}}}", f.name.node, i + 1, f.name.node)
                                })
                                .collect();

                            items.push(CompletionItem {
                                label: format!("{} {{ }}", s.name.node),
                                kind: Some(CompletionItemKind::CONSTRUCTOR),
                                detail: Some("Struct literal".to_string()),
                                insert_text: Some(format!(
                                    "{} {{ {} }}",
                                    s.name.node,
                                    fields_str.join(", ")
                                )),
                                insert_text_format: Some(InsertTextFormat::SNIPPET),
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

                            // Add enum variants
                            for variant in &e.variants {
                                let variant_text = match &variant.fields {
                                    vais_ast::VariantFields::Unit => variant.name.node.clone(),
                                    vais_ast::VariantFields::Tuple(types) => {
                                        let fields_str: Vec<String> = types
                                            .iter()
                                            .enumerate()
                                            .map(|(i, _)| format!("${{{}}}", i + 1))
                                            .collect();
                                        format!("{}({})", variant.name.node, fields_str.join(", "))
                                    }
                                    vais_ast::VariantFields::Struct(fields) => {
                                        let fields_str: Vec<String> = fields
                                            .iter()
                                            .enumerate()
                                            .map(|(i, f)| {
                                                format!(
                                                    "{}: ${{{}:{}}}",
                                                    f.name.node,
                                                    i + 1,
                                                    f.name.node
                                                )
                                            })
                                            .collect();
                                        format!(
                                            "{} {{ {} }}",
                                            variant.name.node,
                                            fields_str.join(", ")
                                        )
                                    }
                                };

                                items.push(CompletionItem {
                                    label: variant.name.node.clone(),
                                    kind: Some(CompletionItemKind::ENUM_MEMBER),
                                    detail: Some(format!("{}::{}", e.name.node, variant.name.node)),
                                    insert_text: Some(variant_text),
                                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                                    ..Default::default()
                                });
                            }
                        }
                        vais_ast::Item::Trait(t) => {
                            items.push(CompletionItem {
                                label: t.name.node.clone(),
                                kind: Some(CompletionItemKind::INTERFACE),
                                detail: Some("Trait".to_string()),
                                ..Default::default()
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        // AI-based completions: analyze context and suggest patterns
        if let Some(doc) = self.documents.get(uri) {
            let content: String = doc.content.chars().collect();
            let ai_ctx = AiContext::from_document(&content, position, doc.ast.as_ref());
            items.extend(generate_ai_completions(&ai_ctx));
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
            // Convert position to offset
            let line = position.line as usize;
            let col = position.character as usize;
            let line_start = doc.content.line_to_char(line);
            let offset = line_start + col;
            drop(doc); // Release read lock

            // Use the new find_definition_at method (uses cache)
            if let Some(def) = self.find_definition_at(uri, offset) {
                if let Some(doc) = self.documents.get(uri) {
                    let range = self.span_to_range(&doc.content, &def.span);
                    return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                        uri: uri.clone(),
                        range,
                    })));
                }
            }
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some(doc) = self.documents.get(uri) {
            // Convert position to offset
            let line = position.line as usize;
            let col = position.character as usize;
            let line_start = doc.content.line_to_char(line);
            let offset = line_start + col;
            drop(doc); // Release read lock

            // Get the symbol name at the position (uses cache)
            if let Some(symbol_name) = self.get_identifier_at(uri, offset) {
                let spans = self.find_all_references(uri, &symbol_name);

                if let Some(doc) = self.documents.get(uri) {
                    let locations: Vec<Location> = spans
                        .iter()
                        .map(|span| Location {
                            uri: uri.clone(),
                            range: self.span_to_range(&doc.content, span),
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
                            #[allow(deprecated)]
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
                            #[allow(deprecated)]
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
                            #[allow(deprecated)]
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
                            #[allow(deprecated)]
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
        let uri = &params.text_document.uri;
        let range = params.range;
        let mut actions = Vec::new();

        if let Some(doc) = self.documents.get(uri) {
            // Get diagnostics from the context
            let diagnostics = &params.context.diagnostics;
            if !diagnostics.is_empty() {
                for diagnostic in diagnostics {
                    // Quick fix for undefined variables
                    if diagnostic.message.starts_with("Undefined variable:") {
                        let var_name = diagnostic
                            .message
                            .strip_prefix("Undefined variable: ")
                            .unwrap_or("");

                        // Suggest creating a variable
                        let insert_position = Position::new(range.start.line, 0);
                        let edit = WorkspaceEdit {
                            changes: Some({
                                let mut map = std::collections::HashMap::new();
                                map.insert(
                                    uri.clone(),
                                    vec![TextEdit {
                                        range: Range::new(insert_position, insert_position),
                                        new_text: format!("L {}: i64 = 0\n", var_name),
                                    }],
                                );
                                map
                            }),
                            document_changes: None,
                            change_annotations: None,
                        };

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: format!("Create variable '{}'", var_name),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            edit: Some(edit),
                            ..Default::default()
                        }));
                    }

                    // Quick fix for undefined functions - suggest import
                    if diagnostic.message.starts_with("Undefined function:") {
                        let func_name = diagnostic
                            .message
                            .strip_prefix("Undefined function: ")
                            .unwrap_or("");

                        // Map of common functions to their modules
                        let module_suggestions = [
                            ("sqrt", "std/math"),
                            ("sin", "std/math"),
                            ("cos", "std/math"),
                            ("tan", "std/math"),
                            ("pow", "std/math"),
                            ("log", "std/math"),
                            ("exp", "std/math"),
                            ("floor", "std/math"),
                            ("ceil", "std/math"),
                            ("abs", "std/math"),
                            ("abs_i64", "std/math"),
                            ("min", "std/math"),
                            ("max", "std/math"),
                            ("read_i64", "std/io"),
                            ("read_f64", "std/io"),
                            ("read_line", "std/io"),
                            ("read_char", "std/io"),
                        ];

                        for (name, module) in &module_suggestions {
                            if func_name == *name {
                                // Check if import already exists
                                let has_import = if let Some(ast) = &doc.ast {
                                    ast.items.iter().any(|item| {
                                        if let vais_ast::Item::Use(use_item) = &item.node {
                                            // Convert path to string for comparison
                                            let path_str = use_item
                                                .path
                                                .iter()
                                                .map(|s| s.node.as_str())
                                                .collect::<Vec<_>>()
                                                .join("/");
                                            path_str == *module
                                        } else {
                                            false
                                        }
                                    })
                                } else {
                                    false
                                };

                                if !has_import {
                                    let edit = WorkspaceEdit {
                                        changes: Some({
                                            let mut map = std::collections::HashMap::new();
                                            map.insert(
                                                uri.clone(),
                                                vec![TextEdit {
                                                    range: Range::new(
                                                        Position::new(0, 0),
                                                        Position::new(0, 0),
                                                    ),
                                                    new_text: format!("U {}\n", module),
                                                }],
                                            );
                                            map
                                        }),
                                        document_changes: None,
                                        change_annotations: None,
                                    };

                                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                        title: format!("Import {} from {}", func_name, module),
                                        kind: Some(CodeActionKind::QUICKFIX),
                                        diagnostics: Some(vec![diagnostic.clone()]),
                                        edit: Some(edit),
                                        ..Default::default()
                                    }));
                                }
                                break;
                            }
                        }
                    }

                    // Quick fix for type mismatches - suggest type cast
                    if diagnostic.message.starts_with("Type mismatch:")
                        && (diagnostic.message.contains("expected i64, found f64")
                            || diagnostic.message.contains("expected f64, found i64"))
                    {
                        let cast_type = if diagnostic.message.contains("expected i64") {
                            "i64"
                        } else {
                            "f64"
                        };

                        // Get the text at the diagnostic range
                        let line = diagnostic.range.start.line as usize;
                        if let Some(line_rope) = doc.content.get_line(line) {
                            let line_str: String = line_rope.chars().collect();
                            let start = diagnostic.range.start.character as usize;
                            let end = diagnostic.range.end.character as usize;
                            if end <= line_str.len() {
                                let text = &line_str[start..end];

                                let edit = WorkspaceEdit {
                                    changes: Some({
                                        let mut map = std::collections::HashMap::new();
                                        map.insert(
                                            uri.clone(),
                                            vec![TextEdit {
                                                range: diagnostic.range,
                                                new_text: format!("{} as {}", text, cast_type),
                                            }],
                                        );
                                        map
                                    }),
                                    document_changes: None,
                                    change_annotations: None,
                                };

                                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                    title: format!("Cast to {}", cast_type),
                                    kind: Some(CodeActionKind::QUICKFIX),
                                    diagnostics: Some(vec![diagnostic.clone()]),
                                    edit: Some(edit),
                                    ..Default::default()
                                }));
                            }
                        }
                    }

                    // Quick fix for unused variable - suggest prefixing with _
                    if diagnostic.message.contains("unused variable") {
                        let var_name = diagnostic.message.split('\'').nth(1).unwrap_or("");

                        if !var_name.is_empty() {
                            let line = diagnostic.range.start.line as usize;
                            if let Some(line_rope) = doc.content.get_line(line) {
                                let line_str: String = line_rope.chars().collect();
                                let _start = diagnostic.range.start.character as usize;
                                let end = diagnostic.range.end.character as usize;

                                if end <= line_str.len() {
                                    let edit = WorkspaceEdit {
                                        changes: Some({
                                            let mut map = std::collections::HashMap::new();
                                            map.insert(
                                                uri.clone(),
                                                vec![TextEdit {
                                                    range: diagnostic.range,
                                                    new_text: format!("_{}", var_name),
                                                }],
                                            );
                                            map
                                        }),
                                        document_changes: None,
                                        change_annotations: None,
                                    };

                                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                        title: format!("Prefix with underscore: _{}", var_name),
                                        kind: Some(CodeActionKind::QUICKFIX),
                                        diagnostics: Some(vec![diagnostic.clone()]),
                                        edit: Some(edit),
                                        ..Default::default()
                                    }));
                                }
                            }
                        }
                    }

                    // Quick fix for missing return type
                    if diagnostic.message.contains("missing return")
                        || diagnostic.message.contains("expected return")
                    {
                        let line = diagnostic.range.start.line as usize;
                        if let Some(line_rope) = doc.content.get_line(line) {
                            let line_str: String = line_rope.chars().collect();

                            // Find function signature
                            if let Some(paren_pos) = line_str.find(')') {
                                let insert_pos = paren_pos + 1;
                                let position =
                                    Position::new(diagnostic.range.start.line, insert_pos as u32);

                                let edit = WorkspaceEdit {
                                    changes: Some({
                                        let mut map = std::collections::HashMap::new();
                                        map.insert(
                                            uri.clone(),
                                            vec![TextEdit {
                                                range: Range::new(position, position),
                                                new_text: " -> i64".to_string(),
                                            }],
                                        );
                                        map
                                    }),
                                    document_changes: None,
                                    change_annotations: None,
                                };

                                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                    title: "Add return type: -> i64".to_string(),
                                    kind: Some(CodeActionKind::QUICKFIX),
                                    diagnostics: Some(vec![diagnostic.clone()]),
                                    edit: Some(edit),
                                    ..Default::default()
                                }));
                            }
                        }
                    }

                    // Quick fix for missing semicolon/expression
                    if diagnostic.message.contains("expected") && diagnostic.message.contains(";") {
                        let line = diagnostic.range.end.line as usize;
                        if let Some(line_rope) = doc.content.get_line(line) {
                            let line_str: String = line_rope.chars().collect();
                            let line_end = line_str.trim_end().len();

                            let position =
                                Position::new(diagnostic.range.end.line, line_end as u32);

                            let edit = WorkspaceEdit {
                                changes: Some({
                                    let mut map = std::collections::HashMap::new();
                                    map.insert(
                                        uri.clone(),
                                        vec![TextEdit {
                                            range: Range::new(position, position),
                                            new_text: ";".to_string(),
                                        }],
                                    );
                                    map
                                }),
                                document_changes: None,
                                change_annotations: None,
                            };

                            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                title: "Add semicolon".to_string(),
                                kind: Some(CodeActionKind::QUICKFIX),
                                diagnostics: Some(vec![diagnostic.clone()]),
                                edit: Some(edit),
                                ..Default::default()
                            }));
                        }
                    }
                }
            }

            // Refactor: Extract to variable (if there's a selection)
            if range.start != range.end {
                let start_line = range.start.line as usize;
                let end_line = range.end.line as usize;

                // Only support single-line selections for now
                if start_line == end_line {
                    if let Some(line_rope) = doc.content.get_line(start_line) {
                        let line_str: String = line_rope.chars().collect();
                        let start_char = range.start.character as usize;
                        let end_char = range.end.character as usize;

                        if end_char <= line_str.len() {
                            let selected_text = &line_str[start_char..end_char];

                            // Only suggest if selection is not empty and looks like an expression
                            if !selected_text.trim().is_empty()
                                && !selected_text.trim().starts_with("L ")
                            {
                                let var_name = "value";
                                let indent = line_str
                                    .chars()
                                    .take_while(|c| c.is_whitespace())
                                    .collect::<String>();

                                let edit = WorkspaceEdit {
                                    changes: Some({
                                        let mut map = std::collections::HashMap::new();
                                        map.insert(
                                            uri.clone(),
                                            vec![
                                                // Insert variable declaration above
                                                TextEdit {
                                                    range: Range::new(
                                                        Position::new(range.start.line, 0),
                                                        Position::new(range.start.line, 0),
                                                    ),
                                                    new_text: format!(
                                                        "{}L {}: _ = {}\n",
                                                        indent, var_name, selected_text
                                                    ),
                                                },
                                                // Replace selection with variable reference
                                                TextEdit {
                                                    range,
                                                    new_text: var_name.to_string(),
                                                },
                                            ],
                                        );
                                        map
                                    }),
                                    document_changes: None,
                                    change_annotations: None,
                                };

                                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                    title: "Extract to variable".to_string(),
                                    kind: Some(CodeActionKind::REFACTOR_EXTRACT),
                                    diagnostics: None,
                                    edit: Some(edit),
                                    ..Default::default()
                                }));
                            }
                        }
                    }
                }
            }

            // Refactor: Extract to function (for multi-line or complex selections)
            if range.start.line != range.end.line
                || (range.end.character - range.start.character) > 30
            {
                let start_line = range.start.line as usize;
                let end_line = range.end.line as usize;

                // Collect selected lines
                let mut selected_lines = Vec::new();
                for line_idx in start_line..=end_line {
                    if let Some(line_rope) = doc.content.get_line(line_idx) {
                        let line_str: String = line_rope.chars().collect();

                        if line_idx == start_line && line_idx == end_line {
                            // Single line case
                            let start_char = range.start.character as usize;
                            let end_char = range.end.character as usize;
                            if end_char <= line_str.len() {
                                selected_lines.push(line_str[start_char..end_char].to_string());
                            }
                        } else if line_idx == start_line {
                            // First line
                            let start_char = range.start.character as usize;
                            if start_char < line_str.len() {
                                selected_lines.push(line_str[start_char..].to_string());
                            }
                        } else if line_idx == end_line {
                            // Last line
                            let end_char = range.end.character as usize;
                            if end_char <= line_str.len() {
                                selected_lines.push(line_str[..end_char].to_string());
                            }
                        } else {
                            // Middle lines
                            selected_lines.push(line_str);
                        }
                    }
                }

                if !selected_lines.is_empty() {
                    let selected_text = selected_lines.join("\n");

                    if !selected_text.trim().is_empty() {
                        let func_name = "extracted_function";

                        // Find indentation of first line
                        let first_line_str: String = doc
                            .content
                            .get_line(start_line)
                            .map(|rope| rope.chars().collect())
                            .unwrap_or_default();
                        let indent = first_line_str
                            .chars()
                            .take_while(|c| c.is_whitespace())
                            .collect::<String>();

                        // Create function definition
                        let function_def = format!(
                            "\nF {}() -> _ {{\n{}{}\n}}\n",
                            func_name,
                            indent,
                            selected_text
                                .lines()
                                .collect::<Vec<_>>()
                                .join(&format!("\n{}", indent))
                        );

                        let edit = WorkspaceEdit {
                            changes: Some({
                                let mut map = std::collections::HashMap::new();
                                map.insert(
                                    uri.clone(),
                                    vec![
                                        // Insert function at top of file
                                        TextEdit {
                                            range: Range::new(
                                                Position::new(0, 0),
                                                Position::new(0, 0),
                                            ),
                                            new_text: function_def,
                                        },
                                        // Replace selection with function call
                                        TextEdit {
                                            range,
                                            new_text: format!("{}()", func_name),
                                        },
                                    ],
                                );
                                map
                            }),
                            document_changes: None,
                            change_annotations: None,
                        };

                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Extract to function".to_string(),
                            kind: Some(CodeActionKind::REFACTOR_EXTRACT),
                            diagnostics: None,
                            edit: Some(edit),
                            ..Default::default()
                        }));
                    }
                }
            }

            // Refactor: Inline Variable
            if let Some(ast) = &doc.ast {
                // Convert range.start to offset
                let cursor_line = range.start.line as usize;
                let cursor_char = range.start.character as usize;
                let cursor_offset =
                    if let Ok(line_start_char) = doc.content.try_line_to_char(cursor_line) {
                        line_start_char + cursor_char
                    } else {
                        0
                    };

                // Find Let statement at cursor
                for item in &ast.items {
                    if let Item::Function(func) = &item.node {
                        if let FunctionBody::Block(stmts) = &func.body {
                            for (stmt_idx, stmt) in stmts.iter().enumerate() {
                                if let Stmt::Let { name, value, .. } = &stmt.node {
                                    // Check if cursor is on this let statement
                                    if cursor_offset >= stmt.span.start
                                        && cursor_offset <= stmt.span.end
                                    {
                                        let var_name = &name.node;

                                        // Get the initializer expression text
                                        let init_text: String = doc
                                            .content
                                            .chars()
                                            .skip(value.span.start)
                                            .take(value.span.end - value.span.start)
                                            .collect();

                                        // Find all references to this variable in the function
                                        let mut reference_ranges = Vec::new();

                                        // Look in subsequent statements for references
                                        for ref_stmt in &stmts[stmt_idx + 1..] {
                                            self.find_var_references_in_stmt(
                                                ref_stmt,
                                                var_name,
                                                &mut reference_ranges,
                                                &doc.content,
                                            );
                                        }

                                        if !reference_ranges.is_empty() {
                                            let mut edits = Vec::new();

                                            // Remove the let statement line
                                            let let_range =
                                                self.span_to_range(&doc.content, &stmt.span);
                                            // Extend to include the whole line
                                            let let_line_start =
                                                Position::new(let_range.start.line, 0);
                                            let let_line_end =
                                                Position::new(let_range.end.line + 1, 0);
                                            edits.push(TextEdit {
                                                range: Range::new(let_line_start, let_line_end),
                                                new_text: String::new(),
                                            });

                                            // Replace each reference with the initializer
                                            for ref_range in reference_ranges {
                                                edits.push(TextEdit {
                                                    range: ref_range,
                                                    new_text: init_text.clone(),
                                                });
                                            }

                                            let edit = WorkspaceEdit {
                                                changes: Some({
                                                    let mut map = std::collections::HashMap::new();
                                                    map.insert(uri.clone(), edits);
                                                    map
                                                }),
                                                document_changes: None,
                                                change_annotations: None,
                                            };

                                            actions.push(CodeActionOrCommand::CodeAction(
                                                CodeAction {
                                                    title: format!(
                                                        "Inline variable '{}'",
                                                        var_name
                                                    ),
                                                    kind: Some(CodeActionKind::REFACTOR_INLINE),
                                                    diagnostics: None,
                                                    edit: Some(edit),
                                                    ..Default::default()
                                                },
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Refactor: Convert to/from Expression Body
            if let Some(ast) = &doc.ast {
                let cursor_line = range.start.line as usize;
                let cursor_char = range.start.character as usize;
                let _cursor_offset =
                    if let Ok(line_start_char) = doc.content.try_line_to_char(cursor_line) {
                        line_start_char + cursor_char
                    } else {
                        0
                    };

                for item in &ast.items {
                    if let Item::Function(func) = &item.node {
                        let func_range = self.span_to_range(&doc.content, &item.span);

                        // Check if cursor is in this function
                        if position_in_range(&range.start, &func_range) {
                            match &func.body {
                                // Convert block body to expression body
                                FunctionBody::Block(stmts) if stmts.len() == 1 => {
                                    let stmt = &stmts[0];
                                    let expr_span = match &stmt.node {
                                        Stmt::Return(Some(expr)) => Some(&expr.span),
                                        Stmt::Expr(expr) => Some(&expr.span),
                                        _ => None,
                                    };

                                    if let Some(expr_span) = expr_span {
                                        // Get expression text
                                        let expr_text: String = doc
                                            .content
                                            .chars()
                                            .skip(expr_span.start)
                                            .take(expr_span.end - expr_span.start)
                                            .collect();

                                        // Find the opening brace of the function body
                                        let body_start =
                                            if let FunctionBody::Block(stmts) = &func.body {
                                                if let Some(first_stmt) = stmts.first() {
                                                    // Work backwards from first statement to find '{'
                                                    let mut brace_offset = first_stmt.span.start;
                                                    while brace_offset > 0 {
                                                        brace_offset -= 1;
                                                        if let Some(ch) =
                                                            doc.content.get_char(brace_offset)
                                                        {
                                                            if ch == '{' {
                                                                break;
                                                            }
                                                        }
                                                    }
                                                    brace_offset
                                                } else {
                                                    item.span.start
                                                }
                                            } else {
                                                item.span.start
                                            };

                                        let body_end = item.span.end;

                                        let edit = WorkspaceEdit {
                                            changes: Some({
                                                let mut map = std::collections::HashMap::new();
                                                map.insert(
                                                    uri.clone(),
                                                    vec![TextEdit {
                                                        range: Range {
                                                            start: self.offset_to_position(
                                                                &doc.content,
                                                                body_start,
                                                            ),
                                                            end: self.offset_to_position(
                                                                &doc.content,
                                                                body_end,
                                                            ),
                                                        },
                                                        new_text: format!("= {}", expr_text),
                                                    }],
                                                );
                                                map
                                            }),
                                            document_changes: None,
                                            change_annotations: None,
                                        };

                                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                            title: "Convert to expression body".to_string(),
                                            kind: Some(CodeActionKind::REFACTOR_REWRITE),
                                            diagnostics: None,
                                            edit: Some(edit),
                                            ..Default::default()
                                        }));
                                    }
                                }
                                // Convert expression body to block body
                                FunctionBody::Expr(expr) => {
                                    let expr_text: String = doc
                                        .content
                                        .chars()
                                        .skip(expr.span.start)
                                        .take(expr.span.end - expr.span.start)
                                        .collect();

                                    // Find the '=' before the expression
                                    let mut eq_offset = expr.span.start;
                                    while eq_offset > 0 {
                                        eq_offset -= 1;
                                        if let Some(ch) = doc.content.get_char(eq_offset) {
                                            if ch == '=' {
                                                break;
                                            }
                                        }
                                    }

                                    let edit = WorkspaceEdit {
                                        changes: Some({
                                            let mut map = std::collections::HashMap::new();
                                            map.insert(
                                                uri.clone(),
                                                vec![TextEdit {
                                                    range: Range {
                                                        start: self.offset_to_position(
                                                            &doc.content,
                                                            eq_offset,
                                                        ),
                                                        end: self.offset_to_position(
                                                            &doc.content,
                                                            expr.span.end,
                                                        ),
                                                    },
                                                    new_text: format!("{{\n    {}\n}}", expr_text),
                                                }],
                                            );
                                            map
                                        }),
                                        document_changes: None,
                                        change_annotations: None,
                                    };

                                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                        title: "Convert to block body".to_string(),
                                        kind: Some(CodeActionKind::REFACTOR_REWRITE),
                                        diagnostics: None,
                                        edit: Some(edit),
                                        ..Default::default()
                                    }));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            // Refactor: Introduce Named Parameter
            if let Some(ast) = &doc.ast {
                let cursor_line = range.start.line as usize;
                let cursor_char = range.start.character as usize;
                let cursor_offset =
                    if let Ok(line_start_char) = doc.content.try_line_to_char(cursor_line) {
                        line_start_char + cursor_char
                    } else {
                        0
                    };

                // Find function call at cursor and offer to convert to named arguments
                for item in &ast.items {
                    if let Item::Function(func) = &item.node {
                        if let FunctionBody::Block(stmts) = &func.body {
                            for stmt in stmts {
                                self.find_call_at_cursor_in_stmt(
                                    stmt,
                                    cursor_offset,
                                    &doc.content,
                                    ast,
                                    uri,
                                    &mut actions,
                                );
                            }
                        } else if let FunctionBody::Expr(expr) = &func.body {
                            self.find_call_at_cursor_in_expr(
                                expr,
                                cursor_offset,
                                &doc.content,
                                ast,
                                uri,
                                &mut actions,
                            );
                        }
                    }
                }
            }
        }

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    // ===== New LSP Methods =====

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

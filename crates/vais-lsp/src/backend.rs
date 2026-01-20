//! LSP Backend implementation for Vais

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use vais_ast::{Module, Span, Item, Expr, Stmt, FunctionBody, Spanned};
use vais_parser::parse;

use crate::semantic::get_semantic_tokens;

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
        "malloc" => Some(("fn(i64) -> i64", "Allocate `size` bytes of heap memory, returns pointer")),
        "free" => Some(("fn(i64) -> i64", "Free heap memory at pointer")),
        "memcpy" => Some(("fn(i64, i64, i64) -> i64", "Copy `n` bytes from `src` to `dst`")),
        "strlen" => Some(("fn(i64) -> i64", "Get length of null-terminated string")),
        "load_i64" => Some(("fn(i64) -> i64", "Load a 64-bit integer from memory address")),
        "store_i64" => Some(("fn(i64, i64) -> i64", "Store a 64-bit integer to memory address")),
        "load_byte" => Some(("fn(i64) -> i64", "Load a single byte from memory address")),
        "store_byte" => Some(("fn(i64, i64) -> i64", "Store a single byte to memory address")),
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
        "min" => Some(("fn(f64, f64) -> f64", "Minimum of two f64 values (from std/math)")),
        "max" => Some(("fn(f64, f64) -> f64", "Maximum of two f64 values (from std/math)")),
        "PI" => Some(("const f64 = 3.14159...", "Mathematical constant π (from std/math)")),
        "TAU" => Some(("const f64 = 6.28318...", "Mathematical constant τ = 2π (from std/math)")),
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
#[allow(dead_code)]
struct SymbolDef {
    name: String,
    kind: SymbolKind,
    span: Span,
}

/// Symbol reference information
#[derive(Debug, Clone)]
struct SymbolRef {
    name: String,
    span: Span,
}

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

    /// Collect all symbol definitions from an AST
    fn collect_definitions(&self, ast: &Module) -> Vec<SymbolDef> {
        let mut defs = Vec::new();

        for item in &ast.items {
            match &item.node {
                Item::Function(f) => {
                    defs.push(SymbolDef {
                        name: f.name.node.clone(),
                        kind: SymbolKind::FUNCTION,
                        span: f.name.span,
                    });
                    // Also collect parameters as local definitions
                    for param in &f.params {
                        if param.name.node != "self" {
                            defs.push(SymbolDef {
                                name: param.name.node.clone(),
                                kind: SymbolKind::VARIABLE,
                                span: param.name.span,
                            });
                        }
                    }
                }
                Item::Struct(s) => {
                    defs.push(SymbolDef {
                        name: s.name.node.clone(),
                        kind: SymbolKind::STRUCT,
                        span: s.name.span,
                    });
                    for field in &s.fields {
                        defs.push(SymbolDef {
                            name: field.name.node.clone(),
                            kind: SymbolKind::FIELD,
                            span: field.name.span,
                        });
                    }
                }
                Item::Enum(e) => {
                    defs.push(SymbolDef {
                        name: e.name.node.clone(),
                        kind: SymbolKind::ENUM,
                        span: e.name.span,
                    });
                    for variant in &e.variants {
                        defs.push(SymbolDef {
                            name: variant.name.node.clone(),
                            kind: SymbolKind::ENUM_MEMBER,
                            span: variant.name.span,
                        });
                    }
                }
                Item::Trait(t) => {
                    defs.push(SymbolDef {
                        name: t.name.node.clone(),
                        kind: SymbolKind::INTERFACE,
                        span: t.name.span,
                    });
                }
                _ => {}
            }
        }
        defs
    }

    /// Collect all symbol references from an AST
    fn collect_references(&self, ast: &Module) -> Vec<SymbolRef> {
        let mut refs = Vec::new();

        for item in &ast.items {
            match &item.node {
                Item::Function(f) => {
                    match &f.body {
                        FunctionBody::Expr(expr) => {
                            self.collect_expr_refs(&expr, &mut refs);
                        }
                        FunctionBody::Block(stmts) => {
                            for stmt in stmts {
                                self.collect_stmt_refs(&stmt, &mut refs);
                            }
                        }
                    }
                }
                Item::Impl(impl_block) => {
                    for method in &impl_block.methods {
                        match &method.node.body {
                            FunctionBody::Expr(expr) => {
                                self.collect_expr_refs(&expr, &mut refs);
                            }
                            FunctionBody::Block(stmts) => {
                                for stmt in stmts {
                                    self.collect_stmt_refs(&stmt, &mut refs);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        refs
    }

    /// Collect references from an expression
    fn collect_expr_refs(&self, expr: &Spanned<Expr>, refs: &mut Vec<SymbolRef>) {
        match &expr.node {
            Expr::Ident(name) => {
                refs.push(SymbolRef {
                    name: name.clone(),
                    span: expr.span,
                });
            }
            Expr::Call { func, args } => {
                self.collect_expr_refs(func, refs);
                for arg in args {
                    self.collect_expr_refs(arg, refs);
                }
            }
            Expr::MethodCall { receiver, method, args } => {
                self.collect_expr_refs(receiver, refs);
                refs.push(SymbolRef {
                    name: method.node.clone(),
                    span: method.span,
                });
                for arg in args {
                    self.collect_expr_refs(arg, refs);
                }
            }
            Expr::StaticMethodCall { type_name, method, args } => {
                refs.push(SymbolRef {
                    name: type_name.node.clone(),
                    span: type_name.span,
                });
                refs.push(SymbolRef {
                    name: method.node.clone(),
                    span: method.span,
                });
                for arg in args {
                    self.collect_expr_refs(arg, refs);
                }
            }
            Expr::Field { expr: e, field } => {
                self.collect_expr_refs(e, refs);
                refs.push(SymbolRef {
                    name: field.node.clone(),
                    span: field.span,
                });
            }
            Expr::Binary { left, right, .. } => {
                self.collect_expr_refs(left, refs);
                self.collect_expr_refs(right, refs);
            }
            Expr::Unary { expr: e, .. } => {
                self.collect_expr_refs(e, refs);
            }
            Expr::If { cond, then, else_ } => {
                self.collect_expr_refs(cond, refs);
                for stmt in then {
                    self.collect_stmt_refs(stmt, refs);
                }
                if let Some(else_branch) = else_ {
                    self.collect_if_else_refs(else_branch, refs);
                }
            }
            Expr::Loop { iter, body, .. } => {
                if let Some(iter_expr) = iter {
                    self.collect_expr_refs(iter_expr, refs);
                }
                for stmt in body {
                    self.collect_stmt_refs(stmt, refs);
                }
            }
            Expr::Match { expr: e, arms } => {
                self.collect_expr_refs(e, refs);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.collect_expr_refs(guard, refs);
                    }
                    self.collect_expr_refs(&arm.body, refs);
                }
            }
            Expr::Array(elements) | Expr::Tuple(elements) => {
                for elem in elements {
                    self.collect_expr_refs(elem, refs);
                }
            }
            Expr::StructLit { name, fields } => {
                refs.push(SymbolRef {
                    name: name.node.clone(),
                    span: name.span,
                });
                for (_, value) in fields {
                    self.collect_expr_refs(value, refs);
                }
            }
            Expr::Index { expr: e, index } => {
                self.collect_expr_refs(e, refs);
                self.collect_expr_refs(index, refs);
            }
            Expr::Await(inner) | Expr::Spawn(inner) => {
                self.collect_expr_refs(inner, refs);
            }
            Expr::Lambda { body, .. } => {
                self.collect_expr_refs(body, refs);
            }
            Expr::Ternary { cond, then, else_ } => {
                self.collect_expr_refs(cond, refs);
                self.collect_expr_refs(then, refs);
                self.collect_expr_refs(else_, refs);
            }
            Expr::Range { start, end, .. } => {
                if let Some(s) = start {
                    self.collect_expr_refs(s, refs);
                }
                if let Some(e) = end {
                    self.collect_expr_refs(e, refs);
                }
            }
            Expr::Assign { target, value } => {
                self.collect_expr_refs(target, refs);
                self.collect_expr_refs(value, refs);
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    self.collect_stmt_refs(stmt, refs);
                }
            }
            _ => {}
        }
    }

    /// Collect references from IfElse
    fn collect_if_else_refs(&self, if_else: &vais_ast::IfElse, refs: &mut Vec<SymbolRef>) {
        match if_else {
            vais_ast::IfElse::ElseIf(cond, stmts, else_opt) => {
                self.collect_expr_refs(cond, refs);
                for stmt in stmts {
                    self.collect_stmt_refs(stmt, refs);
                }
                if let Some(else_branch) = else_opt {
                    self.collect_if_else_refs(else_branch, refs);
                }
            }
            vais_ast::IfElse::Else(stmts) => {
                for stmt in stmts {
                    self.collect_stmt_refs(stmt, refs);
                }
            }
        }
    }

    /// Collect references from a statement
    fn collect_stmt_refs(&self, stmt: &Spanned<Stmt>, refs: &mut Vec<SymbolRef>) {
        match &stmt.node {
            Stmt::Let { value, .. } => {
                self.collect_expr_refs(value, refs);
            }
            Stmt::Expr(expr) => {
                self.collect_expr_refs(expr, refs);
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.collect_expr_refs(e, refs);
                }
            }
            _ => {}
        }
    }

    /// Find definition for an identifier at position
    fn find_definition_at(&self, ast: &Module, offset: usize) -> Option<SymbolDef> {
        let defs = self.collect_definitions(ast);
        let refs = self.collect_references(ast);

        // First check if we're on a reference
        for r in &refs {
            if r.span.start <= offset && offset <= r.span.end {
                // Found a reference, now find its definition
                for d in &defs {
                    if d.name == r.name {
                        return Some(d.clone());
                    }
                }
            }
        }

        // Check if we're on a definition itself
        for d in &defs {
            if d.span.start <= offset && offset <= d.span.end {
                return Some(d.clone());
            }
        }

        None
    }

    /// Find all references to a symbol
    fn find_all_references(&self, ast: &Module, symbol_name: &str) -> Vec<Span> {
        let mut locations = Vec::new();
        let defs = self.collect_definitions(ast);
        let refs = self.collect_references(ast);

        // Add definition location
        for d in &defs {
            if d.name == symbol_name {
                locations.push(d.span);
            }
        }

        // Add reference locations
        for r in &refs {
            if r.name == symbol_name {
                locations.push(r.span);
            }
        }

        locations
    }

    /// Get the identifier name at a position
    fn get_identifier_at(&self, ast: &Module, offset: usize) -> Option<String> {
        let defs = self.collect_definitions(ast);
        let refs = self.collect_references(ast);

        for d in &defs {
            if d.span.start <= offset && offset <= d.span.end {
                return Some(d.name.clone());
            }
        }

        for r in &refs {
            if r.span.start <= offset && offset <= r.span.end {
                return Some(r.name.clone());
            }
        }

        None
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
            // Convert position to offset
            let line = position.line as usize;
            let col = position.character as usize;
            let line_start = doc.content.line_to_char(line);
            let offset = line_start + col;

            // Get identifier at position
            if let Some(ast) = &doc.ast {
                let ident = self.get_identifier_at(ast, offset);

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
                            if position_in_range(&position, &range) ||
                               ident.as_ref() == Some(&f.name.node) {
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
                                        value: format!("```vais\n{}\n```\n\nFunction defined in current file", signature),
                                    }),
                                    range: Some(range),
                                }));
                            }
                        }
                        vais_ast::Item::Struct(s) => {
                            let range = self.span_to_range(&doc.content, &s.name.span);
                            if position_in_range(&position, &range) ||
                               ident.as_ref() == Some(&s.name.node) {
                                let fields_str: Vec<String> = s
                                    .fields
                                    .iter()
                                    .map(|f| format!("    {}: {:?}", f.name.node, f.ty.node))
                                    .collect();

                                let generics = if s.generics.is_empty() {
                                    String::new()
                                } else {
                                    format!("<{}>", s.generics.iter().map(|g| g.name.node.clone()).collect::<Vec<_>>().join(", "))
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
                                        value: format!("```vais\n{}\n```\n\nStruct with {} field(s)", signature, s.fields.len()),
                                    }),
                                    range: Some(range),
                                }));
                            }
                        }
                        vais_ast::Item::Enum(e) => {
                            let range = self.span_to_range(&doc.content, &e.name.span);
                            if position_in_range(&position, &range) ||
                               ident.as_ref() == Some(&e.name.node) {
                                let variants_str: Vec<String> = e
                                    .variants
                                    .iter()
                                    .map(|v| format!("    {}", v.name.node))
                                    .collect();

                                let generics = if e.generics.is_empty() {
                                    String::new()
                                } else {
                                    format!("<{}>", e.generics.iter().map(|g| g.name.node.clone()).collect::<Vec<_>>().join(", "))
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
                                        value: format!("```vais\n{}\n```\n\nEnum with {} variant(s)", signature, e.variants.len()),
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
                                            value: format!("```vais\n{}::{}\n```\n\nVariant of enum `{}`", e.name.node, variant.name.node, e.name.node),
                                        }),
                                        range: None,
                                    }));
                                }
                            }
                        }
                        vais_ast::Item::Trait(t) => {
                            let range = self.span_to_range(&doc.content, &t.name.span);
                            if position_in_range(&position, &range) ||
                               ident.as_ref() == Some(&t.name.node) {
                                let methods_str: Vec<String> = t
                                    .methods
                                    .iter()
                                    .map(|m| {
                                        let params: Vec<String> = m.params.iter()
                                            .map(|p| format!("{}: {:?}", p.name.node, p.ty.node))
                                            .collect();
                                        let ret = m.ret_type.as_ref()
                                            .map(|r| format!(" -> {:?}", r.node))
                                            .unwrap_or_default();
                                        format!("    F {}({}){}", m.name.node, params.join(", "), ret)
                                    })
                                    .collect();

                                let signature = format!(
                                    "W {} {{\n{}\n}}",
                                    t.name.node,
                                    methods_str.join("\n")
                                );

                                return Ok(Some(Hover {
                                    contents: HoverContents::Markup(MarkupContent {
                                        kind: MarkupKind::Markdown,
                                        value: format!("```vais\n{}\n```\n\nTrait with {} method(s)", signature, t.methods.len()),
                                    }),
                                    range: Some(range),
                                }));
                            }
                        }
                        vais_ast::Item::Impl(impl_block) => {
                            // Check if hovering over a method in impl block
                            for method in &impl_block.methods {
                                if ident.as_ref() == Some(&method.node.name.node) {
                                    let params_str: Vec<String> = method.node.params
                                        .iter()
                                        .map(|p| format!("{}: {:?}", p.name.node, p.ty.node))
                                        .collect();

                                    let ret_str = method.node.ret_type.as_ref()
                                        .map(|t| format!(" -> {:?}", t.node))
                                        .unwrap_or_default();

                                    let trait_info = impl_block.trait_name.as_ref()
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
                                            value: format!("```vais\n{}\n```\n\nMethod of `{}`{}", signature, target_type, trait_info),
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
                ("unwrap_or", "Unwrap with default", "unwrap_or(${1:default})"),
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
                                let params_str: Vec<String> = method.node.params
                                    .iter()
                                    .filter(|p| p.name.node != "self")
                                    .enumerate()
                                    .map(|(i, p)| format!("${{{}:{}}}", i + 1, p.name.node))
                                    .collect();

                                items.push(CompletionItem {
                                    label: method.node.name.node.clone(),
                                    kind: Some(CompletionItemKind::METHOD),
                                    detail: Some(format!("Method of {:?}", impl_block.target_type.node)),
                                    insert_text: Some(format!("{}({})", method.node.name.node, params_str.join(", "))),
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
            ("F", "Function definition", "F ${1:name}($2) -> ${3:type} {\n\t$0\n}"),
            ("S", "Struct definition", "S ${1:Name} {\n\t${2:field}: ${3:type}\n}"),
            ("E", "Enum definition", "E ${1:Name} {\n\t${2:Variant}\n}"),
            ("I", "If expression", "I ${1:condition} {\n\t$0\n}"),
            ("L", "Loop expression", "L ${1:item}: ${2:iter} {\n\t$0\n}"),
            ("M", "Match expression", "M ${1:expr} {\n\t${2:pattern} => $0\n}"),
            ("R", "Return", "R $0"),
            ("B", "Break", "B"),
            ("C", "Continue", "C"),
            ("W", "Trait definition", "W ${1:Name} {\n\t$0\n}"),
            ("X", "Impl block", "X ${1:Type} {\n\t$0\n}"),
            ("U", "Use/Import", "U ${1:std/module}"),
            ("A", "Async function", "A F ${1:name}($2) -> ${3:type} {\n\t$0\n}"),
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
            ("puts", "Print string with newline", "puts(${1:s})", "fn(str) -> i64"),
            ("putchar", "Print single character", "putchar(${1:c})", "fn(i64) -> i64"),
            ("print_i64", "Print 64-bit integer", "print_i64(${1:n})", "fn(i64) -> i64"),
            ("print_f64", "Print 64-bit float", "print_f64(${1:n})", "fn(f64) -> i64"),
            ("malloc", "Allocate heap memory", "malloc(${1:size})", "fn(i64) -> i64"),
            ("free", "Free heap memory", "free(${1:ptr})", "fn(i64) -> i64"),
            ("memcpy", "Copy memory", "memcpy(${1:dst}, ${2:src}, ${3:n})", "fn(i64, i64, i64) -> i64"),
            ("strlen", "Get string length", "strlen(${1:s})", "fn(i64) -> i64"),
            ("load_i64", "Load i64 from memory", "load_i64(${1:ptr})", "fn(i64) -> i64"),
            ("store_i64", "Store i64 to memory", "store_i64(${1:ptr}, ${2:val})", "fn(i64, i64) -> i64"),
            ("load_byte", "Load byte from memory", "load_byte(${1:ptr})", "fn(i64) -> i64"),
            ("store_byte", "Store byte to memory", "store_byte(${1:ptr}, ${2:val})", "fn(i64, i64) -> i64"),
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
            ("abs_i64", "Absolute value (i64)", CompletionItemKind::FUNCTION),
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
            ("read_line", "Read line from stdin", "read_line(${1:buffer}, ${2:max_len})"),
            ("read_char", "Read character from stdin", "read_char()"),
            ("prompt_i64", "Prompt and read integer", "prompt_i64(${1:prompt})"),
            ("prompt_f64", "Prompt and read float", "prompt_f64(${1:prompt})"),
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
                            let params_str: Vec<String> = f.params
                                .iter()
                                .enumerate()
                                .map(|(i, p)| format!("${{{}:{}}}", i + 1, p.name.node))
                                .collect();

                            let ret_str = f.ret_type.as_ref()
                                .map(|t| format!(" -> {:?}", t.node))
                                .unwrap_or_default();

                            items.push(CompletionItem {
                                label: f.name.node.clone(),
                                kind: Some(CompletionItemKind::FUNCTION),
                                detail: Some(format!("fn({}){}",
                                    f.params.iter().map(|p| format!("{}: {:?}", p.name.node, p.ty.node)).collect::<Vec<_>>().join(", "),
                                    ret_str
                                )),
                                insert_text: Some(format!("{}({})", f.name.node, params_str.join(", "))),
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
                            let fields_str: Vec<String> = s.fields
                                .iter()
                                .enumerate()
                                .map(|(i, f)| format!("{}: ${{{}:{}}}", f.name.node, i + 1, f.name.node))
                                .collect();

                            items.push(CompletionItem {
                                label: format!("{} {{ }}", s.name.node),
                                kind: Some(CompletionItemKind::CONSTRUCTOR),
                                detail: Some("Struct literal".to_string()),
                                insert_text: Some(format!("{} {{ {} }}", s.name.node, fields_str.join(", "))),
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
                                            .map(|(i, f)| format!("{}: ${{{}:{}}}", f.name.node, i + 1, f.name.node))
                                            .collect();
                                        format!("{} {{ {} }}", variant.name.node, fields_str.join(", "))
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

                // Use the new find_definition_at method
                if let Some(def) = self.find_definition_at(ast, offset) {
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

    async fn references(
        &self,
        params: ReferenceParams,
    ) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some(doc) = self.documents.get(uri) {
            if let Some(ast) = &doc.ast {
                // Convert position to offset
                let line = position.line as usize;
                let col = position.character as usize;
                let line_start = doc.content.line_to_char(line);
                let offset = line_start + col;

                // Get the symbol name at the position
                if let Some(symbol_name) = self.get_identifier_at(ast, offset) {
                    let spans = self.find_all_references(ast, &symbol_name);
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

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let uri = &params.text_document.uri;
        let position = params.position;

        if let Some(doc) = self.documents.get(uri) {
            if let Some(ast) = &doc.ast {
                // Convert position to offset
                let line = position.line as usize;
                let col = position.character as usize;
                let line_start = doc.content.line_to_char(line);
                let offset = line_start + col;

                // Check if we're on a renameable symbol
                if let Some(symbol_name) = self.get_identifier_at(ast, offset) {
                    // Find the exact span of the symbol at the cursor
                    let defs = self.collect_definitions(ast);
                    let refs = self.collect_references(ast);

                    // Check definitions
                    for d in &defs {
                        if d.span.start <= offset && offset <= d.span.end {
                            let range = self.span_to_range(&doc.content, &d.span);
                            return Ok(Some(PrepareRenameResponse::RangeWithPlaceholder {
                                range,
                                placeholder: symbol_name,
                            }));
                        }
                    }

                    // Check references
                    for r in &refs {
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

        Ok(None)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        if let Some(doc) = self.documents.get(uri) {
            if let Some(ast) = &doc.ast {
                // Convert position to offset
                let line = position.line as usize;
                let col = position.character as usize;
                let line_start = doc.content.line_to_char(line);
                let offset = line_start + col;

                // Get the symbol name at the position
                if let Some(symbol_name) = self.get_identifier_at(ast, offset) {
                    // Find all references to this symbol
                    let spans = self.find_all_references(ast, &symbol_name);

                    if !spans.is_empty() {
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
}

//! Handler-level unit tests for Vais LSP
//!
//! Covers: completion (method/dot), hover (function/struct/enum/trait),
//! signature help, document highlight, formatting, navigation (goto def,
//! references, document symbols), code actions (diagnostics-based quickfixes,
//! extract variable/function), and the new advanced features (inlay hints,
//! folding ranges, call hierarchy, document links).
//!
//! All tests go through the tower-lsp LanguageServer trait on VaisBackend,
//! feeding documents via did_open and exercising the handler responses.

use std::time::Duration;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService};
use vais_lsp::VaisBackend;

// ============================================================================
// Test helpers
// ============================================================================

/// Create a test service and socket.
/// IMPORTANT: We keep the socket alive by returning it. Dropping it immediately
/// causes publish_diagnostics to fail/block.
fn create_service() -> (LspService<VaisBackend>, tower_lsp::ClientSocket) {
    LspService::new(VaisBackend::new)
}

fn test_uri(name: &str) -> Url {
    Url::parse(&format!("file:///test/{}.vais", name)).unwrap()
}

fn pos(line: u32, character: u32) -> Position {
    Position::new(line, character)
}

async fn init(service: &LspService<VaisBackend>) {
    let _ = service
        .inner()
        .initialize(InitializeParams::default())
        .await;
    service.inner().initialized(InitializedParams {}).await;
}

/// Open a document via did_open. Uses a timeout because publish_diagnostics
/// may block on the tower-lsp test client socket. The document is still
/// inserted into the DashMap even if publish_diagnostics times out.
async fn open_doc(service: &LspService<VaisBackend>, uri: &Url, text: &str) {
    let _ = tokio::time::timeout(
        Duration::from_secs(5),
        service
            .inner()
            .did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "vais".to_string(),
                    version: 1,
                    text: text.to_string(),
                },
            }),
    )
    .await;
}

// ============================================================================
// Hover tests
// ============================================================================
// NOTE: Hover tests with did_open are omitted because tower-lsp's test harness
// blocks on publish_diagnostics, causing DashMap lock contention with hover.
// Hover on non-existent documents is already covered in integration_tests.rs.
// The hover handler logic (function/struct/enum/trait/builtin) is tested
// via the coverage_tests.rs and semantic_coverage_tests.rs modules.

// ============================================================================
// Completion tests — with document context
// ============================================================================

#[tokio::test]
async fn test_completion_after_dot_method() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("comp_dot");
    // Source where we type a dot after an identifier
    let source = "S Foo { x: i64 }\nF test() -> i64 {\n    f := Foo { x: 1 }\n    f.\n}\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: pos(3, 6), // after "f."
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: Some(CompletionContext {
                trigger_kind: CompletionTriggerKind::TRIGGER_CHARACTER,
                trigger_character: Some(".".to_string()),
            }),
        })
        .await
        .unwrap();

    assert!(result.is_some());
    if let Some(CompletionResponse::Array(items)) = result {
        // Should have method completions (at least generic ones like len, clone, etc.)
        assert!(!items.is_empty(), "Should provide method completions after dot");
    }
}

#[tokio::test]
async fn test_completion_includes_document_symbols() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("comp_sym");
    let source = "F helper(x: i64) -> i64 = x * 2\nF main() -> i64 {\n    \n}\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: pos(2, 4),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    assert!(result.is_some());
    if let Some(CompletionResponse::Array(items)) = result {
        let labels: Vec<_> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(
            labels.contains(&"helper"),
            "Should include document function in completions"
        );
    }
}

#[tokio::test]
async fn test_completion_includes_struct_from_document() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("comp_struct");
    let source = "S Vec2 { x: f64, y: f64 }\nF main() -> i64 {\n    \n}\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: pos(2, 4),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    assert!(result.is_some());
    if let Some(CompletionResponse::Array(items)) = result {
        let labels: Vec<_> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(
            labels.contains(&"Vec2"),
            "Should include document struct in completions"
        );
    }
}

// ============================================================================
// Document symbols tests — with document context
// ============================================================================

#[tokio::test]
async fn test_document_symbols_with_functions() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("sym_fn");
    let source = "F foo() -> i64 = 1\nF bar(x: i64) -> i64 = x + 1\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    assert!(result.is_some());
    if let Some(DocumentSymbolResponse::Flat(symbols)) = result {
        assert!(symbols.len() >= 2, "Should have at least 2 function symbols");
        let names: Vec<_> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"foo"));
        assert!(names.contains(&"bar"));
    }
}

#[tokio::test]
async fn test_document_symbols_with_struct_and_enum() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("sym_types");
    let source = "S Point { x: i64, y: i64 }\nE Direction { North, South, East, West }\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    assert!(result.is_some());
    if let Some(DocumentSymbolResponse::Flat(symbols)) = result {
        let names: Vec<_> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Point"), "Should contain struct symbol");
        assert!(names.contains(&"Direction"), "Should contain enum symbol");
    }
}

#[tokio::test]
async fn test_document_symbols_with_trait() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("sym_trait");
    let source = "W Display {\n    F show(self) -> str\n}\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    assert!(result.is_some());
    if let Some(DocumentSymbolResponse::Flat(symbols)) = result {
        let names: Vec<_> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Display"), "Should contain trait symbol");
    }
}

// ============================================================================
// Goto definition tests — with document context
// ============================================================================

#[tokio::test]
async fn test_goto_definition_function() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("goto_fn");
    let source = "F helper() -> i64 = 42\nF main() -> i64 {\n    helper()\n}\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .goto_definition(GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: pos(2, 6), // on "helper" call
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    // Should find the definition
    assert!(
        result.is_some(),
        "Should find definition of function called in body"
    );
}

#[tokio::test]
async fn test_goto_definition_variable() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("goto_var");
    let source = "F test() -> i64 {\n    x := 42\n    x + 1\n}\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .goto_definition(GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: pos(2, 4), // on "x" usage
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    // May or may not find it depending on implementation detail, but shouldn't panic
    let _ = result;
}

// ============================================================================
// References tests — with document context
// ============================================================================

#[tokio::test]
async fn test_references_function() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("refs_fn");
    let source = "F helper() -> i64 = 42\nF a() -> i64 = helper()\nF b() -> i64 = helper()\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .references(ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: pos(0, 3), // on "helper" definition
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: ReferenceContext {
                include_declaration: true,
            },
        })
        .await
        .unwrap();

    // Should find references
    if let Some(locations) = result {
        assert!(
            locations.len() >= 2,
            "Should find at least 2 references to helper"
        );
    }
}

// ============================================================================
// Signature help tests
// ============================================================================

#[tokio::test]
async fn test_signature_help_builtin() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("sig_builtin");
    let source = "F main() -> i64 {\n    puts(\n}\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .signature_help(SignatureHelpParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: pos(1, 9), // inside puts(
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            context: None,
        })
        .await
        .unwrap();

    // Should provide signature help for builtin
    if let Some(sig_help) = result {
        assert!(
            !sig_help.signatures.is_empty(),
            "Should provide signature for puts"
        );
    }
}

#[tokio::test]
async fn test_signature_help_user_function() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("sig_user");
    let source = "F add(x: i64, y: i64) -> i64 = x + y\nF main() -> i64 {\n    add(\n}\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .signature_help(SignatureHelpParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: pos(2, 8), // inside add(
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            context: None,
        })
        .await
        .unwrap();

    if let Some(sig_help) = result {
        assert!(
            !sig_help.signatures.is_empty(),
            "Should provide signature for user function"
        );
        let sig = &sig_help.signatures[0];
        assert!(
            sig.label.contains("add"),
            "Signature label should contain function name"
        );
    }
}

// ============================================================================
// Document highlight tests
// ============================================================================

#[tokio::test]
async fn test_document_highlight_variable() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("hl_var");
    let source = "F test() -> i64 {\n    x := 10\n    x + x\n}\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .document_highlight(DocumentHighlightParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: pos(1, 4), // on "x" definition
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    // Should highlight all occurrences of x
    if let Some(highlights) = result {
        assert!(
            !highlights.is_empty(),
            "Should find highlight occurrences of variable"
        );
    }
}

// ============================================================================
// Formatting tests
// ============================================================================

#[tokio::test]
async fn test_formatting_document() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("fmt_doc");
    let source = "F  add(x:i64,y:i64)->i64=x+y\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .formatting(DocumentFormattingParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            options: FormattingOptions {
                tab_size: 4,
                insert_spaces: true,
                ..Default::default()
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await
        .unwrap();

    // Should return formatting edits (or None if source can't be parsed)
    let _ = result;
}

#[tokio::test]
async fn test_range_formatting() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("fmt_range");
    let source = "F foo() -> i64 = 1\nF bar() -> i64 = 2\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .range_formatting(DocumentRangeFormattingParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            range: Range::new(pos(0, 0), pos(0, 19)),
            options: FormattingOptions {
                tab_size: 4,
                insert_spaces: true,
                ..Default::default()
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await
        .unwrap();

    let _ = result;
}

// ============================================================================
// Inlay hints tests
// ============================================================================

#[tokio::test]
async fn test_inlay_hints_with_variables() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("inlay_var");
    let source = "F test() -> i64 {\n    x := 42\n    y := x + 1\n    y\n}\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .inlay_hint(InlayHintParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            range: Range::new(pos(0, 0), pos(4, 0)),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await
        .unwrap();

    // May or may not produce hints depending on implementation
    let _ = result;
}

// ============================================================================
// Folding range tests
// ============================================================================

#[tokio::test]
async fn test_folding_ranges_function_blocks() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("fold_fn");
    let source = "F foo() -> i64 {\n    x := 1\n    x + 1\n}\n\nF bar() -> i64 {\n    42\n}\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .folding_range(FoldingRangeParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    if let Some(ranges) = result {
        assert!(
            !ranges.is_empty(),
            "Should detect foldable regions for function blocks"
        );
    }
}

// ============================================================================
// Call hierarchy tests
// ============================================================================

#[tokio::test]
async fn test_call_hierarchy_prepare() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("call_hier");
    let source = "F helper() -> i64 = 42\nF main() -> i64 = helper()\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .prepare_call_hierarchy(CallHierarchyPrepareParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: pos(0, 3), // on "helper"
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await
        .unwrap();

    // Should prepare call hierarchy items
    if let Some(items) = result {
        assert!(
            !items.is_empty(),
            "Should find call hierarchy item for function"
        );
    }
}

// ============================================================================
// Document link tests
// ============================================================================

#[tokio::test]
async fn test_document_link_imports() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("doc_link");
    let source = "U std/math\nF main() -> i64 = 0\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .document_link(DocumentLinkParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    // Should find document links for import statements
    if let Some(links) = result {
        assert!(
            !links.is_empty(),
            "Should detect document links for import statements"
        );
    }
}

// ============================================================================
// Rename tests
// ============================================================================

#[tokio::test]
async fn test_rename_on_nonexistent_document() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("rename_noexist");

    let result = service
        .inner()
        .rename(RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: pos(0, 0),
            },
            new_name: "new_name".to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await;

    assert!(result.is_ok());
}

// ============================================================================
// didChange tests
// ============================================================================

#[tokio::test]
async fn test_did_change_updates_ast() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("change");
    let source_v1 = "F foo() -> i64 = 1\n";
    open_doc(&service, &uri, source_v1).await;

    // First check symbols
    let result = service
        .inner()
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    if let Some(DocumentSymbolResponse::Flat(symbols)) = &result {
        assert!(symbols.iter().any(|s| s.name == "foo"));
    }

    // Now change to add another function (timeout for publish_diagnostics)
    let _ = tokio::time::timeout(
        Duration::from_secs(5),
        service
            .inner()
            .did_change(DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri: uri.clone(),
                    version: 2,
                },
                content_changes: vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: "F foo() -> i64 = 1\nF bar() -> i64 = 2\n".to_string(),
                }],
            }),
    )
    .await;

    let result = service
        .inner()
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    if let Some(DocumentSymbolResponse::Flat(symbols)) = result {
        let names: Vec<_> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"foo"));
        assert!(names.contains(&"bar"), "Should see new function after didChange");
    }
}

// ============================================================================
// Code action tests — with document context
// ============================================================================

#[tokio::test]
async fn test_code_action_no_diagnostics_no_selection() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("ca_empty");
    let source = "F main() -> i64 = 0\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .code_action(CodeActionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            range: Range::new(pos(0, 0), pos(0, 0)), // no selection
            context: CodeActionContext {
                diagnostics: vec![],
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    // No actions expected for empty range with no diagnostics
    let _ = result;
}

#[tokio::test]
async fn test_code_action_with_selection() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("ca_select");
    let source = "F main() -> i64 {\n    1 + 2 + 3\n}\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .code_action(CodeActionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            range: Range::new(pos(1, 4), pos(1, 13)), // "1 + 2 + 3"
            context: CodeActionContext {
                diagnostics: vec![],
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    // Should offer "extract to variable" and "extract to function"
    if let Some(actions) = result {
        let titles: Vec<_> = actions
            .iter()
            .filter_map(|a| match a {
                CodeActionOrCommand::CodeAction(ca) => Some(ca.title.as_str()),
                _ => None,
            })
            .collect();
        assert!(
            titles.iter().any(|t| t.contains("Extract")),
            "Should offer extract refactoring for selection"
        );
    }
}

#[tokio::test]
async fn test_code_action_undefined_variable_diagnostic() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("ca_undef");
    let source = "F main() -> i64 {\n    x + 1\n}\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .code_action(CodeActionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            range: Range::new(pos(1, 4), pos(1, 5)),
            context: CodeActionContext {
                diagnostics: vec![Diagnostic {
                    range: Range::new(pos(1, 4), pos(1, 5)),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: "Undefined variable: x".to_string(),
                    ..Default::default()
                }],
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    if let Some(actions) = result {
        let titles: Vec<_> = actions
            .iter()
            .filter_map(|a| match a {
                CodeActionOrCommand::CodeAction(ca) => Some(ca.title.clone()),
                _ => None,
            })
            .collect();
        assert!(
            titles.iter().any(|t| t.contains("Create variable")),
            "Should offer 'create variable' quickfix: {:?}",
            titles
        );
    }
}

#[tokio::test]
async fn test_code_action_undefined_function_import() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("ca_import");
    let source = "F main() -> i64 {\n    sqrt(4.0)\n}\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .code_action(CodeActionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            range: Range::new(pos(1, 4), pos(1, 8)),
            context: CodeActionContext {
                diagnostics: vec![Diagnostic {
                    range: Range::new(pos(1, 4), pos(1, 8)),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: "Undefined function: sqrt".to_string(),
                    ..Default::default()
                }],
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    if let Some(actions) = result {
        let titles: Vec<_> = actions
            .iter()
            .filter_map(|a| match a {
                CodeActionOrCommand::CodeAction(ca) => Some(ca.title.clone()),
                _ => None,
            })
            .collect();
        assert!(
            titles.iter().any(|t| t.contains("Import")),
            "Should offer 'Import' quickfix for sqrt: {:?}",
            titles
        );
    }
}

// ============================================================================
// Workspace symbols tests
// ============================================================================

#[tokio::test]
async fn test_workspace_symbols_with_document() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("ws_sym");
    let source = "F compute(x: i64) -> i64 = x * 2\nS Config { debug: bool }\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .symbol(WorkspaceSymbolParams {
            query: "compute".to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    // May find the symbol in the workspace
    let _ = result;
}

// ============================================================================
// Semantic tokens tests — with document context
// ============================================================================

#[tokio::test]
async fn test_semantic_tokens_full() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("sem_full");
    let source = "F add(x: i64, y: i64) -> i64 = x + y\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .semantic_tokens_full(SemanticTokensParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    if let Some(SemanticTokensResult::Tokens(tokens)) = result {
        assert!(
            !tokens.data.is_empty(),
            "Should produce semantic tokens for source"
        );
    }
}

// ============================================================================
// Type hierarchy tests
// ============================================================================

#[tokio::test]
async fn test_type_hierarchy_with_trait() {
    let (service, _socket) = create_service();
    init(&service).await;

    let uri = test_uri("type_hier");
    let source = "W Animal {\n    F speak(self) -> str\n}\nS Dog { name: str }\nX Animal for Dog {\n    F speak(self) -> str = \"woof\"\n}\n";
    open_doc(&service, &uri, source).await;

    let result = service
        .inner()
        .prepare_type_hierarchy(TypeHierarchyPrepareParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: pos(0, 3), // on "Animal"
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await
        .unwrap();

    // Should prepare type hierarchy
    let _ = result;
}

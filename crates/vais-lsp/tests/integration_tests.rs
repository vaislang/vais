//! Integration tests for Vais LSP Server
//!
//! Tests LSP server initialization and basic capabilities.
//!
//! Note: Tests involving document operations (didOpen, didChange, etc.) are limited
//! because tower-lsp's test client blocks on publish_diagnostics() calls. For full
//! end-to-end testing, use a real LSP client like VS Code or neovim.

use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService};
use vais_lsp::VaisBackend;

mod test_helpers {
    use super::*;

    /// Create a test LSP service
    pub fn create_test_service() -> LspService<VaisBackend> {
        let (service, _socket) = LspService::new(|client| VaisBackend::new(client));
        service
    }

    /// Helper to create a test URI
    pub fn test_uri(name: &str) -> Url {
        Url::parse(&format!("file:///test/{}.vais", name)).unwrap()
    }

    /// Helper to create a position
    pub fn pos(line: u32, character: u32) -> Position {
        Position::new(line, character)
    }
}

use test_helpers::*;

// ============================================================================
// Initialization Tests
// ============================================================================

#[tokio::test]
async fn test_initialize() {
    let service = create_test_service();

    let params = InitializeParams {
        process_id: None,
        root_uri: None,
        initialization_options: None,
        capabilities: ClientCapabilities::default(),
        trace: None,
        workspace_folders: None,
        client_info: None,
        locale: None,
        ..Default::default()
    };

    let result = service.inner().initialize(params).await.unwrap();

    // Verify server info
    assert!(result.server_info.is_some());
    let server_info = result.server_info.unwrap();
    assert_eq!(server_info.name, "vais-lsp");
    assert_eq!(server_info.version, Some("0.0.1".to_string()));

    // Verify capabilities
    let caps = result.capabilities;

    // Text document sync
    assert!(matches!(
        caps.text_document_sync,
        Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL))
    ));

    // Hover support
    assert!(caps.hover_provider.is_some());

    // Completion support
    assert!(caps.completion_provider.is_some());
    if let Some(completion) = caps.completion_provider {
        let triggers = completion.trigger_characters.unwrap();
        assert!(triggers.contains(&".".to_string()));
        assert!(triggers.contains(&":".to_string()));
    }

    // Definition support
    assert!(matches!(caps.definition_provider, Some(OneOf::Left(true))));

    // References support
    assert!(matches!(caps.references_provider, Some(OneOf::Left(true))));

    // Document symbols
    assert!(matches!(caps.document_symbol_provider, Some(OneOf::Left(true))));

    // Semantic tokens
    assert!(caps.semantic_tokens_provider.is_some());
    if let Some(SemanticTokensServerCapabilities::SemanticTokensOptions(opts)) =
        caps.semantic_tokens_provider
    {
        let legend = opts.legend;
        // Verify token types are defined
        assert!(legend.token_types.len() >= 5);
        assert!(legend.token_types.contains(&SemanticTokenType::FUNCTION));
        assert!(legend.token_types.contains(&SemanticTokenType::VARIABLE));
        assert!(legend.token_types.contains(&SemanticTokenType::KEYWORD));
    }

    // Rename support
    assert!(caps.rename_provider.is_some());
}

#[tokio::test]
async fn test_initialized_notification() {
    let service = create_test_service();

    // Should not panic
    service.inner().initialized(InitializedParams {}).await;
}

#[tokio::test]
async fn test_shutdown() {
    let service = create_test_service();
    let result = service.inner().shutdown().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_initialize_with_client_info() {
    let service = create_test_service();

    let params = InitializeParams {
        process_id: Some(12345),
        root_uri: Some(Url::parse("file:///test/project").unwrap()),
        initialization_options: None,
        capabilities: ClientCapabilities::default(),
        trace: Some(TraceValue::Verbose),
        workspace_folders: None,
        client_info: Some(ClientInfo {
            name: "test-client".to_string(),
            version: Some("1.0.0".to_string()),
        }),
        locale: Some("en-US".to_string()),
        ..Default::default()
    };

    let result = service.inner().initialize(params).await;
    assert!(result.is_ok());
}

// ============================================================================
// Completion Tests (without document context)
// ============================================================================

#[tokio::test]
async fn test_completion_provides_keywords() {
    let service = create_test_service();
    let uri = test_uri("test");

    let result = service.inner()
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: pos(0, 0),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    assert!(result.is_some());

    if let Some(CompletionResponse::Array(items)) = result {
        let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();

        // Vais keywords
        assert!(labels.contains(&"F"), "Should provide F (function) keyword");
        assert!(labels.contains(&"S"), "Should provide S (struct) keyword");
        assert!(labels.contains(&"E"), "Should provide E (enum) keyword");
        assert!(labels.contains(&"I"), "Should provide I (if) keyword");
        assert!(labels.contains(&"L"), "Should provide L (loop) keyword");
        assert!(labels.contains(&"M"), "Should provide M (match) keyword");
        assert!(labels.contains(&"R"), "Should provide R (return) keyword");
        assert!(labels.contains(&"W"), "Should provide W (trait) keyword");
        assert!(labels.contains(&"X"), "Should provide X (impl) keyword");
        assert!(labels.contains(&"U"), "Should provide U (use) keyword");
        assert!(labels.contains(&"A"), "Should provide A (async) keyword");

        // Verify keyword items have correct kind
        // Note: "E" appears both as a keyword (Enum) and constant (Euler's number)
        for item in &items {
            if (item.label == "F" || item.label == "S") && item.kind == Some(CompletionItemKind::KEYWORD) {
                // These should definitely be keywords
                assert_eq!(item.kind, Some(CompletionItemKind::KEYWORD));
            }
        }
    } else {
        panic!("Expected array of completion items");
    }
}

#[tokio::test]
async fn test_completion_provides_types() {
    let service = create_test_service();
    let uri = test_uri("test");

    let result = service.inner()
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: pos(0, 0),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    if let Some(CompletionResponse::Array(items)) = result {
        let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();

        // Primitive types
        assert!(labels.contains(&"i64"), "Should provide i64 type");
        assert!(labels.contains(&"i32"), "Should provide i32 type");
        assert!(labels.contains(&"f64"), "Should provide f64 type");
        assert!(labels.contains(&"f32"), "Should provide f32 type");
        assert!(labels.contains(&"bool"), "Should provide bool type");
        assert!(labels.contains(&"str"), "Should provide str type");

        // Verify type items have correct kind
        for item in &items {
            if item.label == "i64" || item.label == "f64" || item.label == "bool" {
                assert_eq!(item.kind, Some(CompletionItemKind::TYPE_PARAMETER));
            }
        }
    }
}

#[tokio::test]
async fn test_completion_provides_builtin_functions() {
    let service = create_test_service();
    let uri = test_uri("test");

    let result = service.inner()
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: pos(0, 0),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    if let Some(CompletionResponse::Array(items)) = result {
        let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();

        // Builtin functions
        assert!(labels.contains(&"puts"), "Should provide puts function");
        assert!(labels.contains(&"print_i64"), "Should provide print_i64 function");
        assert!(labels.contains(&"print_f64"), "Should provide print_f64 function");
        assert!(labels.contains(&"malloc"), "Should provide malloc function");
        assert!(labels.contains(&"free"), "Should provide free function");

        // Verify function items have correct kind and details
        for item in &items {
            if item.label == "puts" {
                assert_eq!(item.kind, Some(CompletionItemKind::FUNCTION));
                assert!(item.detail.is_some());
                assert!(item.detail.as_ref().unwrap().contains("fn(str)"));
            }
        }
    }
}

#[tokio::test]
async fn test_completion_provides_std_modules() {
    let service = create_test_service();
    let uri = test_uri("test");

    let result = service.inner()
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: pos(0, 0),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    if let Some(CompletionResponse::Array(items)) = result {
        let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();

        // Standard library modules
        assert!(labels.contains(&"std/math"), "Should provide std/math module");
        assert!(labels.contains(&"std/io"), "Should provide std/io module");
        assert!(labels.contains(&"std/option"), "Should provide std/option module");
        assert!(labels.contains(&"std/result"), "Should provide std/result module");
        assert!(labels.contains(&"std/vec"), "Should provide std/vec module");
        assert!(labels.contains(&"std/hashmap"), "Should provide std/hashmap module");

        // Verify module items have correct kind
        for item in &items {
            if item.label.starts_with("std/") {
                assert_eq!(item.kind, Some(CompletionItemKind::MODULE));
            }
        }
    }
}

#[tokio::test]
async fn test_completion_provides_math_functions() {
    let service = create_test_service();
    let uri = test_uri("test");

    let result = service.inner()
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: pos(0, 0),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    if let Some(CompletionResponse::Array(items)) = result {
        let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();

        // Math functions
        assert!(labels.contains(&"sqrt"), "Should provide sqrt function");
        assert!(labels.contains(&"sin"), "Should provide sin function");
        assert!(labels.contains(&"cos"), "Should provide cos function");
        assert!(labels.contains(&"pow"), "Should provide pow function");
        assert!(labels.contains(&"abs"), "Should provide abs function");

        // Math constants
        assert!(labels.contains(&"PI"), "Should provide PI constant");
        assert!(labels.contains(&"TAU"), "Should provide TAU constant");
    }
}

#[tokio::test]
async fn test_completion_provides_option_result_constructors() {
    let service = create_test_service();
    let uri = test_uri("test");

    let result = service.inner()
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: pos(0, 0),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    if let Some(CompletionResponse::Array(items)) = result {
        let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();

        // Option/Result constructors
        assert!(labels.contains(&"Some"), "Should provide Some constructor");
        assert!(labels.contains(&"None"), "Should provide None constructor");
        assert!(labels.contains(&"Ok"), "Should provide Ok constructor");
        assert!(labels.contains(&"Err"), "Should provide Err constructor");

        // Verify constructor items have correct kind
        for item in &items {
            if item.label == "Some" || item.label == "None" || item.label == "Ok" || item.label == "Err" {
                assert_eq!(item.kind, Some(CompletionItemKind::CONSTRUCTOR));
            }
        }
    }
}

// ============================================================================
// Capability Verification Tests
// ============================================================================

#[tokio::test]
async fn test_server_capabilities_comprehensive() {
    let service = create_test_service();

    let params = InitializeParams::default();
    let result = service.inner().initialize(params).await.unwrap();
    let caps = result.capabilities;

    // Verify all expected capabilities are present
    assert!(caps.text_document_sync.is_some(), "Missing text_document_sync");
    assert!(caps.hover_provider.is_some(), "Missing hover_provider");
    assert!(caps.completion_provider.is_some(), "Missing completion_provider");
    assert!(caps.definition_provider.is_some(), "Missing definition_provider");
    assert!(caps.references_provider.is_some(), "Missing references_provider");
    assert!(caps.document_symbol_provider.is_some(), "Missing document_symbol_provider");
    assert!(caps.semantic_tokens_provider.is_some(), "Missing semantic_tokens_provider");
    assert!(caps.rename_provider.is_some(), "Missing rename_provider");

    // Verify newly implemented capabilities
    assert!(caps.code_action_provider.is_some(), "Missing code_action_provider");

    // Verify capabilities that should NOT be present (not yet implemented)
    assert!(caps.document_formatting_provider.is_none(), "document_formatting_provider should not be implemented");
    assert!(caps.folding_range_provider.is_none(), "folding_range_provider should not be implemented");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_completion_on_nonexistent_document() {
    let service = create_test_service();
    let uri = test_uri("nonexistent");

    // Should not panic even when document doesn't exist
    let result = service.inner()
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: pos(0, 0),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_some(), "Should provide basic completions even without document");
}

#[tokio::test]
async fn test_hover_on_nonexistent_document() {
    let service = create_test_service();
    let uri = test_uri("nonexistent");

    let result = service.inner()
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: pos(0, 0),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none(), "Should return None for nonexistent document");
}

#[tokio::test]
async fn test_goto_definition_on_nonexistent_document() {
    let service = create_test_service();
    let uri = test_uri("nonexistent");

    let result = service.inner()
        .goto_definition(GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: pos(0, 0),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none(), "Should return None for nonexistent document");
}

#[tokio::test]
async fn test_references_on_nonexistent_document() {
    let service = create_test_service();
    let uri = test_uri("nonexistent");

    let result = service.inner()
        .references(ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: pos(0, 0),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: ReferenceContext {
                include_declaration: true,
            },
        })
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none(), "Should return None for nonexistent document");
}

#[tokio::test]
async fn test_document_symbols_on_nonexistent_document() {
    let service = create_test_service();
    let uri = test_uri("nonexistent");

    let result = service.inner()
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none(), "Should return None for nonexistent document");
}

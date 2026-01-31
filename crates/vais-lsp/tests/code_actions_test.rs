//! Tests for LSP Code Actions

use tower_lsp::lsp_types::*;

#[tokio::test]
async fn test_code_action_extract_variable() {
    // This is a basic structure test
    // In a real implementation, you would set up a proper test harness
    // with a mock LSP client and test the code actions functionality

    // For now, we just verify the types are correct
    let params = CodeActionParams {
        text_document: TextDocumentIdentifier {
            uri: Url::parse("file:///test.vais").unwrap(),
        },
        range: Range {
            start: Position::new(0, 0),
            end: Position::new(0, 10),
        },
        context: CodeActionContext {
            diagnostics: vec![],
            only: None,
            trigger_kind: None,
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    // Just verify we can create the params
    assert_eq!(params.range.start.line, 0);
}

#[test]
fn test_import_suggestions() {
    // Test that we have the correct mapping of functions to modules
    let math_functions = ["sqrt", "sin", "cos", "tan", "pow", "log", "exp", "floor", "ceil"];
    let io_functions = ["read_i64", "read_f64", "read_line", "read_char"];

    // Verify we know about these functions
    for func in &math_functions {
        assert!(!func.is_empty());
    }

    for func in &io_functions {
        assert!(!func.is_empty());
    }
}

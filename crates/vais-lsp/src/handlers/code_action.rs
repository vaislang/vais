//! Code action handler implementation

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use vais_ast::{FunctionBody, Item, Stmt};

use crate::backend::{position_in_range, VaisBackend};

pub(crate) async fn handle_code_action(
    backend: &VaisBackend,
    params: CodeActionParams,
) -> Result<Option<CodeActionResponse>> {
    let uri = &params.text_document.uri;
    let range = params.range;
    let mut actions = Vec::new();

    if let Some(doc) = backend.documents.get(uri) {
        // Get diagnostics from the context
        let diagnostics = &params.context.diagnostics;
        if !diagnostics.is_empty() {
            for diagnostic in diagnostics {
                handle_diagnostic_quickfixes(backend, uri, &range, diagnostic, &doc, &mut actions);
            }
        }

        // Refactor: Extract to variable (if there's a selection)
        if range.start != range.end {
            add_extract_to_variable_action(backend, uri, &range, &doc, &mut actions);
        }

        // Refactor: Extract to function (for multi-line or complex selections)
        if range.start.line != range.end.line || (range.end.character - range.start.character) > 30
        {
            add_extract_to_function_action(backend, uri, &range, &doc, &mut actions);
        }

        // Refactor: Inline Variable
        if let Some(ast) = &doc.ast {
            add_inline_variable_action(backend, uri, &range, &doc, ast, &mut actions);
        }

        // Refactor: Convert to/from Expression Body
        if let Some(ast) = &doc.ast {
            add_convert_body_actions(backend, uri, &range, &doc, ast, &mut actions);
        }

        // Refactor: Introduce Named Parameter
        if let Some(ast) = &doc.ast {
            add_named_parameter_actions(backend, uri, &range, &doc, ast, &mut actions);
        }
    }

    if actions.is_empty() {
        Ok(None)
    } else {
        Ok(Some(actions))
    }
}

fn handle_diagnostic_quickfixes(
    _backend: &VaisBackend,
    uri: &Url,
    range: &Range,
    diagnostic: &Diagnostic,
    doc: &dashmap::mapref::one::Ref<'_, Url, crate::backend::Document>,
    actions: &mut Vec<CodeActionOrCommand>,
) {
    // Quick fix for undefined variables
    if diagnostic.message.starts_with("Undefined variable:") {
        let var_name = diagnostic
            .message
            .strip_prefix("Undefined variable: ")
            .unwrap_or("");

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
                let has_import = if let Some(ast) = &doc.ast {
                    ast.items.iter().any(|item| {
                        if let vais_ast::Item::Use(use_item) = &item.node {
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
                                    range: Range::new(Position::new(0, 0), Position::new(0, 0)),
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

    // Quick fix for type mismatches
    if diagnostic.message.starts_with("Type mismatch:")
        && (diagnostic.message.contains("expected i64, found f64")
            || diagnostic.message.contains("expected f64, found i64"))
    {
        let cast_type = if diagnostic.message.contains("expected i64") {
            "i64"
        } else {
            "f64"
        };

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

    // Quick fix for unused variable
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

            if let Some(paren_pos) = line_str.find(')') {
                let insert_pos = paren_pos + 1;
                let position = Position::new(diagnostic.range.start.line, insert_pos as u32);

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

    // Quick fix for missing semicolon
    if diagnostic.message.contains("expected") && diagnostic.message.contains(";") {
        let line = diagnostic.range.end.line as usize;
        if let Some(line_rope) = doc.content.get_line(line) {
            let line_str: String = line_rope.chars().collect();
            let line_end = line_str.trim_end().len();

            let position = Position::new(diagnostic.range.end.line, line_end as u32);

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

fn add_extract_to_variable_action(
    _backend: &VaisBackend,
    uri: &Url,
    range: &Range,
    doc: &dashmap::mapref::one::Ref<'_, Url, crate::backend::Document>,
    actions: &mut Vec<CodeActionOrCommand>,
) {
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
                if !selected_text.trim().is_empty() && !selected_text.trim().starts_with("L ") {
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
                                        range: *range,
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

fn add_extract_to_function_action(
    _backend: &VaisBackend,
    uri: &Url,
    range: &Range,
    doc: &dashmap::mapref::one::Ref<'_, Url, crate::backend::Document>,
    actions: &mut Vec<CodeActionOrCommand>,
) {
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
                                range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                                new_text: function_def,
                            },
                            // Replace selection with function call
                            TextEdit {
                                range: *range,
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

fn add_inline_variable_action(
    backend: &VaisBackend,
    uri: &Url,
    range: &Range,
    doc: &dashmap::mapref::one::Ref<'_, Url, crate::backend::Document>,
    ast: &vais_ast::Module,
    actions: &mut Vec<CodeActionOrCommand>,
) {
    // Convert range.start to offset
    let cursor_line = range.start.line as usize;
    let cursor_char = range.start.character as usize;
    let cursor_offset = if let Ok(line_start_char) = doc.content.try_line_to_char(cursor_line) {
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
                        if cursor_offset >= stmt.span.start && cursor_offset <= stmt.span.end {
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
                                backend.find_var_references_in_stmt(
                                    ref_stmt,
                                    var_name,
                                    &mut reference_ranges,
                                    &doc.content,
                                );
                            }

                            if !reference_ranges.is_empty() {
                                let mut edits = Vec::new();

                                // Remove the let statement line
                                let let_range = backend.span_to_range(&doc.content, &stmt.span);
                                // Extend to include the whole line
                                let let_line_start = Position::new(let_range.start.line, 0);
                                let let_line_end = Position::new(let_range.end.line + 1, 0);
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

                                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                    title: format!("Inline variable '{}'", var_name),
                                    kind: Some(CodeActionKind::REFACTOR_INLINE),
                                    diagnostics: None,
                                    edit: Some(edit),
                                    ..Default::default()
                                }));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn add_convert_body_actions(
    backend: &VaisBackend,
    uri: &Url,
    range: &Range,
    doc: &dashmap::mapref::one::Ref<'_, Url, crate::backend::Document>,
    ast: &vais_ast::Module,
    actions: &mut Vec<CodeActionOrCommand>,
) {
    let _cursor_line = range.start.line as usize;
    let _cursor_char = range.start.character as usize;

    for item in &ast.items {
        if let Item::Function(func) = &item.node {
            let func_range = backend.span_to_range(&doc.content, &item.span);

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
                            let body_start = if let FunctionBody::Block(stmts) = &func.body {
                                if let Some(first_stmt) = stmts.first() {
                                    // Work backwards from first statement to find '{'
                                    let mut brace_offset = first_stmt.span.start;
                                    while brace_offset > 0 {
                                        brace_offset -= 1;
                                        if let Some(ch) = doc.content.get_char(brace_offset) {
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
                                                start: backend
                                                    .offset_to_position(&doc.content, body_start),
                                                end: backend
                                                    .offset_to_position(&doc.content, body_end),
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
                                            start: backend
                                                .offset_to_position(&doc.content, eq_offset),
                                            end: backend
                                                .offset_to_position(&doc.content, expr.span.end),
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

fn add_named_parameter_actions(
    backend: &VaisBackend,
    uri: &Url,
    range: &Range,
    doc: &dashmap::mapref::one::Ref<'_, Url, crate::backend::Document>,
    ast: &vais_ast::Module,
    actions: &mut Vec<CodeActionOrCommand>,
) {
    let cursor_line = range.start.line as usize;
    let cursor_char = range.start.character as usize;
    let cursor_offset = if let Ok(line_start_char) = doc.content.try_line_to_char(cursor_line) {
        line_start_char + cursor_char
    } else {
        0
    };

    // Find function call at cursor and offer to convert to named arguments
    for item in &ast.items {
        if let Item::Function(func) = &item.node {
            if let FunctionBody::Block(stmts) = &func.body {
                for stmt in stmts {
                    backend.find_call_at_cursor_in_stmt(
                        stmt,
                        cursor_offset,
                        &doc.content,
                        ast,
                        uri,
                        actions,
                    );
                }
            } else if let FunctionBody::Expr(expr) = &func.body {
                backend.find_call_at_cursor_in_expr(
                    expr,
                    cursor_offset,
                    &doc.content,
                    ast,
                    uri,
                    actions,
                );
            }
        }
    }
}

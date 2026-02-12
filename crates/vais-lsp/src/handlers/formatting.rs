//! Range formatting handler for Vais LSP

use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use vais_codegen::formatter::{FormatConfig, Formatter};
use vais_parser::parse;

use crate::backend::VaisBackend;

/// Handle document range formatting request
///
/// Strategy: Format the entire document, then extract only the lines
/// that fall within the requested range.
pub(crate) async fn handle_range_formatting(
    backend: &VaisBackend,
    params: DocumentRangeFormattingParams,
) -> Result<Option<Vec<TextEdit>>> {
    let uri = &params.text_document.uri;
    let range = params.range;

    if let Some(doc) = backend.documents.get(uri) {
        let source = doc.content.to_string();

        // Parse the source to get an AST
        if let Ok(module) = parse(&source) {
            // Create format config from options
            let config = FormatConfig {
                indent_size: params.options.tab_size as usize,
                use_tabs: !params.options.insert_spaces,
                ..FormatConfig::default()
            };
            let mut formatter = Formatter::new(config);
            let formatted = formatter.format_module(&module);

            // Convert the formatted text to Rope for line operations
            let formatted_rope = Rope::from_str(&formatted);

            // Calculate the range in terms of character offsets
            let start_line = range.start.line as usize;
            let end_line = range.end.line as usize;

            // Ensure we don't go out of bounds
            let actual_start_line = start_line.min(doc.content.len_lines().saturating_sub(1));
            let actual_end_line = end_line.min(doc.content.len_lines().saturating_sub(1));

            // Extract the formatted lines within the range
            let formatted_start_line =
                actual_start_line.min(formatted_rope.len_lines().saturating_sub(1));
            let formatted_end_line =
                actual_end_line.min(formatted_rope.len_lines().saturating_sub(1));

            // Get end line content for range calculation
            let end_line_content = doc.content.line(actual_end_line);

            // Extract the formatted text for the range
            let formatted_start_char = formatted_rope.line_to_char(formatted_start_line);
            let formatted_end_line_content = formatted_rope.line(formatted_end_line);
            let formatted_end_char = formatted_rope.line_to_char(formatted_end_line)
                + formatted_end_line_content.len_chars();

            let new_text = if formatted_start_char < formatted_rope.len_chars()
                && formatted_end_char <= formatted_rope.len_chars()
            {
                formatted_rope
                    .slice(formatted_start_char..formatted_end_char)
                    .to_string()
            } else {
                // Fallback to empty if out of bounds
                String::new()
            };

            // Create the edit
            let edit = TextEdit {
                range: Range {
                    start: Position::new(actual_start_line as u32, 0),
                    end: Position::new(actual_end_line as u32, end_line_content.len_chars() as u32),
                },
                new_text,
            };

            return Ok(Some(vec![edit]));
        }
    }

    Ok(None)
}

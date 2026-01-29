//! AI-based code completion for the Vais LSP.
//!
//! Provides context-aware completion suggestions using pattern analysis
//! and heuristic-based code generation. Designed to work alongside the
//! existing static completion system.
//!
//! # Architecture
//!
//! The AI completion engine analyzes:
//! 1. **Surrounding code context** - lines before/after cursor
//! 2. **AST context** - current function, scope, available types
//! 3. **Pattern recognition** - common coding patterns in Vais
//!
//! Results are returned as standard LSP `CompletionItem`s with the
//! `AI_COMPLETION` sort prefix to distinguish them from static completions.

use tower_lsp::lsp_types::*;
use vais_ast::{Item, Module};

/// Sort text prefix to group AI completions after static ones.
const AI_SORT_PREFIX: &str = "zz_ai_";

/// Maximum number of context lines to analyze.
const MAX_CONTEXT_LINES: usize = 20;

/// Context extracted from the cursor position for AI analysis.
#[derive(Debug)]
pub struct CompletionContext {
    /// Lines before the cursor (up to MAX_CONTEXT_LINES).
    pub prefix_lines: Vec<String>,
    /// The current line up to the cursor.
    pub current_line_prefix: String,
    /// Lines after the cursor (up to MAX_CONTEXT_LINES).
    pub suffix_lines: Vec<String>,
    /// Current function name, if inside one.
    pub current_function: Option<String>,
    /// Current function return type, if known.
    pub current_return_type: Option<String>,
    /// Available local variables in scope (name, type hint).
    pub locals_in_scope: Vec<(String, Option<String>)>,
    /// Available functions in the module.
    pub available_functions: Vec<String>,
    /// Available struct names.
    pub available_structs: Vec<String>,
}

impl CompletionContext {
    /// Extract completion context from a document and position.
    pub fn from_document(
        content: &str,
        position: Position,
        ast: Option<&Module>,
    ) -> Self {
        let lines: Vec<&str> = content.lines().collect();
        let line_idx = position.line as usize;
        let col = position.character as usize;

        let prefix_lines = lines[..line_idx]
            .iter()
            .rev()
            .take(MAX_CONTEXT_LINES)
            .rev()
            .map(|s| s.to_string())
            .collect();

        let current_line_prefix = lines
            .get(line_idx)
            .map(|l| {
                let chars: Vec<char> = l.chars().collect();
                chars[..col.min(chars.len())].iter().collect()
            })
            .unwrap_or_default();

        let suffix_lines = lines
            .get(line_idx + 1..)
            .unwrap_or(&[])
            .iter()
            .take(MAX_CONTEXT_LINES)
            .map(|s| s.to_string())
            .collect();

        let mut ctx = CompletionContext {
            prefix_lines,
            current_line_prefix,
            suffix_lines,
            current_function: None,
            current_return_type: None,
            locals_in_scope: vec![],
            available_functions: vec![],
            available_structs: vec![],
        };

        if let Some(module) = ast {
            ctx.extract_from_ast(module, line_idx);
        }

        ctx
    }

    fn extract_from_ast(&mut self, module: &Module, _cursor_line: usize) {
        for item in &module.items {
            match &item.node {
                Item::Function(f) => {
                    self.available_functions.push(f.name.node.clone());
                }
                Item::Struct(s) => {
                    self.available_structs.push(s.name.node.clone());
                }
                _ => {}
            }
        }
    }
}

/// Generate AI-based completion suggestions.
pub fn generate_ai_completions(ctx: &CompletionContext) -> Vec<CompletionItem> {
    let mut items = vec![];

    // Pattern: incomplete function body
    items.extend(suggest_function_body(ctx));

    // Pattern: match arm completion
    items.extend(suggest_match_arms(ctx));

    // Pattern: struct field initialization
    items.extend(suggest_struct_fields(ctx));

    // Pattern: common idioms
    items.extend(suggest_idioms(ctx));

    // Pattern: error handling
    items.extend(suggest_error_handling(ctx));

    // Pattern: loop patterns
    items.extend(suggest_loop_patterns(ctx));

    items
}

/// Suggest function body based on return type and name.
fn suggest_function_body(ctx: &CompletionContext) -> Vec<CompletionItem> {
    let mut items = vec![];
    let line = ctx.current_line_prefix.trim();

    // Detect empty function body
    if line.is_empty() || line == "{" {
        // Check if we're inside a function
        for pline in ctx.prefix_lines.iter().rev() {
            let trimmed = pline.trim();
            if trimmed.starts_with("F ") && trimmed.contains("->") {
                // Extract return type
                if let Some(ret) = trimmed.split("->").last() {
                    let ret = ret.trim().trim_end_matches('{').trim();
                    match ret {
                        "i64" | "i32" => {
                            items.push(ai_completion(
                                "return 0",
                                "R 0",
                                "Return zero (default integer)",
                                "a_body",
                            ));
                        }
                        "bool" => {
                            items.push(ai_completion(
                                "return true",
                                "R true",
                                "Return true (default boolean)",
                                "a_body",
                            ));
                            items.push(ai_completion(
                                "return false",
                                "R false",
                                "Return false",
                                "a_body2",
                            ));
                        }
                        "str" => {
                            items.push(ai_completion(
                                "return empty string",
                                "R \"\"",
                                "Return empty string",
                                "a_body",
                            ));
                        }
                        _ if ret.starts_with("Option") => {
                            items.push(ai_completion(
                                "return None",
                                "None",
                                "Return None (no value)",
                                "a_body",
                            ));
                        }
                        _ if ret.starts_with("Result") => {
                            items.push(ai_completion(
                                "return Ok",
                                "Ok(${1:value})",
                                "Return Ok result",
                                "a_body",
                            ));
                        }
                        _ => {}
                    }
                }
                break;
            }
        }
    }

    items
}

/// Suggest match arms for enum/option/result patterns.
fn suggest_match_arms(ctx: &CompletionContext) -> Vec<CompletionItem> {
    let mut items = vec![];
    let line = ctx.current_line_prefix.trim();

    // Detect "M expr {" pattern
    if line.starts_with("M ") && line.ends_with('{') {
        // Option match
        if line.contains("option") || line.contains("opt") {
            items.push(ai_snippet(
                "Option match arms",
                "Some(${1:val}) => ${2:expr},\n        None => ${3:expr}",
                "Complete Option match with Some/None arms",
                "a_match_opt",
            ));
        }
        // Result match
        else if line.contains("result") || line.contains("res") {
            items.push(ai_snippet(
                "Result match arms",
                "Ok(${1:val}) => ${2:expr},\n        Err(${3:e}) => ${4:expr}",
                "Complete Result match with Ok/Err arms",
                "a_match_res",
            ));
        }
        // Bool match
        else {
            items.push(ai_snippet(
                "Boolean match arms",
                "true => ${1:expr},\n        false => ${2:expr}",
                "Complete boolean match",
                "a_match_bool",
            ));
        }
    }

    items
}

/// Suggest struct field initialization.
fn suggest_struct_fields(ctx: &CompletionContext) -> Vec<CompletionItem> {
    let mut items = vec![];
    let line = ctx.current_line_prefix.trim();

    // Detect struct literal pattern "Name {"
    for struct_name in &ctx.available_structs {
        if line.ends_with(&format!("{} {{", struct_name)) || line.ends_with(&format!("{}{{", struct_name)) {
            items.push(ai_completion(
                &format!("{} fields", struct_name),
                "${1:field}: ${2:value}",
                &format!("Initialize {} fields", struct_name),
                "a_struct",
            ));
            break;
        }
    }

    items
}

/// Suggest common Vais idioms.
fn suggest_idioms(ctx: &CompletionContext) -> Vec<CompletionItem> {
    let mut items = vec![];
    let line = ctx.current_line_prefix.trim();

    // Empty line → suggest common patterns
    if line.is_empty() {
        // Check context for appropriate suggestions
        let in_function = ctx.prefix_lines.iter().any(|l| l.trim().starts_with("F "));

        if in_function {
            items.push(ai_snippet(
                "if-else expression",
                "I ${1:condition} {\n    ${2:then_expr}\n} E {\n    ${3:else_expr}\n}",
                "AI: if-else expression",
                "b_if",
            ));

            items.push(ai_snippet(
                "loop with break",
                "L {\n    I ${1:condition} {\n        B\n    }\n    ${2:body}\n}",
                "AI: loop with break condition",
                "b_loop",
            ));

            items.push(ai_snippet(
                "for range loop",
                "L ${1:i} : ${2:0}..${3:10} {\n    ${4:body}\n}",
                "AI: for loop over range",
                "b_for",
            ));
        }
    }

    // "let" pattern → suggest variable binding
    if line.ends_with(":=") || line.ends_with(":= ") {
        items.push(ai_snippet(
            "function call",
            "${1:function_name}(${2:args})",
            "AI: assign from function call",
            "b_assign_call",
        ));
    }

    items
}

/// Suggest error handling patterns.
fn suggest_error_handling(ctx: &CompletionContext) -> Vec<CompletionItem> {
    let mut items = vec![];
    let line = ctx.current_line_prefix.trim();

    // After a Result-returning call
    if line.contains("Err(") || line.contains("error") {
        items.push(ai_snippet(
            "error propagation",
            "M ${1:result} {\n    Ok(val) => val,\n    Err(e) => R Err(e)\n}",
            "AI: propagate error with match",
            "c_err_prop",
        ));
    }

    items
}

/// Suggest loop-related patterns.
fn suggest_loop_patterns(ctx: &CompletionContext) -> Vec<CompletionItem> {
    let mut items = vec![];
    let line = ctx.current_line_prefix.trim();

    if line == "L" || line == "L " {
        items.push(ai_snippet(
            "infinite loop",
            "L {\n    ${1:body}\n}",
            "AI: infinite loop",
            "b_loop_inf",
        ));

        items.push(ai_snippet(
            "counted loop",
            "L ${1:i} : 0..${2:n} {\n    ${3:body}\n}",
            "AI: counted range loop",
            "b_loop_count",
        ));
    }

    items
}

/// Create a simple AI completion item.
fn ai_completion(label: &str, insert_text: &str, detail: &str, sort_key: &str) -> CompletionItem {
    CompletionItem {
        label: format!("⚡ {}", label),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some(format!("AI: {}", detail)),
        insert_text: Some(insert_text.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        sort_text: Some(format!("{}{}", AI_SORT_PREFIX, sort_key)),
        ..Default::default()
    }
}

/// Create an AI snippet completion item.
fn ai_snippet(label: &str, snippet: &str, detail: &str, sort_key: &str) -> CompletionItem {
    CompletionItem {
        label: format!("⚡ {}", label),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some(detail.to_string()),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        sort_text: Some(format!("{}{}", AI_SORT_PREFIX, sort_key)),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context(prefix: &[&str], current: &str) -> CompletionContext {
        CompletionContext {
            prefix_lines: prefix.iter().map(|s| s.to_string()).collect(),
            current_line_prefix: current.to_string(),
            suffix_lines: vec![],
            current_function: None,
            current_return_type: None,
            locals_in_scope: vec![],
            available_functions: vec![],
            available_structs: vec![],
        }
    }

    #[test]
    fn test_function_body_suggestion_i64() {
        let ctx = make_context(
            &["F add(a: i64, b: i64) -> i64 {"],
            "",
        );
        let items = suggest_function_body(&ctx);
        assert!(!items.is_empty());
        assert!(items[0].label.contains("return 0"));
    }

    #[test]
    fn test_function_body_suggestion_bool() {
        let ctx = make_context(
            &["F is_valid(x: i64) -> bool {"],
            "",
        );
        let items = suggest_function_body(&ctx);
        assert!(items.len() >= 2); // true and false
    }

    #[test]
    fn test_match_arms_option() {
        let ctx = make_context(&[], "M option {");
        let items = suggest_match_arms(&ctx);
        assert!(!items.is_empty());
        assert!(items[0].label.contains("Option"));
    }

    #[test]
    fn test_match_arms_result() {
        let ctx = make_context(&[], "M result {");
        let items = suggest_match_arms(&ctx);
        assert!(!items.is_empty());
        assert!(items[0].label.contains("Result"));
    }

    #[test]
    fn test_idiom_suggestions_in_function() {
        let ctx = make_context(&["F main() -> i64 {"], "");
        let items = suggest_idioms(&ctx);
        assert!(items.len() >= 2); // if-else, loop, for
    }

    #[test]
    fn test_loop_pattern_suggestions() {
        let ctx = make_context(&[], "L ");
        let items = suggest_loop_patterns(&ctx);
        assert_eq!(items.len(), 2); // infinite loop and counted loop
    }

    #[test]
    fn test_struct_field_suggestion() {
        let mut ctx = make_context(&[], "Point {");
        ctx.available_structs = vec!["Point".to_string()];
        let items = suggest_struct_fields(&ctx);
        assert!(!items.is_empty());
    }

    #[test]
    fn test_generate_ai_completions() {
        let ctx = make_context(&["F test() -> i64 {"], "");
        let items = generate_ai_completions(&ctx);
        // Should include function body + idioms
        assert!(!items.is_empty());
    }

    #[test]
    fn test_ai_completion_sort_prefix() {
        let item = ai_completion("test", "test", "test", "key");
        assert!(item.sort_text.unwrap().starts_with(AI_SORT_PREFIX));
    }

    #[test]
    fn test_completion_context_from_document() {
        let source = "F main() -> i64 {\n    \n}\n";
        let pos = Position {
            line: 1,
            character: 4,
        };
        let ctx = CompletionContext::from_document(source, pos, None);
        assert_eq!(ctx.prefix_lines.len(), 1);
        assert_eq!(ctx.current_line_prefix, "    ");
    }
}

//! Signature Help handler implementation

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use vais_ast::Item;

use crate::backend::VaisBackend;

/// Get builtin function signature information
fn get_builtin_signature(name: &str) -> Option<(String, Vec<ParameterInformation>)> {
    let info = match name {
        "puts" => ("fn(str) -> i64", vec!["str"]),
        "putchar" => ("fn(i64) -> i64", vec!["char"]),
        "print_i64" => ("fn(i64) -> i64", vec!["value"]),
        "print_f64" => ("fn(f64) -> i64", vec!["value"]),
        "malloc" => ("fn(i64) -> i64", vec!["size"]),
        "free" => ("fn(i64) -> i64", vec!["ptr"]),
        "memcpy" => ("fn(i64, i64, i64) -> i64", vec!["dst", "src", "n"]),
        "strlen" => ("fn(i64) -> i64", vec!["str"]),
        "load_i64" => ("fn(i64) -> i64", vec!["addr"]),
        "store_i64" => ("fn(i64, i64) -> i64", vec!["addr", "value"]),
        "load_byte" => ("fn(i64) -> i64", vec!["addr"]),
        "store_byte" => ("fn(i64, i64) -> i64", vec!["addr", "value"]),
        "sqrt" => ("fn(f64) -> f64", vec!["x"]),
        "sin" => ("fn(f64) -> f64", vec!["x"]),
        "cos" => ("fn(f64) -> f64", vec!["x"]),
        "tan" => ("fn(f64) -> f64", vec!["x"]),
        "pow" => ("fn(f64, f64) -> f64", vec!["x", "y"]),
        "log" => ("fn(f64) -> f64", vec!["x"]),
        "exp" => ("fn(f64) -> f64", vec!["x"]),
        "floor" => ("fn(f64) -> f64", vec!["x"]),
        "ceil" => ("fn(f64) -> f64", vec!["x"]),
        "round" => ("fn(f64) -> f64", vec!["x"]),
        "abs" => ("fn(f64) -> f64", vec!["x"]),
        "abs_i64" => ("fn(i64) -> i64", vec!["x"]),
        "min" => ("fn(f64, f64) -> f64", vec!["a", "b"]),
        "max" => ("fn(f64, f64) -> f64", vec!["a", "b"]),
        "read_i64" => ("fn() -> i64", vec![]),
        "read_f64" => ("fn() -> f64", vec![]),
        "read_line" => ("fn(i64, i64) -> i64", vec!["buffer", "size"]),
        "read_char" => ("fn() -> i64", vec![]),
        _ => return None,
    };

    let (sig, param_names) = info;
    let parameters: Vec<ParameterInformation> = param_names
        .iter()
        .map(|name| ParameterInformation {
            label: ParameterLabel::Simple(name.to_string()),
            documentation: None,
        })
        .collect();

    Some((sig.to_string(), parameters))
}

/// Find the function call context at the given offset
/// Returns (function_name, active_parameter_index)
fn find_call_context(content: &str, offset: usize) -> Option<(String, usize)> {
    // Walk backwards from cursor to find the opening parenthesis
    let chars: Vec<char> = content.chars().collect();
    if offset > chars.len() {
        return None;
    }

    let mut paren_depth = 0;
    let mut i = offset.saturating_sub(1);

    // Walk backwards to find the matching opening parenthesis
    loop {
        if i >= chars.len() {
            break;
        }

        match chars[i] {
            ')' => paren_depth += 1,
            '(' => {
                if paren_depth == 0 {
                    // Found the opening paren of our call
                    // Now extract the function name
                    let func_name = extract_function_name(&chars, i)?;

                    // Count commas to determine active parameter
                    let active_param = count_active_parameter(&chars, i + 1, offset);

                    return Some((func_name, active_param));
                }
                paren_depth -= 1;
            }
            _ => {}
        }

        if i == 0 {
            break;
        }
        i -= 1;
    }

    None
}

/// Extract the function name before the opening parenthesis
fn extract_function_name(chars: &[char], paren_pos: usize) -> Option<String> {
    if paren_pos == 0 {
        return None;
    }

    let mut i = paren_pos - 1;

    // Skip whitespace
    while i < chars.len() && chars[i].is_whitespace() {
        if i == 0 {
            return None;
        }
        i -= 1;
    }

    // Collect identifier characters
    let mut name = String::new();
    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
        name.push(chars[i]);
        if i == 0 {
            break;
        }
        i -= 1;
    }

    if name.is_empty() {
        return None;
    }

    // Reverse since we collected backwards
    Some(name.chars().rev().collect())
}

/// Count the active parameter index based on commas
fn count_active_parameter(chars: &[char], start: usize, end: usize) -> usize {
    let mut paren_depth = 0;
    let mut comma_count = 0;

    for ch in chars.iter().take(end.min(chars.len())).skip(start) {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            ',' if paren_depth == 0 => comma_count += 1,
            _ => {}
        }
    }

    comma_count
}

pub(crate) async fn handle_signature_help(
    backend: &VaisBackend,
    params: SignatureHelpParams,
) -> Result<Option<SignatureHelp>> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    if let Some(doc) = backend.documents.get(uri) {
        // Convert position to offset
        let line = position.line as usize;
        let col = position.character as usize;
        let line_start = doc.content.line_to_char(line);
        let offset = line_start + col;

        let content = doc.content.to_string();

        // Find the call context (function name and active parameter)
        if let Some((func_name, active_param)) = find_call_context(&content, offset) {
            // First check if it's a builtin function
            if let Some((sig, params)) = get_builtin_signature(&func_name) {
                let signature = SignatureInformation {
                    label: sig,
                    documentation: Some(Documentation::String(format!(
                        "Built-in function `{}`",
                        func_name
                    ))),
                    parameters: Some(params),
                    active_parameter: None,
                };

                return Ok(Some(SignatureHelp {
                    signatures: vec![signature],
                    active_signature: Some(0),
                    active_parameter: Some(active_param as u32),
                }));
            }

            // Check if it's a user-defined function in the AST
            if let Some(ast) = &doc.ast {
                for item in &ast.items {
                    if let Item::Function(f) = &item.node {
                        if f.name.node == func_name {
                            // Build signature string
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
                            let sig_label = format!(
                                "{}F {}({}){}",
                                is_async,
                                f.name.node,
                                params_str.join(", "),
                                ret_str
                            );

                            // Build parameter information
                            let param_infos: Vec<ParameterInformation> = f
                                .params
                                .iter()
                                .map(|p| {
                                    let label = format!("{}: {:?}", p.name.node, p.ty.node);
                                    ParameterInformation {
                                        label: ParameterLabel::Simple(label),
                                        documentation: None,
                                    }
                                })
                                .collect();

                            let signature = SignatureInformation {
                                label: sig_label,
                                documentation: Some(Documentation::String(
                                    "Function defined in current file".to_string()
                                )),
                                parameters: Some(param_infos),
                                active_parameter: None,
                            };

                            return Ok(Some(SignatureHelp {
                                signatures: vec![signature],
                                active_signature: Some(0),
                                active_parameter: Some(active_param.min(f.params.len().saturating_sub(1)) as u32),
                            }));
                        }
                    }

                    // Check methods in impl blocks
                    if let Item::Impl(impl_block) = &item.node {
                        for method in &impl_block.methods {
                            if method.node.name.node == func_name {
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

                                let is_async = if method.node.is_async { "A " } else { "" };
                                let sig_label = format!(
                                    "{}F {}({}){}",
                                    is_async,
                                    method.node.name.node,
                                    params_str.join(", "),
                                    ret_str
                                );

                                let param_infos: Vec<ParameterInformation> = method
                                    .node
                                    .params
                                    .iter()
                                    .map(|p| {
                                        let label = format!("{}: {:?}", p.name.node, p.ty.node);
                                        ParameterInformation {
                                            label: ParameterLabel::Simple(label),
                                            documentation: None,
                                        }
                                    })
                                    .collect();

                                let target_type = format!("{:?}", impl_block.target_type.node);
                                let signature = SignatureInformation {
                                    label: sig_label,
                                    documentation: Some(Documentation::String(format!(
                                        "Method of `{}`",
                                        target_type
                                    ))),
                                    parameters: Some(param_infos),
                                    active_parameter: None,
                                };

                                return Ok(Some(SignatureHelp {
                                    signatures: vec![signature],
                                    active_signature: Some(0),
                                    active_parameter: Some(active_param.min(method.node.params.len().saturating_sub(1)) as u32),
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

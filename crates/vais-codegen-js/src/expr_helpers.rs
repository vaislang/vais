//! Helper functions for JavaScript expression code generation

use vais_ast::{BinOp, UnaryOp};

/// Convert Vais BinOp to JavaScript operator string
pub(crate) fn binop_to_js(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Mod => "%",
        BinOp::Lt => "<",
        BinOp::Lte => "<=",
        BinOp::Gt => ">",
        BinOp::Gte => ">=",
        BinOp::Eq => "===",
        BinOp::Neq => "!==",
        BinOp::And => "&&",
        BinOp::Or => "||",
        BinOp::BitAnd => "&",
        BinOp::BitOr => "|",
        BinOp::BitXor => "^",
        BinOp::Shl => "<<",
        BinOp::Shr => ">>",
    }
}

/// Convert Vais UnaryOp to JavaScript operator string
pub(crate) fn unaryop_to_js(op: &UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "-",
        UnaryOp::Not => "!",
        UnaryOp::BitNot => "~",
    }
}

/// Escape special characters in a JavaScript string
pub(crate) fn escape_js_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\0' => result.push_str("\\0"),
            _ => result.push(ch),
        }
    }
    result
}

/// Escape special characters in a template literal
pub(crate) fn escape_template_literal(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '`' => result.push_str("\\`"),
            '$' => result.push_str("\\$"),
            '\\' => result.push_str("\\\\"),
            _ => result.push(ch),
        }
    }
    result
}

/// Sanitize a Vais identifier for use in JavaScript
pub(crate) fn sanitize_js_ident(name: &str) -> String {
    // JS reserved words that need renaming
    match name {
        "class" => "_class".to_string(),
        "delete" => "_delete".to_string(),
        "export" => "_export".to_string(),
        "import" => "_import".to_string(),
        "new" => "_new".to_string(),
        "super" => "_super".to_string(),
        "switch" => "_switch".to_string(),
        "this" => "_this".to_string(),
        "throw" => "_throw".to_string(),
        "typeof" => "_typeof".to_string(),
        "var" => "_var".to_string(),
        "void" => "_void".to_string(),
        "with" => "_with".to_string(),
        "yield" => "_yield".to_string(),
        "await" => "_await".to_string(),
        "enum" => "_enum".to_string(),
        "implements" => "_implements".to_string(),
        "interface" => "_interface".to_string(),
        "package" => "_package".to_string(),
        "private" => "_private".to_string(),
        "protected" => "_protected".to_string(),
        "public" => "_public".to_string(),
        "static" => "_static".to_string(),
        "arguments" => "_arguments".to_string(),
        "eval" => "_eval".to_string(),
        _ => name.to_string(),
    }
}

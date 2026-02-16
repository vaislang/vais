//! Common subexpression elimination.

use std::collections::HashMap;

/// Common subexpression elimination
pub(crate) fn common_subexpression_elimination(ir: &str) -> String {
    let mut result = String::with_capacity(ir.len());
    let mut expr_to_var: HashMap<String, String> = HashMap::new();

    for line in ir.lines() {
        let trimmed = line.trim();

        // Reset CSE map at function boundaries
        if line.starts_with("define ") {
            expr_to_var.clear();
        }

        // Pattern: %N = BINOP TYPE A, B
        if let Some((dest, expr)) = extract_binop_expr(trimmed) {
            // Check if we've seen this expression before
            if let Some(existing) = expr_to_var.get(&expr) {
                // Replace with reference to existing computation
                result.push_str(&format!(
                    "  {} = add i64 0, {}  ; CSE: reusing {}\n",
                    dest, existing, expr
                ));
                continue;
            } else {
                // Record this expression
                expr_to_var.insert(expr, dest.clone());
            }
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Extract binary operation expression for CSE
fn extract_binop_expr(line: &str) -> Option<(String, String)> {
    let ops = ["add", "sub", "mul", "sdiv", "and", "or", "xor"];

    for op in &ops {
        let pattern = format!(" = {} i64 ", op);
        if line.contains(&pattern) {
            let parts: Vec<&str> = line.split(&pattern).collect();
            if parts.len() == 2 {
                let dest = parts[0].trim().to_string();
                let expr = format!("{} i64 {}", op, parts[1].trim());
                return Some((dest, expr));
            }
        }
    }
    None
}

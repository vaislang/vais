//! Constant folding - evaluate constant expressions at compile time

/// Constant folding - evaluate constant expressions at compile time
pub(crate) fn constant_folding(ir: &str) -> String {
    let mut result = String::with_capacity(ir.len());

    for line in ir.lines() {
        let trimmed = line.trim();

        // Pattern: %N = add i64 X, Y where both are constants
        if let Some(folded) = try_fold_binary_op(trimmed, "add", |a, b| a + b) {
            result.push_str(&folded);
            result.push('\n');
            continue;
        }
        if let Some(folded) = try_fold_binary_op(trimmed, "sub", |a, b| a - b) {
            result.push_str(&folded);
            result.push('\n');
            continue;
        }
        if let Some(folded) = try_fold_binary_op(trimmed, "mul", |a, b| a * b) {
            result.push_str(&folded);
            result.push('\n');
            continue;
        }
        if let Some(folded) =
            try_fold_binary_op(trimmed, "sdiv", |a, b| if b != 0 { a / b } else { 0 })
        {
            result.push_str(&folded);
            result.push('\n');
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Try to fold a binary operation with constant operands
fn try_fold_binary_op<F>(line: &str, op: &str, f: F) -> Option<String>
where
    F: Fn(i64, i64) -> i64,
{
    // Pattern: %N = add i64 X, Y
    let pattern = format!(" = {} i64 ", op);
    if !line.contains(&pattern) {
        return None;
    }

    let parts: Vec<&str> = line.split(&pattern).collect();
    if parts.len() != 2 {
        return None;
    }

    let dest = parts[0].trim();
    let operands: Vec<&str> = parts[1].split(',').map(|s| s.trim()).collect();
    if operands.len() != 2 {
        return None;
    }

    // Try to parse both operands as constants
    let a = operands[0].parse::<i64>().ok()?;
    let b = operands[1].parse::<i64>().ok()?;

    let result = f(a, b);
    Some(format!(
        "  {} = add i64 0, {}  ; folded from {} {} {}",
        dest, result, a, op, b
    ))
}


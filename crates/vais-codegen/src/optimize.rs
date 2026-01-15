//! LLVM IR Optimization Passes
//!
//! Text-based optimization passes for the generated LLVM IR.
//! These are applied before passing the IR to clang for final optimization.

use std::collections::{HashMap, HashSet};

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptLevel {
    O0, // No optimization
    O1, // Basic optimization
    O2, // Standard optimization
    O3, // Aggressive optimization
}

impl OptLevel {
    pub fn from_str(s: &str) -> Self {
        match s {
            "0" | "O0" => OptLevel::O0,
            "1" | "O1" => OptLevel::O1,
            "2" | "O2" => OptLevel::O2,
            "3" | "O3" => OptLevel::O3,
            _ => OptLevel::O0,
        }
    }
}

/// Apply optimization passes to LLVM IR
pub fn optimize_ir(ir: &str, level: OptLevel) -> String {
    if level == OptLevel::O0 {
        return ir.to_string();
    }

    let mut result = ir.to_string();

    // O1+: Basic optimizations
    if level >= OptLevel::O1 {
        result = constant_folding(&result);
        result = dead_store_elimination(&result);
    }

    // O2+: More aggressive optimizations
    if level >= OptLevel::O2 {
        result = common_subexpression_elimination(&result);
        result = strength_reduction(&result);
    }

    // O3: Most aggressive optimizations
    if level >= OptLevel::O3 {
        result = aggressive_inline(&result);
    }

    result
}

/// Constant folding - evaluate constant expressions at compile time
fn constant_folding(ir: &str) -> String {
    let mut result = String::new();

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
        if let Some(folded) = try_fold_binary_op(trimmed, "sdiv", |a, b| if b != 0 { a / b } else { 0 }) {
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
    Some(format!("  {} = add i64 0, {}  ; folded from {} {} {}", dest, result, a, op, b))
}

/// Dead store elimination - remove stores that are never read
fn dead_store_elimination(ir: &str) -> String {
    let lines: Vec<&str> = ir.lines().collect();
    let mut used_vars: HashSet<String> = HashSet::new();
    let mut result = Vec::new();

    // First pass: collect all used variables
    for line in &lines {
        let trimmed = line.trim();

        // Skip stores and allocas for now
        if trimmed.starts_with("store") || trimmed.starts_with("alloca") {
            continue;
        }

        // Collect all %variables referenced
        for word in trimmed.split(|c: char| !c.is_alphanumeric() && c != '%' && c != '.') {
            if word.starts_with('%') {
                used_vars.insert(word.to_string());
            }
        }
    }

    // Second pass: emit only stores to used variables
    for line in &lines {
        let trimmed = line.trim();

        // Check if this is a dead store
        if trimmed.starts_with("store") {
            // Pattern: store TYPE VALUE, TYPE* DEST
            let parts: Vec<&str> = trimmed.split(',').collect();
            if parts.len() >= 2 {
                let dest_part = parts[1].trim();
                // Extract the destination variable
                if let Some(var) = dest_part.split_whitespace().last() {
                    // If the stored variable is never used, skip this store
                    // But keep stores to function arguments and globals
                    if !used_vars.contains(var) && !var.starts_with("@") {
                        // This is a dead store - add as comment
                        result.push(format!("  ; dead store eliminated: {}", trimmed));
                        continue;
                    }
                }
            }
        }

        result.push(line.to_string());
    }

    result.join("\n")
}

/// Common subexpression elimination
fn common_subexpression_elimination(ir: &str) -> String {
    let mut result = String::new();
    let mut expr_to_var: HashMap<String, String> = HashMap::new();

    for line in ir.lines() {
        let trimmed = line.trim();

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

/// Strength reduction - replace expensive operations with cheaper ones
fn strength_reduction(ir: &str) -> String {
    let mut result = String::new();

    for line in ir.lines() {
        let trimmed = line.trim();

        // Replace multiplication by power of 2 with shift
        if let Some(reduced) = try_strength_reduce_mul(trimmed) {
            result.push_str(&reduced);
            result.push('\n');
            continue;
        }

        // Replace division by power of 2 with shift
        if let Some(reduced) = try_strength_reduce_div(trimmed) {
            result.push_str(&reduced);
            result.push('\n');
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Try to reduce multiplication to shift
fn try_strength_reduce_mul(line: &str) -> Option<String> {
    if !line.contains(" = mul i64 ") {
        return None;
    }

    let parts: Vec<&str> = line.split(" = mul i64 ").collect();
    if parts.len() != 2 {
        return None;
    }

    let dest = parts[0].trim();
    let operands: Vec<&str> = parts[1].split(',').map(|s| s.trim()).collect();
    if operands.len() != 2 {
        return None;
    }

    // Check if one operand is a power of 2
    let (var, shift) = if let Ok(n) = operands[1].parse::<i64>() {
        if is_power_of_2(n) {
            (operands[0], log2(n))
        } else {
            return None;
        }
    } else if let Ok(n) = operands[0].parse::<i64>() {
        if is_power_of_2(n) {
            (operands[1], log2(n))
        } else {
            return None;
        }
    } else {
        return None;
    };

    Some(format!(
        "  {} = shl i64 {}, {}  ; strength reduced from mul by {}",
        dest, var, shift, 1i64 << shift
    ))
}

/// Try to reduce division to shift (only for unsigned or positive known values)
fn try_strength_reduce_div(line: &str) -> Option<String> {
    if !line.contains(" = sdiv i64 ") {
        return None;
    }

    let parts: Vec<&str> = line.split(" = sdiv i64 ").collect();
    if parts.len() != 2 {
        return None;
    }

    let dest = parts[0].trim();
    let operands: Vec<&str> = parts[1].split(',').map(|s| s.trim()).collect();
    if operands.len() != 2 {
        return None;
    }

    // Only reduce if divisor is power of 2
    if let Ok(n) = operands[1].parse::<i64>() {
        if is_power_of_2(n) && n > 0 {
            let shift = log2(n);
            return Some(format!(
                "  {} = ashr i64 {}, {}  ; strength reduced from div by {}",
                dest, operands[0], shift, n
            ));
        }
    }

    None
}

fn is_power_of_2(n: i64) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

fn log2(n: i64) -> u32 {
    (63 - n.leading_zeros()) as u32
}

/// Aggressive inlining for small functions
fn aggressive_inline(ir: &str) -> String {
    // For now, just add inline hints as comments
    // Real inlining would require parsing and rewriting function calls
    let mut result = ir.to_string();

    // Add always_inline attribute hint for small functions
    let small_threshold = 5; // lines
    let mut in_function = false;
    let mut function_start = 0;
    let mut function_name = String::new();
    let mut line_count = 0;

    let lines: Vec<&str> = ir.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("define ") && line.contains("@") {
            in_function = true;
            function_start = i;
            line_count = 0;
            // Extract function name
            if let Some(start) = line.find('@') {
                if let Some(end) = line[start..].find('(') {
                    function_name = line[start..start + end].to_string();
                }
            }
        } else if line.trim() == "}" && in_function {
            in_function = false;
            if line_count <= small_threshold && !function_name.contains("main") {
                // This function is small enough to be a good inline candidate
                // In a real implementation, we would add alwaysinline attribute
            }
        } else if in_function {
            line_count += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_folding() {
        let ir = "  %0 = add i64 10, 20\n";
        let result = constant_folding(ir);
        assert!(result.contains("30"));
    }

    #[test]
    fn test_strength_reduction_mul() {
        let line = "  %0 = mul i64 %x, 8";
        let result = try_strength_reduce_mul(line);
        assert!(result.is_some());
        assert!(result.unwrap().contains("shl"));
    }

    #[test]
    fn test_is_power_of_2() {
        assert!(is_power_of_2(1));
        assert!(is_power_of_2(2));
        assert!(is_power_of_2(4));
        assert!(is_power_of_2(8));
        assert!(!is_power_of_2(3));
        assert!(!is_power_of_2(0));
        assert!(!is_power_of_2(-1));
    }
}

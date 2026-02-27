//! Branch optimization and simplification.

use std::collections::{HashMap, HashSet};

/// Branch optimization - simplify branches with constant conditions
pub(crate) fn branch_optimization(ir: &str) -> String {
    let mut result = Vec::new();
    let mut skip_until_label = false;
    let target_label = String::new();

    for line in ir.lines() {
        let trimmed = line.trim();

        // If we're skipping dead code, wait for the target label
        if skip_until_label {
            if trimmed.ends_with(':') {
                let label = trimmed.trim_end_matches(':');
                if label == target_label {
                    skip_until_label = false;
                    result.push(line.to_string());
                } else {
                    result.push(format!("  ; dead block skipped: {}", trimmed));
                }
            } else {
                result.push(format!("  ; dead code: {}", trimmed));
            }
            continue;
        }

        // Pattern: br i1 true/false, label %then, label %else
        if trimmed.starts_with("br i1 ") {
            if trimmed.contains("br i1 true,") || trimmed.contains("br i1 1,") {
                // Always branch to 'then'
                if let Some(then_label) = extract_branch_label(trimmed, true) {
                    result.push(format!(
                        "  br label %{}  ; simplified from conditional",
                        then_label
                    ));
                    continue;
                }
            } else if trimmed.contains("br i1 false,") || trimmed.contains("br i1 0,") {
                // Always branch to 'else'
                if let Some(else_label) = extract_branch_label(trimmed, false) {
                    result.push(format!(
                        "  br label %{}  ; simplified from conditional",
                        else_label
                    ));
                    continue;
                }
            }
        }

        // Pattern: icmp eq X, X (always true)
        if trimmed.contains(" = icmp eq ") {
            let parts: Vec<&str> = trimmed.split(" = icmp eq ").collect();
            if parts.len() == 2 {
                let operands_str = parts[1].trim();
                // Extract operands after type
                if let Some(type_end) = operands_str.find(' ') {
                    let operands_part = &operands_str[type_end + 1..];
                    let operands: Vec<&str> = operands_part.split(',').map(|s| s.trim()).collect();
                    if operands.len() == 2 && operands[0] == operands[1] {
                        result.push(format!(
                            "  {} = add i1 0, true  ; simplified: X == X",
                            parts[0].trim()
                        ));
                        continue;
                    }
                }
            }
        }

        result.push(line.to_string());
    }

    result.join("\n")
}

/// Extract branch target label
fn extract_branch_label(line: &str, take_then: bool) -> Option<String> {
    // Pattern: br i1 COND, label %THEN, label %ELSE
    let parts: Vec<&str> = line.split("label %").collect();
    if parts.len() >= 3 {
        let then_label = parts[1].split(',').next()?.trim();
        let else_label = parts[2].trim();
        if take_then {
            return Some(then_label.to_string());
        } else {
            return Some(else_label.to_string());
        }
    }
    None
}

/// Conditional branch simplification
/// Removes redundant zext i1 + icmp ne patterns:
///   %X = icmp ... (produces i1)
///   %Y = zext i1 %X to i64
///   %Z = icmp ne i64 %Y, 0
///   br i1 %Z, label %then, label %else
/// Becomes:
///   %X = icmp ... (produces i1)
///   br i1 %X, label %then, label %else
///
/// IMPORTANT: Only removes zext/icmp if the result is ONLY used in br i1
pub(crate) fn conditional_branch_simplification(ir: &str) -> String {
    let lines: Vec<&str> = ir.lines().collect();
    let mut result = Vec::new();

    // Track i1 sources: zext_dest -> (original_i1_var, line_index)
    let mut i1_sources: HashMap<String, (String, usize)> = HashMap::new();
    // Track icmp ne %zext, 0 -> (original_i1_var, line_index, zext_var)
    let mut icmp_to_i1: HashMap<String, (String, usize, String)> = HashMap::new();
    // Track variable uses: var_name -> count of uses (excluding its definition)
    let mut var_uses: HashMap<String, usize> = HashMap::new();

    // First pass: collect zext i1 to i64 patterns
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Pattern: %Y = zext i1 %X to i64
        if trimmed.contains(" = zext i1 ") && trimmed.contains(" to i64") {
            if let Some((dest, src)) = parse_zext_i1(trimmed) {
                i1_sources.insert(dest, (src, i));
            }
        }
    }

    // Second pass: find icmp ne i64 %zext, 0
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Pattern: %Z = icmp ne i64 %Y, 0
        if trimmed.contains(" = icmp ne i64 ") {
            if let Some((dest, operand)) = parse_icmp_ne_zero(trimmed) {
                if let Some((original_i1, _)) = i1_sources.get(&operand) {
                    icmp_to_i1.insert(dest, (original_i1.clone(), i, operand.clone()));
                }
            }
        }
    }

    // Third pass: count variable uses
    for line in lines.iter() {
        let trimmed = line.trim();
        // Skip comments
        if trimmed.starts_with(';') {
            continue;
        }
        // Count all %var references in this line
        for word in trimmed.split(|c: char| !c.is_alphanumeric() && c != '%' && c != '.') {
            if word.starts_with('%') && !word.is_empty() {
                // Check if this is a use (not a definition)
                // Definition pattern: "%var = ..."
                if !trimmed.starts_with(word) || !trimmed.contains(" = ") {
                    *var_uses.entry(word.to_string()).or_insert(0) += 1;
                } else if trimmed.starts_with(word) && trimmed.contains(" = ") {
                    // This is a definition, but also check for uses in the RHS
                    // safe: checked above that trimmed contains " = "
                    let def_end = trimmed.find(" = ").unwrap() + 3;
                    let rhs = &trimmed[def_end..];
                    if rhs.contains(word) {
                        *var_uses.entry(word.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    // Determine which lines can be safely removed:
    // - icmp ne line can be removed if its result is only used once in a br i1
    // - zext line can be removed if its result is only used once in the icmp ne
    let mut dead_lines: HashSet<usize> = HashSet::new();
    let mut safe_icmp_replacements: HashMap<String, String> = HashMap::new();

    for (icmp_var, (original_i1, icmp_line, zext_var)) in &icmp_to_i1 {
        // Check if icmp result is only used in br i1
        let icmp_uses = var_uses.get(icmp_var).copied().unwrap_or(0);
        // Check if zext result is only used in this icmp
        let zext_uses = var_uses.get(zext_var).copied().unwrap_or(0);

        // icmp should be used exactly once (in br i1)
        // zext should be used exactly once (in this icmp)
        if icmp_uses == 1 && zext_uses == 1 {
            dead_lines.insert(*icmp_line);
            if let Some((_, zext_line)) = i1_sources.get(zext_var) {
                dead_lines.insert(*zext_line);
            }
            safe_icmp_replacements.insert(icmp_var.clone(), original_i1.clone());
        }
    }

    // Fourth pass: generate optimized output
    for (i, line) in lines.iter().enumerate() {
        if dead_lines.contains(&i) {
            // Skip dead zext/icmp lines, add comment for debugging
            result.push(format!("  ; optimized out: {}", line.trim()));
            continue;
        }

        let trimmed = line.trim();

        // Pattern: br i1 %Z, label %then, label %else
        if trimmed.starts_with("br i1 ") {
            if let Some(replaced) = try_replace_branch_cond(trimmed, &safe_icmp_replacements) {
                result.push(format!("  {}", replaced));
                continue;
            }
        }

        result.push(line.to_string());
    }

    result.join("\n")
}

/// Parse: %Y = zext i1 %X to i64
fn parse_zext_i1(line: &str) -> Option<(String, String)> {
    // Pattern: %dest = zext i1 %src to i64
    let parts: Vec<&str> = line.split(" = zext i1 ").collect();
    if parts.len() != 2 {
        return None;
    }

    let dest = parts[0].trim().to_string();
    let rest = parts[1].trim();

    // Extract source variable
    if let Some(src_end) = rest.find(" to i64") {
        let src = rest[..src_end].trim().to_string();
        return Some((dest, src));
    }

    None
}

/// Parse: %Z = icmp ne i64 %Y, 0
fn parse_icmp_ne_zero(line: &str) -> Option<(String, String)> {
    // Pattern: %dest = icmp ne i64 %operand, 0
    let parts: Vec<&str> = line.split(" = icmp ne i64 ").collect();
    if parts.len() != 2 {
        return None;
    }

    let dest = parts[0].trim().to_string();
    let operands = parts[1].trim();

    // Check if second operand is 0
    if operands.ends_with(", 0") {
        let operand = operands.trim_end_matches(", 0").trim().to_string();
        return Some((dest, operand));
    }

    None
}

/// Try to replace branch condition with original i1 value
fn try_replace_branch_cond(line: &str, icmp_to_i1: &HashMap<String, String>) -> Option<String> {
    // Pattern: br i1 %Z, label %then, label %else
    let prefix = "br i1 ";
    if !line.starts_with(prefix) {
        return None;
    }

    let rest = &line[prefix.len()..];

    // Extract condition variable
    if let Some(comma_pos) = rest.find(',') {
        let cond = rest[..comma_pos].trim();
        let labels = &rest[comma_pos..];

        // Check if this condition can be replaced
        if let Some(original_i1) = icmp_to_i1.get(cond) {
            return Some(format!("br i1 {}{} ; simplified", original_i1, labels));
        }
    }

    None
}

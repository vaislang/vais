//! Dead code and dead store elimination.

use std::collections::{HashMap, HashSet};

/// Dead store elimination - remove stores that are never read
pub(crate) fn dead_store_elimination(ir: &str) -> String {
    let lines: Vec<&str> = ir.lines().collect();
    let mut loaded_vars: HashSet<String> = HashSet::new();
    let mut result = Vec::new();

    // First pass: collect all variables that are loaded (read)
    for line in &lines {
        let trimmed = line.trim();

        // Look for load instructions: %N = load TYPE, TYPE* %ptr
        if trimmed.contains(" = load ") {
            // Extract the source pointer
            if let Some(ptr_start) = trimmed.rfind('%') {
                let ptr = &trimmed[ptr_start..];
                // Clean up the variable name
                let var: String = ptr
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '%' || *c == '.' || *c == '_')
                    .collect();
                loaded_vars.insert(var);
            }
        }
    }

    // Second pass: emit only stores to variables that are loaded
    for line in &lines {
        let trimmed = line.trim();

        // Check if this is a potentially dead store
        if trimmed.starts_with("store") {
            // Pattern: store TYPE VALUE, TYPE* DEST
            let parts: Vec<&str> = trimmed.split(',').collect();
            if parts.len() >= 2 {
                let dest_part = parts[1].trim();
                // Extract the destination variable
                if let Some(var) = dest_part.split_whitespace().last() {
                    // If the stored variable is never loaded, it's a dead store
                    // But keep stores to globals (@)
                    if !loaded_vars.contains(var) && !var.starts_with("@") {
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

/// Dead code elimination - remove unused definitions
pub(crate) fn dead_code_elimination(ir: &str) -> String {
    let lines: Vec<&str> = ir.lines().collect();
    let mut used_vars: HashSet<String> = HashSet::new();
    let mut var_definitions: HashMap<String, usize> = HashMap::new();

    // First pass: collect all variable uses and definitions
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip comments and labels
        if trimmed.starts_with(';') || trimmed.ends_with(':') {
            continue;
        }

        // Track definitions
        if let Some(def_var) = extract_definition(trimmed) {
            var_definitions.insert(def_var, i);
        }

        // Track uses (excluding the definition itself)
        for word in
            trimmed.split(|c: char| !c.is_alphanumeric() && c != '%' && c != '_' && c != '.')
        {
            if word.starts_with('%') && !word.is_empty() {
                // Check if this is a use, not just the definition
                if !trimmed.starts_with(&format!("{} =", word))
                    && !trimmed.starts_with(&format!("  {} =", word))
                {
                    used_vars.insert(word.to_string());
                }
            }
        }
    }

    // Also mark return values, call arguments, and branch conditions as used
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("ret ")
            || trimmed.starts_with("br ")
            || trimmed.starts_with("store ")
            || trimmed.contains("call ")
        {
            for word in
                trimmed.split(|c: char| !c.is_alphanumeric() && c != '%' && c != '_' && c != '.')
            {
                if word.starts_with('%') {
                    used_vars.insert(word.to_string());
                }
            }
        }
    }

    // Second pass: emit only used definitions
    let mut result = Vec::new();
    for line in lines.iter() {
        let trimmed = line.trim();

        // Check if this is a definition of an unused variable
        if let Some(def_var) = extract_definition(trimmed) {
            if !used_vars.contains(&def_var) {
                // Check if this is a side-effect free instruction
                if is_side_effect_free(trimmed) {
                    result.push(format!("  ; DCE removed: {}", trimmed));
                    continue;
                }
            }
        }

        result.push(line.to_string());
    }

    result.join("\n")
}

/// Extract the variable being defined, if any
pub(super) fn extract_definition(line: &str) -> Option<String> {
    // Pattern: %VAR = ...
    let trimmed = line.trim();
    if let Some(eq_pos) = trimmed.find(" = ") {
        let var = trimmed[..eq_pos].trim();
        if var.starts_with('%') {
            return Some(var.to_string());
        }
    }
    None
}

/// Check if an instruction has no side effects
fn is_side_effect_free(line: &str) -> bool {
    let trimmed = line.trim();
    // Pure operations that can be eliminated if unused
    let pure_ops = [
        "add ",
        "sub ",
        "mul ",
        "sdiv ",
        "udiv ",
        "and ",
        "or ",
        "xor ",
        "shl ",
        "ashr ",
        "lshr ",
        "icmp ",
        "fcmp ",
        "select ",
        "zext ",
        "sext ",
        "trunc ",
        "bitcast ",
        "getelementptr ",
        "extractvalue ",
        "insertvalue ",
        "load ",
    ];

    for op in &pure_ops {
        if trimmed.contains(op) {
            return true;
        }
    }
    false
}

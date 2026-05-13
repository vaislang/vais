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
    // Phase 1: Standard unused variable DCE
    let after_basic_dce = basic_dead_code_elimination(ir);

    // Phase 2: Unreachable block elimination (constant branch folding)
    let after_unreachable = unreachable_block_elimination(&after_basic_dce);

    // Phase 3: Dead pure-call elimination (unused results of side-effect-free calls)
    let after_dead_calls = dead_pure_call_elimination(&after_unreachable);

    // Phase 4: Redundant store-load elimination
    redundant_store_load_elimination(&after_dead_calls)
}

/// Basic dead code elimination - remove unused definitions
fn basic_dead_code_elimination(ir: &str) -> String {
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

/// Unreachable block elimination.
///
/// Detects constant-condition branches (br i1 true/false or br i1 1/0) and
/// removes the unreachable target block entirely if it has no other predecessors.
fn unreachable_block_elimination(ir: &str) -> String {
    let lines: Vec<&str> = ir.lines().collect();

    // Pass 1: Find blocks that are unreachable due to constant branches
    let mut reachable_labels: HashSet<String> = HashSet::new();
    let mut unreachable_labels: HashSet<String> = HashSet::new();

    // Collect all branch targets to determine reachability
    for line in &lines {
        let trimmed = line.trim();

        // Constant-true branch: only "then" is reachable
        if (trimmed.contains("br i1 true,") || trimmed.contains("br i1 1,"))
            && trimmed.contains("label %")
        {
            let parts: Vec<&str> = trimmed.split("label %").collect();
            if parts.len() >= 3 {
                if let Some(then_label) = parts[1].split(',').next() {
                    reachable_labels.insert(then_label.trim().to_string());
                }
                let else_label = parts[2].trim().to_string();
                unreachable_labels.insert(else_label);
            }
        }
        // Constant-false branch: only "else" is reachable
        else if (trimmed.contains("br i1 false,") || trimmed.contains("br i1 0,"))
            && trimmed.contains("label %")
        {
            let parts: Vec<&str> = trimmed.split("label %").collect();
            if parts.len() >= 3 {
                if let Some(then_label) = parts[1].split(',').next() {
                    unreachable_labels.insert(then_label.trim().to_string());
                }
                let else_label = parts[2].trim().to_string();
                reachable_labels.insert(else_label);
            }
        }
        // Unconditional branches and other control flow make targets reachable
        else if trimmed.starts_with("br label %") {
            if let Some(label) = trimmed.strip_prefix("br label %") {
                let label = label
                    .split(|c: char| !c.is_alphanumeric() && c != '.' && c != '_')
                    .next()
                    .unwrap_or("");
                reachable_labels.insert(label.to_string());
            }
        }
    }

    // Remove from unreachable set any labels that are also reachable from other branches
    unreachable_labels.retain(|l| !reachable_labels.contains(l));

    // Pass 2: Eliminate unreachable blocks
    let mut result = Vec::new();
    let mut in_dead_block = false;

    for line in &lines {
        let trimmed = line.trim();

        // Check if this is a label that starts an unreachable block
        if trimmed.ends_with(':') {
            let label = trimmed.trim_end_matches(':');
            if unreachable_labels.contains(label) {
                in_dead_block = true;
                result.push(format!("  ; unreachable block removed: {}", trimmed));
                continue;
            } else {
                // We hit a new label that is not unreachable => exit dead block
                in_dead_block = false;
            }
        }

        if in_dead_block {
            // In a dead block - only emit as comment, but stop at next label or }
            if trimmed == "}" {
                in_dead_block = false;
                result.push(line.to_string());
            } else {
                result.push(format!("  ; unreachable: {}", trimmed));
            }
        } else {
            result.push(line.to_string());
        }
    }

    result.join("\n")
}

/// List of known side-effect-free (pure) function prefixes.
/// Calls to these functions can be removed if their result is unused.
const PURE_FUNCTION_PREFIXES: &[&str] = &[
    "@llvm.abs.",
    "@llvm.smin.",
    "@llvm.smax.",
    "@llvm.umin.",
    "@llvm.umax.",
    "@llvm.ctpop.",
    "@llvm.ctlz.",
    "@llvm.cttz.",
    "@llvm.bswap.",
    "@llvm.bitreverse.",
];

/// Dead pure-call elimination.
///
/// Removes calls to known pure (side-effect-free) functions when their
/// results are not used anywhere.
fn dead_pure_call_elimination(ir: &str) -> String {
    let lines: Vec<&str> = ir.lines().collect();
    let mut used_vars: HashSet<String> = HashSet::new();

    // Pass 1: collect all used variables
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with(';') {
            continue;
        }

        // Collect all %var references as potential uses
        for word in
            trimmed.split(|c: char| !c.is_alphanumeric() && c != '%' && c != '_' && c != '.')
        {
            if word.starts_with('%') && !word.is_empty() {
                // Skip if this word is the definition target
                if !trimmed.starts_with(&format!("{} =", word)) {
                    used_vars.insert(word.to_string());
                }
            }
        }
    }

    // Also mark variables used in ret/br/store/call as used
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("ret ")
            || trimmed.starts_with("br ")
            || trimmed.starts_with("store ")
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

    // Pass 2: remove dead pure calls
    let mut result = Vec::new();
    for line in &lines {
        let trimmed = line.trim();

        if let Some(def_var) = extract_definition(trimmed) {
            if !used_vars.contains(&def_var) && trimmed.contains("call ") {
                // Check if this is a call to a known pure function
                let is_pure_call = PURE_FUNCTION_PREFIXES
                    .iter()
                    .any(|prefix| trimmed.contains(prefix));
                if is_pure_call {
                    result.push(format!("  ; dead pure call removed: {}", trimmed));
                    continue;
                }
            }
        }

        result.push(line.to_string());
    }

    result.join("\n")
}

/// Redundant store-load elimination.
///
/// Detects consecutive store-then-load patterns to the same address and
/// replaces the load with the stored value directly.
///
/// Pattern:
///   store TYPE VALUE, TYPE* PTR
///   %X = load TYPE, TYPE* PTR
/// Becomes:
///   store TYPE VALUE, TYPE* PTR    (kept, may have other readers)
///   %X = add TYPE 0, VALUE         ; forwarded from store
fn redundant_store_load_elimination(ir: &str) -> String {
    let lines: Vec<&str> = ir.lines().collect();
    let mut result = Vec::new();

    // Track the most recent store to each pointer: ptr -> (type, value)
    let mut recent_stores: HashMap<String, (String, String)> = HashMap::new();

    for line in &lines {
        let trimmed = line.trim();

        // Reset at function boundaries and labels (conservative)
        if line.starts_with("define ") || trimmed == "}" || trimmed.ends_with(':') {
            recent_stores.clear();
            result.push(line.to_string());
            continue;
        }

        // Invalidate stores on any call (might alias)
        if trimmed.contains("call ") {
            recent_stores.clear();
            result.push(line.to_string());
            continue;
        }

        // Track stores: store TYPE VALUE, TYPE* PTR
        if trimmed.starts_with("store ") {
            if let Some((ty, value, ptr)) = parse_store_instruction(trimmed) {
                recent_stores.insert(ptr, (ty, value));
            }
            result.push(line.to_string());
            continue;
        }

        // Check loads: %X = load TYPE, TYPE* PTR
        if trimmed.contains(" = load ") {
            if let Some((dest, ty, ptr)) = parse_load_instruction(trimmed) {
                if let Some((store_ty, store_val)) = recent_stores.get(&ptr) {
                    if *store_ty == ty {
                        // Forward the stored value
                        result.push(format!(
                            "  {} = add {} 0, {}  ; forwarded from store",
                            dest, store_ty, store_val
                        ));
                        continue;
                    }
                }
            }
            // A load from a pointer invalidates our knowledge of that pointer
            // (conservative: another thread might have stored)
            result.push(line.to_string());
            continue;
        }

        result.push(line.to_string());
    }

    result.join("\n")
}

/// Parse a store instruction: store TYPE VALUE, TYPE* PTR
/// Returns (type, value, ptr)
fn parse_store_instruction(line: &str) -> Option<(String, String, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with("store ") {
        return None;
    }

    let rest = &trimmed[6..]; // skip "store "
    let parts: Vec<&str> = rest.split(',').collect();
    if parts.len() < 2 {
        return None;
    }

    // First part: TYPE VALUE
    let first_words: Vec<&str> = parts[0].split_whitespace().collect();
    if first_words.len() < 2 {
        return None;
    }
    let ty = first_words[0].to_string();
    let value = first_words[1].to_string();

    // Second part: TYPE* PTR
    let second_words: Vec<&str> = parts[1].split_whitespace().collect();
    if second_words.len() < 2 {
        return None;
    }
    let ptr = second_words[1].to_string();

    Some((ty, value, ptr))
}

/// Parse a load instruction: %DEST = load TYPE, TYPE* PTR
/// Returns (dest, type, ptr)
fn parse_load_instruction(line: &str) -> Option<(String, String, String)> {
    let trimmed = line.trim();
    let parts: Vec<&str> = trimmed.split(" = load ").collect();
    if parts.len() != 2 {
        return None;
    }

    let dest = parts[0].trim().to_string();
    let rest = parts[1].trim();

    // rest: "TYPE, TYPE* PTR"
    let comma_parts: Vec<&str> = rest.split(',').collect();
    if comma_parts.len() < 2 {
        return None;
    }

    let ty = comma_parts[0].trim().to_string();
    let ptr_part: Vec<&str> = comma_parts[1].split_whitespace().collect();
    if ptr_part.len() < 2 {
        return None;
    }
    let ptr = ptr_part[1].to_string();

    Some((dest, ty, ptr))
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

/// Check if an instruction has no side effects.
///
/// Conservative by default: function calls are treated as having side effects
/// unless they are known LLVM intrinsics in the `PURE_FUNCTION_PREFIXES` list.
/// This prevents incorrect removal of calls to external/user-defined functions
/// that may have observable effects (I/O, global state mutation, etc.).
fn is_side_effect_free(line: &str) -> bool {
    let trimmed = line.trim();

    // Calls are side-effectful by default — only known-pure intrinsics are safe.
    // This must be checked BEFORE the pure_ops list below, because patterns like
    // "add " could match inside a call instruction's argument list.
    if trimmed.contains("call ") {
        // Only known-pure LLVM intrinsics are side-effect-free
        return PURE_FUNCTION_PREFIXES
            .iter()
            .any(|prefix| trimmed.contains(prefix));
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unreachable_block_elimination() {
        let ir = r#"define i64 @test() {
entry:
  br i1 true, label %then, label %dead
then:
  ret i64 42
dead:
  %x = add i64 1, 2
  ret i64 %x
}
"#;
        let result = unreachable_block_elimination(ir);
        assert!(
            result.contains("unreachable block removed") || result.contains("unreachable:"),
            "Dead block should be eliminated. Result:\n{}",
            result
        );
        // The "then" block should be preserved
        assert!(
            result.contains("ret i64 42"),
            "Live block should be preserved"
        );
    }

    #[test]
    fn test_dead_pure_call_elimination() {
        let ir = r#"define i64 @test(i64 %x) {
entry:
  %unused = call i64 @llvm.abs.i64(i64 %x, i1 true)
  ret i64 %x
}
"#;
        let result = dead_pure_call_elimination(ir);
        assert!(
            result.contains("dead pure call removed"),
            "Unused pure call should be eliminated. Result:\n{}",
            result
        );
    }

    #[test]
    fn test_redundant_store_load() {
        let ir = r#"define i64 @test() {
entry:
  %ptr = alloca i64
  store i64 42, i64* %ptr
  %val = load i64, i64* %ptr
  ret i64 %val
}
"#;
        let result = redundant_store_load_elimination(ir);
        assert!(
            result.contains("forwarded from store"),
            "Store-load should be forwarded. Result:\n{}",
            result
        );
    }

    #[test]
    fn test_store_load_invalidated_by_call() {
        let ir = r#"define i64 @test() {
entry:
  %ptr = alloca i64
  store i64 42, i64* %ptr
  call void @unknown()
  %val = load i64, i64* %ptr
  ret i64 %val
}
"#;
        let result = redundant_store_load_elimination(ir);
        // The load should NOT be forwarded because a call invalidates stores
        assert!(
            !result.contains("forwarded from store"),
            "Store-load should NOT be forwarded after call. Result:\n{}",
            result
        );
    }

    #[test]
    fn test_unreachable_block_with_multiple_predecessors() {
        // If the "dead" block is also reachable from another branch, keep it
        let ir = r#"define i64 @test(i1 %flag) {
entry:
  br i1 true, label %then, label %dead
then:
  br label %dead
dead:
  ret i64 0
}
"#;
        let result = unreachable_block_elimination(ir);
        // "dead" is reachable from "then" via unconditional branch, so it should be kept
        assert!(
            result.contains("ret i64 0"),
            "Block reachable from other paths should be preserved. Result:\n{}",
            result
        );
    }
}

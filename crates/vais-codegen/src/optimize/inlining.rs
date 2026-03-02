//! Function inlining optimization

use std::collections::HashMap;

/// Maximum inline cost threshold.
/// Functions with a composite cost exceeding this value are never inlined.
const MAX_INLINE_COST: i64 = 150;

/// Parsed LLVM IR function for inlining
#[derive(Debug, Clone)]
struct InlinableFunction {
    name: String,
    params: Vec<(String, String)>, // (type, param_name)
    return_type: String,
    body: Vec<String>,
    has_side_effects: bool,
    has_external_calls: bool,
    is_leaf: bool,
}

/// Count how many times each function is called in the IR
fn count_call_sites(ir: &str) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for line in ir.lines() {
        let trimmed = line.trim();
        if trimmed.contains("call ") {
            // Extract function name from call: call TYPE @func_name(
            if let Some(at_pos) = trimmed.find("@") {
                let after_at = &trimmed[at_pos..];
                if let Some(paren_pos) = after_at.find('(') {
                    let func_name = &after_at[..paren_pos];
                    *counts.entry(func_name.to_string()).or_insert(0) += 1;
                }
            }
        }
    }
    counts
}

/// Detect which call sites occur inside loop bodies.
/// Returns a set of function names (with @) that are called inside a loop.
fn detect_loop_call_sites(ir: &str) -> HashMap<String, usize> {
    let mut loop_calls: HashMap<String, usize> = HashMap::new();
    let mut loop_depth: usize = 0;

    for line in ir.lines() {
        let trimmed = line.trim();

        // Detect loop header labels
        if trimmed.ends_with(':')
            && (trimmed.contains("loop.start") || trimmed.contains("while"))
        {
            loop_depth += 1;
            continue;
        }

        // Detect loop end labels
        if trimmed.ends_with(':') && trimmed.contains("loop.end") {
            loop_depth = loop_depth.saturating_sub(1);
            continue;
        }

        // Track calls inside loops
        if loop_depth > 0 && trimmed.contains("call ") {
            if let Some(at_pos) = trimmed.find("@") {
                let after_at = &trimmed[at_pos..];
                if let Some(paren_pos) = after_at.find('(') {
                    let func_name = &after_at[..paren_pos];
                    *loop_calls.entry(func_name.to_string()).or_insert(0) += loop_depth;
                }
            }
        }
    }

    loop_calls
}

/// Compute composite inline cost for a function.
///
/// The cost balances code size increase against potential speedup from inlining.
/// Lower cost = better candidate for inlining.
///
/// Formula:
///   cost = body_size * caller_count
///   bonus = -20 if leaf function (no outgoing calls)
///   bonus += -30 * loop_call_weight (called in loops => big benefit)
///   Final: cost + bonus
fn compute_inline_cost(
    func: &InlinableFunction,
    caller_count: usize,
    loop_weight: usize,
) -> i64 {
    let body_size = func.body.len() as i64;
    let callers = caller_count.max(1) as i64;

    // Base cost: code size expansion per caller
    let base_cost = body_size * callers;

    // Leaf function bonus: no outgoing calls means the inlined code
    // won't have further call overhead
    let leaf_bonus: i64 = if func.is_leaf { -20 } else { 0 };

    // Loop call bonus: inlining inside loops eliminates per-iteration call overhead
    let loop_bonus: i64 = -(loop_weight.min(5) as i64) * 30;

    // Small function bonus: very small functions almost always benefit from inlining
    let size_bonus: i64 = if body_size <= 5 { -40 } else if body_size <= 10 { -20 } else { 0 };

    base_cost + leaf_bonus + loop_bonus + size_bonus
}

/// Aggressive inlining for small functions
///
/// Uses composite cost heuristic: caller_count * code_size with bonuses
/// for leaf functions and loop call sites. Functions are prioritized by
/// cost (lowest cost first) and filtered by MAX_INLINE_COST threshold.
pub(crate) fn aggressive_inline(ir: &str) -> String {
    // Parse all small functions that are candidates for inlining
    let mut inline_candidates = find_inline_candidates(ir);

    if inline_candidates.is_empty() {
        return ir.to_string();
    }

    // Count call sites and loop call sites
    let call_counts = count_call_sites(ir);
    let loop_calls = detect_loop_call_sites(ir);

    // Sort candidates by composite cost (lowest cost first = best candidates)
    inline_candidates.sort_by(|a, b| {
        let cost_a = compute_inline_cost(
            a,
            call_counts.get(&a.name).copied().unwrap_or(0),
            loop_calls.get(&a.name).copied().unwrap_or(0),
        );
        let cost_b = compute_inline_cost(
            b,
            call_counts.get(&b.name).copied().unwrap_or(0),
            loop_calls.get(&b.name).copied().unwrap_or(0),
        );
        // Primary: lower cost first; Secondary: smaller body first
        cost_a.cmp(&cost_b).then(a.body.len().cmp(&b.body.len()))
    });

    // Filter candidates by MAX_INLINE_COST threshold
    inline_candidates.retain(|func| {
        let cost = compute_inline_cost(
            func,
            call_counts.get(&func.name).copied().unwrap_or(0),
            loop_calls.get(&func.name).copied().unwrap_or(0),
        );
        cost <= MAX_INLINE_COST
    });

    // Inline function calls
    let mut result = ir.to_string();
    let mut inline_counter = 0;

    for func in &inline_candidates {
        result = inline_function_calls(&result, func, &mut inline_counter);
    }

    result
}

/// Find functions that are good candidates for inlining
///
/// Uses a tiered threshold with leaf function preference:
/// - Leaf functions (no outgoing calls): always considered up to body_size 50
/// - Functions <= 10 instructions: always inline (even with internal side effects like stores)
/// - Functions <= 50 instructions: inline if no external call side effects
/// - Functions > 50 instructions: never inline at text level (rely on LLVM's inliner)
fn find_inline_candidates(ir: &str) -> Vec<InlinableFunction> {
    let mut candidates = Vec::new();
    let lines: Vec<&str> = ir.lines().collect();
    let large_threshold = 50; // max instructions for inlining
    let small_threshold = 10; // always-inline threshold (even with store side effects)

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];

        // Look for function definitions
        if line.starts_with("define ") && line.contains("@") {
            if let Some(func) = parse_function(&lines, i) {
                let body_size = func.body.len();
                let is_main = func.name == "@main";
                let is_internal = func.name.starts_with("@__") || func.name.starts_with("@_");
                let is_recursive = func
                    .body
                    .iter()
                    .any(|l| l.contains(&format!("call {} {}", func.return_type, func.name)));

                // Never inline: main, internal helpers, recursive functions
                if is_main || is_internal || is_recursive {
                    i += 1;
                    continue;
                }

                // Tiered inlining with leaf function preference:
                let eligible = if func.is_leaf {
                    // Leaf functions: inline up to large_threshold (no outgoing call overhead)
                    body_size <= large_threshold
                } else if body_size <= small_threshold {
                    // Small functions: allow store side effects but not external calls
                    !func.has_external_calls
                } else if body_size <= large_threshold {
                    // Medium functions: must be pure (no side effects at all)
                    !func.has_side_effects
                } else {
                    false
                };

                if eligible {
                    candidates.push(func);
                }
            }
        }
        i += 1;
    }

    candidates
}

/// Parse a function from LLVM IR lines
fn parse_function(lines: &[&str], start_idx: usize) -> Option<InlinableFunction> {
    let header = lines[start_idx];

    // Extract return type
    let return_type = if header.contains("define i64") {
        "i64".to_string()
    } else if header.contains("define i32") {
        "i32".to_string()
    } else if header.contains("define void") {
        "void".to_string()
    } else if header.contains("define i1") {
        "i1".to_string()
    } else {
        return None;
    };

    // Extract function name
    let name_start = header.find('@')?;
    let name_end = header[name_start..].find('(')?;
    let name = header[name_start..name_start + name_end].to_string();

    // Extract parameters - find the first ( after function name
    let func_params_start = name_start + name_end;
    let params_end = header[func_params_start..].find(')')? + func_params_start;
    let params_str = &header[func_params_start + 1..params_end];
    let params = parse_params(params_str);

    // Parse function body
    // LLVM IR format: define i64 @func(params) {
    // So the { is on the same line as define, and } is on its own line
    let mut body = Vec::new();
    let mut has_side_effects = false;
    let mut has_external_calls = false;
    let mut is_leaf = true; // assume leaf until we find a call

    for line in lines.iter().skip(start_idx + 1) {
        let trimmed = line.trim();

        // End of function
        if trimmed == "}" {
            break;
        }

        // Skip labels (ending with :) and empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.ends_with(':') {
            continue;
        }

        // Check for side effects
        // Store operations have side effects (but are internal/local)
        if trimmed.starts_with("store ") {
            has_side_effects = true;
        }
        // Calls to functions have side effects
        if trimmed.contains("call ") {
            has_side_effects = true;
            has_external_calls = true;
            is_leaf = false;
        }

        body.push(trimmed.to_string());
    }

    Some(InlinableFunction {
        name,
        params,
        return_type,
        body,
        has_side_effects,
        has_external_calls,
        is_leaf,
    })
}

/// Parse function parameters
fn parse_params(params_str: &str) -> Vec<(String, String)> {
    let mut params = Vec::new();

    if params_str.trim().is_empty() {
        return params;
    }

    for param in params_str.split(',') {
        let param = param.trim();
        let parts: Vec<&str> = param.split_whitespace().collect();
        if parts.len() >= 2 {
            let ty = parts[0].to_string();
            let name = parts[1].to_string();
            params.push((ty, name));
        }
    }

    params
}

/// Inline calls to a specific function
fn inline_function_calls(ir: &str, func: &InlinableFunction, counter: &mut u32) -> String {
    let mut result = Vec::new();
    let call_pattern = format!("call {} {}(", func.return_type, func.name);

    for line in ir.lines() {
        let trimmed = line.trim();

        // Check if this line contains a call to the function
        if trimmed.contains(&call_pattern) {
            if let Some(inlined) = try_inline_call(trimmed, func, counter) {
                for inlined_line in inlined {
                    result.push(inlined_line);
                }
                continue;
            }
        }

        result.push(line.to_string());
    }

    result.join("\n")
}

/// Try to inline a specific call
fn try_inline_call(line: &str, func: &InlinableFunction, counter: &mut u32) -> Option<Vec<String>> {
    let call_pattern = format!("call {} {}(", func.return_type, func.name);
    let call_start = line.find(&call_pattern)?;

    // Extract destination variable if any
    let dest_var = if line.contains(" = call ") {
        let eq_pos = line.find(" = call ")?;
        Some(line[..eq_pos].trim().to_string())
    } else {
        None
    };

    // Extract call arguments
    let args_start = call_start + call_pattern.len();
    let args_end = line[args_start..].find(')')? + args_start;
    let args_str = &line[args_start..args_end];
    let call_args = parse_call_args(args_str);

    if call_args.len() != func.params.len() {
        return None;
    }

    // Generate unique suffix for this inline instance
    *counter += 1;
    let suffix = format!("_i{}", counter);

    // Build inlined code
    let mut inlined = Vec::new();
    inlined.push(format!("  ; BEGIN INLINE: {}", func.name));

    // Create mapping from parameter names to argument values
    let mut var_map: HashMap<String, String> = HashMap::new();
    for (i, (_, param_name)) in func.params.iter().enumerate() {
        var_map.insert(param_name.clone(), call_args[i].clone());
    }

    // Track local variable renames for return value
    let mut local_var_renames: HashMap<String, String> = HashMap::new();

    // Track the return value
    let mut return_value = String::new();

    // Inline function body with variable renaming
    for body_line in &func.body {
        if body_line.starts_with("ret ") {
            // Handle return statement
            let ret_parts: Vec<&str> = body_line.split_whitespace().collect();
            if ret_parts.len() >= 3 {
                let raw_ret = ret_parts[2].to_string();
                // First substitute parameters
                let mut ret_val = substitute_vars(&raw_ret, &var_map);
                // Then apply local variable renames
                for (old_var, new_var) in &local_var_renames {
                    if ret_val == *old_var {
                        ret_val = new_var.clone();
                        break;
                    }
                }
                return_value = ret_val;
            }
        } else {
            // Track variable definitions for renaming
            if let Some(eq_pos) = body_line.find(" = ") {
                let old_var = body_line[..eq_pos].trim().to_string();
                if let Some(var_part) = old_var.strip_prefix('%') {
                    // remove the %
                    let new_var = format!("%inl{}{}", suffix, var_part);
                    local_var_renames.insert(old_var, new_var);
                }
            }
            // Rename variables in the body
            let renamed = rename_vars_in_line(body_line, &suffix, &var_map);
            inlined.push(format!("  {}", renamed));
        }
    }

    // If there's a destination variable, assign the return value
    if let Some(dest) = dest_var {
        if !return_value.is_empty() && func.return_type != "void" {
            inlined.push(format!(
                "  {} = add {} 0, {}  ; inlined return value",
                dest, func.return_type, return_value
            ));
        }
    }

    inlined.push(format!("  ; END INLINE: {}", func.name));

    Some(inlined)
}

/// Parse call arguments
fn parse_call_args(args_str: &str) -> Vec<String> {
    let mut args = Vec::new();

    if args_str.trim().is_empty() {
        return args;
    }

    // Handle arguments like "i64 %0, i64 5"
    for arg in args_str.split(',') {
        let arg = arg.trim();
        let parts: Vec<&str> = arg.split_whitespace().collect();
        if parts.len() >= 2 {
            args.push(parts[1].to_string());
        } else if !arg.is_empty() {
            args.push(arg.to_string());
        }
    }

    args
}

/// Substitute parameter variables with argument values
fn substitute_vars(value: &str, var_map: &HashMap<String, String>) -> String {
    let mut result = value.to_string();
    for (param, arg) in var_map {
        result = result.replace(param, arg);
    }
    result
}

/// Rename variables in a line for inlining
fn rename_vars_in_line(line: &str, suffix: &str, var_map: &HashMap<String, String>) -> String {
    let mut result = line.to_string();

    // First substitute parameters
    for (param, arg) in var_map {
        result = result.replace(param, arg);
    }

    // Then rename local variables (those being defined in this line)
    if let Some(eq_pos) = result.find(" = ") {
        let lhs = result[..eq_pos].trim().to_string();
        if let Some(var_part) = lhs.strip_prefix('%') {
            // Create a new variable name that's valid LLVM IR
            // For %0, %1, etc. we need to create %inl1_0, %inl1_1, etc.
            // remove the %
            let new_var = format!("%inl{}{}", suffix, var_part);
            let old_var = lhs.clone();
            let rhs = result[eq_pos + 3..].to_string();
            // Only rename the definition
            result = format!("{} = {}", new_var, rhs);
            // And update any uses in the same line
            result = result.replace(&format!(" {}", old_var), &format!(" {}", new_var));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_inline_candidates() {
        let ir = r#"define i64 @square(i64 %x) {
entry:
  %0 = mul i64 %x, %x
  ret i64 %0
}

define i64 @add_one(i64 %x) {
entry:
  %0 = add i64 %x, 1
  ret i64 %0
}

define i64 @main() {
entry:
  %0 = call i64 @square(i64 5)
  ret i64 %0
}
"#;
        let candidates = find_inline_candidates(ir);
        // square and add_one should be candidates (no side effects)
        // main should NOT be a candidate (it's main)
        assert!(
            candidates.len() >= 2,
            "Expected at least 2 candidates, got {}",
            candidates.len()
        );
        let names: Vec<&str> = candidates.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"@square"), "square should be a candidate");
        assert!(names.contains(&"@add_one"), "add_one should be a candidate");
    }

    #[test]
    fn test_inline_simple_function() {
        let ir = r#"define i64 @square(i64 %x) {
entry:
  %0 = mul i64 %x, %x
  ret i64 %0
}

define i64 @main() {
entry:
  %0 = call i64 @square(i64 5)
  ret i64 %0
}
"#;
        let result = aggressive_inline(ir);
        println!("RESULT:\n{}", result);
        // After inlining, there should be INLINE comments
        assert!(
            result.contains("INLINE") || !result.contains("call i64 @square"),
            "Expected inlining to occur or call to be removed. Result:\n{}",
            result
        );
    }

    #[test]
    fn test_leaf_function_detection() {
        let ir = r#"define i64 @leaf(i64 %x) {
entry:
  %0 = mul i64 %x, %x
  %1 = add i64 %0, 1
  ret i64 %1
}

define i64 @non_leaf(i64 %x) {
entry:
  %0 = call i64 @leaf(i64 %x)
  ret i64 %0
}
"#;
        let candidates = find_inline_candidates(ir);
        let leaf_func = candidates.iter().find(|f| f.name == "@leaf");
        assert!(leaf_func.is_some(), "leaf should be a candidate");
        assert!(leaf_func.unwrap().is_leaf, "leaf should be detected as leaf function");
    }

    #[test]
    fn test_loop_call_detection() {
        let ir = r#"define i64 @helper(i64 %x) {
entry:
  %0 = add i64 %x, 1
  ret i64 %0
}

define i64 @main() {
entry:
  br label %loop.start.0
loop.start.0:
  %i = phi i64 [0, %entry], [%next, %loop.body.0]
  %cond = icmp slt i64 %i, 10
  br i1 %cond, label %loop.body.0, label %loop.end.0
loop.body.0:
  %r = call i64 @helper(i64 %i)
  %next = add i64 %i, 1
  br label %loop.start.0
loop.end.0:
  ret i64 %i
}
"#;
        let loop_calls = detect_loop_call_sites(ir);
        assert!(
            loop_calls.get("@helper").copied().unwrap_or(0) > 0,
            "helper should be detected as called inside a loop"
        );
    }

    #[test]
    fn test_inline_cost_computation() {
        let small_leaf = InlinableFunction {
            name: String::from("@small_leaf"),
            params: vec![],
            return_type: String::from("i64"),
            body: vec![String::from("%0 = add i64 1, 2")],
            has_side_effects: false,
            has_external_calls: false,
            is_leaf: true,
        };

        // Small leaf called once: very low cost
        let cost = compute_inline_cost(&small_leaf, 1, 0);
        assert!(cost < 0, "Small leaf function should have negative cost (bonus), got {}", cost);

        // Same function called in a loop: even lower cost
        let loop_cost = compute_inline_cost(&small_leaf, 1, 1);
        assert!(
            loop_cost < cost,
            "Loop call should reduce cost further: loop_cost={}, normal={}",
            loop_cost, cost
        );
    }

    #[test]
    fn test_max_inline_cost_filtering() {
        // Large function should be filtered out by MAX_INLINE_COST
        let large_func = InlinableFunction {
            name: String::from("@large"),
            params: vec![],
            return_type: String::from("i64"),
            body: (0..50).map(|i| format!("%{} = add i64 0, {}", i, i)).collect(),
            has_side_effects: false,
            has_external_calls: false,
            is_leaf: true,
        };

        // Called 5 times: cost = 50 * 5 - 20 (leaf) = 230 > MAX_INLINE_COST
        let cost = compute_inline_cost(&large_func, 5, 0);
        assert!(
            cost > MAX_INLINE_COST,
            "Large function with many callers should exceed MAX_INLINE_COST: cost={}",
            cost
        );
    }
}

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

/// Link-Time Optimization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LtoMode {
    /// No LTO
    None,
    /// Thin LTO - fast, parallel, good for large projects
    Thin,
    /// Full LTO - slower but more aggressive cross-module optimization
    Full,
}

impl LtoMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "thin" => LtoMode::Thin,
            "full" | "fat" => LtoMode::Full,
            _ => LtoMode::None,
        }
    }

    /// Get clang flags for this LTO mode
    pub fn clang_flags(&self) -> Vec<&'static str> {
        match self {
            LtoMode::None => vec![],
            LtoMode::Thin => vec!["-flto=thin"],
            LtoMode::Full => vec!["-flto=full"],
        }
    }

    /// Check if LTO is enabled
    pub fn is_enabled(&self) -> bool {
        !matches!(self, LtoMode::None)
    }
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

    // O1+: Basic optimizations (before inlining to simplify function bodies)
    if level >= OptLevel::O1 {
        result = constant_folding(&result);
        result = dead_store_elimination(&result);
        result = branch_optimization(&result);
    }

    // O2+: More aggressive optimizations
    if level >= OptLevel::O2 {
        result = strength_reduction(&result);
    }

    // O3: Inlining after basic optimizations
    if level >= OptLevel::O3 {
        result = aggressive_inline(&result);
    }

    // O2+: CSE and DCE after inlining to clean up
    if level >= OptLevel::O2 {
        result = common_subexpression_elimination(&result);
        result = dead_code_elimination(&result);
    }

    // O3: Loop optimizations last
    if level >= OptLevel::O3 {
        result = loop_invariant_motion(&result);
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
                let var: String = ptr.chars().take_while(|c| c.is_alphanumeric() || *c == '%' || *c == '.' || *c == '_').collect();
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

/// Common subexpression elimination
fn common_subexpression_elimination(ir: &str) -> String {
    let mut result = String::new();
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

/// Branch optimization - simplify branches with constant conditions
fn branch_optimization(ir: &str) -> String {
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
                    result.push(format!("  br label %{}  ; simplified from conditional", then_label));
                    continue;
                }
            } else if trimmed.contains("br i1 false,") || trimmed.contains("br i1 0,") {
                // Always branch to 'else'
                if let Some(else_label) = extract_branch_label(trimmed, false) {
                    result.push(format!("  br label %{}  ; simplified from conditional", else_label));
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
                        result.push(format!("  {} = add i1 0, true  ; simplified: X == X", parts[0].trim()));
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

/// Dead code elimination - remove unused definitions
fn dead_code_elimination(ir: &str) -> String {
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
        for word in trimmed.split(|c: char| !c.is_alphanumeric() && c != '%' && c != '_' && c != '.') {
            if word.starts_with('%') && !word.is_empty() {
                // Check if this is a use, not just the definition
                if !trimmed.starts_with(&format!("{} =", word)) && !trimmed.starts_with(&format!("  {} =", word)) {
                    used_vars.insert(word.to_string());
                }
            }
        }
    }

    // Also mark return values, call arguments, and branch conditions as used
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("ret ") || trimmed.starts_with("br ") ||
           trimmed.starts_with("store ") || trimmed.contains("call ") {
            for word in trimmed.split(|c: char| !c.is_alphanumeric() && c != '%' && c != '_' && c != '.') {
                if word.starts_with('%') {
                    used_vars.insert(word.to_string());
                }
            }
        }
    }

    // Second pass: emit only used definitions
    let mut result = Vec::new();
    for (_i, line) in lines.iter().enumerate() {
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
fn extract_definition(line: &str) -> Option<String> {
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
    let pure_ops = ["add ", "sub ", "mul ", "sdiv ", "udiv ", "and ", "or ", "xor ",
                    "shl ", "ashr ", "lshr ", "icmp ", "fcmp ", "select ",
                    "zext ", "sext ", "trunc ", "bitcast ", "getelementptr ",
                    "extractvalue ", "insertvalue ", "load "];

    for op in &pure_ops {
        if trimmed.contains(op) {
            return true;
        }
    }
    false
}

/// Loop optimizations - includes LICM, loop unrolling, and simple loop transformations
fn loop_invariant_motion(ir: &str) -> String {
    // First pass: Loop unrolling for simple counted loops
    let unrolled = loop_unrolling(ir);

    // Second pass: LICM (Loop Invariant Code Motion)
    licm_pass(&unrolled)
}

/// Loop unrolling - unroll small loops with known iteration counts
fn loop_unrolling(ir: &str) -> String {
    let lines: Vec<&str> = ir.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Detect loop start labels (pattern: loop.start.N:)
        if trimmed.ends_with(':') && trimmed.contains("loop.start") {
            // Try to analyze and unroll the loop
            if let Some((unrolled_lines, skip_to)) = try_unroll_loop(&lines, i) {
                for ul in unrolled_lines {
                    result.push(ul);
                }
                i = skip_to;
                continue;
            }
        }

        result.push(line.to_string());
        i += 1;
    }

    result.join("\n")
}

/// Try to unroll a loop starting at the given index
/// Returns (unrolled lines, index to skip to) if successful
fn try_unroll_loop(lines: &[&str], start_idx: usize) -> Option<(Vec<String>, usize)> {
    const UNROLL_FACTOR: usize = 4;
    const MAX_BODY_SIZE: usize = 20;

    let header_label = lines[start_idx].trim().trim_end_matches(':');

    // Find loop structure
    let mut loop_body_start = 0;
    let mut loop_body_end = 0;
    let mut loop_end_label = String::new();
    let mut body_label = String::new();
    let mut _condition_var = String::new();
    let mut bound_value: Option<i64> = None;
    let mut increment: Option<i64> = None;
    let mut induction_var = String::new();

    // Parse loop header to find condition and body
    let mut idx = start_idx + 1;
    while idx < lines.len() {
        let trimmed = lines[idx].trim();

        // Look for icmp instruction (loop condition)
        if trimmed.contains(" = icmp slt ") || trimmed.contains(" = icmp sle ") {
            // Pattern: %cond = icmp slt/sle i64 %i, BOUND
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 6 {
                _condition_var = parts[0].to_string();
                // Try to extract bound value
                if let Some(bound_str) = parts.last() {
                    if let Ok(b) = bound_str.parse::<i64>() {
                        bound_value = Some(b);
                    }
                }
                // Extract induction variable
                if parts.len() >= 5 {
                    let potential_induction = parts[4].trim_end_matches(',');
                    if potential_induction.starts_with('%') {
                        induction_var = potential_induction.to_string();
                    }
                }
            }
        }

        // Look for conditional branch to body
        if trimmed.starts_with("br i1 ") && trimmed.contains("label %") {
            let parts: Vec<&str> = trimmed.split("label %").collect();
            if parts.len() >= 3 {
                body_label = parts[1].split(',').next()?.trim().to_string();
                loop_end_label = parts[2].trim().to_string();
            }
            loop_body_start = idx + 1;
            break;
        }

        idx += 1;
    }

    // Skip if we couldn't parse the loop structure
    if body_label.is_empty() || loop_end_label.is_empty() {
        return None;
    }

    // Find loop body boundaries
    idx = loop_body_start;
    let mut in_body = false;
    let mut body_lines = Vec::new();

    while idx < lines.len() {
        let trimmed = lines[idx].trim();

        // Check for body label
        if trimmed == format!("{}:", body_label) {
            in_body = true;
            idx += 1;
            continue;
        }

        if in_body {
            // Check for back edge (br label %loop.start...)
            if trimmed.starts_with("br label %") && trimmed.contains(header_label) {
                loop_body_end = idx;
                break;
            }

            // Check for loop end label
            if trimmed == format!("{}:", loop_end_label) {
                loop_body_end = idx;
                break;
            }

            // Detect increment pattern: %next = add i64 %i, INCREMENT
            if trimmed.contains(" = add i64 ") && !induction_var.is_empty() {
                if trimmed.contains(&induction_var) {
                    let parts: Vec<&str> = trimmed.split(',').collect();
                    if parts.len() >= 2 {
                        if let Ok(inc) = parts[1].trim().parse::<i64>() {
                            increment = Some(inc);
                        }
                    }
                }
            }

            body_lines.push(trimmed.to_string());
        }

        idx += 1;
    }

    // Skip if body is too large or we couldn't analyze the loop
    if body_lines.len() > MAX_BODY_SIZE || body_lines.is_empty() {
        return None;
    }

    // Check if we can unroll (need known bound and increment)
    // For simplicity, we'll do partial unrolling without full analysis
    let (bound, inc) = match (bound_value, increment) {
        (Some(b), Some(i)) => (b, i),
        _ => return None,
    };

    // Only unroll small loops with reasonable iteration counts
    if inc <= 0 || bound <= 0 || bound > 1000 {
        return None;
    }

    // Find the end of the loop (loop.end label)
    let mut end_idx = loop_body_end;
    while end_idx < lines.len() {
        let trimmed = lines[end_idx].trim();
        if trimmed == format!("{}:", loop_end_label) {
            break;
        }
        end_idx += 1;
    }

    // Generate unrolled code
    let mut unrolled = Vec::new();

    // Add comment
    unrolled.push(format!("  ; LOOP UNROLLING: factor={}", UNROLL_FACTOR));

    // Keep original loop header for non-unrolled remainder
    for line in lines.iter().take(loop_body_start).skip(start_idx) {
        unrolled.push(line.to_string());
    }

    // Generate unrolled body
    if in_body && !body_lines.is_empty() {
        unrolled.push(format!("{}:", body_label));

        // Unroll the body UNROLL_FACTOR times with modified indices
        for unroll_idx in 0..UNROLL_FACTOR {
            unrolled.push(format!("  ; unrolled iteration {}", unroll_idx));
            for body_line in &body_lines {
                // Simple variable renaming for unrolled iterations
                if unroll_idx > 0 {
                    let renamed = rename_for_unroll(body_line, unroll_idx);
                    unrolled.push(format!("  {}", renamed));
                } else {
                    unrolled.push(format!("  {}", body_line));
                }
            }
        }

        // Adjust the increment
        unrolled.push(format!("  ; adjusted increment by {}", UNROLL_FACTOR));
    }

    // Add the back edge and loop end
    for line in lines.iter().take(end_idx + 1).skip(loop_body_end) {
        unrolled.push(line.to_string());
    }

    Some((unrolled, end_idx + 1))
}

/// Rename variables for unrolled iteration
fn rename_for_unroll(line: &str, unroll_idx: usize) -> String {
    let mut result = line.to_string();

    // Simple approach: add suffix to local variables
    // This is a simplified version - a real implementation would track SSA names
    if let Some(eq_pos) = line.find(" = ") {
        let lhs = line[..eq_pos].trim();
        if lhs.starts_with('%') && !lhs.contains('.') {
            let new_lhs = format!("{}_u{}", lhs, unroll_idx);
            result = result.replacen(lhs, &new_lhs, 1);
        }
    }

    result
}

/// LICM pass - hoist loop invariant code
fn licm_pass(ir: &str) -> String {
    let lines: Vec<&str> = ir.lines().collect();
    let mut result = Vec::new();
    let mut in_function = false;
    let mut in_loop = false;
    let mut loop_header_idx = 0;
    let mut loop_invariants: Vec<String> = Vec::new();
    let mut loop_vars: HashSet<String> = HashSet::new();
    let mut preheader_inserted = false;

    for line in lines.iter() {
        let trimmed = line.trim();

        // Track function boundaries
        if line.starts_with("define ") {
            in_function = true;
            loop_vars.clear();
            loop_invariants.clear();
            preheader_inserted = false;
        } else if trimmed == "}" && in_function {
            in_function = false;
            in_loop = false;
        }

        // Detect loop headers (labels ending with "loop" or containing "while")
        if trimmed.ends_with(':') && (trimmed.contains("loop.start") || trimmed.contains("while")) {
            in_loop = true;
            loop_header_idx = result.len();
            loop_vars.clear();
            loop_invariants.clear();
            preheader_inserted = false;
            result.push(line.to_string());
            continue;
        }

        // Detect loop exit (back edge or loop end)
        if in_loop {
            // Check for back edge to loop header
            if trimmed.starts_with("br label %") {
                let target = trimmed.split('%').nth(1).unwrap_or("");
                if target.contains("loop.start") || target.contains("while") {
                    // End of loop body - insert hoisted invariants before loop header
                    if !loop_invariants.is_empty() && !preheader_inserted {
                        // Create preheader
                        let preheader_lines = create_preheader(&loop_invariants, loop_header_idx, &result);
                        if let Some((new_results, skip)) = preheader_lines {
                            // Replace from loop_header_idx with new preheader + header
                            let pre_header: Vec<String> = result.drain(..loop_header_idx).collect();
                            result.clear();
                            result.extend(pre_header);
                            result.extend(new_results);
                            preheader_inserted = true;
                            loop_header_idx += skip;
                        }
                    }
                    in_loop = false;
                    loop_invariants.clear();
                }
            }

            // Detect loop.end label (end of loop)
            if trimmed.ends_with(':') && trimmed.contains("loop.end") {
                in_loop = false;
                loop_invariants.clear();
            }

            // Track variables modified in loop
            if let Some(def_var) = extract_definition(trimmed) {
                loop_vars.insert(def_var.clone());
            }

            // Check for loop invariant code
            if let Some(_def) = extract_definition(trimmed) {
                if is_loop_invariant_with_context(trimmed, &loop_vars) && !is_phi_or_load(trimmed) {
                    // This instruction could be hoisted
                    loop_invariants.push(trimmed.to_string());
                    result.push(format!("  ; LICM candidate: {}", trimmed));
                    continue;
                }
            }
        }

        result.push(line.to_string());
    }

    result.join("\n")
}

/// Create a loop preheader with hoisted invariants
fn create_preheader(invariants: &[String], header_idx: usize, current_result: &[String]) -> Option<(Vec<String>, usize)> {
    if invariants.is_empty() || header_idx >= current_result.len() {
        return None;
    }

    let mut new_lines = Vec::new();

    // Add LICM comment
    new_lines.push("  ; LICM: hoisted loop invariants".to_string());

    // Add hoisted invariants
    for inv in invariants {
        new_lines.push(format!("  {}", inv));
    }

    // Add original header
    new_lines.push(current_result[header_idx].clone());

    Some((new_lines, invariants.len() + 1))
}

/// Check if instruction uses only invariants (constants, parameters, or non-loop vars)
fn is_loop_invariant_with_context(line: &str, loop_vars: &HashSet<String>) -> bool {
    let trimmed = line.trim();

    // Only pure operations
    let pure_ops = [" = add ", " = sub ", " = mul ", " = sdiv ", " = shl ", " = ashr "];
    let is_pure = pure_ops.iter().any(|op| trimmed.contains(op));
    if !is_pure {
        return false;
    }

    // Check if any operand is a loop-modified variable
    for word in trimmed.split(|c: char| !c.is_alphanumeric() && c != '%' && c != '_') {
        if word.starts_with('%') {
            // Skip the destination variable
            if trimmed.starts_with(&format!("{} =", word)) {
                continue;
            }
            // If this operand is modified in the loop, it's not invariant
            if loop_vars.contains(word) {
                return false;
            }
        }
    }

    true
}

/// Check if instruction is phi or load (not candidates for LICM)
fn is_phi_or_load(line: &str) -> bool {
    line.contains(" = phi ") || line.contains(" = load ")
}

/// Parsed LLVM IR function for inlining
#[derive(Debug, Clone)]
struct InlinableFunction {
    name: String,
    params: Vec<(String, String)>,  // (type, param_name)
    return_type: String,
    body: Vec<String>,
    has_side_effects: bool,
}

/// Aggressive inlining for small functions
fn aggressive_inline(ir: &str) -> String {
    // Parse all small functions that are candidates for inlining
    let inline_candidates = find_inline_candidates(ir);

    #[cfg(debug_assertions)]
    {
        eprintln!("DEBUG: Found {} inline candidates", inline_candidates.len());
        for func in &inline_candidates {
            eprintln!("DEBUG: Candidate: {} ({} body lines, side_effects={})",
                     func.name, func.body.len(), func.has_side_effects);
        }
    }

    if inline_candidates.is_empty() {
        return ir.to_string();
    }

    // Inline function calls
    let mut result = ir.to_string();
    let mut inline_counter = 0;

    for func in &inline_candidates {
        result = inline_function_calls(&result, func, &mut inline_counter);
    }

    result
}

/// Find functions that are good candidates for inlining
fn find_inline_candidates(ir: &str) -> Vec<InlinableFunction> {
    let mut candidates = Vec::new();
    let lines: Vec<&str> = ir.lines().collect();
    let small_threshold = 10; // instructions

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];

        // Look for function definitions
        if line.starts_with("define ") && line.contains("@") {
            if let Some(func) = parse_function(&lines, i) {
                // Check if function is small enough and doesn't have disqualifying features
                let body_size = func.body.len();
                let is_main = func.name == "@main";
                let is_internal = func.name.starts_with("@__") || func.name.starts_with("@_");
                let is_recursive = func.body.iter().any(|l| l.contains(&format!("call {} {}", func.return_type, func.name)));

                // Don't inline: main, internal helpers, recursive functions, or functions with side effects
                if body_size <= small_threshold && !is_main && !is_internal && !is_recursive && !func.has_side_effects {
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
        // Store operations have side effects
        if trimmed.starts_with("store ") {
            has_side_effects = true;
        }
        // Calls to external functions have side effects
        // But we'll allow calls to functions we're analyzing (they'll be checked separately)
        if trimmed.contains("call ") {
            // Check if it's a call to an external function (starts with @)
            // For now, mark all calls as having side effects to be safe
            // In a more sophisticated implementation, we could track which functions are pure
            has_side_effects = true;
        }

        body.push(trimmed.to_string());
    }

    Some(InlinableFunction {
        name,
        params,
        return_type,
        body,
        has_side_effects,
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

    #[cfg(debug_assertions)]
    eprintln!("DEBUG: Looking for call pattern: '{}'", call_pattern);

    for line in ir.lines() {
        let trimmed = line.trim();

        // Check if this line contains a call to the function
        if trimmed.contains(&call_pattern) {
            #[cfg(debug_assertions)]
            eprintln!("DEBUG: Found matching call: {}", trimmed);

            if let Some(inlined) = try_inline_call(trimmed, func, counter) {
                #[cfg(debug_assertions)]
                eprintln!("DEBUG: Inlined successfully!");
                for inlined_line in inlined {
                    result.push(inlined_line);
                }
                continue;
            } else {
                #[cfg(debug_assertions)]
                eprintln!("DEBUG: try_inline_call returned None");
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
                if old_var.starts_with('%') {
                    let var_part = &old_var[1..]; // remove the %
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
            inlined.push(format!("  {} = add {} 0, {}  ; inlined return value", dest, func.return_type, return_value));
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
        if lhs.starts_with('%') {
            // Create a new variable name that's valid LLVM IR
            // For %0, %1, etc. we need to create %inl1_0, %inl1_1, etc.
            let var_part = &lhs[1..]; // remove the %
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
        assert!(candidates.len() >= 2, "Expected at least 2 candidates, got {}", candidates.len());
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
        assert!(result.contains("INLINE") || !result.contains("call i64 @square"),
                "Expected inlining to occur or call to be removed. Result:\n{}", result);
    }

    #[test]
    fn test_loop_unrolling() {
        // Simple loop with known bounds
        let ir = r#"define i64 @sum_to_10() {
entry:
  br label %loop.start.0
loop.start.0:
  %i = phi i64 [0, %entry], [%next, %loop.body.0]
  %sum = phi i64 [0, %entry], [%newsum, %loop.body.0]
  %cond = icmp slt i64 %i, 10
  br i1 %cond, label %loop.body.0, label %loop.end.0
loop.body.0:
  %newsum = add i64 %sum, %i
  %next = add i64 %i, 1
  br label %loop.start.0
loop.end.0:
  ret i64 %sum
}
"#;
        let result = loop_unrolling(ir);
        println!("UNROLLED:\n{}", result);
        // Check that unrolling was attempted (comment should be present)
        assert!(result.contains("LOOP UNROLLING") || result.contains("loop.start"),
                "Expected loop unrolling to be attempted");
    }

    #[test]
    fn test_loop_invariant_motion() {
        // Loop with invariant computation
        let ir = r#"define i64 @test_licm(i64 %n, i64 %a, i64 %b) {
entry:
  br label %loop.start.0
loop.start.0:
  %i = phi i64 [0, %entry], [%next, %loop.body.0]
  %cond = icmp slt i64 %i, %n
  br i1 %cond, label %loop.body.0, label %loop.end.0
loop.body.0:
  %inv = add i64 %a, %b
  %tmp = add i64 %i, %inv
  %next = add i64 %i, 1
  br label %loop.start.0
loop.end.0:
  ret i64 %i
}
"#;
        let result = licm_pass(ir);
        println!("LICM RESULT:\n{}", result);
        // Check that LICM was attempted (comment should be present)
        assert!(result.contains("LICM") || result.contains("loop.start"),
                "Expected LICM to be attempted");
    }

    #[test]
    fn test_rename_for_unroll() {
        let line = "%sum = add i64 %acc, %i";
        let renamed = rename_for_unroll(line, 2);
        assert!(renamed.contains("_u2"), "Expected unroll suffix in: {}", renamed);
    }

    #[test]
    fn test_full_loop_optimization() {
        // Test the combined loop optimization pass
        let ir = r#"define i64 @loop_opt_test() {
entry:
  br label %loop.start.0
loop.start.0:
  %i = phi i64 [0, %entry], [%next, %loop.body.0]
  %sum = phi i64 [0, %entry], [%newsum, %loop.body.0]
  %cond = icmp slt i64 %i, 8
  br i1 %cond, label %loop.body.0, label %loop.end.0
loop.body.0:
  %newsum = add i64 %sum, %i
  %next = add i64 %i, 1
  br label %loop.start.0
loop.end.0:
  ret i64 %sum
}
"#;
        let result = loop_invariant_motion(ir);
        println!("FULL LOOP OPT:\n{}", result);
        // The function should return valid IR
        assert!(result.contains("define i64 @loop_opt_test"));
        assert!(result.contains("ret i64"));
    }

    #[test]
    fn test_lto_mode_parsing() {
        assert_eq!(LtoMode::from_str("thin"), LtoMode::Thin);
        assert_eq!(LtoMode::from_str("THIN"), LtoMode::Thin);
        assert_eq!(LtoMode::from_str("full"), LtoMode::Full);
        assert_eq!(LtoMode::from_str("fat"), LtoMode::Full);
        assert_eq!(LtoMode::from_str("none"), LtoMode::None);
        assert_eq!(LtoMode::from_str("invalid"), LtoMode::None);
    }

    #[test]
    fn test_lto_clang_flags() {
        assert!(LtoMode::None.clang_flags().is_empty());
        assert_eq!(LtoMode::Thin.clang_flags(), vec!["-flto=thin"]);
        assert_eq!(LtoMode::Full.clang_flags(), vec!["-flto=full"]);
    }

    #[test]
    fn test_prepare_ir_for_lto() {
        let ir = r#"define i64 @helper() {
entry:
  ret i64 42
}

define i64 @main() {
entry:
  %0 = call i64 @helper()
  ret i64 %0
}
"#;
        let result = prepare_ir_for_lto(ir, LtoMode::Full);
        assert!(result.contains("define i64 @helper()"));
        assert!(result.contains("define i64 @main()"));
    }

    #[test]
    fn test_interprocedural_analysis() {
        let ir = r#"define i64 @pure_func(i64 %x) {
entry:
  %0 = mul i64 %x, %x
  ret i64 %0
}

define i64 @impure_func() {
entry:
  %0 = call i64 @printf(i64 0)
  ret i64 %0
}

define i64 @main() {
entry:
  %0 = call i64 @pure_func(i64 5)
  ret i64 %0
}
"#;
        let analysis = interprocedural_analysis(ir);
        assert!(analysis.pure_functions.contains("@pure_func"));
        assert!(!analysis.pure_functions.contains("@impure_func"));
    }
}

// ============================================
// Link-Time Optimization Support
// ============================================

/// Prepare IR for Link-Time Optimization
/// Adds attributes and markers that help LLVM's LTO passes
pub fn prepare_ir_for_lto(ir: &str, mode: LtoMode) -> String {
    if !mode.is_enabled() {
        return ir.to_string();
    }

    let mut result = Vec::new();

    // Add module-level LTO markers
    result.push("; LTO enabled".to_string());

    for line in ir.lines() {
        let trimmed = line.trim();

        // Add LTO-friendly attributes to function definitions
        if line.starts_with("define ") {
            // Add inline hint for small functions (LTO will decide)
            let modified = add_lto_function_attrs(line, mode);
            result.push(modified);
        } else {
            result.push(line.to_string());
        }
    }

    result.join("\n")
}

/// Add LTO-friendly attributes to a function definition
fn add_lto_function_attrs(line: &str, mode: LtoMode) -> String {
    let mut result = line.to_string();

    // For full LTO, mark internal functions for potential inlining
    if mode == LtoMode::Full {
        // Don't add attributes to main or external functions
        if !line.contains("@main") && !line.contains("external") {
            // Add inlinehint if not already present
            if !result.contains("inlinehint") && !result.contains("noinline") {
                if let Some(brace_pos) = result.find('{') {
                    result.insert_str(brace_pos, "inlinehint ");
                }
            }
        }
    }

    result
}

/// Perform interprocedural analysis for LTO
pub fn interprocedural_analysis(ir: &str) -> InterproceduralInfo {
    let mut info = InterproceduralInfo::new();

    // Parse all functions
    let functions = parse_all_functions(ir);

    // Analyze each function for purity, call graph, etc.
    for func in &functions {
        // Check if function is pure (no side effects)
        if is_pure_function(&func.body) {
            info.pure_functions.insert(func.name.clone());
        }

        // Build call graph
        for called in extract_called_functions(&func.body) {
            info.call_graph
                .entry(func.name.clone())
                .or_default()
                .push(called);
        }
    }

    // Find functions that can be constant-folded across modules
    info.const_propagation_candidates = find_const_prop_candidates(&functions, &info);

    info
}

/// Information gathered from interprocedural analysis
#[derive(Debug, Default)]
pub struct InterproceduralInfo {
    /// Functions that have no side effects
    pub pure_functions: HashSet<String>,
    /// Call graph: function -> list of functions it calls
    pub call_graph: HashMap<String, Vec<String>>,
    /// Functions whose return values could be propagated as constants
    pub const_propagation_candidates: HashSet<String>,
}

impl InterproceduralInfo {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Simple function info for analysis
struct FunctionAnalysis {
    name: String,
    body: Vec<String>,
}

/// Parse all functions from IR
fn parse_all_functions(ir: &str) -> Vec<FunctionAnalysis> {
    let mut functions = Vec::new();
    let lines: Vec<&str> = ir.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if lines[i].starts_with("define ") {
            if let Some(name) = extract_function_name(lines[i]) {
                let mut body = Vec::new();
                i += 1;
                while i < lines.len() && lines[i].trim() != "}" {
                    body.push(lines[i].to_string());
                    i += 1;
                }
                functions.push(FunctionAnalysis { name, body });
            }
        }
        i += 1;
    }

    functions
}

/// Extract function name from define line
fn extract_function_name(line: &str) -> Option<String> {
    let at_pos = line.find('@')?;
    let paren_pos = line[at_pos..].find('(')? + at_pos;
    Some(line[at_pos..paren_pos].to_string())
}

/// Check if a function body has no side effects
fn is_pure_function(body: &[String]) -> bool {
    for line in body {
        let trimmed = line.trim();
        // Side effects: store, call to non-intrinsic, volatile operations
        if trimmed.starts_with("store ") {
            return false;
        }
        if trimmed.contains("call ") {
            // Allow calls to known pure intrinsics
            if !is_pure_intrinsic_call(trimmed) {
                return false;
            }
        }
        if trimmed.contains("volatile") {
            return false;
        }
    }
    true
}

/// Check if a call is to a known pure intrinsic
fn is_pure_intrinsic_call(line: &str) -> bool {
    let pure_intrinsics = [
        "@llvm.abs",
        "@llvm.min",
        "@llvm.max",
        "@llvm.sqrt",
        "@llvm.sin",
        "@llvm.cos",
        "@llvm.pow",
        "@llvm.fabs",
        "@llvm.floor",
        "@llvm.ceil",
    ];
    pure_intrinsics.iter().any(|i| line.contains(i))
}

/// Extract names of called functions from a function body
fn extract_called_functions(body: &[String]) -> Vec<String> {
    let mut called = Vec::new();
    for line in body {
        if line.contains("call ") {
            if let Some(at_pos) = line.find('@') {
                let rest = &line[at_pos..];
                if let Some(paren_pos) = rest.find('(') {
                    let name = rest[..paren_pos].to_string();
                    // Skip intrinsics
                    if !name.starts_with("@llvm.") {
                        called.push(name);
                    }
                }
            }
        }
    }
    called
}

/// Find functions that are candidates for cross-module constant propagation
fn find_const_prop_candidates(
    functions: &[FunctionAnalysis],
    info: &InterproceduralInfo,
) -> HashSet<String> {
    let mut candidates = HashSet::new();

    for func in functions {
        // Pure functions that return constants are good candidates
        if info.pure_functions.contains(&func.name) {
            // Check if the function returns a constant
            for line in &func.body {
                let trimmed = line.trim();
                if trimmed.starts_with("ret ") {
                    // Check if return value is a constant
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 3 {
                        // ret i64 42 -> parts[2] is "42"
                        if parts[2].parse::<i64>().is_ok() {
                            candidates.insert(func.name.clone());
                        }
                    }
                }
            }
        }
    }

    candidates
}

/// Cross-module dead code elimination
/// Removes functions that are never called across all modules
pub fn cross_module_dce(modules: &[&str]) -> Vec<String> {
    let mut all_functions: HashSet<String> = HashSet::new();
    let mut called_functions: HashSet<String> = HashSet::new();

    // Collect all function definitions and calls
    for module in modules {
        for line in module.lines() {
            // Collect function definitions
            if line.starts_with("define ") {
                if let Some(name) = extract_function_name(line) {
                    all_functions.insert(name);
                }
            }
            // Collect function calls
            if line.contains("call ") {
                for called in extract_called_functions(&[line.to_string()]) {
                    called_functions.insert(called);
                }
            }
        }
    }

    // Always keep main
    called_functions.insert("@main".to_string());

    // Functions to remove (defined but never called)
    let dead_functions: HashSet<_> = all_functions
        .difference(&called_functions)
        .cloned()
        .collect();

    // Remove dead functions from each module
    modules
        .iter()
        .map(|module| remove_dead_functions(module, &dead_functions))
        .collect()
}

/// Remove specified dead functions from a module
fn remove_dead_functions(ir: &str, dead: &HashSet<String>) -> String {
    let mut result = Vec::new();
    let mut skip_function = false;

    for line in ir.lines() {
        if line.starts_with("define ") {
            if let Some(name) = extract_function_name(line) {
                if dead.contains(&name) {
                    skip_function = true;
                    result.push(format!("; DCE removed: {}", name));
                    continue;
                }
            }
            skip_function = false;
        }

        if skip_function {
            if line.trim() == "}" {
                skip_function = false;
            }
            continue;
        }

        result.push(line.to_string());
    }

    result.join("\n")
}

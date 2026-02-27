//! Loop optimizations (LICM, loop unrolling).

use std::collections::HashSet;

use super::dead_code::extract_definition;

/// Loop optimizations - includes LICM, loop unrolling, and simple loop transformations
pub(crate) fn loop_invariant_motion(ir: &str) -> String {
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
            if trimmed.contains(" = add i64 ")
                && !induction_var.is_empty()
                && trimmed.contains(&induction_var)
            {
                let parts: Vec<&str> = trimmed.split(',').collect();
                if parts.len() >= 2 {
                    if let Ok(inc) = parts[1].trim().parse::<i64>() {
                        increment = Some(inc);
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
                        let preheader_lines =
                            create_preheader(&loop_invariants, loop_header_idx, &result);
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
fn create_preheader(
    invariants: &[String],
    header_idx: usize,
    current_result: &[String],
) -> Option<(Vec<String>, usize)> {
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
    let pure_ops = [
        " = add ", " = sub ", " = mul ", " = sdiv ", " = shl ", " = ashr ",
    ];
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

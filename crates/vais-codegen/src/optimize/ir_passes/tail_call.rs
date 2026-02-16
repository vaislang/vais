//! Tail call optimization.

use super::helpers::extract_function_name;

/// Tail call optimization - mark tail calls with 'tail' keyword.
/// Detects patterns where a call result is immediately returned:
///   %result = call ... @func_name(...)
///   ret ... %result
/// And marks the call with 'tail' for LLVM to optimize.
pub(crate) fn tail_call_optimization(ir: &str) -> String {
    let lines: Vec<&str> = ir.lines().collect();
    let mut result = Vec::new();
    let mut current_fn_name: Option<String> = None;

    for i in 0..lines.len() {
        let trimmed = lines[i].trim();

        // Track current function name
        if trimmed.starts_with("define ") {
            current_fn_name = extract_function_name(trimmed);
        } else if trimmed == "}" {
            current_fn_name = None;
        }

        // Look for: %x = call TYPE @func(...) followed by ret TYPE %x
        if trimmed.contains(" = call ")
            && !trimmed.starts_with("tail ")
            && !trimmed.starts_with("musttail ")
        {
            // Check if the next non-empty line is a ret with the same value
            if let Some(next_i) = (i + 1..lines.len()).find(|&j| !lines[j].trim().is_empty()) {
                let next_trimmed = lines[next_i].trim();
                // Extract the destination variable from the call
                if let Some(dest) = trimmed.split(" = call ").next() {
                    let dest = dest.trim();
                    // Check if next line is "ret TYPE %dest"
                    if next_trimmed.starts_with("ret ") && next_trimmed.contains(dest) {
                        // Check if this is a self-recursive call (calls the current function)
                        let is_self_call = current_fn_name
                            .as_ref()
                            .is_some_and(|fn_name| trimmed.contains(&format!("@{}(", fn_name)));

                        // Mark as tail call
                        let prefix = if is_self_call { "musttail" } else { "tail" };
                        // safe: checked at line 603 that trimmed contains " = call "
                        let call_pos = trimmed.find(" = call ").unwrap();
                        let dest_part = &trimmed[..call_pos];
                        let call_part = &trimmed[call_pos + 3..]; // " = call ..."
                        result.push(format!(
                            "  {} = {} {}",
                            dest_part.trim(),
                            prefix,
                            call_part.trim()
                        ));
                        continue;
                    }
                }
            }
        }

        result.push(lines[i].to_string());
    }

    result.join("\n")
}

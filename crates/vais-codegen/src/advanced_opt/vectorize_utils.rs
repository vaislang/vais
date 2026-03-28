//! Utility functions for auto-vectorization analysis
//!
//! This module contains free helper functions used by the auto-vectorization
//! pass: side-effect detection, trip count estimation, memory access parsing,
//! GEP index extraction, dependence distance computation, and LLVM loop
//! metadata string generation.

use super::*;

// ── Side-effect detection ────────────────────────────────────────────────────

/// Check if a function call has side effects (non-pure)
pub(super) fn has_side_effects(line: &str) -> bool {
    // List of known pure/safe functions that don't prevent vectorization
    let pure_functions = [
        "@llvm.sqrt",
        "@llvm.fabs",
        "@llvm.sin",
        "@llvm.cos",
        "@llvm.exp",
        "@llvm.exp2",
        "@llvm.log",
        "@llvm.log2",
        "@llvm.log10",
        "@llvm.pow",
        "@llvm.fma",
        "@llvm.floor",
        "@llvm.ceil",
        "@llvm.round",
        "@llvm.trunc",
        "@llvm.copysign",
        "@llvm.minnum",
        "@llvm.maxnum",
        "@llvm.minimum",
        "@llvm.maximum",
        "@llvm.abs",
        "@llvm.smin",
        "@llvm.smax",
        "@llvm.umin",
        "@llvm.umax",
        "@llvm.ctpop",
        "@llvm.ctlz",
        "@llvm.cttz",
        "@llvm.sadd.with.overflow",
        "@llvm.uadd.with.overflow",
        "@llvm.ssub.with.overflow",
        "@llvm.usub.with.overflow",
        "@llvm.smul.with.overflow",
        "@llvm.umul.with.overflow",
        "@llvm.sadd.sat",
        "@llvm.uadd.sat",
        "@llvm.ssub.sat",
        "@llvm.usub.sat",
        "@llvm.bswap",
        "@llvm.bitreverse",
        // Debug intrinsics are also safe
        "@llvm.dbg.declare",
        "@llvm.dbg.value",
        "@llvm.dbg.label",
        "@llvm.lifetime.start",
        "@llvm.lifetime.end",
        "@llvm.assume",
        "@llvm.expect",
    ];

    // Extract the function name from the call
    if let Some(at_pos) = line.find('@') {
        let func_start = &line[at_pos..];
        // Check if the called function is in the pure list
        for pure_fn in &pure_functions {
            if func_start.starts_with(pure_fn) {
                return false;
            }
        }
        // Any other function call is assumed to have side effects
        return true;
    }

    // Indirect call (call via function pointer) - assume side effects
    true
}

// ── Trip count detection ─────────────────────────────────────────────────────

/// Detect trip count from loop structure in the IR
pub(super) fn detect_trip_count(ir: &str, header_label: &str) -> Option<u64> {
    let mut in_loop = false;
    let mut bound_value: Option<i64> = None;
    let mut init_value: Option<i64> = None;

    for line in ir.lines() {
        let trimmed = line.trim();

        if trimmed == format!("{}:", header_label) {
            in_loop = true;
            continue;
        }

        if !in_loop {
            continue;
        }

        // Look for PHI node to find initial value
        // Pattern: %i = phi i64 [0, %entry], [%i.next, %loop]
        if trimmed.contains(" = phi ") {
            // Try to extract the initial constant value
            if let Some(bracket_pos) = trimmed.find('[') {
                let after_bracket = &trimmed[bracket_pos + 1..];
                if let Some(comma_pos) = after_bracket.find(',') {
                    let init_str = after_bracket[..comma_pos].trim();
                    if let Ok(val) = init_str.parse::<i64>() {
                        init_value = Some(val);
                    }
                }
            }
        }

        // Look for comparison that controls the loop exit
        // Pattern: %cond = icmp slt i64 %i.next, 100
        // Pattern: %cond = icmp ult i64 %i.next, %n
        if trimmed.contains(" = icmp ")
            && (trimmed.contains("slt")
                || trimmed.contains("ult")
                || trimmed.contains("sle")
                || trimmed.contains("ule")
                || trimmed.contains("ne")
                || trimmed.contains("sgt")
                || trimmed.contains("ugt"))
        {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            // The bound is typically the last operand
            if let Some(last) = parts.last() {
                let bound_str = last.trim_end_matches(')');
                if let Ok(val) = bound_str.parse::<i64>() {
                    bound_value = Some(val);
                }
            }
        }

        // Stop at end of loop
        if trimmed.ends_with(':') && !trimmed.starts_with(header_label) {
            // Check if this is still part of the loop body
            if !trimmed.starts_with("loop") {
                break;
            }
        }
    }

    // Calculate trip count from init and bound
    match (init_value, bound_value) {
        (Some(init), Some(bound)) if bound > init => Some((bound - init) as u64),
        (None, Some(bound)) if bound > 0 => {
            // Assume init is 0 if not found
            Some(bound as u64)
        }
        _ => None,
    }
}

// ── IR parsing helpers ───────────────────────────────────────────────────────

pub(super) fn extract_func_name(line: &str) -> Option<String> {
    let at_pos = line.find('@')?;
    let paren_pos = line[at_pos..].find('(')?;
    Some(line[at_pos + 1..at_pos + paren_pos].to_string())
}

pub(super) fn extract_branch_targets(line: &str) -> Vec<String> {
    let mut targets = Vec::new();
    for part in line.split("label %") {
        if part.contains("br ") || part.is_empty() {
            continue;
        }
        let target = part
            .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
            .next()
            .unwrap_or("");
        if !target.is_empty() {
            targets.push(target.to_string());
        }
    }
    targets
}

pub(super) fn parse_memory_access(line: &str, is_write: bool) -> Option<MemoryAccess> {
    // Extract base pointer and index from load/store instructions
    let base = if is_write {
        // store TYPE VALUE, TYPE* BASE
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            parts[1].split_whitespace().last()?.to_string()
        } else {
            return None;
        }
    } else {
        // %x = load TYPE, TYPE* BASE
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            parts[1].split_whitespace().last()?.to_string()
        } else {
            return None;
        }
    };

    // Determine element size from LLVM IR type annotations
    let element_size = detect_element_size(line);

    // Try to extract index from GEP-based pointer
    let (index, stride) = extract_gep_info(line, &base, element_size);

    Some(MemoryAccess {
        instruction: line.to_string(),
        base,
        index,
        stride,
        is_write,
        element_size,
    })
}

/// Detect element size from LLVM IR type string
pub(super) fn detect_element_size(line: &str) -> usize {
    // Check for explicit types in order of specificity
    if line.contains("i128") {
        16
    } else if line.contains("double") || line.contains("i64") {
        8
    } else if line.contains("float") || line.contains("i32") {
        4
    } else if line.contains("i16") {
        2
    } else if line.contains("i8") {
        1
    } else {
        // Default: assume pointer-sized (8 bytes on 64-bit)
        8
    }
}

/// Extract GEP index and stride information from a memory access
pub(super) fn extract_gep_info(
    line: &str,
    _base: &str,
    element_size: usize,
) -> (Option<String>, Option<i64>) {
    // Look for getelementptr pattern in the instruction or referenced pointer
    // Pattern: getelementptr TYPE, TYPE* BASE, i64 INDEX
    if let Some(gep_pos) = line.find("getelementptr") {
        let gep_str = &line[gep_pos..];
        // Extract the last operand as the index
        let parts: Vec<&str> = gep_str.split(',').collect();
        if let Some(last_part) = parts.last() {
            let trimmed = last_part.trim();
            // Parse "i64 %i" or "i64 3" etc.
            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            if tokens.len() >= 2 {
                let index_str = tokens[tokens.len() - 1].trim_end_matches(')');
                let index = Some(index_str.to_string());

                // Determine stride: for simple induction variable access (e.g., a[i]),
                // stride is 1 element. For constant indices, stride is 0 (invariant).
                let stride = if index_str.starts_with('%') {
                    // Variable index - assume unit stride (1 element per iteration)
                    Some(1i64)
                } else if let Ok(_val) = index_str.parse::<i64>() {
                    // Constant index - loop invariant access, stride 0
                    Some(0i64)
                } else {
                    // Unknown
                    Some(1i64)
                };

                return (index, stride);
            }
        }
        // GEP found but couldn't parse - assume unit stride
        return (None, Some(1));
    }

    // No GEP in this line - the pointer was computed elsewhere
    // Check if the base looks like a GEP result (starts with %)
    // Assume unit stride as default for array accesses
    let _ = element_size;
    (None, Some(1))
}

pub(super) fn extract_phi_var(line: &str) -> Option<String> {
    let eq_pos = line.find(" = ")?;
    let var = line[..eq_pos].trim();
    if var.starts_with('%') {
        Some(var.to_string())
    } else {
        None
    }
}

// ── Dependence analysis helpers ──────────────────────────────────────────────

pub(super) fn analyze_access_pair(
    a1: &MemoryAccess,
    a2: &MemoryAccess,
    _alias_analysis: Option<&AliasAnalysis>,
) -> LoopDependence {
    // Read-read is never a dependence
    if !a1.is_write && !a2.is_write {
        return LoopDependence::None;
    }

    // Calculate dependence distance from index expressions
    let distance = compute_dependence_distance(a1, a2);

    // Determine dependence type based on read/write patterns
    match (a1.is_write, a2.is_write) {
        (false, false) => LoopDependence::None,
        (true, false) => LoopDependence::Flow { distance },
        (false, true) => LoopDependence::Anti { distance },
        (true, true) => LoopDependence::Output { distance },
    }
}

/// Compute the dependence distance between two memory accesses
pub(super) fn compute_dependence_distance(a1: &MemoryAccess, a2: &MemoryAccess) -> Option<i64> {
    // If both accesses have constant indices, compute exact distance
    if let (Some(idx1), Some(idx2)) = (&a1.index, &a2.index) {
        if let (Ok(i1), Ok(i2)) = (idx1.parse::<i64>(), idx2.parse::<i64>()) {
            return Some(i2 - i1);
        }
        // If both use the same variable index (e.g., %i), distance is 0
        // (same iteration access)
        if idx1 == idx2 {
            return Some(0);
        }
        // Check for patterns like %i and %i.next (distance = 1)
        if idx2.starts_with(idx1.as_str()) && idx2.contains(".next") {
            return Some(1);
        }
        if idx1.starts_with(idx2.as_str()) && idx1.contains(".next") {
            return Some(-1);
        }
    }

    // If both have stride information and same base, and one is shifted
    if a1.stride == Some(0) || a2.stride == Some(0) {
        // One is loop-invariant, the other varies - no loop-carried dependence
        // within a single iteration pair, but could alias
        return None;
    }

    // Cannot determine distance statically
    None
}

// ── LLVM metadata generation ─────────────────────────────────────────────────

pub(super) fn generate_loop_metadata(
    loop_id: u32,
    candidate: &VectorizationCandidate,
    target_width: &VectorWidth,
) -> String {
    let width = candidate
        .recommended_width
        .unwrap_or(target_width.lanes(64));

    let mut md = format!(
        "!{} = distinct !{{!\"llvm.loop.vectorize.enable\", i1 true}}\n",
        loop_id
    );

    md.push_str(&format!(
        "!{} = !{{!\"llvm.loop.vectorize.width\", i32 {}}}\n",
        loop_id + 1000,
        width
    ));

    // Add interleave hint for better performance
    md.push_str(&format!(
        "!{} = !{{!\"llvm.loop.interleave.count\", i32 2}}\n",
        loop_id + 2000
    ));

    // Add unroll hint
    md.push_str(&format!(
        "!{} = !{{!\"llvm.loop.unroll.count\", i32 {}}}\n",
        loop_id + 3000,
        std::cmp::min(width, 8)
    ));

    md
}

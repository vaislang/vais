//! Bounds Check Elimination via Range Analysis
//!
//! This module implements range analysis to prove array accesses are safe,
//! allowing removal of unnecessary bounds checks.
//!
//! # Key techniques
//!
//! 1. **Loop induction variable analysis**: Determine that `i` in `L i := 0; i < n; i += 1`
//!    is always in range `[0, n)`.
//! 2. **Guard-based elimination**: If `I i < arr.len` dominates `arr[i]`, the check is proven.
//! 3. **Constant index elimination**: `arr[0]` on a known-length array needs no check.

use std::collections::HashMap;

/// A value range: lower bound (inclusive) .. upper bound (exclusive).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueRange {
    /// Minimum value (inclusive), None = unbounded below
    pub lo: Option<i64>,
    /// Maximum value (exclusive), None = unbounded above
    pub hi: Option<i64>,
    /// Symbolic upper bound (e.g. "%arr.len")
    pub hi_sym: Option<String>,
}

impl ValueRange {
    pub fn constant(v: i64) -> Self {
        Self {
            lo: Some(v),
            hi: Some(v + 1),
            hi_sym: None,
        }
    }

    pub fn bounded(lo: i64, hi_exclusive: i64) -> Self {
        Self {
            lo: Some(lo),
            hi: Some(hi_exclusive),
            hi_sym: None,
        }
    }

    pub fn bounded_sym(lo: i64, sym: String) -> Self {
        Self {
            lo: Some(lo),
            hi: None,
            hi_sym: Some(sym),
        }
    }

    /// True if this range is entirely within [0, length).
    pub fn is_safe_for_length(&self, length: &str) -> bool {
        // Lower bound must be >= 0
        let lo_ok = self.lo.is_some_and(|lo| lo >= 0);
        if !lo_ok {
            return false;
        }
        // Upper bound must be < length (symbolically)
        if let Some(ref sym) = self.hi_sym {
            return sym == length;
        }
        false
    }

    /// True if this is a known constant range fully below a concrete length.
    pub fn is_safe_for_const_length(&self, length: i64) -> bool {
        match (self.lo, self.hi) {
            (Some(lo), Some(hi)) => lo >= 0 && hi <= length,
            _ => false,
        }
    }
}

/// Information about a bounds check in the IR.
#[derive(Debug, Clone)]
pub struct BoundsCheck {
    /// The SSA variable being checked (index)
    pub index_var: String,
    /// The length variable or constant
    pub length: String,
    /// Line number in the IR (for replacement)
    pub line_idx: usize,
    /// Is this an explicit `icmp ult` + `br` pattern?
    pub is_explicit: bool,
}

/// Range analysis result for a function.
#[derive(Debug, Default)]
pub struct RangeAnalysis {
    /// Proven ranges for SSA variables
    pub ranges: HashMap<String, ValueRange>,
    /// Bounds checks that can be eliminated
    pub eliminable: Vec<BoundsCheck>,
}

/// Analyze LLVM IR for bounds check elimination opportunities.
///
/// Detects patterns:
/// 1. Loop induction: `%i = phi [0, ...], [%i.next, ...]` with `icmp slt %i, %n`
/// 2. Guard: `icmp ult %idx, %len` followed by `br` → proven safe in the true branch
/// 3. Constant index into known-length array
pub fn analyze_bounds_checks(ir: &str) -> RangeAnalysis {
    let mut analysis = RangeAnalysis::default();
    let lines: Vec<&str> = ir.lines().collect();

    // Pass 1: Find loop induction variables and their ranges
    analyze_induction_variables(&lines, &mut analysis);

    // Pass 2: Find guard comparisons that prove ranges
    analyze_guards(&lines, &mut analysis);

    // Pass 3: Find constant array accesses
    analyze_constant_accesses(&lines, &mut analysis);

    analysis
}

/// Apply bounds check elimination to LLVM IR.
///
/// For each eliminable check, replaces the conditional branch with an
/// unconditional branch to the safe path (the "true" target).
pub fn eliminate_bounds_checks(ir: &str) -> String {
    let analysis = analyze_bounds_checks(ir);
    if analysis.eliminable.is_empty() {
        return ir.to_string();
    }

    let lines: Vec<&str> = ir.lines().collect();
    let mut result = String::new();
    let eliminable_lines: std::collections::HashSet<usize> =
        analysis.eliminable.iter().map(|bc| bc.line_idx).collect();

    for (i, line) in lines.iter().enumerate() {
        if eliminable_lines.contains(&i) {
            // Replace the conditional branch with unconditional to the safe path
            if let Some(safe_target) = extract_true_target(line) {
                result.push_str(&format!("  br label {}\n", safe_target));
                // Add a comment noting the elimination
                result.push_str("  ; bounds check eliminated by range analysis\n");
                continue;
            }
        }
        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Find loop induction variables: `%i = phi i64 [0, %entry], [%i.next, %loop]`
/// paired with `icmp slt i64 %i, %n` as the loop guard.
fn analyze_induction_variables(lines: &[&str], analysis: &mut RangeAnalysis) {
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Match phi nodes that look like induction variables
        // Pattern: %var = phi i64 [ 0, %entry ], [ %var.next, %loop_body ]
        if let Some(var) = parse_induction_phi(trimmed) {
            // Look for the loop guard: icmp slt i64 %var, %bound
            if let Some(bound) = find_loop_bound(lines, i, &var) {
                analysis
                    .ranges
                    .insert(var.clone(), ValueRange::bounded_sym(0, bound));
            }
        }
    }
}

/// Parse a phi node that looks like a loop induction variable.
/// Returns the variable name if it starts at 0 and has a `.next` increment.
fn parse_induction_phi(line: &str) -> Option<String> {
    // Pattern: %var = phi i64 [ 0, %label ], [ %var.next, %label2 ]
    //          %var = phi i32 [ 0, %label ], [ %var.next, %label2 ]
    if !line.contains("= phi") || !line.contains("[ 0,") {
        return None;
    }

    let var = line.split('=').next()?.trim().to_string();
    if !var.starts_with('%') {
        return None;
    }

    // Check that increment variable is %var.next or %var_next
    let var_base = var.trim_start_matches('%');
    let next_pattern1 = format!("%{}.next", var_base);
    let next_pattern2 = format!("%{}next", var_base);
    let next_pattern3 = format!("%{}_next", var_base);

    if line.contains(&next_pattern1)
        || line.contains(&next_pattern2)
        || line.contains(&next_pattern3)
    {
        Some(var)
    } else {
        None
    }
}

/// Find the loop bound for an induction variable by scanning nearby comparisons.
fn find_loop_bound(lines: &[&str], phi_line: usize, var: &str) -> Option<String> {
    // Search within a window around the phi node for icmp slt/ult
    let search_end = std::cmp::min(phi_line + 20, lines.len());
    for line in lines.iter().take(search_end).skip(phi_line) {
        let trimmed = line.trim();
        // Pattern: %cmp = icmp slt i64 %var, %bound
        //          %cmp = icmp ult i64 %var, %bound
        if (trimmed.contains("icmp slt") || trimmed.contains("icmp ult")) && trimmed.contains(var) {
            // Extract the bound (second operand of icmp)
            let parts: Vec<&str> = trimmed.split(',').collect();
            if parts.len() >= 2 {
                let bound = parts[1].trim().to_string();
                return Some(bound);
            }
        }
    }
    None
}

/// Find guard comparisons: `icmp ult %idx, %len` that dominate array accesses.
fn analyze_guards(lines: &[&str], analysis: &mut RangeAnalysis) {
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Pattern: %check = icmp ult i64 %idx, %len
        if trimmed.contains("icmp ult") || trimmed.contains("icmp slt") {
            if let Some((cmp_var, idx_var, bound_var)) = parse_comparison(trimmed) {
                // Look for a conditional branch using this comparison
                if let Some(br_line) = find_branch_on_cmp(lines, i, &cmp_var) {
                    // The true branch of this comparison has proven %idx < %bound
                    analysis.ranges.insert(
                        idx_var.clone(),
                        ValueRange::bounded_sym(0, bound_var.clone()),
                    );

                    // Check if there's a GEP in the true branch using this index
                    // If so, any subsequent bounds check is redundant
                    if has_array_access_in_true_branch(lines, br_line, &idx_var) {
                        analysis.eliminable.push(BoundsCheck {
                            index_var: idx_var,
                            length: bound_var,
                            line_idx: br_line,
                            is_explicit: true,
                        });
                    }
                }
            }
        }

        i += 1;
    }
}

/// Parse icmp comparison: `%cmp = icmp ult i64 %a, %b`
fn parse_comparison(line: &str) -> Option<(String, String, String)> {
    // Split on '='
    let (lhs, rhs) = line.split_once('=')?;
    let cmp_var = lhs.trim().to_string();

    // Parse `icmp ult i64 %a, %b` or `icmp slt i64 %a, %b`
    let rhs = rhs.trim();
    let after_icmp = if rhs.contains("icmp ult") {
        rhs.split("icmp ult").nth(1)?
    } else if rhs.contains("icmp slt") {
        rhs.split("icmp slt").nth(1)?
    } else {
        return None;
    };

    // After icmp: `i64 %a, %b` or `i32 %a, %b`
    let after_type = after_icmp.trim();
    // Skip the type (first word)
    let rest = after_type
        .split_whitespace()
        .skip(1)
        .collect::<Vec<_>>()
        .join(" ");
    let parts: Vec<&str> = rest.split(',').collect();
    if parts.len() >= 2 {
        let idx = parts[0].trim().to_string();
        let bound = parts[1].trim().to_string();
        Some((cmp_var, idx, bound))
    } else {
        None
    }
}

/// Find a conditional branch that uses a comparison result.
fn find_branch_on_cmp(lines: &[&str], cmp_line: usize, cmp_var: &str) -> Option<usize> {
    let search_end = std::cmp::min(cmp_line + 5, lines.len());
    for (i, line) in lines.iter().enumerate().take(search_end).skip(cmp_line + 1) {
        let trimmed = line.trim();
        if trimmed.starts_with("br i1") && trimmed.contains(cmp_var) {
            return Some(i);
        }
    }
    None
}

/// Check if the true branch of a conditional branch contains an array access
/// using the given index variable.
fn has_array_access_in_true_branch(lines: &[&str], br_line: usize, idx_var: &str) -> bool {
    // Extract true target label from: br i1 %cmp, label %then, label %else
    let br = lines[br_line].trim();
    let true_label = extract_true_label(br);
    if true_label.is_none() {
        return false;
    }
    // safe: checked above that true_label is Some
    let true_label = true_label.unwrap();

    // Find the true label block and check for GEP using idx_var
    let mut in_true_block = false;
    for line in lines.iter().skip(br_line + 1) {
        let trimmed = line.trim();

        // Check if we've entered the true block
        if trimmed.starts_with(&format!("{}:", true_label.trim_start_matches('%'))) {
            in_true_block = true;
            continue;
        }

        if in_true_block {
            // End of block
            if trimmed.starts_with("br ") || trimmed.starts_with("ret ") || trimmed.ends_with(':') {
                break;
            }
            // Found GEP using our index variable
            if trimmed.contains("getelementptr") && trimmed.contains(idx_var) {
                return true;
            }
        }
    }
    false
}

/// Extract the true target label from a conditional branch.
fn extract_true_label(br_line: &str) -> Option<String> {
    // br i1 %cmp, label %then, label %else
    let parts: Vec<&str> = br_line.split("label").collect();
    if parts.len() >= 2 {
        let label = parts[1].trim().trim_matches(',').trim().to_string();
        Some(label)
    } else {
        None
    }
}

/// Analyze constant array accesses: index by constant into known-length arrays.
fn analyze_constant_accesses(lines: &[&str], analysis: &mut RangeAnalysis) {
    // Track arrays with known lengths from alloca
    let mut array_lengths: HashMap<String, i64> = HashMap::new();

    for line in lines {
        let trimmed = line.trim();

        // Pattern: %arr = alloca [10 x i64]
        if trimmed.contains("alloca") && trimmed.contains("[") {
            if let Some((var, len)) = parse_array_alloca(trimmed) {
                array_lengths.insert(var, len);
            }
        }
    }

    // Find GEP with constant indices into known-length arrays
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains("getelementptr") {
            if let Some((base, const_idx)) = parse_const_gep(trimmed) {
                if let Some(&arr_len) = array_lengths.get(&base) {
                    if const_idx >= 0 && const_idx < arr_len {
                        // This access is provably safe
                        let idx_name = format!("__const_idx_{}_{}", i, const_idx);
                        analysis
                            .ranges
                            .insert(idx_name, ValueRange::constant(const_idx));
                    }
                }
            }
        }
    }
}

/// Parse `%arr = alloca [10 x i64]` → ("%arr", 10)
fn parse_array_alloca(line: &str) -> Option<(String, i64)> {
    let (lhs, rhs) = line.split_once('=')?;
    let var = lhs.trim().to_string();

    if let Some(bracket_start) = rhs.find('[') {
        if let Some(x_pos) = rhs[bracket_start..].find(" x ") {
            let count_str = &rhs[bracket_start + 1..bracket_start + x_pos];
            if let Ok(count) = count_str.trim().parse::<i64>() {
                return Some((var, count));
            }
        }
    }
    None
}

/// Parse a GEP with constant index → (base_var, constant_index)
fn parse_const_gep(line: &str) -> Option<(String, i64)> {
    // Pattern: %ptr = getelementptr [10 x i64], [10 x i64]* %arr, i64 0, i64 3
    // We want the last index if it's a constant
    if !line.contains("getelementptr") {
        return None;
    }

    // Find the base pointer (the %var before the last comma-separated indices)
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 3 {
        return None;
    }

    // Find base variable
    let mut base = String::new();
    for part in &parts {
        let trimmed = part.trim();
        if trimmed.contains('*') && trimmed.contains('%') {
            // This contains the base pointer
            for word in trimmed.split_whitespace() {
                if word.starts_with('%') {
                    base = word.to_string();
                    break;
                }
            }
        }
    }

    // Get last index
    let last = parts.last()?.trim();
    // Pattern: i64 3 or i32 3
    let idx_str = last.split_whitespace().last()?;
    if let Ok(idx) = idx_str.parse::<i64>() {
        if !base.is_empty() {
            return Some((base, idx));
        }
    }

    None
}

/// Extract the true target from a conditional branch line.
fn extract_true_target(line: &str) -> Option<String> {
    // br i1 %cond, label %true_target, label %false_target
    let parts: Vec<&str> = line.trim().split("label").collect();
    if parts.len() >= 2 {
        let target = parts[1].trim().trim_matches(',').trim().to_string();
        Some(target)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_range_safe_for_length() {
        let range = ValueRange::bounded_sym(0, "%n".to_string());
        assert!(range.is_safe_for_length("%n"));
        assert!(!range.is_safe_for_length("%m"));
    }

    #[test]
    fn test_value_range_safe_for_const_length() {
        let range = ValueRange::bounded(0, 5);
        assert!(range.is_safe_for_const_length(5));
        assert!(range.is_safe_for_const_length(10));
        assert!(!range.is_safe_for_const_length(3));
    }

    #[test]
    fn test_parse_induction_phi() {
        let phi = "%i = phi i64 [ 0, %entry ], [ %i.next, %loop ]";
        assert_eq!(parse_induction_phi(phi), Some("%i".to_string()));

        let non_induction = "%x = phi i64 [ 1, %entry ], [ %y, %loop ]";
        assert_eq!(parse_induction_phi(non_induction), None);
    }

    #[test]
    fn test_parse_comparison() {
        let cmp = "%check = icmp ult i64 %i, %n";
        let result = parse_comparison(cmp);
        assert!(result.is_some());
        let (cmp_var, idx, bound) = result.unwrap();
        assert_eq!(cmp_var, "%check");
        assert_eq!(idx, "%i");
        assert_eq!(bound, "%n");
    }

    #[test]
    fn test_parse_array_alloca() {
        let alloca = "%arr = alloca [10 x i64]";
        let result = parse_array_alloca(alloca);
        assert_eq!(result, Some(("%arr".to_string(), 10)));
    }

    #[test]
    fn test_analyze_loop_with_guard() {
        let ir = r#"
define i64 @sum_array(i64* %arr, i64 %n) {
entry:
  br label %loop

loop:
  %i = phi i64 [ 0, %entry ], [ %i.next, %loop.body ]
  %sum = phi i64 [ 0, %entry ], [ %sum.next, %loop.body ]
  %cmp = icmp slt i64 %i, %n
  br i1 %cmp, label %loop.body, label %exit

loop.body:
  %ptr = getelementptr i64, i64* %arr, i64 %i
  %val = load i64, i64* %ptr
  %sum.next = add i64 %sum, %val
  %i.next = add i64 %i, 1
  br label %loop

exit:
  ret i64 %sum
}
"#;
        let analysis = analyze_bounds_checks(ir);
        // Should detect %i as induction variable with range [0, %n)
        assert!(analysis.ranges.contains_key("%i"));
        let range = &analysis.ranges["%i"];
        assert_eq!(range.lo, Some(0));
    }

    #[test]
    fn test_constant_access_safe() {
        let ir = r#"
define i64 @get_third() {
entry:
  %arr = alloca [10 x i64]
  %ptr = getelementptr [10 x i64], [10 x i64]* %arr, i64 0, i64 3
  %val = load i64, i64* %ptr
  ret i64 %val
}
"#;
        let analysis = analyze_bounds_checks(ir);
        // Should detect constant access at index 3 into length-10 array as safe
        assert!(!analysis.ranges.is_empty());
    }

    #[test]
    fn test_eliminate_bounds_checks_noop() {
        let ir = "define i64 @foo() {\n  ret i64 0\n}\n";
        let result = eliminate_bounds_checks(ir);
        assert!(result.contains("ret i64 0"));
    }
}

//! Advanced Optimization Passes
//!
//! This module implements advanced optimization techniques for Vais:
//!
//! - **Interprocedural Alias Analysis**: Cross-function pointer alias tracking
//! - **Auto-vectorization**: Automatic loop vectorization with LLVM hints
//! - **Cache-friendly Data Layout**: Structure layout optimization for cache efficiency
//! - **Bounds Check Elimination**: Range analysis to remove redundant bounds checks

pub mod alias_analysis;
pub mod auto_vectorize;
pub mod bounds_check_elim;
pub mod data_layout;

pub use alias_analysis::{
    analyze_aliases, propagate_alias_info, AliasAnalysis, AliasResult, FunctionSummary, PointerInfo,
};
pub use auto_vectorize::{
    analyze_vectorization, detect_reductions, generate_vectorization_hints, AutoVectorizer,
    LoopDependence, ReductionKind, VectorWidth, VectorizationCandidate,
};
pub use bounds_check_elim::{
    analyze_bounds_checks, eliminate_bounds_checks, BoundsCheck, RangeAnalysis, ValueRange,
};
pub use data_layout::{
    optimize_struct_layout, padding_savings, suggest_aos_to_soa, suggest_field_reorder,
    DataLayoutOptimizer, FieldInfo, LayoutSuggestion, StructLayout,
};

use crate::optimize::OptLevel;

/// Advanced optimization configuration
#[derive(Debug, Clone)]
pub struct AdvancedOptConfig {
    /// Enable interprocedural alias analysis
    pub alias_analysis: bool,
    /// Enable auto-vectorization hints
    pub auto_vectorize: bool,
    /// Enable data layout optimization
    pub data_layout_opt: bool,
    /// Enable bounds check elimination via range analysis
    pub bounds_check_elim: bool,
    /// Target vector width (default: 256 for AVX2)
    pub vector_width: VectorWidth,
    /// Cache line size in bytes (default: 64)
    pub cache_line_size: usize,
    /// Enable aggressive optimizations (may change semantics in edge cases)
    pub aggressive: bool,
}

impl Default for AdvancedOptConfig {
    fn default() -> Self {
        Self {
            alias_analysis: true,
            auto_vectorize: true,
            data_layout_opt: true,
            bounds_check_elim: true,
            vector_width: VectorWidth::AVX2,
            cache_line_size: 64,
            aggressive: false,
        }
    }
}

impl AdvancedOptConfig {
    /// Create config based on optimization level
    pub fn from_opt_level(level: OptLevel) -> Self {
        match level {
            OptLevel::O0 => Self {
                alias_analysis: false,
                auto_vectorize: false,
                data_layout_opt: false,
                bounds_check_elim: false,
                ..Default::default()
            },
            OptLevel::O1 => Self {
                alias_analysis: true,
                auto_vectorize: false,
                data_layout_opt: false,
                bounds_check_elim: false,
                ..Default::default()
            },
            OptLevel::O2 => Self {
                alias_analysis: true,
                auto_vectorize: true,
                data_layout_opt: true,
                bounds_check_elim: true,
                aggressive: false,
                ..Default::default()
            },
            OptLevel::O3 => Self {
                alias_analysis: true,
                auto_vectorize: true,
                data_layout_opt: true,
                bounds_check_elim: true,
                aggressive: true,
                ..Default::default()
            },
        }
    }
}

/// Apply all advanced optimizations to LLVM IR
pub fn apply_advanced_optimizations(ir: &str, config: &AdvancedOptConfig) -> String {
    let mut result = ir.to_string();

    // Phase 1: Alias analysis (informational, used by other passes)
    let alias_info = if config.alias_analysis {
        Some(analyze_aliases(&result))
    } else {
        None
    };

    // Phase 2: Bounds check elimination via range analysis (before vectorization)
    if config.bounds_check_elim {
        result = eliminate_bounds_checks(&result);
    }

    // Phase 3: Auto-vectorization hints
    if config.auto_vectorize {
        result = generate_vectorization_hints(&result, config.vector_width, alias_info.as_ref());
    }

    // Phase 4: Data layout optimization annotations
    if config.data_layout_opt {
        result = apply_data_layout_hints(&result, config.cache_line_size);
    }

    result
}

/// Apply data layout hints to LLVM IR
fn apply_data_layout_hints(ir: &str, cache_line_size: usize) -> String {
    let mut result = String::new();
    let mut in_struct_def = false;
    let mut current_struct = String::new();
    let mut struct_fields: Vec<(String, String)> = Vec::new();

    for line in ir.lines() {
        let trimmed = line.trim();

        // Detect struct type definitions
        if trimmed.starts_with("%") && trimmed.contains(" = type {") {
            in_struct_def = true;
            current_struct = trimmed.split_whitespace().next().unwrap_or("").to_string();
            struct_fields.clear();
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_struct_def {
            // Parse struct fields and add alignment hints
            if trimmed.contains('}') {
                in_struct_def = false;

                // Add alignment hint comment if struct is large enough
                let estimated_size = estimate_struct_size(&struct_fields);
                if estimated_size >= cache_line_size {
                    result.push_str(&format!(
                        "; layout hint: {} may benefit from cache-line alignment (size ~{} bytes, cache line {} bytes)\n",
                        current_struct, estimated_size, cache_line_size
                    ));
                }
            }
            result.push_str(line);
            result.push('\n');
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// LLVM metadata annotations for Inkwell backend
#[derive(Debug, Clone, Default)]
pub struct LlvmOptHints {
    /// Loop unrolling hints: (function_name, loop_id, unroll_count)
    pub unroll_hints: Vec<(String, usize, u32)>,
    /// Vectorization hints: (function_name, loop_id, vector_width)
    pub vectorize_hints: Vec<(String, usize, VectorWidth)>,
    /// Function inlining hints: (function_name, priority: 0=never, 1=normal, 2=always)
    pub inline_hints: Vec<(String, u8)>,
    /// Alignment hints: (struct_name, alignment_bytes)
    pub alignment_hints: Vec<(String, usize)>,
    /// No-alias hints for pointer parameters: (function_name, param_index)
    pub noalias_hints: Vec<(String, usize)>,
}

/// Analyze IR and generate optimization hints for Inkwell backend
pub fn generate_inkwell_hints(ir: &str, config: &AdvancedOptConfig) -> LlvmOptHints {
    let mut hints = LlvmOptHints::default();

    // Use existing analysis passes to generate hints
    if config.alias_analysis {
        let alias_info = analyze_aliases(ir);
        // Extract no-alias hints from alias analysis
        // Iterate through functions by extracting them from IR
        let function_names = extract_all_function_names(ir);
        for func_name in function_names {
            if let Some(summary) = alias_info.get_function_summary(&func_name) {
                for (param_idx, _) in summary.param_aliases.iter().enumerate() {
                    if !summary.escapes.contains(&param_idx) {
                        hints.noalias_hints.push((func_name.clone(), param_idx));
                    }
                }
            }
        }
    }

    if config.auto_vectorize {
        let vectorizer = analyze_vectorization(ir, config.vector_width);
        for (loop_id, candidate) in vectorizer.candidates.iter().enumerate() {
            if candidate.is_vectorizable {
                // Extract function name from loop header context
                let function_name = extract_function_for_loop(ir, &candidate.header);
                hints.vectorize_hints.push((
                    function_name,
                    loop_id,
                    config.vector_width,
                ));
            }
        }
    }

    if config.data_layout_opt {
        // Analyze struct definitions for alignment hints
        for line in ir.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('%') && trimmed.contains(" = type {") {
                let struct_name = trimmed.split_whitespace().next().unwrap_or("").to_string();
                let estimated = estimate_struct_size_from_line(trimmed);
                if estimated >= config.cache_line_size {
                    hints.alignment_hints.push((struct_name, config.cache_line_size));
                }
            }
        }
    }

    // Generate loop unrolling hints based on bounds check elimination
    if config.bounds_check_elim {
        let bounds = analyze_bounds_checks(ir);
        // For each eliminable bounds check, suggest loop unrolling
        for (loop_id, check) in bounds.eliminable.iter().enumerate() {
            let function_name = extract_function_for_variable(ir, &check.index_var);
            hints.unroll_hints.push((
                function_name,
                loop_id,
                if config.aggressive { 8 } else { 4 },
            ));
        }
    }

    hints
}

/// Helper to estimate struct size from a type definition line
fn estimate_struct_size_from_line(line: &str) -> usize {
    // Parse "{ i64, i64, i8* }" etc.
    if let Some(start) = line.find('{') {
        if let Some(end) = line.rfind('}') {
            let fields: Vec<&str> = line[start+1..end].split(',').collect();
            return fields.iter().map(|f| estimate_type_size(f.trim())).sum();
        }
    }
    0
}

/// Extract function name for a loop header label
fn extract_function_for_loop(ir: &str, loop_header: &str) -> String {
    let mut current_func = String::from("unknown");
    for line in ir.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("define ") {
            if let Some(name) = extract_func_name_from_def(trimmed) {
                current_func = name;
            }
        }
        if trimmed == format!("{}:", loop_header) {
            return current_func;
        }
    }
    current_func
}

/// Extract function name for a variable
fn extract_function_for_variable(ir: &str, var: &str) -> String {
    let mut current_func = String::from("unknown");
    for line in ir.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("define ") {
            if let Some(name) = extract_func_name_from_def(trimmed) {
                current_func = name;
            }
        }
        if trimmed.contains(var) {
            return current_func;
        }
    }
    current_func
}

/// Extract function name from a define line
fn extract_func_name_from_def(line: &str) -> Option<String> {
    if let Some(at_pos) = line.find('@') {
        let rest = &line[at_pos + 1..];
        if let Some(paren_pos) = rest.find('(') {
            return Some(rest[..paren_pos].to_string());
        }
    }
    None
}

/// Extract all function names from IR
fn extract_all_function_names(ir: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in ir.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("define ") {
            if let Some(name) = extract_func_name_from_def(trimmed) {
                names.push(name);
            }
        }
    }
    names
}

/// Estimate struct size from field types
fn estimate_struct_size(fields: &[(String, String)]) -> usize {
    fields.iter().map(|(_, ty)| estimate_type_size(ty)).sum()
}

/// Estimate size of a type
fn estimate_type_size(ty: &str) -> usize {
    match ty {
        "i1" => 1,
        "i8" => 1,
        "i16" => 2,
        "i32" | "float" => 4,
        "i64" | "double" => 8,
        "i128" => 16,
        _ if ty.ends_with('*') => 8, // Pointer
        _ if ty.starts_with('[') => {
            // Array type: [N x T]
            if let Some(rest) = ty.strip_prefix('[') {
                if let Some((count_str, elem_ty)) = rest.split_once(" x ") {
                    let count: usize = count_str.parse().unwrap_or(1);
                    let elem_size = estimate_type_size(elem_ty.trim_end_matches(']'));
                    return count * elem_size;
                }
            }
            8
        }
        _ if ty.starts_with('<') => {
            // Vector type: <N x T>
            if let Some(rest) = ty.strip_prefix('<') {
                if let Some((count_str, elem_ty)) = rest.split_once(" x ") {
                    let count: usize = count_str.parse().unwrap_or(1);
                    let elem_size = estimate_type_size(elem_ty.trim_end_matches('>'));
                    return count * elem_size;
                }
            }
            32
        }
        _ => 8, // Default to pointer size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_opt_config() {
        let config = AdvancedOptConfig::from_opt_level(OptLevel::O3);
        assert!(config.alias_analysis);
        assert!(config.auto_vectorize);
        assert!(config.data_layout_opt);
        assert!(config.aggressive);
    }

    #[test]
    fn test_type_size_estimation() {
        assert_eq!(estimate_type_size("i8"), 1);
        assert_eq!(estimate_type_size("i32"), 4);
        assert_eq!(estimate_type_size("i64"), 8);
        assert_eq!(estimate_type_size("double"), 8);
        assert_eq!(estimate_type_size("i8*"), 8);
        assert_eq!(estimate_type_size("[4 x i32]"), 16);
        assert_eq!(estimate_type_size("<4 x float>"), 16);
    }

    #[test]
    fn test_generate_inkwell_hints_empty() {
        let config = AdvancedOptConfig::from_opt_level(OptLevel::O0);
        let hints = generate_inkwell_hints("", &config);
        assert!(hints.unroll_hints.is_empty());
        assert!(hints.vectorize_hints.is_empty());
    }

    #[test]
    fn test_generate_inkwell_hints_with_struct() {
        let config = AdvancedOptConfig::from_opt_level(OptLevel::O2);
        let ir = "%MyStruct = type { i64, i64, i64, i64, i64, i64, i64, i64, i64 }";
        let hints = generate_inkwell_hints(ir, &config);
        assert!(!hints.alignment_hints.is_empty());
    }

    #[test]
    fn test_vector_width_auto_detect() {
        let width = VectorWidth::auto_detect();
        assert!(width.bits() >= 128);
    }

    #[test]
    fn test_vector_width_lanes() {
        assert_eq!(VectorWidth::SSE.f32_lanes(), 4);
        assert_eq!(VectorWidth::AVX2.f32_lanes(), 8);
        assert_eq!(VectorWidth::SSE.f64_lanes(), 2);
        assert_eq!(VectorWidth::AVX2.f64_lanes(), 4);
        assert_eq!(VectorWidth::AVX512.i32_lanes(), 16);
    }

    #[test]
    fn test_llvm_opt_hints_default() {
        let hints = LlvmOptHints::default();
        assert!(hints.unroll_hints.is_empty());
        assert!(hints.inline_hints.is_empty());
        assert!(hints.noalias_hints.is_empty());
    }

    #[test]
    fn test_estimate_struct_size_from_line() {
        assert_eq!(estimate_struct_size_from_line("%Foo = type { i64, i64 }"), 16);
        assert_eq!(estimate_struct_size_from_line("%Bar = type { i32, i8 }"), 5);
        assert_eq!(estimate_struct_size_from_line(""), 0);
    }

    #[test]
    fn test_advanced_opt_o0_disables_all() {
        let config = AdvancedOptConfig::from_opt_level(OptLevel::O0);
        assert!(!config.alias_analysis);
        assert!(!config.auto_vectorize);
        assert!(!config.data_layout_opt);
        assert!(!config.bounds_check_elim);
    }

    #[test]
    fn test_advanced_opt_o3_enables_aggressive() {
        let config = AdvancedOptConfig::from_opt_level(OptLevel::O3);
        assert!(config.aggressive);
    }
}

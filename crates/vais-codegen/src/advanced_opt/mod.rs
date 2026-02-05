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
}

//! Link-Time Optimization (LTO) support

use std::collections::{HashMap, HashSet};

use super::extract_function_name;

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
    pub fn parse(s: &str) -> Self {
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
        let _trimmed = line.trim();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lto_mode_parsing() {
        assert_eq!(LtoMode::parse("thin"), LtoMode::Thin);
        assert_eq!(LtoMode::parse("THIN"), LtoMode::Thin);
        assert_eq!(LtoMode::parse("full"), LtoMode::Full);
        assert_eq!(LtoMode::parse("fat"), LtoMode::Full);
        assert_eq!(LtoMode::parse("none"), LtoMode::None);
        assert_eq!(LtoMode::parse("invalid"), LtoMode::None);
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
        assert!(analysis.pure_functions.contains("pure_func"));
        assert!(!analysis.pure_functions.contains("impure_func"));
    }
}

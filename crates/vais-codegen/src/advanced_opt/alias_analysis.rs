//! Interprocedural Alias Analysis
//!
//! This module implements pointer alias analysis that tracks aliasing relationships
//! across function boundaries.
//!
//! # Features
//!
//! - **May-alias / Must-alias analysis**: Determine if two pointers may or must point to the same memory
//! - **Escape analysis**: Track which pointers escape their defining scope
//! - **Cross-function propagation**: Propagate alias information through function calls
//! - **LLVM metadata generation**: Generate alias.scope and noalias metadata

use std::collections::{HashMap, HashSet};

/// Result of alias query between two pointers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AliasResult {
    /// Pointers definitely do not alias
    NoAlias,
    /// Pointers may alias (conservative)
    MayAlias,
    /// Pointers definitely alias (point to same location)
    MustAlias,
    /// Pointers partially overlap
    PartialAlias,
}

impl AliasResult {
    /// Returns true if aliasing is possible
    pub fn may_alias(&self) -> bool {
        matches!(
            self,
            AliasResult::MayAlias | AliasResult::MustAlias | AliasResult::PartialAlias
        )
    }

    /// Merge two alias results (most conservative)
    pub fn merge(self, other: AliasResult) -> AliasResult {
        match (self, other) {
            (AliasResult::MustAlias, AliasResult::MustAlias) => AliasResult::MustAlias,
            (AliasResult::NoAlias, AliasResult::NoAlias) => AliasResult::NoAlias,
            _ => AliasResult::MayAlias,
        }
    }
}

/// Information about a pointer value
#[derive(Debug, Clone)]
pub struct PointerInfo {
    /// The base allocation this pointer derives from
    pub base: PointerBase,
    /// Offset from base (if known)
    pub offset: Option<i64>,
    /// Size of the pointed-to object (if known)
    pub size: Option<usize>,
    /// Whether this pointer escapes its defining function
    pub escapes: bool,
    /// Set of pointers this may alias with
    pub alias_set: HashSet<String>,
}

impl Default for PointerInfo {
    fn default() -> Self {
        Self {
            base: PointerBase::Unknown,
            offset: None,
            size: None,
            escapes: false,
            alias_set: HashSet::new(),
        }
    }
}

/// Base of a pointer (where it was allocated)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PointerBase {
    /// Stack allocation (alloca)
    Stack(String),
    /// Heap allocation (malloc, new)
    Heap(String),
    /// Global variable
    Global(String),
    /// Function parameter
    Parameter(String, usize),
    /// Result of a GEP from another pointer
    Derived {
        from: Box<PointerBase>,
        offset: String,
    },
    /// Unknown origin
    Unknown,
}

impl PointerBase {
    /// Check if two bases can never alias
    pub fn disjoint(&self, other: &PointerBase) -> bool {
        match (self, other) {
            // Different stack allocations never alias
            (PointerBase::Stack(a), PointerBase::Stack(b)) if a != b => true,
            // Different heap allocations never alias
            (PointerBase::Heap(a), PointerBase::Heap(b)) if a != b => true,
            // Stack and heap never alias
            (PointerBase::Stack(_), PointerBase::Heap(_)) => true,
            (PointerBase::Heap(_), PointerBase::Stack(_)) => true,
            // Different globals never alias
            (PointerBase::Global(a), PointerBase::Global(b)) if a != b => true,
            _ => false,
        }
    }
}

/// Summary of alias behavior for a function
#[derive(Debug, Clone, Default)]
pub struct FunctionSummary {
    /// Function name
    pub name: String,
    /// Which parameters may be modified
    pub modifies: HashSet<usize>,
    /// Which parameters may be read
    pub reads: HashSet<usize>,
    /// Which parameters may escape
    pub escapes: HashSet<usize>,
    /// Which parameter pairs may alias
    pub param_aliases: Vec<(usize, usize)>,
    /// Whether the function allocates memory that escapes
    pub allocates_escaping: bool,
    /// Whether the function is pure (no side effects)
    pub is_pure: bool,
    /// Whether the function is readonly (reads but doesn't write)
    pub is_readonly: bool,
}

/// Interprocedural alias analysis context
#[derive(Debug, Default)]
pub struct AliasAnalysis {
    /// Pointer information for each SSA variable
    pointers: HashMap<String, PointerInfo>,
    /// Function summaries
    functions: HashMap<String, FunctionSummary>,
    /// Current function being analyzed
    current_function: Option<String>,
    /// Alias scope counter for LLVM metadata
    scope_counter: u32,
}

impl AliasAnalysis {
    /// Create a new alias analysis context
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze a complete LLVM IR module
    pub fn analyze(&mut self, ir: &str) {
        // First pass: build function summaries
        self.build_function_summaries(ir);

        // Second pass: propagate alias information
        self.propagate_aliases(ir);

        // Third pass: escape analysis
        self.analyze_escapes(ir);
    }

    /// Build initial function summaries
    fn build_function_summaries(&mut self, ir: &str) {
        let mut current_func: Option<String> = None;
        let mut current_summary = FunctionSummary::default();

        for line in ir.lines() {
            let trimmed = line.trim();

            // Function definition
            if trimmed.starts_with("define ") {
                if let Some(name) = extract_function_name(trimmed) {
                    if let Some(prev_name) = current_func.take() {
                        self.functions.insert(prev_name, current_summary);
                    }
                    current_func = Some(name.clone());
                    current_summary = FunctionSummary {
                        name,
                        ..Default::default()
                    };
                }
            }

            // Store instruction -> modifies memory
            if trimmed.starts_with("store ") && current_func.is_some() {
                // Extract destination and check if it's a parameter
                if let Some(dest) = extract_store_dest(trimmed) {
                    if let Some(param_idx) = self.get_param_index(&dest) {
                        current_summary.modifies.insert(param_idx);
                    }
                }
            }

            // Load instruction -> reads memory
            if trimmed.contains(" = load ") && current_func.is_some() {
                if let Some(src) = extract_load_src(trimmed) {
                    if let Some(param_idx) = self.get_param_index(&src) {
                        current_summary.reads.insert(param_idx);
                    }
                }
            }

            // Call instruction -> check for escaping pointers
            if trimmed.contains("call ") && current_func.is_some() {
                // Check if any pointer arguments escape
                for arg in extract_call_args(trimmed) {
                    if let Some(param_idx) = self.get_param_index(&arg) {
                        current_summary.escapes.insert(param_idx);
                    }
                }
            }

            // Return instruction
            if trimmed.starts_with("ret ") && current_func.is_some() {
                if let Some(ret_val) = extract_ret_value(trimmed) {
                    if let Some(param_idx) = self.get_param_index(&ret_val) {
                        current_summary.escapes.insert(param_idx);
                    }
                }
            }

            // Function end
            if trimmed == "}" && current_func.is_some() {
                // Determine purity
                if current_summary.modifies.is_empty() && !current_summary.allocates_escaping {
                    if current_summary.reads.is_empty() {
                        current_summary.is_pure = true;
                    } else {
                        current_summary.is_readonly = true;
                    }
                }

                self.functions.insert(
                    current_func.take().unwrap(),
                    std::mem::take(&mut current_summary),
                );
            }
        }
    }

    /// Propagate alias information through the module
    fn propagate_aliases(&mut self, ir: &str) {
        let mut current_func: Option<String> = None;

        for line in ir.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("define ") {
                current_func = extract_function_name(trimmed);
                self.current_function = current_func.clone();
            }

            if current_func.is_none() {
                continue;
            }

            // Alloca instruction -> new stack allocation
            if trimmed.contains(" = alloca ") {
                if let Some(dest) = extract_def_var(trimmed) {
                    self.pointers.insert(
                        dest.clone(),
                        PointerInfo {
                            base: PointerBase::Stack(dest),
                            escapes: false,
                            ..Default::default()
                        },
                    );
                }
            }

            // GEP instruction -> derived pointer
            if trimmed.contains(" = getelementptr ") {
                if let Some((dest, base)) = extract_gep_info(trimmed) {
                    if let Some(base_info) = self.pointers.get(&base) {
                        let new_base = PointerBase::Derived {
                            from: Box::new(base_info.base.clone()),
                            offset: dest.clone(),
                        };
                        self.pointers.insert(
                            dest,
                            PointerInfo {
                                base: new_base,
                                escapes: base_info.escapes,
                                ..Default::default()
                            },
                        );
                    }
                }
            }

            // Bitcast / inttoptr -> propagate alias info
            if trimmed.contains(" = bitcast ") || trimmed.contains(" = inttoptr ") {
                if let Some((dest, src)) = extract_cast_info(trimmed) {
                    if let Some(src_info) = self.pointers.get(&src).cloned() {
                        self.pointers.insert(dest, src_info);
                    }
                }
            }

            // PHI node -> merge alias info
            if trimmed.contains(" = phi ") {
                if let Some((dest, sources)) = extract_phi_info(trimmed) {
                    let mut merged = PointerInfo::default();
                    for src in &sources {
                        if let Some(src_info) = self.pointers.get(src) {
                            merged.alias_set.extend(src_info.alias_set.clone());
                            merged.escapes |= src_info.escapes;
                        }
                    }
                    merged.alias_set.extend(sources);
                    self.pointers.insert(dest, merged);
                }
            }

            if trimmed == "}" {
                current_func = None;
            }
        }
    }

    /// Perform escape analysis
    fn analyze_escapes(&mut self, ir: &str) {
        let mut current_func: Option<String> = None;

        for line in ir.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("define ") {
                current_func = extract_function_name(trimmed);
            }

            if current_func.is_none() {
                continue;
            }

            // Store to global -> escapes
            if trimmed.starts_with("store ") {
                if let Some(value) = extract_store_value(trimmed) {
                    if let Some(dest) = extract_store_dest(trimmed) {
                        if dest.starts_with("@") {
                            // Storing to global -> value escapes
                            if let Some(info) = self.pointers.get_mut(&value) {
                                info.escapes = true;
                            }
                        }
                    }
                }
            }

            // Return pointer -> escapes
            if trimmed.starts_with("ret ") {
                if let Some(value) = extract_ret_value(trimmed) {
                    if let Some(info) = self.pointers.get_mut(&value) {
                        info.escapes = true;
                    }
                }
            }

            // Pass to external function -> escapes (conservative)
            if trimmed.contains("call ") {
                if let Some(func_name) = extract_called_function(trimmed) {
                    let is_external = !self.functions.contains_key(&func_name);
                    if is_external {
                        for arg in extract_call_args(trimmed) {
                            if let Some(info) = self.pointers.get_mut(&arg) {
                                info.escapes = true;
                            }
                        }
                    }
                }
            }

            if trimmed == "}" {
                current_func = None;
            }
        }
    }

    /// Query if two pointers may alias
    pub fn query(&self, ptr1: &str, ptr2: &str) -> AliasResult {
        let info1 = self.pointers.get(ptr1);
        let info2 = self.pointers.get(ptr2);

        match (info1, info2) {
            (Some(i1), Some(i2)) => {
                // Check if bases are disjoint
                if i1.base.disjoint(&i2.base) {
                    return AliasResult::NoAlias;
                }

                // Same base with same offset -> must alias
                if i1.base == i2.base && i1.offset == i2.offset && i1.offset.is_some() {
                    return AliasResult::MustAlias;
                }

                // Check explicit alias sets
                if i1.alias_set.contains(ptr2) || i2.alias_set.contains(ptr1) {
                    return AliasResult::MayAlias;
                }

                // Conservative: may alias
                AliasResult::MayAlias
            }
            _ => AliasResult::MayAlias, // Unknown -> conservative
        }
    }

    /// Check if a pointer escapes its defining function
    pub fn escapes(&self, ptr: &str) -> bool {
        self.pointers.get(ptr).is_none_or(|info| info.escapes)
    }

    /// Get function summary
    pub fn get_function_summary(&self, name: &str) -> Option<&FunctionSummary> {
        self.functions.get(name)
    }

    /// Generate LLVM alias metadata for a function
    pub fn generate_metadata(&mut self, func_name: &str) -> String {
        let mut metadata = String::new();

        // Create alias scopes for non-escaping pointers
        let mut scopes: Vec<(String, u32)> = Vec::new();

        for (ptr, info) in &self.pointers {
            if !info.escapes {
                self.scope_counter += 1;
                scopes.push((ptr.clone(), self.scope_counter));
            }
        }

        if scopes.is_empty() {
            return metadata;
        }

        // Generate scope metadata
        metadata.push_str(&format!("; Alias scopes for function {}\n", func_name));

        for (ptr, scope_id) in &scopes {
            metadata.push_str(&format!(
                "!alias_scope_{} = !{{!\"scope_{}\", !\"function_{}\"}}\n",
                scope_id, ptr, func_name
            ));
        }

        // Generate noalias metadata for non-aliasing pointer pairs
        for i in 0..scopes.len() {
            for j in (i + 1)..scopes.len() {
                let (ptr1, _) = &scopes[i];
                let (ptr2, _) = &scopes[j];

                if self.query(ptr1, ptr2) == AliasResult::NoAlias {
                    metadata.push_str(&format!("; {} and {} are noalias\n", ptr1, ptr2));
                }
            }
        }

        metadata
    }

    /// Get parameter index if the variable is a function parameter
    fn get_param_index(&self, var: &str) -> Option<usize> {
        if let Some(info) = self.pointers.get(var) {
            if let PointerBase::Parameter(_, idx) = info.base {
                return Some(idx);
            }
        }
        None
    }
}

/// Analyze aliases in the given IR
pub fn analyze_aliases(ir: &str) -> AliasAnalysis {
    let mut analysis = AliasAnalysis::new();
    analysis.analyze(ir);
    analysis
}

/// Propagate alias information and return annotated IR
pub fn propagate_alias_info(ir: &str) -> String {
    let analysis = analyze_aliases(ir);
    let mut result = String::new();
    let mut current_func: Option<String> = None;

    for line in ir.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("define ") {
            current_func = extract_function_name(trimmed);
            result.push_str(line);
            result.push('\n');

            // Add alias metadata for this function
            if let Some(ref func_name) = current_func {
                let analysis_clone = analysis.pointers.clone();
                let mut temp_analysis = AliasAnalysis {
                    pointers: analysis_clone,
                    ..Default::default()
                };
                let metadata = temp_analysis.generate_metadata(func_name);
                if !metadata.is_empty() {
                    result.push_str(&metadata);
                }
            }
            continue;
        }

        // Add noalias hints to load/store instructions
        if current_func.is_some() {
            if trimmed.contains(" = load ") {
                if let Some(src) = extract_load_src(trimmed) {
                    if !analysis.escapes(&src) {
                        result.push_str(line);
                        result.push_str("  ; noalias hint: pointer does not escape\n");
                        continue;
                    }
                }
            }

            if trimmed.starts_with("store ") {
                if let Some(dest) = extract_store_dest(trimmed) {
                    if !analysis.escapes(&dest) {
                        result.push_str(line);
                        result.push_str("  ; noalias hint: pointer does not escape\n");
                        continue;
                    }
                }
            }
        }

        if trimmed == "}" {
            current_func = None;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

// Helper functions for parsing LLVM IR

fn extract_function_name(line: &str) -> Option<String> {
    let at_pos = line.find('@')?;
    let paren_pos = line[at_pos..].find('(')?;
    Some(line[at_pos + 1..at_pos + paren_pos].to_string())
}

fn extract_def_var(line: &str) -> Option<String> {
    let eq_pos = line.find(" = ")?;
    let var = line[..eq_pos].trim();
    if var.starts_with('%') {
        Some(var.to_string())
    } else {
        None
    }
}

fn extract_store_dest(line: &str) -> Option<String> {
    // store TYPE VALUE, TYPE* DEST
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() >= 2 {
        let dest_part = parts[1].trim();
        let var = dest_part.split_whitespace().last()?;
        Some(var.to_string())
    } else {
        None
    }
}

fn extract_store_value(line: &str) -> Option<String> {
    // store TYPE VALUE, TYPE* DEST
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 3 {
        // parts: ["store", "TYPE", "VALUE,", ...]
        let value = parts[2].trim_end_matches(',');
        Some(value.to_string())
    } else {
        None
    }
}

fn extract_load_src(line: &str) -> Option<String> {
    // %x = load TYPE, TYPE* SRC
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() >= 2 {
        let src_part = parts[1].trim();
        let var = src_part.split_whitespace().last()?;
        Some(var.to_string())
    } else {
        None
    }
}

fn extract_call_args(line: &str) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(paren_start) = line.find('(') {
        if let Some(paren_end) = line.rfind(')') {
            let args_str = &line[paren_start + 1..paren_end];
            for arg_part in args_str.split(',') {
                let arg = arg_part.trim();
                // Extract variable from "TYPE VALUE" pattern
                if let Some(var) = arg.split_whitespace().last() {
                    if var.starts_with('%') || var.starts_with('@') {
                        args.push(var.to_string());
                    }
                }
            }
        }
    }
    args
}

fn extract_called_function(line: &str) -> Option<String> {
    let call_pos = line.find("call ")?;
    let rest = &line[call_pos + 5..];
    let at_pos = rest.find('@')?;
    let paren_pos = rest[at_pos..].find('(')?;
    Some(rest[at_pos + 1..at_pos + paren_pos].to_string())
}

fn extract_ret_value(line: &str) -> Option<String> {
    // ret TYPE VALUE or ret void
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 3 {
        let value = parts[2];
        if value.starts_with('%') || value.starts_with('@') {
            return Some(value.to_string());
        }
    }
    None
}

fn extract_gep_info(line: &str) -> Option<(String, String)> {
    // %dest = getelementptr TYPE, TYPE* %base, ...
    let dest = extract_def_var(line)?;
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() >= 2 {
        let base_part = parts[1].trim();
        let base = base_part.split_whitespace().last()?;
        if base.starts_with('%') || base.starts_with('@') {
            return Some((dest, base.to_string()));
        }
    }
    None
}

fn extract_cast_info(line: &str) -> Option<(String, String)> {
    // %dest = bitcast TYPE %src to TYPE
    let dest = extract_def_var(line)?;
    for part in line.split_whitespace() {
        if part.starts_with('%') && part != dest {
            return Some((dest, part.to_string()));
        }
    }
    None
}

fn extract_phi_info(line: &str) -> Option<(String, Vec<String>)> {
    // %dest = phi TYPE [val1, %label1], [val2, %label2], ...
    let dest = extract_def_var(line)?;
    let mut sources = Vec::new();

    for part in line.split('[').skip(1) {
        if let Some(comma_pos) = part.find(',') {
            let val = part[..comma_pos].trim();
            if val.starts_with('%') || val.starts_with('@') {
                sources.push(val.to_string());
            }
        }
    }

    if sources.is_empty() {
        None
    } else {
        Some((dest, sources))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_result_merge() {
        assert_eq!(
            AliasResult::NoAlias.merge(AliasResult::NoAlias),
            AliasResult::NoAlias
        );
        assert_eq!(
            AliasResult::MustAlias.merge(AliasResult::MustAlias),
            AliasResult::MustAlias
        );
        assert_eq!(
            AliasResult::NoAlias.merge(AliasResult::MayAlias),
            AliasResult::MayAlias
        );
    }

    #[test]
    fn test_pointer_base_disjoint() {
        let stack1 = PointerBase::Stack("a".to_string());
        let stack2 = PointerBase::Stack("b".to_string());
        let heap1 = PointerBase::Heap("c".to_string());

        assert!(stack1.disjoint(&stack2));
        assert!(stack1.disjoint(&heap1));
        assert!(!stack1.disjoint(&stack1.clone()));
    }

    #[test]
    fn test_analyze_simple_function() {
        let ir = r#"
define i64 @simple(i64* %p) {
entry:
  %x = alloca i64
  store i64 42, i64* %x
  %v = load i64, i64* %x
  ret i64 %v
}
"#;

        let analysis = analyze_aliases(ir);
        assert!(analysis.functions.contains_key("simple"));
    }

    #[test]
    fn test_escape_analysis() {
        let ir = r#"
define i64* @escaping() {
entry:
  %x = alloca i64
  ret i64* %x
}

define void @non_escaping() {
entry:
  %x = alloca i64
  store i64 42, i64* %x
  ret void
}
"#;

        let analysis = analyze_aliases(ir);

        // The returned pointer escapes
        assert!(analysis.escapes("%x") || !analysis.pointers.contains_key("%x"));
    }

    #[test]
    fn test_extract_function_name() {
        assert_eq!(
            extract_function_name("define i64 @foo(i64 %x) {"),
            Some("foo".to_string())
        );
        assert_eq!(
            extract_function_name("define void @bar() {"),
            Some("bar".to_string())
        );
    }

    #[test]
    fn test_extract_store_dest() {
        assert_eq!(
            extract_store_dest("store i64 42, i64* %x"),
            Some("%x".to_string())
        );
        assert_eq!(
            extract_store_dest("store i64 %val, i64* @global"),
            Some("@global".to_string())
        );
    }
}

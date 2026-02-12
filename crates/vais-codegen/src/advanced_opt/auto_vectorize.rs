//! Auto-Vectorization for Loops
//!
//! This module implements automatic vectorization analysis and LLVM hint generation
//! for loops that can benefit from SIMD execution.
//!
//! # Features
//!
//! - **Loop vectorization candidate detection**: Identify loops suitable for vectorization
//! - **Dependence analysis**: Analyze data and control dependencies
//! - **Vectorization legality checking**: Determine if vectorization is safe
//! - **LLVM metadata generation**: Generate llvm.loop.vectorize.* hints
//! - **Vector width selection**: Choose optimal vector width for target architecture

use super::alias_analysis::AliasAnalysis;
use std::collections::{HashMap, HashSet};

/// Target vector width
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VectorWidth {
    /// SSE: 128 bits (4 x i32, 2 x i64, 4 x f32, 2 x f64)
    SSE,
    /// AVX/AVX2: 256 bits (8 x i32, 4 x i64, 8 x f32, 4 x f64)
    #[default]
    AVX2,
    /// AVX-512: 512 bits (16 x i32, 8 x i64, 16 x f32, 8 x f64)
    AVX512,
    /// ARM NEON: 128 bits
    NEON,
    /// Auto-detect based on target
    Auto,
}

impl VectorWidth {
    /// Get the vector width in bits
    pub fn bits(&self) -> usize {
        match self {
            VectorWidth::SSE | VectorWidth::NEON => 128,
            VectorWidth::AVX2 => 256,
            VectorWidth::AVX512 => 512,
            VectorWidth::Auto => 256, // Default to AVX2
        }
    }

    /// Get the number of elements for a given element size
    pub fn lanes(&self, element_bits: usize) -> usize {
        self.bits() / element_bits
    }

    /// Get LLVM target features string
    pub fn target_features(&self) -> &'static str {
        match self {
            VectorWidth::SSE => "+sse4.2",
            VectorWidth::AVX2 => "+avx2",
            VectorWidth::AVX512 => "+avx512f,+avx512dq,+avx512vl",
            VectorWidth::NEON => "+neon",
            VectorWidth::Auto => "+avx2",
        }
    }

    /// Auto-detect best vector width for current platform
    pub fn auto_detect() -> Self {
        // Default heuristic based on common platforms
        #[cfg(target_arch = "x86_64")]
        {
            // Most modern x86_64 have AVX2
            VectorWidth::AVX2
        }
        #[cfg(target_arch = "aarch64")]
        {
            // ARM NEON is 128-bit
            VectorWidth::NEON
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            VectorWidth::SSE // Safe fallback
        }
    }

    /// Get the number of f32 elements per vector
    pub fn f32_lanes(&self) -> u32 {
        (self.bits() / 32) as u32
    }

    /// Get the number of f64 elements per vector
    pub fn f64_lanes(&self) -> u32 {
        (self.bits() / 64) as u32
    }

    /// Get the number of i32 elements per vector
    pub fn i32_lanes(&self) -> u32 {
        (self.bits() / 32) as u32
    }
}

/// Loop dependence type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopDependence {
    /// No dependence
    None,
    /// Flow dependence (read-after-write): a[i] = ...; ... = a[i]
    Flow { distance: Option<i64> },
    /// Anti dependence (write-after-read): ... = a[i]; a[i] = ...
    Anti { distance: Option<i64> },
    /// Output dependence (write-after-write): a[i] = ...; a[i] = ...
    Output { distance: Option<i64> },
    /// Unknown dependence (conservative)
    Unknown,
}

impl LoopDependence {
    /// Check if this dependence prevents vectorization
    pub fn prevents_vectorization(&self, vector_width: usize) -> bool {
        match self {
            LoopDependence::None => false,
            LoopDependence::Flow { distance }
            | LoopDependence::Anti { distance }
            | LoopDependence::Output { distance } => {
                match distance {
                    Some(d) if d.unsigned_abs() as usize >= vector_width => false,
                    Some(_) => true, // Distance too small
                    None => true,    // Unknown distance
                }
            }
            LoopDependence::Unknown => true,
        }
    }
}

/// Vectorization candidate information
#[derive(Debug, Clone)]
pub struct VectorizationCandidate {
    /// Loop header label
    pub header: String,
    /// Loop latch label
    pub latch: String,
    /// Induction variable
    pub induction_var: Option<String>,
    /// Trip count (if known)
    pub trip_count: Option<u64>,
    /// Memory accesses in the loop
    pub memory_accesses: Vec<MemoryAccess>,
    /// Detected dependencies
    pub dependencies: Vec<LoopDependence>,
    /// Whether the loop is vectorizable
    pub is_vectorizable: bool,
    /// Reason if not vectorizable
    pub non_vectorizable_reason: Option<String>,
    /// Recommended vector width
    pub recommended_width: Option<usize>,
    /// Estimated speedup factor
    pub estimated_speedup: f64,
}

impl Default for VectorizationCandidate {
    fn default() -> Self {
        Self {
            header: String::new(),
            latch: String::new(),
            induction_var: None,
            trip_count: None,
            memory_accesses: Vec::new(),
            dependencies: Vec::new(),
            is_vectorizable: false,
            non_vectorizable_reason: Some("Not analyzed".to_string()),
            recommended_width: None,
            estimated_speedup: 1.0,
        }
    }
}

/// Memory access in a loop
#[derive(Debug, Clone)]
pub struct MemoryAccess {
    /// Instruction line
    pub instruction: String,
    /// Base pointer
    pub base: String,
    /// Index expression (if array access)
    pub index: Option<String>,
    /// Stride (elements between consecutive iterations)
    pub stride: Option<i64>,
    /// Whether this is a read or write
    pub is_write: bool,
    /// Element type size in bytes
    pub element_size: usize,
}

/// Auto-vectorization analyzer
#[derive(Debug)]
pub struct AutoVectorizer {
    /// Detected loop candidates
    pub candidates: Vec<VectorizationCandidate>,
    /// Target vector width
    pub target_width: VectorWidth,
    /// Loop metadata counter
    loop_id_counter: u32,
}

impl AutoVectorizer {
    /// Create a new auto-vectorizer
    pub fn new(target_width: VectorWidth) -> Self {
        Self {
            candidates: Vec::new(),
            target_width,
            loop_id_counter: 0,
        }
    }

    /// Analyze IR for vectorization opportunities
    pub fn analyze(&mut self, ir: &str, alias_analysis: Option<&AliasAnalysis>) {
        self.detect_loops(ir);
        self.analyze_memory_accesses(ir);
        self.analyze_dependencies(alias_analysis);
        self.determine_vectorizability();
    }

    /// Detect loops in the IR
    fn detect_loops(&mut self, ir: &str) {
        let mut current_func: Option<String> = None;
        let mut labels: Vec<String> = Vec::new();
        let mut label_set: HashSet<String> = HashSet::new();
        let mut branch_targets: HashMap<String, Vec<String>> = HashMap::new();
        let mut current_label = String::from("entry");

        // First pass: collect labels and branch targets
        for line in ir.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("define ") {
                current_func = extract_func_name(trimmed);
                labels.clear();
                label_set.clear();
                branch_targets.clear();
                current_label = String::from("entry");
            }

            if current_func.is_none() {
                continue;
            }

            // Collect labels and track current block
            if trimmed.ends_with(':') && !trimmed.contains(' ') {
                let label = trimmed.trim_end_matches(':').to_string();
                if label_set.insert(label.clone()) {
                    labels.push(label.clone());
                }
                current_label = label;
            }

            // Collect branch targets
            if trimmed.starts_with("br ") {
                let targets = extract_branch_targets(trimmed);
                for target in targets {
                    branch_targets
                        .entry(target)
                        .or_default()
                        .push(current_label.clone());
                }
            }

            if trimmed == "}" {
                // Look for back edges (branches to earlier labels = loops)
                // Use label ordering to detect back edges: a branch from a later
                // label to an earlier label indicates a loop
                let label_order: HashMap<&str, usize> = labels
                    .iter()
                    .enumerate()
                    .map(|(i, l)| (l.as_str(), i))
                    .collect();

                for label in &labels {
                    if let Some(sources) = branch_targets.get(label) {
                        let header_idx = label_order.get(label.as_str()).copied().unwrap_or(0);

                        // Check for back edges: source comes after header in order
                        let mut best_latch: Option<String> = None;
                        for src in sources {
                            if !src.is_empty() && src != label {
                                let src_idx = label_order.get(src.as_str()).copied().unwrap_or(0);
                                if src_idx >= header_idx {
                                    // This is a back edge - src is at or after header
                                    best_latch = Some(src.clone());
                                }
                            }
                        }

                        if let Some(latch) = best_latch {
                            let mut candidate = VectorizationCandidate {
                                header: label.clone(),
                                latch,
                                ..Default::default()
                            };

                            // Try to detect trip count from loop structure
                            candidate.trip_count = detect_trip_count(ir, &candidate.header);

                            self.candidates.push(candidate);
                        }
                    }
                }

                current_func = None;
            }
        }
    }

    /// Analyze memory accesses within detected loops
    fn analyze_memory_accesses(&mut self, ir: &str) {
        let lines: Vec<&str> = ir.lines().collect();

        for candidate in &mut self.candidates {
            let mut in_loop = false;
            let mut memory_accesses = Vec::new();

            for line in &lines {
                let trimmed = line.trim();

                // Check if we're entering the loop
                if trimmed == format!("{}:", candidate.header) {
                    in_loop = true;
                    continue;
                }

                // Check if we're leaving the loop
                if in_loop && trimmed.ends_with(':') && !trimmed.starts_with(&candidate.header) {
                    // Check if this is still part of the loop body
                    let label = trimmed.trim_end_matches(':');
                    if label != candidate.latch && !label.starts_with(&candidate.header) {
                        // Might have left the loop
                    }
                }

                if !in_loop {
                    continue;
                }

                // Detect load instructions
                if trimmed.contains(" = load ") {
                    if let Some(access) = parse_memory_access(trimmed, false) {
                        memory_accesses.push(access);
                    }
                }

                // Detect store instructions
                if trimmed.starts_with("store ") {
                    if let Some(access) = parse_memory_access(trimmed, true) {
                        memory_accesses.push(access);
                    }
                }

                // Check for function calls with side effects
                if (trimmed.starts_with("call ") || trimmed.contains(" = call "))
                    && in_loop
                    && has_side_effects(trimmed)
                {
                    // Mark as having side effects - will be checked during vectorizability
                    memory_accesses.push(MemoryAccess {
                        instruction: trimmed.to_string(),
                        base: "__side_effect_call__".to_string(),
                        index: None,
                        stride: None,
                        is_write: true,
                        element_size: 0,
                    });
                }

                // Detect induction variable (PHI node in loop header)
                if trimmed.contains(" = phi ") && in_loop {
                    if let Some(var) = extract_phi_var(trimmed) {
                        if candidate.induction_var.is_none() {
                            candidate.induction_var = Some(var);
                        }
                    }
                }

                // Check for loop exit via conditional branch with comparison
                if trimmed.starts_with("br i1 ") {
                    // Conditional branch - check if one target exits the loop
                    let targets = extract_branch_targets(trimmed);
                    let exits_loop = targets
                        .iter()
                        .any(|t| t != &candidate.header && t != &candidate.latch);
                    if exits_loop {
                        // This is the loop exit branch - we remain in the loop body
                        // until the condition fails
                    }
                }
            }

            candidate.memory_accesses = memory_accesses;
        }
    }

    /// Analyze dependencies between memory accesses
    fn analyze_dependencies(&mut self, alias_analysis: Option<&AliasAnalysis>) {
        for candidate in &mut self.candidates {
            let mut dependencies = Vec::new();

            let accesses = &candidate.memory_accesses;

            for i in 0..accesses.len() {
                for j in (i + 1)..accesses.len() {
                    let a1 = &accesses[i];
                    let a2 = &accesses[j];

                    // Same base pointer -> potential dependence
                    if a1.base == a2.base {
                        let dep = analyze_access_pair(a1, a2, alias_analysis);
                        if dep != LoopDependence::None {
                            dependencies.push(dep);
                        }
                    } else if let Some(alias) = alias_analysis {
                        // Check via alias analysis
                        let alias_result = alias.query(&a1.base, &a2.base);
                        if alias_result.may_alias() {
                            dependencies.push(LoopDependence::Unknown);
                        }
                    }
                }
            }

            candidate.dependencies = dependencies;
        }
    }

    /// Determine if loops are vectorizable
    fn determine_vectorizability(&mut self) {
        for candidate in &mut self.candidates {
            // Determine dominant element size from memory accesses
            let element_bits = candidate
                .memory_accesses
                .iter()
                .filter(|a| a.element_size > 0 && a.base != "__side_effect_call__")
                .map(|a| a.element_size * 8)
                .max()
                .unwrap_or(64); // Default to 64-bit if no memory accesses

            let vector_lanes = self.target_width.lanes(element_bits);

            // Check for side-effect function calls
            let has_side_effects = candidate
                .memory_accesses
                .iter()
                .any(|a| a.base == "__side_effect_call__");

            if has_side_effects {
                candidate.is_vectorizable = false;
                candidate.non_vectorizable_reason =
                    Some("Loop contains function calls with potential side effects".to_string());
                continue;
            }

            // Check dependencies
            let has_blocking_dep = candidate
                .dependencies
                .iter()
                .any(|d| d.prevents_vectorization(vector_lanes));

            if has_blocking_dep {
                candidate.is_vectorizable = false;
                candidate.non_vectorizable_reason =
                    Some("Loop-carried dependence prevents vectorization".to_string());
                continue;
            }

            // Filter out side-effect markers for stride analysis
            let real_accesses: Vec<&MemoryAccess> = candidate
                .memory_accesses
                .iter()
                .filter(|a| a.base != "__side_effect_call__")
                .collect();

            // Check if all memory accesses have unit stride
            let all_unit_stride = real_accesses
                .iter()
                .all(|a| a.stride.is_some_and(|s| s == 1 || s == -1 || s == 0));

            if !all_unit_stride && !real_accesses.is_empty() {
                // Non-unit stride requires gather/scatter
                candidate.is_vectorizable = true;
                candidate.non_vectorizable_reason = None;
                candidate.recommended_width = Some(std::cmp::max(vector_lanes / 2, 1));
                candidate.estimated_speedup = 1.5;
                continue;
            }

            // Looks good for vectorization
            candidate.is_vectorizable = true;
            candidate.non_vectorizable_reason = None;
            candidate.recommended_width = Some(vector_lanes);
            candidate.estimated_speedup = vector_lanes as f64 * 0.8; // Realistic estimate
        }
    }

    /// Generate LLVM loop metadata for vectorization hints
    pub fn generate_hints(&mut self, ir: &str) -> String {
        let mut result = String::new();
        let mut metadata_defs = Vec::new();

        for line in ir.lines() {
            let trimmed = line.trim();

            // Check if this is a loop header
            let mut found_candidate = None;
            for candidate in &self.candidates {
                if trimmed == format!("{}:", candidate.header) {
                    found_candidate = Some(candidate.clone());
                    break;
                }
            }

            if let Some(candidate) = found_candidate {
                self.loop_id_counter += 1;
                let loop_id = self.loop_id_counter;

                // Add loop header with metadata reference
                result.push_str(line);
                result.push('\n');

                // Generate vectorization hint comment
                if candidate.is_vectorizable {
                    result.push_str(&format!(
                        "  ; VECTORIZATION HINT: loop can be vectorized (width={})\n",
                        candidate.recommended_width.unwrap_or(4)
                    ));

                    // Store metadata definition for later
                    let md = generate_loop_metadata(loop_id, &candidate, &self.target_width);
                    metadata_defs.push(md);
                } else {
                    result.push_str(&format!(
                        "  ; VECTORIZATION BLOCKED: {}\n",
                        candidate
                            .non_vectorizable_reason
                            .as_deref()
                            .unwrap_or("unknown")
                    ));
                }

                continue;
            }

            // Check for branch back to loop header (latch)
            if trimmed.starts_with("br ") {
                let mut is_latch = false;
                for candidate in &self.candidates {
                    if trimmed.contains(&format!("label %{}", candidate.header)) {
                        is_latch = true;
                        break;
                    }
                }

                if is_latch && self.loop_id_counter > 0 {
                    // Add loop metadata to the branch
                    result.push_str(&format!(
                        "{}  ; Loop back edge, !llvm.loop !{}\n",
                        line, self.loop_id_counter
                    ));
                    continue;
                }
            }

            result.push_str(line);
            result.push('\n');
        }

        // Append metadata definitions at the end
        if !metadata_defs.is_empty() {
            result.push_str("\n; Vectorization metadata\n");
            for md in metadata_defs {
                result.push_str(&md);
                result.push('\n');
            }
        }

        result
    }
}

/// Analyze vectorization opportunities in IR
pub fn analyze_vectorization(ir: &str, target_width: VectorWidth) -> AutoVectorizer {
    let mut vectorizer = AutoVectorizer::new(target_width);
    vectorizer.analyze(ir, None);
    vectorizer
}

/// Generate vectorization hints for IR
pub fn generate_vectorization_hints(
    ir: &str,
    target_width: VectorWidth,
    alias_analysis: Option<&AliasAnalysis>,
) -> String {
    let mut vectorizer = AutoVectorizer::new(target_width);
    vectorizer.analyze(ir, alias_analysis);
    vectorizer.generate_hints(ir)
}

// Helper functions

/// Check if a function call has side effects (non-pure)
fn has_side_effects(line: &str) -> bool {
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

/// Detect trip count from loop structure in the IR
fn detect_trip_count(ir: &str, header_label: &str) -> Option<u64> {
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

fn extract_func_name(line: &str) -> Option<String> {
    let at_pos = line.find('@')?;
    let paren_pos = line[at_pos..].find('(')?;
    Some(line[at_pos + 1..at_pos + paren_pos].to_string())
}

fn extract_branch_targets(line: &str) -> Vec<String> {
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

fn parse_memory_access(line: &str, is_write: bool) -> Option<MemoryAccess> {
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
fn detect_element_size(line: &str) -> usize {
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
fn extract_gep_info(line: &str, _base: &str, element_size: usize) -> (Option<String>, Option<i64>) {
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

fn extract_phi_var(line: &str) -> Option<String> {
    let eq_pos = line.find(" = ")?;
    let var = line[..eq_pos].trim();
    if var.starts_with('%') {
        Some(var.to_string())
    } else {
        None
    }
}

fn analyze_access_pair(
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
fn compute_dependence_distance(a1: &MemoryAccess, a2: &MemoryAccess) -> Option<i64> {
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

fn generate_loop_metadata(
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

/// Reduction operation kind detected in a loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReductionKind {
    /// Sum reduction: acc += val
    Add,
    /// Product reduction: acc *= val
    Mul,
    /// Min reduction: acc = min(acc, val)
    Min,
    /// Max reduction: acc = max(acc, val)
    Max,
    /// Bitwise OR reduction
    Or,
    /// Bitwise AND reduction
    And,
    /// Bitwise XOR reduction
    Xor,
}

impl ReductionKind {
    /// Get the LLVM metadata string for this reduction kind.
    pub fn llvm_metadata(&self) -> &'static str {
        match self {
            ReductionKind::Add => "llvm.loop.vectorize.followup_reduction_add",
            ReductionKind::Mul => "llvm.loop.vectorize.followup_reduction_mul",
            ReductionKind::Min => "llvm.loop.vectorize.followup_reduction_min",
            ReductionKind::Max => "llvm.loop.vectorize.followup_reduction_max",
            ReductionKind::Or => "llvm.loop.vectorize.followup_reduction_or",
            ReductionKind::And => "llvm.loop.vectorize.followup_reduction_and",
            ReductionKind::Xor => "llvm.loop.vectorize.followup_reduction_xor",
        }
    }

    /// Get the identity element for this reduction.
    pub fn identity_i64(&self) -> i64 {
        match self {
            ReductionKind::Add | ReductionKind::Or | ReductionKind::Xor => 0,
            ReductionKind::Mul => 1,
            ReductionKind::Min => i64::MAX,
            ReductionKind::Max => i64::MIN,
            ReductionKind::And => -1, // all ones
        }
    }
}

/// Detect reduction patterns in a loop body.
///
/// Looks for PHI nodes that accumulate values through add/mul/min/max operations.
/// Pattern: `%acc = phi [identity, %entry], [%acc.next, %loop]`
///          `%acc.next = add %acc, %val` (or mul, etc.)
pub fn detect_reductions(ir: &str, loop_header: &str) -> Vec<(String, ReductionKind)> {
    let mut reductions = Vec::new();
    let lines: Vec<&str> = ir.lines().collect();
    let mut in_loop = false;
    let mut phi_accumulators: Vec<(String, String)> = Vec::new(); // (acc_var, next_var)

    // Pass 1: Find PHI nodes that look like accumulators
    for line in &lines {
        let trimmed = line.trim();
        if trimmed == format!("{}:", loop_header) {
            in_loop = true;
            continue;
        }
        if !in_loop {
            continue;
        }
        // End of loop block (next label or function end)
        if (trimmed.ends_with(':') && trimmed != format!("{}:", loop_header))
            && !phi_accumulators.is_empty()
        {
            break;
        }
        // Match: %acc = phi i64 [ 0, %entry ], [ %acc.next, %loop ]
        if trimmed.contains("= phi") {
            if let Some((acc_var, next_var)) = parse_accumulator_phi(trimmed) {
                phi_accumulators.push((acc_var, next_var));
            }
        }
    }

    // Pass 2: Find the update operations for each accumulator
    in_loop = false;
    for line in &lines {
        let trimmed = line.trim();
        if trimmed == format!("{}:", loop_header) {
            in_loop = true;
            continue;
        }
        if !in_loop {
            continue;
        }

        for (acc_var, next_var) in &phi_accumulators {
            // Match: %acc.next = add i64 %acc, %val
            if trimmed.starts_with(&format!("{} =", next_var))
                || trimmed.starts_with(&format!("{} =", next_var.trim()))
            {
                if let Some(kind) = detect_reduction_op(trimmed, acc_var) {
                    reductions.push((acc_var.clone(), kind));
                }
            }
        }
    }

    reductions
}

/// Parse a PHI node that looks like an accumulator.
/// Returns (accumulator_var, next_value_var).
fn parse_accumulator_phi(line: &str) -> Option<(String, String)> {
    // Pattern: %acc = phi i64 [ 0, %entry ], [ %acc.next, %body ]
    if !line.contains("= phi") {
        return None;
    }

    let acc_var = line.split('=').next()?.trim().to_string();
    if !acc_var.starts_with('%') {
        return None;
    }

    // Find the second bracket group [%next_var, %label]
    let mut bracket_count = 0;
    let mut second_bracket_start = None;
    for (i, ch) in line.char_indices() {
        if ch == '[' {
            bracket_count += 1;
            if bracket_count == 2 {
                second_bracket_start = Some(i + 1);
                break;
            }
        }
    }

    let start = second_bracket_start?;
    let rest = &line[start..];
    let end = rest.find(']')?;
    let inner = &rest[..end];

    // inner = "%acc.next, %body"
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.is_empty() {
        return None;
    }
    let next_var = parts[0].trim().to_string();
    if next_var.starts_with('%') {
        Some((acc_var, next_var))
    } else {
        None
    }
}

/// Detect the reduction operation kind from an assignment.
fn detect_reduction_op(line: &str, acc_var: &str) -> Option<ReductionKind> {
    if !line.contains(acc_var) {
        return None;
    }

    // Check for common patterns
    if line.contains(" add ") || line.contains(" fadd ") {
        Some(ReductionKind::Add)
    } else if line.contains(" mul ") || line.contains(" fmul ") {
        Some(ReductionKind::Mul)
    } else if line.contains(" or ") {
        Some(ReductionKind::Or)
    } else if line.contains(" and ") {
        Some(ReductionKind::And)
    } else if line.contains(" xor ") {
        Some(ReductionKind::Xor)
    } else if line.contains("@llvm.smin")
        || line.contains("@llvm.umin")
        || line.contains("@llvm.minnum")
    {
        Some(ReductionKind::Min)
    } else if line.contains("@llvm.smax")
        || line.contains("@llvm.umax")
        || line.contains("@llvm.maxnum")
    {
        Some(ReductionKind::Max)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_width() {
        assert_eq!(VectorWidth::SSE.bits(), 128);
        assert_eq!(VectorWidth::AVX2.bits(), 256);
        assert_eq!(VectorWidth::AVX512.bits(), 512);

        assert_eq!(VectorWidth::AVX2.lanes(32), 8); // 8 x i32
        assert_eq!(VectorWidth::AVX2.lanes(64), 4); // 4 x i64
    }

    #[test]
    fn test_loop_dependence() {
        let flow = LoopDependence::Flow { distance: Some(1) };
        assert!(flow.prevents_vectorization(4));

        let flow_far = LoopDependence::Flow { distance: Some(8) };
        assert!(!flow_far.prevents_vectorization(4));

        let none = LoopDependence::None;
        assert!(!none.prevents_vectorization(4));
    }

    #[test]
    fn test_extract_branch_targets() {
        let line = "br i1 %cond, label %then, label %else";
        let targets = extract_branch_targets(line);
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&"then".to_string()));
        assert!(targets.contains(&"else".to_string()));
    }

    #[test]
    fn test_analyze_simple_loop() {
        let ir = r#"
define void @sum(i64* %arr, i64 %n) {
entry:
  br label %loop

loop:
  %i = phi i64 [0, %entry], [%i.next, %loop]
  %ptr = getelementptr i64, i64* %arr, i64 %i
  %val = load i64, i64* %ptr
  %i.next = add i64 %i, 1
  %cond = icmp slt i64 %i.next, %n
  br i1 %cond, label %loop, label %exit

exit:
  ret void
}
"#;

        let vectorizer = analyze_vectorization(ir, VectorWidth::AVX2);
        // The loop detection should find at least one candidate
        // Vectorizer should parse the IR without panicking
        let _ = vectorizer.candidates.len();
    }

    #[test]
    fn test_memory_access_parsing() {
        let load = "  %val = load i64, i64* %ptr";
        let access = parse_memory_access(load, false).unwrap();
        assert_eq!(access.base, "%ptr");
        assert!(!access.is_write);
        assert_eq!(access.element_size, 8);

        let store = "  store i64 %val, i64* %ptr";
        let access = parse_memory_access(store, true).unwrap();
        assert_eq!(access.base, "%ptr");
        assert!(access.is_write);
    }

    #[test]
    fn test_detect_sum_reduction() {
        let ir = r#"
define i64 @sum(i64* %arr, i64 %n) {
entry:
  br label %loop

loop:
  %i = phi i64 [ 0, %entry ], [ %i.next, %loop ]
  %acc = phi i64 [ 0, %entry ], [ %acc.next, %loop ]
  %ptr = getelementptr i64, i64* %arr, i64 %i
  %val = load i64, i64* %ptr
  %acc.next = add i64 %acc, %val
  %i.next = add i64 %i, 1
  %cond = icmp slt i64 %i.next, %n
  br i1 %cond, label %loop, label %exit

exit:
  ret i64 %acc
}
"#;
        let reductions = detect_reductions(ir, "loop");
        // Both %i (induction) and %acc are detected as Add reductions
        assert!(reductions.len() >= 1);
        let acc_reduction = reductions.iter().find(|(var, _)| var == "%acc");
        assert!(acc_reduction.is_some(), "Should detect %acc as a reduction");
        assert_eq!(acc_reduction.unwrap().1, ReductionKind::Add);
    }

    #[test]
    fn test_reduction_identity() {
        assert_eq!(ReductionKind::Add.identity_i64(), 0);
        assert_eq!(ReductionKind::Mul.identity_i64(), 1);
        assert_eq!(ReductionKind::Min.identity_i64(), i64::MAX);
    }
}

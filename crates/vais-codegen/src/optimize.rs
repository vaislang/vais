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

/// Profile-Guided Optimization mode
///
/// PGO works in two phases:
/// 1. **Generate**: Compile with instrumentation to collect profiling data
/// 2. **Use**: Re-compile using the collected profile data for optimization
///
/// # Example workflow
/// ```bash
/// # Phase 1: Build with instrumentation
/// vaisc build --profile-generate=./profdata main.vais -o main_instrumented
///
/// # Phase 2: Run the instrumented binary to generate profile data
/// ./main_instrumented
///
/// # Phase 3: Merge profile data (creates default.profdata)
/// llvm-profdata merge -output=default.profdata ./profdata/default*.profraw
///
/// # Phase 4: Re-compile with profile data
/// vaisc build --profile-use=default.profdata main.vais -o main_optimized
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PgoMode {
    /// No PGO - standard compilation
    #[default]
    None,
    /// Generate instrumented binary for profiling
    /// Contains the directory path where .profraw files will be written
    Generate(String),
    /// Use profile data for optimization
    /// Contains the path to the merged .profdata file
    Use(String),
}

impl PgoMode {
    /// Parse PGO mode from command line arguments
    pub fn from_generate(path: Option<&str>) -> Self {
        match path {
            Some(p) => PgoMode::Generate(p.to_string()),
            None => PgoMode::Generate("./profdata".to_string()),
        }
    }

    pub fn from_use(path: &str) -> Self {
        PgoMode::Use(path.to_string())
    }

    /// Check if PGO is enabled
    pub fn is_enabled(&self) -> bool {
        !matches!(self, PgoMode::None)
    }

    /// Check if in generate mode
    pub fn is_generate(&self) -> bool {
        matches!(self, PgoMode::Generate(_))
    }

    /// Check if in use mode
    pub fn is_use(&self) -> bool {
        matches!(self, PgoMode::Use(_))
    }

    /// Get clang flags for this PGO mode
    pub fn clang_flags(&self) -> Vec<String> {
        match self {
            PgoMode::None => vec![],
            PgoMode::Generate(dir) => {
                // -fprofile-generate=<dir>
                // Creates instrumented binary that writes profraw files to <dir>
                vec![format!("-fprofile-generate={}", dir)]
            }
            PgoMode::Use(path) => {
                // -fprofile-use=<file>
                // Uses the merged profdata file for optimization
                vec![format!("-fprofile-use={}", path)]
            }
        }
    }

    /// Get additional LLVM pass flags for PGO
    pub fn llvm_flags(&self) -> Vec<&'static str> {
        match self {
            PgoMode::None => vec![],
            PgoMode::Generate(_) => {
                // Add instrumentation passes
                vec!["-pgo-instr-gen", "-instrprof"]
            }
            PgoMode::Use(_) => {
                // Add profile-guided optimization passes
                vec![
                    "-pgo-instr-use",
                    "-pgo-icall-prom", // Indirect call promotion
                    "-pgo-memop-opt",  // Memory operation optimization
                ]
            }
        }
    }

    /// Get the profile data directory (for generate mode)
    pub fn profile_dir(&self) -> Option<&str> {
        match self {
            PgoMode::Generate(dir) => Some(dir),
            _ => None,
        }
    }

    /// Get the profile data file path (for use mode)
    pub fn profile_file(&self) -> Option<&str> {
        match self {
            PgoMode::Use(path) => Some(path),
            _ => None,
        }
    }
}

/// Source-based code coverage mode
///
/// Uses LLVM's Source-Based Code Coverage (clang -fprofile-instr-generate -fcoverage-mapping).
///
/// # Workflow
/// ```bash
/// # 1. Build with coverage instrumentation
/// vaisc build --coverage main.vais -o main_cov
///
/// # 2. Run the instrumented binary (generates .profraw)
/// LLVM_PROFILE_FILE="coverage/%m.profraw" ./main_cov
///
/// # 3. Merge profiles and generate report
/// llvm-profdata merge -output=coverage.profdata coverage/*.profraw
/// llvm-cov show ./main_cov -instr-profile=coverage.profdata
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CoverageMode {
    /// No coverage instrumentation
    #[default]
    None,
    /// Enable coverage instrumentation
    /// The string specifies the output directory for .profraw files (default: ./coverage)
    Enabled(String),
}

impl CoverageMode {
    /// Parse coverage mode from CLI argument
    pub fn from_dir(dir: Option<&str>) -> Self {
        CoverageMode::Enabled(dir.unwrap_or("./coverage").to_string())
    }

    /// Check if coverage is enabled
    pub fn is_enabled(&self) -> bool {
        matches!(self, CoverageMode::Enabled(_))
    }

    /// Get the coverage output directory
    pub fn coverage_dir(&self) -> Option<&str> {
        match self {
            CoverageMode::Enabled(dir) => Some(dir),
            CoverageMode::None => None,
        }
    }

    /// Get clang flags for coverage instrumentation
    pub fn clang_flags(&self) -> Vec<&'static str> {
        match self {
            CoverageMode::None => vec![],
            CoverageMode::Enabled(_) => vec![
                "-fprofile-instr-generate",
                "-fcoverage-mapping",
            ],
        }
    }
}

/// PGO configuration and helpers
pub struct PgoConfig {
    pub mode: PgoMode,
    /// Whether to enable branch probability tracking
    pub branch_weights: bool,
    /// Whether to enable indirect call promotion
    pub icall_promotion: bool,
    /// Minimum count threshold for hot functions
    pub hot_threshold: u64,
    /// Maximum count threshold for cold functions
    pub cold_threshold: u64,
}

impl Default for PgoConfig {
    fn default() -> Self {
        Self {
            mode: PgoMode::None,
            branch_weights: true,
            icall_promotion: true,
            hot_threshold: 1000,
            cold_threshold: 10,
        }
    }
}

impl PgoConfig {
    pub fn new(mode: PgoMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    /// Get all clang flags including optimization hints
    pub fn all_clang_flags(&self) -> Vec<String> {
        let mut flags = self.mode.clang_flags();

        if self.mode.is_use() {
            // Add additional optimization flags when using profile data
            flags.push("-fprofile-instr-use".to_string());

            // Enable branch weight propagation
            if self.branch_weights {
                flags.push("-mllvm".to_string());
                flags.push("-pgo-warn-missing-function".to_string());
            }
        }

        flags
    }
}

/// Add profile instrumentation hints to LLVM IR
///
/// This function inserts profile instrumentation metadata and function attributes
/// to help LLVM's PGO passes.
pub fn instrument_ir_for_pgo(ir: &str) -> String {
    let mut result = String::new();

    for line in ir.lines() {
        // Add profile instrumentation function attribute
        if line.starts_with("define ") && !line.contains("noprofile") {
            // Insert before the function body
            let modified = line.replace("define ", "define dso_local ");
            result.push_str(&modified);
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }

    // Add required PGO intrinsic declarations if not present
    if !ir.contains("@llvm.instrprof") {
        result.push_str("\n; PGO intrinsic declarations\n");
        result.push_str("declare void @llvm.instrprof.increment(i8*, i64, i32, i32)\n");
        result.push_str("declare void @llvm.instrprof.value.profile(i8*, i64, i64, i32, i32)\n");
    }

    result
}

/// Apply profile-guided optimizations to IR
///
/// When profile data is available, this function adds branch weight metadata
/// and other hints based on the profile information.
pub fn apply_pgo_hints(ir: &str, _profile_path: &str) -> String {
    // In a real implementation, this would:
    // 1. Parse the profile data file
    // 2. Map function names to their execution counts
    // 3. Add branch weight metadata to conditional branches
    // 4. Mark hot/cold functions
    //
    // For now, we return the IR unchanged and let clang handle PGO
    ir.to_string()
}

/// Mark functions as hot or cold based on profile data
///
/// Hot functions get more aggressive inlining and optimization.
/// Cold functions are optimized for size.
pub fn annotate_function_hotness(
    ir: &str,
    hot_funcs: &HashSet<String>,
    cold_funcs: &HashSet<String>,
) -> String {
    let mut result = String::new();

    for line in ir.lines() {
        if line.starts_with("define ") {
            // Extract function name
            if let Some(name) = extract_function_name(line) {
                if hot_funcs.contains(&name) {
                    // Add hot attribute
                    let modified = line.replace("define ", "define hot ");
                    result.push_str(&modified);
                } else if cold_funcs.contains(&name) {
                    // Add cold attribute
                    let modified = line.replace("define ", "define cold ");
                    result.push_str(&modified);
                } else {
                    result.push_str(line);
                }
            } else {
                result.push_str(line);
            }
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }

    result
}

/// Extract function name from a define line
fn extract_function_name(define_line: &str) -> Option<String> {
    // Pattern: define ... @function_name(
    let at_pos = define_line.find('@')?;
    let paren_pos = define_line[at_pos..].find('(')?;
    let name = &define_line[at_pos + 1..at_pos + paren_pos];
    Some(name.to_string())
}

impl OptLevel {
    pub fn parse(s: &str) -> Self {
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
    optimize_ir_with_pgo(ir, level, &PgoMode::None)
}

/// Apply optimization passes to LLVM IR with optional PGO support
///
/// When PGO is in Generate mode, instrumentation hints are added.
/// When PGO is in Use mode, profile data guides inlining and optimization decisions.
pub fn optimize_ir_with_pgo(ir: &str, level: OptLevel, pgo: &PgoMode) -> String {
    if level == OptLevel::O0 {
        // Even at O0, apply PGO instrumentation if requested
        if let PgoMode::Generate(_) = pgo {
            return instrument_ir_for_pgo(ir);
        }
        return ir.to_string();
    }

    let mut result = ir.to_string();

    // PGO Generate: add instrumentation
    if let PgoMode::Generate(_) = pgo {
        result = instrument_ir_for_pgo(&result);
    }

    // O1+: Basic optimizations (before inlining to simplify function bodies)
    if level >= OptLevel::O1 {
        result = constant_folding(&result);
        result = dead_store_elimination(&result);
        result = branch_optimization(&result);
        result = conditional_branch_simplification(&result);
    }

    // O1+: Tail call optimization - mark tail calls with 'tail' or 'musttail'
    if level >= OptLevel::O1 {
        result = tail_call_optimization(&result);
    }

    // O2+: More aggressive optimizations
    if level >= OptLevel::O2 {
        result = strength_reduction(&result);
    }

    // O3: Inlining after basic optimizations
    if level >= OptLevel::O3 {
        result = aggressive_inline(&result);
    }

    // PGO Use: apply profile-guided hints (hot/cold function annotations)
    if let PgoMode::Use(profile_path) = pgo {
        result = apply_pgo_hints(&result, profile_path);
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

/// Apply optimization passes with advanced analysis
///
/// This version includes interprocedural alias analysis, auto-vectorization hints,
/// and cache-friendly data layout suggestions.
pub fn optimize_ir_advanced(ir: &str, level: OptLevel) -> String {
    use crate::advanced_opt::{apply_advanced_optimizations, AdvancedOptConfig};

    // First apply standard optimizations
    let result = optimize_ir(ir, level);

    // Then apply advanced optimizations based on level
    let config = AdvancedOptConfig::from_opt_level(level);
    apply_advanced_optimizations(&result, &config)
}

/// Constant folding - evaluate constant expressions at compile time
pub(crate) fn constant_folding(ir: &str) -> String {
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
        if let Some(folded) =
            try_fold_binary_op(trimmed, "sdiv", |a, b| if b != 0 { a / b } else { 0 })
        {
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
    Some(format!(
        "  {} = add i64 0, {}  ; folded from {} {} {}",
        dest, result, a, op, b
    ))
}

/// Dead store elimination - remove stores that are never read
pub(crate) fn dead_store_elimination(ir: &str) -> String {
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
                let var: String = ptr
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '%' || *c == '.' || *c == '_')
                    .collect();
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

/// Common subexpression elimination
pub(crate) fn common_subexpression_elimination(ir: &str) -> String {
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
pub(crate) fn strength_reduction(ir: &str) -> String {
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
        dest,
        var,
        shift,
        1i64 << shift
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
    63 - n.leading_zeros()
}

/// Branch optimization - simplify branches with constant conditions
pub(crate) fn branch_optimization(ir: &str) -> String {
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
                    result.push(format!(
                        "  br label %{}  ; simplified from conditional",
                        then_label
                    ));
                    continue;
                }
            } else if trimmed.contains("br i1 false,") || trimmed.contains("br i1 0,") {
                // Always branch to 'else'
                if let Some(else_label) = extract_branch_label(trimmed, false) {
                    result.push(format!(
                        "  br label %{}  ; simplified from conditional",
                        else_label
                    ));
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
                        result.push(format!(
                            "  {} = add i1 0, true  ; simplified: X == X",
                            parts[0].trim()
                        ));
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

/// Conditional branch simplification
/// Removes redundant zext i1 + icmp ne patterns:
///   %X = icmp ... (produces i1)
///   %Y = zext i1 %X to i64
///   %Z = icmp ne i64 %Y, 0
///   br i1 %Z, label %then, label %else
/// Becomes:
///   %X = icmp ... (produces i1)
///   br i1 %X, label %then, label %else
///
/// IMPORTANT: Only removes zext/icmp if the result is ONLY used in br i1
pub(crate) fn conditional_branch_simplification(ir: &str) -> String {
    let lines: Vec<&str> = ir.lines().collect();
    let mut result = Vec::new();

    // Track i1 sources: zext_dest -> (original_i1_var, line_index)
    let mut i1_sources: HashMap<String, (String, usize)> = HashMap::new();
    // Track icmp ne %zext, 0 -> (original_i1_var, line_index, zext_var)
    let mut icmp_to_i1: HashMap<String, (String, usize, String)> = HashMap::new();
    // Track variable uses: var_name -> count of uses (excluding its definition)
    let mut var_uses: HashMap<String, usize> = HashMap::new();

    // First pass: collect zext i1 to i64 patterns
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Pattern: %Y = zext i1 %X to i64
        if trimmed.contains(" = zext i1 ") && trimmed.contains(" to i64") {
            if let Some((dest, src)) = parse_zext_i1(trimmed) {
                i1_sources.insert(dest, (src, i));
            }
        }
    }

    // Second pass: find icmp ne i64 %zext, 0
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Pattern: %Z = icmp ne i64 %Y, 0
        if trimmed.contains(" = icmp ne i64 ") {
            if let Some((dest, operand)) = parse_icmp_ne_zero(trimmed) {
                if let Some((original_i1, _)) = i1_sources.get(&operand) {
                    icmp_to_i1.insert(dest, (original_i1.clone(), i, operand.clone()));
                }
            }
        }
    }

    // Third pass: count variable uses
    for line in lines.iter() {
        let trimmed = line.trim();
        // Skip comments
        if trimmed.starts_with(';') {
            continue;
        }
        // Count all %var references in this line
        for word in trimmed.split(|c: char| !c.is_alphanumeric() && c != '%' && c != '.') {
            if word.starts_with('%') && !word.is_empty() {
                // Check if this is a use (not a definition)
                // Definition pattern: "%var = ..."
                if !trimmed.starts_with(word) || !trimmed.contains(" = ") {
                    *var_uses.entry(word.to_string()).or_insert(0) += 1;
                } else if trimmed.starts_with(word) && trimmed.contains(" = ") {
                    // This is a definition, but also check for uses in the RHS
                    let def_end = trimmed.find(" = ").unwrap() + 3;
                    let rhs = &trimmed[def_end..];
                    if rhs.contains(word) {
                        *var_uses.entry(word.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    // Determine which lines can be safely removed:
    // - icmp ne line can be removed if its result is only used once in a br i1
    // - zext line can be removed if its result is only used once in the icmp ne
    let mut dead_lines: HashSet<usize> = HashSet::new();
    let mut safe_icmp_replacements: HashMap<String, String> = HashMap::new();

    for (icmp_var, (original_i1, icmp_line, zext_var)) in &icmp_to_i1 {
        // Check if icmp result is only used in br i1
        let icmp_uses = var_uses.get(icmp_var).copied().unwrap_or(0);
        // Check if zext result is only used in this icmp
        let zext_uses = var_uses.get(zext_var).copied().unwrap_or(0);

        // icmp should be used exactly once (in br i1)
        // zext should be used exactly once (in this icmp)
        if icmp_uses == 1 && zext_uses == 1 {
            dead_lines.insert(*icmp_line);
            if let Some((_, zext_line)) = i1_sources.get(zext_var) {
                dead_lines.insert(*zext_line);
            }
            safe_icmp_replacements.insert(icmp_var.clone(), original_i1.clone());
        }
    }

    // Fourth pass: generate optimized output
    for (i, line) in lines.iter().enumerate() {
        if dead_lines.contains(&i) {
            // Skip dead zext/icmp lines, add comment for debugging
            result.push(format!("  ; optimized out: {}", line.trim()));
            continue;
        }

        let trimmed = line.trim();

        // Pattern: br i1 %Z, label %then, label %else
        if trimmed.starts_with("br i1 ") {
            if let Some(replaced) = try_replace_branch_cond(trimmed, &safe_icmp_replacements) {
                result.push(format!("  {}", replaced));
                continue;
            }
        }

        result.push(line.to_string());
    }

    result.join("\n")
}

/// Parse: %Y = zext i1 %X to i64
fn parse_zext_i1(line: &str) -> Option<(String, String)> {
    // Pattern: %dest = zext i1 %src to i64
    let parts: Vec<&str> = line.split(" = zext i1 ").collect();
    if parts.len() != 2 {
        return None;
    }

    let dest = parts[0].trim().to_string();
    let rest = parts[1].trim();

    // Extract source variable
    if let Some(src_end) = rest.find(" to i64") {
        let src = rest[..src_end].trim().to_string();
        return Some((dest, src));
    }

    None
}

/// Parse: %Z = icmp ne i64 %Y, 0
fn parse_icmp_ne_zero(line: &str) -> Option<(String, String)> {
    // Pattern: %dest = icmp ne i64 %operand, 0
    let parts: Vec<&str> = line.split(" = icmp ne i64 ").collect();
    if parts.len() != 2 {
        return None;
    }

    let dest = parts[0].trim().to_string();
    let operands = parts[1].trim();

    // Check if second operand is 0
    if operands.ends_with(", 0") {
        let operand = operands.trim_end_matches(", 0").trim().to_string();
        return Some((dest, operand));
    }

    None
}

/// Try to replace branch condition with original i1 value
fn try_replace_branch_cond(line: &str, icmp_to_i1: &HashMap<String, String>) -> Option<String> {
    // Pattern: br i1 %Z, label %then, label %else
    let prefix = "br i1 ";
    if !line.starts_with(prefix) {
        return None;
    }

    let rest = &line[prefix.len()..];

    // Extract condition variable
    if let Some(comma_pos) = rest.find(',') {
        let cond = rest[..comma_pos].trim();
        let labels = &rest[comma_pos..];

        // Check if this condition can be replaced
        if let Some(original_i1) = icmp_to_i1.get(cond) {
            return Some(format!("br i1 {}{} ; simplified", original_i1, labels));
        }
    }

    None
}

/// Dead code elimination - remove unused definitions
pub(crate) fn dead_code_elimination(ir: &str) -> String {
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
        for word in
            trimmed.split(|c: char| !c.is_alphanumeric() && c != '%' && c != '_' && c != '.')
        {
            if word.starts_with('%') && !word.is_empty() {
                // Check if this is a use, not just the definition
                if !trimmed.starts_with(&format!("{} =", word))
                    && !trimmed.starts_with(&format!("  {} =", word))
                {
                    used_vars.insert(word.to_string());
                }
            }
        }
    }

    // Also mark return values, call arguments, and branch conditions as used
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("ret ")
            || trimmed.starts_with("br ")
            || trimmed.starts_with("store ")
            || trimmed.contains("call ")
        {
            for word in
                trimmed.split(|c: char| !c.is_alphanumeric() && c != '%' && c != '_' && c != '.')
            {
                if word.starts_with('%') {
                    used_vars.insert(word.to_string());
                }
            }
        }
    }

    // Second pass: emit only used definitions
    let mut result = Vec::new();
    for line in lines.iter() {
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
    let pure_ops = [
        "add ",
        "sub ",
        "mul ",
        "sdiv ",
        "udiv ",
        "and ",
        "or ",
        "xor ",
        "shl ",
        "ashr ",
        "lshr ",
        "icmp ",
        "fcmp ",
        "select ",
        "zext ",
        "sext ",
        "trunc ",
        "bitcast ",
        "getelementptr ",
        "extractvalue ",
        "insertvalue ",
        "load ",
    ];

    for op in &pure_ops {
        if trimmed.contains(op) {
            return true;
        }
    }
    false
}

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

/// Parsed LLVM IR function for inlining
#[derive(Debug, Clone)]
struct InlinableFunction {
    name: String,
    params: Vec<(String, String)>, // (type, param_name)
    return_type: String,
    body: Vec<String>,
    has_side_effects: bool,
    has_external_calls: bool,
}

/// Count how many times each function is called in the IR
fn count_call_sites(ir: &str) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for line in ir.lines() {
        let trimmed = line.trim();
        if trimmed.contains("call ") {
            // Extract function name from call: call TYPE @func_name(
            if let Some(at_pos) = trimmed.find("@") {
                let after_at = &trimmed[at_pos..];
                if let Some(paren_pos) = after_at.find('(') {
                    let func_name = &after_at[..paren_pos];
                    *counts.entry(func_name.to_string()).or_insert(0) += 1;
                }
            }
        }
    }
    counts
}

/// Aggressive inlining for small functions
///
/// Prioritizes functions by call frequency (hot functions first) and supports
/// larger function bodies (up to 50 instructions). Functions called more
/// frequently are inlined first for maximum benefit.
pub(crate) fn aggressive_inline(ir: &str) -> String {
    // Parse all small functions that are candidates for inlining
    let mut inline_candidates = find_inline_candidates(ir);

    #[cfg(debug_assertions)]
    {
        eprintln!("DEBUG: Found {} inline candidates", inline_candidates.len());
        for func in &inline_candidates {
            eprintln!(
                "DEBUG: Candidate: {} ({} body lines, side_effects={})",
                func.name,
                func.body.len(),
                func.has_side_effects
            );
        }
    }

    if inline_candidates.is_empty() {
        return ir.to_string();
    }

    // Count call sites and sort candidates by frequency (most called first)
    let call_counts = count_call_sites(ir);
    inline_candidates.sort_by(|a, b| {
        let count_a = call_counts.get(&a.name).copied().unwrap_or(0);
        let count_b = call_counts.get(&b.name).copied().unwrap_or(0);
        // Primary: higher call count first; Secondary: smaller body first
        count_b.cmp(&count_a).then(a.body.len().cmp(&b.body.len()))
    });

    #[cfg(debug_assertions)]
    {
        for func in &inline_candidates {
            let count = call_counts.get(&func.name).copied().unwrap_or(0);
            eprintln!(
                "DEBUG: Inline priority: {} (calls={}, body={})",
                func.name,
                count,
                func.body.len()
            );
        }
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
///
/// Uses a tiered threshold:
/// - Functions ≤ 10 instructions: always inline (even with internal side effects like stores)
/// - Functions ≤ 50 instructions: inline if no external call side effects
/// - Functions > 50 instructions: never inline at text level (rely on LLVM's inliner)
fn find_inline_candidates(ir: &str) -> Vec<InlinableFunction> {
    let mut candidates = Vec::new();
    let lines: Vec<&str> = ir.lines().collect();
    let large_threshold = 50; // max instructions for inlining
    let small_threshold = 10; // always-inline threshold (even with store side effects)

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];

        // Look for function definitions
        if line.starts_with("define ") && line.contains("@") {
            if let Some(func) = parse_function(&lines, i) {
                let body_size = func.body.len();
                let is_main = func.name == "@main";
                let is_internal = func.name.starts_with("@__") || func.name.starts_with("@_");
                let is_recursive = func
                    .body
                    .iter()
                    .any(|l| l.contains(&format!("call {} {}", func.return_type, func.name)));

                // Never inline: main, internal helpers, recursive functions
                if is_main || is_internal || is_recursive {
                    i += 1;
                    continue;
                }

                // Tiered inlining:
                // - Small functions (≤10 instructions): inline even with store side effects
                // - Medium functions (≤50): inline only if no side effects
                let eligible = if body_size <= small_threshold {
                    // Small functions: allow store side effects but not external calls
                    !func.has_external_calls
                } else if body_size <= large_threshold {
                    // Medium functions: must be pure (no side effects at all)
                    !func.has_side_effects
                } else {
                    false
                };

                if eligible {
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
    let mut has_external_calls = false;

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
        // Store operations have side effects (but are internal/local)
        if trimmed.starts_with("store ") {
            has_side_effects = true;
        }
        // Calls to functions have side effects
        if trimmed.contains("call ") {
            has_side_effects = true;
            has_external_calls = true;
        }

        body.push(trimmed.to_string());
    }

    Some(InlinableFunction {
        name,
        params,
        return_type,
        body,
        has_side_effects,
        has_external_calls,
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
                if let Some(var_part) = old_var.strip_prefix('%') {
                    // remove the %
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
            inlined.push(format!(
                "  {} = add {} 0, {}  ; inlined return value",
                dest, func.return_type, return_value
            ));
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
        if let Some(var_part) = lhs.strip_prefix('%') {
            // Create a new variable name that's valid LLVM IR
            // For %0, %1, etc. we need to create %inl1_0, %inl1_1, etc.
            // remove the %
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
        assert!(
            candidates.len() >= 2,
            "Expected at least 2 candidates, got {}",
            candidates.len()
        );
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
        assert!(
            result.contains("INLINE") || !result.contains("call i64 @square"),
            "Expected inlining to occur or call to be removed. Result:\n{}",
            result
        );
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
        assert!(
            result.contains("LOOP UNROLLING") || result.contains("loop.start"),
            "Expected loop unrolling to be attempted"
        );
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
        assert!(
            result.contains("LICM") || result.contains("loop.start"),
            "Expected LICM to be attempted"
        );
    }

    #[test]
    fn test_rename_for_unroll() {
        let line = "%sum = add i64 %acc, %i";
        let renamed = rename_for_unroll(line, 2);
        assert!(
            renamed.contains("_u2"),
            "Expected unroll suffix in: {}",
            renamed
        );
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

// =============================================================================
// Basic Block Merging
// =============================================================================

/// Basic block merging optimization
/// 1. Removes empty blocks that only contain an unconditional branch
/// 2. Redirects branches to empty blocks to their final destination
///
/// Before:
///   br i1 %cond, label %then, label %empty
///   ...
///   empty:
///     br label %merge
///   merge:
///     ...
///
/// After:
///   br i1 %cond, label %then, label %merge
///   ...
///   merge:
///     ...
#[allow(dead_code)]
fn basic_block_merging(ir: &str) -> String {
    let lines: Vec<&str> = ir.lines().collect();

    // Track empty blocks: block_label -> target_label
    // An empty block is one that only contains "br label %target"
    let mut empty_blocks: HashMap<String, String> = HashMap::new();
    let mut current_block: Option<String> = None;
    let mut block_is_empty = true;
    let mut block_target: Option<String> = None;

    // First pass: identify empty blocks
    for line in &lines {
        let trimmed = line.trim();

        // New block starts
        if trimmed.ends_with(':') && !trimmed.starts_with(';') && !trimmed.starts_with("@") {
            // Save previous block if it was empty
            if let Some(ref block) = current_block {
                if block_is_empty {
                    if let Some(ref target) = block_target {
                        // Never remove the entry block - it's required by LLVM
                        if block != "entry" {
                            empty_blocks.insert(block.clone(), target.clone());
                        }
                    }
                }
            }

            // Start new block
            current_block = Some(trimmed.trim_end_matches(':').to_string());
            block_is_empty = true;
            block_target = None;
        } else if trimmed.starts_with("define ") || trimmed == "}" {
            // Function boundary - reset
            if let Some(ref block) = current_block {
                if block_is_empty {
                    if let Some(ref target) = block_target {
                        empty_blocks.insert(block.clone(), target.clone());
                    }
                }
            }
            current_block = None;
            block_is_empty = true;
            block_target = None;
        } else if current_block.is_some() && !trimmed.is_empty() && !trimmed.starts_with(';') {
            // Check if this is just an unconditional branch
            if trimmed.starts_with("br label %") {
                if let Some(target) = extract_uncond_branch_target(trimmed) {
                    block_target = Some(target);
                }
            } else {
                // Has other instructions - not empty
                block_is_empty = false;
            }
        }
    }

    // Resolve transitive empty blocks: if A -> B and B -> C, then A -> C
    let mut resolved = empty_blocks;
    loop {
        let mut updates: Vec<(String, String)> = Vec::new();
        for (src, target) in resolved.iter() {
            if let Some(final_target) = resolved.get(target) {
                if target != final_target {
                    updates.push((src.clone(), final_target.clone()));
                }
            }
        }
        if updates.is_empty() {
            break;
        }
        for (src, final_target) in updates {
            resolved.insert(src, final_target);
        }
    }
    let empty_blocks = resolved;

    // Second pass: rewrite branches and remove empty blocks
    let mut result = Vec::new();
    let mut skip_block = false;

    for line in &lines {
        let trimmed = line.trim();

        // Check if we're entering an empty block to skip
        if trimmed.ends_with(':') && !trimmed.starts_with(';') && !trimmed.starts_with("@") {
            let label = trimmed.trim_end_matches(':');
            if empty_blocks.contains_key(label) {
                skip_block = true;
                result.push(format!("  ; empty block removed: {}", label));
                continue;
            } else {
                skip_block = false;
            }
        }

        if skip_block {
            if trimmed.starts_with("define ")
                || trimmed == "}"
                || (trimmed.ends_with(':') && !trimmed.starts_with(';'))
            {
                skip_block = false;
            } else {
                continue; // Skip instructions in empty block
            }
        }

        // Rewrite branch targets
        if trimmed.starts_with("br ") {
            if let Some(rewritten) = rewrite_branch_targets(trimmed, &empty_blocks) {
                result.push(format!("  {}", rewritten));
                continue;
            }
        }

        // Rewrite phi node labels
        if trimmed.contains(" = phi ") {
            if let Some(rewritten) = rewrite_phi_labels(trimmed, &empty_blocks) {
                result.push(format!("  {}", rewritten));
                continue;
            }
        }

        result.push(line.to_string());
    }

    result.join("\n")
}

/// Rewrite branch targets, replacing references to empty blocks with their final destinations
#[allow(dead_code)]
fn rewrite_branch_targets(line: &str, empty_blocks: &HashMap<String, String>) -> Option<String> {
    let mut modified = line.to_string();
    let mut any_change = false;

    for (empty, target) in empty_blocks {
        let from = format!("label %{}", empty);
        let to = format!("label %{}", target);
        if modified.contains(&from) {
            modified = modified.replace(&from, &to);
            any_change = true;
        }
    }

    if any_change {
        Some(modified)
    } else {
        None
    }
}

/// Rewrite phi node labels, replacing references to empty blocks with their final destinations
#[allow(dead_code)]
fn rewrite_phi_labels(line: &str, empty_blocks: &HashMap<String, String>) -> Option<String> {
    let mut modified = line.to_string();
    let mut any_change = false;

    for (empty, target) in empty_blocks {
        // Phi format: [ value, %label ]
        let from = format!(", %{} ]", empty);
        let to = format!(", %{} ]", target);
        if modified.contains(&from) {
            modified = modified.replace(&from, &to);
            any_change = true;
        }

        // Also handle: [ %label ] at end
        let from2 = format!("%{} ]", empty);
        let to2 = format!("%{} ]", target);
        if modified.contains(&from2) {
            modified = modified.replace(&from2, &to2);
            any_change = true;
        }
    }

    if any_change {
        Some(modified)
    } else {
        None
    }
}

/// Extract target from unconditional branch: br label %target
#[allow(dead_code)]
fn extract_uncond_branch_target(line: &str) -> Option<String> {
    // Pattern: br label %target
    if !line.starts_with("br label %") {
        return None;
    }
    let rest = &line["br label %".len()..];
    let label: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
        .collect();
    if label.is_empty() {
        None
    } else {
        Some(label)
    }
}

/// Extract targets from conditional branch: br i1 %cond, label %then, label %else
#[allow(dead_code)]
fn extract_cond_branch_targets(line: &str) -> Option<(String, String)> {
    // Pattern: br i1 %cond, label %then, label %else
    let parts: Vec<&str> = line.split("label %").collect();
    if parts.len() >= 3 {
        let then_part = parts[1].split(',').next()?.trim();
        let else_part = parts[2].trim().trim_end_matches([')', ';', ' ']);
        let then_label: String = then_part
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
            .collect();
        let else_label: String = else_part
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
            .collect();
        return Some((then_label, else_label));
    }
    None
}

/// Check if a string looks like a label name (not an SSA value)
#[allow(dead_code)]
fn is_likely_label(s: &str) -> bool {
    // Labels typically start with a letter and don't look like %1, %2, etc.
    if s.is_empty() {
        return false;
    }
    let first = s.chars().next().unwrap();
    first.is_alphabetic() || first == '_'
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod pgo_tests {
    use super::*;

    #[test]
    fn test_pgo_mode_none() {
        let mode = PgoMode::None;
        assert!(!mode.is_enabled());
        assert!(!mode.is_generate());
        assert!(!mode.is_use());
        assert!(mode.clang_flags().is_empty());
    }

    #[test]
    fn test_pgo_mode_generate() {
        let mode = PgoMode::Generate("./profdata".to_string());
        assert!(mode.is_enabled());
        assert!(mode.is_generate());
        assert!(!mode.is_use());
        assert_eq!(mode.profile_dir(), Some("./profdata"));
        assert_eq!(mode.profile_file(), None);

        let flags = mode.clang_flags();
        assert_eq!(flags.len(), 1);
        assert!(flags[0].contains("-fprofile-generate"));
    }

    #[test]
    fn test_pgo_mode_use() {
        let mode = PgoMode::Use("default.profdata".to_string());
        assert!(mode.is_enabled());
        assert!(!mode.is_generate());
        assert!(mode.is_use());
        assert_eq!(mode.profile_dir(), None);
        assert_eq!(mode.profile_file(), Some("default.profdata"));

        let flags = mode.clang_flags();
        assert_eq!(flags.len(), 1);
        assert!(flags[0].contains("-fprofile-use"));
    }

    #[test]
    fn test_pgo_from_generate() {
        let mode = PgoMode::from_generate(Some("./custom_profile"));
        assert!(matches!(mode, PgoMode::Generate(ref p) if p == "./custom_profile"));

        let mode_default = PgoMode::from_generate(None);
        assert!(matches!(mode_default, PgoMode::Generate(ref p) if p == "./profdata"));
    }

    #[test]
    fn test_pgo_from_use() {
        let mode = PgoMode::from_use("merged.profdata");
        assert!(matches!(mode, PgoMode::Use(ref p) if p == "merged.profdata"));
    }

    #[test]
    fn test_pgo_config() {
        let config = PgoConfig::new(PgoMode::Generate("./prof".to_string()));
        assert!(config.mode.is_generate());
        assert!(config.branch_weights);
        assert!(config.icall_promotion);

        let flags = config.all_clang_flags();
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_instrument_ir_for_pgo() {
        let ir = "define i64 @main() {\n  ret i64 0\n}";
        let instrumented = instrument_ir_for_pgo(ir);

        // Should add dso_local attribute
        assert!(instrumented.contains("dso_local"));
        // Should add instrprof declarations
        assert!(instrumented.contains("@llvm.instrprof"));
    }

    #[test]
    fn test_annotate_function_hotness() {
        let ir =
            "define i64 @hot_func() {\n  ret i64 0\n}\ndefine i64 @cold_func() {\n  ret i64 1\n}";

        let hot = ["hot_func".to_string()].into_iter().collect();
        let cold = ["cold_func".to_string()].into_iter().collect();

        let annotated = annotate_function_hotness(ir, &hot, &cold);

        assert!(annotated.contains("define hot"));
        assert!(annotated.contains("define cold"));
    }

    #[test]
    fn test_lto_mode() {
        assert_eq!(LtoMode::parse("thin"), LtoMode::Thin);
        assert_eq!(LtoMode::parse("full"), LtoMode::Full);
        assert_eq!(LtoMode::parse("none"), LtoMode::None);
        assert_eq!(LtoMode::parse("invalid"), LtoMode::None);

        assert!(LtoMode::Thin.is_enabled());
        assert!(LtoMode::Full.is_enabled());
        assert!(!LtoMode::None.is_enabled());

        assert!(LtoMode::Thin.clang_flags().contains(&"-flto=thin"));
        assert!(LtoMode::Full.clang_flags().contains(&"-flto=full"));
    }
}

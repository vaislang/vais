//! Profile-Guided Optimization (PGO) support

use std::collections::HashSet;

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
            CoverageMode::Enabled(_) => vec!["-fprofile-instr-generate", "-fcoverage-mapping"],
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
    let mut result = String::with_capacity(ir.len());

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
    let mut result = String::with_capacity(ir.len());

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
}

//! Parallel compilation support for Vais
//!
//! Provides utilities for parallelizing compilation stages:
//! - Parallel optimization passes (run independent IR passes per-function concurrently)
//! - Pipeline-level parallelism configuration
//!
//! Note: Parallel module parsing is handled in the CLI crate (vaisc) since it
//! depends on vais-lexer and vais-parser which are not dependencies of vais-codegen.

use rayon::prelude::*;

/// Configuration for parallel compilation
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of threads to use (0 = auto-detect based on CPU cores)
    pub num_threads: usize,
    /// Enable parallel module parsing
    pub parallel_parse: bool,
    /// Enable parallel optimization passes
    pub parallel_optimize: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            num_threads: 0, // auto-detect
            parallel_parse: true,
            parallel_optimize: true,
        }
    }
}

impl ParallelConfig {
    pub fn new(num_threads: usize) -> Self {
        Self {
            num_threads,
            parallel_parse: true,
            parallel_optimize: true,
        }
    }

    /// Initialize the global rayon thread pool with the configured thread count
    pub fn init_thread_pool(&self) -> Result<(), String> {
        if self.num_threads > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(self.num_threads)
                .build_global()
                .map_err(|e| format!("Failed to initialize thread pool: {}", e))?;
        }
        Ok(())
    }

    /// Get the effective thread count (resolves 0 to actual CPU count)
    pub fn effective_threads(&self) -> usize {
        if self.num_threads == 0 {
            rayon::current_num_threads()
        } else {
            self.num_threads
        }
    }
}

/// Split LLVM IR into per-function chunks for parallel processing
///
/// Returns the non-function preamble and a list of (function_name, function_ir) pairs
pub fn split_ir_into_functions(ir: &str) -> (String, Vec<(String, String)>) {
    let mut preamble = String::with_capacity(ir.len() / 4);
    let mut functions: Vec<(String, String)> = Vec::new();
    let mut current_fn_name: Option<String> = None;
    let mut current_fn_ir = String::with_capacity(1024);
    let mut in_function = false;
    let mut brace_depth: i32 = 0;

    for line in ir.lines() {
        let trimmed = line.trim();

        if !in_function {
            if trimmed.starts_with("define ") {
                in_function = true;
                brace_depth = 0;
                let name = extract_function_name(trimmed).unwrap_or_else(|| "unknown".to_string());
                current_fn_name = Some(name);
                current_fn_ir.clear();
                current_fn_ir.push_str(line);
                current_fn_ir.push('\n');
                brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
                brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;
                if brace_depth <= 0 {
                    in_function = false;
                    if let Some(name) = current_fn_name.take() {
                        functions.push((name, std::mem::take(&mut current_fn_ir)));
                    }
                }
            } else {
                preamble.push_str(line);
                preamble.push('\n');
            }
        } else {
            current_fn_ir.push_str(line);
            current_fn_ir.push('\n');
            brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;
            if brace_depth <= 0 {
                in_function = false;
                if let Some(name) = current_fn_name.take() {
                    functions.push((name, std::mem::take(&mut current_fn_ir)));
                }
            }
        }
    }

    if in_function {
        if let Some(name) = current_fn_name.take() {
            functions.push((name, current_fn_ir));
        }
    }

    (preamble, functions)
}

fn extract_function_name(line: &str) -> Option<String> {
    // Pattern: define <rettype> @<name>(...)
    let at_pos = line.find('@')?;
    let after_at = &line[at_pos + 1..];
    let end = after_at.find('(')?;
    Some(after_at[..end].to_string())
}

/// Apply an optimization pass to each function in parallel, then reassemble
///
/// Splits the IR into preamble + per-function chunks, applies the given pass
/// to each function concurrently via rayon, then reassembles the result.
pub fn parallel_optimize_functions<F>(ir: &str, pass: F) -> String
where
    F: Fn(&str) -> String + Send + Sync,
{
    let (preamble, functions) = split_ir_into_functions(ir);

    if functions.len() < 2 {
        // Not worth parallelizing for 0-1 functions
        return pass(ir);
    }

    let optimized_functions: Vec<String> = functions
        .par_iter()
        .map(|(_name, fn_ir)| pass(fn_ir))
        .collect();

    // Reassemble
    let mut result = preamble;
    for fn_ir in optimized_functions {
        result.push_str(&fn_ir);
        result.push('\n');
    }
    result
}

/// Parallel optimization pipeline that runs independent passes concurrently
///
/// Strategy:
/// - Per-function passes (constant folding, DSE, branch opt, etc.) are applied
///   to each function independently in parallel via rayon
/// - Global passes (inlining) are run sequentially since they need cross-function info
/// - Passes are grouped by their dependencies to maximize parallelism
pub fn parallel_optimize_ir(ir: &str, level: crate::optimize::OptLevel) -> String {
    use crate::optimize::*;

    if level == OptLevel::O0 {
        return ir.to_string();
    }

    let result = ir.to_string();

    if level >= OptLevel::O1 {
        // Group 1: Per-function basic optimizations in parallel
        let result = parallel_optimize_functions(&result, |fn_ir| {
            let mut r = constant_folding(fn_ir);
            r = dead_store_elimination(&r);
            r = branch_optimization(&r);
            r = conditional_branch_simplification(&r);
            r
        });

        if level >= OptLevel::O2 {
            // Group 2: Strength reduction per function in parallel
            let result = parallel_optimize_functions(&result, strength_reduction);

            if level >= OptLevel::O3 {
                // Inlining must be done globally (cross-function)
                let result = aggressive_inline(&result);

                // Group 3: Post-inline cleanup per function in parallel
                let result = parallel_optimize_functions(&result, |fn_ir| {
                    let mut r = common_subexpression_elimination(fn_ir);
                    r = dead_code_elimination(&r);
                    r = loop_invariant_motion(&r);
                    r
                });

                return result;
            }

            // O2: CSE + DCE per function in parallel
            let result = parallel_optimize_functions(&result, |fn_ir| {
                let mut r = common_subexpression_elimination(fn_ir);
                r = dead_code_elimination(&r);
                r
            });

            return result;
        }

        return result;
    }

    result
}

/// Statistics from parallel compilation
#[derive(Debug, Clone, Default)]
pub struct ParallelStats {
    /// Number of modules parsed in parallel
    pub modules_parsed: usize,
    /// Number of functions optimized in parallel
    pub functions_optimized: usize,
    /// Thread count used
    pub thread_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert_eq!(config.num_threads, 0);
        assert!(config.parallel_parse);
        assert!(config.parallel_optimize);
    }

    #[test]
    fn test_parallel_config_custom() {
        let config = ParallelConfig::new(4);
        assert_eq!(config.num_threads, 4);
        assert!(config.effective_threads() > 0);
    }

    #[test]
    fn test_split_ir_into_functions() {
        let ir = r#"; ModuleID = 'test'
source_filename = "<vais>"

@.str0 = private unnamed_addr constant [6 x i8] c"hello\00"

define i64 @add(i64 %a, i64 %b) {
entry:
  %result = add i64 %a, %b
  ret i64 %result
}

define i64 @main() {
entry:
  %x = call i64 @add(i64 1, i64 2)
  ret i64 %x
}
"#;
        let (preamble, functions) = split_ir_into_functions(ir);
        assert!(preamble.contains("ModuleID"));
        assert!(preamble.contains("@.str0"));
        assert_eq!(functions.len(), 2);
        assert_eq!(functions[0].0, "add");
        assert_eq!(functions[1].0, "main");
        assert!(functions[0].1.contains("add i64 %a, %b"));
        assert!(functions[1].1.contains("call i64 @add"));
    }

    #[test]
    fn test_extract_function_name() {
        assert_eq!(
            extract_function_name("define i64 @add(i64 %a, i64 %b) {"),
            Some("add".to_string())
        );
        assert_eq!(
            extract_function_name("define void @main() {"),
            Some("main".to_string())
        );
        assert_eq!(
            extract_function_name("define i64 @Vec_i64_push(%Vec_i64* %self, i64 %val) {"),
            Some("Vec_i64_push".to_string())
        );
    }

    #[test]
    fn test_parallel_optimize_functions_identity() {
        let ir = r#"define i64 @add(i64 %a, i64 %b) {
entry:
  %result = add i64 %a, %b
  ret i64 %result
}

define i64 @sub(i64 %a, i64 %b) {
entry:
  %result = sub i64 %a, %b
  ret i64 %result
}
"#;
        let result = parallel_optimize_functions(ir, |fn_ir| fn_ir.to_string());
        assert!(result.contains("@add"));
        assert!(result.contains("@sub"));
    }

    #[test]
    fn test_parallel_stats_default() {
        let stats = ParallelStats::default();
        assert_eq!(stats.modules_parsed, 0);
        assert_eq!(stats.functions_optimized, 0);
        assert_eq!(stats.thread_count, 0);
    }

    // ========== ParallelConfig ==========

    #[test]
    fn test_parallel_config_effective_threads_auto() {
        let config = ParallelConfig::default();
        // Auto mode should return something > 0
        assert!(config.effective_threads() > 0);
    }

    #[test]
    fn test_parallel_config_effective_threads_specified() {
        let config = ParallelConfig::new(8);
        assert_eq!(config.effective_threads(), 8);
    }

    #[test]
    fn test_parallel_config_clone() {
        let config = ParallelConfig::new(4);
        let cloned = config.clone();
        assert_eq!(cloned.num_threads, 4);
        assert_eq!(cloned.parallel_parse, true);
        assert_eq!(cloned.parallel_optimize, true);
    }

    // ========== split_ir edge cases ==========

    #[test]
    fn test_split_ir_empty() {
        let (preamble, functions) = split_ir_into_functions("");
        assert!(preamble.is_empty() || preamble.trim().is_empty());
        assert!(functions.is_empty());
    }

    #[test]
    fn test_split_ir_preamble_only() {
        let ir = "; ModuleID = 'test'\nsource_filename = \"<vais>\"\n";
        let (preamble, functions) = split_ir_into_functions(ir);
        assert!(preamble.contains("ModuleID"));
        assert!(functions.is_empty());
    }

    #[test]
    fn test_split_ir_single_function() {
        let ir = "define i64 @foo() {\nentry:\n  ret i64 42\n}\n";
        let (preamble, functions) = split_ir_into_functions(ir);
        assert!(preamble.is_empty() || preamble.trim().is_empty());
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].0, "foo");
    }

    #[test]
    fn test_split_ir_nested_braces() {
        let ir = r#"define i64 @test() {
entry:
  br i1 true, label %then, label %else
then:
  ret i64 1
else:
  ret i64 0
}
"#;
        let (_, functions) = split_ir_into_functions(ir);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].0, "test");
        assert!(functions[0].1.contains("then:"));
        assert!(functions[0].1.contains("else:"));
    }

    #[test]
    fn test_split_ir_with_declares() {
        let ir = r#"declare i64 @printf(i8*, ...)

define i64 @main() {
entry:
  ret i64 0
}
"#;
        let (preamble, functions) = split_ir_into_functions(ir);
        assert!(preamble.contains("declare"));
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].0, "main");
    }

    #[test]
    fn test_split_ir_preserves_function_content() {
        let ir = r#"define i64 @add(i64 %a, i64 %b) {
entry:
  %result = add i64 %a, %b
  ret i64 %result
}
"#;
        let (_, functions) = split_ir_into_functions(ir);
        assert_eq!(functions.len(), 1);
        assert!(functions[0].1.contains("add i64 %a, %b"));
        assert!(functions[0].1.contains("ret i64 %result"));
    }

    // ========== extract_function_name edge cases ==========

    #[test]
    fn test_extract_function_name_void_return() {
        assert_eq!(
            extract_function_name("define void @init() {"),
            Some("init".to_string())
        );
    }

    #[test]
    fn test_extract_function_name_complex_return() {
        assert_eq!(
            extract_function_name("define { i64, i64 } @pair() {"),
            Some("pair".to_string())
        );
    }

    #[test]
    fn test_extract_function_name_no_at_sign() {
        assert_eq!(extract_function_name("not a function definition"), None);
    }

    #[test]
    fn test_extract_function_name_no_parens() {
        assert_eq!(extract_function_name("define i64 @no_parens"), None);
    }

    // ========== parallel_optimize_functions ==========

    #[test]
    fn test_parallel_optimize_single_function_uses_pass_directly() {
        let ir = "define i64 @only_one() {\nentry:\n  ret i64 0\n}\n";
        let called = std::sync::atomic::AtomicBool::new(false);
        let result = parallel_optimize_functions(ir, |fn_ir| {
            called.store(true, std::sync::atomic::Ordering::SeqCst);
            fn_ir.to_string()
        });
        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
        assert!(result.contains("@only_one"));
    }

    #[test]
    fn test_parallel_optimize_transforms_functions() {
        let ir = r#"define i64 @a() {
entry:
  ret i64 0
}

define i64 @b() {
entry:
  ret i64 0
}
"#;
        let result =
            parallel_optimize_functions(ir, |fn_ir| fn_ir.replace("ret i64 0", "ret i64 42"));
        // Both functions should be transformed
        assert!(result.contains("ret i64 42"));
        assert!(!result.contains("ret i64 0"));
    }

    // ========== ParallelStats ==========

    #[test]
    fn test_parallel_stats_clone() {
        let mut stats = ParallelStats::default();
        stats.modules_parsed = 10;
        stats.functions_optimized = 50;
        stats.thread_count = 4;
        let cloned = stats.clone();
        assert_eq!(cloned.modules_parsed, 10);
        assert_eq!(cloned.functions_optimized, 50);
        assert_eq!(cloned.thread_count, 4);
    }

    #[test]
    fn test_parallel_stats_debug() {
        let stats = ParallelStats::default();
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("ParallelStats"));
    }
}
